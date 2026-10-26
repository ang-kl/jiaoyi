//! Persistent assessment in the existing service, independent of a chat turn.
//!
//! The supervisor reads trading evidence and appends reports to the existing
//! decision history. It cannot place orders, pause strategies, alter settings
//! or tune a strategy. A numeric window reference is not a consistency verdict.

use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_store::goals;
use jiaoyi_store::sqlx::{self, PgPool, Row};
use serde_json::{json, Value};

use crate::shutdown::Shutdown;

pub const POLL_SECS: u64 = 60;
pub const HEARTBEAT_SECS: i64 = 300;
pub const STALE_AFTER_SECS: i64 = HEARTBEAT_SECS + 2 * POLL_SECS as i64;
const ACCOUNT_TIMEOUT: Duration = Duration::from_secs(10);
const ERROR_REPORT_TIMEOUT: Duration = Duration::from_secs(3);
const STRATEGIES: [&str; 2] = ["s1", "s3"];

fn kind(strategy: &str) -> &'static str {
    match strategy {
        "s1" => "s1_supervisor_report",
        "s3" => "s3_supervisor_report",
        _ => "invalid_supervisor_strategy",
    }
}

fn valid_strategy(strategy: &str) -> Result<(), sqlx::Error> {
    if STRATEGIES.contains(&strategy) {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(
            "unsupported supervisor strategy".into(),
        ))
    }
}

fn basis(mut assessment: Value) -> Value {
    if let Some(a) = assessment.as_object_mut() {
        // A successful history refresh can advance its receipt clock without
        // changing the cohort or evidence verdict. Keep semantic freshness
        // (history_complete/state/missing_reasons), not those moving clocks.
        for key in [
            "assessed_at",
            "history_fetched_at",
            "history_through",
            "history_window_from",
        ] {
            a.remove(key);
        }
    }
    assessment
}

fn timestamp(value: &Value) -> Option<DateTime<Utc>> {
    value.as_str()?.parse().ok()
}

fn assessment_unavailable(assessment: &Value) -> bool {
    assessment["missing_reasons"]
        .as_array()
        .is_some_and(|reasons| reasons.iter().any(|r| r == "assessment_unavailable"))
}

/// Persist a changed assessment or a bounded heartbeat. This is not a broker
/// action; the same existing decision history survives process restarts.
pub async fn record_report(
    pool: &PgPool,
    account_id: i64,
    strategy: &str,
    assessment: Value,
    checked_at: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    valid_strategy(strategy)?;
    if assessment["account_id"].as_i64() != Some(account_id)
        || assessment["strategy"].as_str() != Some(strategy)
    {
        return Err(sqlx::Error::Protocol(
            "assessment scope does not match the report".into(),
        ));
    }
    let report_kind = kind(strategy);
    let mut tx = pool.begin().await?;
    // This report-only lock prevents old/new Railway instances from both
    // announcing the same milestone. The SELECT has a new READ COMMITTED
    // snapshot after acquiring the lock; it cannot see the pre-wait state.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("jiaoyi:supervisor-report:{account_id}:{strategy}"))
        .execute(&mut *tx)
        .await?;
    let previous: Option<Value> = sqlx::query_scalar(
        "SELECT detail FROM trading_decisions WHERE account_id = $1 AND kind = $2
         ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(report_kind)
    .fetch_optional(&mut *tx)
    .await?;
    let previous_detail = previous.as_ref();
    let previous_check = previous_detail.and_then(|d| timestamp(&d["checked_at"]));
    let wall_now = Utc::now();
    let previous_clock_is_future = previous_check.is_some_and(|last| last > wall_now);
    if !previous_clock_is_future {
        // A poll can finish computation before another poll, then wait behind
        // it for the report lock. That old result is not clock recovery. Also
        // compare the assessment cutoff so a slow old computation cannot
        // replace newer evidence merely by completing later.
        let previous_source = previous_detail
            .and_then(|d| timestamp(&d["assessment"]["assessed_at"]))
            .filter(|at| *at <= wall_now)
            .or(previous_check);
        let incoming_source = timestamp(&assessment["assessed_at"]).unwrap_or(checked_at);
        if previous_check.is_some_and(|last| checked_at < last)
            || previous_source.is_some_and(|last| incoming_source < last)
        {
            tx.commit().await?;
            return Ok(false);
        }
    }
    let current_basis = basis(assessment.clone());
    let changed = previous_detail
        .and_then(|d| d.get("assessment"))
        .is_none_or(|a| basis(a.clone()) != current_basis);
    let heartbeat_due = previous_check.is_none_or(|last| {
        let age = checked_at.signed_duration_since(last).num_seconds();
        previous_clock_is_future || age >= HEARTBEAT_SECS
    });
    if !changed && !heartbeat_due {
        tx.commit().await?;
        return Ok(false);
    }
    // This records a completed computation, even if it found unknown broker
    // evidence. Failure heartbeats must not masquerade as successful checks.
    // Use only durable evidence, including during restart/rolling deployment;
    // an old report without this field can prove its own successful check.
    let previous_success = previous_detail.and_then(|d| match d.get("last_success_at") {
        Some(at) => timestamp(at),
        None if !assessment_unavailable(&d["assessment"]) => timestamp(&d["checked_at"]),
        None => None,
    });
    let last_success_at = if !assessment_unavailable(&assessment) && checked_at <= wall_now {
        Some(checked_at)
    } else {
        previous_success.filter(|at| *at <= wall_now && *at <= checked_at)
    };
    let count = assessment["measured_closed_positions"]
        .as_u64()
        .unwrap_or(0);
    let previous_count = previous_detail
        .and_then(|d| {
            d["milestone_high_water"]
                .as_u64()
                .or_else(|| d["assessment"]["measured_closed_positions"].as_u64())
        })
        .unwrap_or(0);
    let milestones: Vec<u64> = [5, 10, 20]
        .into_iter()
        .filter(|n| count >= *n && previous_count < *n)
        .collect();
    let report_type = if changed {
        "assessment_changed"
    } else if previous_clock_is_future {
        "clock_recovery"
    } else {
        "heartbeat"
    };
    let reason = match assessment["state"].as_str() {
        Some("window_meets_numeric_reference") => {
            "latest measured window meets numeric references; consistency is not established"
        }
        Some("below_target") => "latest measured window is below the numeric references",
        Some("insufficient_evidence") => {
            "fewer than 20 measured closes; goal evidence is insufficient"
        }
        _ => "assessment is unknown or unavailable; inspect the missing evidence",
    };
    let detail = json!({
        "supervisor": "railway_assessment_supervisor",
        "strategy": strategy,
        "checked_at": checked_at,
        "last_success_at": last_success_at,
        "report_type": report_type,
        "milestones_newly_reached": milestones,
        "milestone_high_water": count.max(previous_count),
        "milestone_meaning": "measured closed positions, not economic acceptance",
        "assessment": assessment,
        "broker_actions_allowed": false,
        "automatic_pause_allowed": false,
        "risk_or_strategy_changes_allowed": false,
    });
    sqlx::query(
        "INSERT INTO trading_decisions (account_id, kind, reason, detail)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(account_id)
    .bind(report_kind)
    .bind(reason)
    .bind(detail)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// One real assessment. The run loop bounds its total time, including storage.
pub async fn poll_account(
    pool: &PgPool,
    account_id: i64,
    strategy: &str,
) -> Result<bool, sqlx::Error> {
    valid_strategy(strategy)?;
    let assessment = goals::assess(pool, account_id, strategy).await?;
    let checked_at = Utc::now();
    let value = serde_json::to_value(assessment)
        .map_err(|_| sqlx::Error::Protocol("goal assessment could not be serialized".into()))?;
    record_report(pool, account_id, strategy, value, checked_at).await
}

async fn report_unavailable(pool: &PgPool, account_id: i64, strategy: &str) {
    let now = Utc::now();
    let result = tokio::time::timeout(
        ERROR_REPORT_TIMEOUT,
        record_report(
            pool,
            account_id,
            strategy,
            json!({
                "account_id": account_id,
                "strategy": strategy,
                "state": "unknown",
                "source": "broker closing deals and original recorded risk",
                "missing_reasons": ["assessment_unavailable"],
                "consistency": "not established",
            }),
            now,
        ),
    )
    .await;
    if !matches!(result, Ok(Ok(_))) {
        tracing::warn!(
            account_id,
            strategy,
            "supervisor report could not be persisted; retrying on next poll"
        );
    }
}

/// Runs throughout market sessions and SGT sleeping hours while this service
/// is running. It neither depends on nor impersonates a continuously running LLM.
pub async fn run(pool: PgPool, mut accounts: Vec<i64>, mut stop: Shutdown) {
    accounts.sort_unstable();
    accounts.dedup();
    let mut tick = tokio::time::interval(Duration::from_secs(POLL_SECS));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = stop.wait() => return,
            _ = tick.tick() => {}
        }
        for account in &accounts {
            for strategy in STRATEGIES {
                let result = tokio::select! {
                    _ = stop.wait() => return,
                    result = tokio::time::timeout(ACCOUNT_TIMEOUT, poll_account(&pool, *account, strategy)) => result,
                };
                if !matches!(result, Ok(Ok(_))) {
                    tracing::warn!(
                        account_id = account,
                        strategy,
                        "supervisor assessment unavailable; other accounts continue"
                    );
                    tokio::select! {
                        _ = stop.wait() => return,
                        _ = report_unavailable(&pool, *account, strategy) => {}
                    }
                }
            }
        }
    }
}

/// Latest reports for the caller's allowlisted accounts only. `checked_at`
/// proves the last persisted assessment, not every intervening 60-second poll.
pub async fn reports(pool: &PgPool, accounts: &[i64]) -> Result<Value, sqlx::Error> {
    let now = Utc::now();
    let mut allowed = accounts.to_vec();
    allowed.sort_unstable();
    allowed.dedup();
    let kinds = [kind("s1"), kind("s3")];
    let rows = sqlx::query(
        "SELECT DISTINCT ON (account_id, kind) account_id, kind, at AS recorded_at, detail
         FROM trading_decisions
         WHERE account_id = ANY($1) AND kind = ANY($2)
         ORDER BY account_id, kind, id DESC",
    )
    .bind(&allowed)
    .bind(&kinds[..])
    .fetch_all(pool)
    .await?;
    let mut reports = Vec::new();
    for account in &allowed {
        for strategy in STRATEGIES {
            if let Some(row) = rows.iter().find(|row| {
                row.get::<i64, _>("account_id") == *account
                    && row.get::<String, _>("kind") == kind(strategy)
            }) {
                let detail: Value = row.get("detail");
                let checked_at = timestamp(&detail["checked_at"]);
                let age = checked_at.map(|at| now.signed_duration_since(at).num_seconds());
                let state = if checked_at.is_some_and(|at| at > now) {
                    "clock_unknown"
                } else if age.is_some_and(|age| (0..=STALE_AFTER_SECS).contains(&age)) {
                    "alive"
                } else {
                    "stale"
                };
                reports.push(json!({
                    "account_id": account,
                    "strategy": strategy,
                    "monitor_state": state,
                    "checked_at": checked_at,
                    "recorded_at": row.get::<DateTime<Utc>, _>("recorded_at"),
                    "age_secs": age,
                    "report": detail,
                }));
            } else {
                reports.push(json!({
                    "account_id": account,
                    "strategy": strategy,
                    "monitor_state": "not_checked",
                    "checked_at": null,
                    "report": null,
                }));
            }
        }
    }
    let recent_rows = sqlx::query(
        "SELECT account_id, kind, at AS recorded_at, detail FROM trading_decisions
         WHERE account_id = ANY($1) AND kind = ANY($2)
           AND detail->>'report_type' IS DISTINCT FROM 'heartbeat'
         ORDER BY id DESC LIMIT 100",
    )
    .bind(&allowed)
    .bind(&kinds[..])
    .fetch_all(pool)
    .await?;
    let recent_events: Vec<Value> = recent_rows
        .into_iter()
        .map(|row| {
            let detail: Value = row.get("detail");
            json!({
                "account_id": row.get::<i64, _>("account_id"),
                "strategy": detail["strategy"],
                "recorded_at": row.get::<DateTime<Utc>, _>("recorded_at"),
                "report": detail,
            })
        })
        .collect();
    Ok(json!({
        "service": "railway_assessment_supervisor",
        "as_of": now,
        "poll_interval_secs": POLL_SECS,
        "heartbeat_interval_secs": HEARTBEAT_SECS,
        "stale_after_secs": STALE_AFTER_SECS,
        "checked_at_meaning": "last persisted assessment; unchanged reports are heartbeated every five minutes",
        "last_success_at_meaning": "last persisted completed assessment computation, including unknown broker evidence; assessment_unavailable does not advance it; null means no known successful check",
        "monitor_state_meaning": "recency of persisted assessment, not broker-loop liveness or a goal-acceptance verdict",
        "report_delivery": "durable decision history and this read-only route; external push delivery is not configured",
        "authority": {
            "broker_actions": false,
            "automatic_pause": false,
            "risk_or_strategy_changes": false,
            "human_decisions": ["new margin floor", "Strategy 2 basket rule", "changes to risk limits, registered stop rules or account activation"],
        },
        "consistency": "no single window establishes sustained goal achievement",
        "reports": reports,
        "recent_events_limit": 100,
        "recent_events": recent_events,
    }))
}
