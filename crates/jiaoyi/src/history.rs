//! Edge verification, data only: download the broker's own history for each
//! permitted account (owner-approved plan, 24-10-2026).
//!
//! Each sync, per account, in this order:
//! 1. the trader record (registration date: Spotware's floor for history requests);
//! 2. deal history (`ProtoOADealListReq`) from registration to now, in
//!    `deals.window_days` windows, stored exactly as the broker reports it;
//! 3. bid and ask ticks (`ProtoOAGetTickDataReq`) for every resolved symbol,
//!    newest week first, `history.tick_days_back` days deep;
//! 4. an availability probe: how far back the broker's tick history goes,
//!    searched over `history.probe_years`, without downloading it.
//!
//! Windows sit on a fixed grid, so a sync resumes where the last one stopped
//! and a window already fetched completely is not fetched again. Every window
//! is recorded, whether it held ticks or not, and whether it failed.
//! Nothing here can trade: only read requests are on the payload allowlist.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Environment, PermittedAccount};
use jiaoyi_ctrader::messages::{ProtoOaDeal, ProtoOaDealStatus, ProtoOaTradeSide};
use jiaoyi_ctrader::tickdata::QuoteSide;
use jiaoyi_ctrader::{AccountSession, ClientError, Connection};
use jiaoyi_store::history::{self as hist, DealFetchRecord, DealRow, FetchRecord, HistoryTick};
use jiaoyi_store::sqlx::PgPool;
use prost::Message;
use serde::Serialize;

use crate::config::BrokerConfig;
use crate::loader;
use crate::shutdown::Shutdown;

/// One tick window: Spotware's maximum span for a tick request.
pub const WEEK_MS: i64 = 604_800_000;
/// Windows start on Mondays 00:00 UTC (1970-01-05 was a Monday).
pub const GRID_ORIGIN_MS: i64 = 4 * 86_400_000;
/// A symbol's availability is probed again after this many days.
pub const PROBE_EVERY_DAYS: i64 = 7;
/// Pause before retrying after a failed sync.
pub const RETRY_AFTER: Duration = Duration::from_secs(300);

/// The grid windows `[start, end]` (inclusive, milliseconds) that intersect
/// `[from_ms, to_ms]`, newest first. `width` is the window length.
pub fn grid(from_ms: i64, to_ms: i64, origin: i64, width: i64) -> Vec<(i64, i64)> {
    if to_ms < from_ms || width <= 0 {
        return vec![];
    }
    let first = (from_ms - origin).div_euclid(width);
    let last = (to_ms - origin).div_euclid(width);
    (first..=last)
        .rev()
        .map(|k| (origin + k * width, origin + (k + 1) * width - 1))
        .collect()
}

fn ms(t: DateTime<Utc>) -> i64 {
    t.timestamp_millis()
}

fn at(ms: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_millis(ms).unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct HistoryStatus {
    pub last_sync_started_at: Option<DateTime<Utc>>,
    pub last_sync_finished_at: Option<DateTime<Utc>>,
    pub settings_version: Option<i64>,
    pub deal_windows_fetched: u64,
    pub deals_inserted: u64,
    pub tick_windows_fetched: u64,
    pub tick_windows_failed: u64,
    pub ticks_stored: u64,
    pub probes: u64,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
}

pub type HistoryStatusMap = Arc<Mutex<HashMap<i64, HistoryStatus>>>;

fn with_status(m: &HistoryStatusMap, account: i64, f: impl FnOnce(&mut HistoryStatus)) {
    f(m.lock().unwrap().entry(account).or_default());
}

struct SyncSettings {
    version: i64,
    tick_days_back: i64,
    probe_years: i64,
    deal_window_days: i64,
    names: Vec<String>,
}

async fn sync_settings(pool: &PgPool, account_id: i64) -> anyhow::Result<SyncSettings> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let get = |k: &str| s.integer(k).map_err(anyhow::Error::msg);
    Ok(SyncSettings {
        version: s.version,
        tick_days_back: get("history.tick_days_back")?,
        probe_years: get("history.probe_years")?,
        deal_window_days: get("deals.window_days")?,
        names: s
            .universe("universe.symbols")
            .map_err(anyhow::Error::msg)?
            .into_iter()
            .map(|(n, _)| n)
            .collect(),
    })
}

fn side_name(s: QuoteSide) -> &'static str {
    s.as_str()
}

pub fn deal_row(d: &ProtoOaDeal) -> DealRow {
    let c = d.close_position_detail.as_ref();
    let side = ProtoOaTradeSide::try_from(d.trade_side)
        .map(|s| s.as_str_name().to_string())
        .unwrap_or_else(|_| format!("UNKNOWN({})", d.trade_side));
    let status = ProtoOaDealStatus::try_from(d.deal_status)
        .map(|s| s.as_str_name().to_string())
        .unwrap_or_else(|_| format!("UNKNOWN({})", d.deal_status));
    DealRow {
        deal_id: d.deal_id,
        order_id: d.order_id,
        position_id: d.position_id,
        broker_symbol_id: d.symbol_id,
        volume: d.volume,
        filled_volume: d.filled_volume,
        trade_side: side,
        deal_status: status,
        created_at: at(d.create_timestamp),
        executed_at: at(d.execution_timestamp),
        last_update_at: d.utc_last_update_timestamp.map(at),
        execution_price: d.execution_price,
        margin_rate: d.margin_rate,
        commission: d.commission,
        money_digits: d.money_digits.map(|v| v as i32),
        base_to_usd_rate: d.base_to_usd_conversion_rate,
        is_closing: c.is_some(),
        close_entry_price: c.map(|c| c.entry_price),
        close_gross_profit: c.map(|c| c.gross_profit),
        close_swap: c.map(|c| c.swap),
        close_commission: c.map(|c| c.commission),
        close_balance: c.map(|c| c.balance),
        close_quote_to_deposit_rate: c.and_then(|c| c.quote_to_deposit_conversion_rate),
        close_closed_volume: c.and_then(|c| c.closed_volume),
        close_balance_version: c.and_then(|c| c.balance_version),
        close_money_digits: c.and_then(|c| c.money_digits.map(|v| v as i32)),
        close_pnl_conversion_fee: c.and_then(|c| c.pnl_conversion_fee),
        raw: d.encode_to_vec(),
    }
}

/// Is this error the end of the session (stop the sync) rather than a
/// problem with one window (record it and go on)?
fn session_fatal(e: &ClientError) -> bool {
    matches!(
        e,
        ClientError::Disconnected(_) | ClientError::Frame(_) | ClientError::Timeout(_)
    ) || e.is_token_rejected()
}

/// One sync of one account on an authorised session.
pub async fn sync_account(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account: &PermittedAccount,
    status: &HistoryStatusMap,
    shutdown: &Shutdown,
) -> anyhow::Result<()> {
    let id = account.id();
    let cfg = sync_settings(pool, id).await?;
    let now = Utc::now();
    with_status(status, id, |s| {
        s.last_sync_started_at = Some(now);
        s.settings_version = Some(cfg.version);
    });

    // 1. The trader record.
    let trader = session.trader().await?;
    let registration = trader.registration_timestamp.map(at);
    hist::record_account_snapshot(
        pool,
        &hist::AccountSnapshot {
            account_id: id,
            trader_login: trader.trader_login,
            deposit_asset_id: trader.deposit_asset_id,
            balance: trader.balance,
            money_digits: trader.money_digits.map(|v| v as i32),
            registration_at: registration,
            raw: trader.encode_to_vec(),
        },
    )
    .await?;

    // 2. Deals, from registration (or as far as the probe looks) to now.
    let deal_from = registration
        .map(ms)
        .unwrap_or_else(|| ms(now) - cfg.probe_years.max(1) * 366 * 86_400_000);
    let width = cfg.deal_window_days.max(1) * 86_400_000;
    let done = hist::complete_deal_windows(pool, id).await?;
    for (from, to) in grid(deal_from, ms(now), 0, width) {
        if shutdown.is_set() {
            return Ok(());
        }
        if done.contains(&at(from)) {
            continue;
        }
        let to_req = to.min(ms(now));
        let complete = to < ms(now);
        let (deals, error) = match session.deal_history(from, to_req).await {
            Ok(d) => (d, None),
            Err(e) if session_fatal(&e) => return Err(e.into()),
            Err(e) => (vec![], Some(e.to_string())),
        };
        let rows: Vec<DealRow> = deals.iter().map(deal_row).collect();
        let upsert = hist::upsert_deals(pool, id, &rows, Utc::now()).await?;
        hist::record_deal_fetch(
            pool,
            &DealFetchRecord {
                account_id: id,
                window_from: at(from),
                window_to: at(to_req),
                complete,
                deals: rows.len() as i32,
                upsert,
                error,
            },
        )
        .await?;
        with_status(status, id, |s| {
            s.deal_windows_fetched += 1;
            s.deals_inserted += upsert.inserted as u64;
        });
    }

    // 3. Ticks for every resolved symbol, newest week first.
    let symbols = jiaoyi_store::ticks::resolved_symbol_ids(pool, id, &cfg.names).await?;
    let tick_from = ms(now) - cfg.tick_days_back.max(1) * 86_400_000;
    for &symbol in &symbols {
        for side in [QuoteSide::Bid, QuoteSide::Ask] {
            let done =
                hist::complete_windows(pool, id, symbol, side_name(side), "download").await?;
            for (from, to) in grid(tick_from, ms(now), GRID_ORIGIN_MS, WEEK_MS) {
                if shutdown.is_set() {
                    return Ok(());
                }
                if done.contains(&at(from)) {
                    continue;
                }
                download_window(
                    pool,
                    session,
                    id,
                    symbol,
                    side,
                    from,
                    to.min(ms(now)),
                    to < ms(now),
                    status,
                )
                .await?;
            }
        }
    }

    // 4. Probe how far back each symbol's ticks go.
    if cfg.probe_years > 0 {
        for &symbol in &symbols {
            if shutdown.is_set() {
                return Ok(());
            }
            let recent = hist::last_probe_at(pool, id, symbol)
                .await?
                .is_some_and(|t| now - t < chrono::Duration::days(PROBE_EVERY_DAYS));
            if !recent {
                probe(pool, session, id, symbol, now, cfg.probe_years, status).await?;
            }
        }
    }

    with_status(status, id, |s| s.last_sync_finished_at = Some(Utc::now()));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn download_window(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol: i64,
    side: QuoteSide,
    from: i64,
    to: i64,
    complete: bool,
    status: &HistoryStatusMap,
) -> anyhow::Result<()> {
    let mut rec = FetchRecord {
        account_id,
        broker_symbol_id: symbol,
        side: side_name(side),
        kind: "download",
        window_from: at(from),
        window_to: at(to),
        complete,
        ticks: 0,
        pages: 0,
        first_tick: None,
        last_tick: None,
        delta_signs: vec![],
        error: None,
    };
    let ticks: Vec<HistoryTick> = match session.tick_history(symbol, side, from, to).await {
        Ok(f) => {
            rec.ticks = f.ticks.len() as i32;
            rec.pages = f.pages as i32;
            rec.delta_signs = f.signs.iter().map(|s| s.as_str().to_string()).collect();
            rec.delta_signs.dedup();
            let rows: Vec<HistoryTick> = f
                .ticks
                .iter()
                .map(|(t, dup)| HistoryTick {
                    ts: at(t.ts_ms),
                    price_e5: t.price_e5,
                    dup: *dup,
                })
                .collect();
            rec.first_tick = rows.first().map(|t| t.ts);
            rec.last_tick = rows.last().map(|t| t.ts);
            rows
        }
        Err(e) if session_fatal(&e) => return Err(e.into()),
        Err(e) => {
            rec.error = Some(e.to_string());
            vec![]
        }
    };
    let failed = rec.error.is_some();
    let (_, stored) = hist::record_window(pool, &rec, &ticks).await?;
    with_status(status, account_id, |s| {
        if failed {
            s.tick_windows_failed += 1;
        } else {
            s.tick_windows_fetched += 1;
        }
        s.ticks_stored += stored as u64;
    });
    Ok(())
}

/// Find the earliest week with bid ticks by binary search over the grid, from
/// `probe_years` ago to the last complete week. Assumes the broker's history is
/// continuous from its start (recorded as such in the inventory). Every week
/// asked is recorded as a probe, with how many ticks its first page held.
async fn probe(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol: i64,
    now: DateTime<Utc>,
    years: i64,
    status: &HistoryStatusMap,
) -> anyhow::Result<()> {
    let windows = grid(
        ms(now) - years * 366 * 86_400_000,
        ms(now) - WEEK_MS,
        GRID_ORIGIN_MS,
        WEEK_MS,
    );
    // Oldest first.
    let windows: Vec<(i64, i64)> = windows.into_iter().rev().collect();
    if windows.is_empty() {
        return Ok(());
    }
    let (mut lo, mut hi) = (0usize, windows.len() - 1);
    // Does the newest complete week have data at all?
    if !probe_one(pool, session, account_id, symbol, windows[hi], status).await? {
        return Ok(());
    }
    while lo < hi {
        let mid = (lo + hi) / 2;
        if probe_one(pool, session, account_id, symbol, windows[mid], status).await? {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Ok(())
}

async fn probe_one(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol: i64,
    (from, to): (i64, i64),
    status: &HistoryStatusMap,
) -> anyhow::Result<bool> {
    let mut rec = FetchRecord {
        account_id,
        broker_symbol_id: symbol,
        side: "bid",
        kind: "probe",
        window_from: at(from),
        window_to: at(to),
        complete: true,
        ticks: 0,
        pages: 1,
        first_tick: None,
        last_tick: None,
        delta_signs: vec![],
        error: None,
    };
    let has = match session.tick_page(symbol, QuoteSide::Bid, from, to).await {
        Ok((ticks, _)) => {
            rec.ticks = ticks.len() as i32;
            rec.first_tick = ticks.last().map(|t| at(t.ts_ms));
            rec.last_tick = ticks.first().map(|t| at(t.ts_ms));
            !ticks.is_empty()
        }
        Err(e) if session_fatal(&e) => return Err(e.into()),
        Err(e) => {
            rec.error = Some(e.to_string());
            false
        }
    };
    hist::record_window(pool, &rec, &[]).await?;
    with_status(status, account_id, |s| s.probes += 1);
    Ok(has)
}

/// Run until shutdown: connect, authorise, sync, wait `history.interval_hours`.
pub async fn run(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    env: Environment,
    status: HistoryStatusMap,
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
        shutdown,
    )
    .await
}

pub async fn run_with<C, F>(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    status: HistoryStatusMap,
    connect: C,
    mut shutdown: Shutdown,
) where
    C: Fn() -> F,
    F: std::future::Future<Output = anyhow::Result<Connection>>,
{
    loop {
        if shutdown.is_set() {
            return;
        }
        // The sync stops between windows on shutdown; a window being written is
        // one transaction, so cutting the sync short loses nothing recorded.
        let watcher = shutdown.clone();
        let result = async {
            let conn = connect().await?;
            let session = loader::authorise(&pool, &conn, &broker, &account).await?;
            sync_account(&pool, &session, &account, &status, &watcher).await
        };
        let outcome = tokio::select! {
            r = result => r,
            _ = shutdown.wait() => return,
        };
        let wait = match outcome {
            Ok(()) => {
                let hours = jiaoyi_store::settings::snapshot(&pool, Some(account.id()))
                    .await
                    .ok()
                    .and_then(|s| s.integer("history.interval_hours").ok())
                    .unwrap_or(1);
                Duration::from_secs(hours.max(1) as u64 * 3600)
            }
            Err(e) => {
                tracing::warn!(account = %account, error = %format!("{e:#}"), "history sync failed; retrying");
                with_status(&status, account.id(), |s| {
                    s.last_error = Some(format!("{e:#}"));
                    s.last_error_at = Some(Utc::now());
                });
                RETRY_AFTER
            }
        };
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = shutdown.wait() => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_windows_are_aligned_newest_first_and_cover_the_range() {
        // Origin 0, width 10: [3, 27] touches windows 0..9, 10..19, 20..29.
        assert_eq!(grid(3, 27, 0, 10), vec![(20, 29), (10, 19), (0, 9)]);
        assert_eq!(grid(10, 19, 0, 10), vec![(10, 19)]);
        assert!(grid(5, 4, 0, 10).is_empty());
        // Monday alignment: 1970-01-05 00:00 UTC starts a window.
        let w = grid(GRID_ORIGIN_MS, GRID_ORIGIN_MS + 1, GRID_ORIGIN_MS, WEEK_MS);
        assert_eq!(w, vec![(GRID_ORIGIN_MS, GRID_ORIGIN_MS + WEEK_MS - 1)]);
        assert_eq!(at(w[0].0).format("%a %H:%M").to_string(), "Mon 00:00");
        // Before the origin too.
        assert_eq!(grid(-5, -1, 0, 10), vec![(-10, -1)]);
    }
}
