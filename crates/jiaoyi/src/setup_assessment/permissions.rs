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
mod tests {
    use super::*;
    use jiaoyi_core::universe::{AssetClass, Resolution};
    use jiaoyi_ctrader::messages as pb;
    use jiaoyi_store as store;
    use serde_json::json;

    fn observed(strategy: &str) -> Input {
        let mut input = super::super::State::new(7, 1).input;
        input.revision = 12;
        input.complete = true;
        input.book_at = Some(Utc::now());
        input.positions.push(pb::ProtoOaPosition {
            position_id: 10,
            position_status: 1,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: 9,
                volume: 10,
                trade_side: 1,
                label: Some(
                    if strategy == "s3" {
                        "jiaoyi-s3"
                    } else {
                        "jiaoyi-s1"
                    }
                    .into(),
                ),
                ..Default::default()
            },
            ..Default::default()
        });
        input.strategy.insert(
            10,
            StrategyEvidence {
                strategy: strategy.into(),
                side: "buy".into(),
                detail: json!({}),
                recorded_at: Utc::now(),
                original_volume: 10,
                entry_key: format!("{strategy}:7:9:opening"),
            },
        );
        input
    }

    #[sqlx::test(migrator = "store::MIGRATOR")]
    async fn reads_actual_pause_cost_and_session_gates_without_assigning_manual_orders(
        pool: PgPool,
    ) {
        store::upsert_account(&pool, 7, Environment::Demo)
            .await
            .unwrap();
        store::settings::change(
            &pool,
            "universe.symbols",
            None,
            Some(json!([{"name":"BTCUSD","class":"crypto"}])),
            "test",
            "permission evidence",
        )
        .await
        .unwrap();
        let run = store::start_run(&pool, 7).await.unwrap();
        store::record_resolution(
            &pool,
            run,
            7,
            "BTCUSD",
            AssetClass::Crypto,
            &Resolution::Resolved { symbol_id: 9 },
        )
        .await
        .unwrap();
        let input = observed("s3");
        let before = collect(&pool, &input).await;
        assert_eq!(before["s3"].owner_paused, Some(false));
        assert_eq!(before["s3"].cost_veto_by_symbol.get(&9), Some(&false));
        assert_eq!(before["s3"].entry_recorded_by_symbol.get(&9), Some(&false));
        assert_eq!(before["s3"].live_single_position_blocked, Some(false));
        assert_eq!(before["s3"].book_revision, 12);
        store::settings::change(
            &pool,
            "trading.s3_paused",
            Some(7),
            Some(json!(1)),
            "test",
            "new entries paused; exits retained",
        )
        .await
        .unwrap();
        let at = Utc::now();
        let rule = sessions::rule_for("BTCUSD", AssetClass::Crypto).unwrap();
        let date = sessions::session_at(&rule, at).unwrap().date;
        let hash = crate::strategy3::prereg_hash();
        store::trading::record_decision(&pool, 7, Some("BTCUSD"), "s3_skip", "registered cost veto",
            &json!({"label":"jiaoyi-s3","prereg_hash":hash,"session_date":date,"skip_code":"cost_ge_r"}), None).await.unwrap();
        store::strategy3::record_order(
            &pool,
            &store::strategy3::OrderRow {
                account_id: 7,
                entry_key: "s3:7:9:opening".into(),
                label: "jiaoyi-s3".into(),
                prereg_hash: crate::strategy3::prereg_hash(),
                kind: "open".into(),
                symbol: "BTCUSD".into(),
                symbol_id: 9,
                position_id: Some(10),
                side: Some("buy".into()),
                session_date: Some(date),
                decided_at: at,
                bid: Some(100.0),
                ask: Some(101.0),
                volume: Some(10),
                detail: json!({}),
            },
        )
        .await
        .unwrap();
        let after = collect(&pool, &input).await;
        assert_eq!(after["s3"].owner_paused, Some(true));
        assert_eq!(after["s3"].cost_veto_by_symbol.get(&9), Some(&true));
        assert_eq!(after["s3"].entry_recorded_by_symbol.get(&9), Some(&true));
        let mut manual = input.clone();
        manual.strategy.clear();
        assert!(collect(&pool, &manual).await.is_empty());
        // A second universe name pointing at the same broker identity cannot
        // pick one of two session definitions without proof.
        store::settings::change(
            &pool,
            "universe.symbols",
            None,
            Some(json!([{"name":"BTCUSD","class":"crypto"},{"name":"ETHUSD","class":"crypto"}])),
            "test",
            "ambiguous observation mapping",
        )
        .await
        .unwrap();
        store::record_resolution(
            &pool,
            run,
            7,
            "ETHUSD",
            AssetClass::Crypto,
            &Resolution::Resolved { symbol_id: 9 },
        )
        .await
        .unwrap();
        let ambiguous = collect(&pool, &input).await;
        assert!(ambiguous["s3"].cost_veto_by_symbol.is_empty());
        assert!(ambiguous["s3"].entry_recorded_by_symbol.is_empty());
    }

    #[sqlx::test(migrator = "store::MIGRATOR")]
    async fn s1_uses_actual_edge_gate_and_live_capacity_keeps_known_blocks_on_read_failure(
        pool: PgPool,
    ) {
        store::upsert_account(&pool, 7, Environment::Demo)
            .await
            .unwrap();
        let mut input = observed("s1");
        let read = collect(&pool, &input).await;
        assert_eq!(read["s1"].s1_edge_passed, Some(true));
        assert_eq!(read["s1"].owner_paused, Some(false));
        pool.close().await;
        assert_eq!(
            live_capacity(&pool, &input, Some(Environment::Live)).await,
            Some(true)
        );
        input.positions.clear();
        assert_eq!(
            live_capacity(&pool, &input, Some(Environment::Live)).await,
            None
        );
        assert_eq!(
            live_capacity(&pool, &input, Some(Environment::Demo)).await,
            Some(false)
        );
    }
}
