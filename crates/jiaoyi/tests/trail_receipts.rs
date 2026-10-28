//! Real-ledger read-back of immutable-R trail requests and raw broker receipts.
use std::collections::BTreeSet;
use std::sync::{atomic::AtomicBool, Arc};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use jiaoyi::{
    http::{router, AppState},
    trail_receipts,
};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_ctrader::messages::{ProtoOaExecutionType, ProtoOaPosition, ProtoOaTradeData};
use jiaoyi_store::{
    self as store,
    sqlx::{self, PgPool},
};
use prost::Message;
use serde_json::{json, Value};
use tower::ServiceExt;

const A: i64 = 49236051;
const B: i64 = 46130058;

// Real ExecutionEvent wire fields, independently checked through the existing
// cTrader execution decoder when seeding the accepted acknowledgement below.
#[derive(Clone, PartialEq, Message)]
struct Execution {
    #[prost(int64, required, tag = "2")]
    account: i64,
    #[prost(int32, required, tag = "3")]
    kind: i32,
    #[prost(message, optional, tag = "4")]
    position: Option<ProtoOaPosition>,
}

struct Fixture {
    trail: String,
    client: String,
    at: chrono::DateTime<Utc>,
    stop: f64,
}

async fn seed(
    pool: &PgPool,
    account: i64,
    id: i64,
    buy: bool,
    original_r: f64,
    reported_r: f64,
    owned: bool,
) -> Fixture {
    store::upsert_account(pool, account, Environment::Demo)
        .await
        .unwrap();
    sqlx::query("INSERT INTO symbols(account_id,broker_symbol_id,name,asset_class,digits,pip_position,raw,read_at) VALUES($1,1,'BTCUSD','crypto',2,2,''::bytea,now()) ON CONFLICT DO NOTHING")
        .bind(account).execute(pool).await.unwrap();
    let at = Utc::now() - Duration::minutes(2);
    let open = format!("s3:{account}:1:2026-10-28:{id}");
    let trail = format!("s3-trail:{account}:{id}:10500000");
    let client = format!("correlated-{account}-{id}");
    let side = if buy { "buy" } else { "sell" };
    let stop = if buy { 105.0 } else { 95.0 };
    let from = 100.0;
    let best = if buy { 115.0 } else { 85.0 };
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,side,session_date,decided_at,volume,detail) VALUES($1,$2,'jiaoyi-s3',$3,'open','BTCUSD',1,$4,$5,$6,100,$7)")
        .bind(account).bind(&open).bind("a".repeat(64)).bind(side).bind(at.date_naive()).bind(at)
        .bind(json!({"entry":{"r":original_r,"entry_price":100.0},"private":"PRIVATE_PARENT_SECRET"})).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk,created_at) VALUES($1,$2,$2,'open',1,$3,100,1000000,1,1,$4)")
        .bind(account).bind(&open).bind(side).bind(at).execute(pool).await.unwrap();
    if owned {
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,symbol_id,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,1,100,''::bytea,$4)")
            .bind(account).bind(&open).bind(id).bind(at).execute(pool).await.unwrap();
    }
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,session_date,decided_at,volume,detail) VALUES($1,$2,'jiaoyi-s3',$3,'trail','BTCUSD',1,$4,$5,$6,$7,100,$8)")
        .bind(account).bind(&trail).bind("b".repeat(64)).bind(id).bind(side).bind(at.date_naive()).bind(at+Duration::seconds(2))
        .bind(json!({"open_entry_key":open,"r":reported_r,"best":best,"new_stop":stop,"from_stop":from,"balance":123456789.12,"private":"PRIVATE_TRAIL_SECRET"})).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,position_id,side,stop_price,take_profit,max_risk,created_at) VALUES($1,$2,$3,'amend_stop',$4,$5,$6,150,1,$7)")
        .bind(account).bind(&trail).bind(&client).bind(id).bind(side).bind(stop).bind(at+Duration::seconds(2)).execute(pool).await.unwrap();
    Fixture {
        trail,
        client,
        at: at + Duration::seconds(3),
        stop,
    }
}

#[allow(clippy::too_many_arguments)]
async fn ack(
    pool: &PgPool,
    account: i64,
    id: i64,
    buy: bool,
    f: &Fixture,
    stored_kind: &str,
    raw_account: i64,
    broker_stop: f64,
    tp: f64,
) {
    let position = ProtoOaPosition {
        position_id: id,
        trade_data: ProtoOaTradeData {
            symbol_id: 1,
            volume: 100,
            trade_side: if buy { 1 } else { 2 },
            label: Some("jiaoyi-s3".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        stop_loss: Some(broker_stop),
        take_profit: Some(tp),
        ..Default::default()
    };
    let bytes = Execution {
        account: raw_account,
        kind: ProtoOaExecutionType::OrderReplaced as i32,
        position: Some(position),
    }
    .encode_to_vec();
    // Pin the fixture's wire shape against the established protocol decoder.
    let event = jiaoyi_ctrader::trading::decode_execution(
        raw_account,
        &jiaoyi_ctrader::ReceivedExecution {
            client_msg_id: Some(f.client.clone()),
            payload_type: 2126,
            payload: bytes.clone(),
            received_at: f.at,
        },
    )
    .unwrap();
    assert_eq!(event.kind, "ORDER_REPLACED");
    assert_eq!(event.stop_loss, Some(broker_stop));
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,symbol_id,stop_loss,raw,received_at) VALUES($1,$2,$3,2126,$4,$5,1,$6,$7,$8)")
        .bind(account).bind(&f.trail).bind(&f.client).bind(stored_kind).bind(id).bind(broker_stop).bind(bytes).bind(f.at).execute(pool).await.unwrap();
}

async fn value(pool: &PgPool, allowed: &[i64]) -> Value {
    serde_json::to_value(trail_receipts::report(pool, allowed).await.unwrap()).unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn accepted_long_and_short_receipts_prove_original_r_and_broker_tp_read_only(pool: PgPool) {
    for (id, buy) in [(10, true), (11, false)] {
        let f = seed(&pool, A, id, buy, 10.0, 10.0, true).await;
        ack(&pool, A, id, buy, &f, "ORDER_REPLACED", A, f.stop, 150.0).await;
    }
    let before:(i64,i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM trading_intents),(SELECT count(*) FROM trading_events),(SELECT count(*) FROM strategy3_orders)").fetch_one(&pool).await.unwrap();
    let v = value(&pool, &[A]).await;
    let rows = v["receipts"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for r in rows {
        assert_eq!(r["state"], "passed", "{r}");
        assert_eq!(r["original_r"], 10.0);
        assert_eq!(r["reported_r"], 10.0);
        assert_eq!(r["checks"]["take_profit_preserved"], true);
        assert_eq!(r["checks"]["broker_stop_matches"], true);
        assert!(r["acknowledged_at"].is_string());
        assert_eq!(r["parent_hash"], "a".repeat(64));
    }
    let after:(i64,i64,i64)=sqlx::query_as("SELECT (SELECT count(*) FROM trading_intents),(SELECT count(*) FROM trading_events),(SELECT count(*) FROM strategy3_orders)").fetch_one(&pool).await.unwrap();
    assert_eq!(before, after);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unproven_ack_ownership_and_mutable_r_never_pass(pool: PgPool) {
    let missing = seed(&pool, A, 20, true, 10.0, 10.0, true).await;
    let wrong_account = seed(&pool, A, 21, true, 10.0, 10.0, true).await;
    ack(
        &pool,
        A,
        21,
        true,
        &wrong_account,
        "ORDER_REPLACED",
        B,
        wrong_account.stop,
        150.0,
    )
    .await;
    let accepted_only = seed(&pool, A, 22, true, 10.0, 10.0, true).await;
    ack(
        &pool,
        A,
        22,
        true,
        &accepted_only,
        "ORDER_ACCEPTED",
        A,
        accepted_only.stop,
        150.0,
    )
    .await;
    let mutable = seed(&pool, A, 23, true, 10.0, 5.0, true).await;
    ack(
        &pool,
        A,
        23,
        true,
        &mutable,
        "ORDER_REPLACED",
        A,
        mutable.stop,
        150.0,
    )
    .await;
    let unowned = seed(&pool, A, 24, true, 10.0, 10.0, false).await;
    ack(
        &pool,
        A,
        24,
        true,
        &unowned,
        "ORDER_REPLACED",
        A,
        unowned.stop,
        150.0,
    )
    .await;
    let v = value(&pool, &[A]).await;
    let rows = v["receipts"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    let find = |key: &str| rows.iter().find(|r| r["entry_key"] == key).unwrap();
    for key in [
        &missing.trail,
        &wrong_account.trail,
        &accepted_only.trail,
        &unowned.trail,
    ] {
        assert_eq!(find(key)["state"], "unknown", "{}", find(key));
    }
    assert_eq!(find(&mutable.trail)["state"], "refused");
    assert_eq!(find(&mutable.trail)["checks"]["original_r_matches"], false);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn route_is_account_scoped_bounded_private_and_get_only(pool: PgPool) {
    for id in 30..37 {
        let f = seed(&pool, A, id, true, 10.0, 10.0, true).await;
        ack(&pool, A, id, true, &f, "ORDER_REPLACED", A, f.stop, 150.0).await;
    }
    let foreign = seed(&pool, B, 40, true, 10.0, 10.0, true).await;
    ack(
        &pool,
        B,
        40,
        true,
        &foreign,
        "ORDER_REPLACED",
        B,
        foreign.stop,
        150.0,
    )
    .await;
    let app = router(AppState {
        pool,
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([A])),
    });
    let response = app
        .clone()
        .oneshot(
            Request::get("/operations/trailing-receipts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = std::str::from_utf8(&bytes).unwrap();
    for forbidden in [
        "PRIVATE_",
        "123456789.12",
        &B.to_string(),
        "raw",
        "client_msg_id",
    ] {
        assert!(!text.contains(forbidden), "leaked {forbidden}");
    }
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["receipts"].as_array().unwrap().len(), 5);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/operations/trailing-receipts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
