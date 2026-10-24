//! The whole phase-1 load, recorder and report against the in-memory fake
//! broker and a real PostgreSQL (fresh database per test via `DATABASE_URL`).

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::Utc;
use jiaoyi::config::BrokerConfig;
use jiaoyi::{loader, recorder, report, service};
use jiaoyi_core::allowlist::{plan_accounts, Allowlist, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages::{ProtoOaLightSymbol, ProtoOaSymbol, ProtoOaTrendbar};
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::PgPool;

const A: i64 = 49236051;

fn broker(seed: Option<(&str, &str)>) -> BrokerConfig {
    BrokerConfig {
        demo_host: "unused".into(),
        live_host: "unused".into(),
        port: 0,
        client_id: Some("client".into()),
        client_secret: Some(Secret::new("client-secret")),
        seed_access_token: seed.map(|s| Secret::new(s.0)),
        seed_refresh_token: seed.map(|s| Secret::new(s.1)),
        seed_expires_at: None,
    }
}

fn permitted() -> PermittedAccount {
    Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(A)
        .unwrap()
}

fn light(id: i64, name: &str) -> ProtoOaLightSymbol {
    ProtoOaLightSymbol {
        symbol_id: id,
        symbol_name: Some(name.into()),
        enabled: Some(true),
        ..Default::default()
    }
}

fn detail(id: i64) -> ProtoOaSymbol {
    ProtoOaSymbol {
        symbol_id: id,
        digits: 5,
        pip_position: 4,
        lot_size: Some(10_000_000),
        min_volume: Some(100_000),
        step_volume: Some(100_000),
        swap_long: Some(-7.5),
        swap_short: Some(2.1),
        precise_trading_commission_rate: Some(3_000_000_000),
        ..Default::default()
    }
}

/// `days` closed daily bars ending yesterday, plus today's forming bar.
/// Each bar: low L, high L+100, open L+20, close L+60 (relative units), so
/// every true range is 100 relative = 0.001 and ATR = 0.001.
fn daily_bars(days: i64) -> Vec<ProtoOaTrendbar> {
    let today = Utc::now().date_naive();
    (0..=days)
        .map(|i| {
            let d = today - chrono::Duration::days(days - i);
            let low = 108_000;
            d1_bar(d, low + 20, low + 100, low, low + 60, 1000)
        })
        .collect()
}

fn fake() -> FakeBroker {
    FakeBroker {
        account_id: A,
        light_symbols: vec![
            light(1, "EURUSD"),
            light(7, "XAUUSD"),
            light(8, "XAUUSD"),
            light(9, "ukoil"),
        ],
        symbols: vec![detail(1), detail(7), detail(8)],
        // 11 years of history.
        bars: HashMap::from([(1, daily_bars(4018))]),
        // spreads 8, 9, 11 relative → median 9 = 0.00009; ATR 0.001 → 9%.
        // (Exactly 10% is tested in jiaoyi-core with exact inputs; an ATR
        // built from floats lands a hair either side of it.)
        quotes: HashMap::from([(
            1,
            vec![
                (Some(108_000), Some(108_008)),
                (Some(108_000), Some(108_009)),
                (None, Some(108_011)),
            ],
        )]),
        ..Default::default()
    }
}

async fn small_universe(pool: &PgPool) {
    settings::change(
        pool,
        "universe.symbols",
        None,
        Some(json!([
            {"name": "EURUSD", "class": "fx"},
            {"name": "XAUUSD", "class": "metal"},
            {"name": "UKOIL", "class": "energy"}
        ])),
        "test",
        "three-symbol universe for the test",
    )
    .await
    .unwrap();
    settings::change(
        pool,
        "spread.sample_secs",
        None,
        Some(json!(5)),
        "test",
        "short window",
    )
    .await
    .unwrap();
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_full_load_stores_closed_bars_costs_resolutions_and_a_report(pool: PgPool) {
    store::upsert_account(&pool, A, jiaoyi_core::allowlist::Environment::Demo)
        .await
        .unwrap();
    small_universe(&pool).await;
    let (conn, log) = fake().spawn();
    let out = loader::load_account(&pool, &conn, &broker(Some(("tok", "ref"))), &permitted())
        .await
        .unwrap();
    assert_eq!(out.status, "ok", "{:?}", out.summary.errors);
    let s = &out.summary;
    assert_eq!(s.settings_version, Some(19));
    assert_eq!((s.symbols_resolved, s.symbols_unresolved), (1, 2));
    assert_eq!(s.bars_inserted, 4018, "every closed bar");
    assert_eq!(s.forming_bars_dropped, 1, "today's bar is not stored");
    assert_eq!(s.spreads_measured, 1);
    assert_eq!(s.crosscheck.len(), 1);
    assert_eq!(s.crosscheck[0]["mismatched"], json!(0));
    assert_eq!(s.crosscheck[0]["missing_in_db"], json!(0));
    assert!(s.crosscheck[0]["matched"].as_i64().unwrap() >= 29);

    // Only allowlisted read requests reached the broker.
    let sent: HashSet<u32> = log.lock().unwrap().payload_types.iter().copied().collect();
    for t in &sent {
        assert!(jiaoyi_ctrader::check_sendable(*t).is_ok(), "{t}");
    }
    assert_eq!(log.lock().unwrap().account_auths, vec![A]);

    let today = Utc::now().date_naive();
    let stored = store::bars(&pool, A, 1).await.unwrap();
    assert_eq!(
        stored.last().unwrap().bar_date,
        today - chrono::Duration::days(1)
    );

    let res = store::latest_resolutions(&pool, A).await.unwrap();
    let by: HashMap<_, _> = res.iter().map(|r| (r.name.as_str(), r)).collect();
    assert_eq!(by["EURUSD"].broker_symbol_id, Some(1));
    assert_eq!(by["XAUUSD"].status, "ambiguous");
    assert_eq!(by["XAUUSD"].candidate_ids, vec![7, 8]);
    assert_eq!(by["UKOIL"].status, "missing");
    assert!(
        by["UKOIL"]
            .reason
            .as_deref()
            .unwrap()
            .contains("near names not used"),
        "{:?}",
        by["UKOIL"].reason
    );

    let costs: (Option<rust_decimal::Decimal>, i32, Option<f64>, Option<i64>, serde_json::Value) = sqlx::query_as(
        "SELECT spread_median, spread_samples, swap_long, precise_trading_commission_rate, null_reasons
         FROM symbol_costs WHERE broker_symbol_id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(costs.0, Some(rust_decimal::Decimal::new(9, 5)));
    assert_eq!(costs.1, 3);
    assert_eq!(costs.2, Some(-7.5));
    assert_eq!(costs.3, Some(3_000_000_000));
    assert!(costs.4["commission_per_lot_side"]
        .as_str()
        .unwrap()
        .contains("not derived"));

    let r = report::build(&pool).await.unwrap();
    let a = &r.accounts[0];
    let sym: HashMap<_, _> = a.symbols.iter().map(|s| (s.name.as_str(), s)).collect();
    let eur = sym["EURUSD"];
    assert_eq!(eur.bars, 4018);
    assert!((eur.atr.unwrap() - 0.001).abs() < 1e-12);
    assert!((eur.spread_to_atr.unwrap() - 0.09).abs() < 1e-9);
    assert!(eur.years_of_history.unwrap() > 10.0);
    assert_eq!(eur.bar_open_times_utc.get("00:00"), Some(&4018));
    assert_eq!(
        eur.verdict.decision,
        jiaoyi_core::verdict::Decision::Keep,
        "{:?}",
        eur.verdict
    );
    assert_eq!(
        sym["XAUUSD"].verdict.decision,
        jiaoyi_core::verdict::Decision::Drop
    );
    assert_eq!(
        sym["UKOIL"].verdict.decision,
        jiaoyi_core::verdict::Decision::Drop
    );
    assert_eq!(r.settings_version, 19);

    // A second load stores nothing new and changes nothing.
    let (conn, _) = fake().spawn();
    let again = loader::load_account(&pool, &conn, &broker(None), &permitted())
        .await
        .unwrap();
    assert_eq!(
        (again.summary.bars_inserted, again.summary.bars_changed),
        (0, 0)
    );
    assert_eq!(again.summary.bars_unchanged, 4018);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rejected_token_is_refreshed_once_and_the_new_pair_stored(pool: PgPool) {
    store::upsert_account(&pool, A, jiaoyi_core::allowlist::Environment::Demo)
        .await
        .unwrap();
    small_universe(&pool).await;
    let mut f = fake();
    f.expired_tokens = HashSet::from(["stale".to_string()]);
    f.refresh_reply = Some(("fresh".into(), "fresh-refresh".into(), 2_628_000));
    let (conn, log) = f.spawn();
    let out = loader::load_account(
        &pool,
        &conn,
        &broker(Some(("stale", "old-refresh"))),
        &permitted(),
    )
    .await
    .unwrap();
    assert_eq!(out.status, "ok", "{:?}", out.summary.errors);
    let t = store::load_token(&pool).await.unwrap().unwrap();
    assert_eq!(t.access_token.expose(), "fresh");
    assert_eq!(t.refresh_token.expose(), "fresh-refresh");
    assert_eq!(t.source, "refresh");
    assert!(t.expires_at.is_some());
    assert_eq!(log.lock().unwrap().account_auths, vec![A, A]);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn no_token_anywhere_fails_the_load_and_says_why(pool: PgPool) {
    store::upsert_account(&pool, A, jiaoyi_core::allowlist::Environment::Demo)
        .await
        .unwrap();
    let (conn, log) = fake().spawn();
    let out = loader::load_account(&pool, &conn, &broker(None), &permitted())
        .await
        .unwrap();
    assert_eq!(out.status, "failed");
    assert!(
        out.summary.errors[0].contains("no token"),
        "{:?}",
        out.summary.errors
    );
    assert!(log.lock().unwrap().account_auths.is_empty());
    let runs = store::latest_runs(&pool).await.unwrap();
    assert_eq!(runs[0].status, "failed");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_recorder_writes_each_quote_update_to_ticks(pool: PgPool) {
    store::upsert_account(&pool, A, jiaoyi_core::allowlist::Environment::Demo)
        .await
        .unwrap();
    small_universe(&pool).await;
    let (conn, _) = fake().spawn();
    loader::load_account(&pool, &conn, &broker(Some(("tok", "ref"))), &permitted())
        .await
        .unwrap();
    drop(conn);

    let (conn, _) = fake().spawn();
    let status = recorder::StatusMap::default();
    let until = tokio::time::Instant::now() + Duration::from_millis(1500);
    let n = recorder::record(
        &pool,
        &conn,
        &broker(None),
        &permitted(),
        &status,
        Some(until),
    )
    .await
    .unwrap();
    assert_eq!(n, 3);
    let rows: Vec<(i64, Option<i64>, Option<i64>)> =
        sqlx::query_as("SELECT broker_symbol_id, bid_e5, ask_e5 FROM ticks ORDER BY received_at")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        rows,
        vec![
            (1, Some(108_000), Some(108_008)),
            (1, Some(108_000), Some(108_009)),
            (1, None, Some(108_011)), // stored as sent: only the ask changed
        ]
    );
    let st = status.lock().unwrap()[&A].clone();
    assert_eq!(st.subscribed_symbols, vec![1]);
    assert_eq!(st.ticks_written, 3);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn accounts_not_on_the_allowlist_are_refused_and_recorded(pool: PgPool) {
    let allowlist = Allowlist::parse(Some("49236051,43002148")).unwrap();
    let plan = plan_accounts(&allowlist, Some("49236051:demo,12345678:live"));
    let cfg = jiaoyi::config::Config {
        allowlist,
        plan,
        database_url: Secret::new("unused"),
        http_port: 0,
        broker: broker(None),
    };
    service::register_accounts(&pool, &cfg).await.unwrap();
    let refused: Vec<(String, String)> =
        sqlx::query_as("SELECT account_ref, reason FROM account_refusals ORDER BY account_ref")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(refused.len(), 2, "{refused:?}");
    assert_eq!(refused[0].0, "12345678");
    assert!(refused[0].1.contains("not on JIAOYI_ALLOWED_ACCOUNTS"));
    assert_eq!(refused[1].0, "43002148");
    assert!(refused[1].1.contains("no environment declared"));
    let accounts: Vec<i64> = sqlx::query_scalar("SELECT account_id FROM accounts")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(accounts, vec![A]);
}
