//! Decoding of `ProtoOATrendbar` into daily bars.
//!
//! Spotware's model (OpenApiModelMessages.proto, `ProtoOATrendbar`, and the
//! Open API guide "Symbol data", help.ctrader.com/open-api/symbol-data/):
//!
//! - `low` is the bar low; `open = low + deltaOpen`, `high = low + deltaHigh`,
//!   `close = low + deltaClose`;
//! - all four are relative prices: "get the low price of a trendbar and divide
//!   it by 100000. Then, round the result to symbol digits"
//!   (C# sample: `Math.Round(relative / 100000.0, symbol.Digits)`);
//! - `utcTimestampInMinutes` is "the Unix time in minutes of the bar, equal to
//!   the timestamp of the open tick".
//!
//! Prices are kept exact here, as `Decimal` with scale 5 (relative ÷ 100000).
//! Rounding to the symbol's digits is a display step; for a broker-quoted
//! price it changes nothing, and if it ever would, the stored exact value shows it.
//!
//! The deltas are proto2 `optional uint64` with no explicit default, so an
//! absent delta reads as 0, the protocol default (as in Spotware's samples). An
//! absent `low` or timestamp has no safe default: that bar is refused.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::pb::{ProtoOaTrendbar, ProtoOaTrendbarPeriod};

/// Relative prices are in 1/100000 of a price unit.
pub const PRICE_SCALE: u32 = 5;

/// One decoded daily bar. Prices are exact (relative ÷ 100000).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyBar {
    /// Open time of the bar (UTC), from `utcTimestampInMinutes`.
    pub open_time: DateTime<Utc>,
    /// UTC calendar date of `open_time`.
    pub date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    /// `volume`: bar volume in ticks.
    pub tick_volume: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TrendbarError {
    #[error("trendbar has no low price")]
    MissingLow,
    #[error("trendbar has no utcTimestampInMinutes")]
    MissingTimestamp,
    #[error("trendbar period is {0}, expected {1}")]
    WrongPeriod(i32, i32),
    #[error("trendbar price overflows: low {low} + delta {delta}")]
    Overflow { low: i64, delta: u64 },
    #[error("trendbar timestamp {0} minutes is out of range")]
    BadTimestamp(u32),
}

fn relative(low: i64, delta: Option<u64>) -> Result<i64, TrendbarError> {
    let d = delta.unwrap_or(0);
    i64::try_from(d)
        .ok()
        .and_then(|d| low.checked_add(d))
        .ok_or(TrendbarError::Overflow { low, delta: d })
}

/// Decode one D1 trendbar.
pub fn decode_trendbar(tb: &ProtoOaTrendbar) -> Result<DailyBar, TrendbarError> {
    decode_bar(tb, ProtoOaTrendbarPeriod::D1)
}

/// Decode one trendbar of `period` (a bar that names another period is
/// refused). `date` is the UTC date of its open.
pub fn decode_bar(
    tb: &ProtoOaTrendbar,
    period: ProtoOaTrendbarPeriod,
) -> Result<DailyBar, TrendbarError> {
    if let Some(p) = tb.period {
        if p != period as i32 {
            return Err(TrendbarError::WrongPeriod(p, period as i32));
        }
    }
    let low = tb.low.ok_or(TrendbarError::MissingLow)?;
    let minutes = tb
        .utc_timestamp_in_minutes
        .ok_or(TrendbarError::MissingTimestamp)?;
    let open_time = DateTime::<Utc>::from_timestamp(i64::from(minutes) * 60, 0)
        .ok_or(TrendbarError::BadTimestamp(minutes))?;
    let price = |rel: i64| Decimal::new(rel, PRICE_SCALE);
    Ok(DailyBar {
        open_time,
        date: open_time.date_naive(),
        open: price(relative(low, tb.delta_open)?),
        high: price(relative(low, tb.delta_high)?),
        low: price(low),
        close: price(relative(low, tb.delta_close)?),
        tick_volume: tb.volume,
    })
}

/// Keep only closed bars. A D1 bar that opened at `t` is closed once
/// `t + 24 h <= now`; the bar still forming is dropped and returned apart so
/// the caller can count it. Bars are returned sorted by open time.
pub fn closed_daily_bars(
    mut bars: Vec<DailyBar>,
    now: DateTime<Utc>,
) -> (Vec<DailyBar>, Vec<DailyBar>) {
    bars.sort_by_key(|b| b.open_time);
    bars.into_iter()
        .partition(|b| b.open_time + Duration::days(1) <= now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{decode_frame, encode_frame};
    use crate::pb::{ProtoMessage, ProtoOaGetTrendbarsRes, ProtoOaPayloadType};
    use chrono::TimeZone;
    use prost::Message;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    /// One ProtoOATrendbar, encoded by hand from the .proto field numbers so the
    /// test does not depend on the generated encoder:
    ///   volume=100 (field 3), period=D1 (4), low=108512 (5), deltaOpen=30 (6),
    ///   deltaClose=75 (7), deltaHigh=120 (8),
    ///   utcTimestampInMinutes=28487520 (9) = 2024-03-01T00:00:00Z.
    const HAND_TRENDBAR: [u8; 19] = [
        0x18, 0x64, // field 3 varint 100
        0x20, 0x0C, // field 4 varint 12 (D1)
        0x28, 0xE0, 0xCF, 0x06, // field 5 varint 108512
        0x30, 0x1E, // field 6 varint 30
        0x38, 0x4B, // field 7 varint 75
        0x40, 0x78, // field 8 varint 120
        0x48, 0xE0, 0xDE, 0xCA, 0x0D, // field 9 varint 28487520
    ];

    #[test]
    fn decodes_a_hand_built_trendbar_with_known_numbers() {
        let tb = ProtoOaTrendbar::decode(&HAND_TRENDBAR[..]).unwrap();
        let bar = decode_trendbar(&tb).unwrap();
        assert_eq!(bar.low, d("1.08512"));
        assert_eq!(bar.open, d("1.08542")); // 108512 + 30
        assert_eq!(bar.close, d("1.08587")); // 108512 + 75
        assert_eq!(bar.high, d("1.08632")); // 108512 + 120
        assert_eq!(bar.tick_volume, 100);
        assert_eq!(
            bar.open_time,
            Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap()
        );
        assert_eq!(bar.date, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    }

    #[test]
    fn decodes_a_trendbar_inside_a_full_response_frame() {
        // GetTrendbarsRes{ctidTraderAccountId=49236051, period=D1, trendbar=[hand bar]}
        // wrapped in ProtoMessage{payloadType=2138} and framed.
        let tb = ProtoOaTrendbar::decode(&HAND_TRENDBAR[..]).unwrap();
        let res = ProtoOaGetTrendbarsRes {
            ctid_trader_account_id: 49236051,
            period: ProtoOaTrendbarPeriod::D1 as i32,
            trendbar: vec![tb],
            ..Default::default()
        };
        let frame = encode_frame(&ProtoMessage {
            payload_type: ProtoOaPayloadType::ProtoOaGetTrendbarsRes as u32,
            payload: Some(res.encode_to_vec()),
            client_msg_id: Some("7".into()),
        });
        let (msg, _) = decode_frame(&frame).unwrap().unwrap();
        assert_eq!(msg.payload_type, 2138);
        let back = ProtoOaGetTrendbarsRes::decode(msg.payload.unwrap().as_slice()).unwrap();
        let bar = decode_trendbar(&back.trendbar[0]).unwrap();
        assert_eq!(
            (bar.open, bar.high, bar.low, bar.close),
            (d("1.08542"), d("1.08632"), d("1.08512"), d("1.08587"))
        );
    }

    #[test]
    fn large_prices_and_two_digit_symbols_stay_exact() {
        // BTCUSD-like: low 61234.56 → relative 6_123_456_000.
        let tb = ProtoOaTrendbar {
            volume: 5,
            period: Some(ProtoOaTrendbarPeriod::D1 as i32),
            low: Some(6_123_456_000),
            delta_open: Some(12_345_000),
            delta_close: Some(0),
            delta_high: Some(250_000_000),
            utc_timestamp_in_minutes: Some(28487520),
        };
        let bar = decode_trendbar(&tb).unwrap();
        assert_eq!(bar.low, d("61234.56"));
        assert_eq!(bar.open, d("61358.01")); // 6_123_456_000 + 12_345_000
        assert_eq!(bar.close, d("61234.56"));
        assert_eq!(bar.high, d("63734.56"));
    }

    #[test]
    fn absent_delta_is_the_proto2_default_zero_but_absent_low_is_refused() {
        let mut tb = ProtoOaTrendbar {
            volume: 1,
            period: None,
            low: Some(100_000),
            delta_open: None,
            delta_close: None,
            delta_high: None,
            utc_timestamp_in_minutes: Some(28487520),
        };
        let bar = decode_trendbar(&tb).unwrap();
        assert_eq!((bar.open, bar.high, bar.close), (d("1"), d("1"), d("1")));
        tb.low = None;
        assert_eq!(decode_trendbar(&tb), Err(TrendbarError::MissingLow));
        tb.low = Some(1);
        tb.utc_timestamp_in_minutes = None;
        assert_eq!(decode_trendbar(&tb), Err(TrendbarError::MissingTimestamp));
    }

    #[test]
    fn non_daily_period_is_refused() {
        let tb = ProtoOaTrendbar {
            volume: 1,
            period: Some(ProtoOaTrendbarPeriod::H1 as i32),
            low: Some(1),
            utc_timestamp_in_minutes: Some(1),
            ..Default::default()
        };
        assert_eq!(decode_trendbar(&tb), Err(TrendbarError::WrongPeriod(9, 12)));
        // The same bar is an H1 bar to the period-aware decoder.
        assert!(decode_bar(&tb, ProtoOaTrendbarPeriod::H1).is_ok());
    }

    fn bar_at(y: i32, m: u32, day: u32, h: u32) -> DailyBar {
        let t = Utc.with_ymd_and_hms(y, m, day, h, 0, 0).unwrap();
        DailyBar {
            open_time: t,
            date: t.date_naive(),
            open: d("1"),
            high: d("1"),
            low: d("1"),
            close: d("1"),
            tick_volume: 1,
        }
    }

    #[test]
    fn forming_bar_is_dropped() {
        let now = Utc.with_ymd_and_hms(2024, 3, 5, 15, 30, 0).unwrap();
        let bars = vec![
            bar_at(2024, 3, 5, 0), // opened today 00:00, closes 03-06 00:00: forming
            bar_at(2024, 3, 3, 0),
            bar_at(2024, 3, 4, 0), // closed exactly at 03-05 00:00 ≤ now
        ];
        let (closed, forming) = closed_daily_bars(bars, now);
        assert_eq!(
            closed
                .iter()
                .map(|b| b.date.to_string())
                .collect::<Vec<_>>(),
            vec!["2024-03-03", "2024-03-04"]
        );
        assert_eq!(forming.len(), 1);
        assert_eq!(forming[0].date.to_string(), "2024-03-05");
    }

    #[test]
    fn bar_closing_exactly_now_is_closed_and_one_second_earlier_is_forming() {
        let bar = bar_at(2024, 3, 4, 22);
        let close = Utc.with_ymd_and_hms(2024, 3, 5, 22, 0, 0).unwrap();
        let (closed, _) = closed_daily_bars(vec![bar.clone()], close);
        assert_eq!(closed.len(), 1);
        let (closed, forming) = closed_daily_bars(vec![bar], close - Duration::seconds(1));
        assert!(closed.is_empty());
        assert_eq!(forming.len(), 1);
    }
}
