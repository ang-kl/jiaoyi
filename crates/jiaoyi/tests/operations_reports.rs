use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{self as store, sqlx::PgPool};
use serde_json::{json, Value};
use tower::ServiceExt;

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn operations_reports_are_read_only_and_account_scoped(pool: PgPool) {
    let account = 49236051;
    let foreign = 46130058;
    store::upsert_account(&pool, account, Environment::Demo)
        .await
        .unwrap();
    store::upsert_account(&pool, foreign, Environment::Demo)
        .await
        .unwrap();
    let assessment = json!({"account_id":account,"strategy":"s3","state":"below_target","measured_closed_positions":20,"sample_identity":"our-window"});
    let first_check = Utc::now() - Duration::seconds(601);
    jiaoyi::supervisor::record_report(&pool, account, "s3", assessment.clone(), first_check)
        .await
        .unwrap();
    jiaoyi::supervisor::record_report(
        &pool,
        account,
        "s3",
        assessment,
        first_check + Duration::seconds(301),
    )
    .await
    .unwrap();
    jiaoyi::supervisor::record_report(
        &pool,
        foreign,
        "s3",
        json!({"account_id":foreign,"strategy":"s3","state":"below_target","measured_closed_positions":20,"sample_identity":"FOREIGN_REPORT_MUST_STAY_PRIVATE"}),
        Utc::now(),
    ).await.unwrap();
    let app = router(AppState {
        pool,
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([account])),
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/operations/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert!(!String::from_utf8_lossy(&body).contains("FOREIGN_REPORT_MUST_STAY_PRIVATE"));
    let reports = value["reports"]
        .as_array()
        .expect("explicit report rows even before first assessment");
    assert_eq!(reports.len(), 2, "one row per allowed strategy/account");
    for row in reports {
        assert_eq!(row["account_id"], account);
    }
    assert_eq!(
        reports[1]["report"]["last_success_at"],
        json!(first_check + Duration::seconds(301))
    );
    let events = value["recent_events"].as_array().unwrap();
    assert_eq!(
        events.len(),
        1,
        "the initial milestone remains visible after its heartbeat"
    );
    assert_eq!(events[0]["account_id"], account);
    assert_eq!(
        events[0]["report"]["milestones_newly_reached"],
        json!([5, 10, 20])
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/operations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(html.contains("Continuous assessment"));
    assert!(html.contains("Telegram HTML summaries use their separate schedules"));
    assert!(html.contains("this page does not imply an immediate alert was delivered"));
    assert!(html.contains("Recent milestones and assessment changes"));
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/operations/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[test]
fn operations_page_escapes_report_text() {
    let page = jiaoyi::operations::page(&json!({
        "reports":[{"account_id":1,"strategy":"s3","monitor_state":"alive",
        "report":{"assessment":{"state":"unknown","missing_reasons":["<script>untrusted</script>"]}}}]
    }));
    assert!(!page.contains("<script>untrusted</script>"));
    assert!(page.contains("&lt;script&gt;untrusted&lt;/script&gt;"));
}

#[test]
fn operations_page_reports_qualified_benchmark_gaps_and_unknown_comparisons() {
    let row = json!({"account_id":46130058,"strategy":"s3","monitor_state":"alive",
        "report":{"assessment":{"state":"below_target","history_complete":true,
        "missing_reasons":[],"assessed_at":"2026-10-28T22:47:08Z",
        "window":{"count":20,"wins":5,"win_rate":0.25,"realized_payoff":2.978904,
        "realized_payoff_state":"defined"}}}});
    let snapshot = json!({"as_of":"2026-10-28T22:49:36Z","reports":[row.clone()]});
    let page = jiaoyi::operations::page(&snapshot);
    assert!(page.contains("48.00 pp below 73%; 10 net wins below 15/20"));
    assert!(page.contains("1.598904 above or equal to 1.38"));
    assert!(page.contains("№OPS-CONTINUOUS"));
    assert!(page.contains("§1"));
    assert!(page.contains("§2"));
    assert!(page.contains("29-10-2026 06:49:36 SGT"));
    assert!(page.contains("29-10-2026 06:47:08 SGT"));
    assert!(page.contains("Assessment computed"));
    assert!(!page.contains("Assessment source cutoff"));
    assert!(page.contains("Comparable cohort deltas: unknown"));
    assert!(page.contains("Engineering-attributable financial movement: unknown"));

    // Keep the attractive raw metrics while varying the qualifying evidence.
    // Unknown/stale sources must not acquire a supported benchmark verdict.
    for (field, value) in [
        ("monitor_state", json!("stale")),
        ("state", json!("unknown")),
        ("history_complete", json!(false)),
        (
            "missing_reasons",
            json!(["history changed during assessment"]),
        ),
    ] {
        let mut invalid = row.clone();
        if field == "monitor_state" {
            invalid[field] = value;
        } else {
            invalid["report"]["assessment"][field] = value;
        }
        let page = jiaoyi::operations::page(&json!({"reports":[invalid]}));
        assert!(!page.contains("48.00 pp below"), "{field}");
        assert!(!page.contains("1.598904 above"), "{field}");
        assert!(
            page.contains("unknown (incomplete or stale evidence)"),
            "{field}"
        );
        assert!(page.contains("not recorded"));
        assert!(page.contains("№OPS-CONTINUOUS"));
    }
    let mut partial = row;
    partial["report"]["assessment"]["state"] = json!("insufficient_evidence");
    partial["report"]["assessment"]["window"]["count"] = json!(19);
    let page = jiaoyi::operations::page(&json!({"reports":[partial]}));
    assert!(page.contains("1 measured closes short of 20; win gap unknown"));
    assert!(page.contains("unknown (insufficient evidence)"));
    assert!(!page.contains("1.598904 above"));
}

#[test]
fn operations_page_shows_last_success_separately_from_failure_time() {
    let page = jiaoyi::operations::page(&json!({
        "reports":[{"account_id":1,"strategy":"s3","monitor_state":"alive",
        "checked_at":"2026-10-26T17:05:00Z",
        "report":{"last_success_at":"2026-10-26T17:00:00Z",
        "assessment":{"state":"unknown","missing_reasons":["assessment_unavailable"]}}},
        {"account_id":2,"strategy":"s3","report":{"last_success_at":null}}]
    }));
    assert!(page.contains("Last completed assessment"));
    assert!(page.contains("27-10-2026 01:05:00 SGT</td><td>27-10-2026 01:00:00 SGT"));
    assert!(page.contains("assessment_unavailable"));
    assert!(page.contains("not recorded"));
    assert!(page.contains("a completed computation can still report unknown broker evidence"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn source_diagnostics_expose_only_typed_whitelist_for_allowed_accounts(pool: PgPool) {
    let account = 47790949;
    let foreign = 46130058;
    let legacy = 49236051;
    let absent = 43002148;
    let at = chrono::DateTime::from_timestamp_micros(Utc::now().timestamp_micros()).unwrap();
    for id in [account, foreign, legacy, absent] {
        store::upsert_account(&pool, id, Environment::Demo)
            .await
            .unwrap();
    }
    let counts = json!({"missing_pnl_position_count":0,"extra_pnl_position_count":1,
        "pending_zero_pnl_position_count":0,"unresolved_extra_pnl_position_count":1,
        "duplicate_position_id_count":0,"duplicate_pnl_id_count":0,"non_open_position_count":0});
    let relationships = json!({"pending_entry_position_match_count":1,
        "pending_protection_position_match_count":1,"pending_order_missing_position_ref_count":1,
        "opening_deal_observed_count":1,"closing_deal_observed_count":1,"owned_fill_observed_count":1,
        "evidence_state":"stored_observations_only_closure_unknown"});
    let mut private = json!({"account_id":foreign,"observed_at":"PRIVATE_TIMESTAMP",
        "book_stable":true,"balance_stable":true,"pnl_coverage_complete":false,"source_complete":false,
        "failure_reasons":["pnl_position_coverage_incomplete"],"pnl_coverage_diagnostics":counts,
        "extra_pnl_relationship_diagnostics":relationships,
        "balance":1234567.89,"float":98765.43,"equity":999999.99,"price":54321.12,
        "notes":["PRIVATE_NOTES"],"raw_payload":{"secret":"PRIVATE_PAYLOAD"}});
    private["pnl_coverage_diagnostics"]["unapproved"] = json!("PRIVATE_NESTED");
    private["extra_pnl_relationship_diagnostics"]["unapproved"] =
        json!({"position_id":"PRIVATE_POSITION_ID","amount":"PRIVATE_RELATION_AMOUNT"});
    for (id, detail) in [
        (account, private),
        (foreign, json!({"notes":["FOREIGN_SNAPSHOT"]})),
        (
            legacy,
            json!({"book_stable":"PRIVATE_STRING","balance_stable":null,
            "failure_reasons":["book_changed","PRIVATE_REASON"],
            "pnl_coverage_diagnostics":{"missing_pnl_position_count":-1,"extra_pnl_position_count":"PRIVATE_COUNT"},
            "extra_pnl_relationship_diagnostics":{"pending_entry_position_match_count":-1,
                "pending_protection_position_match_count":"PRIVATE_COUNT",
                "pending_order_missing_position_ref_count":1.5,"opening_deal_observed_count":true,
                "closing_deal_observed_count":{"value":1},"owned_fill_observed_count":[1],
                "evidence_state":"PRIVATE_RELATION_REASON"}}),
        ),
    ] {
        store::sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES($1,'latest',$2,$3)")
            .bind(id).bind(at).bind(detail).execute(&pool).await.unwrap();
    }
    let app = router(AppState {
        pool,
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([account, legacy, absent])),
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/operations/reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = std::str::from_utf8(&body).unwrap();
    let forbidden_values = [
        "PRIVATE_",
        "FOREIGN_SNAPSHOT",
        "1234567.89",
        "98765.43",
        "999999.99",
        "54321.12",
    ];
    for forbidden in forbidden_values {
        assert!(!text.contains(forbidden), "leaked {forbidden}");
    }
    let value: Value = serde_json::from_slice(&body).unwrap();
    let rows = value["account_source_diagnostics"]
        .as_array()
        .expect("curated source evidence");
    assert_eq!(rows.len(), 3);
    let row = rows.iter().find(|r| r["account_id"] == account).unwrap();
    assert_eq!(
        row,
        &json!({"account_id":account,"observed_at":at,
        "book_stable":true,"balance_stable":true,"pnl_coverage_complete":false,"source_complete":false,
        "failure_reasons":["pnl_position_coverage_incomplete"],"pnl_coverage_diagnostics":counts,
        "extra_pnl_relationship_diagnostics":relationships,
        "unmatched_excluded_from_owned_window":false})
    );
    let unknown_counts = json!({"missing_pnl_position_count":null,"extra_pnl_position_count":null,
        "pending_zero_pnl_position_count":null,"unresolved_extra_pnl_position_count":null,
        "duplicate_position_id_count":null,"duplicate_pnl_id_count":null,"non_open_position_count":null});
    let unknown_relationships = json!({"pending_entry_position_match_count":null,
        "pending_protection_position_match_count":null,"pending_order_missing_position_ref_count":null,
        "opening_deal_observed_count":null,"closing_deal_observed_count":null,"owned_fill_observed_count":null,
        "evidence_state":null});
    for id in [legacy, absent] {
        assert_eq!(
            rows.iter().find(|r| r["account_id"] == id).unwrap(),
            &json!({"account_id":id,"observed_at":if id==legacy {Some(at)} else {None},
                "book_stable":null,"balance_stable":null,"pnl_coverage_complete":null,"source_complete":null,
                "failure_reasons":null,"pnl_coverage_diagnostics":unknown_counts,
                "extra_pnl_relationship_diagnostics":unknown_relationships,
                "unmatched_excluded_from_owned_window":false})
        );
    }
    assert!(rows.iter().all(|r| r["account_id"] != foreign));

    // Health embeds the same projection, so it must preserve the exact whitelist
    // and account scope even when the saved snapshot contains private values.
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = std::str::from_utf8(&body).unwrap();
    for forbidden in forbidden_values {
        assert!(!text.contains(forbidden), "health leaked {forbidden}");
    }
    let health: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        health["supervision"]["account_source_diagnostics"],
        value["account_source_diagnostics"]
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn margin_admission_view_is_typed_account_scoped_and_distinguishes_unconfigured(
    pool: PgPool,
) {
    let account = 49236051;
    let foreign = 46130058;
    let absent = 43002148;
    for id in [account, foreign, absent] {
        store::upsert_account(&pool, id, Environment::Demo)
            .await
            .unwrap();
    }
    store::settings::change(
        &pool,
        "trading.margin_level_floor_pct",
        Some(account),
        Some(json!(250)),
        "test",
        "synthetic owner policy",
    )
    .await
    .unwrap();
    let detail = json!({"state":"passed","projected_level_pct":300.0,"stressed_level_pct":200.0,
        "warning_pct":150.0,"positions":0,"pending_entries":1,
        "balance":1234567.89,"reason":"PRIVATE_MARGIN_REASON","secret":"PRIVATE_MARGIN_SECRET"});
    store::trading::record_decision(
        &pool,
        account,
        None,
        "margin_admission",
        "PRIVATE_REASON",
        &detail,
        None,
    )
    .await
    .unwrap();
    store::trading::record_decision(
        &pool,
        foreign,
        None,
        "margin_admission",
        "FOREIGN_REASON",
        &detail,
        None,
    )
    .await
    .unwrap();
    let before: i64 = store::sqlx::query_scalar("SELECT count(*) FROM trading_decisions")
        .fetch_one(&pool)
        .await
        .unwrap();
    let report = jiaoyi::supervisor::reports(&pool, &[account, absent])
        .await
        .unwrap();
    let rows = report["margin_admission"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let own = rows.iter().find(|r| r["account_id"] == account).unwrap();
    assert_eq!(own["configured"], true);
    assert_eq!(own["floor_pct"], 250.0);
    assert_eq!(own["last_state"], "passed");
    assert_eq!(own["pending_entries"], 1);
    let unknown = rows.iter().find(|r| r["account_id"] == absent).unwrap();
    assert_eq!(unknown["configured"], false);
    assert!(unknown["floor_pct"].is_null());
    assert!(unknown["last_checked_at"].is_null());
    for forbidden in ["PRIVATE_", "1234567.89", "FOREIGN_REASON"] {
        assert!(!report.to_string().contains(forbidden));
    }
    assert_eq!(
        store::sqlx::query_scalar::<_, i64>("SELECT count(*) FROM trading_decisions")
            .fetch_one(&pool)
            .await
            .unwrap(),
        before
    );
}
