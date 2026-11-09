//! The order gate: the only way a trading request reaches the socket
//! (owner, 25-10-2026: "I want live to execute trading by 1 PM today", with
//! the edge gate kept for live). There is no on/off switch (owner, 24-10:
//! "why do i need JIAOYI_TRADING_ENABLED, removed it").
//!
//! [`send`] runs every check before anything is written, in this order:
//! 1. the account is re-checked against the allowlist;
//! 2. the account's edge gate is met (live waits for a held-out PASS; demo
//!    does not);
//! 3. per command:
//!    - open: the broker's account type and open positions are read fresh
//!      (`ProtoOATraderReq`, `ProtoOAReconcileReq`). On a NETTED account a
//!      new order on a symbol with ANY open position would net against it,
//!      so the entry is refused; on a HEDGED account an entry is refused on a
//!      symbol with any position at all (an external one is never
//!      interacted with; one of ours means one position per symbol); an
//!      unknown or spread-betting account type refuses every entry (PR #13
//!      finding, 25-10-2026: 46130058 and 47790949 may still hold bot-trade's
//!      positions);
//!    - open: a stop is given (relative, > 0, on the symbol's digits); a
//!      take-profit, when given, is relative, > 0 and on the digits; the
//!      volume is on the step, at or above the broker minimum and at or below
//!      its maximum (never rounded up here or anywhere); the quote→deposit
//!      rate is known; the risk recomputed from the exact volume, stop and
//!      rate is within the policy's maximum;
//!    - amend: the position is one this program opened; the new stop moves
//!      only in the trade's favour, from the trader's stop and from the
//!      broker's own, read fresh (`ProtoOAReconcileReq`); the position's
//!      take-profit is carried, because cTrader's amend REPLACES the
//!      protection and a stop-only amend would delete it;
//!    - close: the position is one this program opened;
//! 4. the intent is persisted (a unique entry key: an intent already recorded
//!    is never sent again, after a restart included). A failing ledger means
//!    no frame;
//! 5. the frame is written through the crate-private trading path, and every
//!    execution message carrying its id is recorded until a terminal one or
//!    the wait ends (then `Unconfirmed`: the trader's reconcile decides, the
//!    order is never resent).

use std::future::Future;
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::Allowlist;
use prost::Message;
use serde::Serialize;

use crate::client::{AccountSession, ClientError, ReceivedExecution};
use crate::entry_guard::EntryGuard;
use crate::pb::{self, ProtoOaPayloadType as Oa, ProtoPayloadType as Common};

/// A trade's direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

/// Authority over the residual of one positively evidenced opening. This is
/// supplied by the durable ledger, never inferred from a current broker label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManagementAuthority {
    pub entry_key: String,
    pub symbol_id: i64,
    pub side: Side,
    pub remaining_volume: i64,
}

impl ManagementAuthority {
    pub fn validate_current(&self, p: &pb::ProtoOaPosition) -> Result<(), String> {
        // №18,403 LAW 1: historical fills never override current label ownership.
        if self.entry_key.starts_with("s3:") && p.trade_data.label.as_deref() != Some("jiaoyi-s3") {
            return Err("Strategy 3 label ownership changed: no amend or close".into());
        }
        if p.position_status != pb::ProtoOaPositionStatus::PositionStatusOpen as i32 {
            return Err("authority incomplete: broker position is not open".into());
        }
        if p.trade_data.symbol_id != self.symbol_id
            || Side::from_proto(p.trade_data.trade_side) != Some(self.side)
        {
            return Err(
                "authority mixed: broker symbol/direction differs from the owned opening".into(),
            );
        }
        if self.remaining_volume <= 0 || p.trade_data.volume <= 0 {
            return Err("authority incomplete: no positive owned residual".into());
        }
        // Compare with reconstructed actual fills/exits, never with requested
        // order volume or a stale cached position quantity. A discrepancy can
        // be a late legitimate partial; it must be recovered, not called foreign.
        if p.trade_data.volume != self.remaining_volume {
            return Err(format!(
                "authority incomplete: broker quantity {} differs from evidenced residual {}",
                p.trade_data.volume, self.remaining_volume
            ));
        }
        Ok(())
    }
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        }
    }

    fn proto(self) -> i32 {
        match self {
            Side::Buy => pb::ProtoOaTradeSide::Buy as i32,
            Side::Sell => pb::ProtoOaTradeSide::Sell as i32,
        }
    }

    /// From `ProtoOATradeSide` (1 buy, 2 sell).
    pub fn from_proto(v: i32) -> Option<Side> {
        match v {
            1 => Some(Side::Buy),
            2 => Some(Side::Sell),
            _ => None,
        }
    }
}

/// The broker's volume rules for one symbol, in cents of units, and its digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VolumeLimits {
    pub min: i64,
    pub step: i64,
    pub max: Option<i64>,
    pub digits: i32,
}

/// What the trader asks the gate to send.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// A MARKET order with a stop `relative_stop` (1/100000 of a price) from
    /// the fill, and a take-profit `relative_take_profit` from the fill when
    /// given (Strategy 3's broker target; Strategy 1 has none).
    Open {
        symbol_id: i64,
        side: Side,
        volume: i64,
        relative_stop: i64,
        relative_take_profit: Option<i64>,
        limits: VolumeLimits,
        /// Deposit currency per one unit of the symbol's quote currency, from
        /// the broker's own quotes; `None` refuses.
        quote_to_deposit: Option<f64>,
        label: String,
    },
    /// Move an owned position's stop to `new_stop`.
    AmendStop {
        position_id: i64,
        side: Side,
        /// The stop the broker holds now; `None`: the position has none.
        current_stop: Option<f64>,
        new_stop: f64,
    },
    /// Close an owned position.
    Close { position_id: i64, volume: i64 },
}

impl Command {
    fn kind(&self) -> &'static str {
        match self {
            Command::Open { .. } => "open",
            Command::AmendStop { .. } => "amend_stop",
            Command::Close { .. } => "close",
        }
    }
}

/// The account's trading policy at the moment of the send, read fresh from
/// the settings by the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// The edge gate: `Ok` when met or not required, `Err(why)` otherwise.
    pub edge: Result<(), String>,
    /// The most an open may risk, in the deposit currency.
    pub max_risk: f64,
}

/// The intent written before the frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Intent {
    pub entry_key: String,
    pub client_msg_id: String,
    pub account_id: i64,
    pub kind: &'static str,
    pub symbol_id: Option<i64>,
    pub position_id: Option<i64>,
    pub side: Option<Side>,
    pub volume: Option<i64>,
    pub relative_stop: Option<i64>,
    pub stop_price: Option<f64>,
    /// An amend's take-profit: the position's own, read fresh, carried so the
    /// amend (which replaces the protection) does not delete it.
    pub take_profit: Option<f64>,
    /// The risk recomputed by the gate, in the deposit currency.
    pub risk: Option<f64>,
    pub max_risk: f64,
}

/// Whether the intent was new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentWrite {
    Recorded,
    /// The entry key is already recorded: the order is never sent again.
    AlreadyRecorded,
}

/// One execution message as recorded.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EventRecord {
    pub account_id: i64,
    pub client_msg_id: Option<String>,
    pub entry_key: Option<String>,
    pub payload_type: u32,
    /// `ORDER_FILLED`, `ORDER_REJECTED`, …, `ORDER_ERROR`, `ERROR_RES`.
    pub kind: String,
    pub position_id: Option<i64>,
    pub order_id: Option<i64>,
    pub deal_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub side: Option<Side>,
    pub price: Option<f64>,
    pub volume: Option<i64>,
    pub stop_loss: Option<f64>,
    pub label: Option<String>,
    pub error_code: Option<String>,
    pub description: Option<String>,
    pub is_server_event: Option<bool>,
    #[serde(skip)]
    pub raw: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

/// Where intents and events are kept (Postgres in the service).
pub trait Ledger {
    /// A shared logical account namespace (production uses the database).
    fn admission_namespace(&self) -> String;
    fn margin_floor_pct(&self, account_id: i64)
        -> impl Future<Output = Result<f64, String>> + Send;
    fn margin_has_unresolved_open(
        &self,
        _account_id: i64,
    ) -> impl Future<Output = Result<bool, String>> + Send {
        async { Ok(false) }
    }
    fn record_margin(
        &self,
        _account_id: i64,
        _symbol_id: i64,
        _receipt: &crate::margin::Receipt,
    ) -> impl Future<Output = Result<(), String>> + Send {
        async { Ok(()) }
    }
    fn record_intent(&self, i: &Intent)
        -> impl Future<Output = Result<IntentWrite, String>> + Send;
    fn record_event(&self, e: &EventRecord) -> impl Future<Output = Result<(), String>> + Send;
    /// Whether this program opened the position (an intent of its own filled it).
    fn owns_position(
        &self,
        account_id: i64,
        position_id: i64,
    ) -> impl Future<Output = Result<bool, String>> + Send;
    /// Current lifecycle authority. Historical ownership alone is insufficient.
    fn management_authority(
        &self,
        _account_id: i64,
        _position_id: i64,
    ) -> impl Future<Output = Result<ManagementAuthority, String>> + Send {
        async { Err("authority incomplete: current lifecycle evidence unavailable".into()) }
    }
    fn record_authority_absence(
        &self,
        _account_id: i64,
        _position_id: i64,
    ) -> impl Future<Output = Result<(), String>> + Send {
        async { Err("authority absence persistence unavailable".into()) }
    }
}

/// A refusal: nothing was written to the socket.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{reason}")]
pub struct Refused {
    pub reason: String,
}

fn refuse<T>(reason: impl Into<String>) -> Result<T, Refused> {
    Err(Refused {
        reason: reason.into(),
    })
}

/// How the broker answered.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Outcome {
    Filled {
        position_id: Option<i64>,
        price: Option<f64>,
        volume: Option<i64>,
    },
    /// An amend the broker answered with an execution event.
    Amended { kind: String },
    Rejected {
        code: Option<String>,
        description: Option<String>,
    },
    /// No terminal answer within the wait. Never resent: reconcile decides.
    Unconfirmed,
}

/// A sent command: its id, the broker's answer and every message recorded.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sent {
    pub client_msg_id: String,
    pub outcome: Outcome,
    pub events: Vec<EventRecord>,
}

/// Why `send` did not complete after the intent was recorded.
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error(transparent)]
    Refused(#[from] Refused),
    #[error("the frame could not be written: {0}")]
    Write(#[from] ClientError),
    #[error(
        "execution evidence could not be persisted after send: {0}; outcome remains unresolved"
    )]
    Evidence(String),
    #[error("management continuity could not be persisted: {0}")]
    Authority(String),
}

/// Decode an execution message into its record. `None` for a message that
/// does not decode.
pub fn decode_execution(account_id: i64, m: &ReceivedExecution) -> Option<EventRecord> {
    let mut e = EventRecord {
        account_id,
        client_msg_id: m.client_msg_id.clone(),
        entry_key: None,
        payload_type: m.payload_type,
        kind: String::new(),
        position_id: None,
        order_id: None,
        deal_id: None,
        symbol_id: None,
        side: None,
        price: None,
        volume: None,
        stop_loss: None,
        label: None,
        error_code: None,
        description: None,
        is_server_event: None,
        raw: m.payload.clone(),
        received_at: m.received_at,
    };
    let pt = m.payload_type as i32;
    if pt == Oa::ProtoOaExecutionEvent as i32 {
        let ev = pb::ProtoOaExecutionEvent::decode(m.payload.as_slice()).ok()?;
        if ev.ctid_trader_account_id != account_id {
            return None;
        }
        e.kind = pb::ProtoOaExecutionType::try_from(ev.execution_type)
            .map(|t| t.as_str_name().to_string())
            .unwrap_or_else(|_| format!("UNKNOWN({})", ev.execution_type));
        e.error_code = ev.error_code;
        e.is_server_event = ev.is_server_event;
        if let Some(p) = &ev.position {
            e.position_id = Some(p.position_id);
            e.symbol_id = Some(p.trade_data.symbol_id);
            e.side = Side::from_proto(p.trade_data.trade_side);
            e.price = p.price;
            e.stop_loss = p.stop_loss;
            e.label = p.trade_data.label.clone();
        }
        if let Some(o) = &ev.order {
            e.order_id = Some(o.order_id);
            if e.label.is_none() {
                e.label = o.trade_data.label.clone();
            }
        }
        if let Some(d) = &ev.deal {
            e.deal_id = Some(d.deal_id);
            e.position_id = Some(d.position_id);
            e.symbol_id = Some(d.symbol_id);
            e.side = Side::from_proto(d.trade_side);
            e.volume = Some(d.filled_volume);
            if d.execution_price.is_some() {
                e.price = d.execution_price;
            }
        }
    } else if pt == Oa::ProtoOaOrderErrorEvent as i32 {
        let ev = pb::ProtoOaOrderErrorEvent::decode(m.payload.as_slice()).ok()?;
        if ev.ctid_trader_account_id != account_id {
            return None;
        }
        e.kind = "ORDER_ERROR".into();
        e.error_code = Some(ev.error_code);
        e.description = ev.description;
        e.order_id = ev.order_id;
        e.position_id = ev.position_id;
    } else if pt == Oa::ProtoOaErrorRes as i32 {
        let ev = pb::ProtoOaErrorRes::decode(m.payload.as_slice()).ok()?;
        if ev.ctid_trader_account_id.is_some_and(|a| a != account_id) {
            return None;
        }
        e.kind = "ERROR_RES".into();
        e.error_code = Some(ev.error_code);
        e.description = ev.description;
    } else if pt == Common::ErrorRes as i32 {
        let ev = pb::ProtoErrorRes::decode(m.payload.as_slice()).ok()?;
        e.kind = "ERROR_RES".into();
        e.error_code = Some(ev.error_code);
        e.description = ev.description;
    } else {
        return None;
    }
    Some(e)
}

/// A closing deal in an execution event: what a broker-executed exit needs
/// for its cost record. The position's own side and levels, when the event
/// carries its position.
// Claude · № 17,020 25-Oct (Codex P2 on jiaoyi #15): broker exits are costs too.
#[derive(Debug, Clone, PartialEq)]
pub struct ClosingDeal {
    pub deal_id: i64,
    pub order_id: i64,
    pub position_id: i64,
    pub symbol_id: i64,
    /// The deal's side (a long's exit sells).
    pub deal_side: Option<Side>,
    pub position_side: Option<Side>,
    pub price: Option<f64>,
    pub volume: i64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    /// The order was the position's stop-loss/take-profit order.
    pub sl_tp_order: bool,
    pub stop_out: bool,
}

/// The closing deal of a fill event on `account_id`; `None` for anything
/// else (an opening deal, no deal, another account, another message).
pub fn closing_deal(account_id: i64, m: &ReceivedExecution) -> Option<ClosingDeal> {
    if m.payload_type as i32 != Oa::ProtoOaExecutionEvent as i32 {
        return None;
    }
    let ev = pb::ProtoOaExecutionEvent::decode(m.payload.as_slice()).ok()?;
    if ev.ctid_trader_account_id != account_id {
        return None;
    }
    let fill = ev.execution_type == pb::ProtoOaExecutionType::OrderFilled as i32
        || ev.execution_type == pb::ProtoOaExecutionType::OrderPartialFill as i32;
    let d = ev.deal.as_ref()?;
    if !fill || d.close_position_detail.is_none() {
        return None;
    }
    let p = ev.position.as_ref();
    let o = ev.order.as_ref();
    Some(ClosingDeal {
        deal_id: d.deal_id,
        order_id: d.order_id,
        position_id: d.position_id,
        symbol_id: d.symbol_id,
        deal_side: Side::from_proto(d.trade_side),
        position_side: p.and_then(|p| Side::from_proto(p.trade_data.trade_side)),
        price: d.execution_price,
        volume: d.filled_volume,
        stop_loss: p.and_then(|p| p.stop_loss),
        take_profit: p.and_then(|p| p.take_profit),
        sl_tp_order: o
            .is_some_and(|o| o.order_type == pb::ProtoOaOrderType::StopLossTakeProfit as i32),
        stop_out: o.and_then(|o| o.is_stop_out).unwrap_or(false),
    })
}

/// The risk of `volume` cents with a stop `relative_stop` (1/100000 of a
/// price) away, in the deposit currency.
pub fn risk_of(volume: i64, relative_stop: i64, quote_to_deposit: f64) -> f64 {
    volume as f64 / 100.0 * relative_stop as f64 / 100_000.0 * quote_to_deposit
}

/// `ProtoOAAccountType`: HEDGED 0, NETTED 1, SPREAD_BETTING 2.
pub const HEDGED: i32 = 0;
pub const NETTED: i32 = 1;

/// The labels of this program's strategies: Strategy 1 and Strategy 3.
/// Owner, 25-10-2026 (Strategy 3 v2): on a HEDGED account each strategy may
/// hold the same symbol, managing only its own label; a position with any
/// other label is external.
pub const PROGRAM_LABELS: &[&str] = &["jiaoyi-s1", "jiaoyi-s3"];

/// Whether an entry on `symbol_id` may be sent, given the broker's account
/// type and the account's open positions as (symbol id, label).
pub fn entry_allowed(
    account_type: Option<i32>,
    positions: &[(i64, Option<String>)],
    symbol_id: i64,
    label: &str,
) -> Result<(), String> {
    let on_symbol: Vec<&Option<String>> = positions
        .iter()
        .filter(|(s, _)| *s == symbol_id)
        .map(|(_, l)| l)
        .collect();
    // Claude · № 17,020 25-Oct (ordered: owner's Strategy 3 v2 approval; co-holding)
    let external = on_symbol
        .iter()
        .filter(|l| {
            !l.as_deref().is_some_and(|l| {
                l == label || (label != "jiaoyi-s3" && PROGRAM_LABELS.contains(&l))
            })
        })
        .count();
    let ours = on_symbol
        .iter()
        .filter(|l| l.as_deref() == Some(label))
        .count();
    match account_type {
        Some(NETTED) if !on_symbol.is_empty() => Err(format!(
            "NETTED account: symbol {symbol_id} has {} open position(s) ({external} external); a new order would net against them: refused",
            on_symbol.len()
        )),
        Some(NETTED) => Ok(()),
        Some(HEDGED) if external > 0 => Err(format!(
            "symbol {symbol_id} has {external} external position(s): never interacted with, no entry"
        )),
        Some(HEDGED) if ours > 0 => Err(format!(
            "symbol {symbol_id} already has {ours} position(s) of this strategy ({label}): one per symbol"
        )),
        Some(HEDGED) => Ok(()),
        Some(t) => Err(format!(
            "account type {t} (not HEDGED or NETTED): entries refused"
        )),
        None => Err("the broker gave no account type: entries refused".into()),
    }
}

/// A command's checks, before any write. Returns the intent fields it fixes.
async fn check<L: Ledger>(
    account_id: i64,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    cmd: &Command,
) -> Result<(Option<f64>, Option<f64>), Refused> {
    if let Err(r) = allowlist.permit(account_id) {
        return refuse(format!("allowlist: {}", r.reason));
    }
    if let Err(why) = &policy.edge {
        return refuse(format!("edge gate not met: {why}"));
    }
    match cmd {
        Command::Open {
            volume,
            relative_stop,
            relative_take_profit,
            limits,
            quote_to_deposit,
            ..
        } => {
            if *relative_stop <= 0 {
                return refuse("no stop: an order is never sent without one");
            }
            let tick = 10_i64.pow((5 - limits.digits.clamp(0, 5)) as u32);
            if relative_stop % tick != 0 {
                return refuse(format!(
                    "the stop distance {relative_stop} is not on the symbol's {} digits",
                    limits.digits
                ));
            }
            if let Some(tp) = relative_take_profit {
                if *tp <= 0 || tp % tick != 0 {
                    return refuse(format!(
                        "the take-profit distance {tp} is not positive on the symbol's {} digits",
                        limits.digits
                    ));
                }
            }
            if limits.step <= 0 || limits.min <= 0 {
                return refuse("the symbol's volume rules are unknown");
            }
            if volume % limits.step != 0 {
                return refuse(format!(
                    "volume {volume} is not a multiple of the step {}",
                    limits.step
                ));
            }
            if *volume < limits.min {
                return refuse(format!(
                    "volume {volume} is below the broker minimum {}: skipped, never rounded up",
                    limits.min
                ));
            }
            if let Some(max) = limits.max {
                if *volume > max {
                    return refuse(format!("volume {volume} is above the broker maximum {max}"));
                }
            }
            let Some(rate) = quote_to_deposit.filter(|r| r.is_finite() && *r > 0.0) else {
                return refuse("no broker quote→deposit rate: the risk cannot be computed");
            };
            let risk = risk_of(*volume, *relative_stop, rate);
            if !(policy.max_risk.is_finite() && policy.max_risk > 0.0) {
                return refuse("no risk allowance (fresh equity unread)");
            }
            if risk > policy.max_risk * (1.0 + 1e-9) {
                return refuse(format!(
                    "risk {risk:.4} is above the allowed {:.4} (deposit currency)",
                    policy.max_risk
                ));
            }
            Ok((Some(risk), None))
        }
        Command::AmendStop {
            position_id,
            side,
            current_stop,
            new_stop,
        } => {
            match ledger.owns_position(account_id, *position_id).await {
                Ok(true) => {}
                Ok(false) => {
                    return refuse(format!(
                        "position {position_id} was not opened by this program: never amended"
                    ))
                }
                Err(e) => return refuse(format!("ownership unreadable: {e}")),
            }
            if !new_stop.is_finite() || *new_stop <= 0.0 {
                return refuse("the new stop is not a price");
            }
            if let Some(cur) = current_stop {
                let favourable = match side {
                    Side::Buy => *new_stop > *cur,
                    Side::Sell => *new_stop < *cur,
                };
                if !favourable {
                    return refuse(format!(
                        "stop {new_stop} does not move in the trade's favour from {cur}: refused"
                    ));
                }
            }
            Ok((None, Some(*new_stop)))
        }
        Command::Close {
            position_id,
            volume,
        } => {
            match ledger.owns_position(account_id, *position_id).await {
                Ok(true) => {}
                Ok(false) => {
                    return refuse(format!(
                        "position {position_id} was not opened by this program: never closed"
                    ))
                }
                Err(e) => return refuse(format!("ownership unreadable: {e}")),
            }
            if *volume <= 0 {
                return refuse("close volume must be positive");
            }
            Ok((None, None))
        }
    }
}

/// The frame for a command. `take_profit`: an amend's, carried from the position.
fn frame(
    account_id: i64,
    cmd: &Command,
    entry_key: &str,
    take_profit: Option<f64>,
) -> (u32, Vec<u8>) {
    match cmd {
        Command::Open {
            symbol_id,
            side,
            volume,
            relative_stop,
            relative_take_profit,
            label,
            ..
        } => (
            Oa::ProtoOaNewOrderReq as u32,
            pb::ProtoOaNewOrderReq {
                payload_type: None,
                ctid_trader_account_id: account_id,
                symbol_id: *symbol_id,
                order_type: pb::ProtoOaOrderType::Market as i32,
                trade_side: side.proto(),
                volume: *volume,
                relative_stop_loss: Some(*relative_stop),
                relative_take_profit: *relative_take_profit,
                label: Some(label.clone()),
                // The entry key, so a deal can be traced back to its intent.
                client_order_id: Some(entry_key.chars().take(50).collect()),
                ..Default::default()
            }
            .encode_to_vec(),
        ),
        Command::AmendStop {
            position_id,
            new_stop,
            ..
        } => (
            Oa::ProtoOaAmendPositionSltpReq as u32,
            pb::ProtoOaAmendPositionSltpReq {
                payload_type: None,
                ctid_trader_account_id: account_id,
                position_id: *position_id,
                stop_loss: Some(*new_stop),
                // An amend replaces the position's protection: the
                // take-profit it holds is sent again, or it would be deleted.
                take_profit,
                ..Default::default()
            }
            .encode_to_vec(),
        ),
        Command::Close {
            position_id,
            volume,
        } => (
            Oa::ProtoOaClosePositionReq as u32,
            pb::ProtoOaClosePositionReq {
                payload_type: None,
                ctid_trader_account_id: account_id,
                position_id: *position_id,
                volume: *volume,
            }
            .encode_to_vec(),
        ),
    }
}

/// A fresh, command-bound background admission. It grants no send authority by
/// itself: the final account lock, epoch, broker revision and quote guard remain.
#[derive(Debug, Clone)]
pub struct PreparedEntry {
    command: Command,
    policy: Policy,
    account_id: i64,
    namespace: String,
    epoch: u64,
    generation: u64,
    revision: u64,
    captured_at: tokio::time::Instant,
    occupied_symbols: Vec<i64>,
    pub receipt: crate::margin::Receipt,
    pub calendar: crate::market_hours::ContractCalendar,
}

pub const PREPARED_ENTRY_MAX_AGE: Duration = Duration::from_secs(5);
static ENTRY_EPOCHS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<(String, i64), u64>>,
> = std::sync::OnceLock::new();
fn entry_epoch(namespace: &str, account: i64, advance: bool) -> u64 {
    let mut all = ENTRY_EPOCHS.get_or_init(Default::default).lock().unwrap();
    let value = all.entry((namespace.to_owned(), account)).or_default();
    if advance {
        *value = value.wrapping_add(1);
    }
    *value
}
impl PreparedEntry {
    /// Include earlier equity/settings/quote reads in the same maximum age.
    /// This can only shorten the receipt's authority, never extend it.
    pub fn restrict_source_start(&mut self, at: tokio::time::Instant) {
        self.captured_at = self.captured_at.min(at);
    }
    pub fn expires(&self) -> tokio::time::Instant {
        self.captured_at + PREPARED_ENTRY_MAX_AGE
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn validate(
        &self,
        session: &AccountSession<'_>,
        namespace: &str,
        command: &Command,
        policy: &Policy,
    ) -> Result<(), Refused> {
        if self.command != *command
            || self.policy != *policy
            || self.account_id != session.account().id()
            || self.namespace != namespace
        {
            return refuse("prepared admission identity/command/policy changed");
        }
        if tokio::time::Instant::now() >= self.expires() {
            return refuse("prepared admission expired; background refresh required");
        }
        if session.connection().entry_generation() != self.generation
            || session.connection().entry_revision(self.account_id) != self.revision
            || entry_epoch(namespace, self.account_id, false) != self.epoch
        {
            return refuse("prepared admission account changed; background refresh required");
        }
        Ok(())
    }
}

async fn broker_entry_admission<L: Ledger>(
    session: &AccountSession<'_>,
    ledger: &L,
    cmd: &Command,
    persist_receipt: bool,
) -> Result<
    (
        crate::margin::Receipt,
        pb::ProtoOaTrader,
        pb::ProtoOaReconcileRes,
    ),
    Refused,
> {
    let account_id = session.account().id();
    let Command::Open {
        symbol_id, label, ..
    } = cmd
    else {
        return refuse("prepared admission requires opening command");
    };
    let symbol_id = *symbol_id;
    let (trader, rec) = tokio::join!(session.trader(), session.margin_book());
    let trader = trader.map_err(|e| Refused {
        reason: format!("the account type is unreadable ({e}): no entry"),
    })?;
    let rec = rec.map_err(|e| Refused {
        reason: format!("the open positions are unreadable ({e}): no entry"),
    })?;
    let positions: Vec<(i64, Option<String>)> = rec
        .position
        .iter()
        .map(|p| (p.trade_data.symbol_id, p.trade_data.label.clone()))
        .collect();
    // №18,403 LAW 2 includes pending external orders, not only positions.
    if label == "jiaoyi-s3"
        && rec.order.iter().any(|o| {
            o.trade_data.symbol_id == symbol_id
                && o.trade_data.label.as_deref() != Some("jiaoyi-s3")
        })
    {
        return refuse("yield_to_human_position_or_order: external pending order on symbol");
    }
    entry_allowed(trader.account_type, &positions, symbol_id, label)
        .map_err(|reason| Refused { reason })?;
    let margin = async {
        if ledger.margin_has_unresolved_open(account_id).await? {
            return Err("unresolved entry prevents a reliable margin reservation".into());
        }
        let floor = ledger.margin_floor_pct(account_id).await?;
        crate::margin::check(session, &trader, &rec, cmd, floor).await
    };
    let result = tokio::time::timeout(Duration::from_secs(5), margin)
        .await
        .unwrap_or_else(|_| Err("margin read timed out; no entry".into()));
    let receipt = result
        .as_ref()
        .cloned()
        .unwrap_or_else(|e| crate::margin::Receipt::unavailable(e.clone()));
    if persist_receipt {
        ledger
            .record_margin(account_id, symbol_id, &receipt)
            .await
            .map_err(|reason| Refused {
                reason: format!("margin receipt could not be recorded: {reason}"),
            })?;
    }
    let receipt = result.map_err(|reason| Refused {
        reason: format!("margin buffer: {reason}"),
    })?;
    Ok((receipt, trader, rec))
}

/// Read-only work performed before a pullback touch, never an order or durable
/// reservation. The complete account snapshot must remain current through send.
pub async fn prepare_entry<L: Ledger>(
    session: &AccountSession<'_>,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    command: &Command,
    live_minimum: bool,
) -> Result<PreparedEntry, Refused> {
    let captured_at = tokio::time::Instant::now();
    let account_id = session.account().id();
    let namespace = ledger.admission_namespace();
    let generation = session.connection().entry_generation();
    let revision = session.connection().entry_revision(account_id);
    check(account_id, allowlist, ledger, policy, command).await?;
    let epoch = entry_epoch(&namespace, account_id, false);
    let Command::Open {
        symbol_id, side, ..
    } = command
    else {
        return refuse("prepared admission requires opening command");
    };
    let symbol_ids = [*symbol_id];
    let (admission, symbols) = tokio::join!(
        // Unused warm receipts stay in memory. The chosen S3 proof is stored
        // atomically with its intent/order; do not write one row per refresh.
        broker_entry_admission(session, ledger, command, false),
        session.symbols_by_id(&symbol_ids)
    );
    let (mut receipt, _, book) = admission?;
    let symbols = symbols.map_err(|e| Refused {
        reason: format!("broker calendar unavailable: {e}"),
    })?;
    let mut matching = symbols.symbol.iter().filter(|s| s.symbol_id == *symbol_id);
    let symbol = matching.next().ok_or_else(|| Refused {
        reason: "broker calendar symbol missing".into(),
    })?;
    if matching.next().is_some() {
        return refuse("broker calendar symbol ambiguous");
    }
    let calendar =
        crate::market_hours::ContractCalendar::from_symbol(symbol).map_err(|e| Refused {
            reason: format!("broker calendar invalid: {e:?}"),
        })?;
    calendar.validate(Utc::now(), *side).map_err(|e| Refused {
        reason: format!("broker market closed: {e:?}"),
    })?;
    if let Some(proof) = receipt.fx_guard.as_mut() {
        proof.candidate_max_risk = Some(policy.max_risk);
    }
    if live_minimum
        && (book.position.iter().any(|p| {
            p.trade_data
                .label
                .as_deref()
                .is_some_and(|l| PROGRAM_LABELS.contains(&l))
        }) || book.order.iter().any(|p| {
            p.trade_data
                .label
                .as_deref()
                .is_some_and(|l| PROGRAM_LABELS.contains(&l))
        }))
    {
        return refuse(
            "live: one minimum trade at a time, account-wide; existing program exposure",
        );
    }
    let out = PreparedEntry {
        command: command.clone(),
        policy: policy.clone(),
        account_id,
        namespace,
        generation,
        revision,
        epoch,
        captured_at,
        occupied_symbols: book
            .position
            .iter()
            .map(|p| p.trade_data.symbol_id)
            .chain(book.order.iter().map(|o| o.trade_data.symbol_id))
            .collect(),
        receipt,
        calendar,
    };
    // A read-only refresh must never monopolise the actual dispatch lock.
    // Concurrent sends invalidate the epoch; hold the lock only while checking
    // that the collected snapshot still describes the current account.
    let _lock = crate::margin::try_lock(out.namespace.clone(), account_id)
        .map_err(|reason| Refused { reason })?;
    out.validate(session, &out.namespace, command, policy)?;
    Ok(out)
}

/// Consume a background admission; only the durable exactly-once intent remains
/// on the hot path. Quote/cost/FX checks execute again in the actual socket writer.
#[allow(clippy::too_many_arguments)]
pub async fn send_prepared_entry<L: Ledger>(
    session: &AccountSession<'_>,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    cmd: Command,
    entry_key: &str,
    wait: Duration,
    guard: &EntryGuard,
    prepared: &PreparedEntry,
) -> Result<Sent, SendError> {
    match &guard.signal {
        crate::entry_guard::Signal::PullbackBounded {
            margin: Some(proof),
            calendar,
            ..
        }
        | crate::entry_guard::Signal::PullbackConfirmed {
            margin: Some(proof),
            calendar: Some(calendar),
            ..
        } if Some(proof) == prepared.receipt.fx_guard.as_ref()
            && calendar == &prepared.calendar => {}
        _ => {
            return Err(Refused {
                reason: "prepared dispatch requires its bound FX/risk/calendar proof".into(),
            }
            .into())
        }
    }
    if !matches!(&cmd,Command::Open{symbol_id,side,..} if guard.account_id==session.account().id() && guard.symbol_id==*symbol_id && guard.side==*side)
    {
        return Err(Refused {
            reason: "entry guard does not match prepared command".into(),
        }
        .into());
    }
    if guard.expires > prepared.expires()
        || guard.account_revision != prepared.revision
        || guard.connection_generation != prepared.generation
    {
        return Err(Refused {
            reason: "guard does not preserve prepared admission expiry/identity".into(),
        }
        .into());
    }
    send_inner(
        session,
        allowlist,
        ledger,
        policy,
        cmd,
        entry_key,
        wait,
        Some(guard),
        Some(prepared),
    )
    .await
}

/// Run the checks, persist the intent, write the frame and collect the
/// answer. See the module notes for the order.
pub async fn send<L: Ledger>(
    session: &AccountSession<'_>,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    cmd: Command,
    entry_key: &str,
    wait: Duration,
) -> Result<Sent, SendError> {
    send_inner(
        session, allowlist, ledger, policy, cmd, entry_key, wait, None, None,
    )
    .await
}

/// Production entries supply a frozen decision guard. Exits keep using `send`.
#[allow(clippy::too_many_arguments)]
pub async fn send_entry<L: Ledger>(
    session: &AccountSession<'_>,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    cmd: Command,
    entry_key: &str,
    wait: Duration,
    guard: &EntryGuard,
) -> Result<Sent, SendError> {
    match &cmd {
        Command::Open {
            symbol_id, side, ..
        } if guard.account_id == session.account().id()
            && guard.symbol_id == *symbol_id
            && guard.side == *side => {}
        _ => {
            return Err(Refused {
                reason: "entry guard does not match the opening command".into(),
            }
            .into())
        }
    }
    let mut guard = guard.clone();
    if matches!(
        guard.signal,
        crate::entry_guard::Signal::PullbackConfirmed { calendar: None, .. }
    ) {
        // The fallback path has no cached preparation. Obtain the same broker
        // schedule proof; final writer validation still refuses absent evidence.
        let symbols = session
            .symbols_by_id(&[guard.symbol_id])
            .await
            .map_err(|e| Refused {
                reason: format!("broker calendar unavailable: {e}"),
            })?;
        let mut matching = symbols
            .symbol
            .iter()
            .filter(|s| s.symbol_id == guard.symbol_id);
        let symbol = matching.next().ok_or_else(|| Refused {
            reason: "broker calendar symbol missing".into(),
        })?;
        if matching.next().is_some() {
            return Err(Refused {
                reason: "broker calendar symbol ambiguous".into(),
            }
            .into());
        }
        let proof =
            crate::market_hours::ContractCalendar::from_symbol(symbol).map_err(|e| Refused {
                reason: format!("broker calendar invalid: {e:?}"),
            })?;
        if let crate::entry_guard::Signal::PullbackConfirmed { calendar, .. } = &mut guard.signal {
            *calendar = Some(proof);
        }
    }
    send_inner(
        session,
        allowlist,
        ledger,
        policy,
        cmd,
        entry_key,
        wait,
        Some(&guard),
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn send_inner<L: Ledger>(
    session: &AccountSession<'_>,
    allowlist: &Allowlist,
    ledger: &L,
    policy: &Policy,
    cmd: Command,
    entry_key: &str,
    wait: Duration,
    guard: Option<&EntryGuard>,
    prepared: Option<&PreparedEntry>,
) -> Result<Sent, SendError> {
    let account_id = session.account().id();
    let (risk, stop_price) = check(account_id, allowlist, ledger, policy, &cmd).await?;
    let _margin_lock = if matches!(cmd, Command::Open { .. }) {
        Some(
            crate::margin::try_lock(ledger.admission_namespace(), account_id)
                .map_err(|reason| Refused { reason })?,
        )
    } else {
        None
    };
    let mut take_profit = None;
    match &cmd {
        Command::Open { .. } => {
            if let Some(prepared) = prepared {
                prepared.validate(session, &ledger.admission_namespace(), &cmd, policy)?;
                if let Some(guard) = guard {
                    guard
                        .validate_pair_occupancy(&prepared.occupied_symbols)
                        .map_err(|reason| Refused {
                            reason: reason.into(),
                        })?;
                }
            } else {
                let (_, _, book) = broker_entry_admission(session, ledger, &cmd, true).await?;
                if let Some(guard) = guard {
                    let occupied: Vec<_> = book
                        .position
                        .iter()
                        .map(|p| p.trade_data.symbol_id)
                        .chain(book.order.iter().map(|o| o.trade_data.symbol_id))
                        .collect();
                    guard
                        .validate_pair_occupancy(&occupied)
                        .map_err(|reason| Refused {
                            reason: reason.into(),
                        })?;
                }
            }
            // Every attempted send invalidates all older prepared snapshots,
            // across S1/S3 connections, including uncertain socket outcomes.
            entry_epoch(&ledger.admission_namespace(), account_id, true);
        }
        Command::AmendStop {
            position_id,
            side,
            new_stop,
            ..
        } => {
            // The position as the broker holds it now: its stop must be
            // passed in the trade's favour, and its take-profit is carried.
            let authority = ledger
                .management_authority(account_id, *position_id)
                .await
                .map_err(|reason| Refused { reason })?;
            if authority.side != *side {
                return Err(Refused {
                    reason: "authority mixed: command direction differs from owned opening".into(),
                }
                .into());
            }
            let rec = session.reconcile().await.map_err(|e| Refused {
                reason: format!("the open positions are unreadable ({e}): no amend"),
            })?;
            let Some(p) = rec.position.iter().find(|p| p.position_id == *position_id) else {
                ledger
                    .record_authority_absence(account_id, *position_id)
                    .await
                    .map_err(SendError::Authority)?;
                return Err(Refused {
                    reason: format!("position {position_id} is not open at the broker: no amend"),
                }
                .into());
            };
            authority
                .validate_current(p)
                .map_err(|reason| Refused { reason })?;
            // Historical ownership and the caller's cached side do not prove
            // the direction of the position the broker currently holds. Using
            // a stale opposite side would let a supposedly favourable amend
            // widen the actual position's stop. Refuse before recording/sending.
            if Side::from_proto(p.trade_data.trade_side) != Some(*side) {
                return Err(Refused {
                    reason: format!(
                        "position {position_id} broker direction {} differs from command {side:?}: reconcile before amending",
                        p.trade_data.trade_side
                    ),
                }
                .into());
            }
            if let Some(cur) = p.stop_loss {
                let favourable = match side {
                    Side::Buy => *new_stop > cur,
                    Side::Sell => *new_stop < cur,
                };
                if !favourable {
                    return Err(Refused {
                        reason: format!(
                            "stop {new_stop} does not move in the trade's favour from the broker's {cur}: refused"
                        ),
                    }
                    .into());
                }
            }
            take_profit = p.take_profit.filter(|t| t.is_finite() && *t > 0.0);
        }
        Command::Close {
            position_id,
            volume,
        } => {
            let authority = ledger
                .management_authority(account_id, *position_id)
                .await
                .map_err(|reason| Refused { reason })?;
            let rec = session.reconcile().await.map_err(|e| Refused {
                reason: format!("current position unreadable ({e}): no close"),
            })?;
            let Some(p) = rec.position.iter().find(|p| p.position_id == *position_id) else {
                ledger
                    .record_authority_absence(account_id, *position_id)
                    .await
                    .map_err(SendError::Authority)?;
                return Err(Refused {
                    reason: format!("position {position_id} is not open at the broker: no close"),
                }
                .into());
            };
            authority
                .validate_current(p)
                .map_err(|reason| Refused { reason })?;
            if *volume > authority.remaining_volume {
                return Err(Refused {
                    reason: "close exceeds the evidenced owned residual; reconcile and re-plan"
                        .into(),
                }
                .into());
            }
        }
    }
    let conn = session.connection();
    let client_msg_id = conn.next_trading_id();
    let intent = Intent {
        entry_key: entry_key.to_string(),
        client_msg_id: client_msg_id.clone(),
        account_id,
        kind: cmd.kind(),
        symbol_id: match &cmd {
            Command::Open { symbol_id, .. } => Some(*symbol_id),
            _ => None,
        },
        position_id: match &cmd {
            Command::AmendStop { position_id, .. } | Command::Close { position_id, .. } => {
                Some(*position_id)
            }
            Command::Open { .. } => None,
        },
        side: match &cmd {
            Command::Open { side, .. } | Command::AmendStop { side, .. } => Some(*side),
            Command::Close { .. } => None,
        },
        volume: match &cmd {
            Command::Open { volume, .. } | Command::Close { volume, .. } => Some(*volume),
            Command::AmendStop { .. } => None,
        },
        relative_stop: match &cmd {
            Command::Open { relative_stop, .. } => Some(*relative_stop),
            _ => None,
        },
        stop_price,
        take_profit,
        risk,
        max_risk: policy.max_risk,
    };
    match ledger.record_intent(&intent).await {
        Ok(IntentWrite::Recorded) => {}
        Ok(IntentWrite::AlreadyRecorded) => {
            return Err(Refused {
                reason: format!(
                    "entry {entry_key} is already recorded: an intent is never sent twice"
                ),
            }
            .into())
        }
        Err(e) => {
            return Err(Refused {
                reason: format!("the intent could not be recorded ({e}): no frame sent"),
            }
            .into())
        }
    }
    // Subscribe before writing, so no answer is missed.
    let mut rx = conn.executions();
    let (pt, payload) = frame(account_id, &cmd, entry_key, take_profit);
    let written = if guard.is_some() {
        conn.send_guarded_trading_frame(pt, payload, client_msg_id.clone(), guard)
            .await
    } else {
        conn.send_trading_frame(pt, payload, client_msg_id.clone())
            .await
            .map(|_| None)
    };
    if let Err(ClientError::EntryRefused(refusal)) = &written {
        // The transport verified this before arming its write attempt. This
        // event is local, not a broker rejection; it releases only this proven
        // unsent intent. Persistence failure keeps the intent unresolved.
        let event = EventRecord {
            account_id,
            client_msg_id: Some(client_msg_id.clone()),
            entry_key: Some(entry_key.to_string()),
            payload_type: 0,
            kind: "REFUSED_BEFORE_SEND".into(),
            position_id: None,
            order_id: None,
            deal_id: None,
            symbol_id: intent.symbol_id,
            side: intent.side,
            price: None,
            volume: None,
            stop_loss: None,
            label: None,
            error_code: Some(refusal.code.into()),
            description: Some(refusal.reason.into()),
            is_server_event: Some(false),
            raw: serde_json::to_vec(refusal).map_err(|e| SendError::Evidence(e.to_string()))?,
            received_at: Utc::now(),
        };
        ledger
            .record_event(&event)
            .await
            .map_err(SendError::Evidence)?;
        return Err(Refused {
            reason: refusal.to_string(),
        }
        .into());
    }
    if let Some(receipt) = written? {
        // The frame has already been written and the transport lock released.
        // This is local validation evidence, not a fill or a zero-byte refusal.
        // The execution subscription above retains replies during persistence.
        let event = EventRecord {
            account_id,
            client_msg_id: Some(client_msg_id.clone()),
            entry_key: Some(entry_key.to_string()),
            payload_type: 0,
            kind: "ENTRY_VALIDATED_BEFORE_SEND".into(),
            position_id: None,
            order_id: None,
            deal_id: None,
            symbol_id: intent.symbol_id,
            side: intent.side,
            price: None,
            volume: None,
            stop_loss: None,
            label: None,
            error_code: None,
            description: Some(
                "final entry validation passed; this receipt does not establish a fill or edge"
                    .into(),
            ),
            is_server_event: Some(false),
            raw: serde_json::to_vec(&receipt).map_err(|e| SendError::Evidence(e.to_string()))?,
            received_at: Utc::now(),
        };
        ledger
            .record_event(&event)
            .await
            .map_err(SendError::Evidence)?;
    }
    let deadline = tokio::time::Instant::now() + wait;
    let mut events = Vec::new();
    let outcome = loop {
        let msg = match tokio::time::timeout_at(deadline, rx.recv()).await {
            Err(_) => break Outcome::Unconfirmed,
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(_)) => break Outcome::Unconfirmed,
            Ok(Ok(m)) => m,
        };
        if msg.client_msg_id.as_deref() != Some(client_msg_id.as_str()) {
            continue;
        }
        let Some(mut rec) = decode_execution(account_id, &msg) else {
            continue;
        };
        rec.entry_key = Some(entry_key.to_string());
        if let Err(e) = ledger.record_event(&rec).await {
            // The frame has already been written. Never call this a refusal,
            // manufacture a confirmed fill, or let the actor keep managing on
            // a stale residual. Its recorded intent survives for recovery.
            return Err(SendError::Evidence(e));
        }
        events.push(rec.clone());
        match (rec.kind.as_str(), &cmd) {
            (
                "ORDER_ACCEPTED" | "ORDER_PARTIAL_FILL",
                Command::Open { .. } | Command::Close { .. },
            ) => continue,
            ("ORDER_FILLED", _) => {
                break Outcome::Filled {
                    position_id: rec.position_id,
                    price: rec.price,
                    volume: rec.volume,
                }
            }
            (
                "ORDER_REJECTED" | "ORDER_CANCELLED" | "ORDER_EXPIRED" | "ORDER_ERROR"
                | "ERROR_RES",
                _,
            ) => {
                break Outcome::Rejected {
                    code: rec.error_code.clone(),
                    description: rec.description.clone(),
                }
            }
            (kind, Command::AmendStop { .. }) => {
                break Outcome::Amended {
                    kind: kind.to_string(),
                }
            }
            _ => continue,
        }
    };
    Ok(Sent {
        client_msg_id,
        outcome,
        events,
    })
}
