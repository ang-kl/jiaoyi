//! Strategy 3: intraday momentum on the noise band.
//!
//! **Refinement, 27-10-2026 (v3).** The 25 Oct registration entered on the
//! first tick beyond the band, with TP1 at 3R + C. The 27 Oct statements
//! printed the barrier rate of that geometry (27–34% on the demos, 0% on
//! ETH/metals/oils), and the measured median C was ~2.2R, so the barrier
//! was ~5R. A no-drift process cannot hit that often enough to pay for the
//! stops. This loop now trades the v3 registration:
//!
//! - Entry is a 30-minute mark decision on the mid, not the first tick.
//! - A symbol is skipped when the cost reserve C is at least R.
//! - `REQUIRED_RR` is 1.38. Owner, 27-10-2026: target near 1.4R.
//!
//! The rules are `docs/preregistration-2026-10-28-strategy3-v6.md`, embedded
//! and hashed: every decision, intent and trade records that hash and the
//! label `jiaoyi-s3`. The 25 Oct file remains the record of the rule that
//! produced the statement losses. It is not the rule this loop trades.
//!
//! **Repeat, 28-10-2026 (v6).** Owner: “Proceed to fix.” One open per symbol
//! per session. A cost skip is remembered for that session and not decided
//! again. The band is not refit. Grok 4.7 · № 17,682.
//!
//! **Trailing, 28-10-2026 (v5).** Owner: “change the forbid.” The 25 Oct
//! “no stop amends” sentence is retired. An owned position may trail its
//! stop to best − R (long) or best + R (short). The amend carries the
//! take-profit, so the target is not deleted. The stop never loosens.
//!
//! One loop per permitted account, demo and live:
//! - **Sessions.** Each symbol trades its own session (`jiaoyi_core::strategy3::sessions`).
//! - **Band.** At the session's first quote after 30 minutes, the M30 bars
//!   of the last sessions are read from the broker. They give the session's
//!   open and σ at each 30-minute mark. A session without a first bar with
//!   ticks is a holiday.
//! - **Entry.** Only at a 30-minute mark, within 5 seconds of it. The mid
//!   above the upper band buys; the mid below the lower band sells. No
//!   entry in the session's last 30 minutes, and at most one Strategy 3
//!   position per symbol. If C ≥ R the entry is skipped.
//! - **Exits** (bot-trade `risk_coverage_partial_v1`).
//!   - The stop is R from the fill.
//!   - The broker take-profit is TP2, or TP1 when the size cannot be split.
//!   - At TP1 the trader closes the planned part.
//!   - At the session end it closes what is left.
//! - **Size.** A demo account sizes `trading.risk_pct` of fresh equity, floored
//!   to the step, and skips below the minimum. A live account takes one
//!   minimum trade at a time, at most `trading.min_volume_max_risk_pct` of
//!   equity, and only while no position of this program is open on it.
//! - **Budget.** Strategy 3's open risk at its stops stays within
//!   `trading.s3_open_risk_pct` of equity.
//!
//! Every order goes through `jiaoyi_ctrader::trading::send`, the one gate.

pub mod preopen;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::strategy3::band::{self, Band, SigmaTable, M30};
use jiaoyi_core::strategy3::sessions::{self, Session, SessionRule};
use jiaoyi_core::strategy3::targets::{self, Dir, Plan};
use jiaoyi_core::strategy3::{cost_reserve, slippage_bps_per_side, CostReserve};
use jiaoyi_core::universe::AssetClass;
use jiaoyi_ctrader::trading::{
    self as gate, Command, EventRecord, Intent, IntentWrite, Ledger, Outcome, Policy, SendError,
    Side, VolumeLimits,
};
use jiaoyi_ctrader::{AccountSession, Connection, ReceivedExecution};
use jiaoyi_store::sqlx::{PgPool, Row};
use jiaoyi_store::strategy3 as s3s;
use jiaoyi_store::trading as ts;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::broadcast::error::RecvError;

use crate::config::Config;
use crate::edge::costs;
use crate::edge::h1job::QuoteAssets;
use crate::loader;
use crate::shutdown::Shutdown;
use crate::trading::{fresh_equity, relative_stop, risk_at_stop, PgLedger};

/// The strategy's name on every route and record.
pub const STRATEGY: &str = "Strategy 3: intraday momentum";
/// The label on every position Strategy 3 opens.
pub const LABEL: &str = "jiaoyi-s3";
/// The pre-registration, relative to the repository root.
/// v4 (28-10-2026): Kelly skip in front of size(). The 27 Oct file is the
/// record of the mark decision and the cost gate.
pub const PREREG_FILE: &str = "docs/preregistration-2026-10-28-strategy3-v6.md";
/// Its text, compiled in.
pub const PREREG_TEXT: &str =
    include_str!("../../../docs/preregistration-2026-10-28-strategy3-v6.md");
/// A trigger that was skipped or refused is taken again on that symbol only
/// after this long (self-derived: the gate reads the broker twice per try).
pub const ENTRY_RETRY: Duration = Duration::from_secs(60);
/// A session whose band could not be read (a broker error) is read again
/// after this long.
pub const PREPARE_RETRY: Duration = Duration::from_secs(300);
/// Calendar days of M30 bars read for the band (30 sessions and weekends).
pub const BAR_DAYS: i64 = 45;
/// The commission book is read again this often.
pub const COMMISSION_REFRESH: Duration = Duration::from_secs(3600);

/// SHA-256 of the embedded pre-registration, hex.
pub fn prereg_hash() -> String {
    Sha256::digest(PREREG_TEXT.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn dir_side(d: Dir) -> Side {
    match d {
        Dir::Buy => Side::Buy,
        Dir::Sell => Side::Sell,
    }
}

// ---------------------------------------------------------------- settings

/// Strategy 3's rules for one account, read fresh.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rules {
    pub version: i64,
    pub live: bool,
    pub risk_pct: f64,
    pub cap_factor: f64,
    pub min_volume_max_risk_pct: f64,
    pub open_risk_pct: f64,
    pub reconcile_secs: u64,
    pub order_wait_secs: u64,
    pub retry_secs: u64,
    /// The owner's pause of Strategy 3 on this account (`trading.s3_paused`):
    /// no new entry; open positions keep their exits. Only the owner sets it.
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
    pub paused: bool,
}

pub async fn rules(pool: &PgPool, account_id: i64, env: Environment) -> anyhow::Result<Rules> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let i = |k: &str| s.integer(k).map_err(anyhow::Error::msg);
    let n = |k: &str| s.number(k).map_err(anyhow::Error::msg);
    Ok(Rules {
        version: s.version,
        live: env == Environment::Live,
        risk_pct: n("trading.risk_pct")?,
        cap_factor: n("trading.cap_factor")?,
        min_volume_max_risk_pct: n("trading.min_volume_max_risk_pct")?,
        open_risk_pct: n("trading.s3_open_risk_pct")?,
        reconcile_secs: i("trading.reconcile_secs")?.max(1) as u64,
        order_wait_secs: i("trading.order_wait_secs")?.max(1) as u64,
        retry_secs: i("trading.retry_secs")?.max(1) as u64,
        paused: i(crate::limits::S3.pause_key)? == 1,
    })
}

/// A symbol's distance to its Strategy 3 trigger at `now`: the band at that
/// instant, the last bid and ask, and the distance from the nearer side's
/// quote to its edge (the ask to the upper, the bid to the lower) in
/// half-widths (open × σ), negative once through it. `null`, with the
/// reason, outside the entry window, after a session cost veto, or without
/// a prepared session.
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
pub fn trigger_distance(sym: &Sym, now: DateTime<Utc>) -> Value {
    let none = |why: &str| json!({"at": now, "bid": sym.bid, "ask": sym.ask, "distance_half_widths": null, "reason": why});
    let Some(rule) = sym.rule else {
        return none("no session rule for this symbol");
    };
    let Some(s) = sessions::session_at(&rule, now) else {
        return none("outside the symbol's session");
    };
    if let Err(why) = band::entry_window(&s, now) {
        return none(&why);
    }
    // Restoring the durable veto returns before preparation. Its active
    // session blocker is known even when this connection has no Day cache.
    if cost_skip_stands(sym.cost_skip_session, s.date) {
        let mut blocked = none("entry blocked for this session by the recorded cost veto (C >= R)");
        blocked["session_date"] = json!(s.date);
        blocked["reason_code"] = json!("session_cost_veto");
        return blocked;
    }
    let day = match &sym.day {
        Some(Ok(d)) if d.session.date == s.date => d,
        _ => return none("today's session is not prepared yet"),
    };
    let open = match &day.open {
        Ok(o) => *o,
        Err(why) => return none(why),
    };
    let b = match band::band_at(&day.table, &s, open, now) {
        Ok(b) => b,
        Err(why) => return none(&why),
    };
    let up = sym.ask.map(|a| (b.upper - a) / b.half_width);
    let down = sym.bid.map(|x| (x - b.lower) / b.half_width);
    let (edge, distance) = match (up, down) {
        (Some(u), Some(d)) if u <= d => ("upper", Some(u)),
        (Some(_), Some(d)) => ("lower", Some(d)),
        (Some(u), None) => ("upper", Some(u)),
        (None, Some(d)) => ("lower", Some(d)),
        (None, None) => ("none", None),
    };
    json!({
        "at": now,
        "session_date": s.date,
        "open": open,
        "sigma": b.sigma,
        "upper": b.upper,
        "lower": b.lower,
        "bid": sym.bid,
        "ask": sym.ask,
        "nearer_edge": edge,
        "distance_half_widths": distance.filter(|d| d.is_finite()),
        "reason": if distance.is_none() { Some("no quote yet") } else { None },
    })
}

// ---------------------------------------------------------------- pure parts

/// The Chandelier stop: best − R for a long, best + R for a short.
/// Returns `None` when the inputs are not a price or R is not positive.
/// This is an exit rule. It is not MAE and it is not MFE.
pub fn chandelier_stop(side: Side, best: f64, r: f64) -> Option<f64> {
    if !best.is_finite() || best <= 0.0 || !r.is_finite() || r <= 0.0 {
        return None;
    }
    let stop = match side {
        Side::Buy => best - r,
        Side::Sell => best + r,
    };
    (stop.is_finite() && stop > 0.0).then_some(stop)
}

/// True when this symbol already recorded an open in the session.
/// A later signal is not a second entry. Grok 4.7 · № 17,682.
pub fn session_already_entered(opens: i64) -> bool {
    opens >= 1
}

/// True when a cost skip was already recorded for this session date.
/// A later mark does not decide the cost again. Grok 4.7 · № 17,682.
pub fn cost_skip_stands(remembered: Option<chrono::NaiveDate>, session: chrono::NaiveDate) -> bool {
    remembered == Some(session)
}

/// A finished trade in original R. Profit is positive. A stop from the
/// entry is -1. `banks_a_win` is true only when the fill is past the entry.
/// The chandelier hangs one R behind the best price, so a favourable move
/// that never exceeds one R still closes at a loss. The 28 Oct JPN225
/// shorts did that: entry 64061.1, R 327.5, stop-out 64155.3, about -0.29R.
/// That cuts a loss. It does not add a win to the 15 of 20.
/// Grok 4.7 · № 17,659. This does not move a stop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloseResult {
    pub result_r: f64,
    pub banks_a_win: bool,
}

pub fn close_result(side: Side, entry: f64, r: f64, fill: f64) -> Option<CloseResult> {
    if !entry.is_finite()
        || entry <= 0.0
        || !r.is_finite()
        || r <= 0.0
        || !fill.is_finite()
        || fill <= 0.0
    {
        return None;
    }
    let result_r = match side {
        Side::Buy => (fill - entry) / r,
        Side::Sell => (entry - fill) / r,
    };
    result_r.is_finite().then_some(CloseResult {
        result_r,
        banks_a_win: result_r > 0.0,
    })
}

/// Maximum adverse and maximum favourable excursion, in price units from the
/// entry. These are measurements. They do not move a stop and they do not
/// close a position.
pub fn update_excursions(side: Side, entry: f64, price: f64, mae: f64, mfe: f64) -> (f64, f64) {
    if !entry.is_finite() || !price.is_finite() {
        return (mae, mfe);
    }
    let adverse = match side {
        Side::Buy => (entry - price).max(0.0),
        Side::Sell => (price - entry).max(0.0),
    };
    let favourable = match side {
        Side::Buy => (price - entry).max(0.0),
        Side::Sell => (entry - price).max(0.0),
    };
    (mae.max(adverse), mfe.max(favourable))
}

/// Attach a Grok reading only when it can block a send or unblock a close.
/// A measurement that nothing acts on is ignored. The side, stop, target and
/// volume are not changed here. Grok 4.7 · № 17,615 28-Oct (ordered: owner
/// "time is money"): do not store a reading that fills a slot.
pub fn attach_sentiment(detail: &mut serde_json::Value, reading: &serde_json::Value) {
    let effect = reading.get("effect").and_then(|v| v.as_str());
    if matches!(effect, Some("block_a_send") | Some("unblock_a_close")) {
        detail["grok"] = reading.clone();
    }
}

/// The opposite side. The reverse strategy is this function and a separate
/// label. It is not the primary signal and it does not share the primary
/// hash's evidence.
pub fn reverse_side(side: Dir) -> Dir {
    match side {
        Dir::Buy => Dir::Sell,
        Dir::Sell => Dir::Buy,
    }
}

/// The entry signal of a quote at `now`.
///
/// v3 (27-10-2026): a decision only at a 30-minute mark, on the mid. A quote
/// that is not at a mark is `Ok(None)` — not an error, and not an entry.
/// The 25 Oct tick rule (`band::triggered`) is not called here.
pub fn signal(
    table: &SigmaTable,
    session: &Session,
    open: f64,
    now: DateTime<Utc>,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Result<Option<(Dir, Band)>, String> {
    band::entry_window(session, now)?;
    let Some(mark) = band::decision_mark(session, now) else {
        return Ok(None);
    };
    let b = band::band_at(table, session, open, mark)?;
    Ok(band::mark_decision(&b, bid, ask).map(|d| (d, b)))
}

/// How the volume was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Sized {
    pub volume: i64,
    /// The most the gate lets this open risk.
    pub max_risk: f64,
    pub mode: &'static str,
    pub risk: f64,
}

/// Size an open: demo `risk_pct` of equity floored to the step (below the
/// minimum: skipped); live exactly the broker minimum, within
/// `min_volume_max_risk_pct` of equity, and only with no other position of
/// this program open on the account.
pub fn size(
    r: &Rules,
    equity: f64,
    relative_stop: i64,
    quote_to_deposit: f64,
    limits: &VolumeLimits,
    program_positions_open: usize,
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
    if r.live {
        if program_positions_open > 0 {
            return Err(format!(
                "live: one minimum trade at a time, and {program_positions_open} position(s) of this program are open on the account"
            ));
        }
        let risk = gate::risk_of(limits.min, relative_stop, quote_to_deposit);
        let ceiling = equity * r.min_volume_max_risk_pct / 100.0;
        if risk > ceiling {
            return Err(format!(
                "live: the broker minimum {} would risk {risk:.2}, above {}% of equity ({ceiling:.2})",
                limits.min, r.min_volume_max_risk_pct
            ));
        }
        return Ok(Sized {
            volume: limits.min,
            max_risk: ceiling,
            mode: "live_broker_minimum",
            risk,
        });
    }
    let per_cent = gate::risk_of(1, relative_stop, quote_to_deposit);
    let allowed = equity * r.risk_pct / 100.0;
    let raw = allowed / per_cent;
    let mut volume = (raw / limits.step as f64).floor() as i64 * limits.step;
    if let Some(max) = limits.max {
        volume = volume.min(max - max % limits.step);
    }
    if volume < limits.min {
        return Err(format!(
            "{}% of equity sizes {raw:.2} cents, below the broker minimum {}: skipped, never rounded up",
            r.risk_pct, limits.min
        ));
    }
    Ok(Sized {
        volume,
        max_risk: r.cap_factor * allowed,
        mode: "risk_pct",
        risk: gate::risk_of(volume, relative_stop, quote_to_deposit),
    })
}

/// A price distance in whole ticks of `digits`, as 1/100000 units.
pub fn distance_e5(distance: f64, digits: i32) -> i64 {
    let ticks = (distance.abs() * 10f64.powi(digits)).round() as i64;
    ticks * 10_i64.pow((5 - digits.clamp(0, 5)) as u32)
}

/// Everything an entry decides, before sizing: R and the cost reserve.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entry {
    pub side: Dir,
    pub band: Band,
    /// The triggering quote side: the ask for a buy, the bid for a sell.
    pub entry_price: f64,
    pub relative_stop: i64,
    pub r: f64,
    pub cost: CostReserve,
}

/// The order of an entry once sized: the plan and its relative brackets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Order {
    pub plan: Plan,
    pub relative_take_profit: i64,
    /// TP1's distance from the fill, in price units.
    pub tp1_distance: f64,
    /// TP2's distance from the fill.
    pub tp2_distance: f64,
}

/// Plan the exits for `volume` (bot-trade's arithmetic). `Err` is its reason.
pub fn plan_order(e: &Entry, limits: &VolumeLimits, volume: i64) -> Result<Order, String> {
    let r = e.relative_stop as f64 / 100_000.0;
    let stop = match e.side {
        Dir::Buy => e.entry_price - r,
        Dir::Sell => e.entry_price + r,
    };
    let plan = targets::plan(&targets::Input {
        side: e.side,
        entry: e.entry_price,
        original_stop: stop,
        required_rr: targets::REQUIRED_RR,
        cost_reserve: e.cost.total,
        digits: limits.digits,
        volume,
        min_volume: limits.min,
        step_volume: limits.step,
    })
    .map_err(|why| format!("bot-trade {}: {why}", targets::POLICY))?;
    let tp1 = (plan.trigger - e.entry_price).abs();
    let tp2 = (plan.runner_target - e.entry_price).abs();
    let broker = (plan.broker_target - e.entry_price).abs();
    Ok(Order {
        plan,
        relative_take_profit: distance_e5(broker, limits.digits),
        tp1_distance: tp1,
        tp2_distance: tp2,
    })
}

/// What an entry planned its exits with: the side, the decision price, R and
/// the cost reserve C, read back from its recorded order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanBasis {
    pub side: Dir,
    pub entry: f64,
    pub r: f64,
    pub cost: f64,
    /// Frozen opening-plan distance, including its original reward policy
    /// and outward rounding. Never replan an old position with today's Q.
    pub tp1_distance: f64,
}

/// The plan basis in a recorded order's detail (`entry`), if complete.
pub fn plan_basis(detail: &Value) -> Option<PlanBasis> {
    let e = &detail["entry"];
    let side = match e["side"].as_str()? {
        "buy" => Dir::Buy,
        "sell" => Dir::Sell,
        _ => return None,
    };
    Some(PlanBasis {
        side,
        entry: e["entry_price"].as_f64()?,
        r: e["r"].as_f64()?,
        cost: e["cost"]["total"].as_f64()?,
        tp1_distance: detail["order"]["tp1_distance"].as_f64()?,
    })
}

/// TP1's close for the volume the broker holds, from the opening plan's
/// frozen R, C and actual rounded TP1 distance. Today's reward multiple
/// cannot change an older position's allocation. An unsplittable volume
/// closes whole at TP1, as in the original coverage policy.
pub fn tp1_close_volume(b: &PlanBasis, limits: &VolumeLimits, volume: i64) -> i64 {
    if !b.r.is_finite()
        || b.r <= 0.0
        || !b.cost.is_finite()
        || b.cost < 0.0
        || !b.tp1_distance.is_finite()
        || b.tp1_distance <= b.cost
        || limits.min <= 0
        || limits.step <= 0
        || volume < limits.min
        || volume % limits.step != 0
    {
        return volume;
    }
    // The original coverage identity, using the recorded distance instead
    // of the global required_rr. This works for both 3R and 1.38R plans.
    let fraction = (b.r + b.cost) / (b.tp1_distance + b.r);
    let steps =
        ((volume as f64 * fraction).max(limits.min as f64) / limits.step as f64).ceil() as i64;
    let close = steps.checked_mul(limits.step).unwrap_or(volume);
    if close < volume && volume - close >= limits.min {
        close
    } else {
        volume
    }
}

/// Whether a quote reaches a long's (bid) or a short's (ask) TP1, compared
/// in whole ticks of `digits`.
pub fn tp1_reached(
    side: Side,
    fill: f64,
    tp1_distance: f64,
    digits: i32,
    bid: Option<f64>,
    ask: Option<f64>,
) -> bool {
    let f = 10f64.powi(digits.clamp(0, 10));
    let ticks = |p: f64| (p * f).round() as i64;
    let target = match side {
        Side::Buy => ticks(fill) + ticks(tp1_distance),
        Side::Sell => ticks(fill) - ticks(tp1_distance),
    };
    match side {
        Side::Buy => bid.is_some_and(|b| ticks(b) >= target),
        Side::Sell => ask.is_some_and(|a| ticks(a) <= target),
    }
}

/// Canonical broker price for the registered best ± original R trail.
/// Work in whole symbol ticks: floating subtraction/addition alone can send
/// e.g. 64134.299999999996 where the broker permits only one decimal.
/// Unknown precision or genuinely off-grid evidence keeps the existing stop.
fn trail_price(side: Side, best: f64, r: f64, current: f64, digits: i32) -> Option<f64> {
    let factor = 10f64.powi(digits);
    let grid = |price: f64| {
        let ticks = targets::price_ticks(price, digits)?;
        // Same float-residue convention as the original target planner.
        (price > 0.0 && (price * factor - ticks as f64).abs() < 1e-6).then_some(ticks)
    };
    let (best, r, current) = (grid(best)?, grid(r)?, grid(current)?);
    if r <= 0 {
        return None;
    }
    let stop = match side {
        Side::Buy => best.checked_sub(r)?,
        Side::Sell => best.checked_add(r)?,
    };
    let improves = match side {
        Side::Buy => stop > current,
        Side::Sell => stop < current,
    };
    (stop > 0 && improves).then_some(stop as f64 / factor)
}

/// The open-risk budget: Strategy 3's open risk at its stops plus a new
/// entry's must stay within `pct` of equity.
pub fn within_budget(open_risk: f64, new_risk: f64, equity: f64, pct: f64) -> Result<(), String> {
    let budget = equity * pct / 100.0;
    if open_risk + new_risk > budget * (1.0 + 1e-9) {
        Err(format!(
            "Strategy 3's open risk {open_risk:.2} plus this entry's {new_risk:.2} would exceed its budget {budget:.2} ({pct}% of equity): skipped"
        ))
    } else {
        Ok(())
    }
}

/// The deposit currency per unit of `quote`, from the live mid of a pair
/// joining them (`QUOTEDEPOSIT` or `DEPOSITQUOTE`).
pub fn quote_to_deposit(quote: &str, deposit: &str, mids: &HashMap<String, f64>) -> Option<f64> {
    if quote == deposit {
        return Some(1.0);
    }
    let mid = |n: &str| mids.get(n).copied().filter(|m| m.is_finite() && *m > 0.0);
    mid(&format!("{quote}{deposit}")).or_else(|| mid(&format!("{deposit}{quote}")).map(|m| 1.0 / m))
}

// ---------------------------------------------------------------- ledger

/// The gate's ledger for Strategy 3: the intent in `trading_intents`, then
/// the decided order (hash, label, quote, band, plan) in `strategy3_orders`,
/// before the frame. If the second write fails, nothing is sent.
pub struct S3Ledger<'a> {
    pub inner: PgLedger<'a>,
    pub order: s3s::OrderRow,
    /// When the intent was recorded: just before the frame is written.
    pub recorded_at: Mutex<Option<DateTime<Utc>>>,
}

impl Ledger for S3Ledger<'_> {
    fn admission_namespace(&self) -> String {
        self.inner.admission_namespace()
    }
    async fn margin_floor_pct(&self, account_id: i64) -> Result<f64, String> {
        self.inner.margin_floor_pct(account_id).await
    }
    async fn margin_has_unresolved_open(&self, account_id: i64) -> Result<bool, String> {
        self.inner.margin_has_unresolved_open(account_id).await
    }
    async fn record_margin(
        &self,
        account_id: i64,
        symbol_id: i64,
        receipt: &jiaoyi_ctrader::margin::Receipt,
    ) -> Result<(), String> {
        self.inner
            .record_margin(account_id, symbol_id, receipt)
            .await
    }

    async fn record_intent(&self, i: &Intent) -> Result<IntentWrite, String> {
        let w = self.inner.record_intent(i).await?;
        if w == IntentWrite::Recorded {
            s3s::record_order(self.inner.pool, &self.order)
                .await
                .map_err(|e| format!("the Strategy 3 order record failed: {e}"))?;
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

// ---------------------------------------------------------------- state

/// One symbol's session preparation: the open, σ and the median spread.
#[derive(Debug, Clone)]
pub struct Day {
    pub session: Session,
    pub open: Result<f64, String>,
    pub table: SigmaTable,
    pub spread: Result<(f64, i64), String>,
    pub prepared_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments)]
async fn load_day(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    rule: SessionRule,
    s: Session,
    now: DateTime<Utc>,
    prepared_table: Option<SigmaTable>,
) -> Result<Day, (String, DateTime<Utc>)> {
    let from = if prepared_table.is_some() {
        s.open
    } else {
        s.open - chrono::Duration::days(BAR_DAYS)
    };
    match session.m30_bars(symbol_id, from, now).await {
        Err(e) => Err((format!("M30 bars unreadable: {e}"), now)),
        Ok(raw) => {
            let mut bars = Vec::new();
            let mut first_volume = None;
            for tb in &raw {
                let Ok(b) = jiaoyi_ctrader::decode_bar(
                    tb,
                    jiaoyi_ctrader::messages::ProtoOaTrendbarPeriod::M30,
                ) else {
                    continue;
                };
                if now < s.open && b.open_time + chrono::Duration::minutes(30) > now {
                    continue;
                }
                let (Some(o), Some(c)) = (b.open.to_f64(), b.close.to_f64()) else {
                    continue;
                };
                if b.open_time == s.open {
                    first_volume = Some(b.tick_volume);
                }
                bars.push(M30 {
                    open_time: b.open_time,
                    open: o,
                    close: c,
                });
            }
            let table = prepared_table.unwrap_or_else(|| band::sigma_table(&rule, &s, &bars));
            let open = match (bars.iter().find(|b| b.open_time == s.open), first_volume) {
                (Some(b), Some(v)) if v > 0 => Ok(b.open),
                _ => Err(format!(
                    "no M30 bar with ticks at the session open {}: a holiday, skipped",
                    s.open
                )),
            };
            let spread = match s3s::median_spread(pool, account_id, symbol_id).await {
                Ok(Some(x)) => Ok(x),
                Ok(None) => Err("no recorded two-sided quote in 7 days: no spread".into()),
                Err(e) => Err(format!("recorded quotes unreadable: {e}")),
            };
            Ok(Day {
                session: s,
                open,
                table,
                spread,
                prepared_at: now,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sym {
    pub id: i64,
    pub name: String,
    pub class: AssetClass,
    pub rule: Option<SessionRule>,
    pub limits: Option<VolumeLimits>,
    pub quote: Option<String>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_at: Option<DateTime<Utc>>,
    pub ask_at: Option<DateTime<Utc>>,
    pub day: Option<Result<Day, (String, DateTime<Utc>)>>,
    pub preopen: Option<preopen::Prepared>,
    /// The last entry attempt, so a refused trigger waits [`ENTRY_RETRY`].
    pub last_try: Option<DateTime<Utc>>,
    pub last_decision_mark: Option<DateTime<Utc>>,
    /// Session date of a recorded open. Later marks do not send another.
    pub entered_session: Option<chrono::NaiveDate>,
    /// Session date of a cost skip. Later marks do not decide it again.
    pub cost_skip_session: Option<chrono::NaiveDate>,
}

/// A Strategy 3 position as reconciled, with its plan when recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Own {
    pub position_id: i64,
    pub symbol_id: i64,
    pub side: Side,
    pub volume: i64,
    pub entry: Option<f64>,
    pub stop: Option<f64>,
    pub take_profit: Option<f64>,
    /// From the recorded plan.
    pub tp1_distance: Option<f64>,
    pub close_volume: Option<i64>,
    pub partial: bool,
    /// What the entry planned with, so TP1's close is planned again for the
    /// volume the broker holds (a partial fill: Codex P2 on #15).
    pub plan_basis: Option<PlanBasis>,
    pub session_date: Option<chrono::NaiveDate>,
    pub entry_key: Option<String>,
    /// A TP1 close filled, partly filled or unanswered: never sent again.
    pub tp1_sent: bool,
    /// After a TP1 close refused or rejected with nothing filled, when the
    /// next may go (Codex P2 on #15).
    pub tp1_next_try: Option<DateTime<Utc>>,
    /// Session-end closes tried so far (each its own intent key), and when
    /// the next may go (Codex P1 on #15: a refused or rejected close is
    /// retried, never given up).
    pub end_attempts: i64,
    pub end_next_try: Option<DateTime<Utc>>,
    /// Best executable price since the position was seen (highest bid for a
    /// long, lowest ask for a short). The Chandelier hangs from this.
    pub best: Option<f64>,
    /// Maximum adverse excursion from the entry, in price units. A measurement.
    /// The Chandelier does not read it.
    pub mae: f64,
    /// Maximum favourable excursion from the entry, in price units. A measurement.
    /// The Chandelier does not read it.
    pub mfe: f64,
    /// When the last stop amend was sent, so a trail is not sent more often
    /// than once per [`TRAIL_AMEND_SECS`].
    pub last_amend: Option<DateTime<Utc>>,
}

/// After a session-end or TP1 close the gate refused or the broker rejected,
/// the next try waits this long (self-derived).
pub const END_RETRY: Duration = Duration::from_secs(10);

/// A Strategy 3 trail is not sent more often than this per position.
pub const TRAIL_AMEND_SECS: i64 = 30;

// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design"):
// one rule for both strategies' fills.
use crate::fills::partly_filled;

/// An open sent but not answered (Unconfirmed): its risk is held in the
/// budget, and on a live account it counts as an open position, until a
/// reconcile finds it.
#[derive(Debug, Clone, PartialEq)]
struct Pending {
    entry_key: String,
    symbol_id: i64,
    risk: f64,
    at: DateTime<Utc>,
}

/// Records Strategy 3's decisions (kinds `s3_*`) when they change, with the
/// pre-registration hash and the label.
pub struct Decisions<'a> {
    pool: &'a PgPool,
    account_id: i64,
    last: HashMap<String, String>,
    version: Option<i64>,
    hash: String,
}

impl<'a> Decisions<'a> {
    pub fn new(pool: &'a PgPool, account_id: i64) -> Decisions<'a> {
        Decisions {
            pool,
            account_id,
            last: HashMap::new(),
            version: None,
            hash: prereg_hash(),
        }
    }

    pub async fn note(
        &mut self,
        symbol: Option<&str>,
        kind: &str,
        reason: &str,
        mut detail: Value,
    ) {
        let k = format!("{}:{kind}", symbol.unwrap_or("-"));
        if self.last.get(&k).map(String::as_str) == Some(reason) {
            return;
        }
        self.last.insert(k, reason.to_string());
        detail["strategy"] = json!(STRATEGY);
        detail["label"] = json!(LABEL);
        detail["prereg_hash"] = json!(self.hash);
        tracing::info!(
            account = self.account_id,
            symbol = ?symbol,
            kind,
            reason,
            prereg_hash = %self.hash,
            "strategy 3 decision"
        );
        if let Err(e) = ts::record_decision(
            self.pool,
            self.account_id,
            symbol,
            &format!("s3_{kind}"),
            reason,
            &detail,
            self.version,
        )
        .await
        {
            tracing::error!(error = %e, "strategy 3 decision not recorded");
        }
    }
}

/// The trader of one account on an authorised session. Its methods take the
/// time, so the tests drive it quote by quote.
pub struct Trader<'a, 'c> {
    pub pool: &'a PgPool,
    pub allowlist: &'a Allowlist,
    pub env: Environment,
    pub session: &'a AccountSession<'c>,
    pub syms: HashMap<i64, Sym>,
    pub own: HashMap<i64, Own>,
    pub deposit: Option<String>,
    pub commission: Result<costs::CommissionBook, String>,
    commission_read: Option<std::time::Instant>,
    pending: Vec<Pending>,
    /// A broker funding rejection applies to this account, across symbols.
    /// Only entry retries wait; existing positions keep every exit.
    funding_retry_at: Option<DateTime<Utc>>,
    /// Program positions on the account at the last reconcile (any strategy).
    program_positions: usize,
    pub d: Decisions<'a>,
    hash: String,
    protect_during_reads: bool,
    read_shutdown: Option<Shutdown>,
    event_offset: chrono::Duration,
    protected_through: HashMap<i64, DateTime<Utc>>,
    /// The loops' liveness, shared with `/health` (each reconcile run beats
    /// it). A trader driven on its own keeps its own map.
    // Claude · № 17,074 25-Oct (ordered: owner "merge when green and deploy";
    // Strategy 3's heartbeat, as #16 gave Strategy 1)
    pub live: crate::trading::LivenessMap,
}

impl<'a, 'c> Trader<'a, 'c> {
    /// Read the account's symbols, deposit currency and commission book.
    pub async fn new(
        pool: &'a PgPool,
        allowlist: &'a Allowlist,
        env: Environment,
        session: &'a AccountSession<'c>,
    ) -> anyhow::Result<Trader<'a, 'c>> {
        let account_id = session.account().id();
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
        let snap = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
        let universe = snap
            .universe("universe.symbols")
            .map_err(anyhow::Error::msg)?;
        let resolutions = jiaoyi_store::latest_resolutions(pool, account_id).await?;
        let mut syms = HashMap::new();
        for (name, class) in universe {
            let Some(id) = resolutions
                .iter()
                .find(|r| r.name == name)
                .and_then(|r| r.broker_symbol_id)
            else {
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
                    name: name.clone(),
                    class,
                    rule: sessions::rule_for(&name, class),
                    limits,
                    quote: quotes
                        .as_ref()
                        .and_then(|q| q.quote_of(id).ok().map(|x| x.0)),
                    bid: None,
                    ask: None,
                    bid_at: None,
                    ask_at: None,
                    day: None,
                    preopen: None,
                    last_try: None,
                    last_decision_mark: None,
                    entered_session: None,
                    cost_skip_session: None,
                },
            );
        }
        let mut t = Trader {
            pool,
            allowlist,
            env,
            session,
            syms,
            own: HashMap::new(),
            deposit,
            commission: Err("not read yet".into()),
            commission_read: None,
            pending: Vec::new(),
            funding_retry_at: None,
            program_positions: 0,
            d: Decisions::new(pool, account_id),
            hash: prereg_hash(),
            protect_during_reads: false,
            read_shutdown: None,
            event_offset: chrono::Duration::zero(),
            protected_through: HashMap::new(),
            live: Default::default(),
        };
        t.read_commission().await;
        if preopen::restore(pool, account_id, &mut t.syms, Utc::now())
            .await
            .is_err()
        {
            tracing::warn!(
                account = account_id,
                "pre-open cache unavailable at reconnect; entry-time preparation remains available"
            );
        }
        Ok(t)
    }

    fn account_id(&self) -> i64 {
        self.session.account().id()
    }

    /// Await non-execution data while the same mutable trader continues
    /// registered exits. This is one writer, never a concurrent controller.
    async fn protect_while_reading<F: std::future::Future>(
        &mut self,
        read: F,
    ) -> anyhow::Result<F::Output> {
        if !self.protect_during_reads {
            return Ok(read.await);
        }
        let mut quotes = self.session.connection().spot_events();
        let mut executions = self.session.connection().executions();
        let mut stop = self
            .read_shutdown
            .clone()
            .expect("runtime protection has a shutdown receiver");
        let mut exits = tokio::time::interval(crate::fast_monitor::INTERVAL);
        exits.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut clock = tokio::time::interval(Duration::from_secs(1));
        clock.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        tokio::pin!(read);
        let progress = crate::fast_monitor::WorkGuard::start(
            &self.live,
            self.account_id(),
            "historical_read_with_exit_pump",
        );
        loop {
            tokio::select! {
                _ = stop.wait() => anyhow::bail!("shutdown during historical preparation"),
                result = &mut read => { progress.complete(); return Ok(result); },
                _ = exits.tick() => self.check_exits(Utc::now()+self.event_offset).await?,
                _ = clock.tick() => self.session_end(Utc::now()+self.event_offset).await?,
                event = quotes.recv() => match event {
                    Ok(q) if q.event.ctid_trader_account_id == self.account_id() => {
                        let id=q.event.symbol_id;
                        // Flat symbols are left for the outer entry path,
                        // preserving their first eligible quote and order.
                        if self.own.values().any(|p| p.symbol_id==id) {
                            let at=q.received_at+self.event_offset;
                            if let Some(sym)=self.syms.get_mut(&id) {
                                if let Some(b)=q.event.bid {sym.bid=Some(b as f64/100_000.0);sym.bid_at=Some(at);}
                                if let Some(a)=q.event.ask {sym.ask=Some(a as f64/100_000.0);sym.ask_at=Some(at);}
                            }
                            self.protected_through.insert(id,at);
                            self.check_exits(Utc::now()+self.event_offset).await?;
                            crate::fast_monitor::quote_processed(&self.live,self.account_id(),q.received_at);
                        }
                    },
                    Ok(_) | Err(RecvError::Lagged(_)) => {},
                    Err(RecvError::Closed) => anyhow::bail!("quote stream closed during historical preparation"),
                },
                event=executions.recv()=>match event {
                    Ok(e)=>{
                        self.server_event(&e).await;
                        crate::fast_monitor::execution_processed(&self.live,self.account_id(),e.received_at);
                    },
                    Err(RecvError::Lagged(_))=>{},
                    Err(RecvError::Closed)=>anyhow::bail!("execution stream closed during historical preparation"),
                },
            }
        }
    }

    /// An exit-only pass over owned positions. Cached executable sides must
    /// be fresh; a timer cannot manufacture a quote or enter a new trade.
    pub async fn check_exits(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let ids: Vec<i64> = self.own.values().map(|p| p.symbol_id).collect();
        for id in ids {
            let Some(sym) = self.syms.get(&id) else {
                continue;
            };
            let bid = sym.bid.filter(|b| {
                b.is_finite() && *b > 0.0 && crate::fast_monitor::quote_is_fresh(sym.bid_at, now)
            });
            let ask = sym.ask.filter(|a| {
                a.is_finite() && *a > 0.0 && crate::fast_monitor::quote_is_fresh(sym.ask_at, now)
            });
            if bid.zip(ask).is_some_and(|(b, a)| b > a) {
                continue;
            }
            self.exits_for_symbol(id, bid, ask, now).await?;
        }
        crate::fast_monitor::exit_check_completed(
            &self.live,
            self.account_id(),
            self.own.len(),
            self.pending.len(),
        );
        Ok(())
    }

    async fn read_commission(&mut self) {
        self.commission =
            match jiaoyi_store::research::commission_per_unit(self.pool, self.account_id()).await {
                Ok(samples) => costs::commission_book(&samples),
                Err(e) => Err(format!("deals unreadable: {e}")),
            };
        self.commission_read = Some(std::time::Instant::now());
    }

    fn mids(&self) -> HashMap<String, f64> {
        self.syms
            .values()
            .filter_map(|s| Some((s.name.clone(), (s.bid? + s.ask?) / 2.0)))
            .collect()
    }

    fn rate(&self, symbol_id: i64) -> Option<f64> {
        let s = self.syms.get(&symbol_id)?;
        quote_to_deposit(s.quote.as_deref()?, self.deposit.as_deref()?, &self.mids())
    }

    /// Reconcile with the broker: Strategy 3's own positions, with their
    /// recorded plans, and the count of this program's positions.
    pub async fn reconcile(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let account_id = self.account_id();
        let res = self.session.reconcile().await?;
        let mut plans = s3s::open_plans(self.pool, account_id).await?;
        // Strategy 3's unresolved opens: accepted only, or no answer (Codex P1
        // on #14). Never resent.
        let mut unresolved: Vec<ts::UnresolvedIntent> =
            ts::open_intents_unresolved(self.pool, account_id)
                .await?
                .into_iter()
                .filter(|u| u.entry_key.starts_with("s3:"))
                .collect();
        let mut seen = Vec::new();
        self.program_positions = 0;
        for p in &res.position {
            let label = p.trade_data.label.as_deref();
            if label.is_some_and(|l| gate::PROGRAM_LABELS.contains(&l)) {
                self.program_positions += 1;
            }
            if label != Some(LABEL) {
                continue;
            }
            let mut owned = ts::owns_position(self.pool, account_id, p.position_id).await?;
            if !owned {
                // In full only at the requested volume; a part keeps the
                // intent unresolved, its remainder reserved below until it
                // fills, ends or is absent after the grace (Codex P1 on #15,
                // as #16 does for Strategy 1).
                // Claude · № 17,060 25-Oct (ordered: owner "merge when green
                // and deploy"; Codex round 3 on #15)
                if let Some(i) = unresolved.iter().position(|u| {
                    u.symbol_id == Some(p.trade_data.symbol_id) && u.position_id.is_none()
                }) {
                    let part = unresolved[i]
                        .volume
                        .is_some_and(|v| p.trade_data.volume < v);
                    let key = unresolved[i].entry_key.clone();
                    if part {
                        unresolved[i].position_id = Some(p.position_id);
                    } else {
                        unresolved.remove(i);
                    }
                    self.adopt(&key, p, part).await?;
                    // Claude · № 17,020 25-Oct (Codex P1 on #15): the adopted
                    // position's plan is read after its fill row exists.
                    plans = s3s::open_plans(self.pool, account_id).await?;
                    owned = true;
                }
            }
            if !owned {
                continue;
            }
            let Some(side) = Side::from_proto(p.trade_data.trade_side) else {
                continue;
            };
            seen.push(p.position_id);
            let plan = plans.iter().rev().find(|x| x.position_id == p.position_id);
            let tp1_sent = s3s::tp1_done(self.pool, account_id, p.position_id).await?;
            let end_attempts =
                s3s::orders_for_position(self.pool, account_id, p.position_id, "session_end")
                    .await?;
            let e = self.own.entry(p.position_id).or_insert_with(|| Own {
                position_id: p.position_id,
                symbol_id: p.trade_data.symbol_id,
                side,
                volume: p.trade_data.volume,
                entry: p.price,
                stop: p.stop_loss,
                take_profit: p.take_profit,
                tp1_distance: plan.and_then(|x| x.detail["order"]["tp1_distance"].as_f64()),
                close_volume: plan.and_then(|x| x.detail["order"]["plan"]["close_volume"].as_i64()),
                partial: plan.is_some_and(|x| {
                    x.detail["order"]["plan"]["mode"].as_str() == Some("partial_runner")
                }),
                plan_basis: plan.and_then(|x| plan_basis(&x.detail)),
                session_date: plan.and_then(|x| {
                    x.detail["session_date"]
                        .as_str()
                        .and_then(|d| d.parse().ok())
                }),
                entry_key: plan.map(|x| x.entry_key.clone()),
                tp1_sent,
                tp1_next_try: None,
                end_attempts,
                end_next_try: None,
                best: None,
                mae: 0.0,
                mfe: 0.0,
                last_amend: None,
            });
            // A plan recorded after the position was first seen fills what is
            // still unknown.
            if let Some(x) = plan {
                if e.tp1_distance.is_none() {
                    e.tp1_distance = x.detail["order"]["tp1_distance"].as_f64();
                    e.close_volume = x.detail["order"]["plan"]["close_volume"].as_i64();
                    e.partial =
                        x.detail["order"]["plan"]["mode"].as_str() == Some("partial_runner");
                }
                if e.plan_basis.is_none() {
                    e.plan_basis = plan_basis(&x.detail);
                }
                if e.session_date.is_none() {
                    e.session_date = x.detail["session_date"]
                        .as_str()
                        .and_then(|d| d.parse().ok());
                }
                if e.entry_key.is_none() {
                    e.entry_key = Some(x.entry_key.clone());
                }
            }
            // The broker wins.
            e.volume = p.trade_data.volume;
            e.stop = p.stop_loss;
            e.take_profit = p.take_profit;
            if p.price.is_some() {
                e.entry = p.price;
            }
            e.tp1_sent |= tp1_sent;
        }
        self.own.retain(|id, _| seen.contains(id));
        let broker_positions: Vec<i64> = res.position.iter().map(|p| p.position_id).collect();
        self.recover_history_exits(&broker_positions).await?;
        // An unresolved open still pending at the broker, or younger than one
        // reconcile interval, stays pending (its risk held, its symbol
        // blocked); an older one with nothing at the broker is released
        // (RECONCILED_ABSENT), never resent.
        let r = rules(self.pool, account_id, self.env).await?;
        let grace = chrono::Duration::seconds(r.reconcile_secs as i64);
        self.pending.clear();
        for u in unresolved {
            // What of it is filled: the broker's position when it is held,
            // else the deals recorded. The adopted part's risk is the
            // position's; the remainder's is held here, never both.
            let filled = u
                .position_id
                .and_then(|p| self.own.get(&p))
                .map(|o| o.volume)
                .unwrap_or(u.filled);
            if u.volume.is_some_and(|v| filled >= v) {
                ts::record_event(
                    self.pool,
                    &crate::trading::reconciled_row(
                        account_id,
                        &u.entry_key,
                        "RECONCILED_FILL",
                        None,
                        "a Strategy 3 open found filled in full at the broker by reconcile",
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
                self.pending.push(Pending {
                    risk: crate::trading::outstanding_risk(u.risk, u.volume, filled).unwrap_or(0.0),
                    entry_key: u.entry_key,
                    symbol_id: u.symbol_id.unwrap_or_default(),
                    at: u.created_at,
                });
                continue;
            }
            ts::record_event(
                self.pool,
                &crate::trading::reconciled_row(
                    account_id,
                    &u.entry_key,
                    "RECONCILED_ABSENT",
                    None,
                    "an unresolved Strategy 3 open neither filled nor pending at the broker after its grace: released, never resent",
                ),
            )
            .await?;
        }
        self.d.version = Some(r.version);
        let detail = json!({
            "positions": self.own.values().map(|o| json!({
                "position_id": o.position_id,
                "symbol": self.syms.get(&o.symbol_id).map(|s| s.name.clone()),
                "side": o.side.as_str(),
                "volume": o.volume,
                "entry": o.entry, "stop": o.stop, "take_profit": o.take_profit,
                "tp1_sent": o.tp1_sent,
            })).collect::<Vec<_>>(),
            "program_positions": self.program_positions,
        });
        self.d
            .note(
                None,
                "reconcile",
                &format!(
                    "{} Strategy 3 position(s); {} position(s) of this program",
                    self.own.len(),
                    self.program_positions
                ),
                detail,
            )
            .await;
        // Every run, recorded or not: the decision is kept only when it
        // changes, so its time cannot tell a quiet loop from a stalled one.
        crate::trading::beat(&self.live, account_id, |l| {
            l.last_reconcile_run_at = Some(Utc::now())
        });
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
        // the limits alert; they never pause.
        crate::limits::check(
            self.pool,
            account_id,
            &crate::limits::S3,
            json!({"strategy": STRATEGY, "label": LABEL, "prereg_hash": self.hash}),
            self.d.version,
        )
        .await?;
        Ok(())
    }

    /// An Unconfirmed open found filled, in full or in `part`: recorded once,
    /// with its fill.
    async fn adopt(
        &mut self,
        key: &str,
        p: &jiaoyi_ctrader::messages::ProtoOaPosition,
        part: bool,
    ) -> anyhow::Result<()> {
        let account_id = self.account_id();
        let (kind, outcome, description) = if part {
            (
                "RECONCILED_PARTIAL_FILL",
                "reconciled_partial_fill",
                "an unconfirmed Strategy 3 open found partly filled by reconcile: the position adopted, the remainder held",
            )
        } else {
            (
                "RECONCILED_FILL",
                "reconciled_fill",
                "an unconfirmed Strategy 3 open found filled by reconcile",
            )
        };
        ts::record_event(
            self.pool,
            &ts::EventRow {
                account_id,
                entry_key: Some(key.to_string()),
                client_msg_id: None,
                payload_type: 0,
                kind: kind.into(),
                position_id: Some(p.position_id),
                order_id: None,
                deal_id: None,
                symbol_id: Some(p.trade_data.symbol_id),
                side: Side::from_proto(p.trade_data.trade_side).map(|s| s.as_str().to_string()),
                price: p.price,
                volume: Some(p.trade_data.volume),
                stop_loss: p.stop_loss,
                label: p.trade_data.label.clone(),
                error_code: None,
                description: Some(description.into()),
                is_server_event: None,
                raw: vec![],
                received_at: Utc::now(),
            },
        )
        .await?;
        let symbol = self
            .syms
            .get(&p.trade_data.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        s3s::record_fill(
            self.pool,
            &s3s::FillRow {
                account_id,
                entry_key: key.to_string(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol,
                position_id: Some(p.position_id),
                side: Side::from_proto(p.trade_data.trade_side).map(|s| s.as_str().to_string()),
                outcome: outcome.into(),
                sent_at: Utc::now(),
                answered_at: None,
                latency_ms: None,
                decision_price: None,
                fill_price: p.price,
                slippage: None,
                spread: None,
                volume: Some(p.trade_data.volume),
                detail: json!({"note": "the open was Unconfirmed; reconcile found its position"}),
            },
        )
        .await?;
        Ok(())
    }

    /// Prepare a symbol's session: the M30 bars, the open and σ, and the
    /// median spread. Kept for the session; a broker error is retried after
    /// [`PREPARE_RETRY`].
    pub async fn prepare(
        &mut self,
        symbol_id: i64,
        s: Session,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let fresh = match self.syms.get(&symbol_id).and_then(|x| x.day.as_ref()) {
            Some(Ok(d)) => d.session.date != s.date,
            Some(Err((_, at))) => {
                now - *at >= chrono::Duration::from_std(PREPARE_RETRY).unwrap_or_default()
            }
            None => true,
        };
        if !fresh {
            return Ok(());
        }
        let account_id = self.account_id();
        let Some(rule) = self.syms.get(&symbol_id).and_then(|x| x.rule) else {
            return Ok(());
        };
        let table = self
            .syms
            .get(&symbol_id)
            .and_then(|sym| sym.preopen.as_ref())
            .filter(|p| p.matches(symbol_id, s) && p.table.is_some() && p.missing_marks == 0)
            .and_then(|p| p.table.clone());
        let read = load_day(
            self.pool,
            self.session,
            account_id,
            symbol_id,
            rule,
            s,
            now,
            table,
        );
        let day = self.protect_while_reading(read).await?;
        let name = self.syms[&symbol_id].name.clone();
        match &day {
            Ok(d) => {
                let detail = json!({
                    "session_date": d.session.date, "open_at": d.session.open, "close_at": d.session.close,
                    "open": d.open.as_ref().ok(), "open_refused": d.open.as_ref().err(),
                    "sigma_marks": d.table.marks.iter()
                        .map(|(m, v)| (m.to_string(), v.as_ref().map(|x| json!(x)).unwrap_or_else(|e| json!(e))))
                        .collect::<serde_json::Map<_, _>>(),
                    "sessions_used": d.table.sessions_used,
                    "median_spread": d.spread.as_ref().ok().map(|x| x.0),
                    "spread_quotes": d.spread.as_ref().ok().map(|x| x.1),
                    "rule": rule.describe(),
                });
                let reason = match &d.open {
                    Ok(o) => format!("session {} prepared: open {o}", d.session.date),
                    Err(why) => why.clone(),
                };
                self.d.note(Some(&name), "session", &reason, detail).await;
            }
            Err((why, _)) => {
                self.d.note(Some(&name), "session", why, json!({})).await;
            }
        }
        if let Some(x) = self.syms.get_mut(&symbol_id) {
            x.day = Some(day);
        }
        Ok(())
    }

    fn open_risk(&self) -> Result<f64, String> {
        let mut total: f64 = self.pending.iter().map(|p| p.risk).sum();
        for o in self.own.values() {
            let (Some(entry), Some(stop)) = (o.entry, o.stop) else {
                return Err(format!(
                    "position {} has no known entry or stop: its open risk is unknown",
                    o.position_id
                ));
            };
            let Some(rate) = self.rate(o.symbol_id) else {
                return Err(format!(
                    "position {} has no quote→deposit rate: its open risk is unknown",
                    o.position_id
                ));
            };
            total += risk_at_stop(o.side, entry, stop, o.volume, rate);
        }
        Ok(total)
    }

    /// A quote: update the symbol, then TP1 for Strategy 3's position on it,
    /// or an entry when it has none.
    pub async fn on_quote(
        &mut self,
        symbol_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        // The nested exit pump already managed these owned-symbol events.
        // Replaying them must not turn a formerly owned quote into an entry
        // or move the cached market back in time.
        if self
            .protected_through
            .get(&symbol_id)
            .is_some_and(|at| now <= *at)
        {
            return Ok(());
        }
        let Some(sym) = self.syms.get_mut(&symbol_id) else {
            return Ok(());
        };
        if bid.is_some() {
            sym.bid = bid;
            sym.bid_at = Some(now);
        }
        if ask.is_some() {
            sym.ask = ask;
            sym.ask_at = Some(now);
        }
        let (bid, ask) = (sym.bid, sym.ask);
        let reacted = self.react(symbol_id, bid, ask, now).await;
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
        // the distance to the trigger, after the quote is acted on (so a
        // session this quote prepared is shown).
        if let Some(sym) = self.syms.get(&symbol_id) {
            let (distance, name) = (trigger_distance(sym, now), sym.name.clone());
            crate::trading::beat(&self.live, self.account_id(), |l| {
                l.triggers.insert(name, distance);
            });
        }
        reacted
    }

    /// A quote's action: TP1 for this symbol's positions, or an entry.
    async fn react(
        &mut self,
        symbol_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if self.exits_for_symbol(symbol_id, bid, ask, now).await? {
            return Ok(());
        }
        if self.pending.iter().any(|p| p.symbol_id == symbol_id) {
            return Ok(());
        }
        self.entry(symbol_id, now).await
    }

    async fn exits_for_symbol(
        &mut self,
        symbol_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let positions: Vec<i64> = self
            .own
            .values()
            .filter(|o| o.symbol_id == symbol_id)
            .map(|o| o.position_id)
            .collect();
        let owned = !positions.is_empty();
        if owned {
            for pid in positions {
                self.tp1(pid, bid, ask, now).await?;
                // Grok 4.7 · № 17,608 28-Oct (ordered: owner "change the forbid"):
                // the trail moves the stop only in the trade's favour and
                // carries the take-profit, so the amend does not delete it.
                self.trail(pid, bid, ask, now).await?;
            }
        }
        Ok(owned)
    }

    async fn tp1(
        &mut self,
        position_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(o) = self.own.get(&position_id).cloned() else {
            return Ok(());
        };
        let (Some(fill), Some(dist), Some(close), true, false) = (
            o.entry,
            o.tp1_distance,
            o.close_volume,
            o.partial,
            o.tp1_sent,
        ) else {
            return Ok(());
        };
        let limits = self.syms.get(&o.symbol_id).and_then(|s| s.limits);
        let digits = limits.map_or(5, |l| l.digits);
        if o.tp1_next_try.is_some_and(|t| now < t) {
            return Ok(());
        }
        // Claude · № 17,074 25-Oct (ordered: owner "merge when green and
        // deploy"; Codex round-4 P2 on #15): the close is planned again for
        // the volume the broker holds, the whole of it when it cannot be
        // split. With no recorded basis, the recorded close, if it fits.
        let close = match (o.plan_basis, limits) {
            (Some(b), Some(l)) => tp1_close_volume(&b, &l, o.volume),
            _ if close < o.volume => close,
            _ => return Ok(()),
        };
        if !tp1_reached(o.side, fill, dist, digits, bid, ask) {
            return Ok(());
        }
        // Claude · № 17,020 25-Oct (Codex P2 on #15): every attempt is its own
        // intent, numbered from the recorded ones. A refusal, or a rejection
        // with nothing filled, is tried again after END_RETRY while TP1 is
        // still reached; a fill, a partial fill or no answer is never resent.
        let attempt =
            s3s::orders_for_position(self.pool, self.account_id(), position_id, "tp1").await? + 1;
        if let Some(x) = self.own.get_mut(&position_id) {
            x.tp1_sent = true;
        }
        let sent = self.close(&o, close, "tp1", attempt, bid, ask, now).await?;
        let again = match &sent {
            None => true,
            Some(s) => matches!(s.outcome, Outcome::Rejected { .. }) && !partly_filled(s),
        };
        if again {
            if let Some(x) = self.own.get_mut(&position_id) {
                x.tp1_sent = false;
                x.tp1_next_try =
                    Some(now + chrono::Duration::from_std(END_RETRY).unwrap_or_default());
            }
        }
        Ok(())
    }

    /// Move an owned position's stop to best − R (long) or best + R (short).
    /// The amend carries the broker's take-profit, so the target is not
    /// deleted. The stop never loosens. An amend is not sent in the session's
    /// last 30 minutes, and not more often than once per [`TRAIL_AMEND_SECS`].
    ///
    /// Grok 4.7 · № 17,608 28-Oct (ordered: owner "change the forbid"):
    /// the 25 Oct "no stop amends" sentence is retired by the v5 registration.
    /// The gate already re-sends the take-profit on an amend.
    async fn trail(
        &mut self,
        position_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(o) = self.own.get(&position_id).cloned() else {
            return Ok(());
        };
        let (Some(_), Some(current_stop)) = (o.entry, o.stop) else {
            return Ok(());
        };
        // v5 uses the original opening-plan R, also restored on reconcile.
        // The broker's current stop changes after each amend and can reach
        // entry; it cannot establish the original distance when the plan is
        // missing. Keep the existing protection in that case.
        let Some(r) = o.plan_basis.map(|basis| basis.r) else {
            return Ok(());
        };
        if !r.is_finite() || r <= 0.0 {
            return Ok(());
        }
        let price = match o.side {
            Side::Buy => bid,
            Side::Sell => ask,
        };
        let Some(p) = price.filter(|x| x.is_finite() && *x > 0.0) else {
            return Ok(());
        };
        let best = match (o.best, o.side) {
            (Some(b), Side::Buy) => b.max(p),
            (Some(b), Side::Sell) => b.min(p),
            (None, _) => p,
        };
        if let Some(x) = self.own.get_mut(&position_id) {
            x.best = Some(best);
            if let Some(entry) = x.entry {
                let (mae, mfe) = update_excursions(x.side, entry, p, x.mae, x.mfe);
                x.mae = mae;
                x.mfe = mfe;
            }
        }
        let Some(digits) = self
            .syms
            .get(&o.symbol_id)
            .and_then(|s| s.limits)
            .map(|l| l.digits)
        else {
            return Ok(());
        };
        let Some(new_stop) = trail_price(o.side, best, r, current_stop, digits) else {
            return Ok(());
        };
        if o.last_amend
            .is_some_and(|t| now.signed_duration_since(t).num_seconds() < TRAIL_AMEND_SECS)
        {
            return Ok(());
        }
        // No amend in the session's last 30 minutes.
        if let Some(sym) = self.syms.get(&o.symbol_id) {
            if let Some(session) = sym.rule.and_then(|rule| sessions::session_at(&rule, now)) {
                if session.close - now < chrono::Duration::minutes(30) {
                    return Ok(());
                }
            }
        }
        let account_id = self.account_id();
        let name = self
            .syms
            .get(&o.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let ticks = (new_stop * 100_000.0).round() as i64;
        let key = format!("s3-trail:{account_id}:{}:{ticks}", o.position_id);
        let rules = rules(self.pool, account_id, self.env).await?;
        // kind 'trail' is allowed by migration 0019. Before that the check
        // rejected the row after the intent was stored, so the frame was
        // never written. Grok 4.7 · № 17,655.
        let order = s3s::OrderRow {
            account_id,
            entry_key: key.clone(),
            label: LABEL.into(),
            prereg_hash: self.hash.clone(),
            kind: "trail".into(),
            symbol: name.clone(),
            symbol_id: o.symbol_id,
            position_id: Some(o.position_id),
            side: Some(o.side.as_str().into()),
            session_date: o.session_date,
            decided_at: now,
            bid,
            ask,
            volume: Some(o.volume),
            detail: json!({"strategy": STRATEGY, "from_stop": current_stop, "best": best,
                "r": r, "new_stop": new_stop, "open_entry_key": o.entry_key}),
        };
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(rules.version),
            },
            order,
            recorded_at: Mutex::new(None),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: f64::INFINITY,
        };
        let cmd = Command::AmendStop {
            position_id: o.position_id,
            side: o.side,
            current_stop: Some(current_stop),
            new_stop,
        };
        match gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(rules.order_wait_secs),
        )
        .await
        {
            Ok(sent) => {
                if let Some(x) = self.own.get_mut(&position_id) {
                    x.last_amend = Some(now);
                    if matches!(sent.outcome, Outcome::Amended { .. }) {
                        x.stop = Some(new_stop);
                    }
                }
                self.record_fill(
                    &ledger,
                    &key,
                    "trail",
                    &name,
                    Some(o.position_id),
                    o.side,
                    Some(new_stop),
                    bid,
                    ask,
                    o.volume,
                    &sent,
                )
                .await?;
                self.d
                    .note(
                        Some(&name),
                        "trail",
                        &format!("stop to {new_stop}: {:?}", sent.outcome),
                        json!({"position_id": o.position_id, "from_stop": current_stop, "best": best,
                            "entry_key": key}),
                    )
                    .await;
            }
            Err(e) => {
                if let Some(x) = self.own.get_mut(&position_id) {
                    x.last_amend = Some(now);
                }
                self.d
                    .note(
                        Some(&name),
                        "trail_refused",
                        &e.to_string(),
                        json!({"position_id": o.position_id, "entry_key": key}),
                    )
                    .await;
            }
        }
        Ok(())
    }

    /// The session end: every Strategy 3 position whose session is over, or
    /// whose symbol is outside its session, closes.
    pub async fn session_end(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let owns: Vec<Own> = self.own.values().cloned().collect();
        for o in owns {
            if o.end_next_try.is_some_and(|t| now < t) {
                continue;
            }
            let Some(sym) = self.syms.get(&o.symbol_id) else {
                continue;
            };
            let current = sym.rule.and_then(|r| sessions::session_at(&r, now));
            let over = match (current, o.session_date) {
                (None, _) => true,
                (Some(s), Some(d)) => s.date != d,
                (Some(_), None) => false,
            };
            if !over {
                continue;
            }
            let (bid, ask) = (sym.bid, sym.ask);
            // Claude · № 17,020 25-Oct (Codex P1 on #15): every attempt is its
            // own intent; a refusal or rejection is tried again after
            // END_RETRY, a sent close after one reconcile interval if the
            // broker still shows the position.
            let attempt = o.end_attempts + 1;
            let sent = self
                .close(&o, o.volume, "session_end", attempt, bid, ask, now)
                .await?;
            let r = rules(self.pool, self.account_id(), self.env).await?;
            let wait = match sent.map(|s| s.outcome) {
                Some(Outcome::Filled { .. }) | Some(Outcome::Unconfirmed) => {
                    chrono::Duration::seconds(r.reconcile_secs as i64)
                }
                _ => chrono::Duration::from_std(END_RETRY).unwrap_or_default(),
            };
            if let Some(x) = self.own.get_mut(&o.position_id) {
                x.end_attempts = attempt;
                x.end_next_try = Some(now + wait);
            }
        }
        Ok(())
    }

    /// Send a close (attempt `attempt` of its kind, each its own intent key);
    /// `None` when the gate refused it (nothing sent).
    #[allow(clippy::too_many_arguments)]
    async fn close(
        &mut self,
        o: &Own,
        volume: i64,
        kind: &str,
        attempt: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<gate::Sent>> {
        let account_id = self.account_id();
        let r = rules(self.pool, account_id, self.env).await?;
        let name = self
            .syms
            .get(&o.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let key = match kind {
            "tp1" => format!("s3-tp1:{account_id}:{}:{attempt}", o.position_id),
            _ => format!("s3-end:{account_id}:{}:{attempt}", o.position_id),
        };
        let decision_price = match o.side {
            Side::Buy => bid,
            Side::Sell => ask,
        };
        let order = s3s::OrderRow {
            account_id,
            entry_key: key.clone(),
            label: LABEL.into(),
            prereg_hash: self.hash.clone(),
            kind: kind.into(),
            symbol: name.clone(),
            symbol_id: o.symbol_id,
            position_id: Some(o.position_id),
            side: Some(o.side.as_str().into()),
            session_date: o.session_date,
            decided_at: now,
            bid,
            ask,
            volume: Some(volume),
            detail: json!({"strategy": STRATEGY, "position_volume": o.volume,
                "tp1_distance": o.tp1_distance, "entry": o.entry, "open_entry_key": o.entry_key}),
        };
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(r.version),
            },
            order,
            recorded_at: Mutex::new(None),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: f64::INFINITY,
        };
        let cmd = Command::Close {
            position_id: o.position_id,
            volume,
        };
        let res = gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
        )
        .await;
        match res {
            Ok(sent) => {
                self.record_fill(
                    &ledger,
                    &key,
                    kind,
                    &name,
                    Some(o.position_id),
                    o.side,
                    decision_price,
                    bid,
                    ask,
                    volume,
                    &sent,
                )
                .await?;
                self.d
                    .note(
                        Some(&name),
                        kind,
                        &format!(
                            "close {volume} of position {}: {:?}",
                            o.position_id, sent.outcome
                        ),
                        json!({"entry_key": key}),
                    )
                    .await;
                Ok(Some(sent))
            }
            Err(SendError::Refused(x)) => {
                self.d
                    .note(
                        Some(&name),
                        &format!("{kind}_refused"),
                        &x.reason,
                        json!({"entry_key": key}),
                    )
                    .await;
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_fill(
        &self,
        ledger: &S3Ledger<'_>,
        key: &str,
        kind: &str,
        symbol: &str,
        position_id: Option<i64>,
        side: Side,
        decision_price: Option<f64>,
        bid: Option<f64>,
        ask: Option<f64>,
        volume: i64,
        sent: &gate::Sent,
    ) -> anyhow::Result<()> {
        let sent_at = ledger.recorded_at.lock().unwrap().unwrap_or_else(Utc::now);
        // The order's deals, summed up by the rule both strategies share: the
        // deals' position, their sum at their volume-weighted price (Codex
        // P1s on #15).
        // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
        let s = crate::fills::summarize(sent, position_id, volume);
        let (outcome, price, pid, vol) = (s.outcome, s.price, s.position_id, s.volume);
        let answered_at = s.filled_at;
        // A buy pays the ask and a sell gets the bid: positive is adverse.
        // An exit of a long is a sell, of a short a buy.
        let trade_side = if kind == "open" {
            side
        } else {
            crate::fills::exit_side(side)
        };
        let slippage = crate::fills::slippage(trade_side, price, decision_price);
        s3s::record_fill(
            self.pool,
            &s3s::FillRow {
                account_id: self.account_id(),
                entry_key: key.to_string(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: kind.into(),
                symbol: symbol.into(),
                position_id: pid,
                side: Some(side.as_str().into()),
                outcome: outcome.into(),
                sent_at,
                answered_at,
                latency_ms: answered_at.map(|a| (a - sent_at).num_milliseconds()),
                decision_price,
                fill_price: price,
                slippage,
                spread: match (bid, ask) {
                    (Some(b), Some(a)) => Some(a - b),
                    _ => None,
                },
                volume: vol,
                detail: json!({"outcome": sent.outcome, "client_msg_id": sent.client_msg_id}),
            },
        )
        .await?;
        Ok(())
    }

    async fn skip(
        &mut self,
        symbol_id: i64,
        name: &str,
        why: &str,
        detail: Value,
        now: DateTime<Utc>,
    ) {
        if let Some(s) = self.syms.get_mut(&symbol_id) {
            s.last_try = Some(now);
        }
        self.d.note(Some(name), "skip", why, detail).await;
    }

    /// An entry on a quote of a symbol with no Strategy 3 position.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    async fn entry(&mut self, symbol_id: i64, now: DateTime<Utc>) -> anyhow::Result<()> {
        let sym = self.syms[&symbol_id].clone();
        let name = sym.name.clone();
        let Some(rule) = sym.rule else {
            return Ok(());
        };
        let Some(s) = sessions::session_at(&rule, now) else {
            return Ok(());
        };
        if band::entry_window(&s, now).is_err() {
            return Ok(());
        }
        let Some(mark) = band::decision_mark(&s, now) else {
            return Ok(());
        };
        let (Some(bid), Some(ask)) = (sym.bid, sym.ask) else {
            return Ok(());
        };
        if !bid.is_finite() || !ask.is_finite() || bid > ask {
            return Ok(());
        }
        if sym.last_decision_mark.is_some_and(|at| at >= mark) {
            return Ok(());
        }
        // The session veto is durable, unlike the per-connection symbol cache.
        // A failed read must not consume this mark or permit a new entry.
        if cost_skip_stands(sym.cost_skip_session, s.date) {
            return Ok(());
        }
        if s3s::cost_skipped_in_session(self.pool, self.account_id(), &name, s.date, &self.hash)
            .await?
        {
            self.syms.get_mut(&symbol_id).unwrap().cost_skip_session = Some(s.date);
            tracing::info!(account = self.account_id(), symbol = %name,
                session_date = %s.date, prereg_hash = %self.hash,
                "strategy 3 restored durable session cost veto");
            return Ok(());
        }
        // Persist the first eligible quote before any preparation or entry
        // gate. An inside/no-band/refused decision must not be retried on a
        // later crossing within this mark, including after reconnect.
        let previous: Option<Option<String>> = jiaoyi_store::sqlx::query_scalar(
            "SELECT detail->>'decision_mark' FROM trading_decisions WHERE account_id=$1 AND symbol=$2 AND kind='s3_mark_decision' ORDER BY id DESC LIMIT 1")
            .bind(self.account_id()).bind(&name).fetch_optional(self.pool).await?;
        if let Some(at) = previous
            .flatten()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        {
            if at >= mark {
                self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(at);
                return Ok(());
            }
        }
        if sym.entered_session == Some(s.date) {
            self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(mark);
            return Ok(());
        }
        let prior = s3s::opens_in_session(self.pool, self.account_id(), symbol_id, s.date).await?;
        if session_already_entered(prior) {
            if let Some(x) = self.syms.get_mut(&symbol_id) {
                x.entered_session = Some(s.date);
                x.last_decision_mark = Some(mark);
            }
            self.skip(
                symbol_id,
                &name,
                "one entry already recorded for this symbol in this session",
                json!({"session_date": s.date, "opens": prior}),
                now,
            )
            .await;
            return Ok(());
        }
        ts::record_decision(self.pool, self.account_id(), Some(&name), "s3_mark_decision",
            "first eligible quote consumed this decision mark",
            &json!({"strategy":STRATEGY,"label":LABEL,"prereg_hash":self.hash,"decision_mark":mark,"decided_at":now,"bid":bid,"ask":ask}),
            self.d.version).await?;
        self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(mark);
        if sym
            .last_try
            .is_some_and(|t| now - t < chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default())
        {
            return Ok(());
        }
        if let Some(retry_at) = self.funding_retry_at.filter(|t| now < *t) {
            self.d
                .note(
                    Some(&name),
                    "funding_backoff",
                    "the broker rejected an entry for insufficient funds: account entry retries wait; exits continue",
                    json!({"retry_at": retry_at}),
                )
                .await;
            return Ok(());
        }
        self.prepare(symbol_id, s, now).await?;
        let mut sym = self.syms[&symbol_id].clone();
        sym.bid = Some(bid);
        sym.ask = Some(ask);
        let day = match &sym.day {
            Some(Ok(d)) if d.session.date == s.date => d.clone(),
            _ => return Ok(()),
        };
        let Ok(open) = day.open.clone() else {
            return Ok(());
        };
        let (side, b) = match signal(&day.table, &s, open, now, sym.bid, sym.ask) {
            Ok(Some(x)) => x,
            Ok(None) => return Ok(()),
            Err(why) => {
                self.d.note(Some(&name), "no_band", &why, json!({})).await;
                return Ok(());
            }
        };
        let entry_price = match side {
            Dir::Buy => sym.ask,
            Dir::Sell => sym.bid,
        }
        .unwrap_or(f64::NAN);
        let mut detail = json!({
            "session_date": s.date, "decided_at": now, "bid": sym.bid, "ask": sym.ask,
            "side": side, "band": b, "open": open, "sigma": b.sigma,
        });
        let Some(limits) = sym.limits else {
            self.skip(
                symbol_id,
                &name,
                "the symbol's volume rules are not stored",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        // Keep the actual entry-time grid and UTC session for later replay;
        // today's symbol metadata or calendar must not replace this snapshot.
        detail["volume_limits"] = json!(limits);
        detail["session"] = json!(s);
        let spread = match &day.spread {
            Ok((m, _)) => *m,
            Err(why) => {
                self.skip(symbol_id, &name, why, detail, now).await;
                return Ok(());
            }
        };
        let Some(rate) = self.rate(symbol_id) else {
            self.skip(
                symbol_id,
                &name,
                "no broker quote→deposit rate for this symbol yet",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        if self
            .commission_read
            .is_none_or(|t| t.elapsed() >= COMMISSION_REFRESH)
        {
            self.read_commission().await;
        }
        let commission_rt = match &self.commission {
            Ok(book) => {
                let (per_unit, label) = book
                    .per_class
                    .get(sym.class.as_str())
                    .cloned()
                    .unwrap_or((book.overall_median, "estimated: the overall median".into()));
                detail["commission_label"] = json!(label);
                2.0 * per_unit / rate
            }
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("commission not measured: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        let cost = match cost_reserve(
            spread,
            commission_rt,
            slippage_bps_per_side(sym.class),
            entry_price,
        ) {
            Ok(c) => c,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        let rel = relative_stop(b.half_width, limits.digits);
        let r_price = rel as f64 / 100_000.0;
        // v3 cost gate (27-10-2026). The statements measured median C ≈ 2.2R,
        // so TP1 sat near 5R and a no-drift process hit it ~16% of the time.
        // Skip when the reserve is at least the stop. This does not change Q.
        if !(r_price > 0.0) || cost.total >= r_price {
            detail["strategy"] = json!(STRATEGY);
            detail["label"] = json!(LABEL);
            detail["prereg_hash"] = json!(self.hash);
            detail["skip_code"] = json!("cost_ge_r");
            let reason = format!(
                "cost reserve C ({:.6}) >= R ({:.6}): barrier too far for a no-drift process; skipped",
                cost.total, r_price
            );
            // This is execution state, not best-effort reporting. Persist it
            // before the cache suppresses subsequent quotes for this session.
            ts::record_decision(
                self.pool,
                self.account_id(),
                Some(&name),
                "s3_skip",
                &reason,
                &detail,
                self.d.version,
            )
            .await?;
            if let Some(x) = self.syms.get_mut(&symbol_id) {
                x.cost_skip_session = Some(s.date);
                x.last_try = Some(now);
            }
            tracing::info!(account = self.account_id(), symbol = %name,
                kind = "skip", reason, prereg_hash = %self.hash, "strategy 3 decision");
            return Ok(());
        }
        let e = Entry {
            side,
            band: b,
            entry_price,
            relative_stop: rel,
            r: rel as f64 / 100_000.0,
            cost,
        };
        detail["entry"] = json!(e);
        detail["quote_to_deposit"] = json!(rate);
        let r = rules(self.pool, self.account_id(), self.env).await?;
        self.d.version = Some(r.version);
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
        // the owner's pause ("only human can pause trade"): no new entry. TP1,
        // the session end and the broker's stop and take-profit are untouched.
        if r.paused {
            if let Some(s) = self.syms.get_mut(&symbol_id) {
                s.last_try = Some(now);
            }
            self.d
                .note(
                    Some(&name),
                    "paused",
                    "Strategy 3 is paused by the owner on this account (trading.s3_paused): no new entry; open positions keep their exits",
                    detail,
                )
                .await;
            return Ok(());
        }
        // Claude · № 17,020 25-Oct (Codex P1 on #15): a live entry is one
        // broker minimum, admitted account-wide (both strategies' loops) from
        // fresh broker state; the account's lock is held through the send.
        let _admission = if r.live {
            let g = crate::admission::lock(self.account_id()).await;
            if let Err(why) = crate::admission::live_minimum(self.pool, self.session).await {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
            Some(g)
        } else {
            None
        };
        let equity = match fresh_equity(self.session).await {
            Ok(x) => x,
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("fresh equity unreadable: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        // Live: the fresh account-wide admission above is the count.
        let open_programs = if r.live {
            0
        } else {
            self.program_positions + self.pending.len()
        };
        // Kelly (№ 17,588–17,589, 28-10-2026). A ceiling and a skip in front of
        // size(). It does not write risk_pct, the live minimum, or the budget.
        // Under 20 closes on this hash and symbol: not an input. A non-positive
        // fraction skips. A positive fraction is recorded and does not raise size.
        // Grok 4.7. Other models: do not plug in the goal, and do not double
        // after a loss.
        let keys =
            s3s::open_keys_for_hash(self.pool, self.account_id(), symbol_id, &self.hash).await?;
        let (closed, _) = ts::closed_trades(self.pool, self.account_id(), "s3:").await?;
        let rs: Vec<f64> = closed
            .iter()
            .filter(|t| keys.iter().any(|k| k == &t.entry_key))
            .map(|t| t.r)
            .collect();
        let kelly = jiaoyi_core::strategy3::kelly::read(&rs);
        detail["kelly"] = json!({
            "n": kelly.n,
            "wins": kelly.wins,
            "p": kelly.p,
            "b": kelly.b,
            "f_star": kelly.f_star,
            "quarter": kelly.quarter,
            "min_closes": jiaoyi_core::strategy3::kelly::MIN_CLOSES,
            "writes_size": false,
        });
        if jiaoyi_core::strategy3::kelly::skip(&kelly) {
            self.skip(
                symbol_id,
                &name,
                &format!(
                    "Kelly f* ({:.4}) is not positive on {} closes under this hash: skipped",
                    kelly.f_star.unwrap_or(f64::NAN),
                    kelly.n
                ),
                detail,
                now,
            )
            .await;
            return Ok(());
        }
        let sized = match size(&r, equity, rel, rate, &limits, open_programs) {
            Ok(x) => x,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["sizing"] = json!(sized);
        let order = match plan_order(&e, &limits, sized.volume) {
            Ok(o) => o,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["order"] = json!(order);
        let budget = self
            .open_risk()
            .and_then(|held| within_budget(held, sized.risk, equity, r.open_risk_pct));
        if let Err(why) = budget {
            self.skip(symbol_id, &name, &why, detail, now).await;
            return Ok(());
        }
        let account_id = self.account_id();
        let n = s3s::opens_in_session(self.pool, account_id, symbol_id, s.date).await?;
        let key = format!("s3:{account_id}:{symbol_id}:{}:{}", s.date, n + 1);
        if let Some(x) = self.syms.get_mut(&symbol_id) {
            x.last_try = Some(now);
        }
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(r.version),
            },
            order: s3s::OrderRow {
                account_id,
                entry_key: key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol: name.clone(),
                symbol_id,
                position_id: None,
                side: Some(dir_side(side).as_str().into()),
                session_date: Some(s.date),
                decided_at: now,
                bid: sym.bid,
                ask: sym.ask,
                volume: Some(sized.volume),
                detail: detail.clone(),
            },
            recorded_at: Mutex::new(None),
        };
        let cmd = Command::Open {
            symbol_id,
            side: dir_side(side),
            volume: sized.volume,
            relative_stop: rel,
            relative_take_profit: Some(order.relative_take_profit),
            limits,
            quote_to_deposit: Some(rate),
            label: LABEL.into(),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: sized.max_risk,
        };
        match gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
        )
        .await
        {
            Ok(sent) => {
                self.record_fill(
                    &ledger,
                    &key,
                    "open",
                    &name,
                    None,
                    dir_side(side),
                    Some(entry_price),
                    sym.bid,
                    sym.ask,
                    sized.volume,
                    &sent,
                )
                .await?;
                // Any fill, whatever the last answer (several deals, or part
                // of the order before a rejection or no answer), is read from
                // the broker at once: its aggregate volume, entry, stop and
                // take-profit, with its recorded plan. An event carries one
                // deal, not the position (Codex P1 on #15; Strategy 1 does
                // the same).
                // Claude · № 17,060 25-Oct (ordered: owner "merge when green
                // and deploy"; Codex round 3 on #15)
                if matches!(&sent.outcome, Outcome::Rejected { code: Some(code), .. } if code == "NOT_ENOUGH_MONEY")
                {
                    self.funding_retry_at =
                        Some(now + chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default());
                }
                if matches!(sent.outcome, Outcome::Filled { .. }) || partly_filled(&sent) {
                    self.reconcile(Utc::now()).await?;
                } else if matches!(sent.outcome, Outcome::Unconfirmed) {
                    self.pending.push(Pending {
                        entry_key: key.clone(),
                        symbol_id,
                        risk: sized.risk,
                        at: now,
                    });
                }
                self.d
                    .note(
                        Some(&name),
                        "sent",
                        &format!(
                            "{STRATEGY} {} {} cents ({}): {:?}",
                            dir_side(side).as_str(),
                            sized.volume,
                            sized.mode,
                            sent.outcome
                        ),
                        json!({"entry_key": key, "relative_stop": rel,
                               "relative_take_profit": order.relative_take_profit}),
                    )
                    .await;
            }
            Err(SendError::Refused(x)) => {
                self.d.note(Some(&name), "refused", &x.reason, detail).await;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    /// A broker-side execution message on a Strategy 3 position (a stop or
    /// a take-profit hit): recorded as it came, and a closing deal also as a
    /// broker exit, so `/trading/costs` reports it.
    // Codex · № 17,197 26-Oct (ordered: O-1, make skipped owned executions observable).
    pub async fn server_event(&self, m: &ReceivedExecution) {
        let Some(e) = gate::decode_execution(self.account_id(), m) else {
            return;
        };
        let owned = e.position_id.is_some_and(|id| self.own.contains_key(&id));
        // Retain the original protobuf privately, before either silent filter.
        // No payload, label text, prices or client request ids enter the log.
        if owned {
            if let Err(err) = ts::record_event(self.pool, &crate::trading::event_row(&e)).await {
                tracing::error!(error = %err, "strategy 3 owned execution not recorded");
            }
        }
        // Codex · № 17,217 26-Oct: observed broker closing deals retain a
        // client id. Correlation alone does not make a fill our close request.
        if m.client_msg_id.is_some() {
            let allowed_close = if let Some(deal) = gate::closing_deal(self.account_id(), m) {
                async {
                    let known = owned
                        || s3s::has_open_fill(self.pool, self.account_id(), deal.position_id)
                            .await?;
                    let recorded_close = s3s::is_local_close_deal(
                        self.pool,
                        self.account_id(),
                        deal.position_id,
                        deal.deal_id,
                    )
                    .await?;
                    Ok::<_, jiaoyi_store::sqlx::Error>(known && !recorded_close)
                }
                .await
            } else {
                Ok(false)
            };
            match allowed_close {
                Ok(true) => {}
                Ok(false) => {
                    if owned {
                        self.warn_execution_drop(&e, "client_msg_id_present");
                    }
                    return;
                }
                Err(error) => {
                    tracing::error!(account = self.account_id(), error = %error, "strategy 3 close ownership lookup failed");
                    return;
                }
            }
        }
        if e.label.as_deref() != Some(LABEL) {
            if owned {
                self.warn_execution_drop(&e, "label_mismatch");
            }
            return;
        }
        if !owned {
            if let Err(err) = ts::record_event(self.pool, &crate::trading::event_row(&e)).await {
                tracing::error!(error = %err, "strategy 3 server execution event not recorded");
            }
        }
        if let Err(err) = self.broker_exit(m).await {
            tracing::error!(error = %err, "strategy 3 broker exit not recorded");
        }
    }

    // Codex · № 17,225 26-Oct: actual broker absence plus independently synced deals.
    async fn recover_history_exits(&self, broker_positions: &[i64]) -> anyhow::Result<()> {
        let account = self.account_id();
        let mut recovered = 0;
        for exit in s3s::history_exits(self.pool, account, broker_positions).await? {
            if !self
                .can_record_broker_exit(exit.position_id, exit.deal_id)
                .await?
            {
                continue;
            }
            let volume = match exit.closed_volume {
                Some(v) if v > 0 && v <= exit.filled_volume => v,
                _ => {
                    tracing::warn!(
                        account,
                        position = exit.position_id,
                        deal = exit.deal_id,
                        "strategy 3 history exit has unknown closed volume"
                    );
                    continue;
                }
            };
            let key = format!("s3-broker:{account}:{}:{}", exit.position_id, exit.deal_id);
            let detail = json!({"strategy": STRATEGY, "reason": "history_reconciled",
                "evidence_source": "broker_history", "deal_id": exit.deal_id,
                "order_id": exit.order_id, "open_entry_key": exit.entry_key});
            let inserted = s3s::record_broker_exit(
                self.pool,
                &s3s::OrderRow {
                    account_id: account,
                    entry_key: key.clone(),
                    label: LABEL.into(),
                    prereg_hash: exit.prereg_hash.clone(),
                    kind: "broker_exit".into(),
                    symbol: exit.symbol.clone(),
                    symbol_id: exit.symbol_id,
                    position_id: Some(exit.position_id),
                    side: exit.side.clone(),
                    session_date: exit.session_date,
                    decided_at: exit.executed_at,
                    bid: None,
                    ask: None,
                    volume: Some(volume),
                    detail: detail.clone(),
                },
                &s3s::FillRow {
                    account_id: account,
                    entry_key: key,
                    label: LABEL.into(),
                    prereg_hash: exit.prereg_hash,
                    kind: "broker_exit".into(),
                    symbol: exit.symbol,
                    position_id: Some(exit.position_id),
                    side: exit.side,
                    outcome: "filled".into(),
                    sent_at: exit.executed_at,
                    answered_at: Some(exit.executed_at),
                    latency_ms: None,
                    decision_price: None,
                    fill_price: exit.fill_price,
                    slippage: None,
                    spread: None,
                    volume: Some(volume),
                    detail,
                },
            )
            .await?;
            recovered += i64::from(inserted);
        }
        if recovered > 0 {
            tracing::info!(account, recovered, "strategy 3 history exits recovered");
        }
        Ok(())
    }

    /// Stream and history must use the same durable close-accounting boundary.
    async fn can_record_broker_exit(&self, position: i64, deal: i64) -> anyhow::Result<bool> {
        let account = self.account_id();
        let (local_deal, unsettled) =
            s3s::close_accounting(self.pool, account, position, deal).await?;
        if local_deal {
            return Ok(false);
        }
        if unsettled {
            tracing::warn!(
                account,
                position,
                deal,
                reason = "unsettled_local_close",
                "strategy 3 broker exit accounting deferred"
            );
            return Ok(false);
        }
        Ok(true)
    }

    fn warn_execution_drop(&self, event: &EventRecord, reason: &str) {
        tracing::warn!(
            account = self.account_id(), position = ?event.position_id,
            deal = ?event.deal_id, payload_type = event.payload_type,
            reason, "strategy 3 owned execution skipped"
        );
    }

    /// A closing deal the broker executed itself (the stop, the take-profit
    /// it holds, a stop-out, or a close made outside this program), once per
    /// deal: its fill against the level it executed, and the spread of the
    /// last quote.
    // Claude · № 17,020 25-Oct (Codex P2 on #15): broker exits are costs too.
    async fn broker_exit(&self, m: &ReceivedExecution) -> anyhow::Result<()> {
        let account_id = self.account_id();
        // An opening deal (a late fill of an open) is not an exit.
        let Some(deal) = gate::closing_deal(account_id, m) else {
            if let Some(e) = gate::decode_execution(account_id, m) {
                if e.position_id.is_some_and(|id| self.own.contains_key(&id)) {
                    self.warn_execution_drop(&e, "no_closing_fill_detail");
                }
            }
            return Ok(());
        };
        if !self
            .can_record_broker_exit(deal.position_id, deal.deal_id)
            .await?
        {
            return Ok(());
        }
        let own = self.own.get(&deal.position_id);
        let side = deal.position_side.or(own.map(|o| o.side)).or_else(|| {
            deal.deal_side.map(|s| match s {
                Side::Buy => Side::Sell,
                Side::Sell => Side::Buy,
            })
        });
        let Some(side) = side else {
            if own.is_some() {
                if let Some(e) = gate::decode_execution(account_id, m) {
                    self.warn_execution_drop(&e, "no_side");
                }
            }
            return Ok(());
        };
        let fill = deal.price;
        let stop = deal.stop_loss.or(own.and_then(|o| o.stop));
        let target = deal.take_profit.or(own.and_then(|o| o.take_profit));
        // The level it executed (the nearer of the stop and the
        // take-profit) and the slippage against it, by the rule both
        // strategies share. Positive is adverse.
        // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
        let (reason, decision_price) = crate::fills::exit_level(&deal, stop, target);
        let slippage = crate::fills::slippage(crate::fills::exit_side(side), fill, decision_price);
        let sym = self.syms.get(&deal.symbol_id);
        let (bid, ask) = (sym.and_then(|s| s.bid), sym.and_then(|s| s.ask));
        let name = sym.map(|s| s.name.clone()).unwrap_or_default();
        let key = format!(
            "s3-broker:{account_id}:{}:{}",
            deal.position_id, deal.deal_id
        );
        let scored = (|| {
            let o = own?;
            let entry = o.entry.or_else(|| o.plan_basis.map(|b| b.entry))?;
            let r = o.plan_basis.map(|b| b.r)?;
            close_result(side, entry, r, fill?)
        })();
        let mut detail = json!({"strategy": STRATEGY, "reason": reason, "deal_id": deal.deal_id,
            "order_id": deal.order_id, "level": decision_price,
            "open_entry_key": own.and_then(|o| o.entry_key.clone())});
        if let Some(s) = scored {
            detail["result_r"] = json!(s.result_r);
            detail["banks_a_win"] = json!(s.banks_a_win);
        }
        let new = s3s::record_broker_exit(
            self.pool,
            &s3s::OrderRow {
                account_id,
                entry_key: key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "broker_exit".into(),
                symbol: name.clone(),
                symbol_id: deal.symbol_id,
                position_id: Some(deal.position_id),
                side: Some(side.as_str().into()),
                session_date: own.and_then(|o| o.session_date),
                decided_at: m.received_at,
                bid,
                ask,
                volume: Some(deal.volume),
                detail: detail.clone(),
            },
            &s3s::FillRow {
                account_id,
                entry_key: key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "broker_exit".into(),
                symbol: name.clone(),
                position_id: Some(deal.position_id),
                side: Some(side.as_str().into()),
                outcome: "filled".into(),
                sent_at: m.received_at,
                answered_at: Some(m.received_at),
                latency_ms: None,
                decision_price,
                fill_price: fill,
                slippage,
                spread: match (bid, ask) {
                    (Some(b), Some(a)) => Some(a - b),
                    _ => None,
                },
                volume: Some(deal.volume),
                detail: detail.clone(),
            },
        )
        .await?;
        if new {
            let why = match scored {
                Some(s) => format!("{reason} {:.2}R banks_a_win={}", s.result_r, s.banks_a_win),
                None => reason.to_string(),
            };
            let mut d = Decisions::new(self.pool, account_id);
            d.hash = self.hash.clone();
            d.note(Some(&name), "close", &why, detail.clone()).await;
            tracing::info!(
                account = account_id,
                position = deal.position_id,
                reason,
                "strategy 3 broker exit recorded"
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- loop

/// Run every permitted account's Strategy 3 loop until shutdown.
pub async fn run(
    pool: PgPool,
    cfg: Arc<Config>,
    live: crate::trading::LivenessMap,
    shutdown: Shutdown,
) {
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
    live: &crate::trading::LivenessMap,
    mut shutdown: Shutdown,
) {
    crate::trading::beat(live, account.id(), |l| l.started_at = Some(Utc::now()));
    loop {
        if shutdown.is_set() {
            return;
        }
        let r = session(pool, cfg, account, env, live, shutdown.clone()).await;
        if shutdown.is_set() {
            return;
        }
        if let Err(e) = r {
            let mut d = Decisions::new(pool, account.id());
            d.note(
                None,
                "error",
                &format!("the Strategy 3 session stopped: {e:#}"),
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

async fn session(
    pool: &PgPool,
    cfg: &Config,
    account: &PermittedAccount,
    env: Environment,
    live: &crate::trading::LivenessMap,
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
        live,
        &mut ends,
        &mut shutdown,
    )
    .await
}

/// The Strategy 3 loop on an authorised session.
pub async fn run_session(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    live: &crate::trading::LivenessMap,
    ends: &mut tokio::sync::broadcast::Receiver<jiaoyi_ctrader::SessionEnd>,
    shutdown: &mut Shutdown,
) -> anyhow::Result<()> {
    run_session_with_event_offset(
        pool,
        allowlist,
        env,
        session,
        live,
        ends,
        shutdown,
        chrono::Duration::zero(),
    )
    .await
}

/// The production loop with a deterministic event-clock offset for its
/// scripted-broker test. Production preserves the actual received clock.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub async fn run_session_with_event_offset(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    live: &crate::trading::LivenessMap,
    ends: &mut tokio::sync::broadcast::Receiver<jiaoyi_ctrader::SessionEnd>,
    shutdown: &mut Shutdown,
    offset: chrono::Duration,
) -> anyhow::Result<()> {
    let account_id = session.account().id();
    crate::trading::beat(live, account_id, |l| {
        l.started_at.get_or_insert_with(Utc::now);
    });
    let conn = session.connection();
    let _observer =
        crate::fast_monitor::SessionObserver::start(conn, live, account_id, shutdown.clone());
    let initializing = crate::fast_monitor::WorkGuard::start(live, account_id, "initializing");
    let mut t = Trader::new(pool, allowlist, env, session).await?;
    t.live = live.clone();
    t.protect_during_reads = true;
    t.read_shutdown = Some(shutdown.clone());
    t.event_offset = offset;
    if t.syms.is_empty() {
        anyhow::bail!("no resolved symbols stored for this account yet (a data load comes first)");
    }
    t.reconcile(Utc::now()).await?;
    let ids: Vec<i64> = t.syms.keys().copied().collect();
    let mut spots = conn.spot_events();
    let mut execs = conn.executions();
    session.subscribe_spots(&ids).await?;
    let r0 = rules(pool, account_id, env).await?;
    let mut reconcile_tick = tokio::time::interval(Duration::from_secs(r0.reconcile_secs));
    reconcile_tick.tick().await;
    let mut clock = tokio::time::interval(Duration::from_secs(1));
    let mut exits = tokio::time::interval(crate::fast_monitor::INTERVAL);
    exits.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let specs = preopen::specs(pool, account_id, &t.syms, t.deposit.is_some()).await?;
    let (prepared_tx, mut prepared_rx) = tokio::sync::watch::channel(HashMap::new());
    let preparation = preopen::run(pool, session, specs, prepared_tx, shutdown.clone(), offset);
    tokio::pin!(preparation);
    initializing.complete();
    let management = async {
        loop {
            tokio::select! {
                changed=prepared_rx.changed()=>{
                    if changed.is_err() {return Ok(())}
                    let ready=prepared_rx.borrow_and_update().clone();
                    for p in ready.values() {
                        if let Some(sym)=p.symbol_id.and_then(|id|t.syms.get_mut(&id)) {sym.preopen=Some(p.clone());}
                    }
                },
                _ = shutdown.wait() => return Ok(()),
                e = ends.recv() => {
                    if let Ok(e) = e {
                        if e.affects(account_id) {
                            anyhow::bail!("the broker ended the session: {}", e.reason);
                        }
                    }
                }
                _ = reconcile_tick.tick() => crate::fast_monitor::observe(live, account_id, "reconcile", t.reconcile(Utc::now())).await?,
                _ = clock.tick() => crate::fast_monitor::observe(live, account_id, "session_exit", t.session_end(Utc::now()+offset)).await?,
                _ = exits.tick() => crate::fast_monitor::observe(live,account_id,"registered_exits",t.check_exits(Utc::now()+offset)).await?,
                m = execs.recv() => match m {
                    Ok(m) => {
                        crate::fast_monitor::observe(live, account_id, "execution", t.server_event(&m)).await;
                        crate::fast_monitor::execution_processed(live, account_id, m.received_at);
                    },
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
                },
                s = spots.recv() => match s {
                    Ok(spot) => {
                        let ev = spot.event;
                        if ev.ctid_trader_account_id != account_id {
                            continue;
                        }
                        crate::fast_monitor::observe(live, account_id, "quote", t.on_quote(
                            ev.symbol_id,
                            ev.bid.map(|b| b as f64 / 100_000.0),
                            ev.ask.map(|a| a as f64 / 100_000.0),
                            spot.received_at+offset,
                        ))
                        .await?;
                        crate::fast_monitor::quote_processed(live, account_id, spot.received_at);
                        crate::trading::beat(live, account_id, |l| l.last_quote_at = Some(Utc::now()));
                    }
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
                },
            }
        }
    };
    tokio::pin!(management);
    // Poll the read-only worker even while an actor handler awaits the
    // database or an execution receipt. Only the actor owns trading state.
    tokio::select! {result=&mut management=>result, result=&mut preparation=>result}
}

// ---------------------------------------------------------------- health

/// `/health` → `strategy3`: per account, the label, the pre-registration,
/// the open positions at the last reconcile and today's decisions and
/// trades. No money.
pub async fn health(
    pool: &PgPool,
    live: &HashMap<i64, crate::trading::Liveness>,
) -> jiaoyi_store::Result<Value> {
    let accounts = jiaoyi_store::sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for a in accounts {
        let id: i64 = a.get("account_id");
        let env: String = a.get("environment");
        let environment = if env == "live" {
            Environment::Live
        } else {
            Environment::Demo
        };
        // In memory: every reconcile run and the last quote, recorded or not
        // (as Strategy 1's).
        let r = rules(pool, id, environment).await.ok();
        let reconcile_secs = r.as_ref().map(|r| r.reconcile_secs);
        let l = live.get(&id);
        let liveness = json!({
            "broker_snapshot": crate::fast_monitor::broker_snapshot(pool, id).await,
            "state": reconcile_secs.map(|s| crate::trading::liveness_state(l, s, Utc::now())),
            "stalled_after_secs": reconcile_secs.map(|s| 2 * s),
            "started_at": l.and_then(|l| l.started_at),
            "last_reconcile_run_at": l.and_then(|l| l.last_reconcile_run_at),
            "last_quote_at": l.and_then(|l| l.last_quote_at),
        });
        let rec = ts::latest_decision_detail(pool, id, "s3_reconcile").await?;
        let today = s3s::today(pool, id).await?;
        out.push(json!({
            "account_id": id,
            "environment": env,
            "strategy": STRATEGY,
            "label": LABEL,
            "trades": if r.as_ref().is_some_and(|r| r.paused) { "paused by the owner: no new entry" } else { "yes" },
            "sizing": if env == "live" { "one broker-minimum trade at a time" } else { "risk_pct of fresh equity" },
            "prereg_file": PREREG_FILE,
            "prereg_hash": prereg_hash(),
            "open_positions": rec.as_ref().map(|(_, d)| d["positions"].clone()).unwrap_or(json!([])),
            "last_reconcile_at": rec.map(|(at, _)| at),
            "today": today,
            "preopen_preparation": preopen::health(pool,id,Utc::now()).await.unwrap_or_else(|_|json!({"state":"unavailable"})),
            "liveness": liveness,
            // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
            "performance": crate::limits::health(pool, id, &crate::limits::S3).await.unwrap_or_else(|e| json!({"error": e.to_string()})),
            "owner_pause": crate::limits::owner_pause(pool, id, &crate::limits::S3).await.unwrap_or_else(|e| json!({"error": e.to_string()})),
            "trigger_distance": l.map(|l| json!(l.triggers)).unwrap_or(json!({})),
        }));
    }
    Ok(json!(out))
}

/// Read-only diagnostic input. Ownership comes from the opening intent and
/// its confirmed fill, not a closing action's distinct entry key. Stored deal
/// volumes must prove full closure; this is not the official net-R evaluator.
pub(crate) async fn recent_closed_signals(
    pool: &PgPool,
    accounts: &[i64],
) -> Result<Vec<jiaoyi_store::sqlx::postgres::PgRow>, jiaoyi_store::sqlx::Error> {
    jiaoyi_store::sqlx::query(
        r#"WITH owned AS (
          SELECT o.account_id,o.entry_key,o.symbol,o.side,o.session_date,o.prereg_hash,
            o.detail #>> '{band,minutes}' AS mark_minutes,e.position_id,
            MAX(e.volume) AS observed_open
          FROM strategy3_orders o
          JOIN trading_intents i ON i.account_id=o.account_id AND i.entry_key=o.entry_key AND i.kind='open'
          JOIN trading_events e ON e.account_id=i.account_id AND e.entry_key=i.entry_key
          WHERE o.account_id=ANY($1) AND o.kind='open' AND o.label='jiaoyi-s3'
            AND i.entry_key LIKE 's3:%' AND e.kind=ANY($2) AND e.volume>0
            AND e.position_id IS NOT NULL AND e.received_at<=now()
          GROUP BY o.account_id,o.entry_key,o.symbol,o.side,o.session_date,o.prereg_hash,
            o.detail #>> '{band,minutes}',e.position_id
        )
        SELECT o.*,h.closed_at,
          EXISTS(SELECT 1 FROM strategy3_fills f
            WHERE f.account_id=o.account_id AND f.position_id=o.position_id
              AND f.kind='broker_exit' AND f.outcome IN ('filled','partly_filled')
              AND f.detail->>'reason'='stop_loss'
              AND COALESCE(f.answered_at,f.sent_at)<=now()) AS stopped
        FROM owned o
        JOIN LATERAL (
          SELECT COALESCE(SUM(d.filled_volume) FILTER(WHERE NOT d.is_closing),0) AS opened,
            COALESCE(SUM(d.filled_volume) FILTER(WHERE d.is_closing),0) AS closed,
            MAX(d.executed_at) FILTER(WHERE d.is_closing) AS closed_at,
            BOOL_AND(d.deal_status IN ('FILLED','PARTIALLY_FILLED')) AS valid
          FROM deals d WHERE d.account_id=o.account_id AND d.position_id=o.position_id
            AND d.filled_volume>0 AND d.executed_at<=now()
        ) h ON h.opened>0 AND h.opened=h.closed AND h.opened>=o.observed_open AND h.valid
        WHERE h.closed_at>=now()-interval '14 days'
          AND NOT EXISTS(SELECT 1 FROM owned other WHERE other.account_id=o.account_id
            AND other.position_id=o.position_id AND other.entry_key<>o.entry_key)
        ORDER BY h.closed_at DESC,o.account_id,o.position_id"#,
    )
    .bind(accounts)
    .bind(jiaoyi_store::trading::FILL_KINDS)
    .fetch_all(pool)
    .await
}

/// Copied accounts are one signal episode, separated by registration and mark.
/// Recent means latest actual broker close, rather than symbol alphabetic order.
pub async fn episodes(pool: &PgPool, accounts: &[i64]) -> jiaoyi_store::Result<Value> {
    let rows = recent_closed_signals(pool, accounts).await?;
    Ok(episodes_from_rows(&rows))
}

/// Project the same fresh request snapshot used by recovery diagnostics.
pub(crate) fn episodes_from_rows(rows: &[jiaoyi_store::sqlx::postgres::PgRow]) -> Value {
    let mut groups: std::collections::BTreeMap<String, (DateTime<Utc>, Value)> =
        std::collections::BTreeMap::new();
    for row in rows {
        let account: i64 = row.get("account_id");
        let symbol: String = row.get("symbol");
        let side: Option<String> = row.get("side");
        let session: Option<chrono::NaiveDate> = row.get("session_date");
        let mark: Option<String> = row.get("mark_minutes");
        let hash: String = row.get("prereg_hash");
        let at: DateTime<Utc> = row.get("closed_at");
        // Missing signal identity cannot prove copies of one signal.
        let identity = match (&side, session, &mark) {
            (Some(side), Some(session), Some(mark)) => format!("{side}|{session}|{mark}"),
            _ => format!(
                "unclassified:{account}:{}",
                row.get::<String, _>("entry_key")
            ),
        };
        let key = format!("{hash}|{symbol}|{identity}");
        let (closed_at, episode) = groups.entry(key).or_insert_with(|| {
            (
                at,
                json!({"symbol":symbol,"side":side,"session_date":session,
                "mark_minutes":mark,"prereg_hash":hash,"accounts":[],"copies":0}),
            )
        });
        *closed_at = (*closed_at).max(at);
        let accounts = episode["accounts"].as_array_mut().unwrap();
        if !accounts.contains(&json!(account)) {
            accounts.push(json!(account));
        }
    }
    let mut ordered: Vec<_> = groups.into_iter().collect();
    ordered.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then(a.0.cmp(&b.0)));
    let copied = ordered
        .iter()
        .filter(|(_, (_, e))| e["accounts"].as_array().unwrap().len() > 1)
        .count();
    let total = ordered.len();
    let latest: Vec<Value> = ordered
        .into_iter()
        .take(20)
        .map(|(_, (at, mut e))| {
            e["copies"] = json!(e["accounts"].as_array().unwrap().len());
            e["last_closed_at"] = json!(at);
            e
        })
        .collect();
    json!({"available":true,
        "note":"stored complete-volume owned positions over 14 days; pending history excluded; official net-R latest-20 stays per strategy/account",
        "episodes":total,"latest20_episodes":latest.len(),"copied":copied,"rows":latest})
}

#[cfg(test)]
mod tests {
    #[test]
    fn review_existing_three_r_plan_keeps_its_original_close_allocation() {
        let detail = json!({"entry":{"side":"buy","entry_price":60460.0,"r":450.0,"cost":{"total":301.208}},"order":{"tp1_distance":1651.21}});
        let b = plan_basis(&detail).unwrap();
        assert_eq!(tp1_close_volume(&b, &BTC, 13), 5);
        assert_eq!(tp1_close_volume(&b, &BTC, 4), 2);
    }

    #[test]
    fn review_a_late_mark_uses_its_own_sigma_without_the_next_mark() {
        let s = sessions::session_on(
            &sessions::rule_for("BTCUSD", AssetClass::Crypto).unwrap(),
            "2026-10-27".parse().unwrap(),
        )
        .unwrap();
        let table = SigmaTable {
            marks: [(30, Ok(0.01))].into(),
            sessions_used: vec![],
        };
        let x = signal(
            &table,
            &s,
            100.0,
            s.open + chrono::Duration::minutes(30) + chrono::Duration::milliseconds(1),
            Some(102.0),
            Some(102.1),
        )
        .unwrap()
        .unwrap();
        assert_eq!(x.1.minutes, 30.0);
        assert_eq!(x.1.sigma, 0.01);
    }
    use super::*;

    fn rules_demo() -> Rules {
        Rules {
            version: 1,
            live: false,
            risk_pct: 0.6,
            cap_factor: 1.1,
            min_volume_max_risk_pct: 10.0,
            open_risk_pct: 3.0,
            reconcile_secs: 300,
            order_wait_secs: 20,
            retry_secs: 60,
            paused: false,
        }
    }

    const BTC: VolumeLimits = VolumeLimits {
        min: 1,
        step: 1,
        max: None,
        digits: 2,
    };

    /// The fixture's entry: a buy at 60,460.00, R 450.00, C 301.208.
    /// Q = 1.38, so TP1 distance = 1.38×450 + 301.208 = 922.208.
    /// f = 751.208 / 1,372.208. Expected closes from that formula.
    #[test]
    fn tp1_s_close_follows_the_volume_held() {
        let b = PlanBasis {
            side: Dir::Buy,
            entry: 60_460.0,
            r: 450.0,
            cost: 301.208,
            tp1_distance: 922.21,
        };
        let close = |v| tp1_close_volume(&b, &BTC, v);
        assert_eq!(close(13), 8, "the planned close");
        assert_eq!(close(4), 3);
        assert_eq!(close(2), 2, "the rest would be below the minimum: whole");
        assert_eq!(close(1), 1, "unsplittable: the whole position");
        let sell = PlanBasis {
            side: Dir::Sell,
            ..b
        };
        assert_eq!(tp1_close_volume(&sell, &BTC, 13), 8);
        let detail = json!({"entry": {"side": "sell", "entry_price": 60_455.0, "r": 450.0,
                                       "cost": {"total": 301.208}}, "order":{"tp1_distance":922.21}});
        assert_eq!(
            plan_basis(&detail),
            Some(PlanBasis {
                side: Dir::Sell,
                entry: 60_455.0,
                r: 450.0,
                cost: 301.208,
                tp1_distance: 922.21
            })
        );
        assert_eq!(plan_basis(&json!({"entry": {"side": "buy"}})), None);
    }

    #[test]
    fn the_embedded_pre_registration_is_the_file() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/preregistration-2026-10-28-strategy3-v6.md"
        );
        let disk = std::fs::read(path).unwrap();
        let disk_hash: String = Sha256::digest(&disk)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(disk_hash, prereg_hash());
        assert!(PREREG_TEXT.contains("one entry"));
        assert!(PREREG_TEXT.contains("cost skip"));
        assert!(PREREG_TEXT.contains("Do not refit the band"));
        assert!(session_already_entered(1));
        assert!(!session_already_entered(0));
        let day = chrono::NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        assert!(cost_skip_stands(Some(day), day));
        assert!(!cost_skip_stands(None, day));
        assert!(!cost_skip_stands(
            Some(chrono::NaiveDate::from_ymd_opt(2026, 10, 27).unwrap()),
            day
        ));
    }

    #[test]
    fn demo_sizes_risk_pct_and_live_takes_one_minimum() {
        // $10,000 at 0.6% = $60; a 200.00 stop on 1 cent (0.01 BTC) risks $2.
        let s = size(&rules_demo(), 10_000.0, 20_000_000, 1.0, &BTC, 0).unwrap();
        assert_eq!((s.volume, s.mode), (30, "risk_pct"));
        assert!((s.risk - 60.0).abs() < 1e-9);
        // Below the minimum on demo: skipped.
        let e = size(&rules_demo(), 100.0, 20_000_000, 1.0, &BTC, 0).unwrap_err();
        assert!(e.contains("below the broker minimum"), "{e}");
        let live = Rules {
            live: true,
            ..rules_demo()
        };
        // Live: exactly the minimum (2.00 risk = 5.1% of 38.98 ≤ 10%).
        let s = size(&live, 38.98, 20_000_000, 1.0, &BTC, 0).unwrap();
        assert_eq!((s.volume, s.mode), (1, "live_broker_minimum"));
        assert!((s.max_risk - 3.898).abs() < 1e-9);
        // Only while no other position of this program is open.
        let e = size(&live, 38.98, 20_000_000, 1.0, &BTC, 1).unwrap_err();
        assert!(e.contains("one minimum trade at a time"), "{e}");
        // Above 10% of equity: skipped.
        let e = size(&live, 15.0, 20_000_000, 1.0, &BTC, 0).unwrap_err();
        assert!(e.contains("above 10% of equity"), "{e}");
    }

    #[test]
    fn the_budget_holds_strategy_3_within_its_share() {
        assert!(within_budget(200.0, 100.0, 10_000.0, 3.0).is_ok());
        let e = within_budget(250.0, 100.0, 10_000.0, 3.0).unwrap_err();
        assert!(e.contains("exceed its budget"), "{e}");
    }

    #[test]
    fn the_order_carries_tp2_and_tp1_as_distances_from_the_fill() {
        // bot-trade's numbers: entry 100, R 10, C 0.4, 10,000 cents.
        let e = Entry {
            side: Dir::Buy,
            band: Band {
                open: 95.0,
                sigma: 0.1,
                lower: 85.5,
                upper: 99.9,
                half_width: 10.0,
                minutes: 45.0,
            },
            entry_price: 100.0,
            relative_stop: 1_000_000,
            r: 10.0,
            cost: CostReserve {
                spread: 0.4,
                commission_round_trip: 0.0,
                slippage: 0.0,
                total: 0.4,
            },
        };
        let lim = VolumeLimits {
            min: 100,
            step: 100,
            max: None,
            digits: 2,
        };
        let o = plan_order(&e, &lim, 10_000).unwrap();
        // Q = 1.38: TP1 = 1.38×10 + 0.4 = 14.2; TP2 = 24.2; f = 10.4/24.2.
        assert_eq!(o.plan.close_volume, 4300);
        assert!((o.tp1_distance - 14.2).abs() < 1e-9);
        assert!((o.tp2_distance - 24.2).abs() < 1e-9);
        // The broker's take-profit is TP2, 24.20 from the fill.
        assert_eq!(o.relative_take_profit, 2_420_000);
        // Unsplittable: the broker's take-profit is TP1.
        let o = plan_order(&e, &lim, 100).unwrap();
        assert_eq!(o.plan.mode, targets::Mode::WholePositionMinimum);
        assert_eq!(o.relative_take_profit, 1_420_000);
        // A short whose TP1 would fall below zero: refused with bot-trade's
        // reason (target_price_invalid).
        let dear = Entry {
            side: Dir::Sell,
            cost: CostReserve {
                total: 1e3,
                ..e.cost
            },
            ..e.clone()
        };
        let why = plan_order(&dear, &lim, 10_000).unwrap_err();
        assert!(why.contains("target_price_invalid"), "{why}");
        // TP1 is reached on the bid for a long, on the ask for a short.
        assert!(tp1_reached(Side::Buy, 100.0, 30.4, 2, Some(130.4), None));
        assert!(!tp1_reached(
            Side::Buy,
            100.0,
            30.4,
            2,
            Some(130.39),
            Some(131.0)
        ));
        assert!(tp1_reached(Side::Sell, 100.0, 30.4, 2, None, Some(69.6)));
        assert!(!tp1_reached(
            Side::Sell,
            100.0,
            30.4,
            2,
            Some(1.0),
            Some(69.61)
        ));
        // Float sums that miss the grid by an ulp still compare as ticks.
        assert!(tp1_reached(
            Side::Buy,
            60_462.0,
            1_651.21,
            2,
            Some(62_113.21),
            None
        ));
    }

    #[test]
    fn conversion_comes_from_the_brokers_own_pair() {
        let mids = HashMap::from([("USDJPY".to_string(), 150.0), ("EURUSD".to_string(), 1.1)]);
        assert_eq!(quote_to_deposit("USD", "USD", &mids), Some(1.0));
        assert!((quote_to_deposit("JPY", "USD", &mids).unwrap() - 1.0 / 150.0).abs() < 1e-12);
        assert!((quote_to_deposit("EUR", "USD", &mids).unwrap() - 1.1).abs() < 1e-12);
        assert_eq!(quote_to_deposit("GBP", "USD", &mids), None);
    }

    #[test]
    fn chandelier_mae_mfe_and_reverse_are_distinct() {
        // Chandelier is an exit. It hangs from the best by R and never
        // loosens. It does not read MAE or MFE.
        let long = chandelier_stop(Side::Buy, 110.0, 5.0).unwrap();
        assert_eq!(long, 105.0);
        let short = chandelier_stop(Side::Sell, 90.0, 5.0).unwrap();
        assert_eq!(short, 95.0);
        assert!(chandelier_stop(Side::Buy, 110.0, 0.0).is_none());
        assert!(chandelier_stop(Side::Buy, f64::NAN, 5.0).is_none());

        // MAE and MFE are measurements. They grow, and they do not move a stop.
        let (mae, mfe) = update_excursions(Side::Buy, 100.0, 97.0, 0.0, 0.0);
        assert_eq!((mae, mfe), (3.0, 0.0));
        let (mae, mfe) = update_excursions(Side::Buy, 100.0, 108.0, mae, mfe);
        assert_eq!((mae, mfe), (3.0, 8.0));
        // A later price inside the extremes does not shrink either.
        let (mae, mfe) = update_excursions(Side::Buy, 100.0, 102.0, mae, mfe);
        assert_eq!((mae, mfe), (3.0, 8.0));
        // A short's adverse is up, its favourable is down.
        let (mae, mfe) = update_excursions(Side::Sell, 100.0, 104.0, 0.0, 0.0);
        assert_eq!((mae, mfe), (4.0, 0.0));
        let (mae, mfe) = update_excursions(Side::Sell, 100.0, 93.0, mae, mfe);
        assert_eq!((mae, mfe), (4.0, 7.0));
        // The Chandelier stop is unchanged by these numbers.
        assert_eq!(chandelier_stop(Side::Buy, 110.0, 5.0), Some(105.0));

        // The 28 Oct JPN225 stop-out. R is the original stop distance.
        // Fill 64155.3 is still a loss, so it does not bank a win.
        let stopped = close_result(Side::Sell, 64061.1, 327.5, 64155.3).unwrap();
        assert!((stopped.result_r - (64061.1 - 64155.3) / 327.5).abs() < 1e-12);
        assert!(!stopped.banks_a_win);
        assert!(stopped.result_r > -1.0 && stopped.result_r < 0.0);
        let banked = close_result(Side::Sell, 64061.1, 327.5, 63700.0).unwrap();
        assert!(banked.banks_a_win);
        assert!(close_result(Side::Buy, 100.0, 0.0, 99.0).is_none());

        // Reverse flips the side and is not the primary signal.
        assert_eq!(reverse_side(Dir::Buy), Dir::Sell);
        assert_eq!(reverse_side(Dir::Sell), Dir::Buy);
        assert_ne!(reverse_side(Dir::Buy), Dir::Buy);
    }

    #[test]
    fn a_grok_reading_is_kept_only_when_it_can_change_a_send() {
        let mut detail = json!({"side": "buy", "band": {"sigma": 1.0}});
        attach_sentiment(
            &mut detail,
            &json!({"effect": "block_a_send", "headline": "halt"}),
        );
        assert_eq!(detail["side"], "buy");
        assert_eq!(detail["grok"]["effect"], "block_a_send");
        let mut other = json!({"side": "sell"});
        attach_sentiment(
            &mut other,
            &json!({"effect": "measurement", "sentiment": "neutral"}),
        );
        attach_sentiment(&mut other, &json!({"effect": "order", "side": "buy"}));
        assert_eq!(other["side"], "sell");
        assert!(other.get("grok").is_none());
    }
}
