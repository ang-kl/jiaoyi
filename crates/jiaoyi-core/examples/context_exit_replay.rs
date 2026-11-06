//! Offline bounded adapter: native target planner and existing exit engine.
//! No broker, database, registration or live signal integration.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::{
    research::exit_replay::{self as exits, Costs, Coverage, ExitReplay, Quote, Side},
    strategy3::{cost_reserve, targets},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    decision_at: DateTime<Utc>,
    entered_at: DateTime<Utc>,
    decision_price: f64,
    fill: f64,
    side: Side,
    observed_range: f64,
    median_spread: f64,
    volume: i64,
    min_volume: i64,
    step_volume: i64,
    digits: i32,
    session_end: DateTime<Utc>,
    quotes: Vec<Quote>,
}

fn run(r: Request) -> Result<Value, String> {
    let through = r.session_end + Duration::seconds(5);
    if through > jiaoyi_core::seal::seal_instant()
        || r.decision_at >= r.entered_at
        || r.entered_at - r.decision_at >= Duration::seconds(5)
        || r.entered_at >= r.session_end
        || r.session_end - r.decision_at > Duration::hours(3)
        || r.quotes.is_empty()
        || r.quotes.len() > 400_000
        || !(0..=5).contains(&r.digits)
        || !r.observed_range.is_finite()
        || r.observed_range <= 0.0
        || r.fill <= 0.0
        || r.decision_price <= 0.0
        || r.volume > 1_000_000_000
    {
        return Err("invalid bounded sealed replay request".into());
    }
    let first = r.quotes.first().unwrap();
    let price = |q: &Quote| if r.side == Side::Long { q.ask } else { q.bid };
    if first.at != r.entered_at || price(first) != r.fill {
        return Err("fill must equal the first executable quote".into());
    }
    let quantum = 10_f64.powi(-r.digits);
    let initial_r = (r.observed_range / quantum).round().max(1.0) * quantum;
    let cost = cost_reserve(r.median_spread, 0.0, 24.0, r.fill)?.total;
    let decision_cost = cost_reserve(r.median_spread, 0.0, 24.0, r.decision_price)?.total;
    if cost >= initial_r || decision_cost >= initial_r {
        return Ok(
            json!({"status":"candidate_cost_ge_r", "initial_r":initial_r,
            "cost_reserve":cost,"decision_cost_reserve":decision_cost}),
        );
    }
    let long = r.side == Side::Long;
    let target = targets::plan(&targets::Input {
        side: if long {
            targets::Dir::Buy
        } else {
            targets::Dir::Sell
        },
        entry: r.fill,
        original_stop: r.fill - if long { initial_r } else { -initial_r },
        required_rr: targets::REQUIRED_RR,
        cost_reserve: cost,
        digits: r.digits,
        volume: r.volume,
        min_volume: r.min_volume,
        step_volume: r.step_volume,
    })
    .map_err(str::to_owned)?;
    let input = exits::Input {
        entered_at: r.entered_at,
        fill: r.fill,
        side: r.side,
        initial_r,
        volume: r.volume,
        tp1_close_volume: target.close_volume,
        tp1_distance: (target.trigger - r.fill).abs(),
        broker_tp_distance: (target.broker_target - r.fill).abs(),
        session_end: r.session_end,
        digits: r.digits,
        cadence: exits::TrailCadence::S3ThirtySeconds,
    };
    let mut engine = ExitReplay::new(input.clone()).map_err(|e| e.to_string())?;
    let mut last = None;
    for q in r.quotes {
        if q.at < r.entered_at
            || q.at >= through
            || last.is_some_and(|at| q.at <= at)
            || q.bid <= 0.0
            || q.ask <= 0.0
        {
            return Err("quote outside strict ordered replay boundary".into());
        }
        last = Some(q.at);
        engine
            .push(q)
            .map_err(|e| format!("invalid quote: {e:?}"))?;
    }
    let comparison = engine.finish(
        Coverage {
            through,
            censored: true,
            ..Coverage::recorded_path(through)
        },
        Costs::default(),
    );
    let mut sensitivities = Vec::new();
    for (arm, outcome) in [
        ("fixed_stop", &comparison.baseline),
        ("trailing", &comparison.candidate),
    ] {
        let weighted_exit: f64 = outcome
            .fills
            .iter()
            .map(|f| f.price * f.volume as f64 / input.volume as f64)
            .sum();
        for bps in [2.0, 3.74, 24.0] {
            let value = outcome.scenario_gross_r.map(|gross| {
                gross - (input.fill + weighted_exit) * bps / 10000.0 / input.initial_r
            });
            sensitivities.push(json!({"arm":arm,"bps_per_side":bps,
                "scenario_net_r":value,"qualified_net_r":null}));
        }
    }
    Ok(
        json!({"status":"simulated_entry","input":input,"target":target,
        "cost_reserve":cost,"comparison":comparison,"sensitivities":sensitivities}),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    std::io::stdin().take(64_000_001).read_to_end(&mut bytes)?;
    if bytes.len() > 64_000_000 {
        return Err("replay input exceeds64MB".into());
    }
    let result = run(serde_json::from_slice(&bytes)?)?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Request {
        serde_json::from_value(json!({"decision_at":"2025-08-05T08:00:00Z",
            "entered_at":"2025-08-05T08:00:01Z","decision_price":100.0,"fill":100.0,
            "side":"long","observed_range":2.0,"median_spread":0.1,
            "volume":100,"min_volume":10,"step_volume":10,"digits":2,
            "session_end":"2025-08-05T11:00:00Z","quotes":[
                {"at":"2025-08-05T08:00:01Z","bid":99.9,"ask":100.0,"gap_before":false},
                {"at":"2025-08-05T11:00:00Z","bid":101.0,"ask":101.1,"gap_before":true}]}))
        .unwrap()
    }
    #[test]
    fn native_targets_costs_censoring_and_seal() {
        let result = run(fixture()).unwrap();
        assert_eq!(result["target"]["mode"], "partial_runner");
        assert_eq!(result["comparison"]["baseline"]["scenario_gross_r"], 0.5);
        let net = result["sensitivities"][0]["scenario_net_r"]
            .as_f64()
            .unwrap();
        assert!((net - (0.5 - 201.0 * 2.0 / 10000.0 / 2.0)).abs() < 1e-12);
        assert!(result["comparison"]["candidate"]["net_r"].is_null());
        let mut r = fixture();
        r.quotes.pop();
        assert!(run(r).unwrap()["sensitivities"][0]["scenario_net_r"].is_null());
        let mut r = fixture();
        r.observed_range = 0.1;
        assert_eq!(run(r).unwrap()["status"], "candidate_cost_ge_r");
        let mut r = fixture();
        r.session_end = "2026-11-05T11:00:00Z".parse().unwrap();
        assert!(run(r).is_err());
    }

    #[test]
    fn costs_weight_partial_and_runner_exits_once() {
        let mut r = fixture();
        let end = r.session_end;
        r.quotes.truncate(1);
        for (seconds, bid) in [(60, 103.34), (61, 103.35), (62, 105.34)] {
            r.quotes.push(Quote {
                at: end - Duration::hours(3) + Duration::seconds(seconds),
                bid,
                ask: bid + 0.1,
                gap_before: false,
            });
        }
        let result = run(r).unwrap();
        let fills = result["comparison"]["baseline"]["fills"]
            .as_array()
            .unwrap();
        assert_eq!(fills.len(), 2);
        assert_eq!(fills[0]["reason"], "tp1_market_close");
        assert_eq!(fills[1]["reason"], "broker_take_profit");
        assert_eq!(
            fills
                .iter()
                .map(|f| f["volume"].as_i64().unwrap())
                .sum::<i64>(),
            100
        );
        let weighted: f64 = fills
            .iter()
            .map(|f| f["price"].as_f64().unwrap() * f["volume"].as_f64().unwrap() / 100.0)
            .sum();
        let gross = result["comparison"]["baseline"]["scenario_gross_r"]
            .as_f64()
            .unwrap();
        let net = result["sensitivities"][0]["scenario_net_r"]
            .as_f64()
            .unwrap();
        assert!((net - (gross - (100.0 + weighted) * 2.0 / 10000.0 / 2.0)).abs() < 1e-12);
    }
}
