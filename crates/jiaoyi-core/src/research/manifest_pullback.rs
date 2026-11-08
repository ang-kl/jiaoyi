//! №18,358: manifest-qualified, causal three-arm experiment. No transport or live route.
use super::{
    exit_replay as exits, option_ab,
    s3_signal::Request,
    s3_ticks,
    seasonal::{self, M30},
};
use crate::strategy3::targets::price_ticks;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub status: String,
    pub drift_5: Option<f64>,
    pub drift_20: Option<f64>,
    pub latest_bar_end: Option<DateTime<Utc>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub request: Request,
    pub source: s3_ticks::Source,
    pub future: Vec<M30>,
    pub own_context: Context,
    pub peer_context: Option<Context>,
    pub peer_available: bool,
    pub demo_entry_bps: Option<f64>,
}
fn direction(i: &exits::Input) -> f64 {
    if i.side == exits::Side::Long {
        1.0
    } else {
        -1.0
    }
}
fn executable(q: &s3_ticks::Snapshot, side: exits::Side) -> f64 {
    if side == exits::Side::Long {
        q.quote.ask
    } else {
        q.quote.bid
    }
}
fn target(r: f64, digits: i32) -> Result<f64, String> {
    let ticks = price_ticks(r, digits).ok_or("invalid risk grid")?;
    Ok(((ticks
        .checked_mul(138)
        .and_then(|x| x.checked_add(99))
        .ok_or("target overflow")?)
        / 100) as f64
        / 10_f64.powi(digits))
}
fn guard_context(c: &Context, at: DateTime<Utc>) -> Result<(), String> {
    if c.latest_bar_end.is_some_and(|t| t > at)
        || c.drift_5.is_some_and(|d| !d.is_finite())
        || c.drift_20.is_some_and(|d| !d.is_finite())
    {
        return Err("noncausal/invalid macro context".into());
    }
    Ok(())
}
fn aligns(c: &Context, sign: f64, strict: bool) -> bool {
    c.status == "complete"
        && [c.drift_5, c.drift_20].iter().all(|d| {
            d.is_some_and(|v| {
                if strict {
                    v * sign > 0.0
                } else {
                    v * sign >= 0.0
                }
            })
        })
}
fn macro_filter(i: &Input, sign: f64) -> (&'static str, bool) {
    if !aligns(&i.own_context, sign, true) {
        return ("own_5d20d_disagree_or_missing", false);
    }
    if !i.peer_available {
        return ("own_aligned_peer_absent", true);
    }
    match &i.peer_context {
        Some(c) if aligns(c, sign, false) => ("own_and_peer_aligned", true),
        Some(c) if c.status == "complete" => ("peer_conflict", false),
        _ => ("peer_context_missing", false),
    }
}
fn pullback(
    original: &exits::Input,
    quotes: &[s3_ticks::Snapshot],
    origin: f64,
    to: DateTime<Utc>,
    admission_bps: f64,
) -> Result<(Option<exits::Input>, Value), String> {
    let q = quotes
        .iter()
        .find(|q| q.fresh && q.quote.at == original.entered_at)
        .ok_or("missing impulse quote")?;
    let sign = direction(original);
    let extreme = (q.quote.bid + q.quote.ask) / 2.0;
    if sign * (extreme - origin) <= 0.0 {
        return Ok((None, json!({"status":"wrong_impulse_sign"})));
    }
    let scale = 10_f64.powi(original.digits);
    let raw = origin + 0.5 * (extreme - origin);
    let level = if sign > 0.0 {
        (raw * scale).floor() / scale
    } else {
        (raw * scale).ceil() / scale
    };
    let mut detail = json!({"status":"no_retest","origin":origin,"impulse_mid":extreme,"impulse_at":original.entered_at,"touch_level":level});
    for q in quotes
        .iter()
        .filter(|q| q.fresh && q.quote.at > original.entered_at && q.quote.at < to)
    {
        let fill = executable(q, original.side);
        if sign * (fill - level) > 0.0 {
            continue;
        }
        if sign * (fill - origin) <= 0.0 {
            detail["status"] = json!("gap_through_origin");
            detail["at"] = json!(q.quote.at);
            return Ok((None, detail));
        }
        let reserve = (q.quote.ask - q.quote.bid) + 2.0 * fill * admission_bps / 10000.0;
        let risk = (((fill - origin).abs() + reserve) * scale).ceil() / scale;
        let distance = target(risk, original.digits)?;
        let entry = exits::Input {
            entered_at: q.quote.at,
            fill,
            initial_r: risk,
            tp1_distance: distance,
            broker_tp_distance: distance,
            tp1_close_volume: original.volume,
            session_end: q.quote.at + Duration::hours(72),
            ..original.clone()
        };
        if fill - sign * risk <= 0.0 {
            detail["status"] = json!("invalid_structural_stop");
            return Ok((None, detail));
        }
        detail["status"] = json!("entered");
        detail["at"] = json!(q.quote.at);
        detail["reserve_price_units"] = json!(reserve);
        detail["initial_stop"] = json!(fill - sign * risk);
        detail["pullback_r_bps"] = json!(risk / fill * 10000.0);
        return Ok((Some(entry), detail));
    }
    Ok((None, detail))
}
fn benchmark(v: &mut Value, i: &exits::Input, entry_bps: Option<f64>) {
    let price = v["outcome"]["fills"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|f| f["price"].as_f64())
        .or_else(|| v["exit"]["price"].as_f64());
    let Some((g, p)) = v["metrics"]["gross_r"].as_f64().zip(price) else {
        return;
    };
    let b = entry_bps.map(|e| {
        json!({"demo_entry_plus_2_exit":g-(i.fill*e+p*2.0)/10000.0/i.initial_r,
        "demo_entry_plus_3.74_exit":g-(i.fill*e+p*3.74)/10000.0/i.initial_r})
    });
    v["metrics"]["demo_entry_benchmark"] = json!(b);
    v["metrics"]["round_trip_cost_verified"] = json!(false);
}
fn paths(entry: &exits::Input, quotes: &[s3_ticks::Snapshot], i: &Input) -> Result<Value, String> {
    let mut strict = option_ab::tick_run(entry, quotes, i.request.to, false)?;
    let mut forced = option_ab::tick_run(entry, quotes, i.request.to, true)?;
    let q = quotes
        .iter()
        .find(|q| q.quote.at == entry.entered_at && q.fresh)
        .ok_or("entry quote missing")?;
    let spread = q.quote.ask - q.quote.bid;
    let mut continuation =
        option_ab::continue_bars(entry, &strict, &i.future, i.request.to, spread)?;
    continuation["entry"] = json!(entry);
    continuation["short_ask_spread_proxy"] = json!(spread);
    for v in [&mut strict, &mut forced, &mut continuation] {
        benchmark(v, entry, i.demo_entry_bps)
    }
    Ok(json!({"status":"admitted","strict":strict,"forced":forced,"continuation":continuation}))
}
pub fn study(i: &Input) -> Result<Value, String> {
    study_policy(i, &super::tiered_pullback::Policy::default())
}
pub fn study_policy(i: &Input, policy: &super::tiered_pullback::Policy) -> Result<Value, String> {
    policy.validate()?;
    if i.request.account <= 0
        || i.request.to != i.request.from + Duration::hours(4)
        || i.request.commission_round_trip != 0.0
    {
        return Err("invalid manifest scope".into());
    }
    guard_context(&i.own_context, i.request.from)?;
    if let Some(c) = &i.peer_context {
        guard_context(c, i.request.from)?
    }
    option_ab::validate_future(&i.request, &i.future)?;
    // Defence in depth: all source preparation timestamps and tick tails stay outside the protected interval.
    let start = "2025-08-19T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let end = "2025-09-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    if (i.request.from < end && i.request.to + Duration::seconds(5) > start)
        || i.source
            .bars
            .iter()
            .any(|b| b.at < end && b.at + Duration::minutes(30) > start)
    {
        return Err("protected source dependency".into());
    }
    let (quotes, quality) = s3_ticks::snapshots(&i.request, &i.source)?;
    let origin = quotes
        .iter()
        .find(|q| {
            q.fresh
                && q.quote.at >= i.request.from
                && q.quote.at < i.request.from + Duration::seconds(5)
        })
        .map(|q| (q.quote.bid + q.quote.ask) / 2.0)
        .ok_or("no fresh opening midpoint")?;
    let spread = seasonal::spread(&quotes, i.request.from + Duration::minutes(30))
        .ok_or("no causal prepared spread")?;
    let mut request = i.request.clone();
    request.spread = spread;
    let (entry, marks) = if policy.tier == "tier2" {
        super::tiered_pullback::opening_impulse(i, &quotes, origin, spread, policy)?
    } else {
        let native = if policy.admission_bps == 24.0 && policy.threshold_bps == 48.0 {
            s3_ticks::study_manifest_floor(&request, &i.source)?
        } else {
            s3_ticks::study_manifest_policy(
                &request,
                &i.source,
                policy.admission_bps,
                policy.threshold_bps,
            )?
        };
        (native.trade.map(|t| t.entry), json!(native.marks))
    };
    let Some(mut a) = entry else {
        return Ok(
            json!({"arms":{},"status":"no_admitted_impulse","marks":marks,"quality":quality}),
        );
    };
    a.tp1_close_volume = a.volume;
    a.tp1_distance = target(a.initial_r, a.digits)?;
    a.broker_tp_distance = a.tp1_distance;
    a.session_end = a.entered_at + Duration::hours(72);
    let (b, detail) = pullback(&a, &quotes, origin, i.request.to, policy.admission_bps)?;
    let (reason, pass) = macro_filter(i, direction(&a));
    let empty =
        |reason: &str| json!({"status":reason,"strict":null,"forced":null,"continuation":null});
    let bpaths = if let Some(b) = &b {
        paths(b, &quotes, i)?
    } else {
        empty(detail["status"].as_str().unwrap())
    };
    let cpaths = if pass { bpaths.clone() } else { empty(reason) };
    Ok(
        json!({"status":"impulse_admitted","marks":marks,"quality":quality,"prepared_spread":spread,
        "impulse":a,"pullback":detail,"macro":{"reason":reason,"pass":pass},"arms":{
        "arm_a_breakout_baseline":paths(&a,&quotes,i)?,"arm_b_pure_pullback_retest":bpaths,"arm_c_macro_filtered_pullback":cpaths},
        "financial_pass":false,"new_broker_requests":0}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at() -> DateTime<Utc> {
        "2024-11-04T07:00:00Z".parse().unwrap()
    }
    fn input(side: exits::Side) -> exits::Input {
        exits::Input {
            entered_at: at(),
            side,
            fill: if side == exits::Side::Long {
                102.02
            } else {
                97.98
            },
            initial_r: 1.0,
            volume: 1,
            tp1_close_volume: 1,
            tp1_distance: 1.38,
            broker_tp_distance: 1.38,
            session_end: at() + Duration::hours(72),
            digits: 2,
            cadence: exits::TrailCadence::S3ThirtySeconds,
        }
    }
    #[test]
    fn pullback_is_symmetric_later_executable_and_rejects_origin_gap() {
        for side in [exits::Side::Long, exits::Side::Short] {
            let i = input(side);
            let s = direction(&i);
            let q = |sec, mid| s3_ticks::Snapshot {
                fresh: true,
                quote: exits::Quote {
                    at: at() + Duration::seconds(sec),
                    bid: mid - 0.01,
                    ask: mid + 0.01,
                    gap_before: false,
                },
            };
            let quotes = [
                q(0, 100.0 + s * 2.0),
                q(1, 100.0 + s * 1.3),
                q(2, 100.0 + s * 0.98),
            ];
            let (b, d) = pullback(&i, &quotes, 100.0, at() + Duration::hours(4), 24.0).unwrap();
            let b = b.unwrap();
            assert_eq!(b.entered_at, at() + Duration::seconds(2));
            assert_eq!(d["status"], "entered");
            assert!(s * (b.fill - s * b.initial_r - 100.0) < 0.0);
            let bad = [quotes[0], q(2, 100.0 - s * 0.5)];
            assert!(pullback(&i, &bad, 100.0, at() + Duration::hours(4), 24.0)
                .unwrap()
                .0
                .is_none());
            let same = [quotes[0], q(0, 100.0 + s * 0.98)];
            assert!(pullback(&i, &same, 100.0, at() + Duration::hours(4), 24.0)
                .unwrap()
                .0
                .is_none());
        }
    }
    #[test]
    fn trend_context_is_causal_and_requires_both_signs() {
        let mut c = Context {
            status: "complete".into(),
            drift_5: Some(0.01),
            drift_20: Some(0.02),
            latest_bar_end: Some(at()),
        };
        assert!(aligns(&c, 1.0, true));
        assert!(!aligns(&c, -1.0, true));
        c.drift_5 = Some(0.0);
        assert!(!aligns(&c, 1.0, true));
        assert!(aligns(&c, 1.0, false));
        c.latest_bar_end = Some(at() + Duration::seconds(1));
        assert!(guard_context(&c, at()).is_err());
    }
}
