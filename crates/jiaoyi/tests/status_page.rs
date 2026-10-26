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

// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
/// `/trading/orders` through the real router: a parameter outside its
/// bounds is refused (400) with the reason; the list answers JSON with no
/// money; `/trading/costs` names both strategies.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_order_list_refuses_bad_parameters_and_answers_without_money(pool: PgPool) {
    for uri in [
        "/trading/orders?limit=0",
        "/trading/orders?limit=501",
        "/trading/orders?strategy=s2",
        "/trading/orders?account=abc",
        "/trading/orders?since=yesterday",
    ] {
        let (status, _, body) = call(app(&pool, Default::default()), Method::GET, uri).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}: {body}");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["status"], "refused", "{uri}");
        assert!(v["reason"].is_string(), "{uri}");
    }
    let (status, ctype, body) = call(
        app(&pool, Default::default()),
        Method::GET,
        "/trading/orders?strategy=s1&limit=500",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(ctype.starts_with("application/json"), "{ctype}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["count"], 0);
    assert_eq!(v["rows"], json!([]));
    assert_eq!(v["strategies"]["s1"], "Strategy 1: tick-triggered trend");
    let lower = body.to_lowercase();
    for word in [
        "balance",
        "equity",
        "pnl",
        "profit",
        "commission",
        "\"risk\"",
        "max_risk",
    ] {
        assert!(!lower.contains(word), "{word}: {body}");
    }
    let (status, _, body) = call(
        app(&pool, Default::default()),
        Method::GET,
        "/trading/costs",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        v["strategies"]["s1"].is_string() && v["strategies"]["s3"].is_string(),
        "{v}"
    );
}

// Codex · № 17,210 26-Oct (ordered: O-1, bounded execution diagnostics).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn execution_view_scopes_owned_positions_and_keeps_payloads_private(pool: PgPool) {
    use jiaoyi_ctrader::messages::{ProtoOaPosition, ProtoOaTradeData};
    use jiaoyi_ctrader::testkit::server_fill;
    for account in [LIVE, LIVE + 1] {
        store::upsert_account(&pool, account, Environment::Live)
            .await
            .unwrap();
        let key = format!("diagnostic:{account}");
        sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,decided_at)
            VALUES ($1,$2,'jiaoyi-s3','test','open','BTCUSD',22,now())")
            .bind(account).bind(&key).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at)
            VALUES ($1,$2,'jiaoyi-s3','test','open','BTCUSD',1001,'filled',now())")
            .bind(account).bind(&key).execute(&pool).await.unwrap();
        for position_id in [1001, 1002] {
            let p = ProtoOaPosition {
                position_id,
                trade_data: ProtoOaTradeData {
                    symbol_id: 22,
                    volume: 13,
                    trade_side: 1,
                    label: Some("hidden-label".into()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut message =
                server_fill(account, p, position_id + 10000, 60_000.0, 13, true, false);
            message.client_msg_id = Some("hidden-correlation".into());
            let event = jiaoyi_ctrader::trading::decode_execution(account, &message).unwrap();
            store::trading::record_event(&pool, &jiaoyi::trading::event_row(&event))
                .await
                .unwrap();
        }
    }
    let (status, _, body) = call(
        app(&pool, Default::default()),
        Method::GET,
        "/trading/executions?limit=1",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["count"], 1);
    let row = &v["rows"][0];
    assert_eq!(row["account_id"], LIVE);
    assert_eq!(row["position_id"], 1001);
    assert_eq!(row["closing_deal_present"], true);
    assert_eq!(row["execution_type"], "ORDER_FILLED");
    assert_eq!(row["label_matches_strategy"], false);
    assert_eq!(row["client_msg_id_present"], true);
    for hidden in [
        "hidden-label",
        "hidden-correlation",
        "60000",
        "commission",
        "balance",
        "\"raw\"",
        "\"price\"",
    ] {
        assert!(!body.contains(hidden), "leaked {hidden}: {body}");
    }
    let malformed = store::trading::EventRow {
        account_id: LIVE,
        entry_key: None,
        client_msg_id: None,
        payload_type: 2126,
        kind: "ORDER_FILLED".into(),
        position_id: Some(1001),
        order_id: None,
        deal_id: Some(99999),
        symbol_id: Some(22),
        side: None,
        price: None,
        volume: None,
        stop_loss: None,
        label: None,
        error_code: None,
        description: None,
        is_server_event: None,
        raw: vec![255],
        received_at: chrono::Utc::now() + chrono::Duration::seconds(1),
    };
    store::trading::record_event(&pool, &malformed)
        .await
        .unwrap();
    let (_, _, unknown) = call(
        app(&pool, Default::default()),
        Method::GET,
        "/trading/executions?limit=1",
    )
    .await;
    let unknown: serde_json::Value = serde_json::from_str(&unknown).unwrap();
    assert_eq!(unknown["rows"][0]["decodable"], false);
    assert!(unknown["rows"][0]["closing_deal_present"].is_null());
    assert!(unknown["rows"][0]["label_matches_strategy"].is_null());
    for uri in [
        "/trading/executions?limit=0",
        "/trading/executions?strategy=s1",
    ] {
        assert_eq!(
            call(app(&pool, Default::default()), Method::GET, uri)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        call(
            app(&pool, Default::default()),
            Method::GET,
            &format!("/trading/executions?account={}", LIVE + 1)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (_, _, body) = call(
        app(&pool, Default::default()),
        Method::GET,
        "/trading/executions?since=2099-01-01T00:00:00Z",
    )
    .await;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&body).unwrap()["count"],
        0
    );
    assert_eq!(
        call(
            app(&pool, Default::default()),
            Method::POST,
            "/trading/executions"
        )
        .await
        .0,
        StatusCode::METHOD_NOT_ALLOWED
    );
}
