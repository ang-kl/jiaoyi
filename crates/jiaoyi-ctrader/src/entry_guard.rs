//! Entry-only validation at the final controlled point before a frame write.
//! Quotes are supplied by the socket reader, independently of strategy work.

use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::strategy3::{band, cost_reserve, targets::Dir};
use tokio::time::Instant;

use crate::trading::Side;

pub const POLICY: &str = "entry-dispatch-v1-2026-11-01";
pub const MAX_QUOTE_AGE: Duration = Duration::from_secs(6);

#[derive(Debug, Clone, Default)]
pub struct Quote {
    pub bid: Option<(f64, DateTime<Utc>)>,
    pub ask: Option<(f64, DateTime<Utc>)>,
}

#[derive(Debug, Clone)]
pub enum Signal {
    Trend { level: f64 },
    Mark { band: band::Band },
}

/// Prepared spread/commission and the registered slippage formula. A live
/// spread is deliberately not substituted for the registered spread input.
#[derive(Debug, Clone)]
pub struct Cost {
    pub prepared_spread: f64,
    pub commission_round_trip: f64,
    pub slippage_bps_per_side: f64,
    pub original_r: f64,
}

#[derive(Debug, Clone)]
pub struct EntryGuard {
    pub connection_generation: u64,
    pub account_id: i64,
    pub symbol_id: i64,
    pub side: Side,
    pub signal: Signal,
    pub cost: Option<Cost>,
    pub expires: Instant,
    /// Any account execution received during the pre-entry awaits invalidates
    /// the admission snapshot; the next allowed decision reconciles anew.
    pub account_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, serde::Serialize)]
#[error("{code}: {reason}")]
pub struct GuardRefusal {
    pub code: &'static str,
    pub reason: &'static str,
    /// Captured synchronously from the exact reader snapshot used by the guard.
    pub evidence: serde_json::Value,
}

impl EntryGuard {
    pub(crate) fn snapshot(
        &self,
        quote: Option<&Quote>,
        account_revision: u64,
        now: DateTime<Utc>,
        monotonic_now: Instant,
    ) -> serde_json::Value {
        let side = |q: Option<(f64, DateTime<Utc>)>| {
            q.map(|(price, received_at)| {
                serde_json::json!({
                    "price": price, "received_at": received_at,
                    "receipt_age_ms": now.signed_duration_since(received_at).num_milliseconds(),
                })
            })
        };
        serde_json::json!({
            "policy": POLICY, "account_id": self.account_id, "symbol_id": self.symbol_id,
            "selected_side": self.side, "checked_at": now,
            "bid": quote.and_then(|q| side(q.bid)), "ask": quote.and_then(|q| side(q.ask)),
            "account_revision": account_revision, "expected_account_revision": self.account_revision,
            "expected_connection_generation": self.connection_generation,
            "deadline_elapsed": monotonic_now >= self.expires,
            "remaining_dispatch_ms": self.expires.saturating_duration_since(monotonic_now).as_millis(),
            "max_quote_age_ms": MAX_QUOTE_AGE.as_millis(),
        })
    }

    pub(crate) fn refusal(
        &self,
        code: &'static str,
        reason: &'static str,
        quote: Option<&Quote>,
        account_revision: u64,
        now: DateTime<Utc>,
        monotonic_now: Instant,
    ) -> GuardRefusal {
        GuardRefusal {
            code,
            reason,
            evidence: self.snapshot(quote, account_revision, now, monotonic_now),
        }
    }

    pub fn validate(
        &self,
        quote: Option<&Quote>,
        account_revision: u64,
        now: DateTime<Utc>,
        monotonic_now: Instant,
    ) -> Result<(), GuardRefusal> {
        let fail =
            |code, reason| self.refusal(code, reason, quote, account_revision, now, monotonic_now);
        if monotonic_now >= self.expires {
            return Err(fail("ENTRY_EXPIRED", "entry dispatch deadline elapsed"));
        }
        if account_revision != self.account_revision {
            return Err(fail(
                "ENTRY_ACCOUNT_CHANGED",
                "account execution changed during admission",
            ));
        }
        let current = |value: Option<(f64, DateTime<Utc>)>| {
            value
                .filter(|(p, at)| {
                    p.is_finite()
                        && *p > 0.0
                        && *at <= now
                        && now
                            .signed_duration_since(*at)
                            .to_std()
                            .is_ok_and(|age| age <= MAX_QUOTE_AGE)
                })
                .map(|(p, _)| p)
        };
        let (Some(bid), Some(ask)) = (
            quote.and_then(|q| current(q.bid)),
            quote.and_then(|q| current(q.ask)),
        ) else {
            return Err(fail(
                "ENTRY_QUOTE_UNKNOWN",
                "fresh two-sided quote unavailable",
            ));
        };
        if bid > ask {
            return Err(fail("ENTRY_QUOTE_UNKNOWN", "quote is crossed"));
        }
        let side = match &self.signal {
            Signal::Trend { level } if level.is_finite() && *level > 0.0 => match self.side {
                Side::Buy => (ask >= *level).then_some(Side::Buy),
                Side::Sell => (bid <= *level).then_some(Side::Sell),
            },
            Signal::Trend { .. } => None,
            Signal::Mark { band } => {
                band::mark_decision(band, Some(bid), Some(ask)).map(|d| match d {
                    Dir::Buy => Side::Buy,
                    Dir::Sell => Side::Sell,
                })
            }
        };
        if side != Some(self.side) {
            return Err(fail(
                "ENTRY_SIGNAL_CHANGED",
                "original selected side is no longer supported",
            ));
        }
        if let Some(c) = &self.cost {
            let price = if self.side == Side::Buy { ask } else { bid };
            let cost = cost_reserve(
                c.prepared_spread,
                c.commission_round_trip,
                c.slippage_bps_per_side,
                price,
            )
            .map_err(|_| fail("ENTRY_COST_UNKNOWN", "registered cost inputs invalid"))?;
            if !c.original_r.is_finite() || c.original_r <= 0.0 || !cost.total.is_finite() {
                return Err(fail("ENTRY_COST_UNKNOWN", "registered risk/cost invalid"));
            }
            if cost.total >= c.original_r {
                return Err(fail(
                    "ENTRY_COST_GE_R",
                    "registered cost reserve is at least original R",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> EntryGuard {
        EntryGuard {
            connection_generation: 1,
            account_id: 7,
            symbol_id: 9,
            side: Side::Buy,
            signal: Signal::Trend { level: 100.0 },
            cost: None,
            expires: Instant::now() + Duration::from_secs(5),
            account_revision: 0,
        }
    }

    #[test]
    fn rejects_stale_crossed_changed_and_expired_but_preserves_valid_side() {
        let now = Utc::now();
        let good = Quote {
            bid: Some((101.0, now)),
            ask: Some((102.0, now)),
        };
        let g = guard();
        assert!(g.validate(Some(&good), 0, now, Instant::now()).is_ok());
        assert_eq!(
            g.validate(Some(&good), 1, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_ACCOUNT_CHANGED"
        );
        assert_eq!(
            g.validate(Some(&good), 0, now, g.expires).unwrap_err().code,
            "ENTRY_EXPIRED"
        );
        assert_eq!(
            g.validate(
                Some(&good),
                0,
                now + chrono::Duration::seconds(7),
                Instant::now()
            )
            .unwrap_err()
            .code,
            "ENTRY_QUOTE_UNKNOWN"
        );
        let crossed = Quote {
            bid: Some((103.0, now)),
            ..good.clone()
        };
        assert_eq!(
            g.validate(Some(&crossed), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_QUOTE_UNKNOWN"
        );
        let wrong = Quote {
            bid: Some((98.0, now)),
            ask: Some((99.0, now)),
        };
        assert_eq!(
            g.validate(Some(&wrong), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_SIGNAL_CHANGED"
        );
    }

    #[test]
    fn mark_uses_midpoint_and_prepared_cost_without_turning_expiry_into_cost_veto() {
        let now = Utc::now();
        let mut g = guard();
        g.signal = Signal::Mark {
            band: band::Band {
                open: 100.0,
                sigma: 0.02,
                lower: 98.0,
                upper: 102.0,
                half_width: 2.0,
                minutes: 30.0,
            },
        };
        let quote = Quote {
            bid: Some((101.0, now)),
            ask: Some((103.0, now)),
        };
        assert_eq!(
            g.validate(Some(&quote), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_SIGNAL_CHANGED"
        );
        let quote = Quote {
            bid: Some((103.0, now)),
            ask: Some((104.0, now)),
        };
        g.cost = Some(Cost {
            prepared_spread: 0.1,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 0.0,
            original_r: 0.5,
        });
        assert!(
            g.validate(Some(&quote), 0, now, Instant::now()).is_ok(),
            "current spread 1 is not the prepared spread 0.1"
        );
        g.cost.as_mut().unwrap().original_r = 0.1;
        assert_eq!(
            g.validate(Some(&quote), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_COST_GE_R"
        );
        assert_eq!(
            g.validate(Some(&quote), 0, now, g.expires)
                .unwrap_err()
                .code,
            "ENTRY_EXPIRED"
        );
    }
}
