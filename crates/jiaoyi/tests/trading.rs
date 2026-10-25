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
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker, GrantEntry};
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
fn q(bid: f64, ask: f64) -> (Option<u64>, Option<u64>) {
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
    let (conn, log) = f.clone().spawn();
    let session = loader::authorise(pool, &conn, &broker(), &permitted())
        .await
        .unwrap();
    let mut ends = conn.session_ends();
    let (trigger, mut stop) = shutdown::channel();
    let mut d = trading::decisions(pool, DEMO);
    let list = allowlist();
    tokio::select! {
        r = trading::run_session(pool, &list, env, &session, &mut d, &mut ends, &mut stop) => {
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
    let h = trading::health(&pool).await.unwrap();
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
    let h = trading::health(&pool).await.unwrap();
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
    assert_eq!(keys, vec!["accepted", "silent"]);
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
    let h = trading::health(&pool).await.unwrap();
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
    let h = trading::health(&pool).await.unwrap();
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
