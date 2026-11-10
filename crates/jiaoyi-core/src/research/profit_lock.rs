//! №18,412: frozen exit-only experiment. No live caller. The historical
//! two-stage engine stays unchanged and supplies the paired baseline.
use super::exit_replay::{
    self as exits, Comparison, Costs, Coverage, DataIssue, ExitFill, ExitReason, Input,
    InvalidConfig, Outcome, Quote, Side,
};
use crate::strategy3::targets::price_ticks;
use chrono::{DateTime, Utc};

pub(super) struct ProfitLockReplay {
    fixed: exits::ExitReplay,
    input: Input,
    entry: i64,
    risk: i64,
    target: i64,
    stop: i64,
    pending: Option<(DateTime<Utc>, i64)>,
    best: Option<i64>,
    fills: Vec<ExitFill>,
    proposals: usize,
    applied: usize,
    rejected: usize,
    gap: bool,
    fatal: Option<DataIssue>,
    last: Option<DateTime<Utc>>,
    processed: usize,
}

impl ProfitLockReplay {
    fn sign(&self) -> i64 {
        if self.input.side == Side::Long {
            1
        } else {
            -1
        }
    }

    pub(super) fn new(input: Input) -> Result<Self, InvalidConfig> {
        if input.tp1_close_volume != input.volume {
            return Err(InvalidConfig::Volume);
        }
        // Reuse the registered input validator and fixed-stop baseline unchanged.
        let fixed = exits::ExitReplay::new(input.clone())?;
        let entry = price_ticks(input.fill, input.digits).unwrap();
        let risk = price_ticks(input.initial_r, input.digits).unwrap();
        let sign = if input.side == Side::Long { 1 } else { -1 };
        let target = entry + sign * price_ticks(input.broker_tp_distance, input.digits).unwrap();
        Ok(Self {
            fixed,
            input,
            entry,
            risk,
            target,
            stop: entry - sign * risk,
            pending: None,
            best: None,
            fills: vec![],
            proposals: 0,
            applied: 0,
            rejected: 0,
            gap: false,
            fatal: None,
            last: None,
            processed: 0,
        })
    }

    fn close(&mut self, at: DateTime<Utc>, price: i64, reason: ExitReason) {
        self.fills.push(ExitFill {
            at,
            price: price as f64 / 10_f64.powi(self.input.digits),
            volume: self.input.volume,
            reason,
        });
        self.pending = None;
    }

    pub(super) fn push(&mut self, q: Quote) -> Result<(), DataIssue> {
        if let Err(e) = self.fixed.push(q) {
            if self.fills.is_empty() {
                self.fatal = Some(e);
            }
            return Err(e);
        }
        if !self.fills.is_empty() || self.fatal.is_some() {
            return Ok(());
        }
        self.last = Some(q.at);
        self.processed += 1;
        self.gap |= q.gap_before;
        let price = price_ticks(
            if self.input.side == Side::Long {
                q.bid
            } else {
                q.ask
            },
            self.input.digits,
        )
        .unwrap();
        let sign = self.sign();
        // Existing server bracket wins before a pending amend or clock close.
        if sign * (self.stop - price) >= 0 {
            self.close(
                q.at,
                price,
                if price == self.stop {
                    ExitReason::Stop
                } else {
                    ExitReason::GapThroughStop
                },
            );
            return Ok(());
        }
        if sign * (price - self.target) >= 0 {
            self.close(q.at, self.target, ExitReason::BrokerTakeProfit);
            return Ok(());
        }
        if q.at >= self.input.session_end {
            self.close(q.at, price, ExitReason::SessionEnd);
            return Ok(());
        }
        if let Some((_, proposed)) = self.pending.filter(|(at, _)| q.at > *at) {
            self.pending = None;
            if sign * (price - proposed) > 0 {
                self.stop = proposed;
                self.applied += 1;
            } else {
                self.rejected += 1;
            }
        }
        self.best = Some(match self.best {
            Some(best) if sign * (best - price) >= 0 => best,
            _ => price,
        });
        let excursion = i128::from(sign * (price - self.entry));
        let risk = i128::from(self.risk);
        // Integer price grid; round the half-R stop towards entry, never claim
        // a larger locked distance than the frozen +0.50R policy.
        let proposed = if excursion * 100 >= risk * 80 {
            Some(self.entry + sign * (self.risk / 2))
        } else if excursion * 100 >= risk * 65 {
            Some(self.entry)
        } else {
            None
        };
        if let Some(stop) = proposed.filter(|stop| sign * (*stop - self.stop) > 0) {
            if self.pending.is_none() {
                self.pending = Some((q.at, stop));
                self.proposals += 1;
            }
        }
        Ok(())
    }

    pub(super) fn finish(self, coverage: Coverage, costs: Costs) -> Comparison {
        let sign = self.sign() as f64;
        let fixed = self.fixed.finish(coverage.clone(), costs);
        let closed = !self.fills.is_empty();
        let valid_boundary = coverage.through >= self.input.entered_at
            && self.last.is_none_or(|at| at <= coverage.through);
        let gross = self
            .fills
            .iter()
            .map(|f| sign * (f.price - self.input.fill) / self.input.initial_r)
            .sum();
        let mut issues = vec![];
        if self.processed == 0 {
            issues.push(DataIssue::NoQuotes);
        }
        if let Some(e) = self.fatal {
            issues.push(e);
        }
        if self.gap {
            issues.push(DataIssue::GapBeforeQuote);
        }
        if !valid_boundary {
            issues.push(DataIssue::InvalidCoverageBoundary);
        }
        if !closed {
            issues.push(if coverage.censored {
                DataIssue::Censored
            } else {
                DataIssue::NotClosed
            });
        }
        let scenario = (closed && valid_boundary && self.fatal.is_none()).then_some(gross);
        let candidate = Outcome {
            scenario_gross_r: scenario,
            qualified_gross_r: None,
            net_r: None,
            cost_provenance: None,
            net_r_missing_reason: Some(
                "offline policy and recorded quote evidence remain unqualified".into(),
            ),
            realized_scenario_gross_r: gross,
            remaining_volume: if closed { 0 } else { self.input.volume },
            fills: self.fills,
            active_stop: self.stop as f64 / 10_f64.powi(self.input.digits),
            best_executable: self.best.map(|p| p as f64 / 10_f64.powi(self.input.digits)),
            trail_proposals: self.proposals,
            trail_applied: self.applied,
            trail_rejected_on_next_quote: self.rejected,
            data_issues: issues,
            missing_evidence: vec![
                "unqualified experimental exit policy and source continuity".into()
            ],
        };
        Comparison {
            scenario_delta_r:fixed.baseline.scenario_gross_r.zip(scenario).map(|(b,c)|c-b),
            qualified_delta_r:None,net_delta_r:None,baseline:fixed.baseline,candidate,
            quotes_processed:fixed.quotes_processed,coverage,
            scope:"offline two-stage exit experiment with changed gross target; not production policy or financial PASS",
            assumptions:"original -1R stop; +0.65R proposes entry stop, +0.80R proposes +0.50R stop; valid amendment on a strictly later quote; old bracket first; gross target unchanged; neither stop level guarantees net profit",
        }
    }
}

/// Both paths receive identical quotes, fills, volume, original risk and expiry.
/// Costs are repriced by the caller; the quotes already include bid/ask spread.
pub fn compare(
    input: Input,
    quotes: &[Quote],
    through: DateTime<Utc>,
) -> Result<serde_json::Value, String> {
    let mut old =
        super::repaired_exit::TwoStageReplay::new(input.clone()).map_err(|e| e.to_string())?;
    let mut new = ProfitLockReplay::new(input).map_err(|e| e.to_string())?;
    for q in quotes {
        old.push(*q).map_err(|e| format!("baseline quote: {e:?}"))?;
        new.push(*q)
            .map_err(|e| format!("candidate quote: {e:?}"))?;
    }
    let coverage = Coverage {
        censored: true,
        ..Coverage::recorded_path(through)
    };
    Ok(serde_json::json!({
        "baseline": old.finish(coverage.clone(), Costs::default()).candidate,
        "candidate": new.finish(coverage, Costs::default()).candidate,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    fn input(side: Side) -> Input {
        Input {
            entered_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            fill: 100.0,
            initial_r: 10.0,
            side,
            volume: 100,
            tp1_close_volume: 100,
            tp1_distance: 13.8,
            broker_tp_distance: 13.8,
            session_end: Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap(),
            digits: 2,
            cadence: exits::TrailCadence::S3ThirtySeconds,
        }
    }
    fn quotes(i: &Input, prices: &[f64]) -> Vec<Quote> {
        prices
            .iter()
            .enumerate()
            .map(|(n, p)| Quote {
                at: i.entered_at + Duration::seconds(n as i64),
                bid: *p,
                ask: *p,
                gap_before: false,
            })
            .collect()
    }
    #[test]
    fn both_directions_lock_half_r_only_after_later_valid_quote_and_gap_fills_are_real() {
        for side in [Side::Long, Side::Short] {
            let i = input(side);
            let s = if side == Side::Long { 1.0 } else { -1.0 };
            let prices: Vec<_> = [0., 6.5, 7., 8., 8.5, 4.]
                .into_iter()
                .map(|p| 100. + s * p)
                .collect();
            let q = quotes(&i, &prices);
            let v = compare(i.clone(), &q, q.last().unwrap().at).unwrap();
            assert!(v["baseline"]["scenario_gross_r"].is_null());
            assert_eq!(v["candidate"]["trail_applied"], 2);
            assert!((v["candidate"]["scenario_gross_r"].as_f64().unwrap() - 0.4).abs() < 1e-12);
            assert_eq!(v["candidate"]["fills"][0]["reason"], "gap_through_stop");
        }
    }
    #[test]
    fn no_retroactive_lock_when_next_quote_gaps_through_initial_stop() {
        let i = input(Side::Long);
        let q = quotes(&i, &[100., 108., 89.]);
        let v = compare(i, &q, q.last().unwrap().at).unwrap();
        assert_eq!(v["candidate"]["trail_applied"], 0);
        assert!((v["candidate"]["scenario_gross_r"].as_f64().unwrap() + 1.1).abs() < 1e-12);
        assert_eq!(
            v["candidate"]["scenario_gross_r"],
            v["baseline"]["scenario_gross_r"]
        );
    }
}

use super::{s3_signal::Request, s3_ticks};
use chrono::Duration;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    entry: Input,
    through: DateTime<Utc>,
    #[serde(default)]
    quotes: Vec<Quote>,
    request: Option<Request>,
    source: Option<s3_ticks::Source>,
    forced: bool,
}
pub fn compare_case(case: Case) -> Result<serde_json::Value, String> {
    let quotes = match (case.request, case.source) {
        (Some(req), Some(src)) if case.quotes.is_empty() => {
            let (points, _) = s3_ticks::snapshots(&req, &src)?;
            let through = if case.forced {
                req.to + Duration::seconds(5)
            } else {
                req.to
            };
            if through != case.through {
                return Err("boundary mismatch".into());
            }
            points
                .into_iter()
                .filter(|q| {
                    q.quote.at >= case.entry.entered_at
                        && q.quote.at <= through
                        && (!case.forced || q.quote.at < req.to || q.fresh)
                })
                .map(|q| q.quote)
                .collect()
        }
        (None, None) if case.quotes.len() <= 200_000 => case.quotes,
        _ => return Err("ambiguous or oversized quote source".into()),
    };
    compare(case.entry, &quotes, case.through)
}
