//! What an order's answers add up to, for every strategy's fill record: the
//! outcome, the fill (the deals' sum at their volume-weighted price, into the
//! deals' position), the answer times, and the slippage against the quote at
//! the decision or the level a broker exit executed. One rule for Strategy 1
//! and Strategy 3.
// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")

use chrono::{DateTime, Utc};
use jiaoyi_ctrader::trading::{self as gate, ClosingDeal, Outcome, Side};

/// Whether part of a sent order filled before its terminal answer.
pub fn partly_filled(sent: &gate::Sent) -> bool {
    sent.events.iter().any(|e| e.kind == "ORDER_PARTIAL_FILL")
}

/// An order's answer, summed up.
#[derive(Debug, Clone, PartialEq)]
pub struct FillSummary {
    /// `filled`, `partly_filled`, `rejected`, `amended` or `unconfirmed`.
    pub outcome: &'static str,
    pub price: Option<f64>,
    pub position_id: Option<i64>,
    pub volume: Option<i64>,
    /// When the fill answer (`ORDER_FILLED`) arrived, if one did.
    pub filled_at: Option<DateTime<Utc>>,
    /// When the last answer arrived, whatever it was.
    pub answered_at: Option<DateTime<Utc>>,
}

/// Sum up a sent order. Its deals each carry the broker's position id,
/// whatever the last answer, and an order filled in several deals is their
/// sum at their volume-weighted price: an event carries one deal, not the
/// position (Codex P1s on #15). `position_id` is the caller's (a close's or
/// an amend's position); `volume` the requested volume.
pub fn summarize(sent: &gate::Sent, position_id: Option<i64>, volume: i64) -> FillSummary {
    let deals: Vec<_> = sent
        .events
        .iter()
        .filter(|e| e.kind == "ORDER_FILLED" || e.kind == "ORDER_PARTIAL_FILL")
        .collect();
    let deal_pid = deals.iter().find_map(|e| e.position_id);
    let deal_volume: i64 = deals.iter().filter_map(|e| e.volume).sum();
    let deal_price = deals
        .iter()
        .map(|e| Some(e.price? * e.volume? as f64))
        .sum::<Option<f64>>()
        .filter(|_| deal_volume > 0)
        .map(|n| n / deal_volume as f64);
    let (outcome, price, pid, vol) = match &sent.outcome {
        Outcome::Filled {
            position_id: p,
            price,
            volume: v,
        } if deal_volume > 0 => (
            "filled",
            deal_price.or(*price),
            deal_pid.or(*p).or(position_id),
            Some(deal_volume.max(v.unwrap_or(0))),
        ),
        Outcome::Filled {
            position_id: p,
            price,
            volume: v,
        } => ("filled", *price, p.or(position_id), v.or(Some(volume))),
        // A rejection, or no last answer, after part of the order filled is
        // a partial fill, into the deals' position.
        Outcome::Rejected { .. } | Outcome::Unconfirmed if partly_filled(sent) => (
            "partly_filled",
            deal_price,
            deal_pid.or(position_id),
            Some(deal_volume),
        ),
        Outcome::Rejected { .. } => ("rejected", None, position_id, None),
        Outcome::Amended { .. } => ("amended", None, position_id, None),
        Outcome::Unconfirmed => ("unconfirmed", None, position_id, None),
    };
    FillSummary {
        outcome,
        price,
        position_id: pid,
        volume: vol,
        filled_at: sent
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "ORDER_FILLED")
            .map(|e| e.received_at),
        answered_at: sent.events.last().map(|e| e.received_at),
    }
}

/// The side an exit of a `position` trades: a long's exit sells, a short's buys.
pub fn exit_side(position: Side) -> Side {
    match position {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    }
}

/// A fill against the price it was meant at, for the side that `traded`:
/// a buy pays more, a sell gets less. Positive is adverse.
pub fn slippage(traded: Side, fill: Option<f64>, meant: Option<f64>) -> Option<f64> {
    let (f, q) = (fill?, meant?);
    Some(match traded {
        Side::Buy => f - q,
        Side::Sell => q - f,
    })
}

/// A broker-executed closing deal: why it closed (`stop_loss`,
/// `take_profit`, `stop_out` or `other`) and the level it executed, the
/// nearer of the stop and the take-profit to the fill.
pub fn exit_level(
    deal: &ClosingDeal,
    stop: Option<f64>,
    target: Option<f64>,
) -> (&'static str, Option<f64>) {
    let level: Option<(&'static str, f64)> =
        match (deal.sl_tp_order && !deal.stop_out, deal.price, stop, target) {
            (true, Some(f), Some(s), Some(t)) if (f - s).abs() <= (f - t).abs() => {
                Some(("stop_loss", s))
            }
            (true, Some(_), _, Some(t)) => Some(("take_profit", t)),
            (true, Some(_), Some(s), None) => Some(("stop_loss", s)),
            _ => None,
        };
    match level {
        Some((why, p)) => (why, Some(p)),
        None if deal.stop_out => ("stop_out", None),
        None => ("other", None),
    }
}
