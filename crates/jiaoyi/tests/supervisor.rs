//! Durable milestone/report behaviour uses real PostgreSQL; no broker is run.

use chrono::{Duration, Utc};
use jiaoyi::{shutdown, supervisor};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{self as store, sqlx, trading as ts};
use serde_json::{json, Value};
use sqlx::PgPool;

const A: i64 = 49236051;
const B: i64 = 46130058;

fn assessment(account: i64, strategy: &str, count: usize) -> Value {
    json!({
        "account_id": account,
        "strategy": strategy,
        "assessed_at": Utc::now(),
        "state": if count < 20 { "insufficient_evidence" } else { "below_target" },
        "measured_closed_positions": count,
        "window": {"count": count.min(20), "position_ids": [1,2,3]},
        "missing_reasons": [],
    })
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn supervisor_reports_survive_restart_and_heartbeat_without_poll_spam(pool: PgPool) {
    // Exercise elapsed logical time without creating future durable clocks.
    let now = Utc::now() - Duration::minutes(10);
    assert!(
        supervisor::record_report(&pool, A, "s3", assessment(A, "s3", 5), now)
            .await
            .unwrap()
    );
    // A new assessed_at clock value alone is not changed goal evidence.
    assert!(!supervisor::record_report(
        &pool,
        A,
        "s3",
        assessment(A, "s3", 5),
        now + Duration::seconds(60)
    )
    .await
    .unwrap());
    assert!(supervisor::record_report(
        &pool,
        A,
        "s3",
        assessment(A, "s3", 5),
        now + Duration::seconds(supervisor::HEARTBEAT_SECS)
    )
    .await
    .unwrap());
    let rows: Vec<Value> = sqlx::query_scalar("SELECT detail FROM trading_decisions ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["milestones_newly_reached"], json!([5]));
    assert_eq!(rows[1]["report_type"], "heartbeat");
    assert_eq!(rows[1]["milestones_newly_reached"], json!([]));
    // No process memory is consulted: a fresh caller sees the last durable
    // report and does not repeat the milestone after a restart.
    assert!(!supervisor::record_report(
        &pool,
        A,
        "s3",
        assessment(A, "s3", 5),
        now + Duration::seconds(301)
    )
    .await
    .unwrap());
    assert!(supervisor::record_report(
        &pool,
        A,
        "s3",
        assessment(A, "s3", 20),
        now + Duration::seconds(302)
    )
    .await
    .unwrap());
    let (_, latest) = ts::latest_limit_decision(&pool, A, &["s3_supervisor_report"])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest["milestones_newly_reached"], json!([10, 20]));
    assert_eq!(latest["automatic_pause_allowed"], false);
    assert_eq!(latest["broker_actions_allowed"], false);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unavailable_evidence_and_recovery_do_not_reannounce_old_milestones(pool: PgPool) {
    let now = Utc::now();
    supervisor::record_report(&pool, A, "s3", assessment(A, "s3", 20), now)
        .await
        .unwrap();
    let unavailable = json!({"account_id": A, "strategy":"s3", "state":"unknown", "missing_reasons":["assessment_unavailable"]});
    supervisor::record_report(&pool, A, "s3", unavailable, now + Duration::seconds(1))
        .await
        .unwrap();
    supervisor::record_report(
        &pool,
        A,
        "s3",
        assessment(A, "s3", 20),
        now + Duration::seconds(2),
    )
    .await
    .unwrap();
    let (_, latest) = ts::latest_limit_decision(&pool, A, &["s3_supervisor_report"])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest["milestones_newly_reached"], json!([]));
    assert_eq!(latest["milestone_high_water"], 20);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn last_success_survives_failures_heartbeats_and_legacy_reports(pool: PgPool) {
    let start = Utc::now() - Duration::minutes(20);
    let unavailable = |account| {
        json!({
            "account_id": account, "strategy":"s3", "state":"unknown",
            "missing_reasons":["assessment_unavailable"]
        })
    };
    // A completed computation can report unknown financial evidence. Its
    // success clock says nothing about goal acceptance or broker freshness.
    let mut completed = assessment(A, "s3", 20);
    completed["assessed_at"] = json!(start);
    completed["state"] = json!("unknown");
    completed["missing_reasons"] = json!(["no broker-history fetch receipt is stored"]);
    supervisor::record_report(&pool, A, "s3", completed.clone(), start)
        .await
        .unwrap();
    let report = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(
        report["reports"][1]["report"]["last_success_at"],
        json!(start)
    );

    // A rolling deployment can inherit a successful report from the old
    // version, which did not store the new field.
    sqlx::query("INSERT INTO trading_decisions(account_id,kind,reason,detail) SELECT account_id,kind,reason,detail-'last_success_at' FROM trading_decisions WHERE account_id=$1 ORDER BY id DESC LIMIT 1")
        .bind(A).execute(&pool).await.unwrap();
    for seconds in [1, 301] {
        assert!(supervisor::record_report(
            &pool,
            A,
            "s3",
            unavailable(A),
            start + Duration::seconds(seconds)
        )
        .await
        .unwrap());
        let report = supervisor::reports(&pool, &[A]).await.unwrap();
        let detail = &report["reports"][1]["report"];
        assert_eq!(detail["last_success_at"], json!(start));
        assert_eq!(detail["milestones_newly_reached"], json!([]));
    }
    let recovered_at = start + Duration::seconds(302);
    completed["assessed_at"] = json!(recovered_at);
    supervisor::record_report(&pool, A, "s3", completed, recovered_at)
        .await
        .unwrap();
    let report = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(
        report["reports"][1]["report"]["last_success_at"],
        json!(recovered_at)
    );

    // Failure on first observation, including a legacy failure heartbeat,
    // has no known successful computation. A fresh caller must preserve null.
    supervisor::record_report(&pool, B, "s3", unavailable(B), start)
        .await
        .unwrap();
    sqlx::query("INSERT INTO trading_decisions(account_id,kind,reason,detail) SELECT account_id,kind,reason,detail-'last_success_at' FROM trading_decisions WHERE account_id=$1 ORDER BY id DESC LIMIT 1")
        .bind(B).execute(&pool).await.unwrap();
    supervisor::record_report(
        &pool,
        B,
        "s3",
        unavailable(B),
        start + Duration::seconds(300),
    )
    .await
    .unwrap();
    let report = supervisor::reports(&pool, &[B]).await.unwrap();
    assert_eq!(
        report["reports"][1]["report"]["last_success_at"],
        Value::Null
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unavailable_report_cannot_inherit_a_future_legacy_success_clock(pool: PgPool) {
    let now = Utc::now() - Duration::minutes(1);
    let mut completed = assessment(A, "s3", 20);
    completed["assessed_at"] = json!(now);
    supervisor::record_report(&pool, A, "s3", completed.clone(), now + Duration::hours(1))
        .await
        .unwrap();
    sqlx::query("INSERT INTO trading_decisions(account_id,kind,reason,detail) SELECT account_id,kind,reason,detail-'last_success_at' FROM trading_decisions WHERE account_id=$1 ORDER BY id DESC LIMIT 1")
        .bind(A).execute(&pool).await.unwrap();
    let unavailable = json!({"account_id":A,"strategy":"s3","state":"unknown","missing_reasons":["assessment_unavailable"]});
    supervisor::record_report(&pool, A, "s3", unavailable, now)
        .await
        .unwrap();
    let report = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(
        report["reports"][1]["report"]["last_success_at"],
        Value::Null
    );
    let recovered_at = now + Duration::seconds(1);
    completed["assessed_at"] = json!(recovered_at);
    supervisor::record_report(&pool, A, "s3", completed, recovered_at)
        .await
        .unwrap();
    let report = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(
        report["reports"][1]["report"]["last_success_at"],
        json!(recovered_at)
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn delayed_old_poll_cannot_replace_a_newer_failing_window(pool: PgPool) {
    let now = Utc::now();
    let mut newer = assessment(A, "s3", 21);
    newer["assessed_at"] = json!(now - Duration::seconds(2));
    newer["window"]["position_ids"] = json!([2, 3, 4]);
    supervisor::record_report(&pool, A, "s3", newer, now - Duration::seconds(1))
        .await
        .unwrap();
    let mut older = assessment(A, "s3", 20);
    older["assessed_at"] = json!(now - Duration::seconds(4));
    older["state"] = json!("window_meets_numeric_reference");
    assert!(
        !supervisor::record_report(&pool, A, "s3", older.clone(), now - Duration::seconds(3))
            .await
            .unwrap(),
        "an older completed poll is not clock recovery"
    );
    assert!(
        !supervisor::record_report(&pool, A, "s3", older.clone(), now)
            .await
            .unwrap(),
        "a late-completing old assessment cannot replace a newer source cutoff"
    );
    // A future source clock is not permission to overwrite a valid newer
    // durable report with an older completed poll.
    older["assessed_at"] = json!(now + Duration::hours(1));
    assert!(
        !supervisor::record_report(&pool, A, "s3", older, now - Duration::seconds(3))
            .await
            .unwrap()
    );
    let (_, latest) = ts::latest_limit_decision(&pool, A, &["s3_supervisor_report"])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest["assessment"]["state"], "below_target");
    assert_eq!(
        latest["assessment"]["window"]["position_ids"],
        json!([2, 3, 4])
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn fresh_poll_repairs_future_report_clock_without_waiting_for_wall_time(pool: PgPool) {
    let now = Utc::now();
    let mut a = assessment(A, "s3", 20);
    a["history_fetched_at"] = json!(now);
    a["history_through"] = json!(now);
    a["history_complete"] = json!(true);
    supervisor::record_report(&pool, A, "s3", a.clone(), now + Duration::hours(1))
        .await
        .unwrap();
    let before = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(before["reports"][1]["monitor_state"], "clock_unknown");
    assert_eq!(
        before["reports"][1]["report"]["last_success_at"],
        Value::Null
    );
    assert!(supervisor::record_report(&pool, A, "s3", a.clone(), now)
        .await
        .unwrap());
    let after = supervisor::reports(&pool, &[A]).await.unwrap();
    assert_eq!(after["reports"][1]["monitor_state"], "alive");
    assert_eq!(after["reports"][1]["report"]["last_success_at"], json!(now));
    assert_eq!(
        after["reports"][1]["report"]["report_type"],
        "clock_recovery"
    );
    assert_eq!(
        after["reports"][1]["report"]["milestones_newly_reached"],
        json!([])
    );
    a["history_fetched_at"] = json!(now + Duration::seconds(60));
    a["history_through"] = json!(now + Duration::seconds(60));
    assert!(
        !supervisor::record_report(&pool, A, "s3", a.clone(), now + Duration::seconds(60))
            .await
            .unwrap(),
        "only receipt time advanced"
    );
    a["history_complete"] = json!(false);
    a["state"] = json!("unknown");
    a["missing_reasons"] = json!(["history_refresh_stale"]);
    assert!(
        supervisor::record_report(&pool, A, "s3", a, now + Duration::seconds(61))
            .await
            .unwrap(),
        "a freshness verdict change is durable immediately"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn reports_are_allowlisted_partitioned_and_stale_reports_cannot_look_alive(pool: PgPool) {
    let now = Utc::now();
    supervisor::record_report(
        &pool,
        A,
        "s1",
        assessment(A, "s1", 5),
        now - Duration::seconds(supervisor::STALE_AFTER_SECS + 1),
    )
    .await
    .unwrap();
    supervisor::record_report(&pool, B, "s3", assessment(B, "s3", 20), now)
        .await
        .unwrap();
    assert!(
        supervisor::record_report(&pool, A, "s1", assessment(B, "s1", 20), now)
            .await
            .is_err()
    );
    assert!(supervisor::poll_account(&pool, A, "s2").await.is_err());
    let report = supervisor::reports(&pool, &[A, A]).await.unwrap();
    let rows = report["reports"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        2,
        "one row per allowed strategy/account, no duplicate account"
    );
    assert!(rows.iter().all(|r| r["account_id"] == A));
    assert_eq!(rows[0]["strategy"], "s1");
    assert_eq!(rows[0]["monitor_state"], "stale");
    assert_eq!(rows[1]["strategy"], "s3");
    assert_eq!(rows[1]["monitor_state"], "not_checked");
    assert_eq!(report["authority"]["risk_or_strategy_changes"], false);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn overlapping_instances_write_one_changed_milestone(pool: PgPool) {
    // Hold the INSERT briefly after each caller has had time to read the old
    // report. This exposes a rolling old/new-process read-then-write race.
    sqlx::query("CREATE FUNCTION delay_supervisor_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN PERFORM pg_sleep(0.05); RETURN NEW; END $$")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER delay_supervisor BEFORE INSERT ON trading_decisions FOR EACH ROW EXECUTE FUNCTION delay_supervisor_insert()")
        .execute(&pool).await.unwrap();
    let now = Utc::now();
    let (a, b) = tokio::join!(
        supervisor::record_report(&pool, A, "s3", assessment(A, "s3", 20), now),
        supervisor::record_report(&pool, A, "s3", assessment(A, "s3", 20), now),
    );
    assert_eq!(
        usize::from(a.unwrap()) + usize::from(b.unwrap()),
        1,
        "one changed assessment across overlapping processes"
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trading_decisions WHERE kind = 's3_supervisor_report'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn runtime_assesses_without_a_chat_and_stops_without_trading(pool: PgPool) {
    store::upsert_account(&pool, A, Environment::Demo)
        .await
        .unwrap();
    let (trigger, stop) = shutdown::channel();
    let task = tokio::spawn(supervisor::run(pool.clone(), vec![A], stop));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let report = supervisor::reports(&pool, &[A]).await.unwrap();
            if report["reports"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["monitor_state"] == "alive")
            {
                for row in report["reports"].as_array().unwrap() {
                    assert_eq!(row["report"]["assessment"]["state"], "unknown");
                    assert!(
                        row["report"]["assessment"]["missing_reasons"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|r| r == "no broker-history fetch receipt is stored"),
                        "a real missing-history assessment, not a generic polling failure"
                    );
                }
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("runtime did not persist its first real assessments");
    trigger.fire();
    tokio::time::timeout(std::time::Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap();
    let intents: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_intents")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(intents, 0, "the assessment service sends no order");
}
