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
        "ticks_history",
        "history_fetches",
        "deals",
        "deal_fetches",
        "account_snapshots",
        "trader_reads",
        "research_trials",
        "token_grants",
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
async fn seeded_settings_are_the_rulebook_values_then_the_owners_energy_names(pool: PgPool) {
    let s = settings::snapshot(&pool, None).await.unwrap();
    // 17 phase-1 seeds, 4 history seeds (0004), 4 load/spread seeds (0005),
    // the owner's universe change (0005), the session-retry seed (0007), the
    // owner's SpotBrent change (0008) and the 10 trading seeds (0012).
    assert_eq!(s.version, 38);
    assert_eq!(s.integer("history.session_retry_secs").unwrap(), 5);
    assert_eq!(s.integer("history.tick_days_back").unwrap(), 7);
    assert_eq!(s.integer("spread.min_samples").unwrap(), 30);
    assert_eq!(s.integer("load.retry_minutes").unwrap(), 5);
    // The seed (version 1) is rulebook §3, in its order.
    let h = settings::history(&pool, Some("universe.symbols"))
        .await
        .unwrap();
    let seed = h.iter().find(|c| c.version == 1).unwrap();
    let seeded = jiaoyi_core::universe::parse_universe(seed.new_value.as_ref().unwrap()).unwrap();
    assert_eq!(
        seeded.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        vec![
            "US500", "NAS100", "US30", "GER40", "UK100", "JPN225", "AUS200", "EURUSD", "GBPUSD",
            "USDJPY", "AUDUSD", "USDCAD", "NZDUSD", "XAUUSD", "XAGUSD", "USOIL", "UKOIL", "NATGAS",
            "BTCUSD", "ETHUSD"
        ]
    );
    // Now the owner's ruling (24-10): the broker's energy names, UKOIL out.
    let u = s.universe("universe.symbols").unwrap();
    let names: Vec<&str> = u.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "US500",
            "NAS100",
            "US30",
            "GER40",
            "UK100",
            "JPN225",
            "AUS200",
            "EURUSD",
            "GBPUSD",
            "USDJPY",
            "AUDUSD",
            "USDCAD",
            "NZDUSD",
            "XAUUSD",
            "XAGUSD",
            "SpotCrude",
            "NatGas",
            "SpotBrent",
            "BTCUSD",
            "ETHUSD"
        ]
    );
    for gone in ["USOIL", "UKOIL", "NATGAS"] {
        assert!(!names.contains(&gone), "{gone}");
    }
    assert!(u
        .iter()
        .filter(|(n, _)| *n == "SpotCrude" || *n == "NatGas" || *n == "SpotBrent")
        .all(|(_, c)| *c == jiaoyi_core::universe::AssetClass::Energy));
    // h[0] is the SpotBrent change (its own test); h[1] the energy names.
    let change = &h[1];
    assert_eq!(change.version, 26);
    assert_eq!(change.changed_by, "owner");
    assert_eq!(
        change.reason,
        "owner 24-10: use the broker's names for energy"
    );
    assert_eq!(change.old_value, seed.new_value);
    assert_eq!(s.integer("indicators.atr_length").unwrap(), 20);
    assert_eq!(s.number("verdict.min_history_years").unwrap(), 10.0);
    assert_eq!(s.number("verdict.min_history_years_crypto").unwrap(), 6.0);
    assert_eq!(s.number("verdict.max_spread_to_atr").unwrap(), 0.10);
    assert_eq!(s.integer("ticks.retention_days").unwrap(), 30);
    assert_eq!(s.integer("spread.sample_secs").unwrap(), 60);
    // Every seed and the owner's change are logged (28, then the 10 trading
    // seeds of 0012).
    let h = settings::history(&pool, None).await.unwrap();
    assert_eq!(h.len(), 38);
    assert!(h
        .iter()
        .filter(|c| c.version != 26 && c.version != 28)
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
    assert_eq!(s.version, 38, "a refused change does not bump the version");
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
    assert_eq!(v, 39);
    let h = settings::history(&pool, Some("ticks.retention_days"))
        .await
        .unwrap();
    assert_eq!(h[0].version, 39);
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
        session_id: "s1".into(),
        seq: (dd * 100 + h) as i64,
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
    assert_eq!(t.seed_fingerprint, None, "saved without a seed");
}

/// PR #6: a stored seed remembers its fingerprint, and a refresh of it keeps
/// that fingerprint; the grant reads are append-only and hold no token.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_seed_keeps_its_fingerprint_through_a_refresh_and_grant_reads_are_append_only(
    pool: PgPool,
) {
    use jiaoyi_core::secret::Secret;
    let seed = Secret::new("seed-access");
    store::store_seed(&pool, &seed, &Secret::new("seed-refresh"), None)
        .await
        .unwrap();
    let t = store::load_token(&pool).await.unwrap().unwrap();
    assert_eq!(t.source, "env_seed");
    assert_eq!(t.seed_fingerprint, Some(seed.fingerprint()));
    store::save_token(
        &pool,
        &Secret::new("refreshed"),
        &Secret::new("refreshed-r"),
        Some("bearer"),
        Some(Utc::now()),
        "refresh",
    )
    .await
    .unwrap();
    let t = store::load_token(&pool).await.unwrap().unwrap();
    assert_eq!(t.access_token.expose(), "refreshed");
    assert_eq!(t.seed_fingerprint, Some(seed.fingerprint()), "kept");

    let read = store::GrantRead {
        account_id: A,
        host: "demo.ctraderapi.com".into(),
        token_fingerprint: t.access_token.fingerprint(),
        token_source: t.source.clone(),
        verdict: "not_listed",
        reason: Some(
            "token grant does not include account 49236051 (grant lists: [43002148])".into(),
        ),
        permission_scope: Some("SCOPE_TRADE".into()),
        accounts: Some(json!([{ "ctid": 43002148, "is_live": true, "login": 1 }])),
    };
    store::record_grant(&pool, &read).await.unwrap();
    let latest = store::latest_grants(&pool).await.unwrap();
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].verdict, "not_listed");
    for q in [
        "UPDATE token_grants SET verdict = 'listed'",
        "DELETE FROM token_grants",
    ] {
        let e = sqlx::query(q).execute(&pool).await.unwrap_err();
        assert!(e.to_string().contains("append-only"), "{q}: {e}");
    }
    // A fingerprint is 16 hex characters; anything else is refused.
    let bad = store::GrantRead {
        token_fingerprint: "refreshed".into(),
        ..read
    };
    assert!(store::record_grant(&pool, &bad).await.is_err());
}

/// Review finding on #3: a flush is all or nothing, and a retried flush
/// cannot insert a quote twice.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_failed_flush_stores_nothing_and_a_retry_stores_each_tick_once(pool: PgPool) {
    let day = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
    ticks::ensure_partitions(&pool, day, 1).await.unwrap();
    let rows: Vec<TickRow> = (0..10)
        .map(|i| TickRow {
            account_id: A,
            session_id: "s1".into(),
            seq: i,
            broker_symbol_id: 1,
            received_at: Utc.with_ymd_and_hms(2024, 3, 1, 9, 0, i as u32).unwrap(),
            broker_timestamp: Some(i),
            bid_e5: Some(108_500 + i),
            ask_e5: Some(108_510 + i),
        })
        .collect();
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM ticks")
            .fetch_one(&pool)
            .await
            .unwrap()
    };

    // Chunks of 4: rows 0-3, 4-7, 8-9. Row 6 breaks the second chunk (a tick
    // with neither side violates the table's CHECK).
    let mut bad = rows.clone();
    bad[6].bid_e5 = None;
    bad[6].ask_e5 = None;
    assert!(ticks::insert_chunked(&pool, &bad, 4).await.is_err());
    assert_eq!(
        count().await,
        0,
        "the first chunk must not survive the failed flush"
    );

    assert_eq!(ticks::insert_chunked(&pool, &rows, 4).await.unwrap(), 10);
    // The same flush again (a retry after an unknown outcome) adds nothing.
    assert_eq!(ticks::insert_chunked(&pool, &rows, 4).await.unwrap(), 0);
    assert_eq!(count().await, 10);
}

fn fetch(kind: &'static str, error: Option<&str>) -> jiaoyi_store::history::FetchRecord {
    jiaoyi_store::history::FetchRecord {
        account_id: A,
        broker_symbol_id: 1,
        side: "bid",
        kind,
        window_from: Utc.with_ymd_and_hms(2024, 3, 4, 0, 0, 0).unwrap(),
        window_to: Utc.with_ymd_and_hms(2024, 3, 10, 23, 59, 59).unwrap(),
        complete: true,
        ticks: 3,
        pages: 1,
        first_tick: None,
        last_tick: None,
        delta_signs: vec!["add".into()],
        error: error.map(String::from),
        session_ended: false,
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_tick_window_is_stored_once_with_its_fetch_record(pool: PgPool) {
    use jiaoyi_store::history::{self as hist, HistoryTick};
    let t = |s: u32, p: i64, dup: i32| HistoryTick {
        ts: Utc.with_ymd_and_hms(2024, 3, 5, 9, 0, s).unwrap(),
        price_e5: p,
        dup,
    };
    let ticks = vec![t(0, 108_500, 0), t(1, 108_501, 0), t(1, 108_501, 1)];
    let (_, stored) = hist::record_window(&pool, &fetch("download", None), &ticks)
        .await
        .unwrap();
    assert_eq!(
        stored, 3,
        "a repeated tick at the same instant is kept as dup 1"
    );
    // The same window again: no new ticks, but the fetch itself is recorded.
    let (_, stored) = hist::record_window(&pool, &fetch("download", None), &ticks)
        .await
        .unwrap();
    assert_eq!(stored, 0);
    let fetches: i64 = sqlx::query_scalar("SELECT count(*) FROM history_fetches")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fetches, 2);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 3);
    let source: String = sqlx::query_scalar("SELECT DISTINCT source FROM ticks_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(source, "broker_history");
    // A failed window must carry its error; it is still recorded.
    hist::record_window(
        &pool,
        &fetch("download", Some("tick data does not decode")),
        &[],
    )
    .await
    .unwrap();
    let done = hist::complete_windows(&pool, A, 1, "bid", "download")
        .await
        .unwrap();
    let f = fetch("download", None);
    assert!(done.covers(
        f.window_from.timestamp_millis(),
        f.window_to.timestamp_millis()
    ));
    // Append-only.
    assert!(sqlx::query("UPDATE history_fetches SET ticks = 0")
        .execute(&pool)
        .await
        .is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_deal_the_broker_reports_differently_is_recorded_then_replaced(pool: PgPool) {
    use jiaoyi_store::history::{self as hist, DealRow};
    let deal = |commission: i64| DealRow {
        deal_id: 9,
        order_id: 8,
        position_id: 7,
        broker_symbol_id: 1,
        volume: 100_000,
        filled_volume: 100_000,
        trade_side: "BUY".into(),
        deal_status: "FILLED".into(),
        created_at: Utc.with_ymd_and_hms(2024, 3, 5, 9, 0, 0).unwrap(),
        executed_at: Utc.with_ymd_and_hms(2024, 3, 5, 9, 0, 0).unwrap(),
        last_update_at: None,
        execution_price: Some(1.085),
        margin_rate: None,
        commission: Some(commission),
        money_digits: Some(2),
        base_to_usd_rate: None,
        is_closing: false,
        close_entry_price: None,
        close_gross_profit: None,
        close_swap: None,
        close_commission: None,
        close_balance: None,
        close_quote_to_deposit_rate: None,
        close_closed_volume: None,
        close_balance_version: None,
        close_money_digits: None,
        close_pnl_conversion_fee: None,
        raw: vec![commission as u8],
    };
    let u = hist::upsert_deals(&pool, A, &[deal(-336)], Utc::now())
        .await
        .unwrap();
    assert_eq!((u.inserted, u.changed), (1, 0));
    let u = hist::upsert_deals(&pool, A, &[deal(-336)], Utc::now())
        .await
        .unwrap();
    assert_eq!(u.unchanged, 1);
    let u = hist::upsert_deals(&pool, A, &[deal(-484)], Utc::now())
        .await
        .unwrap();
    assert_eq!(u.changed, 1);
    let c: Option<i64> = sqlx::query_scalar("SELECT commission FROM deals")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        c,
        Some(-484),
        "the broker's value, as an integer in 10^-money_digits"
    );
    let ev: i64 =
        sqlx::query_scalar("SELECT count(*) FROM reconcile_events WHERE kind = 'deal_changed'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ev, 1);
    assert!(sqlx::query("DELETE FROM deals")
        .execute(&pool)
        .await
        .is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn research_trials_are_kept_whatever_their_result(pool: PgPool) {
    use jiaoyi_store::history::{self as hist, Trial};
    let id = hist::record_trial(
        &pool,
        &Trial {
            hypothesis: "H0 placeholder",
            params: &json!({}),
            data_range: &json!({"from": "2024-01-01"}),
            sample: "development",
            results: None,
            verdict: None,
            settings_version: 21,
        },
    )
    .await
    .unwrap();
    assert!(id > 0);
    assert!(sqlx::query("UPDATE research_trials SET verdict = 'edge'")
        .execute(&pool)
        .await
        .is_err());
    assert!(sqlx::query("DELETE FROM research_trials")
        .execute(&pool)
        .await
        .is_err());
    let bad = sqlx::query(
        "INSERT INTO research_trials (hypothesis, params, data_range, sample, settings_version)
         VALUES ('x', '{}', '{}', 'peeked', 1)",
    )
    .execute(&pool)
    .await;
    assert!(bad.is_err(), "sample is development or held_out");
}

/// Deploy fix (452da09a): two recorders creating the same day's partition at
/// once must both succeed, for the daily ticks and the monthly history.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn concurrent_partition_creators_all_succeed(pool: PgPool) {
    for d in 0..12 {
        let day = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap() + chrono::Duration::days(d * 31);
        let at = day.and_hms_opt(12, 0, 0).unwrap().and_utc();
        let mut tasks: Vec<tokio::task::JoinHandle<Result<(), String>>> = Vec::new();
        for _ in 0..8 {
            let p = pool.clone();
            tasks.push(tokio::spawn(async move {
                ticks::ensure_partitions(&p, day, 1)
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }));
            let p = pool.clone();
            tasks.push(tokio::spawn(async move {
                jiaoyi_store::history::ensure_partitions(&p, at, at)
                    .await
                    .map_err(|e| e.to_string())
            }));
        }
        for t in tasks {
            t.await.unwrap().unwrap();
        }
    }
    let daily: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class WHERE relkind = 'r' AND relname LIKE 'ticks\\_p2027%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(daily, 12);
    let monthly: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class WHERE relkind = 'r' AND relname LIKE 'ticks\\_history\\_p2027%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(monthly, 12);
}

/// The server notices sqlx reports (target `sqlx::postgres::notice`).
#[derive(Clone, Default)]
struct Notices(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

struct Message(String);

impl tracing::field::Visit for Message {
    fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
        if f.name() == "message" {
            self.0 = format!("{v:?}");
        }
    }
    fn record_str(&mut self, f: &tracing::field::Field, v: &str) {
        if f.name() == "message" {
            self.0 = v.to_string();
        }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Notices {
    fn on_event(&self, e: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if e.metadata().target() == "sqlx::postgres::notice" {
            let mut m = Message(String::new());
            e.record(&mut m);
            self.0.lock().unwrap().push(m.0);
        }
    }
}

/// PR #6, item C: the deploy log carried `relation "ticks_p2026102x" already
/// exists, skipping` from every partition check. An existing partition is now
/// found with to_regclass before the CREATE, so the notice cannot occur; the
/// advisory lock and the 42P07 handling stay for the race.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_existing_partition_raises_no_notice(pool: PgPool) {
    use tracing_subscriber::layer::SubscriberExt;
    let notices = Notices::default();
    let _g = tracing::subscriber::set_default(tracing_subscriber::registry().with(notices.clone()));
    // The counter sees a notice when one is raised (it is not blind).
    sqlx::query("DO $$ BEGIN RAISE NOTICE 'counter check'; END $$")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        notices.0.lock().unwrap().clone(),
        vec!["counter check".to_string()]
    );
    notices.0.lock().unwrap().clear();

    let day = NaiveDate::from_ymd_opt(2027, 3, 1).unwrap();
    let at = day.and_hms_opt(12, 0, 0).unwrap().and_utc();
    for _ in 0..3 {
        ticks::ensure_partitions(&pool, day, 2).await.unwrap();
        jiaoyi_store::history::ensure_partitions(&pool, at, at)
            .await
            .unwrap();
    }
    let made: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class WHERE relkind = 'r'
           AND relname IN ('ticks_p20270301', 'ticks_p20270302', 'ticks_history_p202703')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(made, 3);
    assert_eq!(notices.0.lock().unwrap().clone(), Vec::<String>::new());
}

/// PR #6, item D. Owner, 24-10-2026: "use the broker's names for energy";
/// Brent is SpotBrent on the broker (energy_candidates, load run 6 on
/// 43002148). One logged change of universe.symbols by the owner; no futures
/// (-F) or perpetual (-PERP) names; the demo account's names are not guessed.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_owner_adds_spot_brent_to_the_energy_class(pool: PgPool) {
    let s = settings::snapshot(&pool, None).await.unwrap();
    let u = s.universe("universe.symbols").unwrap();
    let energy: Vec<&str> = u
        .iter()
        .filter(|(_, c)| *c == AssetClass::Energy)
        .map(|(n, _)| n.as_str())
        .collect();
    assert_eq!(energy, vec!["SpotCrude", "NatGas", "SpotBrent"]);
    assert!(u
        .iter()
        .all(|(n, _)| !n.ends_with("-F") && !n.ends_with("-PERP")));
    let h = settings::history(&pool, Some("universe.symbols"))
        .await
        .unwrap();
    assert_eq!(h.len(), 3, "seed, energy names, SpotBrent");
    let c = &h[0];
    // The latest universe change: SpotBrent (0008) at version 28; the 0012
    // trading seeds come after it.
    assert_eq!(c.version, 28, "the latest universe change");
    assert_eq!(
        h[1].version,
        c.version - 2,
        "one settings seed between them (0007)"
    );
    assert_eq!(c.changed_by, "owner");
    assert_eq!(
        c.reason,
        "owner 24-10: use the broker's names for energy; Brent is SpotBrent on the broker (energy_candidates, load run 6 on 43002148)"
    );
    assert_eq!(c.old_value, h[1].new_value);
    // Only SpotBrent was added; nothing else moved.
    let old = jiaoyi_core::universe::parse_universe(c.old_value.as_ref().unwrap()).unwrap();
    let mut expected = old.clone();
    let after_natgas = expected.iter().position(|(n, _)| n == "NatGas").unwrap() + 1;
    expected.insert(after_natgas, ("SpotBrent".to_string(), AssetClass::Energy));
    assert_eq!(u, expected);
}
