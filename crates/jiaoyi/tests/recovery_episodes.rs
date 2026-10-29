//! Regression coverage for PR62 using real ownership events and broker deals.
use std::collections::BTreeSet;
use std::sync::{atomic::AtomicBool, Arc};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{DateTime, Duration, Utc};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{self as store, sqlx::PgPool};
use serde_json::json;
use tower::ServiceExt;

const A: i64 = 47790949;
const B: i64 = 49236051;

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn health_shares_fresh_diagnostics_without_changing_ownership_or_episode_grain(pool: PgPool) {
    position(&pool, A, 101, "BTCUSD", 30, day(), true, 100, true, "v6").await;
    position(
        &pool,
        B,
        102,
        "BTCUSD",
        30,
        day() + Duration::seconds(1),
        true,
        100,
        true,
        "v6",
    )
    .await;
    position(
        &pool,
        A,
        103,
        "BTCUSD",
        60,
        day() + Duration::hours(1),
        false,
        100,
        true,
        "v6",
    )
    .await;
    position(&pool, A, 104, "PARTIAL", 30, day(), true, 40, true, "v6").await;
    position(&pool, A, 105, "UNOWNED", 30, day(), true, 100, false, "v6").await;
    position(
        &pool,
        43002148,
        106,
        "FOREIGN",
        30,
        day(),
        true,
        100,
        true,
        "v6",
    )
    .await;
    let recovery = jiaoyi::stop_loss_review::recovery(&pool, &[A, B])
        .await
        .unwrap();
    let episodes = jiaoyi::strategy3::episodes(&pool, &[A, B]).await.unwrap();
    assert_eq!(recovery["stops"], 2);
    assert_eq!(recovery["same_session_later_close"], 1);
    assert_eq!(recovery["score"], 0.5);
    assert_eq!(episodes["episodes"], 2);
    assert_eq!(episodes["copied"], 1);
    let app = router(AppState {
        pool,
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([A, B])),
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let timing = response
        .headers()
        .get("server-timing")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let phases = timing
        .split(',')
        .map(|entry| {
            let (name, duration) = entry.split_once(";dur=").unwrap();
            let ms: f64 = duration.parse().unwrap();
            assert!(ms.is_finite() && ms >= 0.0);
            name
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        phases,
        BTreeSet::from([
            "base_health",
            "supervision",
            "fast_attach",
            "recent_signals",
            "projection"
        ])
    );
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["recovery"], recovery);
    assert_eq!(value["episodes"], episodes);
    let diagnostics =
        serde_json::to_string(&json!([value["recovery"], value["episodes"]])).unwrap();
    assert!(
        !diagnostics.contains("PARTIAL")
            && !diagnostics.contains("UNOWNED")
            && !diagnostics.contains("FOREIGN")
    );
}

fn day() -> DateTime<Utc> {
    (Utc::now() - Duration::days(2))
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_utc()
}

#[allow(clippy::too_many_arguments)]
async fn position(
    pool: &PgPool,
    account: i64,
    id: i64,
    symbol: &str,
    mark: i64,
    closed_at: DateTime<Utc>,
    stop: bool,
    closed_volume: i64,
    owned: bool,
    hash: &str,
) {
    store::upsert_account(pool, account, Environment::Demo)
        .await
        .unwrap();
    let key = format!("s3:{account}:1:{id}");
    store::sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,side,session_date,decided_at,volume,detail) VALUES($1,$2,'jiaoyi-s3',$3,'open',$4,1,'buy',$5,$6,100,$7)")
        .bind(account).bind(&key).bind(hash).bind(symbol).bind(day().date_naive()).bind(closed_at-Duration::hours(1))
        .bind(json!({"band":{"minutes":mark}})).execute(pool).await.unwrap();
    if owned {
        store::sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'buy',100,100000,1,1)")
            .bind(account).bind(&key).execute(pool).await.unwrap();
        store::sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,symbol_id,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$4,1,100,''::bytea,$5)")
            .bind(account).bind(&key).bind(id).bind(id*10).bind(closed_at-Duration::hours(1)).execute(pool).await.unwrap();
    }
    // Closing keys differ from the opening key, just as the real writers do.
    let close = format!("s3-broker:{account}:{id}");
    store::sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,session_date,decided_at,volume,detail) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit',$4,1,$5,'buy',$6,$7,$8,$9)")
        .bind(account).bind(&close).bind(hash).bind(symbol).bind(id).bind(day().date_naive()).bind(closed_at).bind(closed_volume)
        .bind(json!({"open_entry_key":key})).execute(pool).await.unwrap();
    // Live execution receipts arrive after the broker's execution timestamp.
    store::sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,answered_at,volume,detail) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit',$4,$5,'filled',$6,$6+interval '50 milliseconds',$7,$8)")
        .bind(account).bind(&close).bind(hash).bind(symbol).bind(id).bind(closed_at).bind(closed_volume)
        .bind(json!({"reason":if stop {"stop_loss"} else {"take_profit"}})).execute(pool).await.unwrap();
    for (deal, volume, closing, at) in [
        (id * 10, 100, false, closed_at - Duration::hours(1)),
        (id * 10 + 1, closed_volume, true, closed_at),
    ] {
        store::sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,money_digits,is_closing,raw,fetched_at) VALUES($1,$2,$2,$3,1,$4,$4,'BUY','FILLED',$5,$5,2,$6,''::bytea,now())")
            .bind(account).bind(deal).bind(id).bind(volume).bind(at).bind(closing).execute(pool).await.unwrap();
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn episodes_join_real_open_ownership_and_keep_marks_and_hashes_separate(pool: PgPool) {
    position(&pool, A, 1, "ZZZ", 30, day(), false, 100, true, "v5").await;
    position(
        &pool,
        B,
        2,
        "ZZZ",
        30,
        day() + Duration::minutes(1),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(
        &pool,
        A,
        3,
        "AAA",
        60,
        day() + Duration::minutes(2),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(
        &pool,
        A,
        4,
        "ZZZ",
        30,
        day() + Duration::minutes(3),
        false,
        100,
        true,
        "v4",
    )
    .await;
    position(
        &pool,
        A,
        5,
        "ZZZ",
        60,
        day() + Duration::minutes(4),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(&pool, A, 6, "PARTIAL", 30, day(), false, 50, true, "v5").await;
    position(&pool, A, 7, "FOREIGN", 30, day(), false, 100, false, "v5").await;
    let result = jiaoyi::strategy3::episodes(&pool, &[A, B]).await.unwrap();
    assert_eq!(result["episodes"], 4);
    assert_eq!(result["copied"], 1);
    let rows = result["rows"].as_array().unwrap();
    assert_eq!(rows[0]["symbol"], "ZZZ");
    assert_eq!(rows[0]["mark_minutes"], "60");
    assert_eq!(rows[1]["prereg_hash"], "v4");
    assert_eq!(rows[3]["copies"], 2);
    assert_eq!(rows[3]["accounts"].as_array().unwrap().len(), 2);
    let scoped = jiaoyi::strategy3::episodes(&pool, &[A]).await.unwrap();
    assert_eq!(scoped["copied"], 0);
    assert!(!result.to_string().contains("PARTIAL"));
    assert!(!result.to_string().contains("FOREIGN"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn latest_twenty_episodes_are_chronological_not_alphabetical(pool: PgPool) {
    for id in 1..=23 {
        position(
            &pool,
            A,
            id,
            if id == 23 { "ZZZ" } else { "AAA" },
            id * 30,
            day() + Duration::minutes(id),
            false,
            100,
            true,
            "v5",
        )
        .await;
    }
    let result = jiaoyi::strategy3::episodes(&pool, &[A]).await.unwrap();
    assert_eq!(result["episodes"], 23);
    assert_eq!(result["latest20_episodes"], 20);
    assert_eq!(result["rows"][0]["symbol"], "ZZZ");
    assert_eq!(result["rows"][19]["mark_minutes"], "120");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn recovery_requires_owned_complete_closes_in_the_same_account_day_and_cohort(pool: PgPool) {
    position(&pool, A, 1, "BTCUSD", 30, day(), true, 100, true, "v5").await;
    position(
        &pool,
        A,
        2,
        "BTCUSD",
        60,
        day() + Duration::minutes(1),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(&pool, A, 3, "ETHUSD", 30, day(), true, 100, true, "v5").await;
    // None of these proves later activity after the ETH stop on A/v5/day.
    position(
        &pool,
        A,
        4,
        "ETHUSD",
        60,
        day() + Duration::minutes(1),
        false,
        50,
        true,
        "v5",
    )
    .await;
    position(
        &pool,
        B,
        5,
        "ETHUSD",
        60,
        day() + Duration::minutes(1),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(
        &pool,
        A,
        6,
        "ETHUSD",
        60,
        day() + Duration::minutes(1),
        false,
        100,
        true,
        "v4",
    )
    .await;
    position(
        &pool,
        A,
        7,
        "ETHUSD",
        60,
        day() + Duration::days(1),
        false,
        100,
        true,
        "v5",
    )
    .await;
    position(&pool, A, 8, "UNOWNED", 30, day(), true, 100, false, "v5").await;
    let result = jiaoyi::stop_loss_review::recovery(&pool, &[A, B])
        .await
        .unwrap();
    assert_eq!(result["stops"], 2);
    assert_eq!(result["same_session_later_close"], 1);
    assert_eq!(result["score"], 0.5);
    assert!(result["path"]
        .as_str()
        .unwrap()
        .contains("not a quote path or profitable-recovery rate"));
    assert_eq!(
        jiaoyi::stop_loss_review::recovery(&pool, &[])
            .await
            .unwrap()["score"],
        serde_json::Value::Null
    );
}
