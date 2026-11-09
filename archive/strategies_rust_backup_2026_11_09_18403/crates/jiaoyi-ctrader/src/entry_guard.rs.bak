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
#[allow(clippy::large_enum_variant)] // Keep the bounded proof together on the rare entry path.
pub enum Signal {
    Trend {
        level: f64,
    },
    Mark {
        band: band::Band,
    },
    Pullback {
        origin: f64,
        level: f64,
        planned_entry: f64,
        price_digits: i32,
        peer: Option<(i64, f64)>,
    },
    /// Owner №18,392: a strict touch has occurred, and risk was reserved
    /// beforehand for the worst executable price inside the 10% bounce band.
    PullbackBounded {
        origin: f64,
        level: f64,
        impulse_range: f64,
        admitted_entry: f64,
        price_digits: i32,
        peer: Option<(i64, f64)>,
        trigger_quote: Quote,
        margin: Option<crate::margin::CachedGuard>,
        calendar: crate::market_hours::ContractCalendar,
    },
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
            "broker_calendar_symbol_id": match &self.signal {
                Signal::PullbackBounded { calendar, .. } => Some(calendar.symbol_id),
                _ => None,
            },
            "pullback_admitted_price": match &self.signal {
                Signal::Pullback { planned_entry, price_digits, .. } => Some(serde_json::json!({"price":planned_entry,"digits":price_digits})),
                Signal::PullbackBounded { admitted_entry, price_digits, origin, level, impulse_range, trigger_quote, .. } => Some(serde_json::json!({
                    "price":admitted_entry,"digits":price_digits,"bounded":true,
                    "origin":origin,"level":level,"impulse_range":impulse_range,
                    "bounce_fraction":jiaoyi_core::strategy3::macro_pullback::BOUNCE_FRACTION,
                    "trigger_bid":side(trigger_quote.bid),"trigger_ask":side(trigger_quote.ask)
                })),
                _ => None,
            },
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
        if let Signal::PullbackBounded { calendar, .. } = &self.signal {
            if calendar.symbol_id != self.symbol_id {
                return Err(fail(
                    "ENTRY_BROKER_CALENDAR_MISMATCH",
                    "broker calendar does not belong to this entry symbol",
                ));
            }
            calendar
                .validate(now, self.side)
                .map_err(|error| fail(error.code, error.reason))?;
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
        if let Signal::PullbackBounded {
            origin,
            level,
            trigger_quote,
            ..
        } = &self.signal
        {
            // A relaxed final bound must never manufacture the first retest.
            // Preserve both source-leg times, not just the later callback time.
            let touched_leg = |leg: Option<(f64, DateTime<Utc>)>| {
                leg.filter(|(price, at)| {
                    price.is_finite()
                        && *price > 0.0
                        && *at <= now
                        && now.signed_duration_since(*at).num_milliseconds() <= 5_000
                })
                .map(|(price, _)| price)
            };
            let touched = match (
                touched_leg(trigger_quote.bid),
                touched_leg(trigger_quote.ask),
            ) {
                (Some(tb), Some(ta)) => jiaoyi_core::strategy3::macro_pullback::retest(
                    self.side == Side::Buy,
                    *origin,
                    *level,
                    tb,
                    ta,
                ),
                _ => false,
            };
            if !touched {
                return Err(fail(
                    "ENTRY_TOUCH_UNVERIFIED",
                    "bounded dispatch requires a fresh strict pullback touch",
                ));
            }
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
            Signal::Pullback { origin, level, .. } => {
                jiaoyi_core::strategy3::macro_pullback::retest(
                    self.side == Side::Buy,
                    *origin,
                    *level,
                    bid,
                    ask,
                )
                .then_some(self.side)
            }
            Signal::PullbackBounded {
                origin,
                level,
                impulse_range,
                ..
            } => jiaoyi_core::strategy3::macro_pullback::dispatch_retest(
                self.side == Side::Buy,
                *origin,
                *level,
                *impulse_range,
                bid,
                ask,
            )
            .then_some(self.side),
        };
        if side != Some(self.side) {
            return Err(fail(
                "ENTRY_SIGNAL_CHANGED",
                "original selected side is no longer supported",
            ));
        }
        if let Signal::Pullback {
            planned_entry,
            price_digits,
            ..
        } = &self.signal
        {
            // Structural R and risk sizing used this price before admission.
            // Re-scaling a moved entry here would bypass those account checks.
            let executable = if self.side == Side::Buy { ask } else { bid };
            let planned =
                jiaoyi_core::strategy3::targets::price_ticks(*planned_entry, *price_digits)
                    .filter(|n| *n > 0);
            let current = jiaoyi_core::strategy3::targets::price_ticks(executable, *price_digits)
                .filter(|n| *n > 0);
            if planned.is_none() || current != planned {
                return Err(fail(
                    "ENTRY_PULLBACK_PRICE_CHANGED",
                    "pullback executable price differs from admitted structural geometry",
                ));
            }
        }
        if let Signal::PullbackBounded {
            admitted_entry,
            price_digits,
            ..
        } = &self.signal
        {
            let executable = if self.side == Side::Buy { ask } else { bid };
            let bound =
                jiaoyi_core::strategy3::targets::price_ticks(*admitted_entry, *price_digits)
                    .filter(|n| *n > 0);
            let current = jiaoyi_core::strategy3::targets::price_ticks(executable, *price_digits)
                .filter(|n| *n > 0);
            // Rounding a genuinely off-grid quote down must not admit a fill
            // beyond the funded bound. Only float residue on a native tick is
            // tolerated here.
            let scale = 10_f64.powi(*price_digits);
            let on_grid = (executable * scale - (executable * scale).round()).abs() <= 1e-6
                && (*admitted_entry * scale - (*admitted_entry * scale).round()).abs() <= 1e-6;
            let inside = on_grid
                && matches!((current, bound), (Some(c),Some(b)) if
                (self.side == Side::Buy && c <= b) || (self.side == Side::Sell && c >= b));
            if !inside {
                return Err(fail(
                    "ENTRY_PULLBACK_BOUND_EXCEEDED",
                    "executable price exceeds the funded structural bound",
                ));
            }
        }
        if let Some(c) = &self.cost {
            let price = if self.side == Side::Buy { ask } else { bid };
            let cost = cost_reserve(
                if matches!(
                    self.signal,
                    Signal::Pullback { .. } | Signal::PullbackBounded { .. }
                ) {
                    c.prepared_spread.max(ask - bid)
                } else {
                    c.prepared_spread
                },
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

    /// Fresh peer observations may veto; unavailable/stale peers remain explicitly
    /// optional under the owner's production revision, never synthetic returns.
    pub(crate) fn validate_peer(
        &self,
        peer: Option<&Quote>,
        now: DateTime<Utc>,
    ) -> Result<serde_json::Value, GuardRefusal> {
        let peer_ref = match &self.signal {
            Signal::Pullback { peer, .. } | Signal::PullbackBounded { peer, .. } => peer,
            _ => return Ok(serde_json::json!({"status":"not_available"})),
        };
        let Some((id, origin)) = peer_ref else {
            return Ok(serde_json::json!({"status":"not_available"}));
        };
        let fresh = |q: Option<(f64, DateTime<Utc>)>| {
            q.filter(|(p, t)| {
                p.is_finite()
                    && *p > 0.0
                    && *t <= now
                    && now
                        .signed_duration_since(*t)
                        .to_std()
                        .is_ok_and(|d| d <= MAX_QUOTE_AGE)
            })
        };
        let (Some((bid, bt)), Some((ask, at))) = (
            peer.and_then(|q| fresh(q.bid)),
            peer.and_then(|q| fresh(q.ask)),
        ) else {
            return Ok(serde_json::json!({"status":"unavailable_or_stale","symbol_id":id}));
        };
        let sign = if self.side == Side::Buy { 1.0 } else { -1.0 };
        let detail = serde_json::json!({"symbol_id":id,"origin":origin,"bid":bid,"ask":ask,"bid_at":bt,"ask_at":at,"checked_at":now});
        if !origin.is_finite()
            || *origin <= 0.0
            || bid > ask
            || (bid + (ask - bid) / 2.0 - origin) * sign < 0.0
        {
            return Err(GuardRefusal {
                code: "ENTRY_PEER_CONFLICT",
                reason: "fresh peer opposes the selected impulse",
                evidence: detail,
            });
        }
        Ok(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calendar_symbol() -> crate::messages::ProtoOaSymbol {
        crate::messages::ProtoOaSymbol {
            symbol_id: 9,
            schedule_time_zone: Some("UTC".into()),
            schedule: vec![crate::messages::ProtoOaInterval {
                start_second: 0,
                end_second: 7 * 86400,
            }],
            enable_short_selling: Some(true),
            ..Default::default()
        }
    }

    fn calendar() -> crate::market_hours::ContractCalendar {
        crate::market_hours::ContractCalendar::from_symbol(&calendar_symbol()).unwrap()
    }

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
    #[test]
    fn pullback_cost_guard_reprices_widened_current_spread() {
        let now = Utc::now();
        let mut g = guard();
        g.side = Side::Buy;
        g.signal = Signal::Pullback {
            origin: 100.0,
            level: 100.5,
            planned_entry: 100.51,
            price_digits: 2,
            peer: None,
        };
        g.cost = Some(Cost {
            prepared_spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 3.74,
            original_r: 0.2,
        });
        let narrow = Quote {
            bid: Some((100.49, now)),
            ask: Some((100.51, now)),
        };
        assert!(g.validate(Some(&narrow), 0, now, Instant::now()).is_ok());
        // Same ask and a valid midpoint retest; widening alone breaches C<R.
        let wide = Quote {
            bid: Some((100.3, now)),
            ask: Some((100.51, now)),
        };
        assert_eq!(
            g.validate(Some(&wide), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_COST_GE_R"
        );
    }

    #[test]
    fn pullback_writer_guard_refuses_peak_and_invalidated_origin() {
        let now = Utc::now();
        for side in [Side::Buy, Side::Sell] {
            let sign = if side == Side::Buy { 1.0 } else { -1.0 };
            let mut g = guard();
            g.side = side;
            g.signal = Signal::Pullback {
                origin: 100.0,
                level: 100.0 + sign * 0.5,
                planned_entry: 100.0 + sign * 0.51,
                price_digits: 2,
                peer: None,
            };
            let q = |mid| Quote {
                bid: Some((mid - 0.01, now)),
                ask: Some((mid + 0.01, now)),
            };
            assert_eq!(
                g.validate(Some(&q(100.0 + sign)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_SIGNAL_CHANGED"
            );
            assert!(g
                .validate(Some(&q(100.0 + sign * 0.5)), 0, now, Instant::now())
                .is_ok());
            // The midpoint remains inside the retest, but the stored R/size
            // no longer describes this executable entry. It must be refused.
            assert_eq!(
                g.validate(Some(&q(100.0 + sign * 0.4)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_PULLBACK_PRICE_CHANGED"
            );
            assert_eq!(
                g.validate(Some(&q(100.0)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_SIGNAL_CHANGED"
            );
            assert_eq!(
                g.validate(Some(&q(100.0 - sign * 0.1)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_SIGNAL_CHANGED"
            );
        }
    }
    #[test]
    fn bounded_pullback_requires_strict_touch_and_keeps_expiry_revision_and_origin_guards() {
        let now = Utc::now();
        for side in [Side::Buy, Side::Sell] {
            let sign = if side == Side::Buy { 1.0 } else { -1.0 };
            let q = |mid, at| Quote {
                bid: Some((mid - 0.01, at)),
                ask: Some((mid + 0.01, at)),
            };
            let mut g = guard();
            g.side = side;
            g.signal = Signal::PullbackBounded {
                origin: 100.0,
                level: 100.0 + sign * 0.5,
                impulse_range: 1.0,
                admitted_entry: 100.0 + sign * 0.61,
                price_digits: 2,
                peer: None,
                trigger_quote: q(100.0 + sign * 0.5, now),
                margin: None,
                calendar: calendar(),
            };
            let bounce = q(100.0 + sign * 0.55, now);
            assert!(g.validate(Some(&bounce), 0, now, Instant::now()).is_ok());
            assert_eq!(
                g.validate(Some(&bounce), 1, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_ACCOUNT_CHANGED"
            );
            assert_eq!(
                g.validate(Some(&bounce), 0, now, g.expires)
                    .unwrap_err()
                    .code,
                "ENTRY_EXPIRED"
            );
            assert_eq!(
                g.validate(Some(&q(100.0, now)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_SIGNAL_CHANGED"
            );
            assert_eq!(
                g.validate(Some(&q(100.0 + sign * 0.61, now)), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_SIGNAL_CHANGED"
            );
            if let Signal::PullbackBounded { trigger_quote, .. } = &mut g.signal {
                *trigger_quote = bounce.clone();
            }
            assert_eq!(
                g.validate(Some(&bounce), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_TOUCH_UNVERIFIED"
            );
            if let Signal::PullbackBounded { trigger_quote, .. } = &mut g.signal {
                *trigger_quote = q(
                    100.0 + sign * 0.5,
                    now - chrono::Duration::milliseconds(5001),
                );
            }
            assert_eq!(
                g.validate(Some(&bounce), 0, now, Instant::now())
                    .unwrap_err()
                    .code,
                "ENTRY_TOUCH_UNVERIFIED"
            );
        }
    }

    #[test]
    fn bounded_writer_rechecks_broker_window_at_send_and_binds_calendar_symbol() {
        let now: DateTime<Utc> = "2026-11-09T12:00:00Z".parse().unwrap();
        let quote = Quote {
            bid: Some((100.49, now)),
            ask: Some((100.51, now)),
        };
        let mut contract = calendar_symbol();
        contract.schedule = vec![crate::messages::ProtoOaInterval {
            start_second: 86400 + 12 * 3600,
            end_second: 86400 + 12 * 3600 + 1,
        }];
        let mut g = guard();
        g.signal = Signal::PullbackBounded {
            origin: 100.0,
            level: 100.5,
            impulse_range: 1.0,
            admitted_entry: 100.61,
            price_digits: 2,
            peer: None,
            trigger_quote: quote.clone(),
            margin: None,
            calendar: crate::market_hours::ContractCalendar::from_symbol(&contract).unwrap(),
        };
        assert!(g.validate(Some(&quote), 0, now, Instant::now()).is_ok());
        assert_eq!(
            g.validate(
                Some(&quote),
                0,
                now + chrono::Duration::seconds(2),
                Instant::now(),
            )
            .unwrap_err()
            .code,
            "ENTRY_BROKER_MARKET_CLOSED",
            "a fresh quote and prepared admission cannot survive a broker closure"
        );
        g.symbol_id += 1;
        assert_eq!(
            g.validate(Some(&quote), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_BROKER_CALENDAR_MISMATCH"
        );
    }

    #[test]
    fn bounded_pullback_refuses_unfunded_executable_and_cost_widening() {
        let now = Utc::now();
        let mut g = guard();
        g.signal = Signal::PullbackBounded {
            origin: 100.0,
            level: 100.5,
            impulse_range: 1.0,
            admitted_entry: 100.61,
            price_digits: 2,
            peer: None,
            trigger_quote: Quote {
                bid: Some((100.49, now)),
                ask: Some((100.51, now)),
            },
            margin: None,
            calendar: calendar(),
        };
        let wide = Quote {
            bid: Some((100.4, now)),
            ask: Some((100.7, now)),
        };
        assert_eq!(
            g.validate(Some(&wide), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_PULLBACK_BOUND_EXCEEDED"
        );
        let off_grid = Quote {
            bid: Some((100.49, now)),
            ask: Some((100.614, now)),
        };
        assert_eq!(
            g.validate(Some(&off_grid), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_PULLBACK_BOUND_EXCEEDED"
        );
        let allowed = Quote {
            bid: Some((100.49, now)),
            ask: Some((100.61, now)),
        };
        g.cost = Some(Cost {
            prepared_spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 0.0,
            original_r: 0.1,
        });
        assert_eq!(
            g.validate(Some(&allowed), 0, now, Instant::now())
                .unwrap_err()
                .code,
            "ENTRY_COST_GE_R"
        );
        g.cost.as_mut().unwrap().original_r = 1.0;
        assert!(g.validate(Some(&allowed), 0, now, Instant::now()).is_ok());
    }

    #[test]
    fn peer_guard_vetoes_fresh_opposition_and_labels_missing_or_stale() {
        let now = Utc::now();
        let mut g = guard();
        g.signal = Signal::Pullback {
            origin: 100.0,
            level: 100.5,
            planned_entry: 100.51,
            price_digits: 2,
            peer: Some((22, 200.0)),
        };
        let peer = Quote {
            bid: Some((199.0, now)),
            ask: Some((199.5, now)),
        };
        assert_eq!(
            g.validate_peer(Some(&peer), now).unwrap_err().code,
            "ENTRY_PEER_CONFLICT"
        );
        assert_eq!(
            g.validate_peer(Some(&peer), now + chrono::Duration::seconds(7))
                .unwrap()["status"],
            "unavailable_or_stale"
        );
        assert_eq!(
            g.validate_peer(None, now).unwrap()["status"],
            "unavailable_or_stale"
        );
        let neutral = Quote {
            bid: Some((199.5, now)),
            ask: Some((200.5, now)),
        };
        assert!(g.validate_peer(Some(&neutral), now).is_ok());
        g.signal = Signal::Pullback {
            origin: 100.0,
            level: 100.5,
            planned_entry: 100.51,
            price_digits: 2,
            peer: None,
        };
        assert_eq!(
            g.validate_peer(None, now).unwrap()["status"],
            "not_available"
        );
    }
}
