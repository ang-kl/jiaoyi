//! Two frozen causal first-hour hypotheses, separated by asset class in scoring.
//! Research only: no evidence of a live edge, no account or order authority.
use super::{
    exit_replay as exits,
    opening_range::EntryRule,
    s3_signal::Request,
    s3_ticks::{self, Mark, Quality, Snapshot, Source, Study, Trade},
};
use crate::{
    strategy3::{
        cost_reserve, sessions,
        targets::{self, Dir},
    },
    universe::AssetClass,
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

pub const SYMBOLS: [&str; 20] = [
    "US500",
    "NAS100",
    "US30",
    "GER40",
    "UK100",
    "JPN225",
    "AUS200",
    "EURUSD",
    "GBPUSD",
    "USDJPY",
    "AUDUSD",
    "USDCAD",
    "NZDUSD",
    "XAUUSD",
    "XAGUSD",
    "NatGas",
    "SpotCrude",
    "SpotBrent",
    "BTCUSD",
    "ETHUSD",
];
pub fn asset_class(symbol: &str) -> Option<AssetClass> {
    Some(match symbol {
        "US500" | "NAS100" | "US30" | "GER40" | "UK100" | "JPN225" | "AUS200" => {
            AssetClass::EquityIndex
        }
        "EURUSD" | "GBPUSD" | "USDJPY" | "AUDUSD" | "USDCAD" | "NZDUSD" => AssetClass::Fx,
        "XAUUSD" | "XAGUSD" => AssetClass::Metal,
        "NatGas" | "SpotCrude" | "SpotBrent" => AssetClass::Energy,
        "BTCUSD" | "ETHUSD" => AssetClass::Crypto,
        _ => return None,
    })
}
pub fn validate(req: &Request) -> Result<(), String> {
    let class = asset_class(&req.symbol).ok_or("unsupported original-20 symbol")?;
    let rule = sessions::rule_for(&req.symbol, class).ok_or("session undefined")?;
    let session = sessions::session_at(&rule, req.from).ok_or("not a registered session")?;
    let opts = req.tick_replay.as_ref().ok_or("tick options missing")?;
    if req.from != session.open
        || req.to != req.from + Duration::hours(4)
        || req.to > session.close
        || !session.trading_at(req.from + Duration::hours(1))
        || !session.trading_at(req.to - Duration::milliseconds(1))
    {
        return Err("class research requires four hours from registered local session open, with entry and exit outside lunch".into());
    }
    if !(1..=1_000_000_000).contains(&opts.volume)
        || req.slippage_bps_per_side != 24.0
        || req.spread != 0.0
    {
        return Err("class research requires bounded fixed volume, causal sampled spread (spread=0), and unchanged 24bp/side reserve".into());
    }
    Ok(())
}
#[derive(Debug, Serialize)]
pub struct Features {
    pub asset_class: AssetClass,
    pub decision_at: DateTime<Utc>,
    pub sampled_points: usize,
    pub first_hour_return_bps: f64,
    pub path_efficiency: f64,
    pub observed_mid_range: f64,
    pub sampled_spread_median: f64,
}
fn mid(q: &Snapshot) -> f64 {
    (q.quote.bid + q.quote.ask) / 2.0
}
fn price(q: &Snapshot, side: Dir) -> f64 {
    if side == Dir::Buy {
        q.quote.ask
    } else {
        q.quote.bid
    }
}

pub(super) fn study(
    req: &Request,
    src: &Source,
    quotes: &[Snapshot],
    quality: Quality,
) -> Result<Study, String> {
    let opts = req.tick_replay.as_ref().ok_or("options missing")?;
    let at = req.from + Duration::hours(1);
    let mut out = Study { mode: "class_first_hour_prediction_v1", quality, marks: vec![], trade: None,
        fx_diagnostic: None, prediction_features: None, notes: vec![
        "Research hypotheses only: first-hour directional continuation versus reversal, scored separately by symbol/class. Neither is established by external literature for this CFD/horizon.",
        "Thirteen equally scheduled five-minute fresh midpoint samples through hour one; direction is first-to-last change, efficiency=absolute change/sum absolute changes. Fixed efficiency >=0.5, no fitted coefficients.",
        "Observed fresh first-hour midpoint range fixes original R, rounded to contract grid. No later price enters features, cost admission or direction selection. Current contract is not certified historical metadata.",
        "One decision at hour one; next strictly later fresh quote within its five-second window is a fill proxy, not a live latency guarantee. Missing samples/fill stay missing. Direction is rechecked at simulated fill.",
        "Causal median of the thirteen sampled spreads, explicit commission scenario and unchanged 24bp/side reserve. Observed bid/ask already charges spread in net results; do not subtract it twice.",
        "Existing Q=1.38 target planner, minimum native volume, fixed-stop and original-R trailing exit comparisons. Mandatory close at hour four, with at most five seconds tail. A missing closing quote censors the position.",
        "No account Kelly/margin, dividend/funding, historical event expectations, order rejection or liquidity reconstruction. Qualified economic/account result remains unverified, even if scenario win rate exceeds 50%.",
    ]};
    let mut mark = Mark {
        at,
        reason: "missing_first_hour_samples",
        decision_at: None,
        side: None,
        r: None,
        cost: None,
    };
    let samples: Vec<_> = (0..=60)
        .step_by(5)
        .filter_map(|minute| {
            let time = req.from + Duration::minutes(minute);
            quotes.iter().enumerate().find(|(_, q)| {
                q.fresh && q.quote.at >= time && q.quote.at < time + Duration::seconds(5)
            })
        })
        .collect();
    if samples.len() != 13 {
        out.marks.push(mark);
        return Ok(out);
    }
    let (idx, q) = samples[12];
    mark.decision_at = Some(q.quote.at);
    let change = mid(q) - mid(samples[0].1);
    let path: f64 = samples
        .windows(2)
        .map(|w| (mid(w[1].1) - mid(w[0].1)).abs())
        .sum();
    let efficiency = if path > 0.0 { change.abs() / path } else { 0.0 };
    let mut spreads: Vec<_> = samples
        .iter()
        .map(|(_, s)| s.quote.ask - s.quote.bid)
        .collect();
    spreads.sort_by(f64::total_cmp);
    let prefix: Vec<_> = quotes
        .iter()
        .take(idx + 1)
        .filter(|x| x.fresh)
        .map(mid)
        .collect();
    let range = prefix.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - prefix.iter().copied().fold(f64::INFINITY, f64::min);
    out.prediction_features = Some(Features {
        asset_class: asset_class(&req.symbol).unwrap(),
        decision_at: q.quote.at,
        sampled_points: 13,
        first_hour_return_bps: change / mid(samples[0].1) * 10000.0,
        path_efficiency: efficiency,
        observed_mid_range: range,
        sampled_spread_median: spreads[6],
    });
    mark.reason = "first_hour_not_directionally_coherent";
    if efficiency < 0.5 || change == 0.0 || range <= 0.0 {
        out.marks.push(mark);
        return Ok(out);
    }
    let follow = opts.entry_rule == EntryRule::ClassMomentum1h;
    let side = if (change > 0.0) == follow {
        Dir::Buy
    } else {
        Dir::Sell
    };
    let quantum = 10_f64.powi(-src.contract.digits);
    let r = (range / quantum).round().max(1.0) * quantum;
    let c = cost_reserve(spreads[6], req.commission_round_trip, 24.0, price(q, side))?.total;
    mark.side = Some(side);
    mark.r = Some(r);
    mark.cost = Some(c);
    mark.reason = "candidate_cost_ge_r";
    if c >= r {
        out.marks.push(mark);
        return Ok(out);
    }
    mark.reason = "no_next_fresh_fill_quote_before_deadline";
    let Some((fi, fq)) = quotes.iter().enumerate().skip(idx + 1).find(|(_, f)| {
        f.fresh && f.quote.at > q.quote.at && f.quote.at < at + Duration::seconds(5)
    }) else {
        out.marks.push(mark);
        return Ok(out);
    };
    if (mid(fq) - mid(samples[0].1)) * change <= 0.0 {
        mark.reason = "candidate_changed_before_fill";
        out.marks.push(mark);
        return Ok(out);
    }
    let fill = price(fq, side);
    // Recheck cost at the executable proxy, keeping the original R fixed.
    let c = cost_reserve(spreads[6], req.commission_round_trip, 24.0, fill)?.total;
    mark.cost = Some(c);
    if c >= r {
        mark.reason = "candidate_cost_ge_r_at_fill";
        out.marks.push(mark);
        return Ok(out);
    }
    let sign = if side == Dir::Buy { 1.0 } else { -1.0 };
    let plan = targets::plan(&targets::Input {
        side,
        entry: fill,
        original_stop: fill - sign * r,
        required_rr: targets::REQUIRED_RR,
        cost_reserve: c,
        digits: src.contract.digits,
        volume: opts.volume,
        min_volume: src.contract.min_volume,
        step_volume: src.contract.step_volume,
    })
    .map_err(str::to_string)?;
    let input = exits::Input {
        entered_at: fq.quote.at,
        fill,
        side: if side == Dir::Buy {
            exits::Side::Long
        } else {
            exits::Side::Short
        },
        initial_r: r,
        volume: opts.volume,
        tp1_close_volume: plan.close_volume,
        tp1_distance: (plan.trigger - fill).abs(),
        broker_tp_distance: (plan.broker_target - fill).abs(),
        session_end: req.to,
        digits: src.contract.digits,
        cadence: exits::TrailCadence::S3ThirtySeconds,
    };
    let mut engine = exits::ExitReplay::new(input.clone()).map_err(|e| e.to_string())?;
    for f in quotes.iter().skip(fi) {
        engine
            .push(f.quote)
            .map_err(|e| format!("quote invalid: {e:?}"))?;
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
    let trailing = s3_ticks::net(req, &input, &comparison.candidate);
    let fixed = s3_ticks::net(req, &input, &comparison.baseline);
    out.trade = Some(Trade {
        decision_at: q.quote.at,
        entry: input,
        exit_comparison: comparison,
        trailing_scenario_net_r: trailing,
        fixed_stop_scenario_net_r: fixed,
    });
    mark.reason = "simulated_entry";
    out.marks.push(mark);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::s3_ticks::{Contract, Options, Tick};
    use super::*;
    fn fixture() -> (Request, Source) {
        let from = "2025-08-05T00:00:00Z".parse().unwrap();
        let req = Request {
            account: 42,
            symbol: "BTCUSD".into(),
            from,
            to: from + Duration::hours(4),
            spread: 0.0,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 1,
                entry_rule: EntryRule::ClassMomentum1h,
            }),
        };
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
        for (offset, bid) in (0..=12)
            .map(|i| (i * 300000, 10_000_000 + i * 20_000))
            .chain([
                (3600001, 10_241_000),
                (7200000, 10_800_000),
                (14400000, 10_800_000),
            ])
        {
            let ts_ms = req.from.timestamp_millis() + offset;
            src.bids.push(Tick {
                ts_ms,
                price_e5: bid,
            });
            src.asks.push(Tick {
                ts_ms,
                price_e5: bid + 2000,
            });
        }
        (req, src)
    }
    #[test]
    fn all_twenty_use_local_sessions_and_preserve_mode_boundaries() {
        for s in SYMBOLS {
            let (mut req, _) = fixture();
            req.symbol = s.into();
            let rule = req.rule();
            for date in ["2025-08-05", "2025-10-20"] {
                let session = sessions::session_on(&rule, date.parse().unwrap()).unwrap();
                req.from = session.open;
                req.to = req.from + Duration::hours(4);
                assert!(req.validate(Utc::now()).is_ok(), "{s} {date}");
                req.from += Duration::minutes(30);
                assert!(req.validate(Utc::now()).is_err());
            }
        }
        let (mut req, _) = fixture();
        req.symbol = "USDCHF".into();
        assert!(req.validate(Utc::now()).is_err());
        req.symbol = "GER40".into();
        req.tick_replay.as_mut().unwrap().entry_rule = EntryRule::CurrentS3;
        assert!(req.validate(Utc::now()).is_err());
        let (mut req, _) = fixture();
        req.slippage_bps_per_side = 0.0;
        assert!(req.validate(Utc::now()).is_err());
    }
    #[test]
    fn future_changes_cannot_rewrite_prediction_and_opposite_candidate_is_explicit() {
        let (mut req, mut src) = fixture();
        let a = s3_ticks::study(&req, &src).unwrap();
        assert!(a.trade.is_some());
        for t in src
            .bids
            .iter_mut()
            .chain(src.asks.iter_mut())
            .filter(|t| t.ts_ms > req.from.timestamp_millis() + 3600001)
        {
            t.price_e5 -= 1_000_000;
        }
        let b = s3_ticks::study(&req, &src).unwrap();
        assert_eq!(
            serde_json::to_value(&a.prediction_features).unwrap(),
            serde_json::to_value(&b.prediction_features).unwrap()
        );
        assert_eq!(
            a.trade.as_ref().unwrap().entry,
            b.trade.as_ref().unwrap().entry
        );
        assert_ne!(
            a.trade.unwrap().trailing_scenario_net_r,
            b.trade.unwrap().trailing_scenario_net_r
        );
        req.tick_replay.as_mut().unwrap().entry_rule = EntryRule::ClassReversion1h;
        assert_eq!(
            s3_ticks::study(&req, &src)
                .unwrap()
                .trade
                .unwrap()
                .entry
                .side,
            exits::Side::Short
        );
    }
    #[test]
    fn missing_data_contract_cost_and_censoring_cannot_create_a_win() {
        let (req, mut src) = fixture();
        src.bids.remove(2);
        src.asks.remove(2);
        assert!(s3_ticks::study(&req, &src).unwrap().trade.is_none());
        let (req, mut src) = fixture();
        src.contract.min_volume = 2;
        assert!(s3_ticks::study(&req, &src).is_err());
        let (mut req, src) = fixture();
        req.commission_round_trip = 20.0;
        assert!(s3_ticks::study(&req, &src).unwrap().trade.is_none());
        let (req, mut src) = fixture();
        src.bids.truncate(14);
        src.asks.truncate(14);
        let t = s3_ticks::study(&req, &src).unwrap().trade.unwrap();
        assert_eq!(t.trailing_scenario_net_r, None);
        assert_eq!(t.exit_comparison.candidate.qualified_gross_r, None);
    }
}
