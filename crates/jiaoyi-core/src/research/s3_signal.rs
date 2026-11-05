//! Read-only, fixed-horizon S3 signal study. Not an order/SL/TP simulator.
use crate::strategy3::{band, sessions, targets::Dir};
use crate::universe::AssetClass;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAJOR_FX: [&str; 7] = [
    "EURUSD", "GBPUSD", "USDJPY", "USDCHF", "USDCAD", "AUDUSD", "NZDUSD",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    pub at: DateTime<Utc>,
    pub open: f64,
    pub close: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub account: i64,
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub spread: f64,
    pub commission_round_trip: f64,
    pub slippage_bps_per_side: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_replay: Option<super::s3_ticks::Options>,
}
impl Request {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), String> {
        if !matches!(self.symbol.as_str(), "BTCUSD" | "NatGas") && !self.is_major_fx() {
            return Err(
                "symbol must be BTCUSD, NatGas or an explicitly supported major FX pair".into(),
            );
        }
        if self.is_major_fx()
            && self.tick_replay.as_ref().is_none_or(|o| {
                o.entry_rule != super::opening_range::EntryRule::FxSessionDiagnostic
            })
        {
            return Err(
                "major FX currently supports session diagnostics only; no entry setup is validated"
                    .into(),
            );
        }
        if self.to > crate::seal::seal_instant() {
            return Err("sealed research interval: this runner accepts development data ending by 2025-10-24T00:00:00Z only".into());
        }
        if self.account <= 0
            || self.from >= self.to
            || self.to > now
            || self.to - self.from > Duration::days(14)
        {
            return Err(
                "require a positive account and a closed historical range of at most 14 days"
                    .into(),
            );
        }
        if self.from.timestamp() % 1800 != 0
            || self.to.timestamp() % 1800 != 0
            || self.from.timestamp_subsec_nanos() != 0
            || self.to.timestamp_subsec_nanos() != 0
        {
            return Err("range boundaries must be exact UTC half-hour marks".into());
        }
        for n in [
            self.spread,
            self.commission_round_trip,
            self.slippage_bps_per_side,
        ] {
            if !n.is_finite() || !(0.0..=100_000.0).contains(&n) {
                return Err("cost assumptions must be finite, nonnegative and <=100000".into());
            }
        }
        if self.tick_replay.is_some() {
            super::s3_ticks::validate_request(self)?;
        }
        Ok(())
    }
    pub fn is_major_fx(&self) -> bool {
        MAJOR_FX.contains(&self.symbol.as_str())
    }
    pub fn rule(&self) -> sessions::SessionRule {
        sessions::rule_for(
            &self.symbol,
            if self.symbol == "BTCUSD" {
                AssetClass::Crypto
            } else if self.is_major_fx() {
                AssetClass::Fx
            } else {
                AssetClass::Energy
            },
        )
        .unwrap()
    }
    pub fn history_from(&self) -> DateTime<Utc> {
        self.from - Duration::days(50)
    }
}
#[derive(Debug, Serialize)]
pub struct Outcome {
    pub minutes: i64,
    pub exit_at: DateTime<Utc>,
    pub gross_bps: Option<f64>,
    pub net_bps: Option<f64>,
    pub missing: Option<&'static str>,
}
#[derive(Debug, Serialize)]
pub struct Signal {
    pub at: DateTime<Utc>,
    pub side: &'static str,
    pub signal_close: f64,
    pub entry_open_proxy: Option<f64>,
    pub r: f64,
    pub cost: Option<f64>,
    pub cost_admitted: bool,
    pub outcomes: Vec<Outcome>,
}
#[derive(Debug, Serialize)]
pub struct Study {
    pub mode: &'static str,
    pub session: String,
    pub input_bars: usize,
    pub eligible_marks: usize,
    pub missing_bands: usize,
    pub missing_session_opens: usize,
    pub signals: Vec<Signal>,
    pub notes: Vec<&'static str>,
}

pub fn study(req: &Request, bars: &[Bar]) -> Result<Study, String> {
    req.validate(Utc::now())?;
    let mut map = BTreeMap::new();
    for b in bars {
        if !b.open.is_finite()
            || !b.close.is_finite()
            || b.open <= 0.0
            || b.close <= 0.0
            || b.at.timestamp() % 1800 != 0
            || b.at.timestamp_subsec_nanos() != 0
            || b.at < req.history_from()
            || b.at + Duration::minutes(30) > req.to
        {
            return Err("invalid, out-of-range or incomplete M30 bar".into());
        }
        if map.insert(b.at, b).is_some() {
            return Err("duplicate M30 timestamp".into());
        }
    }
    let rule = req.rule();
    let m30: Vec<_> = bars
        .iter()
        .map(|b| band::M30 {
            open_time: b.at,
            open: b.open,
            close: b.close,
        })
        .collect();
    let mut result=Study{mode:"all_mark_signal_followthrough",session:rule.describe(),input_bars:bars.len(),eligible_marks:0,missing_bands:0,missing_session_opens:0,signals:vec![],notes:vec![
        "Broker M30 single-price bars are proxies, not executable bid/ask ticks. No order is sent.",
        "All raw qualifying marks are studied, including those rejected by costs; overlapping outcomes are not independent trades.",
        "Session veto, one-entry limit, Kelly, sizing and exits are deliberately not applied: this isolates signal follow-through.",
        "Entry uses the next M30 open; each horizon requires every intervening bar. No missing price is filled forward.",
        "Costs are explicit scenarios, not certified historical costs. No volume-based VWAP is inferred from tick volume.",
        "This is retrospective development evidence, not an untouched holdout or a per-account goal PASS.",
    ]};
    let mut date = req.from.date_naive();
    while date <= req.to.date_naive() {
        if let Some(session) = sessions::session_on(&rule, date) {
            if session.close > req.from && session.open < req.to {
                if let Some(open) = map.get(&session.open) {
                    let table = band::sigma_table(&rule, &session, &m30);
                    let mut at = (session.open + Duration::minutes(30)).max(req.from);
                    while at < session.close - Duration::minutes(30) && at < req.to {
                        result.eligible_marks += 1;
                        let signal_bar = map.get(&(at - Duration::minutes(30)));
                        match (signal_bar, band::band_at(&table, &session, open.open, at)) {
                            (Some(sb), Ok(b)) => {
                                if let Some(side) = band::mark_decision(
                                    &b,
                                    Some(sb.close - req.spread / 2.0),
                                    Some(sb.close + req.spread / 2.0),
                                ) {
                                    let sign = if side == Dir::Buy { 1.0 } else { -1.0 };
                                    let entry = map.get(&at).map(|x| x.open);
                                    let cost = entry.map(|x| {
                                        req.spread
                                            + req.commission_round_trip
                                            + 2.0 * req.slippage_bps_per_side / 10000.0 * x
                                    });
                                    let mut horizons =
                                        vec![30, 60, 120, (session.close - at).num_minutes()];
                                    horizons.sort();
                                    horizons.dedup();
                                    let mut outcomes = vec![];
                                    for minutes in horizons {
                                        let end = at + Duration::minutes(minutes);
                                        let complete = end <= session.close
                                            && end <= req.to
                                            && (0..minutes / 30).all(|i| {
                                                map.contains_key(&(at + Duration::minutes(i * 30)))
                                            });
                                        let price = if complete {
                                            map.get(&(end - Duration::minutes(30))).map(|b| b.close)
                                        } else {
                                            None
                                        };
                                        let pair = entry.zip(price).zip(cost);
                                        outcomes.push(Outcome {
                                            minutes,
                                            exit_at: end,
                                            gross_bps: pair
                                                .map(|((e, x), _)| sign * (x - e) / e * 10000.0),
                                            net_bps: pair.map(|((e, x), c)| {
                                                (sign * (x - e) - c) / e * 10000.0
                                            }),
                                            missing: pair.is_none().then_some(
                                                "missing bar or horizon outside session/range",
                                            ),
                                        });
                                    }
                                    result.signals.push(Signal {
                                        at,
                                        side: if sign > 0.0 { "buy" } else { "sell" },
                                        signal_close: sb.close,
                                        entry_open_proxy: entry,
                                        r: b.half_width,
                                        cost,
                                        cost_admitted: cost.is_some_and(|c| {
                                            c < b.half_width && b.half_width > 0.0
                                        }),
                                        outcomes,
                                    });
                                }
                            }
                            _ => result.missing_bands += 1,
                        }
                        at += Duration::minutes(30);
                    }
                } else {
                    result.missing_session_opens += 1;
                }
            }
        }
        date += Duration::days(1);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(symbol: &str) -> (Request, Vec<Bar>) {
        let from = "2025-10-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let req = Request {
            account: 1,
            symbol: symbol.into(),
            from,
            to: from + Duration::days(1),
            spread: 0.02,
            commission_round_trip: 0.01,
            slippage_bps_per_side: 2.0,
            tick_replay: None,
        };
        let mut bars = vec![];
        let mut at = req.history_from();
        while at < req.to {
            bars.push(Bar {
                at,
                open: 100.0,
                close: 101.0,
            });
            at += Duration::minutes(30);
        }
        (req, bars)
    }
    #[test]
    fn next_open_cost_and_missing_path_are_explicit() {
        let (req, mut bars) = fixture("BTCUSD");
        let mark = req.from + Duration::minutes(30);
        bars.iter_mut().find(|b| b.at == req.from).unwrap().close = 104.0;
        let entry = bars.iter_mut().find(|b| b.at == mark).unwrap();
        entry.open = 105.0;
        entry.close = 103.0;
        let out = study(&req, &bars).unwrap();
        let sig = &out.signals[0];
        assert_eq!(sig.at, mark);
        assert_eq!(sig.side, "buy");
        assert_eq!(sig.entry_open_proxy, Some(105.0));
        assert!((sig.r - 1.0).abs() < 1e-9);
        let h = sig.outcomes.iter().find(|x| x.minutes == 30).unwrap();
        assert!((h.net_bps.unwrap() - ((-2.0 - 0.072) / 105.0 * 10000.0)).abs() < 1e-9);
        // A future change does not alter the signal band or the completed 30m outcome.
        bars.iter_mut()
            .find(|b| b.at == mark + Duration::minutes(60))
            .unwrap()
            .close = 150.0;
        let changed = study(&req, &bars).unwrap();
        assert_eq!(changed.signals[0].r, sig.r);
        assert_eq!(changed.signals[0].outcomes[0].net_bps, h.net_bps);
        bars.retain(|b| b.at != mark);
        let missing = study(&req, &bars).unwrap();
        assert!(missing.signals[0]
            .outcomes
            .iter()
            .all(|h| h.net_bps.is_none()));
    }
    #[test]
    fn energy_session_short_sign_and_duplicates() {
        let (req, mut bars) = fixture("NatGas");
        let open = req.from + Duration::hours(7);
        let mark = open + Duration::minutes(30);
        bars.iter_mut().find(|b| b.at == open).unwrap().close = 96.0;
        let entry = bars.iter_mut().find(|b| b.at == mark).unwrap();
        entry.open = 95.0;
        entry.close = 93.0;
        let out = study(&req, &bars).unwrap();
        assert_eq!(out.eligible_marks, 24);
        assert_eq!(out.signals[0].side, "sell");
        assert_eq!(out.signals[0].at, mark);
        assert!(out.signals[0].outcomes[0].net_bps.unwrap() > 0.0);
        bars.push(bars[0].clone());
        assert!(study(&req, &bars).unwrap_err().contains("duplicate"));
    }
    #[test]
    fn bounds_and_history_insufficiency_do_not_manufacture_results() {
        let (mut req, bars) = fixture("BTCUSD");
        let short: Vec<_> = bars.into_iter().filter(|b| b.at >= req.from).collect();
        let out = study(&req, &short).unwrap();
        assert!(out.signals.is_empty());
        assert_eq!(out.missing_bands, 46);
        req.to = req.from + Duration::days(15);
        assert!(req.validate(req.to).is_err());
    }
}
