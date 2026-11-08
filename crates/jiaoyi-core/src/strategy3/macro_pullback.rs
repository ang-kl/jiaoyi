//! Owner-authorised production revision №18,377. No transport or mutable settings.
//! This revision uses observed impulse distance and midpoint retests; it must
//! not inherit financial results from the different Phase4 research candidate.
use super::{
    band::{self, Band, M30},
    sessions::{self, Session, SessionRule},
    targets,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MODEL: &str = "s3_macro_pullback_v1";
pub const POLICY: &str = "s3_macro_pullback_owner_18380_v1";
/// Explicit owner-selected allowance, not a statistically calibrated full
/// round-trip execution-cost estimate. The 24bp stress model stays in research.
pub const LIVE_SLIPPAGE_BPS_PER_SIDE: f64 = 3.74;
pub fn two_stage_policy(policy: &str) -> bool {
    matches!(
        policy,
        "s3_macro_pullback_owner_18377_v1" | "s3_macro_pullback_owner_18380_v1"
    )
}
pub const CORE: [&str; 8] = [
    "SpotCrude",
    "SpotBrent",
    "US500",
    "NAS100",
    "XAUUSD",
    "XAGUSD",
    "BTCUSD",
    "ETHUSD",
];
pub const IMPULSE_FLOOR_BPS: f64 = 48.0;

pub fn peer(symbol: &str) -> Option<&'static str> {
    match symbol {
        "SpotCrude" => Some("SpotBrent"),
        "SpotBrent" => Some("SpotCrude"),
        "US500" => Some("NAS100"),
        "NAS100" => Some("US500"),
        "XAUUSD" => Some("XAGUSD"),
        "XAGUSD" => Some("XAUUSD"),
        "BTCUSD" => Some("ETHUSD"),
        "ETHUSD" => Some("BTCUSD"),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub drift_5: f64,
    pub drift_20: f64,
    pub latest_bar_end: DateTime<Utc>,
}

/// Complete native sessions, causal at cutoff; no sparse-bar substitution.
pub fn context(
    rule: &SessionRule,
    session: Session,
    bars: &[M30],
    cutoff: DateTime<Utc>,
) -> Result<Context, String> {
    let mut prior = Vec::new();
    for ago in (1..=45).rev() {
        if let Some(s) = sessions::session_on(rule, session.date - Duration::days(ago)) {
            prior.push(s);
        }
    }
    if prior.len() < 20 {
        return Err("macro_missing_20_sessions".into());
    }
    let mut by_time = BTreeMap::new();
    for b in bars {
        if !(b.open.is_finite() && b.close.is_finite() && b.open > 0.0 && b.close > 0.0) {
            return Err("macro_invalid_bar".into());
        }
        if let Some(old) = by_time.insert(b.open_time, b) {
            if old.open != b.open || old.close != b.close {
                return Err("macro_conflicting_bar".into());
            }
        }
    }
    let drift = |n: usize| -> Result<(f64, DateTime<Utc>), String> {
        let mut first = None;
        let mut last = None;
        for s in &prior[prior.len() - n..] {
            let mut at = s.open;
            while at + Duration::minutes(30) <= s.close {
                let end = at + Duration::minutes(30);
                if s.trading_at(at) {
                    if end > cutoff || end > session.open {
                        return Err("macro_incomplete_or_noncausal_session".into());
                    }
                    let b = by_time.get(&at).ok_or("macro_missing_m30_segment")?;
                    first.get_or_insert(b.open);
                    last = Some((b.close, end));
                }
                at = end;
            }
        }
        let (close, end) = last.ok_or("macro_empty_session")?;
        Ok((close / first.ok_or("macro_empty_session")? - 1.0, end))
    };
    let (drift_5, _) = drift(5)?;
    let (drift_20, latest_bar_end) = drift(20)?;
    Ok(Context {
        drift_5,
        drift_20,
        latest_bar_end,
    })
}

pub fn aligns(context: Context, long: bool, open: DateTime<Utc>) -> bool {
    let sign = if long { 1.0 } else { -1.0 };
    context.latest_bar_end <= open
        && [context.drift_5, context.drift_20]
            .into_iter()
            .all(|v| v.is_finite() && v * sign > 0.0)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ArmedPullback {
    pub long: bool,
    pub origin: f64,
    pub extreme: f64,
    pub level: f64,
    pub impulse_bps: f64,
    pub impulse_range: f64,
    pub armed_at: DateTime<Utc>,
    pub session: Session,
    pub macro_context: Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touch {
    Waiting,
    Trigger,
    Invalidated,
    Expired,
}

pub fn retest(long: bool, origin: f64, level: f64, bid: f64, ask: f64) -> bool {
    if ![origin, level, bid, ask]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0)
        || bid > ask
    {
        return false;
    }
    let mid = bid + (ask - bid) / 2.0;
    if long {
        origin < mid && mid <= level
    } else {
        level <= mid && mid < origin
    }
}

impl ArmedPullback {
    #[allow(clippy::too_many_arguments)]
    pub fn arm(
        symbol: &str,
        session: Session,
        b: Band,
        bid: f64,
        ask: f64,
        at: DateTime<Utc>,
        context: Context,
        peer_move: Option<f64>,
        cost: f64,
    ) -> Result<Self, &'static str> {
        if peer(symbol).is_none() {
            return Err("symbol_retired_from_s3_core");
        }
        if band::decision_mark(&session, at).is_none() || band::entry_window(&session, at).is_err()
        {
            return Err("outside_impulse_mark");
        }
        if ![bid, ask, b.open, cost].into_iter().all(f64::is_finite)
            || bid <= 0.0
            || ask < bid
            || b.open <= 0.0
            || cost < 0.0
        {
            return Err("invalid_impulse_inputs");
        }
        let long = match band::mark_decision(&b, Some(bid), Some(ask)) {
            Some(targets::Dir::Buy) => true,
            Some(targets::Dir::Sell) => false,
            None => return Err("inside_noise_band"),
        };
        let extreme = bid + (ask - bid) / 2.0;
        let impulse_range = (extreme - b.open).abs();
        let impulse_bps = impulse_range / b.open * 10000.0;
        if impulse_bps < IMPULSE_FLOOR_BPS {
            return Err("impulse_below_48bps");
        }
        if impulse_range <= cost {
            return Err("cost_ge_r");
        }
        if !aligns(context, long, session.open) {
            return Err("vetoed_by_5d_20d_macro_trend");
        }
        let sign = if long { 1.0 } else { -1.0 };
        if peer_move.is_some_and(|v| !v.is_finite() || v * sign < 0.0) {
            return Err("peer_coherence");
        }
        Ok(Self {
            long,
            origin: b.open,
            extreme,
            level: b.open + 0.5 * (extreme - b.open),
            impulse_bps,
            impulse_range,
            armed_at: at,
            session,
            macro_context: context,
        })
    }
    pub fn touch(&self, bid: f64, ask: f64, at: DateTime<Utc>) -> Touch {
        if at >= self.session.close - Duration::minutes(30) {
            return Touch::Expired;
        }
        if at <= self.armed_at
            || !self.session.trading_at(at)
            || ![bid, ask].into_iter().all(|v| v.is_finite() && v > 0.0)
            || ask < bid
        {
            return Touch::Waiting;
        }
        let mid = bid + (ask - bid) / 2.0;
        if (self.long && mid <= self.origin) || (!self.long && mid >= self.origin) {
            return Touch::Invalidated;
        }
        if retest(self.long, self.origin, self.level, bid, ask) {
            Touch::Trigger
        } else {
            Touch::Waiting
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Geometry {
    pub risk: f64,
    pub stop: f64,
    pub target_distance: f64,
    pub target: f64,
}

pub fn geometry(
    long: bool,
    fill: f64,
    origin: f64,
    cost: f64,
    digits: i32,
) -> Result<Geometry, &'static str> {
    if !(0..=5).contains(&digits)
        || ![fill, origin, cost].into_iter().all(f64::is_finite)
        || fill <= 0.0
        || origin <= 0.0
        || cost < 0.0
        || (if long { fill <= origin } else { fill >= origin })
    {
        return Err("invalid_structural_geometry");
    }
    let scale = 10_f64.powi(digits);
    let ticks = ((fill - origin).abs() + cost) * scale;
    let ticks = if (ticks - ticks.round()).abs() < 1e-6 {
        ticks.round()
    } else {
        ticks.ceil()
    };
    if ticks <= 0.0 || ticks > i64::MAX as f64 / 138.0 {
        return Err("invalid_risk_grid");
    }
    let risk = ticks / scale;
    let target_distance = ((ticks as i64 * 138 + 99) / 100) as f64 / scale;
    let sign = if long { 1.0 } else { -1.0 };
    let stop = fill - sign * risk;
    let target = fill + sign * target_distance;
    if ![risk, target_distance, stop, target]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0)
    {
        return Err("invalid_bracket_price");
    }
    Ok(Geometry {
        risk,
        stop,
        target_distance,
        target,
    })
}

/// Exactly one profitable-direction amendment, based on original opening R.
/// Entry-price breakeven is gross-flat, not a guarantee of net breakeven.
pub fn breakeven(
    long: bool,
    entry: f64,
    risk: f64,
    executable: f64,
    current_stop: f64,
    digits: i32,
) -> Option<f64> {
    if !(0..=5).contains(&digits)
        || ![entry, risk, executable, current_stop]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
    {
        return None;
    }
    let scale = 10_f64.powi(digits);
    let sign = if long { 1.0 } else { -1.0 };
    if (executable - entry) * sign + 1e-8 < risk {
        return None;
    }
    // A multi-fill VWAP may lie between native price ticks. The nearest
    // representable non-losing entry stop is at most one tick beyond VWAP.
    let scaled = entry * scale;
    let nearest = scaled.round();
    let ticks = if (scaled - nearest).abs() <= 1e-7 {
        nearest
    } else if long {
        scaled.ceil()
    } else {
        scaled.floor()
    };
    let stop = ticks / scale;
    if !stop.is_finite() {
        return None;
    }
    if (stop - current_stop) * sign * scale < 1.0 - 1e-6 {
        return None;
    }
    Some(stop)
}

/// Explicit cost assumptions for parity research. The spread reserve sizes R;
/// observed bid/ask execution already includes spread in realised gross P&L.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayCosts {
    pub prepared_spread: f64,
    pub commission_round_trip: f64,
    pub slippage_bps_per_side: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPeer {
    pub origin: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_at: DateTime<Utc>,
    pub ask_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayQuote {
    pub at: DateTime<Utc>,
    pub bid: f64,
    pub ask: f64,
    pub bid_at: DateTime<Utc>,
    pub ask_at: DateTime<Utc>,
    pub peer: Option<ReplayPeer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayEvent {
    pub at: DateTime<Utc>,
    pub kind: &'static str,
    pub bid: f64,
    pub ask: f64,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplayResult {
    pub policy: &'static str,
    pub status: &'static str,
    pub entry: Option<f64>,
    pub exit: Option<f64>,
    pub geometry: Option<Geometry>,
    pub gross_r: Option<f64>,
    pub net_r_assuming_costs: Option<f64>,
    pub cost_assumptions: ReplayCosts,
    pub execution_assumption: &'static str,
    pub financial_pass: bool,
    pub events: Vec<ReplayEvent>,
}

fn fresh_leg(at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    at <= now && now - at <= Duration::seconds(6)
}

/// Replay ONE frozen arm; the caller prepares it through `context`, `band_at`
/// and `ArmedPullback::arm`, exactly as production does. This deliberately does
/// not replace the frozen Phase-4 candidate or claim account/sizing parity.
/// The entry fills at its executable quote; a gap stop fills at the observed
/// worse executable quote; a target receives its limit price (no improvement).
/// A BE amendment becomes effective only on the next later quote. There is no
/// broker latency/ack model; the assumption is explicit in the result. The first
/// fresh quote at/after native close executes session-end. Missing that quote
/// leaves the trade censored, never closed at an invented end-of-cache price.
pub fn replay_armed(
    armed: ArmedPullback,
    digits: i32,
    costs: ReplayCosts,
    quotes: &[ReplayQuote],
) -> Result<ReplayResult, String> {
    if quotes.len() > 2_000_000
        || !(0..=5).contains(&digits)
        || ![
            armed.origin,
            armed.extreme,
            armed.level,
            armed.impulse_range,
            armed.impulse_bps,
        ]
        .into_iter()
        .all(|v| v.is_finite() && v > 0.0)
        || armed.level != armed.origin + 0.5 * (armed.extreme - armed.origin)
        || armed.impulse_range != (armed.extreme - armed.origin).abs()
        || (armed.impulse_bps - armed.impulse_range / armed.origin * 10000.0).abs() > 1e-8
        || armed.impulse_bps < IMPULSE_FLOOR_BPS
        || !aligns(armed.macro_context, armed.long, armed.session.open)
        || (armed.extreme > armed.origin) != armed.long
        || band::decision_mark(&armed.session, armed.armed_at).is_none()
        || band::entry_window(&armed.session, armed.armed_at).is_err()
    {
        return Err("invalid bounded frozen arm".into());
    }
    super::cost_reserve(
        costs.prepared_spread,
        costs.commission_round_trip,
        costs.slippage_bps_per_side,
        armed.extreme,
    )?;
    // Validate the whole bounded input before any early close can return.
    // A bad/held-out trailing observation must not hide behind an earlier fill.
    let mut previous = None;
    for q in quotes {
        if previous.is_some_and(|at| q.at <= at)
            || q.at < armed.armed_at
            || ![q.bid, q.ask].into_iter().all(|v| v.is_finite() && v > 0.0)
            || q.bid > q.ask
            || q.bid_at > q.at
            || q.ask_at > q.at
            || q.at > armed.session.close + Duration::seconds(6)
            || q.peer.is_some_and(|p| p.bid_at > q.at || p.ask_at > q.at)
        {
            return Err("invalid, future, unordered or out-of-session quote".into());
        }
        previous = Some(q.at);
    }
    let mut result = ReplayResult {
        policy: POLICY, status: "no_retest", entry: None, exit: None, geometry: None,
        gross_r: None, net_r_assuming_costs: None, cost_assumptions: costs,
        execution_assumption: "executable-quote entry and stop; target-limit price; next-quote BE ack; account admission and latency not simulated",
        financial_pass: false, events: Vec::new(),
    };
    let mut live: Option<(f64, Geometry, f64)> = None;
    for q in quotes {
        if !fresh_leg(q.bid_at, q.at) || !fresh_leg(q.ask_at, q.at) {
            result.events.push(ReplayEvent {
                at: q.at,
                kind: "stale_quote_skipped",
                bid: q.bid,
                ask: q.ask,
                price: None,
            });
            continue;
        }
        let executable = if armed.long { q.bid } else { q.ask };
        let sign = if armed.long { 1.0 } else { -1.0 };
        if let Some((entry, g, stop)) = live {
            let closed = if (executable - stop) * sign <= 0.0 {
                Some((
                    if stop == g.stop {
                        "initial_stop_loss"
                    } else {
                        "breakeven_stop_after_1r"
                    },
                    executable,
                ))
            } else if (executable - g.target) * sign >= 0.0 {
                Some(("strict_target_1_38r", g.target))
            } else if q.at >= armed.session.close {
                Some(("session_end", executable))
            } else {
                None
            };
            if let Some((reason, exit)) = closed {
                let gross = (exit - entry) * sign / g.risk;
                result.status = reason;
                result.exit = Some(exit);
                result.gross_r = Some(gross);
                result.net_r_assuming_costs = Some(
                    gross
                        - (costs.commission_round_trip
                            + (entry + exit) * costs.slippage_bps_per_side / 10000.0)
                            / g.risk,
                );
                result.events.push(ReplayEvent {
                    at: q.at,
                    kind: reason,
                    bid: q.bid,
                    ask: q.ask,
                    price: Some(exit),
                });
                return Ok(result);
            }
            if let Some(next) = breakeven(armed.long, entry, g.risk, executable, stop, digits) {
                live = Some((entry, g, next));
                result.events.push(ReplayEvent {
                    at: q.at,
                    kind: "breakeven_amend_effective_next_quote",
                    bid: q.bid,
                    ask: q.ask,
                    price: Some(next),
                });
            }
            continue;
        }
        match armed.touch(q.bid, q.ask, q.at) {
            Touch::Waiting => continue,
            Touch::Expired | Touch::Invalidated => {
                result.status = if armed.touch(q.bid, q.ask, q.at) == Touch::Expired {
                    "pullback_not_reached_expired"
                } else {
                    "pullback_origin_invalidated"
                };
                return Ok(result);
            }
            Touch::Trigger => {}
        }
        if let Some(p) = q.peer {
            if p.bid_at > q.at || p.ask_at > q.at {
                return Err("future peer observation".into());
            }
            if fresh_leg(p.bid_at, q.at)
                && fresh_leg(p.ask_at, q.at)
                && (![p.origin, p.bid, p.ask]
                    .into_iter()
                    .all(|v| v.is_finite() && v > 0.0)
                    || p.bid > p.ask
                    || (p.bid + (p.ask - p.bid) / 2.0 - p.origin) * sign < 0.0)
            {
                result.status = "peer_coherence";
                return Ok(result);
            }
        }
        let entry = if armed.long { q.ask } else { q.bid };
        let cost = super::cost_reserve(
            costs.prepared_spread,
            costs.commission_round_trip,
            costs.slippage_bps_per_side,
            entry,
        )?
        .total;
        if cost >= armed.impulse_range {
            result.status = "cost_ge_r";
            return Ok(result);
        }
        let g = geometry(armed.long, entry, armed.origin, cost, digits)?;
        result.entry = Some(entry);
        result.geometry = Some(g);
        result.status = "censored_open";
        result.events.push(ReplayEvent {
            at: q.at,
            kind: "pullback_entry",
            bid: q.bid,
            ask: q.ask,
            price: Some(entry),
        });
        live = Some((entry, g, g.stop));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup(long: bool) -> (Session, Band, Context) {
        let s = sessions::session_on(&SessionRule::UtcDay, "2026-11-08".parse().unwrap()).unwrap();
        (
            s,
            Band {
                open: 100.0,
                sigma: 0.004,
                lower: 99.6,
                upper: 100.4,
                half_width: 0.4,
                minutes: 30.0,
            },
            Context {
                drift_5: if long { 0.01 } else { -0.01 },
                drift_20: if long { 0.02 } else { -0.02 },
                latest_bar_end: s.open,
            },
        )
    }
    #[test]
    fn production_allowance_is_separate_from_research_stress_and_keeps_floor() {
        let (session, b, c) = setup(true);
        let at = session.open + Duration::minutes(30);
        let (bid, ask) = (100.474, 100.486); // exactly 48bp midpoint impulse
        let operational =
            super::super::cost_reserve(ask - bid, 0.0, LIVE_SLIPPAGE_BPS_PER_SIDE, ask).unwrap();
        let stress = super::super::cost_reserve(ask - bid, 0.0, 24.0, ask).unwrap();
        assert!(ArmedPullback::arm(
            "BTCUSD",
            session,
            b,
            bid,
            ask,
            at,
            c,
            None,
            operational.total
        )
        .is_ok());
        assert_eq!(
            ArmedPullback::arm("BTCUSD", session, b, bid, ask, at, c, None, stress.total),
            Err("cost_ge_r")
        );
        assert_eq!(
            ArmedPullback::arm(
                "BTCUSD",
                session,
                b,
                100.46,
                100.48,
                at,
                c,
                None,
                operational.total
            ),
            Err("impulse_below_48bps")
        );
        assert!(two_stage_policy("s3_macro_pullback_owner_18377_v1"));
        assert!(two_stage_policy(POLICY));
        assert!(!two_stage_policy("unknown_future_policy"));
    }

    #[test]
    fn monday_macro_excludes_weekends_but_never_fills_a_missing_active_slot() {
        let rule = sessions::rule_for("US500", crate::universe::AssetClass::EquityIndex).unwrap();
        let session =
            sessions::session_on(&rule, chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap())
                .unwrap();
        let mut bars = Vec::new();
        for ago in 1..=45 {
            if let Some(s) = sessions::session_on(&rule, session.date - Duration::days(ago)) {
                let mut at = s.open;
                while at + Duration::minutes(30) <= s.close {
                    bars.push(M30 {
                        open_time: at,
                        open: 100.0,
                        close: 101.0,
                    });
                    at += Duration::minutes(30);
                }
            }
        }
        assert!(context(&rule, session, &bars, session.open).is_ok());
        bars.remove(0);
        assert_eq!(
            context(&rule, session, &bars, session.open),
            Err("macro_missing_m30_segment".into())
        );
    }

    #[test]
    fn fractional_fill_breakeven_uses_first_non_losing_price_tick() {
        assert_eq!(breakeven(true, 100.005, 1.0, 100.995, 99.0, 2), None);
        assert_eq!(
            breakeven(true, 100.005, 1.0, 101.005, 99.0, 2),
            Some(100.01)
        );
        assert_eq!(
            breakeven(false, 100.005, 1.0, 99.005, 101.1, 2),
            Some(100.00)
        );
        assert_eq!(breakeven(true, 100.005, 1.0, 102.0, 100.01, 2), None);
    }

    #[test]
    fn core_macro_and_causal_gates() {
        let (s, b, c) = setup(true);
        let at = s.open + Duration::minutes(30);
        for symbol in [
            "NatGas", "AUDUSD", "NZDUSD", "GBPUSD", "USDJPY", "EURUSD", "USDCAD", "AUS200",
            "JPN225", "UK100", "GER40",
        ] {
            assert_eq!(
                ArmedPullback::arm(symbol, s, b, 100.99, 101.01, at, c, None, 0.2),
                Err("symbol_retired_from_s3_core")
            );
        }
        for symbol in CORE {
            assert_eq!(peer(peer(symbol).unwrap()), Some(symbol));
        }
        assert_eq!(
            ArmedPullback::arm(
                "BTCUSD",
                s,
                b,
                100.99,
                101.01,
                at,
                Context {
                    drift_20: -0.1,
                    ..c
                },
                None,
                0.2
            ),
            Err("vetoed_by_5d_20d_macro_trend")
        );
        assert_eq!(
            ArmedPullback::arm("BTCUSD", s, b, 100.99, 101.01, at, c, Some(-0.1), 0.2),
            Err("peer_coherence")
        );
        assert!(!aligns(
            Context {
                latest_bar_end: s.open + Duration::seconds(1),
                ..c
            },
            true,
            s.open
        ));
    }
    #[test]
    fn peak_never_enters_fresh_midpoint_retest_or_expiry_does() {
        for long in [true, false] {
            let (s, b, c) = setup(long);
            let at = s.open + Duration::minutes(30);
            let sign = if long { 1.0 } else { -1.0 };
            let extreme = 100.0 + sign;
            let a = ArmedPullback::arm(
                "BTCUSD",
                s,
                b,
                extreme - 0.01,
                extreme + 0.01,
                at,
                c,
                None,
                0.2,
            )
            .unwrap();
            assert_eq!(a.touch(extreme - 0.01, extreme + 0.01, at), Touch::Waiting);
            assert_eq!(a.touch(a.level - 0.01, a.level + 0.01, at), Touch::Waiting);
            assert_eq!(
                a.touch(
                    a.level - 0.01,
                    a.level + 0.01,
                    at + Duration::milliseconds(1)
                ),
                Touch::Trigger
            );
            assert_eq!(
                a.touch(99.99, 100.01, at + Duration::seconds(1)),
                Touch::Invalidated
            );
            assert_eq!(
                a.touch(a.level - 0.01, a.level + 0.01, s.close),
                Touch::Expired
            );
        }
    }
    #[test]
    fn exact_two_stage_geometry_and_no_early_trailing() {
        for long in [true, false] {
            let sign = if long { 1.0 } else { -1.0 };
            let fill = 100.0 + sign * 0.5;
            let g = geometry(long, fill, 100.0, 0.5, 2).unwrap();
            assert_eq!(g.risk, 1.0);
            assert_eq!(g.target_distance, 1.38);
            assert!(breakeven(long, fill, g.risk, fill + sign * 0.99, g.stop, 2).is_none());
            assert_eq!(
                breakeven(long, fill, g.risk, fill + sign, g.stop, 2),
                Some(fill)
            );
            assert!(breakeven(long, fill, g.risk, fill + sign * 1.38, fill, 2).is_none());
        }
    }
    #[test]
    fn macro_requires_complete_native_sessions_not_sparse_endpoints() {
        let (s, _, _) = setup(true);
        let mut bars = Vec::new();
        for day in 1..=45 {
            for m in 0..48 {
                bars.push(M30 {
                    open_time: s.open - Duration::days(day) + Duration::minutes(m * 30),
                    open: 100.0,
                    close: 101.0,
                });
            }
        }
        assert!(context(&SessionRule::UtcDay, s, &bars, s.open).is_ok());
        bars.retain(|b| b.open_time != s.open - Duration::hours(2));
        assert!(context(&SessionRule::UtcDay, s, &bars, s.open).is_err());
    }
    fn quote(at: DateTime<Utc>, bid: f64, ask: f64) -> ReplayQuote {
        ReplayQuote {
            at,
            bid,
            ask,
            bid_at: at,
            ask_at: at,
            peer: None,
        }
    }
    fn replay_setup(long: bool) -> (ArmedPullback, ReplayCosts) {
        let (s, b, c) = setup(long);
        let extreme = if long { 101.0 } else { 99.0 };
        let costs = ReplayCosts {
            prepared_spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 2.0,
        };
        (
            ArmedPullback::arm(
                "BTCUSD",
                s,
                b,
                extreme - 0.01,
                extreme + 0.01,
                s.open + Duration::minutes(30),
                c,
                None,
                0.07,
            )
            .unwrap(),
            costs,
        )
    }
    #[test]
    fn replay_shares_entry_geometry_and_exact_target_in_both_directions() {
        for long in [true, false] {
            let (a, costs) = replay_setup(long);
            let t = a.armed_at;
            let mut path = vec![
                quote(t, a.extreme - 0.01, a.extreme + 0.01),
                quote(
                    t + Duration::milliseconds(1),
                    a.level - 0.01,
                    a.level + 0.01,
                ),
            ];
            let entry = if long { path[1].ask } else { path[1].bid };
            let cost = super::super::cost_reserve(costs.prepared_spread, 0.0, 2.0, entry)
                .unwrap()
                .total;
            let g = geometry(long, entry, a.origin, cost, 2).unwrap();
            let sign = if long { 1.0 } else { -1.0 };
            let ex = entry + sign * g.risk;
            let mk = |at, ex| {
                if long {
                    quote(at, ex, ex + 0.02)
                } else {
                    quote(at, ex - 0.02, ex)
                }
            };
            path.push(mk(t + Duration::seconds(1), ex));
            path.push(mk(t + Duration::seconds(2), g.target + sign * 0.1));
            let r = replay_armed(a, 2, costs, &path).unwrap();
            assert_eq!(r.status, "strict_target_1_38r");
            assert_eq!(r.entry, Some(entry));
            assert_eq!(r.exit, Some(g.target));
            assert_eq!(r.events[0].at, t + Duration::milliseconds(1));
            assert_eq!(r.events[1].kind, "breakeven_amend_effective_next_quote");
            assert_eq!(r.events[1].price, Some(entry));
            assert!(!r.financial_pass);
            assert!(r.net_r_assuming_costs.unwrap() < r.gross_r.unwrap());
        }
    }
    #[test]
    fn replay_censors_missing_close_and_never_backdates_breakeven() {
        let (a, costs) = replay_setup(true);
        let t = a.armed_at;
        let entry = quote(
            t + Duration::milliseconds(1),
            a.level - 0.01,
            a.level + 0.01,
        );
        let initial = replay_armed(a, 2, costs, &[entry]).unwrap();
        assert_eq!(initial.status, "censored_open");
        assert_eq!(initial.exit, None);
        let g = initial.geometry.unwrap();
        let reach = quote(
            t + Duration::seconds(1),
            entry.ask + g.risk,
            entry.ask + g.risk + 0.02,
        );
        let back = quote(t + Duration::seconds(2), entry.ask, entry.ask + 0.02);
        let r = replay_armed(a, 2, costs, &[entry, reach, back]).unwrap();
        assert_eq!(r.status, "breakeven_stop_after_1r");
        assert_eq!(r.exit, Some(entry.ask));
        let close = quote(a.session.close, entry.ask + 0.01, entry.ask + 0.03);
        let r = replay_armed(a, 2, costs, &[entry, close]).unwrap();
        assert_eq!(r.status, "session_end");
        let stale = ReplayQuote {
            ask_at: entry.at - Duration::seconds(7),
            ..entry
        };
        assert_eq!(replay_armed(a, 2, costs, &[stale]).unwrap().entry, None);
        let crossed = ReplayQuote {
            bid: entry.ask + 1.0,
            ..entry
        };
        assert!(replay_armed(a, 2, costs, &[crossed]).is_err());
        assert!(replay_armed(a, 2, costs, &[entry, entry]).is_err());
    }
}
