//! Strategy 1 (the tick-triggered trend) and the order gate against a real
//! PostgreSQL and the scripted broker (owner, 25-10-2026): demo trades at
//! once, live waits for the edge gate, the open-risk budget, the reviewed
//! settings seed, one entry per completed day, the trail, and positions this
//! program did not open.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{Duration as Days, Utc};
use jiaoyi::config::BrokerConfig;
use jiaoyi::{loader, shutdown, trading};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_core::universe::{AssetClass, Resolution};
use jiaoyi_ctrader::messages::{ProtoOaPosition, ProtoOaSymbol, ProtoOaTradeData, ProtoOaTrader};
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker, GrantEntry, SidedQuote};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::PgPool;

const DEMO: i64 = 49236051;
const LIVE: i64 = 43002148;
const ETH: i64 = 22;
const BTC: i64 = 23;
const JP225: i64 = 30;
const USDJPY: i64 = 31;
const HASH: &str = "9105c5eb812ff6b3d267627a53e64d924018c799753d70e598830381a909098f";

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

/// The stored data a load leaves: the account, ETHUSD (digits 2, minimum
/// and step 1 cent), its resolution, the asset list (USD) and the trader
/// record's deposit asset.
async fn loaded(pool: &PgPool) {
    loaded_with(pool, &[(ETH, "ETHUSD", AssetClass::Crypto, 1)]).await;
}

/// The same for these symbols: (id, name, class, quote asset id), each with
/// digits 2 and a minimum and step of 1 cent. Assets: 1 USD, 3 JPY.
async fn loaded_with(pool: &PgPool, syms: &[(i64, &str, AssetClass, i64)]) {
    store::upsert_account(pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    let run = store::start_run(pool, DEMO).await.unwrap();
    let mut listed = Vec::new();
    for (id, name, class, quote) in syms {
        let s = ProtoOaSymbol {
            symbol_id: *id,
            digits: 2,
            pip_position: 2,
            min_volume: Some(1),
            step_volume: Some(1),
            ..Default::default()
        };
        store::upsert_symbol(
            pool,
            &loader::symbol_row(DEMO, name, *class, Some(true), &s, Utc::now()),
        )
        .await
        .unwrap();
        store::record_resolution(
            pool,
            run,
            DEMO,
            name,
            *class,
            &Resolution::Resolved { symbol_id: *id },
        )
        .await
        .unwrap();
        listed.push(json!({"id": id, "quote_asset_id": quote, "base_asset_id": 9}));
    }
    store::finish_run(
        pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(json!(listed)),
            assets: Some(
                json!([{"asset_id": 1, "name": "USD"}, {"asset_id": 3, "name": "JPY"},
                {"asset_id": 9, "name": "ETH"}]),
            ),
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
            balance: 100_244,
            money_digits: Some(2),
            registration_at: None,
            raw: vec![],
        },
    )
    .await
    .unwrap();
}

async fn set(pool: &PgPool, key: &str, value: serde_json::Value) {
    settings::change(pool, key, Some(DEMO), Some(value), "test", "trading test")
        .await
        .unwrap();
}

/// 160 completed daily bars ending yesterday, rising 10 a day, each `half`
/// (in 1/100) either side of its open: EMA(50) > EMA(100), the level is the
/// last high, ATR(20) = 2 × half / 100.
fn bars(half: i64) -> Vec<jiaoyi_ctrader::messages::ProtoOaTrendbar> {
    let today = Utc::now().date_naive();
    (0..160)
        .map(|i| {
            let d = today - Days::days(160 - i);
            let b = 200_000 + 1_000 * i; // ×1/100 → 2000.00 + 10 a day, in 1/100000
            d1_bar(
                d,
                b * 1_000,
                (b + half) * 1_000,
                (b - half) * 1_000,
                (b + 500) * 1_000,
                10,
            )
        })
        .collect()
}

/// A quote in the fake's units.
fn q(bid: f64, ask: f64) -> SidedQuote {
    (
        Some((bid * 100_000.0).round() as u64),
        Some((ask * 100_000.0).round() as u64),
    )
}

/// ETHUSD with `bars(2_000)`: the level is the last high (3610.00), ATR(20) = 40.
fn fake(quotes: Vec<(f64, f64)>) -> FakeBroker {
    FakeBroker {
        quotes: HashMap::from([(ETH, quotes.iter().map(|(b, a)| q(*b, *a)).collect())]),
        fill_quotes: HashMap::from([(ETH, quotes[0])]),
        ..fake_on(&[ETH], 2_000)
    }
}

/// The scripted broker with `bars(half)` on each symbol and no quotes.
fn fake_on(ids: &[i64], half: i64) -> FakeBroker {
    FakeBroker {
        account_id: DEMO,
        bars: ids.iter().map(|id| (*id, bars(half))).collect(),
        trader: Some(ProtoOaTrader {
            ctid_trader_account_id: DEMO,
            balance: 100_244,
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

/// Run the Strategy 1 loop on `f` for `secs` as an account on `env`, then stop it.
async fn trade(
    pool: &PgPool,
    f: &FakeBroker,
    secs: u64,
    env: Environment,
) -> std::sync::Arc<std::sync::Mutex<jiaoyi_ctrader::testkit::FakeLog>> {
    trade_with(pool, f, secs, env, &trading::LivenessMap::default()).await
}

/// `trade`, with the loop's liveness kept in `live`.
async fn trade_with(
    pool: &PgPool,
    f: &FakeBroker,
    secs: u64,
    env: Environment,
    live: &trading::LivenessMap,
) -> std::sync::Arc<std::sync::Mutex<jiaoyi_ctrader::testkit::FakeLog>> {
    let (conn, log) = f.clone().spawn();
    let session = loader::authorise(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let mut d = trading::decisions(pool, DEMO);
    let list = allowlist();
    tokio::select! {
        r = trading::run_session(pool, &list, env, &session, &mut d, live, &mut ends, &mut stop) => {
            r.unwrap();
        }
        _ = tokio::time::sleep(Duration::from_secs(secs)) => trigger.fire(),
    }
    log
}

async fn decisions(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as("SELECT kind, reason FROM trading_decisions WHERE account_id = $1 ORDER BY id")
        .bind(DEMO)
        .fetch_all(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn demo_trades_without_the_gate_and_live_waits_for_it(pool: PgPool) {
    for id in [49236051, 46130058, 47790949] {
        let r = trading::rules(&pool, id, Environment::Demo).await.unwrap();
        assert!(!r.edge_required, "{id}");
        assert!(!r.allow_min_volume, "demo skips below the minimum: {id}");
    }
    let r = trading::rules(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    assert!(r.edge_required);
    assert!(r.allow_min_volume);
    assert_eq!(r.risk_pct, 0.6);
    assert_eq!(r.min_volume_max_risk_pct, 10.0);
    assert_eq!(r.trend_open_risk_pct, 3.0);
    // No on/off switch exists (owner, 24-10).
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM setting_definitions WHERE key LIKE '%enabled%'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_live_edge_gate_opens_only_on_a_held_out_pass(pool: PgPool) {
    let r = trading::rules(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    let g = trading::edge_gate(&pool, &r).await.unwrap();
    assert!(g.unwrap_err().contains("no H1 held-out trial"));
    let trial = |verdict: &'static str, sample: &'static str, hash: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query(
                "INSERT INTO research_trials (hypothesis, params, data_range, sample, verdict, settings_version, prereg_hash)
                 VALUES ('H1', '{}', '{}', $1, $2, 1, $3)",
            )
            .bind(sample)
            .bind(verdict)
            .bind(hash)
            .execute(&pool)
            .await
            .unwrap();
        }
    };
    // A development PASS, or a PASS under another hash, does not open it.
    trial("PASS", "development", HASH).await;
    trial(
        "PASS",
        "held_out",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .await;
    assert!(trading::edge_gate(&pool, &r).await.unwrap().is_err());
    trial("FAIL", "held_out", HASH).await;
    let g = trading::edge_gate(&pool, &r).await.unwrap();
    assert!(g.unwrap_err().contains("is FAIL"));
    trial("PASS", "held_out", HASH).await;
    assert!(trading::edge_gate(&pool, &r).await.unwrap().is_ok());
    // A demo account never waits for it.
    let demo = trading::rules(&pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    assert!(trading::edge_gate(&pool, &demo).await.unwrap().is_ok());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_reviewed_seed_change_is_applied_once_and_logged(pool: PgPool) {
    let text = r#"[
      {"id": "2026-10-25-live-min-off", "key": "trading.live_allow_min_volume", "account_id": 43002148, "value": 0,
       "changed_by": "owner", "reason": "test: live skips below the minimum"},
      {"id": "2026-10-25-bad", "key": "trading.trend_open_risk_pct", "account_id": 46130058, "value": 500,
       "changed_by": "owner", "reason": "out of bounds"}
    ]"#;
    let applied = trading::apply_seed(&pool, text).await.unwrap();
    assert_eq!(applied, vec!["2026-10-25-live-min-off".to_string()]);
    let r = trading::rules(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    assert!(!r.allow_min_volume);
    let other = trading::rules(&pool, 46130058, Environment::Demo)
        .await
        .unwrap();
    assert_eq!(
        other.trend_open_risk_pct, 3.0,
        "the refused entry changed nothing"
    );
    let log: Vec<(String, String)> = sqlx::query_as(
        "SELECT changed_by, reason FROM settings_changes WHERE key = 'trading.live_allow_min_volume' AND account_id = $1",
    )
    .bind(LIVE)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        log,
        vec![("owner".into(), "test: live skips below the minimum".into())]
    );
    // Applied once: a restart changes nothing.
    assert!(trading::apply_seed(&pool, text).await.unwrap().is_empty());
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM settings_changes WHERE key LIKE 'trading.%' AND account_id IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_demo_account_sends_one_entry_at_once_and_trails_it(pool: PgPool) {
    loaded(&pool).await;
    // No setting is changed: a demo account trades as soon as it runs.
    // The ask reaches the level (3610.00); then the bid rises to 3700.00.
    let f = fake(vec![(3608.0, 3611.0), (3700.0, 3701.0)]);
    let log = trade(&pool, &f, 3, Environment::Demo).await;
    let orders = log.lock().unwrap().orders.clone();
    assert_eq!(orders.len(), 1, "{:?}", decisions(&pool).await);
    let o = &orders[0];
    assert_eq!(o.symbol_id, ETH);
    assert_eq!(o.trade_side, 1, "buy");
    // 3 × ATR(20) = 120.00 → 12,000,000 in 1/100000.
    assert_eq!(o.relative_stop_loss, Some(12_000_000));
    // 0.6% of $1,002.44 = $6.0146; one cent risks 0.01 × 120 = $1.20 → 5 cents.
    assert_eq!(o.volume, 5);
    assert_eq!(o.label.as_deref(), Some("jiaoyi-s1"));
    assert_eq!(o.take_profit, None);
    // The trail: best bid 3700.00 − 120.00 = 3580.00, above the fill's 3491.00.
    let amends = log.lock().unwrap().amends.clone();
    assert_eq!(amends.len(), 1, "{:?}", decisions(&pool).await);
    assert_eq!(amends[0].stop_loss, Some(3580.0));
    let kinds: Vec<String> = sqlx::query_scalar("SELECT kind FROM trading_intents ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(kinds, vec!["open", "amend_stop"]);
    let events: Vec<String> = sqlx::query_scalar("SELECT kind FROM trading_events ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(
        events,
        vec!["ORDER_ACCEPTED", "ORDER_FILLED", "ORDER_REPLACED"]
    );
    // A restart after the position was stopped out: the same completed day's
    // entry key is recorded, so nothing is sent again.
    f.positions.lock().unwrap().clear();
    let log2 = trade(&pool, &f, 2, Environment::Demo).await;
    assert!(log2.lock().unwrap().orders.is_empty());
    // /health shows the strategy and the decision, and no money.
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["strategy"], "Strategy 1: tick-triggered trend");
    assert_eq!(h[0]["trades"], "yes");
    assert_eq!(h[0]["edge_gate"], "not required (demo)");
    assert!(!h.to_string().contains("balance"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_live_account_is_armed_and_holds_the_signal_until_the_edge_passes(pool: PgPool) {
    loaded(&pool).await;
    let log = trade(&pool, &fake(vec![(3608.0, 3611.0)]), 2, Environment::Live).await;
    assert!(log.lock().unwrap().orders.is_empty());
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "signal_held" && r.contains("edge gate not met")),
        "{d:?}"
    );
    // The key is not spent: the signal can be taken once the gate opens.
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_intents")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_entry_beyond_the_open_risk_budget_is_skipped(pool: PgPool) {
    loaded(&pool).await;
    // 0.5% of $1,002.44 = $5.01, below this entry's $6.00.
    set(&pool, "trading.trend_open_risk_pct", json!(0.5)).await;
    let log = trade(&pool, &fake(vec![(3608.0, 3611.0)]), 2, Environment::Demo).await;
    assert!(log.lock().unwrap().orders.is_empty());
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "skip" && r.contains("exceed its budget")),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_position_this_program_did_not_open_blocks_the_entry_and_is_reported(pool: PgPool) {
    loaded(&pool).await;
    let f = fake(vec![(3608.0, 3611.0)]);
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 77,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 300,
            trade_side: 2,
            label: Some("bot-trade".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        stop_loss: Some(4000.0),
        ..Default::default()
    });
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    let l = log.lock().unwrap().clone();
    assert!(l.orders.is_empty());
    assert!(l.amends.is_empty() && l.closes.is_empty(), "never touched");
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "refused" && r.contains("external")),
        "{d:?}"
    );
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    let ext = &h[0]["last_reconcile"]["external_positions"];
    assert_eq!(ext[0]["position_id"], 77);
    assert_eq!(ext[0]["side"], "sell");
    assert_eq!(ext[0]["volume"], 300);
}

async fn event_kinds(pool: &PgPool) -> Vec<(Option<String>, String)> {
    sqlx::query_as("SELECT entry_key, kind FROM trading_events ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

fn intent(key: &str, kind: &str) -> store::trading::IntentRow {
    store::trading::IntentRow {
        account_id: DEMO,
        entry_key: key.into(),
        client_msg_id: format!("t-{key}"),
        kind: kind.into(),
        symbol_id: Some(ETH),
        position_id: None,
        side: Some("buy".into()),
        volume: Some(5),
        relative_stop: Some(12_000_000),
        stop_price: None,
        take_profit: None,
        risk: Some(6.0),
        max_risk: 6.6,
        settings_version: None,
    }
}

fn event(key: &str, kind: &str, at: chrono::DateTime<Utc>) -> store::trading::EventRow {
    store::trading::EventRow {
        account_id: DEMO,
        entry_key: Some(key.into()),
        client_msg_id: Some(format!("t-{key}")),
        payload_type: 2126,
        kind: kind.into(),
        position_id: None,
        order_id: None,
        deal_id: None,
        symbol_id: Some(ETH),
        side: None,
        price: None,
        volume: None,
        stop_loss: None,
        label: None,
        error_code: None,
        description: None,
        is_server_event: None,
        raw: vec![],
        received_at: at,
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_accepted_only_open_is_unresolved_and_an_answer_is_kept_once(pool: PgPool) {
    // Codex P1 on #14: an open with only ORDER_ACCEPTED may still fill.
    let ts = store::trading::record_intent;
    let at = Utc::now();
    for (key, kind) in [
        ("accepted", Some("ORDER_ACCEPTED")),
        ("filled", Some("ORDER_FILLED")),
        ("partial", Some("ORDER_PARTIAL_FILL")),
        ("rejected", Some("ORDER_REJECTED")),
        ("errored", Some("ORDER_ERROR")),
        ("released", Some("RECONCILED_ABSENT")),
        ("silent", None),
    ] {
        assert!(ts(&pool, &intent(key, "open")).await.unwrap());
        if let Some(kind) = kind {
            store::trading::record_event(&pool, &event(key, kind, at))
                .await
                .unwrap();
        }
    }
    assert!(ts(&pool, &intent("trail", "amend_stop")).await.unwrap());
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    let keys: Vec<&str> = u.iter().map(|u| u.entry_key.as_str()).collect();
    // A partial fill leaves its remainder open (Codex round 3 on #14).
    assert_eq!(keys, vec!["accepted", "partial", "silent"]);
    assert_eq!(u[0].risk, Some(6.0));
    assert_eq!(u[0].symbol_id, Some(ETH));
    // The gate and the trader both receive one answer: kept once.
    store::trading::record_event(&pool, &event("accepted", "ORDER_ACCEPTED", at))
        .await
        .unwrap();
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM trading_events WHERE entry_key = 'accepted'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_late_fill_is_recorded_under_its_intent_adopted_and_trailed(pool: PgPool) {
    loaded(&pool).await;
    // The gate gives up after 1 s; the broker fills at 2 s (Codex P1 on #14).
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake_on(&[ETH], 2_000);
    f.fill_quotes = HashMap::from([(ETH, (3608.0, 3611.0))]);
    f.fill_delay = Some(Duration::from_secs(2));
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0)), (4_000, ETH, q(3700.0, 3701.0))];
    let log = trade(&pool, &f, 6, Environment::Demo).await;
    let l = log.lock().unwrap().clone();
    assert_eq!(l.orders.len(), 1, "{:?}", decisions(&pool).await);
    let key = trading::entry_key(DEMO, ETH, Utc::now().date_naive() - Days::days(1));
    let ev = event_kinds(&pool).await;
    assert!(
        ev.contains(&(Some(key.clone()), "ORDER_FILLED".into())),
        "the late fill is recorded under its intent: {ev:?}"
    );
    // Adopted at once, then trailed: 3700.00 − 120.00.
    assert_eq!(l.amends.len(), 1, "{:?}", decisions(&pool).await);
    assert_eq!(l.amends[0].stop_loss, Some(3580.0));
    assert!(l.orders.len() == 1, "never resent");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_accepted_only_open_is_adopted_after_a_restart(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    // Accepted, and not filled within this session.
    let mut f = fake(vec![(3608.0, 3611.0)]);
    f.fill_delay = Some(Duration::from_secs(600));
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1);
    let key = trading::entry_key(DEMO, ETH, Utc::now().date_naive() - Days::days(1));
    assert_eq!(
        event_kinds(&pool).await,
        vec![(Some(key.clone()), "ORDER_ACCEPTED".to_string())]
    );
    // It filled while the service was down: the position is there at restart.
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 1001,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 5,
            trade_side: 1,
            label: Some("jiaoyi-s1".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        price: Some(3611.0),
        stop_loss: Some(3491.0),
        ..Default::default()
    });
    f.fill_delay = None;
    let log2 = trade(&pool, &f, 2, Environment::Demo).await;
    assert!(log2.lock().unwrap().orders.is_empty(), "never resent");
    let ev = event_kinds(&pool).await;
    assert!(
        ev.contains(&(Some(key), "RECONCILED_FILL".into())),
        "adopted, not external: {ev:?}"
    );
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["last_reconcile"]["own"], 1, "{h}");
    assert_eq!(h[0]["last_reconcile"]["external_positions"], json!([]));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unconfirmed_open_holds_its_risk_in_the_budget(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    // 1% of $1,002.44 = $10.02: one $6.00 entry fits, two do not.
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake_on(&[ETH, BTC], 2_000);
    f.silent_orders = true;
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0)), (100, BTC, q(3608.0, 3611.0))];
    let log = trade(&pool, &f, 4, Environment::Demo).await;
    let d = decisions(&pool).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "{d:?}");
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains("open risk 6.00 plus this entry's 6.00 would exceed its budget 10.02")),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_conversion_quote_arriving_later_lets_the_signal_through(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (JP225, "JP225", AssetClass::EquityIndex, 3),
            (USDJPY, "USDJPY", AssetClass::Fx, 3),
        ],
    )
    .await;
    let mut f = fake_on(&[JP225], 2_000);
    f.fill_quotes = HashMap::from([(JP225, (3608.0, 3611.0))]);
    // The JPY index triggers before any USDJPY quote (Codex P2 on #14),
    // then USDJPY quotes, then the index triggers again.
    f.quote_sequence = vec![
        (0, JP225, q(3608.0, 3611.0)),
        (300, USDJPY, q(150.00, 150.02)),
        (600, JP225, q(3608.0, 3611.0)),
    ];
    let log = trade(&pool, &f, 3, Environment::Demo).await;
    let d = decisions(&pool).await;
    assert!(
        d.iter()
            .any(|(k, r)| k == "skip" && r.contains("no broker quote→deposit rate")),
        "{d:?}"
    );
    let orders = log.lock().unwrap().orders.clone();
    assert_eq!(orders.len(), 1, "{d:?}");
    assert_eq!(orders[0].symbol_id, JP225);
    // 0.6% of $1,002.44 = $6.0146; one cent risks 0.01 × 120 / 150.01 = $0.0080 → 751.
    assert_eq!(orders[0].volume, 751);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn live_takes_one_minimum_trade_at_a_time_with_the_gate_lifted(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    // The shipped seed: the owner's 25-10 "Live too, from now".
    trading::apply_seed(&pool, trading::SEED_FILE)
        .await
        .unwrap();
    // $38.98; ATR(20) 95.00, so the stop is 285.00 and ETHUSD's minimum
    // (1 cent) risks $2.85 (7.3%), above the 3% budget ($1.17).
    let mut f = fake_on(&[ETH, BTC], 4_750);
    f.trader.as_mut().unwrap().balance = 3_898;
    f.fill_quotes = HashMap::from([(ETH, (3636.0, 3638.0)), (BTC, (3636.0, 3638.0))]);
    f.quote_sequence = vec![(0, ETH, q(3636.0, 3638.0)), (500, BTC, q(3636.0, 3638.0))];
    let log = trade(&pool, &f, 3, Environment::Live).await;
    let d = decisions(&pool).await;
    let orders = log.lock().unwrap().orders.clone();
    assert_eq!(orders.len(), 1, "{d:?}");
    assert_eq!(orders[0].volume, 1, "exactly the broker minimum");
    assert_eq!(orders[0].relative_stop_loss, Some(28_500_000));
    assert!(
        d.iter()
            .any(|(k, r)| k == "skip" && r.contains("one minimum trade at a time")),
        "the second waits: {d:?}"
    );
    // /health and the status page: live trading, and who lifted the gate.
    store::upsert_account(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    let live = h
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["account_id"] == LIVE)
        .unwrap();
    let when = Utc::now().format("%d-%m");
    assert_eq!(
        live["trades"],
        format!("trading (edge gate lifted by the owner {when})")
    );
    let demo = h
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["account_id"] == DEMO)
        .unwrap();
    assert_eq!(demo["trades"], "yes");
}

// Claude · № 17,020 25-Oct (Codex P1 on #15): the live rule is account-wide.
/// A Strategy 3 position open on the live account (another symbol) stops
/// Strategy 1's broker-minimum entry: read fresh, across both loops.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_live_minimum_waits_for_a_strategy_3_position_on_the_account(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    trading::apply_seed(&pool, trading::SEED_FILE)
        .await
        .unwrap();
    let mut f = fake_on(&[ETH, BTC], 4_750);
    f.trader.as_mut().unwrap().balance = 3_898;
    f.fill_quotes = HashMap::from([(ETH, (3636.0, 3638.0))]);
    f.quote_sequence = vec![(0, ETH, q(3636.0, 3638.0))];
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 900,
        trade_data: ProtoOaTradeData {
            symbol_id: BTC,
            volume: 1,
            trade_side: 1,
            label: Some("jiaoyi-s3".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        ..Default::default()
    });
    let log = trade(&pool, &f, 2, Environment::Live).await;
    let d = decisions(&pool).await;
    assert!(log.lock().unwrap().orders.is_empty(), "{d:?}");
    assert!(
        d.iter()
            .any(|(k, r)| k == "skip" && r.contains("account-wide") && r.contains("1 position(s)")),
        "{d:?}"
    );
    // Strategy 3's position is neither Strategy 1's nor external.
    assert!(
        d.iter()
            .any(|(k, r)| k == "reconcile" && r.contains("0 external")),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_demo_account_never_takes_the_minimum(pool: PgPool) {
    loaded(&pool).await;
    trading::apply_seed(&pool, trading::SEED_FILE)
        .await
        .unwrap();
    let mut f = fake_on(&[ETH], 4_750);
    f.trader.as_mut().unwrap().balance = 3_898;
    f.fill_quotes = HashMap::from([(ETH, (3636.0, 3638.0))]);
    f.quote_sequence = vec![(0, ETH, q(3636.0, 3638.0))];
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    let d = decisions(&pool).await;
    assert!(log.lock().unwrap().orders.is_empty(), "{d:?}");
    assert!(
        d.iter()
            .any(|(k, r)| k == "skip" && r.contains("below the broker minimum")),
        "{d:?}"
    );
}

/// Two crypto symbols at $20,001.00: a 0.6% entry is 100 cents (risk
/// $120.00 at the 120.00 stop). The fake fills ETHUSD's order in `parts`.
async fn two_symbols_filled_in(
    pool: &PgPool,
    parts: Vec<(u64, i64, f64)>,
    cancel: bool,
    quotes: Vec<(u64, i64, SidedQuote)>,
    secs: u64,
) -> std::sync::Arc<std::sync::Mutex<jiaoyi_ctrader::testkit::FakeLog>> {
    loaded_with(
        pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    let mut f = fake_on(&[ETH, BTC], 2_000);
    f.trader.as_mut().unwrap().balance = 2_000_100;
    f.fill_parts = parts;
    f.cancel_remainder = cancel;
    f.quote_sequence = quotes;
    trade(pool, &f, secs, Environment::Demo).await
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_fill_in_several_deals_is_adopted_at_the_brokers_aggregate(pool: PgPool) {
    // Codex round 2 on #14: 90 + 10 cents was tracked as the last deal's 10.
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    let log = two_symbols_filled_in(
        &pool,
        vec![(0, 90, 3611.0), (0, 10, 3621.0)],
        false,
        vec![(0, ETH, q(3608.0, 3611.0)), (500, BTC, q(3608.0, 3611.0))],
        3,
    )
    .await;
    let d = decisions(&pool).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "{d:?}");
    // 100 cents at the broker's VWAP 3612.00 with its stop 3491.00: $121.00.
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains(
                "open risk 121.00 plus this entry's 120.00 would exceed its budget 200.01"
            )),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partial_fill_with_its_remainder_cancelled_is_adopted_and_trailed(pool: PgPool) {
    // Codex round 2 on #14: the gate answers Rejected (the remainder was
    // cancelled), yet 60 cents are open at the broker.
    set(&pool, "trading.trend_open_risk_pct", json!(0.9)).await;
    let log = two_symbols_filled_in(
        &pool,
        vec![(0, 60, 3611.0)],
        true,
        vec![
            (0, ETH, q(3608.0, 3611.0)),
            (300, BTC, q(3608.0, 3611.0)),
            (600, ETH, q(3700.0, 3701.0)),
        ],
        3,
    )
    .await;
    let d = decisions(&pool).await;
    let l = log.lock().unwrap().clone();
    assert_eq!(l.orders.len(), 1, "{d:?}");
    // Trailed: 3700.00 − 120.00.
    assert_eq!(l.amends.len(), 1, "{d:?}");
    assert_eq!(l.amends[0].stop_loss, Some(3580.0));
    // The second entry sees its risk: 60 cents from 3611.00 to 3491.00 = $72.00.
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains(
                "open risk 72.00 plus this entry's 120.00 would exceed its budget 180.01"
            )),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_remainder_filled_after_the_wait_is_read_from_the_broker(pool: PgPool) {
    // 60 cents within the 1 s wait, the other 40 at 2 s.
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let log = two_symbols_filled_in(
        &pool,
        vec![(0, 60, 3611.0), (2_000, 40, 3621.0)],
        false,
        vec![(0, ETH, q(3608.0, 3611.0)), (3_000, BTC, q(3608.0, 3611.0))],
        4,
    )
    .await;
    let d = decisions(&pool).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "{d:?}");
    // 100 cents at the broker's VWAP 3615.00 with its stop 3491.00: $124.00.
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains(
                "open risk 124.00 plus this entry's 120.00 would exceed its budget 200.01"
            )),
        "{d:?}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_open_remainder_is_reserved_until_it_fills(pool: PgPool) {
    // Codex round 3 on #14: 60 of 100 cents fill inside the 1 s wait; at
    // 1.5 s BTCUSD signals. Its $120.00 entry must not take the room the
    // ETHUSD remainder (40 cents, $48.00) still needs; it fills at 2 s.
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let log = two_symbols_filled_in(
        &pool,
        vec![(0, 60, 3611.0), (2_000, 40, 3621.0)],
        false,
        vec![(0, ETH, q(3608.0, 3611.0)), (1_500, BTC, q(3608.0, 3611.0))],
        3,
    )
    .await;
    let d = decisions(&pool).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "{d:?}");
    // $72.00 adopted (60 cents from 3611.00 to 3491.00) plus $48.00 reserved.
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains(
                "open risk 120.00 plus this entry's 120.00 would exceed its budget 200.01"
            )),
        "{d:?}"
    );
    // Once the remainder filled, the intent is resolved: nothing is reserved.
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert!(u.is_empty(), "{u:?}");
}

/// Codex round 4 on #14: an accepted open of 100 cents filled 60 while the
/// service was down. At startup the position is adopted at the broker's 60,
/// the intent stays unresolved and its 40-cent remainder ($48.00) stays
/// reserved, so BTCUSD's $120.00 entry is skipped. Once the broker holds all
/// 100, the open is recorded filled in full and nothing is reserved.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partial_fill_recovered_at_startup_keeps_its_remainder_reserved(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake_on(&[ETH, BTC], 2_000);
    f.trader.as_mut().unwrap().balance = 2_000_100;
    // Accepted, and not filled within this session.
    f.fill_delay = Some(Duration::from_secs(600));
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0))];
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    let o = log.lock().unwrap().orders.clone();
    assert_eq!(o.len(), 1, "{:?}", decisions(&pool).await);
    assert_eq!(o[0].volume, 100);
    let key = trading::entry_key(DEMO, ETH, Utc::now().date_naive() - Days::days(1));
    // While the service was down, 60 of the 100 cents filled.
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 1001,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 60,
            trade_side: 1,
            label: Some("jiaoyi-s1".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        price: Some(3611.0),
        stop_loss: Some(3491.0),
        ..Default::default()
    });
    // A second labelled ETHUSD position no intent of this program filled.
    let mut other = f.positions.lock().unwrap()[0].clone();
    other.position_id = 1002;
    other.trade_data.volume = 10;
    f.positions.lock().unwrap().push(other);
    f.fill_delay = None;
    f.quote_sequence = vec![(500, BTC, q(3608.0, 3611.0))];
    let log2 = trade(&pool, &f, 2, Environment::Demo).await;
    let d = decisions(&pool).await;
    assert!(log2.lock().unwrap().orders.is_empty(), "{d:?}");
    // $72.00 adopted (60 cents from 3611.00 to 3491.00) plus $48.00 reserved.
    assert!(
        d.iter().any(|(k, r)| k == "skip"
            && r.contains(
                "open risk 120.00 plus this entry's 120.00 would exceed its budget 200.01"
            )),
        "{d:?}"
    );
    let ev: Vec<(String, Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT kind, position_id, volume FROM trading_events
         WHERE entry_key = $1 AND kind LIKE 'RECONCILED%' ORDER BY id",
    )
    .bind(&key)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        ev,
        vec![("RECONCILED_PARTIAL_FILL".into(), Some(1001), Some(60))],
        "adopted at the broker's volume, not filled in full"
    );
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert_eq!(u.len(), 1, "{u:?}");
    assert_eq!((u[0].filled, u[0].position_id), (60, Some(1001)), "{u:?}");
    // The adopted intent claims no second position.
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["last_reconcile"]["own"], 1, "{h}");
    let ext = &h[0]["last_reconcile"]["external_positions"];
    assert_eq!(ext.as_array().map(Vec::len), Some(1), "{h}");
    assert_eq!(ext[0]["position_id"], 1002, "{h}");
    f.positions
        .lock()
        .unwrap()
        .retain(|p| p.position_id == 1001);
    // The remainder fills: the broker holds all 100 cents.
    f.positions.lock().unwrap()[0].trade_data.volume = 100;
    f.quote_sequence = vec![];
    let log3 = trade(&pool, &f, 2, Environment::Demo).await;
    assert!(log3.lock().unwrap().orders.is_empty(), "never resent");
    let kinds: Vec<String> = sqlx::query_scalar(
        "SELECT kind FROM trading_events WHERE entry_key = $1 AND kind LIKE 'RECONCILED%' ORDER BY id",
    )
    .bind(&key)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(kinds, vec!["RECONCILED_PARTIAL_FILL", "RECONCILED_FILL"]);
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert!(u.is_empty(), "{u:?}");
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["last_reconcile"]["own"], 1, "{h}");
}

/// Codex on #16: a reconcile can adopt a partial fill before the deal's own
/// execution message is processed; the message then records the same 60
/// cents. They are one fill, not 120 cents: once the position stops out with
/// the remainder still within its grace, the open is not recorded filled in
/// full and its 40-cent remainder stays reserved.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_reconciled_partial_and_its_late_deal_are_one_fill(pool: PgPool) {
    loaded_with(&pool, &[(ETH, "ETHUSD", AssetClass::Crypto, 1)]).await;
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake_on(&[ETH], 2_000);
    f.trader.as_mut().unwrap().balance = 2_000_100;
    f.fill_delay = Some(Duration::from_secs(600));
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0))];
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1);
    let key = trading::entry_key(DEMO, ETH, Utc::now().date_naive() - Days::days(1));
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 1001,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 60,
            trade_side: 1,
            label: Some("jiaoyi-s1".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        price: Some(3611.0),
        stop_loss: Some(3491.0),
        ..Default::default()
    });
    f.fill_delay = None;
    f.quote_sequence = vec![];
    trade(&pool, &f, 2, Environment::Demo).await;
    // The same deal's execution message, processed after that reconcile.
    let deal = store::trading::EventRow {
        account_id: DEMO,
        entry_key: Some(key.clone()),
        client_msg_id: None,
        payload_type: 2126,
        kind: "ORDER_PARTIAL_FILL".into(),
        position_id: Some(1001),
        order_id: Some(9001),
        deal_id: Some(7001),
        symbol_id: Some(ETH),
        side: Some("buy".into()),
        price: Some(3611.0),
        volume: Some(60),
        stop_loss: Some(3491.0),
        label: Some("jiaoyi-s1".into()),
        error_code: None,
        description: None,
        is_server_event: Some(false),
        raw: vec![],
        received_at: Utc::now(),
    };
    assert!(store::trading::record_event(&pool, &deal).await.unwrap());
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert_eq!(u.len(), 1, "{u:?}");
    assert_eq!(u[0].filled, 60, "one fill, not two: {u:?}");
    // The position stops out; the remainder is within its grace.
    f.positions.lock().unwrap().clear();
    trade(&pool, &f, 2, Environment::Demo).await;
    let kinds: Vec<String> = sqlx::query_scalar(
        "SELECT kind FROM trading_events WHERE entry_key = $1 AND kind LIKE 'RECONCILED%' ORDER BY id",
    )
    .bind(&key)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(kinds, vec!["RECONCILED_PARTIAL_FILL"], "not filled in full");
    let u = store::trading::open_intents_unresolved(&pool, DEMO)
        .await
        .unwrap();
    assert_eq!((u.len(), u[0].filled), (1, 60), "{u:?}");
    let pending: i64 = sqlx::query_scalar(
        "SELECT (detail->>'pending')::BIGINT FROM trading_decisions
         WHERE account_id = $1 AND kind = 'reconcile' ORDER BY id DESC LIMIT 1",
    )
    .bind(DEMO)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(pending, 1, "the remainder stays reserved");
}

/// Production read-back 04:26Z: decisions are recorded only when their
/// reason changes, so /health stayed at the first reconcile while the loops
/// ran. A quiet loop's reconcile runs keep its liveness advancing, with the
/// recorded reconcile decision kept once.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_quiet_loop_keeps_its_reconcile_runs_advancing(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.reconcile_secs", json!(30)).await;
    let f = fake_on(&[ETH], 2_000);
    let live = trading::LivenessMap::default();
    trade_with(&pool, &f, 40, Environment::Demo, &live).await;
    let l = live.lock().unwrap().get(&DEMO).cloned().expect("a beat");
    let (start, ran) = (l.started_at.unwrap(), l.last_reconcile_run_at.unwrap());
    assert!(
        ran - start >= Days::seconds(25),
        "a second reconcile ran: {l:?}"
    );
    // The unchanged reason is recorded once.
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trading_decisions WHERE account_id = $1 AND kind = 'reconcile'",
    )
    .bind(DEMO)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
    let loops = live.lock().unwrap().clone();
    let h = trading::health(&pool, &loops).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    assert_eq!(h[0]["liveness"]["stalled_after_secs"], 60, "{h}");
    let recorded: chrono::DateTime<Utc> =
        serde_json::from_value(h[0]["last_reconcile"]["at"].clone()).unwrap();
    assert!(ran - recorded >= Days::seconds(25), "{h}");
}

/// A loop stopped for more than twice its reconcile interval reads stalled;
/// its last quote is shown.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_stopped_loop_reads_stalled_after_twice_its_reconcile_interval(pool: PgPool) {
    loaded(&pool).await;
    // Below the level (3610.00): quotes, no entry.
    let f = fake(vec![(3500.0, 3501.0)]);
    let live = trading::LivenessMap::default();
    let log = trade_with(&pool, &f, 2, Environment::Demo, &live).await;
    assert!(log.lock().unwrap().orders.is_empty());
    let l = live.lock().unwrap().get(&DEMO).cloned().expect("a beat");
    assert!(l.last_quote_at.is_some(), "{l:?}");
    let loops = live.lock().unwrap().clone();
    let h = trading::health(&pool, &loops).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    assert!(h[0]["liveness"]["last_quote_at"].is_string(), "{h}");
    // The loop is stopped. As if its last reconcile ran `age` ago: within
    // 2 × 300 s it is alive, past it stalled.
    let aged = |age: i64| {
        let mut m = loops.clone();
        let e = m.get_mut(&DEMO).unwrap();
        let at = Utc::now() - Days::seconds(age);
        e.started_at = Some(at - Days::seconds(5));
        e.last_reconcile_run_at = Some(at);
        m
    };
    let h = trading::health(&pool, &aged(590)).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "alive", "{h}");
    let h = trading::health(&pool, &aged(610)).await.unwrap();
    assert_eq!(h[0]["liveness"]["state"], "stalled", "{h}");
}

// ------------------------------------------------------------- the instrument
// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")

/// Strategy 1's orders on the demo account, newest first.
async fn s1_orders(pool: &PgPool) -> Vec<store::trading::OrderView> {
    store::trading::orders(
        pool,
        &store::trading::OrdersQuery {
            account_id: Some(DEMO),
            strategy: Some("s1".into()),
            since: None,
            limit: 100,
        },
    )
    .await
    .unwrap()
}

/// One Strategy 1 fill row: kind, outcome, volume, fill and decision price,
/// slippage, position, latency.
type S1Fill = (
    String,
    String,
    Option<i64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<i64>,
    Option<i64>,
);

async fn s1_fills(pool: &PgPool) -> Vec<S1Fill> {
    sqlx::query_as(
        "SELECT kind, outcome, volume, fill_price, decision_price, slippage, position_id, latency_ms
         FROM strategy1_fills ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

fn close(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-9)
}

/// An open filled in two deals (90 at 3611.00, 10 at 3621.00) is listed at
/// their sum and their VWAP 3612.00, 1.00 above the ask it was decided at,
/// with the decision that placed it; `/trading/costs` reports it.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_1_open_in_two_deals_is_listed_at_their_vwap(pool: PgPool) {
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    two_symbols_filled_in(
        &pool,
        vec![(0, 90, 3611.0), (0, 10, 3621.0)],
        false,
        vec![(0, ETH, q(3608.0, 3611.0))],
        2,
    )
    .await;
    let o = s1_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    let r = &o[0];
    assert_eq!(
        (r.strategy.as_str(), r.label.as_str(), r.kind.as_str()),
        ("s1", "jiaoyi-s1", "open")
    );
    assert_eq!(r.order.as_deref(), Some("open"));
    assert_eq!(r.symbol.as_deref(), Some("ETHUSD"));
    assert_eq!(r.side.as_deref(), Some("buy"));
    assert_eq!(
        (r.requested_volume, r.filled_volume),
        (Some(100), Some(100))
    );
    assert_eq!(r.outcome.as_deref(), Some("ORDER_FILLED"));
    assert!(close(r.fill_price, 3612.0), "{r:?}");
    assert_eq!(r.position_id, Some(1001));
    assert!(r.latency_ms.is_some(), "{r:?}");
    assert!(
        close(r.slippage, 1.0),
        "3612.00 against the ask 3611.00: {r:?}"
    );
    assert!(close(r.spread, 3.0), "{r:?}");
    let d = r
        .decision
        .as_ref()
        .expect("the decision it was placed under");
    assert_eq!(d.kind, "sent");
    assert!(
        d.reason
            .starts_with("Strategy 1: tick-triggered trend buy 100 cents"),
        "{d:?}"
    );
    assert_eq!(d.signal["bid"], json!(3608.0), "{d:?}");
    assert_eq!(d.signal["ask"], json!(3611.0), "{d:?}");
    assert_eq!(d.signal["level"], json!(3610.0), "{d:?}");
    let f = s1_fills(&pool).await;
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!((f[0].0.as_str(), f[0].1.as_str()), ("open", "filled"));
    assert_eq!(f[0].2, Some(100));
    assert!(
        close(f[0].3, 3612.0) && close(f[0].4, 3611.0) && close(f[0].5, 1.0),
        "{f:?}"
    );
    let hash: String = sqlx::query_scalar("SELECT prereg_hash FROM strategy1_fills")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hash, HASH);
    let c = store::trading::costs(&pool).await.unwrap();
    let rows = c["rows"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|r| r["strategy"] == "s1")
        .expect("an s1 row");
    assert_eq!(row["label"], "jiaoyi-s1");
    assert_eq!(row["symbol"], "ETHUSD");
    assert_eq!(row["entries_filled"], 1);
    assert!(close(row["entry_slippage_median"].as_f64(), 1.0), "{row}");
    assert!(row["entry_order_to_fill_ms_median"].is_number(), "{row}");
    assert_eq!(row["stop_amends"], 0);
}

/// 60 of 100 cents filled, the remainder cancelled: the order is listed
/// cancelled with its 60 cents filled into its position.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_1_partial_fill_then_cancel_is_listed_with_its_part(pool: PgPool) {
    set(&pool, "trading.trend_open_risk_pct", json!(0.9)).await;
    two_symbols_filled_in(
        &pool,
        vec![(0, 60, 3611.0)],
        true,
        vec![(0, ETH, q(3608.0, 3611.0))],
        2,
    )
    .await;
    let o = s1_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("ORDER_CANCELLED"), "{o:?}");
    assert_eq!(
        (o[0].requested_volume, o[0].filled_volume),
        (Some(100), Some(60))
    );
    assert!(close(o[0].fill_price, 3611.0), "{o:?}");
    assert_eq!(o[0].position_id, Some(1001));
    let f = s1_fills(&pool).await;
    assert_eq!(
        (f[0].1.as_str(), f[0].2, f[0].6),
        ("partly_filled", Some(60), Some(1001)),
        "{f:?}"
    );
}

/// Codex P2 on #18: an open part-filled (60 of 100) whose remainder reconcile
/// later finds filled is listed at the full 100, not at the 60 its recorded
/// deal carries.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_partial_open_reconciled_in_full_is_listed_at_its_full_volume(pool: PgPool) {
    loaded_with(&pool, &[(ETH, "ETHUSD", AssetClass::Crypto, 1)]).await;
    set(&pool, "trading.trend_open_risk_pct", json!(1.0)).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake_on(&[ETH], 2_000);
    f.trader.as_mut().unwrap().balance = 2_000_100;
    f.fill_parts = vec![(0, 60, 3611.0)];
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0))];
    trade(&pool, &f, 3, Environment::Demo).await;
    let o = s1_orders(&pool).await;
    assert_eq!((o.len(), o[0].filled_volume), (1, Some(60)), "{o:?}");
    // The remainder fills: the broker holds all 100 cents.
    f.positions.lock().unwrap()[0].trade_data.volume = 100;
    f.fill_parts = vec![];
    f.quote_sequence = vec![];
    trade(&pool, &f, 2, Environment::Demo).await;
    let o = s1_orders(&pool).await;
    assert_eq!(o[0].outcome.as_deref(), Some("RECONCILED_FILL"), "{o:?}");
    assert_eq!(
        (o[0].requested_volume, o[0].filled_volume),
        (Some(100), Some(100)),
        "{o:?}"
    );
}

/// An open with no answer in the wait, found filled at the next session's
/// reconcile: listed as RECONCILED_FILL at the broker's 5 cents and price.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unconfirmed_strategy_1_open_adopted_at_reconcile_is_listed(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.order_wait_secs", json!(1)).await;
    let mut f = fake(vec![(3608.0, 3611.0)]);
    f.fill_delay = Some(Duration::from_secs(600));
    trade(&pool, &f, 2, Environment::Demo).await;
    let fl = s1_fills(&pool).await;
    assert_eq!(
        (fl[0].1.as_str(), fl[0].2, fl[0].7),
        ("unconfirmed", None, None),
        "{fl:?}"
    );
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 1001,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 5,
            trade_side: 1,
            label: Some("jiaoyi-s1".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        price: Some(3611.0),
        stop_loss: Some(3491.0),
        ..Default::default()
    });
    f.fill_delay = None;
    trade(&pool, &f, 2, Environment::Demo).await;
    let o = s1_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("RECONCILED_FILL"), "{o:?}");
    assert_eq!(o[0].filled_volume, Some(5));
    assert!(close(o[0].fill_price, 3611.0), "{o:?}");
    assert_eq!(o[0].position_id, Some(1001));
}

/// A broker rejection is listed with nothing filled. A gate refusal sends no
/// frame and writes no intent, so it is no order; its decision carries the
/// entry key it would have had.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_rejected_strategy_1_open_is_listed_and_a_refused_one_is_found_by_its_key(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake(vec![(3608.0, 3611.0)]);
    f.reject_orders = Some("MARKET_CLOSED".into());
    trade(&pool, &f, 2, Environment::Demo).await;
    let o = s1_orders(&pool).await;
    assert_eq!(o.len(), 1, "{o:?}");
    assert_eq!(o[0].outcome.as_deref(), Some("ORDER_REJECTED"), "{o:?}");
    assert_eq!(
        (o[0].filled_volume, o[0].fill_price, o[0].position_id),
        (Some(0), None, None)
    );
    assert_eq!(s1_fills(&pool).await[0].1, "rejected");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_1_open_the_gate_refuses_is_no_order_but_its_decision_has_the_key(pool: PgPool) {
    loaded(&pool).await;
    let f = fake(vec![(3608.0, 3611.0)]);
    f.positions.lock().unwrap().push(ProtoOaPosition {
        position_id: 77,
        trade_data: ProtoOaTradeData {
            symbol_id: ETH,
            volume: 300,
            trade_side: 2,
            label: Some("bot-trade".into()),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        stop_loss: Some(4000.0),
        ..Default::default()
    });
    trade(&pool, &f, 2, Environment::Demo).await;
    assert!(s1_orders(&pool).await.is_empty());
    let key = trading::entry_key(DEMO, ETH, Utc::now().date_naive() - Days::days(1));
    let refused: Vec<String> = sqlx::query_scalar(
        "SELECT detail->>'entry_key' FROM trading_decisions WHERE kind = 'refused'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(refused, vec![key]);
}

/// ETHUSD's position 1001 as the fake fills a 5-cent buy at 3611.00 (stop
/// 3491.00), and the broker's own stop-loss fill of it at `price`.
fn stop_hit(price: f64) -> jiaoyi_ctrader::ReceivedExecution {
    jiaoyi_ctrader::testkit::server_fill(
        DEMO,
        ProtoOaPosition {
            position_id: 1001,
            trade_data: ProtoOaTradeData {
                symbol_id: ETH,
                volume: 5,
                trade_side: 1,
                label: Some("jiaoyi-s1".into()),
                ..Default::default()
            },
            position_status: 1,
            swap: 0,
            price: Some(3611.0),
            stop_loss: Some(3491.0),
            ..Default::default()
        },
        901,
        price,
        5,
        true,
        false,
    )
}

/// The broker's own stop of a Strategy 1 position, received by the running
/// loop: recorded once as a broker exit at 3489.00 against its stop 3491.00
/// (2.00 adverse), listed with the open, counted in `/trading/costs`, and in
/// today's trades.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_strategy_1_stop_the_broker_executes_is_recorded_listed_and_costed(pool: PgPool) {
    loaded(&pool).await;
    let mut f = fake(vec![(3608.0, 3611.0)]);
    let m = stop_hit(3489.0);
    // The same deal twice: recorded once.
    f.execution_sequence = vec![
        (800, m.payload_type, m.payload.clone()),
        (900, m.payload_type, m.payload.clone()),
    ];
    trade(&pool, &f, 2, Environment::Demo).await;
    let fl = s1_fills(&pool).await;
    assert_eq!(fl.len(), 2, "{fl:?}");
    assert_eq!(
        (fl[1].0.as_str(), fl[1].1.as_str()),
        ("broker_exit", "filled")
    );
    assert_eq!((fl[1].2, fl[1].6), (Some(5), Some(1001)));
    assert!(
        close(fl[1].3, 3489.0) && close(fl[1].4, 3491.0) && close(fl[1].5, 2.0),
        "{fl:?}"
    );
    let o = s1_orders(&pool).await;
    assert_eq!(o.len(), 2, "{o:?}");
    let exit = o
        .iter()
        .find(|r| r.kind == "broker_exit")
        .expect("the exit");
    assert_eq!(exit.decision.as_ref().unwrap().reason, "stop_loss");
    // The body `/trading/orders` answers: no money in it.
    let text = jiaoyi::http::orders_body(&o).to_string().to_lowercase();
    for word in [
        "balance",
        "equity",
        "pnl",
        "profit",
        "commission",
        "\"risk\"",
        "max_risk",
    ] {
        assert!(!text.contains(word), "{word}: {text}");
    }
    assert_eq!(
        (exit.filled_volume, exit.position_id),
        (Some(5), Some(1001))
    );
    assert!(close(exit.slippage, 2.0), "{exit:?}");
    let c = store::trading::costs(&pool).await.unwrap();
    let row = &c["rows"][0];
    assert_eq!(row["strategy"], "s1");
    assert_eq!(
        (row["entries_filled"].clone(), row["exits_filled"].clone()),
        (json!(1), json!(1))
    );
    assert_eq!(row["broker_exits_filled"], 1);
    assert!(close(row["exit_slippage_median"].as_f64(), 2.0), "{row}");
    assert!(close(row["entry_slippage_median"].as_f64(), 0.0), "{row}");
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    let trades = h[0]["trades_today"].as_array().unwrap();
    assert_eq!(trades.len(), 2, "{h}");
    assert_eq!(trades[0]["kind"], "broker_exit");
}

/// Run the Strategy 1 loop on `f` for at most `secs`: its result if it ended
/// by itself, `None` if it was still running.
async fn session_result(pool: &PgPool, f: &FakeBroker, secs: u64) -> Option<anyhow::Result<()>> {
    let (conn, _log) = f.clone().spawn();
    let session = loader::authorise(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let mut d = trading::decisions(pool, DEMO);
    let list = allowlist();
    let live = trading::LivenessMap::default();
    tokio::select! {
        r = trading::run_session(pool, &list, Environment::Demo, &session, &mut d, &live, &mut ends, &mut stop) => Some(r),
        _ = tokio::time::sleep(Duration::from_secs(secs)) => {
            trigger.fire();
            None
        }
    }
}

/// `strategy1_fills` refuses rows of `kind`.
async fn refuse_s1_fills(pool: &PgPool, kind: &str) {
    sqlx::query(&format!(
        "CREATE FUNCTION refuse_s1_fill() RETURNS trigger LANGUAGE plpgsql AS $f$
         BEGIN
           IF NEW.kind = '{kind}' THEN RAISE EXCEPTION 'no {kind} record'; END IF;
           RETURN NEW;
         END $f$"
    ))
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TRIGGER refuse_s1_fill BEFORE INSERT ON strategy1_fills
         FOR EACH ROW EXECUTE FUNCTION refuse_s1_fill()",
    )
    .execute(pool)
    .await
    .unwrap();
}

/// A Strategy 1 record that cannot be written ends the session with the
/// error (it then reconnects and reconciles), as Strategy 3's does: never
/// a silent loss. The open's, the trailing amend's and a broker exit's.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_lost_strategy_1_open_record_ends_the_session(pool: PgPool) {
    loaded(&pool).await;
    refuse_s1_fills(&pool, "open").await;
    let r = session_result(&pool, &fake(vec![(3608.0, 3611.0)]), 3).await;
    let e = r.expect("the session ended").expect_err("with the error");
    assert!(format!("{e:#}").contains("no open record"), "{e:#}");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_lost_strategy_1_amend_record_ends_the_session(pool: PgPool) {
    loaded(&pool).await;
    refuse_s1_fills(&pool, "trail").await;
    let f = fake(vec![(3608.0, 3611.0), (3700.0, 3701.0)]);
    let r = session_result(&pool, &f, 3).await;
    let e = r.expect("the session ended").expect_err("with the error");
    assert!(format!("{e:#}").contains("no trail record"), "{e:#}");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_lost_strategy_1_broker_exit_record_ends_the_session(pool: PgPool) {
    loaded(&pool).await;
    refuse_s1_fills(&pool, "broker_exit").await;
    let mut f = fake(vec![(3608.0, 3611.0)]);
    let m = stop_hit(3489.0);
    f.execution_sequence = vec![(800, m.payload_type, m.payload.clone())];
    let r = session_result(&pool, &f, 3).await;
    let e = r.expect("the session ended").expect_err("with the error");
    assert!(format!("{e:#}").contains("no broker_exit record"), "{e:#}");
}

// ---------------------------------------------------------------- limits and the owner's pause
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)

/// The owner's word on Strategy 1's pause on this account (1 paused, 0 not).
async fn owner_sets_pause(pool: &PgPool, value: i64) {
    settings::change(
        pool,
        "trading.s1_paused",
        Some(DEMO),
        Some(json!(value)),
        "owner",
        "owner, in writing (test)",
    )
    .await
    .unwrap();
}

/// `n` closed Strategy 1 trades of −1R on this account (the gate's $6.00,
/// the broker's net −$6.00), two days ago, on a symbol not traded here.
async fn closed_losses(pool: &PgPool, n: i32) {
    sqlx::query(
        "INSERT INTO trading_intents (account_id, entry_key, client_msg_id, kind, symbol_id,
            side, volume, risk, max_risk)
         SELECT $1, 's1:old:' || n, 'c-old-' || n, 'open', 99, 'buy', 10, 6.0, 100
         FROM generate_series(1, $2) n",
    )
    .bind(DEMO)
    .bind(n)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO trading_events (account_id, entry_key, client_msg_id, payload_type, kind,
            position_id, raw, received_at)
         SELECT $1, 's1:old:' || n, 'c-old-' || n, 2126, 'ORDER_FILLED', 900000 + n, '\\x',
                now() - interval '2 days'
         FROM generate_series(1, $2) n",
    )
    .bind(DEMO)
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

/// "only human can pause trade" (owner, 25-10): with ETHUSD held, the owner
/// pauses Strategy 1 on this account. ETHUSD's stop is still trailed; a
/// BTCUSD breakout sends nothing, recorded once, and its signal is not spent,
/// so it enters once the owner lifts the pause.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_owners_pause_stops_new_entries_and_the_trail_still_runs(pool: PgPool) {
    loaded_with(
        &pool,
        &[
            (ETH, "ETHUSD", AssetClass::Crypto, 1),
            (BTC, "BTCUSD", AssetClass::Crypto, 1),
        ],
    )
    .await;
    let mut f = fake_on(&[ETH, BTC], 2_000);
    f.fill_quotes = HashMap::from([(ETH, (3608.0, 3611.0)), (BTC, (3608.0, 3611.0))]);
    f.quote_sequence = vec![(0, ETH, q(3608.0, 3611.0))];
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "ETHUSD entered");
    owner_sets_pause(&pool, 1).await;
    // ETHUSD's bid rises to 3700.00; BTCUSD's ask reaches its level (3610.00).
    f.quote_sequence = vec![(0, ETH, q(3700.0, 3701.0)), (500, BTC, q(3608.0, 3611.0))];
    let live = trading::LivenessMap::default();
    let log = trade_with(&pool, &f, 3, Environment::Demo, &live).await;
    let l = log.lock().unwrap().clone();
    let d = decisions(&pool).await;
    assert!(l.orders.is_empty(), "paused: no new entry: {d:?}");
    assert_eq!(l.amends.len(), 1, "the trail still runs: {d:?}");
    assert_eq!(l.amends[0].stop_loss, Some(3580.0));
    let paused: Vec<&(String, String)> = d.iter().filter(|(k, _)| k == "paused").collect();
    assert_eq!(paused.len(), 1, "{d:?}");
    assert!(paused[0].1.contains("paused by the owner"));
    // /health: paused, by whom, and how near BTCUSD is to its level.
    let map = live.lock().unwrap().clone();
    let h = trading::health(&pool, &map).await.unwrap();
    assert_eq!(h[0]["trades"], "paused by the owner: no new entry");
    assert_eq!(h[0]["owner_pause"]["paused"], true);
    assert_eq!(h[0]["owner_pause"]["changed_by"], "owner");
    let btc = &h[0]["trigger_distance"]["BTCUSD"];
    assert_eq!(btc["filter"], "buy");
    assert_eq!(btc["level"], 3610.0);
    // (3610.00 − 3611.00) / 40.00 = −0.025: through the level.
    assert!(
        (btc["distance_atr"].as_f64().unwrap() + 0.025).abs() < 1e-9,
        "{btc}"
    );
    // Lifted by the owner: BTCUSD's signal is still there.
    owner_sets_pause(&pool, 0).await;
    f.quote_sequence = vec![(0, BTC, q(3608.0, 3611.0))];
    let log = trade(&pool, &f, 2, Environment::Demo).await;
    let orders = log.lock().unwrap().orders.clone();
    assert_eq!(orders.len(), 1, "{:?}", decisions(&pool).await);
    assert_eq!(orders[0].symbol_id, BTC);
    let w = pause_writes(&pool).await;
    assert_eq!(w.len(), 2);
    assert!(
        w.iter()
            .all(|(k, by)| k == "trading.s1_paused" && by == "owner"),
        "{w:?}"
    );
}

/// A limit reached is an alert: recorded once, on /health, and Strategy 1
/// keeps trading. Nothing pauses it.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_limit_reached_is_flagged_and_strategy_1_keeps_trading(pool: PgPool) {
    loaded(&pool).await;
    closed_losses(&pool, 10).await;
    let log = trade(&pool, &fake(vec![(3608.0, 3611.0)]), 2, Environment::Demo).await;
    let d = decisions(&pool).await;
    assert_eq!(log.lock().unwrap().orders.len(), 1, "still trading: {d:?}");
    let reached: Vec<&(String, String)> = d.iter().filter(|(k, _)| k == "limit_reached").collect();
    assert_eq!(reached.len(), 1, "{d:?}");
    assert!(reached[0].1.contains("drawdown 10.00R ≥ 10R"), "{d:?}");
    assert!(pause_writes(&pool).await.is_empty(), "nothing paused");
    let h = trading::health(&pool, &HashMap::new()).await.unwrap();
    assert_eq!(h[0]["trades"], "yes");
    assert_eq!(h[0]["performance"]["owner_decision_needed"], true);
    assert_eq!(h[0]["performance"]["closed_trades"], 10);
}

/// The distance to Strategy 1's trigger, in ATR from the allowed side's
/// 50-day level: positive below it, zero on it (a trigger), negative through it.
#[test]
fn the_distance_to_the_level_is_in_atr_and_turns_negative_through_it() {
    let at = Utc::now();
    let buy = trading::Levels {
        last_completed: at.date_naive(),
        side: Some(jiaoyi_ctrader::trading::Side::Buy),
        level: 3610.0,
        atr: 40.0,
        ema_fast: 2.0,
        ema_slow: 1.0,
    };
    let sell = trading::Levels {
        side: Some(jiaoyi_ctrader::trading::Side::Sell),
        ..buy.clone()
    };
    for (l, bid, ask, want) in [
        (&buy, 3565.0, 3570.0, 1.0),
        (&buy, 3609.0, 3610.0, 0.0),
        (&buy, 3645.0, 3650.0, -1.0),
        (&sell, 3650.0, 3655.0, 1.0),
        (&sell, 3570.0, 3575.0, -1.0),
    ] {
        let x = trading::trigger_distance(&Ok(l.clone()), Some(bid), Some(ask), at);
        let d = x["distance_atr"].as_f64().unwrap();
        assert!((d - want).abs() < 1e-9, "{x}");
        assert_eq!(
            trading::triggered(l, Some(bid), Some(ask)).is_some(),
            d <= 0.0,
            "the sign agrees with the trigger: {x}"
        );
    }
    let none = trading::Levels {
        side: None,
        ..buy.clone()
    };
    let x = trading::trigger_distance(&Ok(none), Some(1.0), Some(2.0), at);
    assert_eq!(x["distance_atr"], serde_json::Value::Null);
    assert!(x["reason"].as_str().unwrap().contains("EMAs are equal"));
    let x = trading::trigger_distance(&Err("fewer than 120 days".into()), None, None, at);
    assert_eq!(x["distance_atr"], serde_json::Value::Null);
    assert!(x["reason"]
        .as_str()
        .unwrap()
        .contains("fewer than 120 days"));
    let x = trading::trigger_distance(&Ok(buy), Some(3600.0), None, at);
    assert_eq!(x["distance_atr"], serde_json::Value::Null);
    assert!(x["reason"].as_str().unwrap().contains("no quote"));
}
