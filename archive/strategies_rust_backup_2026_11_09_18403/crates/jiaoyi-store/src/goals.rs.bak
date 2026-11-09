//! Read-only, per-strategy/account goal evidence. This never changes trading.
//!
//! The realized payoff reference is separate from the registered SL/TP plan.
//! One qualifying rolling window never establishes consistent profitability.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};

use crate::trading::ClosedTrade;

pub const WINDOW: usize = 20;
pub const WIN_RATE_TARGET: f64 = 0.73;
pub const REALIZED_PAYOFF_REFERENCE: f64 = 1.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentState {
    InsufficientEvidence,
    BelowTarget,
    WindowMeetsNumericReference,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowMetrics {
    pub count: usize,
    pub wins: usize,
    pub losses: usize,
    pub breakeven: usize,
    pub net_r: f64,
    pub win_rate: Option<f64>,
    pub realized_payoff: Option<f64>,
    pub realized_payoff_state: String,
    pub profit_factor: Option<f64>,
    pub profit_factor_state: String,
    pub first_closed_at: Option<DateTime<Utc>>,
    pub last_closed_at: Option<DateTime<Utc>>,
    pub position_ids: Vec<i64>,
    pub entry_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Assessment {
    pub account_id: i64,
    pub strategy: String,
    pub state: AssessmentState,
    pub assessed_at: DateTime<Utc>,
    pub measured_closed_positions: usize,
    pub unmeasured_positions: usize,
    pub window: WindowMetrics,
    pub missing_reasons: Vec<String>,
    pub latest_observed_exit_at: Option<DateTime<Utc>>,
    pub source: String,
    pub history_fetched_at: Option<DateTime<Utc>>,
    pub history_through: Option<DateTime<Utc>>,
    pub history_window_from: Option<DateTime<Utc>>,
    pub history_max_age_secs: i64,
    pub history_complete: bool,
    pub consistency: String,
    pub win_rate_target: f64,
    pub realized_payoff_reference: f64,
    pub planned_risk_reward: String,
    /// Stable across polls; contains the known cohort and evidence state,
    /// excluding the poll timestamp. It is an identity, not a cryptographic hash.
    pub sample_identity: String,
}

fn window_metrics(trades: &[ClosedTrade]) -> WindowMetrics {
    let mut ordered = trades.to_vec();
    ordered.sort_by(|a, b| {
        a.closed_at
            .cmp(&b.closed_at)
            .then(a.position_id.cmp(&b.position_id))
            .then(a.entry_key.cmp(&b.entry_key))
    });
    let start = ordered.len().saturating_sub(WINDOW);
    let window = &ordered[start..];
    let wins = window.iter().filter(|t| t.r > 0.0).count();
    let losses = window.iter().filter(|t| t.r < 0.0).count();
    let positive: f64 = window.iter().filter(|t| t.r > 0.0).map(|t| t.r).sum();
    let negative: f64 = window.iter().filter(|t| t.r < 0.0).map(|t| -t.r).sum();
    WindowMetrics {
        count: window.len(),
        wins,
        losses,
        breakeven: window.len() - wins - losses,
        net_r: window.iter().map(|t| t.r).sum(),
        win_rate: (!window.is_empty()).then(|| wins as f64 / window.len() as f64),
        realized_payoff: (wins > 0 && losses > 0)
            .then(|| (positive / wins as f64) / (negative / losses as f64)),
        realized_payoff_state: if wins == 0 && losses == 0 {
            "no_wins_or_losses"
        } else if wins == 0 {
            "no_wins"
        } else if losses == 0 {
            "no_losses"
        } else {
            "defined"
        }
        .into(),
        profit_factor: (negative > 0.0).then(|| positive / negative),
        profit_factor_state: if negative > 0.0 {
            "defined"
        } else if positive > 0.0 {
            "infinite"
        } else {
            "undefined"
        }
        .into(),
        first_closed_at: window.first().map(|t| t.closed_at),
        last_closed_at: window.last().map(|t| t.closed_at),
        position_ids: window.iter().map(|t| t.position_id).collect(),
        entry_keys: window.iter().map(|t| t.entry_key.clone()).collect(),
    }
}

/// Evaluate known, cost-complete positions. Source-quality reasons take
/// precedence over a numerical comparison; incomplete positions are not wins.
pub fn summarize(
    account_id: i64,
    strategy: &str,
    trades: &[ClosedTrade],
    unmeasured_positions: usize,
    missing_reasons: Vec<String>,
    latest_observed_exit_at: Option<DateTime<Utc>>,
) -> Assessment {
    summarize_at(
        account_id,
        strategy,
        trades,
        unmeasured_positions,
        missing_reasons,
        latest_observed_exit_at,
        Utc::now(),
    )
}

#[allow(clippy::too_many_arguments)]
fn summarize_at(
    account_id: i64,
    strategy: &str,
    trades: &[ClosedTrade],
    unmeasured_positions: usize,
    mut missing_reasons: Vec<String>,
    latest_observed_exit_at: Option<DateTime<Utc>>,
    assessed_at: DateTime<Utc>,
) -> Assessment {
    if trades.iter().any(|t| t.closed_at > assessed_at) {
        missing_reasons.push("a broker close timestamp is after the assessment cutoff".into());
    }
    if latest_observed_exit_at.is_some_and(|at| at > assessed_at) {
        missing_reasons.push("an exit receipt timestamp is after the assessment cutoff".into());
    }
    if unmeasured_positions > 0 {
        missing_reasons.push(format!(
            "{unmeasured_positions} position lifecycle(s) lack complete net-R evidence"
        ));
    }
    if trades.iter().any(|t| !t.r.is_finite()) {
        missing_reasons.push("a measured net-R value is not finite".into());
    }
    let finite: Vec<_> = trades
        .iter()
        .filter(|t| t.r.is_finite() && t.closed_at <= assessed_at)
        .cloned()
        .collect();
    let unmeasured_positions = unmeasured_positions + trades.len() - finite.len();
    let window = window_metrics(&finite);
    if !window.net_r.is_finite()
        || window.realized_payoff.is_some_and(|v| !v.is_finite())
        || window.profit_factor.is_some_and(|v| !v.is_finite())
    {
        missing_reasons.push("aggregate net-R evidence is not finite".into());
    }
    let state = if !missing_reasons.is_empty() {
        AssessmentState::Unknown
    } else if window.count < WINDOW {
        AssessmentState::InsufficientEvidence
    } else if window.win_rate.is_some_and(|v| v < WIN_RATE_TARGET) {
        AssessmentState::BelowTarget
    } else {
        match window.realized_payoff {
            Some(v) if v + 1e-12 >= REALIZED_PAYOFF_REFERENCE => {
                AssessmentState::WindowMeetsNumericReference
            }
            Some(_) => AssessmentState::BelowTarget,
            None => {
                missing_reasons.push(
                    "realized payoff is undefined without both winning and losing positions".into(),
                );
                AssessmentState::Unknown
            }
        }
    };
    missing_reasons.sort();
    missing_reasons.dedup();
    let sample_identity = serde_json::to_string(&(
        account_id,
        strategy,
        state,
        &window,
        &missing_reasons,
        unmeasured_positions,
        latest_observed_exit_at,
    ))
    .expect("finite goal evidence serializes");
    Assessment {
        account_id,
        strategy: strategy.into(),
        state,
        assessed_at,
        measured_closed_positions: finite.len(),
        unmeasured_positions,
        window,
        missing_reasons,
        latest_observed_exit_at,
        source: "broker closing deals net of swap, closing commission and conversion fee, divided by original filled-share risk; owned entry receipts and exit records qualify completeness".into(),
        history_fetched_at: None,
        history_through: None,
        history_window_from: None,
        history_max_age_secs: 0,
        history_complete: false,
        consistency: "not_established; one qualifying window is not consistency and overlapping windows are not independent".into(),
        win_rate_target: WIN_RATE_TARGET,
        realized_payoff_reference: REALIZED_PAYOFF_REFERENCE,
        planned_risk_reward: "registered SL/TP geometry unchanged; the 1:1.38 realized payoff comparison does not set a planned target".into(),
        sample_identity,
    }
}

// All source tables are existing action-owner receipts. We only detect gaps;
// we do not infer a fill from absence or create/reconcile any trading record.

async fn watermark(pool: &PgPool, account_id: i64) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT json_build_array(
          (SELECT max(id) FROM trading_intents WHERE account_id=$1),
          (SELECT max(id) FROM trading_events WHERE account_id=$1),
          (SELECT max(id) FROM strategy3_fills WHERE account_id=$1),
          (SELECT max(id) FROM strategy1_fills WHERE account_id=$1),
          (SELECT max(fetched_at) FROM deals WHERE account_id=$1),
          (SELECT count(*) FROM deals WHERE account_id=$1),
          (SELECT max(id) FROM deal_fetches WHERE account_id=$1))::text",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
}

/// The latest 20 owned, fully closed positions. Missing later exits never
/// silently disappear behind an older passing sample. Concurrent receipt or
/// history changes make this poll unknown; a later stable poll can assess it.
///
/// **Episode note (27-10-2026).** This window is per strategy and account.
/// Three demos copying one signal are one episode, not three. A claim about
/// the rule uses the signal, not the accounts. The card does not multiply
/// them. See `docs/preregistration-2026-10-27-strategy3-v3.md`.
pub async fn assess(
    pool: &PgPool,
    account_id: i64,
    strategy: &str,
) -> Result<Assessment, sqlx::Error> {
    if !matches!(strategy, "s1" | "s3") {
        return Ok(summarize(
            account_id,
            strategy,
            &[],
            0,
            vec!["unsupported strategy".into()],
            None,
        ));
    }
    let cutoff = Utc::now();
    let before = watermark(pool, account_id).await?;
    let evidence = crate::outcomes::read(pool, account_id, strategy, cutoff).await?;
    let trades = evidence.trades;
    let unmeasured = evidence.unmeasured;
    let mut reasons = evidence.reasons;
    let latest_exit = evidence.latest_exit;
    let snapshot = crate::settings::snapshot(pool, Some(account_id)).await?;
    let horizon = snapshot
        .integer("trading.reconcile_secs")
        .unwrap_or(300)
        .max(1)
        .saturating_mul(2);
    let fetch = sqlx::query("SELECT window_from,window_to,fetched_at,status,error FROM deal_fetches WHERE account_id=$1 ORDER BY window_to DESC,fetched_at DESC,id DESC LIMIT 1")
        .bind(account_id).fetch_optional(pool).await?;
    let mut history_fetched_at = None;
    let mut history_through = None;
    let mut history_window_from = None;
    let mut history_complete = false;
    if let Some(fetch) = fetch {
        let fetched: DateTime<Utc> = fetch.get("fetched_at");
        let through: DateTime<Utc> = fetch.get("window_to");
        let from: DateTime<Utc> = fetch.get("window_from");
        history_fetched_at = Some(fetched);
        history_through = Some(through);
        history_window_from = Some(from);
        // complete=false means an unsealed live window, not failed pagination.
        let ok = fetch.get::<String, _>("status") == "ok"
            && fetch.get::<Option<String>, _>("error").is_none();
        if !ok {
            reasons.push("the latest broker-history request failed".into());
        } else if fetched > cutoff || through > cutoff || from > through {
            reasons.push(
                "broker-history timestamps are after the assessment cutoff or inconsistent".into(),
            );
        } else if (cutoff - fetched).num_seconds() > horizon
            || (cutoff - through).num_seconds() > horizon
        {
            reasons.push(format!(
                "broker history is older than the existing {horizon}s liveness horizon"
            ));
        } else {
            history_complete = true;
        }
    } else {
        reasons.push("no broker-history fetch receipt is stored".into());
    }
    if watermark(pool, account_id).await? != before {
        reasons.push("account receipts or broker history changed during assessment".into());
    }
    let reference_exit = latest_exit.max(trades.iter().map(|t| t.closed_at).max());
    if let (Some(through), Some(exit)) = (history_through, reference_exit) {
        if through < exit {
            reasons.push("broker-history cutoff precedes the latest observed exit receipt".into());
            history_complete = false;
        }
    }
    let mut assessment = summarize_at(
        account_id,
        strategy,
        &trades,
        unmeasured,
        reasons,
        latest_exit,
        cutoff,
    );
    assessment.history_fetched_at = history_fetched_at;
    assessment.history_through = history_through;
    assessment.history_window_from = history_window_from;
    assessment.history_max_age_secs = horizon;
    assessment.history_complete = history_complete
        && !assessment
            .missing_reasons
            .iter()
            .any(|r| !r.starts_with("realized payoff is undefined"));
    Ok(assessment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn trade(id: i64, r: f64) -> ClosedTrade {
        ClosedTrade {
            entry_key: format!("s3:{id}"),
            position_id: id,
            closed_at: Utc.timestamp_opt(id, 0).unwrap(),
            r,
        }
    }

    #[test]
    fn latest_twenty_sort_ties_and_keep_breakeven_in_denominator() {
        let mut trades: Vec<_> = (1..=21)
            .map(|id| trade(id, if id <= 16 { 1.38 } else { -1.0 }))
            .collect();
        trades[20].r = 0.0;
        trades.reverse();
        let a = summarize(1, "s3", &trades, 0, vec![], None);
        assert_eq!(a.window.position_ids, (2..=21).collect::<Vec<_>>());
        assert_eq!(
            (a.window.wins, a.window.losses, a.window.breakeven),
            (15, 4, 1)
        );
        assert_eq!(a.window.win_rate, Some(0.75));
        assert!((a.window.realized_payoff.unwrap() - 1.38).abs() < 1e-12);
        assert_eq!(a.state, AssessmentState::WindowMeetsNumericReference);
        assert!(a.consistency.starts_with("not_established"));
        let mut tie = vec![trade(3, 1.0), trade(2, 1.0)];
        tie[0].closed_at = tie[1].closed_at;
        assert_eq!(window_metrics(&tie).position_ids, vec![2, 3]);
    }

    #[test]
    fn missing_evidence_precedes_a_passing_old_window_and_identity_is_stable() {
        let trades: Vec<_> = (1..=20)
            .map(|id| trade(id, if id <= 15 { 2.0 } else { -1.0 }))
            .collect();
        let good = summarize(1, "s3", &trades, 0, vec![], None);
        assert_eq!(good.state, AssessmentState::WindowMeetsNumericReference);
        assert_eq!(
            good.sample_identity,
            summarize(1, "s3", &trades, 0, vec![], None).sample_identity
        );
        let stale = summarize(
            1,
            "s3",
            &trades,
            0,
            vec!["new terminal exit awaits history".into()],
            None,
        );
        assert_eq!(stale.state, AssessmentState::Unknown);
        assert_ne!(good.sample_identity, stale.sample_identity);
        assert_eq!(
            summarize(1, "s3", &trades, 1, vec![], None).state,
            AssessmentState::Unknown
        );
    }

    #[test]
    fn undefined_ratios_and_small_samples_are_explicit() {
        let wins: Vec<_> = (1..=20).map(|id| trade(id, 1.0)).collect();
        let a = summarize(1, "s3", &wins, 0, vec![], None);
        assert_eq!(a.state, AssessmentState::Unknown);
        assert_eq!(a.window.realized_payoff, None);
        assert_eq!(a.window.realized_payoff_state, "no_losses");
        assert_eq!(a.window.profit_factor, None);
        assert_eq!(a.window.profit_factor_state, "infinite");
        let loss: Vec<_> = (1..=20).map(|id| trade(id, -1.0)).collect();
        let a = summarize(1, "s3", &loss, 0, vec![], None);
        assert_eq!(a.state, AssessmentState::BelowTarget);
        assert_eq!(a.window.profit_factor, Some(0.0));
        assert_eq!(
            summarize(1, "s3", &loss[..19], 0, vec![], None).state,
            AssessmentState::InsufficientEvidence
        );
    }
}
