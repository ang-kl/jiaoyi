//! Final entry-only broker calendar proof for owner revision №18,392.
//!
//! Built from the actual account's SymbolById response in background admission.
//! The writer checks its current wall clock against these intervals without I/O.
//! Strategy evaluation windows never imply that the broker is open.
use crate::{messages::ProtoOaSymbol, trading::Side};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("{code}: {reason}")]
pub struct CalendarRefusal {
    pub code: &'static str,
    pub reason: &'static str,
}
fn refusal(code: &'static str, reason: &'static str) -> CalendarRefusal {
    CalendarRefusal { code, reason }
}
fn serialize_tz<S: Serializer>(tz: &Tz, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(tz.name())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Holiday {
    id: i64,
    #[serde(serialize_with = "serialize_tz")]
    timezone: Tz,
    date: NaiveDate,
    recurring: bool,
    start_second: u32,
    end_second: u32,
}

/// Account scope, connection generation, invalidation revision and receipt age
/// are supplied by the enclosing prepared-entry guard. This is the immutable
/// symbol contract component of that proof, not a global per-symbol calendar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractCalendar {
    pub symbol_id: i64,
    #[serde(serialize_with = "serialize_tz")]
    timezone: Tz,
    weekly_intervals: Vec<(u32, u32)>,
    holidays: Vec<Holiday>,
    trading_mode: i32,
    short_selling_enabled: bool,
}

impl ContractCalendar {
    pub fn from_symbol(symbol: &ProtoOaSymbol) -> Result<Self, CalendarRefusal> {
        let bad = || {
            refusal(
                "ENTRY_BROKER_CALENDAR_INVALID",
                "broker calendar metadata is missing or invalid",
            )
        };
        if symbol.symbol_id <= 0 {
            return Err(bad());
        }
        let timezone: Tz = symbol
            .schedule_time_zone
            .as_deref()
            .ok_or_else(bad)?
            .parse()
            .map_err(|_| bad())?;
        if symbol.schedule.is_empty() {
            return Err(bad());
        }
        let mut weekly_intervals = Vec::new();
        for interval in &symbol.schedule {
            if interval.start_second >= interval.end_second || interval.end_second > 7 * 86400 {
                return Err(bad());
            }
            weekly_intervals.push((interval.start_second, interval.end_second));
        }
        weekly_intervals.sort_unstable();
        if weekly_intervals
            .windows(2)
            .any(|pair| pair[1].0 < pair[0].1)
        {
            return Err(bad());
        }
        let mut holidays = Vec::new();
        for holiday in &symbol.holiday {
            let timezone = holiday.schedule_time_zone.parse().map_err(|_| bad())?;
            let date = NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .checked_add_signed(Duration::try_days(holiday.holiday_date).ok_or_else(bad)?)
                .ok_or_else(bad)?;
            // The captured broker contract represents all-day closures as 0/0.
            // Absent boundaries are also a whole holiday day; partially missing
            // or overnight interval encodings remain explicitly unsupported.
            let (start_second, end_second) = match (holiday.start_second, holiday.end_second) {
                (None, None) | (Some(0), Some(0)) => (0, 86400),
                (Some(start), Some(end)) if start >= 0 && end > start && end <= 86400 => {
                    (start as u32, end as u32)
                }
                _ => return Err(bad()),
            };
            holidays.push(Holiday {
                id: holiday.holiday_id,
                timezone,
                date,
                recurring: holiday.is_recurring,
                start_second,
                end_second,
            });
        }
        // ENABLED is the declared protobuf default when tradingMode is absent.
        let trading_mode = symbol.trading_mode.unwrap_or(0);
        if !(0..=3).contains(&trading_mode) {
            return Err(bad());
        }
        Ok(Self {
            symbol_id: symbol.symbol_id,
            timezone,
            weekly_intervals,
            holidays,
            trading_mode,
            short_selling_enabled: symbol.enable_short_selling.unwrap_or(false),
        })
    }

    pub fn validate(&self, at: DateTime<Utc>, side: Side) -> Result<(), CalendarRefusal> {
        if self.trading_mode != 0 {
            return Err(refusal(
                "ENTRY_BROKER_TRADING_DISABLED",
                "broker contract does not permit new entries",
            ));
        }
        if side == Side::Sell && !self.short_selling_enabled {
            return Err(refusal(
                "ENTRY_BROKER_SHORT_DISABLED",
                "broker contract does not permit short selling",
            ));
        }
        let local = at.with_timezone(&self.timezone);
        let second =
            local.weekday().num_days_from_sunday() * 86400 + local.num_seconds_from_midnight();
        if !self
            .weekly_intervals
            .iter()
            .any(|(start, end)| *start <= second && second < *end)
        {
            return Err(refusal(
                "ENTRY_BROKER_MARKET_CLOSED",
                "outside the account's broker trading schedule",
            ));
        }
        for holiday in &self.holidays {
            let local = at.with_timezone(&holiday.timezone);
            let date = local.date_naive();
            let same_date = if holiday.recurring {
                (date.month(), date.day()) == (holiday.date.month(), holiday.date.day())
            } else {
                date == holiday.date
            };
            let second = local.num_seconds_from_midnight();
            // Include the holiday's final declared second. This also prevents
            // an accidental one-second reopen for broker endSecond=86399.
            if same_date && holiday.start_second <= second && second <= holiday.end_second {
                return Err(refusal(
                    "ENTRY_BROKER_HOLIDAY_CLOSED",
                    "broker holiday or exceptional closure is active",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{ProtoOaHoliday, ProtoOaInterval};
    fn at(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }
    fn contract() -> ProtoOaSymbol {
        ProtoOaSymbol {
            symbol_id: 22,
            schedule_time_zone: Some("America/New_York".into()),
            schedule: vec![ProtoOaInterval {
                start_second: 0,
                end_second: 7 * 86400,
            }],
            enable_short_selling: Some(true),
            ..Default::default()
        }
    }
    #[test]
    fn actual_account_calendar_enforces_dst_maintenance_and_live_friday_close() {
        let mut symbol = contract();
        // Reproduce the Nov8 captured crypto calendars: daily NY16:59–17:01
        // maintenance; live Friday closes four minutes earlier than demos.
        symbol.schedule = vec![ProtoOaInterval {
            start_second: 0,
            end_second: 16 * 3600 + 59 * 60,
        }];
        for day in 0..7 {
            symbol.schedule.push(ProtoOaInterval {
                start_second: day * 86400 + 17 * 3600 + 60,
                end_second: if day == 6 {
                    604799
                } else {
                    (day + 1) * 86400 + 16 * 3600 + 59 * 60
                },
            });
        }
        let demo = ContractCalendar::from_symbol(&symbol).unwrap();
        symbol.schedule[5].end_second -= 4 * 60;
        let live = ContractCalendar::from_symbol(&symbol).unwrap();
        assert!(live
            .validate(at("2026-11-09T22:00:00Z"), Side::Buy)
            .is_err());
        assert!(live.validate(at("2026-11-09T22:01:00Z"), Side::Buy).is_ok());
        assert!(live
            .validate(at("2026-11-06T21:55:00Z"), Side::Buy)
            .is_err());
        assert!(demo.validate(at("2026-11-06T21:55:00Z"), Side::Buy).is_ok());
        assert!(demo
            .validate(at("2026-11-06T21:59:00Z"), Side::Buy)
            .is_err());
        assert!(live.validate(at("2026-11-07T12:00:00Z"), Side::Buy).is_ok());
        // Before US DST ends, the same local Monday17:01 opens at21:01UTC.
        assert!(live
            .validate(at("2026-10-26T21:00:00Z"), Side::Buy)
            .is_err());
        assert!(live.validate(at("2026-10-26T21:01:00Z"), Side::Buy).is_ok());
    }
    #[test]
    fn holidays_use_their_own_timezone_and_contract_modes_fail_closed() {
        let mut symbol = contract();
        let date = NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let day = (date - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()).num_days();
        symbol.holiday.push(ProtoOaHoliday {
            holiday_id: 1,
            name: "Closed".into(),
            schedule_time_zone: "Europe/Moscow".into(),
            holiday_date: day,
            is_recurring: false,
            start_second: Some(0),
            end_second: Some(0),
            ..Default::default()
        });
        let calendar = ContractCalendar::from_symbol(&symbol).unwrap();
        assert!(calendar
            .validate(at("2026-11-08T20:59:59Z"), Side::Buy)
            .is_ok());
        assert_eq!(
            calendar
                .validate(at("2026-11-08T21:00:00Z"), Side::Buy)
                .unwrap_err()
                .code,
            "ENTRY_BROKER_HOLIDAY_CLOSED"
        );
        assert!(calendar
            .validate(at("2026-11-09T20:59:59Z"), Side::Buy)
            .is_err());
        assert!(calendar
            .validate(at("2026-11-09T21:00:00Z"), Side::Buy)
            .is_ok());
        // Partial recurring closures retain their own local clock and year rule.
        symbol.holiday[0].holiday_date -= 365;
        symbol.holiday[0].is_recurring = true;
        symbol.holiday[0].start_second = Some(13 * 3600);
        symbol.holiday[0].end_second = Some(14 * 3600);
        let partial = ContractCalendar::from_symbol(&symbol).unwrap();
        assert!(partial
            .validate(at("2026-11-09T09:59:59Z"), Side::Buy)
            .is_ok());
        assert!(partial
            .validate(at("2026-11-09T10:00:00Z"), Side::Buy)
            .is_err());
        assert!(partial
            .validate(at("2026-11-09T11:00:01Z"), Side::Buy)
            .is_ok());
        symbol.holiday.clear();
        symbol.trading_mode = Some(3);
        assert!(ContractCalendar::from_symbol(&symbol)
            .unwrap()
            .validate(at("2026-11-09T12:00:00Z"), Side::Buy)
            .is_err());
        symbol.trading_mode = Some(0);
        symbol.enable_short_selling = Some(false);
        assert!(ContractCalendar::from_symbol(&symbol)
            .unwrap()
            .validate(at("2026-11-09T12:00:00Z"), Side::Sell)
            .is_err());
        symbol.schedule_time_zone = Some("unverified".into());
        assert!(ContractCalendar::from_symbol(&symbol).is_err());
    }
}
