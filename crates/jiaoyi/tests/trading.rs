//! The H1 trader and the order gate against a real PostgreSQL and the
//! scripted broker (owner, 25-10-2026): every switch off by default, the edge
//! gate, the reviewed settings seed, one entry per completed day, the trail,
//! and positions this program did not open.

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
const ETH: i64 = 22;
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
    store::upsert_account(pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    let s = ProtoOaSymbol {
        symbol_id: ETH,
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
            "ETHUSD",
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
        "ETHUSD",
        AssetClass::Crypto,
        &Resolution::Resolved { symbol_id: ETH },
    )
    .await
    .unwrap();
    store::finish_run(
        pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(json!([{"id": ETH, "quote_asset_id": 1, "base_asset_id": 9}])),
            assets: Some(json!([{"asset_id": 1, "name": "USD"}, {"asset_id": 9, "name": "ETH"}])),
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

/// 160 completed daily bars ending yesterday, rising 10 a day: EMA(50) >
/// EMA(100), the level is the last high (3610.00), ATR(20) = 40.
fn fake(quotes: Vec<(f64, f64)>) -> FakeBroker {
    let today = Utc::now().date_naive();
    let bars = (0..160)
        .map(|i| {
            let d = today - Days::days(160 - i);
            let b = 200_000 + 1_000 * i; // ×1/100 → 2000.00 + 10 a day, in 1/100000
            d1_bar(
                d,
                b * 1_000,
                (b + 2_000) * 1_000,
                (b - 2_000) * 1_000,
                (b + 500) * 1_000,
                10,
            )
        })
        .collect();
    FakeBroker {
        account_id: DEMO,
        bars: HashMap::from([(ETH, bars)]),
        quotes: HashMap::from([(
            ETH,
            quotes
                .iter()
                .map(|(b, a)| {
                    (
                        Some((b * 100_000.0).round() as u64),
                        Some((a * 100_000.0).round() as u64),
                    )
                })
                .collect(),
        )]),
        fill_quotes: HashMap::from([(ETH, quotes[0])]),
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

/// Run the trading loop on `f` for `secs`, then stop it.
async fn trade(
    pool: &PgPool,
    f: &FakeBroker,
    secs: u64,
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
        r = trading::run_session(pool, &list, &session, &mut d, &mut ends, &mut stop) => {
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
async fn every_accounts_switch_is_off_after_the_migration(pool: PgPool) {
    for id in [43002148, 49236051, 46130058, 47790949] {
        let sw = trading::switches(&pool, id).await.unwrap();
        assert!(!sw.enabled, "{id}");
        assert!(sw.require_edge_pass, "{id}");
        assert!(!sw.allow_min_volume, "{id}");
        assert_eq!(sw.risk_pct, 0.6);
        assert_eq!(sw.min_volume_max_risk_pct, 10.0);
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_edge_gate_opens_only_on_a_held_out_pass(pool: PgPool) {
    let sw = trading::switches(&pool, DEMO).await.unwrap();
    let g = trading::edge_gate(&pool, &sw).await.unwrap();
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
    assert!(trading::edge_gate(&pool, &sw).await.unwrap().is_err());
    trial("FAIL", "held_out", HASH).await;
    let g = trading::edge_gate(&pool, &sw).await.unwrap();
    assert!(g.unwrap_err().contains("is FAIL"));
    trial("PASS", "held_out", HASH).await;
    assert!(trading::edge_gate(&pool, &sw).await.unwrap().is_ok());
    // Not required: open whatever the trials say.
    set(&pool, "trading.require_edge_pass", json!(0)).await;
    sqlx::query("DELETE FROM research_trials")
        .execute(&pool)
        .await
        .ok();
    let sw = trading::switches(&pool, DEMO).await.unwrap();
    assert!(trading::edge_gate(&pool, &sw).await.unwrap().is_ok());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_reviewed_seed_change_is_applied_once_and_logged(pool: PgPool) {
    let text = r#"[
      {"id": "2026-10-25-demo-on", "key": "trading.enabled", "account_id": 49236051, "value": 1,
       "changed_by": "owner", "reason": "owner 25-10: demo trades at 1 PM"},
      {"id": "2026-10-25-bad", "key": "trading.enabled", "account_id": 46130058, "value": 2,
       "changed_by": "owner", "reason": "out of bounds"}
    ]"#;
    let applied = trading::apply_seed(&pool, text).await.unwrap();
    assert_eq!(applied, vec!["2026-10-25-demo-on".to_string()]);
    assert!(trading::switches(&pool, DEMO).await.unwrap().enabled);
    assert!(!trading::switches(&pool, 46130058).await.unwrap().enabled);
    assert!(!trading::switches(&pool, 43002148).await.unwrap().enabled);
    let log: Vec<(String, String)> = sqlx::query_as(
        "SELECT changed_by, reason FROM settings_changes WHERE key = 'trading.enabled' AND account_id = $1",
    )
    .bind(DEMO)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        log,
        vec![("owner".into(), "owner 25-10: demo trades at 1 PM".into())]
    );
    // Applied once: a restart changes nothing.
    assert!(trading::apply_seed(&pool, text).await.unwrap().is_empty());
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM settings_changes WHERE key = 'trading.enabled' AND account_id IS NOT NULL")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_trader_sends_one_h1_entry_and_trails_it(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.enabled", json!(1)).await;
    set(&pool, "trading.require_edge_pass", json!(0)).await;
    // The ask reaches the level (3610.00); then the bid rises to 3700.00.
    let f = fake(vec![(3608.0, 3611.0), (3700.0, 3701.0)]);
    let log = trade(&pool, &f, 3).await;
    let orders = log.lock().unwrap().orders.clone();
    assert_eq!(orders.len(), 1, "{:?}", decisions(&pool).await);
    let o = &orders[0];
    assert_eq!(o.symbol_id, ETH);
    assert_eq!(o.trade_side, 1, "buy");
    // 3 × ATR(20) = 120.00 → 12,000,000 in 1/100000.
    assert_eq!(o.relative_stop_loss, Some(12_000_000));
    // 0.6% of $1,002.44 = $6.0146; one cent risks 0.01 × 120 = $1.20 → 5 cents.
    assert_eq!(o.volume, 5);
    assert_eq!(o.label.as_deref(), Some("jiaoyi-h1"));
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
    let log2 = trade(&pool, &f, 2).await;
    assert!(log2.lock().unwrap().orders.is_empty());
    // /health shows the switch and the decision, and no money.
    let h = trading::health(&pool).await.unwrap();
    assert_eq!(h[0]["enabled"], true);
    assert_eq!(h[0]["edge_gate"], "not required");
    assert!(!h.to_string().contains("balance"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_switch_off_sends_nothing(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.require_edge_pass", json!(0)).await;
    let log = trade(&pool, &fake(vec![(3608.0, 3611.0)]), 2).await;
    assert!(log.lock().unwrap().orders.is_empty());
    assert!(decisions(&pool)
        .await
        .iter()
        .any(|(k, r)| k == "refused" && r.contains("trading.enabled is off")));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unmet_edge_gate_holds_the_signal(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.enabled", json!(1)).await;
    let log = trade(&pool, &fake(vec![(3608.0, 3611.0)]), 2).await;
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
async fn a_position_this_program_did_not_open_blocks_the_entry_and_is_reported(pool: PgPool) {
    loaded(&pool).await;
    set(&pool, "trading.enabled", json!(1)).await;
    set(&pool, "trading.require_edge_pass", json!(0)).await;
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
    let log = trade(&pool, &f, 2).await;
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
