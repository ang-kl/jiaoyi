//! Exact read-only tick statistics, including retained/non-universe symbols.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::{self as store, ticks};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

const LEGACY_STATS: &str = "SELECT a.account_id,
    (SELECT max(received_at) FROM ticks t WHERE t.account_id=a.account_id
        AND t.received_at>now()-interval '2 days') AS last,
    (SELECT count(*) FROM ticks t WHERE t.account_id=a.account_id
        AND t.received_at>now()-interval '1 hour') AS n
    FROM accounts a WHERE a.enabled ORDER BY a.account_id";

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn aggregate_matches_legacy_cutoffs_future_ticks_and_enabled_account_scope(pool: PgPool) {
    const A: i64 = 7001;
    for account in A..=A + 5 {
        store::upsert_account(&pool, account, Environment::Demo)
            .await
            .unwrap();
    }
    sqlx::query("UPDATE accounts SET enabled=false WHERE account_id=$1")
        .bind(A + 3)
        .execute(&pool)
        .await
        .unwrap();
    ticks::ensure_partitions(&pool, Utc::now().date_naive() - Duration::days(3), 5)
        .await
        .unwrap();
    // A dedicated connection keeps server NOW() fixed for exact boundary
    // inserts and both reads. No application-clock substitute is introduced.
    let fixed = PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    sqlx::query("BEGIN").execute(&fixed).await.unwrap();
    let at: DateTime<Utc> = sqlx::query_scalar("SELECT now()")
        .fetch_one(&fixed)
        .await
        .unwrap();
    let rows = [
        (A, at - Duration::days(3), 1),
        (A, at - Duration::days(2), 1),
        (A, at - Duration::hours(1), 1),
        (A, at - Duration::hours(1) + Duration::microseconds(1), 1),
        (A, at - Duration::minutes(30), 999), // no stored symbol or current universe membership
        (A, at + Duration::hours(2), 999),    // future rows were never upper-bounded
        (A + 2, at - Duration::days(2), 1),   // strict two-day boundary, not >=
        (A + 3, at + Duration::hours(3), 1),  // disabled account
        (A + 4, at - Duration::hours(1), 1),  // strict one-hour boundary, not >=
        (A + 5, at - Duration::days(2) + Duration::microseconds(1), 1),
        (A + 99, at, 999), // ticks alone cannot create an enabled account
    ];
    for (seq, (account, received, symbol)) in rows.into_iter().enumerate() {
        sqlx::query("INSERT INTO ticks(account_id,session_id,seq,broker_symbol_id,received_at,bid_e5) VALUES($1,'stats-fixture',$2,$3,$4,100000)")
            .bind(account).bind(seq as i64).bind(symbol as i64).bind(received)
            .execute(&fixed).await.unwrap();
    }
    let legacy = sqlx::query(LEGACY_STATS)
        .fetch_all(&fixed)
        .await
        .unwrap()
        .into_iter()
        .map(|r| ticks::TickStats {
            account_id: r.get("account_id"),
            last_received_at: r.get("last"),
            last_hour: r.get("n"),
        })
        .collect::<Vec<_>>();
    let actual = ticks::stats(&fixed).await.unwrap();
    assert_eq!(actual, legacy);
    assert_eq!(
        actual.iter().map(|r| r.account_id).collect::<Vec<_>>(),
        vec![A, A + 1, A + 2, A + 4, A + 5]
    );
    assert_eq!(actual[0].last_received_at, Some(at + Duration::hours(2)));
    assert_eq!(actual[0].last_hour, 3);
    assert_eq!((actual[1].last_received_at, actual[1].last_hour), (None, 0));
    assert_eq!((actual[2].last_received_at, actual[2].last_hour), (None, 0));
    assert_eq!(
        (actual[3].last_received_at, actual[3].last_hour),
        (Some(at - Duration::hours(1)), 0)
    );
    assert_eq!(
        (actual[4].last_received_at, actual[4].last_hour),
        (Some(at - Duration::days(2) + Duration::microseconds(1)), 0)
    );
    sqlx::query("ROLLBACK").execute(&fixed).await.unwrap();
    fixed.close().await;
}
