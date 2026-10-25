//! Strategy 3's trading sessions (owner-approved rules, 25-10-2026).
//!
//! - **Indices** trade on their exchange's cash session, in the exchange's
//!   own time zone, so daylight saving time follows the IANA database
//!   (`chrono-tz`), never a hand rule:
//!   - US500, NAS100, US30: NYSE, 09:30–16:00 America/New_York;
//!   - GER40: Xetra, 09:00–17:30 Europe/Berlin;
//!   - UK100: LSE, 08:00–16:30 Europe/London;
//!   - JPN225: TSE, 09:00–15:30 Asia/Tokyo, without the 11:30–12:30 lunch;
//!   - AUS200: ASX, 10:00–16:00 Australia/Sydney.
//!
//!   They run Monday to Friday on the exchange's local calendar.
//! - **FX majors, metals and energy**: 07:00–20:00 UTC, Monday to Friday.
//! - **BTCUSD, ETHUSD**: the UTC day, every day.
//!
//! Exchange holidays are not listed here. A session whose first 30 minutes
//! carry no ticks is skipped as a holiday by the trader (rules, "Markets").

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use serde::Serialize;

use crate::universe::AssetClass;

/// How a symbol's sessions are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRule {
    /// An exchange cash session in its own time zone, weekdays only.
    Exchange {
        exchange: &'static str,
        tz: Tz,
        open: NaiveTime,
        close: NaiveTime,
        /// A midday break with no entries (TSE's lunch).
        lunch: Option<(NaiveTime, NaiveTime)>,
    },
    /// A fixed UTC window, weekdays only.
    UtcWindow { open: NaiveTime, close: NaiveTime },
    /// 00:00–24:00 UTC, every day.
    UtcDay,
}

fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).expect("a valid time")
}

impl SessionRule {
    /// The rule's name on routes and records.
    pub fn describe(&self) -> String {
        match self {
            SessionRule::Exchange {
                exchange,
                tz,
                open,
                close,
                lunch,
            } => format!(
                "{exchange} {}–{} {}{}",
                open.format("%H:%M"),
                close.format("%H:%M"),
                tz.name(),
                lunch
                    .map(|(a, b)| format!(", lunch {}–{}", a.format("%H:%M"), b.format("%H:%M")))
                    .unwrap_or_default()
            ),
            SessionRule::UtcWindow { open, close } => {
                format!(
                    "{}–{} UTC, Monday to Friday",
                    open.format("%H:%M"),
                    close.format("%H:%M")
                )
            }
            SessionRule::UtcDay => "the UTC day, every day".into(),
        }
    }
}

/// The session rule of a universe symbol, or `None` when the rules give it
/// none (an index not named in the rules is not traded).
pub fn rule_for(name: &str, class: AssetClass) -> Option<SessionRule> {
    use chrono_tz::{America, Asia, Australia, Europe};
    let ex = |exchange, tz, open, close, lunch| SessionRule::Exchange {
        exchange,
        tz,
        open,
        close,
        lunch,
    };
    match class {
        AssetClass::EquityIndex => Some(match name {
            "US500" | "NAS100" | "US30" => ex("NYSE", America::New_York, t(9, 30), t(16, 0), None),
            "GER40" => ex("Xetra", Europe::Berlin, t(9, 0), t(17, 30), None),
            "UK100" => ex("LSE", Europe::London, t(8, 0), t(16, 30), None),
            "JPN225" => ex(
                "TSE",
                Asia::Tokyo,
                t(9, 0),
                t(15, 30),
                Some((t(11, 30), t(12, 30))),
            ),
            "AUS200" => ex("ASX", Australia::Sydney, t(10, 0), t(16, 0), None),
            _ => return None,
        }),
        AssetClass::Fx | AssetClass::Metal | AssetClass::Energy => Some(SessionRule::UtcWindow {
            open: t(7, 0),
            close: t(20, 0),
        }),
        AssetClass::Crypto => Some(SessionRule::UtcDay),
    }
}

/// One session, in UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Session {
    /// The trading date, on the session's own calendar.
    pub date: NaiveDate,
    pub open: DateTime<Utc>,
    pub close: DateTime<Utc>,
    /// The lunch break, `[start, end)`, if any.
    pub lunch: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl Session {
    /// Whether the session is trading at `at`: open ≤ at < close, outside lunch.
    pub fn trading_at(&self, at: DateTime<Utc>) -> bool {
        at >= self.open && at < self.close && !self.lunch.is_some_and(|(a, b)| at >= a && at < b)
    }
}

fn weekday(d: NaiveDate) -> bool {
    !matches!(d.weekday(), Weekday::Sat | Weekday::Sun)
}

/// A local wall-clock time on `date` in `tz`, as UTC. Exchange opens and
/// closes never fall in a DST gap or overlap (those are at night); if one
/// ever did, the day has no session rather than a guessed one.
fn local(tz: Tz, date: NaiveDate, time: NaiveTime) -> Option<DateTime<Utc>> {
    tz.from_local_datetime(&date.and_time(time))
        .single()
        .map(|x| x.with_timezone(&Utc))
}

/// The session of `rule` on the trading date `date`, or `None` (a weekend).
pub fn session_on(rule: &SessionRule, date: NaiveDate) -> Option<Session> {
    match *rule {
        SessionRule::Exchange {
            tz,
            open,
            close,
            lunch,
            ..
        } => {
            if !weekday(date) {
                return None;
            }
            Some(Session {
                date,
                open: local(tz, date, open)?,
                close: local(tz, date, close)?,
                lunch: match lunch {
                    Some((a, b)) => Some((local(tz, date, a)?, local(tz, date, b)?)),
                    None => None,
                },
            })
        }
        SessionRule::UtcWindow { open, close } => weekday(date).then(|| Session {
            date,
            open: Utc.from_utc_datetime(&date.and_time(open)),
            close: Utc.from_utc_datetime(&date.and_time(close)),
            lunch: None,
        }),
        SessionRule::UtcDay => {
            let open = Utc.from_utc_datetime(&date.and_time(t(0, 0)));
            Some(Session {
                date,
                open,
                close: open + Duration::days(1),
                lunch: None,
            })
        }
    }
}

/// The session that is open at `at` (open ≤ at < close; lunch included), if any.
pub fn session_at(rule: &SessionRule, at: DateTime<Utc>) -> Option<Session> {
    // A session's local date is within a day of the UTC date (Sydney is
    // ahead, New York behind).
    let d = at.date_naive();
    [d - Duration::days(1), d, d + Duration::days(1)]
        .into_iter()
        .filter_map(|x| session_on(rule, x))
        .find(|s| at >= s.open && at < s.close)
}

/// The first session that opens after `at`.
pub fn next_session(rule: &SessionRule, at: DateTime<Utc>) -> Option<Session> {
    let d = at.date_naive() - Duration::days(1);
    (0..10)
        .filter_map(|i| session_on(rule, d + Duration::days(i)))
        .find(|s| s.open > at)
}

/// The `n` sessions that closed at or before `before`, newest first.
pub fn sessions_before(rule: &SessionRule, before: DateTime<Utc>, n: usize) -> Vec<Session> {
    let mut out = Vec::new();
    let mut d = before.date_naive() + Duration::days(1);
    // At most n weekdays plus their weekends, and a margin.
    for _ in 0..(n * 2 + 14) {
        if out.len() == n {
            break;
        }
        if let Some(s) = session_on(rule, d) {
            if s.close <= before {
                out.push(s);
            }
        }
        d -= Duration::days(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn rule(name: &str, class: AssetClass) -> SessionRule {
        rule_for(name, class).unwrap()
    }

    #[test]
    fn europe_left_summer_time_on_25_october_2026() {
        // Xetra 09:00 Berlin: CEST (UTC+2) on Friday 23-10, CET (UTC+1) on Monday 26-10.
        let ger = rule("GER40", AssetClass::EquityIndex);
        let fri = session_on(&ger, date(2026, 10, 23)).unwrap();
        assert_eq!(fri.open, at("2026-10-23T07:00:00Z"));
        assert_eq!(fri.close, at("2026-10-23T15:30:00Z"));
        let mon = session_on(&ger, date(2026, 10, 26)).unwrap();
        assert_eq!(mon.open, at("2026-10-26T08:00:00Z"));
        assert_eq!(mon.close, at("2026-10-26T16:30:00Z"));
        // LSE 08:00 London: BST (UTC+1) then GMT.
        let uk = rule("UK100", AssetClass::EquityIndex);
        assert_eq!(
            session_on(&uk, date(2026, 10, 23)).unwrap().open,
            at("2026-10-23T07:00:00Z")
        );
        assert_eq!(
            session_on(&uk, date(2026, 10, 26)).unwrap().open,
            at("2026-10-26T08:00:00Z")
        );
        assert_eq!(
            session_on(&uk, date(2026, 10, 26)).unwrap().close,
            at("2026-10-26T16:30:00Z")
        );
        // Sunday 25-10 itself has no session.
        assert!(session_on(&ger, date(2026, 10, 25)).is_none());
    }

    #[test]
    fn the_us_leaves_summer_time_on_1_november_2026() {
        // NYSE 09:30 New York: EDT (UTC−4) on Friday 30-10, EST (UTC−5) on Monday 2-11.
        for name in ["US500", "NAS100", "US30"] {
            let us = rule(name, AssetClass::EquityIndex);
            let fri = session_on(&us, date(2026, 10, 30)).unwrap();
            assert_eq!(fri.open, at("2026-10-30T13:30:00Z"), "{name}");
            assert_eq!(fri.close, at("2026-10-30T20:00:00Z"), "{name}");
            let mon = session_on(&us, date(2026, 11, 2)).unwrap();
            assert_eq!(mon.open, at("2026-11-02T14:30:00Z"), "{name}");
            assert_eq!(mon.close, at("2026-11-02T21:00:00Z"), "{name}");
        }
        // The week between: Europe already on winter time, the US not yet.
        let us = rule("US500", AssetClass::EquityIndex);
        assert_eq!(
            session_on(&us, date(2026, 10, 26)).unwrap().open,
            at("2026-10-26T13:30:00Z")
        );
    }

    #[test]
    fn tokyo_has_a_lunch_and_sydney_opens_the_previous_utc_day() {
        let jp = rule("JPN225", AssetClass::EquityIndex);
        let s = session_on(&jp, date(2026, 10, 26)).unwrap();
        assert_eq!(s.open, at("2026-10-26T00:00:00Z"));
        assert_eq!(s.close, at("2026-10-26T06:30:00Z"));
        assert_eq!(
            s.lunch,
            Some((at("2026-10-26T02:30:00Z"), at("2026-10-26T03:30:00Z")))
        );
        assert!(s.trading_at(at("2026-10-26T02:29:59Z")));
        assert!(!s.trading_at(at("2026-10-26T02:30:00Z")));
        assert!(s.trading_at(at("2026-10-26T03:30:00Z")));
        // ASX 10:00 Sydney is AEDT (UTC+11) in October: 23:00 UTC the day before.
        let au = rule("AUS200", AssetClass::EquityIndex);
        let s = session_on(&au, date(2026, 10, 26)).unwrap();
        assert_eq!(s.open, at("2026-10-25T23:00:00Z"));
        assert_eq!(s.close, at("2026-10-26T05:00:00Z"));
        // In June (AEST, UTC+10): 00:00 UTC.
        let june = session_on(&au, date(2026, 6, 15)).unwrap();
        assert_eq!(june.open, at("2026-06-15T00:00:00Z"));
        // Monday's AUS200 session is the one open at Sunday 23:30 UTC.
        assert_eq!(
            session_at(&au, at("2026-10-25T23:30:00Z")).unwrap().date,
            date(2026, 10, 26)
        );
        assert_eq!(
            next_session(&au, at("2026-10-25T03:00:00Z")).unwrap().open,
            at("2026-10-25T23:00:00Z")
        );
    }

    #[test]
    fn fx_metals_energy_and_crypto() {
        for (n, c) in [
            ("EURUSD", AssetClass::Fx),
            ("XAUUSD", AssetClass::Metal),
            ("SpotCrude", AssetClass::Energy),
        ] {
            let r = rule(n, c);
            let s = session_on(&r, date(2026, 10, 26)).unwrap();
            assert_eq!(
                (s.open, s.close),
                (at("2026-10-26T07:00:00Z"), at("2026-10-26T20:00:00Z"))
            );
            assert!(session_on(&r, date(2026, 10, 25)).is_none(), "Sunday");
        }
        let btc = rule("BTCUSD", AssetClass::Crypto);
        let s = session_on(&btc, date(2026, 10, 25)).unwrap();
        assert_eq!(
            (s.open, s.close),
            (at("2026-10-25T00:00:00Z"), at("2026-10-26T00:00:00Z"))
        );
        assert_eq!(
            session_at(&btc, at("2026-10-25T23:59:59Z")).unwrap().date,
            date(2026, 10, 25)
        );
        assert_eq!(
            session_at(&btc, at("2026-10-26T00:00:00Z")).unwrap().date,
            date(2026, 10, 26)
        );
        // An index the rules do not name is not traded.
        assert!(rule_for("HK50", AssetClass::EquityIndex).is_none());
    }

    #[test]
    fn the_sessions_before_skip_weekends_newest_first() {
        let us = rule("US500", AssetClass::EquityIndex);
        let s = sessions_before(&us, at("2026-11-02T14:00:00Z"), 3);
        let dates: Vec<NaiveDate> = s.iter().map(|x| x.date).collect();
        assert_eq!(
            dates,
            vec![date(2026, 10, 30), date(2026, 10, 29), date(2026, 10, 28)]
        );
        // A session still open is not "before".
        let s = sessions_before(&us, at("2026-10-30T15:00:00Z"), 1);
        assert_eq!(s[0].date, date(2026, 10, 29));
        assert_eq!(
            sessions_before(&us, at("2026-11-02T14:00:00Z"), 14).len(),
            14
        );
    }
}
