//! Frozen development candidates, NOT live S3 amendments. Two-hour sampled-mid
//! range followed by a two-hour trading window. Not an OHLC high/low range.
use super::{
    exit_replay as exits,
    s3_signal::Request,
    s3_ticks::{self, Mark, Quality, Snapshot, Source, Study, Trade},
};
use crate::strategy3::{
    cost_reserve,
    targets::{self, Dir},
};
use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryRule {
    #[default]
    CurrentS3,
    /// Read-only market structure observations, with no entry/exit candidate.
    FxSessionDiagnostic,
    ClassMomentum1h,
    ClassReversion1h,
    OpeningRangeBreakout2h,
    OpeningRangeFailedBreakout2h,
    OpeningTickRangeBreakout2h,
    OpeningTickRangeFailedBreakout2h,
}
impl EntryRule {
    pub fn class_prediction(self) -> bool {
        matches!(self, Self::ClassMomentum1h | Self::ClassReversion1h)
    }
    pub fn full_ticks(self) -> bool {
        matches!(
            self,
            Self::OpeningTickRangeBreakout2h | Self::OpeningTickRangeFailedBreakout2h
        )
    }
    fn failed_breakout(self) -> bool {
        matches!(
            self,
            Self::OpeningRangeFailedBreakout2h | Self::OpeningTickRangeFailedBreakout2h
        )
    }
    pub fn is_current(&self) -> bool {
        *self == Self::CurrentS3
    }
}
fn mid(q: &Snapshot) -> f64 {
    (q.quote.bid + q.quote.ask) / 2.0
}
fn entry(q: &Snapshot, side: Dir) -> f64 {
    if side == Dir::Buy {
        q.quote.ask
    } else {
        q.quote.bid
    }
}
fn sign(side: Dir) -> f64 {
    if side == Dir::Buy {
        1.0
    } else {
        -1.0
    }
}
fn inside(p: f64, lo: f64, hi: f64) -> bool {
    p >= lo && p <= hi
}

pub(super) fn study(
    req: &Request,
    src: &Source,
    quotes: &[Snapshot],
    quality: Quality,
) -> Result<Study, String> {
    let opts = req.tick_replay.as_ref().ok_or("options missing")?;
    let mut out=Study{mode:"opening_sampled_range_research_v1",quality,marks:vec![],trade:None,fx_diagnostic:None,prediction_features:None,notes:vec![
        "Experimental entry rules only; current S3/live settings unchanged. Not an economic or account-goal PASS.",
        "First fresh midpoint at minutes 30/60/90/120 defines the fixed sampled range. This is not an intrabar high/low range. Missing any preparation mark excludes the episode.",
        "At minutes 150/180/210, breakout follows the range break; failed-breakout enters opposite only after an outside mark returns inside. A missing mark resets the failed excursion.",
        "Breakout stop is the opposite sampled-range boundary. Failed-breakout stop is the most extreme previously observed outside-mark midpoint. R is rounded to the broker grid.",
        "Cost C<R is checked for each eligible candidate mark; failing one mark does not veto the whole session. One simulated entry per episode. This is a new candidate, not promotion of prior veto-only experiments.",
        "Q=1.38, partial/whole target policy, fixed native volume and 24bp/side reserve retained. Spread/commission inputs remain explicit uncalibrated scenarios.",
        "Entry uses the next strictly later fresh quote before the five-second mark deadline. The trigger condition is rechecked. No measured latency/liquidity/rejection simulation.",
        "Close at the first recorded quote at/after hour four, allowing five seconds of exit coverage. No quote means censored, not zero profit. Original-R trail: 30-second proposals, frozen in the episode's final half hour.",
        "Separate bid/ask streams are grouped by millisecond. Continuity, historical contract/account sizing and execution costs remain unproven. Scenario results do not qualify for the live 20-position goal.",
    ]};
    let quote_at = |minute| {
        let mark = req.from + Duration::minutes(minute);
        quotes.iter().enumerate().find(|(_, q)| {
            q.fresh && q.quote.at >= mark && q.quote.at < mark + Duration::seconds(5)
        })
    };
    let mut prep = vec![];
    let ready = if opts.entry_rule.full_ticks() {
        let mut covered = [false; 4];
        for q in quotes.iter().filter(|q| {
            q.fresh && q.quote.at >= req.from && q.quote.at < req.from + Duration::minutes(120)
        }) {
            let block = (q.quote.at - req.from).num_minutes() as usize / 30;
            covered[block] = true;
            prep.push(mid(q));
        }
        out.notes[1]="Full observed fresh tick-midpoint range over the first120 minutes; at least one fresh observation in every30-minute block. This remains an observed-path range, not a completeness certificate.";
        covered.into_iter().all(|x| x)
    } else {
        for minute in [30, 60, 90, 120] {
            if let Some((_, q)) = quote_at(minute) {
                prep.push(mid(q));
            }
        }
        prep.len() == 4
    };
    let lo = prep.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = prep.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut excursion: Option<(Dir, f64)> = None;
    for minute in [150, 180, 210] {
        let at = req.from + Duration::minutes(minute);
        let mut mark = Mark {
            at,
            reason: "no_fresh_quote_in_mark_window",
            decision_at: None,
            side: None,
            r: None,
            cost: None,
        };
        if !ready || hi <= lo {
            mark.reason = "missing_or_flat_preparation_range";
        } else if out.trade.is_some() {
            mark.reason = "one_entry_already_consumed";
        } else if let Some((idx, q)) = quote_at(minute) {
            mark.decision_at = Some(q.quote.at);
            let p = mid(q);
            let outside = if p > hi {
                Some(Dir::Buy)
            } else if p < lo {
                Some(Dir::Sell)
            } else {
                None
            };
            let proposal = match opts.entry_rule {
                EntryRule::OpeningRangeBreakout2h | EntryRule::OpeningTickRangeBreakout2h => {
                    outside.map(|s| (s, if s == Dir::Buy { lo } else { hi }))
                }
                EntryRule::OpeningRangeFailedBreakout2h
                | EntryRule::OpeningTickRangeFailedBreakout2h => {
                    if inside(p, lo, hi) {
                        excursion.take().map(|(s, peak)| {
                            (if s == Dir::Buy { Dir::Sell } else { Dir::Buy }, peak)
                        })
                    } else {
                        None
                    }
                }
                EntryRule::CurrentS3
                | EntryRule::FxSessionDiagnostic
                | EntryRule::ClassMomentum1h
                | EntryRule::ClassReversion1h => {
                    return Err("this mode uses its own evaluator".into());
                }
            };
            if let Some(s) = outside {
                excursion = Some(match excursion {
                    Some((old, peak)) if old == s => (
                        s,
                        if s == Dir::Buy {
                            peak.max(p)
                        } else {
                            peak.min(p)
                        },
                    ),
                    _ => (s, p),
                });
            }
            mark.reason = "candidate_condition_absent";
            if let Some((side, stop)) = proposal {
                let price = entry(q, side);
                let raw_r = sign(side) * (price - stop);
                let quantum = 10_i64.pow((5 - src.contract.digits) as u32);
                let e5 = (raw_r * 100000.0).round() as i64;
                let r = ((e5 + quantum / 2) / quantum).max(1) as f64 * quantum as f64 / 100000.0;
                let c = cost_reserve(
                    req.spread,
                    req.commission_round_trip,
                    req.slippage_bps_per_side,
                    price,
                )?
                .total;
                mark.side = Some(side);
                mark.r = Some(r);
                mark.cost = Some(c);
                if raw_r <= 0.0 || c >= r {
                    mark.reason = "candidate_cost_ge_r";
                } else {
                    mark.reason = "no_next_fresh_fill_quote_before_deadline";
                    if let Some((fi, fq)) =
                        quotes.iter().enumerate().skip(idx + 1).find(|(_, f)| {
                            f.fresh
                                && f.quote.at > q.quote.at
                                && f.quote.at < at + Duration::seconds(5)
                        })
                    {
                        let trigger = if opts.entry_rule.failed_breakout() {
                            inside(mid(fq), lo, hi)
                        } else if side == Dir::Buy {
                            mid(fq) > hi
                        } else {
                            mid(fq) < lo
                        };
                        if !trigger {
                            mark.reason = "candidate_changed_before_fill";
                        } else {
                            let plan = targets::plan(&targets::Input {
                                side,
                                entry: price,
                                original_stop: price - sign(side) * r,
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
                                fill: entry(fq, side),
                                side: if side == Dir::Buy {
                                    exits::Side::Long
                                } else {
                                    exits::Side::Short
                                },
                                initial_r: r,
                                volume: opts.volume,
                                tp1_close_volume: plan.close_volume,
                                tp1_distance: (plan.trigger - price).abs(),
                                broker_tp_distance: (plan.broker_target - price).abs(),
                                session_end: req.to,
                                digits: src.contract.digits,
                                cadence: exits::TrailCadence::S3ThirtySeconds,
                            };
                            let mut engine =
                                exits::ExitReplay::new(input.clone()).map_err(|e| e.to_string())?;
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
                            let baseline = s3_ticks::net(req, &input, &comparison.baseline);
                            out.trade = Some(Trade {
                                decision_at: q.quote.at,
                                entry: input,
                                exit_comparison: comparison,
                                trailing_scenario_net_r: trailing,
                                fixed_stop_scenario_net_r: baseline,
                            });
                            mark.reason = "simulated_entry";
                        }
                    }
                }
            }
        } else {
            excursion = None;
        }
        out.marks.push(mark);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::s3_ticks::{Contract, Options, Tick};
    fn fixture(rule: EntryRule, short: bool) -> (Request, Source) {
        let from = "2025-09-08T00:00:00Z".parse().unwrap();
        let r = Request {
            account: 42,
            symbol: "BTCUSD".into(),
            from,
            to: from + Duration::hours(4),
            spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 1,
                entry_rule: rule,
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
        for (min, price) in [
            (30, 100.0),
            (60, 102.0),
            (90, 101.0),
            (120, 100.0),
            (150, 104.0),
            (180, 101.0),
            (210, 98.0),
            (240, 99.0),
        ] {
            for millis in [0, 100] {
                let price: f64 = if short { 202.0 - price } else { price };
                let at = (from + Duration::minutes(min)).timestamp_millis() + millis;
                src.bids.push(Tick {
                    ts_ms: at,
                    price_e5: ((price - 0.01) * 100000.0).round() as i64,
                });
                src.asks.push(Tick {
                    ts_ms: at,
                    price_e5: ((price + 0.01) * 100000.0).round() as i64,
                });
            }
        }
        (r, src)
    }
    #[test]
    fn breakout_and_failed_breakout_are_distinct_causal_rules_with_fixed_close() {
        for short in [false, true] {
            let (r, s) = fixture(EntryRule::OpeningRangeBreakout2h, short);
            let out = s3_ticks::study(&r, &s).unwrap();
            let t = out.trade.unwrap();
            assert_eq!(
                t.entry.entered_at,
                r.from + Duration::minutes(150) + Duration::milliseconds(100)
            );
            assert_eq!(
                t.entry.side,
                if short {
                    exits::Side::Short
                } else {
                    exits::Side::Long
                }
            );
            assert!((t.entry.initial_r - 4.01).abs() < 1e-8);
            let (r, s) = fixture(EntryRule::OpeningRangeFailedBreakout2h, short);
            let out = s3_ticks::study(&r, &s).unwrap();
            let t = out.trade.unwrap();
            assert_eq!(
                t.entry.entered_at,
                r.from + Duration::minutes(180) + Duration::milliseconds(100)
            );
            assert_eq!(
                t.entry.side,
                if short {
                    exits::Side::Long
                } else {
                    exits::Side::Short
                }
            );
            assert!((t.entry.initial_r - 3.01).abs() < 1e-8);
            assert_eq!(t.entry.session_end, r.to);
            assert!(t.trailing_scenario_net_r.is_some());
            assert_eq!(
                t.exit_comparison.candidate.fills.last().unwrap().reason,
                exits::ExitReason::SessionEnd
            );
        }
    }
    #[test]
    fn missing_preparation_never_invents_a_range_and_boundary_has_no_forced_fill() {
        let (r, mut s) = fixture(EntryRule::OpeningRangeFailedBreakout2h, false);
        s.bids
            .retain(|t| t.ts_ms < (r.from + Duration::hours(4)).timestamp_millis());
        s.asks
            .retain(|t| t.ts_ms < (r.from + Duration::hours(4)).timestamp_millis());
        let out = s3_ticks::study(&r, &s).unwrap();
        assert!(out.trade.unwrap().trailing_scenario_net_r.is_none());
        s.bids.retain(|t| {
            t.ts_ms < (r.from + Duration::minutes(60)).timestamp_millis()
                || t.ts_ms >= (r.from + Duration::minutes(61)).timestamp_millis()
        });
        s.asks.retain(|t| {
            t.ts_ms < (r.from + Duration::minutes(60)).timestamp_millis()
                || t.ts_ms >= (r.from + Duration::minutes(61)).timestamp_millis()
        });
        let out = s3_ticks::study(&r, &s).unwrap();
        assert!(out.trade.is_none());
        assert!(out
            .marks
            .iter()
            .all(|m| m.reason == "missing_or_flat_preparation_range"));
    }
    #[test]
    fn future_quotes_cannot_rewrite_entry_or_stop_and_costs_remain_fixed() {
        let (mut r, mut s) = fixture(EntryRule::OpeningRangeBreakout2h, false);
        let before = s3_ticks::study(&r, &s).unwrap().trade.unwrap().entry;
        for t in s
            .bids
            .iter_mut()
            .chain(s.asks.iter_mut())
            .filter(|t| t.ts_ms > (r.from + Duration::minutes(180)).timestamp_millis())
        {
            t.price_e5 += 1000000;
        }
        assert_eq!(
            s3_ticks::study(&r, &s).unwrap().trade.unwrap().entry,
            before
        );
        r.slippage_bps_per_side = 0.0;
        assert!(s3_ticks::study(&r, &s).is_err());
    }
    #[test]
    fn full_tick_range_handles_sparse_marks_and_includes_between_mark_extremes() {
        let (mut r, mut s) = fixture(EntryRule::OpeningTickRangeBreakout2h, false);
        for t in s.bids.iter_mut().chain(s.asks.iter_mut()) {
            if t.ts_ms < (r.from + Duration::minutes(121)).timestamp_millis() {
                t.ts_ms -= 29 * 60 * 1000;
            }
        }
        let full = s3_ticks::study(&r, &s).unwrap();
        assert!(full.trade.is_some());
        r.tick_replay.as_mut().unwrap().entry_rule = EntryRule::OpeningRangeBreakout2h;
        assert!(s3_ticks::study(&r, &s).unwrap().trade.is_none());
        r.tick_replay.as_mut().unwrap().entry_rule = EntryRule::OpeningTickRangeBreakout2h;
        let at = (r.from + Duration::minutes(80)).timestamp_millis();
        s.bids.push(Tick {
            ts_ms: at,
            price_e5: 10_999_000,
        });
        s.asks.push(Tick {
            ts_ms: at,
            price_e5: 11_001_000,
        });
        s.bids.sort_by_key(|t| t.ts_ms);
        s.asks.sort_by_key(|t| t.ts_ms);
        let full = s3_ticks::study(&r, &s).unwrap();
        assert_eq!(full.marks[0].reason, "candidate_condition_absent");
        // Removing an entire preparation block is still missing coverage.
        for ts in [&mut s.bids, &mut s.asks] {
            ts.retain(|t| {
                !(t.ts_ms >= (r.from + Duration::minutes(30)).timestamp_millis()
                    && t.ts_ms < (r.from + Duration::minutes(60)).timestamp_millis())
            });
        }
        assert!(s3_ticks::study(&r, &s).unwrap().trade.is_none());
    }
}
