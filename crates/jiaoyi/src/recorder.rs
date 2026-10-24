//! Records the live spot quote stream of each permitted account's resolved
//! symbols into `ticks` (partitioned by UTC day), and applies the retention
//! setting by dropping whole expired partitions.
//!
//! One long-lived connection per account. On any error the session is closed,
//! the error kept in the status, and a new session opens after a pause. The
//! settings (`ticks.flush_millis`, `ticks.retention_days`) are read at the
//! start of each session and at each hourly maintenance.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Environment, PermittedAccount};
use jiaoyi_ctrader::Connection;
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::ticks::{self, TickRow};
use serde::Serialize;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;

use crate::config::BrokerConfig;
use crate::loader;

/// Pause before a new session after a failure.
pub const RETRY_AFTER: Duration = Duration::from_secs(30);
/// Partition maintenance and the resolved-symbol check run this often.
pub const MAINTAIN_EVERY: Duration = Duration::from_secs(3600);
/// Partitions created ahead of today.
pub const PARTITIONS_AHEAD: u32 = 3;

#[derive(Debug, Clone, Default, Serialize)]
pub struct RecorderStatus {
    pub subscribed_symbols: Vec<i64>,
    pub session_started_at: Option<DateTime<Utc>>,
    pub last_flush_at: Option<DateTime<Utc>>,
    pub ticks_written: u64,
    pub ticks_missed_lagging: u64,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
    pub partitions_dropped: Vec<String>,
}

pub type StatusMap = Arc<Mutex<HashMap<i64, RecorderStatus>>>;

fn with_status(status: &StatusMap, account: i64, f: impl FnOnce(&mut RecorderStatus)) {
    f(status.lock().unwrap().entry(account).or_default());
}

/// Create upcoming partitions and drop expired ones per `ticks.retention_days`.
pub async fn maintain_partitions(pool: &PgPool) -> anyhow::Result<Vec<String>> {
    let today = Utc::now().date_naive();
    ticks::ensure_partitions(pool, today, PARTITIONS_AHEAD).await?;
    let settings = jiaoyi_store::settings::snapshot(pool, None).await?;
    let keep = settings
        .integer("ticks.retention_days")
        .map_err(anyhow::Error::msg)?;
    let dropped = ticks::drop_expired(pool, today, keep as i32).await?;
    if !dropped.is_empty() {
        tracing::info!(
            ?dropped,
            keep_days = keep,
            "expired tick partitions dropped"
        );
    }
    Ok(dropped)
}

/// Run forever for one account.
pub async fn run(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    env: Environment,
    status: StatusMap,
) {
    loop {
        let result = match loader::connect(&broker, env).await {
            Ok(conn) => record(&pool, &conn, &broker, &account, &status, None).await,
            Err(e) => Err(e.into()),
        };
        if let Err(e) = result {
            tracing::warn!(account = %account, error = %format!("{e:#}"), "tick recording stopped");
            with_status(&status, account.id(), |s| {
                s.last_error = Some(format!("{e:#}"));
                s.last_error_at = Some(Utc::now());
                s.subscribed_symbols.clear();
            });
        }
        tokio::time::sleep(RETRY_AFTER).await;
    }
}

/// One recording session on an open connection. Returns `Ok` when the
/// resolved symbol set changed (the caller reconnects) or `until` passed.
pub async fn record(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
    status: &StatusMap,
    until: Option<Instant>,
) -> anyhow::Result<u64> {
    let dropped = maintain_partitions(pool).await?;
    with_status(status, account.id(), |s| {
        s.partitions_dropped.extend(dropped)
    });
    let ids = ticks::resolved_symbol_ids(pool, account.id()).await?;
    if ids.is_empty() {
        anyhow::bail!("no resolved symbols stored yet for this account; waiting for a data load");
    }
    let settings = jiaoyi_store::settings::snapshot(pool, Some(account.id())).await?;
    let flush_every = Duration::from_millis(
        settings
            .integer("ticks.flush_millis")
            .map_err(anyhow::Error::msg)?
            .max(1) as u64,
    );

    let session = loader::authorise(pool, conn, broker, account).await?;
    let mut rx = conn.spot_events();
    session.subscribe_spots(&ids).await?;
    with_status(status, account.id(), |s| {
        s.subscribed_symbols = ids.clone();
        s.session_started_at = Some(Utc::now());
    });
    tracing::info!(account = %account, symbols = ids.len(), "recording spot quotes");

    let mut buf: Vec<TickRow> = Vec::new();
    let mut written = 0u64;
    let mut flush = tokio::time::interval(flush_every);
    let mut maintain = tokio::time::interval_at(Instant::now() + MAINTAIN_EVERY, MAINTAIN_EVERY);
    let deadline = until.map(tokio::time::sleep_until);
    tokio::pin!(deadline);

    let outcome: anyhow::Result<()> = loop {
        tokio::select! {
            r = rx.recv() => match r {
                Ok(spot) => {
                    let ev = spot.event;
                    if ev.ctid_trader_account_id != account.id() || !ids.contains(&ev.symbol_id) {
                        continue;
                    }
                    let (bid, ask) = (ev.bid.and_then(|v| i64::try_from(v).ok()), ev.ask.and_then(|v| i64::try_from(v).ok()));
                    if bid.is_none() && ask.is_none() {
                        continue;
                    }
                    buf.push(TickRow {
                        account_id: account.id(),
                        broker_symbol_id: ev.symbol_id,
                        received_at: spot.received_at,
                        broker_timestamp: ev.timestamp,
                        bid_e5: bid,
                        ask_e5: ask,
                    });
                }
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(account = %account, missed = n, "tick recorder lagged; quotes missed");
                    with_status(status, account.id(), |s| s.ticks_missed_lagging += n);
                }
                Err(RecvError::Closed) => break Err(anyhow::anyhow!("spot stream closed")),
            },
            _ = flush.tick() => {
                if let Err(e) = write(pool, &mut buf, &mut written, status, account.id()).await {
                    break Err(e);
                }
            }
            _ = maintain.tick() => {
                match maintain_partitions(pool).await {
                    Ok(d) => with_status(status, account.id(), |s| s.partitions_dropped.extend(d)),
                    Err(e) => break Err(e),
                }
                match ticks::resolved_symbol_ids(pool, account.id()).await {
                    Ok(now_ids) if now_ids != ids => break Ok(()),
                    Ok(_) => {}
                    Err(e) => break Err(e.into()),
                }
            }
            _ = async { deadline.as_mut().as_pin_mut().unwrap().await }, if deadline.is_some() => break Ok(()),
        }
    };
    let last = write(pool, &mut buf, &mut written, status, account.id()).await;
    outcome.and(last)?;
    Ok(written)
}

async fn write(
    pool: &PgPool,
    buf: &mut Vec<TickRow>,
    written: &mut u64,
    status: &StatusMap,
    account_id: i64,
) -> anyhow::Result<()> {
    if buf.is_empty() {
        return Ok(());
    }
    let n = match ticks::insert(pool, buf).await {
        Ok(n) => n,
        Err(_) => {
            // A day may have rolled over before maintenance ran: make sure the
            // partitions exist, then try once more.
            ticks::ensure_partitions(pool, Utc::now().date_naive(), PARTITIONS_AHEAD).await?;
            ticks::insert(pool, buf).await?
        }
    };
    buf.clear();
    *written += n;
    with_status(status, account_id, |s| {
        s.ticks_written += n;
        s.last_flush_at = Some(Utc::now());
    });
    Ok(())
}
