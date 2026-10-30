//! Private whole-account marks from the history worker's existing read session.
//! No orders, subscriptions, token refreshes or historical balance estimates.
use std::collections::{BTreeSet, HashMap};

use anyhow::Context;
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use jiaoyi_ctrader::{messages as pb, AccountSession};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use serde_json::{json, Value};

const MAX_CLOSE_DELAY_MS: i64 = 90_000;
const MAX_QUOTE_AGE_SECS: i64 = 120;

fn money(value: i64, digits: Option<u32>) -> Option<f64> {
    let digits = digits.filter(|d| *d <= 18)?;
    Some(value as f64 / 10f64.powi(digits as i32))
}

fn close_candidate(
    started: DateTime<Utc>,
    observed: DateTime<Utc>,
) -> Option<(String, DateTime<Utc>)> {
    let date = observed.with_timezone(&New_York).date_naive();
    let cutoff = New_York
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(17, 0, 0)?))
        .single()?
        .with_timezone(&Utc);
    (started >= cutoff
        && observed >= cutoff
        && observed.signed_duration_since(cutoff).num_milliseconds() <= MAX_CLOSE_DELAY_MS)
        .then(|| (format!("close:{date}"), cutoff))
}

fn same_book(a: &pb::ProtoOaReconcileRes, b: &pb::ProtoOaReconcileRes) -> bool {
    let mut ap: Vec<_> = a.position.iter().collect();
    let mut bp: Vec<_> = b.position.iter().collect();
    ap.sort_by_key(|p| p.position_id);
    bp.sort_by_key(|p| p.position_id);
    let mut ao: Vec<_> = a.order.iter().collect();
    let mut bo: Vec<_> = b.order.iter().collect();
    ao.sort_by_key(|p| p.order_id);
    bo.sort_by_key(|p| p.order_id);
    ap == bp && ao == bo
}

// The margin gate already recognises broker-allocated future position IDs.
// A zero mark for an accepted, explicitly unfilled pending entry changes no
// equity. It is still a pending order, never an owned/open position or zero risk.
fn pending_zero_marks(
    book: &pb::ProtoOaReconcileRes,
    pnl: &pb::ProtoOaGetPositionUnrealizedPnLRes,
) -> BTreeSet<i64> {
    let order_ids: BTreeSet<_> = book.order.iter().map(|o| o.order_id).collect();
    let mark_ids: BTreeSet<_> = pnl
        .position_unrealized_pn_l
        .iter()
        .map(|p| p.position_id)
        .collect();
    if book.ctid_trader_account_id != pnl.ctid_trader_account_id
        || order_ids.len() != book.order.len()
        || order_ids.iter().any(|id| *id <= 0)
        || mark_ids.len() != pnl.position_unrealized_pn_l.len()
    {
        return BTreeSet::new();
    }
    pnl.position_unrealized_pn_l
        .iter()
        .filter(|mark| {
            if mark.position_id <= 0
                || mark.gross_unrealized_pn_l != 0
                || mark.net_unrealized_pn_l != 0
                || book
                    .position
                    .iter()
                    .any(|p| p.position_id == mark.position_id)
            {
                return false;
            }
            let mut entries = book.order.iter().filter(|o| {
                o.position_id == Some(mark.position_id)
                    && o.closing_order != Some(true)
                    && o.order_type != 4
            });
            let Some(order) = entries.next() else {
                return false;
            };
            entries.next().is_none()
                && order.order_status == 1
                && order.executed_volume == Some(0)
                && matches!(order.order_type, 2 | 3 | 6)
                && order.trade_data.volume > 0
                && order.trade_data.symbol_id > 0
                && matches!(order.trade_data.trade_side, 1 | 2)
        })
        .map(|mark| mark.position_id)
        .collect()
}

fn pnl_covers(
    book: &pb::ProtoOaReconcileRes,
    pnl: &pb::ProtoOaGetPositionUnrealizedPnLRes,
) -> bool {
    let positions: BTreeSet<_> = book.position.iter().map(|p| p.position_id).collect();
    let marks: BTreeSet<_> = pnl
        .position_unrealized_pn_l
        .iter()
        .map(|p| p.position_id)
        .collect();
    let pending_zero = pending_zero_marks(book, pnl);
    book.ctid_trader_account_id == pnl.ctid_trader_account_id
        && positions.len() == book.position.len()
        && marks.len() == pnl.position_unrealized_pn_l.len()
        && positions.is_subset(&marks)
        && marks
            .difference(&positions)
            .all(|id| pending_zero.contains(id))
        && book
            .position
            .iter()
            .all(|p| p.position_id > 0 && p.position_status == 1)
}

// Explain the existing predicate using only fixed counts. Duplicate counts
// mean surplus rows beyond each distinct ID, not the number of repeated IDs.
fn pnl_coverage_diagnostics(
    book: &pb::ProtoOaReconcileRes,
    pnl: &pb::ProtoOaGetPositionUnrealizedPnLRes,
) -> Value {
    let positions: BTreeSet<_> = book.position.iter().map(|p| p.position_id).collect();
    let marks: BTreeSet<_> = pnl
        .position_unrealized_pn_l
        .iter()
        .map(|p| p.position_id)
        .collect();
    let pending_zero = pending_zero_marks(book, pnl);
    json!({
        "missing_pnl_position_count": positions.difference(&marks).count(),
        "extra_pnl_position_count": marks.difference(&positions).count(),
        "pending_zero_pnl_position_count": pending_zero.len(),
        "unresolved_extra_pnl_position_count": marks.difference(&positions).filter(|id| !pending_zero.contains(id)).count(),
        "duplicate_position_id_count": book.position.len() - positions.len(),
        "duplicate_pnl_id_count": pnl.position_unrealized_pn_l.len() - marks.len(),
        "non_open_position_count": book.position.iter().filter(|p| p.position_status != 1).count(),
    })
}

// Counts describe observed links only: history sync can lag this sample and a
// closing deal may be partial. No claim about full closure or stale marks follows.
async fn extra_pnl_relationship_diagnostics(
    pool: &PgPool,
    account: i64,
    book: &pb::ProtoOaReconcileRes,
    pnl: &pb::ProtoOaGetPositionUnrealizedPnLRes,
    observed: DateTime<Utc>,
) -> Value {
    let mut result = json!({
        "evidence_state":"response_account_mismatch",
        "pending_entry_position_match_count":null,
        "pending_protection_position_match_count":null,
        "pending_order_missing_position_ref_count":null,
        "opening_deal_observed_count":null,
        "closing_deal_observed_count":null,
        "owned_fill_observed_count":null,
    });
    if book.ctid_trader_account_id != account || pnl.ctid_trader_account_id != account {
        return result;
    }
    if book.position.iter().any(|p| p.position_id <= 0)
        || pnl
            .position_unrealized_pn_l
            .iter()
            .any(|p| p.position_id <= 0)
        || book
            .order
            .iter()
            .any(|o| o.position_id.is_some_and(|id| id <= 0))
    {
        result["evidence_state"] = json!("invalid_position_id");
        return result;
    }
    let positions: BTreeSet<_> = book.position.iter().map(|p| p.position_id).collect();
    let extras: BTreeSet<_> = pnl
        .position_unrealized_pn_l
        .iter()
        .map(|p| p.position_id)
        .filter(|id| !positions.contains(id))
        .collect();
    let protection = |o: &pb::ProtoOaOrder| o.closing_order == Some(true) || o.order_type == 4;
    for (field, protective) in [
        ("pending_entry_position_match_count", false),
        ("pending_protection_position_match_count", true),
    ] {
        result[field] = json!(extras
            .iter()
            .filter(|id| book
                .order
                .iter()
                .any(|o| o.position_id == Some(**id) && protection(o) == protective))
            .count());
    }
    result["pending_order_missing_position_ref_count"] = json!(book
        .order
        .iter()
        .filter(|o| o.position_id.is_none())
        .count());
    if extras.is_empty() {
        result["evidence_state"] = json!("not_applicable");
        for field in [
            "opening_deal_observed_count",
            "closing_deal_observed_count",
            "owned_fill_observed_count",
        ] {
            result[field] = json!(0);
        }
        return result;
    }
    // Only the extra IDs on this account; no payloads/IDs/amounts are retained.
    let ids: Vec<_> = extras.into_iter().collect();
    let rows = sqlx::query(
        r#"SELECT
        EXISTS(SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=x.id
            AND d.executed_at <= $3 AND d.filled_volume>0
            AND d.deal_status IN ('FILLED','PARTIALLY_FILLED') AND NOT d.is_closing) AS opening,
        EXISTS(SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=x.id
            AND d.executed_at <= $3 AND d.filled_volume>0
            AND d.deal_status IN ('FILLED','PARTIALLY_FILLED') AND d.is_closing) AS closing,
        EXISTS(SELECT 1 FROM trading_events e JOIN trading_intents i
            ON i.entry_key=e.entry_key AND i.account_id=e.account_id
            WHERE e.account_id=$1 AND e.position_id=x.id AND i.kind='open'
            AND e.kind=ANY($4) AND e.volume>0 AND e.received_at <= $3) AS owned
        FROM unnest($2::bigint[]) AS x(id)"#,
    )
    .bind(account)
    .bind(&ids)
    .bind(observed)
    .bind(jiaoyi_store::trading::FILL_KINDS)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            result["evidence_state"] = json!("stored_observations_only_closure_unknown");
            for (field, column) in [
                ("opening_deal_observed_count", "opening"),
                ("closing_deal_observed_count", "closing"),
                ("owned_fill_observed_count", "owned"),
            ] {
                result[field] = json!(rows.iter().filter(|r| r.get::<bool, _>(column)).count());
            }
        }
        Err(_) => result["evidence_state"] = json!("store_read_failed"),
    }
    result
}

fn same_balance(a: &pb::ProtoOaTrader, b: &pb::ProtoOaTrader) -> bool {
    a.balance == b.balance
        && a.balance_version == b.balance_version
        && a.money_digits == b.money_digits
        && a.deposit_asset_id == b.deposit_asset_id
        && a.manager_bonus == b.manager_bonus
        && a.ib_bonus == b.ib_bonus
        && a.non_withdrawable_bonus == b.non_withdrawable_bonus
        && a.total_margin_calculation_type == b.total_margin_calculation_type
}

struct Read {
    trader: pb::ProtoOaTrader,
    book: pb::ProtoOaReconcileRes,
    pnl: pb::ProtoOaGetPositionUnrealizedPnLRes,
    started: DateTime<Utc>,
    observed: DateTime<Utc>,
    book_stable: bool,
    balance_stable: bool,
    pnl_coverage_complete: bool,
    source_times: Value,
}

impl Read {
    fn consistent(&self) -> bool {
        self.book_stable && self.balance_stable && self.pnl_coverage_complete
    }

    fn failure_reasons(&self) -> Vec<&'static str> {
        [
            (!self.book_stable, "book_changed"),
            (!self.balance_stable, "balance_changed"),
            (
                !self.pnl_coverage_complete,
                "pnl_position_coverage_incomplete",
            ),
        ]
        .into_iter()
        .filter_map(|(failed, reason)| failed.then_some(reason))
        .collect()
    }
}

async fn read(session: &AccountSession<'_>) -> anyhow::Result<Read> {
    let started = DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    let before = session.trader().await?;
    let trader_before_at = DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    let book_before = session.reconcile().await?;
    let reconcile_before_at =
        DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    let pnl = session.unrealized_pnl().await?;
    let pnl_at = DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    let book = session.reconcile().await?;
    let reconcile_after_at =
        DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    let trader = session.trader().await?;
    // Millisecond marks match the broker's inclusive cash-flow timestamp unit.
    let observed = DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
    // Evaluate each predicate independently so a book/balance mismatch cannot
    // hide actual position-ID coverage. Valuation still requires all three.
    let book_stable = same_book(&book_before, &book);
    let balance_stable = same_balance(&before, &trader);
    let pnl_coverage_complete = pnl_covers(&book, &pnl);
    Ok(Read {
        trader,
        book,
        pnl,
        started,
        observed,
        book_stable,
        balance_stable,
        pnl_coverage_complete,
        source_times: json!({"trader_before":trader_before_at,"reconcile_before":reconcile_before_at,
            "unrealized":pnl_at,"reconcile_after":reconcile_after_at,"trader_after":observed}),
    })
}

#[derive(Default)]
struct Market {
    /// Broker symbol -> (base asset ID, quote asset ID).
    assets: HashMap<i64, (i64, i64)>,
    /// Fresh two-sided account quotes, never another account's or a subscription.
    mid: HashMap<i64, f64>,
}

impl Market {
    fn rate(&self, quote: i64, deposit: i64) -> Option<f64> {
        if quote == deposit {
            return Some(1.0);
        }
        let mut candidate = None;
        for (id, (base, counter)) in &self.assets {
            let Some(mid) = self
                .mid
                .get(id)
                .copied()
                .filter(|p| p.is_finite() && *p > 0.0)
            else {
                continue;
            };
            let rate = if *base == quote && *counter == deposit {
                Some(mid)
            } else if *base == deposit && *counter == quote {
                Some(1.0 / mid)
            } else {
                None
            };
            if let Some(rate) = rate {
                // Multiple broker instruments may represent the same pair.
                // No arbitrary HashMap iteration order selects their price.
                if candidate.replace(rate).is_some() {
                    return None;
                }
            }
        }
        candidate
    }
    fn symbol_rate(&self, symbol: i64, deposit: i64) -> Option<f64> {
        self.rate(self.assets.get(&symbol)?.1, deposit)
    }
}

async fn cached_market(
    pool: &PgPool,
    account: i64,
    observed: DateTime<Utc>,
    listed: &Value,
) -> anyhow::Result<Market> {
    let mut market = Market::default();
    if let Some(rows) = listed.as_array() {
        for r in rows {
            if let (Some(id), Some(base), Some(quote)) = (
                r["id"].as_i64(),
                r["base_asset_id"].as_i64(),
                r["quote_asset_id"].as_i64(),
            ) {
                market.assets.insert(id, (base, quote));
            }
        }
    }
    let rows = sqlx::query("WITH bids AS (SELECT DISTINCT ON (broker_symbol_id) broker_symbol_id,bid_e5 FROM ticks WHERE account_id=$1 AND received_at >= $2 AND received_at <= $3 AND bid_e5>0 ORDER BY broker_symbol_id,received_at DESC), asks AS (SELECT DISTINCT ON (broker_symbol_id) broker_symbol_id,ask_e5 FROM ticks WHERE account_id=$1 AND received_at >= $2 AND received_at <= $3 AND ask_e5>0 ORDER BY broker_symbol_id,received_at DESC) SELECT bids.broker_symbol_id,bid_e5,ask_e5 FROM bids JOIN asks USING (broker_symbol_id)")
        .bind(account).bind(observed - chrono::Duration::seconds(MAX_QUOTE_AGE_SECS))
        .bind(observed).fetch_all(pool).await?;
    for row in rows {
        let bid: i64 = row.get("bid_e5");
        let ask: i64 = row.get("ask_e5");
        if ask >= bid {
            market.mid.insert(
                row.get("broker_symbol_id"),
                (bid as f64 + ask as f64) / 200_000.0,
            );
        }
    }
    Ok(market)
}

fn position_margin(read: &Read) -> Option<f64> {
    if !read.consistent() {
        return None;
    }
    if read.book.position.is_empty() {
        return Some(0.0);
    }
    // MAX/NET margin cannot be established by adding position margins.
    if read.trader.total_margin_calculation_type != Some(1) {
        return None;
    }
    read.book.position.iter().try_fold(0.0, |sum, p| {
        let amount =
            p.used_margin? as f64 / 10f64.powi(p.money_digits.filter(|d| *d <= 18)? as i32);
        Some(sum + amount)
    })
}

fn risk_and_exposure(read: &Read, market: &Market, equity: Option<f64>) -> Value {
    let mut projection = Some(0.0);
    let mut notional = Some(0.0);
    let mut unprotected = 0;
    let mut unpriced = 0;
    for p in &read.book.position {
        let marked = read
            .pnl
            .position_unrealized_pn_l
            .iter()
            .find(|m| m.position_id == p.position_id);
        let rate = market.symbol_rate(p.trade_data.symbol_id, read.trader.deposit_asset_id);
        let units = p.trade_data.volume as f64 / 100.0;
        let side = match p.trade_data.trade_side {
            1 => Some(1.0),
            2 => Some(-1.0),
            _ => None,
        };
        let gross =
            marked.and_then(|m| money(m.gross_unrealized_pn_l, Some(read.pnl.money_digits)));
        let pricing = p
            .price
            .filter(|v| v.is_finite() && *v > 0.0)
            .zip(rate)
            .zip(side)
            .zip(gross)
            .filter(|_| units > 0.0);
        if p.stop_loss.is_none_or(|s| !s.is_finite() || s <= 0.0) {
            unprotected += 1;
        }
        if let Some((((entry, fx), sign), current_gross)) = pricing {
            // Derive current price from the broker's fresh gross P&L. Entry is
            // not a current quote. The same accrued costs cancel at the stop.
            let current = entry + current_gross / (sign * units * fx);
            if current.is_finite() && current > 0.0 {
                notional = notional.map(|n| n + units * current * fx);
            } else {
                notional = None;
            }
            if let Some(stop) = p.stop_loss.filter(|s| s.is_finite() && *s > 0.0) {
                let at_stop_gross = sign * (stop - entry) * units * fx;
                projection = projection.map(|n| n + (current_gross - at_stop_gross).max(0.0));
            } else {
                projection = None;
            }
        } else {
            unpriced += 1;
            projection = None;
            notional = None;
        }
    }
    let pending: Vec<_> = read
        .book
        .order
        .iter()
        .filter(|o| o.closing_order != Some(true) && o.order_type != 4)
        .collect();
    let linked_protection_ambiguous = read
        .book
        .order
        .iter()
        .filter(|o| {
            (o.closing_order == Some(true) || o.order_type == 4)
                && !o
                    .position_id
                    .is_some_and(|id| read.book.position.iter().any(|p| p.position_id == id))
        })
        .count();
    let mut pending_projection = Some(0.0);
    let mut unprotected_pending = 0;
    let mut unpriced_pending = 0;
    for o in &pending {
        let price = match o.order_type {
            2 | 6 => o.limit_price,
            3 => o.stop_price,
            _ => None,
        };
        let rate = market.symbol_rate(o.trade_data.symbol_id, read.trader.deposit_asset_id);
        let sign = match o.trade_data.trade_side {
            1 => Some(1.0),
            2 => Some(-1.0),
            _ => None,
        };
        let stop = o
            .stop_loss
            .or_else(|| Some(price? - sign? * o.relative_stop_loss? as f64 / 100_000.0));
        if stop.is_none_or(|s| !s.is_finite() || s <= 0.0) {
            unprotected_pending += 1;
        }
        // A linked entry can change an existing net position's protection;
        // neither independence nor the treatment of filled volume is guessed.
        let remaining = o
            .executed_volume
            .and_then(|v| o.trade_data.volume.checked_sub(v))
            .filter(|v| *v >= 0);
        if let Some(((((entry, stop), fx), side), volume)) =
            price.zip(stop).zip(rate).zip(sign).zip(remaining).filter(
                |((((entry, stop), _), _), _)| {
                    entry.is_finite() && *entry > 0.0 && stop.is_finite() && *stop > 0.0
                },
            )
        {
            if o.position_id.is_some() {
                pending_projection = None;
                unpriced_pending += 1;
            } else {
                pending_projection = pending_projection
                    .map(|n| n + (side * (entry - stop) * volume as f64 / 100.0 * fx).max(0.0));
            }
        } else {
            pending_projection = None;
            unpriced_pending += 1;
        }
    }
    if !read.consistent() || linked_protection_ambiguous > 0 {
        projection = None;
        pending_projection = None;
        notional = None;
    }
    // The API net UPL explicitly excludes potential closing commissions. A
    // stop projection without those unquoted costs is not total equity risk.
    json!({"open_risk": if read.consistent() && read.book.position.is_empty() { Some(0.0) } else { None::<f64> },
        "pending_risk": if read.consistent() && pending.is_empty() && linked_protection_ambiguous == 0 { Some(0.0) } else { None::<f64> },
        "open_stop_loss_before_unquoted_exit_costs":projection,
        "pending_stop_loss_before_unquoted_costs":pending_projection,
        "gross_leverage":notional.zip(equity).filter(|(_,e)| *e>0.0).map(|(n,e)| n/e),
        "unprotected_positions":unprotected,"unpriced_positions":unpriced,
        "unprotected_pending_orders":unprotected_pending,"unpriced_pending_orders":unpriced_pending,
        "ambiguous_protection_links":linked_protection_ambiguous,
        "positions_with_unquoted_exit_costs":read.book.position.len(),
        "pending_orders_with_unquoted_costs":pending.len()})
}

fn flow_totals(rows: &[pb::ProtoOaDepositWithdraw]) -> Option<(f64, usize, usize)> {
    let mut net = 0.0;
    let mut external = 0;
    let mut internal = 0;
    for row in rows {
        match row.operation_type {
            0 | 1 | 30..=33 | 36 | 37 => {
                let delta = money(row.delta, row.money_digits)?;
                // A contradictory signed delta is not silently re-signed.
                if (matches!(row.operation_type, 0 | 31 | 33 | 36) && delta < 0.0)
                    || (matches!(row.operation_type, 1 | 30 | 32 | 37) && delta > 0.0)
                {
                    return None;
                }
                net += delta;
                external += 1;
            }
            // Broker-defined non-external commissions, rebates, dividends,
            // swaps and guaranteed-stop charges remain part of economic P&L.
            3..=18 | 21 | 22 | 27..=29 | 34 | 35 => {
                internal += 1;
            }
            _ => return None,
        }
    }
    Some((net, external, internal))
}

async fn retain(
    pool: &PgPool,
    account: i64,
    key: &str,
    at: DateTime<Utc>,
    detail: &Value,
) -> anyhow::Result<()> {
    if key == "latest" {
        sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES($1,$2,$3,$4) ON CONFLICT(account_id,sample_key) DO UPDATE SET observed_at=EXCLUDED.observed_at,detail=EXCLUDED.detail WHERE report_account_snapshots.observed_at < EXCLUDED.observed_at")
            .bind(account).bind(key).bind(at).bind(detail).execute(pool).await?;
    } else {
        sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES($1,$2,$3,$4) ON CONFLICT(account_id,sample_key) DO NOTHING")
            .bind(account).bind(key).bind(at).bind(detail).execute(pool).await?;
    }
    Ok(())
}

/// Collect the whole reconciled account using this session. Failures are
/// isolated by the caller; freshness remains visible through observed_at.
pub async fn capture(pool: &PgPool, session: &AccountSession<'_>) -> anyhow::Result<()> {
    let account = session.account().id();
    let environment: Option<String> =
        sqlx::query_scalar("SELECT environment FROM accounts WHERE account_id=$1")
            .bind(account)
            .fetch_optional(pool)
            .await?;
    let cached = jiaoyi_store::latest_assets(pool, account).await?;
    let (assets, listed, asset_source) = if let Some((run, assets, listed)) = cached {
        (assets, listed, json!({"load_run_id":run}))
    } else {
        let assets = session.asset_list().await?;
        (
            serde_json::to_value(assets)?,
            json!([]),
            json!({"read_at":Utc::now()}),
        )
    };
    let mut sample = read(session).await?;
    if !sample.consistent() {
        sample = read(session).await?;
    }
    let currency = assets
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|r| r["asset_id"].as_i64() == Some(sample.trader.deposit_asset_id))
        })
        .and_then(|r| r["name"].as_str())
        .map(str::to_owned);
    let balance = money(sample.trader.balance, sample.trader.money_digits);
    let floating = if sample.consistent() {
        sample
            .pnl
            .position_unrealized_pn_l
            .iter()
            .try_fold(0.0, |sum, p| {
                Some(sum + money(p.net_unrealized_pn_l, Some(sample.pnl.money_digits))?)
            })
    } else {
        None
    };
    let equity = balance.zip(floating).map(|(b, p)| b + p);
    let used_margin = position_margin(&sample);
    let market = cached_market(pool, account, sample.observed, &listed).await?;
    let risk = risk_and_exposure(&sample, &market, equity);
    let relationships = extra_pnl_relationship_diagnostics(
        pool,
        account,
        &sample.book,
        &sample.pnl,
        sample.observed,
    )
    .await;
    let mut notes = vec!["Actual bounded broker observations; not an atomic historical account mark.",
        "Equity is balance plus broker net unrealized P&L; excludes unverified credit/bonuses. Broker net unrealized excludes potential closing commission.",
        "Credit is not directly supplied by these broker responses; no bonus is assumed to be cash credit.",
        "Nonempty-book total stop risk is unknown until future exit costs are verified. Supplemental price-risk projections exclude unquoted exit charges, slippage and gaps."]
        .into_iter().map(str::to_owned).collect::<Vec<_>>();
    let failure_reasons = sample.failure_reasons();
    if !sample.consistent() {
        // Existing report views truncate notes; show the actual failed checks first.
        notes.insert(0, format!("Snapshot incomplete: {}; float/equity/margin/risk withheld after two bounded attempts.", failure_reasons.join(", ")));
    }
    if !sample.book.position.is_empty() && sample.trader.total_margin_calculation_type != Some(1) {
        notes.push("Account margin is unknown: per-position margin is not summed under absent/MAX/NET broker aggregation.".into());
    }
    let mut detail = json!({"environment":environment,"currency":currency,"observed_at":sample.observed,
        "source_times":sample.source_times,"asset_source":asset_source,
        "source_complete":sample.consistent() && balance.is_some() && floating.is_some() && currency.is_some(),
        "book_stable":sample.book_stable,"balance_stable":sample.balance_stable,
        "pnl_coverage_complete":sample.pnl_coverage_complete,"failure_reasons":failure_reasons,
        "pnl_coverage_diagnostics":pnl_coverage_diagnostics(&sample.book, &sample.pnl),
        "extra_pnl_relationship_diagnostics":relationships,
        "balance":balance,"float":floating,"equity":equity,"credit":null,
        "equity_basis":"balance_plus_broker_net_unrealized_excluding_unverified_credit",
        "used_margin":used_margin,"free_margin":equity.zip(used_margin).map(|(e,m)|e-m),
        "margin_level":equity.zip(used_margin).filter(|(_,m)|*m>0.0).map(|(e,m)|100.0*e/m),
        "margin_level_basis":"cash_equity_excluding_unverified_credit",
        "margin_basis":if sample.trader.total_margin_calculation_type==Some(1) {"broker_SUM_position_margins"} else {"unverified_account_aggregation"},
        "broker_leverage":sample.trader.leverage_in_cents.map(|v|v as f64/100.0),
        "open_positions":sample.book.position.len(),"pending_orders":sample.book.order.len(),
        "notes":notes,"flow_window_from":null,"flow_window_to":null,"net_external_flow":null,
        "external_flow_count":null,"internal_flow_count":null,"cashflow_coverage_complete":false,
        "cashflow_source":"not_requested_outside_retained_close_window"});
    detail
        .as_object_mut()
        .context("snapshot object")?
        .extend(risk.as_object().context("risk object")?.clone());
    retain(pool, account, "latest", sample.observed, &detail).await?;
    retain_close(pool, session, sample.started, sample.observed, &mut detail).await
}

async fn retain_close(
    pool: &PgPool,
    session: &AccountSession<'_>,
    started: DateTime<Utc>,
    observed: DateTime<Utc>,
    detail: &mut Value,
) -> anyhow::Result<()> {
    let account = session.account().id();
    if let Some((key, cutoff)) = close_candidate(started, observed) {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM report_account_snapshots WHERE account_id=$1 AND sample_key=$2)")
            .bind(account).bind(&key).fetch_one(pool).await?;
        if exists {
            return Ok(());
        }
        detail["ny_close_cutoff"] = json!(cutoff);
        detail["close_delay_ms"] = json!(observed.signed_duration_since(cutoff).num_milliseconds());
        detail["cashflow_source"] = json!("no_previous_retained_close");
        let previous = sqlx::query("SELECT observed_at,detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key LIKE 'close:%' AND observed_at<$2 ORDER BY observed_at DESC LIMIT 1")
            .bind(account).bind(observed).fetch_optional(pool).await?;
        if let Some(previous) = previous {
            let from: DateTime<Utc> = previous.get("observed_at");
            let previous_detail: Value = previous.get("detail");
            detail["flow_window_from"] = json!(from);
            detail["flow_window_to"] = json!(observed);
            // Millisecond half-open interval (opening,closing] prevents an
            // operation at the opening mark being subtracted twice.
            if observed.signed_duration_since(from).num_milliseconds() <= 604_800_000
                && previous_detail["source_complete"] == true
                && detail["source_complete"] == true
                && previous_detail["currency"] == detail["currency"]
                && detail["currency"].as_str().is_some()
            {
                match session
                    .cash_flow_history(from.timestamp_millis() + 1, observed.timestamp_millis())
                    .await
                {
                    Ok(rows) => {
                        if let Some((net, count, internal)) = flow_totals(&rows) {
                            detail["net_external_flow"] = json!(net);
                            detail["external_flow_count"] = json!(count);
                            detail["internal_flow_count"] = json!(internal);
                            detail["cashflow_coverage_complete"] = json!(true);
                            detail["cashflow_source"] = json!("broker_cash_flow_history_actual_marks_open_exclusive_close_inclusive");
                        } else {
                            detail["cashflow_source"] =
                                json!("unverified_flow_type_digits_or_sign");
                        }
                    }
                    Err(_) => {
                        detail["cashflow_source"] = json!("broker_cash_flow_history_read_failed");
                    }
                }
            } else {
                detail["cashflow_source"] = json!("incomplete_marks_currency_or_over_seven_days");
            }
        }
        retain(pool, account, &key, observed, detail).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiaoyi_core::{
        allowlist::{Allowlist, Environment},
        secret::Secret,
    };
    use jiaoyi_ctrader::testkit::FakeBroker;

    const ACCOUNT: i64 = 49236051;

    #[test]
    fn close_marks_are_only_actual_observations_in_the_first_ninety_seconds() {
        for text in ["2026-10-27T21:00:00Z", "2026-11-02T22:00:00Z"] {
            let cutoff = text.parse::<DateTime<Utc>>().unwrap();
            assert!(close_candidate(cutoff, cutoff).is_some());
            assert!(close_candidate(cutoff, cutoff + chrono::Duration::seconds(90)).is_some());
            assert!(
                close_candidate(cutoff, cutoff + chrono::Duration::milliseconds(90_001)).is_none()
            );
            assert!(close_candidate(cutoff - chrono::Duration::milliseconds(1), cutoff).is_none());
        }
    }

    #[test]
    fn cashflows_separate_external_money_from_internal_charges_without_guessing_digits() {
        let row = |kind, delta, digits| pb::ProtoOaDepositWithdraw {
            operation_type: kind,
            delta,
            money_digits: digits,
            ..Default::default()
        };
        assert_eq!(
            flow_totals(&[
                row(0, 10000, Some(2)),
                row(1, -2500, Some(2)),
                row(4, -30, Some(2))
            ]),
            Some((75.0, 2, 1))
        );
        assert_eq!(flow_totals(&[row(0, 10000, None)]), None);
        assert_eq!(flow_totals(&[row(1, 2500, Some(2))]), None);
        assert_eq!(flow_totals(&[row(99, 0, Some(2))]), None);
        assert_eq!(money(100, None), None);
    }

    #[test]
    fn conversion_requires_fresh_unique_broker_pair_and_keeps_same_currency_exact() {
        let mut market = Market {
            assets: HashMap::from([(1, (2, 3)), (2, (4, 5))]),
            mid: HashMap::from([(1, 1.2)]),
        };
        assert_eq!(market.rate(2, 3), Some(1.2));
        assert_eq!(market.rate(3, 2), Some(1.0 / 1.2));
        assert_eq!(market.rate(2, 2), Some(1.0));
        assert!(market.rate(4, 5).is_none());
        market.assets.insert(3, (2, 3));
        market.mid.insert(3, 1.3);
        assert!(market.rate(2, 3).is_none());
    }

    #[test]
    fn pnl_requires_exact_unique_reconciled_position_coverage() {
        let mut book = pb::ProtoOaReconcileRes::default();
        book.position.push(pb::ProtoOaPosition {
            position_id: 7,
            position_status: 1,
            ..Default::default()
        });
        let mut pnl = pb::ProtoOaGetPositionUnrealizedPnLRes::default();
        assert!(!pnl_covers(&book, &pnl));
        pnl.position_unrealized_pn_l
            .push(pb::ProtoOaPositionUnrealizedPnL {
                position_id: 7,
                ..Default::default()
            });
        assert!(pnl_covers(&book, &pnl));
        pnl.position_unrealized_pn_l
            .push(pb::ProtoOaPositionUnrealizedPnL {
                position_id: 7,
                ..Default::default()
            });
        assert!(!pnl_covers(&book, &pnl));
        pnl.position_unrealized_pn_l[1].position_id = 8;
        assert!(!pnl_covers(&book, &pnl));
    }

    fn pending_zero_fixture() -> (
        pb::ProtoOaReconcileRes,
        pb::ProtoOaGetPositionUnrealizedPnLRes,
    ) {
        let book = pb::ProtoOaReconcileRes {
            ctid_trader_account_id: ACCOUNT,
            order: vec![pb::ProtoOaOrder {
                order_id: 1,
                position_id: Some(8),
                order_type: 3,
                order_status: 1,
                executed_volume: Some(0),
                stop_price: Some(10.0),
                stop_loss: Some(9.0),
                trade_data: pb::ProtoOaTradeData {
                    symbol_id: 1,
                    volume: 100,
                    trade_side: 1,
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let pnl = pb::ProtoOaGetPositionUnrealizedPnLRes {
            ctid_trader_account_id: ACCOUNT,
            money_digits: 2,
            position_unrealized_pn_l: vec![pb::ProtoOaPositionUnrealizedPnL {
                position_id: 8,
                gross_unrealized_pn_l: 0,
                net_unrealized_pn_l: 0,
            }],
            ..Default::default()
        };
        (book, pnl)
    }

    #[test]
    fn pending_zero_mark_is_covered_only_by_unique_unfilled_accepted_entry() {
        let (book, pnl) = pending_zero_fixture();
        assert!(pnl_covers(&book, &pnl));
        for field in ["net", "gross", "account", "duplicate", "unlinked"] {
            let mut bad = pnl.clone();
            match field {
                "net" => bad.position_unrealized_pn_l[0].net_unrealized_pn_l = 1,
                "gross" => bad.position_unrealized_pn_l[0].gross_unrealized_pn_l = -1,
                "account" => bad.ctid_trader_account_id += 1,
                "duplicate" => bad
                    .position_unrealized_pn_l
                    .push(bad.position_unrealized_pn_l[0]),
                "unlinked" => bad.position_unrealized_pn_l[0].position_id += 1,
                _ => unreachable!(),
            }
            assert!(!pnl_covers(&book, &bad), "accepted {field}");
        }
        for field in [
            "filled",
            "unknown_filled",
            "status",
            "protective",
            "unknown_type",
            "duplicate",
            "zero_id",
            "zero_volume",
            "missing_ref",
        ] {
            let mut bad = book.clone();
            match field {
                "filled" => bad.order[0].executed_volume = Some(1),
                "unknown_filled" => bad.order[0].executed_volume = None,
                "status" => bad.order[0].order_status = 2,
                "protective" => bad.order[0].closing_order = Some(true),
                "unknown_type" => bad.order[0].order_type = 99,
                "duplicate" => bad.order.push(bad.order[0].clone()),
                "zero_id" => bad.order[0].order_id = 0,
                "zero_volume" => bad.order[0].trade_data.volume = 0,
                "missing_ref" => bad.order[0].position_id = None,
                _ => unreachable!(),
            }
            assert!(!pnl_covers(&bad, &pnl), "accepted {field}");
        }
        let mut missing_open = book.clone();
        missing_open.position.push(pb::ProtoOaPosition {
            position_id: 7,
            position_status: 1,
            ..Default::default()
        });
        assert!(!pnl_covers(&missing_open, &pnl));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn pending_zero_capture_restores_float_without_adopting_or_zeroing_pending_risk(
        pool: PgPool,
    ) {
        use std::{
            collections::VecDeque,
            sync::{Arc, Mutex},
        };
        jiaoyi_store::upsert_account(&pool, ACCOUNT, Environment::Demo)
            .await
            .unwrap();
        let (book, pnl) = pending_zero_fixture();
        let trader = pb::ProtoOaTrader {
            ctid_trader_account_id: ACCOUNT,
            balance: 123456,
            money_digits: Some(2),
            deposit_asset_id: 1,
            total_margin_calculation_type: Some(1),
            ..Default::default()
        };
        let (conn, log) = FakeBroker {
            account_id: ACCOUNT,
            assets: vec![(1, "USD".into())],
            scripted_traders: Arc::new(Mutex::new(VecDeque::from([trader.clone(), trader]))),
            scripted_books: Arc::new(Mutex::new(VecDeque::from([book.clone(), book]))),
            scripted_pnl: Arc::new(Mutex::new(VecDeque::from([pnl]))),
            ..Default::default()
        }
        .spawn();
        conn.application_auth("client", &Secret::new("secret"))
            .await
            .unwrap();
        let account = Allowlist::parse(Some("49236051"))
            .unwrap()
            .permit(ACCOUNT)
            .unwrap();
        let session = conn
            .account_auth(&account, &Secret::new("tok"))
            .await
            .unwrap();
        capture(&pool, &session).await.unwrap();
        let detail: Value = sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='latest'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        assert_eq!(detail["source_complete"], true);
        assert_eq!(detail["failure_reasons"], json!([]));
        assert_eq!(detail["float"], 0.0);
        assert_eq!(detail["equity"], 1234.56);
        assert_eq!(
            detail["pnl_coverage_diagnostics"]["extra_pnl_position_count"],
            1
        );
        assert_eq!(
            detail["pnl_coverage_diagnostics"]["pending_zero_pnl_position_count"],
            1
        );
        assert_eq!(
            detail["pnl_coverage_diagnostics"]["unresolved_extra_pnl_position_count"],
            0
        );
        assert_eq!(detail["open_positions"], 0);
        assert_eq!(detail["pending_orders"], 1);
        assert!(detail["pending_risk"].is_null());
        assert!(detail["pending_stop_loss_before_unquoted_costs"].is_null());
        assert_eq!(detail["pending_orders_with_unquoted_costs"], 1);
        let owned: i64 =
            sqlx::query_scalar("SELECT count(*) FROM trading_intents WHERE account_id=$1")
                .bind(ACCOUNT)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(owned, 0);
        let log = log.lock().unwrap();
        assert!(log.orders.is_empty() && log.amends.is_empty() && log.closes.is_empty());
        assert_eq!(log.payload_types.iter().filter(|t| **t == pb::ProtoOaPayloadType::ProtoOaGetPositionUnrealizedPnlReq as u32).count(), 1);
    }

    async fn seed_relationship_observations(pool: &PgPool) {
        // A partial closing observation deliberately does not prove full closure.
        sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,is_closing,raw,fetched_at) VALUES ($1,1,1,8,1,100,100,'BUY','FILLED',now()-interval '1 minute',now()-interval '1 minute',false,''::bytea,now()), ($1,2,1,8,1,100,10,'SELL','PARTIALLY_FILLED',now()-interval '1 minute',now()-interval '1 minute',true,''::bytea,now()), ($2,3,1,9,1,100,100,'BUY','FILLED',now(),now(),false,''::bytea,now()), ($1,4,1,10,1,100,0,'BUY','REJECTED',now(),now(),false,''::bytea,now())")
            .bind(ACCOUNT).bind(ACCOUNT+1).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,is_closing,raw,fetched_at) VALUES ($1,5,1,12,1,100,100,'BUY','FILLED',now(),now(),false,''::bytea,now()), ($1,6,1,13,1,100,10,'SELL','PARTIALLY_FILLED',now(),now(),true,''::bytea,now()), ($1,7,1,14,1,100,100,'BUY','FILLED',now(),now()+interval '1 day',false,''::bytea,now())")
            .bind(ACCOUNT).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,max_risk) VALUES($1,'relation-owned','test','open',1),($2,'relation-foreign','test','open',1)")
            .bind(ACCOUNT).bind(ACCOUNT+1).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,volume,raw,received_at) VALUES ($1,'relation-owned',1,'ORDER_PARTIAL_FILL',8,10,''::bytea,now()), ($2,'relation-foreign',1,'ORDER_FILLED',9,100,''::bytea,now()), ($1,'relation-foreign',1,'ORDER_FILLED',9,100,''::bytea,now())")
            .bind(ACCOUNT).bind(ACCOUNT+1).execute(pool).await.unwrap();
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn extra_relationships_preserve_unknowns_and_account_scope(pool: PgPool) {
        seed_relationship_observations(&pool).await;
        let mut book = pb::ProtoOaReconcileRes {
            ctid_trader_account_id: ACCOUNT,
            ..Default::default()
        };
        let mut pnl = pb::ProtoOaGetPositionUnrealizedPnLRes {
            ctid_trader_account_id: ACCOUNT,
            position_unrealized_pn_l: [8, 9, 10, 11]
                .into_iter()
                .map(|position_id| pb::ProtoOaPositionUnrealizedPnL {
                    position_id,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let result =
            extra_pnl_relationship_diagnostics(&pool, ACCOUNT, &book, &pnl, Utc::now()).await;
        for field in [
            "opening_deal_observed_count",
            "closing_deal_observed_count",
            "owned_fill_observed_count",
        ] {
            assert_eq!(
                result[field], 1,
                "{field}: foreign/rejected/unobserved rows must not qualify"
            );
        }
        assert_eq!(
            result["evidence_state"],
            "stored_observations_only_closure_unknown"
        );
        for (ids, expected) in [
            (vec![8, 8], [1, 1, 1]),
            (vec![12], [1, 0, 0]),
            (vec![13], [0, 1, 0]),
            (vec![9, 10, 11, 14], [0, 0, 0]),
        ] {
            let mut selected = pnl.clone();
            selected.position_unrealized_pn_l = ids
                .into_iter()
                .map(|position_id| pb::ProtoOaPositionUnrealizedPnL {
                    position_id,
                    ..Default::default()
                })
                .collect();
            let result =
                extra_pnl_relationship_diagnostics(&pool, ACCOUNT, &book, &selected, Utc::now())
                    .await;
            for (field, expected) in [
                "opening_deal_observed_count",
                "closing_deal_observed_count",
                "owned_fill_observed_count",
            ]
            .into_iter()
            .zip(expected)
            {
                assert_eq!(result[field], expected, "independent observation: {field}");
            }
        }
        book.ctid_trader_account_id += 1;
        let result =
            extra_pnl_relationship_diagnostics(&pool, ACCOUNT, &book, &pnl, Utc::now()).await;
        assert_eq!(result["evidence_state"], "response_account_mismatch");
        assert!(result["opening_deal_observed_count"].is_null());
        assert!(result["pending_entry_position_match_count"].is_null());
        book.ctid_trader_account_id = ACCOUNT;
        pnl.ctid_trader_account_id += 1;
        assert_eq!(
            extra_pnl_relationship_diagnostics(&pool, ACCOUNT, &book, &pnl, Utc::now()).await
                ["evidence_state"],
            "response_account_mismatch"
        );
        pnl.ctid_trader_account_id = ACCOUNT;
        pnl.position_unrealized_pn_l[0].position_id = 0;
        let result =
            extra_pnl_relationship_diagnostics(&pool, ACCOUNT, &book, &pnl, Utc::now()).await;
        assert_eq!(result["evidence_state"], "invalid_position_id");
        assert!(result["owned_fill_observed_count"].is_null());
        pnl.position_unrealized_pn_l[0].position_id = 8;
        let unavailable = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://localhost/unavailable")
            .unwrap();
        unavailable.close().await;
        let result =
            extra_pnl_relationship_diagnostics(&unavailable, ACCOUNT, &book, &pnl, Utc::now())
                .await;
        assert_eq!(result["evidence_state"], "store_read_failed");
        assert!(result["closing_deal_observed_count"].is_null());
        assert_eq!(result["pending_entry_position_match_count"], 0);
    }

    // Exercise real collector reads and persisted JSON, not just predicates.
    async fn diagnostic_fixture(pool: PgPool, failure: Option<&str>, empty: bool) {
        use std::{
            collections::VecDeque,
            sync::{Arc, Mutex},
        };
        jiaoyi_store::upsert_account(&pool, ACCOUNT, Environment::Demo)
            .await
            .unwrap();
        let trader = pb::ProtoOaTrader {
            ctid_trader_account_id: ACCOUNT,
            balance: 1234567,
            money_digits: Some(3),
            deposit_asset_id: 1,
            total_margin_calculation_type: Some(1),
            ..Default::default()
        };
        let mut before_trader = trader.clone();
        if failure == Some("balance_changed") {
            before_trader.balance_version = Some(1);
        }
        let mut book = pb::ProtoOaReconcileRes {
            ctid_trader_account_id: ACCOUNT,
            position: if empty {
                vec![]
            } else {
                vec![pb::ProtoOaPosition {
                    position_id: 7,
                    position_status: 1,
                    money_digits: Some(3),
                    used_margin: Some(1000),
                    ..Default::default()
                }]
            },
            ..Default::default()
        };
        if failure == Some("duplicate_position_id") {
            book.position.push(book.position[0].clone());
        }
        if failure == Some("non_open_position") {
            book.position[0].position_status = 2;
        }
        if failure == Some("extra_pnl_position") {
            // Already-held pre-orders and protection: observe links, never amend.
            book.order = vec![
                pb::ProtoOaOrder {
                    position_id: Some(8),
                    order_type: 3,
                    relative_stop_loss: Some(100),
                    relative_take_profit: Some(200),
                    ..Default::default()
                },
                pb::ProtoOaOrder {
                    position_id: Some(8),
                    order_type: 4,
                    closing_order: Some(true),
                    stop_loss: Some(1.0),
                    take_profit: Some(2.0),
                    ..Default::default()
                },
                pb::ProtoOaOrder {
                    order_type: 2,
                    stop_loss: Some(1.0),
                    ..Default::default()
                },
            ];
            seed_relationship_observations(&pool).await;
        }
        let mut before_book = book.clone();
        if failure == Some("book_changed") {
            before_book.position[0].stop_loss = Some(1.0);
        }
        let mut pnl = pb::ProtoOaGetPositionUnrealizedPnLRes {
            ctid_trader_account_id: ACCOUNT,
            money_digits: 3,
            position_unrealized_pn_l: if empty {
                vec![]
            } else {
                vec![pb::ProtoOaPositionUnrealizedPnL {
                    position_id: if failure == Some("pnl_position_coverage_incomplete") {
                        8
                    } else {
                        7
                    },
                    gross_unrealized_pn_l: 2345,
                    net_unrealized_pn_l: 1234,
                }]
            },
            ..Default::default()
        };
        match failure {
            Some("missing_pnl_position") => pnl.position_unrealized_pn_l.clear(),
            Some("extra_pnl_position") => {
                let mut extra = pnl.position_unrealized_pn_l[0];
                extra.position_id = 8;
                pnl.position_unrealized_pn_l.push(extra);
            }
            Some("duplicate_pnl_id") => pnl
                .position_unrealized_pn_l
                .push(pnl.position_unrealized_pn_l[0]),
            _ => {}
        }
        let coverage_failure =
            failure.is_some_and(|f| !matches!(f, "book_changed" | "balance_changed"));
        let reason = if coverage_failure {
            Some("pnl_position_coverage_incomplete")
        } else {
            failure
        };
        let expected_counts = match failure {
            Some("pnl_position_coverage_incomplete") => [1, 1, 0, 0, 0],
            Some("missing_pnl_position") => [1, 0, 0, 0, 0],
            Some("extra_pnl_position") => [0, 1, 0, 0, 0],
            Some("duplicate_position_id") => [0, 0, 1, 0, 0],
            Some("duplicate_pnl_id") => [0, 0, 0, 1, 0],
            Some("non_open_position") => [0, 0, 0, 0, 1],
            _ => [0; 5],
        };
        let (conn, log) = FakeBroker {
            account_id: ACCOUNT,
            assets: vec![(1, "USD".into())],
            scripted_traders: Arc::new(Mutex::new(VecDeque::from([
                before_trader.clone(),
                trader.clone(),
                before_trader,
                trader,
            ]))),
            scripted_books: Arc::new(Mutex::new(VecDeque::from([
                before_book.clone(),
                book.clone(),
                before_book,
                book,
            ]))),
            scripted_pnl: Arc::new(Mutex::new(VecDeque::from([pnl.clone(), pnl]))),
            ..Default::default()
        }
        .spawn();
        conn.application_auth("client", &Secret::new("secret"))
            .await
            .unwrap();
        let account = Allowlist::parse(Some("49236051"))
            .unwrap()
            .permit(ACCOUNT)
            .unwrap();
        let session = conn
            .account_auth(&account, &Secret::new("tok"))
            .await
            .unwrap();
        capture(&pool, &session).await.unwrap();
        let detail: Value = sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='latest'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        // This assertion reproduces the original mislabel for balance/book only.
        assert_eq!(detail["pnl_coverage_complete"], !coverage_failure);
        assert_eq!(detail["book_stable"], failure != Some("book_changed"));
        assert_eq!(detail["balance_stable"], failure != Some("balance_changed"));
        assert_eq!(
            detail["failure_reasons"],
            json!(reason.into_iter().collect::<Vec<_>>())
        );
        assert_eq!(
            detail["pnl_coverage_diagnostics"],
            json!({
                "missing_pnl_position_count": expected_counts[0],
                "extra_pnl_position_count": expected_counts[1],
                "pending_zero_pnl_position_count": 0,
                "unresolved_extra_pnl_position_count": expected_counts[1],
                "duplicate_position_id_count": expected_counts[2],
                "duplicate_pnl_id_count": expected_counts[3],
                "non_open_position_count": expected_counts[4],
            })
        );
        assert_eq!(
            detail["extra_pnl_relationship_diagnostics"]["evidence_state"],
            if expected_counts[1] > 0 {
                "stored_observations_only_closure_unknown"
            } else {
                "not_applicable"
            }
        );
        if failure == Some("extra_pnl_position") {
            assert_eq!(
                detail["extra_pnl_relationship_diagnostics"],
                json!({
                "evidence_state":"stored_observations_only_closure_unknown",
                "pending_entry_position_match_count":1,"pending_protection_position_match_count":1,
                "pending_order_missing_position_ref_count":1,"opening_deal_observed_count":1,
                "closing_deal_observed_count":1,"owned_fill_observed_count":1})
            );
        }
        assert_eq!(detail["source_complete"], failure.is_none());
        assert_eq!(detail["balance"], 1234.567);
        if let Some(reason) = reason {
            for field in [
                "float",
                "equity",
                "used_margin",
                "free_margin",
                "margin_level",
                "open_risk",
                "pending_risk",
                "gross_leverage",
                "open_stop_loss_before_unquoted_exit_costs",
                "pending_stop_loss_before_unquoted_costs",
            ] {
                assert!(
                    detail[field].is_null(),
                    "{field} must be withheld for {reason}"
                );
            }
            assert!(detail["notes"][0].as_str().unwrap().contains(reason));
            // Use the actual collected notes: the existing view truncates them.
            let period =
                crate::telegram_report::period_due("2026-10-26T21:15:00Z".parse().unwrap());
            let html = crate::trading_report_view::render(
                &period,
                &[],
                &[
                    json!({"account_id":ACCOUNT,"environment":"demo","currency":"USD","snapshot":detail}),
                ],
                &json!({"reports":[]}),
            );
            assert!(html.contains(&format!("Snapshot incomplete: {reason}")));
            assert!(html.contains(&format!("Source note: Snapshot incomplete: {reason}")));
        } else {
            assert_eq!(detail["float"], if empty { 0.0 } else { 1.234 });
            assert_eq!(detail["equity"], 1234.567 + if empty { 0.0 } else { 1.234 });
        }
        let log = log.lock().unwrap();
        let attempts = if failure.is_some() { 2 } else { 1 };
        for (payload, per_attempt) in [
            (pb::ProtoOaPayloadType::ProtoOaTraderReq, 2),
            (pb::ProtoOaPayloadType::ProtoOaReconcileReq, 2),
            (
                pb::ProtoOaPayloadType::ProtoOaGetPositionUnrealizedPnlReq,
                1,
            ),
        ] {
            assert_eq!(
                log.payload_types
                    .iter()
                    .filter(|t| **t == payload as u32)
                    .count(),
                attempts * per_attempt
            );
        }
        assert_eq!(log.account_auths, vec![ACCOUNT]);
        assert!(log.orders.is_empty() && log.amends.is_empty() && log.closes.is_empty());
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn balance_only_mismatch_preserves_actual_pnl_coverage(pool: PgPool) {
        diagnostic_fixture(pool, Some("balance_changed"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn book_only_mismatch_preserves_actual_pnl_coverage(pool: PgPool) {
        diagnostic_fixture(pool, Some("book_changed"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn pnl_id_mismatch_is_reported_independently(pool: PgPool) {
        diagnostic_fixture(pool, Some("pnl_position_coverage_incomplete"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn missing_pnl_position_has_fixed_diagnostics_and_withholds_valuations(pool: PgPool) {
        diagnostic_fixture(pool, Some("missing_pnl_position"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn extra_pnl_position_has_fixed_diagnostics_and_withholds_valuations(pool: PgPool) {
        diagnostic_fixture(pool, Some("extra_pnl_position"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn duplicate_position_id_has_fixed_diagnostics_and_withholds_valuations(pool: PgPool) {
        diagnostic_fixture(pool, Some("duplicate_position_id"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn duplicate_pnl_id_has_fixed_diagnostics_and_withholds_valuations(pool: PgPool) {
        diagnostic_fixture(pool, Some("duplicate_pnl_id"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn non_open_position_has_fixed_diagnostics_and_withholds_valuations(pool: PgPool) {
        diagnostic_fixture(pool, Some("non_open_position"), false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn complete_nonempty_book_preserves_money_precision_and_one_attempt(pool: PgPool) {
        diagnostic_fixture(pool, None, false).await;
    }
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn complete_empty_book_has_no_diagnostic_failures(pool: PgPool) {
        diagnostic_fixture(pool, None, true).await;
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn immutable_close_and_monotonic_latest_keep_original_source_marks(pool: PgPool) {
        let cutoff: DateTime<Utc> = "2026-10-27T21:00:00Z".parse().unwrap();
        let first = cutoff + chrono::Duration::seconds(12);
        let second = cutoff + chrono::Duration::seconds(72);
        let key = close_candidate(cutoff, first).unwrap().0;
        retain(
            &pool,
            ACCOUNT,
            &key,
            first,
            &json!({"equity":100.0,"observed_at":first}),
        )
        .await
        .unwrap();
        retain(
            &pool,
            ACCOUNT,
            &key,
            second,
            &json!({"equity":900.0,"observed_at":second}),
        )
        .await
        .unwrap();
        retain(&pool, ACCOUNT, "latest", second, &json!({"equity":900.0}))
            .await
            .unwrap();
        retain(&pool, ACCOUNT, "latest", first, &json!({"equity":100.0}))
            .await
            .unwrap();
        let row = sqlx::query("SELECT observed_at,detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key=$2")
            .bind(ACCOUNT).bind(key).fetch_one(&pool).await.unwrap();
        assert_eq!(row.get::<DateTime<Utc>, _>("observed_at"), first);
        assert_eq!(row.get::<Value, _>("detail")["equity"], 100.0);
        let latest: Value = sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='latest'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        assert_eq!(latest["equity"], 900.0);
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn collector_captures_empty_account_and_cashflow_reads_without_trades_or_extra_auth(
        pool: PgPool,
    ) {
        jiaoyi_store::upsert_account(&pool, ACCOUNT, Environment::Demo)
            .await
            .unwrap();
        let now = Utc::now().timestamp_millis();
        let fake = FakeBroker {
            account_id: ACCOUNT,
            assets: vec![(1, "USD".into())],
            trader: Some(pb::ProtoOaTrader {
                ctid_trader_account_id: ACCOUNT,
                balance: 123456,
                deposit_asset_id: 1,
                money_digits: Some(2),
                leverage_in_cents: Some(5000),
                ..Default::default()
            }),
            cash_flows: vec![pb::ProtoOaDepositWithdraw {
                balance_history_id: 1,
                operation_type: 0,
                delta: 5000,
                money_digits: Some(2),
                change_balance_timestamp: now - 1000,
                ..Default::default()
            }],
            ..Default::default()
        };
        let (conn, log) = fake.spawn();
        conn.application_auth("client", &Secret::new("secret"))
            .await
            .unwrap();
        let account = Allowlist::parse(Some("49236051"))
            .unwrap()
            .permit(ACCOUNT)
            .unwrap();
        let session = conn
            .account_auth(&account, &Secret::new("tok"))
            .await
            .unwrap();
        capture(&pool, &session).await.unwrap();
        let detail: Value = sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='latest'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        assert_eq!(detail["currency"], "USD");
        assert_eq!(detail["balance"], 1234.56);
        assert_eq!(detail["equity"], 1234.56);
        assert_eq!(detail["float"], 0.0);
        assert_eq!(detail["used_margin"], 0.0);
        assert_eq!(detail["open_risk"], 0.0);
        assert_eq!(detail["pending_risk"], 0.0);
        assert_eq!(detail["source_complete"], true);
        assert!(detail["credit"].is_null());
        assert_eq!(detail["cashflow_coverage_complete"], false);
        let flows = session.cash_flow_history(now - 2000, now).await.unwrap();
        assert_eq!(flow_totals(&flows), Some((50.0, 1, 0)));
        let before = log.lock().unwrap().payload_types.len();
        assert!(session.cash_flow_history(now, now - 1).await.is_err());
        assert!(session.cash_flow_history(0, 604_800_001).await.is_err());
        assert_eq!(
            log.lock().unwrap().payload_types.len(),
            before,
            "invalid windows never reach broker"
        );
        let log = log.lock().unwrap();
        assert_eq!(log.account_auths, vec![ACCOUNT]);
        assert!(log.orders.is_empty() && log.amends.is_empty() && log.closes.is_empty());
        assert!(!log
            .payload_types
            .contains(&(pb::ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as u32)));
        assert!(log
            .payload_types
            .iter()
            .all(|t| jiaoyi_ctrader::check_sendable(*t).is_ok()));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn close_retention_reads_exact_actual_mark_flow_window_once_and_stores_only_totals(
        pool: PgPool,
    ) {
        // Controlled receipt timestamps exercise the production close path,
        // including the real client's broker-window filtering and rate limit.
        let opening: DateTime<Utc> = "2026-10-26T21:00:12Z".parse().unwrap();
        let cutoff: DateTime<Utc> = "2026-10-27T21:00:00Z".parse().unwrap();
        let closing = cutoff + chrono::Duration::seconds(30);
        let mut detail = json!({"source_complete":true,"currency":"USD","equity":1200.0,
            "observed_at":closing,"net_external_flow":null,"cashflow_coverage_complete":false});
        retain(
            &pool,
            ACCOUNT,
            "close:2026-10-26",
            opening,
            &json!({"source_complete":true,"currency":"USD","equity":1000.0}),
        )
        .await
        .unwrap();
        let flow = |id, kind, delta, at: DateTime<Utc>| pb::ProtoOaDepositWithdraw {
            balance_history_id: id,
            operation_type: kind,
            delta,
            money_digits: Some(2),
            change_balance_timestamp: at.timestamp_millis(),
            ..Default::default()
        };
        let (conn, log) = FakeBroker {
            account_id: ACCOUNT,
            cash_flows: vec![
                flow(1, 0, 99999, opening), // opening mark is exclusive
                flow(2, 0, 10000, opening + chrono::Duration::seconds(1)),
                flow(3, 1, -2500, closing), // closing mark is inclusive
                flow(4, 4, -100, closing - chrono::Duration::seconds(1)),
                flow(5, 0, 99999, closing + chrono::Duration::milliseconds(1)),
            ],
            ..Default::default()
        }
        .spawn();
        conn.application_auth("client", &Secret::new("secret"))
            .await
            .unwrap();
        let account = Allowlist::parse(Some("49236051"))
            .unwrap()
            .permit(ACCOUNT)
            .unwrap();
        let session = conn
            .account_auth(&account, &Secret::new("tok"))
            .await
            .unwrap();
        retain_close(&pool, &session, cutoff, closing, &mut detail)
            .await
            .unwrap();
        let stored:Value=sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='close:2026-10-27'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        assert_eq!(stored["flow_window_from"], json!(opening));
        assert_eq!(stored["flow_window_to"], json!(closing));
        assert_eq!(stored["net_external_flow"], 75.0);
        assert_eq!(stored["external_flow_count"], 2);
        assert_eq!(stored["internal_flow_count"], 1);
        assert_eq!(stored["cashflow_coverage_complete"], true);
        assert!(stored.get("deposit_withdraw").is_none());
        detail["equity"] = json!(9000.0);
        retain_close(
            &pool,
            &session,
            cutoff,
            closing + chrono::Duration::seconds(30),
            &mut detail,
        )
        .await
        .unwrap();
        let stored:Value=sqlx::query_scalar("SELECT detail FROM report_account_snapshots WHERE account_id=$1 AND sample_key='close:2026-10-27'")
            .bind(ACCOUNT).fetch_one(&pool).await.unwrap();
        assert_eq!(stored["equity"], 1200.0);
        let request = pb::ProtoOaPayloadType::ProtoOaCashFlowHistoryListReq as u32;
        assert_eq!(
            log.lock()
                .unwrap()
                .payload_types
                .iter()
                .filter(|t| **t == request)
                .count(),
            1
        );
    }

    #[test]
    fn stop_projection_uses_current_broker_pnl_and_does_not_claim_unknown_exit_costs_or_net_margin()
    {
        let mut book = pb::ProtoOaReconcileRes::default();
        book.position.push(pb::ProtoOaPosition {
            position_id: 7,
            position_status: 1,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: 10,
                volume: 100_000,
                trade_side: 1,
                ..Default::default()
            },
            price: Some(1.10),
            stop_loss: Some(1.09),
            used_margin: Some(10000),
            money_digits: Some(2),
            ..Default::default()
        });
        let pnl = pb::ProtoOaGetPositionUnrealizedPnLRes {
            money_digits: 2,
            position_unrealized_pn_l: vec![pb::ProtoOaPositionUnrealizedPnL {
                position_id: 7,
                gross_unrealized_pn_l: 5000,
                net_unrealized_pn_l: 4800,
            }],
            ..Default::default()
        };
        let mut sample = Read {
            trader: pb::ProtoOaTrader {
                deposit_asset_id: 1,
                total_margin_calculation_type: Some(2),
                ..Default::default()
            },
            book,
            pnl,
            started: Utc::now(),
            observed: Utc::now(),
            book_stable: true,
            balance_stable: true,
            pnl_coverage_complete: true,
            source_times: json!({}),
        };
        let market = Market {
            assets: HashMap::from([(10, (2, 1))]),
            ..Default::default()
        };
        let risk = risk_and_exposure(&sample, &market, Some(1000.0));
        // Marked profit50 plus entry-to-stop loss10 is further loss60.
        assert!(
            (risk["open_stop_loss_before_unquoted_exit_costs"]
                .as_f64()
                .unwrap()
                - 60.0)
                .abs()
                < 1e-9
        );
        assert!(risk["open_risk"].is_null());
        assert!((risk["gross_leverage"].as_f64().unwrap() - 1.15).abs() < 1e-9);
        assert!(
            position_margin(&sample).is_none(),
            "NET must not become a sum"
        );
        sample.trader.total_margin_calculation_type = Some(1);
        assert_eq!(position_margin(&sample), Some(100.0));
        sample.book.position[0].money_digits = None;
        assert!(position_margin(&sample).is_none());
        sample.book.position[0].stop_loss = None;
        assert_eq!(
            risk_and_exposure(&sample, &market, Some(1000.0))["unprotected_positions"],
            1
        );
    }
}
