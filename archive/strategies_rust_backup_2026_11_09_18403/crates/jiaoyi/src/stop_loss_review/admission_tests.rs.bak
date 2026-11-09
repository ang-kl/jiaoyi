//! Saved admission history is independent of setup geometry and close hash.
use super::*;
use jiaoyi_store::strategy3::{record_order, OrderRow};

async fn opening(pool: &PgPool, account: i64) -> OrderRow {
    opening_with_stop(pool, account, 100000).await
}
async fn opening_with_stop(pool: &PgPool, account: i64, relative_stop: i64) -> OrderRow {
    let at = Utc::now() - Duration::hours(2);
    let key = format!("s3:{account}:opening");
    sqlx::query("INSERT INTO accounts(account_id,environment,login_missing_reason,currency,role_missing_reason,enabled) VALUES($1,'demo','unread','USD','unread',true)")
        .bind(account).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk,created_at) VALUES($1,$2,$2,'open',1,'buy',100,$4,10,11,$3)")
        .bind(account).bind(&key).bind(at).bind(relative_stop).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,deal_id,symbol_id,volume,received_at,raw) VALUES($1,$2,2126,'ORDER_FILLED',$1,$1*10,1,100,$3,''::bytea)")
        .bind(account).bind(&key).bind(at).execute(pool).await.unwrap();
    OrderRow {
        account_id: account,
        entry_key: key,
        label: "jiaoyi-s3".into(),
        prereg_hash: v7_hash(),
        kind: "open".into(),
        symbol: "BTCUSD".into(),
        symbol_id: 1,
        position_id: None,
        side: Some("buy".into()),
        session_date: Some(at.date_naive()),
        decided_at: at,
        bid: Some(101.0),
        ask: Some(102.0),
        volume: Some(100),
        detail: json!({"session_date":at.date_naive(),"decided_at":at,
            "session":{"date":at.date_naive(),"open":at-Duration::hours(1),
                "close":at+Duration::hours(4),"lunch":null},
            "bid":101.0,"ask":102.0,
            "entry":{"side":"buy","relative_stop":100000,"entry_price":102.0},
            "band":{"open":100.0,"sigma":0.01,"lower":99.0,"upper":101.0,
                "half_width":1.0,"minutes":60.0},
            "order":{"plan":{"broker_target":104.0}}}),
    }
}

async fn record_case(pool: &PgPool, open: &OrderRow) {
    record_order(pool, open).await.unwrap();
    let exit = OrderRow {
        entry_key: format!("s3:exit:{}", open.account_id),
        kind: "broker_exit".into(),
        // Deliberately different: review must use the opening registration.
        prereg_hash: "different-closing-registration".into(),
        position_id: Some(open.account_id),
        decided_at: Utc::now() - Duration::minutes(1),
        detail: json!({"reason":"stop_loss","deal_id":open.account_id*10+1}),
        ..open.clone()
    };
    record_order(pool, &exit).await.unwrap();
    sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,detail) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','BTCUSD',$1,'filled',$4,$5)")
        .bind(open.account_id).bind(&exit.entry_key).bind(&exit.prereg_hash)
        .bind(exit.decided_at).bind(&exit.detail).execute(pool).await.unwrap();
}

async fn skip(pool: &PgPool, open: &OrderRow, at: DateTime<Utc>, typed: bool) {
    let mut detail = open.detail.clone();
    detail["label"] = json!(open.label);
    detail["prereg_hash"] = json!(open.prereg_hash);
    if typed {
        detail["skip_code"] = json!("cost_ge_r");
    }
    sqlx::query("INSERT INTO trading_decisions(account_id,symbol,kind,reason,detail,at) VALUES($1,$2,'s3_skip',$3,$4,$5)")
        .bind(open.account_id).bind(&open.symbol)
        .bind(if typed { "typed cost veto" } else { "cost reserve C (397.888128) >= R (383.030000): barrier too far for a no-drift process; skipped" })
        .bind(detail).bind(at).execute(pool).await.unwrap();
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn prior_cost_veto_is_detected_without_retroactive_or_cross_scope_rejection(pool: PgPool) {
    for account in 1..=4 {
        let open = opening(&pool, account).await;
        record_case(&pool, &open).await;
        match account {
            1 => skip(&pool, &open, open.decided_at - Duration::hours(1), false).await,
            2 => skip(&pool, &open, open.decided_at - Duration::hours(1), true).await,
            3 => skip(&pool, &open, open.decided_at + Duration::minutes(1), true).await,
            _ => {
                for change in 0..5 {
                    let mut other = open.clone();
                    match change {
                        0 => other.account_id = 999,
                        1 => other.symbol = "ETHUSD".into(),
                        2 => other.prereg_hash = "another-registration".into(),
                        3 => {
                            other.detail["session_date"] =
                                json!(open.session_date.unwrap() - Duration::days(1))
                        }
                        _ => other.label = "another-owner".into(),
                    }
                    skip(&pool, &other, open.decided_at - Duration::hours(1), true).await;
                }
                sqlx::query("INSERT INTO trading_decisions(account_id,symbol,kind,reason,detail,at) VALUES($1,'BTCUSD','s3_skip','cost estimate unavailable',$2,$3)")
                    .bind(account).bind(json!({"label":"jiaoyi-s3","prereg_hash":open.prereg_hash,"session_date":open.session_date}))
                    .bind(open.decided_at-Duration::hours(1)).execute(&pool).await.unwrap();
            }
        }
    }
    let result = evidence(&pool, &[1, 2, 3, 4]).await.unwrap();
    assert_eq!(result["cases"].as_array().unwrap().len(), 4);
    for case in result["cases"].as_array().unwrap() {
        let account = case["account_id"].as_i64().unwrap();
        assert_eq!(
            case["checks"]["no_prior_recorded_session_cost_veto"],
            account >= 3,
            "account {account}"
        );
        assert_eq!(case["checks"]["first_recorded_session_entry"], true);
        assert_eq!(case["checks"]["entry_trigger_valid"], true);
        assert_eq!(
            case["original_entry"]["setup_complete"], true,
            "valid saved geometry must not conceal a proven prior admission veto"
        );
    }
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn prior_session_open_counts_across_hashes_but_not_later_or_other_scopes(pool: PgPool) {
    for account in 1..=3 {
        let open = opening(&pool, account).await;
        record_case(&pool, &open).await;
        let mut prior = OrderRow {
            entry_key: format!("s3:{account}:other-open"),
            prereg_hash: "older-registration".into(),
            decided_at: open.decided_at - Duration::minutes(30),
            ..open.clone()
        };
        match account {
            1 => record_order(&pool, &prior).await.unwrap(),
            2 => {
                prior.decided_at = open.decided_at + Duration::minutes(30);
                record_order(&pool, &prior).await.unwrap();
            }
            _ => {
                for change in 0..3 {
                    let mut other = prior.clone();
                    other.entry_key.push_str(&format!(":{change}"));
                    match change {
                        0 => other.account_id = 999,
                        1 => other.symbol_id = 2,
                        _ => {
                            other.session_date =
                                Some(open.session_date.unwrap() - Duration::days(1))
                        }
                    }
                    record_order(&pool, &other).await.unwrap();
                }
            }
        }
    }
    let result = evidence(&pool, &[1, 2, 3]).await.unwrap();
    for case in result["cases"].as_array().unwrap() {
        assert_eq!(
            case["checks"]["first_recorded_session_entry"],
            case["account_id"] != 1
        );
        assert_eq!(case["checks"]["no_prior_recorded_session_cost_veto"], true);
    }
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn unsupported_missing_contradictory_and_same_time_evidence_remains_unknown(pool: PgPool) {
    for account in 1..=5 {
        let mut open = opening(&pool, account).await;
        match account {
            1 => open.prereg_hash = "unsupported-historical-registration".into(),
            2 => {
                open.detail.as_object_mut().unwrap().remove("decided_at");
            }
            3 => {
                open.detail["session_date"] = json!(open.session_date.unwrap() - Duration::days(1))
            }
            4 => skip(&pool, &open, open.decided_at, true).await,
            _ => record_order(
                &pool,
                &OrderRow {
                    entry_key: "s3:same-time-open".into(),
                    ..open.clone()
                },
            )
            .await
            .unwrap(),
        }
        record_case(&pool, &open).await;
    }
    let result = evidence(&pool, &[1, 2, 3, 4, 5]).await.unwrap();
    for case in result["cases"].as_array().unwrap() {
        let account = case["account_id"].as_i64().unwrap();
        let cost_unknown = account <= 4;
        let entry_unknown = account != 4;
        assert_eq!(
            case["checks"]["no_prior_recorded_session_cost_veto"],
            if cost_unknown {
                Value::Null
            } else {
                json!(true)
            }
        );
        assert_eq!(
            case["checks"]["first_recorded_session_entry"],
            if entry_unknown {
                Value::Null
            } else {
                json!(true)
            }
        );
        let missing = case["missing_evidence"].as_array().unwrap();
        assert_eq!(
            missing.contains(&json!("recorded_session_cost_veto_history")),
            cost_unknown
        );
        assert_eq!(
            missing.contains(&json!("recorded_session_entry_history")),
            entry_unknown
        );
    }
}

pub(super) fn v7_hash() -> String {
    format!(
        "{:x}",
        Sha256::digest(include_bytes!(
            "../../../../docs/preregistration-2026-11-01-strategy3-v7.md"
        ))
    )
}

fn v6_hash() -> String {
    format!(
        "{:x}",
        Sha256::digest(include_bytes!(
            "../../../../docs/preregistration-2026-10-28-strategy3-v6.md"
        ))
    )
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn v7_inherits_only_prior_v6_cost_veto_without_rewriting_v6_history(pool: PgPool) {
    for account in 1..=6 {
        let mut open = opening(&pool, account).await;
        if matches!(account, 4 | 5) {
            open.prereg_hash = v6_hash();
        }
        record_case(&pool, &open).await;
        let mut veto = open.clone();
        veto.prereg_hash = if account == 5 { v7_hash() } else { v6_hash() };
        let at = match account {
            2 => open.decided_at + Duration::seconds(1),
            3 => open.decided_at,
            _ => open.decided_at - Duration::seconds(1),
        };
        if account == 6 {
            for change in 0..5 {
                let mut other = veto.clone();
                match change {
                    0 => other.account_id = 999,
                    1 => other.symbol = "ETHUSD".into(),
                    2 => other.prereg_hash = "unknown-predecessor".into(),
                    3 => {
                        other.detail["session_date"] =
                            json!(open.session_date.unwrap() - Duration::days(1));
                    }
                    _ => other.label = "another-owner".into(),
                }
                skip(&pool, &other, at, true).await;
            }
        } else {
            skip(&pool, &veto, at, true).await;
        }
    }
    let result = evidence(&pool, &[1, 2, 3, 4, 5, 6]).await.unwrap();
    assert_eq!(result["cases"].as_array().unwrap().len(), 6);
    for case in result["cases"].as_array().unwrap() {
        let account = case["account_id"].as_i64().unwrap();
        assert_eq!(
            case["checks"]["no_prior_recorded_session_cost_veto"],
            match account {
                1 | 4 => json!(false),
                3 => Value::Null,
                _ => json!(true),
            },
            "account {account}: only the original admission's known vetoes apply"
        );
        assert_eq!(case["checks"]["first_recorded_session_entry"], true);
        assert_eq!(
            case["checks"]["preregistration_matches_current_reference"], false,
            "historical recognition must not relabel v6 as the current registration"
        );
    }
    let old_hashes: Vec<String> = sqlx::query_scalar(
        "SELECT prereg_hash FROM strategy3_orders WHERE account_id IN (4,5) AND kind='open'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(old_hashes, vec![v6_hash(), v6_hash()]);
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn only_v7_uses_unsent_evidence_known_at_the_original_opening(pool: PgPool) {
    for account in 1..=9 {
        let mut open = opening(&pool, account).await;
        if account == 6 {
            open.prereg_hash = v6_hash();
        }
        record_case(&pool, &open).await;
        let prior = OrderRow {
            entry_key: format!("s3:{account}:prior-attempt"),
            decided_at: open.decided_at - Duration::minutes(30),
            ..open.clone()
        };
        record_order(&pool, &prior).await.unwrap();
        if account != 7 {
            let at = match account {
                2 => open.decided_at + Duration::seconds(1),
                3 => open.decided_at,
                _ => open.decided_at - Duration::seconds(1),
            };
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,received_at,raw) VALUES($1,$2,0,'REFUSED_BEFORE_SEND',$3,''::bytea)")
                .bind(if account == 8 { 999 } else { account })
                .bind(if account == 9 { "another-attempt" } else { &prior.entry_key })
                .bind(at).execute(&pool).await.unwrap();
        }
        if matches!(account, 4 | 5) {
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,received_at,raw) VALUES($1,$2,2126,'ORDER_ACCEPTED',$3,''::bytea)")
                .bind(account).bind(&prior.entry_key)
                .bind(open.decided_at + Duration::seconds(if account == 4 { -1 } else { 1 }))
                .execute(&pool).await.unwrap();
        }
    }
    let result = evidence(&pool, &[1, 2, 3, 4, 5, 6, 7, 8, 9]).await.unwrap();
    assert_eq!(result["cases"].as_array().unwrap().len(), 9);
    for case in result["cases"].as_array().unwrap() {
        let account = case["account_id"].as_i64().unwrap();
        assert_eq!(
            case["checks"]["first_recorded_session_entry"],
            match account {
                1 | 5 => json!(true),
                3 => Value::Null,
                _ => json!(false),
            },
            "account {account}: only a known durable unsent outcome releases v7 allowance"
        );
        assert_eq!(case["checks"]["no_prior_recorded_session_cost_veto"], true);
    }
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn pullback_review_uses_arm_not_peak_and_does_not_inherit_legacy_cost_veto(pool: PgPool) {
    use jiaoyi_core::strategy3::{band::Band, cost_reserve, targets::Dir};
    use jiaoyi_ctrader::trading::VolumeLimits;
    let legacy_hash = format!(
        "{:x}",
        Sha256::digest(include_bytes!(
            "../../../../docs/preregistration-2026-11-08-strategy3-macro-pullback-18380.md"
        ))
    );
    for account in 1..=6 {
        let planned_cost = cost_reserve(0.02, 0.0, 24.0, 100.51).unwrap();
        let planned = mp::geometry(true, 100.51, 100.0, planned_cost.total, 2).unwrap();
        let mut open = opening_with_stop(
            &pool,
            account,
            crate::trading::relative_stop(planned.risk, 2),
        )
        .await;
        open.prereg_hash = legacy_hash.clone();
        let at = open.decided_at;
        let session = Session {
            date: at.date_naive(),
            open: at - Duration::minutes(45),
            close: at + Duration::hours(4),
            lunch: None,
        };
        let band = Band {
            open: 100.0,
            sigma: 0.004,
            lower: 99.6,
            upper: 100.4,
            half_width: 0.4,
            minutes: 30.0,
        };
        let context = mp::Context {
            drift_5: 0.01,
            drift_20: 0.02,
            latest_bar_end: session.open,
        };
        let arm_cost = cost_reserve(0.02, 0.0, 24.0, 101.01).unwrap();
        let mut armed = mp::ArmedPullback::arm(
            "BTCUSD",
            session,
            band,
            100.99,
            101.01,
            session.open + Duration::minutes(30),
            context,
            None,
            arm_cost.total,
        )
        .unwrap();
        let limits = VolumeLimits {
            min: 100,
            step: 100,
            max: None,
            digits: 2,
        };
        let cost = cost_reserve(0.02, 0.0, 24.0, 100.51).unwrap();
        let g = mp::geometry(true, 100.51, 100.0, cost.total, 2).unwrap();
        let relative = crate::trading::relative_stop(g.risk, 2);
        let entry = crate::strategy3::Entry {
            side: Dir::Buy,
            band,
            entry_price: 100.51,
            relative_stop: relative,
            r: g.risk,
            cost,
        };
        let order = crate::strategy3::pullback_order(&entry, &limits, 100).unwrap();
        if account == 2 {
            armed.macro_context.drift_20 = -0.02;
        }
        open.bid = Some(100.49);
        open.ask = Some(100.51);
        open.detail = json!({"production_policy":mp::LEGACY_REPLAY_POLICY,"session_date":session.date,"session":session,
            "decided_at":at,"armed":armed,"macro_context":armed.macro_context,"band":band,
            "bid":100.49,"ask":100.51,"entry":entry,"volume_limits":limits,"order":order});
        if account == 4 {
            open.detail["bid"] = json!(100.99);
            open.detail["ask"] = json!(101.01);
        }
        if account != 3 {
            let arm_receipt = json!({"production_policy":mp::LEGACY_REPLAY_POLICY,"prereg_hash":open.prereg_hash,
                "state":armed,"bid":100.99,"ask":101.01,
                "detail":{"band":band,"cost":arm_cost,"peer_move":null}});
            sqlx::query("INSERT INTO trading_decisions(account_id,symbol,kind,reason,detail,at) VALUES($1,'BTCUSD','s3_pullback_state','armed_pullback',$2,$3)")
                .bind(account).bind(arm_receipt).bind(armed.armed_at).execute(&pool).await.unwrap();
            let guard = json!({"policy":jiaoyi_ctrader::entry_guard::POLICY,"account_id":account,"symbol_id":1,
                "stage":"after_writer_lock","result":"passed","deadline_elapsed":false,
                "account_revision":0,"expected_account_revision":0,"selected_side":"buy",
                "bid":{"price":100.49,"receipt_age_ms":0},"ask":{"price":100.51,"receipt_age_ms":0},
                "pullback_admitted_price":{"price":100.51,"digits":2}});
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,received_at,raw) VALUES($1,$2,0,'ENTRY_VALIDATED_BEFORE_SEND',$3,$4)")
                .bind(account).bind(&open.entry_key).bind(at).bind(serde_json::to_vec(&guard).unwrap()).execute(&pool).await.unwrap();
        }
        // Only the new policy's own cost veto applies, never legacy v6/v7.
        let mut veto = open.clone();
        if account != 5 {
            veto.prereg_hash = v6_hash();
        }
        skip(&pool, &veto, at - Duration::minutes(1), true).await;
        if account == 6 {
            let prior = OrderRow {
                entry_key: "s3:6:proven-unsent".into(),
                decided_at: at - Duration::minutes(1),
                ..open.clone()
            };
            record_order(&pool, &prior).await.unwrap();
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,received_at,raw) VALUES(6,$1,0,'REFUSED_BEFORE_SEND',$2,''::bytea)")
                .bind(prior.entry_key).bind(at-Duration::seconds(1)).execute(&pool).await.unwrap();
        }
        record_case(&pool, &open).await;
    }
    let out = evidence(&pool, &[1, 2, 3, 4, 5, 6]).await.unwrap();
    for c in out["cases"].as_array().unwrap() {
        let id = c["account_id"].as_i64().unwrap();
        let checks = &c["checks"];
        assert_eq!(checks["preregistration_matches_current_reference"], false);
        assert_eq!(checks["entry_trigger_valid"], id != 4, "retouch {id}");
        assert_eq!(
            checks["original_decision_mark_valid"], true,
            "arm mark, not later entry"
        );
        assert_eq!(checks["original_macro_alignment_valid"], id != 2);
        assert_eq!(
            checks["original_impulse_valid"],
            if id == 3 { Value::Null } else { json!(id != 2) }
        );
        assert_eq!(checks["original_stop_geometry_valid"], true);
        assert_eq!(checks["original_target_geometry_valid"], true);
        assert_eq!(
            checks["final_dispatch_guard_verified"],
            if id == 3 { Value::Null } else { json!(true) }
        );
        assert_eq!(checks["no_prior_recorded_session_cost_veto"], id != 5);
        assert_eq!(checks["first_recorded_session_entry"], true);
        assert_eq!(
            c["original_entry"]["rule_registration"],
            "docs/preregistration-2026-11-08-strategy3-macro-pullback-18380.md"
        );
        assert_eq!(c["original_entry"]["setup_complete"], id != 3);
        assert_eq!(
            c["net_costs_complete"], false,
            "missing history must never become measured costs"
        );
    }
}

#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn bounded_pullback_review_separates_trigger_bound_and_window_history(pool: PgPool) {
    use jiaoyi_core::strategy3::{band::Band, cost_reserve, targets::Dir};
    use jiaoyi_ctrader::trading::VolumeLimits;
    let hash = format!(
        "{:x}",
        Sha256::digest(include_bytes!(
            "../../../../docs/preregistration-2026-11-09-strategy3-macro-pullback-18392.md"
        ))
    );
    for account in 1..=6 {
        let planned_cost = cost_reserve(0.02, 0.0, 3.74, 100.61).unwrap();
        let planned_risk = mp::geometry(true, 100.61, 100.0, planned_cost.total, 2)
            .unwrap()
            .risk;
        let mut open = opening_with_stop(
            &pool,
            account,
            crate::trading::relative_stop(planned_risk, 2),
        )
        .await;
        let at = open.decided_at;
        let session = ps::session_at(ps::Market::Crypto, at).unwrap();
        let band = Band {
            open: 100.0,
            sigma: 0.004,
            lower: 99.6,
            upper: 100.4,
            half_width: 0.4,
            minutes: 0.0,
        };
        let context = mp::Context {
            drift_5: 0.01,
            drift_20: 0.02,
            latest_bar_end: session.open,
        };
        let arm_cost = cost_reserve(0.02, 0.0, 3.74, 101.01).unwrap();
        let armed = mp::ArmedPullback::arm_window(
            "BTCUSD",
            session,
            band,
            100.99,
            101.01,
            session.open + Duration::seconds(1),
            context,
            None,
            arm_cost.total,
        )
        .unwrap();
        let price = mp::dispatch_entry_bound(&armed, 0.02, 2).unwrap();
        let cost = cost_reserve(0.02, 0.0, 3.74, price).unwrap();
        let geometry = mp::geometry(true, price, armed.origin, cost.total, 2).unwrap();
        let relative = crate::trading::relative_stop(geometry.risk, 2);
        let limits = VolumeLimits {
            min: 100,
            step: 100,
            max: None,
            digits: 2,
        };
        let entry = crate::strategy3::Entry {
            side: Dir::Buy,
            band,
            entry_price: price,
            relative_stop: relative,
            r: geometry.risk,
            cost,
        };
        let order = crate::strategy3::pullback_order(&entry, &limits, 100).unwrap();
        open.prereg_hash = hash.clone();
        open.session_date = Some(session.date);
        open.bid = Some(100.49);
        open.ask = Some(100.51);
        open.detail = json!({"production_policy":mp::POLICY,"session_date":session.date,"session":session,
            "decided_at":at,"armed":armed,"macro_context":context,"band":band,
            "bid":100.49,"ask":100.51,"entry":entry,"volume_limits":limits,"order":order});
        let receipt = json!({"production_policy":mp::POLICY,"prereg_hash":hash,"state":armed,
            "detail":{"impulse_quote":{"bid":100.99,"ask":101.01},"band":band,"cost":arm_cost,"peer_move":null}});
        sqlx::query("INSERT INTO trading_decisions(account_id,symbol,kind,reason,detail,at) VALUES($1,'BTCUSD','s3_pullback_state','armed_pullback',$2,$3)")
            .bind(account).bind(receipt).bind(armed.armed_at).execute(&pool).await.unwrap();
        let checked = at + Duration::milliseconds(50);
        let touched_at = at - Duration::milliseconds(50);
        open.detail["trigger_quote"] = json!({"bid":100.49,"ask":100.51,
            "bid_at":touched_at,"ask_at":touched_at});
        let mut guard = json!({"policy":jiaoyi_ctrader::entry_guard::POLICY,
            "stage":"after_writer_lock","result":"passed","checked_at":checked,"deadline_elapsed":false,
            "account_revision":0,"expected_account_revision":0,"selected_side":"buy",
            "bid":{"price":100.54,"received_at":checked,"receipt_age_ms":0},
            "ask":{"price":100.56,"received_at":checked,"receipt_age_ms":0},
            "pullback_admitted_price":{"bounded":true,"price":price,"digits":2,
                "origin":armed.origin,"level":armed.level,"impulse_range":armed.impulse_range,"bounce_fraction":0.10,
                "trigger_bid":{"price":100.49,"received_at":touched_at,"receipt_age_ms":100},
                "trigger_ask":{"price":100.51,"received_at":touched_at,"receipt_age_ms":100}}});
        if account <= 3 {
            let mut prior = OrderRow {
                entry_key: format!("s3:{account}:prior"),
                decided_at: at - Duration::minutes(1),
                ..open.clone()
            };
            if account == 1 {
                // Same reporting date but another slot cannot veto this slot.
                prior.detail["session"]["open"] = json!(session.open - Duration::hours(4));
            } else if account == 3 {
                prior.detail["session"]
                    .as_object_mut()
                    .unwrap()
                    .remove("open");
            }
            skip(&pool, &prior, prior.decided_at, true).await;
            record_order(&pool, &prior).await.unwrap();
        } else if account == 4 {
            guard["pullback_admitted_price"]
                .as_object_mut()
                .unwrap()
                .remove("bounded");
        } else if account == 5 {
            // A permissible later bounce cannot manufacture the first 50% touch.
            open.detail["bid"] = json!(100.54);
            open.detail["ask"] = json!(100.56);
            open.detail["trigger_quote"]["bid"] = json!(100.54);
            open.detail["trigger_quote"]["ask"] = json!(100.56);
            guard["pullback_admitted_price"]["trigger_bid"]["price"] = json!(100.54);
            guard["pullback_admitted_price"]["trigger_ask"]["price"] = json!(100.56);
        } else if account == 6 {
            guard["ask"]["price"] = json!(price + 0.01);
        }
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,received_at,raw) VALUES($1,$2,0,'ENTRY_VALIDATED_BEFORE_SEND',$3,$4)")
            .bind(account).bind(&open.entry_key).bind(checked).bind(serde_json::to_vec(&guard).unwrap())
            .execute(&pool).await.unwrap();
        record_case(&pool, &open).await;
    }
    let out = evidence(&pool, &[1, 2, 3, 4, 5, 6]).await.unwrap();
    assert_eq!(out["cases"].as_array().unwrap().len(), 6);
    for case in out["cases"].as_array().unwrap() {
        let account = case["account_id"].as_i64().unwrap();
        let checks = &case["checks"];
        for name in [
            "original_session_valid",
            "original_decision_mark_valid",
            "original_impulse_valid",
            "original_stop_geometry_valid",
            "original_target_geometry_valid",
        ] {
            assert_eq!(checks[name], true, "{account} {name}");
        }
        assert_eq!(checks["entry_trigger_valid"], account != 5);
        assert_eq!(
            checks["final_dispatch_guard_verified"],
            match account {
                4 => Value::Null,
                5 | 6 => json!(false),
                _ => json!(true),
            },
            "bounded writer snapshot {account}"
        );
        for name in [
            "no_prior_recorded_session_cost_veto",
            "first_recorded_session_entry",
        ] {
            assert_eq!(
                checks[name],
                match account {
                    2 => json!(false),
                    3 => Value::Null,
                    _ => json!(true),
                },
                "window-scoped evidence {account} {name}"
            );
        }
        assert_eq!(case["net_costs_complete"], false);
    }
}
