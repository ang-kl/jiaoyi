//! FX market-structure diagnostics. No signal, position, or economic PASS.
use super::{
    s3_signal::Request,
    s3_ticks::{Quality, Snapshot, Study},
};
use chrono::{Datelike, Duration, Timelike};
use chrono_tz::{America, Asia, Europe};
use serde::Serialize;

pub fn window(req: &Request) -> Option<&'static str> {
    for (name, tz, hour) in [
        ("Tokyo", Asia::Tokyo, 9),
        ("London", Europe::London, 8),
        ("New York", America::New_York, 8),
    ] {
        let local = req.from.with_timezone(&tz);
        if local.weekday().number_from_monday() <= 5
            && local.hour() == hour
            && local.minute() == 0
            && local.second() == 0
            && local.nanosecond() == 0
        {
            return Some(name);
        }
    }
    None
}
pub fn validate(req: &Request) -> Result<(), String> {
    if !req.is_major_fx() || window(req).is_none() || req.to - req.from != Duration::hours(4) {
        return Err("FX diagnostics require a major pair and a weekday four-hour window at 09:00 Tokyo, 08:00 London or 08:00 New York local time".into());
    }
    let volume = req
        .tick_replay
        .as_ref()
        .ok_or("tick options missing")?
        .volume;
    if volume <= 0
        || volume > 1_000_000_000
        || req.slippage_bps_per_side != 24.0
        || req.spread != 0.0
        || req.commission_round_trip != 0.0
    {
        return Err("FX diagnostic uses bounded volume, observed spread, unknown commission and the unchanged 24bp reserve; not a calibrated cost model".into());
    }
    Ok(())
}
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub window: &'static str,
    pub fresh_snapshots: usize,
    pub half_hour_coverage: [usize; 8],
    pub scheduled_samples: usize,
    pub available_samples: usize,
    pub missing_sample_offsets_minutes: Vec<i64>,
    pub observed_mid_range_bps: Option<f64>,
    pub sampled_spread_median_bps: Option<f64>,
    pub sampled_spread_p95_bps: Option<f64>,
    pub signed_move_bps: Option<f64>,
    pub five_minute_path_efficiency: Option<f64>,
    pub current_round_trip_reserve_ex_commission_bps: Option<f64>,
    pub goal_qualifying_trades: usize,
}
fn quantile(v: &mut [f64], q: f64) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    Some(v[((v.len() - 1) as f64 * q).round() as usize])
}
pub(super) fn study(req: &Request, quotes: &[Snapshot], quality: Quality) -> Study {
    let fresh: Vec<_> = quotes
        .iter()
        .filter(|q| q.fresh && q.quote.at >= req.from && q.quote.at < req.to)
        .collect();
    let mid = |q: &Snapshot| (q.quote.bid + q.quote.ask) / 2.0;
    let mut coverage = [0; 8];
    for q in &fresh {
        coverage[((q.quote.at - req.from).num_minutes() / 30) as usize] += 1;
    }
    let mut samples = Vec::new();
    let mut missing = Vec::new();
    let mut spreads = Vec::new();
    for minutes in (0..=240).step_by(5) {
        let mark = req.from + Duration::minutes(minutes);
        let q = quotes
            .iter()
            .find(|q| q.fresh && q.quote.at >= mark && q.quote.at < mark + Duration::seconds(5));
        if let Some(q) = q {
            let m = mid(q);
            samples.push(m);
            spreads.push((q.quote.ask - q.quote.bid) / m * 10000.0);
        } else {
            missing.push(minutes);
        }
    }
    let median = quantile(&mut spreads, 0.5);
    let p95 = quantile(&mut spreads, 0.95);
    let range = fresh.first().map(|first| {
        let lo = fresh.iter().map(|q| mid(q)).fold(f64::INFINITY, f64::min);
        let hi = fresh
            .iter()
            .map(|q| mid(q))
            .fold(f64::NEG_INFINITY, f64::max);
        (hi - lo) / mid(first) * 10000.0
    });
    let complete = missing.is_empty();
    let signed = complete.then(|| (samples[48] / samples[0] - 1.0) * 10000.0);
    let efficiency = if complete {
        let path: f64 = samples.windows(2).map(|p| (p[1] - p[0]).abs()).sum();
        (path > 0.0).then(|| (samples[48] - samples[0]).abs() / path)
    } else {
        None
    };
    Study {
        mode: "fx_session_diagnostic_v1", quality, marks: vec![], trade: None,
        fx_diagnostic: Some(Diagnostic {
            window: window(req).unwrap(), fresh_snapshots: fresh.len(), half_hour_coverage: coverage,
            scheduled_samples: 49, available_samples: samples.len(), missing_sample_offsets_minutes: missing,
            observed_mid_range_bps: range, sampled_spread_median_bps: median, sampled_spread_p95_bps: p95,
            signed_move_bps: signed, five_minute_path_efficiency: efficiency,
            current_round_trip_reserve_ex_commission_bps: median.map(|s| s + 48.0), goal_qualifying_trades: 0,
        }),
        notes: vec![
            "Read-only market-structure observations: no entry setup, stop, target, trade, win rate or financial PASS.",
            "Local-clock four-hour observation windows are research conventions, not centralised FX exchange opens; DST is respected.",
            "Fresh grouped broker bid/ask snapshots; observed range is hindsight and cannot be treated as capturable profit. Quote history is not consolidated FX volume or signed order flow.",
            "Spread quantiles use equally scheduled five-minute samples within five seconds; not tick-weighted. Missing samples invalidate movement/path efficiency, not zero-fill them. Counts do not certify quote completeness.",
            "48bp round-trip reserve plus observed median spread excludes unknown commission; it describes the current assumption, not measured costs. No lower reserve or cost-free trading is implied.",
            "Historical contract, slippage, economic-event and news-vintage coverage remain unverified; no event-conditioned edge can be scored from this output alone.",
        ],
    }
}
#[cfg(test)]
mod tests {
    use super::super::{
        opening_range::EntryRule,
        s3_ticks::{self, Contract, Options, Source, Tick},
    };
    use super::*;
    fn req(from: &str) -> Request {
        let from = from.parse().unwrap();
        Request {
            account: 1,
            symbol: "EURUSD".into(),
            from,
            to: from + Duration::hours(4),
            spread: 0.0,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 100000,
                entry_rule: EntryRule::FxSessionDiagnostic,
            }),
        }
    }
    #[test]
    fn local_windows_dst_and_scope_are_enforced() {
        for (at, name) in [
            ("2025-09-08T00:00:00Z", "Tokyo"),
            ("2025-09-08T07:00:00Z", "London"),
            ("2025-01-06T08:00:00Z", "London"),
            ("2025-09-08T12:00:00Z", "New York"),
            ("2025-01-06T13:00:00Z", "New York"),
        ] {
            let r = req(at);
            assert_eq!(window(&r), Some(name));
            assert!(r.validate(r.to).is_ok());
        }
        for at in [
            "2025-09-08T08:00:00Z",
            "2025-09-07T07:00:00Z",
            "2025-10-24T00:00:00Z",
        ] {
            let r = req(at);
            assert!(r.validate(r.to).is_err());
        }
        let mut r = req("2025-09-08T07:00:00Z");
        r.tick_replay.as_mut().unwrap().entry_rule = EntryRule::OpeningTickRangeBreakout2h;
        assert!(r.validate(r.to).is_err());
        r.tick_replay.as_mut().unwrap().entry_rule = EntryRule::FxSessionDiagnostic;
        r.symbol = "EURCNH".into();
        assert!(r.validate(r.to).is_err());
        r.symbol = "BTCUSD".into();
        assert!(r.validate(r.to).is_err());
    }
    #[test]
    fn no_trade_missing_samples_and_jpy_price_units() {
        for (symbol, digits, price, spread) in
            [("EURUSD", 5, 110000, 10), ("USDJPY", 3, 15000000, 1000)]
        {
            let mut r = req("2025-09-08T07:00:00Z");
            r.symbol = symbol.into();
            let ticks = |offset| {
                (0..=48)
                    .map(|i| Tick {
                        ts_ms: (r.from + Duration::minutes(i * 5)).timestamp_millis(),
                        price_e5: price + i * spread + offset,
                    })
                    .collect()
            };
            let mut src = Source {
                bars: vec![],
                bids: ticks(0),
                asks: ticks(spread),
                contract: Contract {
                    digits,
                    min_volume: 100000,
                    step_volume: 100000,
                },
            };
            let out = s3_ticks::study(&r, &src).unwrap();
            assert!(out.trade.is_none());
            assert!(out.marks.is_empty());
            let d = out.fx_diagnostic.unwrap();
            assert_eq!(d.available_samples, 49);
            assert_eq!(d.five_minute_path_efficiency, Some(1.0));
            assert!(d.sampled_spread_median_bps.unwrap() > 0.0);
            src.asks.remove(12);
            let d = s3_ticks::study(&r, &src).unwrap().fx_diagnostic.unwrap();
            assert_eq!(d.missing_sample_offsets_minutes, vec![60]);
            assert_eq!(d.signed_move_bps, None);
            assert_eq!(d.five_minute_path_efficiency, None);
        }
    }
}
