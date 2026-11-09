//! Production actor regression for №18,392. All data and orders use the local
//! scripted broker; this never connects to a real account or market.
use std::collections::HashMap;
use std::time::Duration as WallDuration;

use chrono::{Duration, Utc};
use jiaoyi::{config::BrokerConfig, history, loader, shutdown, strategy3, trading};
use jiaoyi_core::allowlist::{Allowlist, Environment};
use jiaoyi_core::secret::Secret;
use jiaoyi_core::strategy3::{production_sessions as prod, sessions};
use jiaoyi_core::universe::{AssetClass, Resolution};
use jiaoyi_ctrader::messages::{
    ProtoOaDeal, ProtoOaInterval, ProtoOaLightSymbol, ProtoOaPayloadType as Payload,
    ProtoOaSpotEvent, ProtoOaSymbol, ProtoOaTrader,
};
use jiaoyi_ctrader::testkit::{m30_bar, FakeBroker, GrantEntry};
use jiaoyi_store::{self as store, settings};
use serde_json::{json, Value};
use sqlx::PgPool;

const ACCOUNT: i64 = 49236051;
const BTC: i64 = 22;
const OPEN: f64 = 60_000.0;
fn e5(price: f64) -> i64 {
    (price * 100_000.0).round() as i64
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn production_actor_prepares_before_touch_and_writes_one_guarded_order(pool: PgPool) {
    let rule = prod::rule_for("BTCUSD", AssetClass::Crypto).unwrap();
    let mut now = Utc::now();
    let mut window = sessions::session_at(&rule, now).unwrap();
    // Keep the actual clock for freshness proofs, but avoid racing an intended
    // production rollover if CI starts in the final seconds of a window.
    if window.close - now < Duration::seconds(10) {
        tokio::time::sleep((window.close - now).to_std().unwrap() + WallDuration::from_secs(1))
            .await;
        now = Utc::now();
        window = sessions::session_at(&rule, now).unwrap();
    }
    let contract = ProtoOaSymbol {
        symbol_id: BTC,
        digits: 2,
        pip_position: 2,
        min_volume: Some(1),
        step_volume: Some(1),
        schedule_time_zone: Some("UTC".into()),
        schedule: vec![ProtoOaInterval {
            start_second: 0,
            end_second: 7 * 86400,
        }],
        enable_short_selling: Some(true),
        trading_mode: Some(0),
        ..Default::default()
    };
    store::upsert_account(&pool, ACCOUNT, Environment::Demo)
        .await
        .unwrap();
    for (key, value) in [
        ("trading.margin_level_floor_pct", json!(200)),
        (
            "universe.symbols",
            json!([{"name":"BTCUSD","class":"crypto"}]),
        ),
    ] {
        settings::change(
            &pool,
            key,
            Some(ACCOUNT),
            Some(value),
            "test",
            "prepared actor fixture",
        )
        .await
        .unwrap();
    }
    store::upsert_symbol(
        &pool,
        &loader::symbol_row(
            ACCOUNT,
            "BTCUSD",
            AssetClass::Crypto,
            Some(true),
            &contract,
            now,
        ),
    )
    .await
    .unwrap();
    let run = store::start_run(&pool, ACCOUNT).await.unwrap();
    store::record_resolution(
        &pool,
        run,
        ACCOUNT,
        "BTCUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: BTC },
    )
    .await
    .unwrap();
    store::finish_run(
        &pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(json!([{"id":BTC,"quote_asset_id":1,"base_asset_id":9}])),
            assets: Some(json!([{"asset_id":1,"name":"USD"},{"asset_id":9,"name":"BTC"}])),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    store::history::record_account_snapshot(
        &pool,
        &store::history::AccountSnapshot {
            account_id: ACCOUNT,
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
    let deal = ProtoOaDeal {
        deal_id: 1,
        order_id: 2,
        position_id: 3,
        volume: 100,
        filled_volume: 100,
        symbol_id: BTC,
        create_timestamp: 0,
        execution_timestamp: now.timestamp_millis() - 86_400_000,
        execution_price: Some(OPEN),
        trade_side: 1,
        deal_status: 2,
        commission: Some(-300),
        money_digits: Some(2),
        ..Default::default()
    };
    store::history::upsert_deals(&pool, ACCOUNT, &[history::deal_row(&deal)], now)
        .await
        .unwrap();
    store::ticks::ensure_partitions(&pool, now.date_naive() - Duration::days(1), 3)
        .await
        .unwrap();
    let mut quotes: Vec<_> = (0..30)
        .map(|i| store::ticks::TickRow {
            account_id: ACCOUNT,
            session_id: "prepared-spread".into(),
            seq: i,
            broker_symbol_id: BTC,
            received_at: now - Duration::seconds(i),
            broker_timestamp: None,
            bid_e5: Some(e5(OPEN - 2.5)),
            ask_e5: Some(e5(OPEN + 2.5)),
        })
        .collect();
    quotes.push(store::ticks::TickRow {
        account_id: ACCOUNT,
        session_id: "prepared-opening".into(),
        seq: 0,
        broker_symbol_id: BTC,
        received_at: window.open,
        broker_timestamp: Some(window.open.timestamp_millis()),
        bid_e5: Some(e5(OPEN - 2.5)),
        ask_e5: Some(e5(OPEN + 2.5)),
    });
    store::ticks::insert(&pool, &quotes).await.unwrap();

    // All prior same-slot bars are causal. Positive multi-day drift and 75bp
    // historical range support the frozen long impulse independently of touch.
    let mut bars = Vec::new();
    for day in 1..=45 {
        let base = OPEN - day as f64;
        for minute in (0..1440).step_by(30) {
            let at = window.open - Duration::days(day) + Duration::minutes(minute);
            if at >= window.open {
                continue;
            }
            let close = base * 1.0075;
            bars.push(m30_bar(
                at,
                e5(base),
                e5(close + 1.0),
                e5(base - 1.0),
                e5(close),
                100,
            ));
        }
    }
    bars.push(m30_bar(
        window.open,
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        e5(OPEN),
        10,
    ));
    let (spots, _) = tokio::sync::broadcast::channel(128);
    let fake = FakeBroker {
        account_id: ACCOUNT,
        assets: vec![(1, "USD".into()), (9, "BTC".into())],
        light_symbols: vec![ProtoOaLightSymbol {
            symbol_id: BTC,
            base_asset_id: Some(9),
            quote_asset_id: Some(1),
            ..Default::default()
        }],
        symbols: vec![contract],
        m30_bars: HashMap::from([(BTC, bars)]),
        fill_quotes: HashMap::from([(BTC, (60_227.5, 60_232.5))]),
        spot_events: Some(spots.clone()),
        trader: Some(ProtoOaTrader {
            ctid_trader_account_id: ACCOUNT,
            balance: 1_000_000,
            deposit_asset_id: 1,
            money_digits: Some(2),
            account_type: Some(0),
            ..Default::default()
        }),
        grant: Some(vec![GrantEntry {
            ctid: ACCOUNT,
            is_live: Some(false),
            login: None,
        }]),
        ..Default::default()
    };
    let (conn, log) = fake.spawn();
    let list = Allowlist::parse(Some("49236051")).unwrap();
    let broker = BrokerConfig {
        demo_host: "unused".into(),
        live_host: "unused".into(),
        port: 0,
        client_id: Some("client".into()),
        client_secret: Some(Secret::new("client-secret")),
        seed_access_token: Some(Secret::new("tok")),
        seed_refresh_token: Some(Secret::new("ref")),
        seed_expires_at: None,
    };
    let account = list.permit(ACCOUNT).unwrap();
    let session = loader::authorise(&pool, &conn, &broker, &account)
        .await
        .unwrap();
    let live = trading::LivenessMap::default();
    let mut ends = conn.session_ends();
    let (stop, mut shutdown) = shutdown::channel();
    let actor = strategy3::run_session_with_event_offset(
        &pool,
        &list,
        Environment::Demo,
        &session,
        &live,
        &mut ends,
        &mut shutdown,
        Duration::zero(),
    );
    tokio::pin!(actor);
    let emit = |mid: f64| {
        spots
            .send(ProtoOaSpotEvent {
                ctid_trader_account_id: ACCOUNT,
                symbol_id: BTC,
                bid: Some(e5(mid - 2.5) as u64),
                ask: Some(e5(mid + 2.5) as u64),
                ..Default::default()
            })
            .unwrap();
    };
    let exercise = async {
        // The genuine actor must arm and publish a reusable admission. A direct
        // Trader fixture would bypass this lane and cannot satisfy the barrier.
        loop {
            emit(60_460.0);
            tokio::time::sleep(WallDuration::from_millis(50)).await;
            let ready = live
                .lock()
                .unwrap()
                .get(&ACCOUNT)
                .and_then(|l| l.triggers.get("BTCUSD"))
                .is_some_and(|v| v["entry_preparation"]["prepared_now"] == true);
            if ready {
                break;
            }
        }
        assert!(
            log.lock().unwrap().orders.is_empty(),
            "arming does not place an order"
        );
        let payload_start = log.lock().unwrap().payload_types.len();
        let touched = std::time::Instant::now();
        emit(60_230.0);
        loop {
            if !log.lock().unwrap().orders.is_empty() {
                break;
            }
            tokio::time::sleep(WallDuration::from_millis(1)).await;
        }
        let elapsed = touched.elapsed();
        let payloads = log.lock().unwrap().payload_types[payload_start..].to_vec();
        let write = payloads
            .iter()
            .position(|p| *p == Payload::ProtoOaNewOrderReq as u32)
            .unwrap();
        let forbidden = [
            Payload::ProtoOaTraderReq,
            Payload::ProtoOaReconcileReq,
            Payload::ProtoOaGetPositionUnrealizedPnlReq,
            Payload::ProtoOaGetTrendbarsReq,
            Payload::ProtoOaExpectedMarginReq,
            Payload::ProtoOaSymbolByIdReq,
        ];
        assert!(
            !payloads[..write]
                .iter()
                .any(|p| forbidden.iter().any(|f| *p == *f as u32)),
            "prepared touch made a new broker admission/history request: {:?}",
            &payloads[..write]
        );
        assert!(
            elapsed < WallDuration::from_millis(250),
            "touch to fake-broker order: {elapsed:?}"
        );
        // A duplicate fresh quote must not duplicate the consumed durable arm.
        for _ in 0..3 {
            emit(60_230.0);
            tokio::time::sleep(WallDuration::from_millis(30)).await;
        }
        let raw: Vec<u8> = loop {
            if let Some(raw) = sqlx::query_scalar::<_,Vec<u8>>(
                "SELECT raw FROM trading_events WHERE account_id=$1 AND kind='ENTRY_VALIDATED_BEFORE_SEND' LIMIT 1")
                .bind(ACCOUNT).fetch_optional(&pool).await.unwrap() { break raw; }
            tokio::task::yield_now().await;
        };
        let receipt: Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(receipt["result"], "passed");
        assert_eq!(receipt["pullback_admitted_price"]["bounded"], true);
        assert_eq!(receipt["broker_calendar_symbol_id"], BTC);
        assert!(receipt["fx_margin_revalidation"].is_object());
        let intents: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM trading_intents WHERE account_id=$1 AND kind='open'",
        )
        .bind(ACCOUNT)
        .fetch_one(&pool)
        .await
        .unwrap();
        let records: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM strategy3_orders WHERE account_id=$1 AND kind='open'",
        )
        .bind(ACCOUNT)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            (intents, records, log.lock().unwrap().orders.len()),
            (1, 1, 1)
        );
        eprintln!("production actor touch-to-write {elapsed:?}; one durable intent, zero intervening admission/history reads");
        stop.fire();
    };
    let result = tokio::time::timeout(WallDuration::from_secs(30), async {
        tokio::select! {
            r = &mut actor => panic!("actor stopped before exercise: {r:?}"),
            _ = exercise => actor.await.unwrap(),
        }
    })
    .await;
    if result.is_err() {
        let state = live
            .lock()
            .unwrap()
            .get(&ACCOUNT)
            .map(|l| l.triggers.clone());
        let decisions: Vec<(String,String)> = sqlx::query_as(
            "SELECT kind,reason FROM trading_decisions WHERE account_id=$1 ORDER BY id DESC LIMIT 8")
            .bind(ACCOUNT).fetch_all(&pool).await.unwrap();
        panic!("production actor timed out: {state:?}; decisions={decisions:?}");
    }
}
