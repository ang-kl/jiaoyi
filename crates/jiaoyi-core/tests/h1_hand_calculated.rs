//! Rulebook §9, phase-2 gate: the H1 engine reproduces three hand-calculated
//! trades on REAL broker bars (live 43002148), exactly.
//!
//! The bars are the fixtures `tests/fixtures/h1_xauusd.json` and
//! `h1_us500.json`: daily bars fetched from the deployed read-only route
//! `GET /research/bars` (main af30b591) at 2026-10-24T11:27:47Z, unchanged.
//! Each fixture is a window of stored bars; the indicators are computed over
//! the window as given (EMAs seeded with the simple mean of the window's
//! first closes).
//!
//! The expected numbers were worked by hand from those bars with an
//! independent implementation of the pre-registration's H1 text in exact
//! rational arithmetic (`tests/fixtures/h1_reference.py`, written from
//! docs/preregistration-2026-10-24.md, not from this crate). EMA and ATR
//! values over the 100-plus bars before each entry are that script's; every
//! step after them is written out below.
//!
//! Test costs (fixed, round, so the arithmetic is legible): bars taken as
//! bid prices; spread 0.30 (XAUUSD) and 0.5 (US500); slippage 1 × spread;
//! commission 0.05 / 0.10 per fill; swap per night long −0.20 / −0.30,
//! short +0.05 / +0.10, in price units per unit of volume.

use chrono::NaiveDate;
use jiaoyi_core::research::h1::{simulate, ExitReason, H1Params, Indicators, Series, Side, Trade};
use serde_json::Value;

struct Costs {
    spread: f64,
    commission: f64,
    swap_long: f64,
    swap_short: f64,
}

fn series(fixture: &str, symbol: &str, c: &Costs) -> Series {
    let v: Value = serde_json::from_str(fixture).unwrap();
    let bars = v["bars"].as_array().unwrap();
    let n = bars.len();
    let num = |b: &Value, k: &str| b[k].as_str().unwrap().parse::<f64>().unwrap();
    Series {
        symbol: symbol.into(),
        dates: bars
            .iter()
            .map(|b| {
                b["bar_date"]
                    .as_str()
                    .unwrap()
                    .parse::<NaiveDate>()
                    .unwrap()
            })
            .collect(),
        open: bars.iter().map(|b| num(b, "open")).collect(),
        high: bars.iter().map(|b| num(b, "high")).collect(),
        low: bars.iter().map(|b| num(b, "low")).collect(),
        close: bars.iter().map(|b| num(b, "close")).collect(),
        // Bid bars: the ask is a spread above, the bid is the bar price.
        to_ask: vec![c.spread; n],
        to_bid: vec![0.0; n],
        spread: vec![c.spread; n],
        spread_estimated: vec![false; n],
        commission: vec![c.commission; n],
        swap_long: vec![c.swap_long; n],
        swap_short: vec![c.swap_short; n],
        first_entry: 0,
        last: n - 1,
    }
}

const DEFAULT: H1Params = H1Params {
    breakout: 50,
    stop_atr: 3.0,
    slip_mult: 1.0,
};

fn trades(fixture: &str, symbol: &str, c: &Costs) -> Vec<Trade> {
    let s = series(fixture, symbol, c);
    simulate(&s, &Indicators::of(&s), &DEFAULT)
}

fn day(s: &str) -> NaiveDate {
    s.parse().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

macro_rules! assert_close {
    ($a:expr, $b:expr) => {
        assert!(
            close($a, $b),
            "{} = {} but the hand value is {}",
            stringify!($a),
            $a,
            $b
        )
    };
}

const XAU: Costs = Costs {
    spread: 0.30,
    commission: 0.05,
    swap_long: -0.20,
    swap_short: 0.05,
};
const SPX: Costs = Costs {
    spread: 0.5,
    commission: 0.10,
    swap_long: -0.30,
    swap_short: 0.10,
};

/// The long: XAUUSD, entered on the bar dated 2016-02-10, stopped on the
/// bar dated 2016-02-14 above its entry by the trailing stop.
///
/// - Filter (bar 2016-02-09): EMA(50) 1112.91168 > EMA(100) 1110.23989 → longs.
/// - Level: highest high of the 50 bars before = 1200.89 (bar 2016-02-07).
/// - 2016-02-10: O 1196.62, H 1263.25 ≥ 1200.89 → entry. It opened below the
///   level, so the fill is from the level: 1200.89 + ask 0.30 + slippage 0.30
///   = 1201.49.
/// - ATR(20) at 2016-02-09 = 16.30073159890; D = 3 × that = 48.90219479671;
///   initial stop 1201.49 − 48.90219479671 = 1152.58780520329.
/// - Entry day: L 1196.62 > 1152.5878, not stopped. Best = H 1263.25;
///   ATR(2016-02-10) = 18.81719501900; 1263.25 − 3 × 18.817195019 =
///   1206.79841494 → stop 1206.79841494.
/// - 2016-02-11: O 1247.84, L 1232.51 > stop; H 1248.09 < best. ATR =
///   18.65533526800; 1263.25 − 55.96600580 = 1207.28399420 → stop.
/// - 2016-02-14: O 1231.95 > stop; L 1202.14 ≤ 1207.28399420 → stopped at
///   1207.28399420; a sell fills at the bid (the bar price) less slippage
///   0.30 = 1206.98399420.
/// - Gross R = (1206.98399420 − 1201.49) ÷ 48.90219479671 = 0.11234657706.
///   Commission 2 × 0.05 ÷ D = 0.00204489799; swap 2 nights × −0.20 ÷ D =
///   −0.00817959197. Net R = 0.11234657706 − 0.00204489799 − 0.00817959197
///   = 0.10212208709.
#[test]
fn a_long_xauusd_2016_02_10() {
    let all = trades(include_str!("fixtures/h1_xauusd.json"), "XAUUSD", &XAU);
    // The window's two trades, as the hand run found them.
    let seen: Vec<(Side, NaiveDate, NaiveDate)> = all
        .iter()
        .map(|t| (t.side, t.entry_date, t.exit_date))
        .collect();
    assert_eq!(
        seen,
        vec![
            (Side::Short, day("2015-12-01"), day("2016-01-05")),
            (Side::Long, day("2016-02-10"), day("2016-02-14")),
        ]
    );
    let t = &all[1];
    assert_eq!(t.level, Some(1200.89));
    assert_eq!(t.entry_ref, 1200.89);
    assert_close!(t.entry_fill, 1201.49);
    assert_close!(t.atr, 16.300731598904537);
    assert_close!(t.stop_distance, 48.902194796713616);
    assert_close!(t.initial_stop, 1152.5878052032863);
    assert_eq!(t.exit_reason, ExitReason::Stop);
    assert_close!(t.exit_ref, 1207.283994195966);
    assert_close!(t.exit_fill, 1206.983994195966);
    assert_eq!(t.nights, 2);
    assert_close!(t.gross_r, 0.11234657705660235);
    assert_close!(t.commission_r, 0.002044897993141206);
    assert_close!(t.swap_r, -0.008179591972564824);
    assert_close!(t.net_r, 0.10212208709089632);
}

/// The short: US500, entered on the bar dated 2016-02-10, stopped on the
/// bar dated 2016-02-16.
///
/// - Filter (bar 2016-02-09): EMA(50) 1949.90618 < EMA(100) 1981.88356 → shorts.
/// - Level: lowest low of the 50 bars before = 1811.6 (bar 2016-01-19).
/// - 2016-02-10: O 1847.8, L 1807.7 ≤ 1811.6 → entry. It opened above the
///   level, so the fill is from the level: a sell at the bid (the bar price)
///   1811.6 less slippage 0.5 = 1811.1.
/// - ATR(20) at 2016-02-09 = 41.70934143724; D = 125.12802431171; initial
///   stop 1811.1 + D = 1936.22802431171.
/// - Entry day: H 1851.2 < stop. Best = L 1807.7; ATR(2016-02-10) =
///   41.79887436540; 1807.7 + 3 × that = 1933.09662310 → stop.
/// - 2016-02-11: H 1864.6 < stop; L 1826.9 > best. ATR 41.59393064710 →
///   1932.48179194.
/// - 2016-02-14: H 1893.9 < stop. ATR 41.07423411480 → 1930.92270234.
/// - 2016-02-15: H 1896.2 < stop. ATR 40.13052240900 → 1928.09156723.
/// - 2016-02-16: O 1892.7 < stop; H 1931.2 ≥ 1928.09156723 → stopped there;
///   a buy fills at the ask, + 0.5 spread + 0.5 slippage = 1929.09156723.
/// - Gross R = (1811.1 − 1929.09156723) ÷ 125.12802431171 = −0.94296675646.
///   Commission 2 × 0.10 ÷ D = 0.00159836297; swap 4 nights × +0.10 ÷ D =
///   +0.00319672593. Net R = −0.94136839349.
#[test]
fn a_short_us500_2016_02_10() {
    let all = trades(include_str!("fixtures/h1_us500.json"), "US500", &SPX);
    let t = all
        .iter()
        .find(|t| t.entry_date == day("2016-02-10"))
        .expect("the hand-calculated short");
    assert_eq!(t.side, Side::Short);
    assert_eq!(t.level, Some(1811.6));
    assert_close!(t.entry_fill, 1811.1);
    assert_close!(t.atr, 41.70934143723706);
    assert_close!(t.stop_distance, 125.12802431171117);
    assert_close!(t.initial_stop, 1936.2280243117111);
    assert_eq!(t.exit_date, day("2016-02-16"));
    assert_eq!(t.exit_reason, ExitReason::Stop);
    assert_close!(t.exit_fill, 1929.0915672270407);
    assert_eq!(t.nights, 4);
    assert_close!(t.gross_r, -0.9429667564566305);
    assert_close!(t.commission_r, 0.0015983629654518672);
    assert_close!(t.swap_r, 0.0031967259309037344);
    assert_close!(t.net_r, -0.9413683934911787);
}

/// The worst-case ambiguous day: US500 long entered on the bar dated
/// 2016-06-22; the next bar (dated 2016-06-23, the session of the UK
/// referendum result) reaches BOTH a new peak and the stop.
///
/// - Filter (bar 2016-06-21): EMA(50) 2073.56128 > EMA(100) 2051.53913 → longs.
/// - Level: highest high of the 50 bars before = 2120.9 (bar 2016-06-07).
/// - 2016-06-22: O 2085.2, H 2121.3 ≥ 2120.9 → entry from the level:
///   2120.9 + 0.5 + 0.5 = 2121.9. ATR(20) at 2016-06-21 = 20.20609965458;
///   D = 60.61829896373; initial stop 2061.28170103627.
/// - Entry day: L 2085.2 > stop. Best = H 2121.3; ATR(2016-06-22) =
///   21.00579467180; 2121.3 − 63.0173840 = 2058.28 is BELOW the stop, so the
///   stop stays 2061.28170103627 (it never moves against the trade).
/// - 2016-06-23: O 2121.0 > stop; H 2127.6 is a new peak (> 2121.3) AND L
///   2007.1 ≤ 2061.28170103627. Worst case: the stop is hit first, at
///   2061.28170103627. (Counting the new peak first, trailing 3 × the last
///   completed ATR 21.0057946718 behind 2127.6, would have raised the stop to
///   2064.58261598 and exited there.) A sell at the bid less 0.5 slippage =
///   2060.78170103627.
/// - Gross R = (2060.78170103627 − 2121.9) ÷ 60.61829896373 = −1.00824833439.
///   Commission 0.2 ÷ D = 0.00329933376; swap 1 night × −0.30 ÷ D =
///   −0.00494900063. Net R = −1.01649666878.
#[test]
fn a_worst_case_ambiguous_day_us500_2016_06_23() {
    let all = trades(include_str!("fixtures/h1_us500.json"), "US500", &SPX);
    // All five trades of the window, as the hand run found them.
    let seen: Vec<(Side, NaiveDate, NaiveDate)> = all
        .iter()
        .map(|t| (t.side, t.entry_date, t.exit_date))
        .collect();
    assert_eq!(
        seen,
        vec![
            (Side::Short, day("2016-01-13"), day("2016-01-28")),
            (Side::Short, day("2016-02-10"), day("2016-02-16")),
            (Side::Long, day("2016-03-29"), day("2016-05-05")),
            (Side::Long, day("2016-06-05"), day("2016-06-15")),
            (Side::Long, day("2016-06-22"), day("2016-06-23")),
        ]
    );
    let t = all.last().unwrap();
    assert_close!(t.entry_fill, 2121.9);
    assert_close!(t.atr, 20.206099654576246);
    assert_close!(t.initial_stop, 2061.2817010362714);
    assert_eq!(t.exit_reason, ExitReason::Stop);
    // The stop before the day, not the peak-first 2064.58261598.
    assert_close!(t.exit_ref, 2061.2817010362714);
    assert_close!(t.exit_fill, 2060.7817010362714);
    assert_eq!(t.nights, 1);
    assert_close!(t.gross_r, -1.0082483343899038);
    assert_close!(t.net_r, -1.0164966687798078);
}
