//! One H1 job: prepare the symbols and their costs through the stage's data
//! layer, simulate at 0×, 1× and 2× slippage, and evaluate the 1× result
//! (the one PASS is judged on): the bootstrap, the control, buy-and-hold,
//! per-year results, the evidence against, notional money and the rules.

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate};
use jiaoyi_core::research::bootstrap::{week_block, Bootstrap};
use jiaoyi_core::research::control::{random_entries, Control};
use jiaoyi_core::research::h1::{simulate, H1Params, Indicators, Series, Side, Trade};
use jiaoyi_core::research::money::{notional, Money, MoneyTrade};
use jiaoyi_core::research::stats::{moments, r_stats, Moments, RStats};
use jiaoyi_core::research::verdict::{held_out_verdict, Rules, H1_MIN_TRADES};
use jiaoyi_core::seal;
use jiaoyi_core::universe::AssetClass;
use jiaoyi_store::research as rs;
use serde::Serialize;
use serde_json::{json, Value};

use super::costs::{self, BasisCounts, CommissionBook, Fx, SpreadBook, SwapTerms, SymbolCosts};
use super::data::{Data, Stage};

/// Fixed by the pre-registration.
pub const CONTROL_RUNS: usize = 1000;
pub const BOOTSTRAP_RESAMPLES: usize = 10_000;
pub const START_EQUITY: f64 = 10_000.0;
pub const RISK_PCT: f64 = 0.6;
/// The seed of the control and the bootstrap (not set by the
/// pre-registration; the H2 sampling seed is reused and logged).
pub const SEED: u64 = 20_261_024;
pub const SLIPPAGES: [f64; 3] = [0.0, 1.0, 2.0];

/// The readings taken where the pre-registration leaves a choice, recorded
/// in every trial's params.
pub fn readings() -> Value {
    json!({
        "held_out_lower_bound": "the 95% lower bound of the week-block bootstrap is the 2.5th percentile of the 10,000 resampled mean R, at the lower order statistic",
        "control_p95": "the 95th percentile of the 1,000 control run means, at the upper order statistic; the strategy must be strictly above it",
        "bootstrap_blocks": "blocks are ISO weeks of entry; only weeks with an entry are blocks",
        "fills": "a buy fills at the bar price (the level, or the open when the day opened beyond it) converted to the ask by the measured bar basis, plus the slippage; a sell at the bar price converted to the bid, minus the slippage; exits the same",
        "trigger": "the day's bar range reaching the level (bars in their measured basis); the fill is then taken on the ask or bid side as above",
        "initial_stop": "stop_atr × ATR(20) of the completed days before entry, from the fill",
        "trailing": "at the end of each day, stop_atr × ATR(20) of the completed days behind the best bar price since entry (highest high, lowest low for shorts, the entry day included); never loosened",
        "entry_day": "the initial stop is live from the fill: an entry day whose range reaches it is stopped that day (worst case)",
        "ambiguous_day": "the stop is checked before the day's new best price counts: a day reaching both is stopped (worst case)",
        "gap": "a day opening beyond the stop exits at the open",
        "same_day_reentry": "no new entry on the day a trade exits (one position per symbol)",
        "period_end": "a trade still open on the period's last day closes at its close, filled like any exit",
        "symbol_start": "a symbol's test starts at its 101st stored bar (100 completed bars) once it has at least one measured spread; years before its first measured June take the earliest measured median, labelled estimated",
        "ema_seed": "each EMA is seeded with the simple mean of its first len closes",
        "development_data": "bars that end (open + 24 h) by 2025-10-24T00:00Z; nothing on or after it is read",
        "held_out_entries": "on bars opening on or after 2025-10-24T00:00Z; earlier bars are warm-up only",
        "slippage": "per fill, in multiples of the year's median spread; PASS is judged at 1×",
        "commission": "per unit of volume per fill, the median over the class's deals of |commission| ÷ filled units, in USD, converted to the quote currency at that day's FX close",
        "quote_currency": "the name, in the broker's asset list (ProtoOAAssetListRes), of the symbol's quote asset in the broker's symbol list, both from one load run; a symbol whose quote asset cannot be read, or whose quote currency no FX pair in the universe converts, is refused",
        "deposit_currency": "the trader record's deposit asset must be named USD in the broker's asset list (otherwise the job fails); every closing deal on a symbol the broker quotes in it must convert at exactly 1, its rate selected in SQL only for those symbols, of any date",
        "quote_cross_check": "each tested symbol's closing deals executed before the stage's end (the seal for a development job, so no rate of a sealed-period deal is read; the held-out end for a held-out job) must convert quote → deposit within 1% of the FX close of the deal's day: the close of the latest FX daily bar opened at or before the execution, when it opened within the 4 days before it (a weekend and a holiday); a deal without one is not checked, never compared with an older close; a disagreement fails the job; a symbol without a checked deal is not cross-checked",
        "swap": "the broker's current swap terms on every night held (each bar end), 3 nights on the triple day, sign as the broker gives it (credits counted)",
        "money": "notional: $10,000 start, compounding, 0.6% risk; a trade below the minimum volume is skipped (money only)",
        "seeds": {"control": SEED, "bootstrap": SEED},
        "spread_samples": "the second full week of June of each year 2013-2025, streamed day by day; a week that failed 3 times is not asked again and its year takes an estimated spread; a symbol's weeks are sampled only after a load run has ended ok with the symbol resolved, so its daily bars (the basis observations) are stored before a sample is recorded",
        "bar_basis": "a symbol's basis is the one explaining at least 90% of its informative open/close comparisons with no other above 50%; otherwise the symbol is refused",
        "tick_recheck": "held-out only: entry days, the days the daily rules exit on and the period's last day are streamed; every individual quote update is checked, those sharing a millisecond included, the entry side's updates first within a millisecond (the true order of a bid and an ask update there is not known; this order lets a stop reached in the fill's millisecond count); the fill is the first ask at or above the level (bid at or below for shorts) plus slippage; from the fill the stop trails on every bid (ask for shorts) on the last completed day's ATR; a day whose ticks never reach the level makes no trade",
        "tick_period_end": "held-out only: a trade still open after the period's last day is streamed closes at that day's last bid (long) or ask (short), made worse by the slippage; when that day has no ticks, at its daily close as on daily bars",
        "held_out_rule_2": "rule 2 on held-out data holds only when both the daily-bar-executed mean (the same rules on daily bars, as the controls are executed) and the tick-counted mean are strictly above the 95th percentile of the daily-bar controls; the controls take the daily-bar execution's entries per symbol per year; both percentiles are reported, and the tick-minus-daily difference is evidence against; rules 1, 3 and 4 and the bootstrap are on the tick-counted trades",
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setting {
    pub breakout: usize,
    pub stop_atr: f64,
    pub account_id: i64,
}

impl Setting {
    pub fn from_json(v: &Value) -> Option<Setting> {
        Some(Setting {
            breakout: v.get("breakout")?.as_u64()? as usize,
            stop_atr: v.get("stop_atr")?.as_f64()?,
            account_id: v.get("account_id")?.as_i64()?,
        })
    }

    pub fn params(&self, slip_mult: f64) -> H1Params {
        H1Params {
            breakout: self.breakout,
            stop_atr: self.stop_atr,
            slip_mult,
        }
    }
}

/// One symbol ready to simulate.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub costs: SymbolCosts,
    pub series: Series,
    pub indicators: Indicators,
    /// USD per one price unit per unit of volume, per day.
    pub usd_per_price: Vec<f64>,
    /// Each bar's open, for the tick re-check.
    pub opens: Vec<chrono::DateTime<chrono::Utc>>,
}

/// The universe as the job saw it.
#[derive(Debug, Clone)]
pub struct Universe {
    pub symbols: Vec<Prepared>,
    /// Symbols left out, with the reason.
    pub refused: Vec<(String, String)>,
    pub settings_version: i64,
    pub commission: CommissionBook,
    pub deposit: Value,
}

/// Whether the account's deposit currency is established as USD.
#[derive(Debug, Clone, PartialEq)]
pub enum Deposit {
    /// Established; the evidence is recorded in the trial.
    Usd(Value),
    /// An input does not exist yet (no trader record, or no load has kept
    /// the asset ids): the job waits, it is not attempted.
    NotReady(String),
    /// The inputs disagree with USD: the job fails, nothing is guessed.
    Failed(String),
}

/// The broker-stated quote currency of each listed symbol (owner,
/// 24-10-2026): the name, in the broker's asset list (ProtoOAAssetListRes),
/// of the symbol's quote asset, both read by one load run. Never inferred
/// from a symbol's name; a symbol whose quote asset cannot be read is
/// refused with the reason.
#[derive(Debug, Clone, Default)]
pub struct QuoteAssets {
    pub run: i64,
    names: HashMap<i64, String>,
    /// Broker symbol id → its quote asset id, when the list gave one.
    quote_ids: HashMap<i64, Option<i64>>,
}

impl QuoteAssets {
    pub fn from_run(run: i64, assets: &Value, listed: &Value) -> QuoteAssets {
        let names = assets
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| {
                Some((
                    a.get("asset_id")?.as_i64()?,
                    a.get("name")?.as_str()?.to_string(),
                ))
            })
            .collect();
        let quote_ids = listed
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| {
                Some((
                    s.get("id")?.as_i64()?,
                    s.get("quote_asset_id").and_then(|q| q.as_i64()),
                ))
            })
            .collect();
        QuoteAssets {
            run,
            names,
            quote_ids,
        }
    }

    pub fn asset_name(&self, asset_id: i64) -> Option<&str> {
        self.names.get(&asset_id).map(String::as_str)
    }

    /// The quote currency of a broker symbol, or why it cannot be read.
    pub fn quote_of(&self, symbol_id: i64) -> Result<(String, i64), String> {
        let run = self.run;
        let Some(quote_id) = self.quote_ids.get(&symbol_id) else {
            return Err(format!(
                "quote asset not read from the broker: symbol {symbol_id} is not in load run {run}'s symbol list"
            ));
        };
        let Some(quote_id) = *quote_id else {
            return Err(format!(
                "quote asset not read from the broker: load run {run}'s symbol list gives symbol {symbol_id} no quote asset id"
            ));
        };
        match self.asset_name(quote_id) {
            Some(n) => Ok((n.to_string(), quote_id)),
            None => Err(format!(
                "quote asset not read from the broker: asset {quote_id} is not in load run {run}'s asset list"
            )),
        }
    }
}

/// The deposit currency (owner, 24-10-2026): the deposit asset of the
/// broker's trader record must be named USD in the broker's asset list;
/// any other deposit fails the job. As a cross-check, every closing deal on
/// a symbol whose broker-stated quote asset is that deposit asset must
/// convert quote → deposit at exactly 1. The query selects only those deals'
/// rates (the filter is in SQL), so no rate of a symbol quoted in another
/// currency is read here, of any date.
pub async fn deposit_currency(
    pool: &jiaoyi_store::sqlx::PgPool,
    account_id: i64,
) -> anyhow::Result<Deposit> {
    let Some(deposit) = rs::deposit_asset_id(pool, account_id).await? else {
        return Ok(Deposit::NotReady(
            "no trader record (ProtoOATraderRes) stored yet".into(),
        ));
    };
    let Some((run, assets, listed)) = jiaoyi_store::latest_assets(pool, account_id).await? else {
        return Ok(Deposit::NotReady(
            "no load has stored the broker's asset list (ProtoOAAssetListRes) yet".into(),
        ));
    };
    let quotes = QuoteAssets::from_run(run, &assets, &listed);
    let name = match quotes.asset_name(deposit) {
        Some(n) => n.to_string(),
        None => {
            return Ok(Deposit::Failed(format!(
                "the deposit asset {deposit} is not in load run {run}'s asset list: the deposit currency cannot be read"
            )))
        }
    };
    if name != "USD" {
        return Ok(Deposit::Failed(format!(
            "the deposit currency is {name} (asset {deposit}), not USD: money cannot be converted"
        )));
    }
    let rates = rs::closing_rates_quoted_in(pool, account_id, run, deposit).await?;
    if rates.is_empty() || rates.iter().any(|r| (r - 1.0).abs() > 1e-9) {
        return Ok(Deposit::Failed(format!(
            "the deal cross-check fails: {} closing deals on symbols quoted in the deposit asset, quote→deposit rates other than 1: {:?}",
            rates.len(),
            rates.iter().filter(|r| (*r - 1.0).abs() > 1e-9).take(5).collect::<Vec<_>>()
        )));
    }
    Ok(Deposit::Usd(json!({
        "currency": "USD",
        "deposit_asset_id": deposit,
        "asset_list_from_load_run": run,
        "cross_check": format!("{} closing deals on symbols the broker quotes in USD, all at quote→deposit rate 1", rates.len()),
    })))
}

/// Read every input through the stage's data layer. A symbol whose costs or
/// bars are missing is refused with the reason; a missing account-level
/// input (commission) fails the job. `deposit`: the established evidence.
pub async fn prepare(data: &Data<'_>, account_id: i64, deposit: Value) -> anyhow::Result<Universe> {
    let pool = data.pool();
    let snap = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let universe = snap
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    let commission = costs::commission_book(&rs::commission_per_unit(pool, account_id).await?)
        .map_err(anyhow::Error::msg)?;
    let samples = rs::spread_samples(pool, account_id).await?;
    let end = data.stage().end();
    // The quote currencies, from the load run that established the deposit.
    let run = deposit
        .get("asset_list_from_load_run")
        .and_then(|r| r.as_i64())
        .ok_or_else(|| anyhow::anyhow!("the deposit evidence names no asset list"))?;
    let (assets, listed) = jiaoyi_store::assets_of_run(pool, account_id, run)
        .await?
        .ok_or_else(|| anyhow::anyhow!("load run {run}'s asset list is not stored"))?;
    let quotes = QuoteAssets::from_run(run, &assets, &listed);
    // FX closes for converting money.
    let mut fx = Fx::default();
    for pair in ["EURUSD", "GBPUSD", "AUDUSD", "NZDUSD", "USDJPY", "USDCAD"] {
        if let [id] = jiaoyi_store::symbol_ids_by_name(pool, account_id, pair)
            .await?
            .as_slice()
        {
            fx.add_bars(pair, &data.daily_bars(account_id, *id, end).await?);
        }
    }
    let mut symbols = Vec::new();
    let mut refused = Vec::new();
    for (name, class) in &universe {
        match prepare_symbol(
            data,
            account_id,
            name,
            *class,
            &samples,
            &commission,
            &fx,
            &quotes,
            end,
        )
        .await
        {
            Ok(p) => symbols.push(p),
            Err(why) => refused.push((name.clone(), why)),
        }
    }
    // The deal cross-check of each tested symbol's quote currency, over the
    // deals the stage may read; a disagreement fails the job.
    for p in &mut symbols {
        let deals = data
            .closing_rates(account_id, p.costs.symbol_id, end)
            .await?;
        p.costs.quote_cross_check = costs::cross_check_quote(&p.costs.quote, &fx, &deals, end)
            .map_err(|why| anyhow::anyhow!("{}: {why}", p.costs.name))?;
    }
    Ok(Universe {
        symbols,
        refused,
        settings_version: snap.version,
        commission,
        deposit,
    })
}

#[allow(clippy::too_many_arguments)]
async fn prepare_symbol(
    data: &Data<'_>,
    account_id: i64,
    name: &str,
    class: AssetClass,
    samples: &[rs::SpreadSample],
    commission: &CommissionBook,
    fx: &Fx,
    quotes: &QuoteAssets,
    end: chrono::DateTime<chrono::Utc>,
) -> Result<Prepared, String> {
    let pool = data.pool();
    let id = match jiaoyi_store::symbol_ids_by_name(pool, account_id, name)
        .await
        .map_err(|e| e.to_string())?
        .as_slice()
    {
        [id] => *id,
        [] => return Err("not stored on the account".into()),
        many => return Err(format!("stored under several broker ids: {many:?}")),
    };
    let (quote, quote_asset) = quotes.quote_of(id)?;
    if quote != "USD" {
        let Some((pair, _)) = costs::fx_pair(&quote) else {
            return Err(format!(
                "quoted in {quote}: no FX pair in the universe converts it to USD"
            ));
        };
        if !fx.closes.contains_key(pair) {
            return Err(format!(
                "quoted in {quote}: its FX pair {pair} has no stored daily bars on the account"
            ));
        }
    }
    let quote_label = format!(
        "the broker's asset list (load run {}): quote asset {quote_asset}",
        quotes.run
    );
    let mine: Vec<&rs::SpreadSample> = samples
        .iter()
        .filter(|s| s.broker_symbol_id == id)
        .collect();
    let spreads = SpreadBook::from_samples(&mine);
    if spreads.measured.is_empty() {
        return Err("no spread measure: no June week with ticks was sampled".into());
    }
    let mut counts = BasisCounts::default();
    for s in &mine {
        if let Some(b) = s
            .basis
            .as_ref()
            .and_then(|b| serde_json::from_value::<BasisCounts>(b.clone()).ok())
        {
            counts.add(&b);
        }
    }
    let basis = costs::decide_basis(&counts)?;
    let terms = rs::symbol_terms(pool, account_id, id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("no symbol details stored")?;
    let swap = SwapTerms::from_terms(&terms)?;
    let (c, label) = commission.per_class[class.as_str()].clone();
    let costs = SymbolCosts {
        name: name.to_string(),
        symbol_id: id,
        class,
        basis,
        basis_counts: counts,
        spreads,
        commission_usd_per_unit: c,
        commission_label: label,
        swap,
        quote,
        quote_label,
        quote_cross_check: String::new(),
        min_units: terms.min_volume.map(|v| v as f64 / 100.0),
        step_units: terms.volume_step.map(|v| v as f64 / 100.0),
    };
    let bars = data
        .daily_bars(account_id, id, end)
        .await
        .map_err(|e| e.to_string())?;
    let first_entry = match data.stage() {
        Stage::Development => 0,
        Stage::HeldOut => bars
            .iter()
            .position(|b| b.bar_open_utc >= seal::seal_instant())
            .ok_or("no bar in the held-out period")?,
    };
    let opens = bars.iter().map(|b| b.bar_open_utc).collect();
    let (series, usd_per_price) = costs::series(&bars, &costs, fx, first_entry)?;
    let indicators = Indicators::of(&series);
    Ok(Prepared {
        costs,
        series,
        indicators,
        usd_per_price,
        opens,
    })
}

/// Every trade of the universe at one slippage multiple.
pub fn simulate_all(u: &Universe, s: &Setting, slip: f64) -> Vec<Vec<Trade>> {
    let p = s.params(slip);
    u.symbols
        .iter()
        .map(|x| simulate(&x.series, &x.indicators, &p))
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct Evaluation {
    pub r: RStats,
    pub moments: Option<Moments>,
    pub sharpe_per_trade: Option<f64>,
    pub bootstrap: Option<Bootstrap>,
    pub control_runs: usize,
    pub control_seed: u64,
    pub control_p95: Option<f64>,
    pub control_percentile: Option<f64>,
    pub control_shortfall: u64,
    pub rules: Rules,
    pub verdict: String,
}

fn iso_week(d: NaiveDate) -> (i32, u32) {
    let w = d.iso_week();
    (w.year(), w.week())
}

/// Evaluate the counted trades (1× slippage; held-out: tick re-checked).
/// `per_symbol[i]` are the trades of `u.symbols[i]`.
pub fn evaluate(
    u: &Universe,
    s: &Setting,
    per_symbol: &[Vec<Trade>],
    stage: Stage,
    daily: Option<&[Vec<Trade>]>,
) -> (Value, Evaluation) {
    let all: Vec<&Trade> = per_symbol.iter().flatten().collect();
    let rs: Vec<f64> = all.iter().map(|t| t.net_r).collect();
    let stats = r_stats(&rs);
    let mom = moments(&rs);
    // Notional money, in the trades' order.
    let mut money_in = Vec::new();
    let mut money_idx = Vec::new();
    for (i, trades) in per_symbol.iter().enumerate() {
        let x = &u.symbols[i];
        for (k, t) in trades.iter().enumerate() {
            if let (Some(min), Some(step)) = (x.costs.min_units, x.costs.step_units) {
                money_in.push(MoneyTrade {
                    entry_date: t.entry_date,
                    exit_date: t.exit_date,
                    stop_distance: t.stop_distance,
                    net_per_unit: t.net_r * t.stop_distance,
                    usd_per_price_entry: x.usd_per_price[t.entry_index],
                    usd_per_price_exit: x.usd_per_price[t.exit_index],
                    min_units: min,
                    step_units: step,
                });
                money_idx.push((i, k));
            }
        }
    }
    let money: Money = notional(&money_in, START_EQUITY, RISK_PCT);
    let mut money_of: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for (j, m) in money.per_trade.iter().enumerate() {
        if let Some(m) = m {
            money_of.insert(money_idx[j], *m);
        }
    }
    let boot_in: Vec<((i32, u32), f64, Option<f64>)> = per_symbol
        .iter()
        .enumerate()
        .flat_map(|(i, trades)| {
            let money_of = &money_of;
            trades.iter().enumerate().map(move |(k, t)| {
                (
                    iso_week(t.entry_date),
                    t.net_r,
                    Some(*money_of.get(&(i, k)).unwrap_or(&0.0)),
                )
            })
        })
        .collect();
    let bootstrap = week_block(&boot_in, BOOTSTRAP_RESAMPLES, SEED);
    // Control: same symbols, same entries per symbol per year, executed on
    // daily bars; on held-out data the counts are the daily-bar execution's
    // (Codex P2 on #9: the control is compared with the strategy executed
    // the same way).
    let counts: Vec<BTreeMap<i32, usize>> = daily
        .unwrap_or(per_symbol)
        .iter()
        .map(|trades| {
            let mut c = BTreeMap::new();
            for t in trades {
                *c.entry(t.entry_year()).or_default() += 1;
            }
            c
        })
        .collect();
    let syms: Vec<(&Series, &Indicators, BTreeMap<i32, usize>)> = u
        .symbols
        .iter()
        .zip(counts)
        .map(|(x, c)| (&x.series, &x.indicators, c))
        .collect();
    let control: Control = random_entries(&syms, &s.params(1.0), CONTROL_RUNS, SEED);
    let daily_stats =
        daily.map(|d| r_stats(&d.iter().flatten().map(|t| t.net_r).collect::<Vec<_>>()));
    let mut rules = Rules::evaluate(
        stats.n,
        stats.mean_r,
        bootstrap.as_ref().map(|b| b.mean_r_lower_2_5),
        control.p95,
        stats.pf(),
        H1_MIN_TRADES,
    );
    // Held-out: rule 2 needs the daily-bar mean above the same p95 too.
    if let Some(ds) = &daily_stats {
        rules = rules.also_above_control_p95(ds.mean_r, control.p95);
    }
    let verdict = match stage {
        Stage::HeldOut => held_out_verdict(&rules).to_string(),
        Stage::Development if rules.rules_1_to_3() => "development: PASS rules 1-3 met".into(),
        Stage::Development => "development: PASS rules 1-3 not met".into(),
    };
    // Buy-and-hold over the trades' days in the market, same costs, in R.
    let p1 = s.params(1.0);
    let bh: Vec<f64> = per_symbol
        .iter()
        .enumerate()
        .flat_map(|(i, trades)| {
            let x = &u.symbols[i].series;
            trades.iter().map(move |t| {
                let slip = |k: usize| p1.slip_mult * x.spread[k];
                let fill = x.open[t.entry_index] + x.to_ask[t.entry_index] + slip(t.entry_index);
                let exit = x.close[t.exit_index] - x.to_bid[t.exit_index] - slip(t.exit_index);
                let swap: f64 = x.swap_long[t.entry_index..t.exit_index].iter().sum();
                (exit - fill - x.commission[t.entry_index] - x.commission[t.exit_index] + swap)
                    / t.stop_distance
            })
        })
        .collect();
    // Per year and per symbol.
    let mut years: BTreeMap<i32, Vec<f64>> = BTreeMap::new();
    for t in &all {
        years.entry(t.entry_year()).or_default().push(t.net_r);
    }
    let per_year: BTreeMap<i32, RStats> = years.iter().map(|(y, v)| (*y, r_stats(v))).collect();
    let worst_year = per_year
        .iter()
        .min_by(|a, b| a.1.total_r.total_cmp(&b.1.total_r))
        .map(|(y, st)| json!({"year": y, "stats": st}));
    let mut sorted = rs.clone();
    sorted.sort_by(|a, b| b.total_cmp(a));
    let without_top5 = r_stats(sorted.get(5..).unwrap_or(&[]));
    let per_symbol_stats: BTreeMap<String, RStats> = per_symbol
        .iter()
        .enumerate()
        .map(|(i, t)| {
            (
                u.symbols[i].costs.name.clone(),
                r_stats(&t.iter().map(|t| t.net_r).collect::<Vec<_>>()),
            )
        })
        .collect();
    let first = all.iter().map(|t| t.entry_date).min();
    let last = all.iter().map(|t| t.exit_date).max();
    let eval = Evaluation {
        sharpe_per_trade: mom.map(|m| if m.sd > 0.0 { m.mean / m.sd } else { 0.0 }),
        moments: mom,
        r: stats.clone(),
        bootstrap: bootstrap.clone(),
        control_runs: control.runs,
        control_seed: control.seed,
        control_p95: control.p95,
        control_percentile: stats.mean_r.and_then(|m| control.percentile_of(m)),
        control_shortfall: control.shortfall,
        rules,
        verdict: verdict.clone(),
    };
    let results = json!({
        "unit": "R = price move ÷ initial stop distance, net of spread, slippage, commission and swap",
        "counted": "1× slippage",
        "trades": stats.n,
        "period": {"first_entry": first, "last_exit": last},
        "symbols": u.symbols.iter().map(|x| x.costs.name.clone()).collect::<Vec<_>>(),
        "r": stats,
        "moments": mom,
        "sharpe_per_trade": eval.sharpe_per_trade,
        "bootstrap": bootstrap,
        "control": {
            "runs": control.runs, "seed": control.seed, "entries_per_run": control.entries_per_run,
            "shortfall": control.shortfall, "p95": control.p95,
            "executed_on": "daily bars",
            "strategy_percentile": eval.control_percentile,
            "daily_bar_mean_r": daily_stats.as_ref().and_then(|d| d.mean_r),
            "daily_bar_percentile": daily_stats.as_ref().and_then(|d| d.mean_r).and_then(|m| control.percentile_of(m)),
        },
        "buy_and_hold": {"definition": "long from each trade's entry-day open to its exit-day close, same costs, in the trade's R", "r": r_stats(&bh)},
        "per_year": per_year,
        "per_symbol": per_symbol_stats,
        "evidence_against": {
            "without_top_5_trades": without_top5,
            "worst_year": worst_year,
            "tick_vs_daily_bar": daily_stats.as_ref().map(|d| json!({
                "tick": stats,
                "daily_bar": d,
                "mean_r_difference": stats.mean_r.zip(d.mean_r).map(|(t, d)| t - d),
                "note": "tick-counted minus daily-bar-executed mean net R; the controls are executed on daily bars",
            })),
        },
        "spread_estimated_trades": all.iter().filter(|t| t.spread_estimated).count(),
        "long_trades": all.iter().filter(|t| t.side == Side::Long).count(),
        "short_trades": all.iter().filter(|t| t.side == Side::Short).count(),
        "rules": rules,
        "verdict": verdict,
        "notional": money,
        "trade_list": all.iter().map(|t| json!({
            "symbol": t.symbol, "side": t.side, "entry": t.entry_date, "exit": t.exit_date,
            "level": t.level, "entry_fill": t.entry_fill, "stop_distance": t.stop_distance,
            "exit_fill": t.exit_fill, "exit_reason": t.exit_reason, "nights": t.nights,
            "gross_r": t.gross_r, "commission_r": t.commission_r, "swap_r": t.swap_r, "net_r": t.net_r,
            "spread_estimated": t.spread_estimated,
        })).collect::<Vec<_>>(),
    });
    (results, eval)
}

/// The params and data range a trial records about its inputs.
pub fn inputs(u: &Universe, s: &Setting, stage: Stage, prereg_hash: &str) -> (Value, Value) {
    let params = json!({
        "hypothesis": "H1",
        "prereg_hash": prereg_hash,
        "stage": stage.as_str(),
        "setting": {"breakout": s.breakout, "stop_atr": s.stop_atr, "default": s.breakout == 50 && s.stop_atr == 3.0},
        "account_id": s.account_id,
        "deposit_currency": u.deposit,
        "readings": readings(),
        "basis_rule": {"min_share": costs::BASIS_MIN_SHARE, "max_other": costs::BASIS_MAX_OTHER},
        "percent_swap_days": costs::PERCENT_SWAP_DAYS,
        "commission": u.commission,
        "symbols": u.symbols.iter().map(|x| json!({
            "name": x.costs.name, "class": x.costs.class, "broker_symbol_id": x.costs.symbol_id,
            "basis": x.costs.basis, "basis_counts": x.costs.basis_counts,
            "spreads_measured": x.costs.spreads.measured,
            "commission_usd_per_unit": x.costs.commission_usd_per_unit, "commission_label": x.costs.commission_label,
            "swap": x.costs.swap, "swap_label": "estimated: the broker's current terms applied to every year",
            "quote": x.costs.quote, "quote_label": x.costs.quote_label,
            "quote_cross_check": x.costs.quote_cross_check,
            "min_units": x.costs.min_units, "step_units": x.costs.step_units,
        })).collect::<Vec<_>>(),
        "refused_symbols": u.refused.iter().map(|(n, r)| json!({"name": n, "reason": r})).collect::<Vec<_>>(),
    });
    let range = json!({
        "stage": stage.as_str(),
        "reads_until": stage.end(),
        "held_out": {"from": seal::held_out_from(), "to": seal::held_out_to()},
        "symbols": u.symbols.iter().map(|x| json!({
            "name": x.costs.name,
            "first_bar": x.series.dates.first(),
            "last_bar": x.series.dates.last(),
            "first_entry_bar": x.series.dates.get(x.series.first_entry.max(jiaoyi_core::research::h1::MIN_BARS)),
        })).collect::<Vec<_>>(),
    });
    (params, range)
}

/// Results for the three slippage multiples (n, mean R, PF) beside the
/// counted 1× evaluation.
pub fn slippage_table(u: &Universe, s: &Setting) -> Value {
    let mut out = serde_json::Map::new();
    for m in SLIPPAGES {
        let rs: Vec<f64> = simulate_all(u, s, m)
            .into_iter()
            .flatten()
            .map(|t| t.net_r)
            .collect();
        out.insert(format!("{m}x"), json!(r_stats(&rs)));
    }
    Value::Object(out)
}
