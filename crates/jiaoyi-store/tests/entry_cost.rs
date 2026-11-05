use jiaoyi_store::{self as store, trading};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn order_entry_cost_uses_matching_receipt_and_deals_not_reconcile(pool: PgPool) {
    for suffix in ["measured", "historical", "adopted"] {
        let key = format!("s3:7:9:{suffix}");
        sqlx::query(
            "INSERT INTO trading_intents
            (account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,max_risk)
            VALUES (7,$1,$1,'open',9,'buy',100,1)",
        )
        .bind(&key)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO strategy3_orders
            (account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,side,decided_at)
            VALUES (7,$1,'jiaoyi-s3','test','open','EURUSD',9,'buy',now())",
        )
        .bind(&key)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO strategy3_fills
            (account_id,entry_key,label,prereg_hash,kind,symbol,side,outcome,sent_at,decision_price)
            VALUES (7,$1,'jiaoyi-s3','test','open','EURUSD','buy','filled',now(),99)",
        )
        .bind(&key)
        .execute(&pool)
        .await
        .unwrap();
        let receipt = serde_json::to_vec(&json!({
            "policy":"entry-dispatch-v1-2026-11-01", "result":"passed",
            "stage":"after_writer_lock", "account_id":7, "symbol_id":9,
            "selected_side":"buy", "checked_at":"2026-11-05T09:00:01Z",
            "bid":{"price":100,"received_at":"2026-11-05T09:00:00Z"},
            "ask":{"price":101,"received_at":"2026-11-05T09:00:00Z"}
        }))
        .unwrap();
        // A different request's receipt must not be borrowed, even with the same key.
        let client = if suffix == "historical" {
            "unrelated-request"
        } else {
            &key
        };
        sqlx::query(
            "INSERT INTO trading_events
            (account_id,entry_key,client_msg_id,payload_type,kind,raw,received_at)
            VALUES (7,$1,$2,0,'ENTRY_VALIDATED_BEFORE_SEND',$3,now())",
        )
        .bind(&key)
        .bind(client)
        .bind(receipt)
        .execute(&pool)
        .await
        .unwrap();
        let kind = if suffix == "adopted" {
            "RECONCILED_FILL"
        } else {
            "ORDER_FILLED"
        };
        sqlx::query("INSERT INTO trading_events
            (account_id,entry_key,client_msg_id,payload_type,kind,deal_id,volume,price,raw,received_at)
            VALUES (7,$1,$1,0,$2,42,100,100.5,''::bytea,now())")
            .bind(&key).bind(kind).execute(&pool).await.unwrap();
    }
    let rows = trading::orders(
        &pool,
        &trading::OrdersQuery {
            account_id: Some(7),
            strategy: Some("s3".into()),
            since: None,
            limit: 10,
        },
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert_eq!(row.fill_price, Some(100.5)); // existing display remains intact
        if row.entry_key.ends_with("measured") {
            let a = row.entry_cost.unwrap();
            assert_eq!(a.decision_price, Some(99.0));
            assert_eq!(a.dispatch_price, 101.0);
            assert!(a.decision_to_dispatch_bps.unwrap() > 0.0);
            assert!(a.dispatch_to_fill_bps < 0.0);
        } else {
            assert!(row.entry_cost.is_none());
        }
    }
}
