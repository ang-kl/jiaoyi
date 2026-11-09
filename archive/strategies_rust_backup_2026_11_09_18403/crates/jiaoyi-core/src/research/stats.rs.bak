//! Summary statistics of trade results in R.

use serde::{Deserialize, Serialize};

pub fn mean(xs: &[f64]) -> Option<f64> {
    (!xs.is_empty()).then(|| xs.iter().sum::<f64>() / xs.len() as f64)
}

/// Gross wins ÷ gross losses. `None` without trades; `f64::INFINITY` with
/// wins and no loss; 0 with losses and no win.
pub fn profit_factor(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let wins: f64 = xs.iter().filter(|x| **x > 0.0).sum();
    let losses: f64 = -xs.iter().filter(|x| **x < 0.0).sum::<f64>();
    Some(if losses > 0.0 {
        wins / losses
    } else if wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    })
}

/// The `p` quantile taken at the LOWER order statistic,
/// `sorted[floor(p × (n − 1))]`: the stricter value for a lower bound.
pub fn lower_quantile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let i = (p * (sorted.len() - 1) as f64).floor() as usize;
    Some(sorted[i.min(sorted.len() - 1)])
}

/// The `p` quantile taken at the UPPER order statistic,
/// `sorted[ceil(p × (n − 1))]`: the stricter value for a threshold to beat.
pub fn upper_quantile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let i = (p * (sorted.len() - 1) as f64).ceil() as usize;
    Some(sorted[i.min(sorted.len() - 1)])
}

/// Moments of a sample of trade results, for the Sharpe ratio and its
/// deflation. Skewness and kurtosis are the population moments
/// m3 / m2^1.5 and m4 / m2² (kurtosis not in excess: 3 for a normal).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Moments {
    pub n: usize,
    pub mean: f64,
    /// Sample standard deviation (n − 1).
    pub sd: f64,
    pub skew: f64,
    pub kurt: f64,
}

pub fn moments(xs: &[f64]) -> Option<Moments> {
    let n = xs.len();
    if n < 2 {
        return None;
    }
    let m = mean(xs)?;
    let c = |k: i32| xs.iter().map(|x| (x - m).powi(k)).sum::<f64>() / n as f64;
    let (m2, m3, m4) = (c(2), c(3), c(4));
    let sd = (m2 * n as f64 / (n - 1) as f64).sqrt();
    Some(Moments {
        n,
        mean: m,
        sd,
        skew: if m2 > 0.0 { m3 / m2.powf(1.5) } else { 0.0 },
        kurt: if m2 > 0.0 { m4 / (m2 * m2) } else { 3.0 },
    })
}

/// n, mean, total, profit factor and win count of a set of results in R.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RStats {
    pub n: usize,
    pub mean_r: Option<f64>,
    pub total_r: f64,
    /// `None` without trades; a profit factor with no losing trade is
    /// reported as `null` with `pf_infinite: true`.
    pub profit_factor: Option<f64>,
    pub pf_infinite: bool,
    pub wins: usize,
    pub losses: usize,
}

pub fn r_stats(xs: &[f64]) -> RStats {
    let pf = profit_factor(xs);
    RStats {
        n: xs.len(),
        mean_r: mean(xs),
        total_r: xs.iter().sum(),
        profit_factor: pf.filter(|v| v.is_finite()),
        pf_infinite: pf.is_some_and(|v| v.is_infinite()),
        wins: xs.iter().filter(|x| **x > 0.0).count(),
        losses: xs.iter().filter(|x| **x < 0.0).count(),
    }
}

impl RStats {
    /// The profit factor as a number, infinite when there is no loss.
    pub fn pf(&self) -> Option<f64> {
        if self.pf_infinite {
            Some(f64::INFINITY)
        } else {
            self.profit_factor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_calculated_summaries() {
        let r = [2.0, -1.0, 0.5, -1.0, 3.5];
        // mean = 4 / 5; PF = (2 + 0.5 + 3.5) / (1 + 1) = 3.
        let s = r_stats(&r);
        assert_eq!((s.n, s.total_r, s.wins, s.losses), (5, 4.0, 3, 2));
        assert!((s.mean_r.unwrap() - 0.8).abs() < 1e-15);
        assert_eq!(s.profit_factor, Some(3.0));
        assert_eq!(profit_factor(&[1.0]), Some(f64::INFINITY));
        assert_eq!(profit_factor(&[-1.0]), Some(0.0));
        assert_eq!(profit_factor(&[]), None);
        // Order statistics on 0..=10: p = 0.25 → index 2.5 → lower 2, upper 3.
        let v: Vec<f64> = (0..=10).map(f64::from).collect();
        assert_eq!(lower_quantile(&v, 0.25), Some(2.0));
        assert_eq!(upper_quantile(&v, 0.25), Some(3.0));
        assert_eq!(lower_quantile(&v, 0.2), Some(2.0));
        assert_eq!(upper_quantile(&v, 0.2), Some(2.0));
        // Moments of 1, 2, 3, 4: mean 2.5, m2 = 1.25, sd = √(5/3), skew 0,
        // m4 = (2·1.5⁴ + 2·0.5⁴) / 4 = 2.5625, kurt = 2.5625 / 1.5625 = 1.64.
        let m = moments(&[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert!((m.sd - (5.0f64 / 3.0).sqrt()).abs() < 1e-15);
        assert!(m.skew.abs() < 1e-15);
        assert!((m.kurt - 1.64).abs() < 1e-12);
    }
}
