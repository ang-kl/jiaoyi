//! Read-only attribution from an existing final-guard receipt and broker deals.
//! No estimate is fabricated for historical orders without these records.
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntryCost {
    pub checked_at: DateTime<Utc>,
    pub decision_price: Option<f64>,
    pub dispatch_price: f64,
    pub dispatch_spread: f64,
    pub dispatch_quote_age_ms: i64,
    pub fill_price: f64,
    /// All three bp figures use the SAME dispatch-price denominator.
    pub decision_to_dispatch_bps: Option<f64>,
    pub dispatch_to_fill_bps: f64,
    pub decision_to_fill_bps: Option<f64>,
}

pub(super) fn attribute(
    raw: Option<&[u8]>,
    account: i64,
    symbol: Option<i64>,
    side: Option<&str>,
    decision: Option<f64>,
    broker_deal_fill: Option<f64>,
) -> Option<EntryCost> {
    let v: Value = serde_json::from_slice(raw?).ok()?;
    let side = side?;
    let sign = match side {
        "buy" => 1.0,
        "sell" => -1.0,
        _ => return None,
    };
    if v["policy"] != "entry-dispatch-v1-2026-11-01"
        || v["result"] != "passed"
        || v["stage"] != "after_writer_lock"
        || v["account_id"].as_i64() != Some(account)
        || v["symbol_id"].as_i64() != symbol
        || symbol.is_none()
        || v["selected_side"].as_str()? != side
    {
        return None;
    }
    let checked_at: DateTime<Utc> = v["checked_at"].as_str()?.parse().ok()?;
    let quote = |key: &str| -> Option<(f64, i64)> {
        let p = positive(v[key]["price"].as_f64())?;
        let received: DateTime<Utc> = v[key]["received_at"].as_str()?.parse().ok()?;
        let age = checked_at
            .signed_duration_since(received)
            .num_milliseconds();
        (received <= checked_at && (0..=6000).contains(&age)).then_some((p, age))
    };
    let (bid, bid_age) = quote("bid")?;
    let (ask, ask_age) = quote("ask")?;
    if bid > ask {
        return None;
    }
    let (dispatch_price, dispatch_quote_age_ms) = if side == "buy" {
        (ask, ask_age)
    } else {
        (bid, bid_age)
    };
    let fill_price = positive(broker_deal_fill)?;
    let decision_price = positive(decision);
    let bps = |movement: f64| sign * movement / dispatch_price * 10_000.0;
    Some(EntryCost {
        checked_at,
        decision_price,
        dispatch_price,
        dispatch_spread: ask - bid,
        dispatch_quote_age_ms,
        fill_price,
        decision_to_dispatch_bps: decision_price.map(|p| bps(dispatch_price - p)),
        dispatch_to_fill_bps: bps(fill_price - dispatch_price),
        decision_to_fill_bps: decision_price.map(|p| bps(fill_price - p)),
    })
}

fn positive(v: Option<f64>) -> Option<f64> {
    v.filter(|x| x.is_finite() && *x > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn receipt(side: &str) -> Value {
        json!({"policy":"entry-dispatch-v1-2026-11-01", "result":"passed",
            "stage":"after_writer_lock", "account_id":7, "symbol_id":9,
            "selected_side":side, "checked_at":"2026-11-05T09:00:01Z",
            "bid":{"price":100.0,"received_at":"2026-11-05T09:00:00Z"},
            "ask":{"price":101.0,"received_at":"2026-11-05T09:00:00Z"}})
    }
    fn check(v: &Value, side: &str, decision: Option<f64>, fill: Option<f64>) -> Option<EntryCost> {
        attribute(
            Some(&serde_json::to_vec(v).unwrap()),
            7,
            Some(9),
            Some(side),
            decision,
            fill,
        )
    }

    #[test]
    fn entry_cost_separates_drift_from_fill_for_both_sides() {
        for (side, decision, fill) in [("buy", 99.0, 100.5), ("sell", 102.0, 100.5)] {
            let a = check(&receipt(side), side, Some(decision), Some(fill)).unwrap();
            assert!(a.decision_to_dispatch_bps.unwrap() > 0.0);
            assert!(a.dispatch_to_fill_bps < 0.0); // favourable execution despite adverse total
            assert!(a.decision_to_fill_bps.unwrap() > 0.0);
            assert!(
                (a.decision_to_dispatch_bps.unwrap() + a.dispatch_to_fill_bps
                    - a.decision_to_fill_bps.unwrap())
                .abs()
                    < 1e-10
            );
            assert_eq!(a.dispatch_quote_age_ms, 1000);
            assert_eq!(a.dispatch_spread, 1.0);
        }
    }

    #[test]
    fn entry_cost_missing_evidence_is_unknown_not_zero() {
        let v = receipt("buy");
        assert!(attribute(None, 7, Some(9), Some("buy"), Some(100.0), Some(101.0)).is_none());
        assert!(check(&v, "buy", Some(100.0), None).is_none());
        assert!(check(&v, "buy", Some(100.0), Some(f64::NAN)).is_none());
        let a = check(&v, "buy", None, Some(101.0)).unwrap();
        assert_eq!(a.decision_to_dispatch_bps, None);
        assert_eq!(a.decision_to_fill_bps, None);
        assert_eq!(a.dispatch_to_fill_bps, 0.0);
    }

    #[test]
    fn entry_cost_rejects_mismatched_or_invalid_receipts() {
        let base = receipt("buy");
        for (path, value) in [
            (vec!["account_id"], json!(8)),
            (vec!["symbol_id"], json!(10)),
            (vec!["selected_side"], json!("sell")),
            (vec!["result"], json!("refused")),
            (vec!["policy"], json!("unknown")),
            (vec!["stage"], json!("before_writer_lock")),
            (vec!["bid", "price"], json!(102)),
            (vec!["ask", "received_at"], json!("2026-11-05T09:00:02Z")),
            (vec!["bid", "received_at"], json!("2026-11-05T08:59:54Z")),
        ] {
            let mut v = base.clone();
            let mut field = &mut v;
            for key in path {
                field = &mut field[key];
            }
            *field = value;
            assert!(check(&v, "buy", Some(100.0), Some(101.0)).is_none());
        }
    }
}
