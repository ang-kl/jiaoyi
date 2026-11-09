//! The deflated Sharpe ratio: D. H. Bailey and M. López de Prado, "The
//! Deflated Sharpe Ratio: Correcting for Selection Bias, Backtest Overfitting
//! and Non-Normality", Journal of Portfolio Management 40(5), 2014.
//!
//! - Expected maximum Sharpe ratio of N trials whose Sharpe ratios have
//!   variance V: SR0 = √V × ((1 − γ) Φ⁻¹(1 − 1/N) + γ Φ⁻¹(1 − 1/(N e))),
//!   γ the Euler–Mascheroni constant.
//! - DSR = Φ((SR − SR0) √(T − 1) / √(1 − γ3 SR + (γ4 − 1)/4 SR²)), with T
//!   observations, γ3 their skewness and γ4 their kurtosis (not in excess).
//!
//! Here the observations are trades and SR is per trade (mean R ÷ sd R).

use super::normal::{cdf, inv_cdf};

const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// SR0: the Sharpe ratio the best of `n_trials` would show by luck alone.
pub fn expected_max_sharpe(var_sr: f64, n_trials: usize) -> Option<f64> {
    if n_trials < 2 || var_sr.is_nan() || var_sr < 0.0 {
        return None;
    }
    let n = n_trials as f64;
    Some(
        var_sr.sqrt()
            * ((1.0 - EULER_GAMMA) * inv_cdf(1.0 - 1.0 / n)
                + EULER_GAMMA * inv_cdf(1.0 - 1.0 / (n * std::f64::consts::E))),
    )
}

/// The probability that the true Sharpe ratio exceeds SR0, given the
/// observed `sr` over `t` observations with skewness `skew` and kurtosis `kurt`.
pub fn deflated_sharpe(sr: f64, t: usize, skew: f64, kurt: f64, sr0: f64) -> Option<f64> {
    if t < 2 {
        return None;
    }
    let denom = 1.0 - skew * sr + (kurt - 1.0) / 4.0 * sr * sr;
    if denom.is_nan() || denom <= 0.0 {
        return None;
    }
    Some(cdf((sr - sr0) * ((t - 1) as f64).sqrt() / denom.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_an_independent_implementation() {
        // The paper's two formulas, computed with Python's
        // statistics.NormalDist (independent of this code):
        // V = 0.01, N = 9 → SR0 = 0.15207781907028006;
        // SR 0.3, T 100, skew −0.5, kurt 4 → DSR = 0.908878231690998.
        let s0 = expected_max_sharpe(0.01, 9).unwrap();
        assert!((s0 - 0.15207781907028006).abs() < 1e-12, "{s0}");
        let d = deflated_sharpe(0.3, 100, -0.5, 4.0, s0).unwrap();
        assert!((d - 0.908878231690998).abs() < 1e-12, "{d}");
        // V = 0.0004 → SR0 = 0.030415563814056013;
        // SR 0.05, T 500, skew 1.2, kurt 6 → DSR = 0.6738182637885344.
        let s0 = expected_max_sharpe(0.0004, 9).unwrap();
        assert!((s0 - 0.030415563814056013).abs() < 1e-12);
        let d = deflated_sharpe(0.05, 500, 1.2, 6.0, s0).unwrap();
        assert!((d - 0.6738182637885344).abs() < 1e-12, "{d}");
        assert_eq!(expected_max_sharpe(0.01, 1), None);
    }
}
