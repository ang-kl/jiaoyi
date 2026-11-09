//! H1: the tick-triggered trend, on daily bars (pre-registration 24-10-2026).
//!
//! Per symbol, one position at a time. On each day `t` (indicators read only
//! the completed days before it):
//! - **Filter:** EMA(50) of the completed daily closes above EMA(100) → longs
//!   only; below → shorts only; equal → no entry. Each EMA is seeded with the
//!   simple mean of its first `len` closes.
//! - **Entry:** long when the day's range reaches the highest high of the
//!   previous `breakout` completed days (short: the lowest low). The fill is
//!   the level, or the day's open if the day opened beyond it, moved to the
//!   side the order fills on (a buy to the ask, a sell to the bid) and then
//!   made worse by the slippage.
//! - **Stop:** starts `stop_atr` × ATR(20) (Wilder, completed days) from the
//!   fill; at the end of each day it trails `stop_atr` × ATR behind the best
//!   price since entry (the highest high, lowest low for shorts, entry day
//!   included) and only ever moves in the trade's favour. A day reaching the
//!   stop exits at it; a day opening beyond it exits at the open. The stop is
//!   checked before the day's new best price is counted, so a day that reaches
//!   both a new peak and the stop is stopped (worst case). On the entry day a
//!   range reaching the initial stop is taken as hit after the fill (worst
//!   case). Exits fill like entries: a sell at the bid, a buy at the ask, minus
//!   or plus the slippage. No take-profit.
//! - **Period end:** a trade still open on the last day of the period closes
//!   at that day's close, filled the same way.
//! - **One position per symbol:** no new entry on the day a trade exits.
//!
//! R = the price move from fill to exit ÷ the initial stop distance, less
//! the commission of both fills and plus the swap of the nights held (both
//! converted to price units per unit of volume by the caller).

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

pub const EMA_FAST: usize = 50;
pub const EMA_SLOW: usize = 100;
pub const ATR_LEN: usize = 20;
/// A symbol's test starts once this many daily bars exist.
pub const MIN_BARS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    /// The day's range reached the stop.
    Stop,
    /// The day opened beyond the stop: filled at the open.
    GapThroughStop,
    /// Still open on the period's last day: closed at its close.
    PeriodEnd,
}

/// One symbol's daily bars with the per-day costs, as the simulation reads
/// them. Every vector has one entry per bar.
#[derive(Debug, Clone, Default)]
pub struct Series {
    pub symbol: String,
    pub dates: Vec<NaiveDate>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    /// Added to a bar price to reach the ask that day (from the measured bar
    /// basis and the day's spread: bid bars + spread, mid bars + spread/2,
    /// ask bars + 0).
    pub to_ask: Vec<f64>,
    /// Subtracted from a bar price to reach the bid that day.
    pub to_bid: Vec<f64>,
    /// The day's median spread in price units (the slippage unit).
    pub spread: Vec<f64>,
    /// The day's spread is an estimate, not a measurement of that year.
    pub spread_estimated: Vec<bool>,
    /// Commission per fill per unit of volume, in price units.
    pub commission: Vec<f64>,
    /// Swap per unit of volume for holding over the end of day t, in price
    /// units, as the broker signs it (negative is a charge), the triple day
    /// already applied.
    pub swap_long: Vec<f64>,
    pub swap_short: Vec<f64>,
    /// The first day index an entry may be made on (the bars before are
    /// warm-up only).
    pub first_entry: usize,
    /// The last day index of the period.
    pub last: usize,
}

/// Indicators per day, each over the completed days up to and including
/// that index (a decision on day t reads index t − 1).
#[derive(Debug, Clone, Default)]
pub struct Indicators {
    pub ema_fast: Vec<Option<f64>>,
    pub ema_slow: Vec<Option<f64>>,
    pub atr: Vec<Option<f64>>,
}

/// EMA with smoothing 2 / (len + 1), seeded with the simple mean of the
/// first `len` values (defined from index len − 1).
pub fn ema(values: &[f64], len: usize) -> Vec<Option<f64>> {
    let mut out = vec![None; values.len()];
    if len == 0 || values.len() < len {
        return out;
    }
    let a = 2.0 / (len as f64 + 1.0);
    let mut e = values[..len].iter().sum::<f64>() / len as f64;
    out[len - 1] = Some(e);
    for i in len..values.len() {
        e += a * (values[i] - e);
        out[i] = Some(e);
    }
    out
}

/// Wilder ATR (as `indicators::atr_wilder`) at every index from `len` on.
pub fn atr(high: &[f64], low: &[f64], close: &[f64], len: usize) -> Vec<Option<f64>> {
    let n = close.len();
    let mut out = vec![None; n];
    if len == 0 || n < len + 1 {
        return out;
    }
    let tr = |i: usize| {
        (high[i] - low[i])
            .max((high[i] - close[i - 1]).abs())
            .max((low[i] - close[i - 1]).abs())
    };
    let l = len as f64;
    let mut a = (1..=len).map(tr).sum::<f64>() / l;
    out[len] = Some(a);
    for (i, slot) in out.iter_mut().enumerate().skip(len + 1) {
        a = (a * (l - 1.0) + tr(i)) / l;
        *slot = Some(a);
    }
    out
}

impl Indicators {
    pub fn of(s: &Series) -> Indicators {
        Indicators {
            ema_fast: ema(&s.close, EMA_FAST),
            ema_slow: ema(&s.close, EMA_SLOW),
            atr: atr(&s.high, &s.low, &s.close, ATR_LEN),
        }
    }
}

/// One of the 9 grid settings, at one slippage multiple.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct H1Params {
    /// Breakout length in completed days: 20, 50 or 100.
    pub breakout: usize,
    /// Stop distance in ATR(20): 2, 3 or 4.
    pub stop_atr: f64,
    /// Slippage per fill, in multiples of the day's median spread: 0, 1, 2.
    pub slip_mult: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Trade {
    pub symbol: String,
    pub side: Side,
    pub entry_index: usize,
    pub entry_date: NaiveDate,
    /// The breakout level (`None` for a control trade, which enters at the open).
    pub level: Option<f64>,
    /// The bar price the entry filled from (the level, or the open).
    pub entry_ref: f64,
    pub entry_fill: f64,
    pub atr: f64,
    pub stop_distance: f64,
    pub initial_stop: f64,
    pub exit_index: usize,
    pub exit_date: NaiveDate,
    pub exit_ref: f64,
    pub exit_fill: f64,
    pub exit_reason: ExitReason,
    pub nights: usize,
    pub gross_r: f64,
    pub commission_r: f64,
    pub swap_r: f64,
    pub net_r: f64,
    pub spread_estimated: bool,
}

impl Trade {
    pub fn entry_year(&self) -> i32 {
        self.entry_date.year()
    }
}

/// The first day index a decision may be taken on: 100 completed bars, the
/// breakout window, the EMAs and the ATR all exist.
pub fn start_index(s: &Series, p: &H1Params) -> usize {
    s.first_entry.max(MIN_BARS).max(p.breakout).max(1)
}

/// The day's entry, if any: side and breakout level.
pub fn entry_signal(s: &Series, ind: &Indicators, p: &H1Params, t: usize) -> Option<(Side, f64)> {
    if t < p.breakout || t == 0 {
        return None;
    }
    let (f, w, _) = (ind.ema_fast[t - 1]?, ind.ema_slow[t - 1]?, ind.atr[t - 1]?);
    let window = (t - p.breakout)..t;
    if f > w {
        let level = s.high[window].iter().copied().fold(f64::MIN, f64::max);
        (s.high[t] >= level).then_some((Side::Long, level))
    } else if f < w {
        let level = s.low[window].iter().copied().fold(f64::MAX, f64::min);
        (s.low[t] <= level).then_some((Side::Short, level))
    } else {
        None
    }
}

/// Fill a buy (`Side::Long` entry, short exit) or a sell from a bar price on day t.
fn fill(s: &Series, p: &H1Params, t: usize, buy: bool, bar_price: f64) -> f64 {
    let slip = p.slip_mult * s.spread[t];
    if buy {
        bar_price + s.to_ask[t] + slip
    } else {
        bar_price - s.to_bid[t] - slip
    }
}

/// Run one trade from its entry on day `t_e` at bar price `entry_ref` to its
/// exit. `level` is recorded only. Needs ATR at `t_e − 1`.
pub fn run_trade(
    s: &Series,
    ind: &Indicators,
    p: &H1Params,
    side: Side,
    t_e: usize,
    entry_ref: f64,
    level: Option<f64>,
) -> Option<Trade> {
    let atr_e = ind.atr[t_e.checked_sub(1)?]?;
    let d = p.stop_atr * atr_e;
    if d.is_nan() || d <= 0.0 {
        return None;
    }
    let long = side == Side::Long;
    let entry_fill = fill(s, p, t_e, long, entry_ref);
    let initial_stop = if long { entry_fill - d } else { entry_fill + d };
    let mut stop = initial_stop;
    let mut exit: Option<(usize, f64, ExitReason)> = None;
    // The entry day: the stop is live from the fill (worst case).
    let mut best;
    if long {
        if s.low[t_e] <= stop {
            exit = Some((t_e, stop, ExitReason::Stop));
        }
        best = s.high[t_e];
    } else {
        if s.high[t_e] >= stop {
            exit = Some((t_e, stop, ExitReason::Stop));
        }
        best = s.low[t_e];
    }
    let trail = |stop: f64, best: f64, t: usize| -> f64 {
        match ind.atr[t] {
            Some(a) if long => stop.max(best - p.stop_atr * a),
            Some(a) => stop.min(best + p.stop_atr * a),
            None => stop,
        }
    };
    if exit.is_none() {
        stop = trail(stop, best, t_e);
        for t in (t_e + 1)..=s.last {
            let (gap, hit) = if long {
                (s.open[t] <= stop, s.low[t] <= stop)
            } else {
                (s.open[t] >= stop, s.high[t] >= stop)
            };
            if gap {
                exit = Some((t, s.open[t], ExitReason::GapThroughStop));
                break;
            }
            if hit {
                exit = Some((t, stop, ExitReason::Stop));
                break;
            }
            best = if long {
                best.max(s.high[t])
            } else {
                best.min(s.low[t])
            };
            stop = trail(stop, best, t);
        }
    }
    let (t_x, exit_ref, exit_reason) =
        exit.unwrap_or((s.last, s.close[s.last], ExitReason::PeriodEnd));
    let exit_fill = fill(s, p, t_x, !long, exit_ref);
    let gross_r = if long {
        (exit_fill - entry_fill) / d
    } else {
        (entry_fill - exit_fill) / d
    };
    let commission_r = (s.commission[t_e] + s.commission[t_x]) / d;
    let swaps = if long { &s.swap_long } else { &s.swap_short };
    let swap_r = swaps[t_e..t_x].iter().sum::<f64>() / d;
    Some(Trade {
        symbol: s.symbol.clone(),
        side,
        entry_index: t_e,
        entry_date: s.dates[t_e],
        level,
        entry_ref,
        entry_fill,
        atr: atr_e,
        stop_distance: d,
        initial_stop,
        exit_index: t_x,
        exit_date: s.dates[t_x],
        exit_ref,
        exit_fill,
        exit_reason,
        nights: t_x - t_e,
        gross_r,
        commission_r,
        swap_r,
        net_r: gross_r - commission_r + swap_r,
        spread_estimated: s.spread_estimated[t_e] || s.spread_estimated[t_x],
    })
}

/// Every H1 trade of one symbol over its period.
pub fn simulate(s: &Series, ind: &Indicators, p: &H1Params) -> Vec<Trade> {
    let mut out = Vec::new();
    if s.dates.is_empty() {
        return out;
    }
    let mut t = start_index(s, p);
    while t <= s.last {
        if let Some((side, level)) = entry_signal(s, ind, p, t) {
            let entry_ref = match side {
                Side::Long => level.max(s.open[t]),
                Side::Short => level.min(s.open[t]),
            };
            if let Some(tr) = run_trade(s, ind, p, side, t, entry_ref, Some(level)) {
                t = tr.exit_index + 1;
                out.push(tr);
                continue;
            }
        }
        t += 1;
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A flat series of `n` bars at 100 (range 99..101), bid basis, spread 0.2.
    pub fn flat(n: usize) -> Series {
        let d0 = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
        Series {
            symbol: "TEST".into(),
            dates: (0..n)
                .map(|i| d0 + chrono::Duration::days(i as i64))
                .collect(),
            open: vec![100.0; n],
            high: vec![101.0; n],
            low: vec![99.0; n],
            close: vec![100.0; n],
            to_ask: vec![0.2; n],
            to_bid: vec![0.0; n],
            spread: vec![0.2; n],
            spread_estimated: vec![false; n],
            commission: vec![0.01; n],
            swap_long: vec![-0.005; n],
            swap_short: vec![0.001; n],
            first_entry: 0,
            last: n - 1,
        }
    }

    #[test]
    fn ema_and_atr_match_hand_values() {
        // EMA(3) of 1, 2, 3, 4: seed (1+2+3)/3 = 2 at index 2; α = 0.5;
        // 2 + 0.5 × (4 − 2) = 3 at index 3.
        assert_eq!(
            ema(&[1.0, 2.0, 3.0, 4.0], 3),
            vec![None, None, Some(2.0), Some(3.0)]
        );
        // The same bars as indicators::atr_wilder's hand test: ATR(3) is 2,
        // 8/3, 19/9 at indices 3, 4, 5.
        let h = [10.0, 11.0, 12.0, 11.0, 14.0, 13.0];
        let l = [8.0, 9.0, 9.0, 10.0, 10.0, 12.0];
        let c = [9.0, 10.0, 11.0, 10.0, 13.0, 12.0];
        let a = atr(&h, &l, &c, 3);
        assert_eq!(a[2], None);
        assert_eq!(a[3], Some(2.0));
        assert!((a[4].unwrap() - 8.0 / 3.0).abs() < 1e-15);
        assert!((a[5].unwrap() - 19.0 / 9.0).abs() < 1e-15);
    }

    #[test]
    fn a_flat_market_makes_no_trade() {
        // EMA(50) == EMA(100) on a flat series: the filter allows nothing.
        let s = flat(300);
        let ind = Indicators::of(&s);
        let p = H1Params {
            breakout: 50,
            stop_atr: 3.0,
            slip_mult: 1.0,
        };
        assert!(simulate(&s, &ind, &p).is_empty());
    }
}
