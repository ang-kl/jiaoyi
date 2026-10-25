//! Strategy 3 (intraday momentum, tick-triggered on the noise band) against a
//! real PostgreSQL and the scripted broker: the band from the broker's M30
//! bars, the entry on the first quote beyond it between 30-minute marks,
//! TP2 at the broker, the TP1 partial close, the session-end close, the
//! budget, the live single minimum, the holiday skip and the records.
// Claude · № 17,020 25-Oct (ordered: owner's Strategy 3 v2 approval, "Tick-triggered only")

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use jiaoyi::config::BrokerConfig;
use jiaoyi::{history, loader, strategy3};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_core::strategy3::sessions::{rule_for, session_on, Session};
use jiaoyi_core::universe::{AssetClass, Resolution};
use jiaoyi_ctrader::messages::{
    ProtoOaDeal, ProtoOaPosition, ProtoOaSymbol, ProtoOaTradeData, ProtoOaTrader,
};
use jiaoyi_ctrader::testkit::{m30_bar, FakeBroker, FakeLog, GrantEntry};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::PgPool;

const DEMO: i64 = 49236051;
const BTC: i64 = 22;
/// The session's open, 60,000.00.
const OPEN: f64 = 60_000.0;

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

fn allowlist() -> Allowlist {
    Allowlist::parse(Some("49236051")).unwrap()
}

fn permitted() -> PermittedAccount {
    allowlist().permit(DEMO).unwrap()
}

/// Today's BTCUSD session (the UTC day).
fn today() -> Session {
    session_on(
        &rule_for("BTCUSD", AssetClass::Crypto).unwrap(),
        Utc::now().date_naive(),
    )
    .unwrap()
}

fn e5(p: f64) -> i64 {
    (p * 100_000.0).round() as i64
}

/// The stored data a load leaves for BTCUSD (digits 2, minimum and step 1
/// cent), the universe of one symbol, the deposit (USD), one deal with a $3
/// commission per BTC, and recorded quotes 5.00 wide.
async fn loaded(pool: &PgPool) {
    store::upsert_account(pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    settings::change(
        pool,
        "universe.symbols",
        Some(DEMO),
        Some(json!([{"name": "BTCUSD", "class": "crypto"}])),
        "test",
        "one symbol",
    )
    .await
    .unwrap();
    let s = ProtoOaSymbol {
        symbol_id: BTC,
        digits: 2,
        pip_position: 2,
        min_volume: Some(1),
        step_volume: Some(1),
        ..Default::default()
    };
    store::upsert_symbol(
        pool,
        &loader::symbol_row(
            DEMO,
            "BTCUSD",
            AssetClass::Crypto,
            Some(true),
            &s,
            Utc::now(),
        ),
    )
    .await
    .unwrap();
    let run = store::start_run(pool, DEMO).await.unwrap();
    store::record_resolution(
        pool,
        run,
        DEMO,
        "BTCUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: BTC },
    )
    .await
    .unwrap();
    store::finish_run(
        pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(json!([{"id": BTC, "quote_asset_id": 1, "base_asset_id": 9}])),
            assets: Some(json!([{"asset_id": 1, "name": "USD"}, {"asset_id": 9, "name": "BTC"}])),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    store::history::record_account_snapshot(
        pool,
        &store::history::AccountSnapshot {
            account_id: DEMO,
            trader_login: None,
            deposit_asset_id: 1,
            balance: 1_000_000,
            money_digits: Some(2),
            registration_at: None,
            raw: vec![],
        },
    )
    .await
    .unwrap();
    // A deal of 1 BTC (100 cents) with a $3.00 commission: $3 per unit a side.
    let deal = ProtoOaDeal {
        deal_id: 1,
        order_id: 2,
        position_id: 3,
        volume: 100,
        filled_volume: 100,
        symbol_id: BTC,
        create_timestamp: 0,
        execution_timestamp: Utc::now().timestamp_millis() - 86_400_000,
        execution_price: Some(60_000.0),
        trade_side: 1,
        deal_status: 2,
        commission: Some(-300),
        money_digits: Some(2),
        ..Default::default()
    };
    store::history::upsert_deals(pool, DEMO, &[history::deal_row(&deal)], Utc::now())
        .await
        .unwrap();
    // Recorded quotes, 5.00 wide.
    store::ticks::ensure_partitions(pool, Utc::now().date_naive() - Duration::days(1), 3)
        .await
        .unwrap();
    let rows: Vec<store::ticks::TickRow> = (0..50)
        .map(|i| store::ticks::TickRow {
            account_id: DEMO,
            session_id: "t".into(),
            seq: i,
            broker_symbol_id: BTC,
            received_at: Utc::now() - Duration::seconds(i),
            broker_timestamp: None,
            bid_e5: Some(e5(60_000.0)),
            ask_e5: Some(e5(60_005.0)),
        })
        .collect();
    store::ticks::insert(pool, &rows).await.unwrap();
}

/// M30 bars: 20 past days, each opening at 60,000.00; the bar ending at +30
/// closes 0.5% from the open, the one ending at +60 1.0% (signs
/// alternating), so σ(30) = 0.005, σ(60) = 0.010 and σ(45) = 0.0075. Today:
/// its first bar opens at 60,000.00 with `first_volume` ticks.
fn bars(first_volume: i64, days: i64) -> Vec<jiaoyi_ctrader::messages::ProtoOaTrendbar> {
    let s = today();
    let mut v = Vec::new();
    for i in 1..=days {
        let open = s.open - Duration::days(i);
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        let c30 = OPEN * (1.0 + sign * 0.005);
        let c60 = OPEN * (1.0 + sign * 0.010);
        let (lo, hi) = (59_000.0, 61_000.0);
        v.push(m30_bar(open, e5(OPEN), e5(hi), e5(lo), e5(c30), 100));
        v.push(m30_bar(
            open + Duration::minutes(30),
            e5(c30),
            e5(hi),
            e5(lo),
            e5(c60),
            100,
        ));
    }
    v.push(m30_bar(
        s.open,
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        first_volume,
    ));
    v
}

fn fake(fill: (f64, f64)) -> FakeBroker {
    FakeBroker {
        account_id: DEMO,
        m30_bars: HashMap::from([(BTC, bars(10, 20))]),
        fill_quotes: HashMap::from([(BTC, fill)]),
        trader: Some(ProtoOaTrader {
            ctid_trader_account_id: DEMO,
            balance: 1_000_000,
            deposit_asset_id: 1,
            money_digits: Some(2),
            account_type: Some(0),
            ..Default::default()
        }),
        grant: Some(vec![GrantEntry {
            ctid: DEMO,
            is_live: Some(false),
            login: None,
        }]),
        ..Default::default()
    }
}

async fn decisions(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as("SELECT kind, reason FROM trading_decisions WHERE account_id = $1 ORDER BY id")
        .bind(DEMO)
        .fetch_all(pool)
        .await
        .unwrap()
}

/// Drive a trader on `f`: `steps` gets the trader and the fake's log.
macro_rules! with_trader {
    ($pool:expr, $f:expr, $env:expr, |$t:ident, $log:ident| $body:block) => {{
        let (conn, $log) = $f.clone().spawn();
        let session = loader::authorise($pool, &conn, &broker(), &permitted())
            .await
            .unwrap();
        let list = allowlist();
        let mut $t = strategy3::Trader::new($pool, &list, $env, &session)
            .await
            .unwrap();
        $t.reconcile(Utc::now()).await.unwrap();
        $body
    }};
}

/// An order the fake received.
#[derive(Debug, Clone, PartialEq)]
struct Sent {
    trade_side: i32,
    label: Option<String>,
    relative_stop_loss: Option<i64>,
    relative_take_profit: Option<i64>,
    volume: i64,
}

fn orders(log: &Arc<Mutex<FakeLog>>) -> Vec<Sent> {
    log.lock()
        .unwrap()
        .orders
        .iter()
        .map(|o| Sent {
            trade_side: o.trade_side,
            label: o.label.clone(),
            relative_stop_loss: o.relative_stop_loss,
            relative_take_profit: o.relative_take_profit,
            volume: o.volume,
        })
        .collect()
}

fn at(min: i64) -> DateTime<Utc> {
    today().open + Duration::minutes(min)
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_quote_beyond_the_band_between_marks_enters_at_that_quote(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        // At 45 minutes σ is 0.0075 (between 0.005 at 30 and 0.010 at 60):
        // the band is 59,550.00 – 60,450.00. An ask of 60,440.00 is above the
        // band at the 30-minute mark (60,300.00) but inside it now: no entry.
        t.on_quote(BTC, Some(60_435.0), Some(60_440.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty(), "inside the band");
        // The first ask above 60,450.00 buys, at that quote.
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let o = orders(&log);
        assert_eq!(o.len(), 1, "{:?}", decisions(&pool).await);
        let o = &o[0];
        assert_eq!(o.trade_side, 1, "buy");
        assert_eq!(o.label.as_deref(), Some("jiaoyi-s3"));
        // R = 60,000 × 0.0075 = 450.00 from the fill.
        assert_eq!(o.relative_stop_loss, Some(45_000_000));
        // C = 5.00 spread + 6.00 commission + 2 × 24 bps × 60,460 (290.208)
        // = 301.208; TP1 = 3R + C = 1,651.21 (outward to a tick); TP2 =
        // TP1 + R = 2,101.21, sent as the broker take-profit.
        assert_eq!(o.relative_take_profit, Some(210_121_000));
        // 0.6% of $10,000 = $60 at $4.50 a cent: 13 cents.
        assert_eq!(o.volume, 13);
        // The records: the order with the hash, the label, the quote and
        // the band, and its fill (slippage 2.00 against the 60,460 ask).
        let (hash, label, bid, ask, detail): (String, String, f64, f64, serde_json::Value) =
            sqlx::query_as(
                "SELECT prereg_hash, label, bid, ask, detail FROM strategy3_orders WHERE kind = 'open'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(hash, strategy3::prereg_hash());
        assert_eq!(label, "jiaoyi-s3");
        assert_eq!((bid, ask), (60_455.0, 60_460.0));
        assert!((detail["band"]["upper"].as_f64().unwrap() - 60_450.0).abs() < 1e-6);
        assert!((detail["sigma"].as_f64().unwrap() - 0.0075).abs() < 1e-12);
        assert!((detail["entry"]["cost"]["total"].as_f64().unwrap() - 301.208).abs() < 1e-6);
        assert_eq!(detail["order"]["plan"]["close_volume"], 5);
        let (outcome, slippage, latency, fill): (String, f64, i64, f64) = sqlx::query_as(
            "SELECT outcome, slippage, latency_ms, fill_price FROM strategy3_fills WHERE kind = 'open'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(outcome, "filled");
        assert!((slippage - 2.0).abs() < 1e-9);
        assert!(latency >= 0);
        assert_eq!(fill, 60_462.0);
        // Every decision carries the hash and the label.
        let n: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM trading_decisions WHERE kind LIKE 's3\\_%'
               AND (detail->>'prereg_hash' IS DISTINCT FROM $1 OR detail->>'label' IS DISTINCT FROM 'jiaoyi-s3')",
        )
        .bind(strategy3::prereg_hash())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n, 0);
        // One position per symbol: another trigger sends nothing.
        t.on_quote(BTC, Some(60_500.0), Some(60_505.0), at(50))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);

        // TP1 at 60,462 + 1,651.21 = 62,113.21 on the bid: 5 of 13 close.
        t.on_quote(BTC, Some(62_113.20), Some(62_118.0), at(60))
            .await
            .unwrap();
        assert!(log.lock().unwrap().closes.is_empty());
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        t.on_quote(BTC, Some(62_200.0), Some(62_205.0), at(62))
            .await
            .unwrap();
        let closes = log.lock().unwrap().closes.clone();
        assert_eq!(closes.len(), 1, "once");
        assert_eq!(closes[0].volume, 5);

        // The session end closes what is left (8 cents).
        t.reconcile(Utc::now()).await.unwrap();
        t.session_end(at(23 * 60 + 59)).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "still in the session");
        t.session_end(at(24 * 60)).await.unwrap();
        let closes = log.lock().unwrap().closes.clone();
        assert_eq!(closes.len(), 2);
        assert_eq!(closes[1].volume, 8);
        t.session_end(at(24 * 60 + 1)).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 2, "sent once");
    });
    // /trading/costs: the entry's actual slippage and order-to-fill time.
    let c = store::strategy3::costs(&pool).await.unwrap();
    let row = &c["rows"][0];
    assert_eq!(row["symbol"], "BTCUSD");
    assert_eq!(row["entries_filled"], 1);
    assert_eq!(row["exits_filled"], 2);
    assert!((row["entry_slippage_median"].as_f64().unwrap() - 2.0).abs() < 1e-9);
    assert!(row["entry_order_to_fill_ms_median"].is_number());
    let text = c.to_string();
    assert!(
        !text.contains("balance") && !text.contains("equity"),
        "{text}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_sell_triggers_on_the_bid_below_the_band(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((59_540.0, 59_545.0)),
        Environment::Demo,
        |t, log| {
            // A bid of 59,550.00 is on the band: inside.
            t.on_quote(BTC, Some(59_550.0), Some(59_555.0), at(45))
                .await
                .unwrap();
            assert!(orders(&log).is_empty());
            t.on_quote(BTC, Some(59_549.99), Some(59_555.0), at(45))
                .await
                .unwrap();
            assert_eq!(orders(&log)[0].trade_side, 2, "sell");
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn no_entry_before_30_minutes_or_in_the_last_30(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((70_000.0, 70_005.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(29))
                .await
                .unwrap();
            t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(24 * 60 - 30))
                .await
                .unwrap();
            assert!(orders(&log).is_empty());
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_budget_and_a_holiday_skip_with_the_reason(pool: PgPool) {
    loaded(&pool).await;
    // 0.1% of $10,000 = $10, below the entry's $58.50.
    settings::change(
        &pool,
        "trading.s3_open_risk_pct",
        Some(DEMO),
        Some(json!(0.1)),
        "test",
        "small budget",
    )
    .await
    .unwrap();
    with_trader!(
        &pool,
        fake((60_455.0, 60_460.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
                .await
                .unwrap();
            assert!(orders(&log).is_empty());
        }
    );
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_skip" && r.contains("exceed its budget")),
        "{d:?}"
    );

    // A session whose first M30 bar has no ticks: a holiday, no entry.
    let mut f = fake((60_455.0, 60_460.0));
    f.m30_bars = HashMap::from([(BTC, bars(0, 20))]);
    settings::change(
        &pool,
        "trading.s3_open_risk_pct",
        Some(DEMO),
        Some(json!(3)),
        "test",
        "back",
    )
    .await
    .unwrap();
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_session" && r.contains("holiday")),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn fewer_than_14_sessions_give_no_band(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_460.0));
    f.m30_bars = HashMap::from([(BTC, bars(10, 13))]);
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_no_band" && r.contains("14 needed")),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn live_takes_one_minimum_trade_only_with_no_other_program_position(pool: PgPool) {
    loaded(&pool).await;
    // A Strategy 1 position open on another symbol: live waits.
    let f = fake((60_455.0, 60_460.0));
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 77,
        trade_data: ProtoOaTradeData {
            symbol_id: 5,
            volume: 100,
            trade_side: 1,
            label: Some("jiaoyi-s1".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        ..Default::default()
    });
    with_trader!(&pool, f, Environment::Live, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_skip" && r.contains("one minimum trade at a time")),
        "{d:?}"
    );
    // With none open: exactly the broker minimum, 1 cent.
    with_trader!(
        &pool,
        fake((60_455.0, 60_460.0)),
        Environment::Live,
        |t, log| {
            t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
                .await
                .unwrap();
            let o = orders(&log);
            assert_eq!(o.len(), 1, "{:?}", decisions(&pool).await);
            assert_eq!(o[0].volume, 1);
            // Unsplittable: the broker's take-profit is TP1, 1,651.21 from the fill.
            assert_eq!(o[0].relative_take_profit, Some(165_121_000));
        }
    );
}

#[test]
fn the_sessions_of_the_seven_indices_follow_their_exchanges() {
    let d = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
    for (n, open) in [
        ("US500", "13:30"),
        ("NAS100", "13:30"),
        ("US30", "13:30"),
        ("GER40", "08:00"),
        ("UK100", "08:00"),
        ("JPN225", "00:00"),
        ("AUS200", "23:00"),
    ] {
        let s = session_on(&rule_for(n, AssetClass::EquityIndex).unwrap(), d).unwrap();
        assert_eq!(s.open.format("%H:%M").to_string(), open, "{n}");
    }
}
