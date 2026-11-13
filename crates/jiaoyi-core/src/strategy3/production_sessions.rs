//! Versioned production windows for owner directive №18,392.
//!
//! This module does not alter the earlier cash-session/research calendar.
//! Traditional daily starts follow the broker schedules captured on 8 Nov 2026:
//! NY18:00, metals tradable at18:01, Brent20:00 except Sunday's18:00.
//! London is the owner-declared07:00UTC anchor; NY09:30 follows IANA DST.
//! Actual contract holidays/trading mode remain a final broker-calendar gate.
use super::{
    band::{self, Band, SigmaTable, M30},
    macro_pullback::Context,
    sessions::{Session, SessionRule},
};
use crate::universe::AssetClass;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::America::New_York;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Market {
    Crypto,
    Metal,
    Crude,
    Brent,
    Index,
}
impl Market {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Crypto => "owner18409: six UTC four-hour entry windows; broker maintenance remains excluded",
            _ => "owner18409: native Asian, London07:00UTC and NewYork09:30 entry windows; broker calendar remains mandatory",
        }
    }
}

pub fn rule_for(symbol: &str, class: AssetClass) -> Option<SessionRule> {
    let market = match symbol {
        "BTCUSD" | "ETHUSD" => Market::Crypto,
        "XAUUSD" | "XAGUSD" => Market::Metal,
        "SpotCrude" => Market::Crude,
        "SpotBrent" => Market::Brent,
        "US500" | "NAS100" => Market::Index,
        _ => return super::sessions::rule_for(symbol, class),
    };
    Some(SessionRule::Production(market))
}

fn utc(date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_hms_opt(hour, minute, 0).expect("valid clock"))
}
fn ny(date: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
    New_York
        .from_local_datetime(&date.and_hms_opt(hour, minute, 0).expect("valid clock"))
        .single()
        .expect("broker anchors do not enter DST overlap/gap")
        .with_timezone(&Utc)
}

/// `date` is the UTC date for crypto and the NY trading date otherwise.
/// The window open, not this date, is the durable session identity.
pub fn windows_on(market: Market, date: NaiveDate) -> Vec<Session> {
    if market == Market::Crypto {
        return (0..24)
            .step_by(4)
            .map(|hour| {
                let open = utc(date, hour, 0);
                Session {
                    date,
                    open,
                    close: open + Duration::hours(4),
                    lunch: None,
                }
            })
            .collect();
    }
    if matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
        return vec![];
    }
    let prior = date - Duration::days(1);
    let asia_hour = if market == Market::Brent && date.weekday() != Weekday::Mon {
        20
    } else {
        18
    };
    let anchors = [
        ny(prior, asia_hour, 0),
        utc(date, 7, 0),
        ny(date, 9, 30),
        ny(
            date,
            16,
            if date.weekday() == Weekday::Fri {
                55
            } else {
                59
            },
        ),
    ];
    anchors
        .windows(2)
        .map(|a| Session {
            date,
            open: a[0],
            close: a[1],
            lunch: None,
        })
        .collect()
}

pub fn session_at(market: Market, at: DateTime<Utc>) -> Option<Session> {
    let date = at.date_naive();
    (-1..=1)
        .flat_map(|d| windows_on(market, date + Duration::days(d)))
        .find(|s| s.trading_at(at))
}
pub fn next_session(market: Market, at: DateTime<Utc>) -> Option<Session> {
    let date = at.date_naive();
    (-1..10)
        .flat_map(|d| windows_on(market, date + Duration::days(d)))
        .filter(|s| s.open > at)
        .min_by_key(|s| s.open)
}
pub fn slot(market: Market, session: Session) -> Option<usize> {
    windows_on(market, session.date)
        .iter()
        .position(|s| *s == session)
}

/// Use the same Asia/London/NY (or UTC four-hour) slot on prior trading dates.
/// Five-/twenty-day context never silently becomes five/twenty four-hour bars.
pub fn same_slot_before(market: Market, opening: DateTime<Utc>, n: usize) -> Vec<Session> {
    let Some(today) = session_at(market, opening) else {
        return vec![];
    };
    if today.open != opening {
        return vec![];
    }
    let Some(index) = slot(market, today) else {
        return vec![];
    };
    (1..=n * 2 + 14)
        .filter_map(|ago| {
            windows_on(market, today.date - Duration::days(ago as i64))
                .get(index)
                .copied()
        })
        .filter(|s| s.close <= opening)
        .take(n)
        .collect()
}

/// Owner №18,425: capture the first recorded two-sided native-open quote within
/// 15 seconds, accommodating evidenced upstream broker reopening latency.
pub const OPENING_QUOTE_WINDOW_SECONDS: i64 = 15;
pub const OPENING_QUOTE_POLICY: &str = "native_open_first_quote_v18425_15s";

/// Metals' M30 opening bar starts18:00NY, but the first valid executable quote
/// is18:01. The capture window starts at that actual broker observation time.
pub fn opening_observation_start(symbol: &str, session: Session) -> DateTime<Utc> {
    if matches!(symbol, "XAUUSD" | "XAGUSD") && slot(Market::Metal, session) == Some(0) {
        session.open + Duration::minutes(1)
    } else {
        session.open
    }
}

/// Weekly availability from the versioned current broker schedule. A fresh
/// runtime broker holiday/trading-mode check must still be applied at dispatch.
pub fn broker_weekly_open(symbol: &str, at: DateTime<Utc>) -> bool {
    if matches!(symbol, "BTCUSD" | "ETHUSD") {
        let local = at.with_timezone(&New_York);
        let second = local.num_seconds_from_midnight();
        return !(16 * 3600 + 59 * 60..17 * 3600 + 60).contains(&second);
    }
    let market = match symbol {
        "XAUUSD" | "XAGUSD" => Market::Metal,
        "SpotCrude" => Market::Crude,
        "SpotBrent" => Market::Brent,
        "US500" | "NAS100" => Market::Index,
        _ => return false,
    };
    session_at(market, at).is_some_and(|s| at >= opening_observation_start(symbol, s))
}

/// Execution-only policy №18,455. Recorded Pepperstone schedules close the
/// six traditional instruments at16:59NY (Mon–Thu),16:55NY (Fri). Do not move
/// Session.close: it also defines preparation and volatility-band samples.
pub const EXIT_DEADLINE_POLICY: &str = "native_preclose_v18455_120s";
pub const PRE_CLOSE_BUFFER_SECONDS: i64 = 120;
pub fn exit_deadline(symbol: &str, session: &Session) -> DateTime<Utc> {
    let traditional = matches!(
        symbol,
        "XAUUSD" | "XAGUSD" | "SpotCrude" | "SpotBrent" | "US500" | "NAS100"
    );
    let hard_close = ny(
        session.date,
        16,
        if session.date.weekday() == Weekday::Fri {
            55
        } else {
            59
        },
    );
    if traditional
        && session.close == hard_close
        && session.open < hard_close - Duration::seconds(PRE_CLOSE_BUFFER_SECONDS)
    {
        hard_close - Duration::seconds(PRE_CLOSE_BUFFER_SECONDS)
    } else {
        session.close
    }
}

/// Strictly future observation boundary, including the metal minute offset
/// when the nominal Asian session has already started. Unknown symbols have
/// no invented schedule; their close retries use the bounded slow poll.
pub fn next_opening_observation_start(symbol: &str, at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let market = match symbol {
        "BTCUSD" | "ETHUSD" => Market::Crypto,
        "XAUUSD" | "XAGUSD" => Market::Metal,
        "SpotCrude" => Market::Crude,
        "SpotBrent" => Market::Brent,
        "US500" | "NAS100" => Market::Index,
        _ => return None,
    };
    (-1..10)
        .flat_map(|d| windows_on(market, at.date_naive() + Duration::days(d)))
        .map(|s| opening_observation_start(symbol, s))
        .filter(|opening| *opening > at)
        .min()
}

pub fn entry_window(s: &Session, at: DateTime<Utc>) -> Result<(), String> {
    if !s.trading_at(at) {
        return Err("outside production evaluation window".into());
    }
    Ok(())
}

/// New index entries end at the cash close; recorded position exits retain
/// their original registry deadline.
pub fn entry_deadline_18407(symbol: &str, session: &Session) -> DateTime<Utc> {
    if matches!(symbol, "US500" | "NAS100") {
        session.close.min(ny(session.date, 16, 0))
    } else {
        session.close
    }
}

/// Owner №18,407: entry eligibility is narrower than the broker/session
/// registry. Keep historical windows for existing-position exits and preparation.
pub fn symbol_entry_window_18407(
    symbol: &str,
    session: &Session,
    at: DateTime<Utc>,
) -> Result<(), String> {
    entry_window(session, at)?;
    let allowed = match symbol {
        "US500" | "NAS100" => session.open == ny(session.date, 9, 30),
        "SpotCrude" | "SpotBrent" | "XAUUSD" | "XAGUSD" => {
            session.open == utc(session.date, 7, 0) || session.open == ny(session.date, 9, 30)
        }
        "BTCUSD" | "ETHUSD" => true,
        _ => false,
    };
    if allowed && at < entry_deadline_18407(symbol, session) {
        Ok(())
    } else {
        Err("outside_authorized_entry_session_18407".into())
    }
}

/// Owner №18,409 restores all registered native windows. This does not
/// authorise broker maintenance, holidays, or an invented session boundary.
pub fn entry_deadline(_symbol: &str, session: &Session) -> DateTime<Utc> {
    session.close
}

pub fn symbol_entry_window(
    symbol: &str,
    session: &Session,
    at: DateTime<Utc>,
) -> Result<(), String> {
    entry_window(session, at)?;
    let market = match symbol {
        "BTCUSD" | "ETHUSD" => Market::Crypto,
        "US500" | "NAS100" => Market::Index,
        "SpotCrude" => Market::Crude,
        "SpotBrent" => Market::Brent,
        "XAUUSD" | "XAGUSD" => Market::Metal,
        _ => return Err("symbol_outside_s3_core".into()),
    };
    if windows_on(market, session.date).contains(session) && broker_weekly_open(symbol, at) {
        Ok(())
    } else {
        Err("outside_native_broker_session_18409".into())
    }
}

/// Current half-hour slot, eligible throughout that slot. The actor consumes
/// a slot only after a qualifying impulse; rejected quotes do not consume it.
/// Historical +30m volatility conservatively covers the first half-hour.
pub fn decision_mark(s: &Session, at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    entry_window(s, at).ok()?;
    Some(s.open + Duration::minutes((at - s.open).num_minutes() / 30 * 30))
}
pub fn arm_expiry(symbol: &str, session: &Session, armed_at: DateTime<Utc>) -> DateTime<Utc> {
    if matches!(symbol, "BTCUSD" | "ETHUSD") {
        armed_at + Duration::hours(4)
    } else {
        session.close
    }
}

/// This table samples the same slot across at most45 preceding calendar days.
/// It keeps the full existing14-session minimum and refuses absent endpoints.
pub fn sigma_table(rule: &SessionRule, today: &Session, bars: &[M30]) -> SigmaTable {
    let SessionRule::Production(market) = rule else {
        return band::sigma_table(rule, today, bars);
    };
    let by_time: BTreeMap<_, _> = bars.iter().map(|b| (b.open_time, *b)).collect();
    let past: Vec<_> = same_slot_before(*market, today.open, band::MAX_SESSIONS_BACK)
        .into_iter()
        .filter(|s| s.date >= today.date - Duration::days(45))
        .filter_map(|s| {
            by_time
                .get(&s.open)
                .filter(|b| b.open.is_finite() && b.open > 0.0)
                .map(|b| (s, b.open))
        })
        .collect();
    let length = (today.close - today.open).num_minutes();
    let end = ((length + 29) / 30) * 30;
    let mut marks = BTreeMap::new();
    for minute in (30..=end).step_by(30) {
        let values: Vec<_> = past
            .iter()
            .filter_map(|(s, open)| {
                let bar_time = s.open + Duration::minutes(minute - 30);
                // Final broker bar may end at17:00 although trading ends16:59.
                if bar_time >= s.close {
                    return None;
                }
                by_time
                    .get(&bar_time)
                    .filter(|b| b.close.is_finite() && b.close > 0.0)
                    .map(|b| (b.close / open - 1.0).abs())
            })
            .take(band::LOOKBACK)
            .collect();
        marks.insert(
            minute,
            if values.len() == band::LOOKBACK {
                Ok(values.iter().sum::<f64>() / values.len() as f64)
            } else {
                Err(format!(
                    "{} same-slot endpoints;{} needed",
                    values.len(),
                    band::LOOKBACK
                ))
            },
        );
    }
    SigmaTable {
        marks,
        sessions_used: past.iter().map(|(s, _)| s.date).collect(),
    }
}

pub fn band_at(
    table: &SigmaTable,
    s: &Session,
    open: f64,
    at: DateTime<Utc>,
) -> Result<Band, String> {
    entry_window(s, at)?;
    let minutes = (at - s.open).num_milliseconds() as f64 / 60000.;
    if minutes >= 30. {
        return band::band_at(table, s, open, at);
    }
    if !open.is_finite() || open <= 0. {
        return Err("no production opening midpoint".into());
    }
    let sigma = table
        .marks
        .get(&30)
        .ok_or("no same-slot30m volatility coefficient")?
        .as_ref()
        .map_err(Clone::clone)?;
    if !sigma.is_finite() || *sigma <= 0. {
        return Err("invalid same-slot volatility".into());
    }
    Ok(Band {
        open,
        sigma: *sigma,
        lower: open * (1. - sigma),
        upper: open * (1. + sigma),
        half_width: open * sigma,
        minutes,
    })
}

pub fn context(
    rule: &SessionRule,
    session: Session,
    bars: &[M30],
    cutoff: DateTime<Utc>,
) -> Result<Context, String> {
    let SessionRule::Production(market) = rule else {
        return super::macro_pullback::context(rule, session, bars, cutoff);
    };
    let prior: Vec<_> = same_slot_before(*market, session.open, 20)
        .into_iter()
        .filter(|s| s.date >= session.date - Duration::days(45))
        .collect();
    if prior.len() != 20 {
        return Err("macro_missing_20_same_slot_days".into());
    }
    let mut by_time = BTreeMap::new();
    for b in bars {
        if ![b.open, b.close]
            .into_iter()
            .all(|x| x.is_finite() && x > 0.)
        {
            return Err("macro_invalid_bar".into());
        }
        if let Some(old) = by_time.insert(b.open_time, b) {
            if old.open != b.open || old.close != b.close {
                return Err("macro_conflicting_bar".into());
            }
        }
    }
    let drift = |n: usize| -> Result<(f64, DateTime<Utc>), String> {
        let mut first = None;
        let mut last = None;
        for s in prior[..n].iter().rev() {
            let mut t = s.open;
            while t < s.close {
                let end = t + Duration::minutes(30);
                if end > cutoff || end > session.open {
                    return Err("macro_incomplete_or_noncausal_session".into());
                }
                let b = by_time.get(&t).ok_or("macro_missing_m30_segment")?;
                first.get_or_insert(b.open);
                last = Some((b.close, end));
                t = end;
            }
        }
        let (close, end) = last.ok_or("macro_empty_session")?;
        Ok((close / first.ok_or("macro_empty_session")? - 1., end))
    };
    let (drift_5, _) = drift(5)?;
    let (drift_20, latest_bar_end) = drift(20)?;
    Ok(Context {
        drift_5,
        drift_20,
        latest_bar_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }
    #[test]
    fn preclose_buffer_changes_only_new_final_traditional_exit_deadlines() {
        for text in ["2026-11-12", "2026-11-13", "2026-07-09", "2026-07-10"] {
            let date: NaiveDate = text.parse().unwrap();
            for (name, market) in [
                ("XAUUSD", Market::Metal),
                ("XAGUSD", Market::Metal),
                ("SpotCrude", Market::Crude),
                ("SpotBrent", Market::Brent),
                ("US500", Market::Index),
                ("NAS100", Market::Index),
            ] {
                let windows = windows_on(market, date);
                let original = windows.clone();
                for (index, s) in windows.iter().enumerate() {
                    let deadline = exit_deadline(name, s);
                    assert_eq!(
                        deadline,
                        if index == 2 {
                            s.close - Duration::seconds(120)
                        } else {
                            s.close
                        }
                    );
                    assert!(broker_weekly_open(name, deadline));
                    assert_eq!(
                        entry_deadline(name, s),
                        s.close,
                        "entry windows stay frozen"
                    );
                    assert!(symbol_entry_window(name, s, s.close - Duration::seconds(1)).is_ok());
                    // Sigma still includes all native session marks, including the final mark.
                    let table = sigma_table(&SessionRule::Production(market), s, &[]);
                    assert!(table
                        .marks
                        .contains_key(&((s.close - s.open).num_minutes() / 30 * 30)));
                }
                assert_eq!(windows, original);
                assert_eq!(windows_on(market, date), original);
            }
            for name in ["BTCUSD", "ETHUSD"] {
                for s in windows_on(Market::Crypto, date) {
                    assert_eq!(exit_deadline(name, &s), s.close);
                }
            }
        }
    }

    #[test]
    fn future_opening_observation_handles_dst_weekends_and_metal_offset() {
        for (now, expected) in [
            ("2026-11-13T21:55:00Z", "2026-11-15T23:01:00Z"),
            ("2026-07-10T20:55:00Z", "2026-07-12T22:01:00Z"),
            ("2026-11-15T23:00:30Z", "2026-11-15T23:01:00Z"),
        ] {
            assert_eq!(
                next_opening_observation_start("XAUUSD", at(now)),
                Some(at(expected))
            );
        }
        let open = at("2026-11-15T23:01:00Z");
        assert!(next_opening_observation_start("XAUUSD", open).unwrap() > open);
        assert!(next_opening_observation_start("UNKNOWN", open).is_none());
    }

    #[test]
    fn owner18409_enables_all_native_core_windows_without_repair4_or_maintenance_bypass() {
        for date in [
            NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(),
            NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
        ] {
            for (symbol, market) in [
                ("US500", Market::Index),
                ("NAS100", Market::Index),
                ("SpotCrude", Market::Crude),
                ("SpotBrent", Market::Brent),
                ("XAUUSD", Market::Metal),
                ("XAGUSD", Market::Metal),
            ] {
                for s in windows_on(market, date) {
                    let start = opening_observation_start(symbol, s);
                    assert!(
                        symbol_entry_window(symbol, &s, start).is_ok(),
                        "{symbol} {s:?}"
                    );
                    assert!(
                        symbol_entry_window(symbol, &s, s.close - Duration::seconds(1)).is_ok()
                    );
                    assert!(symbol_entry_window(symbol, &s, s.close).is_err());
                    assert_eq!(entry_deadline(symbol, &s), s.close);
                    let forged = Session {
                        open: s.open + Duration::minutes(1),
                        ..s
                    };
                    assert!(symbol_entry_window(symbol, &forged, forged.open).is_err());
                }
            }
        }
        let at = at("2026-11-09T22:00:00Z");
        let s = session_at(Market::Crypto, at).unwrap();
        assert!(symbol_entry_window("BTCUSD", &s, at).is_err());
        assert!(symbol_entry_window("NatGas", &s, at).is_err());
    }

    #[test]
    fn crypto_has_no_utc_reset_blackout_and_keeps_six_distinct_windows() {
        let before = at("2026-11-08T23:59:59Z");
        let after = at("2026-11-09T00:00:00Z");
        let a = session_at(Market::Crypto, before).unwrap();
        let b = session_at(Market::Crypto, after).unwrap();
        assert!(entry_window(&a, before).is_ok());
        assert!(entry_window(&b, after).is_ok());
        assert_ne!(a.open, b.open);
        assert_eq!(windows_on(Market::Crypto, b.date).len(), 6);
        assert!(arm_expiry("BTCUSD", &a, before) > b.open);
        assert!(!broker_weekly_open("BTCUSD", at("2026-11-09T22:00:00Z")));
    }
    #[test]
    fn asia_open_respects_brent_metals_and_dst() {
        let mon = at("2026-11-09T00:00:00Z").date_naive();
        let tue = mon + Duration::days(1);
        assert_eq!(
            windows_on(Market::Index, mon)[0].open,
            at("2026-11-08T23:00:00Z")
        );
        assert_eq!(
            windows_on(Market::Brent, tue)[0].open,
            at("2026-11-10T01:00:00Z")
        );
        assert!(!broker_weekly_open("XAUUSD", at("2026-11-08T23:00:30Z")));
        assert!(broker_weekly_open("XAUUSD", at("2026-11-08T23:01:00Z")));
        assert_eq!(
            windows_on(Market::Index, mon)[2].open,
            at("2026-11-09T14:30:00Z")
        );
        assert_eq!(
            windows_on(
                Market::Index,
                NaiveDate::from_ymd_opt(2026, 10, 30).unwrap()
            )[2]
            .open,
            at("2026-10-30T13:30:00Z")
        );
        assert!(session_at(Market::Index, at("2026-11-07T12:00:00Z")).is_none());
    }
    #[test]
    fn same_slot_preparation_is_causal_twenty_days_and_first_half_hour_uses_recorded_coefficient() {
        let today = session_at(Market::Crypto, at("2026-11-09T04:00:00Z")).unwrap();
        let rule = SessionRule::Production(Market::Crypto);
        let mut bars = vec![];
        for s in same_slot_before(Market::Crypto, today.open, 30) {
            let day = (s.date - NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()).num_days() as f64;
            for m in (0..240).step_by(30) {
                bars.push(M30 {
                    open_time: s.open + Duration::minutes(m),
                    open: 100. + day,
                    close: 100.1 + day,
                });
            }
        }
        let c = context(&rule, today, &bars, today.open - Duration::minutes(20)).unwrap();
        assert!(c.drift_5 > 0. && c.drift_20 > 0.);
        assert!(c.latest_bar_end < today.open);
        let table = sigma_table(&rule, &today, &bars);
        let b = band_at(&table, &today, 100., today.open + Duration::seconds(2)).unwrap();
        assert!(b.sigma > 0.);
        assert!(b.minutes < 1.);
        bars.pop();
        assert!(
            context(&rule, today, &bars, today.open).is_ok(),
            "removed older unused bar"
        );
        let needed = same_slot_before(Market::Crypto, today.open, 1)[0].open;
        bars.retain(|b| b.open_time != needed);
        assert!(context(&rule, today, &bars, today.open).is_err());
    }
    #[test]
    fn repaired_entry_windows_exclude_index_london_and_traditional_asia() {
        for date in [
            NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(),
            NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
        ] {
            for symbol in [
                "US500",
                "NAS100",
                "SpotCrude",
                "SpotBrent",
                "XAUUSD",
                "XAGUSD",
            ] {
                let market = match symbol {
                    "US500" | "NAS100" => Market::Index,
                    "SpotCrude" => Market::Crude,
                    "SpotBrent" => Market::Brent,
                    _ => Market::Metal,
                };
                let windows = windows_on(market, date);
                assert!(symbol_entry_window_18407(symbol, &windows[0], windows[0].open).is_err());
                assert_eq!(
                    symbol_entry_window_18407(symbol, &windows[1], windows[1].open).is_ok(),
                    !matches!(symbol, "US500" | "NAS100")
                );
                assert!(symbol_entry_window_18407(symbol, &windows[2], windows[2].open).is_ok());
                assert_eq!(windows[2].open.with_timezone(&New_York).hour(), 9);
                assert_eq!(windows[2].open.with_timezone(&New_York).minute(), 30);
                if matches!(symbol, "US500" | "NAS100") {
                    let cash_close = ny(date, 16, 0);
                    assert!(symbol_entry_window_18407(
                        symbol,
                        &windows[2],
                        cash_close - Duration::milliseconds(1)
                    )
                    .is_ok());
                    assert!(symbol_entry_window_18407(symbol, &windows[2], cash_close).is_err());
                    assert!(
                        windows[2].close > cash_close,
                        "historical exit deadline remains intact"
                    );
                    assert_eq!(entry_deadline_18407(symbol, &windows[2]), cash_close);
                }
            }
            for window in windows_on(Market::Crypto, date) {
                assert!(symbol_entry_window_18407("BTCUSD", &window, window.open).is_ok());
            }
        }
    }
}
