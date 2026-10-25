//! Strategy 1: the tick-triggered trend (the owner's naming, № 16,806; H1
//! is its pre-registration label). Owner, 25-10-2026: "I want live to
//! execute trading by 1 PM today"; "Keep the gate; demo trades at 1 PM";
//! "Allow the minimum volume" for live.
//!
//! There is no on/off switch (owner, 24-10-2026: "why do i need
//! JIAOYI_TRADING_ENABLED, removed it. it should not be doing so"). One loop
//! per permitted account runs from start-up:
//! - a **demo** account trades Strategy 1 at once;
//! - a **live** account runs the same loop armed, and sends nothing until
//!   the H1 held-out trial under the running pre-registration hash has
//!   verdict PASS (`trading.live_requires_edge_pass`).
//!
//! The strategy, as pre-registered (default setting): on the broker's own
//! completed daily bars, re-read every `LEVELS_REFRESH`, EMA(50) of the
//! closes above EMA(100) → longs only, below → shorts only; long on the
//! first ask at or above the highest high of the previous 50 completed
//! days, short on the first bid at or below the lowest low; one position
//! per symbol, at most one entry per symbol after each completed day. The
//! stop is 3 × ATR(20) (Wilder, completed days) from the fill, sent as the
//! order's relative stop; it trails 3 × ATR behind the best bid (ask for
//! shorts) since the fill and only ever moves in the trade's favour, at
//! most one amend per `trading.trail_amend_secs`; no take-profit.
//!
//! Size: `trading.risk_pct` of fresh equity (the broker's balance plus its
//! unrealized P&L), floored to the step; below the broker minimum it is
//! skipped. On a live account with `trading.live_allow_min_volume`, the
//! minimum is taken instead when its risk is within
//! `trading.min_volume_max_risk_pct` of equity. Strategy 1 holds at most
//! `trading.trend_open_risk_pct` of equity in open risk at its stops (half
//! of a 6% account limit; the other half is for Strategy 2); an entry that
//! would exceed it is skipped with the reason.
//!
//! The broker wins: positions are reconciled at start and every
//! `trading.reconcile_secs`; positions without this program's label are
//! external, reported (symbol, side, volume) and never touched.
//!
//! Every order goes through `jiaoyi_ctrader::trading::send`, the one gate.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::research::h1::{atr, ema, ATR_LEN, EMA_FAST, EMA_SLOW};
use jiaoyi_ctrader::trading::{
    self as gate, Command, EventRecord, Intent, IntentWrite, Ledger, Outcome, Policy, SendError,
    Side, VolumeLimits,
};
use jiaoyi_ctrader::{AccountSession, Connection, ReceivedExecution};
use jiaoyi_store::sqlx::{PgPool, Row};
use jiaoyi_store::trading as ts;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;

use crate::config::Config;
use crate::edge::h1job::QuoteAssets;
use crate::edge::prereg;
use crate::loader;
use crate::shutdown::Shutdown;

/// The strategy's name on every route and record (owner, № 16,806).
pub const STRATEGY: &str = "Strategy 1: tick-triggered trend";
/// The label on every position this program opens for Strategy 1.
pub const LABEL: &str = "jiaoyi-s1";
/// The pre-registered H1 default (breakout 50, stop 3 × ATR).
pub const BREAKOUT: usize = 50;
pub const STOP_ATR: f64 = 3.0;
/// Calendar days of daily bars read for the levels.
pub const BAR_DAYS: i64 = 250;
/// How often the levels are recomputed from the broker's bars.
pub const LEVELS_REFRESH: Duration = Duration::from_secs(30 * 60);

/// Each account's Strategy 1 loop as it runs, in memory only: when it
/// started, when a reconcile last ran and when it last processed a quote.
/// Decisions are recorded only when their reason changes, so the last
/// decision's time cannot tell a quiet loop from a stalled one; this can.
// Claude · № 17,020 25-Oct (ordered: production read-back 04:26Z, /health
// held at the 04:02:28 reconcile on all four accounts)
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Liveness {
    pub started_at: Option<DateTime<Utc>>,
    pub last_reconcile_run_at: Option<DateTime<Utc>>,
    pub last_quote_at: Option<DateTime<Utc>>,
    /// Each symbol's distance to its entry trigger at its last quote, so "no
    /// trade" can be told from "cannot trade" (owner-approved plan, 25-10).
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
    pub triggers: std::collections::BTreeMap<String, Value>,
}

/// The accounts' loops, shared by the loops and `/health`.
pub type LivenessMap = Arc<Mutex<HashMap<i64, Liveness>>>;

pub fn beat(live: &LivenessMap, account_id: i64, f: impl FnOnce(&mut Liveness)) {
    f(live.lock().unwrap().entry(account_id).or_default());
}

/// A loop is stalled when no reconcile ran within twice its interval (from
/// its start, before the first); `starting` until then.
pub fn liveness_state(
    l: Option<&Liveness>,
    reconcile_secs: u64,
    now: DateTime<Utc>,
) -> &'static str {
    let Some(l) = l else {
        return "not running";
    };
    let limit = chrono::Duration::seconds(2 * reconcile_secs as i64);
    match (l.last_reconcile_run_at, l.started_at) {
        (Some(at), _) if now - at <= limit => "alive",
        (None, Some(start)) if now - start <= limit => "starting",
        _ => "stalled",
    }
}
/// The pre-registered hypothesis whose held-out PASS opens the live gate.
pub const EDGE_HYPOTHESIS: &str = "H1";

// ---------------------------------------------------------------- settings

/// The rules for one account, read fresh: the settings, with what depends
/// on the account's environment (only live waits for the edge gate and may
/// take the broker minimum).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rules {
    pub version: i64,
    pub live: bool,
    pub edge_required: bool,
    /// A live account whose edge gate is lifted (`trading.live_requires_edge_pass`
    /// 0; owner, 25-10-2026: "Live too, from now").
    pub edge_lifted: bool,
    pub risk_pct: f64,
    pub cap_factor: f64,
    pub allow_min_volume: bool,
    pub min_volume_max_risk_pct: f64,
    pub trend_open_risk_pct: f64,
    pub reconcile_secs: u64,
    pub trail_amend_secs: u64,
    pub order_wait_secs: u64,
    pub retry_secs: u64,
    /// The owner's pause of Strategy 1 on this account (`trading.s1_paused`):
    /// no new entry; open positions keep their exits. Only the owner sets it.
    pub paused: bool,
}

pub async fn rules(pool: &PgPool, account_id: i64, env: Environment) -> anyhow::Result<Rules> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let i = |k: &str| s.integer(k).map_err(anyhow::Error::msg);
    let n = |k: &str| s.number(k).map_err(anyhow::Error::msg);
    let live = env == Environment::Live;
    let pass = i("trading.live_requires_edge_pass")?;
    Ok(Rules {
        version: s.version,
        live,
        edge_required: live && pass == 1,
        edge_lifted: live && pass == 0,
        risk_pct: n("trading.risk_pct")?,
        cap_factor: n("trading.cap_factor")?,
        allow_min_volume: live && i("trading.live_allow_min_volume")? == 1,
        min_volume_max_risk_pct: n("trading.min_volume_max_risk_pct")?,
        trend_open_risk_pct: n("trading.trend_open_risk_pct")?,
        reconcile_secs: i("trading.reconcile_secs")?.max(1) as u64,
        trail_amend_secs: i("trading.trail_amend_secs")?.max(1) as u64,
        order_wait_secs: i("trading.order_wait_secs")?.max(1) as u64,
        retry_secs: i("trading.retry_secs")?.max(1) as u64,
        paused: i(crate::limits::S1.pause_key)? == 1,
    })
}

/// A symbol's distance to its Strategy 1 trigger at a quote: the 50-day
/// level on the side the EMA filter allows, and how far the ask (a long) or
/// the bid (a short) is from it in ATR, negative once through it. `None`
/// distance, with the reason, when there are no levels or no quote.
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
pub fn trigger_distance(
    levels: &Result<Levels, String>,
    bid: Option<f64>,
    ask: Option<f64>,
    at: DateTime<Utc>,
) -> Value {
    let l = match levels {
        Ok(l) => l,
        Err(why) => {
            return json!({"at": at, "distance_atr": null, "reason": format!("no levels: {why}")})
        }
    };
    let (price, distance) = match l.side {
        Some(Side::Buy) => (ask, ask.map(|a| (l.level - a) / l.atr)),
        Some(Side::Sell) => (bid, bid.map(|b| (b - l.level) / l.atr)),
        None => (None, None),
    };
    json!({
        "at": at,
        "filter": l.side.map(|s| s.as_str()),
        "level": (l.side.is_some()).then_some(l.level),
        "atr": l.atr,
        "ema_fast": l.ema_fast,
        "ema_slow": l.ema_slow,
        "bid": bid,
        "ask": ask,
        "quote": price,
        "distance_atr": distance.filter(|d| d.is_finite()),
        "reason": match (l.side, price) {
            (None, _) => Some("the EMAs are equal: no side allowed"),
            (_, None) => Some("no quote on the side that triggers"),
            _ => None,
        },
    })
}

/// The edge gate for an account: met when not required (demo), or when the
/// newest H1 held-out trial under the running pre-registration hash says
/// PASS.
pub async fn edge_gate(pool: &PgPool, r: &Rules) -> anyhow::Result<Result<(), String>> {
    if !r.edge_required {
        return Ok(Ok(()));
    }
    let hash = prereg::hash();
    Ok(
        match ts::held_out_verdict(pool, EDGE_HYPOTHESIS, &hash).await? {
            Some(Some(v)) if v == "PASS" => Ok(()),
            Some(Some(v)) => Err(format!(
                "the H1 held-out verdict under {} is {v}, not PASS",
                &hash[..8]
            )),
            Some(None) => Err(format!(
                "the H1 held-out trial under {} has no verdict",
                &hash[..8]
            )),
            None => Err(format!(
                "no H1 held-out trial under {} yet (the development gate runs first)",
                &hash[..8]
            )),
        },
    )
}

/// The open-risk budget: Strategy 1's open risk at its stops plus a new
/// entry's must stay within `trend_open_risk_pct` of equity.
pub fn within_budget(open_risk: f64, new_risk: f64, equity: f64, pct: f64) -> Result<(), String> {
    let budget = equity * pct / 100.0;
    if open_risk + new_risk > budget * (1.0 + 1e-9) {
        Err(format!(
            "Strategy 1's open risk {open_risk:.2} plus this entry's {new_risk:.2} would exceed its budget {budget:.2} ({pct}% of equity): skipped"
        ))
    } else {
        Ok(())
    }
}

/// Whether Strategy 1 may take one more entry (`open`: its positions and
/// unresolved opens on the account; `held`: their risk at the stops).
/// - An entry sized at `risk_pct` must fit the open-risk budget.
/// - A broker-minimum entry (live only) is taken only while Strategy 1 holds
///   nothing else on the account, and then replaces the budget check for
///   itself (owner, 25-10-2026: "One minimum trade at a time"). A second
///   entry waits until it closes.
pub fn admit(
    sized: &Sized,
    open: usize,
    held: Result<f64, String>,
    equity: f64,
    pct: f64,
) -> Result<(), String> {
    if sized.mode == "broker_minimum" {
        if open > 0 {
            return Err(format!(
                "one minimum trade at a time: Strategy 1 holds {open} other position(s) or unresolved open(s) on this account; this broker-minimum entry waits until they close"
            ));
        }
        return Ok(());
    }
    within_budget(held?, sized.risk, equity, pct)
}

/// The risk still at stake at a position's stop, in the deposit currency:
/// the distance from the entry to the stop on the losing side, or 0 once
/// the stop has passed the entry.
pub fn risk_at_stop(side: Side, entry: f64, stop: f64, volume: i64, quote_to_deposit: f64) -> f64 {
    let d = match side {
        Side::Buy => entry - stop,
        Side::Sell => stop - entry,
    };
    d.max(0.0) * volume as f64 / 100.0 * quote_to_deposit
}

// ---------------------------------------------------------------- sizing

/// How the volume was chosen.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sized {
    pub volume: i64,
    /// The most the gate lets this open risk.
    pub max_risk: f64,
    /// "risk_pct" or "broker_minimum".
    pub mode: &'static str,
    pub risk: f64,
}

/// Size an open. `equity` and the result are in the deposit currency;
/// `relative_stop` in 1/100000 of a price.
pub fn size(
    sw: &Rules,
    equity: f64,
    relative_stop: i64,
    quote_to_deposit: f64,
    limits: &VolumeLimits,
) -> Result<Sized, String> {
    if !(equity.is_finite() && equity > 0.0) {
        return Err(format!("equity {equity} is not positive"));
    }
    if !(quote_to_deposit.is_finite() && quote_to_deposit > 0.0) || relative_stop <= 0 {
        return Err("no stop or no conversion rate".into());
    }
    if limits.step <= 0 || limits.min <= 0 {
        return Err("the symbol's volume rules are unknown".into());
    }
    let per_cent = gate::risk_of(1, relative_stop, quote_to_deposit);
    let allowed = equity * sw.risk_pct / 100.0;
    let raw = allowed / per_cent;
    let mut volume = (raw / limits.step as f64).floor() as i64 * limits.step;
    if let Some(max) = limits.max {
        volume = volume.min(max - max % limits.step);
    }
    if volume >= limits.min {
        return Ok(Sized {
            volume,
            max_risk: sw.cap_factor * allowed,
            mode: "risk_pct",
            risk: gate::risk_of(volume, relative_stop, quote_to_deposit),
        });
    }
    let min_risk = gate::risk_of(limits.min, relative_stop, quote_to_deposit);
    if !sw.allow_min_volume {
        return Err(format!(
            "{}% of equity sizes {raw:.2} cents, below the broker minimum {}: skipped, never rounded up (the minimum would risk {min_risk:.2})",
            sw.risk_pct, limits.min
        ));
    }
    let ceiling = equity * sw.min_volume_max_risk_pct / 100.0;
    if min_risk > ceiling {
        return Err(format!(
            "the broker minimum {} would risk {min_risk:.2}, above {}% of equity ({ceiling:.2}): skipped",
            limits.min, sw.min_volume_max_risk_pct
        ));
    }
    Ok(Sized {
        volume: limits.min,
        max_risk: ceiling,
        mode: "broker_minimum",
        risk: min_risk,
    })
}

/// Round a price distance to the symbol's digits, in 1/100000 units; at least one tick.
pub fn relative_stop(distance: f64, digits: i32) -> i64 {
    let tick = 10_i64.pow((5 - digits.clamp(0, 5)) as u32);
    let e5 = (distance * 100_000.0).round() as i64;
    ((e5 + tick / 2) / tick).max(1) * tick
}

/// Round a trailed stop to the digits on the safe side (a long's down, a
/// short's up).
pub fn round_stop(price: f64, digits: i32, side: Side) -> f64 {
    let f = 10f64.powi(digits.clamp(0, 10));
    match side {
        Side::Buy => (price * f).floor() / f,
        Side::Sell => (price * f).ceil() / f,
    }
}

// ---------------------------------------------------------------- levels

/// One symbol's H1 state after its last completed day.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Levels {
    pub last_completed: NaiveDate,
    /// `None`: the EMAs are equal, no entry.
    pub side: Option<Side>,
    /// The breakout level on the allowed side.
    pub level: f64,
    pub atr: f64,
    pub ema_fast: f64,
    pub ema_slow: f64,
}

/// A completed daily bar: (date, high, low, close).
pub type Bar = (NaiveDate, f64, f64, f64);

/// The levels from completed daily bars, oldest first.
pub fn levels(bars: &[Bar]) -> Result<Levels, String> {
    let need = EMA_SLOW.max(BREAKOUT).max(ATR_LEN + 1);
    if bars.len() < need {
        return Err(format!(
            "{} completed daily bars, {need} needed",
            bars.len()
        ));
    }
    let high: Vec<f64> = bars.iter().map(|b| b.1).collect();
    let low: Vec<f64> = bars.iter().map(|b| b.2).collect();
    let close: Vec<f64> = bars.iter().map(|b| b.3).collect();
    let n = bars.len();
    let f = ema(&close, EMA_FAST)[n - 1].ok_or("no EMA(50)")?;
    let w = ema(&close, EMA_SLOW)[n - 1].ok_or("no EMA(100)")?;
    let a = atr(&high, &low, &close, ATR_LEN)[n - 1].ok_or("no ATR(20)")?;
    let window = n - BREAKOUT..n;
    let (side, level) = if f > w {
        (
            Some(Side::Buy),
            high[window].iter().copied().fold(f64::MIN, f64::max),
        )
    } else if f < w {
        (
            Some(Side::Sell),
            low[window].iter().copied().fold(f64::MAX, f64::min),
        )
    } else {
        (None, f64::NAN)
    };
    Ok(Levels {
        last_completed: bars[n - 1].0,
        side,
        level,
        atr: a,
        ema_fast: f,
        ema_slow: w,
    })
}

/// Whether a quote triggers the entry: a long's ask at or above the level, a
/// short's bid at or below it.
pub fn triggered(l: &Levels, bid: Option<f64>, ask: Option<f64>) -> Option<Side> {
    match l.side? {
        Side::Buy => ask.filter(|a| *a >= l.level).map(|_| Side::Buy),
        Side::Sell => bid.filter(|b| *b <= l.level).map(|_| Side::Sell),
    }
}

/// The trailed stop for a best price, or `None` when it would not move the
/// stop in the trade's favour.
pub fn trail(side: Side, best: f64, atr: f64, current: Option<f64>, digits: i32) -> Option<f64> {
    let raw = match side {
        Side::Buy => best - STOP_ATR * atr,
        Side::Sell => best + STOP_ATR * atr,
    };
    let s = round_stop(raw, digits, side);
    match (side, current) {
        (_, None) => Some(s),
        (Side::Buy, Some(c)) => (s > c).then_some(s),
        (Side::Sell, Some(c)) => (s < c).then_some(s),
    }
}

/// The entry key: at most one entry per symbol after each completed day.
pub fn entry_key(account_id: i64, symbol_id: i64, after: NaiveDate) -> String {
    format!("s1:{account_id}:{symbol_id}:{after}")
}

// ---------------------------------------------------------------- ledger

/// The gate's ledger on Postgres.
pub struct PgLedger<'a> {
    pub pool: &'a PgPool,
    pub settings_version: Option<i64>,
}

fn side_str(s: Option<Side>) -> Option<String> {
    s.map(|s| s.as_str().to_string())
}

pub fn event_row(e: &EventRecord) -> ts::EventRow {
    ts::EventRow {
        account_id: e.account_id,
        entry_key: e.entry_key.clone(),
        client_msg_id: e.client_msg_id.clone(),
        payload_type: e.payload_type as i32,
        kind: e.kind.clone(),
        position_id: e.position_id,
        order_id: e.order_id,
        deal_id: e.deal_id,
        symbol_id: e.symbol_id,
        side: side_str(e.side),
        price: e.price,
        volume: e.volume,
        stop_loss: e.stop_loss,
        label: e.label.clone(),
        error_code: e.error_code.clone(),
        description: e.description.clone(),
        is_server_event: e.is_server_event,
        raw: e.raw.clone(),
        received_at: e.received_at,
    }
}

impl Ledger for PgLedger<'_> {
    async fn record_intent(&self, i: &Intent) -> Result<IntentWrite, String> {
        let row = ts::IntentRow {
            account_id: i.account_id,
            entry_key: i.entry_key.clone(),
            client_msg_id: i.client_msg_id.clone(),
            kind: i.kind.to_string(),
            symbol_id: i.symbol_id,
            position_id: i.position_id,
            side: side_str(i.side),
            volume: i.volume,
            relative_stop: i.relative_stop,
            stop_price: i.stop_price,
            take_profit: i.take_profit,
            risk: i.risk,
            max_risk: i.max_risk,
            settings_version: self.settings_version,
        };
        match ts::record_intent(self.pool, &row).await {
            Ok(true) => Ok(IntentWrite::Recorded),
            Ok(false) => Ok(IntentWrite::AlreadyRecorded),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn record_event(&self, e: &EventRecord) -> Result<(), String> {
        ts::record_event(self.pool, &event_row(e))
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    async fn owns_position(&self, account_id: i64, position_id: i64) -> Result<bool, String> {
        ts::owns_position(self.pool, account_id, position_id)
            .await
            .map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------- fills
// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")

/// Strategy 1's gate ledger: `PgLedger`, and when the intent was recorded,
/// just before the frame, so the order-to-answer time is measured from it
/// as Strategy 3's is.
pub struct TimedLedger<'a> {
    pub inner: PgLedger<'a>,
    pub recorded_at: Mutex<Option<DateTime<Utc>>>,
}

impl<'a> TimedLedger<'a> {
    pub fn new(pool: &'a PgPool, settings_version: Option<i64>) -> Self {
        TimedLedger {
            inner: PgLedger {
                pool,
                settings_version,
            },
            recorded_at: Mutex::new(None),
        }
    }

    fn sent_at(&self) -> DateTime<Utc> {
        self.recorded_at.lock().unwrap().unwrap_or_else(Utc::now)
    }
}

impl Ledger for TimedLedger<'_> {
    async fn record_intent(&self, i: &Intent) -> Result<IntentWrite, String> {
        let w = self.inner.record_intent(i).await?;
        if w == IntentWrite::Recorded {
            *self.recorded_at.lock().unwrap() = Some(Utc::now());
        }
        Ok(w)
    }
    async fn record_event(&self, e: &EventRecord) -> Result<(), String> {
        self.inner.record_event(e).await
    }
    async fn owns_position(&self, account_id: i64, position_id: i64) -> Result<bool, String> {
        self.inner.owns_position(account_id, position_id).await
    }
}

/// What a Strategy 1 order was, for its fill record.
struct FillOf<'s> {
    account_id: i64,
    key: &'s str,
    /// `open` or `trail`.
    kind: &'s str,
    symbol: &'s str,
    /// The order's side; an amend's, its position's.
    side: Side,
    position_id: Option<i64>,
    /// The price the order was meant at: an open's quote at the decision
    /// (the ask for a buy, the bid for a sell), an amend's new stop.
    decision_price: Option<f64>,
    bid: Option<f64>,
    ask: Option<f64>,
    volume: i64,
    detail: Value,
}

/// Record a Strategy 1 order's answer in `strategy1_fills`, by the rule
/// Strategy 3's fills follow (`crate::fills`): its outcome, its fill (the
/// deals' sum at their volume-weighted price, into the deals' position), the
/// order-to-answer time and the slippage against the quote at the decision.
/// Recorded once per entry key.
async fn record_fill(
    pool: &PgPool,
    f: FillOf<'_>,
    sent_at: DateTime<Utc>,
    sent: &gate::Sent,
) -> Result<bool, jiaoyi_store::sqlx::Error> {
    let s = crate::fills::summarize(sent, f.position_id, f.volume);
    // An open's latency runs to its fill; an amend's to its answer.
    let answered_at = if f.kind == "open" {
        s.filled_at
    } else {
        s.answered_at
    };
    let mut detail = f.detail;
    detail["outcome"] = serde_json::to_value(&sent.outcome).unwrap_or_default();
    detail["client_msg_id"] = json!(sent.client_msg_id);
    ts::record_s1_fill(
        pool,
        &ts::FillRow {
            account_id: f.account_id,
            entry_key: f.key.to_string(),
            label: LABEL.into(),
            prereg_hash: prereg::hash(),
            kind: f.kind.into(),
            symbol: f.symbol.into(),
            position_id: s.position_id,
            side: Some(f.side.as_str().into()),
            outcome: s.outcome.into(),
            sent_at,
            answered_at,
            latency_ms: answered_at.map(|a| (a - sent_at).num_milliseconds()),
            decision_price: f.decision_price,
            fill_price: s.price,
            slippage: crate::fills::slippage(f.side, s.price, f.decision_price),
            spread: match (f.bid, f.ask) {
                (Some(b), Some(a)) => Some(a - b),
                _ => None,
            },
            volume: s.volume.filter(|_| f.kind == "open"),
            detail,
        },
    )
    .await
}

/// A closing deal the broker executed itself on a Strategy 1 position (the
/// trailed stop, a stop-out, or a close made outside this program), recorded
/// once per deal in `strategy1_fills` as a `broker_exit`, with its fill
/// against the stop it executed and the spread of the last quote. Anything
/// else (an answer to a request, another label, an opening deal) is not one.
async fn broker_exit(
    pool: &PgPool,
    account_id: i64,
    m: &ReceivedExecution,
    book: &Book,
    syms: &HashMap<i64, Sym>,
) -> Result<bool, jiaoyi_store::sqlx::Error> {
    if m.client_msg_id.is_some() {
        return Ok(false);
    }
    let Some(e) = gate::decode_execution(account_id, m) else {
        return Ok(false);
    };
    if e.label.as_deref() != Some(LABEL) {
        return Ok(false);
    }
    let Some(deal) = gate::closing_deal(account_id, m) else {
        return Ok(false);
    };
    let own = book.own.get(&deal.position_id);
    let side = deal
        .position_side
        .or(own.map(|o| o.side))
        .or(deal.deal_side.map(crate::fills::exit_side));
    let Some(side) = side else {
        return Ok(false);
    };
    let stop = deal.stop_loss.or(own.and_then(|o| o.stop));
    let (reason, level) = crate::fills::exit_level(&deal, stop, deal.take_profit);
    let sym = syms.get(&deal.symbol_id);
    let (bid, ask) = (sym.and_then(|s| s.bid), sym.and_then(|s| s.ask));
    let key = format!(
        "s1-broker:{account_id}:{}:{}",
        deal.position_id, deal.deal_id
    );
    ts::record_s1_fill(
        pool,
        &ts::FillRow {
            account_id,
            entry_key: key,
            label: LABEL.into(),
            prereg_hash: prereg::hash(),
            kind: "broker_exit".into(),
            symbol: sym.map(|s| s.name.clone()).unwrap_or_default(),
            position_id: Some(deal.position_id),
            side: Some(side.as_str().into()),
            outcome: "filled".into(),
            sent_at: m.received_at,
            answered_at: Some(m.received_at),
            latency_ms: None,
            decision_price: level,
            fill_price: deal.price,
            slippage: crate::fills::slippage(crate::fills::exit_side(side), deal.price, level),
            spread: match (bid, ask) {
                (Some(b), Some(a)) => Some(a - b),
                _ => None,
            },
            volume: Some(deal.volume),
            detail: json!({"strategy": STRATEGY, "reason": reason, "deal_id": deal.deal_id,
                "order_id": deal.order_id, "level": level}),
        },
    )
    .await
}

// ---------------------------------------------------------------- seed file

/// The reviewed settings changes, compiled in. Each entry is applied once,
/// through `settings::change` (bounds checked, logged with who and why).
pub const SEED_FILE: &str = include_str!("../../../config/settings-changes.json");

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct SeedChange {
    pub id: String,
    pub key: String,
    pub account_id: Option<i64>,
    pub value: Value,
    pub changed_by: String,
    pub reason: String,
}

pub fn parse_seed(text: &str) -> Result<Vec<SeedChange>, String> {
    let v: Vec<SeedChange> = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut ids = HashSet::new();
    for c in &v {
        if !ids.insert(c.id.clone()) {
            return Err(format!("seed id {} appears twice", c.id));
        }
    }
    Ok(v)
}

/// Apply the seed entries not yet applied. A refused entry is logged and
/// skipped; it is never forced.
pub async fn apply_seed(pool: &PgPool, text: &str) -> anyhow::Result<Vec<String>> {
    let changes = parse_seed(text).map_err(|e| anyhow::anyhow!("settings seed: {e}"))?;
    let mut applied = Vec::new();
    for c in changes {
        if ts::seed_applied(pool, &c.id).await? {
            continue;
        }
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
        // "only human can pause trade": a pause entry is applied only as the
        // owner's word.
        if crate::limits::PAUSE_KEYS.contains(&c.key.as_str()) && c.changed_by != "owner" {
            tracing::error!(seed = %c.id, key = %c.key, changed_by = %c.changed_by,
                "a pause is the owner's alone; refused, not applied");
            continue;
        }
        match jiaoyi_store::settings::change(
            pool,
            &c.key,
            c.account_id,
            Some(c.value.clone()),
            &c.changed_by,
            &c.reason,
        )
        .await
        {
            Ok(version) => {
                ts::mark_seed_applied(pool, &c.id, version).await?;
                tracing::info!(seed = %c.id, key = %c.key, account = ?c.account_id, version, "settings change applied");
                applied.push(c.id);
            }
            Err(e) => {
                tracing::error!(seed = %c.id, key = %c.key, error = %e, "settings change refused; not applied");
            }
        }
    }
    Ok(applied)
}

// ---------------------------------------------------------------- health

/// `/health` → `trading`: per account, the strategy, whether it waits for
/// the edge gate, the minimum-volume rule, the open-risk budget, the last
/// decision and the external positions found at the last reconcile. No money.
pub async fn health(pool: &PgPool, live: &HashMap<i64, Liveness>) -> jiaoyi_store::Result<Value> {
    let accounts = jiaoyi_store::sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let decisions = ts::latest_decisions(pool).await?;
    let mut out = Vec::new();
    for a in accounts {
        let id: i64 = a.get("account_id");
        let env: String = a.get("environment");
        let environment = if env == "live" {
            Environment::Live
        } else {
            Environment::Demo
        };
        let lifted = lifted_by(pool, id).await?;
        let (r, gate) = match rules(pool, id, environment).await {
            Ok(r) => {
                let g = match edge_gate(pool, &r).await {
                    Ok(Ok(())) if r.edge_required => "met".to_string(),
                    Ok(Ok(())) if r.edge_lifted => {
                        format!("{lifted} (trading.live_requires_edge_pass 0)")
                    }
                    Ok(Ok(())) => "not required (demo)".to_string(),
                    Ok(Err(why)) => format!("not met: {why}"),
                    Err(e) => format!("unreadable: {e}"),
                };
                (Some(r), g)
            }
            Err(e) => (None, format!("settings unreadable: {e}")),
        };
        let d = decisions.iter().find(|d| d.account_id == id);
        let rec = ts::latest_decision_detail(pool, id, "reconcile").await?;
        // In memory: every reconcile run and the last quote, recorded or not.
        let l = live.get(&id);
        let liveness = json!({
            "state": r.as_ref().map(|r| liveness_state(l, r.reconcile_secs, Utc::now())),
            "stalled_after_secs": r.as_ref().map(|r| 2 * r.reconcile_secs),
            "started_at": l.and_then(|l| l.started_at),
            "last_reconcile_run_at": l.and_then(|l| l.last_reconcile_run_at),
            "last_quote_at": l.and_then(|l| l.last_quote_at),
        });
        out.push(json!({
            "account_id": id,
            "environment": env,
            "strategy": STRATEGY,
            "trades": match &r {
                Some(r) if r.paused => "paused by the owner: no new entry".to_string(),
                Some(r) if r.edge_required && gate != "met" => {
                    "armed: waits for the edge gate".to_string()
                }
                Some(r) if r.edge_lifted => format!("trading ({lifted})"),
                Some(_) => "yes".to_string(),
                None => "unknown".to_string(),
            },
            "edge_gate": gate,
            "allow_min_volume": r.as_ref().map(|r| r.allow_min_volume),
            "trend_open_risk_pct": r.as_ref().map(|r| r.trend_open_risk_pct),
            "last_decision": d.map(|d| json!({
                "at": d.at, "symbol": d.symbol, "kind": d.kind, "reason": d.reason,
            })),
            "last_reconcile": rec.map(|(at, detail)| json!({
                "at": at, "own": detail["own"], "external_positions": detail["external"],
            })),
            "liveness": liveness,
            // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
            "trades_today": ts::s1_trades_today(pool, id).await?,
            // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
            "performance": crate::limits::health(pool, id, &crate::limits::S1).await.unwrap_or_else(|e| json!({"error": e.to_string()})),
            "owner_pause": crate::limits::owner_pause(pool, id, &crate::limits::S1).await.unwrap_or_else(|e| json!({"error": e.to_string()})),
            "trigger_distance": l.map(|l| json!(l.triggers)).unwrap_or(json!({})),
        }));
    }
    Ok(json!(out))
}

/// Who lifted a live account's edge gate and when, from the settings log:
/// the account's own change when it has one, else the global one.
async fn lifted_by(pool: &PgPool, account_id: i64) -> jiaoyi_store::Result<String> {
    let h = jiaoyi_store::settings::history(pool, Some("trading.live_requires_edge_pass")).await?;
    let row = match h.iter().find(|c| c.account_id == Some(account_id)) {
        Some(c) if c.new_value.is_some() => Some(c),
        _ => h.iter().find(|c| c.account_id.is_none()),
    };
    Ok(match row {
        Some(c) => format!(
            "edge gate lifted by the {} {}",
            c.changed_by,
            c.changed_at.format("%d-%m")
        ),
        None => "edge gate lifted".into(),
    })
}

// ---------------------------------------------------------------- trader

#[derive(Debug, Clone)]
struct Sym {
    id: i64,
    name: String,
    limits: Option<VolumeLimits>,
    quote: Option<String>,
    levels: Result<Levels, String>,
    bid: Option<f64>,
    ask: Option<f64>,
}

#[derive(Debug, Clone)]
struct Own {
    position_id: i64,
    symbol_id: i64,
    side: Side,
    volume: i64,
    /// The broker's VWAP entry price.
    entry: Option<f64>,
    stop: Option<f64>,
    best: Option<f64>,
    last_amend: Option<tokio::time::Instant>,
}

/// An open with neither a fill nor an end recorded (Unconfirmed, or accepted
/// only): the broker may still fill it, so its risk is held in the budget and
/// its symbol takes no entry until reconcile resolves it (Codex P1 on #14).
#[derive(Debug, Clone, PartialEq)]
struct Pending {
    symbol_id: Option<i64>,
    /// The risk the gate recomputed for it, in the deposit currency.
    risk: Option<f64>,
}

/// Strategy 1's book on one session.
#[derive(Default)]
struct Book {
    own: HashMap<i64, Own>,
    /// Unresolved opens, by entry key.
    pending: HashMap<String, Pending>,
    /// Each frame this session sent: its request id to its entry key, so a
    /// late answer is recorded under its intent.
    sent: HashMap<String, String>,
    /// How long an unresolved open with nothing at the broker stays pending.
    grace: chrono::Duration,
}

/// The risk of what is left of an open (`volume` requested, `filled` so far)
/// at the intent's stop distance, from the gate's risk for the whole volume.
pub fn outstanding_risk(risk: Option<f64>, volume: Option<i64>, filled: i64) -> Option<f64> {
    match (risk, volume) {
        (Some(r), Some(v)) if v > 0 => Some(r * (v - filled).clamp(0, v) as f64 / v as f64),
        (r, _) => r,
    }
}

/// Strategy 1's risk at its stops: its positions' and its unresolved opens'.
fn held_risk(book: &Book, syms: &HashMap<i64, Sym>, deposit: Option<&str>) -> Result<f64, String> {
    let mut total = open_risk(&book.own, syms, deposit)?;
    for (key, p) in &book.pending {
        let Some(r) = p.risk else {
            return Err(format!(
                "the unresolved open {key} has no recorded risk: Strategy 1's open risk is unknown"
            ));
        };
        total += r;
    }
    Ok(total)
}

/// Records decisions when they change, not on every tick.
pub struct Decisions<'a> {
    pool: &'a PgPool,
    account_id: i64,
    last: HashMap<String, String>,
    version: Option<i64>,
}

impl Decisions<'_> {
    async fn note(&mut self, symbol: Option<&str>, kind: &str, reason: &str, detail: Value) {
        let k = format!("{}:{kind}", symbol.unwrap_or("-"));
        if self.last.get(&k).map(String::as_str) == Some(reason) {
            return;
        }
        self.last.insert(k, reason.to_string());
        tracing::info!(account = self.account_id, symbol = ?symbol, kind, reason, "strategy 1 decision");
        if let Err(e) = ts::record_decision(
            self.pool,
            self.account_id,
            symbol,
            kind,
            reason,
            &detail,
            self.version,
        )
        .await
        {
            tracing::error!(error = %e, "strategy 1 decision not recorded");
        }
    }
}

/// Run every permitted account's Strategy 1 loop until shutdown.
pub async fn run(pool: PgPool, cfg: Arc<Config>, live: LivenessMap, shutdown: Shutdown) {
    let mut tasks = Vec::new();
    for (account, env) in cfg.plan.permitted.clone() {
        let (pool, cfg, live, shutdown) =
            (pool.clone(), cfg.clone(), live.clone(), shutdown.clone());
        tasks.push(tokio::spawn(async move {
            account_loop(&pool, &cfg, &account, env, &live, shutdown).await
        }));
    }
    for t in tasks {
        let _ = t.await;
    }
}

async fn account_loop(
    pool: &PgPool,
    cfg: &Config,
    account: &PermittedAccount,
    env: Environment,
    live: &LivenessMap,
    mut shutdown: Shutdown,
) {
    let mut d = Decisions {
        pool,
        account_id: account.id(),
        last: HashMap::new(),
        version: None,
    };
    beat(live, account.id(), |l| l.started_at = Some(Utc::now()));
    loop {
        if shutdown.is_set() {
            return;
        }
        let r = session(pool, cfg, account, env, &mut d, live, shutdown.clone()).await;
        if shutdown.is_set() {
            return;
        }
        if let Err(e) = r {
            d.note(
                None,
                "error",
                &format!("the trading session stopped: {e:#}"),
                json!({}),
            )
            .await;
        }
        let wait = match rules(pool, account.id(), env).await {
            Ok(r) => r.retry_secs,
            Err(_) => 60,
        };
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(wait)) => {}
            _ = shutdown.wait() => return,
        }
    }
}

/// The deposit currency per one unit of `quote`, from the broker's live
/// quotes of a pair joining the two (`QUOTEDEPOSIT` or `DEPOSITQUOTE`).
fn quote_to_deposit(quote: &str, deposit: &str, syms: &HashMap<i64, Sym>) -> Option<f64> {
    if quote == deposit {
        return Some(1.0);
    }
    let mid = |name: &str| {
        syms.values()
            .find(|s| s.name == name)
            .and_then(|s| Some((s.bid? + s.ask?) / 2.0))
            .filter(|m| m.is_finite() && *m > 0.0)
    };
    mid(&format!("{quote}{deposit}")).or_else(|| mid(&format!("{deposit}{quote}")).map(|m| 1.0 / m))
}

fn rate_of(sym_id: i64, syms: &HashMap<i64, Sym>, deposit: Option<&str>) -> Option<f64> {
    let s = syms.get(&sym_id)?;
    quote_to_deposit(s.quote.as_deref()?, deposit?, syms)
}

/// Strategy 1's open risk at its stops, or why it cannot be told.
fn open_risk(
    own: &HashMap<i64, Own>,
    syms: &HashMap<i64, Sym>,
    deposit: Option<&str>,
) -> Result<f64, String> {
    let mut total = 0.0;
    for o in own.values() {
        let (Some(entry), Some(stop)) = (o.entry, o.stop) else {
            return Err(format!(
                "position {} has no known entry or stop: its open risk is unknown",
                o.position_id
            ));
        };
        let Some(rate) = rate_of(o.symbol_id, syms, deposit) else {
            return Err(format!(
                "position {} has no quote→deposit rate: its open risk is unknown",
                o.position_id
            ));
        };
        total += risk_at_stop(o.side, entry, stop, o.volume, rate);
    }
    Ok(total)
}

async fn read_levels(session: &AccountSession<'_>, symbol_id: i64) -> Result<Levels, String> {
    let now = Utc::now();
    let from = now - chrono::Duration::days(BAR_DAYS);
    let (raw, _) = session
        .daily_trendbars(symbol_id, from, now)
        .await
        .map_err(|e| format!("daily bars unreadable: {e}"))?;
    let mut bars = Vec::new();
    for tb in &raw {
        bars.push(jiaoyi_ctrader::decode_trendbar(tb).map_err(|e| e.to_string())?);
    }
    let (closed, _) = jiaoyi_ctrader::closed_daily_bars(bars, now);
    let bars: Vec<Bar> = closed
        .iter()
        .filter_map(|b| Some((b.date, b.high.to_f64()?, b.low.to_f64()?, b.close.to_f64()?)))
        .collect();
    levels(&bars)
}

/// Fresh equity: the broker's balance plus the open positions' unrealized
/// P&L, in the deposit currency (also Strategy 3's).
pub async fn fresh_equity(session: &AccountSession<'_>) -> Result<f64, String> {
    let t = session.trader().await.map_err(|e| e.to_string())?;
    let digits = t
        .money_digits
        .ok_or("the trader record gives no moneyDigits")?;
    let u = session.unrealized_pnl().await.map_err(|e| e.to_string())?;
    let scale = 10f64.powi(digits as i32);
    let uscale = 10f64.powi(u.money_digits as i32);
    let unrealized: f64 = u
        .position_unrealized_pn_l
        .iter()
        .map(|p| p.net_unrealized_pn_l as f64 / uscale)
        .sum();
    Ok(t.balance as f64 / scale + unrealized)
}

/// An event reconcile writes under an intent: a fill it found
/// (`RECONCILED_FILL`, or `RECONCILED_PARTIAL_FILL` for part of it), or an
/// unresolved open it released (`RECONCILED_ABSENT`).
pub fn reconciled_row(
    account_id: i64,
    entry_key: &str,
    kind: &str,
    p: Option<&jiaoyi_ctrader::messages::ProtoOaPosition>,
    description: &str,
) -> ts::EventRow {
    ts::EventRow {
        account_id,
        entry_key: Some(entry_key.to_string()),
        client_msg_id: None,
        payload_type: 0,
        kind: kind.into(),
        position_id: p.map(|p| p.position_id),
        order_id: None,
        deal_id: None,
        symbol_id: p.map(|p| p.trade_data.symbol_id),
        side: p
            .and_then(|p| Side::from_proto(p.trade_data.trade_side))
            .map(|s| s.as_str().to_string()),
        price: p.and_then(|p| p.price),
        volume: p.map(|p| p.trade_data.volume),
        stop_loss: p.and_then(|p| p.stop_loss),
        label: p.and_then(|p| p.trade_data.label.clone()),
        error_code: None,
        description: Some(description.into()),
        is_server_event: None,
        raw: vec![],
        received_at: Utc::now(),
    }
}

/// What a reconcile found.
struct Reconciled {
    own: usize,
    pending: usize,
    external: Vec<Value>,
    /// Unresolved opens released (neither filled nor pending at the broker
    /// after their grace).
    released: Vec<String>,
}

/// The broker wins. Positions with this program's label are adopted when an
/// intent of its own filled them, or when an unresolved open on their symbol
/// did (accepted only, or no answer: Codex P1 on #14); any other position is
/// external. An unresolved open still pending at the broker, or younger than
/// the book's grace, stays pending; an older one with nothing at the broker is
/// released (`RECONCILED_ABSENT`), never resent.
async fn reconcile(
    pool: &PgPool,
    session: &AccountSession<'_>,
    book: &mut Book,
) -> anyhow::Result<Reconciled> {
    let grace = book.grace;
    let res = session.reconcile().await?;
    let account_id = session.account().id();
    let mut seen = HashSet::new();
    let mut external: Vec<Value> = Vec::new();
    let ext = |p: &jiaoyi_ctrader::messages::ProtoOaPosition| {
        json!({
            "position_id": p.position_id,
            "symbol_id": p.trade_data.symbol_id,
            "side": Side::from_proto(p.trade_data.trade_side).map(|s| s.as_str()),
            "volume": p.trade_data.volume,
            "label": p.trade_data.label,
        })
    };
    // Strategy 1's own unresolved opens (Strategy 3 reconciles its own).
    let mut unresolved: Vec<ts::UnresolvedIntent> = ts::open_intents_unresolved(pool, account_id)
        .await?
        .into_iter()
        .filter(|u| u.entry_key.starts_with("s1:"))
        .collect();
    for p in &res.position {
        // Claude · № 17,020 25-Oct (Strategy 3 v2: each strategy manages only its own label)
        if p.trade_data.label.as_deref() != Some(LABEL) {
            if !p
                .trade_data
                .label
                .as_deref()
                .is_some_and(|l| gate::PROGRAM_LABELS.contains(&l))
            {
                external.push(ext(p));
            }
            continue;
        }
        let mut owned = ts::owns_position(pool, account_id, p.position_id).await?;
        if !owned {
            // An unresolved open that did fill: adopted once, recorded. In
            // full only at the requested volume; a part keeps the intent
            // unresolved, its remainder reserved below until it fills, ends
            // or is absent after the grace (Codex round 4 on #14).
            // Claude · № 17,020 25-Oct (ordered: owner "Merge now, fix next")
            if let Some(i) = unresolved.iter().position(|u| {
                u.symbol_id == Some(p.trade_data.symbol_id) && u.position_id.is_none()
            }) {
                let part = unresolved[i]
                    .volume
                    .is_some_and(|v| p.trade_data.volume < v);
                let (kind, why) = if part {
                    (
                        "RECONCILED_PARTIAL_FILL",
                        "an unresolved open found partly filled by reconcile: the position adopted, the remainder held",
                    )
                } else {
                    (
                        "RECONCILED_FILL",
                        "an unresolved open found filled by reconcile",
                    )
                };
                ts::record_event(
                    pool,
                    &reconciled_row(account_id, &unresolved[i].entry_key, kind, Some(p), why),
                )
                .await?;
                if part {
                    unresolved[i].position_id = Some(p.position_id);
                } else {
                    unresolved.remove(i);
                }
                owned = true;
            }
        }
        if !owned {
            external.push(ext(p));
            continue;
        }
        let Some(side) = Side::from_proto(p.trade_data.trade_side) else {
            continue;
        };
        seen.insert(p.position_id);
        let e = book.own.entry(p.position_id).or_insert(Own {
            position_id: p.position_id,
            symbol_id: p.trade_data.symbol_id,
            side,
            volume: p.trade_data.volume,
            entry: p.price,
            stop: p.stop_loss,
            best: None,
            last_amend: None,
        });
        // The broker wins.
        e.stop = p.stop_loss;
        e.volume = p.trade_data.volume;
        if p.price.is_some() {
            e.entry = p.price;
        }
    }
    book.own.retain(|id, _| seen.contains(id));
    book.pending.clear();
    let now = Utc::now();
    let mut released = Vec::new();
    for u in unresolved {
        // What of it is filled: the broker's position when it is held, else
        // the deals recorded. The adopted part's risk is the position's; the
        // remainder's is held here, never both (Codex round 3 on #14).
        let filled = u
            .position_id
            .and_then(|p| book.own.get(&p))
            .map(|o| o.volume)
            .unwrap_or(u.filled);
        if u.volume.is_some_and(|v| filled >= v) {
            // Filled in full at the broker, the last answer missed.
            ts::record_event(
                pool,
                &reconciled_row(
                    account_id,
                    &u.entry_key,
                    "RECONCILED_FILL",
                    None,
                    "an open found filled in full at the broker by reconcile",
                ),
            )
            .await?;
            continue;
        }
        let at_broker = res.order.iter().any(|o| {
            o.trade_data.label.as_deref() == Some(LABEL)
                && Some(o.trade_data.symbol_id) == u.symbol_id
        });
        if at_broker || now - u.created_at < grace {
            book.pending.insert(
                u.entry_key,
                Pending {
                    symbol_id: u.symbol_id,
                    risk: outstanding_risk(u.risk, u.volume, filled),
                },
            );
            continue;
        }
        ts::record_event(
            pool,
            &reconciled_row(
                account_id,
                &u.entry_key,
                "RECONCILED_ABSENT",
                None,
                "an unresolved open neither filled nor pending at the broker after its grace: released, never resent",
            ),
        )
        .await?;
        released.push(u.entry_key);
    }
    Ok(Reconciled {
        own: book.own.len(),
        pending: book.pending.len(),
        external,
        released,
    })
}

/// The reconcile decision: own positions, unresolved opens and external
/// positions (symbol, side, volume; never money), recorded when they change.
async fn note_reconcile(d: &mut Decisions<'_>, r: Reconciled) {
    for key in &r.released {
        d.note(
            None,
            "released",
            &format!(
                "the unresolved open {key} was neither filled nor pending at the broker after its grace: released, never resent"
            ),
            json!({ "entry_key": key }),
        )
        .await;
    }
    let summary: Vec<String> = r
        .external
        .iter()
        .map(|e| {
            format!(
                "{}:{}:{}",
                e["symbol_id"],
                e["side"].as_str().unwrap_or("?"),
                e["volume"]
            )
        })
        .collect();
    d.note(
        None,
        "reconcile",
        &format!(
            "{} own position(s); {} unresolved open(s); {} external, never touched [{}]",
            r.own,
            r.pending,
            r.external.len(),
            summary.join(", ")
        ),
        json!({ "own": r.own, "pending": r.pending, "external": r.external }),
    )
    .await;
}

/// One connected session. Returns on shutdown, or with the error that ended it.
async fn session(
    pool: &PgPool,
    cfg: &Config,
    account: &PermittedAccount,
    env: Environment,
    d: &mut Decisions<'_>,
    live: &LivenessMap,
    mut shutdown: Shutdown,
) -> anyhow::Result<()> {
    let conn: Connection = loader::connect(&cfg.broker, env).await?;
    let session = loader::authorise(pool, &conn, &cfg.broker, account).await?;
    let mut ends = conn.session_ends();
    run_session(
        pool,
        &cfg.allowlist,
        env,
        &session,
        d,
        live,
        &mut ends,
        &mut shutdown,
    )
    .await
}

/// How long an unresolved open with nothing at the broker stays pending:
/// one reconcile interval.
fn grace_of(r: &Rules) -> chrono::Duration {
    chrono::Duration::seconds(r.reconcile_secs as i64)
}

/// The Strategy 1 loop on an authorised session (also driven by the tests on
/// the scripted broker).
#[allow(clippy::too_many_arguments)]
pub async fn run_session(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    d: &mut Decisions<'_>,
    live: &LivenessMap,
    ends: &mut tokio::sync::broadcast::Receiver<jiaoyi_ctrader::SessionEnd>,
    shutdown: &mut Shutdown,
) -> anyhow::Result<()> {
    let account_id = session.account().id();
    beat(live, account_id, |l| {
        l.started_at.get_or_insert_with(Utc::now);
    });
    let conn = session.connection();
    let mut syms: HashMap<i64, Sym> = HashMap::new();
    let quotes = jiaoyi_store::latest_assets(pool, account_id)
        .await?
        .map(|(run, assets, listed)| QuoteAssets::from_run(run, &assets, &listed));
    let deposit = match (
        jiaoyi_store::research::deposit_asset_id(pool, account_id).await?,
        &quotes,
    ) {
        (Some(id), Some(q)) => q.asset_name(id).map(str::to_string),
        _ => None,
    };
    for r in jiaoyi_store::latest_resolutions(pool, account_id).await? {
        let Some(id) = r.broker_symbol_id else {
            continue;
        };
        let limits = jiaoyi_store::sqlx::query(
            "SELECT min_volume, volume_step, max_volume, digits FROM symbols
             WHERE account_id = $1 AND broker_symbol_id = $2",
        )
        .bind(account_id)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .and_then(|row| {
            Some(VolumeLimits {
                min: row.get::<Option<i64>, _>("min_volume")?,
                step: row.get::<Option<i64>, _>("volume_step")?,
                max: row.get::<Option<i64>, _>("max_volume"),
                digits: row.get::<i32, _>("digits"),
            })
        });
        syms.insert(
            id,
            Sym {
                id,
                name: r.name.clone(),
                limits,
                quote: quotes
                    .as_ref()
                    .and_then(|q| q.quote_of(id).ok().map(|x| x.0)),
                levels: Err("not read yet".into()),
                bid: None,
                ask: None,
            },
        );
    }
    if syms.is_empty() {
        anyhow::bail!("no resolved symbols stored for this account yet (a data load comes first)");
    }
    let ids: Vec<i64> = syms.keys().copied().collect();
    for id in &ids {
        let l = read_levels(session, *id).await;
        if let Some(s) = syms.get_mut(id) {
            s.levels = l;
        }
    }
    let r0 = rules(pool, account_id, env).await?;
    d.version = Some(r0.version);
    let mut trail_amend_secs = r0.trail_amend_secs;
    let mut book = Book {
        grace: grace_of(&r0),
        ..Book::default()
    };
    let mut execs = conn.executions();
    let rec = reconcile(pool, session, &mut book).await?;
    note_reconcile(d, rec).await;
    beat(live, account_id, |l| {
        l.last_reconcile_run_at = Some(Utc::now())
    });
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
    // the limits alert; they never pause.
    crate::limits::check(
        pool,
        account_id,
        &crate::limits::S1,
        limit_detail(),
        d.version,
    )
    .await?;
    let mut spots = conn.spot_events();
    session.subscribe_spots(&ids).await?;
    let mut reconcile_tick = tokio::time::interval(Duration::from_secs(r0.reconcile_secs));
    reconcile_tick.tick().await;
    let mut levels_tick = tokio::time::interval(LEVELS_REFRESH);
    levels_tick.tick().await;
    let mut tried: HashSet<String> = HashSet::new();
    let ctx = Ctx {
        pool,
        allowlist,
        session,
        env,
    };
    loop {
        tokio::select! {
            _ = shutdown.wait() => return Ok(()),
            e = ends.recv() => {
                if let Ok(e) = e {
                    if e.affects(account_id) {
                        anyhow::bail!("the broker ended the session: {}", e.reason);
                    }
                }
            }
            _ = reconcile_tick.tick() => {
                let r = rules(pool, account_id, env).await?;
                d.version = Some(r.version);
                trail_amend_secs = r.trail_amend_secs;
                book.grace = grace_of(&r);
                let rec = reconcile(pool, session, &mut book).await?;
                note_reconcile(d, rec).await;
                beat(live, account_id, |l| l.last_reconcile_run_at = Some(Utc::now()));
                crate::limits::check(pool, account_id, &crate::limits::S1, limit_detail(), d.version).await?;
            }
            _ = levels_tick.tick() => {
                for id in &ids {
                    let l = read_levels(session, *id).await;
                    if let Some(s) = syms.get_mut(id) { s.levels = l; }
                }
            }
            m = execs.recv() => match m {
                Ok(m) => {
                    // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design"):
                    // a stop the broker executed is recorded with its fill, before
                    // reconcile drops the position. A lost record ends the session.
                    broker_exit(pool, account_id, &m, &book, &syms).await?;
                    // A late answer (the gate had stopped listening): a fill
                    // of any size is read from the broker at once, adopted and
                    // trailed; an end of an unresolved open frees its risk
                    // (Codex rounds 1 and 2 on #14).
                    if let Some(late) = server_event(pool, account_id, &m, &book.sent).await {
                        let fill = ts::FILL_KINDS.contains(&late.kind.as_str());
                        let end = ts::END_KINDS.contains(&late.kind.as_str());
                        if late.new && (fill || (end && book.pending.contains_key(&late.entry_key))) {
                            let rec = reconcile(pool, session, &mut book).await?;
                            note_reconcile(d, rec).await;
                            beat(live, account_id, |l| l.last_reconcile_run_at = Some(Utc::now()));
                        }
                    }
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
            },
            s = spots.recv() => match s {
                Ok(spot) => {
                    let ev = spot.event;
                    let Some(sym) = syms.get_mut(&ev.symbol_id) else { continue };
                    if let Some(b) = ev.bid { sym.bid = Some(b as f64 / 100_000.0); }
                    if let Some(a) = ev.ask { sym.ask = Some(a as f64 / 100_000.0); }
                    let sym = sym.clone();
                    on_quote(&ctx, d, &syms, &sym, deposit.as_deref(), &mut book, &mut tried, trail_amend_secs).await?;
                    let now = Utc::now();
                    let distance = trigger_distance(&sym.levels, sym.bid, sym.ask, now);
                    beat(live, account_id, |l| {
                        l.last_quote_at = Some(now);
                        l.triggers.insert(sym.name.clone(), distance);
                    });
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
            },
        }
    }
}

/// What Strategy 1's limit decisions carry beside the numbers.
fn limit_detail() -> Value {
    json!({"strategy": STRATEGY, "label": LABEL})
}

struct Ctx<'a, 'c> {
    pool: &'a PgPool,
    allowlist: &'a Allowlist,
    session: &'a AccountSession<'c>,
    env: Environment,
}

/// An answer to a frame this session sent, as the trader received it.
struct Answer {
    entry_key: String,
    kind: String,
    /// Not recorded before: it came after the gate stopped listening.
    new: bool,
}

/// An execution message outside the gate's wait, recorded as it came: the
/// broker's own (a stop hit), or an answer to a frame this session sent,
/// recorded under its intent (Codex P1 on #14: a late fill was dropped). An
/// answer the gate already recorded is kept once.
async fn server_event(
    pool: &PgPool,
    account_id: i64,
    m: &ReceivedExecution,
    sent: &HashMap<String, String>,
) -> Option<Answer> {
    let key = match &m.client_msg_id {
        None => None,
        Some(id) => match sent.get(id) {
            Some(k) => Some(k.clone()),
            // Not a frame of the trading path.
            None => return None,
        },
    };
    let mut e = gate::decode_execution(account_id, m)?;
    // Claude · № 17,020 25-Oct: another strategy's position (a broker event
    // with no request of ours): its own loop records it.
    if key.is_none()
        && e.label
            .as_deref()
            .is_some_and(|l| l != LABEL && gate::PROGRAM_LABELS.contains(&l))
    {
        return None;
    }
    e.entry_key = key.clone();
    let new = match ts::record_event(pool, &event_row(&e)).await {
        Ok(new) => new,
        Err(err) => {
            tracing::error!(error = %err, "execution event not recorded");
            true
        }
    };
    key.map(|entry_key| Answer {
        entry_key,
        kind: e.kind,
        new,
    })
}

#[allow(clippy::too_many_arguments)]
async fn on_quote(
    ctx: &Ctx<'_, '_>,
    d: &mut Decisions<'_>,
    syms: &HashMap<i64, Sym>,
    sym: &Sym,
    deposit: Option<&str>,
    book: &mut Book,
    tried: &mut HashSet<String>,
    trail_amend_secs: u64,
) -> anyhow::Result<()> {
    let (pool, allowlist, session) = (ctx.pool, ctx.allowlist, ctx.session);
    let account_id = session.account().id();
    let name = sym.name.as_str();
    // Trail Strategy 1's positions on this symbol.
    let lv = sym.levels.as_ref().ok();
    for o in book.own.values_mut().filter(|o| o.symbol_id == sym.id) {
        let price = match o.side {
            Side::Buy => sym.bid,
            Side::Sell => sym.ask,
        };
        let (Some(p), Some(l), Some(limits)) = (price, lv, sym.limits) else {
            continue;
        };
        let best = match (o.best, o.stop) {
            (Some(b), _) => match o.side {
                Side::Buy => b.max(p),
                Side::Sell => b.min(p),
            },
            // After a restart: the best implied by the broker's stop, or the quote.
            (None, Some(s)) => match o.side {
                Side::Buy => p.max(s + STOP_ATR * l.atr),
                Side::Sell => p.min(s - STOP_ATR * l.atr),
            },
            (None, None) => p,
        };
        o.best = Some(best);
        let Some(new_stop) = trail(o.side, best, l.atr, o.stop, limits.digits) else {
            continue;
        };
        // The amend limit is checked before any read, so a trending quote
        // stream does not read the settings on every new best price.
        if o.last_amend
            .is_some_and(|t| t.elapsed() < Duration::from_secs(trail_amend_secs))
        {
            continue;
        }
        o.last_amend = Some(tokio::time::Instant::now());
        let r = rules(pool, account_id, ctx.env).await?;
        let policy = Policy {
            edge: edge_gate(pool, &r).await?,
            max_risk: f64::INFINITY,
        };
        let key = format!("s1-trail:{account_id}:{}:{new_stop}", o.position_id);
        let ledger = TimedLedger::new(pool, Some(r.version));
        let from_stop = o.stop;
        let cmd = Command::AmendStop {
            position_id: o.position_id,
            side: o.side,
            current_stop: o.stop,
            new_stop,
        };
        match gate::send(
            session,
            allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
        )
        .await
        {
            Ok(sent) => {
                book.sent.insert(sent.client_msg_id.clone(), key.clone());
                if matches!(sent.outcome, Outcome::Amended { .. }) {
                    o.stop = Some(new_stop);
                }
                d.note(
                    Some(name),
                    "trail",
                    &format!("stop to {new_stop}: {:?}", sent.outcome),
                    json!({"position_id": o.position_id, "entry_key": key}),
                )
                .await;
                // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design"):
                // the amend's answer and its time, recorded; nothing it decides reads them.
                // A lost record ends the session (it reconnects and reconciles), as
                // Strategy 3's does.
                let fill = FillOf {
                    account_id,
                    key: &key,
                    kind: "trail",
                    symbol: name,
                    side: o.side,
                    position_id: Some(o.position_id),
                    decision_price: Some(new_stop),
                    bid: sym.bid,
                    ask: sym.ask,
                    volume: o.volume,
                    detail: json!({"strategy": STRATEGY, "from_stop": from_stop, "best": best}),
                };
                record_fill(pool, fill, ledger.sent_at(), &sent).await?;
            }
            Err(e) => {
                d.note(
                    Some(name),
                    "trail_refused",
                    &e.to_string(),
                    json!({"position_id": o.position_id}),
                )
                .await;
            }
        }
    }
    // An entry, when the symbol has no Strategy 1 position.
    if book.own.values().any(|o| o.symbol_id == sym.id) {
        return Ok(());
    }
    let l = match &sym.levels {
        Ok(l) => l,
        Err(why) => {
            d.note(Some(name), "skip", &format!("no levels: {why}"), json!({}))
                .await;
            return Ok(());
        }
    };
    let Some(side) = triggered(l, sym.bid, sym.ask) else {
        return Ok(());
    };
    // An unresolved open on this symbol may still fill: one per symbol.
    if book.pending.values().any(|p| p.symbol_id == Some(sym.id)) {
        d.note(
            Some(name),
            "skip",
            "an unresolved open on this symbol may still fill: no entry until reconcile resolves it",
            json!({}),
        )
        .await;
        return Ok(());
    }
    let key = entry_key(account_id, sym.id, l.last_completed);
    if tried.contains(&key) || ts::intent_exists(pool, &key).await? {
        tried.insert(key);
        return Ok(());
    }
    tried.insert(key.clone());
    let r = rules(pool, account_id, ctx.env).await?;
    let detail = json!({"strategy": STRATEGY, "level": l.level, "atr": l.atr,
        "ema_fast": l.ema_fast, "ema_slow": l.ema_slow, "bid": sym.bid, "ask": sym.ask,
        "side": side.as_str(), "after": l.last_completed});
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
    // the owner's pause ("only human can pause trade"): no new entry, the
    // signal kept for when the owner lifts it. Exits above are untouched.
    if r.paused {
        d.note(
            Some(name),
            "paused",
            "Strategy 1 is paused by the owner on this account (trading.s1_paused): no new entry; open positions keep their exits",
            detail,
        )
        .await;
        tried.remove(&key);
        return Ok(());
    }
    let edge = edge_gate(pool, &r).await?;
    if let Err(why) = &edge {
        d.note(
            Some(name),
            "signal_held",
            &format!(
                "{STRATEGY} {} signal; edge gate not met: {why}",
                side.as_str()
            ),
            detail,
        )
        .await;
        // The key is not recorded: the signal may be taken once the gate opens.
        tried.remove(&key);
        return Ok(());
    }
    let Some(limits) = sym.limits else {
        d.note(
            Some(name),
            "skip",
            "the symbol's volume rules are not stored",
            detail,
        )
        .await;
        return Ok(());
    };
    let rel = relative_stop(STOP_ATR * l.atr, limits.digits);
    let Some(rate) = rate_of(sym.id, syms, deposit) else {
        d.note(
            Some(name),
            "skip",
            "no broker quote→deposit rate for this symbol",
            detail,
        )
        .await;
        // The conversion quote may arrive later the same day (Codex P2 on #14).
        tried.remove(&key);
        return Ok(());
    };
    let equity = match fresh_equity(session).await {
        Ok(e) => e,
        Err(why) => {
            d.note(
                Some(name),
                "skip",
                &format!("fresh equity unreadable: {why}"),
                detail,
            )
            .await;
            tried.remove(&key);
            return Ok(());
        }
    };
    let sized = match size(&r, equity, rel, rate, &limits) {
        Ok(s) => s,
        Err(why) => {
            d.note(Some(name), "skip", &why, detail).await;
            return Ok(());
        }
    };
    // Claude · № 17,020 25-Oct (Codex P1 on #15): a live broker-minimum entry
    // is admitted account-wide, across Strategy 1 and Strategy 3, from fresh
    // broker state; the account's lock is held through the send.
    let _admission = if sized.mode == "broker_minimum" {
        let g = crate::admission::lock(account_id).await;
        if let Err(why) = crate::admission::live_minimum(pool, session).await {
            d.note(Some(name), "skip", &why, detail).await;
            tried.remove(&key);
            return Ok(());
        }
        Some(g)
    } else {
        None
    };
    // Strategy 1's open-risk budget (owner, № 16,806), its unresolved opens
    // included (Codex P1 on #14); one minimum trade at a time (owner, 25-10).
    let admitted = admit(
        &sized,
        book.own.len() + book.pending.len(),
        held_risk(book, syms, deposit),
        equity,
        r.trend_open_risk_pct,
    );
    if let Err(why) = admitted {
        d.note(Some(name), "skip", &why, detail).await;
        tried.remove(&key);
        return Ok(());
    }
    let policy = Policy {
        edge,
        max_risk: sized.max_risk,
    };
    let ledger = TimedLedger::new(pool, Some(r.version));
    let cmd = Command::Open {
        symbol_id: sym.id,
        side,
        volume: sized.volume,
        relative_stop: rel,
        relative_take_profit: None,
        limits,
        quote_to_deposit: Some(rate),
        label: LABEL.into(),
    };
    match gate::send(
        session,
        allowlist,
        &ledger,
        &policy,
        cmd,
        &key,
        Duration::from_secs(r.order_wait_secs),
    )
    .await
    {
        Ok(sent) => {
            book.sent.insert(sent.client_msg_id.clone(), key.clone());
            // Any fill, whatever the last answer (a fill in several deals, or
            // a partial fill whose remainder was cancelled or expired: Codex
            // round 2 on #14), is read from the broker at once: its aggregate
            // volume, entry price and stop are adopted before any further
            // entry is admitted. An event carries one deal, not the position.
            let filled = sent
                .events
                .iter()
                .any(|e| e.kind == "ORDER_FILLED" || e.kind == "ORDER_PARTIAL_FILL");
            if filled {
                let rec = reconcile(pool, session, book).await?;
                note_reconcile(d, rec).await;
            }
            let adopted = book.own.values().any(|o| o.symbol_id == sym.id);
            if !adopted && (filled || matches!(sent.outcome, Outcome::Unconfirmed)) {
                // The broker may hold it, or still fill it: its risk is held
                // and its symbol takes no entry until reconcile resolves it.
                book.pending.insert(
                    key.clone(),
                    Pending {
                        symbol_id: Some(sym.id),
                        risk: Some(sized.risk),
                    },
                );
            }
            // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design"):
            // the decision carries the signal and the quote it was taken at.
            let mut sent_detail = detail.clone();
            sent_detail["entry_key"] = json!(key);
            sent_detail["relative_stop"] = json!(rel);
            sent_detail["sizing"] = json!(sized.mode);
            d.note(
                Some(name),
                "sent",
                &format!(
                    "{STRATEGY} {} {} cents ({}): {:?}",
                    side.as_str(),
                    sized.volume,
                    sized.mode,
                    sent.outcome
                ),
                sent_detail,
            )
            .await;
            // The answer and its time, recorded after everything the trader
            // decides from it; nothing reads it back to trade. A lost record
            // ends the session (it reconnects and reconciles), as Strategy 3's does.
            let fill = FillOf {
                account_id,
                key: &key,
                kind: "open",
                symbol: name,
                side,
                position_id: None,
                decision_price: match side {
                    Side::Buy => sym.ask,
                    Side::Sell => sym.bid,
                },
                bid: sym.bid,
                ask: sym.ask,
                volume: sized.volume,
                detail: json!({"strategy": STRATEGY, "quote_to_deposit": rate,
                    "sizing": sized.mode}),
            };
            record_fill(pool, fill, ledger.sent_at(), &sent).await?;
        }
        Err(SendError::Refused(x)) => {
            let mut detail = detail;
            detail["entry_key"] = json!(key);
            d.note(Some(name), "refused", &x.reason, detail).await;
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

/// For the tests: a decisions recorder on an account.
pub fn decisions(pool: &PgPool, account_id: i64) -> Decisions<'_> {
    Decisions {
        pool,
        account_id,
        last: HashMap::new(),
        version: None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_loop_is_stalled_after_twice_its_reconcile_interval() {
        let t0: chrono::DateTime<Utc> = "2026-10-25T04:02:28Z".parse().unwrap();
        let s = |secs| t0 + chrono::Duration::seconds(secs);
        assert_eq!(liveness_state(None, 300, t0), "not running");
        let started = Liveness {
            started_at: Some(t0),
            ..Liveness::default()
        };
        assert_eq!(liveness_state(Some(&started), 300, s(600)), "starting");
        assert_eq!(liveness_state(Some(&started), 300, s(601)), "stalled");
        let ran = Liveness {
            last_reconcile_run_at: Some(t0),
            ..started
        };
        assert_eq!(liveness_state(Some(&ran), 300, s(600)), "alive");
        assert_eq!(liveness_state(Some(&ran), 300, s(601)), "stalled");
    }

    use super::*;

    fn sw() -> Rules {
        Rules {
            version: 1,
            live: false,
            edge_required: false,
            edge_lifted: false,
            risk_pct: 0.6,
            cap_factor: 1.1,
            allow_min_volume: false,
            min_volume_max_risk_pct: 10.0,
            trend_open_risk_pct: 3.0,
            reconcile_secs: 300,
            trail_amend_secs: 60,
            order_wait_secs: 20,
            retry_secs: 60,
            paused: false,
        }
    }

    const ETH: VolumeLimits = VolumeLimits {
        min: 1,
        step: 1,
        max: None,
        digits: 2,
    };
    const CRUDE: VolumeLimits = VolumeLimits {
        min: 100,
        step: 100,
        max: None,
        digits: 3,
    };

    #[test]
    fn risk_pct_sizing_floors_to_the_step() {
        // $1,002.44 at 0.6% = $6.01; ETH 3 × ATR = 284.57: 1 cent risks 2.8457.
        let s = size(&sw(), 1002.44, 28_457_000, 1.0, &ETH).unwrap();
        assert_eq!(s.volume, 2);
        assert_eq!(s.mode, "risk_pct");
        assert!((s.risk - 5.6914).abs() < 1e-9);
        assert!((s.max_risk - 1.1 * 6.01464).abs() < 1e-6);
    }

    #[test]
    fn below_the_minimum_is_skipped_unless_min_volume_is_allowed() {
        // Live's $38.98: 0.6% = $0.23, below ETH's 1-cent minimum ($2.85).
        let e = size(&sw(), 38.98, 28_457_000, 1.0, &ETH).unwrap_err();
        assert!(e.contains("below the broker minimum"), "{e}");
        let live = Rules {
            allow_min_volume: true,
            ..sw()
        };
        // The owner's figure: the minimum risks $2.85 = 7.3% ≤ 10%.
        let s = size(&live, 38.98, 28_457_000, 1.0, &ETH).unwrap();
        assert_eq!((s.volume, s.mode), (1, "broker_minimum"));
        assert!((s.risk / 38.98 * 100.0 - 7.3).abs() < 0.05);
        assert!((s.max_risk - 3.898).abs() < 1e-9);
        // SpotCrude's minimum (1 unit, 3 × ATR = 12.079) risks $12.08 = 31%: skipped.
        let e = size(&live, 38.98, 1_207_900, 1.0, &CRUDE).unwrap_err();
        assert!(e.contains("above 10% of equity"), "{e}");
    }

    #[test]
    fn strategy_1_holds_at_most_its_open_risk_budget() {
        // $1,002.44 × 3% = $30.07: five 0.6% trades of $6.01 fit (30.07),
        // a sixth does not.
        let eq = 1002.44;
        assert!(within_budget(4.0 * 6.01, 6.01, eq, 3.0).is_ok());
        let e = within_budget(5.0 * 6.01, 6.01, eq, 3.0).unwrap_err();
        assert!(e.contains("exceed its budget"), "{e}");
        // Live at $38.98: 3% is $1.17, below ETHUSD's minimum-volume $2.85.
        assert!(within_budget(0.0, 2.85, 38.98, 3.0).is_err());
    }

    #[test]
    fn an_open_remainder_holds_its_share_of_the_risk() {
        // 100 cents requested at $120.00; 60 filled: the 40 left hold $48.00.
        assert!((outstanding_risk(Some(120.0), Some(100), 60).unwrap() - 48.0).abs() < 1e-9);
        assert_eq!(outstanding_risk(Some(120.0), Some(100), 0), Some(120.0));
        assert_eq!(outstanding_risk(Some(120.0), Some(100), 130), Some(0.0));
        assert_eq!(outstanding_risk(None, Some(100), 60), None);
    }

    #[test]
    fn one_minimum_trade_at_a_time_replaces_the_budget_for_itself() {
        // Owner, 25-10: "One minimum trade at a time". Live at $38.98, the
        // ETHUSD minimum risks $2.85, above the 3% budget ($1.17): taken
        // alone, never beside another position or unresolved open.
        let min = Sized {
            volume: 1,
            max_risk: 3.898,
            mode: "broker_minimum",
            risk: 2.85,
        };
        assert!(admit(&min, 0, Ok(0.0), 38.98, 3.0).is_ok());
        let e = admit(&min, 1, Ok(2.85), 38.98, 3.0).unwrap_err();
        assert!(e.contains("one minimum trade at a time"), "{e}");
        // A 0.6%-sized entry keeps the budget, unresolved opens included.
        let normal = Sized {
            volume: 5,
            max_risk: 6.62,
            mode: "risk_pct",
            risk: 6.0,
        };
        assert!(admit(&normal, 0, Ok(0.0), 1002.44, 3.0).is_ok());
        let e = admit(&normal, 1, Ok(6.0), 1002.44, 1.0).unwrap_err();
        assert!(e.contains("exceed its budget"), "{e}");
        assert!(admit(&normal, 1, Err("unknown".into()), 1002.44, 3.0).is_err());
    }

    #[test]
    fn the_risk_at_a_stop_is_zero_once_the_stop_passes_the_entry() {
        // Long 5 cents (0.05 units) from 3611 with the stop 120 below: $6.00.
        assert!((risk_at_stop(Side::Buy, 3611.0, 3491.0, 5, 1.0) - 6.0).abs() < 1e-9);
        assert_eq!(risk_at_stop(Side::Buy, 3611.0, 3620.0, 5, 1.0), 0.0);
        // Short: the stop above the entry is the risk.
        assert!((risk_at_stop(Side::Sell, 100.0, 102.0, 100, 1.0) - 2.0).abs() < 1e-9);
        assert_eq!(risk_at_stop(Side::Sell, 100.0, 99.0, 100, 1.0), 0.0);
    }

    #[test]
    fn stops_round_to_the_symbols_digits() {
        assert_eq!(relative_stop(284.5712, 2), 28_457_000);
        assert_eq!(relative_stop(0.0128784, 5), 1_288);
        assert_eq!(relative_stop(0.000001, 2), 1_000, "at least one tick");
        assert_eq!(round_stop(1.234567, 4, Side::Buy), 1.2345);
        assert_eq!(round_stop(1.234567, 4, Side::Sell), 1.2346);
    }

    #[test]
    fn the_trail_only_moves_in_the_trades_favour() {
        // Long: best 110, ATR 2 → 104; above 100 moves, below never.
        assert_eq!(trail(Side::Buy, 110.0, 2.0, Some(100.0), 2), Some(104.0));
        assert_eq!(trail(Side::Buy, 110.0, 2.0, Some(105.0), 2), None);
        assert_eq!(trail(Side::Buy, 110.0, 2.0, Some(104.0), 2), None);
        // Short: best 90, ATR 2 → 96.
        assert_eq!(trail(Side::Sell, 90.0, 2.0, Some(100.0), 2), Some(96.0));
        assert_eq!(trail(Side::Sell, 90.0, 2.0, Some(95.0), 2), None);
    }

    fn day(i: i64) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i)
    }

    #[test]
    fn levels_follow_the_pre_registered_rule() {
        // A rising market: EMA(50) > EMA(100) → longs; the level is the
        // highest high of the last 50 completed days; ATR(20) = 40 (every
        // true range is high − low = 40).
        let bars: Vec<Bar> = (0..160)
            .map(|i| {
                let b = 2000.0 + 10.0 * i as f64;
                (day(i), b + 20.0, b - 20.0, b + 5.0)
            })
            .collect();
        let l = levels(&bars).unwrap();
        assert_eq!(l.side, Some(Side::Buy));
        assert_eq!(l.level, 2000.0 + 1590.0 + 20.0);
        assert!((l.atr - 40.0).abs() < 1e-9, "{}", l.atr);
        assert_eq!(l.last_completed, day(159));
        assert_eq!(triggered(&l, Some(3600.0), Some(3609.0)), None);
        assert_eq!(triggered(&l, Some(3600.0), Some(3610.0)), Some(Side::Buy));
        // A falling market: shorts on the bid at or below the lowest low.
        let down: Vec<Bar> = bars
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let x = 5000.0 - 10.0 * i as f64;
                (b.0, x + 20.0, x - 20.0, x - 5.0)
            })
            .collect();
        let l = levels(&down).unwrap();
        assert_eq!(l.side, Some(Side::Sell));
        assert_eq!(l.level, 5000.0 - 1590.0 - 20.0);
        assert_eq!(triggered(&l, Some(3390.0), Some(3391.0)), Some(Side::Sell));
        assert_eq!(triggered(&l, Some(3390.5), Some(3391.0)), None);
        // Too few completed days: no levels.
        assert!(levels(&bars[..99]).is_err());
    }

    #[test]
    fn the_seed_file_parses_and_refuses_duplicate_ids() {
        // The shipped seed: the owner's 25-10 "Live too, from now".
        let seed = parse_seed(SEED_FILE).unwrap();
        assert_eq!(seed.len(), 1);
        assert_eq!(seed[0].key, "trading.live_requires_edge_pass");
        assert_eq!(seed[0].value, json!(0));
        assert_eq!(seed[0].account_id, None);
        assert_eq!(seed[0].changed_by, "owner");
        assert!(seed[0].reason.contains("Live too, from now"));
        let dup = r#"[{"id":"a","key":"trading.risk_pct","account_id":1,"value":0.6,"changed_by":"x","reason":"y"},
                     {"id":"a","key":"trading.risk_pct","account_id":2,"value":0.6,"changed_by":"x","reason":"y"}]"#;
        assert!(parse_seed(dup).unwrap_err().contains("twice"));
    }

    #[test]
    fn a_conversion_rate_comes_from_the_brokers_own_pair() {
        let mk = |id, name: &str, bid, ask| {
            (
                id,
                Sym {
                    id,
                    name: name.into(),
                    limits: None,
                    quote: None,
                    levels: Err(String::new()),
                    bid: Some(bid),
                    ask: Some(ask),
                },
            )
        };
        let syms: HashMap<i64, Sym> = [
            mk(1, "USDJPY", 156.0, 156.02),
            mk(2, "EURUSD", 1.12, 1.1202),
        ]
        .into_iter()
        .collect();
        assert_eq!(quote_to_deposit("USD", "USD", &syms), Some(1.0));
        let jpy = quote_to_deposit("JPY", "USD", &syms).unwrap();
        assert!((jpy - 1.0 / 156.01).abs() < 1e-12);
        let eur = quote_to_deposit("EUR", "USD", &syms).unwrap();
        assert!((eur - 1.1201).abs() < 1e-12);
        assert_eq!(quote_to_deposit("GBP", "USD", &syms), None);
    }
}
