//! Public, read-only evidence for reviews of recorded, owned stop-loss exits.
//! Saved entry facts are compared with registered rules; no current quotes,
//! broker requests, execution decisions, or free-form details cross this API.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use serde_json::{json, Value};
use std::collections::BTreeSet;

mod session;

const CANDIDATES: &str = r#"
WITH exits AS (
 SELECT 's1'::text AS strategy,f.account_id,f.position_id,f.symbol,f.detail,
 COALESCE(f.answered_at,f.sent_at) AS closed_at,f.id
 FROM strategy1_fills f WHERE f.account_id=ANY($1) AND f.kind='broker_exit'
 AND f.outcome IN ('filled','partly_filled') AND f.detail->>'reason'='stop_loss'
 AND COALESCE(f.answered_at,f.sent_at)>=$2
 UNION ALL
 SELECT 's3',f.account_id,f.position_id,f.symbol,f.detail,
 COALESCE(f.answered_at,f.sent_at),f.id
 FROM strategy3_fills f WHERE f.account_id=ANY($1) AND f.kind='broker_exit'
 AND f.outcome IN ('filled','partly_filled') AND f.detail->>'reason'='stop_loss'
 AND COALESCE(f.answered_at,f.sent_at)>=$2
), recent AS (SELECT * FROM exits WHERE closed_at<=$3 ORDER BY closed_at DESC,id DESC LIMIT 100)
SELECT r.*,a.environment,i.entry_key,i.symbol_id,i.side,i.volume,i.relative_stop,
 i.risk,i.max_risk,i.take_profit,i.created_at,i.owner_count,
 COALESCE(s3.detail,s1.detail,'{}'::jsonb) AS setup,
 COALESCE(s3.prereg_hash,s1.prereg_hash) AS prereg_hash
FROM recent r JOIN accounts a ON a.account_id=r.account_id
JOIN LATERAL (
 SELECT i.*,count(*) OVER() AS owner_count FROM trading_intents i
 WHERE i.account_id=r.account_id AND i.kind='open'
 AND i.entry_key LIKE r.strategy||':%'
 AND EXISTS(SELECT 1 FROM trading_events e WHERE e.account_id=i.account_id
   AND e.entry_key=i.entry_key AND e.position_id=r.position_id
   AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL'))
 ORDER BY i.id LIMIT 1
) i ON true
LEFT JOIN LATERAL (
 SELECT o.detail,o.prereg_hash FROM strategy3_orders o
 WHERE r.strategy='s3' AND o.account_id=r.account_id AND o.entry_key=i.entry_key AND o.kind='open'
 LIMIT 1
) s3 ON true
LEFT JOIN LATERAL (
 SELECT d.detail,f.prereg_hash FROM strategy1_fills f
 LEFT JOIN trading_decisions d ON d.account_id=f.account_id AND d.kind='sent'
   AND d.detail->>'entry_key'=f.entry_key
 WHERE r.strategy='s1' AND f.account_id=r.account_id AND f.entry_key=i.entry_key AND f.kind='open'
 ORDER BY d.id LIMIT 1
) s1 ON true
ORDER BY r.closed_at DESC,r.id DESC
"#;

fn number(v: &Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64().filter(|x| x.is_finite())
}

fn saved_band(v: &Value) -> Option<jiaoyi_core::strategy3::band::Band> {
    Some(jiaoyi_core::strategy3::band::Band {
        open: number(v, "open")?,
        sigma: number(v, "sigma")?,
        lower: number(v, "lower")?,
        upper: number(v, "upper")?,
        half_width: number(v, "half_width")?,
        minutes: number(v, "minutes")?,
    })
}

/// Only configuration-permitted accounts are queried. At most 100 recent
/// recorded exits are inspected; position history is fetched only for these.
pub async fn evidence(pool: &PgPool, accounts: &[i64]) -> Result<Value, sqlx::Error> {
    let now = Utc::now();
    let rows = sqlx::query(CANDIDATES)
        .bind(accounts)
        .bind(now - Duration::hours(24))
        .bind(now)
        .fetch_all(pool)
        .await?;
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        let detail: Value = row.get("detail");
        let Some(deal) = detail["deal_id"].as_i64().filter(|d| *d > 0) else {
            continue;
        };
        let account: i64 = row.get("account_id");
        let Some(position) = row.get::<Option<i64>, _>("position_id").filter(|p| *p > 0) else {
            continue;
        };
        let strategy: String = row.get("strategy");
        if !seen.insert((account, position, deal)) {
            continue;
        }
        let setup: Value = row.get("setup");
        let side: Option<String> = row.get("side");
        let sign = match side.as_deref() {
            Some("buy") => Some(1.0),
            Some("sell") => Some(-1.0),
            _ => None,
        };
        let rel: Option<i64> = row.get("relative_stop");
        let risk: Option<f64> = row.get("risk");
        let cap: f64 = row.get("max_risk");
        let gate = risk.map(|r| r.is_finite() && r > 0.0);
        let sizing = risk
            .filter(|r| r.is_finite())
            .and_then(|r| cap.is_finite().then_some(r > 0.0 && r <= cap));
        let saved_hash: Option<String> = row.get("prereg_hash");
        let current_hash = if strategy == "s1" {
            crate::edge::prereg::hash()
        } else {
            crate::strategy3::prereg_hash()
        };
        let registered = saved_hash.as_ref().map(|h| h == &current_hash);
        let mut direction = None;
        let mut trigger = None;
        let mut stop = None;
        let mut target = None;
        let mut band = None;
        if strategy == "s1" {
            if let (Some(s), Some(f), Some(l)) =
                (sign, number(&setup, "ema_fast"), number(&setup, "ema_slow"))
            {
                direction = Some(s * (f - l) > 0.0);
            }
            if let (Some(s), Some(level), Some(quote)) = (
                sign,
                number(&setup, "level"),
                number(&setup, if sign == Some(1.0) { "ask" } else { "bid" }),
            ) {
                trigger = Some(s * (quote - level) >= 0.0);
            }
            if let (Some(r), Some(atr)) = (rel, number(&setup, "atr")) {
                // Consistency with the registered 3 ATR distance on one of
                // the broker's supported precision grids, using saved ATR.
                stop = Some(
                    r > 0
                        && atr > 0.0
                        && (0..=5)
                            .any(|digits| crate::trading::relative_stop(3.0 * atr, digits) == r),
                );
            }
            // Registered Strategy 1 has no original take-profit.
            target = Some(row.get::<Option<f64>, _>("take_profit").is_none());
        } else {
            direction = sign.and_then(|_| {
                setup["entry"]["side"]
                    .as_str()
                    .map(|s| Some(s) == side.as_deref())
            });
            if let Some(b) = saved_band(&setup["band"]) {
                let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
                band = Some(
                    b.open > 0.0
                        && b.sigma > 0.0
                        && b.half_width > 0.0
                        && close(b.half_width, b.open * b.sigma)
                        && close(b.lower, b.open - b.half_width)
                        && close(b.upper, b.open + b.half_width),
                );
                // Use the execution strategy's exact strict boundary and
                // dual-trigger rule, against its persisted original quotes.
                if let Some(expected) = sign {
                    let actual = jiaoyi_core::strategy3::band::triggered(
                        &b,
                        number(&setup, "bid"),
                        number(&setup, "ask"),
                    );
                    trigger = Some(matches!(
                        (expected, actual),
                        (1.0, Some(jiaoyi_core::strategy3::targets::Dir::Buy))
                            | (-1.0, Some(jiaoyi_core::strategy3::targets::Dir::Sell))
                    ));
                }
            }
            if let (Some(r), Some(saved)) = (rel, setup["entry"]["relative_stop"].as_i64()) {
                stop = Some(r > 0 && r == saved);
            }
            let plan = &setup["order"]["plan"];
            if let (Some(s), Some(entry), Some(tp)) = (
                sign,
                number(&setup["entry"], "entry_price"),
                number(plan, "broker_target"),
            ) {
                target = Some(s * (tp - entry) > 0.0);
            }
        }
        // Only the saved original S3 snapshot can establish entry-window
        // eligibility; legacy/date-only evidence remains unknown.
        let session = (strategy == "s3" && registered == Some(true))
            .then(|| session::check(&setup))
            .flatten();
        let checks = json!({"gate_risk_recorded":gate,"sizing_within_gate_cap":sizing,
            "preregistration_matches_current_reference":registered,"entry_direction_valid":direction,
            "entry_trigger_valid":trigger,"original_stop_geometry_valid":stop,
            "original_target_geometry_valid":target,"original_session_valid":session,"original_band_valid":band});
        let created: DateTime<Utc> = row.get("created_at");
        let closed: DateTime<Utc> = row.get("closed_at");
        let key: String = row.get("entry_key");
        let receipts: i64 = sqlx::query_scalar(r#"SELECT GREATEST(COALESCE(MAX(volume),0),COALESCE((
          SELECT SUM(volume)::bigint FROM(SELECT DISTINCT ON(deal_id) volume FROM trading_events
          WHERE account_id=$1 AND entry_key=$2 AND position_id=$3 AND deal_id IS NOT NULL
          AND kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
          ORDER BY deal_id,id DESC) d),0)) FROM trading_events WHERE account_id=$1
          AND entry_key=$2 AND position_id=$3 AND kind IN
          ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')"#)
            .bind(account).bind(&key).bind(position).fetch_one(pool).await?;
        let money=sqlx::query(r#"SELECT
          COALESCE(SUM(filled_volume) FILTER(WHERE NOT is_closing),0)::bigint AS opened,
          COALESCE(SUM(filled_volume) FILTER(WHERE is_closing),0)::bigint AS closed,
          COALESCE(BOOL_AND(deal_status IN ('FILLED','PARTIALLY_FILLED')),false) AS valid,
          COALESCE(BOOL_AND(NOT is_closing OR COALESCE((close_gross_profit IS NOT NULL
            AND close_swap IS NOT NULL AND close_commission IS NOT NULL
            AND close_pnl_conversion_fee IS NOT NULL AND COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18),false)),false) AS priced,
          SUM(CASE WHEN is_closing AND COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18
            THEN (close_gross_profit::float8+close_swap::float8-abs(close_commission::float8)
            -abs(close_pnl_conversion_fee::float8))/power(10::float8,COALESCE(close_money_digits,money_digits)) END) AS net,
          COALESCE(BOOL_OR(deal_id=$3 AND is_closing),false) AS exit_present,
          min(executed_at) AS first_deal_at,max(executed_at) AS last_deal_at
          FROM deals WHERE account_id=$1 AND position_id=$2 AND filled_volume>0
            AND executed_at<=$4 AND fetched_at<=$4"#)
            .bind(account).bind(position).bind(deal).bind(now).fetch_one(pool).await?;
        // `complete` means sealed for history backfill. A successful recent
        // poll also fetched every page; it deliberately leaves that flag false.
        // Require an error-free, coherent, already-observed receipt covering
        // this position. Balanced volumes alone never prove history coverage.
        let first = money
            .get::<Option<DateTime<Utc>>, _>("first_deal_at")
            .map_or(created, |at| created.min(at));
        let last = money
            .get::<Option<DateTime<Utc>>, _>("last_deal_at")
            .map_or(closed, |at| closed.max(at));
        let coverage: bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM deal_fetches WHERE account_id=$1 AND status='ok' AND error IS NULL AND window_from<=$2 AND window_to>=$3 AND window_from<=window_to AND fetched_at>=window_to AND fetched_at>=$3 AND window_to<=$4 AND fetched_at<=$4)")
            .bind(account).bind(first).bind(last).bind(now).fetch_one(pool).await?;
        let opened: i64 = money.get("opened");
        let finished = opened > 0
            && opened >= receipts
            && opened == money.get::<i64, _>("closed")
            && money.get::<bool, _>("valid")
            && money.get::<bool, _>("exit_present")
            && row.get::<i64, _>("owner_count") == 1;
        let priced: bool = money.get("priced");
        let costs = finished && priced && coverage;
        let requested: Option<i64> = row.get("volume");
        let net: Option<f64> = money.get("net");
        let net_r = if costs {
            risk.zip(requested).zip(net).and_then(|((r, v), n)| {
                if r > 0.0 && r.is_finite() && v > 0 && opened <= v {
                    Some(n / (r * opened as f64 / v as f64)).filter(|r| r.is_finite())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let mut missing = Vec::new();
        for (check, code) in [
            ("gate_risk_recorded", "gate_risk"),
            (
                "preregistration_matches_current_reference",
                "registered_reference",
            ),
            ("entry_direction_valid", "entry_direction"),
            ("entry_trigger_valid", "entry_trigger"),
            ("original_stop_geometry_valid", "original_stop_geometry"),
            ("original_target_geometry_valid", "original_target_geometry"),
            ("original_session_valid", "original_session"),
            ("original_band_valid", "original_band"),
        ] {
            if checks[check].is_null() {
                missing.push(code)
            }
        }
        let setup_complete =
            direction.is_some() && trigger.is_some() && stop.is_some() && target.is_some();
        if !setup_complete {
            missing.push("saved_entry_setup")
        }
        if !coverage {
            missing.push("broker_history_coverage")
        }
        if !finished {
            missing.push("complete_position_deals")
        }
        if !costs {
            missing.push("complete_net_costs")
        }
        let symbol: String = row.get("symbol");
        // Symbols are identifiers, never an escape hatch for saved free text.
        let symbol = if symbol.len() <= 32
            && symbol
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c))
        {
            Some(symbol)
        } else {
            None
        };
        cases.push(json!({"case_id":format!("{strategy}:{account}:{deal}"),"reason":"stop_loss",
            "account_id":account,"strategy":strategy,"deal_id":deal,"position_id":position,
            "symbol_id":row.get::<Option<i64>,_>("symbol_id"),"symbol":symbol,"closed_at":closed,
            "environment":row.get::<String,_>("environment"),"ownership_verified":true,
            "reason_evidence":"recorded_inference","net_costs_complete":costs,"net_r":net_r,
            "original_entry":{"recorded_at":created,"side":side,"setup_complete":setup_complete,
                "rule_registration":if strategy=="s1"{"docs/preregistration-2026-10-24.md"}else{"docs/preregistration-2026-10-25-strategy3.md"}},
            "checks":checks,"missing_evidence":missing}));
    }
    Ok(json!({"schema_version":1,"as_of":now,"cases":cases}))
}

/// A later complete owned close after a stopped position on the same UTC
/// date, symbol and registration. This measures subsequent activity, not a
/// quote-price recovery or a profitable result. Incomplete history is excluded.
pub async fn recovery(pool: &PgPool, accounts: &[i64]) -> Result<Value, sqlx::Error> {
    let rows = crate::strategy3::recent_closed_signals(pool, accounts).await?;
    let mut stops = 0;
    let mut later = 0;
    for stop in rows.iter().filter(|r| r.get::<bool, _>("stopped")) {
        stops += 1;
        let account: i64 = stop.get("account_id");
        let symbol: String = stop.get("symbol");
        let hash: String = stop.get("prereg_hash");
        let at: DateTime<Utc> = stop.get("closed_at");
        if rows.iter().any(|r| {
            let closed: DateTime<Utc> = r.get("closed_at");
            r.get::<i64, _>("account_id") == account
                && r.get::<String, _>("symbol") == symbol
                && r.get::<String, _>("prereg_hash") == hash
                && closed > at
                && closed.date_naive() == at.date_naive()
                && !r.get::<bool, _>("stopped")
        }) {
            later += 1;
        }
    }
    let score = (stops > 0).then(|| later as f64 / stops as f64);
    Ok(json!({"available":true,
        "path":"later complete owned close on the same UTC date and registration; not a quote path or profitable-recovery rate",
        "scope":"stored complete-volume positions over 14 days; pending history excluded; not financial acceptance",
        "stops":stops,"same_session_later_close":later,"score":score,
        "threshold_crossed":score.is_some_and(|s| stops>=5 && s<0.5)}))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed(pool: &PgPool, account: i64, position: i64, owned: bool, reason: &str) {
        sqlx::query("INSERT INTO accounts(account_id,environment,login_missing_reason,currency,role_missing_reason,enabled) VALUES($1,'demo','unread','JPY','unread',true) ON CONFLICT DO NOTHING")
            .bind(account).execute(pool).await.unwrap();
        let key = format!("s1:{account}:{position}");
        let opened = Utc::now() - Duration::hours(2);
        if owned {
            sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk,created_at) VALUES($1,$2,$2,'open',1,'buy',100,300000,10,11,$3)")
                .bind(account).bind(&key).bind(opened).execute(pool).await.unwrap();
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,deal_id,symbol_id,volume,received_at,raw) VALUES($1,$2,2126,'ORDER_FILLED',$3,$4,1,100,$5,''::bytea)")
                .bind(account).bind(&key).bind(position).bind(position*10).bind(opened).execute(pool).await.unwrap();
            sqlx::query("INSERT INTO trading_decisions(account_id,kind,reason,detail) VALUES($1,'sent','private prompt money BALANCE_SECRET',$2)")
                .bind(account).bind(json!({"entry_key":key,"ema_fast":20,"ema_slow":10,"level":100,"ask":101,"bid":100,"atr":1,"balance":"BALANCE_SECRET"})).execute(pool).await.unwrap();
            sqlx::query("INSERT INTO strategy1_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,detail) VALUES($1,$2,'jiaoyi-s1',$3,'open','XAUUSD',$4,'filled',$5,'{}')")
                .bind(account).bind(&key).bind(crate::edge::prereg::hash()).bind(position).bind(opened).execute(pool).await.unwrap();
        }
        exit(
            pool,
            account,
            position,
            &format!("exit:{account}:{position}"),
            reason,
        )
        .await;
    }

    async fn exit(pool: &PgPool, account: i64, position: i64, key: &str, reason: &str) {
        sqlx::query("INSERT INTO strategy1_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,detail) VALUES($1,$2,'jiaoyi-s1','private','broker_exit','XAUUSD',$3,'filled',$4,$5)")
            .bind(account).bind(key).bind(position).bind(Utc::now()-Duration::minutes(1))
            .bind(json!({"reason":reason,"deal_id":position*10+1,"raw":"BALANCE_SECRET","commission":500000})).execute(pool).await.unwrap();
    }

    async fn history_deals(pool: &PgPool, account: i64, position: i64, closed: i64, costs: bool) {
        for (closing, volume, id, gross) in [
            (false, 100, position * 10, 0),
            (true, closed, position * 10 + 1, -900),
        ] {
            sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,is_closing,close_gross_profit,close_swap,close_commission,close_pnl_conversion_fee,close_money_digits,raw,fetched_at) VALUES($1,$2,$2,$3,1,$4,$4,'BUY','FILLED',now()-interval '2 hours',CASE WHEN $5 THEN now()-interval '1 minute' ELSE now()-interval '2 hours' END,$5,$6,0,-100,0,$7,''::bytea,now())")
                .bind(account).bind(id).bind(position).bind(volume).bind(closing).bind(gross)
                .bind(costs.then_some(2_i32)).execute(pool).await.unwrap();
        }
    }

    async fn history(pool: &PgPool, account: i64, position: i64, closed: i64, costs: bool) {
        history_deals(pool, account, position, closed, costs).await;
        sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,status,complete) VALUES($1,now()-interval '3 hours',now(),'ok',true)")
            .bind(account).execute(pool).await.unwrap();
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn ownership_allowlist_reason_dedupe_and_saved_setup(pool: PgPool) {
        seed(&pool, 1, 1, true, "stop_loss").await;
        seed(&pool, 1, 2, false, "stop_loss").await;
        seed(&pool, 1, 3, true, "history_reconciled").await;
        seed(&pool, 2, 4, true, "stop_loss").await;
        exit(&pool, 1, 1, "duplicate", "stop_loss").await;
        let result = evidence(&pool, &[1]).await.unwrap();
        assert_eq!(result["cases"].as_array().unwrap().len(), 1);
        let c = &result["cases"][0];
        assert_eq!(c["checks"]["entry_direction_valid"], true);
        assert_eq!(c["checks"]["entry_trigger_valid"], true);
        assert_eq!(c["checks"]["original_stop_geometry_valid"], true);
        assert_eq!(
            c["checks"]["preregistration_matches_current_reference"],
            true
        );
        assert!(c["checks"]["original_session_valid"].is_null());
        assert_eq!(c["reason_evidence"], "recorded_inference");
        assert!(c["net_r"].is_null());
        assert_eq!(evidence(&pool, &[]).await.unwrap()["cases"], json!([]));
        let public = result.to_string();
        for private in [
            "BALANCE_SECRET",
            "commission",
            "balance",
            "gross",
            "risk\":10",
            "volume",
            "ema_fast",
            "ask\"",
            "JPY",
        ] {
            assert!(!public.contains(private), "leaked {private}");
        }
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn successful_live_history_is_coverage_even_when_the_window_is_unsealed(pool: PgPool) {
        seed(&pool, 1, 1, true, "stop_loss").await;
        history_deals(&pool, 1, 1, 100, true).await;
        // The real recent-history poll has fetched every page successfully,
        // but leaves complete=false because the rolling window is not sealed.
        sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete) VALUES(1,now()-interval '3 hours',now(),now(),'ok',false)")
            .execute(&pool).await.unwrap();
        let result = evidence(&pool, &[1]).await.unwrap();
        let case = &result["cases"][0];
        assert_eq!(case["net_costs_complete"], true);
        assert_eq!(case["net_r"], -1.0);
        assert!(!case["missing_evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("broker_history_coverage")));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn history_receipts_must_be_account_scoped_error_free_and_observed(pool: PgPool) {
        let cutoff = Utc::now();
        for account in 1..=7 {
            seed(&pool, account, account, true, "stop_loss").await;
            history_deals(&pool, account, account, 100, true).await;
            let mut from = cutoff - Duration::hours(3);
            let mut to = cutoff - Duration::seconds(10);
            let mut fetched = cutoff - Duration::seconds(5);
            let mut status = "ok";
            let mut error: Option<&str> = None;
            let mut receipt_account = account;
            match account {
                1 => {
                    status = "failed";
                    error = Some("reported source error");
                }
                2 => {
                    to = cutoff - Duration::minutes(2);
                    fetched = cutoff - Duration::minutes(1);
                }
                3 => {
                    to = cutoff + Duration::days(1);
                    fetched = to;
                }
                4 => {
                    from = cutoff;
                }
                5 => {
                    fetched = cutoff - Duration::minutes(2);
                }
                6 => {
                    receipt_account = 999;
                }
                _ => {}
            }
            sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete,error) VALUES($1,$2,$3,$4,$5,false,$6)")
                .bind(receipt_account).bind(from).bind(to).bind(fetched).bind(status).bind(error)
                .execute(&pool).await.unwrap();
        }
        let result = evidence(&pool, &[1, 2, 3, 4, 5, 6, 7]).await.unwrap();
        for case in result["cases"].as_array().unwrap() {
            assert_eq!(case["net_costs_complete"], case["account_id"] == 7);
            if case["account_id"] != 7 {
                assert!(case["net_r"].is_null());
            }
        }
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn r_requires_explicit_cost_digits_complete_position_and_coverage(pool: PgPool) {
        for p in 1..=3 {
            seed(&pool, 1, p, true, "stop_loss").await;
        }
        history(&pool, 1, 1, 100, true).await;
        history(&pool, 1, 2, 50, true).await;
        history(&pool, 1, 3, 100, false).await;
        let result = evidence(&pool, &[1]).await.unwrap();
        let cases = result["cases"].as_array().unwrap();
        for c in cases {
            match c["position_id"].as_i64().unwrap() {
                1 => {
                    assert_eq!(c["net_r"], -1.0);
                    assert_eq!(c["net_costs_complete"], true);
                }
                _ => {
                    assert!(c["net_r"].is_null());
                    assert_eq!(c["net_costs_complete"], false);
                }
            }
        }
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn recent_window_excludes_old_cases_and_missing_saved_setup_stays_unknown(pool: PgPool) {
        seed(&pool, 1, 1, true, "stop_loss").await;
        sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,max_risk) VALUES(1,'s1:no-setup','s1:no-setup','open',1,'sell',100,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,volume,received_at,raw) VALUES(1,'s1:no-setup',2126,'ORDER_FILLED',2,100,now(),''::bytea)")
            .execute(&pool).await.unwrap();
        exit(&pool, 1, 2, "no-setup-exit", "stop_loss").await;
        sqlx::query("INSERT INTO strategy1_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,detail) VALUES(1,'old','jiaoyi-s1','test','broker_exit','XAUUSD',1,'filled',now()-interval '25 hours','{\"reason\":\"stop_loss\",\"deal_id\":999}')")
            .execute(&pool).await.unwrap();
        let result = evidence(&pool, &[1]).await.unwrap();
        assert_eq!(result["cases"].as_array().unwrap().len(), 2);
        let unknown = result["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["position_id"] == 2)
            .unwrap();
        assert_eq!(unknown["original_entry"]["setup_complete"], false);
        assert!(unknown["checks"]["entry_direction_valid"].is_null());
        assert!(unknown["missing_evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("saved_entry_setup")));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn strategy3_real_serializers_preserve_setup_and_exact_trigger_rules(pool: PgPool) {
        use jiaoyi_core::strategy3::{band::Band, cost_reserve, targets::Dir};
        use jiaoyi_ctrader::trading::VolumeLimits;
        use jiaoyi_store::strategy3::{record_order, OrderRow};
        sqlx::query("INSERT INTO accounts(account_id,environment,login_missing_reason,currency,role_missing_reason,enabled) VALUES(1,'demo','unread','USD','unread',true)").execute(&pool).await.unwrap();
        let b = Band {
            open: 100.0,
            sigma: 0.01,
            lower: 99.0,
            upper: 101.0,
            half_width: 1.0,
            minutes: 60.0,
        };
        let limits = VolumeLimits {
            min: 100,
            step: 100,
            max: None,
            digits: 2,
        };
        for (position, bid, ask) in [(1, 101.0, 102.0), (2, 100.0, 101.0), (3, 98.0, 102.0)] {
            let now = Utc::now() - Duration::minutes(2);
            let entry = crate::strategy3::Entry {
                side: Dir::Buy,
                band: b,
                entry_price: ask,
                relative_stop: crate::trading::relative_stop(b.half_width, limits.digits),
                r: 1.0,
                cost: cost_reserve(0.1, 0.1, 1.0, ask).unwrap(),
            };
            let plan = crate::strategy3::plan_order(&entry, &limits, 100).unwrap();
            // This is the same struct serialization and persistence used by
            // strategy3::on_spot/S3Ledger, including the nested target plan.
            let detail = json!({"session_date":now.date_naive(),"decided_at":now,
                "bid":bid,"ask":ask,"side":Dir::Buy,"band":b,"open":b.open,"sigma":b.sigma,
                "entry":entry,"order":plan,
                "session":jiaoyi_core::strategy3::sessions::Session {
                    date:now.date_naive(), open:now-Duration::hours(1),
                    close:now+Duration::hours(1),lunch:None },
                "volume_limits":limits});
            let key = format!("s3:1:1:case:{position}");
            sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk,created_at) VALUES(1,$1,$1,'open',1,'buy',100,100000,10,11,$2)")
                .bind(&key).bind(now).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,deal_id,symbol_id,volume,received_at,raw) VALUES(1,$1,2126,'ORDER_FILLED',$2,$3,1,100,$4,''::bytea)")
                .bind(&key).bind(position).bind(position*10).bind(now).execute(&pool).await.unwrap();
            let open = OrderRow {
                account_id: 1,
                entry_key: key,
                label: crate::strategy3::LABEL.into(),
                prereg_hash: crate::strategy3::prereg_hash(),
                kind: "open".into(),
                symbol: "XAUUSD".into(),
                symbol_id: 1,
                position_id: None,
                side: Some("buy".into()),
                session_date: Some(now.date_naive()),
                decided_at: now,
                bid: Some(bid),
                ask: Some(ask),
                volume: Some(100),
                detail,
            };
            record_order(&pool, &open).await.unwrap();
            let exit = OrderRow {
                entry_key: format!("s3-broker:{position}"),
                kind: "broker_exit".into(),
                position_id: Some(position),
                detail: json!({"reason":"stop_loss","deal_id":position*10+1}),
                ..open
            };
            record_order(&pool, &exit).await.unwrap();
            sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,outcome,sent_at,detail) VALUES(1,$1,'jiaoyi-s3','test','broker_exit','XAUUSD',$2,'filled',now()-interval '1 minute',$3)")
                .bind(&exit.entry_key).bind(position).bind(&exit.detail).execute(&pool).await.unwrap();
        }
        let result = evidence(&pool, &[1]).await.unwrap();
        assert_eq!(result["cases"].as_array().unwrap().len(), 3);
        for c in result["cases"].as_array().unwrap() {
            assert_eq!(c["checks"]["entry_direction_valid"], true);
            assert_eq!(c["checks"]["original_stop_geometry_valid"], true);
            assert_eq!(c["checks"]["original_target_geometry_valid"], true);
            assert_eq!(c["checks"]["original_band_valid"], true);
            assert_eq!(
                c["checks"]["preregistration_matches_current_reference"],
                true
            );
            assert_eq!(c["checks"]["entry_trigger_valid"], c["position_id"] == 1);
            assert_eq!(c["checks"]["original_session_valid"], true);
        }
    }
}
