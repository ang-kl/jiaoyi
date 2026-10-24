//! The recorded spot quote stream (`ticks`, partitioned by UTC day).

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickRow {
    pub account_id: i64,
    /// The recording session that received the quote.
    pub session_id: String,
    /// The quote's place in its session, from 0.
    pub seq: i64,
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

/// Rows per INSERT statement (8 binds each, well under Postgres's limit).
pub const INSERT_CHUNK: usize = 5000;

/// Insert one flush of ticks. Returns the rows newly inserted.
pub async fn insert(pool: &PgPool, rows: &[TickRow]) -> Result<u64, sqlx::Error> {
    insert_chunked(pool, rows, INSERT_CHUNK).await
}

/// Insert in chunks of `chunk` rows, all in ONE transaction: a failure in any
/// chunk stores none of the flush, so a retry of the whole flush is safe. As
/// a second guard, a row whose (account, session, seq, received_at) is
/// already stored is skipped (`ON CONFLICT DO NOTHING`).
pub async fn insert_chunked(
    pool: &PgPool,
    rows: &[TickRow],
    chunk: usize,
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut n = 0;
    for part in rows.chunks(chunk.max(1)) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO ticks (account_id, session_id, seq, broker_symbol_id, received_at, broker_timestamp, bid_e5, ask_e5) ",
        );
        qb.push_values(part, |mut b, t| {
            b.push_bind(t.account_id)
                .push_bind(&t.session_id)
                .push_bind(t.seq)
                .push_bind(t.broker_symbol_id)
                .push_bind(t.received_at)
                .push_bind(t.broker_timestamp)
                .push_bind(t.bid_e5)
                .push_bind(t.ask_e5);
        });
        qb.push(" ON CONFLICT (account_id, session_id, seq, received_at) DO NOTHING");
        n += qb.build().execute(&mut *tx).await?.rows_affected();
    }
    tx.commit().await?;
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

/// The symbols to record for an account: those resolved in the account's
/// latest load run, whose details are stored, and whose name is still in
/// `names` (the current `universe.symbols` setting). Earlier runs do not count,
/// so a name removed from the universe is no longer recorded.
pub async fn resolved_symbol_ids(
    pool: &PgPool,
    account_id: i64,
    names: &[String],
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT r.broker_symbol_id FROM universe_resolutions r
         JOIN symbols s ON s.account_id = r.account_id AND s.broker_symbol_id = r.broker_symbol_id
         WHERE r.account_id = $1
           AND r.status = 'resolved'
           AND r.name = ANY($2)
           AND r.load_run_id = (SELECT max(load_run_id) FROM universe_resolutions WHERE account_id = $1)
         ORDER BY r.broker_symbol_id",
    )
    .bind(account_id)
    .bind(names)
    .fetch_all(pool)
    .await
}
