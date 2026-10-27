//! The factory's limits against a real PostgreSQL (owner, 25-10-2026): net R
//! from the broker's own deals, the −10R drawdown and the profit factor below
//! 0.8 after 50 closed trades. The limits alert ("LIMIT REACHED: owner's
//! decision needed") and never pause: "only human can pause trade".
//! Expected values come from the seeded deals (self-derived); the formula's
//! grounding is the broker's close P&L fields.
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use chrono::{DateTime, Duration, Utc};
use http_body_util::BodyExt;
use jiaoyi::http::{router, AppState};
use jiaoyi::{limits, trading};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_store::history::DealRow;
use jiaoyi_store::{self as store, settings, trading as ts};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const A: i64 = 49236051;
const B: i64 = 46130058;
const BTC: i64 = 22;

/// Three days ago, in whole seconds (as the database keeps it).
fn t0() -> DateTime<Utc> {
    DateTime::from_timestamp(Utc::now().timestamp() - 3 * 86_400, 0).unwrap()
}

fn deal(id: i64, position: i64, filled: i64, at: DateTime<Utc>) -> DealRow {
    DealRow {
        deal_id: id,
        order_id: id,
        position_id: position,
        broker_symbol_id: BTC,
        volume: filled,
        filled_volume: filled,
        trade_side: "BUY".into(),
        deal_status: "FILLED".into(),
        created_at: at,
        executed_at: at,
        last_update_at: None,
        execution_price: Some(60_000.0),
        margin_rate: None,
        // The opening side's commission. The close's commission covers both
        // sides, so this is not taken again.
        commission: Some(-20),
        money_digits: Some(2),
        base_to_usd_rate: None,
        is_closing: false,
        close_entry_price: None,
        close_gross_profit: None,
        close_swap: None,
        close_commission: None,
        close_balance: None,
        close_quote_to_deposit_rate: None,
        close_closed_volume: None,
        close_balance_version: None,
        close_money_digits: None,
        close_pnl_conversion_fee: None,
        raw: vec![],
    }
}

/// One position of `key`'s strategy on `account`: an open of `requested`
/// cents for which the gate recorded `risk`, filled `filled` cents into
/// `position` and, when `net` is given, closed for that many cents net (gross
/// profit less a 40-cent commission for both sides) at `at`.
#[allow(clippy::too_many_arguments)]
async fn position(
    pool: &PgPool,
    account: i64,
    key: &str,
    position: i64,
    requested: i64,
    filled: i64,
    risk: Option<f64>,
    net: Option<i64>,
    at: DateTime<Utc>,
) {
    assert!(ts::record_intent(
        pool,
        &ts::IntentRow {
            account_id: account,
            entry_key: key.into(),
            client_msg_id: format!("c-{key}"),
            kind: "open".into(),
            symbol_id: Some(BTC),
            position_id: None,
            side: Some("buy".into()),
            volume: Some(requested),
            relative_stop: Some(45_000_000),
            stop_price: None,
            take_profit: None,
            risk,
            max_risk: 100.0,
            settings_version: None,
        },
    )
    .await
    .unwrap());
    ts::record_event(
        pool,
        &ts::EventRow {
            account_id: account,
            entry_key: Some(key.into()),
            client_msg_id: Some(format!("c-{key}")),
            payload_type: 2126,
            kind: if filled < requested {
                "ORDER_PARTIAL_FILL"
            } else {
                "ORDER_FILLED"
            }
            .into(),
            position_id: Some(position),
            order_id: None,
            deal_id: None,
            symbol_id: Some(BTC),
            side: Some("buy".into()),
            price: Some(60_000.0),
            volume: Some(filled),
            stop_loss: None,
            label: None,
            error_code: None,
            description: None,
            is_server_event: None,
            raw: vec![],
            received_at: at - Duration::minutes(30),
        },
    )
    .await
    .unwrap();
    let mut deals = vec![deal(
        position * 10,
        position,
        filled,
        at - Duration::minutes(30),
    )];
    if let Some(net) = net {
        let mut c = deal(position * 10 + 1, position, filled, at);
        c.trade_side = "SELL".into();
        c.is_closing = true;
        c.close_gross_profit = Some(net + 40);
        c.close_swap = Some(0);
        c.close_commission = Some(-40);
        c.close_money_digits = Some(2);
        deals.push(c);
    }
    store::history::upsert_deals(pool, account, &deals, Utc::now())
        .await
        .unwrap();
}

/// `n` closed Strategy 1 trades on `account` of `r` R each (risk $6.00),
/// numbered from `first`, a minute apart.
async fn trades(pool: &PgPool, account: i64, code: &str, first: i64, rs: &[f64]) {
    for (i, r) in rs.iter().enumerate() {
        let n = first + i as i64;
        let net = (r * 600.0).round() as i64;
        position(
            pool,
            account,
            &format!("{code}:{account}:{BTC}:{n}"),
            account % 1_000 * 100_000 + n,
            10,
            10,
            Some(6.0),
            Some(net),
            t0() + Duration::minutes(n),
        )
        .await;
    }
}

async fn limit_decisions(pool: &PgPool, account: i64) -> Vec<(String, String, Value)> {
    sqlx::query_as(
        "SELECT kind, reason, detail FROM trading_decisions
         WHERE account_id = $1 AND kind LIKE '%limit%' ORDER BY id",
    )
    .bind(account)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Every write of a pause key not made by a migration.
async fn pause_writes(pool: &PgPool) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT key, changed_by FROM settings_changes
         WHERE key = ANY($1) AND changed_by <> 'migration' ORDER BY version",
    )
    .bind(&limits::PAUSE_KEYS[..])
    .fetch_all(pool)
    .await
    .unwrap()
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_limits_and_the_owners_pause_are_seeded_as_alerts(pool: PgPool) {
    let l = limits::limits(&pool, A).await.unwrap();
    assert_eq!(
        l,
        limits::Limits {
            drawdown_r: 10.0,
            pf_floor: 0.8,
            pf_min_trades: 50
        }
    );
    let snap = settings::snapshot(&pool, Some(A)).await.unwrap();
    for k in limits::PAUSE_KEYS {
        assert_eq!(snap.integer(k).unwrap(), 0, "{k}: not paused");
    }
    // No on/off switch is named "enabled" (owner, 24-10); the pause is the owner's.
    let n: i64 =
        sqlx::query_scalar("SELECT count(*) FROM setting_definitions WHERE key LIKE '%enabled%'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn net_r_is_the_brokers_net_over_the_gates_risk_scaled_by_the_fill(pool: PgPool) {
    let t = t0();
    // +2R: the gate's $6.00, net +$12.00 (gross $12.40, commission $0.40 for
    // both sides; the opening deal's $0.20 is not taken again).
    position(&pool, A, "s1:a:1", 101, 10, 10, Some(6.0), Some(1_200), t).await;
    // Half filled: $6.00 was for 10 cents, so the 5 filled risk $3.00 and a
    // net −$3.00 is −1R (−0.5R unscaled).
    let t2 = t + Duration::minutes(1);
    position(&pool, A, "s1:a:2", 102, 10, 5, Some(6.0), Some(-300), t2).await;
    // Strategy 3's on the same account, and Strategy 1's on another.
    let t3 = t + Duration::minutes(2);
    position(&pool, A, "s3:a:1", 103, 10, 10, Some(6.0), Some(-600), t3).await;
    position(&pool, B, "s1:b:1", 104, 10, 10, Some(6.0), Some(-600), t3).await;
    // Still open, and closed without a recorded risk.
    position(&pool, A, "s1:a:3", 105, 10, 10, Some(6.0), None, t3).await;
    let t4 = t + Duration::minutes(4);
    position(&pool, A, "s1:a:4", 106, 10, 10, None, Some(600), t4).await;

    let (closed, unmeasured) = ts::closed_trades(&pool, A, "s1:").await.unwrap();
    let rs: Vec<f64> = closed.iter().map(|c| c.r).collect();
    assert_eq!(rs.len(), 2, "{closed:?}");
    assert!(near(rs[0], 2.0), "{rs:?}");
    assert!(near(rs[1], -1.0), "partial fill scales the risk: {rs:?}");
    assert_eq!(closed[1].position_id, 102);
    assert_eq!(unmeasured, 1, "no recorded risk");

    let p = ts::performance(&pool, A, "s1").await.unwrap();
    assert_eq!((p.trades, p.unmeasured), (2, 1));
    assert!(near(p.net_r, 1.0));
    assert!(near(p.profit_factor.unwrap(), 2.0));
    assert!(near(p.max_drawdown_r, 1.0) && near(p.current_drawdown_r, 1.0));
    assert_eq!(p.last_closed_at, Some(t2));
    // Each strategy and each account on its own.
    let s3 = ts::performance(&pool, A, "s3").await.unwrap();
    assert_eq!(s3.trades, 1);
    assert!(near(s3.net_r, -1.0));
    let b = ts::performance(&pool, B, "s1").await.unwrap();
    assert_eq!(b.trades, 1);
    assert!(near(b.max_drawdown_r, 1.0));
    assert!(b.profit_factor.is_some_and(|pf| near(pf, 0.0)));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_10r_drawdown_flags_the_limit_once_and_pauses_nothing(pool: PgPool) {
    // Nine losses: 9R down, inside the limit.
    trades(&pool, A, "s1", 1, &[-1.0; 9]).await;
    let x = json!({"strategy": "test"});
    let hit = limits::check(&pool, A, &limits::S1, x.clone(), None)
        .await
        .unwrap();
    assert!(hit.is_empty(), "9R: {hit:?}");
    assert!(limit_decisions(&pool, A).await.is_empty());
    // The tenth reaches 10R.
    trades(&pool, A, "s1", 10, &[-1.0]).await;
    let hit = limits::check(&pool, A, &limits::S1, x.clone(), Some(7))
        .await
        .unwrap();
    assert_eq!(hit, vec!["drawdown_r"]);
    let d = limit_decisions(&pool, A).await;
    assert_eq!(d.len(), 1, "{d:?}");
    let (kind, reason, detail) = &d[0];
    assert_eq!(kind, "limit_reached");
    assert!(
        reason.contains("LIMIT REACHED") && reason.contains("owner's decision needed"),
        "{reason}"
    );
    assert!(reason.contains("drawdown 10.00R ≥ 10R"), "{reason}");
    assert!(reason.contains("only the owner pauses"), "{reason}");
    assert_eq!(detail["limits"], json!(["drawdown_r"]));
    assert_eq!(detail["closed_trades"], 10);
    assert_eq!(detail["strategy"], "test");
    // Checked again (every reconcile), and after a win that leaves the
    // largest fall at 10R: recorded once.
    limits::check(&pool, A, &limits::S1, x.clone(), None)
        .await
        .unwrap();
    trades(&pool, A, "s1", 11, &[3.0]).await;
    let hit = limits::check(&pool, A, &limits::S1, x.clone(), None)
        .await
        .unwrap();
    assert_eq!(hit, vec!["drawdown_r"], "the largest fall stays reached");
    assert_eq!(limit_decisions(&pool, A).await.len(), 1);
    // The alert pauses nothing: no pause written, the trader reads unpaused.
    assert!(pause_writes(&pool).await.is_empty());
    let r = trading::rules(&pool, A, Environment::Demo).await.unwrap();
    assert!(!r.paused);
    let r = jiaoyi::strategy3::rules(&pool, A, Environment::Demo)
        .await
        .unwrap();
    assert!(!r.paused);
    // /health: the numbers and the alert.
    let h = limits::health(&pool, A, &limits::S1).await.unwrap();
    assert_eq!(h["owner_decision_needed"], true);
    assert_eq!(h["limit_reached"][0]["limit"], "drawdown_r");
    assert_eq!(h["limit_reached"][0]["threshold"], 10.0);
    assert!(near(h["max_drawdown_r"].as_f64().unwrap(), 10.0));
    assert!(near(h["current_drawdown_r"].as_f64().unwrap(), 7.0));
    // Another account and the other strategy: nothing reached, nothing recorded.
    assert!(limits::check(&pool, B, &limits::S1, x.clone(), None)
        .await
        .unwrap()
        .is_empty());
    assert!(limits::check(&pool, A, &limits::S3, x, None)
        .await
        .unwrap()
        .is_empty());
    assert!(limit_decisions(&pool, B).await.is_empty());
    assert_eq!(
        limits::health(&pool, B, &limits::S1).await.unwrap()["owner_decision_needed"],
        false
    );
}

/// 25 wins of 0.7R and 24 losses of 1R, alternating from a win: profit
/// factor 17.5 / 24 = 0.73, the largest fall 7.9R.
fn pf_run() -> Vec<f64> {
    (0..49)
        .map(|i| if i % 2 == 0 { 0.7 } else { -1.0 })
        .collect()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_profit_factor_is_judged_from_50_closed_trades(pool: PgPool) {
    trades(&pool, A, "s3", 1, &pf_run()).await;
    let x = json!({});
    let hit = limits::check(&pool, A, &limits::S3, x.clone(), None)
        .await
        .unwrap();
    assert!(hit.is_empty(), "49 trades are not judged: {hit:?}");
    let h = limits::health(&pool, A, &limits::S3).await.unwrap();
    assert_eq!(h["closed_trades"], 49);
    assert!(h["profit_factor"].as_f64().unwrap() < 0.8);
    assert_eq!(h["to_limits"]["pf_trades_to_judge"], 1);
    assert_eq!(h["to_limits"]["pf_margin"], Value::Null);
    assert_eq!(h["owner_decision_needed"], false);
    // The 50th: 17.5 / 25 = 0.70 < 0.8.
    trades(&pool, A, "s3", 50, &[-1.0]).await;
    let hit = limits::check(&pool, A, &limits::S3, x.clone(), None)
        .await
        .unwrap();
    assert_eq!(hit, vec!["profit_factor"]);
    let d = limit_decisions(&pool, A).await;
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].0, "s3_limit_reached");
    assert!(
        d[0].1
            .contains("profit factor 0.70 < 0.8 after 50 closed trades"),
        "{}",
        d[0].1
    );
    // A win of 3R lifts it to 20.5 / 25 = 0.82: cleared, recorded once.
    trades(&pool, A, "s3", 51, &[3.0]).await;
    assert!(limits::check(&pool, A, &limits::S3, x.clone(), None)
        .await
        .unwrap()
        .is_empty());
    limits::check(&pool, A, &limits::S3, x, None).await.unwrap();
    let kinds: Vec<String> = limit_decisions(&pool, A)
        .await
        .into_iter()
        .map(|d| d.0)
        .collect();
    assert_eq!(kinds, vec!["s3_limit_reached", "s3_limit_cleared"]);
    assert!(pause_writes(&pool).await.is_empty());
    // Strategy 1 on the same account is not judged by Strategy 3's trades.
    let s1 = limits::health(&pool, A, &limits::S1).await.unwrap();
    assert_eq!(s1["closed_trades"], 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_owners_pause_is_read_with_who_set_it(pool: PgPool) {
    let p = limits::owner_pause(&pool, A, &limits::S1).await.unwrap();
    assert_eq!(p["paused"], false);
    assert_eq!(p["changed_by"], "migration");
    settings::change(
        &pool,
        "trading.s1_paused",
        Some(A),
        Some(json!(1)),
        "owner",
        "owner 25-10: pause Strategy 1 on 49236051",
    )
    .await
    .unwrap();
    let p = limits::owner_pause(&pool, A, &limits::S1).await.unwrap();
    assert_eq!(p["paused"], true);
    assert_eq!(p["changed_by"], "owner");
    assert_eq!(p["reason"], "owner 25-10: pause Strategy 1 on 49236051");
    // Only that account and that strategy.
    assert_eq!(
        limits::owner_pause(&pool, B, &limits::S1).await.unwrap()["paused"],
        false
    );
    assert_eq!(
        limits::owner_pause(&pool, A, &limits::S3).await.unwrap()["paused"],
        false
    );
    assert!(
        trading::rules(&pool, A, Environment::Demo)
            .await
            .unwrap()
            .paused
    );
    assert!(
        !trading::rules(&pool, B, Environment::Demo)
            .await
            .unwrap()
            .paused
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_seed_entry_pauses_only_as_the_owners_word(pool: PgPool) {
    // The shipped seed pauses nothing.
    for c in trading::parse_seed(trading::SEED_FILE).unwrap() {
        assert!(!limits::PAUSE_KEYS.contains(&c.key.as_str()), "{c:?}");
    }
    let seed = json!([
        {"id": "x-pause-by-factory", "key": "trading.s3_paused", "account_id": A, "value": 1,
         "changed_by": "claude", "reason": "limit reached"},
        {"id": "x-pause-by-owner", "key": "trading.s1_paused", "account_id": A, "value": 1,
         "changed_by": "owner", "reason": "owner 25-10, in writing: pause Strategy 1 on 49236051"}
    ])
    .to_string();
    let applied = trading::apply_seed(&pool, &seed).await.unwrap();
    assert_eq!(applied, vec!["x-pause-by-owner"]);
    assert_eq!(
        pause_writes(&pool).await,
        vec![("trading.s1_paused".to_string(), "owner".to_string())]
    );
}

/// Every `.rs` file under `dir`.
fn sources(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// A source line without its `//` comment (a `//` inside a string is cut
/// too; that only removes text, never adds a call).
fn code(line: &str) -> &str {
    line.find("//").map_or(line, |i| &line[..i])
}

/// No code decides to write a pause: the one settings writer in the program
/// is the reviewed seed (`trading::apply_seed`, owner-only for a pause, as
/// above), and nothing writes the settings tables but `settings::change`.
#[test]
fn no_code_writes_the_pause_settings() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for c in std::fs::read_dir(&crates).unwrap() {
        let src = c.unwrap().path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    assert!(files.len() > 20, "the sources were found: {}", files.len());
    let mut writers = Vec::new();
    let mut sql = Vec::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let name = f.display().to_string();
        let mut in_seed = false;
        for (n, line) in text.lines().enumerate() {
            let line = code(line);
            if line.contains("fn ") {
                in_seed = line.contains("fn apply_seed(");
            }
            if line.contains("settings::change(") && !line.contains("fn change(") {
                writers.push(format!("{name}:{} (in apply_seed: {in_seed})", n + 1));
            }
            let lower = line.to_lowercase();
            if (lower.contains("into settings ")
                || lower.contains("into settings(")
                || lower.contains("update settings ")
                || lower.contains("into settings_changes"))
                && !name.ends_with("settings.rs")
            {
                sql.push(format!("{name}:{}", n + 1));
            }
            // The traders read their pause through the settings snapshot;
            // a pause key is never passed to a writer.
            if line.contains("pause_key") || line.contains("PAUSE_KEYS") {
                assert!(
                    !line.contains("change("),
                    "{name}:{}: a pause key passed to a writer",
                    n + 1
                );
            }
        }
    }
    assert_eq!(writers.len(), 1, "{writers:?}");
    assert!(
        writers[0].contains("trading.rs") && writers[0].ends_with("(in apply_seed: true)"),
        "{writers:?}"
    );
    assert!(
        sql.is_empty(),
        "settings written outside settings.rs: {sql:?}"
    );
}

async fn health(pool: &PgPool) -> Value {
    let app = router(AppState {
        pool: pool.clone(),
        loading: Arc::new(AtomicBool::new(false)),
        recorders: Default::default(),
        history: Default::default(),
        trading_loops: Default::default(),
        strategy3_loops: Default::default(),
        allowed_accounts: Arc::new(BTreeSet::from([A])),
    });
    let res = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn health_shows_performance_limits_and_the_pause_in_r_without_money(pool: PgPool) {
    store::upsert_account(&pool, A, Environment::Demo)
        .await
        .unwrap();
    trades(&pool, A, "s1", 1, &[-1.0; 10]).await;
    trades(&pool, A, "s3", 100, &[2.0, -1.0]).await;
    let h = health(&pool).await;
    let s1 = &h["trading"][0];
    assert_eq!(s1["account_id"], A);
    assert_eq!(s1["performance"]["closed_trades"], 10);
    assert_eq!(s1["performance"]["owner_decision_needed"], true);
    assert_eq!(s1["performance"]["limit_reached"][0]["limit"], "drawdown_r");
    assert_eq!(s1["owner_pause"]["paused"], false);
    assert_eq!(s1["trigger_distance"], json!({}));
    let s3 = &h["strategy3"][0];
    assert_eq!(s3["performance"]["closed_trades"], 2);
    assert!(near(s3["performance"]["net_r"].as_f64().unwrap(), 1.0));
    assert_eq!(s3["performance"]["owner_decision_needed"], false);
    assert_eq!(s3["owner_pause"]["paused"], false);
    // R only: no money field and no money figure. Fixed source diagnostics
    // such as balance_stable and pnl_coverage_complete are permitted.
    let text = h.to_string().to_lowercase();
    for field in ["balance", "gross", "commission", "net_money", "pnl"] {
        assert!(
            !text.contains(&format!("\"{field}\":")),
            "{field} on /health"
        );
    }
    for a in [s1, s3] {
        for part in ["performance", "owner_pause", "trigger_distance"] {
            let t = a[part].to_string().to_lowercase();
            for word in ["equity", "money", "deposit", "usd"] {
                assert!(!t.contains(word), "{word} in {part}: {t}");
            }
        }
    }
    for money in ["-600", "-6.0", "1200", "12.0"] {
        assert!(
            !s1["performance"].to_string().contains(money),
            "{money} in Strategy 1's performance"
        );
    }
}
