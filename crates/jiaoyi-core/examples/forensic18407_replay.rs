//! Offline, original-arm-conditioned counterfactual for owner policy18407.
//! Uses recorded ticks and shared production price/session helpers. Account
//! liquidity, background preparation and broker acknowledgements are assumptions,
//! not a historical reconstruction of counterfactual broker/account state.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::strategy3::{cost_reserve, macro_pullback as mp, production_sessions as ps};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Read};

#[derive(Deserialize)]
struct Seed {
    symbol: String,
    symbol_id: i64,
    peer_id: i64,
    peer_origin: f64,
    arm: mp::ArmedPullback,
    digits: i32,
    min_volume: i64,
    volume_step: i64,
    max_volume: Option<i64>,
    equity_proxy: f64,
    commission_round_trip_per_unit: f64,
    old_net_pnl: f64,
}
#[derive(Deserialize)]
struct Request {
    account_id: i64,
    through: DateTime<Utc>,
    enforce_pair: bool,
    seeds: Vec<Seed>,
    // Receipt micros, symbol ID, optional bid/ask in e5 broker units.
    quotes: Vec<(i64, i64, Option<i64>, Option<i64>)>,
}
#[derive(Clone, Copy, Default)]
struct Book {
    bid: Option<(f64, DateTime<Utc>)>,
    ask: Option<(f64, DateTime<Utc>)>,
}
impl Book {
    fn fresh(self, at: DateTime<Utc>) -> Option<(f64, f64)> {
        let ((b, bt), (a, atime)) = (self.bid?, self.ask?);
        (bt <= at
            && atime <= at
            && at - bt <= Duration::seconds(6)
            && at - atime <= Duration::seconds(6)
            && b > 0.0
            && a >= b)
            .then_some((b, a))
    }
}
#[derive(Clone, Copy)]
struct Plan {
    at: DateTime<Utc>,
    extreme: f64,
    spread: f64,
    admitted_bound: f64,
    risk: f64,
    target_distance: f64,
    volume: i64,
}
#[derive(Clone, Copy)]
struct Position {
    exit_deadline: DateTime<Utc>,
    entry: f64,
    risk: f64,
    stop: f64,
    target: f64,
    volume: i64,
}
struct Trial {
    seed: Seed,
    arm: mp::ArmedPullback,
    done: bool,
    plan: Option<Plan>,
    live: Option<Position>,
    events: Vec<Value>,
    net: Option<f64>,
    gross: Option<f64>,
    status: String,
    entered: bool,
    skipped_stale: usize,
    last_refusal: Option<&'static str>,
}
impl Trial {
    fn refuse(&mut self, reason: &'static str, at: DateTime<Utc>) {
        if self.last_refusal != Some(reason) {
            self.events.push(json!({"at":at,"kind":reason}));
            self.last_refusal = Some(reason);
        }
    }
    fn prepare(&self, bid: f64, ask: f64, at: DateTime<Utc>) -> Result<Plan, &'static str> {
        let spread = ask - bid;
        let bound = mp::dispatch_entry_bound(&self.arm, spread, self.seed.digits)?;
        let cost = cost_reserve(
            spread,
            self.seed.commission_round_trip_per_unit,
            mp::LIVE_SLIPPAGE_BPS_PER_SIDE,
            bound.max(self.arm.origin),
        )
        .map_err(|_| "invalid_cost")?;
        if cost.total >= self.arm.impulse_range {
            return Err("cost_ge_impulse");
        }
        let g = mp::dispatch_geometry(&self.arm, spread, cost.total, self.seed.digits)?.geometry;
        let budget = self.seed.equity_proxy * 0.006;
        let step = self.seed.volume_step;
        if step <= 0 || self.seed.min_volume <= 0 || !budget.is_finite() || budget <= 0.0 {
            return Err("invalid_volume_or_equity");
        }
        let mut volume = ((budget / (g.risk / 100.0) / step as f64).floor() as i64) * step;
        if let Some(max) = self.seed.max_volume {
            volume = volume.min(max - max % step);
        }
        if volume < self.seed.min_volume {
            return Err("min_volume_exceeds_trade_risk_budget");
        }
        Ok(Plan {
            at,
            extreme: self.arm.extreme,
            spread,
            admitted_bound: bound,
            risk: g.risk,
            target_distance: g.target_distance,
            volume,
        })
    }
}
fn run(r: Request) -> Result<Value, String> {
    if r.quotes.len() > 2_000_000 || r.seeds.len() > 8 {
        return Err("bounded input limit".into());
    }
    if r.quotes
        .windows(2)
        .any(|w| (w[0].0, w[0].1) > (w[1].0, w[1].1))
    {
        return Err("unordered input".into());
    }
    let mut trials: Vec<_> = r
        .seeds
        .into_iter()
        .map(|seed| {
            let arm = seed.arm;
            let mut t = Trial {
                seed,
                arm,
                done: false,
                plan: None,
                live: None,
                events: vec![],
                net: None,
                gross: None,
                status: "no_confirmed_entry".into(),
                entered: false,
                skipped_stale: 0,
                last_refusal: None,
            };
            if !arm.confirmation_state_valid() {
                t.done = true;
                t.status = "invalid_seed".into();
            }
            if let Err(reason) = ps::symbol_entry_window(&t.seed.symbol, &arm.session, arm.armed_at)
            {
                t.done = true;
                t.status = reason.to_string();
                t.events
                    .push(json!({"at":arm.armed_at,"kind":"entry_window_refused","reason":reason}));
            }
            t
        })
        .collect();
    let mut books: BTreeMap<i64, Book> = BTreeMap::new();
    let mut previous: BTreeMap<i64, i64> = BTreeMap::new();
    for (us, id, bid, ask) in r.quotes {
        let at = DateTime::<Utc>::from_timestamp_micros(us).ok_or("invalid quote timestamp")?;
        if at > r.through {
            break;
        }
        let book = books.entry(id).or_default();
        if let Some(v) = bid {
            book.bid = Some((v as f64 / 100000., at));
        }
        if let Some(v) = ask {
            book.ask = Some((v as f64 / 100000., at));
        }
        if previous.insert(id, us) == Some(us) {
            continue;
        }
        let quote = book.fresh(at);
        let Some(idx) = trials
            .iter()
            .position(|t| t.seed.symbol_id == id && !t.done)
        else {
            continue;
        };
        let Some((bid, ask)) = quote else {
            trials[idx].skipped_stale += 1;
            continue;
        };
        let held_sibling = r.enforce_pair
            && trials
                .iter()
                .any(|t| t.seed.symbol_id == trials[idx].seed.peer_id && t.live.is_some());
        let held_risk: f64 = trials
            .iter()
            .filter_map(|t| t.live.map(|p| (p, t.arm.long)))
            .map(|(p, long)| {
                (if long {
                    p.entry - p.stop
                } else {
                    p.stop - p.entry
                })
                .max(0.0)
                    * p.volume as f64
                    / 100.
            })
            .sum();
        let peer = books
            .get(&trials[idx].seed.peer_id)
            .copied()
            .and_then(|b| b.fresh(at));
        let t = &mut trials[idx];
        if at <= t.arm.armed_at {
            continue;
        }
        let sign = if t.arm.long { 1. } else { -1. };
        let executable = if t.arm.long { bid } else { ask };
        if let Some(mut pos) = t.live {
            let exit = if (executable - pos.stop) * sign <= 0. {
                Some((
                    if pos.stop == pos.entry {
                        "breakeven_stop"
                    } else {
                        "initial_stop_loss"
                    },
                    executable,
                ))
            } else if (executable - pos.target) * sign >= 0. {
                Some(("target_1_38r", pos.target))
            } else if at >= pos.exit_deadline {
                Some(("window_close", executable))
            } else {
                None
            };
            if let Some((reason, price)) = exit {
                let gross = (price - pos.entry) * sign * pos.volume as f64 / 100.;
                let cost = (t.seed.commission_round_trip_per_unit
                    + (pos.entry + price) * mp::LIVE_SLIPPAGE_BPS_PER_SIDE / 10000.)
                    * pos.volume as f64
                    / 100.;
                t.gross = Some(gross);
                t.net = Some(gross - cost);
                t.live = None;
                t.done = true;
                t.status = reason.into();
                t.events.push(
                    json!({"at":at,"kind":reason,"price":price,"gross_cash":gross,
                    "modeled_net_cash_3_74bp_each_side":gross-cost,"exit_cost_reserve_cash":cost}),
                );
            } else if let Some(stop) = mp::breakeven(
                t.arm.long,
                pos.entry,
                pos.risk,
                executable,
                pos.stop,
                t.seed.digits,
            ) {
                pos.stop = stop;
                t.live = Some(pos);
                t.events
                    .push(json!({"at":at,"kind":"breakeven_effective_next_quote","stop":stop}));
            }
            continue;
        }
        if held_sibling {
            t.done = true;
            t.status = "pair_risk_reservation_active".into();
            t.events.push(
                json!({"at":at,"kind":"pair_risk_reservation_active","action":"arm_disarmed"}),
            );
            continue;
        }
        let update = t.arm.advance_confirmed(
            ps::arm_expiry(&t.seed.symbol, &t.arm.session, t.arm.armed_at),
            bid,
            ask,
            at,
        );
        match update {
            mp::ConfirmationUpdate::Extended => {
                t.plan = None;
                t.events.push(json!({"at":at,"kind":"running_extreme_extended","extreme":t.arm.extreme,"level":t.arm.level}));
            }
            mp::ConfirmationUpdate::Touched => {
                t.events.push(
                    json!({"at":at,"kind":"zone_touch_or_deeper_touch","state":t.arm.confirmation}),
                );
            }
            mp::ConfirmationUpdate::InvalidatedDeep
            | mp::ConfirmationUpdate::Expired
            | mp::ConfirmationUpdate::InvalidState => {
                t.done = true;
                t.status = match update {
                    mp::ConfirmationUpdate::InvalidatedDeep => {
                        "invalidated_deep_retracement_gt_61_8pct"
                    }
                    mp::ConfirmationUpdate::Expired => "expired",
                    _ => "invalid_state",
                }
                .into();
                t.events.push(
                    json!({"at":at,"kind":t.status,"mid":(bid+ask)/2.,"extreme":t.arm.extreme}),
                );
                continue;
            }
            mp::ConfirmationUpdate::Trigger => {
                let market = match t.seed.symbol.as_str() {
                    "ETHUSD" | "BTCUSD" => ps::Market::Crypto,
                    "SpotCrude" => ps::Market::Crude,
                    "SpotBrent" => ps::Market::Brent,
                    "US500" | "NAS100" => ps::Market::Index,
                    "XAUUSD" | "XAGUSD" => ps::Market::Metal,
                    _ => return Err("unsupported symbol".into()),
                };
                let current_window = ps::session_at(market, at).ok_or("no current session")?;
                if let Err(reason) = ps::symbol_entry_window(&t.seed.symbol, &current_window, at) {
                    t.done = true;
                    t.status = reason.to_string();
                    t.events
                        .push(json!({"at":at,"kind":"entry_window_refused","reason":reason}));
                    continue;
                }
                let peer_bps =
                    peer.map(|(b, a)| ((b + a) / 2. / t.seed.peer_origin - 1.) * 10000. * sign);
                let plan = t.plan.filter(|p| {
                    p.at < at && at - p.at <= Duration::seconds(5) && p.extreme == t.arm.extreme
                });
                let executable_entry = if t.arm.long { ask } else { bid };
                let price_guard_refusal = plan.and_then(|p| {
                    let scale = 10_f64.powi(t.seed.digits);
                    let on_grid = (executable_entry * scale - (executable_entry * scale).round())
                        .abs()
                        <= 1e-6;
                    let current = jiaoyi_core::strategy3::targets::price_ticks(
                        executable_entry,
                        t.seed.digits,
                    );
                    let bound = jiaoyi_core::strategy3::targets::price_ticks(
                        p.admitted_bound,
                        t.seed.digits,
                    );
                    if !on_grid
                        || !matches!((current,bound),(Some(c),Some(b)) if
                        (t.arm.long && c<=b)||(!t.arm.long && c>=b))
                    {
                        return Some("entry_pullback_bound_exceeded");
                    }
                    let repriced = cost_reserve(
                        p.spread.max(ask - bid),
                        t.seed.commission_round_trip_per_unit,
                        mp::LIVE_SLIPPAGE_BPS_PER_SIDE,
                        executable_entry,
                    );
                    if !repriced.is_ok_and(|c| c.total < t.arm.impulse_range) {
                        return Some("entry_cost_ge_r_or_invalid");
                    }
                    None
                });
                let refusal = if plan.is_none() {
                    Some("prepared_admission_not_available")
                } else if price_guard_refusal.is_some() {
                    price_guard_refusal
                } else if !peer_bps.is_some_and(|bp| bp >= mp::PEER_HOLD_BPS) {
                    Some("peer_hold_below_24bps_or_unavailable")
                } else if plan.is_some_and(|p| {
                    held_risk + p.risk * p.volume as f64 / 100. > t.seed.equity_proxy * 0.03 + 1e-8
                }) {
                    Some("aggregate_s3_risk_cap")
                } else {
                    None
                };
                if let Some(reason) = refusal {
                    t.refuse(reason, at);
                    if plan.is_some() {
                        t.done = true;
                        t.status = format!("consumed_then_refused_{reason}");
                        continue;
                    }
                } else {
                    let p = plan.ok_or("missing checked plan")?;
                    let proof = t
                        .arm
                        .confirmation_proof(bid, ask, at)
                        .ok_or("missing checked confirmation")?;
                    if !mp::confirmed_retest(&proof, bid, ask, at) {
                        return Err("invalid final confirmation".into());
                    }
                    let fill = if t.arm.long { ask } else { bid };
                    let market = match t.seed.symbol.as_str() {
                        "ETHUSD" | "BTCUSD" => ps::Market::Crypto,
                        "SpotCrude" => ps::Market::Crude,
                        "SpotBrent" => ps::Market::Brent,
                        "US500" | "NAS100" => ps::Market::Index,
                        "XAUUSD" | "XAGUSD" => ps::Market::Metal,
                        _ => return Err("unsupported symbol".into()),
                    };
                    let exit_deadline = ps::session_at(market, at)
                        .ok_or("missing execution window")?
                        .close;
                    let pos = Position {
                        exit_deadline,
                        entry: fill,
                        risk: p.risk,
                        stop: fill - sign * p.risk,
                        target: fill + sign * p.target_distance,
                        volume: p.volume,
                    };
                    t.entered = true;
                    t.live = Some(pos);
                    t.status = "open_at_capture_end".into();
                    t.events.push(json!({"at":at,"kind":"entry","fill":fill,"stop":pos.stop,"target":pos.target,
                        "risk":p.risk,"volume_cents":p.volume,"peer_hold_bps":peer_bps,"confirmation":proof,
                        "prepared_at":p.at,"exit_deadline":exit_deadline,"equity_proxy":t.seed.equity_proxy}));
                    continue;
                }
            }
            mp::ConfirmationUpdate::Waiting => {}
        }
        match t.prepare(bid, ask, at) {
            Ok(plan) => t.plan = Some(plan),
            Err(reason) => {
                t.plan = None;
                t.refuse(reason, at);
            }
        }
    }
    let unclosed = trials.iter().any(|t| t.live.is_some());
    let net: f64 = trials.iter().filter_map(|t| t.net).sum();
    let gross: f64 = trials.iter().filter_map(|t| t.gross).sum();
    Ok(
        json!({"account_id":r.account_id,"policy":mp::POLICY,"through":r.through,
        "pair_enforced":r.enforce_pair,"old_net_pnl":trials.iter().map(|t|t.seed.old_net_pnl).sum::<f64>(),
        "modeled_closed_gross_cash":gross,"modeled_closed_net_cash_3_74bp":net,
        "counterfactual_net_pnl":if unclosed {None}else{Some(net)},"unresolved":unclosed,
        "financial_pass":false,"trials":trials.into_iter().map(|t|json!({"symbol":t.seed.symbol,"status":t.status,
            "entered":t.entered,"skipped_stale_quote_states":t.skipped_stale,"final_arm":t.arm,
            "gross_cash":t.gross,"net_cash_3_74bp":t.net,"events":t.events})).collect::<Vec<_>>()}),
    )
}
fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("read request");
    let result = serde_json::from_str::<Request>(&input)
        .map_err(|e| e.to_string())
        .and_then(run);
    match result {
        Ok(v) => println!("{}", serde_json::to_string(&v).expect("finite JSON")),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at() -> DateTime<Utc> {
        "2026-11-09T04:30:00Z".parse().unwrap()
    }
    fn seed(symbol: &str, id: i64, peer_id: i64) -> Seed {
        let open: DateTime<Utc> = "2026-11-09T04:00:00Z".parse().unwrap();
        Seed {
            symbol: symbol.into(),
            symbol_id: id,
            peer_id,
            peer_origin: 100.,
            arm: mp::ArmedPullback {
                long: false,
                origin: 100.,
                extreme: 99.,
                level: 99.5,
                impulse_bps: 100.,
                impulse_range: 1.,
                armed_at: at(),
                session: jiaoyi_core::strategy3::sessions::Session {
                    date: open.date_naive(),
                    open,
                    close: open + Duration::hours(4),
                    lunch: None,
                },
                macro_context: mp::Context {
                    drift_5: -0.01,
                    drift_20: -0.02,
                    latest_bar_end: open - Duration::minutes(30),
                },
                confirmation: Default::default(),
            },
            digits: 2,
            min_volume: 1,
            volume_step: 1,
            max_volume: None,
            equity_proxy: 10000.,
            commission_round_trip_per_unit: 0.,
            old_net_pnl: 0.,
        }
    }
    fn q(n: i64, id: i64, mid: f64) -> (i64, i64, Option<i64>, Option<i64>) {
        (
            (at() + Duration::seconds(n)).timestamp_micros(),
            id,
            Some(((mid - 0.01) * 100000.).round() as i64),
            Some(((mid + 0.01) * 100000.).round() as i64),
        )
    }
    #[test]
    fn first_peer_fill_disarms_sibling_and_open_cash_stays_censored() {
        let r = Request {
            account_id: 1,
            through: at() + Duration::seconds(10),
            enforce_pair: true,
            seeds: vec![seed("BTCUSD", 1, 2), seed("ETHUSD", 2, 1)],
            quotes: vec![
                q(1, 1, 99.),
                q(2, 2, 99.),
                q(3, 1, 99.55),
                q(4, 2, 99.55),
                q(5, 1, 99.46),
                q(6, 2, 99.46),
            ],
        };
        let result = run(r).unwrap();
        assert_eq!(result["trials"][0]["entered"], true);
        assert_eq!(
            result["trials"][1]["status"],
            "pair_risk_reservation_active"
        );
        assert_eq!(result["counterfactual_net_pnl"], Value::Null);
    }
    #[test]
    fn final_peer_refusal_consumes_original_arm_instead_of_retrying() {
        let r = Request {
            account_id: 1,
            through: at() + Duration::seconds(10),
            enforce_pair: true,
            seeds: vec![seed("ETHUSD", 2, 1)],
            quotes: vec![
                q(1, 1, 99.),
                q(2, 2, 99.55),
                q(3, 1, 99.9),
                q(4, 2, 99.46),
                q(5, 1, 99.),
                q(6, 2, 99.45),
            ],
        };
        let result = run(r).unwrap();
        assert_eq!(result["trials"][0]["entered"], false);
        assert_eq!(
            result["trials"][0]["status"],
            "consumed_then_refused_peer_hold_below_24bps_or_unavailable"
        );
    }
}
