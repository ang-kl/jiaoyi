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
use jiaoyi::{history, loader, shutdown, strategy3, trading};
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
        assert_eq!(
            detail["volume_limits"],
            serde_json::to_value(t.syms[&BTC].limits.unwrap()).unwrap()
        );
        assert_eq!(detail["session"], serde_json::to_value(today()).unwrap());
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
    let c = store::trading::costs(&pool).await.unwrap();
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

fn s3_position(id: i64, volume: i64, price: f64) -> ProtoOaPosition {
    ProtoOaPosition {
        position_id: id,
        trade_data: ProtoOaTradeData {
            symbol_id: BTC,
            volume,
            trade_side: 1,
            label: Some("jiaoyi-s3".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        price: Some(price),
        stop_loss: Some(price - 450.0),
        take_profit: Some(price + 2_101.21),
        ..Default::default()
    }
}

/// Codex P1 on #15: an Unconfirmed open adopted at a reconcile (here after a
/// restart) keeps its plan: its TP1 partial and its session-end close.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unconfirmed_open_adopted_at_reconcile_takes_its_tp1_and_its_session_end(pool: PgPool) {
    loaded(&pool).await;
    settings::change(
        &pool,
        "trading.order_wait_secs",
        Some(DEMO),
        Some(json!(1)),
        "test",
        "short wait",
    )
    .await
    .unwrap();
    let silent = FakeBroker {
        silent_orders: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, silent, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "sent, no answer");
    });
    let outcome: String =
        sqlx::query_scalar("SELECT outcome FROM strategy3_fills WHERE kind = 'open'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(outcome, "unconfirmed");
    // The broker did fill it.
    silent
        .positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 13, 60_462.0));
    let answering = FakeBroker {
        silent_orders: false,
        ..silent.clone()
    };
    with_trader!(&pool, answering, Environment::Demo, |t, log| {
        let o = t.own.get(&5001).cloned().expect("adopted");
        assert!((o.tp1_distance.unwrap() - 1_651.21).abs() < 1e-9, "{o:?}");
        assert_eq!(o.close_volume, Some(5));
        assert!(o.partial);
        assert_eq!(o.session_date, Some(today().date));
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        t.reconcile(Utc::now()).await.unwrap();
        t.session_end(at(24 * 60)).await.unwrap();
        let closes: Vec<i64> = log
            .lock()
            .unwrap()
            .closes
            .iter()
            .map(|c| c.volume)
            .collect();
        assert_eq!(closes, vec![5, 8]);
    });
}

async fn end_orders(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT o.entry_key, f.outcome FROM strategy3_orders o
         JOIN strategy3_fills f ON f.entry_key = o.entry_key
         WHERE o.kind = 'session_end' ORDER BY o.id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Codex P1 on #15: a rejected session-end close is tried again, with a new
/// intent key, until it fills.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rejected_session_end_close_is_retried_with_a_new_key(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_closes: Arc::new(std::sync::atomic::AtomicUsize::new(1)),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let end = at(24 * 60);
        t.session_end(end).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1);
        t.session_end(end + Duration::seconds(5)).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "waits 10 s");
        t.session_end(end + Duration::seconds(11)).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 2);
    });
    let e = end_orders(&pool).await;
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e[0].0.ends_with(":1") && e[0].1 == "rejected", "{e:?}");
    assert!(e[1].0.ends_with(":2") && e[1].1 == "filled", "{e:?}");
}

/// Codex P1 on #15: after a restart, a session-end close that was rejected
/// is tried again at once, with the next key.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_restart_after_a_rejected_session_end_close_retries_it(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_closes: Arc::new(std::sync::atomic::AtomicUsize::new(1)),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        t.session_end(at(24 * 60)).await.unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1);
    });
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.session_end(at(24 * 60) + Duration::seconds(1))
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "retried at once");
    });
    let e = end_orders(&pool).await;
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e[1].0.ends_with(":2") && e[1].1 == "filled", "{e:?}");
}

/// Codex P1 on #15: the live rule is account-wide. A Strategy 1 position
/// opened since Strategy 3's last reconcile, or an unresolved Strategy 1
/// open, stops Strategy 3's live minimum entry (read fresh, not cached).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_live_minimum_waits_for_a_strategy_1_trade_opened_since_the_last_reconcile(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_460.0));
    with_trader!(&pool, f, Environment::Live, |t, log| {
        // Strategy 1 opens ETHUSD after this loop's reconcile.
        f.positions.lock().unwrap().push(ProtoOaPosition {
            position_id: 77,
            trade_data: ProtoOaTradeData {
                symbol_id: 5,
                volume: 1,
                trade_side: 1,
                label: Some("jiaoyi-s1".into()),
                ..Default::default()
            },
            position_status: 1,
            swap: 0,
            ..Default::default()
        });
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_skip" && r.contains("account-wide")),
        "{d:?}"
    );
    // An unresolved Strategy 1 open (no answer yet) counts too.
    f.positions.lock().unwrap().clear();
    store::trading::record_intent(
        &pool,
        &store::trading::IntentRow {
            account_id: DEMO,
            entry_key: format!("s1:{DEMO}:5:2026-10-24"),
            client_msg_id: "t9".into(),
            kind: "open".into(),
            symbol_id: Some(5),
            position_id: None,
            side: Some("buy".into()),
            volume: Some(1),
            relative_stop: Some(1_000),
            stop_price: None,
            take_profit: None,
            risk: Some(1.0),
            max_risk: 4.0,
            settings_version: None,
        },
    )
    .await
    .unwrap();
    with_trader!(&pool, f, Environment::Live, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "s3_skip" && r.contains("1 unresolved open(s)")),
        "{d:?}"
    );
}

async fn tp1_orders(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT o.entry_key, f.outcome FROM strategy3_orders o
         JOIN strategy3_fills f ON f.entry_key = o.entry_key
         WHERE o.kind = 'tp1' ORDER BY o.id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Codex P2 on #15: a TP1 close the broker rejects is tried again, with a
/// new intent key, while TP1 is still reached; once filled it is never sent
/// again.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rejected_tp1_close_is_retried_with_a_new_key(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_closes: Arc::new(std::sync::atomic::AtomicUsize::new(1)),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let tp1 = at(61);
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), tp1)
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "rejected");
        t.on_quote(
            BTC,
            Some(62_113.21),
            Some(62_118.0),
            tp1 + Duration::seconds(5),
        )
        .await
        .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "waits 10 s");
        t.on_quote(
            BTC,
            Some(62_113.21),
            Some(62_118.0),
            tp1 + Duration::seconds(11),
        )
        .await
        .unwrap();
        let closes: Vec<i64> = log
            .lock()
            .unwrap()
            .closes
            .iter()
            .map(|c| c.volume)
            .collect();
        assert_eq!(closes, vec![5, 5], "retried for the same 5 cents");
        t.on_quote(
            BTC,
            Some(62_200.0),
            Some(62_205.0),
            tp1 + Duration::seconds(30),
        )
        .await
        .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 2, "filled: never again");
    });
    let e = tp1_orders(&pool).await;
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e[0].0.ends_with(":1") && e[0].1 == "rejected", "{e:?}");
    assert!(e[1].0.ends_with(":2") && e[1].1 == "filled", "{e:?}");
}

/// Codex P2 on #15: after a restart a rejected TP1 close is tried again at
/// once, with the next key; a filled one is not.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_restart_after_a_rejected_tp1_close_retries_it(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_closes: Arc::new(std::sync::atomic::AtomicUsize::new(1)),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1);
    });
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(
            BTC,
            Some(62_113.21),
            Some(62_118.0),
            at(61) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "retried at once");
    });
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(62_200.0), Some(62_205.0), at(62))
            .await
            .unwrap();
        assert!(
            log.lock().unwrap().closes.is_empty(),
            "filled before the restart"
        );
    });
    let e = tp1_orders(&pool).await;
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e[1].0.ends_with(":2") && e[1].1 == "filled", "{e:?}");
}

/// Codex P2 on #15: the broker's own exits (the stop, the take-profit it
/// holds) are in `/trading/costs`, once per deal, with the fill against the
/// level that executed. A late opening deal and another program's exit are
/// not.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn broker_executed_exits_are_in_the_costs(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        let p = f.positions.lock().unwrap()[0].clone();
        let stop = p.stop_loss.unwrap();
        // The stop (60,012.00) fills 3.00 lower, for all 13 cents; the same
        // event again is the same deal.
        let sl = server_fill(DEMO, p.clone(), 901, stop - 3.0, 13, true, false);
        t.server_event(&sl).await;
        t.server_event(&sl).await;
        // Another long's take-profit, at 0.50 better than its level.
        let other = s3_position(5002, 8, 60_000.0);
        let tp = other.take_profit.unwrap();
        t.server_event(&server_fill(DEMO, other, 902, tp + 0.5, 8, true, false))
            .await;
        // A late opening deal, and another program's stop: not Strategy 3 exits.
        t.server_event(&server_fill(
            DEMO,
            p.clone(),
            903,
            60_462.0,
            13,
            false,
            false,
        ))
        .await;
        let mut s1 = p.clone();
        s1.trade_data.label = Some("jiaoyi-s1".into());
        t.server_event(&server_fill(DEMO, s1, 904, stop, 13, true, false))
            .await;
    });
    let rows: Vec<(String, String, f64, f64, i64)> = sqlx::query_as(
        "SELECT o.detail->>'reason', f.outcome, f.decision_price, f.slippage, f.volume
         FROM strategy3_fills f JOIN strategy3_orders o ON o.entry_key = f.entry_key
         WHERE f.kind = 'broker_exit' ORDER BY f.id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(
        (rows[0].0.as_str(), rows[0].1.as_str()),
        ("stop_loss", "filled")
    );
    assert!((rows[0].2 - 60_012.0).abs() < 1e-6, "{rows:?}");
    assert!((rows[0].3 - 3.0).abs() < 1e-6, "adverse 3.00: {rows:?}");
    assert_eq!(rows[0].4, 13);
    assert_eq!(rows[1].0, "take_profit");
    assert!((rows[1].2 - (60_000.0 + 2_101.21)).abs() < 1e-6, "{rows:?}");
    assert!((rows[1].3 + 0.5).abs() < 1e-6, "favourable 0.50: {rows:?}");
    let c = store::trading::costs(&pool).await.unwrap();
    let row = &c["rows"][0];
    assert_eq!(row["entries_filled"], 1);
    assert_eq!(row["exits_filled"], 2, "{row}");
    assert_eq!(row["broker_exits_filled"], 2, "{row}");
    assert!(
        (row["exit_slippage_median"].as_f64().unwrap() - 1.25).abs() < 1e-9,
        "{row}"
    );
    assert!(
        (row["spread_at_exit_median"].as_f64().unwrap() - 5.0).abs() < 1e-9,
        "{row}"
    );
}

// Codex · № 17,197 26-Oct (ordered: resume O-1; preserve dropped owned events).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn dropped_owned_execution_is_preserved_without_changing_exit_rules(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let mut p = f.positions.lock().unwrap()[0].clone();
        p.trade_data.label = None;
        let message = server_fill(
            DEMO,
            p.clone(),
            99901,
            p.stop_loss.unwrap(),
            13,
            true,
            false,
        );
        t.server_event(&message).await;
        t.server_event(&message).await;
        let raws: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT raw FROM trading_events WHERE account_id = $1 AND deal_id = 99901",
        )
        .bind(DEMO)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            raws,
            vec![message.payload.clone(), message.payload.clone()],
            "retain both received deliveries without modifying their payloads"
        );
        let mut labelled = p.clone();
        labelled.trade_data.label = Some(strategy3::LABEL.into());
        let mut correlated = server_fill(DEMO, labelled.clone(), 99903, 60_000.0, 13, false, false);
        correlated.client_msg_id = Some("diagnostic-correlation".into());
        t.server_event(&correlated).await;
        let raw: Vec<u8> =
            sqlx::query_scalar("SELECT raw FROM trading_events WHERE deal_id = 99903")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            raw, correlated.payload,
            "preserve events skipped for correlation too"
        );
        t.server_event(&server_fill(
            DEMO + 1,
            labelled,
            99904,
            60_000.0,
            13,
            true,
            false,
        ))
        .await;
        let wrong_account: i64 =
            sqlx::query_scalar("SELECT count(*) FROM trading_events WHERE deal_id = 99904")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(wrong_account, 0, "account identity remains authoritative");
        let exits: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind = 'broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(exits, 0, "diagnostics must not change exit attribution");
        let mut foreign = p.clone();
        foreign.position_id += 1000;
        t.server_event(&server_fill(
            DEMO, foreign, 99902, 60_000.0, 13, true, false,
        ))
        .await;
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM trading_events WHERE deal_id = 99902")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "do not collect unrelated positions");
    });
}

/// One open's fill row: (outcome, position, volume, fill price).
async fn open_fill(pool: &PgPool) -> (String, Option<i64>, Option<i64>, Option<f64>) {
    sqlx::query_as(
        "SELECT outcome, position_id, volume, fill_price FROM strategy3_fills WHERE kind = 'open'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Codex round 3 on #15: an open filled in two deals is held at their sum
/// and their volume-weighted price, read from the broker, not at the last
/// deal's 7 cents and price.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_open_filled_in_two_deals_is_held_at_their_aggregate(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 6, 60_462.0), (20, 7, 60_470.0)],
        ..fake((60_455.0, 60_462.0))
    };
    let vwap = (6.0 * 60_462.0 + 7.0 * 60_470.0) / 13.0;
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        assert_eq!(orders(&log)[0].volume, 13);
        let o = t.own.get(&1001).cloned().expect("held");
        assert_eq!(o.volume, 13, "{o:?}");
        assert!((o.entry.unwrap() - vwap).abs() < 1e-6, "{o:?}");
        assert_eq!((o.close_volume, o.partial), (Some(5), true), "{o:?}");
        assert_eq!(o.session_date, Some(today().date), "{o:?}");
    });
    let (outcome, pid, vol, price) = open_fill(&pool).await;
    assert_eq!(
        (outcome.as_str(), pid, vol),
        ("filled", Some(1001), Some(13))
    );
    assert!((price.unwrap() - vwap).abs() < 1e-6, "{price:?}");
}

/// Codex round 3 on #15: an open partly filled before its remainder is
/// cancelled is linked to its plan through the deal's position, so its TP1
/// and its session-end close are known, now and after a restart.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partly_filled_open_keeps_its_plan(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 6, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        let o = t.own.get(&1001).cloned().expect("held");
        assert_eq!(o.volume, 6, "{o:?}");
        assert_eq!(o.session_date, Some(today().date), "{o:?}");
        assert!(o.tp1_distance.is_some(), "{o:?}");
    });
    let (outcome, pid, vol, _) = open_fill(&pool).await;
    assert_eq!(
        (outcome.as_str(), pid, vol),
        ("partly_filled", Some(1001), Some(6))
    );
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        let o = t.own.get(&1001).cloned().expect("held after a restart");
        assert_eq!(o.session_date, Some(today().date), "{o:?}");
        assert!(o.tp1_distance.is_some(), "{o:?}");
    });
}

/// Codex round 3 on #15: an Unconfirmed open of 13 cents found at 6 by a
/// reconcile is adopted at the broker's 6, and the open stays unresolved, its
/// remainder reserved, until the broker holds all 13.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unconfirmed_open_found_partly_filled_keeps_its_remainder_reserved(pool: PgPool) {
    loaded(&pool).await;
    settings::change(
        &pool,
        "trading.order_wait_secs",
        Some(DEMO),
        Some(json!(1)),
        "test",
        "short wait",
    )
    .await
    .unwrap();
    let silent = FakeBroker {
        silent_orders: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, silent, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "sent, no answer");
    });
    silent
        .positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 6, 60_462.0));
    with_trader!(&pool, silent, Environment::Demo, |t, _log| {
        assert_eq!(t.own.get(&5001).map(|o| o.volume), Some(6), "adopted");
    });
    let kinds = || async {
        sqlx::query_scalar::<_, String>(
            "SELECT kind FROM trading_events WHERE kind LIKE 'RECONCILED%' ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    assert_eq!(kinds().await, vec!["RECONCILED_PARTIAL_FILL"]);
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert_eq!(u.len(), 1, "the remainder stays reserved: {u:?}");
    assert_eq!((u[0].filled, u[0].position_id), (6, Some(5001)), "{u:?}");
    // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design"):
    // the order list shows the adopted part, at the broker's position.
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(
        o[0].outcome.as_deref(),
        Some("RECONCILED_PARTIAL_FILL"),
        "{o:?}"
    );
    assert_eq!(
        (o[0].requested_volume, o[0].filled_volume),
        (Some(13), Some(6))
    );
    assert_eq!(o[0].position_id, Some(5001));
    // The remainder fills: the broker holds all 13.
    silent.positions.lock().unwrap()[0].trade_data.volume = 13;
    with_trader!(&pool, silent, Environment::Demo, |t, _log| {
        assert_eq!(t.own.get(&5001).map(|o| o.volume), Some(13));
    });
    assert_eq!(
        kinds().await,
        vec!["RECONCILED_PARTIAL_FILL", "RECONCILED_FILL"]
    );
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert!(u.is_empty(), "{u:?}");
}

/// Run the Strategy 3 loop on `f` for `secs` as the demo account, its
/// liveness kept in `live`, then stop it.
async fn run_s3(pool: &PgPool, f: &FakeBroker, secs: u64, live: &trading::LivenessMap) {
    let (conn, _log) = f.clone().spawn();
    let session = loader::authorise(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let list = allowlist();
    tokio::select! {
        r = strategy3::run_session(pool, &list, Environment::Demo, &session, live, &mut ends, &mut stop) => {
            r.unwrap();
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => trigger.fire(),
    }
}

/// As #16 for Strategy 1: Strategy 3's reconcile decision is recorded only
/// when it changes, so a quiet loop's reconcile runs keep its liveness
/// advancing, with the recorded decision kept once.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_quiet_strategy_3_loop_keeps_its_reconcile_runs_advancing(pool: PgPool) {
    loaded(&pool).await;
    settings::change(
        &pool,
        "trading.reconcile_secs",
        Some(DEMO),
        Some(json!(30)),
        "test",
        "a short interval",
    )
    .await
    .unwrap();
    let live = trading::LivenessMap::default();
    run_s3(&pool, &fake((60_455.0, 60_462.0)), 40, &live).await;
    let l = live.lock().unwrap().get(&DEMO).cloned().expect("a beat");
    let (start, ran) = (l.started_at.unwrap(), l.last_reconcile_run_at.unwrap());
    assert!(
        ran - start >= Duration::seconds(25),
        "a second reconcile ran: {l:?}"
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trading_decisions WHERE account_id = $1 AND kind = 's3_reconcile'",
    )
    .bind(DEMO)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "the unchanged reason is recorded once");
    let loops = live.lock().unwrap().clone();
    let h = strategy3::health(&pool, &loops).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    assert_eq!(h[0]["liveness"]["stalled_after_secs"], 60, "{h}");
    let recorded: DateTime<Utc> =
        serde_json::from_value(h[0]["last_reconcile_at"].clone()).unwrap();
    assert!(ran - recorded >= Duration::seconds(25), "{h}");
}

/// A Strategy 3 loop stopped for more than twice its reconcile interval
/// reads stalled; its last quote is shown; an account with no loop reads
/// not running.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_stopped_strategy_3_loop_reads_stalled_after_twice_its_reconcile_interval(pool: PgPool) {
    loaded(&pool).await;
    // At the session's open: inside the band, no entry.
    let f = FakeBroker {
        quote_sequence: vec![(
            0,
            BTC,
            (
                Some((OPEN * 100_000.0) as u64),
                Some(((OPEN + 1.0) * 100_000.0) as u64),
            ),
        )],
        ..fake((60_455.0, 60_462.0))
    };
    let live = trading::LivenessMap::default();
    run_s3(&pool, &f, 2, &live).await;
    let l = live.lock().unwrap().get(&DEMO).cloned().expect("a beat");
    assert!(l.last_quote_at.is_some(), "{l:?}");
    let loops = live.lock().unwrap().clone();
    let h = strategy3::health(&pool, &loops).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    assert!(h[0]["liveness"]["last_quote_at"].is_string(), "{h}");
    let aged = |age: i64| {
        let mut m = loops.clone();
        let e = m.get_mut(&DEMO).unwrap();
        let at = Utc::now() - Duration::seconds(age);
        e.started_at = Some(at - Duration::seconds(5));
        e.last_reconcile_run_at = Some(at);
        m
    };
    let h = strategy3::health(&pool, &aged(590)).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    let h = strategy3::health(&pool, &aged(610)).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "stalled", "{h}");
    let h = strategy3::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "not running", "{h}");
}

/// The TP1 closes the fake received.
fn closes(log: &Arc<Mutex<FakeLog>>) -> Vec<i64> {
    log.lock()
        .unwrap()
        .closes
        .iter()
        .map(|c| c.volume)
        .collect()
}

/// Enter at 45 minutes on `f`, then quote at TP1 (fill 60,462.00 +
/// 1,651.21): the TP1 closes sent.
async fn tp1_after(pool: &PgPool, f: FakeBroker) -> Vec<i64> {
    let out;
    with_trader!(pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        assert_eq!(orders(&log)[0].volume, 13);
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        out = closes(&log);
    });
    out
}

/// Codex round-4 P2 on #15: a partial fill too small to split (1 of 13
/// cents; the plan's close was 5) takes TP1 as a whole-position close, not
/// none.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unsplittable_partial_fill_closes_whole_at_tp1(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 1, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    assert_eq!(tp1_after(&pool, f).await, vec![1]);
}

/// Codex round-4 P2 on #15: a partial fill of 4 of 13 cents re-plans its TP1
/// close for 4: ceil(4 × (R + C) / (TP1 + R)) = ceil(4 × 751.208 / 2,101.21)
/// = 2, a runner of 2 (self-derived from the planner's own formula).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partial_fill_re_plans_its_tp1_close(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 4, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    assert_eq!(tp1_after(&pool, f).await, vec![2]);
}

/// A full fill keeps the planned close of 5 of 13 (a guard: main already
/// does this).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_full_fill_keeps_its_planned_tp1_close(pool: PgPool) {
    loaded(&pool).await;
    assert_eq!(tp1_after(&pool, fake((60_455.0, 60_462.0))).await, vec![5]);
}

// ------------------------------------------------------------- the instrument
// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")

/// Strategy 3's orders on the demo account, newest first.
async fn s3_orders(pool: &PgPool) -> Vec<store::trading::OrderView> {
    store::trading::orders(
        pool,
        &store::trading::OrdersQuery {
            account_id: Some(DEMO),
            strategy: Some("s3".into()),
            since: None,
            limit: 100,
        },
    )
    .await
    .unwrap()
}

fn near(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-6)
}

/// An open filled in two deals (6 at 60,462.00, 7 at 60,470.00) is listed
/// at 13 cents and their VWAP, against the ask 60,460.00 it was decided at
/// (expected values from the fixture: self-derived), with its decision;
/// `/trading/costs` reports it under s3.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_3_open_in_two_deals_is_listed_at_their_vwap(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 6, 60_462.0), (20, 7, 60_470.0)],
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
    });
    let vwap = (6.0 * 60_462.0 + 7.0 * 60_470.0) / 13.0;
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    let r = &o[0];
    assert_eq!(
        (r.strategy.as_str(), r.label.as_str(), r.kind.as_str()),
        ("s3", "jiaoyi-s3", "open")
    );
    assert_eq!(r.order.as_deref(), Some("open"));
    assert_eq!(r.symbol.as_deref(), Some("BTCUSD"));
    assert_eq!((r.requested_volume, r.filled_volume), (Some(13), Some(13)));
    assert_eq!(r.outcome.as_deref(), Some("ORDER_FILLED"));
    assert!(near(r.fill_price, vwap), "{r:?}");
    assert_eq!(r.position_id, Some(1001));
    assert!(r.latency_ms.is_some(), "{r:?}");
    assert!(near(r.slippage, vwap - 60_460.0), "{r:?}");
    let d = r.decision.as_ref().expect("its decision");
    assert_eq!(d.kind, "s3_sent");
    assert!(d.reason.contains("buy 13 cents"), "{d:?}");
    let c = store::trading::costs(&pool).await.unwrap();
    let row = &c["rows"][0];
    assert_eq!(
        (row["strategy"].clone(), row["label"].clone()),
        (json!("s3"), json!("jiaoyi-s3"))
    );
    assert_eq!(row["entries_filled"], 1);
}

/// 6 of 13 cents, the remainder cancelled: listed cancelled, 6 filled into
/// its position.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_3_partial_fill_then_cancel_is_listed_with_its_part(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 6, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
    });
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("ORDER_CANCELLED"), "{o:?}");
    assert_eq!(
        (o[0].requested_volume, o[0].filled_volume),
        (Some(13), Some(6))
    );
    assert!(near(o[0].fill_price, 60_462.0), "{o:?}");
    assert_eq!(o[0].position_id, Some(1001));
}

/// An Unconfirmed open found filled at a reconcile after a restart: listed
/// as RECONCILED_FILL at the broker's 13 cents, price and position.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unconfirmed_strategy_3_open_adopted_at_reconcile_is_listed(pool: PgPool) {
    loaded(&pool).await;
    settings::change(
        &pool,
        "trading.order_wait_secs",
        Some(DEMO),
        Some(json!(1)),
        "test",
        "short wait",
    )
    .await
    .unwrap();
    let silent = FakeBroker {
        silent_orders: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, silent, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
    });
    silent
        .positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 13, 60_462.0));
    with_trader!(&pool, silent, Environment::Demo, |_t, _log| {});
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("RECONCILED_FILL"), "{o:?}");
    assert_eq!(o[0].filled_volume, Some(13));
    assert!(near(o[0].fill_price, 60_462.0), "{o:?}");
    assert_eq!(o[0].position_id, Some(5001));
    with_trader!(&pool, silent, Environment::Demo, |t, _log| {
        let p = silent.positions.lock().unwrap()[0].clone();
        silent.positions.lock().unwrap().clear();
        t.reconcile(at(50)).await.unwrap();
        assert!(t.own.is_empty());
        let mut close =
            jiaoyi_ctrader::testkit::server_fill(DEMO, p, 99995, 60_000.0, 13, true, false);
        close.client_msg_id = Some("observed-broker-correlation".into());
        t.server_event(&close).await;
        let exits: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind = 'broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            exits, 1,
            "an adopted opening fill is durable ownership evidence too"
        );
    });
}

fn second_prepared_crypto(t: &mut strategy3::Trader<'_, '_>) -> i64 {
    // Two independent USD-quoted crypto symbols use the same deterministic
    // price/session fixture; their broker identities and retry state differ.
    let id = BTC + 1;
    let mut other = t.syms[&BTC].clone();
    other.id = id;
    other.name = "ETHUSD".into();
    other.last_try = None;
    t.syms.insert(id, other);
    id
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn account_funding_rejection_backs_off_other_symbols_then_retries(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_orders: Some("NOT_ENOUGH_MONEY".into()),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let other = second_prepared_crypto(&mut t);
        t.on_quote(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(45) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "an account funding rejection suppresses other entry attempts"
        );
        t.on_quote(
            other,
            Some(60_500.0),
            Some(60_505.0),
            at(46) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            2,
            "the existing retry interval expires; no owner pause is written"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn symbol_rejection_does_not_back_off_other_symbols(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_orders: Some("MARKET_CLOSED".into()),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let other = second_prepared_crypto(&mut t);
        t.on_quote(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(45) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            2,
            "a symbol rejection does not declare the account unfunded"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn funding_backoff_keeps_the_open_positions_tp1(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
    });
    f.reject_orders = Some("NOT_ENOUGH_MONEY".into());
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.prepare(BTC, today(), at(45)).await;
        let other = second_prepared_crypto(&mut t);
        t.on_quote(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(45) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "one rejected entry on the new connection"
        );
        t.on_quote(
            BTC,
            Some(62_113.21),
            Some(62_118.0),
            at(45) + Duration::seconds(2),
        )
        .await
        .unwrap();
        assert_eq!(
            log.lock().unwrap().closes.len(),
            1,
            "TP1 proceeds during funding backoff"
        );
    });
}

/// A broker rejection is listed with nothing filled.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rejected_strategy_3_open_is_listed_with_nothing_filled(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_orders: Some("MARKET_CLOSED".into()),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
    });
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("ORDER_REJECTED"), "{o:?}");
    assert_eq!(
        (o[0].filled_volume, o[0].fill_price, o[0].position_id),
        (Some(0), None, None)
    );
}

/// The broker's own stop of a Strategy 3 position is listed as a broker
/// exit, with why it closed and its fill against the stop.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_3_stop_the_broker_executes_is_listed(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let stop = p.stop_loss.unwrap();
        t.server_event(&server_fill(DEMO, p, 901, stop - 3.0, 13, true, false))
            .await;
    });
    let o = s3_orders(&pool).await;
    assert_eq!(o.len(), 2, "{o:?}");
    let exit = &o
        .iter()
        .find(|r| r.kind == "broker_exit")
        .expect("the exit");
    assert_eq!(exit.strategy, "s3");
    assert_eq!(exit.decision.as_ref().unwrap().reason, "stop_loss");
    assert_eq!(
        (exit.filled_volume, exit.position_id),
        (Some(13), Some(1001))
    );
    assert!(near(exit.slippage, 3.0), "{exit:?}");
}

// ---------------------------------------------------------------- limits and the owner's pause
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)

/// The owner's word on Strategy 3's pause on this account (1 paused, 0 not).
async fn owner_sets_pause(pool: &PgPool, value: i64) {
    settings::change(
        pool,
        "trading.s3_paused",
        Some(DEMO),
        Some(json!(value)),
        "owner",
        "owner, in writing (test)",
    )
    .await
    .unwrap();
}

/// `n` closed Strategy 3 trades of −1R on this account (the gate's $6.00,
/// the broker's net −$6.00), two days ago.
async fn closed_losses(pool: &PgPool, code: &str, n: i32) {
    sqlx::query(
        "INSERT INTO trading_intents (account_id, entry_key, client_msg_id, kind, symbol_id,
            side, volume, risk, max_risk)
         SELECT $1, $2 || ':' || n, 'c-' || $2 || n, 'open', 99, 'buy', 10, 6.0, 100
         FROM generate_series(1, $3) n",
    )
    .bind(DEMO)
    .bind(code)
    .bind(n)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO trading_events (account_id, entry_key, client_msg_id, payload_type, kind,
            position_id, raw, received_at)
         SELECT $1, $2 || ':' || n, 'c-' || $2 || n, 2126, 'ORDER_FILLED', 900000 + n, '\\x',
                now() - interval '2 days'
         FROM generate_series(1, $3) n",
    )
    .bind(DEMO)
    .bind(code)
    .bind(n)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO deals (account_id, deal_id, order_id, position_id, broker_symbol_id, volume,
            filled_volume, trade_side, deal_status, created_at, executed_at, money_digits,
            is_closing, close_gross_profit, close_swap, close_commission, close_money_digits,
            raw, fetched_at)
         SELECT $1, 9000000 + 2 * n + c, 9000000 + 2 * n + c, 900000 + n, 99, 10, 10,
                'BUY', 'FILLED', t, t, 2, c = 1,
                CASE WHEN c = 1 THEN -560 END, CASE WHEN c = 1 THEN 0 END,
                CASE WHEN c = 1 THEN -40 END, CASE WHEN c = 1 THEN 2 END, '\\x', now()
         FROM generate_series(1, $2) n, generate_series(0, 1) c,
              LATERAL (SELECT now() - interval '2 days' + n * interval '1 minute' + c * interval '10 seconds' AS t) x",
    )
    .bind(DEMO)
    .bind(n)
    .execute(pool)
    .await
    .unwrap();
}

/// Every write of a pause key not made by a migration: (key, by).
async fn pause_writes(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT key, changed_by FROM settings_changes
         WHERE key IN ('trading.s1_paused', 'trading.s3_paused') AND changed_by <> 'migration'
         ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// "only human can pause trade" (owner, 25-10): paused by the owner, a band
/// break sends nothing and is recorded once; lifted, the next break enters;
/// paused again, the open position still takes its TP1 and its session-end
/// close.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_owners_pause_stops_new_entries_and_the_exits_still_run(pool: PgPool) {
    loaded(&pool).await;
    owner_sets_pause(&pool, 1).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        // σ(43) = 0.0071667, σ(44) = 0.0073333: the band tops at 60,430.00 and
        // 60,440.00, so an ask of 60,460.00 breaks it both times.
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(43))
            .await
            .unwrap();
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(44))
            .await
            .unwrap();
        assert!(orders(&log).is_empty(), "paused: nothing sent");
        let d = decisions(&pool).await;
        let paused: Vec<&(String, String)> = d.iter().filter(|(k, _)| k == "s3_paused").collect();
        assert_eq!(paused.len(), 1, "recorded once: {d:?}");
        assert!(paused[0].1.contains("paused by the owner"), "{d:?}");
        // Lifted by the owner: the next break (a minute on) enters.
        owner_sets_pause(&pool, 0).await;
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "{:?}", decisions(&pool).await);
        assert_eq!(orders(&log)[0].volume, 13);
        // Paused again: the position keeps its exits.
        owner_sets_pause(&pool, 1).await;
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        assert_eq!(closes(&log), vec![5], "TP1 while paused");
        t.reconcile(Utc::now()).await.unwrap();
        t.session_end(at(24 * 60)).await.unwrap();
        assert_eq!(closes(&log), vec![5, 8], "the session end while paused");
        assert_eq!(orders(&log).len(), 1, "no new entry");
    });
    // Only the owner wrote the pause.
    let w = pause_writes(&pool).await;
    assert_eq!(w.len(), 3);
    assert!(
        w.iter()
            .all(|(k, by)| k == "trading.s3_paused" && by == "owner"),
        "{w:?}"
    );
    let h = strategy3::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["trades"], "paused by the owner: no new entry");
    assert_eq!(h[0]["owner_pause"]["paused"], true);
    assert_eq!(h[0]["owner_pause"]["changed_by"], "owner");
}

/// A limit reached is an alert: recorded, on /health, and Strategy 3 keeps
/// trading. Nothing pauses it.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_limit_reached_is_flagged_and_strategy_3_keeps_trading(pool: PgPool) {
    loaded(&pool).await;
    closed_losses(&pool, "s3:old", 10).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        // The reconcile above checked the limits.
        let d = decisions(&pool).await;
        let reached: Vec<&(String, String)> =
            d.iter().filter(|(k, _)| k == "s3_limit_reached").collect();
        assert_eq!(reached.len(), 1, "{d:?}");
        assert!(reached[0].1.contains("owner's decision needed"));
        t.reconcile(Utc::now()).await.unwrap();
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "still trading: {:?}",
            decisions(&pool).await
        );
    });
    let d = decisions(&pool).await;
    assert_eq!(
        d.iter().filter(|(k, _)| k == "s3_limit_reached").count(),
        1,
        "once"
    );
    assert!(pause_writes(&pool).await.is_empty(), "nothing paused");
    let h = strategy3::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["trades"], "yes");
    assert_eq!(h[0]["performance"]["owner_decision_needed"], true);
    assert_eq!(
        h[0]["performance"]["limit_reached"][0]["limit"],
        "drawdown_r"
    );
    assert_eq!(h[0]["performance"]["closed_trades"], 10);
}

/// How near each symbol is to its trigger: the band now, the quote, and the
/// nearer side's distance to its edge in half-widths (negative once
/// through it); none, with the reason, outside the entry window.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_distance_to_the_trigger_is_shown_inside_outside_and_out_of_the_window(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        let read = |t: &strategy3::Trader| t.live.lock().unwrap()[&DEMO].triggers["BTCUSD"].clone();
        // At 45 minutes the band is 59,550.00 – 60,450.00 (half-width 450.00).
        t.on_quote(BTC, Some(60_435.0), Some(60_440.0), at(45))
            .await
            .unwrap();
        let x = read(&t);
        assert!(
            (x["upper"].as_f64().unwrap() - 60_450.0).abs() < 1e-6,
            "{x}"
        );
        assert!(
            (x["lower"].as_f64().unwrap() - 59_550.0).abs() < 1e-6,
            "{x}"
        );
        assert!((x["sigma"].as_f64().unwrap() - 0.0075).abs() < 1e-12, "{x}");
        assert_eq!(x["nearer_edge"], "upper");
        // (60,450 − 60,440) / 450 = 0.0222: inside, no order.
        let d = x["distance_half_widths"].as_f64().unwrap();
        assert!((d - 10.0 / 450.0).abs() < 1e-9, "{x}");
        assert!(orders(&log).is_empty());
        // Through the upper edge: negative, and the order is sent.
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let x = read(&t);
        let d = x["distance_half_widths"].as_f64().unwrap();
        assert!((d + 10.0 / 450.0).abs() < 1e-9, "{x}");
        assert_eq!(orders(&log).len(), 1);
        // Near the lower edge: the bid's distance, the lower edge.
        t.on_quote(BTC, Some(59_640.0), Some(59_645.0), at(45))
            .await
            .unwrap();
        let x = read(&t);
        assert_eq!(x["nearer_edge"], "lower");
        assert!(
            (x["distance_half_widths"].as_f64().unwrap() - 0.2).abs() < 1e-9,
            "{x}"
        );
        // Out of the window: none, with the reason.
        t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(29))
            .await
            .unwrap();
        let x = read(&t);
        assert_eq!(x["distance_half_widths"], serde_json::Value::Null);
        assert!(
            x["reason"].as_str().unwrap().contains("first 30 minutes"),
            "{x}"
        );
        t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(24 * 60 - 30))
            .await
            .unwrap();
        let x = read(&t);
        assert_eq!(x["distance_half_widths"], serde_json::Value::Null);
        assert!(
            x["reason"].as_str().unwrap().contains("last 30 minutes"),
            "{x}"
        );
        // /health carries it from the loops' map.
        let live = t.live.lock().unwrap().clone();
        let h = strategy3::health(&pool, &live).await.unwrap();
        assert_eq!(h[0]["trigger_distance"]["BTCUSD"], x);
    });
}

// Codex · № 17,217 26-Oct: observed 02:17Z broker closes carry correlation.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn correlated_broker_close_is_recorded_once_after_reconcile(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let client: String = sqlx::query_scalar(
            "SELECT client_msg_id FROM trading_intents WHERE account_id = $1 AND kind = 'open'",
        )
        .bind(DEMO)
        .fetch_one(&pool)
        .await
        .unwrap();
        let mut message = server_fill(
            DEMO,
            p.clone(),
            99991,
            p.stop_loss.unwrap() - 3.0,
            13,
            true,
            false,
        );
        message.client_msg_id = Some(client);
        // The broker position is gone before the event reaches the trading loop.
        f.positions.lock().unwrap().clear();
        t.reconcile(at(50)).await.unwrap();
        assert!(t.own.is_empty());
        let requests_before = log.lock().unwrap().closes.len();
        t.server_event(&message).await;
        t.server_event(&message).await;
        let foreign_id = p.position_id + 1000;
        let foreign_key = "foreign-account-open";
        sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,decided_at)
            VALUES ($1,$2,'jiaoyi-s3','test','open','BTCUSD',22,now())")
            .bind(DEMO+1).bind(foreign_key).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,volume,sent_at)
            VALUES ($1,$2,'jiaoyi-s3','test','open','BTCUSD',$3,'filled',13,now())")
            .bind(DEMO+1).bind(foreign_key).bind(foreign_id).execute(&pool).await.unwrap();
        let mut unrelated = p.clone();
        unrelated.position_id = foreign_id;
        let mut foreign_position = server_fill(DEMO, unrelated, 99993, 60_000.0, 13, true, false);
        foreign_position.client_msg_id = message.client_msg_id.clone();
        t.server_event(&foreign_position).await;
        let mut foreign_account =
            server_fill(DEMO + 1, p.clone(), 99994, 60_000.0, 13, true, false);
        foreign_account.client_msg_id = message.client_msg_id.clone();
        t.server_event(&foreign_account).await;
        let rows: Vec<(i64, Option<f64>)> = sqlx::query_as(
            "SELECT position_id, slippage FROM strategy3_fills WHERE kind = 'broker_exit'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            rows.len(),
            1,
            "one broker exit per deal despite repeated correlation"
        );
        assert_eq!(rows[0].0, p.position_id);
        assert_eq!(rows[0].1, Some(3.0));
        assert_eq!(
            log.lock().unwrap().closes.len(),
            requests_before,
            "recording sends no close"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn correlated_local_close_is_not_double_counted_as_broker_exit(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        let wire: (Option<String>, i32, Vec<u8>, DateTime<Utc>) = sqlx::query_as(
            "SELECT e.client_msg_id,e.payload_type,e.raw,e.received_at FROM trading_events e
             JOIN trading_intents i ON i.entry_key=e.entry_key
             WHERE i.account_id=$1 AND i.kind='close' AND e.deal_id IS NOT NULL LIMIT 1",
        )
        .bind(DEMO)
        .fetch_one(&pool)
        .await
        .unwrap();
        let message = jiaoyi_ctrader::ReceivedExecution {
            client_msg_id: wire.0,
            payload_type: wire.1 as u32,
            payload: wire.2,
            received_at: wire.3,
        };
        t.server_event(&message).await;
        t.server_event(&message).await;
        let counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT kind,count(*) FROM strategy3_fills WHERE kind <> 'open' GROUP BY kind",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            counts,
            vec![("tp1".into(), 1)],
            "our existing close path stays authoritative"
        );
        // Connection-local tN ids can be reused after reconnect. A different
        // broker deal must not disappear just because its client id collides.
        let mut collision = server_fill(DEMO, p, 99996, 60_000.0, 8, true, false);
        collision.client_msg_id = message.client_msg_id.clone();
        t.server_event(&collision).await;
        let exits: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            exits, 1,
            "deal identity distinguishes a new exit from an old local close"
        );
    });
}

// Codex · № 17,225 26-Oct: absence plus confirmed broker history, not absence alone.
#[sqlx::test(migrator = "store::MIGRATOR")]
#[allow(clippy::type_complexity)]
async fn reconcile_recovers_a_stream_missed_close_from_history_once(pool: PgPool) {
    use jiaoyi_ctrader::messages::{ProtoOaClosePositionDetail, ProtoOaDealStatus};
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let close_price = p.stop_loss.unwrap() - 3.0;
        let executed = Utc::now();
        let deal = ProtoOaDeal {
            deal_id: 99997,
            order_id: 99998,
            position_id: p.position_id,
            symbol_id: BTC,
            volume: 13,
            filled_volume: 13,
            trade_side: 2,
            deal_status: ProtoOaDealStatus::Filled as i32,
            execution_price: Some(close_price),
            execution_timestamp: executed.timestamp_millis(),
            close_position_detail: Some(ProtoOaClosePositionDetail {
                entry_price: p.price.unwrap(),
                closed_volume: Some(13),
                ..Default::default()
            }),
            ..Default::default()
        };
        let before = log.lock().unwrap().closes.len();
        f.positions.lock().unwrap().clear();
        t.reconcile(at(50)).await.unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "broker absence alone is not a fill");
        let mut incomplete = deal;
        incomplete
            .close_position_detail
            .as_mut()
            .unwrap()
            .closed_volume = None;
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&incomplete)], Utc::now())
            .await
            .unwrap();
        t.reconcile(at(52)).await.unwrap();
        let unknown: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(unknown, 0, "do not invent missing closed quantity");
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&deal)], Utc::now())
            .await
            .unwrap();
        // History can arrive one sync later than the position's disappearance.
        t.reconcile(at(55)).await.unwrap();
        t.reconcile(at(56)).await.unwrap();
        let rows: Vec<(Option<f64>,Option<f64>,Option<f64>,Option<i64>,Option<i64>)> = sqlx::query_as(
            "SELECT fill_price,slippage,spread,latency_ms,volume FROM strategy3_fills WHERE kind='broker_exit'")
            .fetch_all(&pool).await.unwrap();
        assert_eq!(
            rows,
            vec![(Some(close_price), None, None, None, Some(13))],
            "actual fill, one record, no invented historical quote/latency"
        );
        assert_eq!(
            log.lock().unwrap().closes.len(),
            before,
            "recovery sends nothing"
        );
    });
    with_trader!(&pool, f, Environment::Demo, |_t, _log| {});
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1, "restart does not duplicate a recovered deal");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn history_recovery_preserves_held_positions_and_excludes_local_fills(pool: PgPool) {
    use jiaoyi_ctrader::messages::{ProtoOaClosePositionDetail, ProtoOaDealStatus};
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        // A TP1 filled by the existing gate already has its exact broker deal.
        t.on_quote(BTC, Some(62_113.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        let wire: (Option<String>, i32, Vec<u8>, DateTime<Utc>) = sqlx::query_as(
            "SELECT e.client_msg_id,e.payload_type,e.raw,e.received_at FROM trading_events e
             JOIN trading_intents i ON i.entry_key=e.entry_key
             WHERE i.account_id=$1 AND i.kind='close' AND e.deal_id IS NOT NULL LIMIT 1",
        )
        .bind(DEMO)
        .fetch_one(&pool)
        .await
        .unwrap();
        let message = jiaoyi_ctrader::ReceivedExecution {
            client_msg_id: wire.0,
            payload_type: wire.1 as u32,
            payload: wire.2,
            received_at: wire.3,
        };
        let close = jiaoyi_ctrader::trading::closing_deal(DEMO, &message).unwrap();
        let ids = (close.deal_id, close.order_id);
        let mut own_close = ProtoOaDeal {
            deal_id: ids.0,
            order_id: ids.1,
            position_id: p.position_id,
            symbol_id: BTC,
            volume: 5,
            filled_volume: 5,
            trade_side: 2,
            deal_status: ProtoOaDealStatus::Filled as i32,
            execution_price: Some(62_113.21),
            execution_timestamp: Utc::now().timestamp_millis(),
            close_position_detail: Some(ProtoOaClosePositionDetail {
                entry_price: p.price.unwrap(),
                closed_volume: Some(5),
                ..Default::default()
            }),
            ..Default::default()
        };
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&own_close)], Utc::now())
            .await
            .unwrap();
        // The remaining 8 cents close at the broker. A stale/fresh position
        // snapshot that still contains the position must defer recovery.
        own_close.deal_id = 99981;
        own_close.order_id = 99982;
        own_close.volume = 8;
        own_close.filled_volume = 8;
        own_close
            .close_position_detail
            .as_mut()
            .unwrap()
            .closed_volume = Some(8);
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&own_close)], Utc::now())
            .await
            .unwrap();
        t.reconcile(at(65)).await.unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            count, 0,
            "history alone does not override a present broker position"
        );
        // Foreign account, unowned position and rejected deal cannot qualify.
        let mut wrong = own_close;
        wrong.deal_id = 99983;
        store::history::upsert_deals(&pool, DEMO + 1, &[history::deal_row(&wrong)], Utc::now())
            .await
            .unwrap();
        wrong.deal_id = 99984;
        wrong.position_id += 1000;
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&wrong)], Utc::now())
            .await
            .unwrap();
        wrong.position_id = p.position_id;
        wrong.deal_id = 99985;
        wrong.deal_status = ProtoOaDealStatus::Rejected as i32;
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&wrong)], Utc::now())
            .await
            .unwrap();
        wrong.deal_id = 99986;
        wrong.deal_status = ProtoOaDealStatus::Filled as i32;
        wrong.symbol_id = BTC + 1000;
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&wrong)], Utc::now())
            .await
            .unwrap();
        f.positions.lock().unwrap().clear();
        t.reconcile(at(70)).await.unwrap();
        let volumes: Vec<(String, Option<i64>)> = sqlx::query_as(
            "SELECT kind,volume FROM strategy3_fills WHERE kind<>'open' ORDER BY kind",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            volumes,
            vec![("broker_exit".into(), Some(8)), ("tp1".into(), Some(5))],
            "account for each real closing part once"
        );
    });
}

#[derive(Clone, Copy)]
enum LocalCloseCheckpoint {
    Pending,
    ReceiptOnly,
    IncompleteCoverage,
}

async fn challenge_local_close_checkpoint(pool: &PgPool, case: LocalCloseCheckpoint) {
    loaded(pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(pool, f, Environment::Demo, |t, _log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(45))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let key = "s3-local-checkpoint";
        let partial = matches!(case, LocalCloseCheckpoint::IncompleteCoverage);
        let requested = if partial { 5 } else { 13 };
        let kind = if partial { "tp1" } else { "session_end" };
        store::trading::record_intent(
            pool,
            &store::trading::IntentRow {
                account_id: DEMO,
                entry_key: key.into(),
                client_msg_id: "local-checkpoint".into(),
                kind: "close".into(),
                symbol_id: Some(BTC),
                position_id: Some(p.position_id),
                side: None,
                volume: Some(requested),
                relative_stop: None,
                stop_price: None,
                take_profit: None,
                risk: None,
                max_risk: 0.0,
                settings_version: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,decided_at)
            VALUES ($1,$2,'jiaoyi-s3','checkpoint',$3,'BTCUSD',$4,$5,now())")
            .bind(DEMO).bind(key).bind(kind).bind(BTC).bind(p.position_id).execute(pool).await.unwrap();
        let local_deal = 99960;
        let make_deal = |id, volume| ProtoOaDeal {
            deal_id: id,
            order_id: id + 1,
            position_id: p.position_id,
            symbol_id: BTC,
            volume,
            filled_volume: volume,
            trade_side: 2,
            deal_status: 2,
            execution_price: Some(60_009.0),
            execution_timestamp: Utc::now().timestamp_millis(),
            close_position_detail: Some(jiaoyi_ctrader::messages::ProtoOaClosePositionDetail {
                entry_price: p.price.unwrap(),
                closed_volume: Some(volume),
                ..Default::default()
            }),
            ..Default::default()
        };
        // This is the durable state the gate can leave before its caller
        // writes the local fill summary; no timing/sleep oracle is used.
        if !matches!(case, LocalCloseCheckpoint::Pending) {
            let volume = if partial { 3 } else { 13 };
            let mut wire = jiaoyi_ctrader::testkit::server_fill(
                DEMO,
                p.clone(),
                local_deal,
                62_113.21,
                volume,
                true,
                false,
            );
            wire.client_msg_id = Some("local-checkpoint".into());
            let mut event = jiaoyi_ctrader::trading::decode_execution(DEMO, &wire).unwrap();
            event.entry_key = Some(key.into());
            if partial {
                event.kind = "ORDER_PARTIAL_FILL".into();
                store::trading::record_event(pool, &trading::event_row(&event))
                    .await
                    .unwrap();
                event.kind = "ORDER_FILLED".into();
            }
            store::trading::record_event(pool, &trading::event_row(&event))
                .await
                .unwrap();
        }
        if partial {
            store::strategy3::record_fill(
                pool,
                &store::strategy3::FillRow {
                    account_id: DEMO,
                    entry_key: key.into(),
                    label: "jiaoyi-s3".into(),
                    prereg_hash: "checkpoint".into(),
                    kind: kind.into(),
                    symbol: "BTCUSD".into(),
                    position_id: Some(p.position_id),
                    side: Some("buy".into()),
                    outcome: "filled".into(),
                    sent_at: Utc::now(),
                    answered_at: Some(Utc::now()),
                    latency_ms: None,
                    decision_price: None,
                    fill_price: Some(62_113.21),
                    slippage: None,
                    spread: None,
                    volume: Some(5),
                    detail: json!({}),
                },
            )
            .await
            .unwrap();
        }
        let candidate_id = if partial { local_deal + 1 } else { local_deal };
        let candidate_volume = if partial { 2 } else { 13 };
        let mut deals = vec![history::deal_row(&make_deal(
            candidate_id,
            candidate_volume,
        ))];
        if partial {
            deals.push(history::deal_row(&make_deal(local_deal + 2, 8)));
        }
        store::history::upsert_deals(pool, DEMO, &deals, Utc::now())
            .await
            .unwrap();
        f.positions.lock().unwrap().clear();
        t.reconcile(at(70)).await.unwrap();
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(
            count, 0,
            "do not race or overlap pending/incompletely covered local bookkeeping"
        );
        let mut wire = jiaoyi_ctrader::testkit::server_fill(
            DEMO,
            p.clone(),
            candidate_id,
            60_009.0,
            candidate_volume,
            true,
            false,
        );
        wire.client_msg_id = Some("broker-correlation".into());
        t.server_event(&wire).await;
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(
            count, 0,
            "stream path must preserve the same unresolved local state"
        );
        if partial {
            let mut receipt = jiaoyi_ctrader::testkit::server_fill(
                DEMO,
                p.clone(),
                candidate_id,
                62_113.21,
                2,
                true,
                false,
            );
            receipt.client_msg_id = Some("local-checkpoint".into());
            let mut event = jiaoyi_ctrader::trading::decode_execution(DEMO, &receipt).unwrap();
            event.entry_key = Some(key.into());
            store::trading::record_event(pool, &trading::event_row(&event))
                .await
                .unwrap();
        } else {
            if matches!(case, LocalCloseCheckpoint::Pending) {
                store::trading::record_event(
                    pool,
                    &store::trading::EventRow {
                        account_id: DEMO,
                        entry_key: Some(key.into()),
                        client_msg_id: Some("local-checkpoint".into()),
                        payload_type: 2126,
                        kind: "ORDER_REJECTED".into(),
                        position_id: Some(p.position_id),
                        order_id: None,
                        deal_id: None,
                        symbol_id: Some(BTC),
                        side: None,
                        price: None,
                        volume: None,
                        stop_loss: None,
                        label: Some("jiaoyi-s3".into()),
                        error_code: None,
                        description: None,
                        is_server_event: None,
                        raw: vec![],
                        received_at: Utc::now(),
                    },
                )
                .await
                .unwrap();
            }
            store::strategy3::record_fill(
                pool,
                &store::strategy3::FillRow {
                    account_id: DEMO,
                    entry_key: key.into(),
                    label: "jiaoyi-s3".into(),
                    prereg_hash: "checkpoint".into(),
                    kind: kind.into(),
                    symbol: "BTCUSD".into(),
                    position_id: Some(p.position_id),
                    side: Some("buy".into()),
                    outcome: if matches!(case, LocalCloseCheckpoint::Pending) {
                        "rejected"
                    } else {
                        "filled"
                    }
                    .into(),
                    sent_at: Utc::now(),
                    answered_at: Some(Utc::now()),
                    latency_ms: None,
                    decision_price: None,
                    fill_price: None,
                    slippage: None,
                    spread: None,
                    volume: Some(requested),
                    detail: json!({}),
                },
            )
            .await
            .unwrap();
        }
        t.reconcile(at(75)).await.unwrap();
        let volumes: Vec<Option<i64>> = sqlx::query_scalar(
            "SELECT volume FROM strategy3_fills WHERE kind='broker_exit' ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        let expected = match case {
            LocalCloseCheckpoint::Pending => vec![Some(13)],
            LocalCloseCheckpoint::ReceiptOnly => vec![],
            LocalCloseCheckpoint::IncompleteCoverage => vec![Some(8)],
        };
        assert_eq!(
            volumes, expected,
            "resume only distinct unaccounted broker closing volume after settlement"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn history_defers_intent_before_local_receipt(pool: PgPool) {
    challenge_local_close_checkpoint(&pool, LocalCloseCheckpoint::Pending).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn history_defers_receipt_before_local_summary(pool: PgPool) {
    challenge_local_close_checkpoint(&pool, LocalCloseCheckpoint::ReceiptOnly).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn history_defers_incompletely_persisted_local_volume(pool: PgPool) {
    challenge_local_close_checkpoint(&pool, LocalCloseCheckpoint::IncompleteCoverage).await;
}
