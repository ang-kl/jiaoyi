//! Current management authority from positively correlated broker evidence.
//! Historical ownership, labels, requested quantity and current direction alone
//! never authorise a changed lifecycle. Financial-history completeness is not
//! a prerequisite when execution receipts already prove the owned residual.

use std::collections::{BTreeMap, HashSet};

use chrono::Utc;
use jiaoyi_ctrader::messages::{
    ProtoOaDeal, ProtoOaExecutionEvent, ProtoOaExecutionType, ProtoOaOrder, ProtoOaOrderDetailsRes,
    ProtoOaOrderListByPositionIdRes, ProtoOaPayloadType, ProtoOaPosition,
};
use jiaoyi_ctrader::trading::{ManagementAuthority, Side};
use jiaoyi_store::{authority as evidence, sqlx::PgPool};
use prost::Message;

pub fn validate_current(a: &ManagementAuthority, p: &ProtoOaPosition) -> Result<(), String> {
    a.validate_current(p)
}

pub async fn position_authority(
    pool: &PgPool,
    account: i64,
    position: i64,
) -> Result<ManagementAuthority, String> {
    let e = evidence::position_evidence(pool, account, position)
        .await
        .map_err(|e| format!("authority incomplete: evidence unreadable ({e})"))?;
    qualify(account, position, &e)
}

fn incomplete(why: &str) -> String {
    format!("authority incomplete: {why}")
}

fn mixed(why: &str) -> String {
    format!("authority mixed: {why}")
}

fn side(s: &str) -> Option<Side> {
    match s {
        "buy" => Some(Side::Buy),
        "sell" => Some(Side::Sell),
        _ => None,
    }
}

/// Deterministic reduction of a bounded evidence set. Deal IDs deduplicate
/// stream/history; RECONCILED snapshots are deliberately not increments.
pub fn qualify(
    account: i64,
    position: i64,
    e: &evidence::Evidence,
) -> Result<ManagementAuthority, String> {
    if e.truncated {
        return Err(incomplete("position evidence exceeds the bounded reader"));
    }
    if e.receipts.iter().any(|r| r.kind == "AUTHORITY_DISAPPEARED") {
        return Err(mixed(
            "the prior position was observed absent; its old opening cannot regain authority",
        ));
    }
    let gap = e
        .receipts
        .iter()
        .filter(|r| r.kind == "AUTHORITY_GAP")
        .map(|r| r.received_at)
        .max();
    let recovered = e
        .receipts
        .iter()
        .filter(|r| r.kind == "AUTHORITY_RECOVERY_COMPLETE")
        .map(|r| r.received_at)
        .max();
    if gap.is_some_and(|g| recovered.is_none_or(|r| r < g)) {
        return Err(incomplete(
            "execution continuity gap requires scoped recovery",
        ));
    }
    if e.openings.len() != 1 {
        return Err(if e.openings.len() > 1 {
            mixed("multiple opening anchors share this position")
        } else {
            incomplete("no correlated opening instruction")
        });
    }
    let anchor = &e.openings[0];
    let original_side = anchor
        .side
        .as_deref()
        .and_then(side)
        .ok_or_else(|| incomplete("opening side absent"))?;
    let symbol = anchor
        .symbol_id
        .ok_or_else(|| incomplete("opening symbol absent"))?;
    let requested = anchor
        .volume
        .filter(|v| *v > 0)
        .ok_or_else(|| incomplete("opening quantity absent"))?;
    let mut owned_orders = HashSet::new();
    let mut order_observations = Vec::new();
    let mut deals = BTreeMap::<i64, ProtoOaDeal>::new();
    for r in &e.receipts {
        let correlated = r.entry_key.as_deref() == Some(anchor.entry_key.as_str())
            || r.client_msg_id.as_deref() == Some(anchor.client_msg_id.as_str());
        if r.payload_type == ProtoOaPayloadType::ProtoOaExecutionEvent as i32 {
            let event = ProtoOaExecutionEvent::decode(r.raw.as_slice())
                .map_err(|_| incomplete("execution receipt cannot be decoded"))?;
            if event.ctid_trader_account_id != account {
                return Err(mixed("receipt account mismatch"));
            }
            if event
                .position
                .as_ref()
                .is_some_and(|p| p.position_id != position)
            {
                return Err(mixed("receipt position mismatch"));
            }
            if let Some(d) = event.deal {
                if !matches!(event.execution_type, x if x == ProtoOaExecutionType::OrderFilled as i32 || x == ProtoOaExecutionType::OrderPartialFill as i32)
                {
                    continue;
                }
                if correlated && d.close_position_detail.is_none() {
                    owned_orders.insert(d.order_id);
                }
                merge_deal(&mut deals, d, position, symbol)?;
            }
        } else if r.payload_type == ProtoOaPayloadType::ProtoOaOrderDetailsRes as i32 {
            let reply = ProtoOaOrderDetailsRes::decode(r.raw.as_slice())
                .map_err(|_| incomplete("recovered order receipt cannot be decoded"))?;
            if reply.ctid_trader_account_id != account || reply.order.position_id != Some(position)
            {
                return Err(mixed("recovered order account/position mismatch"));
            }
            if correlated
                && !reply.order.closing_order.unwrap_or(false)
                && reply.order.trade_data.symbol_id == symbol
                && Side::from_proto(reply.order.trade_data.trade_side) == Some(original_side)
            {
                owned_orders.insert(reply.order.order_id);
            }
            for d in reply.deal {
                merge_deal(&mut deals, d, position, symbol)?;
            }
        } else if r.payload_type == ProtoOaPayloadType::ProtoOaOrderListByPositionIdRes as i32 {
            let reply = ProtoOaOrderListByPositionIdRes::decode(r.raw.as_slice())
                .map_err(|_| incomplete("recovered orders cannot be decoded"))?;
            if reply.ctid_trader_account_id != account || reply.has_more {
                return Err(incomplete("recovered orders are incomplete"));
            }
            for o in reply.order {
                if o.position_id.is_some_and(|id| id != position) {
                    return Err(mixed("recovered order position mismatch"));
                }
                if correlated
                    && !o.closing_order.unwrap_or(false)
                    && o.client_order_id.as_deref()
                        == Some(
                            anchor
                                .entry_key
                                .chars()
                                .take(50)
                                .collect::<String>()
                                .as_str(),
                        )
                    && o.trade_data.symbol_id == symbol
                    && Side::from_proto(o.trade_data.trade_side) == Some(original_side)
                {
                    owned_orders.insert(o.order_id);
                }
                order_observations.push(o);
            }
        }
    }
    for raw in &e.deals {
        let d = ProtoOaDeal::decode(raw.as_slice())
            .map_err(|_| incomplete("history deal cannot be decoded"))?;
        merge_deal(&mut deals, d, position, symbol)?;
    }
    validate_order_coverage(&order_observations, &deals, &owned_orders)?;
    let mut ordered = deals.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|d| (d.execution_timestamp, d.deal_id));
    let mut opened = 0_i64;
    let mut remaining = 0_i64;
    let mut terminal = false;
    for d in ordered {
        if d.filled_volume <= 0 {
            continue;
        }
        if let Some(c) = d.close_position_detail {
            let v = c
                .closed_volume
                .filter(|v| *v > 0 && *v <= d.filled_volume)
                .ok_or_else(|| incomplete("closing volume is absent or inconsistent"))?;
            if v != d.filled_volume {
                return Err(mixed("a reversal deal has an unowned opening remainder"));
            }
            if Side::from_proto(d.trade_side)
                != Some(match original_side {
                    Side::Buy => Side::Sell,
                    Side::Sell => Side::Buy,
                })
            {
                return Err(mixed(
                    "closing deal direction contradicts the owned opening",
                ));
            }
            remaining = remaining
                .checked_sub(v)
                .filter(|v| *v >= 0)
                .ok_or_else(|| incomplete("closing quantity exceeds confirmed owned fills"))?;
            terminal |= remaining == 0;
        } else {
            if terminal {
                return Err(mixed("a new opening follows the original terminal close"));
            }
            if !owned_orders.contains(&d.order_id)
                || Side::from_proto(d.trade_side) != Some(original_side)
            {
                return Err(mixed(
                    "opening increase is not correlated to the owned order",
                ));
            }
            opened = opened
                .checked_add(d.filled_volume)
                .filter(|v| *v <= requested)
                .ok_or_else(|| mixed("confirmed opening exceeds the owned instruction"))?;
            remaining = remaining
                .checked_add(d.filled_volume)
                .ok_or_else(|| incomplete("quantity overflow"))?;
        }
    }
    if opened == 0 {
        return Err(incomplete("no positively correlated filled opening"));
    }
    if remaining == 0 {
        return Err(mixed("the original owned lifecycle is closed"));
    }
    Ok(ManagementAuthority {
        entry_key: anchor.entry_key.clone(),
        symbol_id: symbol,
        side: original_side,
        remaining_volume: remaining,
    })
}

/// An order list may reveal an execution before deal history catches up.
fn validate_order_coverage(
    orders: &[ProtoOaOrder],
    deals: &BTreeMap<i64, ProtoOaDeal>,
    owned: &HashSet<i64>,
) -> Result<(), String> {
    let mut coverage_missing = false;
    for order in orders {
        let matched = deals
            .values()
            .filter(|d| d.order_id == order.order_id)
            .collect::<Vec<_>>();
        let known = matched
            .iter()
            .try_fold(0_i64, |sum, d| sum.checked_add(d.filled_volume))
            .ok_or_else(|| incomplete("order/deal quantity overflow"))?;
        let executed = match order.executed_volume {
            Some(v) if v >= 0 => v,
            Some(_) => return Err(incomplete("negative executed order quantity")),
            None if order.order_status == 2 => order.trade_data.volume,
            None if matches!(order.order_status, 1 | 3) && known == 0 => 0,
            None => return Err(incomplete("executed order quantity is unknown")),
        };
        let closes = order.closing_order == Some(true)
            || (!matched.is_empty() && matched.iter().all(|d| d.close_position_detail.is_some()));
        if executed > 0 && !closes && !owned.contains(&order.order_id) {
            return Err(mixed(
                "broker order list proves an executed foreign opening",
            ));
        }
        if known < executed {
            coverage_missing = true;
        }
    }
    if coverage_missing {
        return Err(incomplete(
            "executed broker order is ahead of its available deal evidence",
        ));
    }
    Ok(())
}

fn merge_deal(
    deals: &mut BTreeMap<i64, ProtoOaDeal>,
    d: ProtoOaDeal,
    position: i64,
    symbol: i64,
) -> Result<(), String> {
    // Only actual filled status is quantity evidence. Failed executions are not fills.
    if !matches!(d.deal_status, 2 | 3) {
        return Ok(());
    }
    if d.filled_volume < 0 {
        return Err(incomplete("negative filled quantity"));
    }
    if d.position_id != position || d.symbol_id != symbol {
        return Err(mixed("deal position/symbol mismatch"));
    }
    if let Some(old) = deals.get(&d.deal_id) {
        if old.order_id != d.order_id
            || old.trade_side != d.trade_side
            || old.filled_volume != d.filled_volume
            || old.close_position_detail.as_ref().map(|c| c.closed_volume)
                != d.close_position_detail.as_ref().map(|c| c.closed_volume)
        {
            return Err(incomplete(
                "stream/history disagree on a deal's ownership quantity",
            ));
        }
    } else {
        deals.insert(d.deal_id, d);
    }
    Ok(())
}

pub async fn mark_gap(
    pool: &PgPool,
    account: i64,
    position: i64,
    reason: &str,
) -> Result<(), String> {
    marker(pool, account, position, "AUTHORITY_GAP", reason, Utc::now()).await
}

/// A full-book absence ends the old management authority, without inventing a
/// cash fill. Unlike a stream gap, lagging historical replies cannot clear it.
pub async fn mark_disappeared(
    pool: &PgPool,
    account: i64,
    position: i64,
    reason: &str,
) -> Result<(), String> {
    jiaoyi_store::sqlx::query(
        "INSERT INTO trading_events(account_id,position_id,payload_type,kind,raw,received_at,description)
         SELECT $1,$2,0,'AUTHORITY_DISAPPEARED',''::bytea,now(),$3
         WHERE NOT EXISTS(SELECT 1 FROM trading_events WHERE account_id=$1 AND position_id=$2 AND kind='AUTHORITY_DISAPPEARED')"
    ).bind(account).bind(position).bind(reason).execute(pool).await
        .map(|_| ()).map_err(|e| incomplete(&format!("absence could not be persisted ({e})")))
}

async fn marker(
    pool: &PgPool,
    account: i64,
    position: i64,
    kind: &str,
    reason: &str,
    at: chrono::DateTime<Utc>,
) -> Result<(), String> {
    jiaoyi_store::trading::record_event(
        pool,
        &jiaoyi_store::trading::EventRow {
            account_id: account,
            entry_key: None,
            client_msg_id: None,
            payload_type: 0,
            kind: kind.into(),
            position_id: Some(position),
            order_id: None,
            deal_id: None,
            symbol_id: None,
            side: None,
            price: None,
            volume: None,
            stop_loss: None,
            label: None,
            error_code: None,
            description: Some(reason.into()),
            is_server_event: None,
            raw: vec![],
            received_at: at,
        },
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("authority marker unreadable: {e}"))
}

/// Read-only, scoped recovery. Actor loops poll this separately from their
/// management owner. No inference from labels, and no resend of the opening.
pub async fn recover_position(
    pool: &PgPool,
    session: &jiaoyi_ctrader::AccountSession<'_>,
    p: &ProtoOaPosition,
) -> Result<(), String> {
    let started = Utc::now();
    let work = async {
        let account = session.account().id();
        let (orders, deals) = session
            .authority_history(p.position_id)
            .await
            .map_err(|e| incomplete(&format!("scoped broker recovery failed ({e})")))?;
        // Retain observed executed-order contradictions even if the deal view
        // is lagging. A failed recovery must not erase this fresher evidence.
        let raw_orders = orders.encode_to_vec();
        jiaoyi_store::sqlx::query(
            "INSERT INTO trading_events(account_id,position_id,payload_type,kind,raw,received_at,description)
             SELECT $1,$2,$3,'AUTHORITY_ORDER_OBSERVATION',$4,$5,'scoped broker order evidence'
             WHERE NOT EXISTS(SELECT 1 FROM trading_events WHERE account_id=$1 AND position_id=$2
                AND payload_type=$3 AND kind='AUTHORITY_ORDER_OBSERVATION' AND raw=$4)"
        ).bind(account).bind(p.position_id).bind(ProtoOaPayloadType::ProtoOaOrderListByPositionIdRes as i32)
            .bind(&raw_orders).bind(started).execute(pool).await.map_err(|e| incomplete(&e.to_string()))?;
        if deals
            .deal
            .iter()
            .any(|d| d.position_id != p.position_id || d.symbol_id != p.trade_data.symbol_id)
        {
            return Err(mixed("recovered deal account-position scope disagrees"));
        }
        let mut unique_deals = BTreeMap::new();
        for deal in &deals.deal {
            merge_deal(
                &mut unique_deals,
                *deal,
                p.position_id,
                p.trade_data.symbol_id,
            )?;
        }
        let mut candidates = Vec::new();
        for o in &orders.order {
            if o.closing_order.unwrap_or(false) {
                continue;
            }
            if o.position_id.is_some_and(|id| id != p.position_id)
                || o.trade_data.symbol_id != p.trade_data.symbol_id
            {
                return Err(mixed("recovered order scope disagrees"));
            }
            let Some(key) = o.client_order_id.as_deref() else {
                continue;
            };
            let matches =
                evidence::openings_for_wire_key(pool, account, p.trade_data.symbol_id, key)
                    .await
                    .map_err(|e| incomplete(&e.to_string()))?;
            if matches.len() > 1 {
                return Err(incomplete("truncated client-order key is ambiguous"));
            }
            if let Some(anchor) = matches.into_iter().next() {
                if anchor.side.as_deref().and_then(side)
                    != Side::from_proto(o.trade_data.trade_side)
                {
                    return Err(mixed("recovered order direction differs from instruction"));
                }
                let filled = unique_deals
                    .values()
                    .filter(|d| d.order_id == o.order_id && d.close_position_detail.is_none())
                    .try_fold(0_i64, |sum, d| sum.checked_add(d.filled_volume))
                    .ok_or_else(|| incomplete("recovered opening quantity overflow"))?;
                if filled > 0 {
                    candidates.push((anchor, o.order_id, filled));
                }
            }
        }
        if candidates.len() != 1 {
            return Err(incomplete(
                "scoped recovery found no unique owned opening order",
            ));
        }
        let (anchor, order, filled) = candidates.remove(0);
        validate_order_coverage(&orders.order, &unique_deals, &HashSet::from([order]))?;
        if filled > anchor.volume.unwrap_or(0) {
            return Err(mixed("recovered opening exceeds instruction"));
        }
        let opening_deals = unique_deals
            .values()
            .filter(|d| d.order_id == order && d.close_position_detail.is_none())
            .collect::<Vec<_>>();
        let price = opening_deals
            .iter()
            .all(|d| d.execution_price.is_some_and(|p| p.is_finite() && p > 0.0))
            .then(|| {
                opening_deals
                    .iter()
                    .map(|d| d.execution_price.unwrap() * d.filled_volume as f64)
                    .sum::<f64>()
                    / filled as f64
            });
        // Retain the actual broker response, not a fabricated execution event.
        let raw = orders.encode_to_vec();
        let recorded: bool = jiaoyi_store::sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM trading_events WHERE account_id=$1 AND position_id=$2 AND entry_key=$3 AND payload_type=$4 AND raw=$5)"
        ).bind(account).bind(p.position_id).bind(&anchor.entry_key)
            .bind(ProtoOaPayloadType::ProtoOaOrderListByPositionIdRes as i32).bind(&raw)
            .fetch_one(pool).await.map_err(|e| incomplete(&e.to_string()))?;
        if !recorded {
            jiaoyi_store::trading::record_event(
                pool,
                &jiaoyi_store::trading::EventRow {
                    account_id: account,
                    entry_key: Some(anchor.entry_key),
                    client_msg_id: None,
                    payload_type: ProtoOaPayloadType::ProtoOaOrderListByPositionIdRes as i32,
                    kind: if Some(filled) == anchor.volume {
                        "RECONCILED_FILL"
                    } else {
                        "RECONCILED_PARTIAL_FILL"
                    }
                    .into(),
                    position_id: Some(p.position_id),
                    order_id: Some(order),
                    deal_id: None,
                    symbol_id: Some(p.trade_data.symbol_id),
                    side: anchor.side,
                    price,
                    volume: Some(filled),
                    stop_loss: None,
                    label: p.trade_data.label.clone(),
                    error_code: None,
                    description: Some("unique scoped broker opening-order recovery".into()),
                    is_server_event: None,
                    raw,
                    received_at: started,
                },
            )
            .await
            .map_err(|e| incomplete(&e.to_string()))?;
        }
        let rows = deals
            .deal
            .iter()
            .map(crate::history::deal_row)
            .collect::<Vec<_>>();
        jiaoyi_store::history::upsert_deals(pool, account, &rows, Utc::now())
            .await
            .map_err(|e| incomplete(&e.to_string()))?;
        let needs_marker: bool = jiaoyi_store::sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM trading_events g WHERE g.account_id=$1 AND g.position_id=$2
             AND g.kind='AUTHORITY_GAP' AND g.received_at <= $3
             AND NOT EXISTS(SELECT 1 FROM trading_events c WHERE c.account_id=g.account_id AND c.position_id=g.position_id
                 AND c.kind='AUTHORITY_RECOVERY_COMPLETE' AND c.received_at >= g.received_at))"
        ).bind(account).bind(p.position_id).bind(started).fetch_one(pool).await.map_err(|e| incomplete(&e.to_string()))?;
        if needs_marker {
            marker(
                pool,
                account,
                p.position_id,
                "AUTHORITY_RECOVERY_COMPLETE",
                "complete scoped broker order/deal response",
                started,
            )
            .await?;
        }
        // Completion marks coverage, not ownership: mixed history still fails.
        position_authority(pool, account, p.position_id)
            .await?
            .validate_current(p)
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), work)
        .await
        .map_err(|_| incomplete("scoped broker recovery timed out"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiaoyi_ctrader::messages::{ProtoOaClosePositionDetail, ProtoOaTradeData};

    fn deal(id: i64, order: i64, volume: i64, closing: bool) -> ProtoOaDeal {
        ProtoOaDeal {
            deal_id: id,
            order_id: order,
            position_id: 7,
            symbol_id: 22,
            filled_volume: volume,
            volume,
            execution_timestamp: id,
            trade_side: if closing { 2 } else { 1 },
            deal_status: 2,
            close_position_detail: closing.then_some(ProtoOaClosePositionDetail {
                closed_volume: Some(volume),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn receipt(d: ProtoOaDeal, correlated: bool) -> evidence::Receipt {
        evidence::Receipt {
            kind: "ORDER_FILLED".into(),
            entry_key: correlated.then(|| "s3:owned".into()),
            client_msg_id: correlated.then(|| "request-1".into()),
            payload_type: ProtoOaPayloadType::ProtoOaExecutionEvent as i32,
            raw: ProtoOaExecutionEvent {
                ctid_trader_account_id: 42,
                execution_type: ProtoOaExecutionType::OrderFilled as i32,
                deal: Some(d),
                ..Default::default()
            }
            .encode_to_vec(),
            received_at: Utc::now(),
        }
    }

    fn base() -> evidence::Evidence {
        evidence::Evidence {
            openings: vec![evidence::Opening {
                entry_key: "s3:owned".into(),
                client_msg_id: "request-1".into(),
                symbol_id: Some(22),
                side: Some("buy".into()),
                volume: Some(100),
                created_at: Utc::now(),
            }],
            receipts: vec![receipt(deal(1, 99, 50, false), true)],
            deals: vec![],
            truncated: false,
        }
    }

    #[test]
    fn receipts_before_history_are_sufficient_and_requested_remainder_is_not_owned() {
        assert_eq!(qualify(42, 7, &base()).unwrap().remaining_volume, 50);
    }

    #[test]
    fn same_order_late_partial_is_owned_but_foreign_addition_below_requested_is_not() {
        let mut e = base();
        e.deals.push(deal(2, 99, 25, false).encode_to_vec());
        assert_eq!(qualify(42, 7, &e).unwrap().remaining_volume, 75);
        e.deals[0] = deal(2, 100, 25, false).encode_to_vec();
        assert!(qualify(42, 7, &e).unwrap_err().contains("not correlated"));
    }

    #[test]
    fn closing_deal_side_reduces_original_owned_side_once_across_stream_and_history() {
        let mut e = base();
        let close = deal(2, 100, 10, true);
        e.receipts.push(receipt(close, false));
        e.deals.push(close.encode_to_vec());
        let a = qualify(42, 7, &e).unwrap();
        assert_eq!(a.side, Side::Buy);
        assert_eq!(a.remaining_volume, 40);
    }

    #[test]
    fn same_side_same_size_reopening_and_mixed_reversal_are_refused() {
        let mut e = base();
        e.deals.push(deal(2, 100, 50, true).encode_to_vec());
        e.deals.push(deal(3, 101, 50, false).encode_to_vec());
        assert!(qualify(42, 7, &e).unwrap_err().contains("terminal close"));
        let mut e = base();
        let mut reverse = deal(2, 100, 75, true);
        reverse
            .close_position_detail
            .as_mut()
            .unwrap()
            .closed_volume = Some(50);
        e.deals.push(reverse.encode_to_vec());
        assert!(qualify(42, 7, &e)
            .unwrap_err()
            .contains("opening remainder"));
    }

    #[test]
    fn gap_is_not_cleared_by_old_or_interrupted_recovery() {
        let mut e = base();
        let mut marker = e.receipts[0].clone();
        marker.payload_type = 0;
        marker.raw.clear();
        marker.entry_key = None;
        marker.client_msg_id = None;
        marker.kind = "AUTHORITY_GAP".into();
        let at = marker.received_at;
        e.receipts.push(marker.clone());
        assert!(qualify(42, 7, &e).unwrap_err().contains("continuity gap"));
        marker.kind = "AUTHORITY_RECOVERY_COMPLETE".into();
        marker.received_at = at - chrono::Duration::milliseconds(1);
        e.receipts.push(marker.clone());
        assert!(qualify(42, 7, &e).is_err());
        marker.received_at = at + chrono::Duration::milliseconds(1);
        e.receipts.push(marker);
        assert_eq!(qualify(42, 7, &e).unwrap().remaining_volume, 50);
    }

    #[test]
    fn current_quantity_discrepancy_is_incomplete_not_automatically_foreign() {
        let a = qualify(42, 7, &base()).unwrap();
        let mut p = ProtoOaPosition {
            position_id: 7,
            position_status: 1,
            trade_data: ProtoOaTradeData {
                symbol_id: 22,
                trade_side: 1,
                volume: 50,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(a.validate_current(&p).is_ok());
        p.trade_data.volume = 75;
        assert!(a
            .validate_current(&p)
            .unwrap_err()
            .starts_with("authority incomplete"));
        p.trade_data.trade_side = 2;
        assert!(a
            .validate_current(&p)
            .unwrap_err()
            .starts_with("authority mixed"));
    }

    #[test]
    fn an_empty_raw_reconciled_snapshot_cannot_certify_an_opening() {
        let mut e = base();
        e.receipts[0].payload_type = 0;
        e.receipts[0].raw.clear();
        e.receipts[0].kind = "RECONCILED_FILL".into();
        assert!(qualify(42, 7, &e)
            .unwrap_err()
            .contains("no positively correlated"));
    }

    fn observed_order(id: i64, executed: i64, closing: bool) -> ProtoOaOrder {
        ProtoOaOrder {
            order_id: id,
            position_id: Some(7),
            closing_order: Some(closing),
            order_status: 2,
            executed_volume: Some(executed),
            trade_data: ProtoOaTradeData {
                symbol_id: 22,
                trade_side: if closing { 2 } else { 1 },
                volume: 100,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn observe_orders(e: &mut evidence::Evidence, order: Vec<ProtoOaOrder>) {
        e.receipts.push(evidence::Receipt {
            kind: "AUTHORITY_ORDER_OBSERVATION".into(),
            entry_key: None,
            client_msg_id: None,
            payload_type: ProtoOaPayloadType::ProtoOaOrderListByPositionIdRes as i32,
            raw: ProtoOaOrderListByPositionIdRes {
                ctid_trader_account_id: 42,
                order,
                has_more: false,
                ..Default::default()
            }
            .encode_to_vec(),
            received_at: Utc::now(),
        });
    }

    #[test]
    fn executed_foreign_reopen_is_rejected_even_when_deals_still_show_only_the_owned_open() {
        let mut e = base();
        observe_orders(
            &mut e,
            vec![
                observed_order(99, 50, false),
                observed_order(100, 50, true),
                observed_order(101, 50, false),
            ],
        );
        assert!(qualify(42, 7, &e)
            .unwrap_err()
            .contains("executed foreign opening"));
    }

    #[test]
    fn recovered_partial_and_close_need_executed_deal_coverage_not_requested_quantity() {
        let mut e = base();
        observe_orders(
            &mut e,
            vec![observed_order(99, 50, false), observed_order(100, 10, true)],
        );
        assert!(qualify(42, 7, &e).unwrap_err().contains("ahead"));
        e.deals.push(deal(2, 100, 10, true).encode_to_vec());
        assert_eq!(qualify(42, 7, &e).unwrap().remaining_volume, 40);
    }

    #[test]
    fn recovery_completion_cannot_resurrect_a_position_observed_absent() {
        let mut e = base();
        for kind in ["AUTHORITY_DISAPPEARED", "AUTHORITY_RECOVERY_COMPLETE"] {
            e.receipts.push(evidence::Receipt {
                kind: kind.into(),
                entry_key: None,
                client_msg_id: None,
                payload_type: 0,
                raw: vec![],
                received_at: Utc::now(),
            });
        }
        assert!(qualify(42, 7, &e).unwrap_err().contains("observed absent"));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn thousands_of_trailing_echoes_do_not_exhaust_owned_quantity_evidence(pool: PgPool) {
        let e = base();
        sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,max_risk)
            VALUES(42,'s3:owned','request-1','open',22,'buy',100,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,raw,received_at)
            VALUES(42,'s3:owned','request-1',$1,'ORDER_FILLED',7,1,$2,now())")
            .bind(e.receipts[0].payload_type).bind(&e.receipts[0].raw).execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO trading_events(account_id,payload_type,kind,position_id,raw,received_at)
            SELECT 42,2126,'ORDER_REPLACED',7,''::bytea,now() FROM generate_series(1,2050)",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(
            position_authority(&pool, 42, 7)
                .await
                .unwrap()
                .remaining_volume,
            50
        );
        let loaded = evidence::position_evidence(&pool, 42, 7).await.unwrap();
        assert_eq!(
            loaded.receipts.len(),
            1,
            "amend echoes are not ownership increments"
        );
        assert!(!loaded.truncated);
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn truncated_wire_key_collision_preserves_both_candidates(pool: PgPool) {
        let prefix = "x".repeat(50);
        for suffix in ["one", "two"] {
            sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,max_risk)
                VALUES(42,$1,$2,'open',22,'buy',100,1)")
                .bind(format!("{prefix}{suffix}")).bind(suffix).execute(&pool).await.unwrap();
        }
        assert_eq!(
            evidence::openings_for_wire_key(&pool, 42, 22, &prefix)
                .await
                .unwrap()
                .len(),
            2
        );
    }
}
