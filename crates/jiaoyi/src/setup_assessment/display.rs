//! Display eligibility only. Browser activity never controls account observation
//! or trade management, and an open market is not permission to open a trade.
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use jiaoyi_core::schedule::MarketState;
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

fn market_window(input: &Input, symbol: i64) -> Result<DateTime<Utc>, &'static str> {
    let info = input
        .symbols
        .get(&symbol)
        .ok_or("market_metadata_unavailable")?;
    let s = &info.symbol;
    if s.schedule
        .iter()
        .any(|i| i.start_second >= i.end_second || i.end_second > 604_800)
    {
        return Err("market_schedule_unknown");
    }
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    // Validate external date ranges before calling the shared calendar reader.
    if s.holiday.iter().any(|h| {
        let start = h.start_second.unwrap_or(0);
        let end = h.end_second.unwrap_or(86_400);
        Duration::try_days(h.holiday_date)
            .and_then(|d| epoch.checked_add_signed(d))
            .is_none()
            || start < 0
            || start >= end
            || end > 86_400
    }) {
        return Err("market_schedule_unknown");
    }
    match crate::loader::market_state_of(s, input.now) {
        MarketState::Open => {}
        MarketState::Closed(_) => return Err("market_closed"),
        MarketState::Unknown(_) => return Err("market_schedule_unknown"),
    }
    // The existing schedule evaluator proves UTC here. Stop at midnight so a
    // browser cannot carry today's holiday decision into a different date.
    let day = input.now.date_naive();
    let midnight = day
        .and_hms_opt(0, 0, 0)
        .ok_or("market_schedule_unknown")?
        .and_utc();
    let next_midnight = day
        .succ_opt()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or("market_schedule_unknown")?
        .and_utc();
    let sunday = day
        .checked_sub_signed(Duration::days(day.weekday().num_days_from_sunday().into()))
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or("market_schedule_unknown")?
        .and_utc();
    let week_second =
        day.weekday().num_days_from_sunday() * 86_400 + input.now.num_seconds_from_midnight();
    let end = s
        .schedule
        .iter()
        .filter(|i| i.start_second <= week_second && week_second < i.end_second)
        .map(|i| sunday + Duration::seconds(i.end_second.into()))
        .min()
        .ok_or("market_schedule_unknown")?;
    let mut until = end.min(next_midnight);
    for holiday in &s.holiday {
        let holiday_day = Duration::try_days(holiday.holiday_date)
            .and_then(|d| epoch.checked_add_signed(d))
            .ok_or("market_schedule_unknown")?;
        let start = holiday.start_second.unwrap_or(0);
        let applies = if holiday.is_recurring {
            holiday_day.month() == day.month() && holiday_day.day() == day.day()
        } else {
            holiday_day == day
        };
        let begins = midnight + Duration::seconds(start.into());
        if applies && begins > input.now {
            until = until.min(begins);
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
                        Some("America/New_York".into())
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
}
