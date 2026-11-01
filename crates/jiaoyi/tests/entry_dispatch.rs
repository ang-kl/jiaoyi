//! Final-dispatch outcomes must preserve the existing durable cost-veto scope.
use chrono::Utc;
use jiaoyi::strategy3::S3Ledger;
use jiaoyi::trading::PgLedger;
use jiaoyi_ctrader::trading::{EventRecord, Ledger};
use jiaoyi_store::{self as store, strategy3 as s3};
use sqlx::PgPool;
use std::sync::Mutex;

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn expiry_is_not_a_cost_veto_but_final_registered_cost_refusal_is_durable(pool: PgPool) {
    let at = Utc::now();
    let hash = jiaoyi::strategy3::prereg_hash();
    let ledger = S3Ledger {
        inner: PgLedger {
            pool: &pool,
            settings_version: None,
        },
        order: s3::OrderRow {
            account_id: 7,
            entry_key: "s3:7:9:session:1".into(),
            label: "jiaoyi-s3".into(),
            prereg_hash: hash.clone(),
            kind: "open".into(),
            symbol: "BTCUSD".into(),
            symbol_id: 9,
            position_id: None,
            side: Some("buy".into()),
            session_date: Some(at.date_naive()),
            decided_at: at,
            bid: Some(100.0),
            ask: Some(101.0),
            volume: Some(10),
            detail: serde_json::json!({}),
        },
        recorded_at: Mutex::new(None),
    };
    let mut event = EventRecord {
        account_id: 7,
        client_msg_id: Some("t-1".into()),
        entry_key: Some(ledger.order.entry_key.clone()),
        payload_type: 0,
        kind: "REFUSED_BEFORE_SEND".into(),
        position_id: None,
        order_id: None,
        deal_id: None,
        symbol_id: Some(9),
        side: None,
        price: None,
        volume: None,
        stop_loss: None,
        label: None,
        error_code: Some("ENTRY_EXPIRED".into()),
        description: None,
        is_server_event: Some(false),
        raw: vec![],
        received_at: at,
    };
    ledger.record_event(&event).await.unwrap();
    assert!(
        !s3::cost_skipped_in_session(&pool, 7, "BTCUSD", at.date_naive(), &hash)
            .await
            .unwrap()
    );
    event.client_msg_id = Some("t-2".into());
    event.entry_key = Some("s3:7:9:session:2".into());
    event.error_code = Some("ENTRY_COST_GE_R".into());
    ledger.record_event(&event).await.unwrap();
    assert!(
        s3::cost_skipped_in_session(&pool, 7, "BTCUSD", at.date_naive(), &hash)
            .await
            .unwrap()
    );
    assert!(
        !s3::cost_skipped_in_session(&pool, 8, "BTCUSD", at.date_naive(), &hash)
            .await
            .unwrap()
    );
    assert!(
        !s3::cost_skipped_in_session(&pool, 7, "ETHUSD", at.date_naive(), &hash)
            .await
            .unwrap()
    );
    assert!(
        !s3::cost_skipped_in_session(&pool, 7, "BTCUSD", at.date_naive(), "older-v6")
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn execution_only_v7_preserves_only_its_direct_v6_predecessors_session_cost_veto(
    pool: PgPool,
) {
    use sha2::{Digest, Sha256};
    let date = Utc::now().date_naive();
    let current = jiaoyi::strategy3::prereg_hash();
    let previous: String = Sha256::digest(
        include_str!("../../../docs/preregistration-2026-10-28-strategy3-v6.md").as_bytes(),
    )
    .iter()
    .map(|b| format!("{b:02x}"))
    .collect();
    assert_ne!(current, previous);
    assert!(
        !jiaoyi::strategy3::session_cost_veto(&pool, 7, "BTCUSD", date, &current)
            .await
            .unwrap()
    );
    store::trading::record_decision(
        &pool,
        7,
        Some("BTCUSD"),
        "s3_skip",
        "v6 prepared cost at least R",
        &serde_json::json!({"label":"jiaoyi-s3", "prereg_hash":previous,
            "session_date":date, "skip_code":"cost_ge_r"}),
        None,
    )
    .await
    .unwrap();
    assert!(
        jiaoyi::strategy3::session_cost_veto(&pool, 7, "BTCUSD", date, &current)
            .await
            .unwrap()
    );
    assert!(
        !jiaoyi::strategy3::session_cost_veto(&pool, 8, "BTCUSD", date, &current)
            .await
            .unwrap()
    );
    assert!(
        !jiaoyi::strategy3::session_cost_veto(&pool, 7, "ETHUSD", date, &current)
            .await
            .unwrap()
    );
    assert!(!jiaoyi::strategy3::session_cost_veto(
        &pool,
        7,
        "BTCUSD",
        date.succ_opt().unwrap(),
        &current
    )
    .await
    .unwrap());
    assert!(
        !jiaoyi::strategy3::session_cost_veto(&pool, 7, "BTCUSD", date, "other-experiment")
            .await
            .unwrap()
    );
    assert!(
        !s3::cost_skipped_in_session(&pool, 7, "BTCUSD", date, &current)
            .await
            .unwrap(),
        "compatibility does not relabel historical evidence as v7"
    );
}
