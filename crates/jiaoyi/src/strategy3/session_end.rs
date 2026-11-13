//! Closed-market session exits: no alpha or price-protection changes.
use super::{prod, END_RETRY};
use chrono::{DateTime, Duration, Utc};

pub const POLICY: &str = "market_closed_reopen_v18455";
const SLOW_POLL: Duration = Duration::minutes(30);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarketClosedRetry {
    pub rejected_at: DateTime<Utc>,
    pub next_probe_at: DateTime<Utc>,
    quote_wake_armed: bool,
    last_fresh_quote: Option<DateTime<Utc>>,
}
impl MarketClosedRetry {
    pub fn new(
        symbol: &str,
        rejected_at: DateTime<Utc>,
        fresh: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Self {
        let next_probe_at = if prod::broker_weekly_open(symbol, rejected_at) {
            rejected_at + SLOW_POLL
        } else {
            prod::next_opening_observation_start(symbol, rejected_at)
                .unwrap_or(rejected_at + SLOW_POLL)
        };
        Self {
            rejected_at,
            next_probe_at,
            quote_wake_armed: fresh.is_none(),
            last_fresh_quote: fresh.map(|(b, a)| b.max(a)),
        }
    }

    /// The caller has already validated both quote sides for freshness/order.
    /// A quiet feed re-arms the first-new-quote wake. Continuous pre-open or
    /// holiday quotes do not defeat the slow/scheduled backoff after rejection:
    /// a quote alone is not evidence that the broker will execute an order.
    pub fn due(
        &mut self,
        now: DateTime<Utc>,
        fresh: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> bool {
        if let Some((bid_at, ask_at)) = fresh {
            let latest = bid_at.max(ask_at);
            if self.last_fresh_quote.is_some_and(|last| {
                latest - last > Duration::from_std(crate::fast_monitor::MAX_QUOTE_AGE).unwrap()
            }) {
                self.quote_wake_armed = true;
            }
            self.last_fresh_quote = Some(
                self.last_fresh_quote
                    .map_or(latest, |last| last.max(latest)),
            );
        } else {
            self.quote_wake_armed = true;
        }
        if now < self.rejected_at + Duration::from_std(END_RETRY).unwrap_or_default() {
            return false;
        }
        now >= self.next_probe_at
            || (self.quote_wake_armed
                && fresh.is_some_and(|(b, a)| b > self.rejected_at && a > self.rejected_at))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn weekend_deferral_and_first_two_sided_reopen_quote() {
        let rejected = at("2026-11-13T21:55:00Z");
        let mut retry = MarketClosedRetry::new("XAUUSD", rejected, None);
        assert_eq!(retry.next_probe_at, at("2026-11-15T23:01:00Z"));
        for seconds in [1, 10, 60, 1800, 86400] {
            assert!(!retry.due(rejected + Duration::seconds(seconds), None));
        }
        let early = rejected + Duration::seconds(5);
        assert!(
            !retry.due(early, Some((early, early))),
            "ten-second rejection floor"
        );
        let quote = rejected + Duration::seconds(11);
        assert!(
            !retry.due(quote, Some((quote, rejected))),
            "both sides must be newer"
        );
        assert!(
            retry.due(quote, Some((quote, quote))),
            "fresh reopening quotes wake without waiting for the schedule"
        );
    }

    #[test]
    fn continuous_closed_market_quotes_cannot_create_a_ten_second_loop() {
        for symbol in ["XAUUSD", "UNKNOWN"] {
            let rejected = at("2026-11-12T12:00:00Z");
            let mut retry = MarketClosedRetry::new(symbol, rejected, Some((rejected, rejected)));
            assert_eq!(retry.next_probe_at, rejected + Duration::minutes(30));
            for seconds in 1..1800 {
                let q = rejected + Duration::seconds(seconds);
                assert!(
                    !retry.due(q, Some((q, q))),
                    "continuous quotes do not prove executable trading"
                );
            }
            let probe = rejected + Duration::minutes(30);
            assert!(retry.due(probe, Some((probe, probe))));
            let mut next = MarketClosedRetry::new(symbol, probe, Some((probe, probe)));
            let q = probe + Duration::seconds(1);
            assert!(!next.due(q, Some((q, q))));
            // A feed gap arms a new first-quote wake, including between actor polls.
            let reopen = probe + Duration::seconds(12);
            assert!(next.due(reopen, Some((reopen, reopen))));
        }
    }

    #[test]
    fn daily_break_is_future_and_stale_quotes_rearm_without_sends() {
        let rejected = at("2026-11-12T21:59:00Z");
        let mut retry = MarketClosedRetry::new("XAGUSD", rejected, Some((rejected, rejected)));
        assert_eq!(retry.next_probe_at, at("2026-11-12T23:01:00Z"));
        assert!(!retry.due(rejected + Duration::minutes(15), None));
        let reopen = at("2026-11-12T23:01:00Z");
        assert!(retry.due(reopen, Some((reopen, reopen))));
        let holiday = MarketClosedRetry::new("XAGUSD", reopen, Some((reopen, reopen)));
        assert_eq!(holiday.next_probe_at, reopen + Duration::minutes(30));
    }
}
