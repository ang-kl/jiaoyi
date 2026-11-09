//! The factory's limits and the owner's pause (owner, 25-10-2026).
//!
//! - **Limits alert; they never pause.** A strategy on an account whose
//!   largest fall in cumulative net R reaches `trading.pause_drawdown_r`, or
//!   whose profit factor is below `trading.pause_pf_floor` after at least
//!   `trading.pause_pf_min_trades` closed trades, is flagged "LIMIT REACHED:
//!   owner's decision needed" on `/health` and the status page, and one
//!   decision is recorded per change of what is reached. Trading carries on
//!   ("only human can pause trade"; "always live trade").
//! - **The pause is the owner's.** `trading.s1_paused` and `trading.s3_paused`
//!   are read here and by the traders' entry paths; nothing in this program
//!   writes them. A paused strategy takes no new entry on that account; its
//!   open positions keep every exit.
//!
//! Results are in R only: the broker's net money for each closed position
//! over the risk the gate recorded for its open (`ts::closed_trades`). No
//! money is shown.
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)

use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::trading::{self as ts, Performance};
use serde_json::{json, Value};

/// One strategy's names: its entry-key code, the owner's pause key, and its
/// limit decisions' kinds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strategy {
    pub code: &'static str,
    pub pause_key: &'static str,
    pub reached: &'static str,
    pub cleared: &'static str,
}

pub const S1: Strategy = Strategy {
    code: "s1",
    pause_key: "trading.s1_paused",
    reached: "limit_reached",
    cleared: "limit_cleared",
};

pub const S3: Strategy = Strategy {
    code: "s3",
    pause_key: "trading.s3_paused",
    reached: "s3_limit_reached",
    cleared: "s3_limit_cleared",
};

/// The owner's pause keys. No code decides to write them; a reviewed seed
/// entry (`trading::apply_seed`) may set one only as the owner's word
/// (`changed_by` "owner").
pub const PAUSE_KEYS: [&str; 2] = [S1.pause_key, S3.pause_key];

/// The limits in force for an account.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub drawdown_r: f64,
    pub pf_floor: f64,
    pub pf_min_trades: i64,
}

pub async fn limits(pool: &PgPool, account_id: i64) -> anyhow::Result<Limits> {
    let s = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    Ok(Limits {
        drawdown_r: s
            .number("trading.pause_drawdown_r")
            .map_err(anyhow::Error::msg)?,
        pf_floor: s
            .number("trading.pause_pf_floor")
            .map_err(anyhow::Error::msg)?,
        pf_min_trades: s
            .integer("trading.pause_pf_min_trades")
            .map_err(anyhow::Error::msg)?,
    })
}

/// The limits a strategy's results have reached: (limit, value, threshold).
pub fn reached(p: &Performance, l: &Limits) -> Vec<(&'static str, f64, f64)> {
    let mut out = Vec::new();
    if p.max_drawdown_r >= l.drawdown_r {
        out.push(("drawdown_r", p.max_drawdown_r, l.drawdown_r));
    }
    if p.trades as i64 >= l.pf_min_trades {
        if let Some(pf) = p.profit_factor.filter(|pf| *pf < l.pf_floor) {
            out.push(("profit_factor", pf, l.pf_floor));
        }
    }
    out
}

/// `/health` → `performance`: the results, the limits, how far each is, and
/// what is reached. R only.
pub fn performance_json(p: &Performance, l: &Limits) -> Value {
    let hit = reached(p, l);
    let judged = p.trades as i64 >= l.pf_min_trades;
    let complete = p.unmeasured == 0;
    let show_result = complete || p.trades > 0;
    json!({
        "closed_trades": p.trades,
        "unmeasured": p.unmeasured,
        "evidence_state": if complete { "measured" } else { "incomplete" },
        "performance_scope": "measured closed positions only",
        "limit_status": if !hit.is_empty() { "reached" } else if complete { "within_limits" } else { "unknown" },
        "retained_limit_alerts": [],
        "net_r": show_result.then_some(p.net_r),
        "profit_factor": p.profit_factor,
        "max_drawdown_r": show_result.then_some(p.max_drawdown_r),
        "current_drawdown_r": show_result.then_some(p.current_drawdown_r),
        "last_closed_at": p.last_closed_at,
        "limits": {
            "drawdown_r": l.drawdown_r,
            "pf_floor": l.pf_floor,
            "pf_min_trades": l.pf_min_trades,
        },
        "to_limits": {
            "drawdown_room_r": complete.then_some(l.drawdown_r - p.max_drawdown_r),
            "pf_trades_to_judge": complete.then_some((l.pf_min_trades - p.trades as i64).max(0)),
            "pf_margin": if complete && judged { p.profit_factor.map(|pf| pf - l.pf_floor) } else { None },
        },
        "limit_reached": if hit.is_empty() {
            Value::Null
        } else {
            json!(hit
                .iter()
                .map(|(limit, value, threshold)| json!({
                    "limit": limit, "value": value, "threshold": threshold,
                }))
                .collect::<Vec<_>>())
        },
        "owner_decision_needed": !hit.is_empty(),
    })
}

/// Incomplete evidence cannot clear an earlier alert. Retain its name, not
/// its old numeric value as if it were a fresh measurement.
fn retain_prior_alerts(detail: &mut Value, last: Option<&(String, Value)>, s: &Strategy) {
    if detail["evidence_state"] != "incomplete" {
        return;
    }
    let Some((_, prior)) = last.filter(|(kind, _)| kind == s.reached) else {
        return;
    };
    let mut current = detail["limit_reached"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut retained = Vec::new();
    for name in ["drawdown_r", "profit_factor"] {
        if prior["limits"]
            .as_array()
            .is_some_and(|names| names.iter().any(|value| value == name))
            && !current.iter().any(|value| value["limit"] == name)
        {
            current.push(json!({"limit": name, "value": null, "threshold": null,
                "evidence": "prior_alert_unresolved"}));
            retained.push(name);
        }
    }
    if !retained.is_empty() {
        if detail["limit_status"] == "unknown" {
            detail["limit_status"] = json!("prior_alert_unresolved");
        }
        detail["limit_reached"] = json!(current);
        detail["retained_limit_alerts"] = json!(retained);
        detail["owner_decision_needed"] = json!(true);
    }
}

/// A strategy's results and limits on an account, for `/health`.
pub async fn health(pool: &PgPool, account_id: i64, s: &Strategy) -> anyhow::Result<Value> {
    let p = ts::performance(pool, account_id, s.code).await?;
    let l = limits(pool, account_id).await?;
    let mut detail = performance_json(&p, &l);
    if p.unmeasured > 0 {
        let last = ts::latest_limit_decision(pool, account_id, &[s.reached, s.cleared]).await?;
        retain_prior_alerts(&mut detail, last.as_ref(), s);
    }
    Ok(detail)
}

/// Check a strategy's limits on an account and record a decision when what
/// is reached changes: `reached` with the limits and the numbers, or
/// `cleared` when nothing is reached any more. It records; it never pauses.
/// Returns the limits reached now, including prior alerts retained while
/// results are incomplete. Missing evidence is never a recovery.
pub async fn check(
    pool: &PgPool,
    account_id: i64,
    s: &Strategy,
    extra: Value,
    settings_version: Option<i64>,
) -> anyhow::Result<Vec<&'static str>> {
    let p = ts::performance(pool, account_id, s.code).await?;
    let l = limits(pool, account_id).await?;
    let hit = reached(&p, &l);
    let last = ts::latest_limit_decision(pool, account_id, &[s.reached, s.cleared]).await?;
    let last_names: Option<Vec<String>> = last.as_ref().and_then(|(kind, d)| {
        (kind == s.reached).then(|| {
            d["limits"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        })
    });
    let mut detail = performance_json(&p, &l);
    retain_prior_alerts(&mut detail, last.as_ref(), s);
    let names: Vec<&'static str> = ["drawdown_r", "profit_factor"]
        .into_iter()
        .filter(|name| {
            detail["limit_reached"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row["limit"] == *name))
        })
        .collect();
    if let (Some(d), Some(e)) = (detail.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            d.insert(k.clone(), v.clone());
        }
    }
    detail["limits_reached"] = json!(names);
    detail["limits"] = json!(names);
    detail["thresholds"] = json!({
        "drawdown_r": l.drawdown_r, "pf_floor": l.pf_floor, "pf_min_trades": l.pf_min_trades,
    });
    if !names.is_empty() {
        let same = last_names
            .as_ref()
            .is_some_and(|n| n.iter().map(String::as_str).eq(names.iter().copied()));
        if !same {
            let mut what = hit
                .iter()
                .map(|(limit, value, threshold)| match *limit {
                    "drawdown_r" => format!("drawdown {value:.2}R ≥ {threshold}R"),
                    _ => format!(
                        "profit factor {value:.2} < {threshold} after {} closed trades",
                        p.trades
                    ),
                })
                .collect::<Vec<_>>()
                .join("; ");
            if let Some(retained) = detail["retained_limit_alerts"]
                .as_array()
                .filter(|a| !a.is_empty())
            {
                what.push_str(&format!(
                    "; prior alerts retained while evidence is incomplete: {}",
                    retained
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            ts::record_decision(
                pool,
                account_id,
                None,
                s.reached,
                &format!(
                    "LIMIT REACHED ({what}): owner's decision needed. Trading continues; only the owner pauses"
                ),
                &detail,
                settings_version,
            )
            .await?;
        }
    } else if p.unmeasured == 0 && last_names.is_some() {
        ts::record_decision(
            pool,
            account_id,
            None,
            s.cleared,
            "no limit reached any more",
            &detail,
            settings_version,
        )
        .await?;
    }
    Ok(names)
}

/// The owner's pause of a strategy on an account: whether it is set, and the
/// last change of its key for this account (or the global one): who, when,
/// and why.
pub async fn owner_pause(pool: &PgPool, account_id: i64, s: &Strategy) -> anyhow::Result<Value> {
    let snap = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let paused = snap.integer(s.pause_key).map_err(anyhow::Error::msg)? == 1;
    let h = jiaoyi_store::settings::history(pool, Some(s.pause_key)).await?;
    let row = h
        .iter()
        .find(|c| c.account_id == Some(account_id))
        .or_else(|| h.iter().find(|c| c.account_id.is_none()));
    Ok(json!({
        "paused": paused,
        "key": s.pause_key,
        "changed_by": row.map(|c| c.changed_by.clone()),
        "changed_at": row.map(|c| c.changed_at),
        "reason": row.map(|c| c.reason.clone()),
    }))
}
