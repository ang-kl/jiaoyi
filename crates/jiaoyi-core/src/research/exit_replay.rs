//! Fixed-entry, recorded-quote exit comparison. Pure and bounded: no entry
//! signals, portfolio model, broker calls or live-policy changes.
//!
//! The candidate trails by the ORIGINAL R behind the best executable bid
//! (long) or ask (short). A proposal waits for the next quote; the old broker
//! bracket is checked first. Sixty seconds between proposals is an assumed
//! S1 operational cadence, not measured S3 amendment behavior. Recorded-path
//! gross scenarios can be calculated with unproven coverage, but qualified
//! results require explicit source, original-plan and grid evidence.
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::strategy3::targets::{price_ticks, MAX_PRICE_DIGITS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Long,
    Short,
}

impl Side {
    fn sign(self) -> i64 {
        match self {
            Self::Long => 1,
            Self::Short => -1,
        }
    }
    fn reached(self, price: i64, level: i64) -> bool {
        self.sign() * (price - level) >= 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailCadence {
    /// Fixed preregistered assumption; deliberately no tunable interval.
    AssumedS1SixtySeconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub entered_at: DateTime<Utc>,
    pub fill: f64,
    pub side: Side,
    /// Original fill-to-SL distance; never recomputed after an amendment.
    pub initial_r: f64,
    /// Broker native volume units; all partial volumes use the same units.
    pub volume: i64,
    pub tp1_close_volume: i64,
    pub tp1_distance: f64,
    pub broker_tp_distance: f64,
    pub session_end: DateTime<Utc>,
    pub digits: i32,
    pub cadence: TrailCadence,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub at: DateTime<Utc>,
    pub bid: f64,
    pub ask: f64,
    /// An explicit missing interval. Do not infer completeness from quiet
    /// timestamps. This still permits an exploratory recorded-path scenario.
    pub gap_before: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Unproven,
    Verified { provenance: String },
}

impl Evidence {
    fn verified(&self) -> bool {
        matches!(self,Self::Verified{provenance} if !provenance.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub through: DateTime<Utc>,
    /// Open legs at this boundary are censored, never force-closed.
    pub censored: bool,
    pub quote_continuity: Evidence,
    pub original_plan: Evidence,
    pub price_grid: Evidence,
}

impl Coverage {
    pub fn recorded_path(through: DateTime<Utc>) -> Self {
        Self {
            through,
            censored: false,
            quote_continuity: Evidence::Unproven,
            original_plan: Evidence::Unproven,
            price_grid: Evidence::Unproven,
        }
    }
}

/// Total costs in ORIGINAL whole-position R, including known commission,
/// swap/conversion and any modeled execution charges. Spread and price gaps
/// are already in executable fills: do not charge them a second time. Negative
/// total cost is permitted for a documented net credit. Each arm needs its own
/// explicit evidence/model; zero is never supplied by default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    pub total_cost_r: f64,
    pub provenance: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Costs {
    pub baseline: Option<Cost>,
    pub candidate: Option<Cost>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    Stop,
    GapThroughStop,
    BrokerTakeProfit,
    Tp1MarketClose,
    SessionEnd,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExitFill {
    pub at: DateTime<Utc>,
    pub price: f64,
    pub volume: i64,
    pub reason: ExitReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataIssue {
    NoQuotes,
    BeforeEntry,
    OutOfOrderQuote,
    InvalidQuote,
    GapBeforeQuote,
    InvalidCoverageBoundary,
    Censored,
    NotClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum InvalidConfig {
    #[error("invalid or off-grid fill/R/target plan")]
    PricePlan,
    #[error("invalid original or TP1 close volume")]
    Volume,
    #[error("session end must follow the original entry")]
    Session,
    #[error("unsupported S3 price digits")]
    Digits,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outcome {
    /// Conditional recorded-path result, NOT an actual net performance claim.
    /// None when not fully closed, malformed, or boundary-invalid.
    pub scenario_gross_r: Option<f64>,
    pub qualified_gross_r: Option<f64>,
    /// Qualified gross minus explicitly supplied total cost-R only.
    pub net_r: Option<f64>,
    pub cost_provenance: Option<String>,
    pub net_r_missing_reason: Option<String>,
    pub realized_scenario_gross_r: f64,
    pub remaining_volume: i64,
    /// At most two: one TP1 partial, then the final close.
    pub fills: Vec<ExitFill>,
    pub active_stop: f64,
    pub best_executable: Option<f64>,
    pub trail_proposals: usize,
    pub trail_applied: usize,
    pub trail_rejected_on_next_quote: usize,
    pub data_issues: Vec<DataIssue>,
    pub missing_evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comparison {
    pub baseline: Outcome,
    pub candidate: Outcome,
    pub scenario_delta_r: Option<f64>,
    pub qualified_delta_r: Option<f64>,
    pub net_delta_r: Option<f64>,
    pub quotes_processed: usize,
    pub coverage: Coverage,
    pub scope: &'static str,
    pub assumptions: &'static str,
}

#[derive(Clone)]
struct Arm {
    remaining: i64,
    stop: i64,
    best: Option<i64>,
    tp1_pending: bool,
    tp1_done: bool,
    trail_pending: Option<i64>,
    last_proposal: Option<DateTime<Utc>>,
    proposals: usize,
    applied: usize,
    rejected: usize,
    realized: f64,
    fills: Vec<ExitFill>,
    gap: bool,
    fatal: Option<DataIssue>,
}

/// Constant-size state apart from at most two fills per arm. The caller feeds
/// the original ordered full quotes; this type never sorts or repairs them.
pub struct ExitReplay {
    input: Input,
    entry: i64,
    risk: i64,
    tp1: i64,
    broker_tp: i64,
    factor: f64,
    baseline: Arm,
    candidate: Arm,
    last_at: Option<DateTime<Utc>>,
    processed: usize,
}

fn grid(price: f64, digits: i32) -> Option<i64> {
    let ticks = price_ticks(price, digits)?;
    ((price * 10f64.powi(digits) - ticks as f64).abs() < 1e-6 && ticks > 0).then_some(ticks)
}

impl ExitReplay {
    pub fn new(input: Input) -> Result<Self, InvalidConfig> {
        if !(0..=MAX_PRICE_DIGITS).contains(&input.digits) {
            return Err(InvalidConfig::Digits);
        }
        if input.volume <= 0 || input.tp1_close_volume <= 0 || input.tp1_close_volume > input.volume
        {
            return Err(InvalidConfig::Volume);
        }
        if input.session_end <= input.entered_at {
            return Err(InvalidConfig::Session);
        }
        let entry = grid(input.fill, input.digits).ok_or(InvalidConfig::PricePlan)?;
        let risk = grid(input.initial_r, input.digits).ok_or(InvalidConfig::PricePlan)?;
        let tp1_distance =
            grid(input.tp1_distance, input.digits).ok_or(InvalidConfig::PricePlan)?;
        let broker_distance =
            grid(input.broker_tp_distance, input.digits).ok_or(InvalidConfig::PricePlan)?;
        let whole = input.tp1_close_volume == input.volume;
        if (whole && broker_distance != tp1_distance) || (!whole && broker_distance <= tp1_distance)
        {
            return Err(InvalidConfig::PricePlan);
        }
        let sign = input.side.sign();
        let stop = entry - sign * risk;
        let tp1 = entry + sign * tp1_distance;
        let broker_tp = entry + sign * broker_distance;
        if stop <= 0 || tp1 <= 0 || broker_tp <= 0 {
            return Err(InvalidConfig::PricePlan);
        }
        let arm = Arm {
            remaining: input.volume,
            stop,
            best: None,
            tp1_pending: false,
            tp1_done: whole,
            trail_pending: None,
            last_proposal: None,
            proposals: 0,
            applied: 0,
            rejected: 0,
            realized: 0.0,
            fills: Vec::with_capacity(2),
            gap: false,
            fatal: None,
        };
        Ok(Self {
            factor: 10f64.powi(input.digits),
            input,
            entry,
            risk,
            tp1,
            broker_tp,
            baseline: arm.clone(),
            candidate: arm,
            last_at: None,
            processed: 0,
        })
    }

    /// Stop feeding when both legs are closed. Later data cannot rewrite a
    /// completed exit. On malformed order/quotes, open legs remain unknown.
    pub fn push(&mut self, quote: Quote) -> Result<(), DataIssue> {
        if self.baseline.remaining == 0 && self.candidate.remaining == 0 {
            return Ok(());
        }
        let issue = if quote.at < self.input.entered_at {
            Some(DataIssue::BeforeEntry)
        } else if self.last_at.is_some_and(|at| quote.at < at) {
            Some(DataIssue::OutOfOrderQuote)
        } else if !quote.bid.is_finite()
            || !quote.ask.is_finite()
            || quote.bid > quote.ask
            || grid(quote.bid, self.input.digits).is_none()
            || grid(quote.ask, self.input.digits).is_none()
        {
            Some(DataIssue::InvalidQuote)
        } else {
            None
        };
        if let Some(issue) = issue {
            for arm in [&mut self.baseline, &mut self.candidate] {
                if arm.remaining > 0 && arm.fatal.is_none() {
                    arm.fatal = Some(issue);
                }
            }
            return Err(issue);
        }
        self.last_at = Some(quote.at);
        self.processed += 1;
        let executable = grid(
            match self.input.side {
                Side::Long => quote.bid,
                Side::Short => quote.ask,
            },
            self.input.digits,
        )
        .unwrap();
        Self::step(
            &self.input,
            self.entry,
            self.risk,
            self.tp1,
            self.broker_tp,
            self.factor,
            &mut self.baseline,
            quote,
            executable,
            false,
        );
        Self::step(
            &self.input,
            self.entry,
            self.risk,
            self.tp1,
            self.broker_tp,
            self.factor,
            &mut self.candidate,
            quote,
            executable,
            true,
        );
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn close(
        input: &Input,
        entry: i64,
        risk: i64,
        factor: f64,
        arm: &mut Arm,
        at: DateTime<Utc>,
        price: i64,
        volume: i64,
        reason: ExitReason,
    ) {
        arm.realized += input.side.sign() as f64 * (price - entry) as f64 / risk as f64
            * volume as f64
            / input.volume as f64;
        arm.remaining -= volume;
        arm.fills.push(ExitFill {
            at,
            price: price as f64 / factor,
            volume,
            reason,
        });
        if arm.remaining == 0 {
            arm.tp1_pending = false;
            arm.trail_pending = None;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        input: &Input,
        entry: i64,
        risk: i64,
        tp1: i64,
        broker_tp: i64,
        factor: f64,
        arm: &mut Arm,
        quote: Quote,
        price: i64,
        trailing: bool,
    ) {
        if arm.remaining == 0 || arm.fatal.is_some() {
            return;
        }
        arm.gap |= quote.gap_before;
        // Old server bracket wins every race. A stop gaps at the worse
        // executable quote; a target never gets a better-than-level fill.
        if input.side.reached(arm.stop, price) {
            let reason = if price == arm.stop {
                ExitReason::Stop
            } else {
                ExitReason::GapThroughStop
            };
            Self::close(
                input,
                entry,
                risk,
                factor,
                arm,
                quote.at,
                price,
                arm.remaining,
                reason,
            );
            return;
        }
        if input.side.reached(price, broker_tp) {
            Self::close(
                input,
                entry,
                risk,
                factor,
                arm,
                quote.at,
                broker_tp,
                arm.remaining,
                ExitReason::BrokerTakeProfit,
            );
            return;
        }
        // A known clock deadline needs no quote-based trigger. First quote
        // at/after the deadline fills the scheduled close after server exits.
        if quote.at >= input.session_end {
            Self::close(
                input,
                entry,
                risk,
                factor,
                arm,
                quote.at,
                price,
                arm.remaining,
                ExitReason::SessionEnd,
            );
            return;
        }
        if arm.tp1_pending {
            let volume = input.tp1_close_volume.min(arm.remaining);
            Self::close(
                input,
                entry,
                risk,
                factor,
                arm,
                quote.at,
                price,
                volume,
                ExitReason::Tp1MarketClose,
            );
            arm.tp1_pending = false;
            arm.tp1_done = true;
        }
        if let Some(stop) = arm.trail_pending.take() {
            // A proposal that became marketable is not an accepted amend or
            // an invented fill at its old price. Its original stop still stood.
            if input.side.reached(stop, arm.stop)
                && stop != arm.stop
                && input.side.reached(price, stop)
                && price != stop
            {
                arm.stop = stop;
                arm.applied += 1;
            } else {
                arm.rejected += 1;
            }
        }
        if trailing {
            arm.best = Some(match arm.best {
                Some(best) if input.side.reached(best, price) => best,
                _ => price,
            });
            let stop = arm.best.unwrap() - input.side.sign() * risk;
            let due = arm
                .last_proposal
                .is_none_or(|at| quote.at.signed_duration_since(at) >= Duration::seconds(60));
            if stop > 0
                && input.side.reached(stop, arm.stop)
                && stop != arm.stop
                && input.side.reached(price, stop)
                && price != stop
                && due
            {
                arm.trail_pending = Some(stop);
                arm.last_proposal = Some(quote.at);
                arm.proposals += 1;
            }
        }
        if !arm.tp1_done && !arm.tp1_pending && input.side.reached(price, tp1) {
            arm.tp1_pending = true;
        }
    }

    pub fn finish(self, coverage: Coverage, costs: Costs) -> Comparison {
        let boundary_valid = coverage.through >= self.input.entered_at
            && self.last_at.is_none_or(|at| coverage.through >= at);
        let baseline = self.outcome(
            self.baseline.clone(),
            &coverage,
            costs.baseline,
            boundary_valid,
        );
        let candidate = self.outcome(
            self.candidate.clone(),
            &coverage,
            costs.candidate,
            boundary_valid,
        );
        let scenario_delta_r = baseline
            .scenario_gross_r
            .zip(candidate.scenario_gross_r)
            .map(|(b, c)| c - b);
        let qualified_delta_r = baseline
            .qualified_gross_r
            .zip(candidate.qualified_gross_r)
            .map(|(b, c)| c - b);
        let net_delta_r = baseline.net_r.zip(candidate.net_r).map(|(b, c)| c - b);
        Comparison{baseline,candidate,scenario_delta_r,qualified_delta_r,net_delta_r,quotes_processed:self.processed,coverage,
            scope:"fixed original entries and registered volume/targets only; conditional exit scenarios, not a full strategy backtest or financial PASS",
            assumptions:"candidate original-R trail; sixty-second S1 cadence assumed for simulation, no measured S3 amend latency/rejections; next-quote TP1/amend; scheduled session close at first quote>=end; old server bracket first; no entry/portfolio feedback"}
    }

    fn outcome(
        &self,
        arm: Arm,
        coverage: &Coverage,
        cost: Option<Cost>,
        boundary_valid: bool,
    ) -> Outcome {
        let mut issues = Vec::new();
        if self.processed == 0 {
            issues.push(DataIssue::NoQuotes);
        }
        if let Some(issue) = arm.fatal {
            issues.push(issue);
        }
        if arm.gap {
            issues.push(DataIssue::GapBeforeQuote);
        }
        if !boundary_valid {
            issues.push(DataIssue::InvalidCoverageBoundary);
        }
        if arm.remaining > 0 {
            issues.push(if coverage.censored {
                DataIssue::Censored
            } else {
                DataIssue::NotClosed
            });
        }
        let scenario =
            (arm.remaining == 0 && arm.fatal.is_none() && boundary_valid).then_some(arm.realized);
        let mut missing = Vec::new();
        for (name, evidence) in [
            ("quote continuity", &coverage.quote_continuity),
            ("original plan/volume", &coverage.original_plan),
            ("original price grid", &coverage.price_grid),
        ] {
            if !evidence.verified() {
                missing.push(format!("unproven {name}"));
            }
        }
        let qualified = if missing.is_empty() && issues.is_empty() {
            scenario
        } else {
            None
        };
        let valid_cost =
            cost.filter(|c| c.total_cost_r.is_finite() && !c.provenance.trim().is_empty());
        let net = qualified
            .zip(valid_cost.as_ref())
            .map(|(r, c)| r - c.total_cost_r)
            .filter(|r| r.is_finite());
        let net_r_missing_reason = if net.is_some() {
            None
        } else if qualified.is_none() {
            Some("unqualified or incomplete recorded-path evidence".into())
        } else {
            Some("explicit finite total cost-R with provenance unavailable".into())
        };
        Outcome {
            scenario_gross_r: scenario,
            qualified_gross_r: qualified,
            net_r: net,
            cost_provenance: valid_cost.map(|c| c.provenance),
            net_r_missing_reason,
            realized_scenario_gross_r: arm.realized,
            remaining_volume: arm.remaining,
            fills: arm.fills,
            active_stop: arm.stop as f64 / self.factor,
            best_executable: arm.best.map(|p| p as f64 / self.factor),
            trail_proposals: arm.proposals,
            trail_applied: arm.applied,
            trail_rejected_on_next_quote: arm.rejected,
            data_issues: issues,
            missing_evidence: missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_793_020_000 + seconds, 0).unwrap()
    }
    fn input() -> Input {
        Input {
            entered_at: at(0),
            fill: 100.0,
            side: Side::Long,
            initial_r: 10.0,
            volume: 100,
            tp1_close_volume: 25,
            tp1_distance: 30.0,
            broker_tp_distance: 40.0,
            session_end: at(1000),
            digits: 2,
            cadence: TrailCadence::AssumedS1SixtySeconds,
        }
    }
    fn q(seconds: i64, bid: f64, ask: f64) -> Quote {
        Quote {
            at: at(seconds),
            bid,
            ask,
            gap_before: false,
        }
    }
    fn verified(through: DateTime<Utc>) -> Coverage {
        let evidence = Evidence::Verified {
            provenance: "explicit synthetic complete fixture".into(),
        };
        Coverage {
            through,
            censored: false,
            quote_continuity: evidence.clone(),
            original_plan: evidence.clone(),
            price_grid: evidence,
        }
    }
    fn replay(i: Input, quotes: &[Quote]) -> Comparison {
        let mut r = ExitReplay::new(i).unwrap();
        for &quote in quotes {
            r.push(quote).unwrap();
        }
        r.finish(verified(quotes.last().unwrap().at), Costs::default())
    }
    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-10, "{a} != {b}");
    }

    #[test]
    fn original_r_trail_only_tightens_and_never_uses_proposal_on_trigger_quote() {
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 100.0, 101.0)).unwrap();
        r.push(q(1, 120.0, 121.0)).unwrap();
        assert_eq!(
            r.candidate.stop, 9000,
            "proposal cannot already protect its deciding quote"
        );
        r.push(q(2, 119.0, 120.0)).unwrap();
        assert_eq!(r.candidate.stop, 11000);
        r.push(q(3, 115.0, 116.0)).unwrap();
        assert_eq!(r.candidate.stop, 11000, "pullback cannot loosen");
        r.push(q(61, 125.0, 126.0)).unwrap();
        r.push(q(62, 116.0, 117.0)).unwrap();
        assert_eq!(r.candidate.stop, 11500);
        r.push(q(63, 114.0, 115.0)).unwrap();
        r.push(q(1000, 100.0, 101.0)).unwrap();
        let result = r.finish(verified(at(1000)), Costs::default());
        close(result.candidate.scenario_gross_r.unwrap(), 1.4);
        close(result.baseline.scenario_gross_r.unwrap(), 0.0);
        assert_eq!(result.candidate.best_executable, Some(125.0));
        assert_eq!(result.candidate.trail_applied, 2);
        assert!(
            result.candidate.net_r.is_none(),
            "gross profit is not net evidence"
        );
    }

    #[test]
    fn short_uses_ask_for_best_trigger_and_exit_despite_favorable_bid() {
        let i = Input {
            side: Side::Short,
            ..input()
        };
        let result = replay(
            i,
            &[
                q(0, 89.0, 101.0),
                q(1, 80.0, 90.0),
                q(2, 94.0, 99.0),
                q(3, 90.0, 101.0),
                q(1000, 99.0, 100.0),
            ],
        );
        close(result.candidate.scenario_gross_r.unwrap(), -0.1);
        assert_eq!(result.candidate.fills[0].price, 101.0);
        assert_eq!(result.candidate.best_executable, Some(90.0));
        assert_eq!(result.candidate.active_stop, 100.0);
        let target = replay(
            Input {
                side: Side::Short,
                ..input()
            },
            &[q(0, 60.0, 75.0), q(1, 58.0, 59.0)],
        );
        assert_eq!(
            target.baseline.fills.len(),
            1,
            "bid alone must not trigger short TP1"
        );
        assert_eq!(
            target.baseline.fills[0].reason,
            ExitReason::BrokerTakeProfit
        );
        assert_eq!(
            target.baseline.fills[0].price, 60.0,
            "target does not receive favorable gap price59"
        );
        close(target.baseline.scenario_gross_r.unwrap(), 4.0);
    }

    #[test]
    fn old_broker_stop_and_target_precede_pending_amend_or_market_close() {
        let stopped = replay(input(), &[q(0, 120.0, 121.0), q(1, 85.0, 86.0)]);
        for arm in [&stopped.baseline, &stopped.candidate] {
            close(arm.scenario_gross_r.unwrap(), -1.5);
            assert_eq!(arm.fills[0].price, 85.0);
            assert_eq!(arm.fills[0].reason, ExitReason::GapThroughStop);
        }
        assert_eq!(stopped.candidate.trail_applied, 0);
        let target = replay(input(), &[q(0, 130.0, 131.0), q(1, 145.0, 146.0)]);
        for arm in [&target.baseline, &target.candidate] {
            close(arm.scenario_gross_r.unwrap(), 4.0);
            assert_eq!(
                arm.fills.len(),
                1,
                "TP2 must cancel pending TP1 without double-close"
            );
            assert_eq!(arm.fills[0].volume, 100);
            assert_eq!(arm.fills[0].price, 140.0);
        }
        let deadline = replay(input(), &[q(999, 130.0, 131.0), q(1000, 85.0, 86.0)]);
        assert_eq!(
            deadline.baseline.fills[0].reason,
            ExitReason::GapThroughStop,
            "server bracket also precedes scheduled session close"
        );
    }

    #[test]
    fn tp1_fills_next_quote_once_and_whole_minimum_has_no_client_partial() {
        let result = replay(
            input(),
            &[
                q(0, 130.0, 131.0),
                q(1, 131.0, 132.0),
                q(2, 132.0, 133.0),
                q(1000, 135.0, 136.0),
            ],
        );
        for arm in [&result.baseline, &result.candidate] {
            assert_eq!(arm.fills.len(), 2);
            assert_eq!(arm.fills[0].at, at(1));
            assert_eq!(arm.fills[0].reason, ExitReason::Tp1MarketClose);
            assert_eq!(arm.fills[0].volume, 25);
            assert_eq!(arm.fills[0].price, 131.0);
            assert_eq!(arm.fills[1].volume, 75);
            assert_eq!(arm.fills[1].reason, ExitReason::SessionEnd);
            close(arm.scenario_gross_r.unwrap(), 3.4);
        }
        let minimum = replay(
            Input {
                tp1_close_volume: 100,
                broker_tp_distance: 30.0,
                ..input()
            },
            &[q(0, 131.0, 132.0)],
        );
        for arm in [&minimum.baseline, &minimum.candidate] {
            assert_eq!(arm.fills.len(), 1);
            assert_eq!(arm.fills[0].reason, ExitReason::BrokerTakeProfit);
            assert_eq!(arm.fills[0].volume, 100);
            close(arm.scenario_gross_r.unwrap(), 3.0);
        }
    }

    #[test]
    fn invalid_pending_stop_is_discarded_and_sixty_second_proposal_cadence_is_fixed() {
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 120.0, 121.0)).unwrap();
        r.push(q(1, 100.0, 101.0)).unwrap();
        assert_eq!(r.candidate.stop, 9000);
        assert_eq!(r.candidate.rejected, 1);
        r.push(q(59, 121.0, 122.0)).unwrap();
        assert_eq!(r.candidate.proposals, 1, "no amend before60secs");
        r.push(q(60, 121.0, 122.0)).unwrap();
        assert_eq!(r.candidate.proposals, 2);
        assert_eq!(r.candidate.stop, 9000);
        r.push(q(61, 112.0, 113.0)).unwrap();
        assert_eq!(r.candidate.stop, 11100);
        r.push(q(62, 110.0, 111.0)).unwrap();
        r.push(q(1000, 100.0, 101.0)).unwrap();
        let result = r.finish(verified(at(1000)), Costs::default());
        close(result.candidate.scenario_gross_r.unwrap(), 1.0);
        assert_eq!(result.candidate.trail_rejected_on_next_quote, 1);
    }

    #[test]
    fn prefix_cannot_see_a_future_amend_fill_and_closed_legs_ignore_later_data() {
        let mut prefix = ExitReplay::new(input()).unwrap();
        prefix.push(q(0, 120.0, 121.0)).unwrap();
        let prefix = prefix.finish(verified(at(0)), Costs::default());
        assert!(prefix.candidate.scenario_gross_r.is_none());
        assert_eq!(prefix.candidate.active_stop, 90.0);
        assert_eq!(prefix.candidate.remaining_volume, 100);
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 85.0, 86.0)).unwrap();
        r.push(q(-100, f64::NAN, 0.0)).unwrap();
        let result = r.finish(verified(at(0)), Costs::default());
        assert_eq!(result.quotes_processed, 1);
        close(result.candidate.qualified_gross_r.unwrap(), -1.5);
    }

    #[test]
    fn unproven_or_gapped_paths_get_exploratory_gross_but_never_qualified_or_net_results() {
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 145.0, 146.0)).unwrap();
        let result = r.finish(
            Coverage::recorded_path(at(0)),
            Costs {
                baseline: Some(Cost {
                    total_cost_r: 0.0,
                    provenance: "explicit model".into(),
                }),
                candidate: None,
            },
        );
        close(result.baseline.scenario_gross_r.unwrap(), 4.0);
        assert!(result.baseline.qualified_gross_r.is_none());
        assert!(result.baseline.net_r.is_none());
        assert_eq!(result.baseline.missing_evidence.len(), 3);
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(Quote {
            gap_before: true,
            ..q(0, 85.0, 86.0)
        })
        .unwrap();
        let result = r.finish(verified(at(0)), Costs::default());
        close(result.baseline.scenario_gross_r.unwrap(), -1.5);
        assert!(result.baseline.qualified_gross_r.is_none());
        assert!(result
            .baseline
            .data_issues
            .contains(&DataIssue::GapBeforeQuote));
    }

    #[test]
    fn empty_censored_invalid_order_and_bad_boundary_never_force_a_close() {
        let empty = ExitReplay::new(input())
            .unwrap()
            .finish(verified(at(0)), Costs::default());
        assert!(empty.baseline.scenario_gross_r.is_none());
        assert!(empty.baseline.data_issues.contains(&DataIssue::NoQuotes));
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 100.0, 101.0)).unwrap();
        let result = r.finish(
            Coverage {
                censored: true,
                ..verified(at(500))
            },
            Costs::default(),
        );
        assert!(result.candidate.scenario_gross_r.is_none());
        assert!(result.candidate.data_issues.contains(&DataIssue::Censored));
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(10, 120.0, 121.0)).unwrap();
        assert_eq!(r.push(q(9, 130.0, 131.0)), Err(DataIssue::OutOfOrderQuote));
        r.push(q(1000, 140.0, 141.0)).unwrap();
        let result = r.finish(verified(at(1000)), Costs::default());
        assert!(result.candidate.scenario_gross_r.is_none());
        assert!(result.candidate.fills.is_empty());
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(10, 145.0, 146.0)).unwrap();
        let result = r.finish(verified(at(9)), Costs::default());
        assert!(result.candidate.scenario_gross_r.is_none());
        assert!(result
            .candidate
            .data_issues
            .contains(&DataIssue::InvalidCoverageBoundary));
    }

    #[test]
    fn net_r_requires_separate_explicit_finite_cost_provenance_for_each_arm() {
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 145.0, 146.0)).unwrap();
        let result = r.finish(
            verified(at(0)),
            Costs {
                baseline: Some(Cost {
                    total_cost_r: 0.2,
                    provenance: "baseline measured/modelled full trade costs".into(),
                }),
                candidate: Some(Cost {
                    total_cost_r: 0.3,
                    provenance: "candidate explicit counterfactual cost model".into(),
                }),
            },
        );
        close(result.baseline.net_r.unwrap(), 3.8);
        close(result.candidate.net_r.unwrap(), 3.7);
        close(result.net_delta_r.unwrap(), -0.1);
        let mut r = ExitReplay::new(input()).unwrap();
        r.push(q(0, 145.0, 146.0)).unwrap();
        let result = r.finish(
            verified(at(0)),
            Costs {
                baseline: Some(Cost {
                    total_cost_r: 0.0,
                    provenance: " ".into(),
                }),
                candidate: Some(Cost {
                    total_cost_r: f64::NAN,
                    provenance: "invalid cost".into(),
                }),
            },
        );
        assert!(result.baseline.net_r.is_none());
        assert!(result.candidate.net_r.is_none());
    }

    #[test]
    fn invalid_config_and_malformed_full_quotes_are_explicit() {
        for i in [
            Input {
                initial_r: 0.0,
                ..input()
            },
            Input {
                fill: 100.001,
                ..input()
            },
            Input {
                tp1_close_volume: 0,
                ..input()
            },
            Input {
                volume: 20,
                ..input()
            },
            Input {
                digits: 6,
                ..input()
            },
            Input {
                session_end: at(0),
                ..input()
            },
            Input {
                tp1_close_volume: 100,
                ..input()
            },
        ] {
            assert!(ExitReplay::new(i).is_err());
        }
        for quote in [
            q(0, 101.0, 100.0),
            q(0, 100.001, 101.0),
            q(0, f64::NAN, 101.0),
        ] {
            let mut r = ExitReplay::new(input()).unwrap();
            assert_eq!(r.push(quote), Err(DataIssue::InvalidQuote));
            assert!(r
                .finish(verified(at(0)), Costs::default())
                .candidate
                .scenario_gross_r
                .is_none());
        }
    }
}
