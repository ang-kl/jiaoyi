//! `/`: the read-only status page (R-UI0). The owner reported the service's
//! root URL "cannot be found" (it answered 404). It now answers with plain
//! HTML from the binary: the service, the accounts' grant states and last
//! loads, the tick recorders, the research jobs and links to the JSON
//! routes. No secret and no money beyond what `/research/trials` shows.

use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi::recorder::RecorderStatus;
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store as store;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

const LIVE: i64 = 43002148;

fn app(pool: &PgPool, recorders: jiaoyi::recorder::StatusMap) -> axum::Router {
    router(AppState {
        pool: pool.clone(),
        loading: Arc::new(AtomicBool::new(false)),
        recorders,
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([LIVE])),
    })
}

async fn call(app: axum::Router, method: Method, uri: &str) -> (StatusCode, String, String) {
    let res = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let ctype = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, ctype, String::from_utf8(body.to_vec()).unwrap())
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_root_shows_the_status_page(pool: PgPool) {
    store::upsert_account(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    store::record_grant(
        &pool,
        &store::GrantRead {
            account_id: LIVE,
            host: "live.ctraderapi.com".into(),
            token_fingerprint: "0123456789abcdef".into(),
            token_source: "env_seed".into(),
            verdict: "listed",
            reason: None,
            permission_scope: Some("SCOPE_VIEW".into()),
            accounts: Some(json!([{"ctid": LIVE, "is_live": true}])),
        },
    )
    .await
    .unwrap();
    let run = store::start_run(&pool, LIVE).await.unwrap();
    store::finish_run(
        &pool,
        run,
        "partial",
        &store::RunSummary {
            errors: vec!["asset list: <b>refused</b>".into()],
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let recorders = jiaoyi::recorder::StatusMap::default();
    recorders.lock().unwrap().insert(
        LIVE,
        RecorderStatus {
            subscribed_symbols: vec![1, 41],
            sessions_started: 1,
            ticks_written: 4321,
            ..Default::default()
        },
    );

    let (st, ctype, body) = call(app(&pool, recorders.clone()), Method::GET, "/").await;
    assert_eq!(st, StatusCode::OK);
    assert!(ctype.starts_with("text/html"), "{ctype}");
    for want in [
        "<title>jiaoyi status</title>",
        "Settings version",
        ">43002148<",
        ">listed<",
        ">partial<",
        "asset list: &lt;b&gt;refused&lt;/b&gt;",
        ">4321<",
        "Research jobs",
        r#"href="/health""#,
        r#"href="/research/jobs""#,
        r#"href="/research/trials""#,
    ] {
        assert!(body.contains(want), "missing {want:?}");
    }
    // The engine guard, read from the running binary (R1b).
    let engine = format!(
        "Engine version {} running; the held-out job waits for engine {} or later.",
        jiaoyi::edge::ENGINE_VERSION,
        jiaoyi::edge::HELD_OUT_MIN_ENGINE
    );
    assert!(body.contains(&engine), "missing {engine:?}");
    // The ten pre-registered jobs, each with its status.
    assert_eq!(body.matches("breakout ").count(), 10, "one row per job");
    assert_eq!(
        body.matches(r#"<span class="pill warn">pending</span>"#)
            .count(),
        10
    );
    // Nothing secret: no token fingerprint, scope or broker host.
    for hidden in ["0123456789abcdef", "ctraderapi", "SCOPE_VIEW"] {
        assert!(!body.contains(hidden), "{hidden} on the page");
    }

    // Read-only: the root answers no write.
    let (st, _, _) = call(app(&pool, recorders), Method::POST, "/").await;
    assert_eq!(st, StatusCode::METHOD_NOT_ALLOWED);
}
