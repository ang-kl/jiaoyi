//! `/research/jobs` and `/research/trials`: read-only JSON, counts and R
//! only. Notional money and the trade list stay in the stored trial; these
//! unauthenticated routes do not show them.

use jiaoyi_core::research::dsr::{deflated_sharpe, expected_max_sharpe};
use jiaoyi_core::research::stats::Moments;
use jiaoyi_store::research as rs;
use jiaoyi_store::sqlx::PgPool;
use serde_json::{json, Value};

use super::prereg;

/// The number of trials the Sharpe ratio is deflated for: the 3×3 grid.
pub const DSR_TRIALS: usize = 9;

pub async fn jobs(pool: &PgPool) -> jiaoyi_store::Result<Value> {
    let mut out = Vec::new();
    for j in rs::jobs(pool).await? {
        let attempts = rs::job_attempts(pool, j.id).await?;
        out.push(json!({
            "id": j.id,
            "hypothesis": j.hypothesis,
            "stage": j.stage,
            "setting": j.setting,
            "run_order": j.run_order,
            "status": j.status,
            "reason": j.reason,
            "prereg_hash": j.prereg_hash,
            "created_at": j.created_at,
            "started_at": j.started_at,
            "finished_at": j.finished_at,
            "attempts": attempts,
        }));
    }
    Ok(json!({
        "prereg_file": prereg::PREREG_FILE,
        "prereg_hash_running": prereg::hash(),
        "engine_version_running": super::ENGINE_VERSION,
        "held_out_min_engine": super::HELD_OUT_MIN_ENGINE,
        "spread_sampling": rs::sampling_summary(pool).await?,
        "jobs": out,
    }))
}

fn sr_of(results: &Value) -> Option<(f64, Moments)> {
    let m: Moments = serde_json::from_value(results.get("moments")?.clone()).ok()?;
    Some((results.get("sharpe_per_trade")?.as_f64()?, m))
}

/// Keep counts and R; drop money and the trade list.
fn public(results: &Value) -> Value {
    let mut r = serde_json::Map::new();
    for k in [
        "unit",
        "counted",
        "trades",
        "period",
        "symbols",
        "r",
        "sharpe_per_trade",
        "control",
        "buy_and_hold",
        "per_year",
        "per_symbol",
        "evidence_against",
        "spread_split",
        "spread_estimated_trades",
        "long_trades",
        "short_trades",
        "rules",
        "verdict",
        "slippage_daily_bars",
        "attempts",
    ] {
        if let Some(v) = results.get(k) {
            r.insert(k.into(), v.clone());
        }
    }
    if let Some(b) = results.get("bootstrap") {
        r.insert(
            "bootstrap".into(),
            json!({
                "resamples": b["resamples"], "seed": b["seed"], "blocks": b["blocks"],
                "mean_r_lower_2_5": b["mean_r_lower_2_5"], "mean_r_upper_97_5": b["mean_r_upper_97_5"],
            }),
        );
    }
    if let Some(c) = results.get("recheck") {
        r.insert(
            "recheck".into(),
            json!({"counted": c["counted"], "checked": c["checked"], "no_tick_trade": c["no_tick_trade"], "mean_diff_r": c["mean_diff_r"]}),
        );
    }
    Value::Object(r)
}

pub async fn trials(pool: &PgPool) -> jiaoyi_store::Result<Value> {
    let trials = rs::job_trials(pool).await?;
    // Deflate for the 9 development trials of the same hypothesis and hash.
    let dev_srs: Vec<f64> = trials
        .iter()
        .filter(|t| t.hypothesis == "H1" && t.sample == "development")
        .filter_map(|t| t.results.as_ref().and_then(sr_of).map(|(s, _)| s))
        .collect();
    let sr0 = (dev_srs.len() == DSR_TRIALS)
        .then(|| {
            let n = dev_srs.len() as f64;
            let m = dev_srs.iter().sum::<f64>() / n;
            let var = dev_srs.iter().map(|s| (s - m).powi(2)).sum::<f64>() / (n - 1.0);
            expected_max_sharpe(var, DSR_TRIALS)
        })
        .flatten();
    let out: Vec<Value> = trials
        .iter()
        .map(|t| {
            let dsr = match (sr0, t.results.as_ref().and_then(sr_of)) {
                (Some(s0), Some((sr, m))) => json!({
                    "value": deflated_sharpe(sr, m.n, m.skew, m.kurt, s0),
                    "sr0": s0, "trials": DSR_TRIALS,
                    "method": "Bailey & López de Prado (2014); SR per trade; the variance is the sample variance of the 9 development SRs",
                }),
                _ => json!({"value": null, "pending": format!("{} of {DSR_TRIALS} development trials", dev_srs.len())}),
            };
            json!({
                "trial_id": t.trial_id,
                "job_id": t.job_id,
                "hypothesis": t.hypothesis,
                "sample": t.sample,
                "setting": t.params["setting"],
                "prereg_hash": t.prereg_hash,
                "verdict": t.verdict,
                "settings_version": t.settings_version,
                "run_at": t.run_at,
                "deflated_sharpe": dsr,
                "results": t.results.as_ref().map(public),
                "refused_symbols": t.params["refused_symbols"],
            })
        })
        .collect();
    Ok(json!({
        "notes": [
            "Counts and R only; notional money stays in the stored trial.",
            "PASS is judged on the held-out data at the default setting; development trials only feed the gate.",
        ],
        "trials": out,
    }))
}
