use chrono::Utc;
use jiaoyi_store::{self as store, strategy3 as s3, trading as ts};
use sqlx::PgPool;

async fn rows(pool: &PgPool, key: &str) -> (ts::IntentRow, s3::OrderRow) {
    let version = sqlx::query_scalar("SELECT version FROM settings_version")
        .fetch_one(pool)
        .await
        .unwrap();
    let at = Utc::now();
    (
        ts::IntentRow {
            account_id: 7,
            entry_key: key.into(),
            client_msg_id: format!("t-{key}"),
            kind: "open".into(),
            symbol_id: Some(9),
            position_id: None,
            side: Some("buy".into()),
            volume: Some(10),
            relative_stop: Some(100000),
            stop_price: None,
            take_profit: None,
            risk: Some(0.1),
            max_risk: 1.0,
            settings_version: Some(version),
        },
        s3::OrderRow {
            account_id: 7,
            entry_key: key.into(),
            label: "jiaoyi-s3".into(),
            prereg_hash: "owner-18392".into(),
            kind: "open".into(),
            symbol: "BTCUSD".into(),
            symbol_id: 9,
            position_id: None,
            side: Some("buy".into()),
            session_date: Some(at.date_naive()),
            decided_at: at,
            bid: Some(100.),
            ask: Some(101.),
            volume: Some(10),
            detail: serde_json::json!({"prepared_admission":{}}),
        },
    )
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn prepared_pair_is_atomic_idempotent_and_rejects_stale_settings(pool: PgPool) {
    let (intent, order) = rows(&pool, "prepared-once").await;
    assert!(s3::prepared_intent_and_order(&pool, &intent, &order)
        .await
        .unwrap());
    assert!(!s3::prepared_intent_and_order(&pool, &intent, &order)
        .await
        .unwrap());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM strategy3_orders WHERE entry_key=$1")
        .bind(&intent.entry_key)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    let (mut stale, stale_order) = rows(&pool, "prepared-stale").await;
    stale.settings_version = stale.settings_version.map(|v| v - 1);
    assert!(s3::prepared_intent_and_order(&pool, &stale, &stale_order)
        .await
        .is_err());
    assert!(!ts::intent_exists(&pool, &stale.entry_key).await.unwrap());

    let (broken, broken_order) = rows(&pool, "prepared-order-conflict").await;
    s3::record_order(&pool, &broken_order).await.unwrap();
    assert!(s3::prepared_intent_and_order(&pool, &broken, &broken_order)
        .await
        .is_err());
    assert!(
        !ts::intent_exists(&pool, &broken.entry_key).await.unwrap(),
        "a failed evidence write must not leave an orphan intent"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn settings_update_winning_the_row_lock_prevents_stale_prepared_intent(pool: PgPool) {
    let (intent, order) = rows(&pool, "prepared-racing-pause").await;
    let mut owner = pool.begin().await.unwrap();
    sqlx::query("UPDATE settings_version SET version=version+1")
        .execute(&mut *owner)
        .await
        .unwrap();
    let other = pool.clone();
    let mut task =
        tokio::spawn(async move { s3::prepared_intent_and_order(&other, &intent, &order).await });
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(80), &mut task)
            .await
            .is_err(),
        "admission must not read past an uncommitted owner settings update"
    );
    owner.commit().await.unwrap();
    assert!(task.await.unwrap().is_err());
    assert!(!ts::intent_exists(&pool, "prepared-racing-pause")
        .await
        .unwrap());
}
