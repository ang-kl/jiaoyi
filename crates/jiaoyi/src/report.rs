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
    /// Broker symbol names that may be energy instruments (crude, brent, oil,
    /// wti, gas, xti, xbr; any case), from the latest load's symbol list.
    /// `None` until a load has recorded the list.
    pub energy_candidates: Option<Vec<serde_json::Value>>,
    pub energy_candidates_from_run: Option<i64>,
    /// The last read of the token's grant made for this account (PR #6):
    /// the accounts it listed, by fingerprint of the token, never the token.
    pub token_grant: Option<store::GrantRow>,
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
    let grants = store::latest_grants(pool).await?;
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
        let listed = store::latest_listed_symbols(pool, account_id).await?;
        let energy_candidates = listed.as_ref().map(|(_, _, list)| {
            list.as_array()
                .map(|a| {
                    a.iter()
                        .filter(|s| {
                            s.get("name")
                                .and_then(|n| n.as_str())
                                .is_some_and(jiaoyi_core::universe::is_energy_candidate)
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        });
        out.push(AccountReport {
            account_id,
            environment,
            energy_candidates,
            energy_candidates_from_run: listed.map(|(id, _, _)| id),
            token_grant: grants.iter().find(|g| g.account_id == account_id).cloned(),
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
                min_spread_samples: s.integer("spread.min_samples")?,
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
        (Some(id), Some(_)) => {
            store::spread_readings(pool, account_id, id, params.thresholds.min_spread_samples)
                .await?
        }
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
    // The verdict reads the latest measurement with enough samples; without
    // one, the latest attempt, which the verdict then calls "not measured".
    let judged = measured.as_ref().or(latest.as_ref());
    let spread_samples = judged.map(|r| r.spread_samples as i64);
    let judged_spread = judged
        .and_then(|r| r.spread_median)
        .and_then(|d| d.to_f64());
    let spread_missing_reason = match judged {
        None => Some("no spread measurement stored".to_string()),
        Some(r) => {
            let market = match (r.market_open, r.market_state_reason.as_deref()) {
                (Some(false), Some(why)) => Some(format!("market closed at the sample: {why}")),
                (Some(false), None) => Some("market closed at the sample".to_string()),
                (None, Some(why)) => Some(format!("market state unknown: {why}")),
                _ => None,
            };
            match (r.null_reason.clone(), market) {
                (Some(a), Some(b)) => Some(format!("{a}; {b}")),
                (a, b) => a.or(b),
            }
        }
    };
    let verdict = judge(
        class,
        &Evidence {
            unresolved_reason,
            tradable,
            not_tradable_reason,
            history_years: years,
            atr20: atr,
            spread_median: judged_spread,
            spread_samples,
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
    let accounts = jiaoyi_store::sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let grants = store::latest_grants(pool).await?;
    let trading = crate::trading::health(pool).await?;
    let strategy3 = crate::strategy3::health(pool).await?;
    // Claude · № 16,973 25-Oct (ordered № 16,972·C; the broker's own leverage, no money)
    let traders = store::history::latest_trader_reads(pool).await?;
    let broker_accounts: Vec<Value> = accounts
        .iter()
        .map(|a| {
            let id: i64 = a.get("account_id");
            broker_account(id, traders.iter().find(|t| t.account_id == id))
        })
        .collect();
    let token_grants: Vec<Value> = accounts
        .iter()
        .map(|a| {
            let id: i64 = a.get("account_id");
            let env: String = a.get("environment");
            grant_state(id, &env, grants.iter().find(|g| g.account_id == id))
        })
        .collect();
    Ok(serde_json::json!({
        "status": "ok",
        "service": "jiaoyi",
        "phase": 1,
        "load_in_progress": loading,
        "settings_version": version,
        "tick_recorders": recorders,
        "ticks": ticks,
        "account_refusals_recorded": refusals,
        "token_grants": token_grants,
        "broker_accounts": broker_accounts,
        "trading": trading,
        "strategy3": strategy3,
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

/// `1:N` from the broker's `leverageInCents` (1:50 is 5000); a fraction
/// stays a fraction (2550 is `1:25.5`).
pub fn leverage(leverage_in_cents: i64) -> String {
    if leverage_in_cents % 100 == 0 {
        format!("1:{}", leverage_in_cents / 100)
    } else {
        format!("1:{}", leverage_in_cents as f64 / 100.0)
    }
}

/// One account's latest trader read for `/health`: the broker's leverage
/// and account type, never its money. `max_leverage` is shown as the broker
/// gives it (Spotware does not document its unit).
pub fn broker_account(account_id: i64, t: Option<&store::history::LatestTraderRead>) -> Value {
    use jiaoyi_ctrader::messages::ProtoOaAccountType;
    let account_type = t.and_then(|t| t.account_type).map(|v| {
        ProtoOaAccountType::try_from(v)
            .map(|k| k.as_str_name().to_string())
            .unwrap_or_else(|_| format!("unknown ({v})"))
    });
    serde_json::json!({
        "account_id": account_id,
        "read_at": t.map(|t| t.read_at),
        "leverage": t.and_then(|t| t.leverage_in_cents).map(leverage),
        "leverage_in_cents": t.and_then(|t| t.leverage_in_cents),
        "max_leverage": t.and_then(|t| t.max_leverage),
        "account_type": account_type,
        "is_limited_risk": t.and_then(|t| t.is_limited_risk),
        "state": if t.is_some() { "read" } else { "unread" },
    })
}

/// One account's grant state for `/health`: `listed`, `not_listed`,
/// `environment_mismatch`, or `unread` (no read yet, or the last read was
/// refused), with the reason and the fingerprint of the token last read.
pub fn grant_state(account_id: i64, environment: &str, g: Option<&store::GrantRow>) -> Value {
    let (state, reason) = match g {
        None => ("unread", Some("no grant read recorded yet".to_string())),
        Some(g) if g.verdict == "rejected" => ("unread", g.reason.clone()),
        Some(g) => (g.verdict.as_str(), g.reason.clone()),
    };
    serde_json::json!({
        "account_id": account_id,
        "environment": environment,
        "state": state,
        "reason": reason,
        "token_fingerprint": g.map(|g| g.token_fingerprint.clone()),
        "token_source": g.map(|g| g.token_source.clone()),
        "host": g.map(|g| g.host.clone()),
        "read_at": g.map(|g| g.read_at),
    })
}

#[cfg(test)]
mod tests {
    use super::leverage;

    #[test]
    fn leverage_is_one_to_leverage_in_cents_over_100() {
        // ProtoOATrader.leverageInCents: "if leverage = 1:50 then value = 5000".
        assert_eq!(leverage(5000), "1:50");
        assert_eq!(leverage(20_000), "1:200");
        assert_eq!(leverage(2500), "1:25");
        assert_eq!(leverage(2550), "1:25.5");
    }
}
