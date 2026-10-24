//! Storage for the broker's history: historical ticks, deals, account
//! snapshots and the research trial log (migration 0004).

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::record_reconcile;

/// One historical tick to store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryTick {
    pub ts: DateTime<Utc>,
    pub price_e5: i64,
    pub dup: i32,
}

/// What one tick window request returned.
#[derive(Debug, Clone, PartialEq)]
pub struct FetchRecord {
    pub account_id: i64,
    pub broker_symbol_id: i64,
    pub side: &'static str,
    pub kind: &'static str,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub complete: bool,
    pub ticks: i32,
    pub pages: i32,
    pub first_tick: Option<DateTime<Utc>>,
    pub last_tick: Option<DateTime<Utc>>,
    pub delta_signs: Vec<String>,
    pub error: Option<String>,
}

/// Create the monthly partitions spanning `from..=to`.
pub async fn ensure_partitions(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> crate::Result<()> {
    let mut m = NaiveDate::from_ymd_opt(from.year(), from.month(), 1).unwrap();
    let end = NaiveDate::from_ymd_opt(to.year(), to.month(), 1).unwrap();
    while m <= end {
        sqlx::query("SELECT jiaoyi_ensure_history_partition($1)")
            .bind(m)
            .execute(pool)
            .await?;
        m = if m.month() == 12 {
            NaiveDate::from_ymd_opt(m.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(m.year(), m.month() + 1, 1).unwrap()
        };
    }
    Ok(())
}

/// Store one window: the ticks (download only) and its fetch record, in ONE
/// transaction, so a window is either fully recorded or not at all. Ticks
/// already stored (same key) are skipped. Returns (fetch id, ticks newly stored).
pub async fn record_window(
    pool: &PgPool,
    f: &FetchRecord,
    ticks: &[HistoryTick],
) -> crate::Result<(i64, i32)> {
    if let (Some(a), Some(b)) = (ticks.first(), ticks.last()) {
        ensure_partitions(pool, a.ts, b.ts).await?;
    }
    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar("SELECT nextval('history_fetches_id_seq')")
        .fetch_one(&mut *tx)
        .await?;
    let mut stored = 0i64;
    for part in ticks.chunks(5000) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO ticks_history (account_id, broker_symbol_id, side, ts, price_e5, dup, fetch_id) ",
        );
        qb.push_values(part, |mut b, t| {
            b.push_bind(f.account_id)
                .push_bind(f.broker_symbol_id)
                .push_bind(f.side)
                .push_bind(t.ts)
                .push_bind(t.price_e5)
                .push_bind(t.dup)
                .push_bind(id);
        });
        qb.push(" ON CONFLICT (account_id, broker_symbol_id, side, ts, price_e5, dup) DO NOTHING");
        stored += qb.build().execute(&mut *tx).await?.rows_affected() as i64;
    }
    sqlx::query(
        "INSERT INTO history_fetches (id, account_id, broker_symbol_id, side, kind, window_from, window_to,
            status, complete, ticks, stored, pages, first_tick, last_tick, delta_signs, error)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)",
    )
    .bind(id)
    .bind(f.account_id)
    .bind(f.broker_symbol_id)
    .bind(f.side)
    .bind(f.kind)
    .bind(f.window_from)
    .bind(f.window_to)
    .bind(if f.error.is_some() { "failed" } else { "ok" })
    .bind(f.complete)
    .bind(f.ticks)
    .bind(stored as i32)
    .bind(f.pages)
    .bind(f.first_tick)
    .bind(f.last_tick)
    .bind(&f.delta_signs)
    .bind(&f.error)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((id, stored as i32))
}

/// Window starts already fetched successfully and completely.
pub async fn complete_windows(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    side: &str,
    kind: &str,
) -> crate::Result<HashSet<DateTime<Utc>>> {
    let rows: Vec<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT window_from FROM history_fetches
         WHERE account_id = $1 AND broker_symbol_id = $2 AND side = $3 AND kind = $4
           AND status = 'ok' AND complete",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(side)
    .bind(kind)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

/// The latest probe of a symbol's bid history, if any.
pub async fn last_probe_at(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
) -> crate::Result<Option<DateTime<Utc>>> {
    sqlx::query_scalar(
        "SELECT max(fetched_at) FROM history_fetches
         WHERE account_id = $1 AND broker_symbol_id = $2 AND kind = 'probe'",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

// ------------------------------------------------------------------ deals

/// One deal, field for field as `ProtoOADeal` gives it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DealRow {
    pub deal_id: i64,
    pub order_id: i64,
    pub position_id: i64,
    pub broker_symbol_id: i64,
    pub volume: i64,
    pub filled_volume: i64,
    pub trade_side: String,
    pub deal_status: String,
    pub created_at: DateTime<Utc>,
    pub executed_at: DateTime<Utc>,
    pub last_update_at: Option<DateTime<Utc>>,
    pub execution_price: Option<f64>,
    pub margin_rate: Option<f64>,
    pub commission: Option<i64>,
    pub money_digits: Option<i32>,
    pub base_to_usd_rate: Option<f64>,
    pub is_closing: bool,
    pub close_entry_price: Option<f64>,
    pub close_gross_profit: Option<i64>,
    pub close_swap: Option<i64>,
    pub close_commission: Option<i64>,
    pub close_balance: Option<i64>,
    pub close_quote_to_deposit_rate: Option<f64>,
    pub close_closed_volume: Option<i64>,
    pub close_balance_version: Option<i64>,
    pub close_money_digits: Option<i32>,
    pub close_pnl_conversion_fee: Option<i64>,
    #[serde(skip)]
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DealUpsert {
    pub inserted: i32,
    pub changed: i32,
    pub unchanged: i32,
}

/// Store deals. A deal already stored with different bytes is recorded in
/// `reconcile_events` (old and new) and replaced: the broker is the authority.
pub async fn upsert_deals(
    pool: &PgPool,
    account_id: i64,
    deals: &[DealRow],
    fetched_at: DateTime<Utc>,
) -> crate::Result<DealUpsert> {
    let mut out = DealUpsert::default();
    if deals.is_empty() {
        return Ok(out);
    }
    let mut tx = pool.begin().await?;
    let ids: Vec<i64> = deals.iter().map(|d| d.deal_id).collect();
    let existing: HashMap<i64, Vec<u8>> =
        sqlx::query("SELECT deal_id, raw FROM deals WHERE account_id = $1 AND deal_id = ANY($2)")
            .bind(account_id)
            .bind(&ids)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| (r.get("deal_id"), r.get("raw")))
            .collect();
    for d in deals {
        match existing.get(&d.deal_id) {
            Some(raw) if *raw == d.raw => {
                out.unchanged += 1;
                continue;
            }
            Some(raw) => {
                record_reconcile(
                    &mut *tx,
                    Some(account_id),
                    "deal_changed",
                    &format!("deal {}", d.deal_id),
                    json!({ "stored_raw_hex": hex(raw), "broker": d, "broker_raw_hex": hex(&d.raw) }),
                    "broker value stored",
                )
                .await?;
                out.changed += 1;
            }
            None => out.inserted += 1,
        }
        sqlx::query(
            "INSERT INTO deals (account_id, deal_id, order_id, position_id, broker_symbol_id, volume,
                filled_volume, trade_side, deal_status, created_at, executed_at, last_update_at,
                execution_price, margin_rate, commission, money_digits, base_to_usd_rate, is_closing,
                close_entry_price, close_gross_profit, close_swap, close_commission, close_balance,
                close_quote_to_deposit_rate, close_closed_volume, close_balance_version,
                close_money_digits, close_pnl_conversion_fee, raw, fetched_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,
                     $23,$24,$25,$26,$27,$28,$29,$30)
             ON CONFLICT (account_id, deal_id) DO UPDATE SET
                order_id = EXCLUDED.order_id, position_id = EXCLUDED.position_id,
                broker_symbol_id = EXCLUDED.broker_symbol_id, volume = EXCLUDED.volume,
                filled_volume = EXCLUDED.filled_volume, trade_side = EXCLUDED.trade_side,
                deal_status = EXCLUDED.deal_status, created_at = EXCLUDED.created_at,
                executed_at = EXCLUDED.executed_at, last_update_at = EXCLUDED.last_update_at,
                execution_price = EXCLUDED.execution_price, margin_rate = EXCLUDED.margin_rate,
                commission = EXCLUDED.commission, money_digits = EXCLUDED.money_digits,
                base_to_usd_rate = EXCLUDED.base_to_usd_rate, is_closing = EXCLUDED.is_closing,
                close_entry_price = EXCLUDED.close_entry_price,
                close_gross_profit = EXCLUDED.close_gross_profit, close_swap = EXCLUDED.close_swap,
                close_commission = EXCLUDED.close_commission, close_balance = EXCLUDED.close_balance,
                close_quote_to_deposit_rate = EXCLUDED.close_quote_to_deposit_rate,
                close_closed_volume = EXCLUDED.close_closed_volume,
                close_balance_version = EXCLUDED.close_balance_version,
                close_money_digits = EXCLUDED.close_money_digits,
                close_pnl_conversion_fee = EXCLUDED.close_pnl_conversion_fee,
                raw = EXCLUDED.raw, fetched_at = EXCLUDED.fetched_at",
        )
        .bind(account_id)
        .bind(d.deal_id)
        .bind(d.order_id)
        .bind(d.position_id)
        .bind(d.broker_symbol_id)
        .bind(d.volume)
        .bind(d.filled_volume)
        .bind(&d.trade_side)
        .bind(&d.deal_status)
        .bind(d.created_at)
        .bind(d.executed_at)
        .bind(d.last_update_at)
        .bind(d.execution_price)
        .bind(d.margin_rate)
        .bind(d.commission)
        .bind(d.money_digits)
        .bind(d.base_to_usd_rate)
        .bind(d.is_closing)
        .bind(d.close_entry_price)
        .bind(d.close_gross_profit)
        .bind(d.close_swap)
        .bind(d.close_commission)
        .bind(d.close_balance)
        .bind(d.close_quote_to_deposit_rate)
        .bind(d.close_closed_volume)
        .bind(d.close_balance_version)
        .bind(d.close_money_digits)
        .bind(d.close_pnl_conversion_fee)
        .bind(&d.raw)
        .bind(fetched_at)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(out)
}

pub struct DealFetchRecord {
    pub account_id: i64,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub complete: bool,
    pub deals: i32,
    pub upsert: DealUpsert,
    pub error: Option<String>,
}

pub async fn record_deal_fetch(pool: &PgPool, f: &DealFetchRecord) -> crate::Result<()> {
    sqlx::query(
        "INSERT INTO deal_fetches (account_id, window_from, window_to, status, complete, deals, inserted, changed, error)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(f.account_id)
    .bind(f.window_from)
    .bind(f.window_to)
    .bind(if f.error.is_some() { "failed" } else { "ok" })
    .bind(f.complete)
    .bind(f.deals)
    .bind(f.upsert.inserted)
    .bind(f.upsert.changed)
    .bind(&f.error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_deal_windows(
    pool: &PgPool,
    account_id: i64,
) -> crate::Result<HashSet<DateTime<Utc>>> {
    let rows: Vec<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT window_from FROM deal_fetches WHERE account_id = $1 AND status = 'ok' AND complete",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

// ------------------------------------------------------------------ accounts

pub struct AccountSnapshot {
    pub account_id: i64,
    pub trader_login: Option<i64>,
    pub deposit_asset_id: i64,
    pub balance: i64,
    pub money_digits: Option<i32>,
    pub registration_at: Option<DateTime<Utc>>,
    pub raw: Vec<u8>,
}

pub async fn record_account_snapshot(pool: &PgPool, s: &AccountSnapshot) -> crate::Result<()> {
    sqlx::query(
        "INSERT INTO account_snapshots (account_id, trader_login, deposit_asset_id, balance, money_digits, registration_at, raw)
         VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(s.account_id)
    .bind(s.trader_login)
    .bind(s.deposit_asset_id)
    .bind(s.balance)
    .bind(s.money_digits)
    .bind(s.registration_at)
    .bind(&s.raw)
    .execute(pool)
    .await?;
    Ok(())
}

// ------------------------------------------------------------------ trials

/// One research trial, pass or fail.
pub struct Trial<'a> {
    pub hypothesis: &'a str,
    pub params: &'a Value,
    pub data_range: &'a Value,
    /// "development" or "held_out".
    pub sample: &'a str,
    pub results: Option<&'a Value>,
    pub verdict: Option<&'a str>,
    pub settings_version: i64,
}

/// Log one research trial. Every trial is kept (the table is append-only).
pub async fn record_trial(pool: &PgPool, t: &Trial<'_>) -> crate::Result<i64> {
    sqlx::query_scalar(
        "INSERT INTO research_trials (hypothesis, params, data_range, sample, results_json, verdict, settings_version)
         VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING trial_id",
    )
    .bind(t.hypothesis)
    .bind(t.params)
    .bind(t.data_range)
    .bind(t.sample)
    .bind(t.results)
    .bind(t.verdict)
    .bind(t.settings_version)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
