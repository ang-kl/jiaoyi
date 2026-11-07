//! №18,351: pure offline matched-entry direction/horizon comparison.
//! M30 is an explicitly conditional approximation, never an invented tick path.
use super::{
    exit_replay as exits, offline_repaired,
    repaired_exit::TwoStageReplay,
    s3_signal::Request,
    s3_ticks,
    seasonal::{self, M30},
    three_year::{self, Payload},
};
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};

fn sign(i: &exits::Input) -> f64 {
    if i.side == exits::Side::Long {
        1.0
    } else {
        -1.0
    }
}

fn net(i: &exits::Input, price: f64) -> Value {
    let gross = sign(i) * (price - i.fill) / i.initial_r;
    let values = [24.0, 2.0, 3.74]
        .into_iter()
        .map(|bps| {
            (
                bps.to_string(),
                json!(gross - (i.fill + price) * bps / 10000.0 / i.initial_r),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    json!({"gross_r":gross,"net_by_bps":values})
}

pub(super) fn tick_run(
    i: &exits::Input,
    quotes: &[s3_ticks::Snapshot],
    to: DateTime<Utc>,
    force: bool,
) -> Result<Value, String> {
    let mut input = i.clone();
    if force {
        input.session_end = to;
    }
    let mut engine = TwoStageReplay::new(input.clone()).map_err(|e| e.to_string())?;
    let through = if force { to + Duration::seconds(5) } else { to };
    for q in quotes
        .iter()
        .filter(|q| q.quote.at >= i.entered_at && q.quote.at <= through)
    {
        if force && q.quote.at >= to && !q.fresh {
            continue;
        }
        engine
            .push(q.quote)
            .map_err(|e| format!("tick exit: {e:?}"))?;
    }
    let outcome = engine
        .finish(
            exits::Coverage {
                censored: true,
                ..exits::Coverage::recorded_path(through)
            },
            exits::Costs::default(),
        )
        .candidate;
    let mut v = json!({"status":if outcome.scenario_gross_r.is_some(){"closed_tick"}else{"censored_tick"},
        "entry":input,"outcome":outcome,"metrics":null});
    if let Some(fill) = outcome
        .fills
        .first()
        .filter(|_| outcome.scenario_gross_r.is_some())
    {
        v["metrics"] = net(i, fill.price);
    }
    Ok(v)
}

fn unresolved(status: &str, bars: usize, at: DateTime<Utc>, stop: f64) -> Value {
    json!({"status":status,"bars_processed":bars,"through":at,"active_stop":stop,"metrics":null})
}

fn bar_close(i: &exits::Input, price: f64, bar: &M30, bars: usize, reason: &str) -> Value {
    json!({"status":"closed_m30_approximation","bars_processed":bars,
        "exit":{"price":price,"bar_start_ms":bar.ts_ms,"bar_end_ms":bar.ts_ms+1800000,"reason":reason},
        "metrics":net(i,price),"exact_fill_time_known":false,"financing_included":false})
}

/// No bridging absent bars, no OHLC path invention, no double charging spread.
fn continue_bars(
    i: &exits::Input,
    tick: &Value,
    bars: &[M30],
    start: DateTime<Utc>,
    spread: f64,
) -> Result<Value, String> {
    if tick["status"] == "closed_tick" {
        return Ok(tick.clone());
    }
    let o = &tick["outcome"];
    let s = sign(i);
    let target = i.fill + s * i.broker_tp_distance;
    let mut stop = o["active_stop"].as_f64().ok_or("missing active stop")?;
    let mut pending = o["trail_proposals"].as_u64().unwrap_or(0)
        > o["trail_applied"].as_u64().unwrap_or(0)
            + o["trail_rejected_on_next_quote"].as_u64().unwrap_or(0);
    let deadline = i.entered_at + Duration::hours(72);
    let mut next = start;
    let mut n = 0;
    for b in bars {
        let at = DateTime::from_timestamp_millis(b.ts_ms).ok_or("bad future bar time")?;
        if at < start {
            return Err("future bars overlap tick path".into());
        }
        if at != next {
            return Ok(unresolved("censored_cache_gap", n, next, stop));
        }
        if at + Duration::minutes(30) > deadline {
            return Ok(unresolved("censored_partial_deadline_bar", n, next, stop));
        }
        n += 1;
        // cTrader trendbar bid interpretation; ask is NOT recorded here.
        // Frozen entry spread supplies a disclosed short-exit proxy only.
        let offset = if s < 0.0 { spread } else { 0.0 };
        let open = b.open_e5 as f64 / 100000.0 + offset;
        let high = b.high_e5 as f64 / 100000.0 + offset;
        let low = b.low_e5 as f64 / 100000.0 + offset;
        let close = b.close_e5 as f64 / 100000.0 + offset;
        if s * (stop - open) >= 0.0 {
            return Ok(bar_close(i, open, b, n, "gap_or_open_stop"));
        }
        if s * (open - target) >= 0.0 {
            return Ok(bar_close(i, target, b, n, "open_target"));
        }
        if pending {
            pending = false;
            if s * (open - i.fill) > 0.0 {
                stop = i.fill;
            }
        }
        let best = if s > 0.0 { high } else { low };
        let worst = if s > 0.0 { low } else { high };
        let hit_stop = s * (stop - worst) >= 0.0;
        let hit_target = s * (best - target) >= 0.0;
        let trigger = s * (best - i.fill) >= i.initial_r;
        let can_return = s * (i.fill - worst) >= 0.0;
        // Even stop-only versus +1R then BE can depend on intrabar sequence.
        if (hit_stop && hit_target) || (stop != i.fill && trigger && can_return) {
            let mut v = unresolved("ambiguous_m30_path", n, at, stop);
            v["ambiguous_bar"] = json!(b);
            v["reason"] =
                json!("OHLC cannot order old stop, +1R amend, entry-price stop and target");
            return Ok(v);
        }
        if hit_stop {
            return Ok(bar_close(i, stop, b, n, "stop_touch"));
        }
        if hit_target {
            return Ok(bar_close(i, target, b, n, "target_touch"));
        }
        if stop != i.fill && trigger {
            // A different favourable close proves a later quote after the extreme.
            // If close equals extreme, applied versus pending cannot be known.
            if close != best && s * (close - i.fill) > 0.0 {
                stop = i.fill;
            } else {
                return Ok(unresolved("ambiguous_m30_amend_timing", n, at, stop));
            }
        }
        next = at + Duration::minutes(30);
        if next == deadline {
            return Ok(bar_close(i, close, b, n, "72h_end_close"));
        }
    }
    Ok(unresolved("censored_end_of_cache", n, next, stop))
}

fn validate_future(req: &Request, bars: &[M30]) -> Result<(), String> {
    let mut previous = None;
    for b in bars {
        let at = DateTime::from_timestamp_millis(b.ts_ms).ok_or("future timestamp")?;
        if b.ts_ms % 1800000 != 0
            || previous.is_some_and(|p| p >= b.ts_ms)
            || at < req.to
            || at > req.to + Duration::hours(72)
            || seasonal::held(at)
            || at + Duration::minutes(30) > "2025-10-24T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
            || b.low_e5 <= 0
            || b.low_e5 > b.open_e5
            || b.open_e5 > b.high_e5
            || b.low_e5 > b.close_e5
            || b.close_e5 > b.high_e5
            || b.volume < 0
        {
            return Err("invalid/protected future bar".into());
        }
        previous = Some(b.ts_ms);
    }
    Ok(())
}

pub fn study(req: &Request, payload: &Payload, future: &[M30]) -> Result<Value, String> {
    validate_future(req, future)?;
    let base = three_year::study(req, payload)?;
    let legacy = offline_repaired::two_stage(req, payload, &base["models"]["production_s3"])?;
    let Some(trade) = legacy
        .get("study")
        .and_then(|s| s.get("trade"))
        .filter(|t| !t.is_null())
    else {
        return Ok(json!({"legacy":legacy,"models":{},"status":"no_admitted_entry"}));
    };
    let original: exits::Input =
        serde_json::from_value(trade["entry"].clone()).map_err(|e| e.to_string())?;
    if original.initial_r / original.fill * 10000.0 < 48.0 - 1e-8 {
        return Err("48bp floor failed".into());
    }
    let mut qr = req.clone();
    qr.from = payload.source_from;
    let (quotes, quality) = s3_ticks::snapshots(&qr, &payload.quotes)?;
    let entry_quote = quotes
        .iter()
        .find(|q| q.quote.at == original.entered_at && q.fresh)
        .ok_or("entry quote unavailable")?;
    let spread = entry_quote.quote.ask - entry_quote.quote.bid;
    let mut models = serde_json::Map::new();
    for fade in [false, true] {
        let mut input = original.clone();
        input.session_end = input.entered_at + Duration::hours(72);
        if fade {
            input.side = if original.side == exits::Side::Long {
                exits::Side::Short
            } else {
                exits::Side::Long
            };
            input.fill = if input.side == exits::Side::Long {
                entry_quote.quote.ask
            } else {
                entry_quote.quote.bid
            };
        }
        let tick = tick_run(&input, &quotes, req.to, false)?;
        let mtm = tick_run(&input, &quotes, req.to, true)?;
        let mut extended = continue_bars(&input, &tick, future, req.to, spread)?;
        extended["entry"] = json!(input);
        extended["tick_boundary_status"] = tick["status"].clone();
        extended["spread_proxy"] = json!(spread);
        models.insert(
            if fade {
                "option_a_fade_4h"
            } else {
                "s3_breakout_4h"
            }
            .into(),
            tick,
        );
        models.insert(
            if fade {
                "option_a_fade_4h_mtm"
            } else {
                "s3_breakout_4h_mtm"
            }
            .into(),
            mtm,
        );
        models.insert(
            if fade {
                "option_ab_fade_72h"
            } else {
                "option_b_breakout_72h"
            }
            .into(),
            extended,
        );
    }
    Ok(
        json!({"status":"admitted","legacy":legacy,"models":models,"quality":quality,
        "entry_spread":spread,"new_broker_requests":0,"financial_pass":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(side: exits::Side) -> exits::Input {
        exits::Input {
            entered_at: "2024-01-22T13:00:00Z".parse().unwrap(),
            fill: 100.0,
            side,
            initial_r: 1.0,
            volume: 100,
            tp1_close_volume: 100,
            tp1_distance: 1.38,
            broker_tp_distance: 1.38,
            session_end: "2024-01-25T13:00:00Z".parse().unwrap(),
            digits: 2,
            cadence: exits::TrailCadence::S3ThirtySeconds,
        }
    }
    fn bar(at: DateTime<Utc>, o: f64, h: f64, l: f64, c: f64) -> M30 {
        M30 {
            ts_ms: at.timestamp_millis(),
            open_e5: (o * 100000.0) as i64,
            high_e5: (h * 100000.0) as i64,
            low_e5: (l * 100000.0) as i64,
            close_e5: (c * 100000.0) as i64,
            volume: 1,
        }
    }
    fn empty_tick(i: &exits::Input, to: DateTime<Utc>) -> Value {
        tick_run(i, &[], to, false).unwrap()
    }
    #[test]
    fn continuation_rejects_gaps_and_ambiguous_ohlc_and_never_fabricates_close() {
        let i = input(exits::Side::Long);
        let at = i.entered_at + Duration::hours(4);
        let tick = empty_tick(&i, at);
        assert_eq!(
            continue_bars(&i, &tick, &[], at, 0.02).unwrap()["status"],
            "censored_end_of_cache"
        );
        assert_eq!(
            continue_bars(
                &i,
                &tick,
                &[bar(at + Duration::minutes(30), 100., 102., 98., 100.)],
                at,
                0.02
            )
            .unwrap()["status"],
            "censored_cache_gap"
        );
        assert_eq!(
            continue_bars(&i, &tick, &[bar(at, 100., 102., 98., 100.)], at, 0.02).unwrap()
                ["status"],
            "ambiguous_m30_path"
        );
        let v = continue_bars(&i, &tick, &[bar(at, 98., 98.5, 97., 98.)], at, 0.02).unwrap();
        assert_eq!(v["metrics"]["gross_r"], -2.0);
        assert!(v["metrics"]["net_by_bps"]["24"].as_f64().unwrap() < -2.0);
    }
    #[test]
    fn long_short_targets_spread_and_72h_deadline_are_explicit() {
        for side in [exits::Side::Long, exits::Side::Short] {
            let i = input(side);
            let at = i.entered_at + Duration::hours(4);
            let tick = empty_tick(&i, at);
            let b = if side == exits::Side::Long {
                bar(at, 100.5, 101.5, 100.4, 101.4)
            } else {
                bar(at, 99.5, 99.6, 98.4, 98.5)
            };
            let v = continue_bars(&i, &tick, &[b], at, 0.02).unwrap();
            assert_eq!(v["status"], "closed_m30_approximation");
            assert!((v["metrics"]["gross_r"].as_f64().unwrap() - 1.38).abs() < 1e-10);
            let last = i.entered_at + Duration::hours(72) - Duration::minutes(30);
            let v = continue_bars(&i, &tick, &[bar(last, 100., 100.1, 99.9, 100.)], last, 0.02)
                .unwrap();
            assert_eq!(v["exit"]["reason"], "72h_end_close");
        }
    }
    #[test]
    fn tick_path_retains_broker_priority_and_costs_entry_price_stop() {
        let i = input(exits::Side::Long);
        let at = i.entered_at;
        let quotes =
            [(0, 100.), (1, 101.), (2, 101.1), (3, 100.)].map(|(sec, bid)| s3_ticks::Snapshot {
                quote: exits::Quote {
                    at: at + Duration::seconds(sec),
                    bid,
                    ask: bid + 0.02,
                    gap_before: false,
                },
                fresh: true,
            });
        let v = tick_run(&i, &quotes, at + Duration::hours(4), false).unwrap();
        assert_eq!(v["metrics"]["gross_r"], 0.0);
        assert_eq!(v["outcome"]["trail_applied"], 1);
        assert!((v["metrics"]["net_by_bps"]["24"].as_f64().unwrap() + 0.48).abs() < 1e-10);
        let short = input(exits::Side::Short);
        let v = tick_run(&short, &quotes, at + Duration::hours(4), false).unwrap();
        assert_eq!(v["status"], "closed_tick");
    }
}
