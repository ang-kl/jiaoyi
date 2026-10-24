//! Spread samples and the bar quote basis, from development-period broker
//! ticks streamed day by day and never stored.
//!
//! Per symbol and calendar year: the second full week of June (Monday 00:00
//! UTC to the next Monday), development years only. For that week:
//! - **spread:** at every quote update with both sides known, ask − bid; the
//!   median over the week (exact, from counts);
//! - **bar basis:** for each stored daily bar lying inside the week, its open
//!   is compared with the first bid and first ask at or after the bar's open
//!   time, and its close with the last bid and last ask before the bar's end:
//!   equal to the bid, equal to the ask, or within half a price unit of the
//!   mid. Comparisons where bid = ask cannot tell them apart and are counted
//!   apart.
//!
//! One row per symbol and year goes to `research_spreads` (`measured`,
//! `no_ticks`, or `failed` with the error, which may be taken again). The row
//! also records the ticks downloaded and the wall time, so the download rate
//! with nothing stored is measured.

use std::collections::BTreeMap;
use std::time::Instant;

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc, Weekday};
use jiaoyi_ctrader::tickdata::{HistTick, QuoteSide};
use jiaoyi_ctrader::AccountSession;
use jiaoyi_store::research::{self as rs, SpreadSample};
use jiaoyi_store::BarRow;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde_json::json;

use super::costs::BasisCounts;
use super::data::{Data, DataError};
use super::prereg;

/// The first June sampled. The broker's earliest tick history (the probe,
/// pre-registration inventory) starts 2013-07-22, so June 2013 already
/// shows none; earlier years would only ask for nothing.
pub const FIRST_SAMPLE_YEAR: i32 = 2013;
/// The last development year with a June wholly before the seal.
pub const LAST_SAMPLE_YEAR: i32 = 2025;

/// The second full Monday-to-Sunday week of June, `[from, to)`.
pub fn june_week(year: i32) -> (DateTime<Utc>, DateTime<Utc>) {
    let mut d = NaiveDate::from_ymd_opt(year, 6, 1).expect("June 1");
    while d.weekday() != Weekday::Mon {
        d += Duration::days(1);
    }
    let from = d + Duration::days(7);
    let at = |d: NaiveDate| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).expect("midnight"));
    (at(from), at(from + Duration::days(7)))
}

/// Quote updates of one day, both sides, with each side carried forward:
/// `(ts_ms, bid, ask)` after applying every tick of that millisecond.
pub fn merge(
    bids: &[HistTick],
    asks: &[HistTick],
    mut bid: Option<i64>,
    mut ask: Option<i64>,
) -> Vec<(i64, Option<i64>, Option<i64>)> {
    let mut out = Vec::with_capacity(bids.len() + asks.len());
    let (mut i, mut j) = (0, 0);
    while i < bids.len() || j < asks.len() {
        let ts = match (bids.get(i), asks.get(j)) {
            (Some(b), Some(a)) => b.ts_ms.min(a.ts_ms),
            (Some(b), None) => b.ts_ms,
            (None, Some(a)) => a.ts_ms,
            (None, None) => break,
        };
        while bids.get(i).is_some_and(|b| b.ts_ms == ts) {
            bid = Some(bids[i].price_e5);
            i += 1;
        }
        while asks.get(j).is_some_and(|a| a.ts_ms == ts) {
            ask = Some(asks[j].price_e5);
            j += 1;
        }
        out.push((ts, bid, ask));
    }
    out
}

fn e5(d: Decimal) -> Option<i64> {
    (d * Decimal::from(100_000)).to_i64()
}

/// One bar's comparison targets inside the sampled week.
struct BarCheck {
    open_ms: i64,
    end_ms: i64,
    open_e5: i64,
    close_e5: i64,
    first_bid: Option<i64>,
    first_ask: Option<i64>,
    last_bid: Option<i64>,
    last_ask: Option<i64>,
}

fn compare(bar: i64, bid: Option<i64>, ask: Option<i64>, c: &mut BasisCounts) {
    let (Some(b), Some(a)) = (bid, ask) else {
        return;
    };
    if a == b {
        c.uninformative += 1;
        return;
    }
    c.observations += 1;
    if bar == b {
        c.bid += 1;
    }
    if bar == a {
        c.ask += 1;
    }
    if (2 * bar - (a + b)).abs() <= 1 {
        c.mid += 1;
    }
}

/// What one sampled week found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Week {
    pub median_spread: Option<Decimal>,
    pub samples: i64,
    pub bid_ticks: i64,
    pub ask_ticks: i64,
    pub basis: BasisCounts,
    pub download_ms: i64,
}

/// Stream one symbol's week of ticks, day by day, through the stage's data
/// layer. `bars`: the symbol's stored daily bars (any; those inside the week
/// are compared).
pub async fn sample_week(
    data: &Data<'_>,
    session: &AccountSession<'_>,
    symbol_id: i64,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    bars: &[BarRow],
) -> Result<Week, DataError> {
    let started = Instant::now();
    let (from_ms, to_ms) = (from.timestamp_millis(), to.timestamp_millis());
    let mut checks: Vec<BarCheck> = bars
        .iter()
        .filter_map(|b| {
            let open_ms = b.bar_open_utc.timestamp_millis();
            let end_ms = (b.bar_open_utc + Duration::days(1)).timestamp_millis();
            (open_ms >= from_ms && end_ms <= to_ms).then(|| {
                Some(BarCheck {
                    open_ms,
                    end_ms,
                    open_e5: e5(b.open)?,
                    close_e5: e5(b.close)?,
                    first_bid: None,
                    first_ask: None,
                    last_bid: None,
                    last_ask: None,
                })
            })?
        })
        .collect();
    let mut counts: BTreeMap<i64, i64> = BTreeMap::new();
    let (mut bid, mut ask) = (None, None);
    let mut w = Week::default();
    let mut day = from_ms;
    while day < to_ms {
        let end = (day + 86_400_000).min(to_ms) - 1;
        let b = data
            .ticks(session, symbol_id, QuoteSide::Bid, day, end)
            .await?;
        let a = data
            .ticks(session, symbol_id, QuoteSide::Ask, day, end)
            .await?;
        let bids: Vec<HistTick> = b.ticks.iter().map(|(t, _)| *t).collect();
        let asks: Vec<HistTick> = a.ticks.iter().map(|(t, _)| *t).collect();
        w.bid_ticks += bids.len() as i64;
        w.ask_ticks += asks.len() as i64;
        for t in &bids {
            for c in checks.iter_mut() {
                if t.ts_ms >= c.open_ms && c.first_bid.is_none() {
                    c.first_bid = Some(t.price_e5);
                }
                if t.ts_ms >= c.open_ms && t.ts_ms < c.end_ms {
                    c.last_bid = Some(t.price_e5);
                }
            }
        }
        for t in &asks {
            for c in checks.iter_mut() {
                if t.ts_ms >= c.open_ms && c.first_ask.is_none() {
                    c.first_ask = Some(t.price_e5);
                }
                if t.ts_ms >= c.open_ms && t.ts_ms < c.end_ms {
                    c.last_ask = Some(t.price_e5);
                }
            }
        }
        let events = merge(&bids, &asks, bid, ask);
        if let Some(&(_, b2, a2)) = events.last() {
            bid = b2;
            ask = a2;
        }
        for (_, b, a) in events {
            if let (Some(b), Some(a)) = (b, a) {
                *counts.entry(a - b).or_default() += 1;
                w.samples += 1;
            }
        }
        day += 86_400_000;
    }
    for c in &checks {
        compare(c.open_e5, c.first_bid, c.first_ask, &mut w.basis);
        compare(c.close_e5, c.last_bid, c.last_ask, &mut w.basis);
    }
    if w.samples > 0 {
        // The exact median of the counted spreads (1/100000 units).
        let n = w.samples;
        let rank = |k: i64| {
            let mut seen = 0;
            for (v, c) in &counts {
                seen += c;
                if seen >= k {
                    return *v;
                }
            }
            0
        };
        let m2 = if n % 2 == 1 {
            2 * rank((n + 1) / 2)
        } else {
            rank(n / 2) + rank(n / 2 + 1)
        };
        w.median_spread = Some(Decimal::new(m2, 5) / Decimal::from(2));
    }
    w.download_ms = started.elapsed().as_millis() as i64;
    Ok(w)
}

/// Take every missing sample of one symbol (years without a measured or
/// no-ticks row). A failed week is recorded and the next year tried; a
/// session end stops and is returned, so the caller re-authorises.
#[allow(clippy::too_many_arguments)]
pub async fn sample_symbol(
    data: &Data<'_>,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    name: &str,
    have: &[i32],
    years: std::ops::RangeInclusive<i32>,
) -> Result<u32, DataError> {
    let bars = data
        .daily_bars(account_id, symbol_id, data.stage().end())
        .await?;
    let mut taken = 0;
    for year in years {
        if have.contains(&year) {
            continue;
        }
        let (from, to) = june_week(year);
        let mut row = SpreadSample {
            account_id,
            broker_symbol_id: symbol_id,
            symbol: name.to_string(),
            year,
            week_from: from,
            week_to: to,
            status: "measured".into(),
            median_spread: None,
            samples: 0,
            bid_ticks: 0,
            ask_ticks: 0,
            download_ms: 0,
            basis: None,
            error: None,
            prereg_hash: prereg::hash(),
        };
        match sample_week(data, session, symbol_id, from, to, &bars).await {
            Ok(w) => {
                row.status = if w.median_spread.is_some() {
                    "measured".into()
                } else {
                    "no_ticks".into()
                };
                row.median_spread = w.median_spread;
                row.samples = w.samples;
                row.bid_ticks = w.bid_ticks;
                row.ask_ticks = w.ask_ticks;
                row.download_ms = w.download_ms;
                row.basis = Some(json!(w.basis));
            }
            Err(DataError::Client(e)) if !session_over(session, &e) => {
                row.status = "failed".into();
                row.error = Some(e.to_string());
            }
            Err(e) => return Err(e),
        }
        rs::record_spread(data.pool(), &row).await?;
        taken += 1;
    }
    Ok(taken)
}

/// The session (or the connection under it) has ended: stop, re-authorise.
pub fn session_over(session: &AccountSession<'_>, e: &jiaoyi_ctrader::ClientError) -> bool {
    use jiaoyi_ctrader::ClientError;
    session.session_ended().is_some()
        || session.closed_reason().is_some()
        || matches!(
            e,
            ClientError::Disconnected(_) | ClientError::Frame(_) | ClientError::Timeout(_)
        )
        || e.is_token_rejected()
        || matches!(e, ClientError::Broker { code, .. }
            if code == "ACCOUNT_NOT_AUTHORIZED" || code == "CH_CLIENT_NOT_AUTHENTICATED")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_full_week_of_june() {
        // June 2024 starts on a Saturday: the first Monday is the 3rd, the
        // second full week runs Monday 10 to Sunday 16.
        let (f, t) = june_week(2024);
        assert_eq!(f.to_rfc3339(), "2024-06-10T00:00:00+00:00");
        assert_eq!(t.to_rfc3339(), "2024-06-17T00:00:00+00:00");
        // June 2025 starts on a Sunday: Mondays 2 and 9.
        assert_eq!(june_week(2025).0.date_naive().to_string(), "2025-06-09");
        // June 2014 starts on a Sunday too.
        assert_eq!(june_week(2014).0.date_naive().to_string(), "2014-06-09");
    }

    #[test]
    fn quotes_merge_with_each_side_carried() {
        let t = |ts, p| HistTick {
            ts_ms: ts,
            price_e5: p,
        };
        let m = merge(&[t(1, 100), t(3, 101)], &[t(2, 103), t(3, 104)], None, None);
        assert_eq!(
            m,
            vec![
                (1, Some(100), None),
                (2, Some(100), Some(103)),
                (3, Some(101), Some(104))
            ]
        );
        let mut c = BasisCounts::default();
        compare(100, Some(100), Some(103), &mut c);
        compare(102, Some(101), Some(103), &mut c);
        compare(101, Some(101), Some(101), &mut c);
        assert_eq!(
            (c.bid, c.ask, c.mid, c.observations, c.uninformative),
            (1, 0, 1, 2, 1)
        );
    }
}
