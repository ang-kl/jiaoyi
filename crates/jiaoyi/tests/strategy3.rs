//! Production macro pullback against PostgreSQL and a wire-scripted broker:
//! causal preparation, durable arming, fresh midpoint retests, final dispatch
//! guards, full broker target and entry-price breakeven. Historical opening
//! fixtures separately retain legacy TP1/trailing, session-close, accounting
//! and ownership regressions. They cannot enable retired entry execution.
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

/// Today's first production BTCUSD four-hour window.
fn today() -> Session {
    jiaoyi_core::strategy3::production_sessions::windows_on(
        jiaoyi_core::strategy3::production_sessions::Market::Crypto,
        Utc::now().date_naive(),
    )[0]
}

/// Recorded pre-18392 positions retain their original full-day exit calendar.
fn legacy_today() -> Session {
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
    settings::change(
        pool,
        "trading.margin_level_floor_pct",
        None,
        Some(json!(200)),
        "test",
        "synthetic margin policy",
    )
    .await
    .unwrap();

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
    // Causal native-session opening midpoint for the new production revision.
    // The earlier M30 fixture is a bid bar and is not an opening-mid substitute.
    store::ticks::insert(
        pool,
        &[store::ticks::TickRow {
            account_id: DEMO,
            session_id: "opening-mid-fixture".into(),
            seq: 0,
            broker_symbol_id: BTC,
            received_at: today().open,
            broker_timestamp: Some(today().open.timestamp_millis()),
            bid_e5: Some(e5(OPEN - 2.5)),
            ask_e5: Some(e5(OPEN + 2.5)),
        }],
    )
    .await
    .unwrap();
}

/// M30 bars: 20 past days, each opening at 60,000.00; the bar ending at +30
/// and +60 both close 0.75% from the open (alternating signs). Thus
/// σ = 0.0075 and R = 450 at both decision marks. Today:
/// its first bar opens at 60,000.00 with `first_volume` ticks.
fn bars(first_volume: i64, days: i64) -> Vec<jiaoyi_ctrader::messages::ProtoOaTrendbar> {
    let s = today();
    let mut v = Vec::new();
    for i in 1..=days {
        let open = s.open - Duration::days(i);
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        let c30 = OPEN * (1.0 + sign * 0.0075);
        let c60 = OPEN * (1.0 + sign * 0.0075);
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
    for day in 1..=days {
        for minute in (60..1440).step_by(30) {
            let open = s.open - Duration::days(day) + Duration::minutes(minute);
            v.push(m30_bar(
                open,
                e5(OPEN),
                e5(61_000.0),
                e5(59_000.0),
                e5(60_060.0),
                100,
            ));
        }
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
        light_symbols: [BTC, 23]
            .into_iter()
            .map(|symbol_id| jiaoyi_ctrader::messages::ProtoOaLightSymbol {
                symbol_id,
                base_asset_id: Some(2),
                quote_asset_id: Some(1),
                ..Default::default()
            })
            .collect(),
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

async fn reconcile_with_scoped_recovery(t: &mut strategy3::Trader<'_, '_>, now: DateTime<Utc>) {
    t.reconcile(now).await.unwrap();
    let snapshot = t.session.reconcile().await.unwrap();
    for p in snapshot.position {
        if p.trade_data.label.as_deref() == Some("jiaoyi-s3") {
            // Exercise the same real read-only recovery as the independent
            // runtime lane; failure remains unverified, never fixture-owned.
            let _ = jiaoyi::authority::recover_position(t.pool, t.session, &p).await;
        }
    }
    t.reconcile(now).await.unwrap();
}

async fn broker_proves_silent_fill(pool: &PgPool, f: &FakeBroker, position: i64, volume: i64) {
    let (key, requested): (String, i64) = sqlx::query_as(
        "SELECT entry_key,volume FROM trading_intents WHERE account_id=$1 AND kind='open' AND entry_key LIKE 's3:%' ORDER BY id DESC LIMIT 1",
    ).bind(DEMO).fetch_one(pool).await.unwrap();
    let p = f
        .positions
        .lock()
        .unwrap()
        .iter()
        .find(|p| p.position_id == position)
        .unwrap()
        .clone();
    let order_id = position * 10 + 9;
    f.authority_orders
        .lock()
        .unwrap()
        .retain(|o| o.order_id != order_id);
    f.authority_orders
        .lock()
        .unwrap()
        .push(jiaoyi_ctrader::messages::ProtoOaOrder {
            order_id,
            position_id: Some(position),
            client_order_id: Some(key.chars().take(50).collect()),
            trade_data: ProtoOaTradeData {
                volume: requested,
                ..p.trade_data.clone()
            },
            closing_order: Some(false),
            executed_volume: Some(volume),
            ..Default::default()
        });
    let mut deals = f.authority_deals.lock().unwrap();
    let filled: i64 = deals
        .iter()
        .filter(|d| d.order_id == order_id)
        .map(|d| d.filled_volume)
        .sum();
    assert!(volume >= filled);
    if volume == filled {
        return;
    }
    let increment = volume - filled;
    let deal_id = position * 10 + deals.iter().filter(|d| d.order_id == order_id).count() as i64;
    deals.push(ProtoOaDeal {
        deal_id,
        order_id,
        position_id: position,
        symbol_id: BTC,
        volume: increment,
        filled_volume: increment,
        trade_side: p.trade_data.trade_side,
        deal_status: 2,
        execution_price: p.price,
        create_timestamp: Utc::now().timestamp_millis(),
        execution_timestamp: Utc::now().timestamp_millis(),
        ..Default::default()
    });
}

/// Exercise the independent connection quote cache as well as the actor. The
/// final dispatch guard must see the same broker observation, not a direct
/// actor-only test injection that never existed on the connection.
struct WireTrader<'a, 'c> {
    inner: strategy3::Trader<'a, 'c>,
    quote_sender: tokio::sync::broadcast::Sender<jiaoyi_ctrader::messages::ProtoOaSpotEvent>,
}

impl<'a, 'c> std::ops::Deref for WireTrader<'a, 'c> {
    type Target = strategy3::Trader<'a, 'c>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for WireTrader<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl WireTrader<'_, '_> {
    /// Explicit historical-position fixture. The retired entry algorithm is
    /// never called: this writes an old opening plan through the fake broker
    /// and the real ledger so existing-position protection remains exercised.
    /// Production callers cannot access this test-only helper.
    async fn legacy_position(
        &mut self,
        symbol: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.legacy_position_from_origin(symbol, bid, ask, now, OPEN)
            .await
    }

    async fn legacy_position_from_origin(
        &mut self,
        symbol: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
        open: f64,
    ) -> anyhow::Result<()> {
        use jiaoyi_core::strategy3::{band, cost_reserve, targets::Dir};
        use jiaoyi_ctrader::trading::{self as gate, Command, Policy, Side};
        let legacy = legacy_today();
        self.syms.get_mut(&symbol).unwrap().rule = rule_for("BTCUSD", AssetClass::Crypto);
        self.prepare(symbol, legacy, now).await?;
        let sym = self.syms[&symbol].clone();
        let day = sym.day.as_ref().unwrap().as_ref().unwrap();
        let band = band::band_at(&day.table, &legacy, open, now).unwrap();
        let long = (bid.unwrap() + ask.unwrap()) / 2.0 > open;
        let side = if long { Side::Buy } else { Side::Sell };
        let price = if long { ask.unwrap() } else { bid.unwrap() };
        let limits = sym.limits.unwrap();
        let relative_stop = trading::relative_stop(band.half_width, limits.digits);
        let entry = strategy3::Entry {
            side: if long { Dir::Buy } else { Dir::Sell },
            band,
            entry_price: price,
            relative_stop,
            r: relative_stop as f64 / 100_000.0,
            cost: cost_reserve(5.0, 6.0, 24.0, price).unwrap(),
        };
        let rules = strategy3::rules(self.pool, DEMO, self.env).await?;
        let sized = strategy3::size(&rules, 10_000.0, relative_stop, 1.0, &limits, 0).unwrap();
        let order = strategy3::plan_order(&entry, &limits, sized.volume).unwrap();
        let n =
            store::strategy3::attempts_in_session(self.pool, DEMO, symbol, today().date).await?;
        let key = format!("s3:{DEMO}:{symbol}:{}:{}", today().date, n + 1);
        let hash = "historical-fixture-pre-18377";
        let ledger = strategy3::S3Ledger {
            inner: trading::PgLedger {
                pool: self.pool,
                settings_version: Some(rules.version),
            },
            order: store::strategy3::OrderRow {
                account_id: DEMO,
                entry_key: key.clone(),
                label: "jiaoyi-s3".into(),
                prereg_hash: hash.into(),
                kind: "open".into(),
                symbol: sym.name.clone(),
                symbol_id: symbol,
                position_id: None,
                side: Some(side.as_str().into()),
                session_date: Some(today().date),
                decided_at: now,
                bid,
                ask,
                volume: Some(sized.volume),
                detail: json!({"entry":entry,"order":order,"volume_limits":limits,"session":legacy,"session_date":legacy.date,"historical_fixture":true}),
            },
            recorded_at: Mutex::new(None),
        };
        let sent = gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &Policy {
                edge: Ok(()),
                max_risk: sized.max_risk,
            },
            Command::Open {
                symbol_id: symbol,
                side,
                volume: sized.volume,
                relative_stop,
                relative_take_profit: Some(order.relative_take_profit),
                limits,
                quote_to_deposit: Some(1.0),
                label: "jiaoyi-s3".into(),
            },
            &key,
            std::time::Duration::from_secs(rules.order_wait_secs),
        )
        .await?;
        let fill = jiaoyi::fills::summarize(&sent, None, sized.volume);
        let sent_at = ledger.recorded_at.lock().unwrap().unwrap();
        store::strategy3::record_fill(
            self.pool,
            &store::strategy3::FillRow {
                account_id: DEMO,
                entry_key: key.clone(),
                label: "jiaoyi-s3".into(),
                prereg_hash: hash.into(),
                kind: "open".into(),
                symbol: sym.name,
                position_id: fill.position_id,
                side: Some(side.as_str().into()),
                outcome: fill.outcome.into(),
                sent_at,
                answered_at: fill.filled_at,
                latency_ms: fill.filled_at.map(|x| (x - sent_at).num_milliseconds()),
                decision_price: Some(price),
                fill_price: fill.price,
                slippage: jiaoyi::fills::slippage(side, fill.price, Some(price)),
                spread: Some(ask.unwrap() - bid.unwrap()),
                volume: fill.volume,
                detail: json!({"outcome":sent.outcome,"client_msg_id":sent.client_msg_id}),
            },
        )
        .await?;
        store::trading::record_decision(self.pool,DEMO,Some(&self.syms[&symbol].name),"s3_sent",&format!("Historical fixture {} {} cents",side.as_str(),sized.volume),&json!({"entry_key":key,"prereg_hash":hash,"label":"jiaoyi-s3","historical_fixture":true}),None).await?;
        let sym = self.syms.get_mut(&symbol).unwrap();
        sym.bid = bid;
        sym.ask = ask;
        sym.bid_at = Some(now);
        sym.ask_at = Some(now);
        self.reconcile(Utc::now()).await?;
        Ok(())
    }

    /// Exercise the real production arm and retest, including the wire quote
    /// cache. `bid`/`ask` are the intended retest, not the initial peak.
    async fn enter_pullback(
        &mut self,
        symbol: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if self.own.values().any(|o| o.symbol_id == symbol) {
            return self.on_quote(symbol, bid, ask, now).await;
        }
        let mid = (bid.unwrap() + ask.unwrap()) / 2.0;
        let peak = OPEN + 2.0 * (mid - OPEN);
        self.on_quote(symbol, Some(peak - 2.5), Some(peak + 2.5), now)
            .await?;
        self.on_quote(symbol, bid, ask, now + Duration::milliseconds(1))
            .await
    }

    async fn on_quote(
        &mut self,
        symbol: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let mut wire = self.session.connection().spot_events();
        let event = jiaoyi_ctrader::messages::ProtoOaSpotEvent {
            ctid_trader_account_id: self.session.account().id(),
            symbol_id: symbol,
            bid: bid.map(|x| e5(x) as u64),
            ask: ask.map(|x| e5(x) as u64),
            ..Default::default()
        };
        self.quote_sender.send(event.clone()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let received = wire.recv().await.unwrap();
                if received.event == event {
                    break;
                }
            }
        })
        .await
        .expect("the fake quote must pass through the connection reader");
        self.inner.on_quote(symbol, bid, ask, now).await
    }
}

/// Drive a trader on `f`: `steps` gets the trader and the fake's log.
macro_rules! with_trader {
    ($pool:expr, $f:expr, $env:expr, |$t:ident, $log:ident| $body:block) => {{
        let mut fake = $f.clone();
        let quote_sender = fake
            .spot_events
            .clone()
            .unwrap_or_else(|| tokio::sync::broadcast::channel(64).0);
        fake.spot_events = Some(quote_sender.clone());
        let (conn, $log) = fake.spawn();
        let session = loader::authorise($pool, &conn, &broker(), &permitted())
            .await
            .unwrap();
        let list = allowlist();
        let inner = strategy3::Trader::new($pool, &list, $env, &session)
            .await
            .unwrap();
        let mut $t = WireTrader {
            inner,
            quote_sender,
        };
        reconcile_with_scoped_recovery(&mut $t, Utc::now()).await;
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

/// Measurement-only fixture, explicitly run before changing dispatch policy.
/// Rates apply to this declared scenario mix, never to production traffic.
#[sqlx::test(migrator = "store::MIGRATOR")]
#[ignore = "manual bounded dispatch benchmark; production impact needs separate evidence"]
async fn dispatch_policy_benchmark(pool: PgPool) {
    loaded(&pool).await;
    sqlx::raw_sql(
        "CREATE TABLE dispatch_bench_control(ledger_delay_ms bigint NOT NULL);
         INSERT INTO dispatch_bench_control VALUES(0);
         CREATE FUNCTION dispatch_bench_delay() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN PERFORM pg_sleep((SELECT ledger_delay_ms::double precision/1000 FROM dispatch_bench_control));
         RETURN NEW; END $$;
         CREATE TRIGGER dispatch_bench_delay BEFORE INSERT ON trading_intents
         FOR EACH ROW EXECUTE FUNCTION dispatch_bench_delay();",
    )
    .execute(&pool)
    .await
    .unwrap();
    // (label, quote offset after mark, first account-read delay, intent delay,
    // prepared cache, repetitions). All prices, risks and fills are synthetic.
    let scenarios = [
        ("normal_early", 0_i64, 0_u64, 0_i64, true, 3),
        ("normal_late_4500ms", 4500, 0, 0, true, 3),
        ("normal_late_4900ms", 4900, 0, 0, true, 3),
        ("normal_late_4990ms", 4990, 0, 0, true, 3),
        ("account_250ms_early", 0, 250, 0, true, 3),
        ("account_250ms_late_4500ms", 4500, 250, 0, true, 3),
        ("account_250ms_late_4900ms", 4900, 250, 0, true, 3),
        ("ledger_250ms_early", 0, 0, 250, true, 3),
        ("ledger_250ms_late_4500ms", 4500, 0, 250, true, 3),
        ("ledger_250ms_late_4900ms", 4900, 0, 250, true, 3),
        ("ledger_5100ms_early", 0, 0, 5100, true, 1),
        ("cold_preparation_early", 0, 0, 0, false, 3),
    ];
    for (label, offset, account_delay, ledger_delay, prepared, repetitions) in scenarios {
        for repetition in 0..repetitions {
            // Only this sqlx test's disposable isolated database is reset.
            sqlx::raw_sql("TRUNCATE strategy3_orders,strategy3_fills,trading_intents,trading_events,trading_decisions;")
                .execute(&pool).await.unwrap();
            sqlx::query("UPDATE dispatch_bench_control SET ledger_delay_ms=$1")
                .bind(ledger_delay)
                .execute(&pool)
                .await
                .unwrap();
            let f = fake((60_455.0, 60_462.0));
            with_trader!(&pool, f, Environment::Demo, |t, log| {
                if prepared {
                    t.prepare(BTC, today(), at(25)).await.unwrap();
                }
                f.trader_reply_delays
                    .lock()
                    .unwrap()
                    .push_back(std::time::Duration::from_millis(account_delay));
                let checked_from = Utc::now();
                let started = std::time::Instant::now();
                t.enter_pullback(
                    BTC,
                    Some(60_455.0),
                    Some(60_462.0),
                    at(30) + Duration::milliseconds(offset),
                )
                .await
                .unwrap();
                let completed_us = started.elapsed().as_micros();
                let receipts: Vec<(String, Option<String>, Vec<u8>)> = sqlx::query_as(
                    "SELECT kind,error_code,raw FROM trading_events WHERE kind IN ('ENTRY_VALIDATED_BEFORE_SEND','REFUSED_BEFORE_SEND') ORDER BY id",
                ).fetch_all(&pool).await.unwrap();
                assert_eq!(
                    receipts.len(),
                    1,
                    "{label}: require exact final validation evidence; {:?}",
                    decisions(&pool).await
                );
                let (kind, error, bytes) = &receipts[0];
                let raw: serde_json::Value = serde_json::from_slice(bytes).unwrap();
                let snapshot = if kind == "REFUSED_BEFORE_SEND" {
                    &raw["evidence"]
                } else {
                    &raw
                };
                let checked_at: DateTime<Utc> =
                    serde_json::from_value(snapshot["checked_at"].clone()).unwrap();
                let frames = orders(&log).len();
                assert_eq!(frames, usize::from(kind == "ENTRY_VALIDATED_BEFORE_SEND"));
                if frames == 0 {
                    assert_eq!(error.as_deref(), Some("ENTRY_EXPIRED"));
                }
                eprintln!(
                    "DISPATCH_BENCH {}",
                    json!({
                        "scenario":label,"repetition":repetition,"quote_offset_ms":offset,
                        "initial_dispatch_budget_ms":5000,"first_account_reply_delay_ms":account_delay,
                        "intent_delay_ms":ledger_delay,"preparation_warmed":prepared,
                        "decision_to_final_guard_us":(checked_at-checked_from).num_microseconds(),
                        "quote_call_completed_us":completed_us,"broker_frames":frames,
                        "receipt_kind":kind,"error_code":error,
                        "remaining_dispatch_ms":snapshot["remaining_dispatch_ms"],
                        "source":"isolated local PostgreSQL + in-memory real-protobuf broker; not production"
                    })
                );
            });
        }
    }
}

/// v5 hangs each stop from the best executable quote by the original R,
/// including after broker reconciliation and restart at break-even.
async fn trail_keeps_original_r(pool: &PgPool, buy: bool) {
    loaded(pool).await;
    let (bid, ask) = if buy {
        (60_455.0, 60_462.0)
    } else {
        (59_538.0, 59_545.0)
    };
    with_trader!(pool, fake((bid, ask)), Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(bid), Some(ask), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        let own = t.own.values().next().unwrap();
        let pid = own.position_id;
        let entry = own.entry.unwrap();
        let tp = own.take_profit;
        let r = own.plan_basis.unwrap().r;
        assert_eq!(r, 450.0);
        let direction = if buy { 1.0 } else { -1.0 };
        for (i, gain) in [200.0, 250.0, 450.0].into_iter().enumerate() {
            let price = entry + direction * gain;
            let (bid, ask) = if buy {
                (price, price + 5.0)
            } else {
                (price - 5.0, price)
            };
            let now = at(31) + Duration::seconds(31 * i as i64);
            t.on_quote(BTC, Some(bid), Some(ask), now).await.unwrap();
            assert_eq!(log.lock().unwrap().amends.len(), i + 1);
            assert_eq!(
                t.own[&pid].stop,
                Some(price - direction * r),
                "each amendment must retain the original 450-point R"
            );
            t.reconcile(now).await.unwrap();
        }
        assert_eq!(t.own[&pid].stop, Some(entry));
        let restarted = strategy3::Trader::new(pool, t.allowlist, t.env, t.session)
            .await
            .unwrap();
        let mut restarted = WireTrader {
            inner: restarted,
            quote_sender: t.quote_sender.clone(),
        };
        reconcile_with_scoped_recovery(&mut restarted, at(33)).await;
        assert_eq!(restarted.own[&pid].stop, Some(entry));
        let price = entry + direction * 500.0;
        let (bid, ask) = if buy {
            (price, price + 5.0)
        } else {
            (price - 5.0, price)
        };
        restarted
            .on_quote(BTC, Some(bid), Some(ask), at(34))
            .await
            .unwrap();
        assert_eq!(
            restarted.own[&pid].stop,
            Some(price - direction * r),
            "restart at break-even must not turn original R into zero"
        );
        let l = log.lock().unwrap();
        assert_eq!(l.amends.len(), 4);
        assert!(l.amends.iter().all(|a| a.take_profit == tp));
        assert!(l.closes.is_empty());
        assert_eq!(l.orders.len(), 1, "recovery must not open another position");
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn long_trail_retains_original_r_after_amends_and_restart(pool: PgPool) {
    trail_keeps_original_r(&pool, true).await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn short_trail_retains_original_r_after_amends_and_restart(pool: PgPool) {
    trail_keeps_original_r(&pool, false).await;
}

/// Replay one-decimal prices like the rejected JPN225 amendment through the
/// existing continuous-session fixture, including the decoded wire request.
async fn fractional_trail_precision(pool: &PgPool, buy: bool) {
    loaded(pool).await;
    sqlx::query("UPDATE symbols SET digits=1 WHERE account_id=$1 AND broker_symbol_id=$2")
        .bind(DEMO)
        .bind(BTC)
        .execute(pool)
        .await
        .unwrap();
    let (bid, ask) = if buy {
        (65_354.1, 65_361.1)
    } else {
        (63_831.9, 63_838.9)
    };
    let mut f = fake((bid, ask));
    let session = today();
    let mut history = Vec::new();
    for day in 1..=20 {
        let open = session.open - Duration::days(day);
        let close = 64_600.0 + if day % 2 == 0 { 343.1 } else { -343.1 };
        for minute in [0, 30] {
            history.push(m30_bar(
                open + Duration::minutes(minute),
                e5(64_600.0),
                e5(66_000.0),
                e5(63_000.0),
                e5(close),
                100,
            ));
        }
    }
    history.push(m30_bar(
        session.open,
        e5(64_600.0),
        e5(64_600.0),
        e5(64_600.0),
        e5(64_600.0),
        10,
    ));
    f.m30_bars.insert(BTC, history);
    with_trader!(pool, f, Environment::Demo, |t, log| {
        t.legacy_position_from_origin(BTC, Some(bid), Some(ask), at(30), 64_600.0)
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        let pid = t.own.values().next().unwrap().position_id;
        let basis = t.own[&pid].plan_basis.unwrap();
        assert_eq!(basis.r, 343.1);
        let tp = t.own[&pid].take_profit;
        let quotes = if buy {
            [65_477.4, 65_491.9]
        } else {
            [63_791.2, 63_777.7]
        };
        let stops = if buy {
            [65_134.3, 65_148.8]
        } else {
            [64_134.3, 64_120.8]
        };
        for (i, (price, stop)) in quotes.into_iter().zip(stops).enumerate() {
            let (b, a) = if buy {
                (price, price + 5.0)
            } else {
                (price - 5.0, price)
            };
            t.on_quote(
                BTC,
                Some(b),
                Some(a),
                at(31) + Duration::seconds(i as i64 * 31),
            )
            .await
            .unwrap();
            {
                let wire = log.lock().unwrap();
                assert_eq!(wire.amends.len(), i + 1);
                assert_eq!(
                    wire.amends[i].stop_loss,
                    Some(stop),
                    "wire stop must have canonical symbol precision"
                );
                assert_eq!(wire.amends[i].take_profit, tp);
            }
            assert_eq!(t.own[&pid].stop, Some(stop));
            assert_eq!(t.own[&pid].plan_basis.unwrap().r, basis.r);
            let saved: (f64, serde_json::Value) = sqlx::query_as(
                "SELECT i.stop_price,o.detail FROM trading_intents i JOIN strategy3_orders o USING(account_id,entry_key) WHERE i.account_id=$1 AND i.position_id=$2 ORDER BY i.created_at DESC LIMIT 1")
                .bind(DEMO).bind(pid).fetch_one(pool).await.unwrap();
            assert_eq!(saved.0, stop);
            assert_eq!(saved.1["new_stop"], stop);
            assert_eq!(saved.1["r"], basis.r);
            t.reconcile(at(33)).await.unwrap();
        }
        let restarted = strategy3::Trader::new(pool, t.allowlist, t.env, t.session)
            .await
            .unwrap();
        let mut restarted = WireTrader {
            inner: restarted,
            quote_sender: t.quote_sender.clone(),
        };
        reconcile_with_scoped_recovery(&mut restarted, at(34)).await;
        assert_eq!(restarted.own[&pid].plan_basis.unwrap().r, basis.r);
        let (b, a, stop) = if buy {
            (65_500.1, 65_505.1, 65_157.0)
        } else {
            (63_770.5, 63_775.5, 64_118.6)
        };
        restarted
            .on_quote(BTC, Some(b), Some(a), at(35))
            .await
            .unwrap();
        let wire = log.lock().unwrap();
        assert_eq!(wire.amends.len(), 3);
        assert_eq!(wire.amends[2].stop_loss, Some(stop));
        assert!(wire.amends.iter().all(|x| x.take_profit == tp));
        assert_eq!(wire.orders.len(), 1);
        assert!(wire.closes.is_empty());
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn fractional_long_trail_has_canonical_wire_price_and_original_r(pool: PgPool) {
    fractional_trail_precision(&pool, true).await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn fractional_short_trail_has_canonical_wire_price_and_original_r(pool: PgPool) {
    fractional_trail_precision(&pool, false).await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn trail_requires_price_grid_and_one_tick_improvement(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.legacy_position(BTC, Some(60_455.0), Some(60_462.0), at(30))
                .await
                .unwrap();
            let pid = t.own.values().next().unwrap().position_id;
            let basis = t.own[&pid].plan_basis.unwrap();
            let stop = t.own[&pid].stop;
            let limits = t.syms[&BTC].limits.unwrap();
            for (i, digits) in [None, Some(-1), Some(6)].into_iter().enumerate() {
                t.syms.get_mut(&BTC).unwrap().limits =
                    digits.map(|digits| jiaoyi_ctrader::trading::VolumeLimits { digits, ..limits });
                t.on_quote(BTC, Some(60_700.0), Some(60_705.0), at(31 + i as i64))
                    .await
                    .unwrap();
                assert!(log.lock().unwrap().amends.is_empty());
                assert_eq!(t.own[&pid].stop, stop);
            }
            t.syms.get_mut(&BTC).unwrap().limits = Some(limits);
            t.own.get_mut(&pid).unwrap().plan_basis = Some(strategy3::PlanBasis {
                r: 450.001,
                ..basis
            });
            t.on_quote(BTC, Some(60_700.0), Some(60_705.0), at(35))
                .await
                .unwrap();
            assert!(
                log.lock().unwrap().amends.is_empty(),
                "off-grid R must not be silently changed"
            );
            t.own.get_mut(&pid).unwrap().plan_basis = Some(basis);
            t.own.get_mut(&pid).unwrap().stop = Some(60_250.0);
            t.on_quote(BTC, Some(60_700.0 + 1e-10), Some(60_705.0), at(36))
                .await
                .unwrap();
            assert!(
                log.lock().unwrap().amends.is_empty(),
                "float residue is not a one-tick improvement"
            );
            assert_eq!(t.own[&pid].stop, Some(60_250.0));
            t.on_quote(BTC, Some(60_700.01), Some(60_705.01), at(37))
                .await
                .unwrap();
            assert_eq!(log.lock().unwrap().amends.len(), 1);
            assert_eq!(t.own[&pid].stop, Some(60_250.01));
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn trail_requires_known_positive_original_r(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            let own = t.own.values().next().unwrap();
            let pid = own.position_id;
            let basis = own.plan_basis.unwrap();
            let stop = own.stop;
            for (i, r) in [
                None,
                Some(0.0),
                Some(-450.0),
                Some(f64::NAN),
                Some(f64::INFINITY),
            ]
            .into_iter()
            .enumerate()
            {
                t.own.get_mut(&pid).unwrap().plan_basis =
                    r.map(|r| strategy3::PlanBasis { r, ..basis });
                t.on_quote(BTC, Some(60_700.0), Some(60_705.0), at(31 + i as i64))
                    .await
                    .unwrap();
                assert!(
                    log.lock().unwrap().amends.is_empty(),
                    "unknown original R cannot use the current stop as a substitute"
                );
                assert_eq!(t.own[&pid].stop, stop);
            }
            t.own.get_mut(&pid).unwrap().plan_basis = Some(basis);
            t.on_quote(BTC, Some(60_700.0), Some(60_705.0), at(37))
                .await
                .unwrap();
            assert_eq!(log.lock().unwrap().amends.len(), 1);
            assert_eq!(t.own[&pid].stop, Some(60_250.0));
        }
    );
}

/// A flat symbol's slow session preparation cannot delay an already owned
/// symbol's TP1. The fake still answers quotes and trading requests.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn slow_session_bars_do_not_starve_another_positions_tp1(pool: PgPool) {
    const OTHER: i64 = BTC + 1;
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(t.own.len(), 1);
    });
    settings::change(
        &pool,
        "universe.symbols",
        Some(DEMO),
        Some(json!([
            {"name":"BTCUSD","class":"crypto"},{"name":"ETHUSD","class":"crypto"}
        ])),
        "test",
        "two-symbol exit starvation fixture",
    )
    .await
    .unwrap();
    let run = store::start_run(&pool, DEMO).await.unwrap();
    for (id, name) in [(BTC, "BTCUSD"), (OTHER, "ETHUSD")] {
        let s = ProtoOaSymbol {
            symbol_id: id,
            digits: 2,
            pip_position: 2,
            min_volume: Some(1),
            step_volume: Some(1),
            ..Default::default()
        };
        store::upsert_symbol(
            &pool,
            &loader::symbol_row(DEMO, name, AssetClass::Crypto, Some(true), &s, Utc::now()),
        )
        .await
        .unwrap();
        store::record_resolution(
            &pool,
            run,
            DEMO,
            name,
            AssetClass::Crypto,
            &Resolution::Resolved { symbol_id: id },
        )
        .await
        .unwrap();
    }
    store::finish_run(&pool,run,"ok",&store::RunSummary{
        assets:Some(json!([{"asset_id":1,"name":"USD"},{"asset_id":9,"name":"BTC"}])),
        listed_symbols:Some(json!([{"id":BTC,"quote_asset_id":1,"base_asset_id":9},{"id":OTHER,"quote_asset_id":1,"base_asset_id":9}])),
        ..Default::default()
    }).await.unwrap();
    f.m30_bars.insert(OTHER, bars(10, 20));
    f.trendbar_reply_delays
        .lock()
        .unwrap()
        .push_back(std::time::Duration::from_secs(10));
    f.quote_sequence = vec![
        (
            0,
            OTHER,
            (Some(e5(60_455.0) as u64), Some(e5(60_460.0) as u64)),
        ),
        (
            500,
            BTC,
            (Some(e5(61_384.21) as u64), Some(e5(61_389.0) as u64)),
        ),
    ];
    let (conn, log) = f.spawn();
    let session = loader::authorise(&pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let mut ends = conn.session_ends();
    let (_trigger, mut stop) = shutdown::channel();
    let live = trading::LivenessMap::default();
    let list = allowlist();
    let offset = at(30) - Utc::now();
    tokio::select! {
        r=strategy3::run_session_with_event_offset(&pool,&list,Environment::Demo,&session,&live,&mut ends,&mut stop,offset)=>r.unwrap(),
        _=tokio::time::sleep(std::time::Duration::from_secs(2))=>{},
    }
    let l = log.lock().unwrap();
    assert!(
        l.trendbar_requests.iter().any(|request| request.0 == OTHER),
        "the flat symbol's history request started alongside background warm-up"
    );
    assert!(
        l.orders.is_empty(),
        "no new entry before its setup is ready"
    );
    assert_eq!(
        l.closes.len(),
        1,
        "owned TP1 must finish while the other history read is pending"
    );
    assert_eq!(l.closes[0].volume, 8);
    // v5 trails a favourable move. The amend must carry the take-profit.
    // Grok 4.7 · № 17,655. It is not a close.
    assert!(l.amends.iter().all(|a| a.take_profit.is_some()));
}

/// A draining observer does not prove that the trading actor is lossless.
/// Hold a real TP1 receipt and overflow only the actor's quote cursor.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn actor_quote_loss_is_visible_when_the_observer_keeps_up(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(t.own.len(), 1);
    });
    settings::change(
        &pool,
        "trading.order_wait_secs",
        Some(DEMO),
        Some(json!(120)),
        "test",
        "receipt released by a barrier, never by the normal order timeout",
    )
    .await
    .unwrap();
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    // A panic/timeout must release the fake's waiter and its wire-event task.
    struct CloseGate(Arc<tokio::sync::Semaphore>);
    impl Drop for CloseGate {
        fn drop(&mut self) {
            self.0.close();
        }
    }
    let _cleanup = CloseGate(gate.clone());
    let (spots, _) = tokio::sync::broadcast::channel(512);
    f.spot_events = Some(spots.clone());
    f.close_reply_gate = Some(gate.clone());
    let (conn, log) = f.spawn();
    let session = loader::authorise(&pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let list = allowlist();
    let live = trading::LivenessMap::default();
    let mut ends = conn.session_ends();
    let mut received = conn.spot_events();
    let (trigger, mut stop) = shutdown::channel();
    let health = || live.lock().unwrap()[&DEMO].monitor.health();
    let actor = strategy3::run_session_with_event_offset(
        &pool,
        &list,
        Environment::Demo,
        &session,
        &live,
        &mut ends,
        &mut stop,
        at(31) - Utc::now(),
    );
    tokio::pin!(actor);
    let exercise = async {
        // Initialization completed: both actor receivers now exist.
        loop {
            let ready = live.lock().unwrap().get(&DEMO).is_some_and(|l| {
                let h = l.monitor.health();
                h["active_stage"].is_null() && !h["last_work_completed_at"].is_null()
            });
            if ready {
                break;
            }
            tokio::task::yield_now().await;
        }
        spots
            .send(jiaoyi_ctrader::messages::ProtoOaSpotEvent {
                ctid_trader_account_id: DEMO,
                symbol_id: BTC,
                bid: Some(e5(61_384.21) as u64),
                ask: Some(e5(61_389.0) as u64),
                ..Default::default()
            })
            .unwrap();
        let trigger_receipt = json!(received.recv().await.unwrap().received_at);
        while log.lock().unwrap().closes.is_empty() {
            tokio::task::yield_now().await;
        }
        // 5120 events exceed the real actor broadcast capacity (4096).
        // Receipt barriers let the independent observer drain each batch;
        // no sleeps or scheduler-speed assumption creates the overflow.
        for _ in 0..32 {
            for _ in 0..160 {
                spots
                    .send(jiaoyi_ctrader::messages::ProtoOaSpotEvent {
                        ctid_trader_account_id: DEMO,
                        symbol_id: BTC,
                        bid: Some(e5(60_455.0) as u64),
                        ask: Some(e5(60_460.0) as u64),
                        ..Default::default()
                    })
                    .unwrap();
            }
            let mut last = None;
            for _ in 0..160 {
                last = Some(received.recv().await.unwrap().received_at);
            }
            let receipt = json!(last.unwrap());
            while health()["last_quote_received_at"] != receipt {
                tokio::task::yield_now().await;
            }
        }
        assert_eq!(health()["quote_receiver_lagged"], 0);
        assert_eq!(health()["actor_quote_receiver_lagged"], 0);
        gate.add_permits(1);
        loop {
            let h = health();
            let processed = &h["last_quote_processed_receipt_at"];
            if h["actor_quote_receiver_lagged"] != 0
                || (!processed.is_null() && *processed != trigger_receipt)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
        let h = health();
        assert!(h["actor_quote_receiver_lagged"].as_u64().unwrap() >= 1024);
        assert_eq!(h["actor_execution_receiver_lagged"], 0);
        assert_eq!(h["quote_receiver_lagged"], 0);
        assert_eq!(h["quote_receiver_lagged_scope"], "independent_observer");
        trigger.fire();
    };
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        tokio::select! {
            result = &mut actor => panic!("actor stopped before loss assertion: {result:?}"),
            () = exercise => {},
        }
        actor.await.unwrap();
    })
    .await
    .expect("controlled actor overflow and shutdown must complete");
    let l = log.lock().unwrap();
    assert!(l.orders.is_empty(), "monitoring cannot create entries");
    assert_eq!(l.closes.len(), 1, "the held TP1 is never duplicated");
    assert_eq!(l.closes[0].volume, 8);
    assert!(l.amends.iter().all(|a| a.take_profit.is_some()));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn exit_timer_rejects_stale_or_future_quotes_and_does_not_repeat_tp1(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            // Timer checks cannot turn a flat symbol into an entry.
            t.syms.get_mut(&BTC).unwrap().bid = Some(61_384.21);
            t.check_exits(at(30)).await.unwrap();
            assert!(orders(&log).is_empty());
            t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            let now = at(61);
            {
                let s = t.syms.get_mut(&BTC).unwrap();
                s.bid = Some(61_384.21);
                s.bid_at = Some(now - Duration::seconds(7));
            }
            t.check_exits(now).await.unwrap();
            assert!(closes(&log).is_empty());
            t.syms.get_mut(&BTC).unwrap().bid_at = Some(now + Duration::seconds(1));
            t.check_exits(now).await.unwrap();
            assert!(closes(&log).is_empty());
            t.on_quote(BTC, Some(61_384.21), Some(61_389.0), now)
                .await
                .unwrap();
            t.check_exits(now + Duration::seconds(1)).await.unwrap();
            t.check_exits(now + Duration::seconds(2)).await.unwrap();
            assert_eq!(closes(&log), vec![8]);
            assert!(log
                .lock()
                .unwrap()
                .amends
                .iter()
                .all(|a| a.take_profit.is_some()));
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn inside_quote_keeps_slot_available_and_qualified_arm_restores_after_restart(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60700.0, 60705.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(60000.0), Some(60005.0), at(60))
                .await
                .unwrap();
            assert!(
                t.syms[&BTC].pullback.is_none(),
                "inside quote does not arm or consume the slot"
            );
            t.on_quote(
                BTC,
                Some(60700.0),
                Some(60705.0),
                at(60) + Duration::seconds(4),
            )
            .await
            .unwrap();
            assert!(
                orders(&log).is_empty(),
                "a qualifying impulse arms but does not enter at its peak"
            );
            let armed = t.syms[&BTC]
                .pullback
                .expect("later qualifying impulse uses the same slot");
            let restarted =
                strategy3::Trader::new(&pool, t.allowlist, Environment::Demo, t.session)
                    .await
                    .unwrap();
            let mut restarted = WireTrader {
                inner: restarted,
                quote_sender: t.quote_sender.clone(),
            };
            restarted.reconcile(Utc::now()).await.unwrap();
            restarted
                .on_quote(
                    BTC,
                    Some(60700.0),
                    Some(60705.0),
                    at(60) + Duration::seconds(4),
                )
                .await
                .unwrap();
            assert!(
                orders(&log).is_empty(),
                "restart preserves the same arm and still waits for its pullback"
            );
            let restored = restarted.syms[&BTC].pullback.unwrap();
            assert_eq!(
                (restored.session, restored.armed_at, restored.long),
                (armed.session, armed.armed_at, armed.long)
            );
            assert_eq!(
                (
                    restored.origin,
                    restored.extreme,
                    restored.level,
                    restored.impulse_range
                ),
                (
                    armed.origin,
                    armed.extreme,
                    armed.level,
                    armed.impulse_range
                )
            );
            for (observed, expected) in [
                (restored.impulse_bps, armed.impulse_bps),
                (restored.macro_context.drift_5, armed.macro_context.drift_5),
                (
                    restored.macro_context.drift_20,
                    armed.macro_context.drift_20,
                ),
            ] {
                assert!(
                    (observed - expected).abs() <= 1e-12 * expected.abs().max(1.0),
                    "JSON/Postgres float roundtrip may move only rounding residue"
                );
            }
            assert_eq!(
                restored.macro_context.latest_bar_end,
                armed.macro_context.latest_bar_end
            );
        }
    );
}

/// The registered C >= R veto is execution state across fresh Trader instances,
/// even when a later mark's quote would make the cost acceptable.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn durable_cost_veto_survives_restart_and_a_favorable_quote(pool: PgPool) {
    loaded(&pool).await;
    sqlx::query("UPDATE deals SET commission=-60000 WHERE account_id=$1")
        .bind(DEMO)
        .execute(&pool)
        .await
        .unwrap();
    let assert_blocker = |sym: &strategy3::Sym, now| {
        let x = strategy3::trigger_distance(sym, now);
        assert_eq!(x["reason_code"], "session_cost_veto", "{x}");
        assert_eq!(x["session_date"], json!(today().date));
        assert!(x["reason"].as_str().unwrap().contains("recorded cost veto"));
        assert!(x["distance_half_widths"].is_null());
        assert_eq!(x["bid"], json!(sym.bid));
        assert_eq!(x["ask"], json!(sym.ask));
        assert!(
            x.as_object().unwrap().keys().all(|key| matches!(
                key.as_str(),
                "at" | "session_date"
                    | "bid"
                    | "ask"
                    | "distance_half_widths"
                    | "reason"
                    | "reason_code"
            )),
            "the public blocker contains only quote/session/reason fields: {x}"
        );
        x
    };
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            assert!(orders(&log).is_empty());
            assert_eq!(t.syms[&BTC].cost_skip_session, Some(today().open));
            assert!(matches!(&t.syms[&BTC].day, Some(Ok(_))));
            assert_blocker(&t.syms[&BTC], at(30));
            let detail: serde_json::Value = sqlx::query_scalar(
                "SELECT detail FROM trading_decisions WHERE kind='s3_skip' ORDER BY id DESC LIMIT 1",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(detail["skip_code"], "cost_ge_r");
            assert_eq!(detail["label"], "jiaoyi-s3");
            assert_eq!(detail["prereg_hash"], strategy3::prereg_hash());

            let restarted =
                strategy3::Trader::new(&pool, t.allowlist, Environment::Demo, t.session)
                    .await
                    .unwrap();
            let mut restarted = WireTrader {
                inner: restarted,
                quote_sender: t.quote_sender.clone(),
            };
            restarted.reconcile(Utc::now()).await.unwrap();
            assert_eq!(restarted.syms[&BTC].cost_skip_session, None);
            restarted
                .on_quote(BTC, Some(60_455.0), Some(60_460.0), at(60))
                .await
                .unwrap();
            assert!(
                orders(&log).is_empty(),
                "a tighter quote cannot reopen the session"
            );
            assert_eq!(restarted.syms[&BTC].cost_skip_session, Some(today().open));
            assert!(restarted.syms[&BTC].day.is_none());
            let x = assert_blocker(&restarted.syms[&BTC], at(60));
            let live = restarted.live.lock().unwrap().clone();
            let h = strategy3::health(&pool, &live).await.unwrap();
            let mut projected = h[0]["trigger_distance"]["BTCUSD"].clone();
            let preparation = projected
                .as_object_mut()
                .unwrap()
                .remove("entry_preparation")
                .unwrap();
            assert_eq!(preparation["prepared_now"], false);
            assert_eq!(projected, x);

            // The durable blocker does not invent quote data or financial readiness.
            let mut without_quotes = restarted.syms[&BTC].clone();
            without_quotes.bid = None;
            without_quotes.ask = None;
            assert_blocker(&without_quotes, at(60));
            without_quotes.day = t.syms[&BTC].day.clone();
            without_quotes.cost_skip_session = None;
            let no_quote = strategy3::trigger_distance(&without_quotes, at(30));
            assert_eq!(no_quote["reason"], "no quote yet");
            assert!(no_quote["distance_half_widths"].is_null());
            assert!(no_quote["reason_code"].is_null());

            // A prior session's veto is not today's blocker. A genuinely missing
            // preparation and a window exclusion must retain their own reasons.
            let next =
                strategy3::trigger_distance(&restarted.syms[&BTC], at(60) + Duration::days(1));
            assert_eq!(next["reason"], "today's session is not prepared yet");
            assert!(next["reason_code"].is_null());
            assert_blocker(&restarted.syms[&BTC], at(29));
            let mut other = restarted.syms[&BTC].clone();
            other.name = "ETHUSD".into();
            other.cost_skip_session = None;
            let unprepared = strategy3::trigger_distance(&other, at(60));
            assert_eq!(unprepared["reason"], "today's session is not prepared yet");
            assert!(unprepared["reason_code"].is_null());
            let marks: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM trading_decisions WHERE kind='s3_skip' AND detail->>'skip_code'='cost_ge_r'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(
                marks, 1,
                "the durable veto is written once; later quotes do not re-evaluate it"
            );
        }
    );
    assert!(pause_writes(&pool).await.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn durable_cost_veto_is_scoped_and_ordinary_skips_do_not_latch(pool: PgPool) {
    loaded(&pool).await;
    let hash = strategy3::prereg_hash();
    let previous = today().date - Duration::days(1);
    let legacy = json!({"label":"jiaoyi-s3", "prereg_hash":hash, "session_date":previous});
    store::trading::record_decision(
        &pool,
        DEMO,
        Some("BTCUSD"),
        "s3_skip",
        "cost reserve C (500.000000) >= R (450.000000): barrier too far for a no-drift process; skipped",
        &legacy,
        None,
    )
    .await
    .unwrap();
    assert!(
        store::strategy3::cost_skipped_in_session(&pool, DEMO, "BTCUSD", previous, &hash)
            .await
            .unwrap(),
        "legacy receipts remain authoritative"
    );
    for (account, symbol, date, registration) in [
        (DEMO, "BTCUSD", today().date, hash.as_str()),
        (DEMO + 1, "BTCUSD", previous, hash.as_str()),
        (DEMO, "ETHUSD", previous, hash.as_str()),
        (DEMO, "BTCUSD", previous, "another-registration"),
    ] {
        assert!(!store::strategy3::cost_skipped_in_session(
            &pool,
            account,
            symbol,
            date,
            registration,
        )
        .await
        .unwrap());
    }
    store::upsert_account(&pool, DEMO + 1, Environment::Demo)
        .await
        .unwrap();
    // Each near-match must leave today's BTC entry eligible. An ordinary
    // budget skip is deliberately identical in scope to the real entry.
    for (account, symbol, registration, label, kind, reason, code) in [
        (
            DEMO + 1,
            "BTCUSD",
            hash.as_str(),
            "jiaoyi-s3",
            "s3_skip",
            "typed cost veto",
            "cost_ge_r",
        ),
        (
            DEMO,
            "ETHUSD",
            hash.as_str(),
            "jiaoyi-s3",
            "s3_skip",
            "typed cost veto",
            "cost_ge_r",
        ),
        (
            DEMO,
            "BTCUSD",
            "another-registration",
            "jiaoyi-s3",
            "s3_skip",
            "typed cost veto",
            "cost_ge_r",
        ),
        (
            DEMO,
            "BTCUSD",
            hash.as_str(),
            "another-strategy",
            "s3_skip",
            "typed cost veto",
            "cost_ge_r",
        ),
        (
            DEMO,
            "BTCUSD",
            hash.as_str(),
            "jiaoyi-s3",
            "s3_other",
            "typed cost veto",
            "cost_ge_r",
        ),
        (
            DEMO,
            "BTCUSD",
            hash.as_str(),
            "jiaoyi-s3",
            "s3_skip",
            "open risk would exceed its budget",
            "budget",
        ),
    ] {
        store::trading::record_decision(
            &pool,
            account,
            Some(symbol),
            kind,
            reason,
            &json!({"label":label, "prereg_hash":registration,
                "session_date":today().date, "skip_code":code}),
            None,
        )
        .await
        .unwrap();
        assert!(!store::strategy3::cost_skipped_in_session(
            &pool,
            DEMO,
            "BTCUSD",
            today().date,
            &hash,
        )
        .await
        .unwrap());
    }
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            assert_eq!(
                orders(&log).len(),
                1,
                "unrelated skips must permit the valid entry"
            );
        }
    );
    assert!(pause_writes(&pool).await.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn durable_cost_veto_keeps_one_entry_per_session_after_restart(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        // Simulate a broker-flat restart; the durable open receipt, rather
        // than the former in-memory ownership cache, must block re-entry.
        f.positions.lock().unwrap().clear();
        let restarted = strategy3::Trader::new(&pool, t.allowlist, Environment::Demo, t.session)
            .await
            .unwrap();
        let mut restarted = WireTrader {
            inner: restarted,
            quote_sender: t.quote_sender.clone(),
        };
        restarted.reconcile(Utc::now()).await.unwrap();
        assert!(restarted.own.is_empty());
        assert_eq!(restarted.syms[&BTC].entered_session, None);
        restarted
            .enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(60))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        assert_eq!(restarted.syms[&BTC].entered_session, Some(today().open));
        assert_eq!(
            store::strategy3::opens_in_session(&pool, DEMO, BTC, today().date)
                .await
                .unwrap(),
            1
        );
    });
    assert!(pause_writes(&pool).await.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn durable_cost_veto_read_failure_does_not_consume_the_mark(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            sqlx::query(
                "ALTER TABLE trading_decisions RENAME TO decisions_temporarily_unavailable",
            )
            .execute(&pool)
            .await
            .unwrap();
            let result = t
                .on_quote(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await;
            sqlx::query(
                "ALTER TABLE decisions_temporarily_unavailable RENAME TO trading_decisions",
            )
            .execute(&pool)
            .await
            .unwrap();
            assert!(result.is_err(), "unreadable veto state must fail closed");
            assert!(orders(&log).is_empty());
            assert_eq!(t.syms[&BTC].last_decision_mark, None);
            assert_eq!(t.syms[&BTC].cost_skip_session, None);
            t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            assert_eq!(
                orders(&log).len(),
                1,
                "the same mark can retry after recovery"
            );
        }
    );
    assert!(pause_writes(&pool).await.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn durable_cost_veto_persistence_failure_never_latches_unstored_state(pool: PgPool) {
    loaded(&pool).await;
    sqlx::query("UPDATE deals SET commission=-60000 WHERE account_id=$1")
        .bind(DEMO)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(
        "CREATE FUNCTION reject_test_cost_veto() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN IF NEW.kind='s3_skip' AND NEW.detail->>'skip_code'='cost_ge_r' THEN
           RAISE EXCEPTION 'test cost receipt unavailable'; END IF; RETURN NEW; END $$;
         CREATE TRIGGER reject_test_cost_veto BEFORE INSERT ON trading_decisions
           FOR EACH ROW EXECUTE FUNCTION reject_test_cost_veto();",
    )
    .execute(&pool)
    .await
    .unwrap();
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            let result = t
                .on_quote(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await;
            assert!(
                result.is_err(),
                "the critical receipt must not be best effort"
            );
            assert!(orders(&log).is_empty());
            assert_eq!(t.syms[&BTC].cost_skip_session, None);
            let persisted: i64 =
                sqlx::query_scalar("SELECT count(*) FROM trading_decisions WHERE kind='s3_skip'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(persisted, 0);
        }
    );
    assert!(pause_writes(&pool).await.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_impulse_waits_for_pullback_then_locks_breakeven_and_full_target(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_912.5), Some(60_917.5), at(30))
            .await
            .unwrap();
        assert!(orders(&log).is_empty(), "peak only arms; no market entry");
        assert!(t.syms[&BTC].pullback.is_some());
        let restarted = strategy3::Trader::new(&pool, t.allowlist, t.env, t.session)
            .await
            .unwrap();
        let mut t = WireTrader {
            inner: restarted,
            quote_sender: t.quote_sender.clone(),
        };
        reconcile_with_scoped_recovery(&mut t, at(30)).await;
        t.on_quote(
            BTC,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
        )
        .await
        .unwrap();
        let o = orders(&log);
        assert_eq!(o.len(), 1, "{:?}", decisions(&pool).await);
        assert_eq!(o[0].relative_stop_loss, Some(51_623_000));
        assert_eq!(o[0].relative_take_profit, Some(71_240_000));
        assert_eq!(o[0].volume, 11);
        let pid = t.own.values().next().unwrap().position_id;
        assert!(t.own[&pid].plan_basis.unwrap().two_stage);
        assert!(!t.own[&pid].partial);
        t.on_quote(BTC, Some(60_978.22), Some(60_983.22), at(31))
            .await
            .unwrap();
        assert!(
            log.lock().unwrap().amends.is_empty(),
            "no stop tightening below +1R"
        );
        t.on_quote(BTC, Some(60_978.23), Some(60_983.23), at(32))
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().amends.len(), 1);
        assert_eq!(t.own[&pid].stop, Some(60_462.0));
        t.on_quote(BTC, Some(61_000.0), Some(61_005.0), at(33))
            .await
            .unwrap();
        assert_eq!(
            log.lock().unwrap().amends.len(),
            1,
            "BE is a stage, not a continuing trail"
        );
        let target = t.own[&pid].take_profit.unwrap();
        assert_eq!(target, 61_174.40);
        let p = f.positions.lock().unwrap()[0].clone();
        t.server_event(&jiaoyi_ctrader::testkit::server_fill(
            DEMO, p, 991, target, 11, true, false,
        ))
        .await
        .unwrap();
        let exits: i64 =
            sqlx::query_scalar("SELECT count(*) FROM strategy3_fills WHERE kind='broker_exit'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(exits, 1);
        assert!(
            log.lock().unwrap().closes.is_empty(),
            "full TP is broker-side; no TP1 partial"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_short_impulse_waits_for_the_fresh_midpoint_retest(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((59_540.0, 59_545.0));
    for b in f.m30_bars.get_mut(&BTC).unwrap() {
        let minute = b.utc_timestamp_in_minutes.unwrap() as i64;
        if minute % 1440 >= 60 {
            *b = m30_bar(
                DateTime::from_timestamp(minute * 60, 0).unwrap(),
                e5(OPEN),
                e5(61_000.0),
                e5(59_000.0),
                e5(59_940.0),
                100,
            );
        }
    }
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(59_082.5), Some(59_087.5), at(30))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
        t.on_quote(
            BTC,
            Some(59_540.0),
            Some(59_545.0),
            at(30) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(orders(&log).len(), 1, "{:?}", decisions(&pool).await);
        assert_eq!(orders(&log)[0].trade_side, 2);
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn production_crypto_first_and_last_half_hour_still_require_a_fresh_retest(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((70_000.0, 70_005.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(29))
                .await
                .unwrap();
            let armed = t.syms[&BTC].pullback.expect("first half-hour can arm");
            t.on_quote(BTC, Some(70_000.0), Some(70_005.0), at(239))
                .await
                .unwrap();
            assert_eq!(
                t.syms[&BTC].pullback,
                Some(armed),
                "last half-hour keeps the valid arm"
            );
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
            t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(60))
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
async fn insufficient_complete_macro_sessions_refuse_an_entry(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_460.0));
    f.m30_bars = HashMap::from([(BTC, bars(10, 13))]);
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
    });
    let d = decisions(&pool).await;
    assert!(
        d.iter().any(|(k, r)| k == "s3_pullback_assessment"
            && r.starts_with("macro_preparation_unavailable")),
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
            t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(60))
                .await
                .unwrap();
            let o = orders(&log);
            assert_eq!(o.len(), 1, "{:?}", decisions(&pool).await);
            assert_eq!(o[0].volume, 1);
            // Every new position uses the full 1.38R broker target.
            assert_eq!(o[0].relative_take_profit, Some(71_240_000));
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
        used_margin: Some(100),
        money_digits: Some(2),
        price: Some(price),
        stop_loss: Some(price - 450.0),
        take_profit: Some(price + 1_372.21),
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
    broker_proves_silent_fill(&pool, &silent, 5001, 13).await;
    let answering = FakeBroker {
        silent_orders: false,
        ..silent.clone()
    };
    with_trader!(&pool, answering, Environment::Demo, |t, log| {
        let o = t.own.get(&5001).cloned().expect("adopted");
        assert!((o.tp1_distance.unwrap() - 922.21).abs() < 1e-9, "{o:?}");
        assert_eq!(o.close_volume, Some(8));
        assert!(o.partial);
        assert_eq!(o.session_date, Some(today().date));
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), at(61))
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
        assert_eq!(closes, vec![8, 5]);
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(60))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let tp1 = at(61);
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), tp1)
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "rejected");
        t.on_quote(
            BTC,
            Some(61_384.21),
            Some(62_118.0),
            tp1 + Duration::seconds(5),
        )
        .await
        .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1, "waits 10 s");
        t.on_quote(
            BTC,
            Some(61_384.21),
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
        assert_eq!(closes, vec![8, 8], "retried for the same 8 cents");
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        assert_eq!(log.lock().unwrap().closes.len(), 1);
    });
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(
            BTC,
            Some(61_384.21),
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        let p = f.positions.lock().unwrap()[0].clone();
        let stop = p.stop_loss.unwrap();
        // The stop (60,012.00) fills 3.00 lower, for all 13 cents; the same
        // event again is the same deal.
        let sl = server_fill(DEMO, p.clone(), 901, stop - 3.0, 13, true, false);
        t.server_event(&sl).await.unwrap();
        t.server_event(&sl).await.unwrap();
        // Another long's take-profit, at 0.50 better than its level.
        let other = s3_position(5002, 8, 60_000.0);
        let tp = other.take_profit.unwrap();
        t.server_event(&server_fill(DEMO, other, 902, tp + 0.5, 8, true, false))
            .await
            .unwrap();
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
        .await
        .unwrap();
        let mut s1 = p.clone();
        s1.trade_data.label = Some("jiaoyi-s1".into());
        t.server_event(&server_fill(DEMO, s1, 904, stop, 13, true, false))
            .await
            .unwrap();
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
    assert!((rows[1].2 - (60_000.0 + 1_372.21)).abs() < 1e-6, "{rows:?}");
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.server_event(&message).await.unwrap();
        t.server_event(&message).await.unwrap();
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
        t.server_event(&correlated).await.unwrap();
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
        .await
        .unwrap();
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
        .await
        .unwrap();
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        assert_eq!(orders(&log)[0].volume, 13);
        let o = t.own.get(&1001).cloned().expect("held");
        assert_eq!(o.volume, 13, "{o:?}");
        assert!((o.entry.unwrap() - vwap).abs() < 1e-6, "{o:?}");
        assert_eq!((o.close_volume, o.partial), (Some(8), true), "{o:?}");
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "sent, no answer");
    });
    silent
        .positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 6, 60_462.0));
    broker_proves_silent_fill(&pool, &silent, 5001, 6).await;
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
    broker_proves_silent_fill(&pool, &silent, 5001, 13).await;
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
            500,
            BTC,
            (
                Some((OPEN * 100_000.0) as u64),
                Some(((OPEN + 1.0) * 100_000.0) as u64),
            ),
        )],
        ..fake((60_455.0, 60_462.0))
    };
    let live = trading::LivenessMap::default();
    // Allow bounded cold M30 preparation and the background account reads to
    // complete before checking actor processing (the observer alone is not enough).
    run_s3(&pool, &f, 8, &live).await;
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

/// Enter at the 30-minute mark, then quote at TP1 (fill 60,462.00 +
/// 922.21): the TP1 closes sent.
async fn tp1_after(pool: &PgPool, f: FakeBroker) -> Vec<i64> {
    let out;
    with_trader!(pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        assert_eq!(orders(&log)[0].volume, 13);
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        out = closes(&log);
    });
    out
}

/// Codex round-4 P2 on #15: a partial fill too small to split (1 of 13
/// cents; the plan's close was 8) takes TP1 as a whole-position close, not
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
/// close for 4: ceil(4 × (R + C) / (TP1 + R)) = ceil(4 × 751.208 / 1,372.21)
/// = 3, a runner of 1 (self-derived from the planner's own formula).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partial_fill_re_plans_its_tp1_close(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 4, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    assert_eq!(tp1_after(&pool, f).await, vec![3]);
}

/// A full fill keeps the planned close of 8 of 13 (a guard: main already
/// does this).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_full_fill_keeps_its_planned_tp1_close(pool: PgPool) {
    loaded(&pool).await;
    assert_eq!(tp1_after(&pool, fake((60_455.0, 60_462.0))).await, vec![8]);
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
    });
    silent
        .positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 13, 60_462.0));
    broker_proves_silent_fill(&pool, &silent, 5001, 13).await;
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
        t.server_event(&close).await.unwrap();
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
    other.rule =
        jiaoyi_core::strategy3::production_sessions::rule_for("ETHUSD", AssetClass::Crypto);
    if let Some(Ok(day)) = other.day.as_mut() {
        // This synthetic second-symbol fixture shares the first-slot statistics
        // but must not inherit a historical position's 24-hour calendar.
        day.session = today();
    }
    other.last_try = None;
    other.last_decision_mark = None;
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let other = second_prepared_crypto(&mut t);
        t.reconcile(Utc::now()).await.unwrap();
        t.enter_pullback(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "an account funding rejection suppresses other entry attempts"
        );
        t.enter_pullback(
            other,
            Some(60_500.0),
            Some(60_505.0),
            at(60) + Duration::seconds(1),
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let other = second_prepared_crypto(&mut t);
        t.reconcile(Utc::now()).await.unwrap();
        t.enter_pullback(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
    });
    f.reject_orders = Some("NOT_ENOUGH_MONEY".into());
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.prepare(BTC, today(), at(30)).await.unwrap();
        let other = second_prepared_crypto(&mut t);
        t.reconcile(Utc::now()).await.unwrap();
        t.enter_pullback(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
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
            Some(61_384.21),
            Some(62_118.0),
            at(30) + Duration::seconds(2),
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let stop = p.stop_loss.unwrap();
        t.server_event(&server_fill(DEMO, p, 901, stop - 3.0, 13, true, false))
            .await
            .unwrap();
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
            position_id, order_id, deal_id, symbol_id, side, volume, raw, received_at)
         SELECT $1, $2 || ':' || n, 'c-' || $2 || n, 2126, 'ORDER_FILLED', 900000 + n,
                9000000 + 2 * n, 9000000 + 2 * n, 99, 'buy', 10, '\\x',
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
            close_pnl_conversion_fee, raw, fetched_at)
         SELECT $1, 9000000 + 2 * n + c, 9000000 + 2 * n + c, 900000 + n, 99, 10, 10,
                CASE WHEN c = 1 THEN 'SELL' ELSE 'BUY' END, 'FILLED', t, t, 2, c = 1,
                CASE WHEN c = 1 THEN -560 END, CASE WHEN c = 1 THEN 0 END,
                CASE WHEN c = 1 THEN -40 END, CASE WHEN c = 1 THEN 2 END,
                CASE WHEN c = 1 THEN 0 END, '\\x', now()
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
        // A paused mark is consumed once. Lifting the owner pause allows
        // the next mark; it does not reconsider the earlier decision.
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        t.enter_pullback(
            BTC,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert!(orders(&log).is_empty(), "paused: nothing sent");
        let d = decisions(&pool).await;
        let paused: Vec<&(String, String)> = d.iter().filter(|(k, _)| k == "s3_paused").collect();
        assert_eq!(paused.len(), 1, "recorded once: {d:?}");
        assert!(paused[0].1.contains("paused by the owner"), "{d:?}");
        // Lifted by the owner: the next 30-minute mark enters.
        owner_sets_pause(&pool, 0).await;
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(60))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "{:?}", decisions(&pool).await);
        assert_eq!(orders(&log)[0].volume, 11);
        // Paused again: the position keeps its exits.
        owner_sets_pause(&pool, 1).await;
        t.enter_pullback(BTC, Some(61_384.21), Some(62_118.0), at(61))
            .await
            .unwrap();
        assert!(closes(&log).is_empty(), "new policy has no partial TP1");
        assert_eq!(
            log.lock().unwrap().amends.len(),
            1,
            "breakeven remains active while paused"
        );
        t.reconcile(Utc::now()).await.unwrap();
        t.session_end(at(24 * 60)).await.unwrap();
        assert_eq!(closes(&log), vec![11], "full session end while paused");
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
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.on_quote(BTC, Some(60_435.0), Some(60_440.0), at(30))
                .await
                .unwrap();
            let inside = strategy3::trigger_distance(&t.syms[&BTC], at(30));
            assert!((inside["upper"].as_f64().unwrap() - 60_450.0).abs() < 1e-6);
            assert!((inside["distance_half_widths"].as_f64().unwrap() - 10.0 / 450.0).abs() < 1e-9);
            t.on_quote(BTC, Some(60_455.0), Some(60_460.0), at(60))
                .await
                .unwrap();
            let armed = strategy3::trigger_distance(&t.syms[&BTC], at(60));
            assert_eq!(armed["state"], "armed_pullback");
            assert_eq!(armed["pullback_entry_ref"], 60_228.75);
            assert_eq!(armed["distance_half_widths"], 0.5);
            assert!(
                orders(&log).is_empty(),
                "band crossing displays pullback waiting, never immediate entry"
            );
            t.on_quote(BTC, Some(59_995.0), Some(60_000.0), at(61))
                .await
                .unwrap();
            assert!(
                t.syms[&BTC].pullback.is_none(),
                "origin crossing invalidates the arm"
            );
            let first_half_hour = strategy3::trigger_distance(&t.syms[&BTC], at(29));
            assert!(
                first_half_hour["distance_half_widths"].is_number(),
                "production has no first-30m blackout"
            );
            let next_window = strategy3::trigger_distance(&t.syms[&BTC], at(240));
            assert!(next_window["distance_half_widths"].is_null());
            assert_eq!(next_window["reason"], "today's session is not prepared yet");
            let live = t.live.lock().unwrap().clone();
            let h = strategy3::health(&pool, &live).await.unwrap();
            assert_eq!(
                h[0]["trigger_distance"]["BTCUSD"],
                live[&DEMO].triggers["BTCUSD"]
            );
        }
    );
}

// Codex · № 17,217 26-Oct: observed 02:17Z broker closes carry correlation.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn correlated_broker_close_is_recorded_once_after_reconcile(pool: PgPool) {
    use jiaoyi_ctrader::testkit::server_fill;
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.server_event(&message).await.unwrap();
        t.server_event(&message).await.unwrap();
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
        t.server_event(&foreign_position).await.unwrap();
        let mut foreign_account =
            server_fill(DEMO + 1, p.clone(), 99994, 60_000.0, 13, true, false);
        foreign_account.client_msg_id = message.client_msg_id.clone();
        t.server_event(&foreign_account).await.unwrap();
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), at(61))
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
        t.server_event(&message).await.unwrap();
        t.server_event(&message).await.unwrap();
        let counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT kind,count(*) FROM strategy3_fills WHERE kind NOT IN ('open','trail') GROUP BY kind",
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
        t.server_event(&collision).await.unwrap();
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        // A TP1 filled by the existing gate already has its exact broker deal.
        t.on_quote(BTC, Some(61_384.21), Some(62_118.0), at(61))
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
            execution_price: Some(61_384.21),
            execution_timestamp: Utc::now().timestamp_millis(),
            close_position_detail: Some(ProtoOaClosePositionDetail {
                entry_price: p.price.unwrap(),
                closed_volume: Some(8),
                ..Default::default()
            }),
            ..Default::default()
        };
        store::history::upsert_deals(&pool, DEMO, &[history::deal_row(&own_close)], Utc::now())
            .await
            .unwrap();
        // The remaining 5 cents close at the broker. A stale/fresh position
        // snapshot that still contains the position must defer recovery.
        own_close.deal_id = 99981;
        own_close.order_id = 99982;
        own_close.volume = 5;
        own_close.filled_volume = 5;
        own_close
            .close_position_detail
            .as_mut()
            .unwrap()
            .closed_volume = Some(5);
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
            "SELECT kind,volume FROM strategy3_fills WHERE kind NOT IN ('open','trail') ORDER BY kind",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            volumes,
            vec![("broker_exit".into(), Some(5)), ("tp1".into(), Some(8))],
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
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
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
                61_384.21,
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
                    fill_price: Some(61_384.21),
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
        t.server_event(&wire).await.unwrap();
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
                61_384.21,
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

/// The owner's deadline is before the open, even with no incoming quotes.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn crypto_four_hour_preopen_is_ready_then_finalises_causally(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    let mut full = Vec::new();
    for day in 1..=20 {
        for mark in 0..48 {
            full.push(m30_bar(
                today().open - Duration::days(day) + Duration::minutes(mark * 30),
                e5(OPEN),
                e5(61_000.0),
                e5(59_000.0),
                e5(OPEN * 1.0075),
                100,
            ));
        }
    }
    full.push(m30_bar(
        today().open,
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        10,
    ));
    f.m30_bars.insert(BTC, full);
    let (quote_sender, _) = tokio::sync::broadcast::channel(64);
    f.spot_events = Some(quote_sender.clone());
    let (conn, log) = f.spawn();
    let session = loader::authorise(&pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let list = allowlist();
    let live = trading::LivenessMap::default();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let before_open = today().open - Duration::minutes(19);
    let offset = before_open - Utc::now();
    let controller = async {
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        trigger.fire();
    };
    let (result, _) = tokio::join!(
        strategy3::run_session_with_event_offset(
            &pool,
            &list,
            Environment::Demo,
            &session,
            &live,
            &mut ends,
            &mut stop,
            offset
        ),
        controller,
    );
    result.unwrap();
    let prepared: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trading_decisions WHERE account_id=$1 AND kind='s3_preopen'",
    )
    .bind(DEMO)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        prepared > 0,
        "historical setup must complete before open, without waiting for an entry quote"
    );
    let row: serde_json::Value = sqlx::query_scalar(
        "SELECT detail FROM trading_decisions WHERE kind='s3_preopen' AND symbol='BTCUSD' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let prepared: strategy3::preopen::Prepared = serde_json::from_value(row).unwrap();
    assert!(prepared.ready() && prepared.on_time());
    assert!(prepared.table.is_some() && prepared.macro_context.is_some());
    assert_eq!(prepared.session, today());
    assert!(prepared.completed_at < today().open - Duration::minutes(10));
    let health = strategy3::preopen::health(&pool, DEMO, before_open + Duration::seconds(10))
        .await
        .unwrap();
    assert!(health["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["symbol"] == "BTCUSD" && s["state"] == "prepared_on_time"));
    {
        let l = log.lock().unwrap();
        assert!(l.orders.is_empty() && l.amends.is_empty() && l.closes.is_empty());
    }
    let requests = log.lock().unwrap().trendbar_requests.len();
    // Reconnect after the deadline retains the original receipt and does
    // not repeat the historical download or misreport preparation as late.
    let list2 = allowlist();
    let restored = strategy3::Trader::new(&pool, &list2, Environment::Demo, &session)
        .await
        .unwrap();
    let mut restored = WireTrader {
        inner: restored,
        quote_sender,
    };
    // Reconnect is simulated at the first four-hour opening. Restore using
    // that clock, not the wall clock (which may already be in a later slot).
    strategy3::preopen::restore(&pool, DEMO, &mut restored.syms, today().open)
        .await
        .unwrap();
    assert!(
        restored.syms[&BTC]
            .preopen
            .as_ref()
            .is_some_and(|p| p.completed_at == prepared.completed_at),
        "reconnect reloads the durable pre-open receipt"
    );
    restored.reconcile(Utc::now()).await.unwrap();
    restored
        .enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
        .await
        .unwrap();
    {
        let wire = log.lock().unwrap();
        let finalisation = &wire.trendbar_requests[requests..];
        assert_eq!(
            finalisation.len(),
            1,
            "only the current opening bar is needed; prior same-slot context is complete"
        );
        assert_eq!(
            finalisation.iter().map(|r| r.1).min(),
            Some(today().open.timestamp_millis())
        );
        assert!(finalisation
            .iter()
            .all(|r| r.0 == BTC && r.2 <= at(30).timestamp_millis()));
    }
    assert_eq!(
        log.lock().unwrap().orders.len(),
        1,
        "registered entry still works with the prepared statistics"
    );
    // The separate pre-open worker itself sent no trading frames.
    // The on-quote assertion above exercises the unchanged registered entry.
    let log = log.lock().unwrap();
    assert!(log.amends.is_empty() && log.closes.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_preopen_history_wait_does_not_starve_an_owned_tp1(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(t.own.len(), 1);
    });
    f.trendbar_reply_delays
        .lock()
        .unwrap()
        .push_back(std::time::Duration::from_secs(10));
    f.quote_sequence = vec![(
        500,
        BTC,
        (Some(e5(61_384.21) as u64), Some(e5(62_118.0) as u64)),
    )];
    let (conn, log) = f.spawn();
    let session = loader::authorise(&pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let list = allowlist();
    let live = trading::LivenessMap::default();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let offset = today().close - Duration::minutes(19) - Utc::now();
    let close = async {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        trigger.fire();
    };
    let (result, _) = tokio::join!(
        strategy3::run_session_with_event_offset(
            &pool,
            &list,
            Environment::Demo,
            &session,
            &live,
            &mut ends,
            &mut stop,
            offset
        ),
        close
    );
    result.unwrap();
    let l = log.lock().unwrap();
    assert!(
        !l.trendbar_requests.is_empty(),
        "pre-open historical read started"
    );
    assert_eq!(
        l.closes.len(),
        1,
        "TP1 executes while the preparation reply is still held"
    );
    assert_eq!(l.closes[0].volume, 8);
    assert!(l.orders.is_empty() && l.amends.is_empty());
}

// Regression of the three actor-to-gate failures demonstrated by the audit.
async fn changed_lifecycle_is_not_managed(
    pool: PgPool,
    reverse: bool,
    enlarge: bool,
    restart: bool,
) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "valid owned opening control");
        let original = t.own.values().next().unwrap().clone();
        assert_eq!(original.volume, 13);
        {
            let mut positions = f.positions.lock().unwrap();
            let p = positions
                .iter_mut()
                .find(|p| p.position_id == original.position_id)
                .unwrap();
            if reverse {
                p.trade_data.trade_side = 2;
            }
            if enlarge {
                p.trade_data.volume = 26;
            }
        }
        if !restart {
            // The stale actor itself must not bypass the final shared gate.
            t.session_end(at(24 * 60)).await.unwrap();
            assert!(log.lock().unwrap().closes.is_empty());
            t.reconcile(Utc::now()).await.unwrap();
            assert!(!t.own.contains_key(&original.position_id));
            t.session_end(at(24 * 60 + 1)).await.unwrap();
            assert!(log.lock().unwrap().closes.is_empty());
        }
    });
    if restart {
        with_trader!(&pool, f, Environment::Demo, |t, log| {
            assert!(
                t.own.is_empty(),
                "a new actor must not inherit the old plan"
            );
            t.session_end(at(24 * 60)).await.unwrap();
            assert!(log.lock().unwrap().closes.is_empty());
        });
    }
    assert_eq!(
        f.positions.lock().unwrap().len(),
        1,
        "external book untouched"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn external_same_id_reversal_is_not_closed(pool: PgPool) {
    changed_lifecycle_is_not_managed(pool, true, false, false).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn external_same_id_enlargement_is_not_closed(pool: PgPool) {
    changed_lifecycle_is_not_managed(pool, false, true, false).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn external_same_id_reversal_after_restart_is_not_closed(pool: PgPool) {
    changed_lifecycle_is_not_managed(pool, true, true, true).await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn external_increase_within_unfilled_request_is_not_owned(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        fill_parts: vec![(0, 6, 60_462.0)],
        cancel_remainder: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let o = t.own.values().next().unwrap().clone();
        assert_eq!(o.volume, 6);
        assert_eq!(orders(&log)[0].volume, 13);
        f.positions.lock().unwrap()[0].trade_data.volume = 9;
        t.session_end(at(24 * 60)).await.unwrap();
        assert!(
            log.lock().unwrap().closes.is_empty(),
            "9 is below requested13 but above proved6"
        );
        t.reconcile(Utc::now()).await.unwrap();
        assert!(t.own.is_empty());
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn same_side_same_size_external_reopen_retains_raw_evidence_and_refuses_exit(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        let mut close = jiaoyi_ctrader::testkit::server_fill(
            DEMO,
            p.clone(),
            900001,
            60_500.0,
            13,
            true,
            false,
        );
        close.client_msg_id = Some("external-close".into());
        t.server_event(&close).await.unwrap();
        let mut reopen =
            jiaoyi_ctrader::testkit::server_fill(DEMO, p, 900002, 60_500.0, 13, false, false);
        reopen.client_msg_id = Some("external-reopen".into());
        t.server_event(&reopen).await.unwrap();
        let raw: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_events WHERE account_id=$1 AND deal_id IN (900001,900002) AND octet_length(raw)>0")
            .bind(DEMO).fetch_one(&pool).await.unwrap();
        assert_eq!(
            raw, 2,
            "foreign correlation cannot discard tracked lifecycle evidence"
        );
        t.session_end(at(24 * 60)).await.unwrap();
        assert!(log.lock().unwrap().closes.is_empty());
        t.reconcile(Utc::now()).await.unwrap();
        assert!(t.own.is_empty());
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unconfirmed_label_only_match_remains_unmanaged(pool: PgPool) {
    loaded(&pool).await;
    settings::change(
        &pool,
        "trading.order_wait_secs",
        Some(DEMO),
        Some(json!(1)),
        "test",
        "bounded wait",
    )
    .await
    .unwrap();
    let f = FakeBroker {
        silent_orders: true,
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
    });
    f.positions
        .lock()
        .unwrap()
        .push(s3_position(5001, 13, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        assert!(t.own.is_empty());
        t.session_end(at(24 * 60)).await.unwrap();
        assert!(log.lock().unwrap().closes.is_empty());
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn proven_external_reduction_refreshes_the_registered_close_retry(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let mut p = f.positions.lock().unwrap()[0].clone();
        p.trade_data.volume = 5;
        f.positions.lock().unwrap()[0] = p.clone();
        let reduction =
            jiaoyi_ctrader::testkit::server_fill(DEMO, p, 900003, 60_500.0, 8, true, false);
        t.server_event(&reduction).await.unwrap();
        let end = at(24 * 60);
        t.session_end(end).await.unwrap();
        assert!(
            log.lock().unwrap().closes.is_empty(),
            "stale13-unit request is refused"
        );
        assert_eq!(
            t.own.values().next().unwrap().volume,
            5,
            "proved residual is refreshed promptly"
        );
        t.session_end(end + Duration::seconds(11)).await.unwrap();
        assert_eq!(
            log.lock()
                .unwrap()
                .closes
                .iter()
                .map(|c| c.volume)
                .collect::<Vec<_>>(),
            vec![5]
        );
        assert!(f.positions.lock().unwrap().is_empty());
    });
}

async fn quantity_persistence_failure_ends_s3_actor(pool: PgPool, during_read: bool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(t.own.len(), 1);
    });
    let original = f.positions.lock().unwrap()[0].clone();
    if during_read {
        let other = BTC + 1;
        settings::change(
            &pool,
            "universe.symbols",
            Some(DEMO),
            Some(json!([
                {"name":"BTCUSD","class":"crypto"},{"name":"ETHUSD","class":"crypto"}
            ])),
            "test",
            "exercise the inner historical-read execution pump",
        )
        .await
        .unwrap();
        let run = store::start_run(&pool, DEMO).await.unwrap();
        for (id, name) in [(BTC, "BTCUSD"), (other, "ETHUSD")] {
            let symbol = ProtoOaSymbol {
                symbol_id: id,
                digits: 2,
                pip_position: 2,
                min_volume: Some(1),
                step_volume: Some(1),
                ..Default::default()
            };
            store::upsert_symbol(
                &pool,
                &loader::symbol_row(
                    DEMO,
                    name,
                    AssetClass::Crypto,
                    Some(true),
                    &symbol,
                    Utc::now(),
                ),
            )
            .await
            .unwrap();
            store::record_resolution(
                &pool,
                run,
                DEMO,
                name,
                AssetClass::Crypto,
                &Resolution::Resolved { symbol_id: id },
            )
            .await
            .unwrap();
        }
        store::finish_run(&pool,run,"ok",&store::RunSummary {
            assets:Some(json!([{"asset_id":1,"name":"USD"},{"asset_id":9,"name":"BTC"}])),
            listed_symbols:Some(json!([{"id":BTC,"quote_asset_id":1,"base_asset_id":9},{"id":other,"quote_asset_id":1,"base_asset_id":9}])),
            ..Default::default()
        }).await.unwrap();
        f.m30_bars.insert(other, bars(10, 20));
        f.trendbar_reply_delays
            .lock()
            .unwrap()
            .push_back(std::time::Duration::from_secs(10));
        f.quote_sequence.push((
            0,
            other,
            (Some(e5(60_455.0) as u64), Some(e5(60_460.0) as u64)),
        ));
    }
    f.quote_sequence.push((
        if during_read { 1800 } else { 800 },
        BTC,
        (Some(e5(61_384.21) as u64), Some(e5(61_389.0) as u64)),
    ));
    let event = jiaoyi_ctrader::testkit::server_fill(
        DEMO,
        original.clone(),
        990001,
        60_400.0,
        13,
        true,
        false,
    );
    f.execution_sequence = vec![(
        if during_read { 1000 } else { 200 },
        event.payload_type,
        event.payload,
    )];
    sqlx::raw_sql("CREATE FUNCTION reject_quantity_receipt() RETURNS trigger LANGUAGE plpgsql AS $$
        BEGIN IF NEW.deal_id=990001 THEN RAISE EXCEPTION 'authority raw receipt blocked'; END IF; RETURN NEW; END $$;
        CREATE TRIGGER reject_quantity_receipt BEFORE INSERT ON trading_events FOR EACH ROW EXECUTE FUNCTION reject_quantity_receipt();")
        .execute(&pool).await.unwrap();
    let (conn, log) = f.clone().spawn();
    let session = loader::authorise(&pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let list = allowlist();
    let live = trading::LivenessMap::default();
    let mut ends = conn.session_ends();
    let (_trigger, mut stop) = shutdown::channel();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        strategy3::run_session_with_event_offset(
            &pool,
            &list,
            Environment::Demo,
            &session,
            &live,
            &mut ends,
            &mut stop,
            at(30) - Utc::now(),
        ),
    )
    .await
    .expect("actor must terminate on lost quantity evidence");
    let error = result.expect_err("raw persistence failure cannot leave the actor running");
    assert!(
        format!("{error:#}").contains("authority raw receipt blocked"),
        "{error:#}"
    );
    let log = log.lock().unwrap();
    assert!(
        log.closes.is_empty() && log.amends.is_empty(),
        "no later management after lost evidence"
    );
    if during_read {
        assert!(
            (1..=2).contains(&log.trendbar_requests.len()),
            "fatal receipt failure interrupts entry and concurrent warm-up before further pages"
        );
    }
    let positions = f.positions.lock().unwrap();
    assert_eq!(
        (positions[0].stop_loss, positions[0].take_profit),
        (original.stop_loss, original.take_profit)
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn lost_raw_quantity_receipt_ends_s3_actor(pool: PgPool) {
    quantity_persistence_failure_ends_s3_actor(pool, false).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn lost_raw_quantity_receipt_ends_s3_inner_read_pump(pool: PgPool) {
    quantity_persistence_failure_ends_s3_actor(pool, true).await;
}
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn historical_authority_lookup_failure_is_not_external_evidence(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, _log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let p = f.positions.lock().unwrap()[0].clone();
        t.own.clear();
        let event =
            jiaoyi_ctrader::testkit::server_fill(DEMO, p, 990002, 60_400.0, 13, true, false);
        pool.close().await;
        assert!(
            t.server_event(&event).await.is_err(),
            "unreadable ownership is not a successful external-message skip"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn observed_absence_cannot_reuse_the_old_plan_on_same_shape_reappearance(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let original = f.positions.lock().unwrap()[0].clone();
        assert!(t.own.contains_key(&original.position_id));
        f.positions.lock().unwrap().clear();
        t.reconcile(Utc::now()).await.unwrap();
        assert!(t.own.is_empty());
        // The broker has reused the same shape and ID, but history still
        // contains only the original owned opening. Absence is not a cash fill.
        f.positions.lock().unwrap().push(original.clone());
        assert!(
            jiaoyi::authority::recover_position(&pool, t.session, &original)
                .await
                .is_err(),
            "ordinary old-history recovery cannot clear observed lifecycle disappearance"
        );
        t.reconcile(Utc::now()).await.unwrap();
        assert!(t.own.is_empty());
        t.session_end(at(24 * 60)).await.unwrap();
        {
            let sent = log.lock().unwrap();
            assert!(sent.closes.is_empty() && sent.amends.is_empty());
        }
        let markers: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_events WHERE account_id=$1 AND position_id=$2 AND kind='AUTHORITY_DISAPPEARED'")
            .bind(DEMO).bind(original.position_id).fetch_one(&pool).await.unwrap();
        assert_eq!(markers, 1);
        let closed: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM deals WHERE account_id=$1 AND position_id=$2 AND is_closing",
        )
        .bind(DEMO)
        .bind(original.position_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            closed, 0,
            "absence cannot fabricate financial closing evidence"
        );
        {
            let positions = f.positions.lock().unwrap();
            assert_eq!(
                (positions[0].stop_loss, positions[0].take_profit),
                (original.stop_loss, original.take_profit)
            );
        }
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unqualified_position_cannot_free_demo_budget_for_another_symbol(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1);
        f.positions.lock().unwrap()[0].trade_data.volume *= 2;
        t.reconcile(Utc::now()).await.unwrap();
        assert!(t.own.is_empty(), "unproved quantity is not managed");
        let other = second_prepared_crypto(&mut t);
        t.syms.get_mut(&other).unwrap().entered_session = None;
        t.enter_pullback(
            other,
            Some(60_455.0),
            Some(60_460.0),
            at(30) + Duration::seconds(1),
        )
        .await
        .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "quarantined exposure cannot turn into zero risk"
        );
        let decisions = decisions(&pool).await;
        assert!(
            decisions
                .iter()
                .any(|(kind, why)| kind == "s3_skip" && why.contains("await management authority")),
            "{decisions:?}"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn lost_trail_raw_receipt_is_fatal_to_s3_management(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        sqlx::raw_sql("CREATE FUNCTION reject_trail_receipt() RETURNS trigger LANGUAGE plpgsql AS $$
            BEGIN IF NEW.kind='ORDER_REPLACED' THEN RAISE EXCEPTION 'trail raw receipt blocked'; END IF; RETURN NEW; END $$;
            CREATE TRIGGER reject_trail_receipt BEFORE INSERT ON trading_events FOR EACH ROW EXECUTE FUNCTION reject_trail_receipt();")
            .execute(&pool).await.unwrap();
        let error = t
            .on_quote(BTC, Some(60_700.0), Some(60_705.0), at(31))
            .await
            .expect_err("receipt loss after amend must end management");
        assert!(
            format!("{error:#}").contains("trail raw receipt blocked"),
            "{error:#}"
        );
        assert_eq!(
            log.lock().unwrap().amends.len(),
            1,
            "the uncertain amendment is not resent"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn failure_to_persist_shared_gate_absence_is_fatal_to_s3_trailing(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        sqlx::raw_sql("CREATE FUNCTION reject_absence_marker() RETURNS trigger LANGUAGE plpgsql AS $$
            BEGIN IF NEW.kind='AUTHORITY_DISAPPEARED' THEN RAISE EXCEPTION 'absence marker blocked'; END IF; RETURN NEW; END $$;
            CREATE TRIGGER reject_absence_marker BEFORE INSERT ON trading_events FOR EACH ROW EXECUTE FUNCTION reject_absence_marker();")
            .execute(&pool).await.unwrap();
        f.positions.lock().unwrap().clear();
        let error = t
            .on_quote(BTC, Some(60_700.0), Some(60_705.0), at(31))
            .await
            .expect_err("shared gate absence must be persisted, or management ends");
        assert!(
            format!("{error:#}").contains("absence marker blocked"),
            "{error:#}"
        );
        assert!(
            log.lock().unwrap().amends.is_empty(),
            "no write follows unrecorded absence"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn all_retired_symbols_are_inactive_before_preparation_or_dispatch(pool: PgPool) {
    loaded(&pool).await;
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            for name in [
                "NatGas", "AUDUSD", "NZDUSD", "GBPUSD", "USDJPY", "EURUSD", "USDCAD", "AUS200",
                "JPN225", "UK100", "GER40",
            ] {
                t.syms.get_mut(&BTC).unwrap().name = name.into();
                t.on_quote(BTC, Some(60_912.5), Some(60_917.5), at(30))
                    .await
                    .unwrap();
                assert!(t.syms[&BTC].pullback.is_none());
                assert_eq!(
                    strategy3::trigger_distance(&t.syms[&BTC], at(30))["reason_code"],
                    "symbol_retired_from_s3_core"
                );
            }
            assert!(orders(&log).is_empty());
            assert!(log.lock().unwrap().trendbar_requests.is_empty());
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn counter_trend_macro_context_vetoes_the_impulse_before_an_order(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake((60_455.0, 60_462.0));
    for b in f.m30_bars.get_mut(&BTC).unwrap() {
        let minute = b.utc_timestamp_in_minutes.unwrap() as i64;
        if minute % 1440 >= 60 {
            *b = m30_bar(
                DateTime::from_timestamp(minute * 60, 0).unwrap(),
                e5(OPEN),
                e5(61_000.0),
                e5(59_000.0),
                e5(59_940.0),
                100,
            );
        }
    }
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert!(orders(&log).is_empty());
        assert!(t.syms[&BTC].pullback.is_none());
        assert!(decisions(&pool)
            .await
            .iter()
            .any(|(_, r)| r.contains("vetoed_by_5d_20d_macro_trend")));
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn consumed_pullback_is_not_replayed_after_rejected_order_and_restart(pool: PgPool) {
    loaded(&pool).await;
    let f = FakeBroker {
        reject_orders: Some("MARKET_CLOSED".into()),
        ..fake((60_455.0, 60_462.0))
    };
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert_eq!(orders(&log).len(), 1, "{:?}", decisions(&pool).await);
        let inner = strategy3::Trader::new(&pool, t.allowlist, t.env, t.session)
            .await
            .unwrap();
        let mut restarted = WireTrader {
            inner,
            quote_sender: t.quote_sender.clone(),
        };
        restarted
            .on_quote(
                BTC,
                Some(60_455.0),
                Some(60_460.0),
                at(30) + Duration::seconds(2),
            )
            .await
            .unwrap();
        assert_eq!(
            orders(&log).len(),
            1,
            "consumed arm cannot dispatch a second order"
        );
        assert!(restarted.syms[&BTC].pullback.is_none());
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_retired_historical_symbol_outside_settings_keeps_legacy_exits(pool: PgPool) {
    loaded(&pool).await;
    const RETIRED: i64 = 23;
    let metadata = ProtoOaSymbol {
        symbol_id: RETIRED,
        digits: 2,
        pip_position: 2,
        min_volume: Some(1),
        step_volume: Some(1),
        ..Default::default()
    };
    store::upsert_symbol(
        &pool,
        &loader::symbol_row(
            DEMO,
            "SOLUSD",
            AssetClass::Crypto,
            Some(true),
            &metadata,
            Utc::now(),
        ),
    )
    .await
    .unwrap();
    let run = store::start_run(&pool, DEMO).await.unwrap();
    store::record_resolution(
        &pool,
        run,
        DEMO,
        "SOLUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: RETIRED },
    )
    .await
    .unwrap();
    store::finish_run(
        &pool,
        run,
        "ok",
        &store::RunSummary {
            assets: Some(json!([{"asset_id":1,"name":"USD"}])),
            listed_symbols: Some(
                json!([{"id":BTC,"quote_asset_id":1},{"id":RETIRED,"quote_asset_id":1}]),
            ),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut f = fake((60_455.0, 60_462.0));
    f.m30_bars.insert(RETIRED, bars(10, 20));
    f.fill_quotes.insert(RETIRED, (60_455.0, 60_462.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        assert_eq!(
            t.syms[&RETIRED].name, "SOLUSD",
            "historical resolution survives configured universe removal"
        );
        t.on_quote(RETIRED, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        assert!(
            orders(&log).is_empty(),
            "retired data subscription cannot open entries"
        );
        t.legacy_position(RETIRED, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        t.on_quote(RETIRED, Some(61_384.21), Some(61_389.0), at(61))
            .await
            .unwrap();
        assert_eq!(closes(&log), vec![8]);
        t.reconcile(Utc::now()).await.unwrap();
        t.session_end(at(1440)).await.unwrap();
        assert_eq!(
            closes(&log),
            vec![8, 5],
            "old position retains partial and session-close policy"
        );
        assert_eq!(
            orders(&log).len(),
            1,
            "only historical fixture order exists"
        );
    });
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_bid_m30_open_cannot_replace_missing_two_sided_opening_evidence(pool: PgPool) {
    loaded(&pool).await;
    sqlx::query("DELETE FROM ticks WHERE session_id='opening-mid-fixture'")
        .execute(&pool)
        .await
        .unwrap();
    with_trader!(
        &pool,
        fake((60_455.0, 60_462.0)),
        Environment::Demo,
        |t, log| {
            t.enter_pullback(BTC, Some(60_455.0), Some(60_460.0), at(30))
                .await
                .unwrap();
            assert!(orders(&log).is_empty());
            assert!(t.syms[&BTC].pullback.is_none());
            assert!(t.syms[&BTC]
                .day
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .open
                .is_err());
        }
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn reconcile_adopts_and_subscribes_late_core_resolution_without_replacing_held_state(
    pool: PgPool,
) {
    loaded(&pool).await;
    const ETH: i64 = 23;
    let mut f = fake((60_455.0, 60_462.0));
    f.quote_sequence = vec![(0, ETH, (Some(e5(3_000.0) as u64), Some(e5(3_001.0) as u64)))];
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        assert!(
            !t.syms.contains_key(&ETH),
            "loader has not resolved ETH yet"
        );
        t.legacy_position(BTC, Some(60_455.0), Some(60_460.0), at(30))
            .await
            .unwrap();
        let pid = t.own.values().next().unwrap().position_id;
        let before = t.own[&pid].clone();
        let prepared = t.syms[&BTC]
            .day
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .prepared_at;
        let metadata = ProtoOaSymbol {
            symbol_id: ETH,
            digits: 2,
            pip_position: 2,
            min_volume: Some(1),
            step_volume: Some(1),
            ..Default::default()
        };
        store::upsert_symbol(
            &pool,
            &loader::symbol_row(
                DEMO,
                "ETHUSD",
                AssetClass::Crypto,
                Some(true),
                &metadata,
                Utc::now(),
            ),
        )
        .await
        .unwrap();
        let run = store::start_run(&pool, DEMO).await.unwrap();
        store::record_resolution(
            &pool,
            run,
            DEMO,
            "ETHUSD",
            AssetClass::Crypto,
            &Resolution::Resolved { symbol_id: ETH },
        )
        .await
        .unwrap();
        store::finish_run(
            &pool,
            run,
            "ok",
            &store::RunSummary {
                assets: Some(json!([{"asset_id":1,"name":"USD"}])),
                listed_symbols: Some(
                    json!([{"id":BTC,"quote_asset_id":1},{"id":ETH,"quote_asset_id":1}]),
                ),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let mut quotes = t.session.connection().spot_events();
        t.reconcile(Utc::now()).await.unwrap();
        assert_eq!(t.syms[&ETH].name, "ETHUSD");
        assert!(t.syms[&ETH].limits.is_some());
        assert_eq!(t.syms[&ETH].quote.as_deref(), Some("USD"));
        let observed = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let q = quotes.recv().await.unwrap();
                if q.event.symbol_id == ETH {
                    break q;
                }
            }
        })
        .await
        .expect("late resolved core symbol must be subscribed on this connection");
        assert_eq!(observed.event.bid, Some(e5(3_000.0) as u64));
        let after = &t.own[&pid];
        assert_eq!(after.volume, before.volume);
        assert_eq!(after.plan_basis, before.plan_basis);
        assert_eq!(after.stop, before.stop);
        assert_eq!(after.take_profit, before.take_profit);
        assert_eq!(
            t.syms[&BTC]
                .day
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap()
                .prepared_at,
            prepared
        );
        assert_eq!(
            orders(&log).len(),
            1,
            "only the historical opening fixture; refresh sends no order"
        );
    });
}

#[test]
fn human_laws_are_exact_account_local_and_release_when_the_book_clears() {
    use jiaoyi_ctrader::messages::{ProtoOaOrder, ProtoOaReconcileRes};
    assert_eq!(strategy3::LABEL, "jiaoyi-s3");
    assert!(strategy3::LABEL
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-'));
    for name in ["NatGas", "JPN225", "natgas", "jpn225"] {
        assert!(strategy3::human_reserved(name));
    }
    assert!(!strategy3::human_reserved("BTCUSD"));
    let mut book = ProtoOaReconcileRes::default();
    for (id, label) in [
        (1, None),
        (2, Some("")),
        (3, Some("jiaoyi-s1")),
        (4, Some("jiaoyi-s3")),
        (5, Some("jiaoyi-s3:")),
    ] {
        book.position.push(ProtoOaPosition {
            trade_data: ProtoOaTradeData {
                symbol_id: id,
                label: label.map(str::to_owned),
                ..Default::default()
            },
            ..Default::default()
        });
    }
    book.order.push(ProtoOaOrder {
        trade_data: ProtoOaTradeData {
            symbol_id: 6,
            label: None,
            ..Default::default()
        },
        ..Default::default()
    });
    let locks = strategy3::human_symbol_locks(&book);
    assert_eq!(locks.len(), 5);
    assert!(!locks.contains(&4));
    assert!(locks.contains(&6));
    assert!(strategy3::human_symbol_locks(&ProtoOaReconcileRes::default()).is_empty());
    book.position.clear();
    book.order.clear();
    assert!(strategy3::human_symbol_locks(&book).is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn external_position_cancels_armed_setup_and_release_requires_a_new_impulse(pool: PgPool) {
    loaded(&pool).await;
    let f = fake((60700.0, 60705.0));
    with_trader!(&pool, f, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(60700.0), Some(60705.0), at(60))
            .await
            .unwrap();
        assert!(t.syms[&BTC].pullback.is_some());
        let mut human = s3_position(9876, 1, 60705.0);
        human.trade_data.label = None;
        f.positions.lock().unwrap().push(human);
        // Restart before observing the human: the durable old arm must also
        // be cancelled, even though the new actor has not loaded it yet.
        t.inner = strategy3::Trader::new(&pool, t.allowlist, Environment::Demo, t.session)
            .await
            .unwrap();
        t.reconcile(at(60) + Duration::seconds(1)).await.unwrap();
        assert!(
            t.syms[&BTC].pullback.is_none(),
            "human ownership disarms the cached impulse"
        );
        let history_calls = log.lock().unwrap().trendbar_requests.len();
        t.syms.get_mut(&BTC).unwrap().day = None;
        t.on_quote(
            BTC,
            Some(60350.0),
            Some(60355.0),
            at(60) + Duration::seconds(2),
        )
        .await
        .unwrap();
        assert!(t.syms[&BTC].day.is_none(), "locked symbol is not prepared");
        assert_eq!(log.lock().unwrap().trendbar_requests.len(), history_calls);
        assert!(orders(&log).is_empty());
        assert!(closes(&log).is_empty());
        assert!(log.lock().unwrap().amends.is_empty());
        f.positions.lock().unwrap().clear();
        t.reconcile(at(90)).await.unwrap();
        // Once the human has closed, an inside-band quote cannot resurrect the
        // cancelled impulse. A later qualified mark can arm again.
        t.on_quote(BTC, Some(60000.0), Some(60005.0), at(90))
            .await
            .unwrap();
        assert!(t.syms[&BTC].pullback.is_none());
        t.on_quote(
            BTC,
            Some(60700.0),
            Some(60705.0),
            at(90) + Duration::seconds(4),
        )
        .await
        .unwrap();
        assert!(
            t.syms[&BTC].pullback.is_some(),
            "human symbol lock releases automatically"
        );
        assert!(orders(&log).is_empty());
    });
}
