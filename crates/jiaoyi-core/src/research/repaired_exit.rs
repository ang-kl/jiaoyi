//! Isolated two-stage experiment. Historical `exit_replay.rs` remains byte-frozen.
use super::exit_replay::{
    self as exits, Comparison, Costs, Coverage, DataIssue, ExitFill, ExitReason, Input,
    InvalidConfig, Outcome, Quote, Side,
};
use crate::strategy3::targets::price_ticks;
use chrono::{DateTime, Utc};

pub(super) struct TwoStageReplay {
    fixed: exits::ExitReplay,
    input: Input,
    entry: i64,
    risk: i64,
    target: i64,
    stop: i64,
    pending: Option<DateTime<Utc>>,
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

impl TwoStageReplay {
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
        if self.pending.is_some_and(|at| q.at > at) {
            self.pending = None;
            if sign * (price - self.entry) > 0 {
                self.stop = self.entry;
                self.applied += 1;
            } else {
                self.rejected += 1;
            }
        }
        self.best = Some(match self.best {
            Some(best) if sign * (best - price) >= 0 => best,
            _ => price,
        });
        if sign * (price - self.entry) >= self.risk
            && self.stop != self.entry
            && self.pending.is_none()
        {
            self.pending = Some(q.at);
            self.proposals += 1;
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
            assumptions:"original -1R stop until executable +1R; entry-price amend on a strictly later quote if valid; old server bracket first; gross1.38R target; no further trail; entry-price stop is not net breakeven",
        }
    }
}
