//! Bounded, owner-authenticated evidence inventory. No raw prices, SQL input,
//! broker calls, collection or claim of complete broker truth.
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Semaphore;

const MAX_ROWS: i64 = 500_000;
static READ_GATE: Semaphore = Semaphore::const_new(1);

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub account_id: i64,
    pub symbol_id: i64,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

impl Query {
    pub fn validate(&self, allowed_accounts: &[i64]) -> Result<(), String> {
        if self.account_id <= 0 || !allowed_accounts.contains(&self.account_id) {
            return Err("account is not permitted".into());
        }
        if self.symbol_id <= 0 {
            return Err("symbol_id must be positive".into());
        }
        let span = self.to.signed_duration_since(self.from);
        if span <= chrono::Duration::zero() || span > chrono::Duration::hours(24) {
            return Err("from/to must define a positive interval of at most 24 hours".into());
        }
        Ok(())
    }
}

/// Authentication belongs to the owner route; the account allowlist is checked
/// again here before database acquisition. One aggregate request at a time.
pub async fn report(
    pool: &PgPool,
    allowed_accounts: &[i64],
    query: &Query,
) -> anyhow::Result<Value> {
    query
        .validate(allowed_accounts)
        .map_err(anyhow::Error::msg)?;
    let _permit = READ_GATE
        .try_acquire()
        .map_err(|_| anyhow::anyhow!("quote coverage inventory is busy; retry later"))?;
    tokio::time::timeout(Duration::from_secs(10), read(pool, query))
        .await
        .map_err(|_| anyhow::anyhow!("quote coverage inventory timed out; no completeness claim"))?
}

async fn read(pool: &PgPool, q: &Query) -> anyhow::Result<Value> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL statement_timeout = '8s'")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL lock_timeout = '1s'")
        .execute(&mut *tx)
        .await?;

    let live: Value = sqlx::query_scalar(LIVE)
        .bind(q.account_id)
        .bind(q.symbol_id)
        .bind(q.from)
        .bind(q.to)
        .bind(MAX_ROWS)
        .fetch_one(&mut *tx)
        .await?;
    let history: Value = sqlx::query_scalar(HISTORY)
        .bind(q.account_id)
        .bind(q.symbol_id)
        .bind(q.from)
        .bind(q.to)
        .bind(MAX_ROWS)
        .fetch_one(&mut *tx)
        .await?;
    // A name linked to this symbol by an in-window S3 order is direct evidence.
    // Otherwise use only the last resolution per name at/before the lower bound.
    let mapping = sqlx::query(
        "WITH prior AS (
            SELECT DISTINCT ON(name) name, broker_symbol_id, status
            FROM universe_resolutions WHERE account_id=$1 AND at <= $3
            ORDER BY name, id DESC
         ), names AS (
            SELECT name FROM prior WHERE broker_symbol_id=$2 AND status='resolved'
            UNION
            SELECT symbol FROM strategy3_orders WHERE account_id=$1 AND symbol_id=$2
              AND decided_at >= $3 AND decided_at < $4
         ), bounded AS (SELECT name FROM names ORDER BY name LIMIT 9)
         SELECT COALESCE(array_agg(name), ARRAY[]::text[]) AS names,
           EXISTS(SELECT 1 FROM universe_resolutions r WHERE r.account_id=$1
             AND r.at >= $3 AND r.at < $4 AND r.name IN(SELECT name FROM bounded)
             AND (r.status <> 'resolved' OR r.broker_symbol_id <> $2)) AS conflict
         FROM bounded",
    )
    .bind(q.account_id)
    .bind(q.symbol_id)
    .bind(q.from)
    .bind(q.to)
    .fetch_one(&mut *tx)
    .await?;
    let names: Vec<String> = mapping.get("names");
    let conflict: bool = mapping.get("conflict");
    let marks = if !names.is_empty() && names.len() <= 8 && !conflict {
        sqlx::query_scalar::<_, Value>(
            "SELECT jsonb_build_object(
               'rows',count(*),'distinct_recorded_marks',count(DISTINCT detail->>'decision_mark'),
               'with_two_numeric_sides',count(*) FILTER (
                 WHERE jsonb_typeof(detail->'bid')='number' AND jsonb_typeof(detail->'ask')='number'),
               'first_recorded_at',min(at),'last_recorded_at',max(at))
             FROM trading_decisions WHERE account_id=$1 AND symbol=ANY($2)
               AND at >= $3 AND at < $4 AND kind='s3_mark_decision'",
        )
        .bind(q.account_id)
        .bind(&names)
        .bind(q.from)
        .bind(q.to)
        .fetch_one(&mut *tx)
        .await?
    } else {
        Value::Null
    };
    tx.rollback().await?;
    Ok(json!({
        "generated_at": Utc::now(), "account_id":q.account_id, "symbol_id":q.symbol_id,
        "from_inclusive":q.from, "to_exclusive":q.to, "max_rows_per_tick_source":MAX_ROWS,
        "scope":"aggregate stored evidence availability; not broker completeness, signal validity or financial acceptance",
        "live":live, "history":history,
        "recorded_strategy3_marks":marks,
        "mark_symbol_names":names,
        "mark_mapping_state":if conflict || names.len()>8 {"ambiguous"} else if names.is_empty() {"unavailable"} else {"linked"},
        "notes":[
            "Live times are local received_at; historical times are broker history ts. They are not interchangeable information-availability clocks.",
            "A tick may update one side only. Side counts and observed gaps do not reconstruct a continuous executable two-sided quote.",
            "Calendar windows are fully contained UTC half-hour five-second windows, not session eligibility or all strategy opportunities.",
            "Recorded S3 decisions count stored mark records by at; no record while holding a position does not prove a missed opportunity.",
            "No rows or null gaps mean unavailable evidence, not zero market movement. A row cap or timeout prevents a complete inventory claim.",
            "Complete historical fetch ranges do not prove those data were available to the trader before fetched_at."
        ]
    }))
}

const LIVE: &str = r#"
WITH sampled AS MATERIALIZED (
 SELECT received_at, bid_e5, ask_e5, session_id, broker_timestamp
 FROM ticks WHERE account_id=$1 AND broker_symbol_id=$2
   AND received_at >= $3 AND received_at < $4
 ORDER BY received_at, session_id, seq LIMIT ($5 + 1)
), scoped AS MATERIALIZED (SELECT * FROM sampled LIMIT $5),
events AS (
 SELECT 'all' AS side,received_at AS at FROM scoped
 UNION ALL SELECT 'bid',received_at FROM scoped WHERE bid_e5>0
 UNION ALL SELECT 'ask',received_at FROM scoped WHERE ask_e5>0
), gaps AS (
 SELECT side,at,EXTRACT(EPOCH FROM at-lag(at) OVER(PARTITION BY side ORDER BY at))*1000 AS gap_ms
 FROM events
), summary AS (
 SELECT side,count(*) AS updates,min(at) AS first_at,max(at) AS last_at,
   max(gap_ms)::double precision AS max_observed_gap_ms,
   count(*) FILTER(WHERE gap_ms>6000) AS observed_gaps_over_six_seconds
 FROM gaps GROUP BY side
), windows AS (
 SELECT m FROM generate_series(
   date_bin(interval '30 minutes',$3::timestamptz,timestamptz '2000-01-01 00:00:00+00'),
   $4::timestamptz,interval '30 minutes') m
 WHERE m >= $3 AND m + interval '5 seconds' <= $4
), marks AS (
 SELECT m AS mark_at,count(t.received_at) AS updates,
   count(t.bid_e5) FILTER(WHERE t.bid_e5>0) AS bid_updates,
   count(t.ask_e5) FILTER(WHERE t.ask_e5>0) AS ask_updates,
   min(t.received_at) AS first_received_at
 FROM windows w LEFT JOIN scoped t ON t.received_at>=w.m AND t.received_at<w.m+interval '5 seconds'
 GROUP BY m ORDER BY m
)
SELECT jsonb_build_object(
 'rows',(SELECT count(*) FROM scoped),'truncated',(SELECT count(*)>$5 FROM sampled),
 'recording_sessions',(SELECT count(DISTINCT session_id) FROM scoped),
 'both_side_updates',(SELECT count(*) FROM scoped WHERE bid_e5>0 AND ask_e5>0),
 'invalid_side_updates',(SELECT count(*) FROM scoped WHERE bid_e5<=0 OR ask_e5<=0),
 'missing_broker_timestamp',(SELECT count(*) FROM scoped WHERE broker_timestamp IS NULL),
 'series',COALESCE((SELECT jsonb_agg(to_jsonb(s)||jsonb_build_object(
   'gap_from_interval_start_ms',EXTRACT(EPOCH FROM first_at-$3::timestamptz)*1000,
   'gap_to_interval_end_ms',EXTRACT(EPOCH FROM $4::timestamptz-last_at)*1000)) FROM summary s),'[]'::jsonb),
 'utc_half_hour_windows',CASE WHEN (SELECT count(*)>$5 FROM sampled) THEN NULL
   ELSE COALESCE((SELECT jsonb_agg(to_jsonb(m)) FROM marks m),'[]'::jsonb) END)
"#;

const HISTORY: &str = r#"
WITH sampled AS MATERIALIZED (
 SELECT side,ts FROM ticks_history WHERE account_id=$1 AND broker_symbol_id=$2
   AND ts >= $3 AND ts < $4 ORDER BY ts,side,dup LIMIT ($5+1)
), scoped AS MATERIALIZED (SELECT * FROM sampled LIMIT $5), gaps AS (
 SELECT side,ts,EXTRACT(EPOCH FROM ts-lag(ts) OVER(PARTITION BY side ORDER BY ts))*1000 AS gap_ms
 FROM scoped
), summary AS (
 SELECT side,count(*) AS updates,min(ts) AS first_at,max(ts) AS last_at,
   max(gap_ms)::double precision AS max_observed_gap_ms,
   count(*) FILTER(WHERE gap_ms>6000) AS observed_gaps_over_six_seconds
 FROM gaps GROUP BY side
), fetches AS (
 SELECT side,count(*) AS overlapping_fetches,
   count(*) FILTER(WHERE status='ok' AND complete) AS completed_fetches,
   count(*) FILTER(WHERE status<>'ok' OR NOT complete) AS failed_or_incomplete_fetches,
   min(fetched_at) AS first_available_at,max(fetched_at) AS last_available_at,
   COALESCE((range_agg(tstzrange(window_from,window_to,'[]')) FILTER(WHERE status='ok' AND complete))
     @> tstzrange($3,$4,'[)'),false) AS completed_fetch_range_covers_interval
 FROM history_fetches WHERE account_id=$1 AND broker_symbol_id=$2 AND kind='download'
   AND window_from < $4 AND window_to >= $3 GROUP BY side
)
SELECT jsonb_build_object(
 'rows',(SELECT count(*) FROM scoped),'truncated',(SELECT count(*)>$5 FROM sampled),
 'series',COALESCE((SELECT jsonb_agg(to_jsonb(s)||jsonb_build_object(
   'gap_from_interval_start_ms',EXTRACT(EPOCH FROM first_at-$3::timestamptz)*1000,
   'gap_to_interval_end_ms',EXTRACT(EPOCH FROM $4::timestamptz-last_at)*1000)) FROM summary s),'[]'::jsonb),
 'fetches',COALESCE((SELECT jsonb_agg(to_jsonb(f)) FROM fetches f),'[]'::jsonb))
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> Query {
        Query {
            account_id: 42,
            symbol_id: 7,
            from: "2026-10-29T06:59:54Z".parse().unwrap(),
            to: "2026-10-29T07:01:00Z".parse().unwrap(),
        }
    }

    #[test]
    fn validates_scope_ids_and_interval_without_database_access() {
        let q = query();
        assert!(q.validate(&[42]).is_ok());
        assert!(q.validate(&[43]).is_err());
        assert!(Query {
            symbol_id: 0,
            ..q.clone()
        }
        .validate(&[42])
        .is_err());
        assert!(Query {
            account_id: -1,
            ..q.clone()
        }
        .validate(&[-1])
        .is_err());
        assert!(Query {
            to: q.from,
            ..q.clone()
        }
        .validate(&[42])
        .is_err());
        assert!(Query {
            to: q.from - chrono::Duration::seconds(1),
            ..q.clone()
        }
        .validate(&[42])
        .is_err());
        assert!(Query {
            to: q.from + chrono::Duration::hours(24),
            ..q.clone()
        }
        .validate(&[42])
        .is_ok());
        assert!(Query {
            to: q.from + chrono::Duration::hours(24) + chrono::Duration::milliseconds(1),
            ..q
        }
        .validate(&[42])
        .is_err());
    }

    async fn insert_tick(
        pool: &PgPool,
        account: i64,
        symbol: i64,
        seq: i64,
        at: &str,
        bid: Option<i64>,
        ask: Option<i64>,
    ) {
        sqlx::query("INSERT INTO ticks(account_id,broker_symbol_id,session_id,seq,received_at,bid_e5,ask_e5) VALUES($1,$2,'coverage-fixture',$3,$4,$5,$6)")
            .bind(account).bind(symbol).bind(seq).bind(at.parse::<DateTime<Utc>>().unwrap()).bind(bid).bind(ask).execute(pool).await.unwrap();
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn aggregate_scopes_sides_gaps_marks_and_historical_availability(pool: PgPool) {
        let q = query();
        jiaoyi_store::ticks::ensure_partitions(&pool, q.from.date_naive(), 1)
            .await
            .unwrap();
        insert_tick(&pool, 42, 7, 0, "2026-10-29T06:59:54Z", Some(100), None).await;
        insert_tick(&pool, 42, 7, 1, "2026-10-29T07:00:01Z", None, Some(101)).await;
        insert_tick(
            &pool,
            42,
            7,
            2,
            "2026-10-29T07:00:03Z",
            Some(100),
            Some(101),
        )
        .await;
        insert_tick(
            &pool,
            42,
            7,
            3,
            "2026-10-29T07:01:00Z",
            Some(100),
            Some(101),
        )
        .await; // exclusive upper bound
        insert_tick(
            &pool,
            43,
            7,
            4,
            "2026-10-29T07:00:02Z",
            Some(100),
            Some(101),
        )
        .await;
        insert_tick(
            &pool,
            42,
            8,
            5,
            "2026-10-29T07:00:02Z",
            Some(100),
            Some(101),
        )
        .await;
        sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,side,decided_at) VALUES(42,'coverage-open','jiaoyi-s3','fixture','open','BTCUSD',7,'buy','2026-10-29T07:00:02Z')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO trading_decisions(account_id,at,symbol,kind,reason,detail) VALUES(42,'2026-10-29T07:00:01Z','BTCUSD','s3_mark_decision','fixture','{\"decision_mark\":\"2026-10-29T07:00:00Z\",\"bid\":1,\"ask\":1.01}')").execute(&pool).await.unwrap();
        jiaoyi_store::history::ensure_partitions(&pool, q.from, q.to)
            .await
            .unwrap();
        sqlx::query("INSERT INTO history_fetches(account_id,broker_symbol_id,side,kind,window_from,window_to,fetched_at,status,complete,ticks) VALUES(42,7,'bid','download',$1,$2,$2+interval '1 day','ok',true,2)")
            .bind(q.from).bind(q.to).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO ticks_history(account_id,broker_symbol_id,side,ts,price_e5,fetch_id) SELECT 42,7,'bid',t,100,1 FROM unnest(ARRAY[$1::timestamptz,$2::timestamptz]) t")
            .bind(q.from).bind(q.from+chrono::Duration::seconds(10)).execute(&pool).await.unwrap();
        let r = read(&pool, &q).await.unwrap();
        assert_eq!(r["live"]["rows"], 3);
        assert_eq!(r["live"]["truncated"], false);
        assert_eq!(r["live"]["both_side_updates"], 1);
        let series = r["live"]["series"].as_array().unwrap();
        let bid = series.iter().find(|s| s["side"] == "bid").unwrap();
        assert_eq!(bid["updates"], 2);
        assert_eq!(bid["max_observed_gap_ms"], 9000.0);
        assert_eq!(bid["observed_gaps_over_six_seconds"], 1);
        assert_eq!(
            r["live"]["utc_half_hour_windows"].as_array().unwrap().len(),
            1
        );
        assert_eq!(r["live"]["utc_half_hour_windows"][0]["updates"], 2);
        assert_eq!(r["recorded_strategy3_marks"]["rows"], 1);
        assert_eq!(r["recorded_strategy3_marks"]["distinct_recorded_marks"], 1);
        assert_eq!(r["history"]["rows"], 2);
        assert_eq!(
            r["history"]["fetches"][0]["completed_fetch_range_covers_interval"],
            true
        );
        assert_eq!(
            r["history"]["fetches"][0]["first_available_at"]
                .as_str()
                .unwrap()
                .parse::<DateTime<Utc>>()
                .unwrap(),
            q.to + chrono::Duration::days(1)
        );
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn absent_sources_remain_empty_and_small_row_cap_is_explicit(pool: PgPool) {
        let q = query();
        let absent = read(&pool, &q).await.unwrap();
        assert_eq!(absent["live"]["rows"], 0);
        assert_eq!(absent["live"]["series"], json!([]));
        assert!(absent["recorded_strategy3_marks"].is_null());
        jiaoyi_store::ticks::ensure_partitions(&pool, q.from.date_naive(), 1)
            .await
            .unwrap();
        insert_tick(&pool, 42, 7, 0, "2026-10-29T07:00:01Z", Some(100), None).await;
        insert_tick(&pool, 42, 7, 1, "2026-10-29T07:00:02Z", None, Some(101)).await;
        let capped: Value = sqlx::query_scalar(LIVE)
            .bind(q.account_id)
            .bind(q.symbol_id)
            .bind(q.from)
            .bind(q.to)
            .bind(1_i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(capped["rows"], 1);
        assert_eq!(capped["truncated"], true);
        assert!(capped["utc_half_hour_windows"].is_null());
    }
}
