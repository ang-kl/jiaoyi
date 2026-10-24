//! Information-driven bars from quote updates.
//!
//! cTrader streams quotes (bid/ask), not trades. So:
//! - a "tick" here is one quote update after which both bid and ask are known;
//! - the price used for the tick rule is the mid, (bid + ask) / 2.
//!
//! Prices are integers in cTrader's relative unit (1/100000 of a price), and the
//! mid is kept as `mid2 = bid + ask` (twice the mid) so it stays exact.
//!
//! Two builders:
//! - [`TickBarBuilder`]: a bar closes every N ticks (setting `bars.tick.updates_per_bar`).
//! - [`ImbalanceBarBuilder`]: tick-imbalance bars as defined by López de Prado,
//!   *Advances in Financial Machine Learning* (2018), §2.3.2.1:
//!   b_t = sign(Δp_t), or b_{t−1} when Δp_t = 0 (the tick rule);
//!   θ_T = Σ b_t over the bar; the bar closes at the first T with
//!   |θ_T| ≥ E₀[T] · |2P[b_t = 1] − 1|, where E₀[T] is an exponentially weighted
//!   mean of past bar lengths and 2P[b_t = 1] − 1 an exponentially weighted mean
//!   of past bars' θ_T / T. Here p_t is the mid, because there are no trades.
//!
//! The book leaves the start-up values and degenerate cases open. These are
//! settings, not constants: the initial E₀[T] and |2P − 1|, the EWMA span, a
//! floor on |2P − 1| (without it a balanced bar sets the threshold to zero and
//! every later tick closes a bar), and a maximum bar length (without it a
//! threshold that has grown too large never closes a bar). Each bar records
//! which rule closed it.

use serde::Serialize;

/// One quote update with both sides known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quote {
    /// Receive or broker time, milliseconds since the Unix epoch.
    pub ts_ms: i64,
    pub bid: i64,
    pub ask: i64,
}

impl Quote {
    /// Twice the mid price, exact.
    pub fn mid2(&self) -> i64 {
        self.bid + self.ask
    }
}

/// Carries the last bid and ask forward, because a spot event may hold only
/// the side that changed. Yields a [`Quote`] once both sides are known.
#[derive(Debug, Clone, Default)]
pub struct QuoteState {
    bid: Option<i64>,
    ask: Option<i64>,
}

impl QuoteState {
    pub fn update(&mut self, ts_ms: i64, bid: Option<i64>, ask: Option<i64>) -> Option<Quote> {
        if bid.is_none() && ask.is_none() {
            return None;
        }
        if bid.is_some() {
            self.bid = bid;
        }
        if ask.is_some() {
            self.ask = ask;
        }
        Some(Quote {
            ts_ms,
            bid: self.bid?,
            ask: self.ask?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    TickCount,
    Imbalance,
    MaxTicks,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bar {
    pub start_ms: i64,
    pub end_ms: i64,
    pub ticks: u32,
    pub open_mid2: i64,
    pub high_mid2: i64,
    pub low_mid2: i64,
    pub close_mid2: i64,
    pub close_bid: i64,
    pub close_ask: i64,
    /// θ_T, the sum of tick-rule signs over the bar.
    pub imbalance: i64,
    pub close_reason: CloseReason,
    /// The imbalance threshold in force while the bar formed (imbalance bars only).
    pub threshold: Option<f64>,
}

impl Bar {
    /// A `mid2` value as a price.
    pub fn price(mid2: i64) -> f64 {
        mid2 as f64 / 200_000.0
    }
}

/// The tick rule on the mid. The first tick, and every unchanged tick before
/// the first change, has sign 0 (there is no earlier change to carry).
#[derive(Debug, Clone, Default)]
pub struct TickRule {
    prev_mid2: Option<i64>,
    prev_sign: i64,
}

impl TickRule {
    pub fn sign(&mut self, mid2: i64) -> i64 {
        let b = match self.prev_mid2 {
            Some(p) if mid2 > p => 1,
            Some(p) if mid2 < p => -1,
            _ => self.prev_sign,
        };
        self.prev_mid2 = Some(mid2);
        self.prev_sign = b;
        b
    }
}

#[derive(Debug, Clone)]
struct Forming {
    start_ms: i64,
    end_ms: i64,
    ticks: u32,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
    bid: i64,
    ask: i64,
    theta: i64,
}

impl Forming {
    fn new(q: &Quote) -> Self {
        let m = q.mid2();
        Forming {
            start_ms: q.ts_ms,
            end_ms: q.ts_ms,
            ticks: 0,
            open: m,
            high: m,
            low: m,
            close: m,
            bid: q.bid,
            ask: q.ask,
            theta: 0,
        }
    }

    fn add(&mut self, q: &Quote, sign: i64) {
        let m = q.mid2();
        self.end_ms = q.ts_ms;
        self.ticks += 1;
        self.high = self.high.max(m);
        self.low = self.low.min(m);
        self.close = m;
        self.bid = q.bid;
        self.ask = q.ask;
        self.theta += sign;
    }

    fn finish(self, reason: CloseReason, threshold: Option<f64>) -> Bar {
        Bar {
            start_ms: self.start_ms,
            end_ms: self.end_ms,
            ticks: self.ticks,
            open_mid2: self.open,
            high_mid2: self.high,
            low_mid2: self.low,
            close_mid2: self.close,
            close_bid: self.bid,
            close_ask: self.ask,
            imbalance: self.theta,
            close_reason: reason,
            threshold,
        }
    }
}

/// Tick bars: a bar closes after every `n` ticks.
#[derive(Debug, Clone)]
pub struct TickBarBuilder {
    n: u32,
    rule: TickRule,
    forming: Option<Forming>,
}

impl TickBarBuilder {
    /// `n` comes from the setting `bars.tick.updates_per_bar`; zero is refused.
    pub fn new(n: u32) -> Result<Self, String> {
        if n == 0 {
            return Err("ticks per bar must be at least 1".into());
        }
        Ok(TickBarBuilder {
            n,
            rule: TickRule::default(),
            forming: None,
        })
    }

    pub fn push(&mut self, q: Quote) -> Option<Bar> {
        let sign = self.rule.sign(q.mid2());
        let f = self.forming.get_or_insert_with(|| Forming::new(&q));
        f.add(&q, sign);
        if f.ticks >= self.n {
            return self
                .forming
                .take()
                .map(|f| f.finish(CloseReason::TickCount, None));
        }
        None
    }
}

/// Parameters of tick-imbalance bars. Each is a setting (`bars.imbalance.*`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ImbalanceParams {
    /// E₀[T] before any bar has closed.
    pub initial_expected_ticks: f64,
    /// |2P[b=1] − 1| before any bar has closed.
    pub initial_expected_imbalance: f64,
    /// EWMA span in bars; the weight of the newest bar is 2 / (span + 1).
    pub ewma_span_bars: f64,
    /// Floor on |2P[b=1] − 1| used in the threshold.
    pub min_expected_imbalance: f64,
    /// A bar closes at this many ticks whatever its imbalance.
    pub max_ticks_per_bar: u32,
}

impl ImbalanceParams {
    pub fn check(&self) -> Result<(), String> {
        if self.initial_expected_ticks.is_nan() || self.initial_expected_ticks < 1.0 {
            return Err("initial expected ticks must be at least 1".into());
        }
        if !(self.initial_expected_imbalance > 0.0 && self.initial_expected_imbalance <= 1.0) {
            return Err("initial expected imbalance must be in (0, 1]".into());
        }
        if self.ewma_span_bars.is_nan() || self.ewma_span_bars < 1.0 {
            return Err("EWMA span must be at least 1 bar".into());
        }
        if !(self.min_expected_imbalance > 0.0 && self.min_expected_imbalance <= 1.0) {
            return Err("minimum expected imbalance must be in (0, 1]".into());
        }
        if self.max_ticks_per_bar == 0 {
            return Err("maximum ticks per bar must be at least 1".into());
        }
        Ok(())
    }
}

/// Tick-imbalance bars (López de Prado), on the mid.
#[derive(Debug, Clone)]
pub struct ImbalanceBarBuilder {
    p: ImbalanceParams,
    expected_ticks: f64,
    /// EWMA of θ_T / T, i.e. the estimate of 2P[b=1] − 1 (signed).
    expected_imbalance: f64,
    rule: TickRule,
    forming: Option<Forming>,
}

impl ImbalanceBarBuilder {
    pub fn new(p: ImbalanceParams) -> Result<Self, String> {
        p.check()?;
        Ok(ImbalanceBarBuilder {
            p,
            expected_ticks: p.initial_expected_ticks,
            expected_imbalance: p.initial_expected_imbalance,
            rule: TickRule::default(),
            forming: None,
        })
    }

    /// E₀[T] · max(|2P − 1|, floor).
    pub fn threshold(&self) -> f64 {
        self.expected_ticks
            * self
                .expected_imbalance
                .abs()
                .max(self.p.min_expected_imbalance)
    }

    pub fn expected_ticks(&self) -> f64 {
        self.expected_ticks
    }

    pub fn expected_imbalance(&self) -> f64 {
        self.expected_imbalance
    }

    pub fn push(&mut self, q: Quote) -> Option<Bar> {
        let sign = self.rule.sign(q.mid2());
        let threshold = self.threshold();
        let f = self.forming.get_or_insert_with(|| Forming::new(&q));
        f.add(&q, sign);
        let reason = if (f.theta.abs() as f64) >= threshold && f.theta != 0 {
            CloseReason::Imbalance
        } else if f.ticks >= self.p.max_ticks_per_bar {
            CloseReason::MaxTicks
        } else {
            return None;
        };
        let f = self.forming.take()?;
        let (t, theta) = (f.ticks as f64, f.theta as f64);
        let alpha = 2.0 / (self.p.ewma_span_bars + 1.0);
        self.expected_ticks = alpha * t + (1.0 - alpha) * self.expected_ticks;
        self.expected_imbalance = alpha * (theta / t) + (1.0 - alpha) * self.expected_imbalance;
        Some(f.finish(reason, Some(threshold)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quote with mid2 = m (bid = ask = m / 2 would lose the half; use bid = m − 1, ask = 1).
    fn q(ts: i64, mid2: i64) -> Quote {
        Quote {
            ts_ms: ts,
            bid: mid2 - 1,
            ask: 1,
        }
    }

    #[test]
    fn quote_state_carries_the_missing_side_forward() {
        let mut s = QuoteState::default();
        assert_eq!(s.update(1, Some(100), None), None); // ask not known yet
        assert_eq!(
            s.update(2, None, Some(103)),
            Some(Quote {
                ts_ms: 2,
                bid: 100,
                ask: 103
            })
        );
        assert_eq!(
            s.update(3, Some(101), None),
            Some(Quote {
                ts_ms: 3,
                bid: 101,
                ask: 103
            })
        );
        assert_eq!(s.update(4, None, None), None);
    }

    #[test]
    fn tick_rule_carries_the_last_sign_on_unchanged_mids() {
        let mut r = TickRule::default();
        let signs: Vec<i64> = [100, 100, 102, 102, 101, 101, 103]
            .iter()
            .map(|m| r.sign(*m))
            .collect();
        assert_eq!(signs, vec![0, 0, 1, 1, -1, -1, 1]);
    }

    #[test]
    fn tick_bars_close_every_n_updates_with_correct_ohlc() {
        let mut b = TickBarBuilder::new(3).unwrap();
        let mids = [100, 104, 98, 101, 101, 99, 120];
        let bars: Vec<Bar> = mids
            .iter()
            .enumerate()
            .filter_map(|(i, m)| b.push(q(i as i64, *m)))
            .collect();
        assert_eq!(bars.len(), 2, "the 7th update is still forming");
        let b0 = &bars[0];
        assert_eq!(
            (
                b0.open_mid2,
                b0.high_mid2,
                b0.low_mid2,
                b0.close_mid2,
                b0.ticks
            ),
            (100, 104, 98, 98, 3)
        );
        assert_eq!((b0.start_ms, b0.end_ms), (0, 2));
        assert_eq!(b0.close_reason, CloseReason::TickCount);
        let b1 = &bars[1];
        assert_eq!(
            (b1.open_mid2, b1.high_mid2, b1.low_mid2, b1.close_mid2),
            (101, 101, 99, 99)
        );
        // signs: 0,+1,-1 | +1,+1(carried),-1
        assert_eq!((b0.imbalance, b1.imbalance), (0, 1));
        assert!(TickBarBuilder::new(0).is_err());
    }

    fn params() -> ImbalanceParams {
        ImbalanceParams {
            initial_expected_ticks: 4.0,
            initial_expected_imbalance: 0.5,
            ewma_span_bars: 3.0, // α = 0.5
            min_expected_imbalance: 0.1,
            max_ticks_per_bar: 100,
        }
    }

    #[test]
    fn imbalance_bars_match_a_hand_calculation() {
        // Bar 1: threshold 4 × 0.5 = 2. Mids 100 (b 0), 102 (+1), 104 (+1): θ = 2 → close, T = 3.
        //   E[T] = 0.5·3 + 0.5·4 = 3.5; E[b] = 0.5·(2/3) + 0.5·0.5 = 7/12.
        // Bar 2: threshold 3.5 × 7/12 = 49/24 ≈ 2.0417.
        //   104 (carried +1, θ 1), 102 (−1, 0), 100 (−1, −1), 98 (−1, −2), 96 (−1, −3) → close, T = 5.
        //   E[T] = 0.5·5 + 0.5·3.5 = 4.25; E[b] = 0.5·(−3/5) + 0.5·(7/12) = −1/120.
        // Bar 3: |E[b]| = 1/120 is below the floor 0.1, so threshold = 4.25 × 0.1 = 0.425.
        //   96 (carried −1, θ −1) → |θ| ≥ 0.425 → close after one tick.
        let mut b = ImbalanceBarBuilder::new(params()).unwrap();
        let mids = [100, 102, 104, 104, 102, 100, 98, 96, 96];
        let bars: Vec<Bar> = mids
            .iter()
            .enumerate()
            .filter_map(|(i, m)| b.push(q(i as i64, *m)))
            .collect();
        assert_eq!(bars.len(), 3);

        assert_eq!((bars[0].ticks, bars[0].imbalance), (3, 2));
        assert_eq!(bars[0].threshold, Some(2.0));
        assert_eq!(bars[0].close_reason, CloseReason::Imbalance);

        assert_eq!((bars[1].ticks, bars[1].imbalance), (5, -3));
        assert!((bars[1].threshold.unwrap() - 49.0 / 24.0).abs() < 1e-12);
        assert_eq!(
            (bars[1].open_mid2, bars[1].close_mid2, bars[1].low_mid2),
            (104, 96, 96)
        );

        assert_eq!((bars[2].ticks, bars[2].imbalance), (1, -1));
        assert!((bars[2].threshold.unwrap() - 0.425).abs() < 1e-12);

        // After bar 3: E[T] = 0.5·1 + 0.5·4.25 = 2.625; E[b] = 0.5·(−1) + 0.5·(−1/120).
        assert!((b.expected_ticks() - 2.625).abs() < 1e-12);
        assert!((b.expected_imbalance() - (-0.5 - 1.0 / 240.0)).abs() < 1e-12);
    }

    #[test]
    fn a_flat_market_closes_at_max_ticks_not_never() {
        let mut p = params();
        p.max_ticks_per_bar = 5;
        let mut b = ImbalanceBarBuilder::new(p).unwrap();
        let bars: Vec<Bar> = (0..12).filter_map(|i| b.push(q(i, 100))).collect();
        assert_eq!(bars.len(), 2);
        assert!(bars
            .iter()
            .all(|b| b.close_reason == CloseReason::MaxTicks && b.ticks == 5));
        assert!(bars.iter().all(|b| b.imbalance == 0));
    }

    #[test]
    fn invalid_parameters_are_refused() {
        let mut p = params();
        p.min_expected_imbalance = 0.0;
        assert!(ImbalanceBarBuilder::new(p).is_err());
        let mut p = params();
        p.initial_expected_ticks = 0.5;
        assert!(ImbalanceBarBuilder::new(p).is_err());
        let mut p = params();
        p.max_ticks_per_bar = 0;
        assert!(ImbalanceBarBuilder::new(p).is_err());
    }
}
