//! Storage against a real PostgreSQL. `#[sqlx::test]` creates a fresh database
//! per test from `DATABASE_URL` and applies the repository's migrations to it.
//! Without `DATABASE_URL` these tests fail; they do not skip.

use chrono::{NaiveDate, TimeZone, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::settings::Scope;
use jiaoyi_core::universe::{resolve, AssetClass, ListedSymbol};
use jiaoyi_store::settings::{self, ChangeError};
use jiaoyi_store::ticks::{self, TickRow};
use jiaoyi_store::{self as store, BarRow, SymbolRow};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::PgPool;

const A: i64 = 49236051;

async fn account_and_symbol(pool: &PgPool) {
    store::upsert_account(pool, A, Environment::Demo)
        .await
        .unwrap();
    store::upsert_symbol(
        pool,
        &SymbolRow {
            account_id: A,
            broker_symbol_id: 1,
            name: "EURUSD".into(),
            asset_class: "fx".into(),
            listed_enabled: Some(true),
            trading_mode: Some("ENABLED".into()),
            digits: 5,
            pip_position: 4,
            lot_size: Some(10_000_000),
            min_volume: Some(100_000),
            max_volume: None,
            volume_step: Some(100_000),
            schedule_time_zone: None,
            measurement_units: None,
            null_reasons: json!({"max_volume": "broker sent no maxVolume"}),
            raw: vec![8, 1],
            read_at: Utc::now(),
        },
    )
    .await
    .unwrap();
}

fn bar(d: NaiveDate, o: i64, h: i64, l: i64, c: i64) -> BarRow {
    BarRow {
        bar_date: d,
        bar_open_utc: d.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        open: Decimal::new(o, 5),
        high: Decimal::new(h, 5),
        low: Decimal::new(l, 5),
        close: Decimal::new(c, 5),
        tick_volume: 10,
    }
}

fn day(m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, m, d).unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn migrations_create_every_phase1_table(pool: PgPool) {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables WHERE table_schema = 'public' ORDER BY 1",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    for t in [
        "accounts",
        "account_refusals",
        "daily_bars",
        "load_runs",
        "oauth_tokens",
        "reconcile_events",
        "setting_definitions",
        "settings",
        "settings_changes",
        "settings_version",
        "symbol_costs",
        "symbols",
        "ticks",
        "universe_resolutions",
    ] {
        assert!(tables.iter().any(|x| x == t), "{t} missing from {tables:?}");
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn append_only_tables_refuse_update_and_delete(pool: PgPool) {
    account_and_symbol(&pool).await;
    store::record_refusal(&pool, "99999999", "not on JIAOYI_ALLOWED_ACCOUNTS", "test")
        .await
        .unwrap();
    store::record_reconcile(&pool, Some(A), "k", "s", json!({}), "r")
        .await
        .unwrap();
    for (table, col) in [
        ("account_refusals", "reason"),
        ("reconcile_events", "kind"),
        ("settings_changes", "reason"),
    ] {
        let up = sqlx::query(&format!("UPDATE {table} SET {col} = 'x'"))
            .execute(&pool)
            .await;
        assert!(
            up.unwrap_err().to_string().contains("append-only"),
            "{table} accepted an UPDATE"
        );
        let del = sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await;
        assert!(
            del.unwrap_err().to_string().contains("append-only"),
            "{table} accepted a DELETE"
        );
    }
    store::upsert_daily_bars(
        &pool,
        A,
        1,
        None,
        &[bar(day(3, 1), 100, 110, 90, 105)],
        Utc::now(),
    )
    .await
    .unwrap();
    let del = sqlx::query("DELETE FROM daily_bars").execute(&pool).await;
    assert!(del.unwrap_err().to_string().contains("append-only"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn bars_insert_then_unchanged_then_a_broker_revision_is_recorded(pool: PgPool) {
    account_and_symbol(&pool).await;
    let first = vec![
        bar(day(3, 1), 100, 110, 90, 105),
        bar(day(3, 4), 105, 112, 101, 111),
    ];
    let u = store::upsert_daily_bars(&pool, A, 1, None, &first, Utc::now())
        .await
        .unwrap();
    assert_eq!((u.inserted, u.changed, u.unchanged), (2, 0, 0));
    let u = store::upsert_daily_bars(&pool, A, 1, None, &first, Utc::now())
        .await
        .unwrap();
    assert_eq!((u.inserted, u.changed, u.unchanged), (0, 0, 2));

    // The broker now reports a different close for 03-04.
    let revised = vec![bar(day(3, 4), 105, 112, 101, 110)];
    let u = store::upsert_daily_bars(&pool, A, 1, None, &revised, Utc::now())
        .await
        .unwrap();
    assert_eq!(u.changed, 1);
    let stored = store::bars(&pool, A, 1).await.unwrap();
    assert_eq!(stored[1].close, Decimal::new(110, 5));
    let ev: (String, serde_json::Value) = sqlx::query_as(
        "SELECT kind, detail FROM reconcile_events WHERE kind = 'daily_bar_changed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ev.1["stored"]["close"], json!("0.00111"));
    assert_eq!(ev.1["broker"]["close"], json!("0.00110"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn two_bars_on_one_date_are_both_refused_and_recorded(pool: PgPool) {
    account_and_symbol(&pool).await;
    let mut late = bar(day(3, 1), 100, 110, 90, 105);
    late.bar_open_utc = Utc.with_ymd_and_hms(2024, 3, 1, 22, 0, 0).unwrap();
    let u = store::upsert_daily_bars(
        &pool,
        A,
        1,
        None,
        &[bar(day(3, 1), 100, 110, 90, 105), late],
        Utc::now(),
    )
    .await
    .unwrap();
    assert_eq!((u.inserted, u.refused), (0, 2));
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM reconcile_events WHERE kind = 'bar_date_collision'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_impossible_bar_is_refused_by_the_database(pool: PgPool) {
    account_and_symbol(&pool).await;
    // high below low
    let r = store::upsert_daily_bars(
        &pool,
        A,
        1,
        None,
        &[bar(day(3, 1), 100, 80, 90, 95)],
        Utc::now(),
    )
    .await;
    assert!(r.is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_universe_miss_is_recorded_with_its_reason(pool: PgPool) {
    store::upsert_account(&pool, A, Environment::Demo)
        .await
        .unwrap();
    let run = store::start_run(&pool, A).await.unwrap();
    let listed = vec![ListedSymbol {
        symbol_id: 7,
        name: Some("UK Oil".into()),
        enabled: Some(true),
    }];
    let r = resolve("UKOIL", &listed);
    store::record_resolution(&pool, run, A, "UKOIL", AssetClass::Energy, &r)
        .await
        .unwrap();
    let rows = store::latest_resolutions(&pool, A).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "missing");
    assert_eq!(rows[0].broker_symbol_id, None);
    assert_eq!(
        rows[0].reason.as_deref(),
        Some("no symbol named exactly \"UKOIL\" on this account")
    );
    // The database refuses a miss without a reason.
    let bad = sqlx::query(
        "INSERT INTO universe_resolutions (load_run_id, account_id, name, asset_class, status)
         VALUES ($1, $2, 'X', 'fx', 'missing')",
    )
    .bind(run)
    .bind(A)
    .execute(&pool)
    .await;
    assert!(bad.is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn seeded_settings_are_the_rulebook_values_at_version_17(pool: PgPool) {
    let s = settings::snapshot(&pool, None).await.unwrap();
    assert_eq!(s.version, 17);
    let u = s.universe("universe.symbols").unwrap();
    let names: Vec<&str> = u.iter().map(|(n, _)| n.as_str()).collect();
    // Rulebook §3, in its order.
    assert_eq!(
        names,
        vec![
            "US500", "NAS100", "US30", "GER40", "UK100", "JPN225", "AUS200", "EURUSD", "GBPUSD",
            "USDJPY", "AUDUSD", "USDCAD", "NZDUSD", "XAUUSD", "XAGUSD", "USOIL", "UKOIL", "NATGAS",
            "BTCUSD", "ETHUSD"
        ]
    );
    assert_eq!(s.integer("indicators.atr_length").unwrap(), 20);
    assert_eq!(s.number("verdict.min_history_years").unwrap(), 10.0);
    assert_eq!(s.number("verdict.min_history_years_crypto").unwrap(), 6.0);
    assert_eq!(s.number("verdict.max_spread_to_atr").unwrap(), 0.10);
    assert_eq!(s.integer("ticks.retention_days").unwrap(), 30);
    assert_eq!(s.integer("spread.sample_secs").unwrap(), 60);
    // Every seed is a logged change.
    let h = settings::history(&pool, None).await.unwrap();
    assert_eq!(h.len(), 17);
    assert!(h
        .iter()
        .all(|c| c.changed_by == "migration" && c.old_value.is_none()));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_out_of_bounds_value_is_refused_and_nothing_changes(pool: PgPool) {
    let err = settings::change(
        &pool,
        "ticks.retention_days",
        None,
        Some(json!(0)),
        "owner",
        "try",
    )
    .await
    .unwrap_err();
    assert!(
        matches!(&err, ChangeError::Invalid(m) if m == "ticks.retention_days: 0 is below the minimum 1"),
        "{err}"
    );
    let err = settings::change(
        &pool,
        "verdict.max_spread_to_atr",
        None,
        Some(json!("0.2")),
        "owner",
        "try",
    )
    .await
    .unwrap_err();
    assert!(matches!(err, ChangeError::Invalid(_)));
    let s = settings::snapshot(&pool, None).await.unwrap();
    assert_eq!(
        s.integer("ticks.retention_days").unwrap(),
        30,
        "not clamped, not changed"
    );
    assert_eq!(s.version, 17, "a refused change does not bump the version");
    assert!(matches!(
        settings::change(
            &pool,
            "ticks.retention_days",
            None,
            Some(json!(10)),
            "owner",
            "  "
        )
        .await,
        Err(ChangeError::NoReason)
    ));
    assert!(matches!(
        settings::change(&pool, "no.such.key", None, Some(json!(1)), "owner", "r").await,
        Err(ChangeError::UnknownKey(_))
    ));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_change_is_logged_with_old_and_new_values_and_bumps_the_version(pool: PgPool) {
    let v = settings::change(
        &pool,
        "ticks.retention_days",
        None,
        Some(json!(14)),
        "owner",
        "disk is small",
    )
    .await
    .unwrap();
    assert_eq!(v, 18);
    let h = settings::history(&pool, Some("ticks.retention_days"))
        .await
        .unwrap();
    assert_eq!(h[0].version, 18);
    assert_eq!(h[0].old_value, Some(json!(30)));
    assert_eq!(h[0].new_value, Some(json!(14)));
    assert_eq!(h[0].changed_by, "owner");
    assert_eq!(h[0].reason, "disk is small");
    assert_eq!(
        settings::snapshot(&pool, None)
            .await
            .unwrap()
            .integer("ticks.retention_days")
            .unwrap(),
        14
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_account_value_overrides_the_global_one(pool: PgPool) {
    settings::change(
        &pool,
        "bars.tick.updates_per_bar",
        Some(A),
        Some(json!(250)),
        "owner",
        "busier symbols",
    )
    .await
    .unwrap();
    let acct = settings::snapshot(&pool, Some(A)).await.unwrap();
    assert_eq!(acct.integer("bars.tick.updates_per_bar").unwrap(), 250);
    assert_eq!(
        acct.scope_of("bars.tick.updates_per_bar"),
        Some(Scope::Account(A))
    );
    let other = settings::snapshot(&pool, Some(43002148)).await.unwrap();
    assert_eq!(other.integer("bars.tick.updates_per_bar").unwrap(), 100);
    // Removing the override is a logged change too; the global value returns.
    settings::change(
        &pool,
        "bars.tick.updates_per_bar",
        Some(A),
        None,
        "owner",
        "back to global",
    )
    .await
    .unwrap();
    let acct = settings::snapshot(&pool, Some(A)).await.unwrap();
    assert_eq!(acct.integer("bars.tick.updates_per_bar").unwrap(), 100);
    let h = settings::history(&pool, Some("bars.tick.updates_per_bar"))
        .await
        .unwrap();
    assert_eq!(
        (h[0].old_value.clone(), h[0].new_value.clone()),
        (Some(json!(250)), None)
    );
    // The global value cannot be removed.
    assert!(matches!(
        settings::change(&pool, "bars.tick.updates_per_bar", None, None, "owner", "r").await,
        Err(ChangeError::GlobalRemoval(_))
    ));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn ticks_go_to_daily_partitions_and_retention_drops_whole_old_days(pool: PgPool) {
    let d = |m, dd| NaiveDate::from_ymd_opt(2024, m, dd).unwrap();
    ticks::ensure_partitions(&pool, d(3, 1), 3).await.unwrap();
    let t = |h: u32, dd: u32| TickRow {
        account_id: A,
        broker_symbol_id: 1,
        received_at: Utc.with_ymd_and_hms(2024, 3, dd, h, 0, 0).unwrap(),
        broker_timestamp: Some(1),
        bid_e5: Some(108_500),
        ask_e5: None,
    };
    let n = ticks::insert(&pool, &[t(0, 1), t(23, 1), t(5, 2), t(1, 3)])
        .await
        .unwrap();
    assert_eq!(n, 4);
    let in_first: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks_p20240301")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(in_first, 2);
    // A tick with neither side is refused.
    let mut empty = t(2, 2);
    empty.bid_e5 = None;
    assert!(ticks::insert(&pool, &[empty]).await.is_err());
    // A tick outside every partition is refused, not silently placed.
    assert!(ticks::insert(&pool, &[t(1, 20)]).await.is_err());

    // Today 03-04, keep 2 days: 03-01 ended 03-02 ≤ 03-02 → dropped; 03-02 ended 03-03 > 03-02 → kept.
    let dropped = ticks::drop_expired(&pool, d(3, 4), 2).await.unwrap();
    assert_eq!(dropped, vec!["ticks_p20240301".to_string()]);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 2);
    assert!(ticks::drop_expired(&pool, d(3, 4), 0).await.is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn token_round_trip_and_expiry_reason(pool: PgPool) {
    use jiaoyi_core::secret::Secret;
    store::save_token(
        &pool,
        &Secret::new("a1"),
        &Secret::new("r1"),
        None,
        None,
        "env_seed",
    )
    .await
    .unwrap();
    let t = store::load_token(&pool).await.unwrap().unwrap();
    assert_eq!(
        (t.access_token.expose(), t.refresh_token.expose()),
        ("a1", "r1")
    );
    assert_eq!(t.expires_at, None);
    let why: Option<String> =
        sqlx::query_scalar("SELECT expires_at_missing_reason FROM oauth_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(why.unwrap().contains("expiry not known"));
    let exp = Utc.with_ymd_and_hms(2026, 11, 20, 0, 0, 0).unwrap();
    store::save_token(
        &pool,
        &Secret::new("a2"),
        &Secret::new("r2"),
        Some("bearer"),
        Some(exp),
        "refresh",
    )
    .await
    .unwrap();
    let t = store::load_token(&pool).await.unwrap().unwrap();
    assert_eq!((t.access_token.expose(), t.expires_at), ("a2", Some(exp)));
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM oauth_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 1);
}
