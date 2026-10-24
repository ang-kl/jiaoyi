//! ATR with Wilder smoothing (rulebook §5: "ATR(20) on daily bars, Wilder smoothing").
//!
//! True range of bar t (t ≥ 1): max(high − low, |high − close(t−1)|, |low − close(t−1)|).
//! The first ATR is the simple mean of the first `period` true ranges, which ends
//! at bar `period`. Each later ATR is (ATR(t−1) × (period − 1) + TR(t)) ÷ period.
//! J. Welles Wilder, *New Concepts in Technical Trading Systems* (1978).

/// High, low and close of one closed bar, in price units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hlc {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Wilder ATR at the last bar. `None` when there are fewer than `period + 1`
/// bars (each true range needs a previous close), or `period` is zero.
pub fn atr_wilder(bars: &[Hlc], period: usize) -> Option<f64> {
    if period == 0 || bars.len() < period + 1 {
        return None;
    }
    let tr = |i: usize| {
        let (b, prev) = (bars[i], bars[i - 1].close);
        (b.high - b.low)
            .max((b.high - prev).abs())
            .max((b.low - prev).abs())
    };
    let n = period as f64;
    let mut atr = (1..=period).map(tr).sum::<f64>() / n;
    for i in (period + 1)..bars.len() {
        atr = (atr * (n - 1.0) + tr(i)) / n;
    }
    Some(atr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(high: f64, low: f64, close: f64) -> Hlc {
        Hlc { high, low, close }
    }

    // Hand-calculated: true ranges of bars 2..6 are 2, 3, 1, 4, 1.
    // ATR(3): mean(2,3,1) = 2 at bar 4; (2·2+4)/3 = 8/3 at bar 5;
    // (8/3·2+1)/3 = 19/9 at bar 6.
    const BARS: [Hlc; 6] = [
        Hlc {
            high: 10.0,
            low: 8.0,
            close: 9.0,
        },
        Hlc {
            high: 11.0,
            low: 9.0,
            close: 10.0,
        },
        Hlc {
            high: 12.0,
            low: 9.0,
            close: 11.0,
        },
        Hlc {
            high: 11.0,
            low: 10.0,
            close: 10.0,
        },
        Hlc {
            high: 14.0,
            low: 10.0,
            close: 13.0,
        },
        Hlc {
            high: 13.0,
            low: 12.0,
            close: 12.0,
        },
    ];

    #[test]
    fn wilder_atr_matches_hand_calculation() {
        assert_eq!(atr_wilder(&BARS[..4], 3), Some(2.0));
        assert!((atr_wilder(&BARS[..5], 3).unwrap() - 8.0 / 3.0).abs() < 1e-12);
        assert!((atr_wilder(&BARS, 3).unwrap() - 19.0 / 9.0).abs() < 1e-12);
    }

    #[test]
    fn gap_counts_in_true_range() {
        // Previous close 10, then a bar entirely above it: TR = 15 − 10 = 5, not 1.
        let bars = [b(11.0, 9.0, 10.0), b(15.0, 14.0, 14.5)];
        assert_eq!(atr_wilder(&bars, 1), Some(5.0));
    }

    #[test]
    fn too_few_bars_gives_none() {
        let bars: Vec<Hlc> = (0..20).map(|_| b(2.0, 1.0, 1.5)).collect();
        assert_eq!(atr_wilder(&bars, 20), None);
        let mut more = bars.clone();
        more.push(b(2.0, 1.0, 1.5));
        assert_eq!(atr_wilder(&more, 20), Some(1.0));
        assert_eq!(atr_wilder(&more, 0), None);
    }
}
