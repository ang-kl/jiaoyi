//! The week-block bootstrap of trade R (pre-registration, PASS rule 1).
//!
//! Trades are grouped into blocks by the ISO week of their entry. A resample
//! draws as many blocks as there are, with replacement, uniformly, and takes
//! the mean R of all trades in the drawn blocks. Weeks without an entry are
//! not blocks. The "95% lower bound" is read strictly as the 2.5th percentile
//! of the resampled means, at the lower order statistic.

use std::collections::BTreeMap;

use serde::Serialize;

use super::rng::SplitMix64;
use super::stats::{lower_quantile, upper_quantile};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bootstrap {
    pub resamples: usize,
    pub seed: u64,
    pub blocks: usize,
    /// 2.5th percentile of the resampled mean R (lower order statistic).
    pub mean_r_lower_2_5: f64,
    /// 97.5th percentile (upper order statistic).
    pub mean_r_upper_97_5: f64,
    /// The same range for an extra per-trade value (e.g. notional money),
    /// summed over the drawn trades, when one is given.
    pub sum_lower_2_5: Option<f64>,
    pub sum_upper_97_5: Option<f64>,
}

/// `trades`: (block key, R, optional extra value to sum). `None` without trades.
pub fn week_block<K: Ord + Copy>(
    trades: &[(K, f64, Option<f64>)],
    resamples: usize,
    seed: u64,
) -> Option<Bootstrap> {
    if trades.is_empty() || resamples == 0 {
        return None;
    }
    let mut by_week: BTreeMap<K, (f64, usize, f64, bool)> = BTreeMap::new();
    for (k, r, extra) in trades {
        let e = by_week.entry(*k).or_insert((0.0, 0, 0.0, true));
        e.0 += r;
        e.1 += 1;
        match extra {
            Some(x) => e.2 += x,
            None => e.3 = false,
        }
    }
    let blocks: Vec<(f64, usize, f64, bool)> = by_week.into_values().collect();
    let with_extra = blocks.iter().all(|b| b.3);
    let mut rng = SplitMix64::new(seed);
    let mut means = Vec::with_capacity(resamples);
    let mut sums = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let (mut r, mut n, mut x) = (0.0, 0usize, 0.0);
        for _ in 0..blocks.len() {
            let b = &blocks[rng.below(blocks.len() as u64) as usize];
            r += b.0;
            n += b.1;
            x += b.2;
        }
        means.push(r / n as f64);
        sums.push(x);
    }
    means.sort_by(f64::total_cmp);
    sums.sort_by(f64::total_cmp);
    Some(Bootstrap {
        resamples,
        seed,
        blocks: blocks.len(),
        mean_r_lower_2_5: lower_quantile(&means, 0.025)?,
        mean_r_upper_97_5: upper_quantile(&means, 0.975)?,
        sum_lower_2_5: with_extra.then(|| lower_quantile(&sums, 0.025)).flatten(),
        sum_upper_97_5: with_extra.then(|| upper_quantile(&sums, 0.975)).flatten(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_known_weeks() {
        // Week 1: R 1 and 3 (mean 2); week 2: R −2 (mean −2).
        // Resampling 2 blocks: {1,1} → mean 2, {1,2} or {2,1} → (1+3−2)/3 = 2/3,
        // {2,2} → −2, each pair with probability 1/4, 1/2, 1/4. The 2.5th
        // percentile is −2 and the 97.5th is 2 (each has probability 1/4, far
        // more than 2.5%, so 10,000 draws see both).
        let t = [
            (1, 1.0, Some(10.0)),
            (1, 3.0, Some(30.0)),
            (2, -2.0, Some(-20.0)),
        ];
        let b = week_block(&t, 10_000, 1).unwrap();
        assert_eq!(b.blocks, 2);
        assert_eq!(b.mean_r_lower_2_5, -2.0);
        assert_eq!(b.mean_r_upper_97_5, 2.0);
        // Money summed: {2,2} → −40, {1,1} → 80.
        assert_eq!(b.sum_lower_2_5, Some(-40.0));
        assert_eq!(b.sum_upper_97_5, Some(80.0));
        // One week only: every resample is that week.
        let one = week_block(&[(5, 0.5, None), (5, -0.1, None)], 100, 9).unwrap();
        assert!((one.mean_r_lower_2_5 - 0.2).abs() < 1e-15);
        assert_eq!(one.sum_lower_2_5, None);
        // Same seed, same result; another seed may differ.
        let t: Vec<(i32, f64, Option<f64>)> = (0..40)
            .map(|i| (i / 3, (i % 7) as f64 - 2.5, None))
            .collect();
        assert_eq!(week_block(&t, 500, 42), week_block(&t, 500, 42));
        assert!(week_block::<i32>(&[], 10, 1).is_none());
    }
}
