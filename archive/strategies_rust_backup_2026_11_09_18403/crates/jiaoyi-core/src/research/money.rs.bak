//! Notional money beside R (pre-registration: "Size: risk 0.6% of equity per
//! trade, below the minimum volume → skipped. This affects money only; R is
//! unaffected."). Labelled notional: a $10,000 starting equity, compounding.
//!
//! - Equity at an entry is the start plus the money of every trade that
//!   exited on an EARLIER day (same-day exits are not yet counted).
//! - Volume = floor to the step of (0.6% of that equity ÷ (stop distance ×
//!   USD per unit of price)); below the broker's minimum the trade is skipped
//!   (no money), never rounded up.
//! - Money = net price move per unit × volume × USD per unit of price at exit.

use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyTrade {
    pub entry_date: NaiveDate,
    pub exit_date: NaiveDate,
    /// Initial stop distance, price units.
    pub stop_distance: f64,
    /// Net R × stop distance: the net move per unit of volume, price units.
    pub net_per_unit: f64,
    /// USD per one price unit per unit of volume (quote currency → USD).
    pub usd_per_price_entry: f64,
    pub usd_per_price_exit: f64,
    /// Broker volume limits in units.
    pub min_units: f64,
    pub step_units: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Money {
    pub label: &'static str,
    pub start_equity: f64,
    pub risk_pct: f64,
    pub end_equity: f64,
    pub sized: usize,
    pub skipped_below_min_volume: usize,
    /// Per trade, in the order given: `None` when skipped.
    #[serde(skip)]
    pub per_trade: Vec<Option<f64>>,
}

pub fn notional(trades: &[MoneyTrade], start_equity: f64, risk_pct: f64) -> Money {
    let mut order: Vec<usize> = (0..trades.len()).collect();
    order.sort_by_key(|&i| (trades[i].entry_date, i));
    let mut per_trade = vec![None; trades.len()];
    // Money realised per exit date, applied when a later entry looks.
    let mut realised: Vec<(NaiveDate, f64)> = Vec::new();
    let (mut sized, mut skipped) = (0, 0);
    for &i in &order {
        let t = &trades[i];
        let equity = start_equity
            + realised
                .iter()
                .filter(|(d, _)| *d < t.entry_date)
                .map(|(_, m)| m)
                .sum::<f64>();
        let risk = equity * risk_pct / 100.0;
        let per_unit_risk = t.stop_distance * t.usd_per_price_entry;
        let units = if per_unit_risk > 0.0 && t.step_units > 0.0 && risk > 0.0 {
            (risk / per_unit_risk / t.step_units).floor() * t.step_units
        } else {
            0.0
        };
        if units < t.min_units || units <= 0.0 {
            skipped += 1;
            continue;
        }
        let m = t.net_per_unit * units * t.usd_per_price_exit;
        per_trade[i] = Some(m);
        realised.push((t.exit_date, m));
        sized += 1;
    }
    Money {
        label: "notional: $10,000 start, compounding, 0.6% risk per trade; below the minimum volume skipped",
        start_equity,
        risk_pct,
        end_equity: start_equity + realised.iter().map(|(_, m)| m).sum::<f64>(),
        sized,
        skipped_below_min_volume: skipped,
        per_trade,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2020, m, day).unwrap()
    }

    #[test]
    fn sizing_by_hand() {
        // Equity 10,000, risk 0.6% = 60. Stop 0.0050 per unit, USD per price
        // 1: 60 / 0.005 = 12,000 units, step 1,000 → 12,000. Net +0.0100 per
        // unit → +120. The second trade enters after the first exits: equity
        // 10,120, risk 60.72, stop 0.0100 → 6,072 → 6,000 units; net −0.0100
        // → −60. A third, entered on the first's exit day, still sees 10,000:
        // stop 1.0 → 60 units, below the 1,000 minimum: skipped.
        let t = |e, x, sd, net| MoneyTrade {
            entry_date: e,
            exit_date: x,
            stop_distance: sd,
            net_per_unit: net,
            usd_per_price_entry: 1.0,
            usd_per_price_exit: 1.0,
            min_units: 1000.0,
            step_units: 1000.0,
        };
        let m = notional(
            &[
                t(d(1, 2), d(1, 10), 0.005, 0.01),
                t(d(1, 11), d(1, 20), 0.01, -0.01),
                t(d(1, 10), d(1, 12), 1.0, 0.5),
            ],
            10_000.0,
            0.6,
        );
        assert!((m.per_trade[0].unwrap() - 120.0).abs() < 1e-9);
        assert!((m.per_trade[1].unwrap() + 60.0).abs() < 1e-9);
        assert_eq!(m.per_trade[2], None);
        assert_eq!((m.sized, m.skipped_below_min_volume), (2, 1));
        assert!((m.end_equity - 10_060.0).abs() < 1e-9);
    }
}
