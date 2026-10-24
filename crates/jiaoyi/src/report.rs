//! `/phase1/report`: per account and per universe name, what was loaded and
//! the rulebook §3 keep/drop verdict with its reasons.

use std::collections::BTreeMap;

use jiaoyi_core::indicators::{atr_wilder, Hlc};
use jiaoyi_core::verdict::{judge, Evidence, Thresholds};
use jiaoyi_store::sqlx::{PgPool, Row};
use jiaoyi_store::{self as store};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_json::Value;

use crate::loader::tradability;

/// Days per year used for "years of history".
pub const DAYS_PER_YEAR: f64 = 365.25;

#[derive(Debug, Serialize)]
pub struct Report {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// The settings version the verdicts were judged under (global values;
    /// each account's own snapshot version is the same number).
    pub settings_version: i64,
    pub thresholds: ThresholdsOut,
    pub rulebook: &'static str,
    pub notes: Vec<&'static str>,
    pub accounts: Vec<AccountReport>,
}

#[derive(Debug, Serialize)]
pub struct ThresholdsOut {
    pub atr_length: i64,
    pub min_history_years: f64,
    pub min_history_years_crypto: f64,
    pub max_spread_to_atr: f64,
}

#[derive(Debug, Serialize)]
pub struct AccountReport {
    pub account_id: i64,
    pub environment: String,
    pub last_load: Option<store::RunRow>,
    pub symbols: Vec<SymbolReport>,
}

#[derive(Debug, Serialize)]
pub struct SymbolReport {
    pub name: String,
    pub asset_class: &'static str,
    pub resolution: Option<store::ResolutionRow>,
    pub broker_symbol_id: Option<i64>,
    pub digits: Option<i32>,
    pub lot_size: Option<i64>,
    pub min_volume: Option<i64>,
    pub volume_step: Option<i64>,
    pub listed_enabled: Option<bool>,
    pub trading_mode: Option<String>,
    pub bars: usize,
    pub first_date: Option<chrono::NaiveDate>,
    pub last_date: Option<chrono::NaiveDate>,
    pub years_of_history: Option<f64>,
    /// How many bars open at each UTC time of day ("HH:MM"). A daily bar is
    /// dated by the UTC date of its open; this shows whether that is midnight.
    pub bar_open_times_utc: BTreeMap<String, usize>,
    /// ATR over `indicators.atr_length` daily bars (rulebook: 20).
    pub atr: Option<f64>,
    pub median_spread: Option<store::SpreadReading>,
    pub latest_spread_attempt: Option<store::SpreadReading>,
    pub spread_to_atr: Option<f64>,
    pub verdict: jiaoyi_core::verdict::Verdict,
}

pub async fn build(pool: &PgPool) -> store::Result<Report> {
    let accounts = jiaoyi_store::sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let runs = store::latest_runs(pool).await?;
    let global = store::settings::snapshot(pool, None).await?;
    let mut out = Vec::new();
    for a in accounts {
        let account_id: i64 = a.get("account_id");
        let environment: String = a.get("environment");
        let s = store::settings::snapshot(pool, Some(account_id)).await?;
        let params = Params::from(&s).map_err(store::StoreError::Settings)?;
        let resolutions = store::latest_resolutions(pool, account_id).await?;
        let mut symbols = Vec::new();
        for (name, class) in &params.universe {
            let resolution = resolutions.iter().find(|r| &r.name == name).cloned();
            symbols.push(symbol_report(pool, account_id, name, *class, resolution, &params).await?);
        }
        out.push(AccountReport {
            account_id,
            environment,
            last_load: runs.iter().find(|r| r.account_id == account_id).cloned(),
            symbols,
        });
    }
    let gp = Params::from(&global).map_err(store::StoreError::Settings)?;
    Ok(Report {
        generated_at: chrono::Utc::now(),
        settings_version: global.version,
        thresholds: ThresholdsOut {
            atr_length: gp.atr_length as i64,
            min_history_years: gp.thresholds.min_history_years,
            min_history_years_crypto: gp.thresholds.min_history_years_crypto,
            max_spread_to_atr: gp.thresholds.max_spread_to_atr,
        },
        rulebook: "docs/phase-0-rulebook.md §3",
        notes: vec![
            "ATR is Wilder's, over all stored closed bars, in price units; its length is the setting indicators.atr_length.",
            "Each account is judged under its own settings (global values plus its overrides).",
            "median_spread is the latest measurement that had quotes; latest_spread_attempt may be empty when the market was closed.",
            "A verdict is 'undecided' when no check failed but an input is unknown; it is never 'keep' on missing data.",
        ],
        accounts: out,
    })
}

/// The settings the report reads.
pub struct Params {
    pub universe: Vec<(String, jiaoyi_core::universe::AssetClass)>,
    pub atr_length: usize,
    pub thresholds: Thresholds,
}

impl Params {
    pub fn from(s: &jiaoyi_core::settings::Snapshot) -> Result<Params, String> {
        Ok(Params {
            universe: s.universe("universe.symbols")?,
            atr_length: s.integer("indicators.atr_length")?.max(1) as usize,
            thresholds: Thresholds {
                min_history_years: s.number("verdict.min_history_years")?,
                min_history_years_crypto: s.number("verdict.min_history_years_crypto")?,
                max_spread_to_atr: s.number("verdict.max_spread_to_atr")?,
            },
        })
    }
}

async fn symbol_report(
    pool: &PgPool,
    account_id: i64,
    name: &str,
    class: jiaoyi_core::universe::AssetClass,
    resolution: Option<store::ResolutionRow>,
    params: &Params,
) -> store::Result<SymbolReport> {
    let symbol_id = resolution.as_ref().and_then(|r| r.broker_symbol_id);
    let info = match symbol_id {
        Some(id) => store::symbol_info(pool, account_id, id).await?,
        None => None,
    };
    let bars = match (symbol_id, &info) {
        (Some(id), Some(_)) => store::bars(pool, account_id, id).await?,
        _ => vec![],
    };
    let (measured, latest) = match (symbol_id, &info) {
        (Some(id), Some(_)) => store::spread_readings(pool, account_id, id).await?,
        _ => (None, None),
    };

    let first = bars.first().map(|b| b.bar_date);
    let last = bars.last().map(|b| b.bar_date);
    let years = match (first, last) {
        (Some(f), Some(l)) => Some((l - f).num_days() as f64 / DAYS_PER_YEAR),
        _ => None,
    };
    let hlc: Vec<Hlc> = bars
        .iter()
        .filter_map(|b| {
            Some(Hlc {
                high: b.high.to_f64()?,
                low: b.low.to_f64()?,
                close: b.close.to_f64()?,
            })
        })
        .collect();
    let atr = if hlc.len() == bars.len() {
        atr_wilder(&hlc, params.atr_length)
    } else {
        None
    };
    let mut open_times = BTreeMap::new();
    for b in &bars {
        *open_times
            .entry(b.bar_open_utc.format("%H:%M").to_string())
            .or_insert(0) += 1;
    }
    let spread = measured
        .as_ref()
        .and_then(|m| m.spread_median)
        .and_then(|d| d.to_f64());
    let spread_to_atr = match (spread, atr) {
        (Some(s), Some(a)) if a > 0.0 => Some(s / a),
        _ => None,
    };

    let unresolved_reason = match &resolution {
        None => Some("not resolved yet: no load has recorded this name".to_string()),
        Some(r) if r.broker_symbol_id.is_none() => {
            Some(r.reason.clone().unwrap_or_else(|| r.status.clone()))
        }
        Some(_) if info.is_none() => Some("resolved but symbol details were not stored".into()),
        Some(_) => None,
    };
    let (tradable, not_tradable_reason) = match &info {
        Some(i) => tradability(i.listed_enabled, i.trading_mode.as_deref()),
        None => (None, None),
    };
    let spread_missing_reason = latest
        .as_ref()
        .and_then(|l| l.null_reason.clone())
        .or_else(|| Some("no spread measurement stored".into()));
    let verdict = judge(
        class,
        &Evidence {
            unresolved_reason,
            tradable,
            not_tradable_reason,
            history_years: years,
            atr20: atr,
            spread_median: spread,
            spread_missing_reason,
        },
        &params.thresholds,
    );

    Ok(SymbolReport {
        name: name.to_string(),
        asset_class: class.as_str(),
        broker_symbol_id: symbol_id,
        digits: info.as_ref().map(|i| i.digits),
        lot_size: info.as_ref().and_then(|i| i.lot_size),
        min_volume: info.as_ref().and_then(|i| i.min_volume),
        volume_step: info.as_ref().and_then(|i| i.volume_step),
        listed_enabled: info.as_ref().and_then(|i| i.listed_enabled),
        trading_mode: info.as_ref().and_then(|i| i.trading_mode.clone()),
        resolution,
        bars: bars.len(),
        first_date: first,
        last_date: last,
        years_of_history: years,
        bar_open_times_utc: open_times,
        atr,
        median_spread: measured,
        latest_spread_attempt: latest,
        spread_to_atr,
        verdict,
    })
}

/// The `/health` body.
pub async fn health(
    pool: &PgPool,
    loading: bool,
    recorders: std::collections::HashMap<i64, crate::recorder::RecorderStatus>,
) -> store::Result<Value> {
    let runs = store::latest_runs(pool).await?;
    let ticks = store::ticks::stats(pool).await?;
    let version: i64 = jiaoyi_store::sqlx::query_scalar("SELECT version FROM settings_version")
        .fetch_one(pool)
        .await?;
    let refusals: i64 = jiaoyi_store::sqlx::query_scalar("SELECT count(*) FROM account_refusals")
        .fetch_one(pool)
        .await?;
    Ok(serde_json::json!({
        "status": "ok",
        "service": "jiaoyi",
        "phase": 1,
        "load_in_progress": loading,
        "settings_version": version,
        "tick_recorders": recorders,
        "ticks": ticks,
        "account_refusals_recorded": refusals,
        "last_loads": runs.iter().map(|r| serde_json::json!({
            "account_id": r.account_id,
            "run_id": r.id,
            "status": r.status,
            "started_at": r.started_at,
            "finished_at": r.finished_at,
            "symbols_resolved": r.symbols_resolved,
            "symbols_unresolved": r.symbols_unresolved,
            "bars_inserted": r.bars_inserted,
            "bars_changed": r.bars_changed,
            "spreads_measured": r.spreads_measured,
            "settings_version": r.settings_version,
            "errors": r.errors,
        })).collect::<Vec<_>>(),
    }))
}
