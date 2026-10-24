//! The history sync (deals, ticks, probe) and `/research/inventory` against
//! the in-memory fake broker and a real PostgreSQL.

use std::collections::HashMap;

use chrono::{Duration, Utc};
use jiaoyi::config::BrokerConfig;
use jiaoyi::history::{self, GRID_ORIGIN_MS, WEEK_MS};
use jiaoyi::{loader, research, shutdown};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages::{
    ProtoOaClosePositionDetail, ProtoOaDeal, ProtoOaLightSymbol, ProtoOaSymbol, ProtoOaTrader,
};
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::PgPool;

const A: i64 = 49236051;

/// deal_id, commission, money_digits, is_closing, close_swap, close_commission.
type DealCols = (
    i64,
    Option<i64>,
    Option<i32>,
    bool,
    Option<i64>,
    Option<i64>,
);

fn broker() -> BrokerConfig {
    BrokerConfig {
        demo_host: "unused".into(),
        live_host: "unused".into(),
        port: 0,
        client_id: Some("client".into()),
        client_secret: Some(Secret::new("client-secret")),
        seed_access_token: Some(Secret::new("tok")),
        seed_refresh_token: Some(Secret::new("ref")),
        seed_expires_at: None,
    }
}

fn permitted() -> PermittedAccount {
    Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(A)
        .unwrap()
}

fn week_start(ms: i64) -> i64 {
    GRID_ORIGIN_MS + (ms - GRID_ORIGIN_MS).div_euclid(WEEK_MS) * WEEK_MS
}

/// A broker with EURUSD (id 1): daily bars, bid ticks once a week from 40
/// weeks ago, hourly bids and asks over the last 3 days, one ask tick with an
/// impossible price in the current week, and three deals.
fn fake() -> FakeBroker {
    let now = Utc::now().timestamp_millis();
    let today = Utc::now().date_naive();
    let mut bids: Vec<(i64, i64)> = (1..=40).map(|w| (now - w * WEEK_MS, 108_000 + w)).collect();
    bids.extend((1..=72).map(|h| (now - h * 3_600_000, 108_500 + h)));
    let mut asks: Vec<(i64, i64)> = (1..=72)
        .map(|h| (now - h * 3_600_000 + 1, 108_510 + h))
        .collect();
    // An ask that decodes to price 0 in the current (partial) week.
    asks.push((now - 60_000, 0));
    let deal = |id: i64, ts: i64, closing: bool| ProtoOaDeal {
        deal_id: id,
        order_id: id + 1000,
        position_id: 77,
        volume: 100_000,
        filled_volume: 100_000,
        symbol_id: 1,
        create_timestamp: ts,
        execution_timestamp: ts,
        execution_price: Some(1.085),
        trade_side: if closing { 2 } else { 1 },
        deal_status: 2,
        commission: Some(-242),
        money_digits: Some(2),
        close_position_detail: closing.then(|| ProtoOaClosePositionDetail {
            entry_price: 1.085,
            gross_profit: 1500,
            swap: -35,
            commission: -484,
            balance: 1_001_000,
            money_digits: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    };
    FakeBroker {
        account_id: A,
        light_symbols: vec![ProtoOaLightSymbol {
            symbol_id: 1,
            symbol_name: Some("EURUSD".into()),
            enabled: Some(true),
            ..Default::default()
        }],
        symbols: vec![ProtoOaSymbol {
            symbol_id: 1,
            digits: 5,
            pip_position: 4,
            ..Default::default()
        }],
        bars: HashMap::from([(
            1,
            (1..=30)
                .map(|i| {
                    let d = today - Duration::days(i);
                    d1_bar(d, 108_020, 108_100, 108_000, 108_060, 10)
                })
                .collect(),
        )]),
        tick_history: HashMap::from([((1, 1), bids), ((1, 2), asks)]),
        max_ticks_per_response: 25,
        deals: vec![
            deal(1, now - 20 * 86_400_000, false),
            deal(2, now - 10 * 86_400_000, true),
            deal(3, now - 86_400_000, false),
        ],
        max_deals_per_response: 2,
        trader: Some(ProtoOaTrader {
            ctid_trader_account_id: A,
            balance: 1_001_000,
            deposit_asset_id: 15,
            money_digits: Some(2),
            trader_login: Some(5361573),
            registration_timestamp: Some(now - 60 * 86_400_000),
            ..Default::default()
        }),
        ..Default::default()
    }
}

async fn prepare(pool: &PgPool) {
    store::upsert_account(pool, A, Environment::Demo)
        .await
        .unwrap();
    settings::change(
        pool,
        "universe.symbols",
        None,
        Some(json!([{"name": "EURUSD", "class": "fx"}])),
        "test",
        "one symbol",
    )
    .await
    .unwrap();
    settings::change(
        pool,
        "spread.sample_secs",
        None,
        Some(json!(5)),
        "test",
        "short",
    )
    .await
    .unwrap();
    settings::change(
        pool,
        "history.probe_years",
        None,
        Some(json!(2)),
        "test",
        "short probe",
    )
    .await
    .unwrap();
    let (conn, _) = fake().spawn();
    let out = loader::load_account(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    assert_eq!(out.status, "ok", "{:?}", out.summary.errors);
}

async fn sync(
    pool: &PgPool,
    f: FakeBroker,
) -> std::sync::Arc<std::sync::Mutex<jiaoyi_ctrader::testkit::FakeLog>> {
    let (conn, log) = f.spawn();
    let session = loader::authorise(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let status = history::HistoryStatusMap::default();
    let (_t, stop) = shutdown::channel();
    history::sync_account(pool, &session, &permitted(), &status, &stop)
        .await
        .unwrap();
    log
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_sync_stores_deals_ticks_and_the_probe_and_resumes_without_refetching(pool: PgPool) {
    prepare(&pool).await;
    // One fake for both syncs: its ticks are placed relative to the time it was built.
    let broker_data = fake();
    let log = sync(&pool, broker_data.clone()).await;

    // Deals exactly as reported, including the closing detail.
    let deals: Vec<DealCols> = sqlx::query_as(
        "SELECT deal_id, commission, money_digits, is_closing, close_swap, close_commission
         FROM deals ORDER BY deal_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        deals,
        vec![
            (1, Some(-242), Some(2), false, None, None),
            (2, Some(-242), Some(2), true, Some(-35), Some(-484)),
            (3, Some(-242), Some(2), false, None, None),
        ]
    );
    let login: Option<i64> = sqlx::query_scalar("SELECT trader_login FROM account_snapshots")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(login, Some(5361573));

    // Ticks: the 72 hourly bids of the last 3 days, plus the weekly bids that
    // fall in the downloaded weeks; the asks of the failed current week are not stored.
    let bids: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks_history WHERE side = 'bid'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((72..=74).contains(&bids), "{bids}");
    let failed: Vec<(String, String)> =
        sqlx::query_as("SELECT side, error FROM history_fetches WHERE status = 'failed'")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0].0, "ask");
    assert!(failed[0].1.contains("does not decode"), "{failed:?}");

    // The probe found the week of the oldest bid, 40 weeks back.
    let inv = research::build(&pool).await.unwrap();
    let eur = &inv.accounts[0].symbols[0];
    let want = chrono::DateTime::<Utc>::from_timestamp_millis(week_start(
        Utc::now().timestamp_millis() - 40 * WEEK_MS,
    ))
    .unwrap()
    .date_naive();
    assert_eq!(
        eur.probe.earliest_week_with_ticks,
        Some(want),
        "{:?}",
        eur.probe
    );
    assert!(eur.probe.weeks_probed > 0 && eur.probe.weeks_probed < 20);
    assert_eq!(eur.history_bid.ticks_stored, bids);
    assert!(
        eur.history_bid.weeks_missing.len() <= 1,
        "only the current, partial week"
    );
    assert_eq!(eur.history_ask.weeks_failed.len(), 1);
    assert_eq!(eur.deals, 3);
    assert!(
        eur.missing.iter().any(|m| m.contains("ask weeks failed")),
        "{:?}",
        eur.missing
    );
    assert!(eur
        .missing
        .iter()
        .any(|m| m.contains("no live ticks recorded")));
    assert_eq!(inv.accounts[0].deals.deals, 3);
    assert_eq!(inv.accounts[0].deals.positions, 1);
    // No money on this route.
    let text = serde_json::to_string(&inv).unwrap();
    assert!(
        !text.contains("1001000") && !text.contains("-484"),
        "{text}"
    );

    // A second sync re-asks only the current, partial week and adds nothing.
    let first_requests = log.lock().unwrap().tick_requests.len();
    let first_deal_requests = log.lock().unwrap().deal_requests.len();
    let log2 = sync(&pool, broker_data).await;
    let again = log2.lock().unwrap().tick_requests.clone();
    let now_week = week_start(Utc::now().timestamp_millis());
    assert!(
        again.iter().all(|(_, _, from, _)| *from == now_week),
        "only the current week is fetched again: {again:?}"
    );
    assert!(again.len() < first_requests);
    assert!(log2.lock().unwrap().deal_requests.len() < first_deal_requests);
    let bids2: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks_history WHERE side = 'bid'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(bids2, bids, "a re-fetched week stores no tick twice");
    let deals2: i64 = sqlx::query_scalar("SELECT count(*) FROM deals")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(deals2, 3);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn only_read_requests_reach_the_broker_during_a_sync(pool: PgPool) {
    prepare(&pool).await;
    let log = sync(&pool, fake()).await;
    for t in &log.lock().unwrap().payload_types {
        assert!(jiaoyi_ctrader::check_sendable(*t).is_ok(), "{t}");
    }
}

/// Review finding on #4: a complete 1-day deal window must not hide the rest
/// of a later 7-day window that starts on the same day.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn widening_the_deal_window_fetches_the_days_not_yet_covered(pool: PgPool) {
    use jiaoyi_store::history::{self as hist, DealFetchRecord, DealUpsert};
    prepare(&pool).await;
    let day = 86_400_000i64;
    let now = Utc::now().timestamp_millis();
    // A 7-day grid window that ended about two weeks ago.
    let w0 = (now - 21 * day).div_euclid(7 * day) * 7 * day;
    let at = |ms: i64| chrono::DateTime::<Utc>::from_timestamp_millis(ms).unwrap();
    // An earlier sync with deals.window_days = 1 fetched day 1 only.
    settings::change(
        &pool,
        "deals.window_days",
        None,
        Some(json!(1)),
        "test",
        "daily windows",
    )
    .await
    .unwrap();
    hist::record_deal_fetch(
        &pool,
        &DealFetchRecord {
            account_id: A,
            window_from: at(w0),
            window_to: at(w0 + day - 1),
            complete: true,
            deals: 0,
            upsert: DealUpsert::default(),
            error: None,
        },
    )
    .await
    .unwrap();
    settings::change(
        &pool,
        "deals.window_days",
        None,
        Some(json!(7)),
        "test",
        "weekly windows",
    )
    .await
    .unwrap();
    // A deal on day 3 of that week; the account registered on its first day.
    let mut f = fake();
    f.deals = vec![ProtoOaDeal {
        deal_id: 42,
        order_id: 43,
        position_id: 44,
        volume: 100_000,
        filled_volume: 100_000,
        symbol_id: 1,
        create_timestamp: w0 + 2 * day + 3_600_000,
        execution_timestamp: w0 + 2 * day + 3_600_000,
        trade_side: 1,
        deal_status: 2,
        ..Default::default()
    }];
    f.trader.as_mut().unwrap().registration_timestamp = Some(w0);
    let log = sync(&pool, f).await;
    let found: Option<i64> = sqlx::query_scalar("SELECT deal_id FROM deals WHERE deal_id = 42")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert_eq!(found, Some(42), "days 2-7 were fetched");
    assert!(log
        .lock()
        .unwrap()
        .deal_requests
        .iter()
        .any(|(from, to)| *from == w0 && *to == w0 + 7 * day - 1));
}

/// Review finding on #4: (t, 100), (t, 101), (t, 100) at one millisecond are
/// three ticks, and all three are stored.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn alternating_ticks_in_one_millisecond_are_all_stored(pool: PgPool) {
    prepare(&pool).await;
    let t = Utc::now().timestamp_millis() - 3_600_000;
    let mut f = fake();
    f.tick_history = HashMap::from([((1, 1), vec![(t, 108_500), (t, 108_501), (t, 108_500)])]);
    sync(&pool, f).await;
    let rows: Vec<(i64, i32)> = sqlx::query_as(
        "SELECT price_e5, dup FROM ticks_history WHERE side = 'bid' ORDER BY price_e5, dup",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows, vec![(108_500, 0), (108_500, 1), (108_501, 0)]);
}
