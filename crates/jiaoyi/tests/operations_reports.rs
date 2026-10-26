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
    assert!(html.contains("External push delivery is not configured"));
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
