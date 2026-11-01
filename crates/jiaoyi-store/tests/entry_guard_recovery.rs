use chrono::{Duration, Utc};
use jiaoyi_store::{self as store, strategy3 as s3, trading as ts};
use sqlx::PgPool;

async fn intent(pool: &PgPool, key: &str) {
    ts::record_intent(
        pool,
        &ts::IntentRow {
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
            settings_version: None,
        },
    )
    .await
    .unwrap();
}

async fn event(pool: &PgPool, key: &str, kind: &str, at: chrono::DateTime<Utc>) {
    ts::record_event(
        pool,
        &ts::EventRow {
            account_id: 7,
            entry_key: Some(key.into()),
            client_msg_id: Some(format!("t-{key}")),
            payload_type: 0,
            kind: kind.into(),
            position_id: None,
            order_id: None,
            deal_id: None,
            symbol_id: Some(9),
            side: None,
            price: None,
            volume: None,
            stop_loss: None,
            label: None,
            error_code: None,
            description: None,
            is_server_event: Some(false),
            raw: vec![],
            received_at: at,
        },
    )
    .await
    .unwrap();
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn proven_unsent_s1_attempt_releases_risk_and_gets_a_fresh_key_after_cooldown(pool: PgPool) {
    let now = Utc::now();
    let base = "s1:7:9:2026-11-01";
    assert_eq!(
        ts::next_entry_attempt(&pool, 7, base, now).await.unwrap(),
        ts::EntryAttempt::Ready(base.into())
    );
    intent(&pool, base).await;
    assert_eq!(
        ts::next_entry_attempt(&pool, 7, base, now).await.unwrap(),
        ts::EntryAttempt::Held
    );
    assert_eq!(
        ts::open_intents_unresolved(&pool, 7).await.unwrap().len(),
        1
    );
    event(&pool, base, "REFUSED_BEFORE_SEND", now).await;
    assert!(ts::open_intents_unresolved(&pool, 7)
        .await
        .unwrap()
        .is_empty());
    assert!(matches!(
        ts::next_entry_attempt(&pool, 7, base, now).await.unwrap(),
        ts::EntryAttempt::WaitUntil(_)
    ));
    let later = now + Duration::seconds(61);
    let retry = format!("{base}:retry:2");
    assert_eq!(
        ts::next_entry_attempt(&pool, 7, base, later).await.unwrap(),
        ts::EntryAttempt::Ready(retry.clone())
    );
    intent(&pool, &retry).await;
    assert_eq!(
        ts::next_entry_attempt(&pool, 7, base, later).await.unwrap(),
        ts::EntryAttempt::Held
    );
    event(&pool, &retry, "ORDER_REJECTED", later).await;
    assert_eq!(
        ts::next_entry_attempt(&pool, 7, base, later).await.unwrap(),
        ts::EntryAttempt::Held,
        "a real broker reply is not proof of an unsent frame"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn s3_proven_unsent_does_not_consume_entry_but_attempt_serial_and_hash_survive(pool: PgPool) {
    let at = Utc::now();
    let date = at.date_naive();
    for (key, hash) in [("s3:7:9:day:1", "old-v6"), ("s3:7:9:day:2", "new-v7")] {
        intent(&pool, key).await;
        s3::record_order(
            &pool,
            &s3::OrderRow {
                account_id: 7,
                entry_key: key.into(),
                label: "jiaoyi-s3".into(),
                prereg_hash: hash.into(),
                kind: "open".into(),
                symbol: "BTCUSD".into(),
                symbol_id: 9,
                position_id: None,
                side: Some("buy".into()),
                session_date: Some(date),
                decided_at: at,
                bid: Some(100.0),
                ask: Some(101.0),
                volume: Some(10),
                detail: serde_json::json!({}),
            },
        )
        .await
        .unwrap();
        if hash == "old-v6" {
            event(&pool, key, "REFUSED_BEFORE_SEND", at).await;
            assert_eq!(s3::opens_in_session(&pool, 7, 9, date).await.unwrap(), 0);
            assert_eq!(s3::attempts_in_session(&pool, 7, 9, date).await.unwrap(), 1);
        }
    }
    assert_eq!(s3::opens_in_session(&pool, 7, 9, date).await.unwrap(), 1);
    assert_eq!(s3::attempts_in_session(&pool, 7, 9, date).await.unwrap(), 2);
    let hashes: Vec<String> =
        sqlx::query_scalar("SELECT prereg_hash FROM strategy3_orders ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(hashes, ["old-v6", "new-v7"]);
    event(
        &pool,
        "s3:7:9:day:1",
        "ORDER_ACCEPTED",
        at + Duration::seconds(1),
    )
    .await;
    assert_eq!(
        s3::opens_in_session(&pool, 7, 9, date).await.unwrap(),
        2,
        "contradictory broker evidence cannot be excluded as proven unsent"
    );
    assert_eq!(
        ts::open_intents_unresolved(&pool, 7).await.unwrap().len(),
        2,
        "accepted evidence must retain exposure despite a contradictory unsent event"
    );
}
