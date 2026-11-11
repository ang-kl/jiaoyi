use super::*;

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn actual_repeated_preintent_refusals_have_one_outcome_per_consumed_trigger(pool: PgPool) {
    loaded(&pool).await;
    owner_sets_pause(&pool, 1).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            for minute in [30, 60, 90] {
                t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(minute))
                    .await
                    .unwrap();
            }
            let rows: Vec<(i64, i64, serde_json::Value)> = sqlx::query_as(
                "SELECT consumed.id, count(receipt.id),
                    coalesce(jsonb_agg(receipt.detail) FILTER (WHERE receipt.id IS NOT NULL), '[]')
             FROM trading_decisions consumed
             LEFT JOIN trading_decisions receipt
               ON receipt.kind='s3_trigger_outcome'
              AND receipt.account_id=consumed.account_id
              AND receipt.symbol=consumed.symbol
              AND receipt.detail->>'consumed_decision_id'=consumed.id::text
             WHERE consumed.account_id=$1 AND consumed.kind='s3_pullback_state'
               AND consumed.reason='pullback_trigger_consumed'
             GROUP BY consumed.id ORDER BY consumed.id",
            )
            .bind(DEMO)
            .fetch_all(&pool)
            .await
            .unwrap();
            assert_eq!(
                rows.len(),
                3,
                "actual arm/touch/turn path must consume three triggers: {:?}",
                decisions(&pool).await
            );
            assert_eq!(
                rows.iter().filter(|(_, n, _)| *n == 0).count(),
                0,
                "consumed_triggers_without_individual_outcome"
            );
            for (_, count, receipt) in rows {
                assert_eq!(count, 1);
                assert_eq!(receipt[0]["label"], "jiaoyi-s3");
                assert_eq!(receipt[0]["outcome"]["status"], "refused");
                assert_eq!(receipt[0]["outcome"]["reason"], "owner_pause_active");
                assert_eq!(receipt[0]["outcome"]["execution_resolution"], "not_sent");
            }
            assert_eq!(
                decisions(&pool)
                    .await
                    .iter()
                    .filter(|(kind, _)| kind == "s3_paused")
                    .count(),
                1,
                "repetitive idle diagnostics remain deduplicated"
            );
            let intents: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_intents")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(intents, 0, "all three refusals precede order intent");
            assert!(orders(&log).is_empty());
            assert!(closes(&log).is_empty());
            assert!(log.lock().unwrap().amends.is_empty());
        }
    );
}
