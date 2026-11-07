//! №18,353: native cost admission with independent 48bp R floor.
//! R is a market-band distance, not a function of C. Two-stage geometry is fixed.
use super::{
    exit_replay as exits, offline_repaired, option_ab,
    s3_signal::{Bar, Request},
    s3_ticks,
    three_year::{self, Payload},
};
use chrono::{DateTime, Duration};
use serde_json::{json, Value};

pub fn study(req: &Request, payload: &Payload) -> Result<Value, String> {
    // Validates the same immutable corpus/scope/seal before any experimental arm.
    let base = three_year::study(req, payload)?;
    let original = &base["models"]["production_s3"];
    let mut qr = req.clone();
    qr.from = payload.source_from;
    let (quotes, quality) = s3_ticks::snapshots(&qr, &payload.quotes)?;
    let mapped = if matches!(req.symbol.as_str(), "US500" | "XAUUSD") {
        2.0
    } else {
        3.74
    };
    let mut arms = serde_json::Map::new();
    for (name, bps) in [
        ("arm_1_baseline_24bp_geometry", 24.0),
        ("arm_2_native_3_74bp_geometry", 3.74),
        ("arm_3_native_2_00bp_geometry", 2.0),
        ("arm_4_symbol_mapped_native_geometry", mapped),
    ] {
        let Some(spread) = original.get("prepared_spread").and_then(Value::as_f64) else {
            arms.insert(name.into(),json!({"bps":bps,"status":"missing_preparation","marks":[],"strict":null,"forced":null}));
            continue;
        };
        let mut native_req = qr.clone();
        native_req.spread = spread;
        let mut source = payload.quotes.clone();
        source.bars = payload
            .m30
            .iter()
            .map(|b| Bar {
                at: DateTime::from_timestamp_millis(b.ts_ms).unwrap(),
                open: b.open_e5 as f64 / 100000.0,
                close: b.close_e5 as f64 / 100000.0,
            })
            .collect();
        let native = s3_ticks::study_with_native_cost_floor(&native_req, &source, bps)?;
        let production_entry = native.trade.as_ref().map(|t| t.entry.clone());
        let decision = native.trade.as_ref().map(|t| t.decision_at);
        let marks = json!(native.marks);
        let mut arm_req = req.clone();
        arm_req.slippage_bps_per_side = bps;
        let staged = offline_repaired::two_stage(
            &arm_req,
            payload,
            &json!({"study":native,"prepared_spread":spread}),
        )?;
        let Some(t) = staged
            .get("study")
            .and_then(|s| s.get("trade"))
            .filter(|t| !t.is_null())
        else {
            arms.insert(
                name.into(),
                json!({"bps":bps,"status":"no_entry","marks":marks,"strict":null,"forced":null}),
            );
            continue;
        };
        let mut input: exits::Input =
            serde_json::from_value(t["entry"].clone()).map_err(|e| e.to_string())?;
        if input.initial_r * 10000.0 < 48.0 * input.fill {
            return Err("48bp dispatch invariant".into());
        }
        // Matches №18,351: censor at the observation boundary, no earlier clock close.
        input.session_end = input.entered_at + Duration::hours(72);
        let strict = option_ab::tick_run(&input, &quotes, req.to, false)?;
        let forced = option_ab::tick_run(&input, &quotes, req.to, true)?;
        let c = marks
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["reason"] == "simulated_entry")
            .map(|m| m["cost"].clone());
        let p = production_entry.unwrap();
        arms.insert(name.into(),json!({"bps":bps,"status":"admitted","marks":marks,"decision_at":decision,
            "strict":strict,"forced":forced,"geometry":{
                "r_bps":input.initial_r/input.fill*10000.0,"initial_stop_distance":input.initial_r,
                "breakeven_trigger_distance":input.initial_r,"two_stage_target_distance":input.tp1_distance,
                "native_plan_cost_price_units":c,"native_production_tp1_distance":p.tp1_distance,
                "native_production_broker_tp_distance":p.broker_tp_distance,
                "native_production_tp1_volume":p.tp1_close_volume,
                "production_geometry_applied_to_two_stage":false}}));
    }
    Ok(
        json!({"arms":arms,"quality":quality,"financial_pass":false,"new_broker_requests":0,
        "geometry_interpretation":"Native C changes admission and production planner diagnostics. The explicitly retained two-stage R/BE/gross-target distances are independent of C at a fixed entry."}),
    )
}

#[cfg(test)]
mod tests {
    use crate::strategy3::targets::{self, Dir, Input};
    #[test]
    fn cost_changes_production_target_but_never_original_r() {
        for side in [Dir::Buy, Dir::Sell] {
            let stop = if side == Dir::Buy { 99.0 } else { 101.0 };
            let input = Input {
                side,
                entry: 100.,
                original_stop: stop,
                required_rr: 1.38,
                cost_reserve: 0.5,
                digits: 2,
                volume: 100,
                min_volume: 100,
                step_volume: 100,
            };
            let high = targets::plan(&input).unwrap();
            let low = targets::plan(&Input {
                cost_reserve: 0.05,
                ..input
            })
            .unwrap();
            assert_eq!(high.initial_risk, low.initial_risk);
            assert!((high.trigger - 100.).abs() > (low.trigger - 100.).abs());
            assert_eq!((low.initial_risk * 138.).ceil() / 100., 1.38);
        }
    }
}
