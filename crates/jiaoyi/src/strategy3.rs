//! Strategy 3: intraday momentum on the noise band (owner-approved v2,
//! 25-10-2026 ~10:45 SGT; entry "Tick-triggered only", ~10:50 SGT).
//! The rules are `docs/preregistration-2026-10-25-strategy3.md`, embedded
//! here and hashed: every decision, intent and trade records that hash and
//! the label `jiaoyi-s3`.
//!
//! One loop per permitted account, demo and live ("Live too, from now"):
//! - **Sessions.** Each symbol trades its own session (`jiaoyi_core::strategy3::sessions`).
//! - **Band.** At the session's first quote after 30 minutes, the M30 bars
//!   of the last sessions are read from the broker. They give the session's
//!   open and σ at each 30-minute mark. A session without a first bar with
//!   ticks is a holiday.
//! - **Entry.** On every quote: the band at that instant (σ interpolated
//!   between marks). The first ask above it buys, the first bid below it
//!   sells. There is no entry in the session's last 30 minutes, and at most
//!   one Strategy 3 position per symbol.
//! - **Exits** (bot-trade `risk_coverage_partial_v1`).
//!   - The stop is R from the fill.
//!   - The broker take-profit is TP2, or TP1 when the size cannot be split.
//!   - At TP1 the trader closes the planned part.
//!   - At the session end it closes what is left.
//! - **Size.** A demo account sizes `trading.risk_pct` of fresh equity, floored
//!   to the step, and skips below the minimum. A live account takes one
//!   minimum trade at a time, at most `trading.min_volume_max_risk_pct` of
//!   equity, and only while no position of this program is open on it.
//! - **Budget.** Strategy 3's open risk at its stops stays within
//!   `trading.s3_open_risk_pct` of equity.
//!
//! Every order goes through `jiaoyi_ctrader::trading::send`, the one gate.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::strategy3::band::{self, Band, SigmaTable, M30};
use jiaoyi_core::strategy3::sessions::{self, Session, SessionRule};
use jiaoyi_core::strategy3::targets::{self, Dir, Plan};
use jiaoyi_core::strategy3::{cost_reserve, slippage_bps_per_side, CostReserve};
use jiaoyi_core::universe::AssetClass;
use jiaoyi_ctrader::trading::{
    self as gate, Command, EventRecord, Intent, IntentWrite, Ledger, Outcome, Policy, SendError,
    Side, VolumeLimits,
};
use jiaoyi_ctrader::{AccountSession, Connection, ReceivedExecution};
use jiaoyi_store::sqlx::{PgPool, Row};
use jiaoyi_store::strategy3 as s3s;
use jiaoyi_store::trading as ts;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::broadcast::error::RecvError;

use crate::config::Config;
use crate::edge::costs;
use crate::edge::h1job::QuoteAssets;
use crate::loader;
use crate::shutdown::Shutdown;
use crate::trading::{fresh_equity, relative_stop, risk_at_stop, PgLedger};

/// The strategy's name on every route and record.
pub const STRATEGY: &str = "Strategy 3: intraday momentum";
/// The label on every position Strategy 3 opens.
pub const LABEL: &str = "jiaoyi-s3";
/// The pre-registration, relative to the repository root.
pub const PREREG_FILE: &str = "docs/preregistration-2026-10-25-strategy3.md";
/// Its text, compiled in.
pub const PREREG_TEXT: &str = include_str!("../../../docs/preregistration-2026-10-25-strategy3.md");
/// A trigger that was skipped or refused is taken again on that symbol only
/// after this long (self-derived: the gate reads the broker twice per try).
pub const ENTRY_RETRY: Duration = Duration::from_secs(60);
/// A session whose band could not be read (a broker error) is read again
/// after this long.
pub const PREPARE_RETRY: Duration = Duration::from_secs(300);
/// Calendar days of M30 bars read for the band (30 sessions and weekends).
pub const BAR_DAYS: i64 = 45;
/// The commission book is read again this often.
pub const COMMISSION_REFRESH: Duration = Duration::from_secs(3600);

/// SHA-256 of the embedded pre-registration, hex.
pub fn prereg_hash() -> String {
    Sha256::digest(PREREG_TEXT.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn dir_side(d: Dir) -> Side {
    match d {
        Dir::Buy => Side::Buy,
        Dir::Sell => Side::Sell,
    }
}

// ---------------------------------------------------------------- settings

/// Strategy 3's rules for one account, read fresh.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rules {
    pub version: i64,
    pub live: bool,
    pub risk_pct: f64,
    pub cap_factor: f64,
    pub min_volume_max_risk_pct: f64,
    pub open_risk_pct: f64,
    pub reconcile_secs: u64,
    pub order_wait_secs: u64,
    pub retry_secs: u64,
}

pub async fn rules(pool: &PgPool, account_id: i64, env: Environment) -> anyhow::Result<Rules> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let i = |k: &str| s.integer(k).map_err(anyhow::Error::msg);
    let n = |k: &str| s.number(k).map_err(anyhow::Error::msg);
    Ok(Rules {
        version: s.version,
        live: env == Environment::Live,
        risk_pct: n("trading.risk_pct")?,
        cap_factor: n("trading.cap_factor")?,
        min_volume_max_risk_pct: n("trading.min_volume_max_risk_pct")?,
        open_risk_pct: n("trading.s3_open_risk_pct")?,
        reconcile_secs: i("trading.reconcile_secs")?.max(1) as u64,
        order_wait_secs: i("trading.order_wait_secs")?.max(1) as u64,
        retry_secs: i("trading.retry_secs")?.max(1) as u64,
    })
}

// ---------------------------------------------------------------- pure parts

/// The entry signal of a quote at `now`: the band at that instant and the
/// side the quote triggers, `Ok(None)` inside the band, `Err(why)` when no
/// entry may be taken now.
pub fn signal(
    table: &SigmaTable,
    session: &Session,
    open: f64,
    now: DateTime<Utc>,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Result<Option<(Dir, Band)>, String> {
    band::entry_window(session, now)?;
    let b = band::band_at(table, session, open, now)?;
    Ok(band::triggered(&b, bid, ask).map(|d| (d, b)))
}

/// How the volume was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Sized {
    pub volume: i64,
    /// The most the gate lets this open risk.
    pub max_risk: f64,
    pub mode: &'static str,
    pub risk: f64,
}

/// Size an open: demo `risk_pct` of equity floored to the step (below the
/// minimum: skipped); live exactly the broker minimum, within
/// `min_volume_max_risk_pct` of equity, and only with no other position of
/// this program open on the account.
pub fn size(
    r: &Rules,
    equity: f64,
    relative_stop: i64,
    quote_to_deposit: f64,
    limits: &VolumeLimits,
    program_positions_open: usize,
) -> Result<Sized, String> {
    if !(equity.is_finite() && equity > 0.0) {
        return Err(format!("equity {equity} is not positive"));
    }
    if !(quote_to_deposit.is_finite() && quote_to_deposit > 0.0) || relative_stop <= 0 {
        return Err("no stop or no conversion rate".into());
    }
    if limits.step <= 0 || limits.min <= 0 {
        return Err("the symbol's volume rules are unknown".into());
    }
    if r.live {
        if program_positions_open > 0 {
            return Err(format!(
                "live: one minimum trade at a time, and {program_positions_open} position(s) of this program are open on the account"
            ));
        }
        let risk = gate::risk_of(limits.min, relative_stop, quote_to_deposit);
        let ceiling = equity * r.min_volume_max_risk_pct / 100.0;
        if risk > ceiling {
            return Err(format!(
                "live: the broker minimum {} would risk {risk:.2}, above {}% of equity ({ceiling:.2})",
                limits.min, r.min_volume_max_risk_pct
            ));
        }
        return Ok(Sized {
            volume: limits.min,
            max_risk: ceiling,
            mode: "live_broker_minimum",
            risk,
        });
    }
    let per_cent = gate::risk_of(1, relative_stop, quote_to_deposit);
    let allowed = equity * r.risk_pct / 100.0;
    let raw = allowed / per_cent;
    let mut volume = (raw / limits.step as f64).floor() as i64 * limits.step;
    if let Some(max) = limits.max {
        volume = volume.min(max - max % limits.step);
    }
    if volume < limits.min {
        return Err(format!(
            "{}% of equity sizes {raw:.2} cents, below the broker minimum {}: skipped, never rounded up",
            r.risk_pct, limits.min
        ));
    }
    Ok(Sized {
        volume,
        max_risk: r.cap_factor * allowed,
        mode: "risk_pct",
        risk: gate::risk_of(volume, relative_stop, quote_to_deposit),
    })
}

/// A price distance in whole ticks of `digits`, as 1/100000 units.
pub fn distance_e5(distance: f64, digits: i32) -> i64 {
    let ticks = (distance.abs() * 10f64.powi(digits)).round() as i64;
    ticks * 10_i64.pow((5 - digits.clamp(0, 5)) as u32)
}

/// Everything an entry decides, before sizing: R and the cost reserve.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entry {
    pub side: Dir,
    pub band: Band,
    /// The triggering quote side: the ask for a buy, the bid for a sell.
    pub entry_price: f64,
    pub relative_stop: i64,
    pub r: f64,
    pub cost: CostReserve,
}

/// The order of an entry once sized: the plan and its relative brackets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Order {
    pub plan: Plan,
    pub relative_take_profit: i64,
    /// TP1's distance from the fill, in price units.
    pub tp1_distance: f64,
    /// TP2's distance from the fill.
    pub tp2_distance: f64,
}

/// Plan the exits for `volume` (bot-trade's arithmetic). `Err` is its reason.
pub fn plan_order(e: &Entry, limits: &VolumeLimits, volume: i64) -> Result<Order, String> {
    let r = e.relative_stop as f64 / 100_000.0;
    let stop = match e.side {
        Dir::Buy => e.entry_price - r,
        Dir::Sell => e.entry_price + r,
    };
    let plan = targets::plan(&targets::Input {
        side: e.side,
        entry: e.entry_price,
        original_stop: stop,
        required_rr: targets::REQUIRED_RR,
        cost_reserve: e.cost.total,
        digits: limits.digits,
        volume,
        min_volume: limits.min,
        step_volume: limits.step,
    })
    .map_err(|why| format!("bot-trade {}: {why}", targets::POLICY))?;
    let tp1 = (plan.trigger - e.entry_price).abs();
    let tp2 = (plan.runner_target - e.entry_price).abs();
    let broker = (plan.broker_target - e.entry_price).abs();
    Ok(Order {
        plan,
        relative_take_profit: distance_e5(broker, limits.digits),
        tp1_distance: tp1,
        tp2_distance: tp2,
    })
}

/// Whether a quote reaches a long's (bid) or a short's (ask) TP1, compared
/// in whole ticks of `digits`.
pub fn tp1_reached(
    side: Side,
    fill: f64,
    tp1_distance: f64,
    digits: i32,
    bid: Option<f64>,
    ask: Option<f64>,
) -> bool {
    let f = 10f64.powi(digits.clamp(0, 10));
    let ticks = |p: f64| (p * f).round() as i64;
    let target = match side {
        Side::Buy => ticks(fill) + ticks(tp1_distance),
        Side::Sell => ticks(fill) - ticks(tp1_distance),
    };
    match side {
        Side::Buy => bid.is_some_and(|b| ticks(b) >= target),
        Side::Sell => ask.is_some_and(|a| ticks(a) <= target),
    }
}

/// The open-risk budget: Strategy 3's open risk at its stops plus a new
/// entry's must stay within `pct` of equity.
pub fn within_budget(open_risk: f64, new_risk: f64, equity: f64, pct: f64) -> Result<(), String> {
    let budget = equity * pct / 100.0;
    if open_risk + new_risk > budget * (1.0 + 1e-9) {
        Err(format!(
            "Strategy 3's open risk {open_risk:.2} plus this entry's {new_risk:.2} would exceed its budget {budget:.2} ({pct}% of equity): skipped"
        ))
    } else {
        Ok(())
    }
}

/// The deposit currency per unit of `quote`, from the live mid of a pair
/// joining them (`QUOTEDEPOSIT` or `DEPOSITQUOTE`).
pub fn quote_to_deposit(quote: &str, deposit: &str, mids: &HashMap<String, f64>) -> Option<f64> {
    if quote == deposit {
        return Some(1.0);
    }
    let mid = |n: &str| mids.get(n).copied().filter(|m| m.is_finite() && *m > 0.0);
    mid(&format!("{quote}{deposit}")).or_else(|| mid(&format!("{deposit}{quote}")).map(|m| 1.0 / m))
}

// ---------------------------------------------------------------- ledger

/// The gate's ledger for Strategy 3: the intent in `trading_intents`, then
/// the decided order (hash, label, quote, band, plan) in `strategy3_orders`,
/// before the frame. If the second write fails, nothing is sent.
pub struct S3Ledger<'a> {
    pub inner: PgLedger<'a>,
    pub order: s3s::OrderRow,
    /// When the intent was recorded: just before the frame is written.
    pub recorded_at: Mutex<Option<DateTime<Utc>>>,
}

impl Ledger for S3Ledger<'_> {
    async fn record_intent(&self, i: &Intent) -> Result<IntentWrite, String> {
        let w = self.inner.record_intent(i).await?;
        if w == IntentWrite::Recorded {
            s3s::record_order(self.inner.pool, &self.order)
                .await
                .map_err(|e| format!("the Strategy 3 order record failed: {e}"))?;
            *self.recorded_at.lock().unwrap() = Some(Utc::now());
        }
        Ok(w)
    }
    async fn record_event(&self, e: &EventRecord) -> Result<(), String> {
        self.inner.record_event(e).await
    }
    async fn owns_position(&self, account_id: i64, position_id: i64) -> Result<bool, String> {
        self.inner.owns_position(account_id, position_id).await
    }
}

// ---------------------------------------------------------------- state

/// One symbol's session preparation: the open, σ and the median spread.
#[derive(Debug, Clone)]
pub struct Day {
    pub session: Session,
    pub open: Result<f64, String>,
    pub table: SigmaTable,
    pub spread: Result<(f64, i64), String>,
    pub prepared_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Sym {
    pub id: i64,
    pub name: String,
    pub class: AssetClass,
    pub rule: Option<SessionRule>,
    pub limits: Option<VolumeLimits>,
    pub quote: Option<String>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub day: Option<Result<Day, (String, DateTime<Utc>)>>,
    /// The last entry attempt, so a refused trigger waits [`ENTRY_RETRY`].
    pub last_try: Option<DateTime<Utc>>,
}

/// A Strategy 3 position as reconciled, with its plan when recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Own {
    pub position_id: i64,
    pub symbol_id: i64,
    pub side: Side,
    pub volume: i64,
    pub entry: Option<f64>,
    pub stop: Option<f64>,
    pub take_profit: Option<f64>,
    /// From the recorded plan.
    pub tp1_distance: Option<f64>,
    pub close_volume: Option<i64>,
    pub partial: bool,
    pub session_date: Option<chrono::NaiveDate>,
    pub entry_key: Option<String>,
    pub tp1_sent: bool,
    pub end_sent: bool,
}

/// An open sent but not answered (Unconfirmed): its risk is held in the
/// budget, and on a live account it counts as an open position, until a
/// reconcile finds it.
#[derive(Debug, Clone, PartialEq)]
struct Pending {
    entry_key: String,
    symbol_id: i64,
    risk: f64,
    at: DateTime<Utc>,
}

/// Records Strategy 3's decisions (kinds `s3_*`) when they change, with the
/// pre-registration hash and the label.
pub struct Decisions<'a> {
    pool: &'a PgPool,
    account_id: i64,
    last: HashMap<String, String>,
    version: Option<i64>,
    hash: String,
}

impl<'a> Decisions<'a> {
    pub fn new(pool: &'a PgPool, account_id: i64) -> Decisions<'a> {
        Decisions {
            pool,
            account_id,
            last: HashMap::new(),
            version: None,
            hash: prereg_hash(),
        }
    }

    pub async fn note(
        &mut self,
        symbol: Option<&str>,
        kind: &str,
        reason: &str,
        mut detail: Value,
    ) {
        let k = format!("{}:{kind}", symbol.unwrap_or("-"));
        if self.last.get(&k).map(String::as_str) == Some(reason) {
            return;
        }
        self.last.insert(k, reason.to_string());
        detail["strategy"] = json!(STRATEGY);
        detail["label"] = json!(LABEL);
        detail["prereg_hash"] = json!(self.hash);
        tracing::info!(account = self.account_id, symbol = ?symbol, kind, reason, "strategy 3 decision");
        if let Err(e) = ts::record_decision(
            self.pool,
            self.account_id,
            symbol,
            &format!("s3_{kind}"),
            reason,
            &detail,
            self.version,
        )
        .await
        {
            tracing::error!(error = %e, "strategy 3 decision not recorded");
        }
    }
}

/// The trader of one account on an authorised session. Its methods take the
/// time, so the tests drive it quote by quote.
pub struct Trader<'a, 'c> {
    pub pool: &'a PgPool,
    pub allowlist: &'a Allowlist,
    pub env: Environment,
    pub session: &'a AccountSession<'c>,
    pub syms: HashMap<i64, Sym>,
    pub own: HashMap<i64, Own>,
    pub deposit: Option<String>,
    pub commission: Result<costs::CommissionBook, String>,
    commission_read: Option<std::time::Instant>,
    pending: Vec<Pending>,
    /// Program positions on the account at the last reconcile (any strategy).
    program_positions: usize,
    pub d: Decisions<'a>,
    hash: String,
}

impl<'a, 'c> Trader<'a, 'c> {
    /// Read the account's symbols, deposit currency and commission book.
    pub async fn new(
        pool: &'a PgPool,
        allowlist: &'a Allowlist,
        env: Environment,
        session: &'a AccountSession<'c>,
    ) -> anyhow::Result<Trader<'a, 'c>> {
        let account_id = session.account().id();
        let quotes = jiaoyi_store::latest_assets(pool, account_id)
            .await?
            .map(|(run, assets, listed)| QuoteAssets::from_run(run, &assets, &listed));
        let deposit = match (
            jiaoyi_store::research::deposit_asset_id(pool, account_id).await?,
            &quotes,
        ) {
            (Some(id), Some(q)) => q.asset_name(id).map(str::to_string),
            _ => None,
        };
        let snap = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
        let universe = snap
            .universe("universe.symbols")
            .map_err(anyhow::Error::msg)?;
        let resolutions = jiaoyi_store::latest_resolutions(pool, account_id).await?;
        let mut syms = HashMap::new();
        for (name, class) in universe {
            let Some(id) = resolutions
                .iter()
                .find(|r| r.name == name)
                .and_then(|r| r.broker_symbol_id)
            else {
                continue;
            };
            let limits = jiaoyi_store::sqlx::query(
                "SELECT min_volume, volume_step, max_volume, digits FROM symbols
                 WHERE account_id = $1 AND broker_symbol_id = $2",
            )
            .bind(account_id)
            .bind(id)
            .fetch_optional(pool)
            .await?
            .and_then(|row| {
                Some(VolumeLimits {
                    min: row.get::<Option<i64>, _>("min_volume")?,
                    step: row.get::<Option<i64>, _>("volume_step")?,
                    max: row.get::<Option<i64>, _>("max_volume"),
                    digits: row.get::<i32, _>("digits"),
                })
            });
            syms.insert(
                id,
                Sym {
                    id,
                    name: name.clone(),
                    class,
                    rule: sessions::rule_for(&name, class),
                    limits,
                    quote: quotes
                        .as_ref()
                        .and_then(|q| q.quote_of(id).ok().map(|x| x.0)),
                    bid: None,
                    ask: None,
                    day: None,
                    last_try: None,
                },
            );
        }
        let mut t = Trader {
            pool,
            allowlist,
            env,
            session,
            syms,
            own: HashMap::new(),
            deposit,
            commission: Err("not read yet".into()),
            commission_read: None,
            pending: Vec::new(),
            program_positions: 0,
            d: Decisions::new(pool, account_id),
            hash: prereg_hash(),
        };
        t.read_commission().await;
        Ok(t)
    }

    fn account_id(&self) -> i64 {
        self.session.account().id()
    }

    async fn read_commission(&mut self) {
        self.commission =
            match jiaoyi_store::research::commission_per_unit(self.pool, self.account_id()).await {
                Ok(samples) => costs::commission_book(&samples),
                Err(e) => Err(format!("deals unreadable: {e}")),
            };
        self.commission_read = Some(std::time::Instant::now());
    }

    fn mids(&self) -> HashMap<String, f64> {
        self.syms
            .values()
            .filter_map(|s| Some((s.name.clone(), (s.bid? + s.ask?) / 2.0)))
            .collect()
    }

    fn rate(&self, symbol_id: i64) -> Option<f64> {
        let s = self.syms.get(&symbol_id)?;
        quote_to_deposit(s.quote.as_deref()?, self.deposit.as_deref()?, &self.mids())
    }

    /// Reconcile with the broker: Strategy 3's own positions, with their
    /// recorded plans, and the count of this program's positions.
    pub async fn reconcile(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let account_id = self.account_id();
        let res = self.session.reconcile().await?;
        let plans = s3s::open_plans(self.pool, account_id).await?;
        let unconfirmed: Vec<(String, Option<i64>)> =
            ts::open_intents_without_fill(self.pool, account_id)
                .await?
                .into_iter()
                .filter(|(k, _)| k.starts_with("s3:"))
                .collect();
        let mut seen = Vec::new();
        self.program_positions = 0;
        for p in &res.position {
            let label = p.trade_data.label.as_deref();
            if label.is_some_and(|l| gate::PROGRAM_LABELS.contains(&l)) {
                self.program_positions += 1;
            }
            if label != Some(LABEL) {
                continue;
            }
            let mut owned = ts::owns_position(self.pool, account_id, p.position_id).await?;
            if !owned {
                if let Some((key, _)) = unconfirmed
                    .iter()
                    .find(|(_, s)| *s == Some(p.trade_data.symbol_id))
                {
                    self.adopt(key, p).await?;
                    owned = true;
                }
            }
            if !owned {
                continue;
            }
            let Some(side) = Side::from_proto(p.trade_data.trade_side) else {
                continue;
            };
            seen.push(p.position_id);
            let plan = plans.iter().rev().find(|x| x.position_id == p.position_id);
            let tp1_sent =
                s3s::order_for_position(self.pool, account_id, p.position_id, "tp1").await?;
            let end_sent =
                s3s::order_for_position(self.pool, account_id, p.position_id, "session_end")
                    .await?;
            let e = self.own.entry(p.position_id).or_insert_with(|| Own {
                position_id: p.position_id,
                symbol_id: p.trade_data.symbol_id,
                side,
                volume: p.trade_data.volume,
                entry: p.price,
                stop: p.stop_loss,
                take_profit: p.take_profit,
                tp1_distance: plan.and_then(|x| x.detail["order"]["tp1_distance"].as_f64()),
                close_volume: plan.and_then(|x| x.detail["order"]["plan"]["close_volume"].as_i64()),
                partial: plan.is_some_and(|x| {
                    x.detail["order"]["plan"]["mode"].as_str() == Some("partial_runner")
                }),
                session_date: plan.and_then(|x| {
                    x.detail["session_date"]
                        .as_str()
                        .and_then(|d| d.parse().ok())
                }),
                entry_key: plan.map(|x| x.entry_key.clone()),
                tp1_sent,
                end_sent,
            });
            // The broker wins.
            e.volume = p.trade_data.volume;
            e.stop = p.stop_loss;
            e.take_profit = p.take_profit;
            if p.price.is_some() {
                e.entry = p.price;
            }
            e.tp1_sent |= tp1_sent;
        }
        self.own.retain(|id, _| seen.contains(id));
        // A pending open is settled once its key is no longer without a fill,
        // or after two reconciles' time.
        let still: Vec<String> = unconfirmed.iter().map(|(k, _)| k.clone()).collect();
        let r = rules(self.pool, account_id, self.env).await?;
        self.pending.retain(|x| {
            still.contains(&x.entry_key)
                && now - x.at < chrono::Duration::seconds(2 * r.reconcile_secs as i64)
        });
        self.d.version = Some(r.version);
        let detail = json!({
            "positions": self.own.values().map(|o| json!({
                "position_id": o.position_id,
                "symbol": self.syms.get(&o.symbol_id).map(|s| s.name.clone()),
                "side": o.side.as_str(),
                "volume": o.volume,
                "entry": o.entry, "stop": o.stop, "take_profit": o.take_profit,
                "tp1_sent": o.tp1_sent,
            })).collect::<Vec<_>>(),
            "program_positions": self.program_positions,
        });
        self.d
            .note(
                None,
                "reconcile",
                &format!(
                    "{} Strategy 3 position(s); {} position(s) of this program",
                    self.own.len(),
                    self.program_positions
                ),
                detail,
            )
            .await;
        Ok(())
    }

    /// An Unconfirmed open found filled: recorded once, with its fill.
    async fn adopt(
        &mut self,
        key: &str,
        p: &jiaoyi_ctrader::messages::ProtoOaPosition,
    ) -> anyhow::Result<()> {
        let account_id = self.account_id();
        ts::record_event(
            self.pool,
            &ts::EventRow {
                account_id,
                entry_key: Some(key.to_string()),
                client_msg_id: None,
                payload_type: 0,
                kind: "RECONCILED_FILL".into(),
                position_id: Some(p.position_id),
                order_id: None,
                deal_id: None,
                symbol_id: Some(p.trade_data.symbol_id),
                side: Side::from_proto(p.trade_data.trade_side).map(|s| s.as_str().to_string()),
                price: p.price,
                volume: Some(p.trade_data.volume),
                stop_loss: p.stop_loss,
                label: p.trade_data.label.clone(),
                error_code: None,
                description: Some(
                    "an unconfirmed Strategy 3 open found filled by reconcile".into(),
                ),
                is_server_event: None,
                raw: vec![],
                received_at: Utc::now(),
            },
        )
        .await?;
        let symbol = self
            .syms
            .get(&p.trade_data.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        s3s::record_fill(
            self.pool,
            &s3s::FillRow {
                account_id,
                entry_key: key.to_string(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol,
                position_id: Some(p.position_id),
                side: Side::from_proto(p.trade_data.trade_side).map(|s| s.as_str().to_string()),
                outcome: "reconciled_fill".into(),
                sent_at: Utc::now(),
                answered_at: None,
                latency_ms: None,
                decision_price: None,
                fill_price: p.price,
                slippage: None,
                spread: None,
                volume: Some(p.trade_data.volume),
                detail: json!({"note": "the open was Unconfirmed; reconcile found its position"}),
            },
        )
        .await?;
        Ok(())
    }

    /// Prepare a symbol's session: the M30 bars, the open and σ, and the
    /// median spread. Kept for the session; a broker error is retried after
    /// [`PREPARE_RETRY`].
    pub async fn prepare(&mut self, symbol_id: i64, s: Session, now: DateTime<Utc>) {
        let fresh = match self.syms.get(&symbol_id).and_then(|x| x.day.as_ref()) {
            Some(Ok(d)) => d.session.date != s.date,
            Some(Err((_, at))) => {
                now - *at >= chrono::Duration::from_std(PREPARE_RETRY).unwrap_or_default()
            }
            None => true,
        };
        if !fresh {
            return;
        }
        let account_id = self.account_id();
        let Some(rule) = self.syms.get(&symbol_id).and_then(|x| x.rule) else {
            return;
        };
        let from = s.open - chrono::Duration::days(BAR_DAYS);
        let day = match self.session.m30_bars(symbol_id, from, now).await {
            Err(e) => Err((format!("M30 bars unreadable: {e}"), now)),
            Ok(raw) => {
                let mut bars = Vec::new();
                let mut first_volume = None;
                for tb in &raw {
                    let Ok(b) = jiaoyi_ctrader::decode_bar(
                        tb,
                        jiaoyi_ctrader::messages::ProtoOaTrendbarPeriod::M30,
                    ) else {
                        continue;
                    };
                    let (Some(o), Some(c)) = (b.open.to_f64(), b.close.to_f64()) else {
                        continue;
                    };
                    if b.open_time == s.open {
                        first_volume = Some(b.tick_volume);
                    }
                    bars.push(M30 {
                        open_time: b.open_time,
                        open: o,
                        close: c,
                    });
                }
                let table = band::sigma_table(&rule, &s, &bars);
                let open = match (bars.iter().find(|b| b.open_time == s.open), first_volume) {
                    (Some(b), Some(v)) if v > 0 => Ok(b.open),
                    _ => Err(format!(
                        "no M30 bar with ticks at the session open {}: a holiday, skipped",
                        s.open
                    )),
                };
                let spread = match s3s::median_spread(self.pool, account_id, symbol_id).await {
                    Ok(Some(x)) => Ok(x),
                    Ok(None) => Err("no recorded two-sided quote in 7 days: no spread".into()),
                    Err(e) => Err(format!("recorded quotes unreadable: {e}")),
                };
                Ok(Day {
                    session: s,
                    open,
                    table,
                    spread,
                    prepared_at: now,
                })
            }
        };
        let name = self.syms[&symbol_id].name.clone();
        match &day {
            Ok(d) => {
                let detail = json!({
                    "session_date": d.session.date, "open_at": d.session.open, "close_at": d.session.close,
                    "open": d.open.as_ref().ok(), "open_refused": d.open.as_ref().err(),
                    "sigma_marks": d.table.marks.iter()
                        .map(|(m, v)| (m.to_string(), v.as_ref().map(|x| json!(x)).unwrap_or_else(|e| json!(e))))
                        .collect::<serde_json::Map<_, _>>(),
                    "sessions_used": d.table.sessions_used,
                    "median_spread": d.spread.as_ref().ok().map(|x| x.0),
                    "spread_quotes": d.spread.as_ref().ok().map(|x| x.1),
                    "rule": rule.describe(),
                });
                let reason = match &d.open {
                    Ok(o) => format!("session {} prepared: open {o}", d.session.date),
                    Err(why) => why.clone(),
                };
                self.d.note(Some(&name), "session", &reason, detail).await;
            }
            Err((why, _)) => {
                self.d.note(Some(&name), "session", why, json!({})).await;
            }
        }
        if let Some(x) = self.syms.get_mut(&symbol_id) {
            x.day = Some(day);
        }
    }

    fn open_risk(&self) -> Result<f64, String> {
        let mut total: f64 = self.pending.iter().map(|p| p.risk).sum();
        for o in self.own.values() {
            let (Some(entry), Some(stop)) = (o.entry, o.stop) else {
                return Err(format!(
                    "position {} has no known entry or stop: its open risk is unknown",
                    o.position_id
                ));
            };
            let Some(rate) = self.rate(o.symbol_id) else {
                return Err(format!(
                    "position {} has no quote→deposit rate: its open risk is unknown",
                    o.position_id
                ));
            };
            total += risk_at_stop(o.side, entry, stop, o.volume, rate);
        }
        Ok(total)
    }

    /// A quote: update the symbol, then TP1 for Strategy 3's position on it,
    /// or an entry when it has none.
    pub async fn on_quote(
        &mut self,
        symbol_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(sym) = self.syms.get_mut(&symbol_id) else {
            return Ok(());
        };
        if bid.is_some() {
            sym.bid = bid;
        }
        if ask.is_some() {
            sym.ask = ask;
        }
        let (bid, ask) = (sym.bid, sym.ask);
        let positions: Vec<i64> = self
            .own
            .values()
            .filter(|o| o.symbol_id == symbol_id)
            .map(|o| o.position_id)
            .collect();
        if !positions.is_empty() {
            for pid in positions {
                self.tp1(pid, bid, ask, now).await?;
            }
            return Ok(());
        }
        if self.pending.iter().any(|p| p.symbol_id == symbol_id) {
            return Ok(());
        }
        self.entry(symbol_id, now).await
    }

    async fn tp1(
        &mut self,
        position_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(o) = self.own.get(&position_id).cloned() else {
            return Ok(());
        };
        let (Some(fill), Some(dist), Some(close), true, false) = (
            o.entry,
            o.tp1_distance,
            o.close_volume,
            o.partial,
            o.tp1_sent,
        ) else {
            return Ok(());
        };
        let digits = self
            .syms
            .get(&o.symbol_id)
            .and_then(|s| s.limits)
            .map_or(5, |l| l.digits);
        if !tp1_reached(o.side, fill, dist, digits, bid, ask) || close >= o.volume {
            return Ok(());
        }
        if let Some(x) = self.own.get_mut(&position_id) {
            x.tp1_sent = true;
        }
        self.close(&o, close, "tp1", bid, ask, now).await
    }

    /// The session end: every Strategy 3 position whose session is over, or
    /// whose symbol is outside its session, closes.
    pub async fn session_end(&mut self, now: DateTime<Utc>) -> anyhow::Result<()> {
        let owns: Vec<Own> = self.own.values().cloned().collect();
        for o in owns {
            if o.end_sent {
                continue;
            }
            let Some(sym) = self.syms.get(&o.symbol_id) else {
                continue;
            };
            let current = sym.rule.and_then(|r| sessions::session_at(&r, now));
            let over = match (current, o.session_date) {
                (None, _) => true,
                (Some(s), Some(d)) => s.date != d,
                (Some(_), None) => false,
            };
            if !over {
                continue;
            }
            let (bid, ask) = (sym.bid, sym.ask);
            if let Some(x) = self.own.get_mut(&o.position_id) {
                x.end_sent = true;
            }
            self.close(&o, o.volume, "session_end", bid, ask, now)
                .await?;
        }
        Ok(())
    }

    async fn close(
        &mut self,
        o: &Own,
        volume: i64,
        kind: &str,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let account_id = self.account_id();
        let r = rules(self.pool, account_id, self.env).await?;
        let name = self
            .syms
            .get(&o.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let key = format!(
            "s3-{}:{account_id}:{}",
            if kind == "tp1" { "tp1" } else { "end" },
            o.position_id
        );
        let decision_price = match o.side {
            Side::Buy => bid,
            Side::Sell => ask,
        };
        let order = s3s::OrderRow {
            account_id,
            entry_key: key.clone(),
            label: LABEL.into(),
            prereg_hash: self.hash.clone(),
            kind: kind.into(),
            symbol: name.clone(),
            symbol_id: o.symbol_id,
            position_id: Some(o.position_id),
            side: Some(o.side.as_str().into()),
            session_date: o.session_date,
            decided_at: now,
            bid,
            ask,
            volume: Some(volume),
            detail: json!({"strategy": STRATEGY, "position_volume": o.volume,
                "tp1_distance": o.tp1_distance, "entry": o.entry, "open_entry_key": o.entry_key}),
        };
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(r.version),
            },
            order,
            recorded_at: Mutex::new(None),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: f64::INFINITY,
        };
        let cmd = Command::Close {
            position_id: o.position_id,
            volume,
        };
        let res = gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
        )
        .await;
        match res {
            Ok(sent) => {
                self.record_fill(
                    &ledger,
                    &key,
                    kind,
                    &name,
                    Some(o.position_id),
                    o.side,
                    decision_price,
                    bid,
                    ask,
                    volume,
                    &sent,
                )
                .await?;
                self.d
                    .note(
                        Some(&name),
                        kind,
                        &format!(
                            "close {volume} of position {}: {:?}",
                            o.position_id, sent.outcome
                        ),
                        json!({"entry_key": key}),
                    )
                    .await;
            }
            Err(SendError::Refused(x)) => {
                self.d
                    .note(
                        Some(&name),
                        &format!("{kind}_refused"),
                        &x.reason,
                        json!({"entry_key": key}),
                    )
                    .await;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_fill(
        &self,
        ledger: &S3Ledger<'_>,
        key: &str,
        kind: &str,
        symbol: &str,
        position_id: Option<i64>,
        side: Side,
        decision_price: Option<f64>,
        bid: Option<f64>,
        ask: Option<f64>,
        volume: i64,
        sent: &gate::Sent,
    ) -> anyhow::Result<()> {
        let sent_at = ledger.recorded_at.lock().unwrap().unwrap_or_else(Utc::now);
        let filled = sent.events.iter().rev().find(|e| e.kind == "ORDER_FILLED");
        let (outcome, price, pid, vol) = match &sent.outcome {
            Outcome::Filled {
                position_id: p,
                price,
                volume: v,
            } => ("filled", *price, p.or(position_id), v.or(Some(volume))),
            Outcome::Rejected { .. } => ("rejected", None, position_id, None),
            Outcome::Amended { .. } => ("amended", None, position_id, None),
            Outcome::Unconfirmed => ("unconfirmed", None, position_id, None),
        };
        let answered_at = filled.map(|e| e.received_at);
        // A buy pays the ask and a sell gets the bid: positive is adverse.
        // An exit of a long is a sell, of a short a buy.
        let trade_side = if kind == "open" {
            side
        } else {
            match side {
                Side::Buy => Side::Sell,
                Side::Sell => Side::Buy,
            }
        };
        let slippage = match (price, decision_price) {
            (Some(f), Some(q)) => Some(match trade_side {
                Side::Buy => f - q,
                Side::Sell => q - f,
            }),
            _ => None,
        };
        s3s::record_fill(
            self.pool,
            &s3s::FillRow {
                account_id: self.account_id(),
                entry_key: key.to_string(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: kind.into(),
                symbol: symbol.into(),
                position_id: pid,
                side: Some(side.as_str().into()),
                outcome: outcome.into(),
                sent_at,
                answered_at,
                latency_ms: answered_at.map(|a| (a - sent_at).num_milliseconds()),
                decision_price,
                fill_price: price,
                slippage,
                spread: match (bid, ask) {
                    (Some(b), Some(a)) => Some(a - b),
                    _ => None,
                },
                volume: vol,
                detail: json!({"outcome": sent.outcome, "client_msg_id": sent.client_msg_id}),
            },
        )
        .await?;
        Ok(())
    }

    async fn skip(
        &mut self,
        symbol_id: i64,
        name: &str,
        why: &str,
        detail: Value,
        now: DateTime<Utc>,
    ) {
        if let Some(s) = self.syms.get_mut(&symbol_id) {
            s.last_try = Some(now);
        }
        self.d.note(Some(name), "skip", why, detail).await;
    }

    /// An entry on a quote of a symbol with no Strategy 3 position.
    async fn entry(&mut self, symbol_id: i64, now: DateTime<Utc>) -> anyhow::Result<()> {
        let sym = self.syms[&symbol_id].clone();
        let name = sym.name.clone();
        let Some(rule) = sym.rule else {
            return Ok(());
        };
        let Some(s) = sessions::session_at(&rule, now) else {
            return Ok(());
        };
        if band::entry_window(&s, now).is_err() {
            return Ok(());
        }
        if sym
            .last_try
            .is_some_and(|t| now - t < chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default())
        {
            return Ok(());
        }
        self.prepare(symbol_id, s, now).await;
        let sym = self.syms[&symbol_id].clone();
        let day = match &sym.day {
            Some(Ok(d)) if d.session.date == s.date => d.clone(),
            _ => return Ok(()),
        };
        let Ok(open) = day.open.clone() else {
            return Ok(());
        };
        let (side, b) = match signal(&day.table, &s, open, now, sym.bid, sym.ask) {
            Ok(Some(x)) => x,
            Ok(None) => return Ok(()),
            Err(why) => {
                self.d.note(Some(&name), "no_band", &why, json!({})).await;
                return Ok(());
            }
        };
        let entry_price = match side {
            Dir::Buy => sym.ask,
            Dir::Sell => sym.bid,
        }
        .unwrap_or(f64::NAN);
        let mut detail = json!({
            "session_date": s.date, "decided_at": now, "bid": sym.bid, "ask": sym.ask,
            "side": side, "band": b, "open": open, "sigma": b.sigma,
        });
        let Some(limits) = sym.limits else {
            self.skip(
                symbol_id,
                &name,
                "the symbol's volume rules are not stored",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        let spread = match &day.spread {
            Ok((m, _)) => *m,
            Err(why) => {
                self.skip(symbol_id, &name, why, detail, now).await;
                return Ok(());
            }
        };
        let Some(rate) = self.rate(symbol_id) else {
            self.skip(
                symbol_id,
                &name,
                "no broker quote→deposit rate for this symbol yet",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        if self
            .commission_read
            .is_none_or(|t| t.elapsed() >= COMMISSION_REFRESH)
        {
            self.read_commission().await;
        }
        let commission_rt = match &self.commission {
            Ok(book) => {
                let (per_unit, label) = book
                    .per_class
                    .get(sym.class.as_str())
                    .cloned()
                    .unwrap_or((book.overall_median, "estimated: the overall median".into()));
                detail["commission_label"] = json!(label);
                2.0 * per_unit / rate
            }
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("commission not measured: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        let cost = match cost_reserve(
            spread,
            commission_rt,
            slippage_bps_per_side(sym.class),
            entry_price,
        ) {
            Ok(c) => c,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        let rel = relative_stop(b.half_width, limits.digits);
        let e = Entry {
            side,
            band: b,
            entry_price,
            relative_stop: rel,
            r: rel as f64 / 100_000.0,
            cost,
        };
        detail["entry"] = json!(e);
        detail["quote_to_deposit"] = json!(rate);
        let r = rules(self.pool, self.account_id(), self.env).await?;
        self.d.version = Some(r.version);
        let equity = match fresh_equity(self.session).await {
            Ok(x) => x,
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("fresh equity unreadable: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        let open_programs = self.program_positions + self.pending.len();
        let sized = match size(&r, equity, rel, rate, &limits, open_programs) {
            Ok(x) => x,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["sizing"] = json!(sized);
        let order = match plan_order(&e, &limits, sized.volume) {
            Ok(o) => o,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["order"] = json!(order);
        let budget = self
            .open_risk()
            .and_then(|held| within_budget(held, sized.risk, equity, r.open_risk_pct));
        if let Err(why) = budget {
            self.skip(symbol_id, &name, &why, detail, now).await;
            return Ok(());
        }
        let account_id = self.account_id();
        let n = s3s::opens_in_session(self.pool, account_id, symbol_id, s.date).await?;
        let key = format!("s3:{account_id}:{symbol_id}:{}:{}", s.date, n + 1);
        if let Some(x) = self.syms.get_mut(&symbol_id) {
            x.last_try = Some(now);
        }
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(r.version),
            },
            order: s3s::OrderRow {
                account_id,
                entry_key: key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol: name.clone(),
                symbol_id,
                position_id: None,
                side: Some(dir_side(side).as_str().into()),
                session_date: Some(s.date),
                decided_at: now,
                bid: sym.bid,
                ask: sym.ask,
                volume: Some(sized.volume),
                detail: detail.clone(),
            },
            recorded_at: Mutex::new(None),
        };
        let cmd = Command::Open {
            symbol_id,
            side: dir_side(side),
            volume: sized.volume,
            relative_stop: rel,
            relative_take_profit: Some(order.relative_take_profit),
            limits,
            quote_to_deposit: Some(rate),
            label: LABEL.into(),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: sized.max_risk,
        };
        match gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
        )
        .await
        {
            Ok(sent) => {
                self.record_fill(
                    &ledger,
                    &key,
                    "open",
                    &name,
                    None,
                    dir_side(side),
                    Some(entry_price),
                    sym.bid,
                    sym.ask,
                    sized.volume,
                    &sent,
                )
                .await?;
                match &sent.outcome {
                    Outcome::Filled {
                        position_id: Some(pid),
                        price,
                        volume,
                    } => {
                        let fill = price.unwrap_or(entry_price);
                        let sd = dir_side(side);
                        self.own.insert(
                            *pid,
                            Own {
                                position_id: *pid,
                                symbol_id,
                                side: sd,
                                volume: volume.unwrap_or(sized.volume),
                                entry: Some(fill),
                                stop: Some(match sd {
                                    Side::Buy => fill - e.r,
                                    Side::Sell => fill + e.r,
                                }),
                                take_profit: None,
                                tp1_distance: Some(order.tp1_distance),
                                close_volume: Some(order.plan.close_volume),
                                partial: order.plan.mode == targets::Mode::PartialRunner,
                                session_date: Some(s.date),
                                entry_key: Some(key.clone()),
                                tp1_sent: false,
                                end_sent: false,
                            },
                        );
                        self.program_positions += 1;
                    }
                    Outcome::Unconfirmed => self.pending.push(Pending {
                        entry_key: key.clone(),
                        symbol_id,
                        risk: sized.risk,
                        at: now,
                    }),
                    _ => {}
                }
                self.d
                    .note(
                        Some(&name),
                        "sent",
                        &format!(
                            "{STRATEGY} {} {} cents ({}): {:?}",
                            dir_side(side).as_str(),
                            sized.volume,
                            sized.mode,
                            sent.outcome
                        ),
                        json!({"entry_key": key, "relative_stop": rel,
                               "relative_take_profit": order.relative_take_profit}),
                    )
                    .await;
            }
            Err(SendError::Refused(x)) => {
                self.d.note(Some(&name), "refused", &x.reason, detail).await;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    /// A broker-side execution message on a Strategy 3 position (a stop or
    /// TP2 hit): recorded as it came.
    pub async fn server_event(&self, m: &ReceivedExecution) {
        if m.client_msg_id.is_some() {
            return;
        }
        if let Some(e) = gate::decode_execution(self.account_id(), m) {
            if e.label.as_deref() != Some(LABEL) {
                return;
            }
            if let Err(err) = ts::record_event(self.pool, &crate::trading::event_row(&e)).await {
                tracing::error!(error = %err, "strategy 3 server execution event not recorded");
            }
        }
    }
}

// ---------------------------------------------------------------- loop

/// Run every permitted account's Strategy 3 loop until shutdown.
pub async fn run(pool: PgPool, cfg: Arc<Config>, shutdown: Shutdown) {
    let mut tasks = Vec::new();
    for (account, env) in cfg.plan.permitted.clone() {
        let (pool, cfg, shutdown) = (pool.clone(), cfg.clone(), shutdown.clone());
        tasks.push(tokio::spawn(async move {
            account_loop(&pool, &cfg, &account, env, shutdown).await
        }));
    }
    for t in tasks {
        let _ = t.await;
    }
}

async fn account_loop(
    pool: &PgPool,
    cfg: &Config,
    account: &PermittedAccount,
    env: Environment,
    mut shutdown: Shutdown,
) {
    loop {
        if shutdown.is_set() {
            return;
        }
        let r = session(pool, cfg, account, env, shutdown.clone()).await;
        if shutdown.is_set() {
            return;
        }
        if let Err(e) = r {
            let mut d = Decisions::new(pool, account.id());
            d.note(
                None,
                "error",
                &format!("the Strategy 3 session stopped: {e:#}"),
                json!({}),
            )
            .await;
        }
        let wait = match rules(pool, account.id(), env).await {
            Ok(r) => r.retry_secs,
            Err(_) => 60,
        };
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(wait)) => {}
            _ = shutdown.wait() => return,
        }
    }
}

async fn session(
    pool: &PgPool,
    cfg: &Config,
    account: &PermittedAccount,
    env: Environment,
    mut shutdown: Shutdown,
) -> anyhow::Result<()> {
    let conn: Connection = loader::connect(&cfg.broker, env).await?;
    let session = loader::authorise(pool, &conn, &cfg.broker, account).await?;
    let mut ends = conn.session_ends();
    run_session(
        pool,
        &cfg.allowlist,
        env,
        &session,
        &mut ends,
        &mut shutdown,
    )
    .await
}

/// The Strategy 3 loop on an authorised session.
pub async fn run_session(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    ends: &mut tokio::sync::broadcast::Receiver<jiaoyi_ctrader::SessionEnd>,
    shutdown: &mut Shutdown,
) -> anyhow::Result<()> {
    let account_id = session.account().id();
    let conn = session.connection();
    let mut t = Trader::new(pool, allowlist, env, session).await?;
    if t.syms.is_empty() {
        anyhow::bail!("no resolved symbols stored for this account yet (a data load comes first)");
    }
    t.reconcile(Utc::now()).await?;
    let ids: Vec<i64> = t.syms.keys().copied().collect();
    let mut spots = conn.spot_events();
    let mut execs = conn.executions();
    session.subscribe_spots(&ids).await?;
    let r0 = rules(pool, account_id, env).await?;
    let mut reconcile_tick = tokio::time::interval(Duration::from_secs(r0.reconcile_secs));
    reconcile_tick.tick().await;
    let mut clock = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = shutdown.wait() => return Ok(()),
            e = ends.recv() => {
                if let Ok(e) = e {
                    if e.affects(account_id) {
                        anyhow::bail!("the broker ended the session: {}", e.reason);
                    }
                }
            }
            _ = reconcile_tick.tick() => t.reconcile(Utc::now()).await?,
            _ = clock.tick() => t.session_end(Utc::now()).await?,
            m = execs.recv() => match m {
                Ok(m) => t.server_event(&m).await,
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
            },
            s = spots.recv() => match s {
                Ok(spot) => {
                    let ev = spot.event;
                    if ev.ctid_trader_account_id != account_id {
                        continue;
                    }
                    t.on_quote(
                        ev.symbol_id,
                        ev.bid.map(|b| b as f64 / 100_000.0),
                        ev.ask.map(|a| a as f64 / 100_000.0),
                        spot.received_at,
                    )
                    .await?;
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => anyhow::bail!("the connection closed"),
            },
        }
    }
}

// ---------------------------------------------------------------- health

/// `/health` → `strategy3`: per account, the label, the pre-registration,
/// the open positions at the last reconcile and today's decisions and
/// trades. No money.
pub async fn health(pool: &PgPool) -> jiaoyi_store::Result<Value> {
    let accounts = jiaoyi_store::sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let mut out = Vec::new();
    for a in accounts {
        let id: i64 = a.get("account_id");
        let env: String = a.get("environment");
        let rec = ts::latest_decision_detail(pool, id, "s3_reconcile").await?;
        let today = s3s::today(pool, id).await?;
        out.push(json!({
            "account_id": id,
            "environment": env,
            "strategy": STRATEGY,
            "label": LABEL,
            "trades": "yes",
            "sizing": if env == "live" { "one broker-minimum trade at a time" } else { "risk_pct of fresh equity" },
            "prereg_file": PREREG_FILE,
            "prereg_hash": prereg_hash(),
            "open_positions": rec.as_ref().map(|(_, d)| d["positions"].clone()).unwrap_or(json!([])),
            "last_reconcile_at": rec.map(|(at, _)| at),
            "today": today,
        }));
    }
    Ok(json!(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules_demo() -> Rules {
        Rules {
            version: 1,
            live: false,
            risk_pct: 0.6,
            cap_factor: 1.1,
            min_volume_max_risk_pct: 10.0,
            open_risk_pct: 3.0,
            reconcile_secs: 300,
            order_wait_secs: 20,
            retry_secs: 60,
        }
    }

    const BTC: VolumeLimits = VolumeLimits {
        min: 1,
        step: 1,
        max: None,
        digits: 2,
    };

    #[test]
    fn the_embedded_pre_registration_is_the_file() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/preregistration-2026-10-25-strategy3.md"
        );
        let disk = std::fs::read(path).unwrap();
        let disk_hash: String = Sha256::digest(&disk)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(disk_hash, prereg_hash());
        assert!(PREREG_TEXT.contains("Tick-triggered only"));
    }

    #[test]
    fn demo_sizes_risk_pct_and_live_takes_one_minimum() {
        // $10,000 at 0.6% = $60; a 200.00 stop on 1 cent (0.01 BTC) risks $2.
        let s = size(&rules_demo(), 10_000.0, 20_000_000, 1.0, &BTC, 0).unwrap();
        assert_eq!((s.volume, s.mode), (30, "risk_pct"));
        assert!((s.risk - 60.0).abs() < 1e-9);
        // Below the minimum on demo: skipped.
        let e = size(&rules_demo(), 100.0, 20_000_000, 1.0, &BTC, 0).unwrap_err();
        assert!(e.contains("below the broker minimum"), "{e}");
        let live = Rules {
            live: true,
            ..rules_demo()
        };
        // Live: exactly the minimum (2.00 risk = 5.1% of 38.98 ≤ 10%).
        let s = size(&live, 38.98, 20_000_000, 1.0, &BTC, 0).unwrap();
        assert_eq!((s.volume, s.mode), (1, "live_broker_minimum"));
        assert!((s.max_risk - 3.898).abs() < 1e-9);
        // Only while no other position of this program is open.
        let e = size(&live, 38.98, 20_000_000, 1.0, &BTC, 1).unwrap_err();
        assert!(e.contains("one minimum trade at a time"), "{e}");
        // Above 10% of equity: skipped.
        let e = size(&live, 15.0, 20_000_000, 1.0, &BTC, 0).unwrap_err();
        assert!(e.contains("above 10% of equity"), "{e}");
    }

    #[test]
    fn the_budget_holds_strategy_3_within_its_share() {
        assert!(within_budget(200.0, 100.0, 10_000.0, 3.0).is_ok());
        let e = within_budget(250.0, 100.0, 10_000.0, 3.0).unwrap_err();
        assert!(e.contains("exceed its budget"), "{e}");
    }

    #[test]
    fn the_order_carries_tp2_and_tp1_as_distances_from_the_fill() {
        // bot-trade's numbers: entry 100, R 10, C 0.4, 10,000 cents.
        let e = Entry {
            side: Dir::Buy,
            band: Band {
                open: 95.0,
                sigma: 0.1,
                lower: 85.5,
                upper: 99.9,
                half_width: 10.0,
                minutes: 45.0,
            },
            entry_price: 100.0,
            relative_stop: 1_000_000,
            r: 10.0,
            cost: CostReserve {
                spread: 0.4,
                commission_round_trip: 0.0,
                slippage: 0.0,
                total: 0.4,
            },
        };
        let lim = VolumeLimits {
            min: 100,
            step: 100,
            max: None,
            digits: 2,
        };
        let o = plan_order(&e, &lim, 10_000).unwrap();
        assert_eq!(o.plan.close_volume, 2600);
        assert!((o.tp1_distance - 30.4).abs() < 1e-9);
        assert!((o.tp2_distance - 40.4).abs() < 1e-9);
        // The broker's take-profit is TP2, 40.40 from the fill.
        assert_eq!(o.relative_take_profit, 4_040_000);
        // Unsplittable: the broker's take-profit is TP1.
        let o = plan_order(&e, &lim, 100).unwrap();
        assert_eq!(o.plan.mode, targets::Mode::WholePositionMinimum);
        assert_eq!(o.relative_take_profit, 3_040_000);
        // TP1 distance not above C: refused with bot-trade's reason.
        let dear = Entry {
            cost: CostReserve {
                total: 1e9,
                ..e.cost
            },
            ..e.clone()
        };
        assert!(plan_order(&dear, &lim, 10_000).is_err());
        // TP1 is reached on the bid for a long, on the ask for a short.
        assert!(tp1_reached(Side::Buy, 100.0, 30.4, 2, Some(130.4), None));
        assert!(!tp1_reached(
            Side::Buy,
            100.0,
            30.4,
            2,
            Some(130.39),
            Some(131.0)
        ));
        assert!(tp1_reached(Side::Sell, 100.0, 30.4, 2, None, Some(69.6)));
        assert!(!tp1_reached(
            Side::Sell,
            100.0,
            30.4,
            2,
            Some(1.0),
            Some(69.61)
        ));
        // Float sums that miss the grid by an ulp still compare as ticks.
        assert!(tp1_reached(
            Side::Buy,
            60_462.0,
            1_651.21,
            2,
            Some(62_113.21),
            None
        ));
    }

    #[test]
    fn conversion_comes_from_the_brokers_own_pair() {
        let mids = HashMap::from([("USDJPY".to_string(), 150.0), ("EURUSD".to_string(), 1.1)]);
        assert_eq!(quote_to_deposit("USD", "USD", &mids), Some(1.0));
        assert!((quote_to_deposit("JPY", "USD", &mids).unwrap() - 1.0 / 150.0).abs() < 1e-12);
        assert!((quote_to_deposit("EUR", "USD", &mids).unwrap() - 1.1).abs() < 1e-12);
        assert_eq!(quote_to_deposit("GBP", "USD", &mids), None);
    }
}
