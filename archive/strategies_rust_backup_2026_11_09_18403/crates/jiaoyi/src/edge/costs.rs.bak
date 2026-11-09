//! Costs for the H1 simulation, each with its label (measured or estimated),
//! as the pre-registration states them:
//! - **Spread:** the median ask − bid of one June week a year (measured);
//!   years without a measurement take the nearest one (estimated: the
//!   earliest measured median before tick history).
//! - **Bar quote basis:** whether the stored daily bars are bid, ask or mid
//!   prices, measured against the same development-period ticks; a symbol
//!   whose basis is not clearly established is refused, never guessed.
//! - **Commission:** per unit of volume, by symbol class, from the account's
//!   own deals (commission ÷ filled units); a class without deals takes the
//!   overall median (estimated).
//! - **Swap:** the broker's current swap terms applied to every night held,
//!   the triple day included (estimated for past years).
//! - **Money → price units:** with the matching FX daily close on that date,
//!   in the quote currency the broker's asset list names (never inferred),
//!   cross-checked against the account's closing deals.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use jiaoyi_core::research::h1::Series;
use jiaoyi_core::universe::AssetClass;
use jiaoyi_store::research::{SpreadSample, SymbolTerms};
use jiaoyi_store::BarRow;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// The quote side the stored daily bars are priced on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Basis {
    Bid,
    Ask,
    Mid,
}

impl Basis {
    /// Bar price → ask.
    pub fn to_ask(self, spread: f64) -> f64 {
        match self {
            Basis::Bid => spread,
            Basis::Mid => spread / 2.0,
            Basis::Ask => 0.0,
        }
    }

    /// Bar price → bid (subtracted).
    pub fn to_bid(self, spread: f64) -> f64 {
        match self {
            Basis::Bid => 0.0,
            Basis::Mid => spread / 2.0,
            Basis::Ask => spread,
        }
    }
}

/// Bar opens and closes compared with the broker's ticks at those instants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasisCounts {
    /// Bar prices equal to the bid there.
    pub bid: u64,
    pub ask: u64,
    /// Within half a price unit (1e-5) of (bid + ask) / 2.
    pub mid: u64,
    /// Comparisons with bid ≠ ask (the informative ones).
    pub observations: u64,
    /// Comparisons with bid = ask, which cannot tell the bases apart.
    pub uninformative: u64,
}

impl BasisCounts {
    pub fn add(&mut self, o: &BasisCounts) {
        self.bid += o.bid;
        self.ask += o.ask;
        self.mid += o.mid;
        self.observations += o.observations;
        self.uninformative += o.uninformative;
    }
}

/// A basis is established when it explains at least 90% of the informative
/// observations and no other explains more than 50% (self-derived bounds,
/// recorded in the trial params).
pub const BASIS_MIN_SHARE: f64 = 0.9;
pub const BASIS_MAX_OTHER: f64 = 0.5;

pub fn decide_basis(c: &BasisCounts) -> Result<Basis, String> {
    let shown = format!(
        "bid {}/{n}, ask {}/{n}, mid {}/{n} ({} uninformative)",
        c.bid,
        c.ask,
        c.mid,
        c.uninformative,
        n = c.observations
    );
    if c.observations == 0 {
        return Err(format!(
            "bar quote basis not measured: no informative observation ({shown})"
        ));
    }
    let n = c.observations as f64;
    let cands = [
        (Basis::Bid, c.bid),
        (Basis::Ask, c.ask),
        (Basis::Mid, c.mid),
    ];
    let winners: Vec<Basis> = cands
        .iter()
        .filter(|(_, k)| *k as f64 / n >= BASIS_MIN_SHARE)
        .map(|(b, _)| *b)
        .collect();
    match winners.as_slice() {
        [b] if cands
            .iter()
            .filter(|(x, _)| x != b)
            .all(|(_, k)| *k as f64 / n <= BASIS_MAX_OTHER) =>
        {
            Ok(*b)
        }
        _ => Err(format!("bar quote basis ambiguous: {shown}")),
    }
}

/// The spread to use in a year, with its label.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SpreadBook {
    /// Measured median spread per year, price units.
    pub measured: BTreeMap<i32, f64>,
}

impl SpreadBook {
    pub fn from_samples(samples: &[&SpreadSample]) -> SpreadBook {
        SpreadBook {
            measured: samples
                .iter()
                .filter(|s| s.status == "measured")
                .filter_map(|s| Some((s.year, s.median_spread?.to_f64()?)))
                .collect(),
        }
    }

    /// `(spread, estimated, label)`. `None` with no measurement at all.
    pub fn for_year(&self, year: i32) -> Option<(f64, bool, String)> {
        if let Some(v) = self.measured.get(&year) {
            return Some((*v, false, format!("measured (June {year})")));
        }
        let (&first, &v_first) = self.measured.iter().next()?;
        if year < first {
            return Some((
                v_first,
                true,
                format!("estimated: the earliest measured median (June {first})"),
            ));
        }
        let (&y, &v) = self.measured.range(..year).next_back()?;
        Some((
            v,
            true,
            format!("estimated: the nearest earlier measured median (June {y})"),
        ))
    }
}

/// The FX pairs that convert USD to another quote currency, and whether USD
/// is their base.
pub fn fx_pair(ccy: &str) -> Option<(&'static str, bool)> {
    Some(match ccy {
        "EUR" => ("EURUSD", false),
        "GBP" => ("GBPUSD", false),
        "AUD" => ("AUDUSD", false),
        "NZD" => ("NZDUSD", false),
        "JPY" => ("USDJPY", true),
        "CAD" => ("USDCAD", true),
        _ => return None,
    })
}

/// FX daily closes by pair and date.
#[derive(Debug, Clone, Default)]
pub struct Fx {
    pub closes: HashMap<String, BTreeMap<NaiveDate, f64>>,
    /// The same closes keyed by their bar's open, for the deal cross-check.
    pub closes_by_open: HashMap<String, BTreeMap<DateTime<Utc>, f64>>,
}

impl Fx {
    /// Keep a pair's daily closes (bars the stage's data layer returned).
    pub fn add_bars(&mut self, pair: &str, bars: &[BarRow]) {
        for b in bars {
            if let Some(c) = b.close.to_f64() {
                self.closes
                    .entry(pair.to_string())
                    .or_default()
                    .insert(b.bar_date, c);
                self.closes_by_open
                    .entry(pair.to_string())
                    .or_default()
                    .insert(b.bar_open_utc, c);
            }
        }
    }

    /// USD per one unit of `ccy` at the close of the deal day's daily bar:
    /// the latest bar opened at or before `at`, when it opened within
    /// [`DEAL_DAY_WINDOW`] of it (a weekend or holiday deal takes the last
    /// trading day's close). `None` without such a bar or pair.
    pub fn usd_per_at(&self, ccy: &str, at: DateTime<Utc>) -> Option<f64> {
        if ccy == "USD" {
            return Some(1.0);
        }
        let (pair, usd_base) = fx_pair(ccy)?;
        let (open, c) = self.closes_by_open.get(pair)?.range(..=at).next_back()?;
        if at - *open >= DEAL_DAY_WINDOW {
            return None;
        }
        Some(if usd_base { 1.0 / c } else { *c })
    }

    /// USD per one unit of `ccy` on `date`: the latest close on or before
    /// the date; before the pair's first bar, its first close (estimated).
    pub fn usd_per(&self, ccy: &str, date: NaiveDate) -> Option<(f64, bool)> {
        if ccy == "USD" {
            return Some((1.0, false));
        }
        let (pair, usd_base) = fx_pair(ccy)?;
        let series = self.closes.get(pair)?;
        let (c, estimated) = match series.range(..=date).next_back() {
            Some((_, c)) => (*c, false),
            None => (*series.values().next()?, true),
        };
        Some((if usd_base { 1.0 / c } else { c }, estimated))
    }
}

/// How far a deal's quote → deposit rate may stand from the FX close of its
/// day (owner, 24-10-2026).
pub const RATE_TOLERANCE: f64 = 0.01;

/// A deal's day is the latest FX daily bar opened before it within this
/// window (a weekend and a holiday); a deal with none is not checked rather
/// than compared with an older close.
pub const DEAL_DAY_WINDOW: Duration = Duration::days(4);

/// The deal cross-check of a symbol's broker-stated quote currency (owner,
/// 24-10-2026): each closing deal's quote → deposit rate (deposit per one
/// unit of the quote currency; the deposit is USD) must agree within 1% with
/// the FX close of the deal's day, the close of the daily bar containing the
/// execution. `deals` are only those the stage may read (executed before its
/// end); a symbol without one is "not cross-checked". `Err` is a
/// disagreement: it fails the job.
pub fn cross_check_quote(
    quote: &str,
    fx: &Fx,
    deals: &[(DateTime<Utc>, f64)],
    before: DateTime<Utc>,
) -> Result<String, String> {
    if deals.is_empty() {
        return Ok(format!(
            "not cross-checked: no closing deal executed before {before}"
        ));
    }
    let mut checked = 0usize;
    let mut unpriced = 0usize;
    let mut worst = 0.0f64;
    for (at, rate) in deals {
        let Some(expected) = fx.usd_per_at(quote, *at) else {
            unpriced += 1;
            continue;
        };
        let diff = (rate / expected - 1.0).abs();
        if diff.is_nan() || diff > RATE_TOLERANCE {
            return Err(format!(
                "the broker-stated quote currency {quote} disagrees with a deal: the closing deal at {at} converted quote → deposit at {rate}, the {quote} FX close of that day gives {expected} ({:.2}% apart, more than {}%)",
                diff * 100.0,
                RATE_TOLERANCE * 100.0
            ));
        }
        checked += 1;
        worst = worst.max(diff);
    }
    if checked == 0 {
        return Ok(format!(
            "not cross-checked: {unpriced} closing deals executed before {before}, none with an FX close of its day"
        ));
    }
    Ok(format!(
        "{checked} closing deals executed before {before} agree within {}% (largest difference {:.3}%){}",
        RATE_TOLERANCE * 100.0,
        worst * 100.0,
        if unpriced > 0 {
            format!("; {unpriced} without an FX close of their day not checked")
        } else {
            String::new()
        }
    ))
}

/// Commission per unit of volume in the deposit currency (USD), by class.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommissionBook {
    pub per_class: BTreeMap<String, (f64, String)>,
    pub overall_median: f64,
    pub deals: usize,
}

fn median(mut v: Vec<f64>) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    })
}

pub fn commission_book(samples: &[(String, f64)]) -> Result<CommissionBook, String> {
    let overall = median(samples.iter().map(|(_, v)| *v).collect())
        .ok_or("no deal with a commission on the account: commission not measured")?;
    let mut per_class = BTreeMap::new();
    for class in ["equity_index", "fx", "metal", "energy", "crypto"] {
        let v: Vec<f64> = samples
            .iter()
            .filter(|(c, _)| c == class)
            .map(|(_, v)| *v)
            .collect();
        let n = v.len();
        per_class.insert(
            class.to_string(),
            match median(v) {
                Some(m) => (m, format!("measured: median of {n} deals")),
                None => (
                    overall,
                    "estimated: no deal in this class, the overall median".into(),
                ),
            },
        );
    }
    Ok(CommissionBook {
        per_class,
        overall_median: overall,
        deals: samples.len(),
    })
}

/// The broker's swap terms, as price units per unit of volume per night.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwapTerms {
    pub long: f64,
    pub short: f64,
    /// PIPS, POINTS or PERCENTAGE, as the broker names it.
    pub kind: String,
    pub triple_day: Option<String>,
    /// Charges per day (24 ÷ the swap period in hours; 1 when not given).
    pub charges_per_day: f64,
    pub pip_size: f64,
    pub point_size: f64,
    pub read_at: Option<DateTime<Utc>>,
}

/// Annual-percentage swaps are charged per night at rate ÷ 365 of the price
/// (self-derived day count, recorded).
pub const PERCENT_SWAP_DAYS: f64 = 365.0;

impl SwapTerms {
    pub fn from_terms(t: &SymbolTerms) -> Result<SwapTerms, String> {
        let (Some(long), Some(short)) = (t.swap_long, t.swap_short) else {
            return Err("swap terms not read from the broker".into());
        };
        let kind = t
            .swap_calculation_type
            .clone()
            .unwrap_or_else(|| "PIPS".into());
        if !matches!(kind.as_str(), "PIPS" | "POINTS" | "PERCENTAGE") {
            return Err(format!("swap calculation type {kind} is not known"));
        }
        let charges_per_day = match t.swap_period_hours {
            Some(h) if h > 0 => 24.0 / h as f64,
            _ => 1.0,
        };
        Ok(SwapTerms {
            long,
            short,
            kind,
            triple_day: t.swap_rollover_3days.clone(),
            charges_per_day,
            pip_size: 10f64.powi(-t.pip_position),
            point_size: 10f64.powi(-t.digits),
            read_at: t.swap_read_at,
        })
    }

    /// Price units per unit of volume for one night, at a price `close`.
    pub fn per_night(&self, long: bool, close: f64) -> f64 {
        let v = if long { self.long } else { self.short };
        let one = match self.kind.as_str() {
            "PIPS" => v * self.pip_size,
            "POINTS" => v * self.point_size,
            _ => close * v / 100.0 / PERCENT_SWAP_DAYS,
        };
        one * self.charges_per_day
    }

    fn triple(&self) -> Option<Weekday> {
        Some(match self.triple_day.as_deref()? {
            "MONDAY" => Weekday::Mon,
            "TUESDAY" => Weekday::Tue,
            "WEDNESDAY" => Weekday::Wed,
            "THURSDAY" => Weekday::Thu,
            "FRIDAY" => Weekday::Fri,
            "SATURDAY" => Weekday::Sat,
            "SUNDAY" => Weekday::Sun,
            _ => return None,
        })
    }

    /// Nights charged for holding over the end of a bar opening at `open`:
    /// 3 when that rollover falls on the triple day, else 1. The rollover is
    /// taken on the day of the bar's last second.
    pub fn nights(&self, open: DateTime<Utc>) -> f64 {
        let last_second = open + Duration::days(1) - Duration::seconds(1);
        if self.triple() == Some(last_second.weekday()) {
            3.0
        } else {
            1.0
        }
    }
}

/// Everything the series of one symbol needs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolCosts {
    pub name: String,
    pub symbol_id: i64,
    pub class: AssetClass,
    pub basis: Basis,
    pub basis_counts: BasisCounts,
    pub spreads: SpreadBook,
    pub commission_usd_per_unit: f64,
    pub commission_label: String,
    pub swap: SwapTerms,
    /// The quote currency: the name of the symbol's quote asset in the
    /// broker's asset list (ProtoOAAssetListRes), never inferred.
    pub quote: String,
    /// Where the quote currency was read (the load run of the asset list).
    pub quote_label: String,
    /// The deal cross-check of the quote → deposit rate over the deals the
    /// stage may read ("not cross-checked" without one).
    pub quote_cross_check: String,
    /// Broker volume limits in units (volume in cents ÷ 100).
    pub min_units: Option<f64>,
    pub step_units: Option<f64>,
}

/// Bars and costs → the simulation's series. Returns the series and the
/// per-day USD per price unit (for money). `None` when a cost is missing on
/// some day (the caller refuses the symbol with the reason).
pub fn series(
    bars: &[BarRow],
    c: &SymbolCosts,
    fx: &Fx,
    first_entry: usize,
) -> Result<(Series, Vec<f64>), String> {
    let n = bars.len();
    if n == 0 {
        return Err("no stored daily bars".into());
    }
    let mut s = Series {
        symbol: c.name.clone(),
        first_entry,
        last: n - 1,
        ..Default::default()
    };
    let mut usd = Vec::with_capacity(n);
    for b in bars {
        let f = |d: rust_decimal::Decimal| {
            d.to_f64()
                .ok_or_else(|| format!("bar {} not a number", b.bar_date))
        };
        let (o, h, l, cl) = (f(b.open)?, f(b.high)?, f(b.low)?, f(b.close)?);
        let (spread, est, _) = c
            .spreads
            .for_year(b.bar_date.year())
            .ok_or("no spread measure")?;
        let (usd_per_quote, _) = fx
            .usd_per(&c.quote, b.bar_date)
            .ok_or_else(|| format!("no FX close to convert {} on {}", c.quote, b.bar_date))?;
        s.dates.push(b.bar_date);
        s.open.push(o);
        s.high.push(h);
        s.low.push(l);
        s.close.push(cl);
        s.to_ask.push(c.basis.to_ask(spread));
        s.to_bid.push(c.basis.to_bid(spread));
        s.spread.push(spread);
        s.spread_estimated.push(est);
        // USD per unit → quote currency per unit = price units per unit.
        s.commission.push(c.commission_usd_per_unit / usd_per_quote);
        let nights = c.swap.nights(b.bar_open_utc);
        s.swap_long.push(c.swap.per_night(true, cl) * nights);
        s.swap_short.push(c.swap.per_night(false, cl) * nights);
        usd.push(usd_per_quote);
    }
    Ok((s, usd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_basis_is_decided_only_when_clear() {
        let c = |bid, ask, mid, n| BasisCounts {
            bid,
            ask,
            mid,
            observations: n,
            uninformative: 0,
        };
        assert_eq!(decide_basis(&c(95, 0, 3, 100)), Ok(Basis::Bid));
        assert_eq!(decide_basis(&c(0, 0, 100, 100)), Ok(Basis::Mid));
        assert!(decide_basis(&c(89, 0, 0, 100))
            .unwrap_err()
            .contains("ambiguous"));
        assert!(decide_basis(&c(95, 60, 0, 100)).is_err());
        assert!(decide_basis(&c(0, 0, 0, 0))
            .unwrap_err()
            .contains("not measured"));
        assert_eq!(Basis::Bid.to_ask(0.2), 0.2);
        assert_eq!(Basis::Mid.to_bid(0.2), 0.1);
        assert_eq!(Basis::Ask.to_bid(0.2), 0.2);
    }

    #[test]
    fn spreads_before_and_between_measurements_are_estimated() {
        let b = SpreadBook {
            measured: BTreeMap::from([(2015, 0.2), (2018, 0.3)]),
        };
        assert_eq!(b.for_year(2015).map(|x| (x.0, x.1)), Some((0.2, false)));
        let (v, est, why) = b.for_year(1990).unwrap();
        assert_eq!((v, est), (0.2, true));
        assert!(why.contains("earliest"));
        assert_eq!(b.for_year(2016).map(|x| (x.0, x.1)), Some((0.2, true)));
        assert_eq!(b.for_year(2026).map(|x| (x.0, x.1)), Some((0.3, true)));
        assert!(SpreadBook::default().for_year(2020).is_none());
    }

    #[test]
    fn commission_and_fx_and_swap_by_hand() {
        let book = commission_book(&[
            ("fx".into(), 0.00003),
            ("fx".into(), 0.00005),
            ("metal".into(), 0.02),
        ])
        .unwrap();
        assert_eq!(book.per_class["fx"].0, 0.00004);
        assert_eq!(book.per_class["metal"].0, 0.02);
        // No index deal: the overall median of 0.00003, 0.00005, 0.02.
        assert_eq!(book.per_class["equity_index"].0, 0.00005);
        assert!(book.per_class["equity_index"].1.starts_with("estimated"));
        assert!(commission_book(&[]).is_err());

        let d = |s: &str| s.parse::<NaiveDate>().unwrap();
        let fx = Fx {
            closes: HashMap::from([
                ("USDJPY".into(), BTreeMap::from([(d("2020-01-02"), 100.0)])),
                ("EURUSD".into(), BTreeMap::from([(d("2020-01-02"), 1.25)])),
            ]),
            ..Default::default()
        };
        // USD per yen = 1 / 100; per euro = 1.25.
        assert_eq!(fx.usd_per("JPY", d("2020-01-03")), Some((0.01, false)));
        assert_eq!(fx.usd_per("EUR", d("2019-12-01")), Some((1.25, true)));
        assert_eq!(fx.usd_per("USD", d("2019-12-01")), Some((1.0, false)));
        assert_eq!(fx.usd_per("CHF", d("2020-01-03")), None);

        let terms = SymbolTerms {
            digits: 5,
            pip_position: 4,
            min_volume: Some(100_000),
            volume_step: Some(100_000),
            swap_long: Some(-6.5),
            swap_short: Some(1.2),
            swap_calculation_type: Some("PIPS".into()),
            swap_rollover_3days: Some("WEDNESDAY".into()),
            swap_period_hours: Some(24),
            swap_read_at: None,
        };
        let sw = SwapTerms::from_terms(&terms).unwrap();
        // −6.5 pips × 0.0001 = −0.00065 per unit per night.
        assert!((sw.per_night(true, 1.1) - -0.00065).abs() < 1e-15);
        assert!((sw.per_night(false, 1.1) - 0.00012).abs() < 1e-15);
        // An FX bar opening Tuesday 21:00 UTC ends Wednesday: triple.
        let tue = "2020-01-07T21:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(sw.nights(tue), 3.0);
        assert_eq!(sw.nights(tue + Duration::days(1)), 1.0);
        let pct = SwapTerms::from_terms(&SymbolTerms {
            swap_calculation_type: Some("PERCENTAGE".into()),
            swap_long: Some(-3.65),
            ..terms.clone()
        })
        .unwrap();
        // −3.65% a year of a price of 100: 100 × −3.65 / 100 / 365 = −0.01.
        assert!((pct.per_night(true, 100.0) - -0.01).abs() < 1e-15);
        assert_eq!(fx_pair("JPY"), Some(("USDJPY", true)));
        assert_eq!(fx_pair("EUR"), Some(("EURUSD", false)));
        assert_eq!(fx_pair("CHF"), None);
    }

    /// Owner, 24-10-2026: a deal's quote → deposit rate must agree with the
    /// FX close of its day within 1%, or the job fails.
    #[test]
    fn the_quote_cross_check_takes_the_close_of_the_deals_day() {
        let at = |s: &str| s.parse::<DateTime<Utc>>().unwrap();
        let bar = |open: &str, close: &str| BarRow {
            bar_date: at(open).date_naive(),
            bar_open_utc: at(open),
            open: close.parse().unwrap(),
            high: close.parse().unwrap(),
            low: close.parse().unwrap(),
            close: close.parse().unwrap(),
            tick_volume: 1,
        };
        let mut fx = Fx::default();
        fx.add_bars(
            "USDJPY",
            &[
                bar("2025-03-03T22:00:00Z", "150"),
                bar("2025-03-04T22:00:00Z", "100"),
            ],
        );
        let end = at("2025-10-24T00:00:00Z");
        // A deal at 14:00 on the 5th is inside the bar opened on the 4th:
        // its close is 100, so USD per yen is 0.01.
        let ok = cross_check_quote("JPY", &fx, &[(at("2025-03-05T14:00:00Z"), 0.01009)], end);
        assert!(ok.unwrap().starts_with("1 closing deals"));
        // 1.5% apart: the job fails.
        let bad = cross_check_quote("JPY", &fx, &[(at("2025-03-05T14:00:00Z"), 0.01015)], end);
        assert!(bad.unwrap_err().contains("disagrees"));
        // A yen rate read as if the quote were USD: 0.01 against 1.
        let wrong_ccy = cross_check_quote("USD", &fx, &[(at("2025-03-05T14:00:00Z"), 0.01)], end);
        assert!(wrong_ccy.is_err());
        // The bar of the 3rd (close 150) is the deal day for a deal on the 4th
        // before 22:00: 1/150.
        assert!(cross_check_quote(
            "JPY",
            &fx,
            &[(at("2025-03-04T10:00:00Z"), 1.0 / 150.0)],
            end
        )
        .is_ok());
        assert!(cross_check_quote("JPY", &fx, &[(at("2025-03-04T10:00:00Z"), 0.01)], end).is_err());
        // No deal: not cross-checked.
        assert!(cross_check_quote("JPY", &fx, &[], end)
            .unwrap()
            .starts_with("not cross-checked"));
        // Before the first close, or with no close in the 4 days before the
        // deal: not checked (never compared with an older close), not failed.
        assert!(
            cross_check_quote("JPY", &fx, &[(at("2020-01-01T00:00:00Z"), 5.0)], end)
                .unwrap()
                .starts_with("not cross-checked")
        );
        assert!(
            cross_check_quote("JPY", &fx, &[(at("2025-03-08T22:00:00Z"), 5.0)], end)
                .unwrap()
                .starts_with("not cross-checked")
        );
        // Within the window the last bar's close is the day's close (here the
        // bar opened on the 4th, 3.6 days before); 4.6 days is beyond it.
        let both = cross_check_quote(
            "JPY",
            &fx,
            &[
                (at("2025-03-08T12:00:00Z"), 0.01),
                (at("2025-03-09T12:00:00Z"), 5.0),
            ],
            end,
        )
        .unwrap();
        assert!(both.starts_with("1 closing deals"), "{both}");
        assert!(
            both.ends_with("; 1 without an FX close of their day not checked"),
            "{both}"
        );
    }
}
