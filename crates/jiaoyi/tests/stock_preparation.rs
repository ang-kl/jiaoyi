use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use chrono::{DateTime, Duration, Timelike, Utc};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{self as store, sqlx::PgPool};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app(pool: &PgPool, accounts: &[i64]) -> axum::Router {
    router(AppState {
        pool: pool.clone(),
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(accounts.iter().copied().collect::<BTreeSet<_>>()),
    })
}

async fn call(app: axum::Router, method: Method) -> (StatusCode, Vec<u8>) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri("/operations/stock-preparation")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, body.to_vec())
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn stock_preparation_route_reports_unavailable_inputs(pool: PgPool) {
    let account = 49236051;
    store::upsert_account(&pool, account, Environment::Demo)
        .await
        .unwrap();
    let (status, body) = call(app(&pool, &[account]), Method::GET).await;
    assert_eq!(status, StatusCode::OK);
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["source_chat"], "№17,618 §1");
    assert!(value["generated_at_sgt"].as_str().is_some());
    assert_eq!(value["stock_trading_active"], false);
    let accounts = value["accounts"].as_array().unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0]["account_id"], account);
    let sessions = accounts[0]["sessions"].as_array().unwrap();
    assert!(!sessions.is_empty());
    for session in sessions {
        assert_eq!(
            session["calendar_state"],
            "weekdays_only_holidays_unverified"
        );
        assert_eq!(session["preparation_state"], "unavailable");
        assert!(session["prepared_at"].is_null());
        assert_eq!(session["deadline_met"], false);
        let parse = |key: &str| {
            DateTime::parse_from_rfc3339(session[key].as_str().unwrap())
                .unwrap()
                .with_timezone(&Utc)
        };
        let open = parse("session_open");
        assert_eq!(open - parse("deadline_at"), Duration::minutes(10));
        assert_eq!(open - parse("preparation_starts_at"), Duration::minutes(20));
        match session["exchange"].as_str().unwrap() {
            "NASDAQ" => {
                let local = open.with_timezone(&chrono_tz::America::New_York);
                assert_eq!((local.hour(), local.minute()), (9, 30));
            }
            "Xetra" => {
                let local = open.with_timezone(&chrono_tz::Europe::Berlin);
                assert_eq!((local.hour(), local.minute()), (9, 0));
            }
            exchange => panic!("unexpected candidate exchange {exchange}"),
        }
        for candidate in session["candidates"].as_array().unwrap() {
            assert_eq!(candidate["asset_class"], "stock");
            assert_eq!(candidate["eligible"], false);
            assert_eq!(candidate["broker"]["state"], "catalogue_missing");
            assert!(!candidate["missing_inputs"].as_array().unwrap().is_empty());
        }
    }
    for (ticker, country) in [("AAPL", "US"), ("MSFT", "US"), ("SAP", "DE"), ("SIE", "DE")] {
        assert!(sessions.iter().any(|session| session["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |candidate| candidate["ticker"] == ticker && candidate["issuer_country"] == country
            )));
    }
}

async fn mutation_counts(pool: &PgPool) -> (i64, i64, i64, i64, i64, i64) {
    store::sqlx::query_as(
        "SELECT (SELECT count(*) FROM trading_intents),
                (SELECT count(*) FROM trading_events),
                (SELECT count(*) FROM trading_decisions),
                (SELECT count(*) FROM settings_changes),
                (SELECT version FROM settings_version WHERE singleton),
                (SELECT count(*) FROM settings)",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn stock_preparation_is_account_scoped_private_and_read_only(pool: PgPool) {
    let account = 49236051;
    let foreign = 46130058;
    for id in [account, foreign] {
        store::upsert_account(&pool, id, Environment::Demo)
            .await
            .unwrap();
    }
    let run = store::start_run(&pool, foreign).await.unwrap();
    store::finish_run(
        &pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(json!([{"id":111,"name":"FOREIGN_PRIVATE_CATALOGUE",
                "enabled":true,"secret":"PRIVATE_SECRET","balance":1234567.89}])),
            assets: Some(json!([{"asset_id":1,"name":"PRIVATE_ASSET"}])),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let before = mutation_counts(&pool).await;
    let router = app(&pool, &[account]);
    for _ in 0..2 {
        let (status, body) = call(router.clone(), Method::GET).await;
        assert_eq!(status, StatusCode::OK);
        let text = std::str::from_utf8(&body).unwrap();
        for private in ["PRIVATE_", "FOREIGN_PRIVATE", "1234567.89", "46130058"] {
            assert!(!text.contains(private), "unexpected private data marker");
        }
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["accounts"].as_array().unwrap().len(), 1);
        assert_eq!(value["accounts"][0]["account_id"], account);
    }
    let (status, _) = call(router, Method::POST).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(mutation_counts(&pool).await, before);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn stock_catalogue_uses_same_account_and_run_without_promoting_name_to_identity(
    pool: PgPool,
) {
    let account = 49236051;
    store::upsert_account(&pool, account, Environment::Demo)
        .await
        .unwrap();
    let run = store::start_run(&pool, account).await.unwrap();
    let listed = json!([
        {"id":101,"name":"AAPL","enabled":true,"base_asset_id":1,"quote_asset_id":2},
        {"id":102,"name":"MSFT","enabled":true},
        {"id":103,"name":"MSFT","enabled":true},
        {"id":104,"name":"SAP","enabled":false},
        {"id":105,"name":"BABA","enabled":true,"private":"PRIVATE_RAW_CATALOGUE"},
        {"id":106,"name":"PRIVATE_OTHER_SYMBOL","enabled":true}
    ]);
    store::finish_run(
        &pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(listed.clone()),
            assets: Some(json!([{"asset_id":1,"name":"AAPL"},{"asset_id":2,"name":"USD"}])),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let fetch = || async {
        let (status, body) = call(app(&pool, &[account]), Method::GET).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!std::str::from_utf8(&body).unwrap().contains("PRIVATE_"));
        serde_json::from_slice::<Value>(&body).unwrap()
    };
    let states = |value: &Value| {
        value["accounts"][0]["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["candidates"].as_array().unwrap())
            .map(|c| {
                assert_eq!(c["eligible"], false);
                (
                    c["ticker"].as_str().unwrap().to_owned(),
                    c["broker"]["state"].as_str().unwrap().to_owned(),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let first = fetch().await;
    let first_states = states(&first);
    assert_eq!(first_states["AAPL"], "matched_name_identity_unverified");
    assert_eq!(first_states["MSFT"], "ambiguous");
    assert_eq!(first_states["SAP"], "disabled");
    assert_eq!(first_states["SIE"], "not_listed");
    assert!(!first_states.contains_key("BABA"));

    // A newer symbols-only run must not borrow the older run's asset identity.
    let next_run = store::start_run(&pool, account).await.unwrap();
    store::finish_run(
        &pool,
        next_run,
        "partial",
        &store::RunSummary {
            listed_symbols: Some(listed),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    store::sqlx::query("UPDATE load_runs SET started_at=now()-interval '1 minute' WHERE id=$1")
        .bind(run)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(states(&fetch().await)["AAPL"], "asset_identity_missing");

    store::sqlx::query("UPDATE load_runs SET started_at=now()-interval '25 hours' WHERE id=$1")
        .bind(next_run)
        .execute(&pool)
        .await
        .unwrap();
    store::sqlx::query("UPDATE load_runs SET started_at=now()-interval '26 hours' WHERE id=$1")
        .bind(run)
        .execute(&pool)
        .await
        .unwrap();
    assert!(states(&fetch().await)
        .values()
        .all(|state| state == "catalogue_stale_or_future"));
}
