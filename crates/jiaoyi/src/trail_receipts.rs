//! Bounded, read-only evidence of recorded S3 trailing amendments.
//! This reconstructs the saved request/acknowledgement; it never sends a frame.
use chrono::{DateTime, Utc};
use jiaoyi_ctrader::messages::{ProtoOaExecutionType, ProtoOaPosition};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use prost::Message;
use serde::Serialize;
use serde_json::Value;

// Read only the existing ExecutionEvent fields needed for this projection.
// Field numbers come from OpenApiMessages.proto; unknown fields are ignored.
#[derive(Clone, PartialEq, Message)]
struct RecordedExecution {
    #[prost(int32, optional, tag = "1")]
    payload_type: Option<i32>,
    #[prost(int64, required, tag = "2")]
    account: i64,
    #[prost(int32, required, tag = "3")]
    execution_type: i32,
    #[prost(message, optional, tag = "4")]
    position: Option<ProtoOaPosition>,
    #[prost(string, optional, tag = "9")]
    error: Option<String>,
}

const QUERY: &str = r#"
WITH recent AS (
 SELECT o.*,row_number() OVER(PARTITION BY account_id ORDER BY decided_at DESC,id DESC) AS rank
 FROM strategy3_orders o WHERE o.account_id=ANY($1) AND o.kind='trail' AND o.label='jiaoyi-s3'
 AND o.decided_at>=now()-interval '14 days' AND o.decided_at<=now()
)
SELECT t.account_id,t.entry_key,t.position_id,t.symbol,t.symbol_id,t.side,t.decided_at,
 t.detail AS trail_detail,p.entry_key AS parent_key,p.prereg_hash AS parent_hash,
 p.detail AS parent_detail,s.digits,
 i.created_at AS intent_at,i.stop_price AS requested_stop,i.take_profit AS intent_tp,
 (p.entry_key IS NOT NULL AND EXISTS(
   SELECT 1 FROM trading_intents oi JOIN trading_events oe
     ON oe.account_id=oi.account_id AND oe.entry_key=oi.entry_key
   WHERE oi.account_id=t.account_id AND oi.entry_key=p.entry_key AND oi.kind='open'
     AND oi.symbol_id=t.symbol_id AND oe.position_id=t.position_id AND oe.volume>0
     AND oe.received_at<=i.created_at
     AND oe.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 ) AND (SELECT count(DISTINCT oi.entry_key) FROM trading_intents oi JOIN trading_events oe
   ON oe.account_id=oi.account_id AND oe.entry_key=oi.entry_key
   WHERE oi.account_id=t.account_id AND oi.kind='open' AND oi.entry_key LIKE 's3:%'
     AND oe.position_id=t.position_id AND oe.volume>0
     AND oe.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL'))=1) AS owned_parent,
 e.received_at AS acknowledged_at,e.stop_loss AS saved_broker_stop,e.raw,e.matches
FROM recent t
LEFT JOIN trading_intents i ON i.account_id=t.account_id AND i.entry_key=t.entry_key
 AND i.kind='amend_stop' AND i.position_id=t.position_id AND i.side=t.side
LEFT JOIN strategy3_orders p ON p.account_id=t.account_id AND p.entry_key=t.detail->>'open_entry_key'
 AND p.kind='open' AND p.label='jiaoyi-s3' AND p.symbol_id=t.symbol_id AND p.side=t.side
 AND p.decided_at<=t.decided_at
LEFT JOIN symbols s ON s.account_id=t.account_id AND s.broker_symbol_id=t.symbol_id
LEFT JOIN LATERAL (
 SELECT e.received_at,e.stop_loss,e.raw,count(*) OVER() AS matches
 FROM trading_events e WHERE e.account_id=t.account_id AND e.client_msg_id=i.client_msg_id
 AND e.position_id=t.position_id AND e.kind='ORDER_REPLACED'
 AND e.payload_type=2126 AND e.error_code IS NULL AND e.received_at>=i.created_at
 AND e.received_at<=now() AND (e.entry_key IS NULL OR e.entry_key=t.entry_key)
 ORDER BY e.id DESC LIMIT 1
) e ON true
WHERE t.rank<=5 ORDER BY t.account_id,t.decided_at DESC,t.entry_key
"#;

#[derive(Debug, Serialize)]
pub struct Receipt {
    account_id: i64,
    symbol: Option<String>,
    position_id: Option<i64>,
    entry_key: Option<String>,
    open_entry_key: Option<String>,
    parent_hash: Option<String>,
    side: Option<String>,
    decided_at: DateTime<Utc>,
    intent_at: Option<DateTime<Utc>>,
    acknowledged_at: Option<DateTime<Utc>>,
    original_r: Option<f64>,
    reported_r: Option<f64>,
    from_stop: Option<f64>,
    best: Option<f64>,
    requested_stop: Option<f64>,
    broker_stop: Option<f64>,
    intended_take_profit: Option<f64>,
    broker_take_profit: Option<f64>,
    state: &'static str,
    checks: Checks,
    reasons: Vec<&'static str>,
}

#[derive(Debug, Default, Serialize)]
struct Checks {
    owned_parent: bool,
    precision_known: bool,
    original_r_matches: Option<bool>,
    request_matches_plan: Option<bool>,
    request_uses_original_r: Option<bool>,
    stop_improves: Option<bool>,
    broker_acknowledged: bool,
    broker_stop_matches: Option<bool>,
    take_profit_preserved: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    as_of: DateTime<Utc>,
    scope: &'static str,
    per_account_limit: usize,
    lookback_days: usize,
    receipts: Vec<Receipt>,
    note: &'static str,
}

fn number(v: &Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64().filter(|v| v.is_finite() && *v > 0.0)
}
fn finite_positive(v: Option<f64>) -> Option<f64> {
    v.filter(|v| v.is_finite() && *v > 0.0)
}
fn key(value: Option<String>, prefix: &str, account: i64) -> Option<String> {
    value.filter(|v| {
        v.len() <= 160
            && v.starts_with(&format!("{prefix}:{account}:"))
            && v[prefix.len() + 1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b':' || b == b'-')
    })
}
fn same(a: Option<f64>, b: Option<f64>, tick: Option<f64>) -> Option<bool> {
    Some((a? - b?).abs() <= tick? * 1.01)
}

/// Reconstruct only typed facts from existing ledgers for allowlisted accounts.
/// Missing/ambiguous evidence never becomes a passed receipt.
pub async fn report(pool: &PgPool, allowed: &[i64]) -> Result<Report, sqlx::Error> {
    let rows = sqlx::query(QUERY).bind(allowed).fetch_all(pool).await?;
    let mut receipts = Vec::new();
    for row in rows {
        let account: i64 = row.get("account_id");
        let position: Option<i64> = row.get("position_id");
        let symbol_id: i64 = row.get("symbol_id");
        let detail: Value = row.get("trail_detail");
        let parent: Option<Value> = row.get("parent_detail");
        let side: Option<String> = row
            .get::<Option<String>, _>("side")
            .filter(|s| matches!(s.as_str(), "buy" | "sell"));
        let digits: Option<i32> = row.get("digits");
        let tick = digits
            .filter(|d| (0..=10).contains(d))
            .map(|d| 10f64.powi(-d));
        let original_r = parent.as_ref().and_then(|p| number(&p["entry"], "r"));
        let reported_r = number(&detail, "r");
        let best = number(&detail, "best");
        let from_stop = number(&detail, "from_stop");
        let planned_stop = number(&detail, "new_stop");
        let requested = finite_positive(row.get("requested_stop"));
        let intended_tp = finite_positive(row.get("intent_tp"));
        let intent_at: Option<DateTime<Utc>> = row.get("intent_at");
        let ack_at: Option<DateTime<Utc>> = row.get("acknowledged_at");
        let trail_key = key(Some(row.get("entry_key")), "s3-trail", account);
        let parent_key = key(row.get("parent_key"), "s3", account);
        let parent_hash = row
            .get::<Option<String>, _>("parent_hash")
            .filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()));
        let mut check = Checks {
            owned_parent: row.get::<bool, _>("owned_parent")
                && trail_key.is_some()
                && parent_key.is_some()
                && parent_hash.is_some(),
            precision_known: tick.is_some(),
            ..Default::default()
        };
        let raw: Option<Vec<u8>> = row.get("raw");
        let decoded = raw
            .as_deref()
            .and_then(|v| RecordedExecution::decode(v).ok());
        let broker_position = decoded
            .as_ref()
            .filter(|e| {
                e.account == account
                    && e.payload_type.is_none_or(|p| p == 2126)
                    && e.execution_type == ProtoOaExecutionType::OrderReplaced as i32
                    && e.error.is_none()
            })
            .and_then(|e| e.position.as_ref())
            .filter(|p| {
                Some(p.position_id) == position
                    && p.trade_data.symbol_id == symbol_id
                    && Some(if p.trade_data.trade_side == 1 {
                        "buy"
                    } else if p.trade_data.trade_side == 2 {
                        "sell"
                    } else {
                        "unknown"
                    }) == side.as_deref()
            });
        let broker_stop = broker_position.and_then(|p| finite_positive(p.stop_loss));
        let broker_tp = broker_position.and_then(|p| finite_positive(p.take_profit));
        let saved_stop = finite_positive(row.get("saved_broker_stop"));
        let matches: Option<i64> = row.get("matches");
        check.broker_acknowledged =
            broker_position.is_some() && matches == Some(1) && ack_at.is_some();
        check.original_r_matches = same(original_r, reported_r, tick);
        check.request_matches_plan = same(requested, planned_stop, tick);
        let expected = best
            .zip(original_r)
            .zip(side.as_deref())
            .map(|((b, r), s)| if s == "buy" { b - r } else { b + r });
        check.request_uses_original_r = same(requested, expected, tick);
        check.stop_improves = requested
            .zip(from_stop)
            .zip(side.as_deref())
            .map(|((s, old), side)| if side == "buy" { s > old } else { s < old });
        check.broker_stop_matches = same(broker_stop, requested, tick)
            .zip(same(broker_stop, saved_stop, tick))
            .map(|(a, b)| a && b);
        check.take_profit_preserved = if check.broker_acknowledged {
            if row.get::<Option<f64>, _>("intent_tp").is_none()
                && broker_position.is_some_and(|p| p.take_profit.is_none())
            {
                Some(true)
            } else {
                same(intended_tp, broker_tp, tick)
            }
        } else {
            None
        };
        let mut reasons = Vec::new();
        if !check.owned_parent {
            reasons.push("unique_owned_opening_parent_unverified");
        }
        if !check.precision_known {
            reasons.push("symbol_precision_unavailable");
        }
        if !check.broker_acknowledged {
            reasons.push("unique_correlated_broker_replacement_unverified");
        }
        let mut refused = false;
        for (name, value) in [
            ("original_r_matches", check.original_r_matches),
            ("request_matches_plan", check.request_matches_plan),
            ("request_uses_original_r", check.request_uses_original_r),
            ("stop_improves", check.stop_improves),
            ("broker_stop_matches", check.broker_stop_matches),
            ("take_profit_preserved", check.take_profit_preserved),
        ] {
            match value {
                Some(true) => {}
                Some(false) => {
                    refused = true;
                    reasons.push(name);
                }
                None => reasons.push(name),
            }
        }
        let state = if refused {
            "refused"
        } else if reasons.is_empty() {
            "passed"
        } else {
            "unknown"
        };
        receipts.push(Receipt {
            account_id: account,
            symbol: Some(row.get::<String, _>("symbol")).filter(|s| {
                s.len() <= 64
                    && s.bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"._-/".contains(&c))
            }),
            position_id: position.filter(|p| *p > 0),
            entry_key: trail_key,
            open_entry_key: parent_key,
            parent_hash,
            side,
            decided_at: row.get("decided_at"),
            intent_at,
            acknowledged_at: ack_at,
            original_r,
            reported_r,
            from_stop,
            best,
            requested_stop: requested,
            broker_stop,
            intended_take_profit: intended_tp,
            broker_take_profit: broker_tp,
            state,
            checks: check,
            reasons,
        });
    }
    Ok(Report {
        as_of:Utc::now(), scope:"recorded S3 trail request and broker acknowledgement, not current protection or profitability",
        per_account_limit:5, lookback_days:14, receipts,
        note:"passed verifies saved original-R arithmetic and the correlated replacement/TP receipt; it does not establish a price-path maximum, restart behavior or current broker state",
    })
}
