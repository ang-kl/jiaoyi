use super::*;
use jiaoyi_core::universe::{AssetClass, Resolution};
use jiaoyi_ctrader::messages as pb;
use jiaoyi_store as store;
use serde_json::json;

fn observed(strategy: &str) -> Input {
    let mut input = super::super::State::new(7, 1).input;
    input.revision = 12;
    input.complete = true;
    input.book_at = Some(Utc::now());
    input.positions.push(pb::ProtoOaPosition {
        position_id: 10,
        position_status: 1,
        trade_data: pb::ProtoOaTradeData {
            symbol_id: 9,
            volume: 10,
            trade_side: 1,
            label: Some(
                if strategy == "s3" {
                    "jiaoyi-s3"
                } else {
                    "jiaoyi-s1"
                }
                .into(),
            ),
            ..Default::default()
        },
        ..Default::default()
    });
    input.strategy.insert(
        10,
        StrategyEvidence {
            strategy: strategy.into(),
            side: "buy".into(),
            detail: json!({}),
            recorded_at: Utc::now(),
            original_volume: 10,
            entry_key: format!("{strategy}:7:9:opening"),
        },
    );
    input
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn reads_actual_pause_cost_and_session_gates_without_assigning_manual_orders(pool: PgPool) {
    store::upsert_account(&pool, 7, Environment::Demo)
        .await
        .unwrap();
    store::settings::change(
        &pool,
        "universe.symbols",
        None,
        Some(json!([{"name":"BTCUSD","class":"crypto"}])),
        "test",
        "permission evidence",
    )
    .await
    .unwrap();
    let run = store::start_run(&pool, 7).await.unwrap();
    store::record_resolution(
        &pool,
        run,
        7,
        "BTCUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: 9 },
    )
    .await
    .unwrap();
    let input = observed("s3");
    let before = collect(&pool, &input).await;
    assert_eq!(before["s3"].owner_paused, Some(false));
    assert_eq!(before["s3"].cost_veto_by_symbol.get(&9), Some(&false));
    assert_eq!(before["s3"].entry_recorded_by_symbol.get(&9), Some(&false));
    assert_eq!(before["s3"].live_single_position_blocked, Some(false));
    assert_eq!(before["s3"].book_revision, 12);
    store::settings::change(
        &pool,
        "trading.s3_paused",
        Some(7),
        Some(json!(1)),
        "test",
        "new entries paused; exits retained",
    )
    .await
    .unwrap();
    let at = Utc::now();
    let rule = sessions::rule_for("BTCUSD", AssetClass::Crypto).unwrap();
    let date = sessions::session_at(&rule, at).unwrap().date;
    let hash = crate::strategy3::prereg_hash();
    store::trading::record_decision(&pool, 7, Some("BTCUSD"), "s3_skip", "registered cost veto",
        &json!({"label":"jiaoyi-s3","prereg_hash":hash,"session_date":date,"skip_code":"cost_ge_r"}), None).await.unwrap();
    store::strategy3::record_order(
        &pool,
        &store::strategy3::OrderRow {
            account_id: 7,
            entry_key: "s3:7:9:opening".into(),
            label: "jiaoyi-s3".into(),
            prereg_hash: crate::strategy3::prereg_hash(),
            kind: "open".into(),
            symbol: "BTCUSD".into(),
            symbol_id: 9,
            position_id: Some(10),
            side: Some("buy".into()),
            session_date: Some(date),
            decided_at: at,
            bid: Some(100.0),
            ask: Some(101.0),
            volume: Some(10),
            detail: json!({}),
        },
    )
    .await
    .unwrap();
    let after = collect(&pool, &input).await;
    assert_eq!(after["s3"].owner_paused, Some(true));
    assert_eq!(after["s3"].cost_veto_by_symbol.get(&9), Some(&true));
    assert_eq!(after["s3"].entry_recorded_by_symbol.get(&9), Some(&true));
    let mut manual = input.clone();
    manual.strategy.clear();
    assert!(collect(&pool, &manual).await.is_empty());
    // A second universe name pointing at the same broker identity cannot
    // pick one of two session definitions without proof.
    store::settings::change(
        &pool,
        "universe.symbols",
        None,
        Some(json!([{"name":"BTCUSD","class":"crypto"},{"name":"ETHUSD","class":"crypto"}])),
        "test",
        "ambiguous observation mapping",
    )
    .await
    .unwrap();
    store::record_resolution(
        &pool,
        run,
        7,
        "ETHUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: 9 },
    )
    .await
    .unwrap();
    let ambiguous = collect(&pool, &input).await;
    assert!(ambiguous["s3"].cost_veto_by_symbol.is_empty());
    assert!(ambiguous["s3"].entry_recorded_by_symbol.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn s1_uses_actual_edge_gate_and_live_capacity_keeps_known_blocks_on_read_failure(
    pool: PgPool,
) {
    store::upsert_account(&pool, 7, Environment::Demo)
        .await
        .unwrap();
    let mut input = observed("s1");
    let read = collect(&pool, &input).await;
    assert_eq!(read["s1"].s1_edge_passed, Some(true));
    assert_eq!(read["s1"].owner_paused, Some(false));
    pool.close().await;
    assert_eq!(
        live_capacity(&pool, &input, Some(Environment::Live)).await,
        Some(true)
    );
    input.positions.clear();
    assert_eq!(
        live_capacity(&pool, &input, Some(Environment::Live)).await,
        None
    );
    assert_eq!(
        live_capacity(&pool, &input, Some(Environment::Demo)).await,
        Some(false)
    );
}
