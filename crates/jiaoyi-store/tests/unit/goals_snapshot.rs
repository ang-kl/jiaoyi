use super::*;

#[sqlx::test(migrator = "crate::MIGRATOR")]
async fn concurrent_history_settings_and_outcomes_are_seen_only_next_snapshot(pool: PgPool) {
    sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete) VALUES(77,now()-interval '1 day',now()-interval '3 seconds',now()-interval '3 seconds','ok',false)")
        .execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cutoff = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let before = assess_snapshot(&mut tx, 77, "s3", cutoff).await.unwrap();
    assert_eq!(before.state, AssessmentState::InsufficientEvidence);
    assert!(before.history_complete);
    // Three independently committed writers run while the assessment's
    // snapshot remains open. No sleeps or scheduler-dependent race.
    sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete,error) VALUES(77,now()-interval '1 day',clock_timestamp(),clock_timestamp(),'failed',false,'broker unavailable')")
        .execute(&pool).await.unwrap();
    crate::settings::change(
        &pool,
        "trading.reconcile_secs",
        Some(77),
        Some(serde_json::json!(60)),
        "test",
        "concurrent change",
    )
    .await
    .unwrap();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,position_id,side,volume,relative_stop,risk,max_risk) VALUES(77,'s3:77:1','open','open',1,1,'buy',100,100000,1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,symbol_id,volume,raw,received_at) VALUES(77,'s3:77:1','open',2126,'ORDER_FILLED',1,10,1,100,''::bytea,now()-interval '1 minute')")
        .execute(&pool).await.unwrap();
    let same = assess_snapshot(&mut tx, 77, "s3", cutoff).await.unwrap();
    assert_eq!(same, before);
    tx.commit().await.unwrap();
    let after = assess(&pool, 77, "s3").await.unwrap();
    assert_eq!(after.state, AssessmentState::Unknown);
    assert!(!after.history_complete);
    assert_eq!(after.history_max_age_secs, 120);
    assert!(after
        .missing_reasons
        .iter()
        .any(|r| r.contains("request failed")));
    assert_ne!(after.sample_identity, before.sample_identity);
}
