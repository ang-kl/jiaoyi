//! №18,377 offline adapter for the owner-authorised production revision.
//! JSON on stdin, JSON on stdout. No broker, database, dispatch or live settings.
//! This tests one arm; it does not certify receipt provenance or account risk.
use chrono::{DateTime, Duration, NaiveDate, Utc};
use jiaoyi_core::{
    seal,
    strategy3::{band, cost_reserve, macro_pullback as mp, sessions},
    universe::AssetClass,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bar {
    open_time: DateTime<Utc>,
    open: f64,
    close: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    symbol: String,
    date: NaiveDate,
    opening_reference: f64,
    digits: i32,
    costs: mp::ReplayCosts,
    preparation_m30: Vec<Bar>,
    impulse: mp::ReplayQuote,
    quotes: Vec<mp::ReplayQuote>,
}
fn protected(from: DateTime<Utc>, to: DateTime<Utc>) -> bool {
    let start: DateTime<Utc> = "2025-08-19T00:00:00Z".parse().unwrap();
    let end: DateTime<Utc> = "2025-09-06T00:00:00Z".parse().unwrap();
    from < end && to > start
}
fn run(r: Request) -> Result<Value, String> {
    let class = match r.symbol.as_str() {
        "SpotCrude" | "SpotBrent" => AssetClass::Energy,
        "US500" | "NAS100" => AssetClass::EquityIndex,
        "XAUUSD" | "XAGUSD" => AssetClass::Metal,
        "BTCUSD" | "ETHUSD" => AssetClass::Crypto,
        _ => return Err("symbol_retired_from_s3_core".into()),
    };
    let rule = sessions::rule_for(&r.symbol, class).ok_or("native calendar unavailable")?;
    let session = sessions::session_on(&rule, r.date).ok_or("no native session")?;
    if session.close + Duration::seconds(6) > seal::seal_instant()
        || protected(session.open, session.close + Duration::seconds(6))
        || r.preparation_m30.len() > 2300
        || r.quotes.len() > 2_000_000
    {
        return Err("sealed, protected or unbounded replay".into());
    }
    let earliest = session.open - Duration::days(45);
    let mut bars = Vec::with_capacity(r.preparation_m30.len());
    for b in r.preparation_m30 {
        let end = b.open_time + Duration::minutes(30);
        if b.open_time < earliest || end > session.open || protected(b.open_time, end) {
            return Err("noncausal, protected or out-of-preparation dependency".into());
        }
        bars.push(band::M30 {
            open_time: b.open_time,
            open: b.open,
            close: b.close,
        });
    }
    let context = mp::context(&rule, session, &bars, session.open)?;
    let table = band::sigma_table(&rule, &session, &bars);
    let mark = band::decision_mark(&session, r.impulse.at).ok_or("outside_impulse_mark")?;
    let b = band::band_at(&table, &session, r.opening_reference, mark)?;
    for at in [r.impulse.bid_at, r.impulse.ask_at] {
        if at > r.impulse.at || r.impulse.at - at > Duration::seconds(6) {
            return Err("impulse quote is future or stale".into());
        }
    }
    let mut peer_move = None;
    if let Some(p) = r.impulse.peer {
        if p.bid_at > r.impulse.at || p.ask_at > r.impulse.at {
            return Err("future peer".into());
        }
        if r.impulse.at - p.bid_at <= Duration::seconds(6)
            && r.impulse.at - p.ask_at <= Duration::seconds(6)
        {
            if ![p.origin, p.bid, p.ask]
                .into_iter()
                .all(|v| v.is_finite() && v > 0.0)
                || p.bid > p.ask
            {
                return Err("invalid peer".into());
            }
            peer_move = Some(p.bid + (p.ask - p.bid) / 2.0 - p.origin);
        }
    }
    let impulse_mid = r.impulse.bid + (r.impulse.ask - r.impulse.bid) / 2.0;
    let cost = cost_reserve(
        r.costs.prepared_spread,
        r.costs.commission_round_trip,
        r.costs.slippage_bps_per_side,
        if impulse_mid > r.opening_reference {
            r.impulse.ask
        } else {
            r.impulse.bid
        },
    )?
    .total;
    match mp::ArmedPullback::arm(
        &r.symbol,
        session,
        b,
        r.impulse.bid,
        r.impulse.ask,
        r.impulse.at,
        context,
        peer_move,
        cost,
    ) {
        Ok(armed) => Ok(
            json!({"policy":mp::POLICY,"new_broker_requests":0,"receipt_provenance_verified":false,"validation_opened":false,"armed":armed,"result":mp::replay_armed(armed,r.digits,r.costs,&r.quotes)?}),
        ),
        Err(reason) => Ok(
            json!({"policy":mp::POLICY,"new_broker_requests":0,"receipt_provenance_verified":false,"validation_opened":false,"status":reason,"financial_pass":false}),
        ),
    }
}
fn main() {
    let result = (|| -> Result<Value, String> {
        let mut input = String::new();
        std::io::stdin()
            .take(300_000_001)
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > 300_000_000 {
            return Err("input byte limit".into());
        }
        let request = serde_json::from_str(&input).map_err(|e| format!("invalid request: {e}"))?;
        run(request)
    })();
    match result {
        Ok(v) => println!("{}", serde_json::to_string(&v).unwrap()),
        Err(e) => {
            eprintln!("{}", json!({"error":e,"financial_pass":false}));
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(date: &str) -> Request {
        let date: NaiveDate = date.parse().unwrap();
        let s = sessions::session_on(&sessions::SessionRule::UtcDay, date).unwrap();
        let q = |at, bid, ask| mp::ReplayQuote {
            at,
            bid,
            ask,
            bid_at: at,
            ask_at: at,
            peer: None,
        };
        let t = s.open + Duration::minutes(30);
        let preparation_m30 = (1..=45)
            .flat_map(|day| {
                (0..48).map(move |slot| Bar {
                    open_time: s.open - Duration::days(day) + Duration::minutes(slot * 30),
                    open: 100.0,
                    close: 100.4,
                })
            })
            .collect();
        Request {
            symbol: "BTCUSD".into(),
            date,
            opening_reference: 100.0,
            digits: 2,
            costs: mp::ReplayCosts {
                prepared_spread: 0.02,
                commission_round_trip: 0.0,
                slippage_bps_per_side: 24.0,
            },
            preparation_m30,
            impulse: q(t, 100.99, 101.01),
            quotes: vec![
                q(t, 100.99, 101.01),
                q(t + Duration::seconds(1), 100.49, 100.51),
                q(t + Duration::seconds(2), 101.53, 101.55),
                q(t + Duration::seconds(3), 102.0, 102.02),
            ],
        }
    }
    #[test]
    fn json_adapter_uses_native_preparation_and_rejects_all_protected_dependencies() {
        let result = run(request("2024-03-04")).unwrap();
        assert_eq!(result["result"]["status"], "strict_target_1_38r");
        assert_eq!(result["result"]["events"][0]["kind"], "pullback_entry");
        assert_eq!(result["receipt_provenance_verified"], false);
        assert_eq!(result["result"]["financial_pass"], false);
        assert!(run(request("2025-09-07"))
            .unwrap_err()
            .contains("protected"));
        assert!(run(request("2025-11-07")).unwrap_err().contains("sealed"));
        let mut r = request("2024-03-04");
        // Even a quote after the earlier target is checked before replay returns.
        let mut later = *r.quotes.last().unwrap();
        later.at += Duration::days(1);
        later.bid_at = later.at;
        later.ask_at = later.at;
        r.quotes.push(later);
        assert!(run(r).unwrap_err().contains("out-of-session"));
    }
}
