//! Read-only margin-buffer admission. Only new entries use this module.
//! Estimates reserve gross exposure without assuming hedge-margin discounts.
use crate::trading::{Command, Side};
use crate::{pb, AccountSession};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

type Locks = Mutex<HashMap<(String, i64), Arc<AsyncMutex<()>>>>;
static LOCKS: OnceLock<Locks> = OnceLock::new();
pub(crate) fn try_lock(namespace: String, account: i64) -> Result<OwnedMutexGuard<()>, String> {
    let lock = LOCKS
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .entry((namespace, account))
        .or_default()
        .clone();
    lock.try_lock_owned().map_err(|_| {
        "another account entry is being admitted; no concurrent margin reservation".into()
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Receipt {
    pub schema_version: u32,
    pub state: &'static str,
    pub floor_pct: Option<f64>,
    pub projected_level_pct: Option<f64>,
    pub stressed_level_pct: Option<f64>,
    pub warning_pct: Option<f64>,
    pub positions: usize,
    pub pending_entries: usize,
    pub reason: Option<String>,
}
impl Receipt {
    pub fn unavailable(reason: String) -> Self {
        Self {
            schema_version: 1,
            state: "unavailable",
            floor_pct: None,
            projected_level_pct: None,
            stressed_level_pct: None,
            warning_pct: None,
            positions: 0,
            pending_entries: 0,
            reason: Some(reason),
        }
    }
}
fn money(value: i64, digits: Option<u32>) -> Result<f64, String> {
    let d = digits
        .filter(|d| *d <= 18)
        .ok_or("margin money precision missing/invalid")?;
    Ok(value as f64 / 10f64.powi(d as i32))
}
fn same_book(a: &pb::ProtoOaReconcileRes, b: &pb::ProtoOaReconcileRes) -> bool {
    let positions = |r: &pb::ProtoOaReconcileRes| {
        let mut v = r.position.clone();
        v.sort_by_key(|p| p.position_id);
        v
    };
    let orders = |r: &pb::ProtoOaReconcileRes| {
        let mut v = r.order.clone();
        v.sort_by_key(|o| o.order_id);
        v
    };
    positions(a) == positions(b) && orders(a) == orders(b)
}

/// Pure numeric decision, also used by the broker-backed gate tests.
pub fn evaluate(
    equity: f64,
    projected_margin: f64,
    loss_reserve: f64,
    floor: f64,
    warning: f64,
) -> Result<(f64, f64), String> {
    if ![equity, projected_margin, loss_reserve, floor, warning]
        .iter()
        .all(|v| v.is_finite())
        || equity <= 0.0
        || projected_margin <= 0.0
        || loss_reserve < 0.0
        || floor <= 0.0
        || warning <= 0.0
    {
        return Err("margin buffer inputs are missing or invalid".into());
    }
    let projected = 100.0 * equity / projected_margin;
    let stressed = 100.0 * (equity - loss_reserve) / projected_margin;
    if !projected.is_finite() || !stressed.is_finite() {
        return Err("margin projection overflow".into());
    }
    if projected < floor {
        return Err(format!(
            "projected margin level {projected:.2}% below floor {floor:.2}%"
        ));
    }
    if stressed <= warning {
        return Err(format!(
            "margin level after stop reserves {stressed:.2}% does not exceed warning {warning:.2}%"
        ));
    }
    Ok((projected, stressed))
}

/// Uses fresh broker state, account-specific warnings and exact-volume estimates.
/// Unsupported/ambiguous protection is an entry refusal, never a stop change.
pub async fn check(
    session: &AccountSession<'_>,
    trader: &pb::ProtoOaTrader,
    book: &pb::ProtoOaReconcileRes,
    command: &Command,
    floor: f64,
) -> Result<Receipt, String> {
    let Command::Open {
        symbol_id,
        side,
        volume,
        relative_stop,
        quote_to_deposit,
        ..
    } = command
    else {
        return Err("margin admission is for entries only".into());
    };
    let account = session.account().id();
    if trader.ctid_trader_account_id != account || book.ctid_trader_account_id != account {
        return Err("margin source account mismatch".into());
    }
    if trader.is_limited_risk == Some(true)
        || trader.limited_risk_margin_calculation_strategy == Some(1)
    {
        return Err("limited-risk/GSL margin needs compatible guaranteed protection; no ordinary-stop entry".into());
    }
    let pnl = session
        .unrealized_pnl()
        .await
        .map_err(|e| format!("margin P&L unreadable: {e}"))?;
    let calls = session
        .margin_calls()
        .await
        .map_err(|e| format!("margin warnings unreadable: {e}"))?;
    if calls
        .margin_call
        .iter()
        .any(|c| !c.margin_level_threshold.is_finite() || c.margin_level_threshold < 0.0)
    {
        return Err("invalid broker margin warning threshold".into());
    }
    let warning = calls
        .margin_call
        .iter()
        .map(|c| c.margin_level_threshold)
        .filter(|v| v.is_finite() && *v > 0.0)
        .reduce(f64::max)
        .ok_or("no positive broker margin warning threshold")?;
    let symbols = session
        .symbols_list()
        .await
        .map_err(|e| format!("margin symbol identity unreadable: {e}"))?;
    let quote = |id: i64| -> Result<i64, String> {
        let v: Vec<_> = symbols
            .symbol
            .iter()
            .filter(|s| s.symbol_id == id)
            .collect();
        if v.len() != 1 {
            return Err("margin symbol identity missing/ambiguous".into());
        }
        v[0].quote_asset_id
            .filter(|v| *v > 0)
            .ok_or("margin quote asset missing".into())
    };
    let candidate_quote = quote(*symbol_id)?;
    if candidate_quote == trader.deposit_asset_id
        && quote_to_deposit.is_none_or(|r| !r.is_finite() || (r - 1.0).abs() > 1e-9)
    {
        return Err("quote-to-deposit rate disagrees with broker currency identity".into());
    }
    let rate = |id: i64| -> Result<f64, String> {
        let asset = quote(id)?;
        if asset == trader.deposit_asset_id {
            return Ok(1.0);
        }
        if asset == candidate_quote {
            return quote_to_deposit
                .filter(|r| r.is_finite() && *r > 0.0)
                .ok_or("margin conversion rate unknown".into());
        }
        Err("existing exposure has no fresh verified quote-to-deposit conversion".into())
    };
    let mut marks = HashMap::new();
    for p in &pnl.position_unrealized_pn_l {
        if p.position_id <= 0 || marks.insert(p.position_id, p).is_some() {
            return Err("margin P&L IDs invalid/duplicate".into());
        }
    }
    let mut ids = BTreeSet::new();
    let mut current: BTreeMap<i64, [i64; 2]> = BTreeMap::new();
    let mut future: BTreeMap<i64, [i64; 2]> = BTreeMap::new();
    let mut equity = money(trader.balance, trader.money_digits)?;
    let mut used = 0.0;
    let mut reserve = crate::trading::risk_of(*volume, *relative_stop, rate(*symbol_id)?);
    let mut pending = 0;
    let sign = |s: i32| -> Result<(usize, f64), String> {
        match s {
            1 => Ok((0, 1.0)),
            2 => Ok((1, -1.0)),
            _ => Err("margin trade side invalid".into()),
        }
    };
    let add =
        |map: &mut BTreeMap<i64, [i64; 2]>, id: i64, index: usize, v: i64| -> Result<(), String> {
            if v <= 0 {
                return Err("margin volume invalid".into());
            }
            let value = &mut map.entry(id).or_default()[index];
            *value = value.checked_add(v).ok_or("margin volume overflow")?;
            Ok(())
        };
    for p in &book.position {
        if p.position_id <= 0 || p.position_status != 1 || !ids.insert(p.position_id) {
            return Err("margin position identity/status invalid".into());
        }
        let m = marks
            .remove(&p.position_id)
            .ok_or("margin P&L coverage incomplete")?;
        equity += money(m.net_unrealized_pn_l, Some(pnl.money_digits))?;
        used += money(
            i64::try_from(p.used_margin.ok_or("position used margin missing")?)
                .map_err(|_| "used margin overflow")?,
            p.money_digits,
        )?;
        let (index, sgn) = sign(p.trade_data.trade_side)?;
        add(
            &mut current,
            p.trade_data.symbol_id,
            index,
            p.trade_data.volume,
        )?;
        add(
            &mut future,
            p.trade_data.symbol_id,
            index,
            p.trade_data.volume,
        )?;
        let entry = p
            .price
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or("position entry unknown")?;
        let stop = p
            .stop_loss
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or("existing position has no verified stop")?;
        let gross = money(m.gross_unrealized_pn_l, Some(pnl.money_digits))?;
        reserve += (gross
            - sgn * (stop - entry) * p.trade_data.volume as f64 / 100.0
                * rate(p.trade_data.symbol_id)?)
        .max(0.0);
    }
    // Some brokers allocate a future position ID while an entry is pending.
    // An exactly matched zero mark has no equity effect; it is not adopted.
    let mut future_ids = BTreeSet::new();
    for order in book
        .order
        .iter()
        .filter(|o| o.closing_order != Some(true) && o.order_type != 4)
    {
        if let Some(id) = order.position_id {
            if id <= 0
                || ids.contains(&id)
                || order.executed_volume != Some(0)
                || order.order_status != 1
                || !future_ids.insert(id)
            {
                return Err("pending entry has ambiguous position linkage".into());
            }
            if let Some(mark) = marks.remove(&id) {
                if mark.net_unrealized_pn_l != 0 || mark.gross_unrealized_pn_l != 0 {
                    return Err("pending future position has nonzero P&L".into());
                }
            }
        }
    }
    let mut order_ids = BTreeSet::new();
    for o in &book.order {
        if o.order_id <= 0 || !order_ids.insert(o.order_id) {
            return Err("pending order IDs invalid/duplicate".into());
        }
        if o.closing_order == Some(true) || o.order_type == 4 {
            if !o
                .position_id
                .is_some_and(|id| ids.contains(&id) || future_ids.contains(&id))
            {
                return Err("pending protection link ambiguous".into());
            }
            continue;
        }
        if o.trade_data.guaranteed_stop_loss == Some(true) {
            return Err("pending GSL entry needs a compatible margin estimate".into());
        }
        let executed = o.executed_volume.ok_or("pending executed volume unknown")?;
        let remaining = o
            .trade_data
            .volume
            .checked_sub(executed)
            .filter(|v| *v >= 0)
            .ok_or("pending remaining volume invalid")?;
        if remaining == 0 {
            continue;
        }
        let (index, sgn) = sign(o.trade_data.trade_side)?;
        let entry = match o.order_type {
            2 | 6 => o.limit_price,
            3 => o.stop_price,
            _ => None,
        }
        .filter(|v| v.is_finite() && *v > 0.0)
        .ok_or("pending trigger price unknown")?;
        let stop = o
            .stop_loss
            .or_else(|| {
                o.relative_stop_loss
                    .map(|r| entry - sgn * r as f64 / 100000.0)
            })
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or("pending entry has no verified stop")?;
        if sgn * (entry - stop) <= 0.0 {
            return Err("pending stop is not on loss side".into());
        }
        reserve += sgn * (entry - stop) * remaining as f64 / 100.0 * rate(o.trade_data.symbol_id)?;
        add(&mut future, o.trade_data.symbol_id, index, remaining)?;
        pending += 1;
    }
    if !marks.is_empty() {
        return Err("margin P&L has unmatched positions".into());
    }
    add(
        &mut future,
        *symbol_id,
        if *side == Side::Buy { 0 } else { 1 },
        *volume,
    )?;
    let mut initial_margin = 0.0;
    let mut final_margin = 0.0;
    for (id, amounts) in &future {
        let before = current.get(id).copied().unwrap_or_default();
        let volumes: BTreeSet<i64> = before
            .into_iter()
            .chain(*amounts)
            .filter(|v| *v > 0)
            .collect();
        let res = session
            .expected_margin(*id, volumes.iter().copied().collect())
            .await
            .map_err(|e| format!("broker margin estimate unreadable: {e}"))?;
        let value = |v: i64, index: usize| -> Result<f64, String> {
            if v == 0 {
                return Ok(0.0);
            }
            let rows: Vec<_> = res.margin.iter().filter(|m| m.volume == v).collect();
            if rows.len() != 1 {
                return Err("margin estimate missing/duplicate volume".into());
            }
            let raw = if index == 0 {
                rows[0].buy_margin
            } else {
                rows[0].sell_margin
            };
            if raw < 0 {
                return Err("negative margin estimate".into());
            }
            money(raw, res.money_digits)
        };
        for index in 0..2 {
            initial_margin += value(before[index], index)?;
            final_margin += value(amounts[index], index)?;
        }
    }
    if final_margin < initial_margin {
        return Err("margin estimate falls as gross exposure increases".into());
    }
    let projected = final_margin.max(used + (final_margin - initial_margin));
    let after = session
        .margin_book()
        .await
        .map_err(|e| format!("margin final book unreadable: {e}"))?;
    let balance = session
        .trader()
        .await
        .map_err(|e| format!("margin final balance unreadable: {e}"))?;
    if !same_book(book, &after) || trader != &balance {
        return Err("account state changed during margin admission".into());
    }
    let (level, stress) = evaluate(equity, projected, reserve, floor, warning)?;
    Ok(Receipt {
        schema_version: 1,
        state: "passed",
        floor_pct: Some(floor),
        projected_level_pct: Some(level),
        stressed_level_pct: Some(stress),
        warning_pct: Some(warning),
        positions: book.position.len(),
        pending_entries: pending,
        reason: None,
    })
}
