//! `/research/bars` (R0 of the edge test): development-period daily bars,
//! exactly as stored, for allowlisted accounts only. The held-out period
//! (2025-10-24 → 2026-10-23) is sealed: a request reaching into it is
//! refused, never clamped (pre-registration 24-10-2026).

use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi::loader;
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::universe::AssetClass;
use jiaoyi_ctrader::messages::ProtoOaSymbol;
use jiaoyi_store::{self as store, BarRow};
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

const LIVE: i64 = 43002148;
const DEMO: i64 = 49236051;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A bar dated `date` opening at `hour` UTC that day, prices in 1/100000.
fn bar(date: NaiveDate, hour: u32, o: i64, h: i64, l: i64, c: i64) -> BarRow {
    BarRow {
        bar_date: date,
        bar_open_utc: date.and_hms_opt(hour, 0, 0).unwrap().and_utc(),
        open: Decimal::new(o, 5),
        high: Decimal::new(h, 5),
        low: Decimal::new(l, 5),
        close: Decimal::new(c, 5),
        tick_volume: 1000 + o % 7,
    }
}

async fn symbol(pool: &PgPool, account: i64, id: i64, name: &str, class: AssetClass) {
    let s = ProtoOaSymbol {
        symbol_id: id,
        digits: 5,
        pip_position: 4,
        ..Default::default()
    };
    store::upsert_symbol(
        pool,
        &loader::symbol_row(account, name, class, Some(true), &s, Utc::now()),
    )
    .await
    .unwrap();
}

/// Live 43002148: EURUSD (id 1) with FX-style bars opening 21:00 UTC on
/// 20..23-10-2025, and US500 (id 2) with bars opening 00:00 UTC on 22..23-10.
async fn setup(pool: &PgPool) {
    store::upsert_account(pool, LIVE, Environment::Live)
        .await
        .unwrap();
    store::upsert_account(pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    symbol(pool, LIVE, 1, "EURUSD", AssetClass::Fx).await;
    symbol(pool, LIVE, 2, "US500", AssetClass::EquityIndex).await;
    let fx = vec![
        bar(day(2025, 10, 20), 21, 116_012, 116_480, 115_903, 116_402),
        bar(day(2025, 10, 21), 21, 116_402, 116_455, 115_871, 116_078),
        bar(day(2025, 10, 22), 21, 116_078, 116_299, 115_955, 116_120),
        // Dated 23-10 by its UTC open, it runs to 21:00 on 24-10: sealed.
        bar(day(2025, 10, 23), 21, 116_120, 116_250, 115_990, 116_200),
    ];
    store::upsert_daily_bars(pool, LIVE, 1, None, &fx, Utc::now())
        .await
        .unwrap();
    let idx = vec![
        bar(
            day(2025, 10, 22),
            0,
            667_000_000,
            670_000_000,
            665_000_000,
            669_900_000,
        ),
        // Opens at midnight on 23-10: ends exactly at the seal.
        bar(
            day(2025, 10, 23),
            0,
            669_900_000,
            674_000_000,
            668_000_000,
            673_800_000,
        ),
        bar(
            day(2025, 10, 24),
            0,
            673_800_000,
            675_000_000,
            672_000_000,
            674_500_000,
        ),
    ];
    store::upsert_daily_bars(pool, LIVE, 2, None, &idx, Utc::now())
        .await
        .unwrap();
    // GER40 (id 4): nothing stored after 21-10-2025.
    symbol(pool, LIVE, 4, "GER40", AssetClass::EquityIndex).await;
    let ger = vec![bar(
        day(2025, 10, 21),
        0,
        2_400_000_000,
        2_410_000_000,
        2_390_000_000,
        2_405_000_000,
    )];
    store::upsert_daily_bars(pool, LIVE, 4, None, &ger, Utc::now())
        .await
        .unwrap();
}

async fn get(pool: &PgPool, uri: &str) -> (StatusCode, Value) {
    let app = router(AppState {
        pool: pool.clone(),
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([DEMO, LIVE])),
    });
    let res = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn development_bars_come_back_exactly_as_stored(pool: PgPool) {
    setup(&pool).await;
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=EURUSD&from=2025-10-21&to=2025-10-22",
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["count"], 2);
    assert_eq!(v["truncated"], false);
    assert_eq!(v["broker_symbol_id"], 1);
    let b = &v["bars"][0];
    assert_eq!(b["bar_date"], "2025-10-21");
    assert_eq!(b["bar_open_utc"], "2025-10-21T21:00:00Z");
    // Exact decimal strings, as stored.
    assert_eq!(
        (&b["open"], &b["high"], &b["low"], &b["close"]),
        (
            &Value::from("1.16402"),
            &Value::from("1.16455"),
            &Value::from("1.15871"),
            &Value::from("1.16078")
        )
    );
    assert_eq!(v["bars"][1]["bar_date"], "2025-10-22");
    // Without `from`: from the first stored bar.
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=EURUSD&to=2025-10-22",
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["count"], 3);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_request_reaching_the_held_out_period_is_refused_not_clamped(pool: PgPool) {
    setup(&pool).await;
    // `to` on or after the seal: refused, whatever is stored. GER40 holds
    // nothing after 21-10, so only the `to` check can refuse it.
    for symbol in ["US500", "GER40"] {
        for to in ["2025-10-24", "2026-10-23", "2027-01-01"] {
            let (st, v) = get(
                &pool,
                &format!("/research/bars?account=43002148&symbol={symbol}&from=2025-10-20&to={to}"),
            )
            .await;
            assert_eq!(st, StatusCode::FORBIDDEN, "{symbol} {to}: {v}");
            let why = v["reason"].as_str().unwrap();
            assert!(why.contains("sealed") && why.contains("not clamped"), "{v}");
            assert!(v.get("bars").is_none(), "nothing is returned: {v}");
        }
    }
    // The day before the seal: a bar opening at midnight on 23-10 ends at the
    // seal and is development data.
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=US500&from=2025-10-22&to=2025-10-23",
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["count"], 2);
    // An FX bar dated 23-10 opens at 21:00 and runs past the seal: the
    // request is refused (not clamped to 22-10), and says what to ask.
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=EURUSD&from=2025-10-20&to=2025-10-23",
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "{v}");
    let why = v["reason"].as_str().unwrap();
    assert!(
        why.contains("2025-10-23T21:00:00") && why.contains("2025-10-22"),
        "{why}"
    );
    assert!(v.get("bars").is_none());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn only_allowlisted_accounts_are_served(pool: PgPool) {
    setup(&pool).await;
    let (st, v) = get(
        &pool,
        "/research/bars?account=11111111&symbol=EURUSD&to=2025-10-22",
    )
    .await;
    assert_eq!(st, StatusCode::FORBIDDEN, "{v}");
    assert!(
        v["reason"]
            .as_str()
            .unwrap()
            .contains("not on JIAOYI_ALLOWED_ACCOUNTS"),
        "{v}"
    );
    // Allowlisted, but nothing stored under the name for that account.
    let (st, v) = get(
        &pool,
        "/research/bars?account=49236051&symbol=EURUSD&to=2025-10-22",
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND, "{v}");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn missing_parameters_are_refused(pool: PgPool) {
    setup(&pool).await;
    for (uri, missing) in [
        ("/research/bars?symbol=EURUSD&to=2025-10-22", "account"),
        ("/research/bars?account=43002148&to=2025-10-22", "symbol"),
        ("/research/bars?account=43002148&symbol=EURUSD", "to"),
    ] {
        let (st, v) = get(&pool, uri).await;
        assert_eq!(st, StatusCode::BAD_REQUEST, "{uri}: {v}");
        assert!(v["reason"].as_str().unwrap().starts_with(missing), "{v}");
    }
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=EURUSD&from=2025-10-22&to=2025-10-21",
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "{v}");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_long_range_is_capped_and_says_where_to_continue(pool: PgPool) {
    setup(&pool).await;
    symbol(&pool, LIVE, 3, "XAUUSD", AssetClass::Metal).await;
    let first = day(2005, 1, 1);
    let rows: Vec<BarRow> = (0..5003)
        .map(|i| {
            bar(
                first + Duration::days(i),
                0,
                400_000,
                401_000,
                399_000,
                400_500,
            )
        })
        .collect();
    store::upsert_daily_bars(&pool, LIVE, 3, None, &rows, Utc::now())
        .await
        .unwrap();
    let (st, v) = get(
        &pool,
        "/research/bars?account=43002148&symbol=XAUUSD&to=2025-10-22",
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["count"], 5000);
    assert_eq!(v["truncated"], true);
    let next: NaiveDate = serde_json::from_value(v["next_from"].clone()).unwrap();
    assert_eq!(next, first + Duration::days(5000));
    let (st, v) = get(
        &pool,
        &format!("/research/bars?account=43002148&symbol=XAUUSD&from={next}&to=2025-10-22"),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{v}");
    assert_eq!(v["count"], 3);
    assert_eq!(v["truncated"], false);
    let _: DateTime<Utc> = serde_json::from_value(v["bars"][0]["bar_open_utc"].clone()).unwrap();
}

/// Codex P2 on #8: the whole requested range is checked against the seal
/// before it is paged. A long range whose first 5,000 bars are old must
/// still be refused when it holds a bar running past the seal.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_long_range_holding_a_sealed_bar_is_refused_before_paging(pool: PgPool) {
    setup(&pool).await;
    symbol(&pool, LIVE, 5, "GBPUSD", AssetClass::Fx).await;
    // 5,100 FX-style bars opening 21:00 UTC, the last dated 23-10-2025
    // (it runs to 21:00 on 24-10, past the seal).
    let last = day(2025, 10, 23);
    let rows: Vec<BarRow> = (0..5100)
        .map(|i| {
            bar(
                last - Duration::days(5099 - i),
                21,
                130_000,
                130_500,
                129_500,
                130_200,
            )
        })
        .collect();
    store::upsert_daily_bars(&pool, LIVE, 5, None, &rows, Utc::now())
        .await
        .unwrap();
    let from = last - Duration::days(5099);
    let (st, v) = get(
        &pool,
        &format!("/research/bars?account=43002148&symbol=GBPUSD&from={from}&to=2025-10-23"),
    )
    .await;
    assert_eq!(
        st,
        StatusCode::FORBIDDEN,
        "{}",
        &v.to_string()[..200.min(v.to_string().len())]
    );
    let why = v["reason"].as_str().unwrap();
    assert!(
        why.contains("2025-10-23T21:00:00") && why.contains("not clamped"),
        "{why}"
    );
    assert!(v.get("bars").is_none());
    // To the day before: the whole range is development data, served in pages.
    let (st, v) = get(
        &pool,
        &format!("/research/bars?account=43002148&symbol=GBPUSD&from={from}&to=2025-10-22"),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        (v["count"].clone(), v["truncated"].clone()),
        (5000.into(), true.into())
    );
}
