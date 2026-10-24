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
use crate::shutdown::Shutdown;

/// Pause before a new session after a failure or a disconnect.
pub const RETRY_AFTER: Duration = Duration::from_secs(30);
/// Partition maintenance, the settings re-read and the resolved-symbol check
/// run this often.
pub const MAINTAIN_EVERY: Duration = Duration::from_secs(3600);
/// Partitions created ahead of today.
pub const PARTITIONS_AHEAD: u32 = 3;

#[derive(Debug, Clone, Default, Serialize)]
pub struct RecorderStatus {
    pub subscribed_symbols: Vec<i64>,
    pub sessions_started: u64,
    pub session_started_at: Option<DateTime<Utc>>,
    /// The `ticks.flush_millis` in force for the current session.
    pub flush_millis: Option<u64>,
    pub last_flush_at: Option<DateTime<Utc>>,
    pub ticks_written: u64,
    pub ticks_missed_lagging: u64,
    /// Sessions that ended because the broker connection closed.
    pub disconnects: u64,
    /// Sessions the broker ended on an open socket (token invalidated,
    /// account disconnected), and the latest reason.
    pub account_session_ends: u64,
    pub last_session_end: Option<String>,
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

/// The settings a recording session reads, at its start and at each maintenance.
struct SessionSettings {
    names: Vec<String>,
    flush_every: Duration,
}

async fn session_settings(pool: &PgPool, account_id: i64) -> anyhow::Result<SessionSettings> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let names = s
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let flush = s
        .integer("ticks.flush_millis")
        .map_err(anyhow::Error::msg)?;
    Ok(SessionSettings {
        names,
        flush_every: Duration::from_millis(flush.max(1) as u64),
    })
}

/// How a recording session runs. Production uses the defaults plus the
/// service's shutdown receiver.
#[derive(Debug, Clone)]
pub struct RecordOptions {
    /// End the session at this instant (tests); `None` runs until an error
    /// or a change of the resolved symbol set.
    pub until: Option<Instant>,
    pub maintain_every: Duration,
    /// On shutdown the session writes its buffered ticks and ends, and the
    /// recorder does not start another.
    pub shutdown: Option<Shutdown>,
}

impl Default for RecordOptions {
    fn default() -> Self {
        RecordOptions {
            until: None,
            maintain_every: MAINTAIN_EVERY,
            shutdown: None,
        }
    }
}

/// Resolves when shutdown fires; never, without a receiver.
async fn shutdown_fired(s: &mut Option<Shutdown>) {
    match s.as_mut() {
        Some(s) => s.wait().await,
        None => std::future::pending().await,
    }
}

/// Run forever for one account over TLS.
pub async fn run(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    env: Environment,
    status: StatusMap,
    shutdown: Shutdown,
) {
    let b = broker.clone();
    run_with(
        pool,
        broker,
        account,
        status,
        move || {
            let b = b.clone();
            async move { loader::connect(&b, env).await.map_err(anyhow::Error::from) }
        },
        RETRY_AFTER,
        RecordOptions {
            shutdown: Some(shutdown),
            ..Default::default()
        },
    )
    .await
}

/// Run until shutdown: open a connection with `connect`, record until the
/// session ends, record why it ended, wait `retry_after`, and start again.
pub async fn run_with<C, F>(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    status: StatusMap,
    connect: C,
    retry_after: Duration,
    opts: RecordOptions,
) where
    C: Fn() -> F,
    F: std::future::Future<Output = anyhow::Result<Connection>>,
{
    let mut stop = opts.shutdown.clone();
    loop {
        if stop.as_ref().is_some_and(|s| s.is_set()) {
            return;
        }
        let conn = tokio::select! {
            c = connect() => c,
            _ = shutdown_fired(&mut stop) => return,
        };
        let result = match conn {
            Ok(conn) => {
                let r = record(&pool, &conn, &broker, &account, &status, opts.clone()).await;
                if r.is_err() && conn.closed_reason().is_some() {
                    with_status(&status, account.id(), |s| s.disconnects += 1);
                }
                r
            }
            Err(e) => Err(e),
        };
        with_status(&status, account.id(), |s| s.subscribed_symbols.clear());
        if let Err(e) = result {
            tracing::warn!(account = %account, error = %format!("{e:#}"), "tick recording stopped; retrying");
            with_status(&status, account.id(), |s| {
                s.last_error = Some(format!("{e:#}"));
                s.last_error_at = Some(Utc::now());
            });
        }
        if stop.as_ref().is_some_and(|s| s.is_set()) {
            tracing::info!(account = %account, "tick recorder stopped for shutdown");
            return;
        }
        tokio::select! {
            _ = tokio::time::sleep(retry_after) => {}
            _ = shutdown_fired(&mut stop) => return,
        }
    }
}

/// One recording session on an open connection. Returns `Ok` when the
/// resolved symbol set changed (the caller reconnects), `opts.until` passed,
/// or shutdown fired — in every case after writing the buffered ticks.
/// Returns `Err` when the connection or the database fails.
pub async fn record(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
    status: &StatusMap,
    opts: RecordOptions,
) -> anyhow::Result<u64> {
    let mut stop = opts.shutdown.clone();
    if stop.as_ref().is_some_and(|s| s.is_set()) {
        return Ok(0);
    }
    let dropped = maintain_partitions(pool).await?;
    with_status(status, account.id(), |s| {
        s.partitions_dropped.extend(dropped)
    });
    let mut cfg = session_settings(pool, account.id()).await?;
    let ids = ticks::resolved_symbol_ids(pool, account.id(), &cfg.names).await?;
    if ids.is_empty() {
        anyhow::bail!("no resolved symbols stored yet for this account; waiting for a data load");
    }

    let mut ends = conn.session_ends();
    // The token comes from `oauth_tokens`, the one store the loader also uses,
    // so a refresh by either is seen by both.
    let session = loader::authorise(pool, conn, broker, account).await?;
    let mut rx = conn.spot_events();
    session.subscribe_spots(&ids).await?;
    // An end that arrives from here on is seen by `ends` (taken before the
    // authorisation). The quotes received before it are still recorded.
    with_status(status, account.id(), |s| {
        s.subscribed_symbols = ids.clone();
        s.sessions_started += 1;
        s.session_started_at = Some(Utc::now());
        s.flush_millis = Some(cfg.flush_every.as_millis() as u64);
    });
    tracing::info!(account = %account, symbols = ids.len(), "recording spot quotes");

    let mut buf: Vec<TickRow> = Vec::new();
    let mut written = 0u64;
    let session_id = format!(
        "{}-{}-{}",
        account.id(),
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let mut seq: i64 = 0;
    // The first flush comes one interval after the start (a plain `interval`
    // would fire at once).
    let mut flush = tokio::time::interval_at(Instant::now() + cfg.flush_every, cfg.flush_every);
    let mut maintain =
        tokio::time::interval_at(Instant::now() + opts.maintain_every, opts.maintain_every);
    let deadline = opts.until.map(tokio::time::sleep_until);
    tokio::pin!(deadline);

    let outcome: anyhow::Result<()> = loop {
        tokio::select! {
            r = rx.recv() => match r {
                Ok(spot) => {
                    if let Some(row) = tick_row(spot, account.id(), &ids, &session_id, &mut seq) {
                        buf.push(row);
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(account = %account, missed = n, "tick recorder lagged; quotes missed");
                    with_status(status, account.id(), |s| s.ticks_missed_lagging += n);
                }
                Err(RecvError::Closed) => {
                    let why = conn.closed_reason().unwrap_or_else(|| "no reason given".into());
                    break Err(anyhow::anyhow!("broker connection closed: {why}"));
                }
            },
            e = ends.recv() => {
                if let Ok(e) = e {
                    if e.affects(account.id()) {
                        // Keep the quotes that arrived before the end.
                        while let Ok(spot) = rx.try_recv() {
                            if let Some(row) = tick_row(spot, account.id(), &ids, &session_id, &mut seq) {
                                buf.push(row);
                            }
                        }
                        tracing::warn!(account = %account, reason = %e.reason, "account session ended by the broker; re-authorising");
                        with_status(status, account.id(), |s| {
                            s.account_session_ends += 1;
                            s.last_session_end = Some(e.reason.clone());
                        });
                        break Err(anyhow::anyhow!("account session ended by the broker: {}", e.reason));
                    }
                }
            }
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
                match session_settings(pool, account.id()).await {
                    Ok(new) => {
                        if new.flush_every != cfg.flush_every {
                            tracing::info!(account = %account, flush_millis = new.flush_every.as_millis() as u64, "ticks.flush_millis changed");
                            flush = tokio::time::interval_at(Instant::now() + new.flush_every, new.flush_every);
                            with_status(status, account.id(), |s| s.flush_millis = Some(new.flush_every.as_millis() as u64));
                        }
                        cfg = new;
                    }
                    Err(e) => break Err(e),
                }
                match ticks::resolved_symbol_ids(pool, account.id(), &cfg.names).await {
                    Ok(now_ids) if now_ids != ids => break Ok(()),
                    Ok(_) => {}
                    Err(e) => break Err(e.into()),
                }
            }
            _ = shutdown_fired(&mut stop) => {
                tracing::info!(account = %account, buffered = buf.len(), "shutdown: writing buffered ticks");
                break Ok(());
            }
            _ = async { deadline.as_mut().as_pin_mut().unwrap().await }, if deadline.is_some() => break Ok(()),
        }
    };
    let last = write(pool, &mut buf, &mut written, status, account.id()).await;
    outcome.and(last)?;
    Ok(written)
}

/// A spot event as a tick row, numbered in its session. `None` for another
/// account's or symbol's event, or one with neither side.
fn tick_row(
    spot: jiaoyi_ctrader::ReceivedSpot,
    account_id: i64,
    ids: &[i64],
    session_id: &str,
    seq: &mut i64,
) -> Option<TickRow> {
    let ev = spot.event;
    if ev.ctid_trader_account_id != account_id || !ids.contains(&ev.symbol_id) {
        return None;
    }
    let bid = ev.bid.and_then(|v| i64::try_from(v).ok());
    let ask = ev.ask.and_then(|v| i64::try_from(v).ok());
    if bid.is_none() && ask.is_none() {
        return None;
    }
    let row = TickRow {
        account_id,
        session_id: session_id.to_string(),
        seq: *seq,
        broker_symbol_id: ev.symbol_id,
        received_at: spot.received_at,
        broker_timestamp: ev.timestamp,
        bid_e5: bid,
        ask_e5: ask,
    };
    *seq += 1;
    Some(row)
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
