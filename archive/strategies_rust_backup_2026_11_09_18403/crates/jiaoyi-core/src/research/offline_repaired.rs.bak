//! №18,331: frozen offline hypotheses, never a production entry/exit route.
use super::{
    exit_replay as exits,
    repaired_exit::TwoStageReplay,
    s3_signal::{Bar, Request},
    s3_ticks, seasonal,
    three_year::{self, Payload},
};
use chrono::{DateTime, Duration};
use serde_json::{json, Value};

pub(super) fn two_stage(req: &Request, payload: &Payload, source: &Value) -> Result<Value, String> {
    two_stage_with_window_close(req, payload, source, false)
}

fn two_stage_with_window_close(
    req: &Request,
    payload: &Payload,
    source: &Value,
    window_close: bool,
) -> Result<Value, String> {
    let mut model = source.clone();
    let Some(trade) = model
        .get("study")
        .and_then(|s| s.get("trade"))
        .filter(|t| !t.is_null())
    else {
        return Ok(model);
    };
    let mut input: exits::Input =
        serde_json::from_value(trade["entry"].clone()).map_err(|e| e.to_string())?;
    if window_close {
        input.session_end = input.session_end.min(req.to);
    }
    let risk_ticks = crate::strategy3::targets::price_ticks(input.initial_r, input.digits)
        .ok_or("invalid original risk")?;
    // Outward integer-grid rounding; literal gross 1.38R, with costs still charged.
    let distance_ticks = risk_ticks
        .checked_mul(138)
        .and_then(|x| x.checked_add(99))
        .ok_or("target overflow")?
        / 100;
    input.tp1_close_volume = input.volume;
    input.tp1_distance = distance_ticks as f64 / 10_f64.powi(input.digits);
    input.broker_tp_distance = input.tp1_distance;
    let mut engine = TwoStageReplay::new(input.clone()).map_err(|e| e.to_string())?;
    let mut quote_req = req.clone();
    quote_req.from = payload.source_from;
    let (quotes, _) = s3_ticks::snapshots(&quote_req, &payload.quotes)?;
    let through = if req.to == input.session_end {
        req.to + Duration::seconds(5)
    } else {
        req.to
    };
    for q in quotes
        .iter()
        .filter(|q| q.quote.at >= input.entered_at && q.quote.at <= through)
    {
        if window_close && q.quote.at >= input.session_end && !q.fresh {
            continue;
        }
        engine
            .push(q.quote)
            .map_err(|e| format!("invalid repaired exit quote: {e:?}"))?;
    }
    let comparison = engine.finish(
        exits::Coverage {
            censored: true,
            ..exits::Coverage::recorded_path(through)
        },
        exits::Costs::default(),
    );
    let trade = &mut model["study"]["trade"];
    trade["entry"] = json!(input);
    trade["trailing_scenario_net_r"] = json!(s3_ticks::net(req, &input, &comparison.candidate));
    trade["fixed_stop_scenario_net_r"] = json!(s3_ticks::net(req, &input, &comparison.baseline));
    trade["exit_comparison"] = json!(comparison);
    model["study"]["mode"] = json!("offline_two_stage_gross_target_18331");
    model["study"]["notes"] = json!(["Original entry and R held fixed; original stop until +1R, then next-quote entry-price stop; whole-position gross target 1.38R rounded outward. This is not cost-adjusted breakeven or a financial PASS."]);
    model["geometry"] = json!({"trigger_r":1.0,"nominal_gross_target_r":1.38,"actual_gross_target_r":input.tp1_distance/input.initial_r,"further_trailing":false,"partial_exit":false,"qualified_net_r":null});
    if window_close {
        model["study"]["mode"] = json!("offline_two_stage_window_close_18348");
        model["study"]["notes"] = json!(["Owner-requested observation-window close: same entry, R and two-stage brackets; close on a valid quote at/after the four-hour evaluation boundary, within the five-second tail. Missing closing quote remains censored. Separate from the legacy session-end model."]);
        model["geometry"]["window_close_at"] = json!(input.session_end);
    }
    three_year::rerate(&mut model)?;
    Ok(model)
}

/// Owner-requested bounded-close comparison. Existing model and live routes are unchanged.
pub fn high_vol_study(req: &Request, payload: &Payload) -> Result<Value, String> {
    let baseline = three_year::study(req, payload)?;
    let s3 = &baseline["models"]["production_s3"];
    Ok(json!({
        "models":{
            "s3_two_stage":two_stage(req,payload,s3)?,
            "s3_two_stage_window_close":two_stage_with_window_close(req,payload,s3,true)?,
            "always_long_control":baseline["models"]["always_long_control"],
            "own_momentum_fixed_horizon":baseline["models"]["own_momentum_fixed_horizon"],
        },
        "quality":baseline["quality"],"window_quality":baseline["window_quality"],
        "financial_pass":false,"validation_opened":false,
    }))
}

fn dynamic_s3(req: &Request, payload: &Payload, baseline: &Value) -> Result<Value, String> {
    let Some(spread) = baseline.get("prepared_spread").and_then(Value::as_f64) else {
        return Ok(baseline.clone());
    };
    let mut native_req = req.clone();
    native_req.from = payload.source_from;
    native_req.spread = spread;
    let mut src = payload.quotes.clone();
    src.bars = payload
        .m30
        .iter()
        .map(|b| Bar {
            at: DateTime::from_timestamp_millis(b.ts_ms).unwrap(),
            open: b.open_e5 as f64 / 100000.0,
            close: b.close_e5 as f64 / 100000.0,
        })
        .collect();
    let mut study = s3_ticks::study_with_gate(&native_req, &src, true, Some(&payload.m30))?;
    study.mode = "offline_s3_dynamic_admission_18331";
    study.notes = vec!["Only entry admission changes: R must exceed max(2.5 times current spread, 0.15 times causal M30 ATR14), rechecked at dispatch. Original target/exit rules and actual cost deduction remain. No financial PASS."];
    let quotes = s3_ticks::snapshots(&native_req, &src)?.0;
    let diagnostics: Vec<_> = study.marks.iter().filter_map(|m| {
        let q = quotes.iter().find(|q| Some(q.quote.at) == m.decision_at)?;
        let atr = three_year::atr(&payload.m30, m.at);
        let spread = q.quote.ask - q.quote.bid;
        Some(json!({"at":m.at,"r":m.r,"actual_cost_reserve":m.cost,"spread_price_units":spread,"atr14_price_units":atr,"threshold_price_units":atr.map(|a|(2.5*spread).max(0.15*a))}))
    }).collect();
    let mut model = json!({"study":study,"prepared_spread":spread,"dynamic_gate":diagnostics});
    three_year::rerate(&mut model)?;
    Ok(model)
}

pub fn study(req: &Request, payload: &Payload) -> Result<Value, String> {
    // This validates scope, protected dates, contracts, bars, quote bounds and
    // ordering. The baseline is replayed for exact parity with the prior receipt.
    let baseline = three_year::study(req, payload)?;
    let s3 = &baseline["models"]["production_s3"];
    let dynamic = dynamic_s3(req, payload, s3)?;
    let macro_authorized = three_year::authorized_macro(req, payload)?;
    let required_peer = match req.symbol.as_str() {
        "SpotCrude" => "SpotBrent",
        "SpotBrent" => "SpotCrude",
        "US500" => "NAS100",
        "XAUUSD" => "XAGUSD",
        _ => unreachable!(),
    };
    let peer_available = payload.peer_symbol.as_deref() == Some(required_peer)
        && seasonal::range_at(&payload.peer_m30, req.from, 180).is_some();
    Ok(json!({
        "baseline":baseline,
        "models":{
            "s3_two_stage":two_stage(req,payload,s3)?,
            "s3_dynamic_gate":dynamic,
            "s3_dynamic_two_stage":two_stage(req,payload,&dynamic)?,
            "macro_180m_repaired":two_stage(req,payload,&macro_authorized)?,
        },
        "peer":{"required":required_peer,"available":peer_available,"source":payload.peer_symbol},
        "financial_pass":false,"validation_opened":false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn bounded_close_requires_a_closing_quote_and_preserves_bracket_priority() {
        let at: DateTime<Utc> = "2024-11-04T07:00:00Z".parse().unwrap();
        let req = Request {
            account: 49236051,
            symbol: "SpotCrude".into(),
            from: at,
            to: at + Duration::hours(4),
            spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: None,
        };
        let input = exits::Input {
            entered_at: at,
            fill: 100.0,
            side: exits::Side::Long,
            initial_r: 1.0,
            volume: 100,
            tp1_close_volume: 100,
            tp1_distance: 1.38,
            broker_tp_distance: 1.38,
            session_end: at + Duration::hours(10),
            digits: 2,
            cadence: exits::TrailCadence::S3ThirtySeconds,
        };
        let source = json!({"study":{"trade":{"entry":input}}});
        let mut payload: Payload = serde_json::from_value(json!({
            "source_from":at,"m30":[],"peer_symbol":null,"peer_m30":[],
            "quotes":{"bars":[],"contract":{"digits":2,"min_volume":1,"step_volume":1},
                "bids":[{"ts_ms":at.timestamp_millis(),"price_e5":10000000},
                        {"ts_ms":(req.to+Duration::seconds(1)).timestamp_millis(),"price_e5":10050000}],
                "asks":[{"ts_ms":at.timestamp_millis(),"price_e5":10002000},
                        {"ts_ms":(req.to+Duration::seconds(1)).timestamp_millis(),"price_e5":10052000}]}
        })).unwrap();
        let candidate = |v: Value| v["study"]["trade"]["exit_comparison"]["candidate"].clone();
        assert!(
            candidate(two_stage(&req, &payload, &source).unwrap())["scenario_gross_r"].is_null()
        );
        let bounded =
            candidate(two_stage_with_window_close(&req, &payload, &source, true).unwrap());
        assert_eq!(bounded["scenario_gross_r"], json!(0.5));
        assert_eq!(bounded["fills"][0]["reason"], json!("session_end"));
        let last_ask = payload.quotes.asks.pop().unwrap();
        assert!(
            candidate(two_stage_with_window_close(&req, &payload, &source, true).unwrap())
                ["scenario_gross_r"]
                .is_null()
        );
        payload.quotes.asks.push(last_ask);
        payload.quotes.bids[1].price_e5 = 9880000;
        payload.quotes.asks[1].price_e5 = 9882000;
        let stop = candidate(two_stage_with_window_close(&req, &payload, &source, true).unwrap());
        assert_eq!(stop["fills"][0]["reason"], json!("gap_through_stop"));
        payload.quotes.bids.pop();
        payload.quotes.asks.pop();
        assert!(
            candidate(two_stage_with_window_close(&req, &payload, &source, true).unwrap())
                ["scenario_gross_r"]
                .is_null()
        );
    }

    #[test]
    fn two_stage_preserves_original_stop_until_one_r_then_next_quote_and_net_costs() {
        let at: DateTime<Utc> = "2023-01-23T13:00:00Z".parse().unwrap();
        for side in [exits::Side::Long, exits::Side::Short] {
            let sign = if side == exits::Side::Long { 1.0 } else { -1.0 };
            let input = exits::Input {
                entered_at: at,
                fill: 100.0,
                side,
                initial_r: 1.0,
                volume: 100,
                tp1_close_volume: 100,
                tp1_distance: 1.38,
                broker_tp_distance: 1.38,
                session_end: at + Duration::hours(4),
                digits: 2,
                cadence: exits::TrailCadence::S3ThirtySeconds,
            };
            let quote = |sec, r| {
                let p = 100.0 + sign * r;
                exits::Quote {
                    at: at + Duration::seconds(sec),
                    bid: if sign > 0.0 { p } else { p - 0.02 },
                    ask: if sign > 0.0 { p + 0.02 } else { p },
                    gap_before: false,
                }
            };
            let finish = |e: TwoStageReplay, sec| {
                e.finish(
                    exits::Coverage {
                        censored: true,
                        ..exits::Coverage::recorded_path(at + Duration::seconds(sec))
                    },
                    exits::Costs::default(),
                )
            };
            let mut e = TwoStageReplay::new(input.clone()).unwrap();
            e.push(quote(0, 0.99)).unwrap();
            let o = finish(e, 0);
            assert_eq!(o.candidate.trail_proposals, 0);
            assert_eq!(o.candidate.active_stop, 100.0 - sign);
            assert!(o.candidate.scenario_gross_r.is_none());
            let mut e = TwoStageReplay::new(input.clone()).unwrap();
            for (sec, r) in [(0, 1.0), (0, 1.1), (1, 1.1), (2, 0.0)] {
                e.push(quote(sec, r)).unwrap();
            }
            let o = finish(e, 2);
            assert_eq!(o.candidate.trail_proposals, 1);
            assert_eq!(o.candidate.trail_applied, 1);
            assert_eq!(o.candidate.scenario_gross_r, Some(0.0));
            let req = Request {
                account: 49236051,
                symbol: "SpotCrude".into(),
                from: at,
                to: at + Duration::hours(4),
                spread: 0.02,
                commission_round_trip: 0.0,
                slippage_bps_per_side: 24.0,
                tick_replay: None,
            };
            assert!(s3_ticks::net(&req, &input, &o.candidate).unwrap() < 0.0);
            let mut e = TwoStageReplay::new(input.clone()).unwrap();
            e.push(quote(0, 1.0)).unwrap();
            e.push(quote(1, -1.2)).unwrap();
            let o = finish(e, 1);
            assert_eq!(o.candidate.trail_applied, 0);
            assert!((o.candidate.scenario_gross_r.unwrap() + 1.2).abs() < 1e-9);
            let mut e = TwoStageReplay::new(input).unwrap();
            for (sec, r) in [(0, 1.0), (1, 1.1), (2, 2.0)] {
                e.push(quote(sec, r)).unwrap();
            }
            let o = finish(e, 2);
            assert!((o.candidate.scenario_gross_r.unwrap() - 1.38).abs() < 1e-9);
        }
    }
}
