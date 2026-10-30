//! Saved admission history is independent of setup geometry and close hash.
use super::*;
use jiaoyi_store::strategy3::{record_order, OrderRow};

async fn opening(pool: &PgPool, account: i64) -> OrderRow {
    let at = Utc::now() - Duration::hours(2);
    let key = format!("s3:{account}:opening");
    sqlx::query("INSERT INTO accounts(account_id,environment,login_missing_reason,currency,role_missing_reason,enabled) VALUES($1,'demo','unread','USD','unread',true)")
        .bind(account).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk,created_at) VALUES($1,$2,$2,'open',1,'buy',100,100000,10,11,$3)")
        .bind(account).bind(&key).bind(at).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,deal_id,symbol_id,volume,received_at,raw) VALUES($1,$2,2126,'ORDER_FILLED',$1,$1*10,1,100,$3,''::bytea)")
        .bind(account).bind(&key).bind(at).execute(pool).await.unwrap();
    OrderRow {
        account_id: account,
        entry_key: key,
        label: "jiaoyi-s3".into(),
        prereg_hash: crate::strategy3::prereg_hash(),
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
