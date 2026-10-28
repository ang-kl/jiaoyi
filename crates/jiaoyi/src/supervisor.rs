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
    // Compare the stored and incoming JSONB directly: decoding a stored JSON
    // number to f64 can shift its last bit and falsely announce a changed
    // assessment on every poll. JSONB compares exact numeric values without
    // rounding away real metric changes. Only moving receipt clocks are
    // excluded; freshness verdicts, cohort identity and missing evidence stay.
    let previous = sqlx::query(
        "SELECT detail,
           ((detail->'assessment') - ARRAY['assessed_at', 'history_fetched_at',
              'history_through', 'history_window_from'])
           IS DISTINCT FROM
           ($3::jsonb - ARRAY['assessed_at', 'history_fetched_at',
              'history_through', 'history_window_from']) AS assessment_changed
         FROM trading_decisions WHERE account_id = $1 AND kind = $2
         ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(report_kind)
    .bind(&assessment)
    .fetch_optional(&mut *tx)
    .await?;
    let previous_detail = previous.as_ref().map(|row| row.get::<Value, _>("detail"));
    let previous_detail = previous_detail.as_ref();
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
    let changed = previous
        .as_ref()
        .is_none_or(|row| row.get::<bool, _>("assessment_changed"));
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

/// A strictly typed projection of private source evidence. Legacy/malformed
/// fields stay unknown; saved JSON, notes and monetary values never cross here.
fn source_diagnostic(account: i64, observed_at: Option<DateTime<Utc>>, detail: &Value) -> Value {
    let reasons = detail["failure_reasons"].as_array().and_then(|values| {
        values
            .iter()
            .map(|value| {
                value.as_str().filter(|reason| {
                    matches!(
                        *reason,
                        "book_changed" | "balance_changed" | "pnl_position_coverage_incomplete"
                    )
                })
            })
            .collect::<Option<Vec<_>>>()
    });
    let mut result = json!({"account_id":account,"observed_at":observed_at,
        "failure_reasons":reasons,"pnl_coverage_diagnostics":{}});
    for field in [
        "book_stable",
        "balance_stable",
        "pnl_coverage_complete",
        "source_complete",
    ] {
        result[field] = json!(detail[field].as_bool());
    }
    for field in [
        "missing_pnl_position_count",
        "extra_pnl_position_count",
        "duplicate_position_id_count",
        "duplicate_pnl_id_count",
        "non_open_position_count",
    ] {
        result["pnl_coverage_diagnostics"][field] =
            json!(detail["pnl_coverage_diagnostics"][field].as_u64());
    }
    let relationships = &detail["extra_pnl_relationship_diagnostics"];
    let mut curated = json!({"evidence_state":relationships["evidence_state"].as_str().filter(|s|
        matches!(*s, "stored_observations_only_closure_unknown" | "not_applicable" |
            "response_account_mismatch" | "invalid_position_id" | "store_read_failed"))});
    for field in [
        "pending_entry_position_match_count",
        "pending_protection_position_match_count",
        "pending_order_missing_position_ref_count",
        "opening_deal_observed_count",
        "closing_deal_observed_count",
        "owned_fill_observed_count",
    ] {
        curated[field] = json!(relationships[field].as_u64());
    }
    result["extra_pnl_relationship_diagnostics"] = curated;
    // Exclusion describes attribution only. An extra row can still match an
    // owned fill or pending order; its mere presence proves no exclusion and
    // never repairs account-wide P&L coverage.
    let extra = result["pnl_coverage_diagnostics"]["extra_pnl_position_count"].as_u64();
    let no_owned_link = [
        "pending_entry_position_match_count",
        "pending_protection_position_match_count",
        "pending_order_missing_position_ref_count",
        "owned_fill_observed_count",
    ]
    .iter()
    .all(|field| relationships[*field].as_u64() == Some(0));
    result["unmatched_excluded_from_owned_window"] = json!(
        extra.is_some_and(|n| n > 0)
            && no_owned_link
            && relationships["evidence_state"] == "stored_observations_only_closure_unknown"
    );
    result
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
    let snapshots = sqlx::query(
        "SELECT account_id, observed_at, detail FROM report_account_snapshots
         WHERE account_id = ANY($1) AND sample_key = 'latest'",
    )
    .bind(&allowed)
    .fetch_all(pool)
    .await?;
    let account_source_diagnostics: Vec<Value> = allowed
        .iter()
        .map(|account| {
            if let Some(row) = snapshots
                .iter()
                .find(|row| row.get::<i64, _>("account_id") == *account)
            {
                source_diagnostic(
                    *account,
                    Some(row.get("observed_at")),
                    &row.get::<Value, _>("detail"),
                )
            } else {
                source_diagnostic(*account, None, &Value::Null)
            }
        })
        .collect();
    let policy_rows = sqlx::query("SELECT account_id,value FROM settings WHERE key='trading.margin_level_floor_pct' AND (account_id IS NULL OR account_id=ANY($1))")
        .bind(&allowed).fetch_all(pool).await?;
    let margin_rows = sqlx::query("SELECT DISTINCT ON (account_id) account_id,at,detail FROM trading_decisions WHERE account_id=ANY($1) AND kind='margin_admission' ORDER BY account_id,id DESC")
        .bind(&allowed).fetch_all(pool).await?;
    let margin_admission: Vec<Value> = allowed.iter().map(|account| {
        let configured=policy_rows.iter().find(|r|r.get::<Option<i64>,_>("account_id")==Some(*account))
            .or_else(||policy_rows.iter().find(|r|r.get::<Option<i64>,_>("account_id").is_none()))
            .and_then(|r|r.get::<Value,_>("value").as_f64()).filter(|v|v.is_finite()&&*v>=100.0);
        let row=margin_rows.iter().find(|r|r.get::<i64,_>("account_id")==*account);
        let detail=row.map(|r|r.get::<Value,_>("detail")).unwrap_or(Value::Null);
        let number=|field:&str|detail[field].as_f64().filter(|v|v.is_finite());
        json!({"account_id":account,"floor_pct":configured,"configured":configured.is_some(),
            "scope":"new entries only; existing exits bypass margin admission; live one-position rule retained",
            "last_checked_at":row.map(|r|r.get::<DateTime<Utc>,_>("at")),
            "last_state":detail["state"].as_str().filter(|s|matches!(*s,"passed"|"unavailable"|"refused")),
            "projected_level_pct":number("projected_level_pct"),"stressed_level_pct":number("stressed_level_pct"),
            "warning_pct":number("warning_pct"),"positions":detail["positions"].as_u64(),
            "pending_entries":detail["pending_entries"].as_u64(),
            "evidence_scope":"last candidate assessment, not a current balance or margin guarantee"})
    }).collect();
    Ok(json!({
        "service": "railway_assessment_supervisor",
        "as_of": now,
        "poll_interval_secs": POLL_SECS,
        "heartbeat_interval_secs": HEARTBEAT_SECS,
        "stale_after_secs": STALE_AFTER_SECS,
        "checked_at_meaning": "last persisted assessment; unchanged reports are heartbeated every five minutes",
        "last_success_at_meaning": "last persisted completed assessment computation, including unknown broker evidence; assessment_unavailable does not advance it; null means no known successful check",
        "monitor_state_meaning": "recency of persisted assessment, not broker-loop liveness or a goal-acceptance verdict",
        "report_delivery": "durable decision history and this read-only route; owner Telegram HTML summaries use separate schedules and receipts",
        "authority": {
            "broker_actions": false,
            "automatic_pause": false,
            "risk_or_strategy_changes": false,
            "human_decisions": ["new margin floor", "Strategy 2 basket rule", "changes to risk limits, registered stop rules or account activation"],
        },
        "consistency": "no single window establishes sustained goal achievement",
        "reports": reports,
        "account_source_diagnostics": account_source_diagnostics,
        "margin_admission": margin_admission,
        "recent_events_limit": 100,
        "recent_events": recent_events,
    }))
}

#[cfg(test)]
mod exclusion_tests {
    use super::*;

    #[test]
    fn exclusion_requires_known_absence_of_owned_and_pending_links() {
        let mut detail = json!({"pnl_coverage_complete":false,"source_complete":false,
            "pnl_coverage_diagnostics":{"extra_pnl_position_count":1},
            "extra_pnl_relationship_diagnostics":{
                "evidence_state":"stored_observations_only_closure_unknown",
                "pending_entry_position_match_count":0,"pending_protection_position_match_count":0,
                "pending_order_missing_position_ref_count":0,"owned_fill_observed_count":0}});
        let view = source_diagnostic(47790949, None, &detail);
        assert_eq!(view["unmatched_excluded_from_owned_window"], true);
        assert_eq!(view["pnl_coverage_complete"], false);
        assert_eq!(view["source_complete"], false);
        for field in [
            "pending_entry_position_match_count",
            "pending_protection_position_match_count",
            "pending_order_missing_position_ref_count",
            "owned_fill_observed_count",
        ] {
            for value in [json!(1), Value::Null, json!(-1)] {
                let mut linked = detail.clone();
                linked["extra_pnl_relationship_diagnostics"][field] = value;
                assert_eq!(
                    source_diagnostic(47790949, None, &linked)
                        ["unmatched_excluded_from_owned_window"],
                    false
                );
            }
        }
        detail["extra_pnl_relationship_diagnostics"]["evidence_state"] = json!("store_read_failed");
        assert_eq!(
            source_diagnostic(47790949, None, &detail)["unmatched_excluded_from_owned_window"],
            false
        );
    }
}
