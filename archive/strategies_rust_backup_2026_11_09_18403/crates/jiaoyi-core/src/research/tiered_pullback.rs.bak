//! №18,360: fixed offline opening-range policy; no live entry point or transport.
use super::{exit_replay as exits, manifest_pullback::Input, s3_ticks};
use chrono::Duration;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub tier: String,
    pub threshold_bps: f64,
    pub admission_bps: f64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            tier: "tier1".into(),
            threshold_bps: 48.0,
            admission_bps: 24.0,
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<(), String> {
        if !["tier1", "tier2"].contains(&self.tier.as_str())
            || ![48.0, 28.0].contains(&self.threshold_bps)
            || ![24.0, 2.0, 3.74].contains(&self.admission_bps)
        {
            return Err("unregistered tier/threshold/cost".into());
        }
        Ok(())
    }
}

fn side(q: &s3_ticks::Snapshot, origin: f64, r: f64) -> Option<exits::Side> {
    if q.quote.bid > origin + r {
        Some(exits::Side::Long)
    } else if q.quote.ask < origin - r {
        Some(exits::Side::Short)
    } else {
        None
    }
}
fn price(q: &s3_ticks::Snapshot, s: exits::Side) -> f64 {
    if s == exits::Side::Long {
        q.quote.ask
    } else {
        q.quote.bid
    }
}
pub(super) fn opening_impulse(
    i: &Input,
    quotes: &[s3_ticks::Snapshot],
    origin: f64,
    spread: f64,
    p: &Policy,
) -> Result<(Option<exits::Input>, Value), String> {
    s3_ticks::validate_request(&i.request)?;
    if !matches!(
        i.request.symbol.as_str(),
        "SpotCrude" | "SpotBrent" | "US500" | "NAS100" | "XAUUSD" | "XAGUSD" | "BTCUSD" | "ETHUSD"
    ) {
        return Err("outside core research scope".into());
    }
    let c = &i.source.contract;
    let volume = i
        .request
        .tick_replay
        .as_ref()
        .ok_or("volume missing")?
        .volume;
    if !(0..=5).contains(&c.digits)
        || c.min_volume <= 0
        || c.step_volume <= 0
        || volume < c.min_volume
        || volume % c.step_volume != 0
    {
        return Err("invalid contract/volume".into());
    }
    let scale = 10_f64.powi(c.digits);
    let boundary = i.request.from + Duration::minutes(30);
    let raw = quotes
        .iter()
        .filter(|q| q.fresh && q.quote.at >= i.request.from && q.quote.at < boundary)
        .map(|q| ((q.quote.bid + q.quote.ask) / 2.0 - origin).abs())
        .fold(0_f64, f64::max);
    let r = (raw * scale).round() / scale;
    if r <= 0.0 {
        return Ok((None, json!([{"reason":"zero_opening_range"}])));
    }
    let mut mark = boundary;
    let mut marks = vec![];
    let mut veto = false;
    while mark < i.request.to {
        let mut m = json!({"at":mark,"reason":"no_fresh_quote_in_mark_window","r":r,"opening_range_frozen_at":boundary});
        if veto {
            m["reason"] = json!("session_cost_veto");
        } else if let Some((idx, q)) = quotes.iter().enumerate().find(|(_, q)| {
            q.fresh && q.quote.at >= mark && q.quote.at < mark + Duration::seconds(5)
        }) {
            m["decision_at"] = json!(q.quote.at);
            m["reason"] = json!("inside_opening_band");
            if let Some(s) = side(q, origin, r) {
                let decision_price = price(q, s);
                let cost = spread + 2.0 * decision_price * p.admission_bps / 10000.0;
                m["cost"] = json!(cost);
                if cost >= r {
                    veto = true;
                    m["reason"] = json!("cost_ge_r_session_veto");
                } else if !s3_ticks::range_floor_at(r, decision_price, p.threshold_bps) {
                    m["reason"] = json!("range_below_registered_threshold");
                } else {
                    m["reason"] = json!("no_next_fresh_fill_quote_before_deadline");
                    if let Some(f) = quotes.iter().skip(idx + 1).find(|f| {
                        f.fresh
                            && f.quote.at > q.quote.at
                            && f.quote.at < mark + Duration::seconds(5)
                    }) {
                        let fill = price(f, s);
                        if side(f, origin, r) != Some(s) {
                            m["reason"] = json!("direction_changed_before_fill");
                        } else if spread + 2.0 * fill * p.admission_bps / 10000.0 >= r {
                            veto = true;
                            m["reason"] = json!("dispatch_cost_ge_r_session_veto");
                        } else if !s3_ticks::range_floor_at(r, fill, p.threshold_bps) {
                            m["reason"] = json!("dispatch_range_below_registered_threshold");
                        } else {
                            m["reason"] = json!("simulated_entry");
                            marks.push(m);
                            return Ok((
                                Some(exits::Input {
                                    entered_at: f.quote.at,
                                    side: s,
                                    fill,
                                    initial_r: r,
                                    volume,
                                    tp1_close_volume: volume,
                                    tp1_distance: 1.38 * r,
                                    broker_tp_distance: 1.38 * r,
                                    session_end: f.quote.at + Duration::hours(72),
                                    digits: c.digits,
                                    cadence: exits::TrailCadence::S3ThirtySeconds,
                                }),
                                json!(marks),
                            ));
                        }
                    }
                }
            }
        }
        marks.push(m);
        mark += Duration::minutes(30);
    }
    Ok((None, json!(marks)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    #[test]
    fn causal_range_next_quote_floor_and_stress_veto() {
        let at: DateTime<Utc> = "2024-11-04T07:00:00Z".parse().unwrap();
        let input:Input=serde_json::from_value(json!({
            "request":{"account":1,"symbol":"SpotCrude","from":at,"to":at+Duration::hours(4),"spread":0.02,"commission_round_trip":0.0,"slippage_bps_per_side":24.0,"tick_replay":{"volume":100}},
            "source":{"bars":[],"bids":[],"asks":[],"contract":{"digits":2,"min_volume":100,"step_volume":100}},
            "future":[],"own_context":{"status":"missing_segments","drift_5":null,"drift_20":null,"latest_bar_end":null},"peer_context":null,"peer_available":false,"demo_entry_bps":null
        })).unwrap();
        let q = |sec, mid| s3_ticks::Snapshot {
            fresh: true,
            quote: exits::Quote {
                at: at + Duration::seconds(sec),
                bid: mid - 0.01,
                ask: mid + 0.01,
                gap_before: false,
            },
        };
        let quotes = [
            q(0, 100.0),
            q(1700, 100.35),
            q(1800, 100.40),
            q(1801, 100.42),
            q(1900, 110.0),
        ];
        let mut p = Policy {
            tier: "tier2".into(),
            threshold_bps: 28.0,
            admission_bps: 2.0,
        };
        let (e, _) = opening_impulse(&input, &quotes, 100.0, 0.02, &p).unwrap();
        let e = e.unwrap();
        assert_eq!(e.entered_at, at + Duration::seconds(1801));
        assert_eq!(e.initial_r, 0.35);
        p.admission_bps = 24.0;
        let (e, m) = opening_impulse(&input, &quotes, 100.0, 0.02, &p).unwrap();
        assert!(e.is_none());
        assert_eq!(m[0]["reason"], "cost_ge_r_session_veto");
        assert_eq!(m[1]["reason"], "session_cost_veto");
        p.admission_bps = 2.0;
        p.threshold_bps = 48.0;
        assert!(opening_impulse(&input, &quotes, 100.0, 0.02, &p)
            .unwrap()
            .0
            .is_none());
        assert!(!s3_ticks::range_floor_at(0.48, 100.01, 48.0));
    }
}
