//! Regression tests for No.18571 hot read-path indexes, CTE materialization,
//! `ticks::stats` partition index lookups, and `history::record_window` UNNEST batching.

use chrono::{Duration, TimeZone, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{
    self as store,
    goals::{self, AssessmentState},
    history::{self, FetchRecord, HistoryTick},
    ticks,
    trading::FILL_KINDS,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};

const ACCOUNT: i64 = 46130058;

const FACTS_EXPLAIN_SQL: &str = r#"
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
WITH owned AS MATERIALIZED (
 SELECT DISTINCT ON (e.position_id) e.position_id, i.entry_key, i.volume AS requested_volume, i.risk, i.side, i.symbol_id
 FROM trading_intents i JOIN trading_events e
   ON e.entry_key = i.entry_key AND e.account_id = i.account_id
 WHERE i.account_id = $1 AND i.kind = 'open' AND i.entry_key LIKE $2 || ':%'
   AND e.position_id IS NOT NULL AND e.kind = ANY($3)
 ORDER BY e.position_id, (e.volume > 0) DESC NULLS LAST, i.id
), opening_deal_events AS (
 SELECT position_id, SUM(volume)::bigint AS quantity FROM (
   SELECT DISTINCT ON (e.position_id, e.deal_id) e.position_id, e.volume
   FROM trading_events e JOIN owned o ON o.entry_key = e.entry_key AND o.position_id = e.position_id
   WHERE e.account_id = $1 AND e.kind = ANY($3) AND e.volume > 0 AND e.deal_id IS NOT NULL
   ORDER BY e.position_id, e.deal_id, e.id DESC
 ) d GROUP BY position_id
), opening_events AS (
 SELECT o.position_id, GREATEST(COALESCE(d.quantity,0),COALESCE((
  SELECT MAX(e.volume) FROM trading_events e WHERE e.account_id=$1
   AND e.entry_key=o.entry_key AND e.position_id=o.position_id
   AND e.kind=ANY($3) AND e.volume>0
 ),0))::bigint AS quantity
 FROM owned o LEFT JOIN opening_deal_events d ON d.position_id=o.position_id
), owned_orders AS MATERIALIZED (
 SELECT DISTINCT o.position_id, COALESCE(e.order_id,d.order_id) AS order_id
 FROM owned o JOIN trading_events e ON e.account_id=$1 AND e.entry_key=o.entry_key
  AND e.position_id=o.position_id AND e.kind=ANY($3) AND e.volume>0
 LEFT JOIN deals d ON d.account_id=$1 AND d.position_id=o.position_id
  AND d.deal_id=e.deal_id AND NOT d.is_closing AND d.filled_volume>0
), history AS (
 SELECT o.position_id,
   COALESCE(SUM(d.filled_volume) FILTER (WHERE NOT d.is_closing), 0)::bigint AS opened,
   COALESCE(SUM(d.filled_volume) FILTER (WHERE d.is_closing), 0)::bigint AS closed,
   BOOL_OR(d.is_closing AND (d.close_gross_profit IS NULL OR d.close_swap IS NULL
      OR d.close_commission IS NULL OR d.close_pnl_conversion_fee IS NULL OR COALESCE(d.close_money_digits,d.money_digits) IS NULL
      OR COALESCE(d.close_money_digits,d.money_digits) NOT BETWEEN 0 AND 18)) AS missing_costs,
   BOOL_OR(d.deal_status NOT IN ('FILLED','PARTIALLY_FILLED')) AS invalid_status,
   BOOL_OR(NOT d.is_closing AND NOT COALESCE((
     LOWER(d.trade_side)=o.side AND d.broker_symbol_id=o.symbol_id AND (
       EXISTS (SELECT 1 FROM trading_events e WHERE e.account_id=$1
        AND e.entry_key=o.entry_key AND e.position_id=o.position_id
        AND e.kind=ANY($3) AND e.volume>0 AND e.deal_id=d.deal_id)
       OR EXISTS (SELECT 1 FROM owned_orders x WHERE x.position_id=o.position_id
        AND x.order_id=d.order_id)
     )),false)) AS unowned_opening,
   MAX(d.executed_at) FILTER (WHERE d.is_closing) AS latest_history_close
 FROM owned o LEFT JOIN deals d ON d.account_id = $1 AND d.position_id = o.position_id AND d.filled_volume > 0
 GROUP BY o.position_id
), local_receipts AS (
 SELECT position_id, entry_key, SUM(volume)::bigint AS quantity, MAX(received_at) AS at FROM (
  SELECT DISTINCT ON (e.position_id,e.deal_id) e.position_id, i.entry_key, e.volume,e.received_at
  FROM trading_events e JOIN trading_intents i ON i.entry_key = e.entry_key AND i.account_id=e.account_id
  JOIN owned o ON o.position_id=e.position_id
  WHERE e.account_id=$1 AND i.kind='close' AND e.kind=ANY($3) AND e.volume>0 AND e.deal_id IS NOT NULL
  ORDER BY e.position_id,e.deal_id,e.id DESC
 ) d GROUP BY position_id,entry_key
), fills AS (
 SELECT DISTINCT ON (f.entry_key) f.position_id,f.entry_key,f.volume AS quantity,
   COALESCE(f.answered_at,f.sent_at) AS at
 FROM strategy3_fills f JOIN owned o ON o.position_id=f.position_id
 WHERE $2='s3' AND f.account_id=$1 AND f.kind IN ('broker_exit','tp1','session_end')
   AND f.outcome IN ('filled','partly_filled') AND f.volume>0
 ORDER BY f.entry_key,f.id DESC
), s1_fills AS (
 SELECT DISTINCT ON (f.entry_key) f.position_id,f.entry_key,f.volume AS quantity,
   COALESCE(f.answered_at,f.sent_at) AS at
 FROM strategy1_fills f JOIN owned o ON o.position_id=f.position_id
 WHERE $2='s1' AND f.account_id=$1 AND f.kind='broker_exit'
   AND f.outcome IN ('filled','partly_filled') AND f.volume>0
 ORDER BY f.entry_key,f.id DESC
), grouped_receipts AS MATERIALIZED (
 SELECT position_id,entry_key,MAX(quantity) AS quantity, MAX(at) AS at
 FROM (SELECT * FROM local_receipts UNION ALL SELECT * FROM fills UNION ALL SELECT * FROM s1_fills) r
 GROUP BY position_id,entry_key
), closing_receipts AS (
 SELECT position_id,SUM(quantity)::bigint AS quantity,MAX(at) AS at FROM grouped_receipts GROUP BY position_id
)
SELECT o.position_id, o.entry_key, o.requested_volume, o.risk,
 COALESCE(e.quantity,0) AS receipt_opened, GREATEST(COALESCE(e.quantity,0),h.opened) AS opened,
 h.opened AS history_opened,h.closed,COALESCE(h.missing_costs,false) AS missing_costs,
 COALESCE(h.invalid_status,false) AS invalid_status,
 COALESCE(h.unowned_opening,false) AS unowned_opening, h.latest_history_close,
 COALESCE(c.quantity,0) AS observed_closed, c.at AS latest_exit,
 (EXISTS (
  SELECT 1 FROM trading_events x JOIN trading_intents i ON i.account_id=x.account_id AND i.entry_key=x.entry_key
  WHERE x.account_id=$1 AND x.position_id=o.position_id AND i.kind='close'
   AND x.kind=ANY($3) AND x.volume>0 AND x.deal_id IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id AND d.deal_id=x.deal_id AND d.is_closing AND d.filled_volume>=x.volume)
 ) OR EXISTS (
  SELECT 1 FROM grouped_receipts g WHERE g.position_id=o.position_id AND g.entry_key LIKE $2||'-broker:%'
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id AND d.is_closing
     AND d.filled_volume>=g.quantity AND g.entry_key=$2||'-broker:'||$1::text||':'||o.position_id::text||':'||d.deal_id::text)
 )) AS receipt_deal_missing,
 EXISTS (
  SELECT 1 FROM trading_events x WHERE x.account_id=$1 AND x.position_id=o.position_id
   AND x.entry_key=o.entry_key AND x.kind=ANY($3) AND x.volume>0 AND x.deal_id IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id
    AND d.deal_id=x.deal_id AND NOT d.is_closing AND d.filled_volume>=x.volume)
 ) AS opening_receipt_deal_missing,
 EXISTS (
  SELECT 1 FROM trading_events x WHERE x.account_id=$1 AND x.position_id=o.position_id
   AND x.kind=ANY($3) AND x.volume>0 AND x.entry_key IS NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=x.position_id
      AND d.deal_id=x.deal_id AND d.filled_volume>=x.volume)
   AND NOT EXISTS (SELECT 1 FROM grouped_receipts g WHERE g.position_id=x.position_id
      AND g.quantity>=x.volume AND g.entry_key=$2||'-broker:'||$1::text||':'||x.position_id::text||':'||x.deal_id::text)
 ) AS unmatched_owned_fill
FROM owned o JOIN history h ON h.position_id=o.position_id
LEFT JOIN opening_events e ON e.position_id=o.position_id
LEFT JOIN closing_receipts c ON c.position_id=o.position_id
"#;

fn sample_fetch(ticks: i32) -> FetchRecord {
    FetchRecord {
        account_id: ACCOUNT,
        broker_symbol_id: 1,
        side: "bid",
        kind: "download",
        window_from: Utc.with_ymd_and_hms(2026, 11, 10, 0, 0, 0).unwrap(),
        window_to: Utc.with_ymd_and_hms(2026, 11, 10, 23, 59, 59).unwrap(),
        complete: true,
        ticks,
        pages: 2,
        first_tick: None,
        last_tick: None,
        delta_signs: vec!["add".into()],
        error: None,
        session_ended: false,
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn record_window_unnest_batches_chunks_skips_duplicates_and_preserves_fetch_counts(
    pool: PgPool,
) {
    let base = Utc.with_ymd_and_hms(2026, 11, 10, 9, 0, 0).unwrap();
    let mut ticks: Vec<HistoryTick> = (0..5_005i64)
        .map(|i| HistoryTick {
            ts: base + Duration::milliseconds(i / 2),
            price_e5: 108_500 + (i / 2),
            dup: (i % 2) as i32,
        })
        .collect();

    let (_, stored_initial) = history::record_window(&pool, &sample_fetch(5_005), &ticks)
        .await
        .unwrap();
    assert_eq!(stored_initial, 5_005);

    for extra in 5_005..5_008i64 {
        ticks.push(HistoryTick {
            ts: base + Duration::milliseconds(extra),
            price_e5: 108_500 + extra,
            dup: 0,
        });
    }
    let (_, stored_overlap) = history::record_window(&pool, &sample_fetch(5_008), &ticks)
        .await
        .unwrap();
    assert_eq!(stored_overlap, 3);

    let (_, stored_duplicate) = history::record_window(&pool, &sample_fetch(5_008), &ticks)
        .await
        .unwrap();
    assert_eq!(stored_duplicate, 0);

    let total_rows: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(total_rows, 5_008);

    let fetch_counts: Vec<(i32, i32)> =
        sqlx::query_as("SELECT ticks, stored FROM history_fetches ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(fetch_counts, vec![(5_005, 5_005), (5_008, 3), (5_008, 0)]);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn hot_read_queries_use_targeted_indexes_and_preserve_outcomes_parity(pool: PgPool) {
    store::upsert_account(&pool, ACCOUNT, Environment::Demo)
        .await
        .unwrap();
    ticks::ensure_partitions(&pool, Utc::now().date_naive() - Duration::days(2), 3)
        .await
        .unwrap();

    let now = Utc::now();
    for seq in 0..20i64 {
        let received = if seq < 10 {
            now - Duration::minutes(seq * 3)
        } else {
            now - Duration::hours(2 + seq)
        };
        sqlx::query(
            "INSERT INTO ticks (account_id, session_id, seq, broker_symbol_id, received_at, bid_e5)
             VALUES ($1, 'idx-fixture', $2, 1, $3, 100000)",
        )
        .bind(ACCOUNT)
        .bind(seq)
        .bind(received)
        .execute(&pool)
        .await
        .unwrap();
    }

    let base = now - Duration::days(1);
    for id in 1..=20i64 {
        let key = format!("s3:{ACCOUNT}:1:{id}");
        let open_at = base + Duration::minutes(id * 2);
        let close_at = open_at + Duration::minutes(1);
        let r = if id <= 15 { 2.0 } else { -1.0 };
        let gross = (r * 100.0_f64).round() as i64 + 40;

        sqlx::query(
            "INSERT INTO trading_intents (account_id, entry_key, client_msg_id, kind, symbol_id, side, volume, relative_stop, risk, max_risk)
             VALUES ($1, $2, $2, 'open', 1, 'buy', 100, 100000, 1.0, 1.0)",
        )
        .bind(ACCOUNT)
        .bind(&key)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO trading_events (account_id, entry_key, client_msg_id, payload_type, kind, position_id, order_id, deal_id, symbol_id, volume, raw, received_at)
             VALUES ($1, $2, $2, 2126, 'ORDER_FILLED', $3, $4, $5, 1, 100, ''::bytea, $6)",
        )
        .bind(ACCOUNT)
        .bind(&key)
        .bind(id)
        .bind(id * 10)
        .bind(id * 100)
        .bind(open_at)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO deals (account_id, deal_id, order_id, position_id, broker_symbol_id, volume, filled_volume, trade_side, deal_status, created_at, executed_at, money_digits, is_closing, raw, fetched_at)
             VALUES ($1, $2, $3, $4, 1, 100, 100, 'BUY', 'FILLED', $5, $5, 2, false, ''::bytea, now())",
        )
        .bind(ACCOUNT)
        .bind(id * 100)
        .bind(id * 10)
        .bind(id)
        .bind(open_at)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO deals (account_id, deal_id, order_id, position_id, broker_symbol_id, volume, filled_volume, trade_side, deal_status, created_at, executed_at, money_digits, is_closing, close_gross_profit, close_swap, close_commission, close_pnl_conversion_fee, close_money_digits, close_closed_volume, raw, fetched_at)
             VALUES ($1, $2, $3, $4, 1, 100, 100, 'SELL', 'FILLED', $5, $5, 2, true, $6, 0, -40, 0, 2, 100, ''::bytea, now())",
        )
        .bind(ACCOUNT)
        .bind(id * 100 + 1)
        .bind(id * 10 + 1)
        .bind(id)
        .bind(close_at)
        .bind(gross)
        .execute(&pool)
        .await
        .unwrap();

        let exit_key = format!("s3-broker:{ACCOUNT}:{id}:{}", id * 100 + 1);
        sqlx::query(
            "INSERT INTO strategy3_orders (account_id, entry_key, label, prereg_hash, kind, symbol, symbol_id, position_id, side, decided_at, volume)
             VALUES ($1, $2, 'jiaoyi-s3', 'hash-live', 'broker_exit', 'XAUUSD', 1, $3, 'sell', $4, 100)",
        )
        .bind(ACCOUNT)
        .bind(&exit_key)
        .bind(id)
        .bind(close_at)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO strategy3_fills (account_id, entry_key, label, prereg_hash, kind, symbol, position_id, side, outcome, sent_at, answered_at, volume)
             VALUES ($1, $2, 'jiaoyi-s3', 'hash-live', 'broker_exit', 'XAUUSD', $3, 'sell', 'filled', $4, $4, 100)",
        )
        .bind(ACCOUNT)
        .bind(&exit_key)
        .bind(id)
        .bind(close_at)
        .execute(&pool)
        .await
        .unwrap();
    }

    sqlx::query(
        "INSERT INTO deal_fetches (account_id, window_from, window_to, fetched_at, status, complete)
         VALUES ($1, $2, $3, now(), 'ok', false)",
    )
    .bind(ACCOUNT)
    .bind(now - Duration::days(7))
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    for i in 0..100i64 {
        sqlx::query(
            "INSERT INTO trading_decisions (account_id, symbol, kind, reason, detail)
             VALUES ($1, 'XAUUSD', 's3_skip', 'waiting', $2)",
        )
        .bind(ACCOUNT)
        .bind(json!({ "seq": i }))
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO trading_decisions (account_id, symbol, kind, reason, detail)
         VALUES ($1, 'XAUUSD', 's3_preopen', 'ready', $2)",
    )
    .bind(ACCOUNT)
    .bind(json!({
        "prereg_hash": "hash-live",
        "session": { "open": "2026-11-23T01:00:00Z" }
    }))
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("ANALYZE").execute(&pool).await.unwrap();

    let assessment = goals::assess(&pool, ACCOUNT, "s3").await.unwrap();
    assert_eq!(assessment.measured_closed_positions, 20);
    assert_eq!(
        assessment.state,
        AssessmentState::WindowMeetsNumericReference
    );

    let tick_stats = ticks::stats(&pool).await.unwrap();
    assert_eq!(tick_stats.len(), 1);
    assert_eq!(tick_stats[0].account_id, ACCOUNT);
    assert_eq!(tick_stats[0].last_hour, 10);

    for idx in [
        "deals_account_position",
        "strategy3_fills_account_position",
        "strategy1_fills_account_position",
        "trading_intents_open_prefix",
        "trading_decisions_s3_preopen_lookup",
        "ticks_account_received_at",
    ] {
        let present: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM pg_indexes WHERE schemaname = 'public' AND indexname = $1)",
        )
        .bind(idx)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(present, "missing expected index {idx}");
    }

    let child_tick_indexes: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_inherits WHERE inhparent = 'ticks_account_received_at'::regclass",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(child_tick_indexes >= 3);

    let planner = PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    sqlx::query("BEGIN").execute(&planner).await.unwrap();
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&planner)
        .await
        .unwrap();

    let facts_plan = sqlx::query_scalar::<_, String>(FACTS_EXPLAIN_SQL)
        .bind(ACCOUNT)
        .bind("s3")
        .bind(FILL_KINDS)
        .fetch_all(&planner)
        .await
        .unwrap()
        .join("\n");
    assert!(
        facts_plan.contains("deals_account_position"),
        "FACTS_SQL plan did not use deals_account_position:\n{facts_plan}"
    );
    assert!(
        facts_plan.contains("trading_intents_open_prefix"),
        "FACTS_SQL plan did not use trading_intents_open_prefix:\n{facts_plan}"
    );
    assert!(
        facts_plan.contains("CTE Scan on owned")
            && facts_plan.contains("CTE Scan on grouped_receipts"),
        "FACTS_SQL plan did not materialize multi-consumer CTEs:\n{facts_plan}"
    );

    let money_plan = sqlx::query_scalar::<_, String>(
        "EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
         SELECT position_id, is_closing, filled_volume, executed_at, money_digits,
                close_gross_profit, close_swap, close_commission, close_pnl_conversion_fee,
                close_money_digits
         FROM deals WHERE account_id = $1 AND position_id = ANY($2) AND filled_volume > 0
         ORDER BY position_id, executed_at, deal_id",
    )
    .bind(ACCOUNT)
    .bind(vec![1_i64, 2, 3])
    .fetch_all(&planner)
    .await
    .unwrap()
    .join("\n");
    assert!(
        money_plan.contains("deals_account_position"),
        "outcomes::money plan did not use deals_account_position:\n{money_plan}"
    );

    let preopen_plan = sqlx::query_scalar::<_, String>(
        "EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
         SELECT latest.detail
         FROM unnest($2::text[], $3::text[]) WITH ORDINALITY AS requested(symbol, session_open, ordinal)
         LEFT JOIN LATERAL (
           SELECT detail FROM trading_decisions
           WHERE account_id=$1 AND symbol=requested.symbol AND kind='s3_preopen'
             AND detail->>'prereg_hash'=ANY($4) AND detail->'session'->>'open'=requested.session_open
           ORDER BY id DESC LIMIT 1
         ) latest ON true ORDER BY requested.ordinal",
    )
    .bind(ACCOUNT)
    .bind(vec!["XAUUSD"])
    .bind(vec!["2026-11-23T01:00:00Z"])
    .bind(vec!["hash-live", "hash-18407"])
    .fetch_all(&planner)
    .await
    .unwrap()
    .join("\n");
    assert!(
        preopen_plan.contains("trading_decisions_s3_preopen_lookup"),
        "preopen::stored_many plan did not use trading_decisions_s3_preopen_lookup:\n{preopen_plan}"
    );

    let stats_plan = sqlx::query_scalar::<_, String>(
        "EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
         SELECT a.account_id,
                (SELECT t.received_at FROM ticks t
                 WHERE t.account_id = a.account_id
                   AND t.received_at > now() - interval '2 days'
                 ORDER BY t.received_at DESC LIMIT 1) AS last,
                (SELECT count(*) FROM ticks t
                 WHERE t.account_id = a.account_id
                   AND t.received_at > now() - interval '1 hour') AS n
         FROM accounts a
         WHERE a.enabled
         ORDER BY a.account_id",
    )
    .fetch_all(&planner)
    .await
    .unwrap()
    .join("\n");
    assert!(
        stats_plan.contains("_account_id_received_at_idx"),
        "ticks::stats plan did not use partitioned ticks_account_received_at index:\n{stats_plan}"
    );

    sqlx::query("ROLLBACK").execute(&planner).await.unwrap();
    planner.close().await;
}
