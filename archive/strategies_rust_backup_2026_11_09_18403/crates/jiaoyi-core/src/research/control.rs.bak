//! The random-entry control (pre-registration: "1,000 random-entry runs on
//! the same symbols; same number of entries per symbol per year; same exit
//! rules and costs; direction random").
//!
//! One run, per symbol and calendar year, in date order:
//! - draw the year's entry count of distinct days uniformly from the year's
//!   eligible days (those the strategy could have traded on);
//! - take them in date order; a drawn day on which the symbol is still in a
//!   control trade is replaced by a day drawn uniformly from the year's
//!   remaining eligible days after that trade's exit (the symbol is flat);
//! - a draw that finds no such day is a shortfall, counted and reported;
//! - each entry is long or short by a fair coin, enters at the day's open
//!   (filled like any entry), and exits by H1's stop rules.
//!
//! The run's statistic is the mean net R of its trades. One seeded generator
//! drives all runs in order, so the seed reproduces every run.

use std::collections::{BTreeMap, BTreeSet};

use chrono::Datelike;
use serde::Serialize;

use super::h1::{run_trade, start_index, H1Params, Indicators, Series, Side};
use super::rng::SplitMix64;
use super::stats::{mean, upper_quantile};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Control {
    pub runs: usize,
    pub seed: u64,
    /// Entries asked for per run (the strategy's count).
    pub entries_per_run: usize,
    /// Entries that found no flat eligible day, summed over all runs.
    pub shortfall: u64,
    /// Mean net R of each run, sorted ascending.
    pub run_means: Vec<f64>,
    /// The 95th percentile of the run means, at the upper order statistic.
    pub p95: Option<f64>,
}

impl Control {
    /// The share of runs (0–100) whose mean is strictly below `x`.
    pub fn percentile_of(&self, x: f64) -> Option<f64> {
        (!self.run_means.is_empty()).then(|| {
            100.0 * self.run_means.iter().filter(|m| **m < x).count() as f64
                / self.run_means.len() as f64
        })
    }
}

/// `symbols[i]`: a series, its indicators, and the strategy's entry count
/// per calendar year on it.
pub fn random_entries(
    symbols: &[(&Series, &Indicators, BTreeMap<i32, usize>)],
    p: &H1Params,
    runs: usize,
    seed: u64,
) -> Control {
    let mut rng = SplitMix64::new(seed);
    let mut run_means = Vec::with_capacity(runs);
    let mut shortfall = 0u64;
    let entries_per_run = symbols
        .iter()
        .map(|(_, _, c)| c.values().sum::<usize>())
        .sum();
    for _ in 0..runs {
        let mut rs: Vec<f64> = Vec::new();
        for (s, ind, counts) in symbols {
            let start = start_index(s, p);
            // Eligible days per year.
            let mut days: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
            for t in start..=s.last {
                if ind.atr[t - 1].is_some() {
                    days.entry(s.dates[t].year()).or_default().push(t);
                }
            }
            let mut busy_until: Option<usize> = None;
            for (year, &n) in counts {
                let Some(pool) = days.get(year) else {
                    shortfall += n as u64;
                    continue;
                };
                // Draw n distinct days (partial Fisher–Yates).
                let mut idx: Vec<usize> = pool.clone();
                let k = n.min(idx.len());
                shortfall += (n - k) as u64;
                for i in 0..k {
                    let j = i + rng.below((idx.len() - i) as u64) as usize;
                    idx.swap(i, j);
                }
                let mut queue: BTreeSet<usize> = idx[..k].iter().copied().collect();
                let mut used: BTreeSet<usize> = queue.clone();
                while let Some(t) = queue.pop_first() {
                    let t = if busy_until.is_some_and(|b| t <= b) {
                        let after = busy_until.unwrap();
                        let free: Vec<usize> = pool
                            .iter()
                            .copied()
                            .filter(|d| *d > after && !used.contains(d))
                            .collect();
                        if free.is_empty() {
                            shortfall += 1;
                            continue;
                        }
                        let d = free[rng.below(free.len() as u64) as usize];
                        used.insert(d);
                        if queue.first().is_some_and(|q| *q < d) {
                            // Keep date order: take the earlier drawn day first.
                            queue.insert(d);
                            continue;
                        }
                        d
                    } else {
                        t
                    };
                    let side = if rng.coin() { Side::Long } else { Side::Short };
                    if let Some(tr) = run_trade(s, ind, p, side, t, s.open[t], None) {
                        busy_until = Some(tr.exit_index);
                        rs.push(tr.net_r);
                    }
                }
            }
        }
        if let Some(m) = mean(&rs) {
            run_means.push(m);
        }
    }
    run_means.sort_by(f64::total_cmp);
    Control {
        runs,
        seed,
        entries_per_run,
        shortfall,
        p95: upper_quantile(&run_means, 0.95),
        run_means,
    }
}

#[cfg(test)]
mod tests {
    use super::super::h1::tests::flat;
    use super::*;

    #[test]
    fn a_seed_reproduces_every_run() {
        let mut s = flat(900);
        // Some movement so trades end at different times.
        for i in 0..900 {
            let w = ((i as f64) / 9.0).sin() * 5.0;
            s.open[i] = 100.0 + w;
            s.close[i] = 100.0 + ((i as f64 + 0.5) / 9.0).sin() * 5.0;
            s.high[i] = s.open[i].max(s.close[i]) + 1.0;
            s.low[i] = s.open[i].min(s.close[i]) - 1.0;
        }
        let ind = Indicators::of(&s);
        let p = H1Params {
            breakout: 50,
            stop_atr: 3.0,
            slip_mult: 1.0,
        };
        let counts = BTreeMap::from([(2010, 3usize), (2011, 4)]);
        let syms = [(&s, &ind, counts)];
        let a = random_entries(&syms, &p, 50, 20_261_024);
        let b = random_entries(&syms, &p, 50, 20_261_024);
        assert_eq!(a, b);
        assert_eq!(a.run_means.len(), 50);
        assert_eq!(a.entries_per_run, 7);
        let c = random_entries(&syms, &p, 50, 1);
        assert_ne!(a.run_means, c.run_means);
        assert!(a.p95.is_some());
        assert_eq!(a.percentile_of(f64::INFINITY), Some(100.0));
        assert_eq!(a.percentile_of(f64::NEG_INFINITY), Some(0.0));
    }
}
