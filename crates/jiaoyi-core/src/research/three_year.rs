//! Frozen three-year research only. No HTTP/live admission changes.
use super::{
    exit_replay as exits,
    s3_signal::{Bar, Request},
    s3_ticks::{self, Snapshot, Source},
    seasonal::{self, M30},
};
use crate::strategy3::{cost_reserve, sessions, targets::Dir};
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub quotes: Source,
    pub m30: Vec<M30>,
    pub source_from: DateTime<Utc>,
    pub peer_symbol: Option<String>,
    pub peer_m30: Vec<M30>,
}
fn validate_bars(bars: &[M30], first: DateTime<Utc>, last: DateTime<Utc>) -> Result<(), String> {
    let mut prev = None;
    for b in bars {
        let at = DateTime::from_timestamp_millis(b.ts_ms).ok_or("bad bar time")?;
        if b.ts_ms % 1800000 != 0
            || prev.is_some_and(|p| p >= b.ts_ms)
            || at < first
            || at + Duration::minutes(30) > last
            || seasonal::held(at)
            || b.volume < 0
            || b.low_e5 <= 0
            || b.low_e5 > b.open_e5
            || b.open_e5 > b.high_e5
            || b.low_e5 > b.close_e5
            || b.close_e5 > b.high_e5
        {
            return Err("invalid/protected/duplicate M30".into());
        }
        prev = Some(b.ts_ms);
    }
    Ok(())
}
fn atr(bars: &[M30], at: DateTime<Utc>) -> Option<f64> {
    let past: Vec<_> = bars
        .iter()
        .filter(|b| b.ts_ms + 1800000 <= at.timestamp_millis() && b.volume > 0)
        .rev()
        .take(15)
        .collect();
    if past.len() != 15 {
        return None;
    }
    Some(
        past.windows(2)
            .map(|b| {
                (b[0].high_e5 - b[0].low_e5)
                    .max((b[0].high_e5 - b[1].close_e5).abs())
                    .max((b[0].low_e5 - b[1].close_e5).abs()) as f64
                    / 100000.0
            })
            .sum::<f64>()
            / 14.0,
    )
}
fn prior_range(req: &Request, bars: &[M30]) -> Option<f64> {
    let native = sessions::session_on(&req.rule(), req.from.date_naive())?;
    let shift = req.from - native.open;
    let ranges: Option<Vec<_>> = sessions::sessions_before(&req.rule(), req.from, 20)
        .iter()
        .map(|s| seasonal::range_at(bars, s.open + shift, 60).map(|(l, h)| h - l))
        .collect();
    let r = ranges?;
    if r.len() != 20 {
        return None;
    }
    seasonal::median(r)
}
fn peer_move(payload: &Payload, at: DateTime<Utc>, req: &Request) -> Option<f64> {
    let expected = match req.symbol.as_str() {
        "SpotCrude" => "SpotBrent",
        "SpotBrent" => "SpotCrude",
        _ => return None,
    };
    if payload.peer_symbol.as_deref() != Some(expected) {
        return None;
    }
    // Exactly six completed M30 bars; no future quote or bar determines direction.
    seasonal::range_at(&payload.peer_m30, req.from, 180)?;
    let first = payload
        .peer_m30
        .iter()
        .find(|b| b.ts_ms == req.from.timestamp_millis())?;
    let last = payload
        .peer_m30
        .iter()
        .find(|b| b.ts_ms == (at - Duration::minutes(30)).timestamp_millis())?;
    Some((last.close_e5 - first.open_e5) as f64)
}
fn candidate(
    req: &Request,
    src: &Source,
    payload: &Payload,
    quotes: &[Snapshot],
    momentum: bool,
) -> Result<Value, String> {
    let mut out = seasonal::empty(req, src, "three_year_frozen_candidate_v1")?;
    let minutes = if momentum { 180 } else { 60 };
    let first = req.from + Duration::minutes(minutes);
    let reject = |reason| json!({"status":reason});
    let Some((lo, hi)) = seasonal::range_at(&payload.m30, req.from, minutes) else {
        return Ok(reject("missing_opening_range"));
    };
    let Some(s) = seasonal::spread(quotes, first) else {
        return Ok(reject("missing_prepared_spread"));
    };
    let prior = if momentum {
        None
    } else {
        prior_range(req, &payload.m30)
    };
    if !momentum {
        let Some(p) = prior else {
            return Ok(reject("missing_exact_20_prior_ranges"));
        };
        if hi - lo <= 1.25 * p {
            return Ok(reject("range_expansion_veto"));
        }
    }
    let peer = if momentum {
        peer_move(payload, first, req)
    } else {
        None
    };
    if momentum && peer.is_none() {
        return Ok(reject("missing_required_class_peer"));
    }
    let open = payload
        .m30
        .iter()
        .find(|b| b.ts_ms == req.from.timestamp_millis())
        .unwrap()
        .open_e5 as f64
        / 100000.0;
    let quantum = 10_f64.powi(-src.contract.digits);
    let r = ((hi - lo) / quantum).round().max(1.0) * quantum;
    let direction = |q: &Snapshot| {
        let x = seasonal::mid(q);
        if momentum {
            let d = x - open;
            if d * peer.unwrap() <= 0.0 {
                None
            } else {
                Some(if d > 0.0 { Dir::Buy } else { Dir::Sell })
            }
        } else if x > hi {
            Some(Dir::Buy)
        } else if x < lo {
            Some(Dir::Sell)
        } else {
            None
        }
    };
    // ORB checks each half-hour; the newly frozen 180m hypothesis decides once.
    let marks: Vec<_> = if momentum {
        vec![180]
    } else {
        (60..240).step_by(30).collect()
    };
    let mut diagnostics = vec![];
    for minute in marks {
        let at = req.from + Duration::minutes(minute);
        let mut m = seasonal::mark(at, "no_fresh_quote_in_mark_window");
        if let Some((qi, q)) = quotes
            .iter()
            .enumerate()
            .find(|(_, q)| q.fresh && q.quote.at >= at && q.quote.at < at + Duration::seconds(5))
        {
            m.decision_at = Some(q.quote.at);
            m.reason = if momentum {
                "peer_or_direction_veto"
            } else {
                "inside_opening_range"
            };
            if let Some(side) = direction(q) {
                let Some(a) = atr(&payload.m30, at) else {
                    m.reason = "missing_atr14";
                    out.marks.push(m);
                    break;
                };
                let dynamic = (2.5 * s).max(0.15 * a);
                diagnostics
                    .push(json!({"at":at,"atr14":a,"spread":s,"dynamic_threshold":dynamic,"r":r}));
                if r <= dynamic {
                    m.reason = "dynamic_range_veto";
                    out.marks.push(m);
                    break;
                }
                let c = cost_reserve(s, 0.0, 24.0, seasonal::executable(q, side))?.total;
                m.side = Some(side);
                m.r = Some(r);
                m.cost = Some(c);
                if c >= r {
                    m.reason = "cost_ge_r_session_veto";
                    out.marks.push(m);
                    break;
                }
                m.reason = "no_next_fresh_fill_quote_before_deadline";
                if let Some((fi, fq)) = quotes.iter().enumerate().skip(qi + 1).find(|(_, f)| {
                    f.fresh && f.quote.at > q.quote.at && f.quote.at < at + Duration::seconds(5)
                }) {
                    if direction(fq) != Some(side) {
                        m.reason = "direction_changed_before_simulated_fill";
                    } else {
                        let c = cost_reserve(s, 0.0, 24.0, seasonal::executable(fq, side))?.total;
                        m.cost = Some(c);
                        if c >= r {
                            m.reason = "dispatch_cost_ge_r_session_veto";
                            out.marks.push(m);
                            break;
                        }
                        out.trade = Some(seasonal::exit_trade(
                            req, src, quotes, fi, q.quote.at, side, r, s,
                        )?);
                        m.reason = "simulated_entry";
                        out.marks.push(m);
                        break;
                    }
                }
            }
        }
        out.marks.push(m);
    }
    Ok(
        json!({"study":out,"prepared_spread":s,"range":hi-lo,"prior_median":prior,"peer_move":peer,"dynamic_gate":diagnostics}),
    )
}
fn rerate(model: &mut Value) -> Result<(), String> {
    let Some(trade) = model
        .get("study")
        .and_then(|s| s.get("trade"))
        .filter(|t| !t.is_null())
    else {
        return Ok(());
    };
    let input: exits::Input =
        serde_json::from_value(trade["entry"].clone()).map_err(|e| e.to_string())?;
    let mut costs = serde_json::Map::new();
    for (arm, key) in [("trailing", "candidate"), ("fixed", "baseline")] {
        let o = &trade["exit_comparison"][key];
        let gross = o["scenario_gross_r"].as_f64();
        let weighted = o["fills"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                f["price"].as_f64().unwrap() * f["volume"].as_i64().unwrap() as f64
                    / input.volume as f64
            })
            .sum::<f64>();
        costs.insert(arm.into(),json!({"24":gross.map(|g|g-(input.fill+weighted)*24.0/10000.0/input.initial_r),"2":gross.map(|g|g-(input.fill+weighted)*2.0/10000.0/input.initial_r),"3.74":gross.map(|g|g-(input.fill+weighted)*3.74/10000.0/input.initial_r)}));
    }
    model["sensitivities"] = Value::Object(costs);
    Ok(())
}
pub fn study(req: &Request, payload: &Payload) -> Result<Value, String> {
    if !matches!(
        req.symbol.as_str(),
        "SpotCrude" | "SpotBrent" | "US500" | "XAUUSD"
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
        return Err("three-year scope/cost/source refused".into());
    }
    let native =
        sessions::session_on(&req.rule(), req.from.date_naive()).ok_or("no registered day")?;
    let oil = matches!(req.symbol.as_str(), "SpotCrude" | "SpotBrent");
    let expected = if oil {
        native.open + Duration::hours(6)
    } else {
        native.open
    };
    if req.from != expected
        || payload.source_from != native.open
        || req.to != req.from + Duration::hours(4)
        || req.to > native.close
        || req.to + Duration::seconds(5) > crate::seal::seal_instant()
        || req.from.year() < 2023
        || seasonal::held(req.from)
    {
        return Err("three-year window/seal refused".into());
    }
    let mut native_req = req.clone();
    native_req.from = native.open;
    // Native validation is retained at HTTP boundaries. Offline index/gold scope is explicit.
    let c = &payload.quotes.contract;
    let volume = req.tick_replay.as_ref().unwrap().volume;
    if !(0..=5).contains(&c.digits)
        || c.min_volume <= 0
        || c.step_volume <= 0
        || volume < c.min_volume
        || volume > 1_000_000_000
        || volume % c.step_volume != 0
    {
        return Err("invalid contract".into());
    }
    validate_bars(&payload.m30, native.open - Duration::days(45), req.to)?;
    validate_bars(
        &payload.peer_m30,
        req.from,
        req.from + Duration::minutes(180),
    )?;
    let (all, quality) = s3_ticks::snapshots(&native_req, &payload.quotes)?;
    let src = &payload.quotes;
    let first = native.open + Duration::minutes(30);
    let mut models = serde_json::Map::new();
    if let (Some(s), Some(open)) = (
        seasonal::spread(&all, first),
        payload
            .m30
            .iter()
            .find(|b| b.ts_ms == native.open.timestamp_millis() && b.volume > 0),
    ) {
        native_req.spread = s;
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
        models.insert("production_s3".into(),json!({"study":s3_ticks::study_validated(&native_req,&source,true)?,"prepared_spread":s,"opening_volume":open.volume,"scope":"native session open through observation boundary; reconstructed preparation; no peer filter added"}));
    } else {
        models.insert(
            "production_s3".into(),
            json!({"status":"missing_opening_volume_or_prepared_spread"}),
        );
    }
    let window_src = Source {
        bars: vec![],
        bids: src
            .bids
            .iter()
            .filter(|t| t.ts_ms >= req.from.timestamp_millis())
            .cloned()
            .collect(),
        asks: src
            .asks
            .iter()
            .filter(|t| t.ts_ms >= req.from.timestamp_millis())
            .cloned()
            .collect(),
        contract: c.clone(),
    };
    let (quotes, window_quality) = s3_ticks::snapshots(req, &window_src)?;
    models.insert(
        "orb_60m".into(),
        candidate(req, &window_src, payload, &quotes, false)?,
    );
    models.insert(
        "macro_180m".into(),
        candidate(req, &window_src, payload, &quotes, true)?,
    );
    models.insert(
        "always_long_control".into(),
        seasonal::control(req, &window_src, &payload.m30, &quotes, true)?,
    );
    models.insert(
        "own_momentum_fixed_horizon".into(),
        seasonal::control(req, &window_src, &payload.m30, &quotes, false)?,
    );
    for m in models.values_mut() {
        rerate(m)?;
    }
    Ok(
        json!({"models":models,"quality":quality,"window_quality":window_quality,"financial_pass":false,"validation_opened":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::super::opening_range::EntryRule;
    use super::super::s3_ticks::{Contract, Options, Tick};
    use super::*;
    fn fixture() -> (Request, Payload) {
        let at: DateTime<Utc> = "2023-07-24T13:00:00Z".parse().unwrap();
        let req = Request {
            account: 49236051,
            symbol: "SpotCrude".into(),
            from: at,
            to: at + Duration::hours(4),
            spread: 0.0,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 100,
                entry_rule: EntryRule::CurrentS3,
            }),
        };
        let source_from = at - Duration::hours(6);
        let mut bars = vec![];
        for i in 0..20 {
            bars.push(M30 {
                ts_ms: (source_from + Duration::minutes(i * 30)).timestamp_millis(),
                open_e5: 10000000,
                high_e5: 10400000,
                low_e5: 9900000,
                close_e5: if i == 17 { 10400000 } else { 10000000 },
                volume: 10,
            });
        }
        let peers = (0..6)
            .map(|i| M30 {
                ts_ms: (at + Duration::minutes(i * 30)).timestamp_millis(),
                open_e5: 10000000,
                high_e5: 10400000,
                low_e5: 9900000,
                close_e5: if i == 5 { 10400000 } else { 10000000 },
                volume: 10,
            })
            .collect();
        let mut quotes = Source {
            bars: vec![],
            bids: vec![],
            asks: vec![],
            contract: Contract {
                digits: 3,
                min_volume: 100,
                step_volume: 100,
            },
        };
        for (at, mid) in [
            (source_from + Duration::seconds(1), 100.0),
            (at + Duration::seconds(1), 100.0),
            (at + Duration::hours(3), 105.0),
            (at + Duration::hours(3) + Duration::seconds(1), 105.0),
            (at + Duration::hours(4), 106.0),
        ] {
            quotes.bids.push(Tick {
                ts_ms: at.timestamp_millis(),
                price_e5: ((mid - 0.01) * 100000.0) as i64,
            });
            quotes.asks.push(Tick {
                ts_ms: at.timestamp_millis(),
                price_e5: ((mid + 0.01) * 100000.0) as i64,
            });
        }
        (
            req,
            Payload {
                quotes,
                m30: bars,
                source_from,
                peer_symbol: Some("SpotBrent".into()),
                peer_m30: peers,
            },
        )
    }
    #[test]
    fn macro_uses_causal_peer_next_quote_native_exits_and_fixed_fill_costs() {
        let (req, mut p) = fixture();
        let a = study(&req, &p).unwrap();
        let model = &a["models"]["macro_180m"];
        let trade = &model["study"]["trade"];
        assert!(!trade.is_null());
        assert_eq!(trade["entry"]["entered_at"], "2023-07-24T16:00:01Z");
        assert_eq!(trade["exit_comparison"]["candidate"]["remaining_volume"], 0);
        assert!(
            model["sensitivities"]["trailing"]["2"].as_f64().unwrap()
                > model["sensitivities"]["trailing"]["24"].as_f64().unwrap()
        );
        // Own bar completing after the decision cannot alter that entry or its exits.
        for b in p
            .m30
            .iter_mut()
            .filter(|b| b.ts_ms >= (req.from + Duration::hours(3)).timestamp_millis())
        {
            b.high_e5 = 90000000;
            b.close_e5 = 90000000;
        }
        assert_eq!(
            study(&req, &p).unwrap()["models"]["macro_180m"]["study"]["trade"],
            *trade
        );
        p.peer_m30.last_mut().unwrap().close_e5 = 9900000;
        assert!(study(&req, &p).unwrap()["models"]["macro_180m"]["study"]["trade"].is_null());
    }
    #[test]
    fn empty_peers_and_missing_history_never_manufacture_trades() {
        let (req, mut p) = fixture();
        p.peer_symbol = None;
        p.peer_m30.clear();
        let a = study(&req, &p).unwrap();
        assert_eq!(
            a["models"]["macro_180m"]["status"],
            "missing_required_class_peer"
        );
        assert_eq!(
            a["models"]["orb_60m"]["status"],
            "missing_exact_20_prior_ranges"
        );
        assert_eq!(a["financial_pass"], false);
    }
    #[test]
    fn sealed_dates_window_and_http_scope_stay_closed() {
        let (mut req, mut p) = fixture();
        req.from = "2025-08-20T13:00:00Z".parse().unwrap();
        req.to = req.from + Duration::hours(4);
        p.source_from = req.from - Duration::hours(6);
        assert!(study(&req, &p).is_err());
        let (mut req, mut p) = fixture();
        p.source_from = req.from;
        assert!(study(&req, &p).is_err());
        req.symbol = "XAUUSD".into();
        assert!(req.validate(Utc::now()).is_err());
    }
}
