//! Was the market open? From the symbol's own trading schedule
//! (`ProtoOASymbol.schedule`, `holiday`, `scheduleTimeZone`).
//!
//! Spotware: an interval's seconds count "from SUNDAY 00:00 in specified time
//! zone", start inclusive, end exclusive; a holiday's date is days since
//! 1970-01-01, its seconds count from 00:00 of that day.
//!
//! Only a UTC schedule is evaluated here (there is no time-zone database in
//! this program). For any other zone the state is `Unknown`, with that reason:
//! it is never guessed.

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval {
    pub start_second: u32,
    pub end_second: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holiday {
    pub name: String,
    pub schedule_time_zone: String,
    pub date_days: i64,
    pub recurring: bool,
    pub start_second: Option<i32>,
    pub end_second: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "reason", rename_all = "snake_case")]
pub enum MarketState {
    Open,
    Closed(String),
    Unknown(String),
}

impl MarketState {
    /// `Some(true)` open, `Some(false)` closed, `None` unknown.
    pub fn is_open(&self) -> Option<bool> {
        match self {
            MarketState::Open => Some(true),
            MarketState::Closed(_) => Some(false),
            MarketState::Unknown(_) => None,
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            MarketState::Open => None,
            MarketState::Closed(r) | MarketState::Unknown(r) => Some(r),
        }
    }
}

fn is_utc(tz: &str) -> bool {
    matches!(
        tz,
        "UTC" | "Etc/UTC" | "GMT" | "Etc/GMT" | "Universal" | "Zulu"
    )
}

/// The market state at `at` from the schedule as the broker gives it.
pub fn market_state(
    intervals: &[Interval],
    holidays: &[Holiday],
    schedule_time_zone: Option<&str>,
    at: DateTime<Utc>,
) -> MarketState {
    let Some(tz) = schedule_time_zone.filter(|t| !t.is_empty()) else {
        return MarketState::Unknown("the broker gives no schedule time zone".into());
    };
    if !is_utc(tz) {
        return MarketState::Unknown(format!("schedule time zone {tz} is not evaluated"));
    }
    if intervals.is_empty() {
        return MarketState::Unknown("the broker gives no trading schedule".into());
    }
    let day = at.date_naive();
    let second_of_day = at.num_seconds_from_midnight() as i64;
    for h in holidays {
        if !is_utc(&h.schedule_time_zone) {
            return MarketState::Unknown(format!(
                "holiday {} uses time zone {}, not evaluated",
                h.name, h.schedule_time_zone
            ));
        }
        let Some(hday) = NaiveDate::from_ymd_opt(1970, 1, 1)
            .and_then(|e| e.checked_add_signed(chrono::Duration::days(h.date_days)))
        else {
            continue;
        };
        let same_day = if h.recurring {
            hday.month() == day.month() && hday.day() == day.day()
        } else {
            hday == day
        };
        let start = h.start_second.unwrap_or(0) as i64;
        let end = h.end_second.unwrap_or(86_400) as i64;
        if same_day && second_of_day >= start && second_of_day < end {
            return MarketState::Closed(format!("holiday: {}", h.name));
        }
    }
    let week_second = at.weekday().num_days_from_sunday() as i64 * 86_400 + second_of_day;
    if intervals
        .iter()
        .any(|i| week_second >= i.start_second as i64 && week_second < i.end_second as i64)
    {
        MarketState::Open
    } else {
        MarketState::Closed("outside the trading schedule".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// FX-like week, UTC: Sunday 22:00 to Friday 22:00.
    fn fx() -> Vec<Interval> {
        vec![Interval {
            start_second: 22 * 3600,
            end_second: 5 * 86_400 + 22 * 3600,
        }]
    }

    #[test]
    fn saturday_is_closed_and_wednesday_open() {
        // 24-10-2026 is a Saturday.
        let sat = Utc.with_ymd_and_hms(2026, 10, 24, 12, 0, 0).unwrap();
        assert_eq!(
            market_state(&fx(), &[], Some("UTC"), sat),
            MarketState::Closed("outside the trading schedule".into())
        );
        let wed = Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap();
        assert_eq!(
            market_state(&fx(), &[], Some("UTC"), wed),
            MarketState::Open
        );
        // Friday 22:00 is the exclusive end.
        let fri = Utc.with_ymd_and_hms(2026, 10, 23, 22, 0, 0).unwrap();
        assert_eq!(
            market_state(&fx(), &[], Some("UTC"), fri).is_open(),
            Some(false)
        );
    }

    #[test]
    fn holidays_close_the_market_and_other_zones_are_not_guessed() {
        let christmas = Utc.with_ymd_and_hms(2025, 12, 25, 12, 0, 0).unwrap();
        let days = (NaiveDate::from_ymd_opt(2019, 12, 25).unwrap()
            - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
        .num_days();
        let h = Holiday {
            name: "Christmas".into(),
            schedule_time_zone: "UTC".into(),
            date_days: days,
            recurring: true,
            start_second: None,
            end_second: None,
        };
        assert_eq!(
            market_state(&fx(), &[h], Some("UTC"), christmas),
            MarketState::Closed("holiday: Christmas".into())
        );
        let s = market_state(&fx(), &[], Some("America/New_York"), christmas);
        assert_eq!(s.is_open(), None);
        assert!(s.reason().unwrap().contains("not evaluated"));
        assert_eq!(
            market_state(&[], &[], Some("UTC"), christmas).is_open(),
            None
        );
        assert_eq!(market_state(&fx(), &[], None, christmas).is_open(), None);
    }
}
