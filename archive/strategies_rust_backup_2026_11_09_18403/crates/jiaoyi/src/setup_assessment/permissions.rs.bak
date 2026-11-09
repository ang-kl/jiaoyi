//! Read-only new-entry permissions. These never change a held position's exits.
use std::collections::{HashMap, HashSet};

use chrono::Utc;
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::strategy3::sessions;
use jiaoyi_ctrader::trading::PROGRAM_LABELS;
use jiaoyi_store::sqlx::PgPool;

use super::evaluate::{Input, PermissionEvidence, StrategyEvidence};

fn strategy_key(e: &StrategyEvidence) -> Option<&'static str> {
    if matches!(e.strategy.as_str(), "s1" | "strategy1" | "jiaoyi-s1")
        || e.strategy == crate::trading::STRATEGY
    {
        Some("s1")
    } else if matches!(e.strategy.as_str(), "s3" | "strategy3" | "jiaoyi-s3")
        || e.strategy == crate::strategy3::STRATEGY
    {
        Some("s3")
    } else {
        None
    }
}

async fn live_capacity(
    pool: &PgPool,
    input: &Input,
    environment: Option<Environment>,
) -> Option<bool> {
    match environment {
        Some(Environment::Demo) => Some(false),
        Some(Environment::Live) => {
            let ours = |label: &Option<String>| {
                label
                    .as_deref()
                    .is_some_and(|l| PROGRAM_LABELS.contains(&l))
            };
            // The shared admission reader uses reconcile without protection
            // orders. Do not turn our fuller protection inventory into entries.
            let observed_block = input
                .positions
                .iter()
                .any(|p| p.position_status == 1 && ours(&p.trade_data.label))
                || input.orders.iter().any(|o| {
                    o.order_status == 1
                        && o.closing_order != Some(true)
                        && o.order_type != 4
                        && ours(&o.trade_data.label)
                });
            if observed_block {
                return Some(true);
            }
            jiaoyi_store::trading::open_intents_unresolved(pool, input.account_id)
                .await
                .ok()
                .map(|rows| {
                    rows.iter()
                        .any(|r| crate::admission::program_intent(&r.entry_key))
                })
        }
        None => None,
    }
}

pub async fn collect(pool: &PgPool, input: &Input) -> HashMap<String, PermissionEvidence> {
    // Timestamp the beginning: slow reads cannot masquerade as fresh evidence.
    let observed_at = Utc::now();
    let mut held: HashMap<&str, HashSet<i64>> = HashMap::new();
    for p in &input.positions {
        if p.position_status != 1 {
            continue;
        }
        if let Some(key) = input.strategy.get(&p.position_id).and_then(strategy_key) {
            held.entry(key).or_default().insert(p.trade_data.symbol_id);
        }
    }
    if held.is_empty() {
        return HashMap::new();
    }
    let environment = jiaoyi_store::sqlx::query_scalar::<_, String>(
        "SELECT environment FROM accounts WHERE account_id=$1 AND enabled",
    )
    .bind(input.account_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .and_then(|s| match s.as_str() {
        "demo" => Some(Environment::Demo),
        "live" => Some(Environment::Live),
        _ => None,
    });
    let settings = jiaoyi_store::settings::snapshot(pool, Some(input.account_id))
        .await
        .ok();
    let capacity = live_capacity(pool, input, environment).await;
    let mut out = HashMap::new();
    for (key, symbols) in held {
        let pause_key = if key == "s1" {
            "trading.s1_paused"
        } else {
            "trading.s3_paused"
        };
        let mut evidence = PermissionEvidence {
            observed_at,
            book_revision: input.revision,
            owner_paused: settings
                .as_ref()
                .and_then(|s| s.integer(pause_key).ok())
                .map(|v| v == 1),
            cost_veto_by_symbol: HashMap::new(),
            entry_recorded_by_symbol: HashMap::new(),
            live_single_position_blocked: capacity,
            s1_edge_passed: None,
        };
        if key == "s1" {
            if let Some(env) = environment {
                if let Ok(rules) = crate::trading::rules(pool, input.account_id, env).await {
                    evidence.s1_edge_passed = crate::trading::edge_gate(pool, &rules)
                        .await
                        .ok()
                        .map(|r| r.is_ok());
                }
            }
        } else if let (Some(settings), Ok(resolutions)) = (
            &settings,
            jiaoyi_store::latest_resolutions(pool, input.account_id).await,
        ) {
            if let Ok(universe) = settings.universe("universe.symbols") {
                let universe = crate::strategy3::data_universe(universe);
                let hash = crate::strategy3::prereg_hash();
                for symbol in symbols {
                    let matches: Vec<_> = universe
                        .iter()
                        .filter_map(|(name, class)| {
                            resolutions
                                .iter()
                                .any(|r| {
                                    r.name == *name
                                        && r.status == "resolved"
                                        && r.broker_symbol_id == Some(symbol)
                                })
                                .then_some((name, class))
                        })
                        .collect();
                    // A symbol alias ambiguity must not pick an arbitrary session.
                    let [(name, class)] = matches.as_slice() else {
                        continue;
                    };
                    let Some(rule) = sessions::rule_for(name, **class) else {
                        continue;
                    };
                    let Some(session) = sessions::session_at(&rule, observed_at) else {
                        continue;
                    };
                    if let Ok(veto) = crate::strategy3::session_cost_veto(
                        pool,
                        input.account_id,
                        name,
                        session.date,
                        &hash,
                    )
                    .await
                    {
                        evidence.cost_veto_by_symbol.insert(symbol, veto);
                    }
                    if let Ok(count) = jiaoyi_store::strategy3::opens_in_session(
                        pool,
                        input.account_id,
                        symbol,
                        session.date,
                    )
                    .await
                    {
                        evidence.entry_recorded_by_symbol.insert(symbol, count > 0);
                    }
                }
            }
        }
        out.insert(key.into(), evidence);
    }
    out
}

#[cfg(test)]
#[path = "../../test_support/setup_assessment_permissions.rs"]
mod tests;
