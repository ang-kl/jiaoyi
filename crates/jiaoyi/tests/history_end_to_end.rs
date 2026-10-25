//! The history sync (deals, ticks, probe) and `/research/inventory` against
//! the in-memory fake broker and a real PostgreSQL.

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jiaoyi::config::BrokerConfig;
use jiaoyi::history::{self, GRID_ORIGIN_MS, WEEK_MS};
use jiaoyi::{loader, research, shutdown};
use jiaoyi_core::allowlist::{
    demo_accounts, plan_accounts, Allowlist, Environment, PermittedAccount, TickHistory,
};
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages::{
    ProtoOaClosePositionDetail, ProtoOaDeal, ProtoOaLightSymbol, ProtoOaPayloadType, ProtoOaSymbol,
    ProtoOaTrader,
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
    sync_as(pool, f, &permitted(), &TickHistory::On, &Default::default()).await
}

async fn sync_as(
    pool: &PgPool,
    f: FakeBroker,
    account: &PermittedAccount,
    ticks: &TickHistory,
    status: &history::HistoryStatusMap,
) -> std::sync::Arc<std::sync::Mutex<jiaoyi_ctrader::testkit::FakeLog>> {
    let (conn, log) = f.spawn();
    let session = loader::authorise(pool, &conn, &broker(), account)
        .await
        .unwrap();
    let (_t, stop) = shutdown::channel();
    history::sync_account(pool, &session, account, ticks, status, &stop)
        .await
        .unwrap();
    log
}

/// The plan as deployed on 25-10-2026 with A named only by a demo variable:
/// A's account and its tick-history setting, as the service would run them.
fn named_by_a_demo_variable() -> (PermittedAccount, TickHistory) {
    let list = Allowlist::parse(Some("43002148"))
        .unwrap()
        .with_demo_accounts(demo_accounts([("JIAOYI_DEMO_ACCOUNT_ID_2", "49236051")]));
    let plan = plan_accounts(&list, Some("43002148:live"));
    let (account, env) = plan
        .permitted
        .iter()
        .find(|(a, _)| a.id() == A)
        .cloned()
        .unwrap();
    assert_eq!(env, Environment::Demo);
    let ticks = plan.tick_history(A);
    (account, ticks)
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

// Claude · № 16,973 25-Oct (ordered № 16,972·C; owner: "Skip their tick history")
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_account_named_only_by_a_demo_variable_downloads_no_ticks_and_the_rest_runs(
    pool: PgPool,
) {
    prepare(&pool).await;
    let (account, ticks) = named_by_a_demo_variable();
    assert!(
        matches!(&ticks, TickHistory::Off { reason } if reason.contains("JIAOYI_DEMO_ACCOUNT_ID_2")),
        "{ticks:?}"
    );
    let broker_data = fake();
    let status = history::HistoryStatusMap::default();
    let log = sync_as(&pool, broker_data.clone(), &account, &ticks, &status).await;

    // Deals, the trader record and the probe: as for any account.
    assert_eq!(count(&pool, "SELECT count(*) FROM deals").await, 3);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM account_snapshots").await,
        1
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM trader_reads").await, 1);
    assert!(!log.lock().unwrap().deal_requests.is_empty());
    let probes = count(
        &pool,
        "SELECT count(*) FROM history_fetches WHERE kind = 'probe'",
    )
    .await;
    assert!(probes > 0, "the availability probe still runs");
    // No tick window downloaded or recorded, no tick stored, no ask asked.
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM history_fetches WHERE kind = 'download'"
        )
        .await,
        0
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM ticks_history").await, 0);
    let requests = log.lock().unwrap().tick_requests.clone();
    assert_eq!(
        requests.len() as i64,
        probes,
        "only probe requests: {requests:?}"
    );
    assert!(
        requests.iter().all(|(_, side, _, _)| *side == 1),
        "{requests:?}"
    );
    let s = status.lock().unwrap().get(&A).cloned().unwrap();
    assert_eq!((s.tick_windows_fetched, s.ticks_stored), (0, 0));
    assert!(s.deal_windows_fetched > 0 && s.probes > 0);

    // The same account with tick history on (a regular account) downloads.
    sync_as(&pool, broker_data, &permitted(), &TickHistory::On, &status).await;
    assert!(
        count(
            &pool,
            "SELECT count(*) FROM history_fetches WHERE kind = 'download'"
        )
        .await
            > 0
    );
    assert!(count(&pool, "SELECT count(*) FROM ticks_history").await > 0);
}

type TraderCols = (
    Option<i64>,
    Option<i64>,
    i64,
    Option<i32>,
    i64,
    Option<i32>,
    Option<bool>,
    Option<i64>,
    Option<String>,
);

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_trader_record_is_stored_with_the_broker_leverage_and_nulls_where_absent(pool: PgPool) {
    prepare(&pool).await;
    let mut f = fake();
    let t = f.trader.as_mut().unwrap();
    t.leverage_in_cents = Some(20_000);
    t.account_type = Some(1);
    // max_leverage, is_limited_risk and broker_name are left out.
    sync(&pool, f.clone()).await;
    let row: TraderCols = sqlx::query_as(
        "SELECT leverage_in_cents, max_leverage, balance, money_digits, deposit_asset_id,
                account_type, is_limited_risk, trader_login, broker_name
         FROM trader_reads WHERE account_id = $1",
    )
    .bind(A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            Some(20_000),
            None,
            1_001_000,
            Some(2),
            15,
            Some(1),
            None,
            Some(5361573),
            None
        )
    );
    // Every read is kept: a second sync adds a row, nothing is rewritten.
    sync(&pool, f).await;
    assert_eq!(count(&pool, "SELECT count(*) FROM trader_reads").await, 2);
    let up = sqlx::query("UPDATE trader_reads SET leverage_in_cents = 1")
        .execute(&pool)
        .await;
    assert!(up.unwrap_err().to_string().contains("append-only"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn health_shows_the_broker_leverage_and_tick_history_and_no_money(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    prepare(&pool).await;
    let (account, ticks) = named_by_a_demo_variable();
    let mut f = fake();
    let t = f.trader.as_mut().unwrap();
    t.leverage_in_cents = Some(20_000);
    t.account_type = Some(0);
    t.is_limited_risk = Some(false);
    let status = history::HistoryStatusMap::default();
    let (trigger, stop) = shutdown::channel();
    let task = tokio::spawn(history::run_with(
        pool.clone(),
        broker(),
        account,
        ticks,
        status.clone(),
        move || {
            let f = f.clone();
            async move { Ok(f.spawn().0) }
        },
        stop,
    ));
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    while status
        .lock()
        .unwrap()
        .get(&A)
        .and_then(|s| s.last_sync_finished_at)
        .is_none()
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    trigger.fire();
    let _ = task.await;

    let app = jiaoyi::http::router(jiaoyi::http::AppState {
        pool: pool.clone(),
        loading: Default::default(),
        recorders: Default::default(),
        history: status,
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(std::collections::BTreeSet::from([A])),
    });
    let res = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let b = &v["broker_accounts"][0];
    assert_eq!(b["account_id"], A);
    assert_eq!(b["leverage"], "1:200");
    assert_eq!(b["leverage_in_cents"], 20_000);
    assert_eq!(b["account_type"], "HEDGED");
    assert_eq!(b["is_limited_risk"], false);
    assert!(b["max_leverage"].is_null());
    assert!(b["read_at"].is_string());
    let th = &v["history_sync"][A.to_string()]["tick_history"];
    assert_eq!(th["state"], "off", "{th}");
    assert!(
        th["reason"]
            .as_str()
            .unwrap()
            .starts_with("named by JIAOYI_DEMO_ACCOUNT_ID_2 only"),
        "{th}"
    );
    // No money: the balance (1001000, money digits 2) is nowhere on it.
    assert!(
        !text.contains("1001000") && !text.contains("10010.00") && !text.contains("balance"),
        "{text}"
    );
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
            session_ended: false,
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

/// Review of #4, round 2: a rate-limited probe step is a failure, not an
/// empty week. The search ends incomplete with no boundary, and the next
/// sync probes again (no cooldown after an incomplete search).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rate_limited_probe_step_leaves_the_boundary_unset_and_is_probed_again(pool: PgPool) {
    prepare(&pool).await;
    let broker_data = fake();
    let mut limited = broker_data.clone();
    // Weeks older than 10 weeks are refused: the binary search's second step
    // (about a year back) hits the limit.
    limited.block_ticks_before = Some(Utc::now().timestamp_millis() - 10 * WEEK_MS);
    sync(&pool, limited).await;

    let inv = research::build(&pool).await.unwrap();
    let probe = &inv.accounts[0].symbols[0].probe;
    assert_eq!(probe.earliest_week_with_ticks, None, "{probe:?}");
    assert_eq!(probe.probed_at, None, "{probe:?}");
    assert!(
        probe
            .last_incomplete_error
            .as_deref()
            .unwrap_or("")
            .contains("BLOCKED_PAYLOAD_TYPE"),
        "{probe:?}"
    );
    let failed_steps: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM history_fetches WHERE kind = 'probe' AND status = 'failed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(failed_steps, 1, "the failed request is recorded");
    assert!(inv.accounts[0].symbols[0]
        .missing
        .iter()
        .any(|m| m.contains("did not complete")));

    // The next sync probes again and finds the true boundary, 40 weeks back.
    let log = sync(&pool, broker_data).await;
    assert!(
        log.lock()
            .unwrap()
            .tick_requests
            .iter()
            .any(|(_, _, from, _)| *from < Utc::now().timestamp_millis() - 10 * WEEK_MS),
        "probed again"
    );
    let inv = research::build(&pool).await.unwrap();
    let want = chrono::DateTime::<Utc>::from_timestamp_millis(week_start(
        Utc::now().timestamp_millis() - 40 * WEEK_MS,
    ))
    .unwrap()
    .date_naive();
    let probe = &inv.accounts[0].symbols[0].probe;
    assert_eq!(probe.earliest_week_with_ticks, Some(want), "{probe:?}");
    assert_eq!(probe.status.as_deref(), Some("complete"));
}

/// Review of #4, round 2: a fetch read with two difference signs is counted
/// once in the inventory's stored ticks.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_fetch_with_two_signs_is_counted_once(pool: PgPool) {
    use jiaoyi_store::history::{self as hist, FetchRecord, HistoryTick};
    prepare(&pool).await;
    let now = Utc::now();
    let ticks: Vec<HistoryTick> = (0..10)
        .map(|i| HistoryTick {
            ts: now - Duration::hours(1) + Duration::seconds(i),
            price_e5: 108_500 + i,
            dup: 0,
        })
        .collect();
    let (_, stored) = hist::record_window(
        &pool,
        &FetchRecord {
            account_id: A,
            broker_symbol_id: 1,
            side: "bid",
            kind: "download",
            window_from: now - Duration::days(1),
            window_to: now,
            complete: false,
            ticks: 10,
            pages: 2,
            first_tick: Some(ticks[0].ts),
            last_tick: Some(ticks[9].ts),
            delta_signs: vec!["subtract".into(), "add".into()],
            error: None,
            session_ended: false,
        },
        &ticks,
    )
    .await
    .unwrap();
    assert_eq!(stored, 10);
    let inv = research::build(&pool).await.unwrap();
    let bid = &inv.accounts[0].symbols[0].history_bid;
    assert_eq!(bid.ticks_stored, 10);
    assert_eq!(
        bid.delta_signs_seen,
        vec!["add".to_string(), "subtract".to_string()]
    );
}

/// PR #6, item A (deploy 7765158a, 24-10-2026): a token refresh for the demo
/// account invalidated the live session in the middle of its history sync,
/// and every later window failed (all 76 tick windows, 63 of 83 deal
/// windows). The sync now stops on the session end, re-authorises from the
/// shared token store and resumes at the first unfinished window; the window
/// cut short is recorded as `session_ended`, not as a broker failure.
async fn a_session_end_mid_sync(pool: &PgPool, payload_type: i32, nth: usize) {
    prepare(pool).await;
    // A clean broker: no impossible ask, so every window can complete.
    let mut f = fake();
    f.tick_history
        .get_mut(&(1, 2))
        .unwrap()
        .retain(|(_, price)| *price > 0);
    f.invalidate_on = Some((payload_type, Arc::new(AtomicUsize::new(nth))));
    // No pause before re-authorising. The setting is new in PR #6; before it
    // the change is refused and the sync is retried after 300 s.
    let _ = settings::change(
        pool,
        "history.session_retry_secs",
        None,
        Some(json!(0)),
        "test",
        "no wait",
    )
    .await;
    let status = history::HistoryStatusMap::default();
    let (trigger, stop) = shutdown::channel();
    let f2 = f.clone();
    let connects = Arc::new(AtomicUsize::new(0));
    let c2 = connects.clone();
    let task = tokio::spawn(history::run_with(
        pool.clone(),
        broker(),
        permitted(),
        TickHistory::On,
        status.clone(),
        move || {
            let f = f2.clone();
            c2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move { Ok(f.spawn().0) }
        },
        stop,
    ));
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    while status
        .lock()
        .unwrap()
        .get(&A)
        .and_then(|s| s.last_sync_finished_at)
        .is_none()
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    trigger.fire();
    let _ = task.await;
    let st = status.lock().unwrap().get(&A).cloned().unwrap_or_default();
    assert!(
        st.last_sync_finished_at.is_some(),
        "the sync finished: {st:?}"
    );
    // One session end, one window cut short, one re-authorised connection.
    // (Read through the JSON /health shows, so the test builds against the
    // status shape from before the fix too.)
    let sv = serde_json::to_value(&st).unwrap();
    assert_eq!(sv["session_ends"], 1, "{sv}");
    assert_eq!(sv["windows_cut_by_session_end"], 1, "{sv}");
    assert_eq!(sv["tick_windows_failed"], 0, "{sv}");
    assert_eq!(connects.load(std::sync::atomic::Ordering::SeqCst), 2);

    // Every window completed: no failed tick week, no failed deal window.
    let inv = research::build(pool).await.unwrap();
    let eur = &inv.accounts[0].symbols[0];
    for side in [&eur.history_bid, &eur.history_ask] {
        assert!(side.weeks_failed.is_empty(), "{:?}", side.weeks_failed);
        assert!(
            side.weeks_missing.len() <= 1,
            "only the current week: {:?}",
            side.weeks_missing
        );
    }
    assert_eq!(
        inv.accounts[0].deals.windows_failed, 0,
        "{:?}",
        inv.accounts[0].deals
    );
    assert!(inv.accounts[0].deals.windows_ok > 0);
    let broker_failures: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM history_fetches WHERE status = 'failed' AND failure_kind = 'broker')
              + (SELECT count(*) FROM deal_fetches WHERE status = 'failed' AND failure_kind = 'broker')",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(broker_failures, 0);
    // The window cut short is on record, with its reason.
    let ended: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM history_fetches WHERE failure_kind = 'session_ended')
              + (SELECT count(*) FROM deal_fetches WHERE failure_kind = 'session_ended')",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(ended, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_session_ended_during_the_tick_download_is_reauthorised_and_resumed(pool: PgPool) {
    a_session_end_mid_sync(&pool, ProtoOaPayloadType::ProtoOaGetTickdataReq as i32, 2).await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_session_ended_during_the_deal_download_is_reauthorised_and_resumed(pool: PgPool) {
    a_session_end_mid_sync(&pool, ProtoOaPayloadType::ProtoOaDealListReq as i32, 2).await;
}

/// PR #6, item B: `/research/inventory` names, per account, each distinct
/// error of the failed tick windows, deal windows and probe steps, with its
/// count and the last time it happened.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_inventory_names_each_distinct_fetch_error_with_count_and_last_time(pool: PgPool) {
    prepare(&pool).await;
    let tick = |from: &str, kind: &str, error: Option<&str>, at: &str| {
        format!(
            "INSERT INTO history_fetches (account_id, broker_symbol_id, side, kind, window_from, window_to,
                                          fetched_at, status, complete, error)
             VALUES ({A}, 1, 'bid', '{kind}', '{from}', '{from}'::timestamptz + interval '6 days',
                     '{at}', '{}', true, {})",
            if error.is_some() { "failed" } else { "ok" },
            error.map_or("NULL".to_string(), |e| format!("'{e}'"))
        )
    };
    for q in [
        tick("2026-09-07", "download", Some("broker error ACCOUNT_NOT_AUTHORIZED"), "2026-10-24 01:00Z"),
        tick("2026-09-14", "download", Some("broker error ACCOUNT_NOT_AUTHORIZED"), "2026-10-24 02:00Z"),
        tick("2026-09-21", "download", Some("tick data: does not decode"), "2026-10-24 03:00Z"),
        tick("2026-09-28", "download", None, "2026-10-24 04:00Z"),
        tick("2015-01-05", "probe", Some("broker error REQUEST_FREQUENCY_EXCEEDED"), "2026-10-24 05:00Z"),
        format!(
            "INSERT INTO deal_fetches (account_id, window_from, window_to, fetched_at, status, complete, error)
             VALUES ({A}, '2026-08-01', '2026-08-07', '2026-10-24 06:00Z', 'failed', true,
                     'broker error ACCOUNT_NOT_AUTHORIZED')"
        ),
    ] {
        sqlx::query(&q).execute(&pool).await.unwrap();
    }
    let inv = research::build(&pool).await.unwrap();
    let v = serde_json::to_value(&inv).unwrap();
    let errors = v["accounts"][0]["fetch_errors"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let find = |kind: &str, text: &str| {
        errors
            .iter()
            .find(|e| e["kind"] == kind && e["error"] == text)
            .cloned()
            .unwrap_or_else(|| panic!("{kind} {text} not in {errors:?}"))
    };
    let e = find("tick_window", "broker error ACCOUNT_NOT_AUTHORIZED");
    assert_eq!(e["count"], 2);
    assert!(
        e["last_at"]
            .as_str()
            .unwrap()
            .starts_with("2026-10-24T02:00:00"),
        "{e}"
    );
    assert_eq!(
        find("tick_window", "tick data: does not decode")["count"],
        1
    );
    assert_eq!(
        find("deal_window", "broker error ACCOUNT_NOT_AUTHORIZED")["count"],
        1
    );
    let p = find("probe_step", "broker error REQUEST_FREQUENCY_EXCEEDED");
    assert_eq!(p["count"], 1);
    assert_eq!(errors.len(), 4, "{errors:?}");
}
