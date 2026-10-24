//! The recorded spot quote stream (`ticks`, partitioned by UTC day).

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickRow {
    pub account_id: i64,
    pub broker_symbol_id: i64,
    pub received_at: DateTime<Utc>,
    pub broker_timestamp: Option<i64>,
    pub bid_e5: Option<i64>,
    pub ask_e5: Option<i64>,
}

/// Create the partitions for `from` and the following `days - 1` days.
pub async fn ensure_partitions(
    pool: &PgPool,
    from: NaiveDate,
    days: u32,
) -> Result<Vec<String>, sqlx::Error> {
    let mut out = Vec::new();
    for i in 0..days {
        let day = from + chrono::Duration::days(i as i64);
        let name: String = sqlx::query_scalar("SELECT jiaoyi_ensure_tick_partition($1)")
            .bind(day)
            .fetch_one(pool)
            .await?;
        out.push(name);
    }
    Ok(out)
}

/// Drop partitions older than the retention. Returns the dropped names.
pub async fn drop_expired(
    pool: &PgPool,
    today: NaiveDate,
    keep_days: i32,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT jiaoyi_drop_tick_partitions($1, $2)")
        .bind(today)
        .bind(keep_days)
        .fetch_all(pool)
        .await
}

/// Insert a batch of ticks.
pub async fn insert(pool: &PgPool, rows: &[TickRow]) -> Result<u64, sqlx::Error> {
    let mut n = 0;
    for chunk in rows.chunks(5000) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO ticks (account_id, broker_symbol_id, received_at, broker_timestamp, bid_e5, ask_e5) ",
        );
        qb.push_values(chunk, |mut b, t| {
            b.push_bind(t.account_id)
                .push_bind(t.broker_symbol_id)
                .push_bind(t.received_at)
                .push_bind(t.broker_timestamp)
                .push_bind(t.bid_e5)
                .push_bind(t.ask_e5);
        });
        n += qb.build().execute(pool).await?.rows_affected();
    }
    Ok(n)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TickStats {
    pub account_id: i64,
    pub last_received_at: Option<DateTime<Utc>>,
    pub last_hour: i64,
}

/// Per account: the latest tick and the count in the last hour.
pub async fn stats(pool: &PgPool) -> Result<Vec<TickStats>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT a.account_id,
                (SELECT max(received_at) FROM ticks t
                  WHERE t.account_id = a.account_id AND t.received_at > now() - interval '2 days') AS last,
                (SELECT count(*) FROM ticks t
                  WHERE t.account_id = a.account_id AND t.received_at > now() - interval '1 hour') AS n
         FROM accounts a WHERE a.enabled ORDER BY a.account_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| TickStats {
            account_id: r.get("account_id"),
            last_received_at: r.get("last"),
            last_hour: r.get("n"),
        })
        .collect())
}

/// Resolved, stored symbols of an account's latest resolution (ids).
pub async fn resolved_symbol_ids(pool: &PgPool, account_id: i64) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT l.broker_symbol_id FROM (
             SELECT DISTINCT ON (name) name, status, broker_symbol_id
             FROM universe_resolutions WHERE account_id = $1 ORDER BY name, id DESC
         ) l
         JOIN symbols s ON s.account_id = $1 AND s.broker_symbol_id = l.broker_symbol_id
         WHERE l.status = 'resolved'
         ORDER BY l.broker_symbol_id",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}
