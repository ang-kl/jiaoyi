//! Pure whole-book checks. A pass is a named check, never a profitability claim.
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use jiaoyi_ctrader::messages as pb;
use serde::Serialize;
use serde_json::Value;

pub const MAX_QUOTE_AGE_SECS: i64 = 6;
pub const MAX_BOOK_AGE_SECS: i64 = 120;

#[derive(Debug, Clone, Default)]
pub struct Quote {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub bid_at: Option<DateTime<Utc>>,
    pub ask_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub symbol: pb::ProtoOaSymbol,
    pub light: pb::ProtoOaLightSymbol,
    pub quote_to_deposit: Option<f64>,
    pub conversion_at: Option<DateTime<Utc>>,
}

/// Supplied only after matching a positive original instruction to this position.
#[derive(Debug, Clone, Serialize)]
pub struct StrategyEvidence {
    pub strategy: String,
    pub side: String,
    pub detail: Value,
    pub recorded_at: DateTime<Utc>,
    pub original_volume: i64,
    pub entry_key: String,
}

#[derive(Debug, Clone)]
pub struct PendingMargin {
    pub projected_margin: f64,
    pub observed_at: DateTime<Utc>,
    pub revision: u64,
    pub warning_pct: Option<f64>,
    pub by_order: HashMap<i64, f64>,
}

#[derive(Debug, Clone)]
pub struct PermissionEvidence {
    pub observed_at: DateTime<Utc>,
    pub book_revision: u64,
    pub owner_paused: Option<bool>,
    pub cost_veto_by_symbol: HashMap<i64, bool>,
    pub entry_recorded_by_symbol: HashMap<i64, bool>,
    pub live_single_position_blocked: Option<bool>,
    pub s1_edge_passed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub account_id: i64,
    pub generation: u64,
    pub revision: u64,
    pub now: DateTime<Utc>,
    pub book_at: Option<DateTime<Utc>>,
    pub complete: bool,
    pub connected: bool,
    pub positions: Vec<pb::ProtoOaPosition>,
    pub orders: Vec<pb::ProtoOaOrder>,
    pub quotes: HashMap<i64, Quote>,
    pub symbols: HashMap<i64, SymbolInfo>,
    pub trader: Option<pb::ProtoOaTrader>,
    pub strategy: HashMap<i64, StrategyEvidence>,
    pub margin_floor_pct: Option<f64>,
    pub equity: Option<f64>,
    pub equity_at: Option<DateTime<Utc>>,
    pub gross_pnl_by_position: HashMap<i64, f64>,
    pub pending_margin: Option<PendingMargin>,
    pub permissions: HashMap<String, PermissionEvidence>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Check {
    pub name: String,
    pub state: String,
    /// Stable reasons permit transition-only persistence. Tick values stay separate.
    pub reason: String,
    pub value: Option<f64>,
}

fn check(name: &str, state: &str, reason: &str, value: Option<f64>) -> Check {
    Check {
        name: name.into(),
        state: state.into(),
        reason: reason.into(),
        value: value.filter(|v| v.is_finite()),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectAssessment {
    pub kind: String,
    pub id: i64,
    pub position_id: Option<i64>,
    pub symbol_id: i64,
    pub symbol: Option<String>,
    pub side: String,
    pub volume: i64,
    pub units: f64,
    pub lots: Option<f64>,
    pub entry: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub strategy: Option<StrategyEvidence>,
    pub checks: Vec<Check>,
    /// Gross nominal cash from original entry to current stop, not guaranteed loss.
    pub nominal_loss_at_stop: Option<f64>,
    pub nominal_pnl_at_stop: Option<f64>,
    pub nominal_reward_at_target: Option<f64>,
    pub quote_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Assessment {
    pub account_id: i64,
    pub generation: u64,
    pub revision: u64,
    pub assessed_at: DateTime<Utc>,
    pub book_at: Option<DateTime<Utc>>,
    pub evidence_complete: bool,
    pub interval_secs: u64,
    pub no_trade_authority: bool,
    pub position_count: usize,
    pub pending_entry_count: usize,
    pub protective_order_count: usize,
    pub objects: Vec<ObjectAssessment>,
    pub account_checks: Vec<Check>,
    pub nominal_stop_risk: Option<f64>,
    /// Current broker equity giveback to stops, including open profits.
    pub stop_loss_reserve: Option<f64>,
    pub breach_count: usize,
    pub unknown_count: usize,
    /// Entry exclusions are separate from current setup/protection breaches.
    pub blocked_entry_check_count: usize,
}

fn positive(v: Option<f64>) -> Option<f64> {
    v.filter(|v| v.is_finite() && *v > 0.0)
}

fn fresh(at: Option<DateTime<Utc>>, now: DateTime<Utc>, secs: i64) -> bool {
    at.is_some_and(|at| at <= now && (now - at).num_milliseconds() <= secs * 1_000)
}

fn executable(q: Option<&Quote>, now: DateTime<Utc>) -> Option<(f64, f64)> {
    let q = q?;
    if !fresh(q.bid_at, now, MAX_QUOTE_AGE_SECS) || !fresh(q.ask_at, now, MAX_QUOTE_AGE_SECS) {
        return None;
    }
    let (bid, ask) = (positive(q.bid)?, positive(q.ask)?);
    (bid <= ask).then_some((bid, ask))
}

fn direction(side: i32) -> (&'static str, Option<f64>) {
    match side {
        1 => ("buy", Some(1.0)),
        2 => ("sell", Some(-1.0)),
        _ => ("unknown", None),
    }
}

fn market(info: Option<&SymbolInfo>, now: DateTime<Utc>) -> Check {
    let Some(info) = info else {
        return check("market", "unknown", "symbol metadata unavailable", None);
    };
    if info.light.enabled == Some(false) || info.symbol.trading_mode.is_some_and(|x| x != 0) {
        return check(
            "market",
            "breach",
            "symbol is not enabled for unrestricted new entries",
            None,
        );
    }
    if info.light.enabled.is_none() {
        return check(
            "market",
            "unknown",
            "broker symbol enabled flag unavailable",
            None,
        );
    }
    match crate::loader::market_state_of(&info.symbol, now) {
        jiaoyi_core::schedule::MarketState::Open => {
            check("market", "pass", "broker schedule is open", None)
        }
        jiaoyi_core::schedule::MarketState::Closed(_) => check(
            "market",
            "breach",
            "broker schedule is closed; no current entry eligibility",
            None,
        ),
        jiaoyi_core::schedule::MarketState::Unknown(_) => check(
            "market",
            "unknown",
            "broker session schedule is incomplete",
            None,
        ),
    }
}

fn strategy_check(e: Option<&StrategyEvidence>, side: &str, quote: Option<(f64, f64)>) -> Check {
    let Some(e) = e else {
        return check(
            "direction",
            "unknown",
            "no evidenced strategy assigned to this trade",
            None,
        );
    };
    if e.side != side {
        return check(
            "direction",
            "breach",
            "current side differs from evidenced original instruction",
            None,
        );
    }
    let Some((bid, ask)) = quote else {
        return check(
            "direction",
            "unknown",
            "fresh executable quote unavailable",
            None,
        );
    };
    let d = &e.detail;
    let supports = if matches!(e.strategy.as_str(), "s3" | "strategy3" | "jiaoyi-s3")
        || e.strategy == crate::strategy3::STRATEGY
    {
        let lower = positive(d["band"]["lower"].as_f64());
        let upper = positive(d["band"]["upper"].as_f64());
        match (lower, upper) {
            (Some(lo), Some(hi)) if hi > lo && ask - bid <= hi - lo => Some(if side == "buy" {
                (bid + ask) / 2.0 > hi
            } else {
                (bid + ask) / 2.0 < lo
            }),
            (Some(lo), Some(hi)) if hi > lo => Some(false),
            _ => None,
        }
    } else if matches!(e.strategy.as_str(), "s1" | "strategy1" | "jiaoyi-s1")
        || e.strategy == crate::trading::STRATEGY
    {
        match (
            positive(d["level"].as_f64()),
            positive(d["ema_fast"].as_f64()),
            positive(d["ema_slow"].as_f64()),
        ) {
            (Some(level), Some(fast), Some(slow)) => Some(if side == "buy" {
                ask >= level && fast > slow
            } else {
                bid <= level && fast < slow
            }),
            _ => None,
        }
    } else {
        None
    };
    match supports {
        Some(true) => check("direction", "pass", "current quote supports original frozen entry benchmark; not a new entry or exit instruction", None),
        Some(false) => check("direction", "breach", "current quote no longer supports original frozen entry benchmark; registered exits still govern", None),
        None => check("direction", "unknown", "original benchmark inputs are unavailable or unsupported", None),
    }
}

fn permission_checks(
    input: &Input,
    strategy: Option<&StrategyEvidence>,
    symbol: i64,
) -> Vec<Check> {
    let key = strategy.and_then(|e| {
        if matches!(e.strategy.as_str(), "s1" | "strategy1" | "jiaoyi-s1")
            || e.strategy == crate::trading::STRATEGY
        {
            Some("s1")
        } else if matches!(e.strategy.as_str(), "s3" | "strategy3" | "jiaoyi-s3")
            || e.strategy == crate::strategy3::STRATEGY
        {
            Some("s3")
        } else {
            None
        }
    });
    let Some(key) = key else {
        return vec![check(
            "entry_permission",
            "unknown",
            "no evidenced governing strategy; permission for another entry is not inferred",
            None,
        )];
    };
    let source = input.permissions.get(key).filter(|p| {
        input.complete
            && input.connected
            && p.book_revision == input.revision
            && fresh(Some(p.observed_at), input.now, MAX_BOOK_AGE_SECS)
            && fresh(input.book_at, input.now, MAX_BOOK_AGE_SECS)
    });
    let verdict = |name: &str, blocked: Option<bool>, yes: &str, no: &str| {
        match blocked {
        Some(true) => check(name, "blocked", yes, None),
        Some(false) => check(name, "pass", no, None),
        None => check(name, "unknown", "fresh same-revision permission evidence unavailable; existing exits remain governed by their rules", None),
    }
    };
    let observed_owned_position =
        (input.complete && input.connected && fresh(input.book_at, input.now, MAX_BOOK_AGE_SECS))
            .then_some(true);
    let mut out = vec![
        verdict("strategy_symbol_exposure_new_entry", observed_owned_position,
            "an evidenced position of this strategy already occupies the symbol; another entry is blocked while registered exits continue",
            "no evidenced strategy exposure blocks another entry"),
        verdict("owner_pause_new_entry", source.and_then(|p| p.owner_paused),
            "owner pause blocks new entries only; existing position exits continue",
            "owner pause does not block new entries; all other entry gates still apply"),
        verdict("live_capacity_new_entry", source.and_then(|p| p.live_single_position_blocked),
            "whole-account live single-position gate blocks another entry; existing exits continue",
            "live single-position gate does not block this account snapshot; other capacity checks still apply"),
    ];
    if key == "s3" {
        out.push(verdict("durable_cost_veto_new_entry", source.and_then(|p| p.cost_veto_by_symbol.get(&symbol).copied()),
            "registered durable session cost veto blocks a new entry; it is not an exit instruction",
            "no durable cost veto recorded for the applicable symbol/session; current entry cost checks still apply"));
        out.push(verdict("session_entry_new_entry", source.and_then(|p| p.entry_recorded_by_symbol.get(&symbol).copied()),
            "a recorded entry already consumed this symbol/session; repeat entry is blocked while existing exits continue",
            "no prior entry recorded for the applicable symbol/session; other entry rules still apply"));
    } else {
        out.push(verdict(
            "s1_edge_new_entry",
            source.and_then(|p| p.s1_edge_passed.map(|pass| !pass)),
            "S1 edge gate blocks new entries; existing registered exits continue",
            "S1 edge gate permits assessment of a new entry; this is not a profitability guarantee",
        ));
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn assess_object(
    input: &Input,
    kind: &str,
    id: i64,
    position_id: Option<i64>,
    trade: &pb::ProtoOaTradeData,
    entry: Option<f64>,
    sl: Option<f64>,
    tp: Option<f64>,
    trigger_method: Option<i32>,
) -> ObjectAssessment {
    let (side, sign) = direction(trade.trade_side);
    let info = input.symbols.get(&trade.symbol_id);
    let q = input.quotes.get(&trade.symbol_id);
    let quote = input.connected.then(|| executable(q, input.now)).flatten();
    // A pending order can reference an existing position without belonging to
    // its original strategy (for example a manual increase). Position lineage
    // is not proof of this distinct order's intent.
    let evidence = (kind == "position")
        .then(|| position_id.and_then(|id| input.strategy.get(&id)))
        .flatten();
    let current_book =
        input.complete && input.connected && fresh(input.book_at, input.now, MAX_BOOK_AGE_SECS);
    let mut checks = vec![
        check(
            "book",
            if current_book { "pass" } else { "unknown" },
            if current_book {
                "complete broker book within observation age"
            } else {
                "broker book incomplete, disconnected or stale"
            },
            None,
        ),
        check(
            "quote",
            if quote.is_some() { "pass" } else { "unknown" },
            if quote.is_some() {
                "bid and ask are fresh and ordered"
            } else {
                "bid or ask missing, stale, future-dated or crossed"
            },
            None,
        ),
        market(info, input.now),
    ];
    let (quantity_state, quantity_reason) = if trade.volume <= 0 || sign.is_none() {
        ("breach", "broker quantity or side is invalid")
    } else if kind == "position" {
        (
            "pass",
            "positive held quantity; entry minimum is not reapplied to a partial residual",
        )
    } else if let Some(info) = info {
        match (
            info.symbol.min_volume,
            info.symbol.step_volume,
            info.symbol.max_volume,
        ) {
            (Some(min), Some(step), Some(max)) if min > 0 && step > 0 && max >= min => {
                if trade.volume >= min && trade.volume <= max && trade.volume % step == 0 {
                    (
                        "pass",
                        "pending quantity meets broker minimum, maximum and step",
                    )
                } else {
                    (
                        "breach",
                        "pending quantity violates broker minimum, maximum or step",
                    )
                }
            }
            _ => ("unknown", "broker quantity rules unavailable"),
        }
    } else {
        ("unknown", "broker quantity rules unavailable")
    };
    checks.push(check(
        "quantity",
        quantity_state,
        quantity_reason,
        Some(trade.volume as f64),
    ));
    let entry = positive(entry);
    let sl = positive(sl);
    let tp = positive(tp);
    let reference = if kind == "pending_entry" {
        entry
    } else {
        quote.and_then(
            |(bid, ask)| match (trade.trade_side, trigger_method.unwrap_or(1)) {
                (1, 1 | 3) | (2, 2 | 4) => Some(bid),
                (2, 1 | 3) | (1, 2 | 4) => Some(ask),
                _ => None,
            },
        )
    };
    checks.push(match (sl, reference, sign) {
        (None, _, _) if current_book => check(
            "stop_loss",
            "breach",
            "broker reports no positive stop-loss protection",
            None,
        ),
        (Some(stop), Some(reference), Some(sign)) if (reference - stop) * sign > 0.0 => check(
            "stop_loss",
            "pass",
            "stop is protective against the observed trigger side",
            Some(stop),
        ),
        (Some(stop), Some(_), Some(_)) => check(
            "stop_loss",
            "breach",
            "observed price has reached or passed stop; broker execution confirmation may lag",
            Some(stop),
        ),
        _ => check(
            "stop_loss",
            "unknown",
            "stop presence or executable trigger reference is not verifiable",
            None,
        ),
    });
    let s1 = evidence.is_some_and(|e| {
        matches!(e.strategy.as_str(), "s1" | "strategy1" | "jiaoyi-s1")
            || e.strategy == crate::trading::STRATEGY
    });
    checks.push(match (tp, entry, sign) {
        (Some(target), Some(entry), Some(sign)) if (target - entry) * sign > 0.0 => check(
            "take_profit",
            "pass",
            "target is on the profitable side of entry; no claim of likely attainment",
            Some(target),
        ),
        (Some(target), Some(_), Some(_)) => check(
            "take_profit",
            "breach",
            "target is not on the profitable side of the current entry",
            Some(target),
        ),
        (None, _, _) if s1 => check(
            "take_profit",
            "pass",
            "S1 uses registered trailing exits without a fixed take-profit",
            None,
        ),
        _ => check(
            "take_profit",
            "unknown",
            "target reward is unavailable; no target invented",
            None,
        ),
    });
    checks.push(strategy_check(evidence, side, quote));
    checks.extend(permission_checks(input, evidence, trade.symbol_id));
    if let Some(e) = evidence {
        checks.push(check("plan_quantity", if e.original_volume == trade.volume { "pass" } else { "unknown" },
            if e.original_volume == trade.volume { "observed quantity equals original instruction" } else { "quantity changed since entry; partial exit or external edit requires lifecycle evidence" }, None));
    }
    let conversion = info.and_then(|s| {
        positive(s.quote_to_deposit)
            .filter(|_| fresh(s.conversion_at, input.now, MAX_QUOTE_AGE_SECS))
    });
    let cash = |target: Option<f64>| {
        let result = (target? - entry?) * sign? * trade.volume as f64 / 100.0 * conversion?;
        (current_book && trade.volume > 0 && result.is_finite()).then_some(result)
    };
    let pnl_at_stop = cash(sl);
    let reward = cash(tp);
    let opening_ceiling = evidence.and_then(|e| {
        positive(
            e.detail["sizing"]["max_risk"]
                .as_f64()
                .or_else(|| e.detail["opening_max_risk"].as_f64()),
        )
    });
    checks.push(match (pnl_at_stop, opening_ceiling) {
        (Some(pnl), Some(ceiling)) => check("original_opening_risk_ceiling",
            if (-pnl).max(0.0) <= ceiling { "pass" } else { "breach" },
            if (-pnl).max(0.0) <= ceiling { "current nominal stop risk is within the evidenced opening cash ceiling" } else { "current nominal stop risk exceeds the evidenced opening cash ceiling" }, Some(ceiling)),
        _ => check("original_opening_risk_ceiling", "unknown", "current nominal risk or original opening cash ceiling unavailable; no new limit inferred", None),
    });
    checks.push(check(
        "nominal_risk",
        if pnl_at_stop.is_some() {
            "pass"
        } else {
            "unknown"
        },
        if pnl_at_stop.is_some() {
            "gross nominal stop exposure computed; excludes fees, gaps and slippage"
        } else {
            "complete entry, stop, quantity and fresh deposit conversion required"
        },
        pnl_at_stop.map(|v| (-v).max(0.0)),
    ));
    let cost = evidence
        .and_then(|e| e.detail["entry"]["cost"]["total"].as_f64())
        .filter(|v| v.is_finite() && *v >= 0.0);
    checks.push(check(
        "cost_evidence",
        if cost.is_some() { "pass" } else { "unknown" },
        if cost.is_some() {
            "original registered cost reserve retained; not a current all-cost guarantee"
        } else {
            "complete cost evidence unavailable; net reward/risk not asserted"
        },
        cost,
    ));
    ObjectAssessment {
        kind: kind.into(),
        id,
        position_id,
        symbol_id: trade.symbol_id,
        symbol: info.and_then(|s| s.light.symbol_name.clone()),
        side: side.into(),
        volume: trade.volume,
        units: trade.volume as f64 / 100.0,
        lots: info
            .and_then(|s| s.symbol.lot_size)
            .filter(|v| *v > 0)
            .map(|lot| trade.volume as f64 / lot as f64),
        entry,
        stop_loss: sl,
        take_profit: tp,
        strategy: evidence.cloned(),
        checks,
        nominal_loss_at_stop: pnl_at_stop.map(|v| (-v).max(0.0)),
        nominal_pnl_at_stop: pnl_at_stop,
        nominal_reward_at_target: reward,
        quote_at: q.and_then(|q| q.bid_at.zip(q.ask_at).map(|(b, a)| b.min(a))),
    }
}

fn pending_margin(input: &Input) -> Option<&PendingMargin> {
    input.pending_margin.as_ref().filter(|m| {
        input.complete
            && input.connected
            && fresh(input.book_at, input.now, MAX_BOOK_AGE_SECS)
            && m.revision == input.revision
            && fresh(Some(m.observed_at), input.now, MAX_BOOK_AGE_SECS)
            && positive(Some(m.projected_margin)).is_some()
    })
}

pub fn evaluate(input: &Input) -> Assessment {
    let mut objects = Vec::new();
    for p in &input.positions {
        if p.position_status == 1 {
            objects.push(assess_object(
                input,
                "position",
                p.position_id,
                Some(p.position_id),
                &p.trade_data,
                p.price,
                p.stop_loss,
                p.take_profit,
                p.stop_loss_trigger_method,
            ));
        }
    }
    let position_count = objects.len();
    let mut protective_order_count = 0;
    for o in &input.orders {
        if o.order_status != 1 {
            continue;
        }
        if o.closing_order == Some(true) || o.order_type == 4 {
            protective_order_count += 1;
            continue;
        }
        let entry = match o.order_type {
            2 => o.limit_price,
            3 | 6 => o.stop_price,
            _ => None,
        };
        let (_, sign) = direction(o.trade_data.trade_side);
        let relative = |distance: Option<i64>, target: bool| {
            let distance = distance.filter(|v| *v > 0)? as f64 / 100_000.0;
            Some(entry? + sign? * distance * if target { 1.0 } else { -1.0 })
        };
        let mut remaining = o.trade_data.clone();
        remaining.volume = o
            .executed_volume
            .filter(|v| *v >= 0 && *v <= remaining.volume)
            .and_then(|filled| remaining.volume.checked_sub(filled))
            .unwrap_or(remaining.volume);
        let mut assessed = assess_object(
            input,
            "pending_entry",
            o.order_id,
            o.position_id,
            &remaining,
            entry,
            o.stop_loss
                .or_else(|| relative(o.relative_stop_loss, false)),
            o.take_profit
                .or_else(|| relative(o.relative_take_profit, true)),
            Some(1),
        );
        if !o
            .executed_volume
            .is_some_and(|filled| filled >= 0 && filled <= o.trade_data.volume)
        {
            assessed.checks.push(check(
                "remaining_quantity",
                "unknown",
                "filled quantity is unavailable or inconsistent; pending residual is unverified",
                None,
            ));
            assessed.nominal_loss_at_stop = None;
            assessed.nominal_pnl_at_stop = None;
            assessed.nominal_reward_at_target = None;
        }
        let margin_estimate = pending_margin(input)
            .and_then(|m| m.by_order.get(&o.order_id))
            .copied()
            .filter(|v| v.is_finite() && *v >= 0.0);
        assessed.checks.push(match margin_estimate {
            Some(estimate) => check("broker_current_margin_estimate", "pass", "broker exact-volume estimate at current conditions; not guaranteed at the trigger price", Some(estimate)),
            None => check("broker_current_margin_estimate", "unknown", "fresh same-revision broker margin estimate unavailable", None),
        });
        assessed.checks.push(check(
            "trigger_margin_forecast",
            "unknown",
            "future trigger-price margin and liquidity cannot be guaranteed by a current estimate",
            None,
        ));
        assessed.checks.push(if matches!(o.order_type, 2 | 3 | 6) && positive(entry).is_some() {
            check("entry_trigger", "pass", "conditional entry trigger retained; distance from market is not a setup failure", entry)
        } else { check("entry_trigger", "unknown", "entry order type or trigger is unsupported", None) });
        if o.order_type == 6 {
            assessed.checks.push(check(
                "stop_limit",
                "unknown",
                "stop-limit execution range and fill probability are not inferred",
                None,
            ));
        }
        if o.expiration_timestamp
            .is_some_and(|t| t <= input.now.timestamp_millis())
        {
            assessed.checks.push(check(
                "expiry",
                "breach",
                "pending order expiry reached; broker confirmation pending",
                None,
            ));
        }
        objects.push(assessed);
    }
    let evidence_complete =
        input.complete && input.connected && fresh(input.book_at, input.now, MAX_BOOK_AGE_SECS);
    if !evidence_complete {
        for object in &mut objects {
            for c in &mut object.checks {
                if c.state == "pass"
                    && !matches!(c.name.as_str(), "quote" | "market" | "cost_evidence")
                {
                    c.state = "unknown".into();
                    c.reason = "current object state not verifiable from incomplete, stale or disconnected book".into();
                } else if c.state == "breach" && c.name != "market" {
                    c.reason = format!(
                        "last observed object; current state unverified: {}",
                        c.reason
                    );
                }
            }
        }
    }
    let stop_risk = objects
        .iter()
        .try_fold(0.0, |sum, o| o.nominal_loss_at_stop.map(|x| sum + x))
        .filter(|_| evidence_complete);
    let stop_loss_reserve = objects
        .iter()
        .try_fold(0.0, |sum, o| {
            let reserve = if o.kind == "position" {
                let gross = input
                    .gross_pnl_by_position
                    .get(&o.id)
                    .copied()
                    .filter(|v| v.is_finite())?;
                (gross - o.nominal_pnl_at_stop?).max(0.0)
            } else {
                if !o
                    .checks
                    .iter()
                    .any(|c| c.name == "stop_loss" && c.state == "pass")
                {
                    return None;
                }
                o.nominal_loss_at_stop?
            };
            (reserve.is_finite() && (sum + reserve).is_finite()).then_some(sum + reserve)
        })
        .filter(|_| evidence_complete && fresh(input.equity_at, input.now, MAX_BOOK_AGE_SECS));
    let mut account_checks = vec![check(
        "coverage",
        if evidence_complete { "pass" } else { "unknown" },
        if evidence_complete {
            "complete observed book evaluated independently of ownership"
        } else {
            "current full-account coverage is not verifiable"
        },
        None,
    )];
    let margin = input
        .positions
        .iter()
        .filter(|p| p.position_status == 1)
        .try_fold(0.0, |sum, p| {
            let digits = p.money_digits.filter(|d| *d <= 18)?;
            Some(sum + p.used_margin? as f64 / 10f64.powi(digits as i32))
        });
    let equity = input
        .equity
        .filter(|v| v.is_finite())
        .filter(|_| fresh(input.equity_at, input.now, MAX_BOOK_AGE_SECS));
    let margin_level = margin
        .zip(equity)
        .filter(|(m, _)| *m > 0.0 && evidence_complete)
        .map(|(m, e)| 100.0 * e / m);
    account_checks.push(match (margin_level, positive(input.margin_floor_pct)) {
        (Some(level), Some(floor)) => check("current_margin_buffer", if level >= floor { "pass" } else { "breach" },
            if level >= floor { "current broker equity/margin meets the new-entry floor; pending-entry forecast is separate" } else { "current broker equity/margin is below the new-entry floor; does not authorise closing a trade" }, Some(level)),
        _ if position_count == 0 && evidence_complete => check("current_margin_buffer", "pass", "no observed open position margin; pending-entry forecast remains separate", None),
        _ => check("current_margin_buffer", "unknown", "fresh equity, used margin and applicable floor required; balance is not equity", None),
    });
    if objects.len() > position_count {
        let projection = pending_margin(input);
        let level = projection
            .zip(equity)
            .map(|(m, e)| 100.0 * e / m.projected_margin)
            .filter(|v| v.is_finite());
        account_checks.push(match (level, positive(input.margin_floor_pct)) {
            (Some(level), Some(floor)) => check("pending_margin", if level >= floor { "pass" } else { "breach" },
                if level >= floor { "current broker projection for observed entries meets the existing margin floor" } else { "current broker projection for observed entries is below the existing margin floor" }, Some(level)),
            _ => check("pending_margin", "unknown", "fresh same-revision projection, equity and applicable margin floor required", None),
        });
        account_checks.push(match (projection, equity, stop_loss_reserve, positive(input.margin_floor_pct)) {
            (Some(m), Some(equity), Some(reserve), Some(floor)) if positive(m.warning_pct).is_some() => {
                let result = jiaoyi_ctrader::margin::evaluate(equity, m.projected_margin, reserve, floor, m.warning_pct.unwrap());
                let stressed = 100.0*(equity-reserve)/m.projected_margin;
                check("pending_margin_stress", if result.is_ok() { "pass" } else { "breach" },
                    if result.is_ok() { "existing margin thresholds accept the numeric stress projection" } else { "existing margin thresholds reject the numeric stress projection" }, Some(stressed))
            }
            _ => check("pending_margin_stress", "unknown", "complete broker marks, stops, conversion, margin projection and warning threshold required", None),
        });
    }
    let checks = || {
        objects
            .iter()
            .flat_map(|o| &o.checks)
            .chain(account_checks.iter())
    };
    let breach_count = checks().filter(|c| c.state == "breach").count();
    let unknown_count = checks().filter(|c| c.state == "unknown").count();
    let blocked_entry_check_count = checks().filter(|c| c.state == "blocked").count();
    Assessment {
        account_id: input.account_id,
        generation: input.generation,
        revision: input.revision,
        assessed_at: input.now,
        book_at: input.book_at,
        evidence_complete,
        interval_secs: 1,
        no_trade_authority: true,
        position_count,
        pending_entry_count: objects.len() - position_count,
        protective_order_count,
        objects,
        account_checks,
        nominal_stop_risk: stop_risk,
        stop_loss_reserve,
        breach_count,
        unknown_count,
        blocked_entry_check_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use serde_json::json;

    fn input() -> Input {
        let now = DateTime::parse_from_rfc3339("2026-11-01T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        Input {
            account_id: 1,
            generation: 1,
            revision: 1,
            now,
            book_at: Some(now),
            complete: true,
            connected: true,
            positions: vec![pb::ProtoOaPosition {
                position_id: 10,
                position_status: 1,
                trade_data: pb::ProtoOaTradeData {
                    symbol_id: 7,
                    volume: 1000,
                    trade_side: 1,
                    ..Default::default()
                },
                price: Some(100.0),
                stop_loss: Some(95.0),
                take_profit: Some(120.0),
                used_margin: Some(10000),
                money_digits: Some(2),
                ..Default::default()
            }],
            orders: vec![],
            quotes: HashMap::from([(
                7,
                Quote {
                    bid: Some(105.0),
                    ask: Some(106.0),
                    bid_at: Some(now),
                    ask_at: Some(now),
                },
            )]),
            symbols: HashMap::from([(
                7,
                SymbolInfo {
                    symbol: pb::ProtoOaSymbol {
                        symbol_id: 7,
                        digits: 2,
                        pip_position: 2,
                        min_volume: Some(100),
                        step_volume: Some(100),
                        max_volume: Some(10000),
                        lot_size: Some(1000),
                        schedule: vec![pb::ProtoOaInterval {
                            start_second: 0,
                            end_second: 604800,
                        }],
                        schedule_time_zone: Some("UTC".into()),
                        ..Default::default()
                    },
                    light: pb::ProtoOaLightSymbol {
                        symbol_id: 7,
                        symbol_name: Some("TEST".into()),
                        enabled: Some(true),
                        ..Default::default()
                    },
                    quote_to_deposit: Some(1.0),
                    conversion_at: Some(now),
                },
            )]),
            trader: None,
            strategy: HashMap::new(),
            margin_floor_pct: Some(250.0),
            equity: Some(1000.0),
            equity_at: Some(now),
            gross_pnl_by_position: HashMap::from([(10, 50.0)]),
            pending_margin: None,
            permissions: HashMap::new(),
        }
    }

    fn verdict<'a>(o: &'a ObjectAssessment, name: &str) -> &'a str {
        &o.checks.iter().find(|c| c.name == name).unwrap().state
    }

    #[test]
    fn external_positions_and_entries_are_assessed_without_acquiring_authority() {
        let mut i = input();
        i.orders.push(pb::ProtoOaOrder {
            order_id: 20,
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            trade_data: i.positions[0].trade_data.clone(),
            limit_price: Some(90.0),
            relative_stop_loss: Some(500_000),
            relative_take_profit: Some(1_000_000),
            ..Default::default()
        });
        i.orders.push(pb::ProtoOaOrder {
            order_id: 21,
            order_type: 4,
            order_status: 1,
            closing_order: Some(true),
            trade_data: i.positions[0].trade_data.clone(),
            ..Default::default()
        });
        let r = evaluate(&i);
        assert_eq!(
            (
                r.position_count,
                r.pending_entry_count,
                r.protective_order_count
            ),
            (1, 1, 1)
        );
        assert!(r.no_trade_authority);
        assert_eq!(verdict(&r.objects[0], "direction"), "unknown");
        assert_eq!(r.objects[1].stop_loss, Some(85.0));
        assert_eq!(r.objects[1].take_profit, Some(100.0));
        assert_eq!(r.nominal_stop_risk, Some(100.0));
    }

    #[test]
    fn manual_side_volume_and_protection_changes_recompute_actual_exposure() {
        let mut i = input();
        let a = evaluate(&i);
        i.revision += 1;
        i.positions[0].trade_data.trade_side = 2;
        i.positions[0].trade_data.volume = 2000;
        i.positions[0].stop_loss = Some(110.0);
        i.positions[0].take_profit = Some(80.0);
        let b = evaluate(&i);
        assert_ne!(a.revision, b.revision);
        assert_eq!(b.objects[0].side, "sell");
        assert_eq!(b.objects[0].units, 20.0);
        assert_eq!(b.objects[0].lots, Some(2.0));
        assert_eq!(b.nominal_stop_risk, Some(200.0));
        assert_eq!(verdict(&b.objects[0], "stop_loss"), "pass");
    }

    #[test]
    fn stale_or_missing_external_quotes_are_unknown_without_hiding_missing_stop() {
        let mut i = input();
        i.quotes.get_mut(&7).unwrap().bid_at = Some(i.now - Duration::seconds(7));
        i.positions[0].stop_loss = None;
        let r = evaluate(&i);
        assert_eq!(verdict(&r.objects[0], "quote"), "unknown");
        assert_eq!(verdict(&r.objects[0], "stop_loss"), "breach");
        assert!(r.nominal_stop_risk.is_none());
        i.symbols.clear();
        assert_eq!(verdict(&evaluate(&i).objects[0], "market"), "unknown");
    }

    #[test]
    fn profitable_trailing_stop_and_s1_no_target_are_not_false_failures() {
        let mut i = input();
        i.positions[0].stop_loss = Some(102.0);
        i.positions[0].take_profit = None;
        i.strategy.insert(
            10,
            StrategyEvidence {
                strategy: "s1".into(),
                side: "buy".into(),
                detail: json!({"level":101.0,"ema_fast":100.0,"ema_slow":99.0}),
                recorded_at: i.now,
                original_volume: 1000,
                entry_key: "owned-opening".into(),
            },
        );
        let r = evaluate(&i);
        assert_eq!(verdict(&r.objects[0], "stop_loss"), "pass");
        assert_eq!(verdict(&r.objects[0], "take_profit"), "pass");
        assert_eq!(verdict(&r.objects[0], "direction"), "pass");
        assert_eq!(r.objects[0].nominal_pnl_at_stop, Some(20.0));
        assert_eq!(r.nominal_stop_risk, Some(0.0));
    }

    #[test]
    fn s3_frozen_benchmark_uses_midpoint_and_rejects_spread_wider_than_band() {
        let mut i = input();
        i.strategy.insert(
            10,
            StrategyEvidence {
                strategy: "s3".into(),
                side: "buy".into(),
                detail: json!({"band":{"lower":99.0,"upper":105.8}}),
                recorded_at: i.now,
                original_volume: 1000,
                entry_key: "owned-s3".into(),
            },
        );
        assert_eq!(verdict(&evaluate(&i).objects[0], "direction"), "breach");
        i.quotes.get_mut(&7).unwrap().bid = Some(106.0);
        i.quotes.get_mut(&7).unwrap().ask = Some(107.0);
        assert_eq!(verdict(&evaluate(&i).objects[0], "direction"), "pass");
        i.quotes.get_mut(&7).unwrap().ask = Some(120.0);
        assert_eq!(verdict(&evaluate(&i).objects[0], "direction"), "breach");
    }

    #[test]
    fn old_book_or_conversion_cannot_produce_zero_risk_certificate() {
        let mut i = input();
        i.book_at = Some(i.now - Duration::seconds(121));
        assert!(evaluate(&i).nominal_stop_risk.is_none());
        i.book_at = Some(i.now);
        i.symbols.get_mut(&7).unwrap().conversion_at = Some(i.now - Duration::seconds(7));
        assert!(evaluate(&i).nominal_stop_risk.is_none());
    }

    #[test]
    fn partial_residual_is_not_treated_as_new_entry_and_pending_filled_volume_is_deducted() {
        let mut i = input();
        i.positions[0].trade_data.volume = 50;
        i.orders.push(pb::ProtoOaOrder {
            order_id: 20,
            order_type: 2,
            order_status: 1,
            trade_data: pb::ProtoOaTradeData {
                volume: 1000,
                ..i.positions[0].trade_data.clone()
            },
            executed_volume: Some(200),
            limit_price: Some(90.0),
            stop_loss: Some(85.0),
            ..Default::default()
        });
        let r = evaluate(&i);
        assert_eq!(verdict(&r.objects[0], "quantity"), "pass");
        assert_eq!(r.objects[1].volume, 800);
        assert_eq!(verdict(&r.objects[1], "quantity"), "pass");
    }

    #[test]
    fn current_margin_breach_is_distinct_from_pending_margin_unknown() {
        let mut i = input();
        i.equity = Some(200.0);
        let r = evaluate(&i);
        let c = r
            .account_checks
            .iter()
            .find(|c| c.name == "current_margin_buffer")
            .unwrap();
        assert_eq!(c.state, "breach");
        assert_eq!(c.value, Some(200.0));
        i.equity = Some(0.0);
        assert_eq!(evaluate(&i).account_checks[1].state, "breach");
        i.equity_at = None;
        assert_eq!(evaluate(&i).account_checks[1].state, "unknown");
    }

    #[test]
    fn stale_disconnected_or_incomplete_book_cannot_certify_current_setup() {
        let mut i = input();
        i.strategy.insert(
            10,
            StrategyEvidence {
                strategy: "s1".into(),
                side: "buy".into(),
                detail: json!({"level":101.0,"ema_fast":100.0,"ema_slow":99.0}),
                recorded_at: i.now,
                original_volume: 1000,
                entry_key: "owned-opening".into(),
            },
        );
        for state in ["stale", "disconnected", "incomplete"] {
            let mut changed = i.clone();
            match state {
                "stale" => changed.book_at = Some(i.now - Duration::seconds(121)),
                "disconnected" => changed.connected = false,
                _ => changed.complete = false,
            }
            let r = evaluate(&changed);
            for field in [
                "quantity",
                "stop_loss",
                "take_profit",
                "direction",
                "plan_quantity",
            ] {
                assert_ne!(verdict(&r.objects[0], field), "pass", "{state} {field}");
            }
            assert!(!r.evidence_complete);
            assert!(r.nominal_stop_risk.is_none());
        }
    }

    #[test]
    fn changed_stop_is_compared_with_evidenced_opening_risk_ceiling_only() {
        let mut i = input();
        i.strategy.insert(
            10,
            StrategyEvidence {
                strategy: "s3".into(),
                side: "buy".into(),
                detail: json!({"sizing":{"max_risk":40.0}}),
                recorded_at: i.now,
                original_volume: 1000,
                entry_key: "owned-s3".into(),
            },
        );
        assert_eq!(
            verdict(&evaluate(&i).objects[0], "original_opening_risk_ceiling"),
            "breach"
        );
        i.positions[0].stop_loss = Some(102.0);
        assert_eq!(
            verdict(&evaluate(&i).objects[0], "original_opening_risk_ceiling"),
            "pass"
        );
        i.strategy.clear();
        assert_eq!(
            verdict(&evaluate(&i).objects[0], "original_opening_risk_ceiling"),
            "unknown"
        );
    }

    #[test]
    fn pending_order_cannot_inherit_strategy_from_a_shared_position_id() {
        let mut i = input();
        i.strategy.insert(10, StrategyEvidence {
            strategy: "s1".into(), side: "buy".into(),
            detail: json!({"level":101.0,"ema_fast":100.0,"ema_slow":99.0,"opening_max_risk":100.0}),
            recorded_at: i.now, original_volume: 1000, entry_key: "original-position-intent".into(),
        });
        i.orders.push(pb::ProtoOaOrder {
            order_id: 20,
            position_id: Some(10),
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            trade_data: i.positions[0].trade_data.clone(),
            client_order_id: Some("manual-new-order".into()),
            limit_price: Some(90.0),
            stop_loss: Some(85.0),
            take_profit: None,
            ..Default::default()
        });
        let r = evaluate(&i);
        assert!(r.objects[0].strategy.is_some());
        assert_eq!(verdict(&r.objects[0], "direction"), "pass");
        let pending = &r.objects[1];
        assert!(pending.strategy.is_none());
        for check in ["direction", "take_profit", "original_opening_risk_ceiling"] {
            assert_eq!(verdict(pending, check), "unknown");
        }
    }

    fn with_pending_projection() -> Input {
        let mut i = input();
        i.orders.push(pb::ProtoOaOrder {
            order_id: 20,
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            trade_data: i.positions[0].trade_data.clone(),
            limit_price: Some(90.0),
            stop_loss: Some(85.0),
            ..Default::default()
        });
        i.pending_margin = Some(PendingMargin {
            projected_margin: 200.0,
            observed_at: i.now,
            revision: i.revision,
            warning_pct: Some(150.0),
            by_order: HashMap::from([(20, 100.0)]),
        });
        i
    }

    fn account_verdict<'a>(r: &'a Assessment, name: &str) -> &'a str {
        &r.account_checks
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .state
    }

    #[test]
    fn pending_capacity_uses_fresh_projection_and_reserves_open_profit_giveback() {
        let mut i = with_pending_projection();
        let r = evaluate(&i);
        assert_eq!(account_verdict(&r, "pending_margin"), "pass");
        assert_eq!(account_verdict(&r, "pending_margin_stress"), "pass");
        assert_eq!(r.nominal_stop_risk, Some(100.0));
        assert_eq!(r.stop_loss_reserve, Some(150.0));
        assert_eq!(
            verdict(&r.objects[1], "broker_current_margin_estimate"),
            "pass"
        );
        assert_eq!(verdict(&r.objects[1], "trigger_margin_forecast"), "unknown");
        // Same nominal entry risk, but large broker mark profits would be
        // surrendered on a stop: stress must use broker equity giveback.
        i.gross_pnl_by_position.insert(10, 650.0);
        let r = evaluate(&i);
        assert_eq!(account_verdict(&r, "pending_margin"), "pass");
        assert_eq!(account_verdict(&r, "pending_margin_stress"), "breach");
        assert_eq!(r.stop_loss_reserve, Some(750.0));
    }

    #[test]
    fn pending_margin_known_floor_failure_remains_when_stop_stress_is_unknown() {
        let mut i = with_pending_projection();
        i.pending_margin.as_mut().unwrap().projected_margin = 500.0;
        i.positions[0].stop_loss = None;
        let r = evaluate(&i);
        assert_eq!(account_verdict(&r, "pending_margin"), "breach");
        assert_eq!(account_verdict(&r, "pending_margin_stress"), "unknown");
    }

    #[test]
    fn pending_projection_must_match_book_revision_and_observation_age() {
        let i = with_pending_projection();
        for state in ["revision", "age", "book", "equity"] {
            let mut changed = i.clone();
            match state {
                "revision" => changed.pending_margin.as_mut().unwrap().revision += 1,
                "age" => {
                    changed.pending_margin.as_mut().unwrap().observed_at -= Duration::seconds(121)
                }
                "book" => changed.complete = false,
                _ => changed.equity_at = None,
            }
            let r = evaluate(&changed);
            assert_eq!(account_verdict(&r, "pending_margin"), "unknown", "{state}");
            assert_eq!(
                account_verdict(&r, "pending_margin_stress"),
                "unknown",
                "{state}"
            );
        }
    }

    fn with_permission_evidence() -> Input {
        let mut i = input();
        i.strategy.insert(
            10,
            StrategyEvidence {
                strategy: "s3".into(),
                side: "buy".into(),
                detail: json!({"band":{"lower":99.0,"upper":104.0}}),
                recorded_at: i.now,
                original_volume: 1000,
                entry_key: "owned-s3".into(),
            },
        );
        i.permissions.insert(
            "s3".into(),
            PermissionEvidence {
                observed_at: i.now,
                book_revision: i.revision,
                owner_paused: Some(true),
                cost_veto_by_symbol: HashMap::from([(7, true)]),
                entry_recorded_by_symbol: HashMap::from([(7, true)]),
                live_single_position_blocked: Some(true),
                s1_edge_passed: None,
            },
        );
        i
    }

    #[test]
    fn entry_exclusions_do_not_mislabel_a_supported_protected_position() {
        let i = with_permission_evidence();
        let r = evaluate(&i);
        let o = &r.objects[0];
        assert_eq!(verdict(o, "direction"), "pass");
        assert_eq!(verdict(o, "stop_loss"), "pass");
        for name in [
            "strategy_symbol_exposure_new_entry",
            "owner_pause_new_entry",
            "live_capacity_new_entry",
            "durable_cost_veto_new_entry",
            "session_entry_new_entry",
        ] {
            assert_eq!(verdict(o, name), "blocked");
        }
        assert_eq!(r.blocked_entry_check_count, 5);
        assert_eq!(r.breach_count, 0);
        assert!(r.no_trade_authority);
    }

    #[test]
    fn demo_owned_position_still_blocks_repeat_entry_after_session_rollover() {
        let mut i = with_permission_evidence();
        let p = i.permissions.get_mut("s3").unwrap();
        p.owner_paused = Some(false);
        p.cost_veto_by_symbol.insert(7, false);
        p.entry_recorded_by_symbol.insert(7, false);
        p.live_single_position_blocked = Some(false);
        let r = evaluate(&i);
        assert_eq!(
            verdict(&r.objects[0], "strategy_symbol_exposure_new_entry"),
            "blocked"
        );
        assert_eq!(r.blocked_entry_check_count, 1);
        assert_eq!(verdict(&r.objects[0], "direction"), "pass");
        assert_eq!(r.breach_count, 0);
    }

    #[test]
    fn permission_evidence_must_be_fresh_scoped_and_never_assigned_to_manual_trades() {
        let i = with_permission_evidence();
        for state in ["age", "revision", "symbol"] {
            let mut changed = i.clone();
            let permissions = changed.permissions.get_mut("s3").unwrap();
            match state {
                "age" => permissions.observed_at -= Duration::seconds(121),
                "revision" => permissions.book_revision += 1,
                _ => {
                    permissions.cost_veto_by_symbol.clear();
                    permissions.entry_recorded_by_symbol.clear();
                }
            }
            let r = evaluate(&changed);
            for name in ["durable_cost_veto_new_entry", "session_entry_new_entry"] {
                assert_eq!(verdict(&r.objects[0], name), "unknown", "{state}");
            }
            assert_eq!(verdict(&r.objects[0], "stop_loss"), "pass");
        }
        let mut manual = i;
        manual.strategy.clear();
        let r = evaluate(&manual);
        assert_eq!(verdict(&r.objects[0], "entry_permission"), "unknown");
        assert_eq!(r.blocked_entry_check_count, 0);
        assert!(!r.objects[0]
            .checks
            .iter()
            .any(|c| c.name == "owner_pause_new_entry"));
    }
}
