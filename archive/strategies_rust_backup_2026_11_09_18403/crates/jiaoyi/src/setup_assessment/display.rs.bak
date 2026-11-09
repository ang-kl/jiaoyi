//! Display eligibility only. Browser activity never controls account observation
//! or trade management, and an open market is not permission to open a trade.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Offset, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::Serialize;

use super::evaluate::{Input, MAX_BOOK_AGE_SECS, MAX_QUOTE_AGE_SECS};

#[derive(Clone, Debug, Serialize)]
pub struct LiveUpdate {
    pub eligible: bool,
    /// Freshness lease for painting position data, not a polling permission.
    pub valid_until: Option<DateTime<Utc>>,
    /// A verified current broker-open window may still be polled for fresh
    /// evidence while the previous quote or book is stale.
    pub market_open_until: Option<DateTime<Utc>>,
    pub reason: &'static str,
}

fn paused(reason: &'static str, market_open_until: Option<DateTime<Utc>>) -> LiveUpdate {
    LiveUpdate {
        eligible: false,
        valid_until: None,
        market_open_until,
        reason,
    }
}

// UTC instants map unambiguously to local clocks during DST folds and gaps.
// Cache only the next offset boundary, never a market-open verdict. The exact
// second-resolution scan is bounded to UTC midnight and runs once per zone/window,
// not once per position/tick; cache size is bounded by the finite IANA Tz enum.
fn zone_lease(zone: Tz, now: DateTime<Utc>) -> Result<DateTime<Utc>, &'static str> {
    type Window = (DateTime<Utc>, DateTime<Utc>);
    static WINDOWS: OnceLock<Mutex<HashMap<Tz, Window>>> = OnceLock::new();
    let windows = WINDOWS.get_or_init(Default::default);
    let cached = windows
        .lock()
        .unwrap()
        .get(&zone)
        .copied()
        .filter(|(from, until)| *from <= now && now < *until);
    if let Some((_, until)) = cached {
        return Ok(until);
    }
    let end = now
        .date_naive()
        .succ_opt()
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .ok_or("market_schedule_unknown")?
        .and_utc();
    let offset = zone.offset_from_utc_datetime(&now.naive_utc()).fix();
    let mut until = end;
    for seconds in (now.timestamp() + 1)..=end.timestamp() {
        let at = DateTime::from_timestamp(seconds, 0).ok_or("market_schedule_unknown")?;
        if zone.offset_from_utc_datetime(&at.naive_utc()).fix() != offset {
            until = at;
            break;
        }
    }
    windows.lock().unwrap().insert(zone, (now, until));
    Ok(until)
}

fn market_window(input: &Input, symbol: i64) -> Result<DateTime<Utc>, &'static str> {
    let info = input
        .symbols
        .get(&symbol)
        .ok_or("market_metadata_unavailable")?;
    let s = &info.symbol;
    let zone: Tz = s
        .schedule_time_zone
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or("market_schedule_unknown")?;
    if s.schedule.is_empty()
        || s.schedule
            .iter()
            .any(|i| i.start_second >= i.end_second || i.end_second > 604_800)
    {
        return Err("market_schedule_unknown");
    }
    let local = input.now.with_timezone(&zone);
    let second = local.num_seconds_from_midnight();
    let week_second = local.weekday().num_days_from_sunday() * 86_400 + second;
    let remaining = s
        .schedule
        .iter()
        .filter(|i| i.start_second <= week_second && week_second < i.end_second)
        .map(|i| i.end_second - week_second)
        .min()
        .ok_or("market_closed")?;
    let base = input
        .now
        .with_nanosecond(0)
        .ok_or("market_schedule_unknown")?;
    let boundary = |seconds: u32| {
        base.checked_add_signed(Duration::seconds(seconds.into()))
            .ok_or("market_schedule_unknown")
    };
    let stable = zone_lease(zone, input.now)?;
    let mut until = stable
        .min(boundary(remaining)?)
        .min(boundary(86_400 - second)?);
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let utc_day = input.now.date_naive();
    for holiday in &s.holiday {
        let holiday_day = Duration::try_days(holiday.holiday_date)
            .and_then(|d| epoch.checked_add_signed(d))
            .ok_or("market_schedule_unknown")?;
        let same_date = |day: NaiveDate| {
            if holiday.is_recurring {
                holiday_day.month() == day.month() && holiday_day.day() == day.day()
            } else {
                holiday_day == day
            }
        };
        // Every supported IANA local date is within one date of UTC. Historical
        // nonrecurring records cannot invalidate today's display (the broker
        // retains past closed-day records with zero-length windows).
        if ![utc_day.pred_opt(), Some(utc_day), utc_day.succ_opt()]
            .into_iter()
            .flatten()
            .any(same_date)
        {
            continue;
        }
        let holiday_zone: Tz = holiday
            .schedule_time_zone
            .parse()
            .map_err(|_| "market_schedule_unknown")?;
        let holiday_local = input.now.with_timezone(&holiday_zone);
        let holiday_second = holiday_local.num_seconds_from_midnight();
        let stable = zone_lease(holiday_zone, input.now)?;
        // Reassess when this holiday's own local date/offset changes, even if it
        // is tomorrow there. Its timezone need not match the weekly schedule.
        until = until.min(stable).min(boundary(86_400 - holiday_second)?);
        if !same_date(holiday_local.date_naive()) {
            continue;
        }
        let start = holiday.start_second.unwrap_or(0);
        let end = holiday.end_second.unwrap_or(86_400);
        // A current zero-length/inverted record is ambiguous: do not infer that
        // zero means either an open day or a full-day closure.
        if start < 0 || start >= end || end > 86_400 {
            return Err("market_schedule_unknown");
        }
        let start = start as u32;
        let end = end as u32;
        if start <= holiday_second && holiday_second < end {
            return Err("market_closed");
        }
        if holiday_second < start {
            until = until.min(boundary(start - holiday_second)?);
        }
    }
    Ok(until)
}

pub fn eligibility(kind: &str, input: &Input, symbol: i64) -> LiveUpdate {
    if kind != "position" {
        return paused("not_an_open_position", None);
    }
    let market_open_until = match market_window(input, symbol) {
        Ok(until) => until,
        Err(reason) => return paused(reason, None),
    };
    let Some(book_at) = input.book_at.filter(|at| {
        input.connected
            && input.complete
            && *at <= input.now
            && input.now.signed_duration_since(*at) <= Duration::seconds(MAX_BOOK_AGE_SECS)
    }) else {
        return paused("book_unverified_or_stale", Some(market_open_until));
    };
    let quote = input.quotes.get(&symbol);
    if super::evaluate::executable(quote, input.now).is_none() {
        return paused("quote_unverified_or_stale", Some(market_open_until));
    }
    let quote = quote.unwrap();
    // executable() proved both timestamps exist and both sides are fresh.
    let quote_until =
        quote.bid_at.unwrap().min(quote.ask_at.unwrap()) + Duration::seconds(MAX_QUOTE_AGE_SECS);
    let until = market_open_until
        .min(input.now + Duration::seconds(15))
        .min(book_at + Duration::seconds(MAX_BOOK_AGE_SECS))
        .min(quote_until);
    if until <= input.now {
        return paused("display_evidence_expired", Some(market_open_until));
    }
    LiveUpdate {
        eligible: true,
        valid_until: Some(until),
        market_open_until: Some(market_open_until),
        reason: "open_market_with_current_position_evidence",
    }
}

#[cfg(test)]
mod tests {
    use super::super::evaluate::{Quote, SymbolInfo};
    use super::*;
    use jiaoyi_ctrader::messages as pb;

    fn input(at: &str) -> Input {
        let mut i = super::super::State::new(7, 1).input;
        i.now = at.parse().unwrap();
        i.book_at = Some(i.now);
        i.complete = true;
        i.quotes.insert(
            9,
            Quote {
                bid: Some(100.0),
                ask: Some(101.0),
                bid_at: Some(i.now),
                ask_at: Some(i.now),
            },
        );
        i.symbols.insert(
            9,
            SymbolInfo {
                symbol: pb::ProtoOaSymbol {
                    symbol_id: 9,
                    schedule: vec![pb::ProtoOaInterval {
                        start_second: 0,
                        end_second: 604_800,
                    }],
                    schedule_time_zone: Some("UTC".into()),
                    ..Default::default()
                },
                light: pb::ProtoOaLightSymbol {
                    symbol_id: 9,
                    enabled: Some(true),
                    ..Default::default()
                },
                quote_to_deposit: Some(1.0),
                conversion_at: Some(i.now),
            },
        );
        i
    }

    #[test]
    fn existing_close_only_positions_can_update_but_pending_entries_cannot() {
        let mut i = input("2026-11-01T12:00:00Z");
        i.symbols.get_mut(&9).unwrap().symbol.trading_mode = Some(3);
        let live = eligibility("position", &i, 9);
        assert!(live.eligible);
        assert_eq!(live.valid_until, Some(i.now + Duration::seconds(6)));
        assert_eq!(
            live.market_open_until,
            Some("2026-11-02T00:00:00Z".parse().unwrap())
        );
        let pending = eligibility("pending_entry", &i, 9);
        assert!(!pending.eligible);
        assert!(pending.market_open_until.is_none());
    }

    #[test]
    fn stale_disconnected_partial_and_invalid_quotes_do_not_paint_live_values() {
        let base = input("2026-11-01T12:00:00Z");
        for case in [
            "disconnect",
            "partial",
            "stale_book",
            "future_book",
            "stale_bid",
            "missing_ask",
            "crossed",
            "nonfinite",
        ] {
            let mut i = base.clone();
            match case {
                "disconnect" => i.connected = false,
                "partial" => i.complete = false,
                "stale_book" => i.book_at = Some(i.now - Duration::seconds(121)),
                "future_book" => i.book_at = Some(i.now + Duration::seconds(1)),
                "stale_bid" => {
                    i.quotes.get_mut(&9).unwrap().bid_at = Some(i.now - Duration::seconds(7))
                }
                "missing_ask" => i.quotes.get_mut(&9).unwrap().ask = None,
                "crossed" => i.quotes.get_mut(&9).unwrap().bid = Some(102.0),
                "nonfinite" => i.quotes.get_mut(&9).unwrap().ask = Some(f64::NAN),
                _ => unreachable!(),
            }
            let live = eligibility("position", &i, 9);
            assert!(!live.eligible, "{case}");
            assert!(live.valid_until.is_none(), "{case}");
            assert!(
                live.market_open_until.is_some(),
                "{case}: open market can obtain fresh evidence"
            );
        }
    }

    #[test]
    fn leases_stop_at_close_holiday_midnight_and_evidence_expiry() {
        let mut i = input("2026-11-01T12:00:00Z");
        i.symbols.get_mut(&9).unwrap().symbol.schedule[0].end_second = 43_202;
        assert_eq!(
            eligibility("position", &i, 9).valid_until,
            Some(i.now + Duration::seconds(2))
        );
        i.now += Duration::seconds(2);
        assert_eq!(eligibility("position", &i, 9).reason, "market_closed");

        let i = input("2026-11-01T23:59:59Z");
        assert_eq!(
            eligibility("position", &i, 9).valid_until,
            Some(i.now + Duration::seconds(1))
        );
        let mut i = input("2026-11-01T12:00:00Z");
        i.book_at = Some(i.now - Duration::seconds(119));
        assert_eq!(
            eligibility("position", &i, 9).valid_until,
            Some(i.now + Duration::seconds(1))
        );
        i.book_at = Some(i.now);
        i.quotes.get_mut(&9).unwrap().ask_at = Some(i.now - Duration::seconds(5));
        assert_eq!(
            eligibility("position", &i, 9).valid_until,
            Some(i.now + Duration::seconds(1))
        );

        let mut i = input("2026-11-01T12:00:00Z");
        let holiday_day = (NaiveDate::from_ymd_opt(2020, 11, 1).unwrap()
            - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
        .num_days();
        i.symbols
            .get_mut(&9)
            .unwrap()
            .symbol
            .holiday
            .push(pb::ProtoOaHoliday {
                name: "session break".into(),
                schedule_time_zone: "UTC".into(),
                holiday_date: holiday_day,
                is_recurring: true,
                start_second: Some(43_202),
                end_second: Some(43_260),
                ..Default::default()
            });
        let live = eligibility("position", &i, 9);
        assert_eq!(live.market_open_until, Some(i.now + Duration::seconds(2)));
        assert_eq!(live.valid_until, live.market_open_until);
        i.now += Duration::seconds(2);
        assert_eq!(eligibility("position", &i, 9).reason, "market_closed");
    }

    #[test]
    fn unknown_or_malformed_market_metadata_never_opens_a_polling_window() {
        let base = input("2026-11-01T12:00:00Z");
        for case in ["missing", "timezone", "empty", "reversed", "oversized"] {
            let mut i = base.clone();
            match case {
                "missing" => {
                    i.symbols.clear();
                }
                "timezone" => {
                    i.symbols.get_mut(&9).unwrap().symbol.schedule_time_zone =
                        Some("Invalid/Timezone".into())
                }
                "empty" => i.symbols.get_mut(&9).unwrap().symbol.schedule.clear(),
                "reversed" => {
                    i.symbols.get_mut(&9).unwrap().symbol.schedule[0].start_second = 604_800
                }
                "oversized" => {
                    i.symbols.get_mut(&9).unwrap().symbol.schedule[0].end_second = 604_801
                }
                _ => unreachable!(),
            }
            let live = eligibility("position", &i, 9);
            assert!(!live.eligible, "{case}");
            assert!(live.market_open_until.is_none(), "{case}");
        }
    }

    fn ny_input(at: &str) -> Input {
        let mut i = input(at);
        i.symbols.get_mut(&9).unwrap().symbol.schedule_time_zone = Some("America/New_York".into());
        i
    }

    fn holiday(day: &str, zone: &str, start: Option<i32>, end: Option<i32>) -> pb::ProtoOaHoliday {
        let day = NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
        pb::ProtoOaHoliday {
            name: "test broker holiday".into(),
            schedule_time_zone: zone.into(),
            holiday_date: (day - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()).num_days(),
            is_recurring: false,
            start_second: start,
            end_second: end,
            ..Default::default()
        }
    }

    #[test]
    fn new_york_schedule_uses_the_local_week_and_current_dst_offset() {
        for (at, expected) in [
            ("2026-11-01T22:59:59Z", false), // Sunday 17:59:59 EST
            ("2026-11-01T23:00:00Z", true),  // Sunday 18:00 EST
            ("2026-07-05T21:59:59Z", false), // Sunday 17:59:59 EDT
            ("2026-07-05T22:00:00Z", true),  // Sunday 18:00 EDT
            ("2026-11-02T21:59:00Z", false), // Monday 16:59 EST, exclusive end
        ] {
            let mut i = ny_input(at);
            i.symbols.get_mut(&9).unwrap().symbol.schedule = vec![pb::ProtoOaInterval {
                start_second: 64_800,
                end_second: 147_540,
            }];
            assert_eq!(eligibility("position", &i, 9).eligible, expected, "{at}");
        }
        // Changes are display-only: the execution calendar intentionally still
        // returns Unknown for this timezone, and retains its existing policy.
        let i = ny_input("2026-11-01T23:30:00Z");
        assert!(matches!(
            crate::loader::market_state_of(&i.symbols[&9].symbol, i.now),
            jiaoyi_core::schedule::MarketState::Unknown(_)
        ));
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-02T00:00:00Z".parse().unwrap())
        );
    }

    #[test]
    fn fold_and_gap_cut_leases_before_the_clock_jumps_without_guessing_boundaries() {
        let mut fold = ny_input("2026-11-01T05:59:58Z");
        fold.symbols.get_mut(&9).unwrap().symbol.schedule = vec![pb::ProtoOaInterval {
            start_second: 5_400, // Sunday 01:30
            end_second: 9_000,   // 02:30 (unique, after the fold)
        }];
        assert_eq!(
            market_window(&fold, 9),
            Ok("2026-11-01T06:00:00Z".parse().unwrap())
        );
        fold.now = "2026-11-01T06:00:00Z".parse().unwrap();
        assert_eq!(market_window(&fold, 9), Err("market_closed")); // clock returns to 01:00
        fold.now = "2026-11-01T06:30:00Z".parse().unwrap();
        assert_eq!(
            market_window(&fold, 9),
            Ok("2026-11-01T07:30:00Z".parse().unwrap())
        );
        fold.now = "2026-11-01T07:30:00Z".parse().unwrap();
        assert_eq!(market_window(&fold, 9), Err("market_closed"));

        // The spring 02:30 boundary does not exist. The lease ends at the
        // transition, and the post-jump clock is evaluated afresh.
        let mut gap = ny_input("2026-03-08T06:59:58Z");
        gap.symbols.get_mut(&9).unwrap().symbol.schedule = fold.symbols[&9].symbol.schedule.clone();
        assert_eq!(
            market_window(&gap, 9),
            Ok("2026-03-08T07:00:00Z".parse().unwrap())
        );
        gap.now = "2026-03-08T07:00:00Z".parse().unwrap();
        assert_eq!(market_window(&gap, 9), Err("market_closed"));
        gap.symbols.get_mut(&9).unwrap().symbol.schedule[0].end_second = 12_600;
        assert_eq!(
            market_window(&gap, 9),
            Ok("2026-03-08T07:30:00Z".parse().unwrap())
        );
    }

    #[test]
    fn repeated_local_end_and_backward_observation_do_not_reuse_a_later_offset_window() {
        let mut i = ny_input("2026-11-01T06:29:59Z");
        i.symbols.get_mut(&9).unwrap().symbol.schedule = vec![pb::ProtoOaInterval {
            start_second: 3_600,
            end_second: 5_400,
        }];
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-01T06:30:00Z".parse().unwrap())
        );
        i.now = "2026-11-01T05:29:59Z".parse().unwrap();
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-01T05:30:00Z".parse().unwrap())
        );
        i.now = "2026-11-01T05:30:00Z".parse().unwrap();
        assert_eq!(market_window(&i, 9), Err("market_closed"));
        // Cache stores offsets only: changing the broker schedule is immediate.
        i.symbols.get_mut(&9).unwrap().symbol.schedule[0].end_second = 9_000;
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-01T06:00:00Z".parse().unwrap())
        );
    }

    #[test]
    fn holidays_use_their_own_timezone_date_and_stop_before_future_closure() {
        let mut i = ny_input("2026-11-01T23:33:30Z"); // Nov1 in NY, Nov2 in Moscow
        i.symbols.get_mut(&9).unwrap().symbol.holiday = vec![holiday(
            "2026-11-02",
            "Europe/Moscow",
            Some(9_220),
            Some(10_800),
        )];
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-01T23:33:40Z".parse().unwrap())
        );
        i.now = "2026-11-01T23:33:40Z".parse().unwrap();
        assert_eq!(market_window(&i, 9), Err("market_closed"));
        i.now = "2026-11-02T00:00:00Z".parse().unwrap(); // 03:00 Moscow, exclusive end
        assert!(market_window(&i, 9).is_ok());
        i.symbols.get_mut(&9).unwrap().symbol.holiday[0].start_second = None;
        i.symbols.get_mut(&9).unwrap().symbol.holiday[0].end_second = None;
        assert_eq!(market_window(&i, 9), Err("market_closed"));

        i.now = "2026-11-01T20:59:58Z".parse().unwrap(); // still Nov1 in Moscow
        assert_eq!(
            market_window(&i, 9),
            Ok("2026-11-01T21:00:00Z".parse().unwrap())
        );
        i.now = "2026-11-01T21:00:00Z".parse().unwrap();
        assert_eq!(market_window(&i, 9), Err("market_closed"));
    }

    #[test]
    fn irrelevant_holidays_do_not_poison_today_but_applicable_ambiguity_fails_closed() {
        let mut i = ny_input("2026-11-01T23:33:30Z");
        i.symbols.get_mut(&9).unwrap().symbol.holiday = vec![
            holiday("2025-12-25", "Europe/Bucharest", Some(0), Some(0)),
            holiday("2026-01-01", "Europe/Bucharest", Some(0), Some(0)),
            holiday("2025-12-24", "Invalid/Zone", Some(-1), Some(100_000)),
        ];
        assert!(eligibility("position", &i, 9).eligible);
        let mut recurring = holiday("2020-11-02", "Europe/Moscow", None, None);
        recurring.is_recurring = true;
        i.symbols
            .get_mut(&9)
            .unwrap()
            .symbol
            .holiday
            .push(recurring);
        assert_eq!(market_window(&i, 9), Err("market_closed"));
        i.symbols.get_mut(&9).unwrap().symbol.holiday.pop();
        i.symbols.get_mut(&9).unwrap().symbol.holiday.push(holiday(
            "2026-11-02",
            "Europe/Moscow",
            Some(0),
            Some(0),
        ));
        assert_eq!(market_window(&i, 9), Err("market_schedule_unknown"));
        i.symbols
            .get_mut(&9)
            .unwrap()
            .symbol
            .holiday
            .last_mut()
            .unwrap()
            .end_second = Some(86_400);
        assert_eq!(market_window(&i, 9), Err("market_closed"));
        i.symbols
            .get_mut(&9)
            .unwrap()
            .symbol
            .holiday
            .last_mut()
            .unwrap()
            .schedule_time_zone = "Invalid/Zone".into();
        assert_eq!(market_window(&i, 9), Err("market_schedule_unknown"));
    }
}
