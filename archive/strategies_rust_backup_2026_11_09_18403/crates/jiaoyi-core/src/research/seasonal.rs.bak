//! Frozen Stage 1 offline hypotheses. No live/HTTP admission changes.
use super::{
    exit_replay as exits,
    opening_range::EntryRule,
    s3_signal::{Bar, Request},
    s3_ticks::{self, Mark, Snapshot, Source, Study, Trade},
};
use crate::strategy3::{
    cost_reserve, sessions,
    targets::{self, Dir},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct M30 {
    pub ts_ms: i64,
    pub open_e5: i64,
    pub high_e5: i64,
    pub low_e5: i64,
    pub close_e5: i64,
    pub volume: i64,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub quotes: Source,
    pub m30: Vec<M30>,
}
pub(super) fn held(at: DateTime<Utc>) -> bool {
    at >= "2025-08-19T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
        && at < "2025-09-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
}
pub(super) fn mid(q: &Snapshot) -> f64 {
    (q.quote.bid + q.quote.ask) / 2.0
}
pub(super) fn executable(q: &Snapshot, side: Dir) -> f64 {
    if side == Dir::Buy {
        q.quote.ask
    } else {
        q.quote.bid
    }
}
pub(super) fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some((values[(n - 1) / 2] + values[n / 2]) / 2.0)
}
pub(super) fn spread(quotes: &[Snapshot], end: DateTime<Utc>) -> Option<f64> {
    median(
        quotes
            .iter()
            .filter(|q| q.fresh && q.quote.at < end)
            .map(|q| q.quote.ask - q.quote.bid)
            .collect(),
    )
}
pub(super) fn range_at(bars: &[M30], start: DateTime<Utc>, minutes: i64) -> Option<(f64, f64)> {
    let mut lo = i64::MAX;
    let mut hi = 0;
    for offset in (0..minutes).step_by(30) {
        let at = (start + Duration::minutes(offset)).timestamp_millis();
        let b = bars.iter().find(|b| b.ts_ms == at && b.volume > 0)?;
        lo = lo.min(b.low_e5);
        hi = hi.max(b.high_e5);
    }
    (hi > lo).then_some((lo as f64 / 100000.0, hi as f64 / 100000.0))
}
fn prior_median(req: &Request, bars: &[M30], minutes: i64) -> Option<f64> {
    let past = sessions::sessions_before(&req.rule(), req.from, 20);
    if past.len() != 20 {
        return None;
    }
    let ranges: Option<Vec<_>> = past
        .iter()
        .map(|s| range_at(bars, s.open, minutes).map(|(l, h)| h - l))
        .collect();
    median(ranges?)
}
pub(super) fn empty(req: &Request, src: &Source, mode: &'static str) -> Result<Study, String> {
    Ok(Study { mode, quality:s3_ticks::snapshots(req,src)?.1, marks:vec![], trade:None,
        fx_diagnostic:None, prediction_features:None, notes:vec![
        "Frozen offline scenario, not account performance; current contracts and costs are unverified.",
        "ORB uses broker single-price M30 highs/lows and exactly 20 prior registered sessions; no future bars or skipped missing sessions.",
        "Primary admission/geometry stays 24bp/side; sensitivities reprice the same admitted fills only.",
    ]})
}
pub(super) fn mark(at: DateTime<Utc>, reason: &'static str) -> Mark {
    Mark {
        at,
        reason,
        decision_at: None,
        side: None,
        r: None,
        cost: None,
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn exit_trade(
    req: &Request,
    src: &Source,
    quotes: &[Snapshot],
    fi: usize,
    decision_at: DateTime<Utc>,
    side: Dir,
    r: f64,
    prepared_spread: f64,
) -> Result<Trade, String> {
    let fill = executable(&quotes[fi], side);
    let c = cost_reserve(prepared_spread, 0.0, 24.0, fill)?.total;
    let opts = req.tick_replay.as_ref().unwrap();
    let p = targets::plan(&targets::Input {
        side,
        entry: fill,
        original_stop: fill - if side == Dir::Buy { r } else { -r },
        required_rr: targets::REQUIRED_RR,
        cost_reserve: c,
        digits: src.contract.digits,
        volume: opts.volume,
        min_volume: src.contract.min_volume,
        step_volume: src.contract.step_volume,
    })
    .map_err(str::to_string)?;
    let input = exits::Input {
        entered_at: quotes[fi].quote.at,
        fill,
        side: if side == Dir::Buy {
            exits::Side::Long
        } else {
            exits::Side::Short
        },
        initial_r: r,
        volume: opts.volume,
        tp1_close_volume: p.close_volume,
        tp1_distance: (p.trigger - fill).abs(),
        broker_tp_distance: (p.broker_target - fill).abs(),
        session_end: req.to,
        digits: src.contract.digits,
        cadence: exits::TrailCadence::S3ThirtySeconds,
    };
    let mut engine = exits::ExitReplay::new(input.clone()).map_err(|e| e.to_string())?;
    for q in quotes.iter().skip(fi) {
        engine.push(q.quote).map_err(|e| format!("{e:?}"))?;
    }
    let through = req.to + Duration::seconds(5);
    let comparison = engine.finish(
        exits::Coverage {
            through,
            censored: true,
            ..exits::Coverage::recorded_path(through)
        },
        exits::Costs::default(),
    );
    Ok(Trade {
        decision_at,
        trailing_scenario_net_r: s3_ticks::net(req, &input, &comparison.candidate),
        fixed_stop_scenario_net_r: s3_ticks::net(req, &input, &comparison.baseline),
        entry: input,
        exit_comparison: comparison,
    })
}
fn orb(
    req: &Request,
    src: &Source,
    bars: &[M30],
    quotes: &[Snapshot],
    minutes: i64,
) -> Result<Value, String> {
    let mut study = empty(req, src, "seasonal_orb_range_expansion_v1")?;
    let at = req.from + Duration::minutes(minutes);
    let Some((lo, hi)) = range_at(bars, req.from, minutes) else {
        study.marks.push(mark(at, "missing_opening_range"));
        return Ok(json!({"study":study}));
    };
    let Some(prior) = prior_median(req, bars, minutes) else {
        study.marks.push(mark(at, "missing_exact_20_prior_ranges"));
        return Ok(json!({"study":study}));
    };
    let width = hi - lo;
    if width <= 1.25 * prior {
        study.marks.push(mark(at, "range_expansion_veto"));
        return Ok(json!({"study":study,"range":width,"prior_median":prior}));
    }
    let Some(prepared_spread) = spread(quotes, at) else {
        study.marks.push(mark(at, "missing_prepared_spread"));
        return Ok(json!({"study":study}));
    };
    let quantum = 10_f64.powi(-src.contract.digits);
    let r = (width / quantum).round().max(1.0) * quantum;
    let direction = |q: &Snapshot| {
        if mid(q) > hi {
            Some(Dir::Buy)
        } else if mid(q) < lo {
            Some(Dir::Sell)
        } else {
            None
        }
    };
    for minute in (minutes..240).step_by(30) {
        let at = req.from + Duration::minutes(minute);
        let mut m = mark(at, "no_fresh_quote_in_mark_window");
        if let Some((qi, q)) = quotes
            .iter()
            .enumerate()
            .find(|(_, q)| q.fresh && q.quote.at >= at && q.quote.at < at + Duration::seconds(5))
        {
            m.decision_at = Some(q.quote.at);
            m.reason = "inside_opening_range";
            if let Some(side) = direction(q) {
                let c = cost_reserve(prepared_spread, 0.0, 24.0, executable(q, side))?.total;
                m.side = Some(side);
                m.r = Some(r);
                m.cost = Some(c);
                if c >= r {
                    m.reason = "cost_ge_r_session_veto";
                    study.marks.push(m);
                    break;
                }
                m.reason = "no_next_fresh_fill_quote_before_deadline";
                if let Some((fi, fq)) = quotes.iter().enumerate().skip(qi + 1).find(|(_, f)| {
                    f.fresh && f.quote.at > q.quote.at && f.quote.at < at + Duration::seconds(5)
                }) {
                    if direction(fq) != Some(side) {
                        m.reason = "direction_changed_before_simulated_fill";
                    } else {
                        let c =
                            cost_reserve(prepared_spread, 0.0, 24.0, executable(fq, side))?.total;
                        m.cost = Some(c);
                        if c >= r {
                            m.reason = "dispatch_cost_ge_r_session_veto";
                            study.marks.push(m);
                            break;
                        }
                        study.trade = Some(exit_trade(
                            req,
                            src,
                            quotes,
                            fi,
                            q.quote.at,
                            side,
                            r,
                            prepared_spread,
                        )?);
                        m.reason = "simulated_entry";
                        study.marks.push(m);
                        break;
                    }
                }
            }
        }
        study.marks.push(m);
    }
    Ok(json!({"study":study,"range":width,"prior_median":prior,"prepared_spread":prepared_spread}))
}
pub(super) fn control(
    req: &Request,
    src: &Source,
    bars: &[M30],
    quotes: &[Snapshot],
    always_long: bool,
) -> Result<Value, String> {
    let at = req.from + Duration::hours(1);
    let missing = |reason: &str| json!({"status":reason,"control":null});
    let Some((lo, hi)) = range_at(bars, req.from, 60) else {
        return Ok(missing("missing_opening_range"));
    };
    let Some(open) = bars
        .iter()
        .find(|b| b.ts_ms == req.from.timestamp_millis() && b.volume > 0)
    else {
        return Ok(missing("missing_session_open"));
    };
    let Some((qi, q)) = quotes
        .iter()
        .enumerate()
        .find(|(_, q)| q.fresh && q.quote.at >= at && q.quote.at < at + Duration::seconds(5))
    else {
        return Ok(missing("no_fresh_quote_in_mark_window"));
    };
    let change = mid(q) - open.open_e5 as f64 / 100000.0;
    if !always_long && change == 0.0 {
        return Ok(missing("flat_momentum"));
    }
    let side = if always_long || change > 0.0 {
        Dir::Buy
    } else {
        Dir::Sell
    };
    let Some(s) = spread(quotes, at) else {
        return Ok(missing("missing_prepared_spread"));
    };
    let quantum = 10_f64.powi(-src.contract.digits);
    let r = ((hi - lo) / quantum).round().max(1.0) * quantum;
    let c = cost_reserve(s, 0.0, 24.0, executable(q, side))?.total;
    if c >= r {
        return Ok(json!({"status":"cost_ge_r_session_veto","r":r,"cost":c,"prepared_spread":s}));
    }
    let Some((fi, fq)) = quotes.iter().enumerate().skip(qi + 1).find(|(_, f)| {
        f.fresh && f.quote.at > q.quote.at && f.quote.at < at + Duration::seconds(5)
    }) else {
        return Ok(missing("no_next_fresh_fill_quote_before_deadline"));
    };
    if !always_long && (mid(fq) - open.open_e5 as f64 / 100000.0) * change <= 0.0 {
        return Ok(missing("direction_changed_before_simulated_fill"));
    }
    let fill = executable(fq, side);
    let c = cost_reserve(s, 0.0, 24.0, fill)?.total;
    if c >= r {
        return Ok(
            json!({"status":"dispatch_cost_ge_r_session_veto","r":r,"cost":c,"prepared_spread":s}),
        );
    }
    let close = quotes
        .iter()
        .skip(fi)
        .find(|x| x.fresh && x.quote.at >= req.to && x.quote.at <= req.to + Duration::seconds(5));
    let mut nets = BTreeMap::new();
    let mut gross = None;
    if let Some(close) = close {
        let price = if side == Dir::Buy {
            close.quote.bid
        } else {
            close.quote.ask
        };
        let g = (price - fill) * if side == Dir::Buy { 1.0 } else { -1.0 } / r;
        gross = Some(g);
        for bps in [24.0, 2.0, 3.74] {
            nets.insert(bps.to_string(), g - (fill + price) * bps / 10000.0 / r);
        }
    }
    Ok(
        json!({"status":"simulated_entry","control":{"decision_at":q.quote.at,"entered_at":fq.quote.at,
        "fill":fill,"side":side,"initial_r":r,"closed_at":close.map(|q|q.quote.at),"gross_r":gross,
        "net_by_bps":nets,"censored":close.is_none(),"qualified_net_r":null}}),
    )
}

pub fn study(req: &Request, payload: &Payload) -> Result<Value, String> {
    if !matches!(
        req.symbol.as_str(),
        "US500" | "BTCUSD" | "SpotCrude" | "SpotBrent"
    ) || req.account != 49236051
        || req.spread != 0.0
        || req.commission_round_trip != 0.0
        || req.slippage_bps_per_side != 24.0
        || req
            .tick_replay
            .as_ref()
            .is_none_or(|o| !o.entry_rule.is_current())
        || !payload.quotes.bars.is_empty()
    {
        return Err("offline seasonal scope/cost/source mismatch".into());
    }
    let mut bounds = req.clone();
    bounds.tick_replay.as_mut().unwrap().entry_rule = EntryRule::ClassMomentum1h;
    bounds.validate(Utc::now())?; // strict session, four hours, volume and seal; no HTTP relaxation
    if held(req.from) || held(req.to + Duration::seconds(5)) {
        return Err("protected validation interval".into());
    }
    let src = &payload.quotes;
    let c = &src.contract;
    let volume = req.tick_replay.as_ref().unwrap().volume;
    if !(0..=5).contains(&c.digits)
        || c.min_volume <= 0
        || c.step_volume <= 0
        || volume < c.min_volume
        || volume % c.step_volume != 0
    {
        return Err("invalid contract/volume".into());
    }
    let mut last = None;
    for b in &payload.m30 {
        let at = DateTime::from_timestamp_millis(b.ts_ms).ok_or("bad M30 timestamp")?;
        if b.ts_ms % 1800000 != 0
            || last.is_some_and(|v| v >= b.ts_ms)
            || at < req.from - Duration::days(45)
            || at + Duration::minutes(30) > req.to
            || held(at)
            || b.volume < 0
            || b.low_e5 <= 0
            || b.low_e5 > b.open_e5
            || b.open_e5 > b.high_e5
            || b.low_e5 > b.close_e5
            || b.close_e5 > b.high_e5
        {
            return Err("invalid, duplicated, protected or out-of-window M30".into());
        }
        last = Some(b.ts_ms);
    }
    let (quotes, quality) = s3_ticks::snapshots(req, src)?;
    let mut models = serde_json::Map::new();
    let first_mark = req.from + Duration::minutes(30);
    let ps = spread(&quotes, first_mark);
    let opening = payload
        .m30
        .iter()
        .find(|b| b.ts_ms == req.from.timestamp_millis() && b.volume > 0);
    if let (Some(s), Some(open)) = (ps, opening) {
        let mut prepared = req.clone();
        prepared.spread = s;
        let mut source = src.clone();
        source.bars = payload
            .m30
            .iter()
            .map(|b| Bar {
                at: DateTime::from_timestamp_millis(b.ts_ms).unwrap(),
                open: b.open_e5 as f64 / 100000.0,
                close: b.close_e5 as f64 / 100000.0,
            })
            .collect();
        let result = s3_ticks::study_validated(&prepared, &source, true)?;
        models.insert(
            "production_s3".into(),
            json!({"study":result,"prepared_spread":s,"opening_volume":open.volume,
            "preparation":"reconstructed_opening_window_not_historical_preopen_receipt"}),
        );
    } else {
        models.insert(
            "production_s3".into(),
            json!({"status":"missing_opening_volume_or_prepared_spread"}),
        );
    }
    models.insert("orb_30m".into(), orb(req, src, &payload.m30, &quotes, 30)?);
    models.insert("orb_60m".into(), orb(req, src, &payload.m30, &quotes, 60)?);
    models.insert(
        "always_long_control".into(),
        control(req, src, &payload.m30, &quotes, true)?,
    );
    models.insert(
        "own_momentum_fixed_horizon".into(),
        control(req, src, &payload.m30, &quotes, false)?,
    );
    for model in models.values_mut() {
        if let Some(trade_value) = model
            .get("study")
            .and_then(|s| s.get("trade"))
            .filter(|t| !t.is_null())
        {
            // Rerating changes neither admission nor target geometry.
            let trade: Value = trade_value.clone();
            let input: exits::Input =
                serde_json::from_value(trade["entry"].clone()).map_err(|e| e.to_string())?;
            let mut sensitivities = serde_json::Map::new();
            for (arm, field) in [("trailing", "candidate"), ("fixed", "baseline")] {
                let outcome = &trade["exit_comparison"][field];
                let gross = outcome["scenario_gross_r"].as_f64();
                let weighted = outcome["fills"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|f| {
                        f["price"].as_f64().unwrap() * f["volume"].as_i64().unwrap() as f64
                            / input.volume as f64
                    })
                    .sum::<f64>();
                sensitivities.insert(
                    arm.into(),
                    json!({"24":gross.map(|g|g-(input.fill+weighted)*24.0/10000.0/input.initial_r),
                    "2":gross.map(|g|g-(input.fill+weighted)*2.0/10000.0/input.initial_r),
                    "3.74":gross.map(|g|g-(input.fill+weighted)*3.74/10000.0/input.initial_r)}),
                );
            }
            model["sensitivities"] = Value::Object(sensitivities);
        }
    }
    Ok(
        json!({"quality":quality,"models":models,"financial_pass":false,"validation_opened":false,
        "qualification":"SCENARIOS_ONLY_NOT_PRODUCTION_ACCOUNT_PARITY"}),
    )
}

#[cfg(test)]
mod tests {
    use super::super::s3_ticks::{Contract, Options};
    use super::*;
    fn request() -> Request {
        Request {
            account: 49236051,
            symbol: "BTCUSD".into(),
            from: "2024-01-22T00:00:00Z".parse().unwrap(),
            to: "2024-01-22T04:00:00Z".parse().unwrap(),
            spread: 0.0,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 1,
                entry_rule: EntryRule::CurrentS3,
            }),
        }
    }
    #[test]
    fn prior_ranges_exclude_today_and_refuse_missing_session() {
        let req = request();
        let mut bars = vec![];
        for day in 0..=20 {
            for minute in [0, 30] {
                bars.push(M30 {
                    ts_ms: (req.from - Duration::days(day) + Duration::minutes(minute))
                        .timestamp_millis(),
                    open_e5: 10000000,
                    high_e5: 10100000,
                    low_e5: 9900000,
                    close_e5: 10000000,
                    volume: 1,
                });
            }
        }
        assert_eq!(prior_median(&req, &bars, 60), Some(2.0));
        for b in bars
            .iter_mut()
            .filter(|b| b.ts_ms >= req.from.timestamp_millis())
        {
            b.high_e5 = 99900000;
        }
        assert_eq!(prior_median(&req, &bars, 60), Some(2.0));
        bars.retain(|b| b.ts_ms != (req.from - Duration::days(3)).timestamp_millis());
        assert_eq!(prior_median(&req, &bars, 60), None);
    }
    #[test]
    fn seasonal_scope_does_not_relax_http_or_validation() {
        let mut req = request();
        req.symbol = "US500".into();
        req.from = "2024-01-22T14:30:00Z".parse().unwrap();
        req.to = req.from + Duration::hours(4);
        req.tick_replay.as_mut().unwrap().volume = 10;
        let payload = Payload {
            quotes: Source {
                bars: vec![],
                bids: vec![],
                asks: vec![],
                contract: Contract {
                    digits: 1,
                    min_volume: 10,
                    step_volume: 10,
                },
            },
            m30: vec![],
        };
        assert!(req.validate(Utc::now()).is_err());
        assert!(study(&req, &payload).is_ok());
        req.from = "2025-08-20T13:30:00Z".parse().unwrap();
        req.to = req.from + Duration::hours(4);
        assert!(study(&req, &payload).is_err());
    }
    #[test]
    fn orb_uses_only_closed_opening_range_and_sensitivity_holds_entry_fixed() {
        let req = request();
        let mut bars = vec![];
        for day in (1..=20).rev() {
            for minute in [0, 30] {
                bars.push(M30 {
                    ts_ms: (req.from - Duration::days(day) + Duration::minutes(minute))
                        .timestamp_millis(),
                    open_e5: 10000000,
                    high_e5: 10100000,
                    low_e5: 9900000,
                    close_e5: 10000000,
                    volume: 1,
                });
            }
        }
        bars.push(M30 {
            ts_ms: req.from.timestamp_millis(),
            open_e5: 10000000,
            high_e5: 11000000,
            low_e5: 9000000,
            close_e5: 10900000,
            volume: 1,
        });
        let mut src = Source {
            bars: vec![],
            bids: vec![],
            asks: vec![],
            contract: Contract {
                digits: 2,
                min_volume: 1,
                step_volume: 1,
            },
        };
        for (seconds, price) in [
            (0, 10000000),
            (1799, 10900000),
            (1800, 11100000),
            (1801, 11110000),
            (3600, 11500000),
            (14400, 14500000),
        ] {
            let ts_ms = (req.from + Duration::seconds(seconds)).timestamp_millis();
            src.bids.push(s3_ticks::Tick {
                ts_ms,
                price_e5: price,
            });
            src.asks.push(s3_ticks::Tick {
                ts_ms,
                price_e5: price + 1000,
            });
        }
        let mut payload = Payload {
            quotes: src,
            m30: bars,
        };
        let first = study(&req, &payload).unwrap();
        assert_eq!(
            first["models"]["orb_30m"]["study"]["marks"][0]["reason"],
            "simulated_entry"
        );
        let old = &first["models"]["orb_30m"];
        assert!(
            old["sensitivities"]["trailing"]["2"].as_f64().unwrap()
                > old["sensitivities"]["trailing"]["24"].as_f64().unwrap()
        );
        for t in payload
            .quotes
            .bids
            .iter_mut()
            .chain(payload.quotes.asks.iter_mut())
            .filter(|t| t.ts_ms > req.from.timestamp_millis() + 1801000)
        {
            t.price_e5 += 1000000;
        }
        let second = study(&req, &payload).unwrap();
        assert_eq!(
            old["study"]["trade"]["entry"],
            second["models"]["orb_30m"]["study"]["trade"]["entry"]
        );
        assert!(old["study"]["trade"]["exit_comparison"]["candidate"]["net_r"].is_null());
    }
}
