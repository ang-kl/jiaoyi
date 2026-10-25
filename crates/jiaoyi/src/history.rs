//! Edge verification, data only: download the broker's own history for each
//! permitted account (owner-approved plan, 24-10-2026).
//!
//! Each sync, per account, in this order:
//! 1. the trader record (registration date: Spotware's floor for history
//!    requests), stored with the broker's account facts (leverage, account
//!    type) in `trader_reads`;
//! 2. deal history (`ProtoOADealListReq`) from registration to now, in
//!    `deals.window_days` windows, stored exactly as the broker reports it;
//! 3. bid and ask ticks (`ProtoOAGetTickDataReq`) for every resolved symbol,
//!    newest week first, `history.tick_days_back` days deep. Skipped for an
//!    account named only by a `JIAOYI_DEMO_ACCOUNT_ID*` variable (owner,
//!    25-10-2026: "Skip their tick history", Postgres disk); every other step
//!    runs for it as for any account;
//! 4. an availability probe: how far back the broker's tick history goes,
//!    searched over `history.probe_years`, without downloading it.
//!
//! Windows sit on a fixed grid, so a sync resumes where the last one stopped
//! and a window already fetched completely is not fetched again. Every window
//! is recorded, whether it held ticks or not, and whether it failed.
//!
//! When the broker ends the account session (a refresh elsewhere invalidated
//! the token, the account disconnected, the socket closed), the pass stops:
//! the window cut short is recorded as `session_ended`, not as a broker
//! failure of that week, and the sync re-authorises from the shared token
//! store after `history.session_retry_secs` and resumes at the first
//! unfinished window (PR #6; deploy 7765158a, where a demo refresh failed
//! every later live window).
//! Nothing here can trade: only read requests are on the payload allowlist.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Environment, PermittedAccount, TickHistory};
use jiaoyi_ctrader::messages::{ProtoOaDeal, ProtoOaDealStatus, ProtoOaTradeSide, ProtoOaTrader};
use jiaoyi_ctrader::tickdata::QuoteSide;
use jiaoyi_ctrader::{AccountSession, ClientError, Connection};
use jiaoyi_store::history::{
    self as hist, DealFetchRecord, DealRow, FetchRecord, HistoryTick, TraderRead,
};
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
    /// Whether this account downloads tick history (step 3), and if not, why.
    /// Set when the account's sync task starts.
    pub tick_history: Option<TickHistory>,
    pub last_sync_started_at: Option<DateTime<Utc>>,
    pub last_sync_finished_at: Option<DateTime<Utc>>,
    pub settings_version: Option<i64>,
    pub deal_windows_fetched: u64,
    pub deals_inserted: u64,
    pub tick_windows_fetched: u64,
    pub tick_windows_failed: u64,
    pub ticks_stored: u64,
    pub probes: u64,
    /// Passes stopped because the broker ended the account session, the
    /// latest reason, and the windows cut short by it (fetched again later).
    pub session_ends: u64,
    pub last_session_end: Option<String>,
    pub windows_cut_by_session_end: u64,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
    /// The failure last logged; a repeat is not logged again.
    #[serde(skip)]
    pub logged_failure: Option<String>,
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

/// The broker ended the account session under the sync: stop the pass,
/// re-authorise and resume. Not a failure of any week.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("account session ended: {0}")]
pub struct SessionEnded(pub String);

/// Has the session ended (recorded on the connection), before any request?
fn check_session(session: &AccountSession<'_>) -> Result<(), SessionEnded> {
    if let Some(end) = session.session_ended() {
        return Err(SessionEnded(end.reason));
    }
    if let Some(r) = session.closed_reason() {
        return Err(SessionEnded(format!("connection closed: {r}")));
    }
    Ok(())
}

/// Did this request fail because the session ended under it? The connection's
/// own record of a session end is believed first; otherwise the error says so
/// (the socket closed, the account is no longer authorised, the token was
/// refused mid-sync).
fn ended_by(session: &AccountSession<'_>, e: &ClientError) -> Option<SessionEnded> {
    if let Err(end) = check_session(session) {
        return Some(end);
    }
    let ended = match e {
        ClientError::Disconnected(_) | ClientError::Frame(_) => true,
        ClientError::Broker { code, .. } => {
            code == "ACCOUNT_NOT_AUTHORIZED"
                || code == "CH_CLIENT_NOT_AUTHENTICATED"
                || e.is_token_rejected()
        }
        _ => false,
    };
    ended.then(|| SessionEnded(e.to_string()))
}

/// Is this error the end of the sync (stop, retry later) rather than a
/// problem with one window (record it and go on)? A session end is handled
/// before this; a timeout also stops the sync.
fn session_fatal(e: &ClientError) -> bool {
    matches!(
        e,
        ClientError::Disconnected(_) | ClientError::Frame(_) | ClientError::Timeout(_)
    ) || e.is_token_rejected()
}

/// The broker's trader record as stored in `trader_reads`: every field the
/// broker omitted stays `None`.
pub fn trader_read(account_id: i64, t: &ProtoOaTrader) -> TraderRead {
    TraderRead {
        account_id,
        leverage_in_cents: t.leverage_in_cents.map(i64::from),
        max_leverage: t.max_leverage.map(i64::from),
        balance: t.balance,
        balance_version: t.balance_version,
        money_digits: t.money_digits.map(|v| v as i32),
        deposit_asset_id: t.deposit_asset_id,
        account_type: t.account_type,
        is_limited_risk: t.is_limited_risk,
        limited_risk_margin_calculation_strategy: t.limited_risk_margin_calculation_strategy,
        total_margin_calculation_type: t.total_margin_calculation_type,
        access_rights: t.access_rights,
        swap_free: t.swap_free,
        fair_stop_out: t.fair_stop_out,
        stop_out_strategy: t.stop_out_strategy,
        manager_bonus: t.manager_bonus,
        ib_bonus: t.ib_bonus,
        non_withdrawable_bonus: t.non_withdrawable_bonus,
        trader_login: t.trader_login,
        broker_name: t.broker_name.clone(),
        registration_at: t.registration_timestamp.map(at),
    }
}

/// One sync of one account on an authorised session. `ticks` says whether
/// step 3 (the tick download) runs; nothing else depends on it.
pub async fn sync_account(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account: &PermittedAccount,
    ticks: &TickHistory,
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
    check_session(session)?;
    let trader = match session.trader().await {
        Ok(t) => t,
        Err(e) => match ended_by(session, &e) {
            Some(end) => return Err(end.into()),
            None => return Err(e.into()),
        },
    };
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
    // Claude · № 16,973 25-Oct (ordered № 16,972·C; the broker's account facts)
    hist::record_trader_read(pool, &trader_read(id, &trader)).await?;

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
        // Skip only a window whose whole range earlier complete fetches cover:
        // a narrower window that shares its start does not count.
        if done.covers(from, to) {
            continue;
        }
        check_session(session)?;
        let to_req = to.min(ms(now));
        let complete = to < ms(now);
        let mut ended = None;
        let (deals, error) = match session.deal_history(from, to_req).await {
            Ok(d) => (d, None),
            Err(e) => match ended_by(session, &e) {
                Some(end) => {
                    let msg = e.to_string();
                    ended = Some(end);
                    (vec![], Some(msg))
                }
                None if session_fatal(&e) => return Err(e.into()),
                None => (vec![], Some(e.to_string())),
            },
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
                session_ended: ended.is_some(),
            },
        )
        .await?;
        if let Some(end) = ended {
            with_status(status, id, |s| s.windows_cut_by_session_end += 1);
            return Err(end.into());
        }
        with_status(status, id, |s| {
            s.deal_windows_fetched += 1;
            s.deals_inserted += upsert.inserted as u64;
        });
    }

    // 3. Ticks for every resolved symbol, newest week first; not for an
    // account whose tick history is off.
    let symbols = jiaoyi_store::ticks::resolved_symbol_ids(pool, id, &cfg.names).await?;
    let tick_from = ms(now) - cfg.tick_days_back.max(1) * 86_400_000;
    // Claude · № 16,973 25-Oct (ordered № 16,972·C; owner: "Skip their tick history")
    if matches!(ticks, TickHistory::On) {
        for &symbol in &symbols {
            for side in [QuoteSide::Bid, QuoteSide::Ask] {
                let done =
                    hist::complete_windows(pool, id, symbol, side_name(side), "download").await?;
                for (from, to) in grid(tick_from, ms(now), GRID_ORIGIN_MS, WEEK_MS) {
                    if shutdown.is_set() {
                        return Ok(());
                    }
                    if done.covers(from, to) {
                        continue;
                    }
                    check_session(session)?;
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
                check_session(session)?;
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
        session_ended: false,
    };
    let mut ended = None;
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
        Err(e) => match ended_by(session, &e) {
            Some(end) => {
                rec.error = Some(e.to_string());
                rec.session_ended = true;
                ended = Some(end);
                vec![]
            }
            None if session_fatal(&e) => return Err(e.into()),
            None => {
                rec.error = Some(e.to_string());
                vec![]
            }
        },
    };
    let failed = rec.error.is_some();
    let (_, stored) = hist::record_window(pool, &rec, &ticks).await?;
    with_status(status, account_id, |s| {
        if ended.is_some() {
            s.windows_cut_by_session_end += 1;
        } else if failed {
            s.tick_windows_failed += 1;
        } else {
            s.tick_windows_fetched += 1;
        }
        s.ticks_stored += stored as u64;
    });
    match ended {
        Some(end) => Err(end.into()),
        None => Ok(()),
    }
}

/// What one probe request found. A failed request is its own outcome, never
/// an empty week (review of #4, round 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeStep {
    Ticks,
    Empty,
    Failed {
        error: String,
        fatal: bool,
    },
    /// The session ended under the step: the search stops and is made again
    /// after the sync re-authorises.
    SessionEnded(SessionEnded),
}

/// Find the earliest week with bid ticks by binary search over the grid, from
/// `probe_years` ago to the last complete week. Assumes the broker's history is
/// continuous from its start (recorded as such in the inventory). Every week
/// asked is recorded in `history_fetches`; the search as a whole is recorded
/// in `history_probes`. If any step fails the search ends as `incomplete`,
/// with the error and no result, and the next sync probes again.
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
    let (Some(first), Some(last)) = (windows.first(), windows.last()) else {
        return Ok(());
    };
    let mut search = hist::ProbeSearch {
        account_id,
        broker_symbol_id: symbol,
        side: "bid",
        started_at: Utc::now(),
        status: "incomplete",
        searched_from: at(first.0),
        searched_to: at(last.1),
        steps: 0,
        earliest_week_with_ticks: None,
        latest_empty_week_before: None,
        error: None,
    };
    let (mut lo, mut hi) = (0usize, windows.len() - 1);
    let mut first_step = true;
    loop {
        let idx = if first_step { hi } else { (lo + hi) / 2 };
        if !first_step && lo >= hi {
            break;
        }
        search.steps += 1;
        match probe_one(pool, session, account_id, symbol, windows[idx], status).await? {
            ProbeStep::Ticks => {
                hi = idx;
            }
            ProbeStep::Empty if first_step => {
                // The newest complete week has no ticks at all.
                search.status = "no_ticks";
                hist::record_probe_search(pool, &search).await?;
                return Ok(());
            }
            ProbeStep::Empty => {
                search.latest_empty_week_before = Some(at(windows[idx].0));
                lo = idx + 1;
            }
            ProbeStep::SessionEnded(end) => {
                search.status = "incomplete";
                search.error = Some(format!(
                    "step {} ({}): {end}; searched again after re-authorising",
                    search.steps,
                    at(windows[idx].0).date_naive()
                ));
                search.earliest_week_with_ticks = None;
                search.latest_empty_week_before = None;
                hist::record_probe_search(pool, &search).await?;
                return Err(end.into());
            }
            ProbeStep::Failed { error, fatal } => {
                search.status = "incomplete";
                search.error = Some(format!(
                    "step {} ({}): {error}",
                    search.steps,
                    at(windows[idx].0).date_naive()
                ));
                search.earliest_week_with_ticks = None;
                search.latest_empty_week_before = None;
                hist::record_probe_search(pool, &search).await?;
                if fatal {
                    anyhow::bail!("probe ended: {error}");
                }
                return Ok(());
            }
        }
        first_step = false;
    }
    search.status = "complete";
    search.earliest_week_with_ticks = Some(at(windows[hi].0));
    hist::record_probe_search(pool, &search).await?;
    Ok(())
}

async fn probe_one(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol: i64,
    (from, to): (i64, i64),
    status: &HistoryStatusMap,
) -> anyhow::Result<ProbeStep> {
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
        session_ended: false,
    };
    let step = match session.tick_page(symbol, QuoteSide::Bid, from, to).await {
        Ok((ticks, _)) => {
            rec.ticks = ticks.len() as i32;
            rec.first_tick = ticks.last().map(|t| at(t.ts_ms));
            rec.last_tick = ticks.first().map(|t| at(t.ts_ms));
            if ticks.is_empty() {
                ProbeStep::Empty
            } else {
                ProbeStep::Ticks
            }
        }
        Err(e) => {
            // The failed request is recorded too, with its error.
            rec.error = Some(e.to_string());
            match ended_by(session, &e) {
                Some(end) => {
                    rec.session_ended = true;
                    ProbeStep::SessionEnded(end)
                }
                None => ProbeStep::Failed {
                    error: e.to_string(),
                    fatal: session_fatal(&e),
                },
            }
        }
    };
    hist::record_window(pool, &rec, &[]).await?;
    with_status(status, account_id, |s| {
        if matches!(step, ProbeStep::SessionEnded(_)) {
            s.windows_cut_by_session_end += 1;
        } else {
            s.probes += 1;
        }
    });
    Ok(step)
}

/// Run until shutdown: connect, authorise, sync, wait `history.interval_hours`.
pub async fn run(
    pool: PgPool,
    broker: BrokerConfig,
    account: PermittedAccount,
    env: Environment,
    ticks: TickHistory,
    status: HistoryStatusMap,
    shutdown: Shutdown,
) {
    let b = broker.clone();
    run_with(
        pool,
        broker,
        account,
        ticks,
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
    ticks: TickHistory,
    status: HistoryStatusMap,
    connect: C,
    mut shutdown: Shutdown,
) where
    C: Fn() -> F,
    F: std::future::Future<Output = anyhow::Result<Connection>>,
{
    with_status(&status, account.id(), |s| {
        s.tick_history = Some(ticks.clone())
    });
    // Was the previous pass cut short by a session end? A resumed pass whose
    // session ends again before it stores one window waits RETRY_AFTER rather
    // than `history.session_retry_secs`, so a session that keeps ending cannot
    // spin.
    let mut resumed = false;
    loop {
        if shutdown.is_set() {
            return;
        }
        // The sync stops between windows on shutdown; a window being written is
        // one transaction, so cutting the sync short loses nothing recorded.
        let watcher = shutdown.clone();
        let done_before = progress(&status, account.id());
        let result = async {
            let conn = connect().await?;
            let session = loader::authorise(&pool, &conn, &broker, &account).await?;
            sync_account(&pool, &session, &account, &ticks, &status, &watcher).await
        };
        let outcome = tokio::select! {
            r = result => r,
            _ = shutdown.wait() => return,
        };
        let settings = jiaoyi_store::settings::snapshot(&pool, Some(account.id()))
            .await
            .ok();
        let setting = |k: &str, default: i64| {
            settings
                .as_ref()
                .and_then(|s| s.integer(k).ok())
                .unwrap_or(default)
        };
        let wait = match outcome {
            Ok(()) => {
                resumed = false;
                Duration::from_secs(setting("history.interval_hours", 1).max(1) as u64 * 3600)
            }
            Err(e) if e.downcast_ref::<SessionEnded>().is_some() => {
                let made = progress(&status, account.id()) - done_before;
                let quick = !resumed || made > 0;
                resumed = true;
                with_status(&status, account.id(), |s| {
                    s.session_ends += 1;
                    s.last_session_end = Some(format!("{e:#}"));
                });
                tracing::warn!(
                    account = %account,
                    reason = %format!("{e:#}"),
                    windows_this_pass = made,
                    "history sync: the account session ended; re-authorising and resuming"
                );
                if quick {
                    Duration::from_secs(setting("history.session_retry_secs", 5).max(0) as u64)
                } else {
                    RETRY_AFTER
                }
            }
            Err(e) => {
                resumed = false;
                let msg = format!("{e:#}");
                let new_state = {
                    let mut m = status.lock().unwrap();
                    let s = m.entry(account.id()).or_default();
                    s.last_error = Some(msg.clone());
                    s.last_error_at = Some(Utc::now());
                    let new = s.logged_failure.as_deref() != Some(msg.as_str());
                    s.logged_failure = Some(msg.clone());
                    new
                };
                if new_state {
                    tracing::warn!(account = %account, error = %msg, "history sync failed; retrying (repeats are not logged)");
                } else {
                    tracing::debug!(account = %account, error = %msg, "history sync still failing");
                }
                if loader::is_grant_refusal(&e) {
                    // The token cannot authorise this account; a grant read
                    // every load.retry_minutes is enough.
                    Duration::from_secs(setting("load.retry_minutes", 5).max(1) as u64 * 60)
                } else {
                    RETRY_AFTER
                }
            }
        };
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = shutdown.wait() => return,
        }
    }
}

/// Windows and probe steps stored so far for an account (any outcome but a
/// session end).
fn progress(status: &HistoryStatusMap, account: i64) -> u64 {
    status.lock().unwrap().get(&account).map_or(0, |s| {
        s.deal_windows_fetched + s.tick_windows_fetched + s.tick_windows_failed + s.probes
    })
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
