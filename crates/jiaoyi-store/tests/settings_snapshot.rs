//! Settings snapshots use one fresh PostgreSQL statement, with the same
//! global/account precedence and atomic version/value view as before.
use std::sync::Arc;

use jiaoyi_core::settings::{Scope, Snapshot};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::{PgPool, Row};

const ACCOUNT: i64 = 49236051;

async fn reference(pool: &PgPool, account_id: Option<i64>) -> Snapshot {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await
        .unwrap();
    let version = sqlx::query_scalar("SELECT version FROM settings_version")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let global = sqlx::query("SELECT key,value FROM settings WHERE account_id IS NULL")
        .fetch_all(&mut *tx)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.get("key"), r.get("value")))
        .collect();
    let account = match account_id {
        Some(id) => sqlx::query("SELECT key,value FROM settings WHERE account_id=$1")
            .bind(id)
            .fetch_all(&mut *tx)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.get("key"), r.get("value"), id))
            .collect(),
        None => vec![],
    };
    tx.commit().await.unwrap();
    Snapshot::merge(version, global, account)
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn snapshot_matches_legacy_scope_and_reads_fresh_committed_changes(pool: PgPool) {
    settings::change(
        &pool,
        "bars.tick.updates_per_bar",
        Some(ACCOUNT),
        Some(json!(250)),
        "owner",
        "test override",
    )
    .await
    .unwrap();
    // JSON null is a stored value rather than an absent left-join row.
    sqlx::query("UPDATE settings SET value='null'::jsonb WHERE key='ticks.retention_days' AND account_id IS NULL")
        .execute(&pool).await.unwrap();
    for account in [None, Some(ACCOUNT), Some(99999999)] {
        assert_eq!(
            settings::snapshot(&pool, account).await.unwrap(),
            reference(&pool, account).await
        );
    }
    let before = settings::snapshot(&pool, Some(ACCOUNT)).await.unwrap();
    assert_eq!(
        before.scope_of("bars.tick.updates_per_bar"),
        Some(Scope::Account(ACCOUNT))
    );
    let version = settings::change(
        &pool,
        "bars.tick.updates_per_bar",
        Some(ACCOUNT),
        None,
        "owner",
        "test remove override",
    )
    .await
    .unwrap();
    let after = settings::snapshot(&pool, Some(ACCOUNT)).await.unwrap();
    assert_eq!(after.version, version);
    assert!(after.version > before.version);
    assert_eq!(after.integer("bars.tick.updates_per_bar").unwrap(), 100);
    assert_eq!(
        after.scope_of("bars.tick.updates_per_bar"),
        Some(Scope::Global)
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn empty_settings_and_missing_version_keep_their_existing_meaning(pool: PgPool) {
    sqlx::query("DELETE FROM settings")
        .execute(&pool)
        .await
        .unwrap();
    for account in [None, Some(ACCOUNT)] {
        let actual = settings::snapshot(&pool, account).await.unwrap();
        assert_eq!(actual, reference(&pool, account).await);
        assert!(actual.values.is_empty());
    }
    sqlx::query("DELETE FROM settings_version")
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        settings::snapshot(&pool, None).await,
        Err(sqlx::Error::RowNotFound)
    ));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn account_snapshot_needs_one_prepared_settings_read(pool: PgPool) {
    // An isolated one-connection pool makes PostgreSQL's prepared statement
    // inventory an actual read-count check without a timing threshold.
    let measured = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    settings::snapshot(&measured, Some(ACCOUNT)).await.unwrap();
    let statements: Vec<String> = sqlx::query_scalar(
        "SELECT statement FROM pg_prepared_statements WHERE statement LIKE '%FROM settings%' AND statement NOT LIKE '%pg_prepared_statements%'")
        .fetch_all(&measured).await.unwrap();
    assert_eq!(
        statements.len(),
        1,
        "one version/global/override read, got {statements:?}"
    );
    measured.close().await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn concurrent_commits_never_mix_version_global_and_account_values(pool: PgPool) {
    let initial: i64 = sqlx::query_scalar("SELECT version FROM settings_version")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE settings SET value=to_jsonb($1::bigint) WHERE key='ticks.retention_days' AND account_id IS NULL")
        .bind(initial).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO settings(key,account_id,value,version) VALUES('bars.tick.updates_per_bar',$1,to_jsonb($2::bigint),$2)")
        .bind(ACCOUNT).bind(initial).execute(&pool).await.unwrap();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let writer_pool = pool.clone();
    let writer_barrier = barrier.clone();
    let writer = tokio::spawn(async move {
        writer_barrier.wait().await;
        for generation in initial + 1..=initial + 40 {
            let mut tx = writer_pool.begin().await.unwrap();
            sqlx::query("UPDATE settings_version SET version=$1")
                .bind(generation)
                .execute(&mut *tx)
                .await
                .unwrap();
            sqlx::query("UPDATE settings SET value=to_jsonb($1::bigint) WHERE (key='ticks.retention_days' AND account_id IS NULL) OR (key='bars.tick.updates_per_bar' AND account_id=$2)")
                .bind(generation).bind(ACCOUNT).execute(&mut *tx).await.unwrap();
            tx.commit().await.unwrap();
            tokio::task::yield_now().await;
        }
    });
    barrier.wait().await;
    for _ in 0..80 {
        let snapshot = settings::snapshot(&pool, Some(ACCOUNT)).await.unwrap();
        assert_eq!(
            snapshot.integer("ticks.retention_days").unwrap(),
            snapshot.version
        );
        assert_eq!(
            snapshot.integer("bars.tick.updates_per_bar").unwrap(),
            snapshot.version
        );
    }
    writer.await.unwrap();
}
