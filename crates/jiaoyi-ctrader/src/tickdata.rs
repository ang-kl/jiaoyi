//! Historical ticks (`ProtoOAGetTickDataRes`).
//!
//! The encoding, as far as Spotware documents it:
//! - the .proto: `tickData` is "in chronological order (newest first). The
//!   first tick contains Unix time in milliseconds while all subsequent ticks
//!   have the time difference in milliseconds between the previous and the
//!   current one";
//! - help.ctrader.com/open-api/symbol-data/: prices are relative, divide by
//!   100000; one request may span at most one week (604800000 ms); a response
//!   holds at most a backend-configured number of ticks, with `hasMore`;
//! - cTrader community forum (Spotware replies,
//!   community.ctrader.com/forum/connect-api-support/22598 and /39604): the
//!   first element's timestamp and price are absolute and every later element
//!   is the difference from the previous one, for BOTH timestamp and price.
//!
//! The sign of the time difference is not documented. The decoder applies the
//! forum's rule (add the difference) and checks the result: newest first, every
//! tick inside the requested window, every price positive. If that fails it
//! tries subtracting the difference, with the same checks. Whichever passes is
//! recorded with the data; if neither passes the batch is refused. Nothing is
//! kept on a decoding that did not pass its checks.

use serde::Serialize;

use crate::pb::ProtoOaTickData;

/// Spotware's limit on one tick request's span.
pub const MAX_WINDOW_MS: i64 = 604_800_000;

/// Which side of the quote a tick series is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteSide {
    Bid,
    Ask,
}

impl QuoteSide {
    pub fn as_str(self) -> &'static str {
        match self {
            QuoteSide::Bid => "bid",
            QuoteSide::Ask => "ask",
        }
    }
    /// `ProtoOAQuoteType`: BID = 1, ASK = 2.
    pub fn proto(self) -> i32 {
        match self {
            QuoteSide::Bid => crate::pb::ProtoOaQuoteType::Bid as i32,
            QuoteSide::Ask => crate::pb::ProtoOaQuoteType::Ask as i32,
        }
    }
}

/// All ticks of one window, oldest first, with their duplicate ordinals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickFetch {
    pub ticks: Vec<(HistTick, i32)>,
    pub pages: u32,
    pub signs: Vec<DeltaSign>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct HistTick {
    pub ts_ms: i64,
    /// Price in 1/100000.
    pub price_e5: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaSign {
    /// value_i = value_{i-1} + d_i (the forum's rule).
    Add,
    /// value_i = value_{i-1} − d_i.
    Subtract,
}

impl DeltaSign {
    pub fn as_str(self) -> &'static str {
        match self {
            DeltaSign::Add => "add",
            DeltaSign::Subtract => "subtract",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TickDataError {
    #[error("tick data does not decode under either difference sign: add → {add}; subtract → {subtract}")]
    Undecodable { add: String, subtract: String },
    #[error("a page made no progress: more ticks at {ts_ms} ms than one response holds")]
    NoProgress { ts_ms: i64 },
    #[error("window {from_ms}..{to_ms} spans more than one week")]
    WindowTooLong { from_ms: i64, to_ms: i64 },
}

fn decode_with(
    data: &[ProtoOaTickData],
    sign: DeltaSign,
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<HistTick>, String> {
    let mut out = Vec::with_capacity(data.len());
    let Some(first) = data.first() else {
        return Ok(out);
    };
    let (mut ts, mut price) = (first.timestamp, first.tick);
    for (i, d) in data.iter().enumerate() {
        if i > 0 {
            let (dt, dp) = match sign {
                DeltaSign::Add => (d.timestamp, d.tick),
                DeltaSign::Subtract => (-d.timestamp, -d.tick),
            };
            ts = ts.checked_add(dt).ok_or("timestamp overflow")?;
            price = price.checked_add(dp).ok_or("price overflow")?;
            if ts > out.last().map(|t: &HistTick| t.ts_ms).unwrap_or(i64::MAX) {
                return Err(format!("tick {i} at {ts} is newer than the one before it"));
            }
        }
        if ts < from_ms || ts > to_ms {
            return Err(format!("tick {i} at {ts} ms is outside {from_ms}..{to_ms}"));
        }
        if price <= 0 {
            return Err(format!("tick {i} has price {price}"));
        }
        out.push(HistTick {
            ts_ms: ts,
            price_e5: price,
        });
    }
    Ok(out)
}

/// Decode one response, newest first, as described above.
pub fn decode_ticks(
    data: &[ProtoOaTickData],
    from_ms: i64,
    to_ms: i64,
) -> Result<(Vec<HistTick>, DeltaSign), TickDataError> {
    match decode_with(data, DeltaSign::Add, from_ms, to_ms) {
        Ok(t) => Ok((t, DeltaSign::Add)),
        Err(add) => match decode_with(data, DeltaSign::Subtract, from_ms, to_ms) {
            Ok(t) => Ok((t, DeltaSign::Subtract)),
            Err(subtract) => Err(TickDataError::Undecodable { add, subtract }),
        },
    }
}

/// Accumulates the pages of one window. A page continues backwards from the
/// oldest tick of the previous page: its `toTimestamp` is that tick's time, so
/// ticks sharing that millisecond are not lost, and the ones already held are
/// skipped (they come first in the next, newest-first page).
#[derive(Debug, Default)]
pub struct Pages {
    /// Newest first.
    ticks: Vec<HistTick>,
    pub pages: u32,
    pub signs: Vec<DeltaSign>,
}

impl Pages {
    /// The `toTimestamp` for the next page, or `None` before the first.
    pub fn next_to(&self) -> Option<i64> {
        self.ticks.last().map(|t| t.ts_ms)
    }

    /// Add one decoded page (newest first).
    pub fn push(&mut self, page: Vec<HistTick>, sign: DeltaSign) -> Result<(), TickDataError> {
        self.pages += 1;
        self.signs.push(sign);
        let (skip_ts, skip) = match self.ticks.last() {
            None => (i64::MIN, 0),
            Some(last) => (
                last.ts_ms,
                self.ticks
                    .iter()
                    .rev()
                    .take_while(|t| t.ts_ms == last.ts_ms)
                    .count(),
            ),
        };
        let mut skipped = 0;
        let before = self.ticks.len();
        for t in page {
            if t.ts_ms == skip_ts && skipped < skip {
                skipped += 1;
                continue;
            }
            self.ticks.push(t);
        }
        if self.ticks.len() == before && self.pages > 1 {
            return Err(TickDataError::NoProgress { ts_ms: skip_ts });
        }
        Ok(())
    }

    pub fn into_fetch(self) -> TickFetch {
        let (pages, signs) = (self.pages, self.signs.clone());
        TickFetch {
            ticks: self.finish(),
            pages,
            signs,
        }
    }

    /// The ticks, oldest first, each with its ordinal among identical
    /// (timestamp, price) ticks, so that repeats are kept and a re-download is
    /// recognised as the same rows.
    pub fn finish(mut self) -> Vec<(HistTick, i32)> {
        self.ticks.reverse();
        // Count every (timestamp, price) across the whole window, not only the
        // tick just before: (t, 100), (t, 101), (t, 100) numbers the two
        // (t, 100) ticks 0 and 1, so the stored key keeps all three.
        let mut seen: std::collections::HashMap<HistTick, i32> = std::collections::HashMap::new();
        self.ticks
            .into_iter()
            .map(|t| {
                let n = seen.entry(t).or_insert(0);
                let dup = *n;
                *n += 1;
                (t, dup)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn td(ts: i64, tick: i64) -> ProtoOaTickData {
        ProtoOaTickData {
            timestamp: ts,
            tick,
        }
    }

    #[test]
    fn decodes_absolute_first_then_differences_newest_first() {
        // Ticks at 10_000 (1.08510), 9_500 (1.08505), 9_500 (1.08507), 8_000 (1.08500),
        // sent newest first with the forum's rule: later elements are differences.
        let data = [td(10_000, 108_510), td(-500, -5), td(0, 2), td(-1_500, -7)];
        let (t, sign) = decode_ticks(&data, 0, 20_000).unwrap();
        assert_eq!(sign, DeltaSign::Add);
        assert_eq!(
            t,
            vec![
                HistTick {
                    ts_ms: 10_000,
                    price_e5: 108_510
                },
                HistTick {
                    ts_ms: 9_500,
                    price_e5: 108_505
                },
                HistTick {
                    ts_ms: 9_500,
                    price_e5: 108_507
                },
                HistTick {
                    ts_ms: 8_000,
                    price_e5: 108_500
                },
            ]
        );
    }

    #[test]
    fn the_other_sign_is_used_only_when_it_alone_passes_the_checks() {
        // Positive time differences: adding them runs past the window's end
        // and makes ticks newer; subtracting them passes.
        let data = [td(10_000, 108_510), td(500, 5), td(1_500, 3)];
        let (t, sign) = decode_ticks(&data, 0, 10_000).unwrap();
        assert_eq!(sign, DeltaSign::Subtract);
        assert_eq!(
            t[2],
            HistTick {
                ts_ms: 8_000,
                price_e5: 108_502
            }
        );
    }

    #[test]
    fn data_that_passes_neither_reading_is_refused() {
        // The first tick is outside the window.
        let e = decode_ticks(&[td(50_000, 1)], 0, 10_000).unwrap_err();
        assert!(matches!(e, TickDataError::Undecodable { .. }), "{e}");
        // A price that becomes non-positive either way: add gives 3, 0; subtract 3, 6, 0.
        let e = decode_ticks(&[td(10_000, 3), td(0, -3), td(0, 6)], 0, 10_000).unwrap_err();
        assert!(matches!(e, TickDataError::Undecodable { .. }));
        assert_eq!(decode_ticks(&[], 0, 1).unwrap().0, vec![]);
    }

    #[test]
    fn pages_continue_from_the_oldest_millisecond_without_loss_or_repeat() {
        let t = |ts, p| HistTick {
            ts_ms: ts,
            price_e5: p,
        };
        let mut pages = Pages::default();
        // Page 1 ends with two ticks at 7_000; the third tick at 7_000 is on page 2.
        pages
            .push(
                vec![t(9_000, 5), t(8_000, 4), t(7_000, 3), t(7_000, 3)],
                DeltaSign::Add,
            )
            .unwrap();
        assert_eq!(pages.next_to(), Some(7_000));
        // Page 2 asked up to 7_000 inclusive: it repeats the two held, then the new one.
        pages
            .push(
                vec![t(7_000, 3), t(7_000, 3), t(7_000, 3), t(6_000, 2)],
                DeltaSign::Add,
            )
            .unwrap();
        let out = pages.finish();
        assert_eq!(
            out,
            vec![
                (t(6_000, 2), 0),
                (t(7_000, 3), 0),
                (t(7_000, 3), 1),
                (t(7_000, 3), 2),
                (t(8_000, 4), 0),
                (t(9_000, 5), 0),
            ]
        );
    }

    #[test]
    fn repeats_are_numbered_across_the_window_not_just_neighbours() {
        let t = |ts, p| HistTick {
            ts_ms: ts,
            price_e5: p,
        };
        let mut pages = Pages::default();
        // Newest first: (t,100), (t,101), (t,100) all at the same millisecond.
        pages
            .push(
                vec![t(7_000, 100), t(7_000, 101), t(7_000, 100)],
                DeltaSign::Add,
            )
            .unwrap();
        let out = pages.finish();
        let keys: std::collections::HashSet<(i64, i64, i32)> =
            out.iter().map(|(t, d)| (t.ts_ms, t.price_e5, *d)).collect();
        assert_eq!(keys.len(), 3, "three distinct stored keys: {out:?}");
        let mut dups: Vec<i32> = out
            .iter()
            .filter(|(t, _)| t.price_e5 == 100)
            .map(|(_, d)| *d)
            .collect();
        dups.sort();
        assert_eq!(dups, vec![0, 1]);
    }

    #[test]
    fn a_page_that_only_repeats_is_an_error_not_a_loop() {
        let t = |ts, p| HistTick {
            ts_ms: ts,
            price_e5: p,
        };
        let mut pages = Pages::default();
        pages
            .push(vec![t(7_000, 3), t(7_000, 3)], DeltaSign::Add)
            .unwrap();
        let e = pages
            .push(vec![t(7_000, 3), t(7_000, 3)], DeltaSign::Add)
            .unwrap_err();
        assert_eq!(e, TickDataError::NoProgress { ts_ms: 7_000 });
    }
}
