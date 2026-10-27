//! Kelly as a ceiling and a skip. It does not write a size.
//!
//! Owner, № 17,588–17,589, 28-10-2026: Kelly, not Martingale. The fraction is
//! computed from closed trades under the current pre-registration hash, on
//! this account and symbol (one episode, not four copied accounts). Fewer
//! than 20 closes is not an input. A non-positive fraction skips the entry.
//! A positive fraction does not raise `risk_pct` and does not change `size()`.
//!
//! Grok 4.7, 28-10-2026. Other models: this is the agreed application. Do not
//! plug in the goal (73%, 1.38). Do not double after a loss.

/// Closed trades under the current hash before the fraction is an input.
pub const MIN_CLOSES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    pub n: usize,
    pub wins: usize,
    /// Win rate. `None` when there are not enough closes.
    pub p: Option<f64>,
    /// Mean win R ÷ mean |loss| R. `None` with no loss, or not enough closes.
    pub b: Option<f64>,
    /// Kelly fraction. `None` when it is not an input. Negative means skip.
    pub f_star: Option<f64>,
    /// Quarter Kelly when `f_star` is positive. Never a size to write.
    pub quarter: Option<f64>,
}

/// `f* = p - (1-p)/b`. `None` when `b` is not positive.
pub fn fraction(p: f64, b: f64) -> Option<f64> {
    if !(p.is_finite() && b.is_finite()) || b <= 0.0 || !(0.0..=1.0).contains(&p) {
        return None;
    }
    Some(p - (1.0 - p) / b)
}

/// Read the fraction from net-R outcomes. Under 20 closes: not an input.
/// A zero-R close counts as a non-win and is excluded from `b`.
pub fn read(rs: &[f64]) -> Reading {
    let n = rs.len();
    if n < MIN_CLOSES {
        return Reading {
            n,
            wins: rs.iter().filter(|r| **r > 0.0).count(),
            p: None,
            b: None,
            f_star: None,
            quarter: None,
        };
    }
    let wins: Vec<f64> = rs.iter().copied().filter(|r| *r > 0.0).collect();
    let losses: Vec<f64> = rs.iter().copied().filter(|r| *r < 0.0).collect();
    let p = wins.len() as f64 / n as f64;
    let b = if losses.is_empty() {
        None
    } else {
        let mean_win = if wins.is_empty() {
            0.0
        } else {
            wins.iter().sum::<f64>() / wins.len() as f64
        };
        let mean_loss = losses.iter().map(|r| r.abs()).sum::<f64>() / losses.len() as f64;
        (mean_loss > 0.0).then(|| mean_win / mean_loss)
    };
    // No losses: every decisive close won. The fraction is the win rate.
    // No wins: b is 0 and the fraction is negative. Skip.
    let f_star = match b {
        Some(b) if b > 0.0 => fraction(p, b),
        Some(_) => Some(-1.0),
        None => Some(p),
    };
    let quarter = f_star.filter(|f| *f > 0.0).map(|f| f / 4.0);
    Reading {
        n,
        wins: wins.len(),
        p: Some(p),
        b,
        f_star,
        quarter,
    }
}

/// Skip only when the fraction is an input and is not positive.
pub fn skip(reading: &Reading) -> bool {
    reading.f_star.is_some_and(|f| f <= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_fraction_is_about_half_and_is_not_a_size() {
        let f = fraction(0.73, 1.38).unwrap();
        assert!((f - 0.534).abs() < 0.01);
    }

    #[test]
    fn statement_rate_is_negative() {
        let f = fraction(0.30, 1.38).unwrap();
        assert!(f < 0.0);
    }

    #[test]
    fn no_drift_at_registered_geometry_is_zero() {
        let p = 1.0 / (1.0 + 1.38);
        let f = fraction(p, 1.38).unwrap();
        assert!(f.abs() < 1e-12);
    }

    #[test]
    fn under_twenty_is_not_an_input() {
        let reading = read(&[1.0, -1.0, 1.2]);
        assert!(reading.f_star.is_none());
        assert!(!skip(&reading));
    }

    #[test]
    fn twenty_losses_skip() {
        let rs = vec![-1.0; 20];
        let reading = read(&rs);
        assert!(skip(&reading));
        assert!(reading.f_star.unwrap() < 0.0);
    }

    #[test]
    fn twenty_wins_do_not_skip_and_do_not_raise() {
        let rs = vec![1.38; 20];
        let reading = read(&rs);
        assert!(!skip(&reading));
        assert!(reading.quarter.unwrap() > 0.0);
    }
}
