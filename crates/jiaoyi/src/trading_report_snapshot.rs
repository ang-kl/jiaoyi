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
    positions.len() == book.position.len()
        && marks.len() == pnl.position_unrealized_pn_l.len()
        && positions == marks
        && book.position.iter().all(|p| p.position_status == 1)
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
    consistent: bool,
    source_times: Value,
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
    let consistent =
        same_book(&book_before, &book) && same_balance(&before, &trader) && pnl_covers(&book, &pnl);
    Ok(Read {
        trader,
        book,
        pnl,
        started,
        observed,
        consistent,
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
    if !read.consistent {
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
    if !read.consistent || linked_protection_ambiguous > 0 {
        projection = None;
        pending_projection = None;
        notional = None;
    }
    // The API net UPL explicitly excludes potential closing commissions. A
    // stop projection without those unquoted costs is not total equity risk.
    json!({"open_risk": if read.consistent && read.book.position.is_empty() { Some(0.0) } else { None::<f64> },
        "pending_risk": if read.consistent && pending.is_empty() && linked_protection_ambiguous == 0 { Some(0.0) } else { None::<f64> },
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
    if !sample.consistent {
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
    let floating = if sample.consistent {
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
    let mut notes = vec!["Actual bounded broker observations; not an atomic historical account mark.",
        "Equity is balance plus broker net unrealized P&L; excludes unverified credit/bonuses. Broker net unrealized excludes potential closing commission.",
        "Credit is not directly supplied by these broker responses; no bonus is assumed to be cash credit.",
        "Nonempty-book total stop risk is unknown until future exit costs are verified. Supplemental price-risk projections exclude unquoted exit charges, slippage and gaps."];
    if !sample.consistent {
        notes.push("Two bounded attempts could not establish matching book/P&L/balance observations; float/equity/margin/risk are incomplete.");
    }
    if !sample.book.position.is_empty() && sample.trader.total_margin_calculation_type != Some(1) {
        notes.push("Account margin is unknown: per-position margin is not summed under absent/MAX/NET broker aggregation.");
    }
    let mut detail = json!({"environment":environment,"currency":currency,"observed_at":sample.observed,
        "source_times":sample.source_times,"asset_source":asset_source,
        "source_complete":sample.consistent && balance.is_some() && floating.is_some() && currency.is_some(),
        "pnl_coverage_complete":sample.consistent,
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
            consistent: true,
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
