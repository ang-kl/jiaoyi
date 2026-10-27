//! Strategy 3's noise band (owner-approved rules, 25-10-2026; after
//! Zarattini, Aziz & Barbon 2024, SFI RP 24-97).
//!
//! - The session's **open** is its first M30 trendbar's open, the bar that
//!   starts at the session open.
//! - At each 30-minute mark τ after the open, **σ(τ)** is the mean, over the
//!   last [`LOOKBACK`] sessions, of |price at τ ÷ that session's open − 1|.
//!   The price at τ is the close of the M30 bar that ends at τ.
//! - The band is open × (1 ± σ).
//!
//! **Refinement, 27-10-2026 (v3).** The 25 Oct registration entered on the
//! first tick beyond the band. The statements of that book printed the
//! barrier rate of a far target, not an edge. v3 decides only at a
//! 30-minute mark, on the mid, which is the paper's check. [`triggered`] is
//! kept for the recorded tick rule and its tests. Live entry uses
//! [`at_decision_mark`] and [`mark_decision`]. See
//! `docs/preregistration-2026-10-27-strategy3-v3.md`.
//!
//! Readings where the text leaves a choice (self-derived, recorded with
//! every decision):
//! - **Missing bars.** A past session counts only if it has the bar that
//!   starts at its open. At a mark it counts only if it has the bar that
//!   ends at that mark. σ(τ) takes the last 14 sessions that do, looking back
//!   over at most [`MAX_SESSIONS_BACK`] sessions. Fewer than 14 means no band
//!   at τ, with the reason.
//! - **Between marks.** At an instant between two marks, σ is interpolated
//!   linearly between them. v3 does not decide between marks.
//! - **Before the first mark.** There is no band before open + 30 minutes.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::sessions::{sessions_before, Session, SessionRule};

/// Sessions σ is averaged over (rules: "the last 14 sessions").
pub const LOOKBACK: usize = 14;
/// How far back sessions are looked for, to find 14 with a given bar.
pub const MAX_SESSIONS_BACK: usize = 30;
/// The band is checked from this many minutes after the open, every as many.
pub const MARK_MINUTES: i64 = 30;

/// An M30 trendbar: its open time and its open and close prices.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct M30 {
    pub open_time: DateTime<Utc>,
    pub open: f64,
    pub close: f64,
}

/// σ at each 30-minute mark of one session, from the sessions before it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SigmaTable {
    /// Minutes after the open → σ, or why there is none.
    pub marks: BTreeMap<i64, Result<f64, String>>,
    /// The past sessions that had a first bar (newest first): their dates.
    pub sessions_used: Vec<chrono::NaiveDate>,
}

/// The bar starting at `at`, if any.
fn bar_at(bars: &BTreeMap<DateTime<Utc>, M30>, at: DateTime<Utc>) -> Option<&M30> {
    bars.get(&at)
}

/// σ at every 30-minute mark of `today`, from M30 bars of the sessions
/// before it (`bars` may hold any span; only the bars that sit on the
/// sessions' marks are read).
pub fn sigma_table(rule: &SessionRule, today: &Session, bars: &[M30]) -> SigmaTable {
    let by_time: BTreeMap<DateTime<Utc>, M30> = bars.iter().map(|b| (b.open_time, *b)).collect();
    // Past sessions with a first bar, newest first.
    let past: Vec<(Session, f64)> = sessions_before(rule, today.open, MAX_SESSIONS_BACK)
        .into_iter()
        .filter_map(|s| {
            let o = bar_at(&by_time, s.open)?.open;
            (o.is_finite() && o > 0.0).then_some((s, o))
        })
        .collect();
    let length = (today.close - today.open).num_minutes();
    let mut marks = BTreeMap::new();
    let mut tau = MARK_MINUTES;
    while tau <= length {
        let mut moves = Vec::new();
        for (s, o) in &past {
            if moves.len() == LOOKBACK {
                break;
            }
            // The bar that ends at τ starts 30 minutes earlier.
            let start = s.open + Duration::minutes(tau - MARK_MINUTES);
            if start + Duration::minutes(MARK_MINUTES) > s.close {
                continue;
            }
            if let Some(b) = bar_at(&by_time, start) {
                if b.close.is_finite() && b.close > 0.0 {
                    moves.push((b.close / o - 1.0).abs());
                }
            }
        }
        let v = if moves.len() == LOOKBACK {
            Ok(moves.iter().sum::<f64>() / LOOKBACK as f64)
        } else {
            Err(format!(
                "{} of the last {} sessions have a price at {} minutes after the open; {LOOKBACK} needed",
                moves.len(),
                past.len(),
                tau
            ))
        };
        marks.insert(tau, v);
        tau += MARK_MINUTES;
    }
    SigmaTable {
        marks,
        sessions_used: past.iter().map(|(s, _)| s.date).collect(),
    }
}

/// The band at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Band {
    /// The session's open (its first M30 bar's open).
    pub open: f64,
    pub sigma: f64,
    pub lower: f64,
    pub upper: f64,
    /// The half-width, open × σ: the trade's R.
    pub half_width: f64,
    /// Minutes after the session open.
    pub minutes: f64,
}

/// σ at `minutes` after the open: a mark's value, or the linear
/// interpolation between the two marks around it.
pub fn sigma_at(table: &SigmaTable, minutes: f64) -> Result<f64, String> {
    if minutes.is_nan() || minutes < MARK_MINUTES as f64 {
        return Err(format!(
            "no band before {MARK_MINUTES} minutes after the open ({minutes:.1})"
        ));
    }
    let lo = (minutes / MARK_MINUTES as f64).floor() as i64 * MARK_MINUTES;
    let get = |m: i64| match table.marks.get(&m) {
        Some(Ok(v)) => Ok(*v),
        Some(Err(why)) => Err(why.clone()),
        None => Err(format!("no mark at {m} minutes in this session")),
    };
    let a = get(lo)?;
    let frac = (minutes - lo as f64) / MARK_MINUTES as f64;
    if frac <= 0.0 {
        return Ok(a);
    }
    let b = get(lo + MARK_MINUTES)?;
    Ok(a + (b - a) * frac)
}

/// The band at `at` in `today`, around `open`.
pub fn band_at(
    table: &SigmaTable,
    today: &Session,
    open: f64,
    at: DateTime<Utc>,
) -> Result<Band, String> {
    if !(open.is_finite() && open > 0.0) {
        return Err("no session open price".into());
    }
    let minutes = (at - today.open).num_milliseconds() as f64 / 60_000.0;
    let sigma = sigma_at(table, minutes)?;
    let half_width = open * sigma;
    Ok(Band {
        open,
        sigma,
        lower: open - half_width,
        upper: open + half_width,
        half_width,
        minutes,
    })
}

/// The side a quote triggers against the band at its instant: an ask above
/// the upper band buys, a bid below the lower band sells. A quote doing both
/// (a spread wider than the band) triggers nothing; a quote on the band is
/// inside it.
///
/// **Not the live entry.** This is the 25 Oct tick rule, kept so its tests
/// and any replay of that hash stay readable. v3 entry uses
/// [`mark_decision`].
pub fn triggered(band: &Band, bid: Option<f64>, ask: Option<f64>) -> Option<super::targets::Dir> {
    use super::targets::Dir;
    let buy = ask.is_some_and(|a| a > band.upper);
    let sell = bid.is_some_and(|b| b < band.lower);
    match (buy, sell) {
        (true, false) => Some(Dir::Buy),
        (false, true) => Some(Dir::Sell),
        _ => None,
    }
}

/// How long after a 30-minute mark a quote still counts as that mark's
/// decision. The paper checks at the mark. Quotes arrive continuously; this
/// window takes the quote that arrives with the mark and refuses the rest.
pub const MARK_WINDOW_SECS: i64 = 5;

/// Whether `now` is a v3 decision mark: a multiple of 30 minutes after the
/// session open, within [`MARK_WINDOW_SECS`] of that instant, and not before
/// the first mark.
pub fn at_decision_mark(session: &Session, now: DateTime<Utc>) -> bool {
    let elapsed = now - session.open;
    let mins = elapsed.num_minutes();
    if mins < MARK_MINUTES || mins % MARK_MINUTES != 0 {
        return false;
    }
    let into_mark = elapsed.num_seconds() - mins * 60;
    (0..MARK_WINDOW_SECS).contains(&into_mark)
}

/// The exact mark belonging to an accepted quote; keep the quote's real
/// timestamp separately when recording the decision.
pub fn decision_mark(session: &Session, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    at_decision_mark(session, now).then(|| {
        session.open
            + Duration::minutes((now - session.open).num_minutes() / MARK_MINUTES * MARK_MINUTES)
    })
}

/// The v3 side at a mark: the mid buys above the upper band and sells below
/// the lower. Both sides are required. A mid inside the band, a one-sided
/// quote, or a quote wider than the band (mid can sit inside while the
/// sides straddle) enters nothing.
pub fn mark_decision(
    band: &Band,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Option<super::targets::Dir> {
    use super::targets::Dir;
    let (bid, ask) = (bid?, ask?);
    if !(bid.is_finite() && ask.is_finite() && ask >= bid) || ask - bid > band.upper - band.lower {
        return None;
    }
    let mid = (bid + ask) / 2.0;
    if mid > band.upper {
        Some(Dir::Buy)
    } else if mid < band.lower {
        Some(Dir::Sell)
    } else {
        None
    }
}

/// Whether a new entry may be taken at `at`: after the first mark, before the
/// session's last 30 minutes (the session-end exit would close it at once),
/// and outside lunch.
pub fn entry_window(s: &Session, at: DateTime<Utc>) -> Result<(), String> {
    if at < s.open + Duration::minutes(MARK_MINUTES) {
        return Err("before the first 30 minutes of the session have passed".into());
    }
    if at >= s.close - Duration::minutes(MARK_MINUTES) {
        return Err("the session's last 30 minutes: no new entry (the session-end exit would close it at once)".into());
    }
    if s.lunch.is_some_and(|(a, b)| at >= a && at < b) {
        return Err("the lunch break".into());
    }
    if at >= s.close {
        return Err("the session has closed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::sessions::{rule_for, session_on};
    use super::*;
    use crate::universe::AssetClass;
    use chrono::NaiveDate;

    #[test]
    fn review_wide_quotes_do_not_create_a_mid_signal() {
        let b = Band {
            open: 100.0,
            sigma: 0.02,
            lower: 98.0,
            upper: 102.0,
            half_width: 2.0,
            minutes: 30.0,
        };
        assert_eq!(mark_decision(&b, Some(97.0), Some(108.0)), None);
        assert_eq!(mark_decision(&b, Some(92.0), Some(103.0)), None);
    }

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn btc() -> SessionRule {
        rule_for("BTCUSD", AssetClass::Crypto).unwrap()
    }

    /// 20 past UTC days of M30 bars: open 100 each day; the bar ending at
    /// +30 closes at 100 × (1 + d) with d alternating ±k/1000 by day
    /// (|move| = k/1000, k = day index % 4 + 1); the bar ending at +60
    /// closes at 100 × 1.01 every day.
    fn bars(days: i64, today: NaiveDate) -> Vec<M30> {
        let mut v = Vec::new();
        for i in 1..=days {
            let d = today - Duration::days(i);
            let open = at(&format!("{d}T00:00:00Z"));
            let k = (i % 4 + 1) as f64 / 1000.0;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            v.push(M30 {
                open_time: open,
                open: 100.0,
                close: 100.0 * (1.0 + sign * k),
            });
            v.push(M30 {
                open_time: open + Duration::minutes(30),
                open: 100.0,
                close: 101.0,
            });
        }
        v
    }

    #[test]
    fn sigma_is_the_mean_absolute_move_of_the_last_14_sessions() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let s = session_on(&btc(), today).unwrap();
        let t = sigma_table(&btc(), &s, &bars(20, today));
        // Days 1..14: k = i % 4 + 1 → 2,3,4,1, 2,3,4,1, 2,3,4,1, 2,3 (÷1000).
        let want = (2 + 3 + 4 + 1 + 2 + 3 + 4 + 1 + 2 + 3 + 4 + 1 + 2 + 3) as f64 / 14.0 / 1000.0;
        assert!(
            (t.marks[&30].clone().unwrap() - want).abs() < 1e-12,
            "{:?}",
            t.marks[&30]
        );
        assert!((t.marks[&60].clone().unwrap() - 0.01).abs() < 1e-12);
        // A mark with no bars: no σ, with the reason.
        assert!(t.marks[&90]
            .as_ref()
            .unwrap_err()
            .contains("0 of the last 20"));
        // The marks run to the session close (24 h for crypto).
        assert_eq!(*t.marks.keys().last().unwrap(), 1440);
    }

    #[test]
    fn fewer_than_14_sessions_give_no_band() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let s = session_on(&btc(), today).unwrap();
        let t = sigma_table(&btc(), &s, &bars(13, today));
        let e = t.marks[&30].clone().unwrap_err();
        assert!(
            e.contains("13 of the last 13") && e.contains("14 needed"),
            "{e}"
        );
        assert!(band_at(&t, &s, 100.0, s.open + Duration::minutes(30)).is_err());
    }

    #[test]
    fn a_session_without_its_first_bar_does_not_count() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let s = session_on(&btc(), today).unwrap();
        let mut b = bars(15, today);
        // Day 1 loses its first bar: days 2..15 are the 14 used.
        b.retain(|x| x.open_time != at("2026-10-24T00:00:00Z"));
        let t = sigma_table(&btc(), &s, &b);
        assert_eq!(t.sessions_used.len(), 14);
        assert_eq!(
            t.sessions_used[0],
            NaiveDate::from_ymd_opt(2026, 10, 23).unwrap()
        );
        assert!(t.marks[&30].is_ok());
    }

    #[test]
    fn the_band_interpolates_between_marks_and_breaks_out_on_either_side() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let s = session_on(&btc(), today).unwrap();
        let mut t = sigma_table(&btc(), &s, &bars(20, today));
        t.marks.insert(30, Ok(0.002));
        t.marks.insert(60, Ok(0.004));
        let b = band_at(&t, &s, 100.0, s.open + Duration::minutes(30)).unwrap();
        assert!((b.upper - 100.2).abs() < 1e-9 && (b.lower - 99.8).abs() < 1e-9);
        assert!((b.half_width - 0.2).abs() < 1e-12);
        // Halfway between 30 and 60: σ = 0.003.
        let b = band_at(&t, &s, 100.0, s.open + Duration::minutes(45)).unwrap();
        assert!((b.sigma - 0.003).abs() < 1e-12);
        use super::super::targets::Dir;
        // The ask triggers a buy, the bid a sell.
        assert_eq!(triggered(&b, Some(100.2), Some(100.31)), Some(Dir::Buy));
        assert_eq!(triggered(&b, Some(99.69), Some(99.8)), Some(Dir::Sell));
        assert_eq!(
            triggered(&b, Some(100.29), Some(100.3)),
            None,
            "on the band is inside"
        );
        assert_eq!(triggered(&b, Some(100.0), Some(100.01)), None);
        // A bid above the band with the ask inside it is no buy.
        assert_eq!(triggered(&b, Some(100.31), None), None);
        assert_eq!(
            triggered(&b, Some(99.6), Some(100.4)),
            None,
            "wider than the band"
        );
        // Before the first mark: no band.
        assert!(band_at(&t, &s, 100.0, s.open + Duration::minutes(29)).is_err());
    }

    #[test]
    fn v3_decides_only_at_the_mark_on_the_mid() {
        let us = rule_for("US500", AssetClass::EquityIndex).unwrap();
        let s = session_on(&us, NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()).unwrap();
        // 15:00 UTC is open+30. Within 5s is a mark; 6s and mid-interval are not.
        assert!(at_decision_mark(&s, at("2026-11-02T15:00:00Z")));
        assert!(at_decision_mark(&s, at("2026-11-02T15:00:04Z")));
        assert!(!at_decision_mark(&s, at("2026-11-02T15:00:05Z")));
        assert!(!at_decision_mark(&s, at("2026-11-02T15:15:00Z")));
        assert!(at_decision_mark(&s, at("2026-11-02T15:30:02Z")));
        let b = Band {
            open: 100.0,
            sigma: 0.002,
            lower: 99.8,
            upper: 100.2,
            half_width: 0.2,
            minutes: 30.0,
        };
        use super::super::targets::Dir;
        // Mid above the upper buys; mid below the lower sells.
        assert_eq!(
            mark_decision(&b, Some(100.15), Some(100.27)),
            Some(Dir::Buy)
        );
        assert_eq!(mark_decision(&b, Some(99.71), Some(99.85)), Some(Dir::Sell));
        // Mid inside, even if a side has crossed: no entry.
        assert_eq!(mark_decision(&b, Some(100.10), Some(100.25)), None);
        assert_eq!(mark_decision(&b, Some(100.0), Some(100.01)), None);
        // One side missing, or a crossed quote: no decision.
        assert_eq!(mark_decision(&b, None, Some(100.3)), None);
        assert_eq!(mark_decision(&b, Some(100.3), Some(100.1)), None);
    }

    #[test]
    fn entries_wait_30_minutes_stop_30_before_the_close_and_skip_the_lunch() {
        let us = rule_for("US500", AssetClass::EquityIndex).unwrap();
        let s = session_on(&us, NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()).unwrap();
        // 09:30–16:00 EST: 15:00 to 20:30 UTC.
        assert!(entry_window(&s, at("2026-11-02T14:59:59Z")).is_err());
        assert!(entry_window(&s, at("2026-11-02T15:00:00Z")).is_ok());
        assert!(entry_window(&s, at("2026-11-02T20:29:59Z")).is_ok());
        assert!(entry_window(&s, at("2026-11-02T20:30:00Z"))
            .unwrap_err()
            .contains("last 30 minutes"));
        let jp = rule_for("JPN225", AssetClass::EquityIndex).unwrap();
        let s = session_on(&jp, NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()).unwrap();
        // 11:30–12:30 JST is 02:30–03:30 UTC.
        assert!(entry_window(&s, at("2026-10-26T02:29:59Z")).is_ok());
        assert!(entry_window(&s, at("2026-10-26T02:30:00Z"))
            .unwrap_err()
            .contains("lunch"));
        assert!(entry_window(&s, at("2026-10-26T03:30:00Z")).is_ok());
    }
}
