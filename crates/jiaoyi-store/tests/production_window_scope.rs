use chrono::{DateTime, Utc};
use jiaoyi_store::{
    sqlx::{self, PgPool},
    strategy3 as s,
};
use serde_json::json;
#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn six_crypto_windows_do_not_share_admission_or_cost_veto(pool: PgPool) {
    let first: DateTime<Utc> = "2026-11-09T00:00:00Z".parse().unwrap();
    let next = first + chrono::Duration::hours(4);
    let row = s::OrderRow {
        account_id: 49236051,
        entry_key: "s3:window:0".into(),
        label: "jiaoyi-s3".into(),
        prereg_hash: "new-policy".into(),
        kind: "open".into(),
        symbol: "BTCUSD".into(),
        symbol_id: 22,
        position_id: None,
        side: Some("buy".into()),
        session_date: Some(first.date_naive()),
        decided_at: first,
        bid: Some(100.),
        ask: Some(101.),
        volume: Some(1),
        detail: json!({"session":{"open":first}}),
    };
    s::record_order(&pool, &row).await.unwrap();
    assert_eq!(
        s::opens_in_window(&pool, 49236051, 22, first, "new-policy")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        s::opens_in_window(&pool, 49236051, 22, next, "new-policy")
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        s::attempts_in_window(&pool, 49236051, 22, next, "new-policy")
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        s::opens_in_window(&pool, 49236051, 22, first, "other-policy")
            .await
            .unwrap(),
        0
    );
    jiaoyi_store::trading::record_decision(&pool,49236051,Some("BTCUSD"),"s3_skip","cost",&json!({"session":{"open":first},"session_date":first.date_naive(),"prereg_hash":"new-policy","label":"jiaoyi-s3","skip_code":"cost_ge_r"}),None).await.unwrap();
    assert!(
        s::cost_skipped_in_window(&pool, 49236051, "BTCUSD", first, "new-policy")
            .await
            .unwrap()
    );
    assert!(
        !s::cost_skipped_in_window(&pool, 49236051, "BTCUSD", next, "new-policy")
            .await
            .unwrap()
    );
}
