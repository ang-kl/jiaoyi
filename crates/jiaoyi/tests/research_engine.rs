//! The research engine (R1 of the edge test) against a real PostgreSQL and
//! the in-memory fake broker: the pre-registered jobs, the held-out seal,
//! the held-out gate, attempts, and one development job end to end.

use std::collections::HashMap;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use jiaoyi::config::BrokerConfig;
use jiaoyi::edge::data::{Data, DataError, Stage};
use jiaoyi::edge::{prereg, runner};
use jiaoyi::{history, loader, shutdown};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_core::universe::AssetClass;
use jiaoyi_ctrader::messages::{
    ProtoOaClosePositionDetail, ProtoOaDeal, ProtoOaLightSymbol, ProtoOaSymbol,
};
use jiaoyi_ctrader::testkit::FakeBroker;
use jiaoyi_store::research as rs;
use jiaoyi_store::{self as store, settings, BarRow};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx::PgPool;

const LIVE: i64 = 43002148;
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

fn permitted() -> PermittedAccount {
    Allowlist::parse(Some("43002148"))
        .unwrap()
        .permit(LIVE)
        .unwrap()
}

fn at(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_migration_registers_the_ten_h1_jobs_under_the_frozen_hash(pool: PgPool) {
    assert_eq!(prereg::hash(), HASH);
    let jobs = rs::jobs(&pool).await.unwrap();
    assert_eq!(jobs.len(), 10);
    assert!(jobs
        .iter()
        .all(|j| j.hypothesis == "H1" && j.prereg_hash == HASH && j.status == "pending"));
    // The default first, then the other 8 cells, then the held-out job.
    assert_eq!(jobs[0].setting["breakout"], 50);
    assert_eq!(jobs[0].setting["stop_atr"], 3);
    assert_eq!(jobs[0].setting["default"], true);
    let cells: std::collections::BTreeSet<(i64, i64)> = jobs[..9]
        .iter()
        .map(|j| {
            assert_eq!(j.stage, "development");
            (
                j.setting["breakout"].as_i64().unwrap(),
                j.setting["stop_atr"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(cells.len(), 9);
    for b in [20, 50, 100] {
        for k in [2, 3, 4] {
            assert!(cells.contains(&(b, k)));
        }
    }
    assert_eq!(jobs[9].stage, "held_out");
    assert_eq!(jobs[9].setting, jobs[0].setting);
    // The trial cap: no other job can be added under the same definition.
    let dup = sqlx::query(
        "INSERT INTO research_jobs (hypothesis, stage, setting, run_order, prereg_hash)
         SELECT hypothesis, stage, setting, 99, prereg_hash FROM research_jobs WHERE run_order = 1",
    )
    .execute(&pool)
    .await;
    assert!(dup.is_err());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_job_moves_forward_only_and_records_every_attempt(pool: PgPool) {
    let job = rs::next_pending_job(&pool).await.unwrap().unwrap();
    assert_eq!(
        rs::claim_job(&pool, job.id, "commit-a").await.unwrap(),
        Some(1)
    );
    assert_eq!(
        rs::claim_job(&pool, job.id, "commit-a").await.unwrap(),
        None,
        "taken once"
    );
    // A stop interrupts it: nothing was written; it is attempted again.
    assert_eq!(rs::release_interrupted_jobs(&pool).await.unwrap(), 1);
    assert_eq!(
        rs::claim_job(&pool, job.id, "commit-b").await.unwrap(),
        Some(2)
    );
    let a = rs::job_attempts(&pool, job.id).await.unwrap();
    assert_eq!(
        a.iter()
            .map(|x| (x.attempt, x.commit.as_str()))
            .collect::<Vec<_>>(),
        vec![(1, "commit-a"), (2, "commit-b")]
    );
    rs::close_job(&pool, job.id, "failed", "for the test")
        .await
        .unwrap();
    // A finished job never runs again, and is never deleted.
    assert_eq!(rs::claim_job(&pool, job.id, "c").await.unwrap(), None);
    for q in [
        "UPDATE research_jobs SET status = 'pending', started_at = NULL WHERE run_order = 1",
        "UPDATE research_jobs SET status = 'running' WHERE run_order = 1",
        "UPDATE research_jobs SET setting = '{}' WHERE run_order = 2",
        "DELETE FROM research_jobs WHERE run_order = 2",
        "DELETE FROM research_job_attempts",
        "UPDATE research_job_events SET status = 'x'",
    ] {
        assert!(sqlx::query(q).execute(&pool).await.is_err(), "{q}");
    }
    let events: Vec<String> =
        sqlx::query_scalar("SELECT status FROM research_job_events WHERE job_id = $1 ORDER BY id")
            .bind(job.id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        events,
        vec!["pending", "running", "pending", "running", "failed"]
    );
}

// ---------------------------------------------------------------- the seal

async fn symbol(pool: &PgPool, id: i64, name: &str, class: AssetClass, digits: i32, pip: i32) {
    let s = ProtoOaSymbol {
        symbol_id: id,
        digits,
        pip_position: pip,
        min_volume: Some(1000),
        step_volume: Some(1000),
        swap_long: Some(-5.0),
        swap_short: Some(1.0),
        swap_rollover3_days: Some(3),
        swap_calculation_type: Some(0),
        ..Default::default()
    };
    store::upsert_symbol(
        pool,
        &loader::symbol_row(LIVE, name, class, Some(true), &s, Utc::now()),
    )
    .await
    .unwrap();
}

fn bar(open: DateTime<Utc>, o: i64, h: i64, l: i64, c: i64) -> BarRow {
    BarRow {
        bar_date: open.date_naive(),
        bar_open_utc: open,
        open: Decimal::new(o, 5),
        high: Decimal::new(h, 5),
        low: Decimal::new(l, 5),
        close: Decimal::new(c, 5),
        tick_volume: 100,
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_development_job_refuses_held_out_period_data(pool: PgPool) {
    store::upsert_account(&pool, LIVE, Environment::Live)
        .await
        .unwrap();
    symbol(&pool, 1, "EURUSD", AssetClass::Fx, 5, 4).await;
    symbol(&pool, 2, "GER40", AssetClass::EquityIndex, 2, 0).await;
    // EURUSD: FX-style bars at 21:00 UTC up to 23-10-2025 (that bar runs
    // into the held-out period) and one in it. GER40: nothing after 20-10.
    let fx: Vec<BarRow> = (0..10)
        .map(|i| {
            let o = at("2025-10-15T21:00:00Z") + Duration::days(i);
            bar(o, 116_000, 116_500, 115_500, 116_100)
        })
        .collect();
    store::upsert_daily_bars(&pool, LIVE, 1, None, &fx, Utc::now())
        .await
        .unwrap();
    let ger = vec![bar(
        at("2025-10-20T00:00:00Z"),
        2_400_000_000,
        2_410_000_000,
        2_390_000_000,
        2_405_000_000,
    )];
    store::upsert_daily_bars(&pool, LIVE, 2, None, &ger, Utc::now())
        .await
        .unwrap();
    let dev = Data::new(&pool, Stage::Development);
    let seal = at("2025-10-24T00:00:00Z");

    // Up to the seal: only bars that END by it; the 23-10 21:00 bar does not.
    let bars = dev.daily_bars(LIVE, 1, seal).await.unwrap();
    assert_eq!(
        bars.last().unwrap().bar_date,
        NaiveDate::from_ymd_opt(2025, 10, 22).unwrap()
    );
    assert!(bars
        .iter()
        .all(|b| b.bar_open_utc + Duration::days(1) <= seal));
    // Past the seal: refused, not filtered, even where nothing is stored
    // after it (GER40).
    for (id, end) in [
        (1, seal + Duration::seconds(1)),
        (2, at("2026-01-01T00:00:00Z")),
        (2, at("2026-10-24T00:00:00Z")),
    ] {
        match dev.daily_bars(LIVE, id, end).await {
            Err(DataError::Sealed(why)) => assert!(why.contains("refused, not clamped"), "{why}"),
            other => panic!("symbol {id} to {end}: {other:?}"),
        }
    }
    // Ticks: a development window reaching the seal is refused before any
    // request is sent.
    let f = FakeBroker {
        account_id: LIVE,
        grant: Some(vec![jiaoyi_ctrader::testkit::GrantEntry {
            ctid: LIVE,
            is_live: Some(true),
            login: None,
        }]),
        ..Default::default()
    };
    let (conn, log) = f.spawn();
    conn.application_auth("client", &Secret::new("s"))
        .await
        .unwrap();
    let session = conn
        .account_auth(&permitted(), &Secret::new("tok"))
        .await
        .unwrap();
    let s_ms = seal.timestamp_millis();
    for (from, to) in [
        (s_ms - 3_600_000, s_ms),
        (s_ms - 1, s_ms + 1),
        (s_ms, s_ms + 60_000),
    ] {
        let r = dev
            .ticks(
                &session,
                1,
                jiaoyi_ctrader::tickdata::QuoteSide::Bid,
                from,
                to,
            )
            .await;
        assert!(
            matches!(r, Err(DataError::Sealed(_))),
            "{from}..{to}: {r:?}"
        );
    }
    assert!(
        log.lock().unwrap().tick_requests.is_empty(),
        "nothing was asked of the broker"
    );
    // A development window before the seal is served.
    dev.ticks(
        &session,
        1,
        jiaoyi_ctrader::tickdata::QuoteSide::Bid,
        s_ms - 3_600_000,
        s_ms - 1,
    )
    .await
    .unwrap();
    // The held-out stage reads up to the end of the held-out period.
    let held = Data::new(&pool, Stage::HeldOut);
    assert_eq!(
        held.daily_bars(LIVE, 1, at("2026-10-24T00:00:00Z"))
            .await
            .unwrap()
            .len(),
        10
    );
    assert!(matches!(
        held.daily_bars(LIVE, 1, at("2026-10-25T00:00:00Z")).await,
        Err(DataError::Sealed(_))
    ));
}

// ---------------------------------------------------------------- the gate

/// Write a development trial for job `run_order` with the given rules and PF.
async fn dev_trial(pool: &PgPool, run_order: i32, rules: Value, pf: f64) {
    let job = rs::jobs(pool)
        .await
        .unwrap()
        .into_iter()
        .find(|j| j.run_order == run_order)
        .unwrap();
    rs::claim_job(pool, job.id, "test").await.unwrap().unwrap();
    let default = job.setting["default"].as_bool().unwrap();
    rs::finish_job_with_trial(
        pool,
        &rs::JobTrial {
            job_id: job.id,
            prereg_hash: HASH,
            hypothesis: "H1",
            sample: "development",
            params: &json!({"setting": {"default": default}}),
            data_range: &json!({}),
            results: &json!({
                "rules": rules,
                "r": {"n": 300, "mean_r": 0.1, "total_r": 30.0, "profit_factor": pf, "pf_infinite": false, "wins": 100, "losses": 200},
            }),
            verdict: "development",
            settings_version: 1,
        },
    )
    .await
    .unwrap();
}

fn rules(pass: bool) -> Value {
    json!({
        "rule1_mean_positive": true,
        "rule1_bootstrap_lower_positive": pass,
        "rule2_above_control_p95": true,
        "rule3_profit_factor": true,
        "rule4_min_trades": true,
    })
}

async fn run_held_out_step(pool: &PgPool) -> runner::Step {
    let (_t, stop) = shutdown::channel();
    runner::step(
        pool,
        &broker(),
        &[(permitted(), Environment::Live)],
        &|_| async {
            Err::<jiaoyi_ctrader::Connection, _>(anyhow::anyhow!("no broker in this test"))
        },
        &stop,
    )
    .await
    .unwrap()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_held_out_job_is_refused_when_the_development_gate_fails(pool: PgPool) {
    // The default missed rule 1 (bootstrap lower bound); all cells PF 1.2.
    dev_trial(&pool, 1, rules(false), 1.2).await;
    for k in 2..=9 {
        dev_trial(&pool, k, rules(true), 1.2).await;
    }
    let step = run_held_out_step(&pool).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("refused")),
        "{step:?}"
    );
    let job = rs::jobs(&pool).await.unwrap().pop().unwrap();
    assert_eq!(job.stage, "held_out");
    assert_eq!(job.status, "refused");
    let why = job.reason.unwrap();
    assert!(
        why.starts_with("no demonstrated edge") && why.contains("rules 1-3"),
        "{why}"
    );
    // No held-out trial exists.
    let held: i64 =
        sqlx::query_scalar("SELECT count(*) FROM research_trials WHERE sample = 'held_out'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(held, 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_held_out_job_is_refused_with_too_few_cells_above_one(pool: PgPool) {
    dev_trial(&pool, 1, rules(true), 1.5).await;
    // 5 of 9 cells above 1.0 (the default included): one short of 6.
    for (k, pf) in (2..=9).zip([1.1, 1.1, 1.1, 1.1, 1.0, 0.9, 0.8, 0.7]) {
        dev_trial(&pool, k, rules(true), pf).await;
    }
    let step = run_held_out_step(&pool).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("refused")),
        "{step:?}"
    );
    let job = rs::jobs(&pool).await.unwrap().pop().unwrap();
    assert!(job.reason.unwrap().contains("5 of 9"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_held_out_job_goes_ahead_when_the_gate_is_met(pool: PgPool) {
    dev_trial(&pool, 1, rules(true), 1.5).await;
    for (k, pf) in (2..=9).zip([1.1, 1.1, 1.1, 1.1, 1.1, 0.9, 0.8, 0.7]) {
        dev_trial(&pool, k, rules(true), pf).await;
    }
    // Gate met (6 of 9): the job is not refused; here it waits for the
    // deposit currency, so it is still pending.
    let step = run_held_out_step(&pool).await;
    assert!(matches!(&step, runner::Step::Waiting(_)), "{step:?}");
    let job = rs::jobs(&pool).await.unwrap().pop().unwrap();
    assert_eq!(job.status, "pending");
}

// ---------------------------------------------------------------- end to end

/// One symbol's bars: every calendar day at 00:00 UTC from 2012-03-01 to
/// 2014-12-31, a slow wave (period 200 days) so that trends turn, price `base`.
fn wave(base: f64) -> Vec<BarRow> {
    let first = Utc.with_ymd_and_hms(2012, 3, 1, 0, 0, 0).unwrap();
    let mut out = Vec::new();
    let mut prev = base;
    for i in 0..1035 {
        let c = base * (1.0 + 0.08 * (2.0 * std::f64::consts::PI * i as f64 / 200.0).sin());
        let e5 = |x: f64| (x * 100_000.0).round() as i64;
        let (o, cl) = (prev, c);
        let h = o.max(cl) * 1.002;
        let l = o.min(cl) * 0.998;
        out.push(bar(first + Duration::days(i), e5(o), e5(h), e5(l), e5(cl)));
        prev = c;
    }
    out
}

/// Bid and ask ticks for the June 2014 sample week, from the bars: each
/// bar's first bid is its open, its last bid its close; the ask is `spread`
/// above. Bid-based bars, as the engine must then measure.
/// (ts_ms, price in 1/100000) ticks of one side.
type Ticks = Vec<(i64, i64)>;

fn ticks_for(bars: &[BarRow], spread: i64) -> (Ticks, Ticks) {
    let (from, to) = jiaoyi::edge::sampler::june_week(2014);
    let mut bids = Vec::new();
    for b in bars
        .iter()
        .filter(|b| b.bar_open_utc >= from && b.bar_open_utc + Duration::days(1) <= to)
    {
        let o = b.bar_open_utc.timestamp_millis();
        let p = |d: Decimal| {
            (d * Decimal::from(100_000))
                .to_string()
                .parse::<f64>()
                .unwrap() as i64
        };
        bids.push((o + 60_000, p(b.open)));
        bids.push((o + 43_200_000, (p(b.open) + p(b.close)) / 2));
        bids.push((o + 86_340_000, p(b.close)));
    }
    let asks = bids.iter().map(|(t, p)| (*t, p + spread)).collect();
    (bids, asks)
}

/// The broker's statement of quote assets in a load run: its symbol list
/// (quote asset ids) and its asset list (names).
struct Statement {
    listed: Value,
    assets: Option<Value>,
    /// The quote → deposit rate of the XAUUSD closing deal.
    xau_close_rate: f64,
    /// Whether the load stored the daily bars and ended `ok`; otherwise it
    /// ended `partial` before the bars (the loader stores symbol rows first).
    bars_loaded: bool,
}

impl Default for Statement {
    /// Both symbols quoted in USD (asset 15), as the broker states them.
    fn default() -> Self {
        Statement {
            listed: json!([
                {"id": 1, "name": "EURUSD", "enabled": true, "base_asset_id": 3, "quote_asset_id": 15},
                {"id": 41, "name": "XAUUSD", "enabled": true, "base_asset_id": 20, "quote_asset_id": 15},
            ]),
            assets: Some(json!([
                {"asset_id": 15, "name": "USD"},
                {"asset_id": 3, "name": "EUR"},
                {"asset_id": 20, "name": "XAU"},
            ])),
            xau_close_rate: 1.0,
            bars_loaded: true,
        }
    }
}

/// When the setup's deals execute: 2013-06-04 12:00 UTC, inside the
/// stored FX bars, so the quote cross-check has the day's close.
const DEAL_MS: i64 = 1_370_347_200_000;

/// A deal on `sym` executed at `at_ms`: 10,000 units, commission 2.42 USD;
/// a closing one converts quote → deposit at `rate`.
fn deal(id: i64, sym: i64, closing: bool, at_ms: i64, rate: f64) -> ProtoOaDeal {
    ProtoOaDeal {
        deal_id: id,
        order_id: id,
        position_id: id,
        volume: 1_000_000,
        filled_volume: 1_000_000,
        symbol_id: sym,
        create_timestamp: at_ms,
        execution_timestamp: at_ms,
        execution_price: Some(1.17),
        trade_side: 1,
        deal_status: 2,
        commission: Some(-242),
        money_digits: Some(2),
        close_position_detail: closing.then(|| ProtoOaClosePositionDetail {
            entry_price: 1.16,
            quote_to_deposit_conversion_rate: Some(rate),
            money_digits: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    }
}

async fn store_deals(pool: &PgPool, deals: &[ProtoOaDeal]) {
    let rows: Vec<_> = deals.iter().map(history::deal_row).collect();
    store::history::upsert_deals(pool, LIVE, &rows, Utc::now())
        .await
        .unwrap();
}

/// The two symbols' daily bars, as a load stores them.
async fn store_bars(pool: &PgPool) {
    store::upsert_daily_bars(pool, LIVE, 1, None, &wave(1.3), Utc::now())
        .await
        .unwrap();
    store::upsert_daily_bars(pool, LIVE, 41, None, &wave(1250.0), Utc::now())
        .await
        .unwrap();
}

/// A load run resolves both universe names, as the loader does first.
async fn resolve_both(pool: &PgPool, run: i64) {
    use jiaoyi_core::universe::Resolution;
    for (id, name, class) in [
        (1i64, "EURUSD", AssetClass::Fx),
        (41, "XAUUSD", AssetClass::Metal),
    ] {
        store::record_resolution(
            pool,
            run,
            LIVE,
            name,
            class,
            &Resolution::Resolved { symbol_id: id },
        )
        .await
        .unwrap();
    }
}

async fn end_to_end_setup(pool: &PgPool) -> FakeBroker {
    end_to_end_setup_with(pool, Statement::default()).await
}

async fn end_to_end_setup_with(pool: &PgPool, st: Statement) -> FakeBroker {
    store::upsert_account(pool, LIVE, Environment::Live)
        .await
        .unwrap();
    settings::change(
        pool,
        "universe.symbols",
        None,
        Some(json!([{"name": "EURUSD", "class": "fx"}, {"name": "XAUUSD", "class": "metal"}])),
        "test",
        "two symbols",
    )
    .await
    .unwrap();
    symbol(pool, 1, "EURUSD", AssetClass::Fx, 5, 4).await;
    symbol(pool, 41, "XAUUSD", AssetClass::Metal, 2, 1).await;
    let eur = wave(1.3);
    let xau = wave(1250.0);
    if st.bars_loaded {
        store_bars(pool).await;
    }
    // A load run with the symbol list (asset ids: USD is 15) and the costs.
    let run = store::start_run(pool, LIVE).await.unwrap();
    resolve_both(pool, run).await;
    for (id, sym) in [(1i64, "EURUSD"), (41, "XAUUSD")] {
        let s = ProtoOaSymbol {
            symbol_id: id,
            digits: if id == 1 { 5 } else { 2 },
            pip_position: if id == 1 { 4 } else { 1 },
            swap_long: Some(-5.0),
            swap_short: Some(1.0),
            swap_rollover3_days: Some(3),
            swap_calculation_type: Some(0),
            ..Default::default()
        };
        let _ = sym;
        store::insert_cost(
            pool,
            &loader::cost_row(
                LIVE,
                id,
                run,
                Utc::now(),
                std::time::Duration::from_secs(5),
                None,
                Some(&s),
            ),
        )
        .await
        .unwrap();
    }
    store::finish_run(
        pool,
        run,
        if st.bars_loaded { "ok" } else { "partial" },
        &store::RunSummary {
            listed_symbols: Some(st.listed),
            assets: st.assets,
            errors: if st.bars_loaded {
                vec![]
            } else {
                vec!["EURUSD: daily bars: connection closed".into()]
            },
            ..Default::default()
        },
    )
    .await
    .unwrap();
    // The trader record: deposit asset 15.
    store::history::record_account_snapshot(
        pool,
        &store::history::AccountSnapshot {
            account_id: LIVE,
            trader_login: Some(1),
            deposit_asset_id: 15,
            balance: 1_000_000,
            money_digits: Some(2),
            registration_at: None,
            raw: vec![],
        },
    )
    .await
    .unwrap();
    // Deals: commission 2.42 USD on 10,000 units; EURUSD closes at
    // quote→deposit 1, XAUUSD at the statement's rate.
    store_deals(
        pool,
        &[
            deal(1, 1, false, DEAL_MS, 0.0),
            deal(2, 1, true, DEAL_MS, 1.0),
            deal(3, 41, false, DEAL_MS, 0.0),
            deal(4, 41, true, DEAL_MS, st.xau_close_rate),
        ],
    )
    .await;
    // Earlier runs already sampled every June but 2014 (no ticks then):
    // the sampler resumes with the one missing year.
    for (id, name) in [(1i64, "EURUSD"), (41, "XAUUSD")] {
        for year in (2013..=2025).filter(|y| *y != 2014) {
            let (from, to) = jiaoyi::edge::sampler::june_week(year);
            rs::record_spread(
                pool,
                &rs::SpreadSample {
                    account_id: LIVE,
                    broker_symbol_id: id,
                    symbol: name.into(),
                    year,
                    week_from: from,
                    week_to: to,
                    status: "no_ticks".into(),
                    median_spread: None,
                    samples: 0,
                    bid_ticks: 0,
                    ask_ticks: 0,
                    download_ms: 0,
                    basis: Some(json!({"bid": 0, "ask": 0, "mid": 0, "observations": 0, "uninformative": 0})),
                    error: None,
                    prereg_hash: HASH.into(),
                },
            )
            .await
            .unwrap();
        }
    }
    // The broker: ticks for June 2014 only.
    let (eb, ea) = ticks_for(&eur, 2);
    let (xb, xa) = ticks_for(&xau, 30);
    FakeBroker {
        account_id: LIVE,
        grant: Some(vec![jiaoyi_ctrader::testkit::GrantEntry {
            ctid: LIVE,
            is_live: Some(true),
            login: None,
        }]),
        light_symbols: vec![ProtoOaLightSymbol {
            symbol_id: 1,
            symbol_name: Some("EURUSD".into()),
            enabled: Some(true),
            ..Default::default()
        }],
        tick_history: HashMap::from([((1, 1), eb), ((1, 2), ea), ((41, 1), xb), ((41, 2), xa)]),
        ..Default::default()
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn the_default_development_job_runs_to_a_trial(pool: PgPool) {
    let f = end_to_end_setup(&pool).await;
    let (_t, stop) = shutdown::channel();
    let f2 = f.clone();
    let connect = move |_account: i64| {
        let f = f2.clone();
        async move { Ok(f.spawn().0) }
    };
    let step = runner::step(
        &pool,
        &broker(),
        &[(permitted(), Environment::Live)],
        &connect,
        &stop,
    )
    .await
    .unwrap();
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("done")),
        "{step:?}"
    );

    // The spread samples: 13 Junes per symbol, measured where ticks exist.
    let samples = rs::spread_samples(&pool, LIVE).await.unwrap();
    assert_eq!(samples.len(), 26);
    let measured: Vec<_> = samples.iter().filter(|s| s.status == "measured").collect();
    assert_eq!(measured.len(), 2);
    let eur = measured.iter().find(|s| s.symbol == "EURUSD").unwrap();
    assert_eq!(eur.year, 2014);
    assert_eq!(eur.median_spread, Some(Decimal::new(2, 5)), "2 × 1e-5");
    assert!(eur.bid_ticks > 0 && eur.ask_ticks > 0);
    let basis = &eur.basis.as_ref().unwrap();
    assert_eq!(
        basis["observations"], basis["bid"],
        "every comparison found bid bars: {basis}"
    );
    assert!(samples.iter().filter(|s| s.status == "no_ticks").count() == 24);

    // The trial: under the frozen hash, with every reading recorded.
    let trials = rs::job_trials(&pool).await.unwrap();
    assert_eq!(trials.len(), 1);
    let t = &trials[0];
    assert_eq!(t.prereg_hash.as_deref(), Some(HASH));
    assert_eq!(t.sample, "development");
    assert!(t
        .verdict
        .as_deref()
        .unwrap()
        .starts_with("development: PASS rules 1-3"));
    assert_eq!(t.params["setting"]["default"], true);
    assert!(t.params["readings"]["held_out_lower_bound"]
        .as_str()
        .unwrap()
        .contains("2.5th percentile"));
    assert!(t.params["readings"]["symbol_start"]
        .as_str()
        .unwrap()
        .contains("101st"));
    assert_eq!(t.params["deposit_currency"]["currency"], "USD");
    assert!(t.params["deposit_currency"]["cross_check"]
        .as_str()
        .unwrap()
        .starts_with("2 closing deals on symbols the broker quotes in USD"));
    let sym = &t.params["symbols"][0];
    assert_eq!(sym["basis"], "bid");
    // The quote currency is the broker's, cross-checked with the deal.
    assert_eq!(sym["quote"], "USD");
    assert!(sym["quote_label"]
        .as_str()
        .unwrap()
        .starts_with("the broker's asset list (load run"));
    assert_eq!(
        sym["quote_cross_check"],
        "1 closing deals executed before 2025-10-24 00:00:00 UTC agree within 1% (largest difference 0.000%)"
    );
    assert_eq!(sym["spreads_measured"]["2014"], 0.00002);
    assert!(sym["commission_label"]
        .as_str()
        .unwrap()
        .starts_with("measured"));
    // Development data only.
    assert_eq!(t.data_range["reads_until"], "2025-10-24T00:00:00Z");
    let r = t.results.as_ref().unwrap();
    assert!(r["trades"].as_u64().unwrap() > 0, "{}", r["trades"]);
    assert_eq!(r["control"]["runs"], 1000);
    assert_eq!(r["control"]["seed"], 20_261_024);
    assert_eq!(r["bootstrap"]["resamples"], 10_000);
    for k in ["without_top_5_trades", "worst_year", "slippage_2x"] {
        assert!(!r["evidence_against"][k].is_null(), "{k}");
    }
    assert!(r["spread_split"]["estimated_spread_trades"]["n"]
        .as_u64()
        .is_some());
    assert_eq!(r["attempts"][0]["commit"], "unknown");
    assert!(r["per_year"]
        .as_object()
        .unwrap()
        .keys()
        .all(|y| y.as_str() < "2015"));
    // The job is done; the routes show counts and R, no money.
    let jobs = rs::jobs(&pool).await.unwrap();
    assert_eq!(jobs[0].status, "done");
    let v = jiaoyi::edge::routes::trials(&pool).await.unwrap();
    let shown = &v["trials"][0]["results"];
    for hidden in ["notional", "trade_list", "recheck_trades"] {
        assert!(shown.get(hidden).is_none(), "{hidden} on the route");
    }
    assert!(
        shown["bootstrap"].get("sum_lower_2_5").is_none(),
        "no money range on the route"
    );
    assert_eq!(
        v["trials"][0]["deflated_sharpe"]["pending"],
        "1 of 9 development trials"
    );
    let jv = jiaoyi::edge::routes::jobs(&pool).await.unwrap();
    assert_eq!(jv["prereg_hash_running"], HASH);
    assert_eq!(jv["jobs"][0]["attempts"].as_array().unwrap().len(), 1);
}

// ------------------------------------------------- quote currencies (R1)

/// The June 2014 spread samples of both symbols, as measured (bid bars), so
/// a job needs no broker for its inputs.
async fn measured_2014(pool: &PgPool) {
    let (from, to) = jiaoyi::edge::sampler::june_week(2014);
    for (id, name, spread) in [
        (1i64, "EURUSD", Decimal::new(2, 5)),
        (41, "XAUUSD", Decimal::new(30, 2)),
    ] {
        rs::record_spread(
            pool,
            &rs::SpreadSample {
                account_id: LIVE,
                broker_symbol_id: id,
                symbol: name.into(),
                year: 2014,
                week_from: from,
                week_to: to,
                status: "measured".into(),
                median_spread: Some(spread),
                samples: 10,
                bid_ticks: 10,
                ask_ticks: 10,
                download_ms: 1,
                basis: Some(
                    json!({"bid": 10, "ask": 0, "mid": 0, "observations": 10, "uninformative": 0}),
                ),
                error: None,
                prereg_hash: HASH.into(),
            },
        )
        .await
        .unwrap();
    }
}

/// A later load run stating the symbols' quote assets and the asset list.
async fn load_run(pool: &PgPool, listed: Value, assets: Value) -> i64 {
    let run = store::start_run(pool, LIVE).await.unwrap();
    store::finish_run(
        pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(listed),
            assets: Some(assets),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    run
}

async fn dev_step(pool: &PgPool, f: &FakeBroker) -> runner::Step {
    let (_t, stop) = shutdown::channel();
    let f2 = f.clone();
    let connect = move |_account: i64| {
        let f = f2.clone();
        async move { Ok(f.spawn().0) }
    };
    runner::step(
        pool,
        &broker(),
        &[(permitted(), Environment::Live)],
        &connect,
        &stop,
    )
    .await
    .unwrap()
}

/// USD per euro at the close of the stored EURUSD bar containing `at_ms`.
fn eur_close_at(at_ms: i64) -> f64 {
    let t = DateTime::<Utc>::from_timestamp_millis(at_ms).unwrap();
    let b = wave(1.3)
        .into_iter()
        .find(|b| b.bar_open_utc <= t && t < b.bar_open_utc + Duration::days(1))
        .unwrap();
    rust_decimal::prelude::ToPrimitive::to_f64(&b.close).unwrap()
}

/// The broker states XAUUSD's quote asset as EUR (asset 3): the engine
/// takes the statement, not the name, and converts with EURUSD.
fn xau_in_eur(xau_close_rate: f64) -> Statement {
    Statement {
        listed: json!([
            {"id": 1, "name": "EURUSD", "enabled": true, "base_asset_id": 3, "quote_asset_id": 15},
            {"id": 41, "name": "XAUUSD", "enabled": true, "base_asset_id": 20, "quote_asset_id": 3},
        ]),
        xau_close_rate,
        ..Default::default()
    }
}

fn refusals(u: &jiaoyi::edge::h1job::Universe) -> HashMap<&str, &str> {
    u.refused
        .iter()
        .map(|(n, r)| (n.as_str(), r.as_str()))
        .collect()
}

/// Owner, 24-10-2026: the broker's asset list is the sole source of a
/// symbol's quote currency. A symbol whose quote asset cannot be read from
/// it is refused with the reason, as is one quoted in a currency no FX pair
/// of the universe converts; the other symbols are still tested.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_symbol_whose_quote_asset_is_not_read_from_the_broker_is_refused(pool: PgPool) {
    use jiaoyi::edge::h1job::{self, Deposit};
    let eur = json!({"id": 1, "name": "EURUSD", "enabled": true, "base_asset_id": 3, "quote_asset_id": 15});
    let assets = Statement::default().assets.unwrap();
    end_to_end_setup_with(
        &pool,
        Statement {
            listed: json!([eur, {"id": 41, "name": "XAUUSD", "enabled": true, "base_asset_id": 20, "quote_asset_id": 77}]),
            ..Default::default()
        },
    )
    .await;
    measured_2014(&pool).await;
    let prepare = || async {
        let deposit = match h1job::deposit_currency(&pool, LIVE).await.unwrap() {
            Deposit::Usd(v) => v,
            other => panic!("{other:?}"),
        };
        h1job::prepare(&Data::new(&pool, Stage::Development), LIVE, deposit)
            .await
            .unwrap()
    };
    let (run1, _, _) = store::latest_assets(&pool, LIVE).await.unwrap().unwrap();
    let u = prepare().await;
    let tested: Vec<&str> = u.symbols.iter().map(|x| x.costs.name.as_str()).collect();
    assert_eq!(tested, vec!["EURUSD"]);
    assert_eq!(
        refusals(&u)["XAUUSD"],
        format!(
            "quote asset not read from the broker: asset 77 is not in load run {run1}'s asset list"
        )
    );

    // A later load whose symbol list gives XAUUSD no quote asset id.
    let run2 = load_run(
        &pool,
        json!([eur, {"id": 41, "name": "XAUUSD", "enabled": true, "base_asset_id": 20}]),
        assets.clone(),
    )
    .await;
    assert_eq!(
        refusals(&prepare().await)["XAUUSD"],
        format!("quote asset not read from the broker: load run {run2}'s symbol list gives symbol 41 no quote asset id")
    );

    // Quoted in CHF: no FX pair of the universe converts it.
    let mut with_chf = assets.as_array().unwrap().clone();
    with_chf.push(json!({"asset_id": 9, "name": "CHF"}));
    load_run(
        &pool,
        json!([eur, {"id": 41, "name": "XAUUSD", "enabled": true, "base_asset_id": 20, "quote_asset_id": 9}]),
        Value::Array(with_chf),
    )
    .await;
    assert_eq!(
        refusals(&prepare().await)["XAUUSD"],
        "quoted in CHF: no FX pair in the universe converts it to USD"
    );
}

/// Owner, 24-10-2026: the deposit currency is the name the broker's asset
/// list gives the trader record's deposit asset. Without a stored asset
/// list the job waits; a deposit in another currency, or one the list does
/// not name, fails the job.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_deposit_currency_other_than_usd_fails_the_job(pool: PgPool) {
    use jiaoyi::edge::h1job::{self, Deposit};
    let f = end_to_end_setup_with(
        &pool,
        Statement {
            assets: None,
            ..Default::default()
        },
    )
    .await;
    measured_2014(&pool).await;
    assert_eq!(
        h1job::deposit_currency(&pool, LIVE).await.unwrap(),
        Deposit::NotReady(
            "no load has stored the broker's asset list (ProtoOAAssetListRes) yet".into()
        )
    );
    let step = dev_step(&pool, &f).await;
    assert!(
        matches!(&step, runner::Step::Waiting(w) if w.contains("asset list")),
        "{step:?}"
    );
    assert_eq!(rs::jobs(&pool).await.unwrap()[0].status, "pending");

    // The broker names the deposit asset 15 SGD.
    let run = load_run(
        &pool,
        Statement::default().listed,
        json!([{"asset_id": 15, "name": "SGD"}, {"asset_id": 3, "name": "EUR"}]),
    )
    .await;
    let why = "the deposit currency is SGD (asset 15), not USD: money cannot be converted";
    assert_eq!(
        h1job::deposit_currency(&pool, LIVE).await.unwrap(),
        Deposit::Failed(why.into())
    );
    let step = dev_step(&pool, &f).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("failed")),
        "{step:?}"
    );
    let job = &rs::jobs(&pool).await.unwrap()[0];
    assert_eq!(job.status, "failed");
    assert_eq!(job.reason.as_deref(), Some(why));
    assert!(rs::job_trials(&pool).await.unwrap().is_empty());

    // A deposit asset the list does not name is not guessed either.
    let run2 = load_run(
        &pool,
        Statement::default().listed,
        json!([{"asset_id": 3, "name": "EUR"}]),
    )
    .await;
    assert!(run2 > run);
    assert_eq!(
        h1job::deposit_currency(&pool, LIVE).await.unwrap(),
        Deposit::Failed(format!(
            "the deposit asset 15 is not in load run {run2}'s asset list: the deposit currency cannot be read"
        ))
    );
}

/// Owner, 24-10-2026: a closing deal's quote → deposit rate must agree
/// with the FX close of its day within 1%; a disagreement fails the job.
/// XAUUSD is stated as quoted in EUR: its deal on 4 June 2013 converts at
/// the EURUSD close × 1.005 (agrees), its deal on 5 June at × 1.02 (2%).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_quote_rate_disagreeing_with_the_fx_close_fails_the_job(pool: PgPool) {
    let f = end_to_end_setup_with(&pool, xau_in_eur(eur_close_at(DEAL_MS) * 1.005)).await;
    measured_2014(&pool).await;
    let next_day = DEAL_MS + 86_400_000;
    store_deals(
        &pool,
        &[deal(5, 41, true, next_day, eur_close_at(next_day) * 1.02)],
    )
    .await;
    let step = dev_step(&pool, &f).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("failed")),
        "{step:?}"
    );
    let job = &rs::jobs(&pool).await.unwrap()[0];
    assert_eq!(job.status, "failed");
    let why = job.reason.as_deref().unwrap();
    assert!(
        why.starts_with("XAUUSD: the broker-stated quote currency EUR disagrees with a deal: the closing deal at 2013-06-05 12:00:00 UTC"),
        "{why}"
    );
    assert!(why.contains("(2.00% apart, more than 1%)"), "{why}");
    assert!(rs::job_trials(&pool).await.unwrap().is_empty());
}

/// Owner, 24-10-2026: a development job must not read the quote → deposit
/// rate of any deal executed on or after 2025-10-24: that rate is a price of
/// the sealed period. The deposit check reads rates of any date, but only of
/// symbols the broker quotes in USD, selected in SQL. Deals after the seal:
/// XAUUSD (stated in EUR) at a rate no EURUSD close gives, and EURUSD at 1.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_development_job_reads_no_quote_rate_of_a_sealed_period_deal(pool: PgPool) {
    let agreed = eur_close_at(DEAL_MS) * 1.005;
    let f = end_to_end_setup_with(&pool, xau_in_eur(agreed)).await;
    measured_2014(&pool).await;
    let sealed_ms = at("2025-11-03T12:00:00Z").timestamp_millis();
    store_deals(
        &pool,
        &[
            deal(6, 41, true, sealed_ms, 5.0),
            deal(7, 1, true, sealed_ms, 1.0),
        ],
    )
    .await;

    // The development data layer: deals before the seal only, filtered in
    // the query; a read reaching past the seal is refused, not clamped.
    let dev = Data::new(&pool, Stage::Development);
    let rates = dev
        .closing_rates(LIVE, 41, Stage::Development.end())
        .await
        .unwrap();
    assert_eq!(
        rates,
        vec![(
            DateTime::<Utc>::from_timestamp_millis(DEAL_MS).unwrap(),
            agreed
        )]
    );
    assert!(matches!(
        dev.closing_rates(LIVE, 41, Stage::HeldOut.end()).await,
        Err(DataError::Sealed(_))
    ));
    // A held-out job reads every deal up to its end.
    let held = Data::new(&pool, Stage::HeldOut)
        .closing_rates(LIVE, 41, Stage::HeldOut.end())
        .await
        .unwrap();
    assert_eq!(
        held.iter().map(|r| r.1).collect::<Vec<_>>(),
        vec![agreed, 5.0]
    );
    // The deposit check selects only the USD-quoted EURUSD rates.
    let (run, _, _) = store::latest_assets(&pool, LIVE).await.unwrap().unwrap();
    let mut usd = rs::closing_rates_quoted_in(&pool, LIVE, run, 15)
        .await
        .unwrap();
    usd.sort_by(f64::total_cmp);
    assert_eq!(usd, vec![1.0, 1.0], "deals 2 and 7, never XAUUSD's");

    // The job runs; each symbol's cross-check counts the pre-seal deal only.
    let step = dev_step(&pool, &f).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("done")),
        "{step:?}"
    );
    let t = &rs::job_trials(&pool).await.unwrap()[0];
    let by: HashMap<&str, &Value> = t.params["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| (x["name"].as_str().unwrap(), x))
        .collect();
    assert_eq!(by["XAUUSD"]["quote"], "EUR");
    assert_eq!(
        by["XAUUSD"]["quote_cross_check"],
        "1 closing deals executed before 2025-10-24 00:00:00 UTC agree within 1% (largest difference 0.500%)"
    );
    assert_eq!(
        by["EURUSD"]["quote_cross_check"],
        "1 closing deals executed before 2025-10-24 00:00:00 UTC agree within 1% (largest difference 0.000%)"
    );
    assert_eq!(
        t.params["deposit_currency"]["cross_check"],
        "2 closing deals on symbols the broker quotes in USD, all at quote→deposit rate 1"
    );
}

/// Codex P2 on #9 (sampler.rs:330): a spread sample is final (append-only)
/// and carries the week's bar-basis observations, so it is taken only once a
/// load run has ended `ok` with the symbol's daily bars stored. Here the load
/// ended `partial` before the bars: the pass records nothing and waits. Once
/// the bars land with an `ok` load, the next pass records June 2014 with its
/// basis observations, and the job runs.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_spread_sample_waits_for_a_complete_bar_load(pool: PgPool) {
    let f = end_to_end_setup_with(
        &pool,
        Statement {
            bars_loaded: false,
            ..Default::default()
        },
    )
    .await;
    // The runner waits without connecting to the broker or recording.
    let connects = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (_t, stop) = shutdown::channel();
    let (f2, n) = (f.clone(), connects.clone());
    let connect = move |_account: i64| {
        let f = f2.clone();
        n.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move { Ok(f.spawn().0) }
    };
    let step = runner::step(
        &pool,
        &broker(),
        &[(permitted(), Environment::Live)],
        &connect,
        &stop,
    )
    .await
    .unwrap();
    assert!(
        matches!(&step, runner::Step::Waiting(w)
            if w.contains("waiting for a load run with status ok to store the daily bars of EURUSD, XAUUSD")),
        "{step:?}"
    );
    assert_eq!(connects.load(std::sync::atomic::Ordering::SeqCst), 0);
    // The sampler itself records nothing and asks for no tick either.
    let (conn, log) = f.clone().spawn();
    conn.application_auth("client", &Secret::new("s"))
        .await
        .unwrap();
    let session = conn
        .account_auth(&permitted(), &Secret::new("tok"))
        .await
        .unwrap();
    let sampled = jiaoyi::edge::sampler::sample_symbol(
        &Data::new(&pool, Stage::Development),
        &session,
        LIVE,
        1,
        "EURUSD",
        &[],
        2014..=2014,
    )
    .await
    .unwrap();
    assert_eq!(sampled, jiaoyi::edge::sampler::Sampled::WaitingForBars);
    assert!(log.lock().unwrap().tick_requests.is_empty());
    let samples = rs::spread_samples(&pool, LIVE).await.unwrap();
    assert!(
        samples.iter().all(|s| s.year != 2014),
        "nothing recorded for June 2014: {samples:?}"
    );
    assert_eq!(rs::jobs(&pool).await.unwrap()[0].status, "pending");

    // The bars land with an ok load.
    store_bars(&pool).await;
    let run = store::start_run(&pool, LIVE).await.unwrap();
    resolve_both(&pool, run).await;
    let st = Statement::default();
    store::finish_run(
        &pool,
        run,
        "ok",
        &store::RunSummary {
            listed_symbols: Some(st.listed),
            assets: st.assets,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let step = dev_step(&pool, &f).await;
    assert!(
        matches!(&step, runner::Step::Finished(s) if s.contains("done")),
        "{step:?}"
    );
    let samples = rs::spread_samples(&pool, LIVE).await.unwrap();
    for name in ["EURUSD", "XAUUSD"] {
        let s = samples
            .iter()
            .find(|s| s.symbol == name && s.year == 2014)
            .unwrap();
        assert_eq!(s.status, "measured");
        let basis = s.basis.as_ref().unwrap();
        assert!(
            basis["observations"].as_u64().unwrap() > 0,
            "{name}: {basis}"
        );
        assert_eq!(basis["observations"], basis["bid"], "{name}: {basis}");
    }
}

// ------------------------------------------------- the held-out re-check

/// A steady uptrend in the held-out period: close +0.5 a day from 100, range
/// ±1 (ATR 2), bid bars, spread 0.2, entries from day 100. With breakout 50
/// and stop 3 × ATR the daily rules enter on day 100 at the level 150.5.
fn uptrend(n: usize) -> jiaoyi_core::research::h1::Series {
    let first = at("2026-04-01T00:00:00Z");
    let close: Vec<f64> = (0..n).map(|i| 100.0 + 0.5 * i as f64).collect();
    jiaoyi_core::research::h1::Series {
        symbol: "TEST".into(),
        dates: (0..n)
            .map(|i| (first + Duration::days(i as i64)).date_naive())
            .collect(),
        open: (0..n)
            .map(|i| if i == 0 { 100.0 } else { close[i - 1] })
            .collect(),
        high: close.iter().map(|c| c + 1.0).collect(),
        low: close.iter().map(|c| c - 1.0).collect(),
        close: close.clone(),
        to_ask: vec![0.2; n],
        to_bid: vec![0.0; n],
        spread: vec![0.2; n],
        spread_estimated: vec![false; n],
        commission: vec![0.0; n],
        swap_long: vec![0.0; n],
        swap_short: vec![0.0; n],
        first_entry: 100,
        last: n - 1,
    }
}

fn prepared(s: jiaoyi_core::research::h1::Series) -> jiaoyi::edge::h1job::Prepared {
    use jiaoyi::edge::costs::{Basis, BasisCounts, SpreadBook, SwapTerms, SymbolCosts};
    use jiaoyi_core::research::h1::Indicators;
    let n = s.dates.len();
    let first = at("2026-04-01T00:00:00Z");
    jiaoyi::edge::h1job::Prepared {
        costs: SymbolCosts {
            name: "TEST".into(),
            symbol_id: 7,
            class: AssetClass::Fx,
            basis: Basis::Bid,
            basis_counts: BasisCounts::default(),
            spreads: SpreadBook::default(),
            commission_usd_per_unit: 0.0,
            commission_label: String::new(),
            swap: SwapTerms {
                long: 0.0,
                short: 0.0,
                kind: "PIPS".into(),
                triple_day: None,
                charges_per_day: 1.0,
                pip_size: 0.0001,
                point_size: 0.00001,
                read_at: None,
            },
            quote: "USD".into(),
            quote_label: "test".into(),
            quote_cross_check: "test".into(),
            min_units: None,
            step_units: None,
        },
        indicators: Indicators::of(&s),
        series: s,
        usd_per_price: vec![1.0; n],
        opens: (0..n).map(|i| first + Duration::days(i as i64)).collect(),
    }
}

fn e5(p: f64) -> i64 {
    (p * 100_000.0).round() as i64
}

const HOUR: i64 = 3_600_000;

/// The day-100 entry ticks: bid 150.0, 150.5, 151.5 an hour apart, the ask
/// 0.2 above: the ask first reaches 150.5 at 150.7 (fill 150.7 + 0.2).
fn entry_day(x: &jiaoyi::edge::h1job::Prepared) -> Vec<(i64, i64)> {
    let t = x.opens[100].timestamp_millis();
    vec![
        (t + HOUR, e5(150.0)),
        (t + 2 * HOUR, e5(150.5)),
        (t + 3 * HOUR, e5(151.5)),
    ]
}

fn asks_of(bids: &[(i64, i64)]) -> Vec<(i64, i64)> {
    bids.iter().map(|(t, p)| (*t, p + e5(0.2))).collect()
}

/// Re-check `x` (breakout 50, stop 3 × ATR, 1× slippage) against a fake
/// broker serving these ticks. The fake sends ticks newest first and keeps
/// the scripted order of ticks sharing a millisecond, so a burst within one
/// millisecond is scripted newest first. Returns the trades, the notes and
/// the bar opens (ms) of the days streamed.
async fn recheck_on(
    pool: &PgPool,
    x: &jiaoyi::edge::h1job::Prepared,
    bids: Vec<(i64, i64)>,
    asks: Vec<(i64, i64)>,
) -> (
    Vec<jiaoyi_core::research::h1::Trade>,
    Vec<jiaoyi::edge::recheck::Recheck>,
    std::collections::BTreeSet<i64>,
) {
    let f = FakeBroker {
        account_id: LIVE,
        tick_history: HashMap::from([((7, 1), bids), ((7, 2), asks)]),
        ..Default::default()
    };
    let (conn, log) = f.spawn();
    conn.application_auth("client", &Secret::new("s"))
        .await
        .unwrap();
    let session = conn
        .account_auth(&permitted(), &Secret::new("tok"))
        .await
        .unwrap();
    let data = Data::new(pool, Stage::HeldOut);
    let p = jiaoyi_core::research::h1::H1Params {
        breakout: 50,
        stop_atr: 3.0,
        slip_mult: 1.0,
    };
    let (trades, notes) = jiaoyi::edge::recheck::simulate_with_ticks(&data, &session, x, &p)
        .await
        .unwrap();
    let days = log
        .lock()
        .unwrap()
        .tick_requests
        .iter()
        .map(|(_, _, from, _)| *from)
        .collect();
    (trades, notes, days)
}

/// Held-out trades are re-checked tick by tick on their entry and exit days.
/// A steady uptrend (close +0.5 a day, range ±1, ATR 2, bid bars, spread
/// 0.2, slippage 1×) in the held-out period; the daily rules enter on day
/// 100 at 150.5 + 0.2 + 0.2 = 150.9 (stop 3 × 2 = 6 below) and exit on day
/// 111 at the stop 150 − 0.2 = 149.8.
///
/// The broker's ticks: on day 100 the ask first reaches the level at 150.7
/// (fill 150.7 + 0.2 = 150.9); on day 111 the bid rises to 156.5 (a new best:
/// the stop trails to 156.5 − 6 = 150.5 tick by tick) and then falls to
/// 150.4, exiting at 150.4 − 0.2 = 150.2. Tick R = (150.2 − 150.9) ÷ 6;
/// daily R = (149.8 − 150.9) ÷ 6; the difference is 0.4 ÷ 6. The tick
/// result is the one counted.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn held_out_trades_are_rechecked_on_ticks(pool: PgPool) {
    use jiaoyi_core::research::h1::ExitReason;
    let mut s = uptrend(112);
    // Day 111 falls through the stop.
    s.open[111] = 155.0;
    s.high[111] = 155.5;
    s.low[111] = 149.0;
    s.close[111] = 149.5;
    let x = prepared(s);
    let t100 = x.opens[100].timestamp_millis();
    let t111 = x.opens[111].timestamp_millis();
    let mut bids = entry_day(&x);
    bids.extend([
        (t111 + HOUR, e5(154.0)),
        (t111 + 2 * HOUR, e5(156.5)),
        (t111 + 3 * HOUR, e5(150.4)),
    ]);
    let asks = asks_of(&bids);
    let (trades, notes, days) = recheck_on(&pool, &x, bids, asks).await;
    assert_eq!(trades.len(), 1);
    let t = &trades[0];
    assert!((t.entry_fill - 150.9).abs() < 1e-9, "{t:?}");
    assert_eq!(t.exit_index, 111);
    assert_eq!(t.exit_reason, ExitReason::Stop);
    assert!((t.exit_fill - 150.2).abs() < 1e-9, "{t:?}");
    assert!((t.gross_r - (150.2 - 150.9) / 6.0).abs() < 1e-9);
    let d = notes[0].daily.as_ref().unwrap();
    assert!((d.exit_fill - 149.8).abs() < 1e-9, "{d:?}");
    assert!((notes[0].diff_r.unwrap() - 0.4 / 6.0).abs() < 1e-9);
    // Only the entry and exit days were streamed (one window per side each).
    assert_eq!(days, std::collections::BTreeSet::from([t100, t111]));
}

/// Codex P2 on #9 (sampler.rs:79): an ask that reaches the level only
/// inside a burst of updates sharing one millisecond is a fill. The ask
/// bursts 150.3 → 150.6 → 150.3 at the same millisecond and otherwise
/// stays below the level 150.5: the fill is 150.6 + 0.2 = 150.8.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_entry_crossing_inside_one_millisecond_fills(pool: PgPool) {
    let x = prepared(uptrend(112));
    let t = x.opens[100].timestamp_millis();
    let bids = vec![(t + HOUR, e5(150.0)), (t + 3 * HOUR, e5(150.2))];
    let asks = vec![
        (t + HOUR, e5(150.2)),
        // Newest first: oldest-first 150.3, 150.6, 150.3.
        (t + 2 * HOUR, e5(150.3)),
        (t + 2 * HOUR, e5(150.6)),
        (t + 2 * HOUR, e5(150.3)),
        (t + 3 * HOUR, e5(150.4)),
    ];
    let (trades, notes, _) = recheck_on(&pool, &x, bids, asks).await;
    assert_eq!(trades.len(), 1, "{notes:?}");
    assert_eq!(trades[0].entry_index, 100);
    assert!(
        (trades[0].entry_fill - 150.8).abs() < 1e-9,
        "{:?}",
        trades[0]
    );
}

/// Codex P2 on #9 (sampler.rs:79): a bid that crosses the trailed stop only
/// inside a burst sharing one millisecond exits. On day 111 (a daily exit
/// day) the bid bursts 156.5 → 150.4 → 155.0 at one millisecond: the new
/// best 156.5 trails the stop to 150.5 and 150.4 is beyond it, so the trade
/// exits at 150.4 − 0.2 = 150.2. Collapsed to its last price (155.0), the
/// burst is never checked.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_stop_crossing_inside_one_millisecond_exits(pool: PgPool) {
    use jiaoyi_core::research::h1::ExitReason;
    let mut s = uptrend(112);
    s.open[111] = 155.0;
    s.high[111] = 156.6;
    s.low[111] = 149.0;
    s.close[111] = 155.2;
    let x = prepared(s);
    let t111 = x.opens[111].timestamp_millis();
    let mut bids = entry_day(&x);
    bids.extend([
        (t111 + HOUR, e5(154.0)),
        // Newest first: oldest-first 156.5, 150.4, 155.0.
        (t111 + 2 * HOUR, e5(155.0)),
        (t111 + 2 * HOUR, e5(150.4)),
        (t111 + 2 * HOUR, e5(156.5)),
        (t111 + 3 * HOUR, e5(155.2)),
    ]);
    let asks = asks_of(&bids);
    let (trades, _, _) = recheck_on(&pool, &x, bids, asks).await;
    assert_eq!(trades.len(), 1);
    let t = &trades[0];
    assert_eq!(
        (t.exit_index, t.exit_reason),
        (111, ExitReason::Stop),
        "{t:?}"
    );
    assert!((t.exit_fill - 150.2).abs() < 1e-9, "{t:?}");
}

/// Codex P2 on #9 (recheck.rs:271): a trade open on the period's last day
/// is re-checked on that day's ticks. With no stop reached it closes at the
/// day's last bid, 154.3, less the slippage: 154.1 (the daily close would
/// give 155.5 − 0.2 = 155.3).
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_period_end_exit_closes_at_the_last_days_last_quote(pool: PgPool) {
    use jiaoyi_core::research::h1::ExitReason;
    let x = prepared(uptrend(112));
    let t100 = x.opens[100].timestamp_millis();
    let t111 = x.opens[111].timestamp_millis();
    let mut bids = entry_day(&x);
    bids.extend([
        (t111 + HOUR, e5(155.0)),
        (t111 + 2 * HOUR, e5(155.4)),
        (t111 + 3 * HOUR, e5(154.3)),
    ]);
    let asks = asks_of(&bids);
    let (trades, _, days) = recheck_on(&pool, &x, bids, asks).await;
    assert_eq!(trades.len(), 1);
    let t = &trades[0];
    assert_eq!(
        (t.exit_index, t.exit_reason),
        (111, ExitReason::PeriodEnd),
        "{t:?}"
    );
    assert!((t.exit_ref - 154.3).abs() < 1e-9, "{t:?}");
    assert!((t.exit_fill - 154.1).abs() < 1e-9, "{t:?}");
    assert_eq!(days, std::collections::BTreeSet::from([t100, t111]));
}

/// Codex P2 on #9 (recheck.rs:271): the last day is streamed like an exit
/// day, so a tick trail hit on it counts. The bid makes a new best at 157.5
/// (stop trailed to 151.5), then falls to 151.4: a stop exit at 151.2,
/// where the daily rules found no exit (low 154.5) and closed at 155.3.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_trail_hit_on_the_period_end_day_counts(pool: PgPool) {
    use jiaoyi_core::research::h1::ExitReason;
    let x = prepared(uptrend(112));
    let t111 = x.opens[111].timestamp_millis();
    let mut bids = entry_day(&x);
    bids.extend([
        (t111 + HOUR, e5(157.5)),
        (t111 + 2 * HOUR, e5(151.4)),
        (t111 + 3 * HOUR, e5(155.0)),
    ]);
    let asks = asks_of(&bids);
    let (trades, notes, _) = recheck_on(&pool, &x, bids, asks).await;
    assert_eq!(trades.len(), 1);
    let t = &trades[0];
    assert_eq!(
        (t.exit_index, t.exit_reason),
        (111, ExitReason::Stop),
        "{t:?}"
    );
    assert!((t.exit_fill - 151.2).abs() < 1e-9, "{t:?}");
    let d = notes[0].daily.as_ref().unwrap();
    assert_eq!(d.exit_reason, ExitReason::PeriodEnd);
    assert!((d.exit_fill - 155.3).abs() < 1e-9, "{d:?}");
}

/// Codex P2 on #9 (h1job.rs:460): on held-out data, rule 2 holds only when
/// both the daily-bar-executed mean and the tick-counted mean are above the
/// 95th percentile of the daily-bar controls; each half fails it alone.
#[test]
fn held_out_rule_2_needs_the_daily_bar_and_the_tick_mean_above_the_control() {
    use jiaoyi::edge::costs::CommissionBook;
    use jiaoyi::edge::h1job::{evaluate, Setting, Universe};
    use jiaoyi_core::research::h1::{ExitReason, Side, Trade};
    let x = prepared(uptrend(400));
    let trades = |net_r: f64| -> Vec<Trade> {
        (0..100)
            .map(|k| {
                let t = 110 + 2 * k;
                Trade {
                    symbol: "TEST".into(),
                    side: Side::Long,
                    entry_index: t,
                    entry_date: x.series.dates[t],
                    level: Some(x.series.close[t]),
                    entry_ref: x.series.close[t],
                    entry_fill: x.series.close[t],
                    atr: 2.0,
                    stop_distance: 6.0,
                    initial_stop: x.series.close[t] - 6.0,
                    exit_index: t + 1,
                    exit_date: x.series.dates[t + 1],
                    exit_ref: 0.0,
                    exit_fill: 0.0,
                    exit_reason: ExitReason::Stop,
                    nights: 1,
                    gross_r: net_r,
                    commission_r: 0.0,
                    swap_r: 0.0,
                    // Alternate around the mean so the bootstrap and PF hold.
                    net_r: net_r + if k % 2 == 0 { 0.5 } else { -0.5 },
                    spread_estimated: false,
                }
            })
            .collect()
    };
    let u = Universe {
        symbols: vec![x.clone()],
        refused: vec![],
        settings_version: 1,
        commission: CommissionBook {
            per_class: Default::default(),
            overall_median: 0.0,
            deals: 0,
        },
        deposit: json!({}),
    };
    let setting = Setting {
        breakout: 50,
        stop_atr: 3.0,
        account_id: LIVE,
    };
    let rule2 = |tick: f64, daily: f64| {
        let (results, eval) = evaluate(
            &u,
            &setting,
            &[trades(tick)],
            Stage::HeldOut,
            Some(&[trades(daily)]),
        );
        (eval.rules.rule2_above_control_p95, results)
    };
    // Both far above any control run mean: rule 2 holds.
    let (both, results) = rule2(50.0, 40.0);
    assert!(both, "{}", results["control"]);
    // The tick mean above, the daily-bar mean below: rule 2 fails.
    let (tick_only, results) = rule2(50.0, -50.0);
    assert!(!tick_only, "{}", results["control"]);
    assert_eq!(results["control"]["daily_bar_percentile"], 0.0);
    assert_eq!(results["control"]["strategy_percentile"], 100.0);
    assert!(
        (results["evidence_against"]["tick_vs_daily_bar"]["mean_r_difference"]
            .as_f64()
            .unwrap()
            - 100.0)
            .abs()
            < 1e-9
    );
    // The daily-bar mean above, the tick mean below: rule 2 fails.
    let (daily_only, results) = rule2(-50.0, 50.0);
    assert!(!daily_only, "{}", results["control"]);
    assert_eq!(results["control"]["daily_bar_percentile"], 100.0);
}
