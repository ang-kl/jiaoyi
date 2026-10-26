//! PostgreSQL storage for phase 1.
//!
//! Migrations live in the repository's `migrations/` directory and are embedded
//! in the binary at build time, so the service applies them on start.
//! Queries are plain runtime SQL (no compile-time database is needed to build).

use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::secret::Secret;
use jiaoyi_core::universe::{AssetClass, Resolution};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, QueryBuilder, Row};

pub mod goals;
pub mod history;
pub mod research;
pub mod settings;
pub mod strategy3;
pub mod ticks;
pub mod trading;

pub use sqlx;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("settings: {0}")]
    Settings(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// SQLSTATE 42P07: the relation already exists (here: a partition created by
/// a concurrent caller).
pub fn is_duplicate_table(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|c| c == "42P07")
}

pub async fn connect(url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new().max_connections(5).connect(url).await?)
}

pub async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

// ---------------------------------------------------------------- accounts

/// Upsert a permitted account. Login, currency and role are not read in
/// phase 1, so they stay NULL with that reason.
pub async fn upsert_account(pool: &PgPool, account_id: i64, env: Environment) -> Result<()> {
    sqlx::query(
        "INSERT INTO accounts (account_id, environment, login, login_missing_reason,
                               currency, currency_missing_reason, role, role_missing_reason, enabled)
         VALUES ($1, $2, NULL, $3, NULL, $4, NULL, $5, TRUE)
         ON CONFLICT (account_id) DO UPDATE
           SET environment = EXCLUDED.environment, enabled = TRUE, updated_at = now()",
    )
    .bind(account_id)
    .bind(env.as_str())
    .bind("not read in phase 1: ProtoOATraderReq is not on the phase-1 request list")
    .bind("not read in phase 1: ProtoOATraderReq is not on the phase-1 request list")
    .bind("not assigned in phase 1")
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn record_refusal(
    pool: &PgPool,
    account_ref: &str,
    reason: &str,
    source: &str,
) -> Result<()> {
    sqlx::query("INSERT INTO account_refusals (account_ref, reason, source) VALUES ($1, $2, $3)")
        .bind(account_ref)
        .bind(reason)
        .bind(source)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------- load runs

pub async fn start_run(pool: &PgPool, account_id: i64) -> Result<i64> {
    let id: i64 = sqlx::query_scalar("INSERT INTO load_runs (account_id) VALUES ($1) RETURNING id")
        .bind(account_id)
        .fetch_one(pool)
        .await?;
    Ok(id)
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct RunSummary {
    /// The settings version this load read.
    pub settings_version: Option<i64>,
    pub symbols_resolved: i32,
    pub symbols_unresolved: i32,
    pub bars_inserted: i32,
    pub bars_changed: i32,
    pub bars_unchanged: i32,
    pub bars_refused: i32,
    pub forming_bars_dropped: i32,
    pub spreads_measured: i32,
    pub crosscheck: Vec<Value>,
    pub errors: Vec<String>,
    /// The broker's symbol list as this load read it: [{id, name, enabled,
    /// base_asset_id, quote_asset_id, base_asset, quote_asset}].
    pub listed_symbols: Option<Value>,
    /// The account's asset list as this load read it: [{asset_id, name, ...}].
    pub assets: Option<Value>,
}

impl RunSummary {
    pub fn add_bars(&mut self, u: &BarUpsert) {
        self.bars_inserted += u.inserted;
        self.bars_changed += u.changed;
        self.bars_unchanged += u.unchanged;
        self.bars_refused += u.refused;
    }
}

pub async fn finish_run(pool: &PgPool, run_id: i64, status: &str, s: &RunSummary) -> Result<()> {
    sqlx::query(
        "UPDATE load_runs SET finished_at = now(), status = $2,
            symbols_resolved = $3, symbols_unresolved = $4, bars_inserted = $5,
            bars_changed = $6, bars_unchanged = $7, bars_refused = $8,
            forming_bars_dropped = $9, spreads_measured = $10, crosscheck = $11, errors = $12,
            settings_version = $13, listed_symbols = $14, assets = $15
         WHERE id = $1 AND status = 'running'",
    )
    .bind(run_id)
    .bind(status)
    .bind(s.symbols_resolved)
    .bind(s.symbols_unresolved)
    .bind(s.bars_inserted)
    .bind(s.bars_changed)
    .bind(s.bars_unchanged)
    .bind(s.bars_refused)
    .bind(s.forming_bars_dropped)
    .bind(s.spreads_measured)
    .bind(json!(s.crosscheck))
    .bind(json!(s.errors))
    .bind(s.settings_version)
    .bind(&s.listed_symbols)
    .bind(&s.assets)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunRow {
    pub id: i64,
    pub account_id: i64,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
    pub symbols_resolved: i32,
    pub symbols_unresolved: i32,
    pub bars_inserted: i32,
    pub bars_changed: i32,
    pub bars_unchanged: i32,
    pub bars_refused: i32,
    pub forming_bars_dropped: i32,
    pub spreads_measured: i32,
    pub crosscheck: Value,
    pub errors: Value,
    pub settings_version: Option<i64>,
}

/// The latest load run of each account.
pub async fn latest_runs(pool: &PgPool) -> Result<Vec<RunRow>> {
    let rows = sqlx::query(
        "SELECT DISTINCT ON (account_id) id, account_id, started_at, finished_at, status,
                symbols_resolved, symbols_unresolved, bars_inserted, bars_changed, bars_unchanged,
                bars_refused, forming_bars_dropped, spreads_measured, crosscheck, errors,
                settings_version
         FROM load_runs ORDER BY account_id, started_at DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RunRow {
            id: r.get("id"),
            account_id: r.get("account_id"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            status: r.get("status"),
            symbols_resolved: r.get("symbols_resolved"),
            symbols_unresolved: r.get("symbols_unresolved"),
            bars_inserted: r.get("bars_inserted"),
            bars_changed: r.get("bars_changed"),
            bars_unchanged: r.get("bars_unchanged"),
            bars_refused: r.get("bars_refused"),
            forming_bars_dropped: r.get("forming_bars_dropped"),
            spreads_measured: r.get("spreads_measured"),
            crosscheck: r.get("crosscheck"),
            errors: r.get("errors"),
            settings_version: r.get("settings_version"),
        })
        .collect())
}

/// The broker's symbol list from the account's latest load that recorded one:
/// (run id, when, [{id, name, enabled}]).
pub async fn latest_listed_symbols(
    pool: &PgPool,
    account_id: i64,
) -> Result<Option<(i64, DateTime<Utc>, Value)>> {
    let r = sqlx::query(
        "SELECT id, started_at, listed_symbols FROM load_runs
         WHERE account_id = $1 AND listed_symbols IS NOT NULL ORDER BY started_at DESC, id DESC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| (r.get("id"), r.get("started_at"), r.get("listed_symbols"))))
}

/// The latest load of an account that stored the broker's asset list:
/// (load run id, assets, symbol list).
pub async fn latest_assets(pool: &PgPool, account_id: i64) -> Result<Option<(i64, Value, Value)>> {
    let r = sqlx::query(
        "SELECT id, assets, listed_symbols FROM load_runs
         WHERE account_id = $1 AND assets IS NOT NULL AND listed_symbols IS NOT NULL
         ORDER BY started_at DESC, id DESC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| (r.get("id"), r.get("assets"), r.get("listed_symbols"))))
}

/// One load run's stored asset list and symbol list (both must be stored).
pub async fn assets_of_run(
    pool: &PgPool,
    account_id: i64,
    run_id: i64,
) -> Result<Option<(Value, Value)>> {
    let r = sqlx::query(
        "SELECT assets, listed_symbols FROM load_runs
         WHERE id = $1 AND account_id = $2 AND assets IS NOT NULL AND listed_symbols IS NOT NULL",
    )
    .bind(run_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| (r.get("assets"), r.get("listed_symbols"))))
}

// ---------------------------------------------------------------- universe

pub async fn record_resolution(
    pool: &PgPool,
    run_id: i64,
    account_id: i64,
    name: &str,
    class: AssetClass,
    r: &Resolution,
) -> Result<()> {
    let candidates: Vec<i64> = match r {
        Resolution::Ambiguous { symbol_ids, .. } => symbol_ids.clone(),
        _ => vec![],
    };
    sqlx::query(
        "INSERT INTO universe_resolutions
            (load_run_id, account_id, name, asset_class, status, broker_symbol_id, candidate_ids, reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(run_id)
    .bind(account_id)
    .bind(name)
    .bind(class.as_str())
    .bind(r.status())
    .bind(r.symbol_id())
    .bind(candidates)
    .bind(r.reason())
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResolutionRow {
    pub name: String,
    pub asset_class: String,
    pub status: String,
    pub broker_symbol_id: Option<i64>,
    pub candidate_ids: Vec<i64>,
    pub reason: Option<String>,
    pub at: DateTime<Utc>,
}

/// The latest resolution of each universe name on an account.
pub async fn latest_resolutions(pool: &PgPool, account_id: i64) -> Result<Vec<ResolutionRow>> {
    let rows = sqlx::query(
        "SELECT DISTINCT ON (name) name, asset_class, status, broker_symbol_id, candidate_ids, reason, at
         FROM universe_resolutions WHERE account_id = $1 ORDER BY name, id DESC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ResolutionRow {
            name: r.get("name"),
            asset_class: r.get("asset_class"),
            status: r.get("status"),
            broker_symbol_id: r.get("broker_symbol_id"),
            candidate_ids: r.get("candidate_ids"),
            reason: r.get("reason"),
            at: r.get("at"),
        })
        .collect())
}

// ---------------------------------------------------------------- symbols

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolRow {
    pub account_id: i64,
    pub broker_symbol_id: i64,
    pub name: String,
    pub asset_class: String,
    pub listed_enabled: Option<bool>,
    pub trading_mode: Option<String>,
    pub digits: i32,
    pub pip_position: i32,
    pub lot_size: Option<i64>,
    pub min_volume: Option<i64>,
    pub max_volume: Option<i64>,
    pub volume_step: Option<i64>,
    pub schedule_time_zone: Option<String>,
    pub measurement_units: Option<String>,
    /// Column name → why it is NULL. Must name every NULL broker column.
    pub null_reasons: Value,
    pub raw: Vec<u8>,
    pub read_at: DateTime<Utc>,
}

pub async fn upsert_symbol(pool: &PgPool, s: &SymbolRow) -> Result<()> {
    sqlx::query(
        "INSERT INTO symbols (account_id, broker_symbol_id, name, asset_class, listed_enabled,
                              trading_mode, digits, pip_position, lot_size, min_volume, max_volume,
                              volume_step, value_per_point, schedule_time_zone, measurement_units,
                              null_reasons, raw, read_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,NULL,$13,$14,$15,$16,$17)
         ON CONFLICT (account_id, broker_symbol_id) DO UPDATE SET
            name = EXCLUDED.name, asset_class = EXCLUDED.asset_class,
            listed_enabled = EXCLUDED.listed_enabled, trading_mode = EXCLUDED.trading_mode,
            digits = EXCLUDED.digits, pip_position = EXCLUDED.pip_position,
            lot_size = EXCLUDED.lot_size, min_volume = EXCLUDED.min_volume,
            max_volume = EXCLUDED.max_volume, volume_step = EXCLUDED.volume_step,
            value_per_point = NULL, schedule_time_zone = EXCLUDED.schedule_time_zone,
            measurement_units = EXCLUDED.measurement_units, null_reasons = EXCLUDED.null_reasons,
            raw = EXCLUDED.raw, read_at = EXCLUDED.read_at",
    )
    .bind(s.account_id)
    .bind(s.broker_symbol_id)
    .bind(&s.name)
    .bind(&s.asset_class)
    .bind(s.listed_enabled)
    .bind(&s.trading_mode)
    .bind(s.digits)
    .bind(s.pip_position)
    .bind(s.lot_size)
    .bind(s.min_volume)
    .bind(s.max_volume)
    .bind(s.volume_step)
    .bind(&s.schedule_time_zone)
    .bind(&s.measurement_units)
    .bind(&s.null_reasons)
    .bind(&s.raw)
    .bind(s.read_at)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SymbolInfo {
    pub broker_symbol_id: i64,
    pub name: String,
    pub listed_enabled: Option<bool>,
    pub trading_mode: Option<String>,
    pub digits: i32,
    pub lot_size: Option<i64>,
    pub min_volume: Option<i64>,
    pub volume_step: Option<i64>,
    pub null_reasons: Value,
    pub read_at: DateTime<Utc>,
}

pub async fn symbol_info(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
) -> Result<Option<SymbolInfo>> {
    let r = sqlx::query(
        "SELECT broker_symbol_id, name, listed_enabled, trading_mode, digits, lot_size, min_volume,
                volume_step, null_reasons, read_at
         FROM symbols WHERE account_id = $1 AND broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| SymbolInfo {
        broker_symbol_id: r.get("broker_symbol_id"),
        name: r.get("name"),
        listed_enabled: r.get("listed_enabled"),
        trading_mode: r.get("trading_mode"),
        digits: r.get("digits"),
        lot_size: r.get("lot_size"),
        min_volume: r.get("min_volume"),
        volume_step: r.get("volume_step"),
        null_reasons: r.get("null_reasons"),
        read_at: r.get("read_at"),
    }))
}

// ---------------------------------------------------------------- costs

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CostRow {
    pub account_id: i64,
    pub broker_symbol_id: i64,
    pub load_run_id: Option<i64>,
    pub measured_at: DateTime<Utc>,
    pub spread_median: Option<Decimal>,
    pub spread_samples: i32,
    pub spread_window_secs: i32,
    pub commission_type: Option<String>,
    pub precise_trading_commission_rate: Option<i64>,
    pub commission_deprecated: Option<i64>,
    pub precise_min_commission: Option<i64>,
    pub min_commission_type: Option<String>,
    pub min_commission_asset: Option<String>,
    pub swap_long: Option<f64>,
    pub swap_short: Option<f64>,
    pub swap_calculation_type: Option<String>,
    pub swap_rollover_3days: Option<String>,
    pub swap_period_hours: Option<i32>,
    pub swap_time_minutes: Option<i32>,
    /// Whether the market was open at `measured_at`, from the symbol's schedule;
    /// `None` when that cannot be told, with the reason.
    pub market_open: Option<bool>,
    pub market_state_reason: Option<String>,
    pub source: String,
    pub null_reasons: Value,
}

pub async fn insert_cost(pool: &PgPool, c: &CostRow) -> Result<i64> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO symbol_costs (account_id, broker_symbol_id, load_run_id, measured_at,
            spread_median, spread_samples, spread_window_secs, commission_per_lot_side,
            commission_type, precise_trading_commission_rate, commission_deprecated,
            precise_min_commission, min_commission_type, min_commission_asset, swap_long,
            swap_short, swap_calculation_type, swap_rollover_3days, swap_period_hours,
            swap_time_minutes, source, null_reasons, market_open, market_state_reason)
         VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)
         RETURNING id",
    )
    .bind(c.account_id)
    .bind(c.broker_symbol_id)
    .bind(c.load_run_id)
    .bind(c.measured_at)
    .bind(c.spread_median)
    .bind(c.spread_samples)
    .bind(c.spread_window_secs)
    .bind(&c.commission_type)
    .bind(c.precise_trading_commission_rate)
    .bind(c.commission_deprecated)
    .bind(c.precise_min_commission)
    .bind(&c.min_commission_type)
    .bind(&c.min_commission_asset)
    .bind(c.swap_long)
    .bind(c.swap_short)
    .bind(&c.swap_calculation_type)
    .bind(&c.swap_rollover_3days)
    .bind(c.swap_period_hours)
    .bind(c.swap_time_minutes)
    .bind(&c.source)
    .bind(&c.null_reasons)
    .bind(c.market_open)
    .bind(&c.market_state_reason)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SpreadReading {
    pub spread_median: Option<Decimal>,
    pub spread_samples: i32,
    pub measured_at: DateTime<Utc>,
    pub null_reason: Option<String>,
    pub market_open: Option<bool>,
    pub market_state_reason: Option<String>,
}

/// The latest spread measurement from at least `min_samples` quote updates,
/// and the latest attempt of any kind. A sample smaller than that (one stale
/// weekend quote) is never taken as the measurement; the next load samples again.
pub async fn spread_readings(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    min_samples: i64,
) -> Result<(Option<SpreadReading>, Option<SpreadReading>)> {
    let q = |only_measured: bool| {
        format!(
            "SELECT spread_median, spread_samples, measured_at, null_reasons->>'spread_median' AS why,
                    market_open, market_state_reason
             FROM symbol_costs WHERE account_id = $1 AND broker_symbol_id = $2 {}
             ORDER BY measured_at DESC, id DESC LIMIT 1",
            if only_measured {
                "AND spread_median IS NOT NULL AND spread_samples >= $3"
            } else {
                "AND $3::bigint IS NOT NULL"
            }
        )
    };
    let map = |r: sqlx::postgres::PgRow| SpreadReading {
        spread_median: r.get("spread_median"),
        spread_samples: r.get("spread_samples"),
        measured_at: r.get("measured_at"),
        null_reason: r.get("why"),
        market_open: r.get("market_open"),
        market_state_reason: r.get("market_state_reason"),
    };
    let measured = sqlx::query(&q(true))
        .bind(account_id)
        .bind(symbol_id)
        .bind(min_samples)
        .fetch_optional(pool)
        .await?
        .map(map);
    let latest = sqlx::query(&q(false))
        .bind(account_id)
        .bind(symbol_id)
        .bind(min_samples)
        .fetch_optional(pool)
        .await?
        .map(map);
    Ok((measured, latest))
}

// ---------------------------------------------------------------- bars

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BarRow {
    pub bar_date: NaiveDate,
    pub bar_open_utc: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub tick_volume: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct BarUpsert {
    pub inserted: i32,
    pub changed: i32,
    pub unchanged: i32,
    /// Bars not stored: two bars in one read share a UTC date.
    pub refused: i32,
}

/// Store closed daily bars for one symbol. New dates are inserted; a stored
/// bar that the broker now reports differently is recorded in
/// `reconcile_events` (old and new values) and then replaced, because the
/// broker is the authority. Two bars of one read on the same UTC date are both
/// refused and recorded: neither is chosen.
pub async fn upsert_daily_bars(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    run_id: Option<i64>,
    bars: &[BarRow],
    fetched_at: DateTime<Utc>,
) -> Result<BarUpsert> {
    let mut out = BarUpsert::default();
    let mut tx = pool.begin().await?;

    let mut by_date: HashMap<NaiveDate, Vec<&BarRow>> = HashMap::new();
    for b in bars {
        by_date.entry(b.bar_date).or_default().push(b);
    }

    let existing: HashMap<NaiveDate, BarRow> = sqlx::query(
        "SELECT bar_date, bar_open_utc, open, high, low, close, tick_volume
         FROM daily_bars WHERE account_id = $1 AND broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|r| {
        let b = BarRow {
            bar_date: r.get("bar_date"),
            bar_open_utc: r.get("bar_open_utc"),
            open: r.get("open"),
            high: r.get("high"),
            low: r.get("low"),
            close: r.get("close"),
            tick_volume: r.get("tick_volume"),
        };
        (b.bar_date, b)
    })
    .collect();

    let mut new_rows: Vec<&BarRow> = Vec::new();
    let mut dates: Vec<_> = by_date.keys().copied().collect();
    dates.sort();
    for date in dates {
        let group = &by_date[&date];
        if group.len() > 1 {
            out.refused += group.len() as i32;
            record_reconcile(
                &mut *tx,
                Some(account_id),
                "bar_date_collision",
                &format!("symbol {symbol_id} {date}"),
                json!({ "bars": group }),
                "none stored: two or more bars share one UTC date",
            )
            .await?;
            continue;
        }
        let b = group[0];
        match existing.get(&date) {
            None => new_rows.push(b),
            Some(old) if old == b => out.unchanged += 1,
            Some(old) => {
                record_reconcile(
                    &mut *tx,
                    Some(account_id),
                    "daily_bar_changed",
                    &format!("symbol {symbol_id} {date}"),
                    json!({ "stored": old, "broker": b }),
                    "broker value stored",
                )
                .await?;
                sqlx::query(
                    "UPDATE daily_bars SET bar_open_utc = $4, open = $5, high = $6, low = $7,
                        close = $8, tick_volume = $9, fetched_at = $10, load_run_id = $11
                     WHERE account_id = $1 AND broker_symbol_id = $2 AND bar_date = $3",
                )
                .bind(account_id)
                .bind(symbol_id)
                .bind(date)
                .bind(b.bar_open_utc)
                .bind(b.open)
                .bind(b.high)
                .bind(b.low)
                .bind(b.close)
                .bind(b.tick_volume)
                .bind(fetched_at)
                .bind(run_id)
                .execute(&mut *tx)
                .await?;
                out.changed += 1;
            }
        }
    }

    for chunk in new_rows.chunks(1000) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO daily_bars (account_id, broker_symbol_id, bar_date, bar_open_utc, open, high,
                                     low, close, tick_volume, fetched_at, load_run_id) ",
        );
        qb.push_values(chunk, |mut row, b| {
            row.push_bind(account_id)
                .push_bind(symbol_id)
                .push_bind(b.bar_date)
                .push_bind(b.bar_open_utc)
                .push_bind(b.open)
                .push_bind(b.high)
                .push_bind(b.low)
                .push_bind(b.close)
                .push_bind(b.tick_volume)
                .push_bind(fetched_at)
                .push_bind(run_id);
        });
        qb.build().execute(&mut *tx).await?;
        out.inserted += chunk.len() as i32;
    }
    tx.commit().await?;
    Ok(out)
}

/// All stored bars for one symbol, oldest first.
pub async fn bars(pool: &PgPool, account_id: i64, symbol_id: i64) -> Result<Vec<BarRow>> {
    let rows = sqlx::query(
        "SELECT bar_date, bar_open_utc, open, high, low, close, tick_volume
         FROM daily_bars WHERE account_id = $1 AND broker_symbol_id = $2 ORDER BY bar_date",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BarRow {
            bar_date: r.get("bar_date"),
            bar_open_utc: r.get("bar_open_utc"),
            open: r.get("open"),
            high: r.get("high"),
            low: r.get("low"),
            close: r.get("close"),
            tick_volume: r.get("tick_volume"),
        })
        .collect())
}

/// The broker symbol ids an account's stored symbols carry under `name`.
pub async fn symbol_ids_by_name(pool: &PgPool, account_id: i64, name: &str) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar(
        "SELECT broker_symbol_id FROM symbols WHERE account_id = $1 AND name = $2
         ORDER BY broker_symbol_id",
    )
    .bind(account_id)
    .bind(name)
    .fetch_all(pool)
    .await?)
}

/// The first stored daily bar of one symbol dated `from..=to` whose end
/// (open + 1 day) is after `instant`, if any.
pub async fn first_bar_ending_after(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    from: Option<NaiveDate>,
    to: NaiveDate,
    instant: DateTime<Utc>,
) -> Result<Option<BarRow>> {
    let r = sqlx::query(
        "SELECT bar_date, bar_open_utc, open, high, low, close, tick_volume
         FROM daily_bars
         WHERE account_id = $1 AND broker_symbol_id = $2
           AND ($3::date IS NULL OR bar_date >= $3) AND bar_date <= $4
           AND bar_open_utc + interval '1 day' > $5
         ORDER BY bar_date LIMIT 1",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(from)
    .bind(to)
    .bind(instant)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| BarRow {
        bar_date: r.get("bar_date"),
        bar_open_utc: r.get("bar_open_utc"),
        open: r.get("open"),
        high: r.get("high"),
        low: r.get("low"),
        close: r.get("close"),
        tick_volume: r.get("tick_volume"),
    }))
}

/// Stored daily bars of one symbol dated `from..=to` (no lower bound when
/// `from` is `None`), in date order, at most `limit` rows.
pub async fn bars_between(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    from: Option<NaiveDate>,
    to: NaiveDate,
    limit: i64,
) -> Result<Vec<BarRow>> {
    let rows = sqlx::query(
        "SELECT bar_date, bar_open_utc, open, high, low, close, tick_volume
         FROM daily_bars
         WHERE account_id = $1 AND broker_symbol_id = $2
           AND ($3::date IS NULL OR bar_date >= $3) AND bar_date <= $4
         ORDER BY bar_date LIMIT $5",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BarRow {
            bar_date: r.get("bar_date"),
            bar_open_utc: r.get("bar_open_utc"),
            open: r.get("open"),
            high: r.get("high"),
            low: r.get("low"),
            close: r.get("close"),
            tick_volume: r.get("tick_volume"),
        })
        .collect())
}

// ---------------------------------------------------------------- reconcile

pub async fn record_reconcile<'e, E>(
    exec: E,
    account_id: Option<i64>,
    kind: &str,
    subject: &str,
    detail: Value,
    resolution: &str,
) -> Result<()>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    sqlx::query(
        "INSERT INTO reconcile_events (account_id, kind, subject, detail, resolution)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(account_id)
    .bind(kind)
    .bind(subject)
    .bind(detail)
    .bind(resolution)
    .execute(exec)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------- tokens

pub const TOKEN_OWNER: &str = "ctrader";

#[derive(Debug, Clone)]
pub struct StoredToken {
    pub access_token: Secret,
    pub refresh_token: Secret,
    pub expires_at: Option<DateTime<Utc>>,
    pub obtained_at: DateTime<Utc>,
    pub source: String,
    /// The fingerprint of the env seed this token descends from; `None` for a
    /// token stored before migration 0007.
    pub seed_fingerprint: Option<String>,
}

pub async fn load_token(pool: &PgPool) -> Result<Option<StoredToken>> {
    let r = sqlx::query(
        "SELECT access_token, refresh_token, expires_at, obtained_at, source, seed_fingerprint
         FROM oauth_tokens WHERE token_owner = $1",
    )
    .bind(TOKEN_OWNER)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| StoredToken {
        access_token: Secret::new(r.get::<String, _>("access_token")),
        refresh_token: Secret::new(r.get::<String, _>("refresh_token")),
        expires_at: r.get("expires_at"),
        obtained_at: r.get("obtained_at"),
        source: r.get("source"),
        seed_fingerprint: r.get("seed_fingerprint"),
    }))
}

/// Store a token. `expires_at` is NULL when unknown (an env seed without an
/// expiry), with that reason. The seed fingerprint of the stored row is kept:
/// a refreshed token descends from the same seed.
pub async fn save_token(
    pool: &PgPool,
    access: &Secret,
    refresh: &Secret,
    token_type: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
    source: &str,
) -> Result<()> {
    let reason = expires_at.is_none().then_some(
        "expiry not known: seeded from the environment without CTRADER_ACCESS_TOKEN_EXPIRES_AT",
    );
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, token_type, expires_at,
                                   expires_at_missing_reason, obtained_at, source)
         VALUES ($1, $2, $3, $4, $5, $6, now(), $7)
         ON CONFLICT (token_owner) DO UPDATE SET
            access_token = EXCLUDED.access_token, refresh_token = EXCLUDED.refresh_token,
            token_type = EXCLUDED.token_type, expires_at = EXCLUDED.expires_at,
            expires_at_missing_reason = EXCLUDED.expires_at_missing_reason,
            obtained_at = EXCLUDED.obtained_at, source = EXCLUDED.source",
    )
    .bind(TOKEN_OWNER)
    .bind(access.expose())
    .bind(refresh.expose())
    .bind(token_type)
    .bind(expires_at)
    .bind(reason)
    .bind(source)
    .execute(pool)
    .await?;
    Ok(())
}

/// Store an env seed as the token (source `env_seed`), remembering its
/// fingerprint, so that the same seed is not stored over its own refreshed
/// token again, and a different seed is.
pub async fn store_seed(
    pool: &PgPool,
    access: &Secret,
    refresh: &Secret,
    expires_at: Option<DateTime<Utc>>,
) -> Result<()> {
    save_token(pool, access, refresh, None, expires_at, "env_seed").await?;
    set_seed_fingerprint(pool, &access.fingerprint()).await
}

/// Remember which env seed the stored token descends from.
pub async fn set_seed_fingerprint(pool: &PgPool, fingerprint: &str) -> Result<()> {
    sqlx::query("UPDATE oauth_tokens SET seed_fingerprint = $2 WHERE token_owner = $1")
        .bind(TOKEN_OWNER)
        .bind(fingerprint)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------- token grants

/// One read of a token's grant, made before authorising `account_id`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GrantRead {
    pub account_id: i64,
    pub host: String,
    pub token_fingerprint: String,
    pub token_source: String,
    /// `listed`, `not_listed`, `environment_mismatch` or `rejected`.
    pub verdict: &'static str,
    pub reason: Option<String>,
    pub permission_scope: Option<String>,
    /// `[{ctid, is_live, login}]`; `None` when the read was rejected.
    pub accounts: Option<Value>,
}

pub async fn record_grant(pool: &PgPool, g: &GrantRead) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "INSERT INTO token_grants (account_id, host, token_fingerprint, token_source, verdict,
                                   reason, permission_scope, accounts)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(g.account_id)
    .bind(&g.host)
    .bind(&g.token_fingerprint)
    .bind(&g.token_source)
    .bind(g.verdict)
    .bind(&g.reason)
    .bind(&g.permission_scope)
    .bind(&g.accounts)
    .fetch_one(pool)
    .await?)
}

/// The latest grant read made for an account, as stored.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GrantRow {
    pub id: i64,
    pub read_at: DateTime<Utc>,
    pub account_id: i64,
    pub host: String,
    pub token_fingerprint: String,
    pub token_source: String,
    pub verdict: String,
    pub reason: Option<String>,
    pub permission_scope: Option<String>,
    pub accounts: Option<Value>,
}

/// The latest grant read per account.
pub async fn latest_grants(pool: &PgPool) -> Result<Vec<GrantRow>> {
    let rows = sqlx::query(
        "SELECT DISTINCT ON (account_id) id, read_at, account_id, host, token_fingerprint,
                token_source, verdict, reason, permission_scope, accounts
         FROM token_grants ORDER BY account_id, id DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| GrantRow {
            id: r.get("id"),
            read_at: r.get("read_at"),
            account_id: r.get("account_id"),
            host: r.get("host"),
            token_fingerprint: r.get("token_fingerprint"),
            token_source: r.get("token_source"),
            verdict: r.get("verdict"),
            reason: r.get("reason"),
            permission_scope: r.get("permission_scope"),
            accounts: r.get("accounts"),
        })
        .collect())
}

/// The environment an account was declared with (`accounts.environment`).
pub async fn account_environment(pool: &PgPool, account_id: i64) -> Result<Option<Environment>> {
    let env: Option<String> =
        sqlx::query_scalar("SELECT environment FROM accounts WHERE account_id = $1")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    Ok(env.and_then(|e| match e.as_str() {
        "demo" => Some(Environment::Demo),
        "live" => Some(Environment::Live),
        _ => None,
    }))
}
