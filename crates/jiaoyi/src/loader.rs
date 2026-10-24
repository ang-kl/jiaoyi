//! The phase-1 load of one permitted account.
//!
//! Every tunable number is read from the settings at the start of the load,
//! and the settings version is stored on the load run.
//!
//! Order: authorise → list symbols → resolve the universe (recorded) → read and
//! store symbol details → read the maximum daily history and store closed bars
//! → sample spreads and store costs → cross-check recent bars against a fresh
//! broker read → close the load run.
//!
//! Nothing here can send an order: the client has no trading request on its
//! payload allowlist.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::PermittedAccount;
use jiaoyi_core::universe::{resolve_universe, AssetClass, ListedSymbol};
use jiaoyi_ctrader::client::SpreadSamples;
use jiaoyi_ctrader::messages::{
    ProtoOaCommissionType, ProtoOaDayOfWeek, ProtoOaMinCommissionType, ProtoOaSwapCalculationType,
    ProtoOaSymbol, ProtoOaTradingMode,
};
use jiaoyi_ctrader::{closed_daily_bars, decode_trendbar, AccountSession, ClientError, Connection};
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::{self as store, BarRow, CostRow, RunSummary, SymbolRow};
use prost::Message;
use serde_json::{json, Map, Value};

use crate::config::BrokerConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct LoadOutcome {
    pub run_id: i64,
    pub status: &'static str,
    pub summary: RunSummary,
    /// The run failed because the broker rejected the access token.
    pub auth_rejected: bool,
}

/// Get the token to use: the stored one, else the environment seed (stored
/// on first use). `None` means there is no token anywhere.
pub async fn current_token(
    pool: &PgPool,
    broker: &BrokerConfig,
) -> store::Result<Option<store::StoredToken>> {
    if let Some(t) = store::load_token(pool).await? {
        return Ok(Some(t));
    }
    let (Some(a), Some(r)) = (&broker.seed_access_token, &broker.seed_refresh_token) else {
        return Ok(None);
    };
    store::save_token(pool, a, r, None, broker.seed_expires_at, "env_seed").await?;
    store::load_token(pool).await
}

/// One refresh at a time in this process: the refresh token is single-use,
/// and the loader and the tick recorder share it.
static REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Refresh the token over this connection and store the new pair. If another
/// task refreshed it meanwhile, use that token instead of refreshing again.
pub async fn refresh(
    pool: &PgPool,
    conn: &Connection,
    token: &store::StoredToken,
) -> anyhow::Result<store::StoredToken> {
    let _guard = REFRESH_LOCK.lock().await;
    if let Some(now_stored) = store::load_token(pool).await? {
        if now_stored.refresh_token != token.refresh_token {
            return Ok(now_stored);
        }
    }
    let t = conn.refresh_token(&token.refresh_token).await?;
    let expires_at = Utc::now() + chrono::Duration::seconds(t.expires_in_secs);
    store::save_token(
        pool,
        &t.access_token,
        &t.refresh_token,
        Some(&t.token_type),
        Some(expires_at),
        "refresh",
    )
    .await?;
    tracing::info!(expires_at = %expires_at, "cTrader access token refreshed and stored");
    store::load_token(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("token vanished after refresh"))
}

/// Refresh the token when it expires within this window.
pub const REFRESH_WHEN_WITHIN: chrono::Duration = chrono::Duration::days(3);

/// Load one account over an open connection. Records a load run whatever happens.
pub async fn load_account(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
) -> store::Result<LoadOutcome> {
    let run_id = store::start_run(pool, account.id()).await?;
    let mut summary = RunSummary::default();
    let mut auth_rejected = false;
    let status = match run(pool, conn, broker, account, run_id, &mut summary).await {
        Ok(()) if summary.errors.is_empty() => "ok",
        Ok(()) => "partial",
        Err(e) => {
            auth_rejected = e.chain().any(|c| {
                c.downcast_ref::<ClientError>()
                    .is_some_and(ClientError::is_token_rejected)
            });
            summary.errors.push(format!("{e:#}"));
            "failed"
        }
    };
    for n in conn.notices() {
        summary.errors.push(format!("broker notice: {n}"));
    }
    store::finish_run(pool, run_id, status, &summary).await?;
    tracing::info!(account = %account, run_id, status, ?summary.errors, "load finished");
    Ok(LoadOutcome {
        run_id,
        status,
        summary,
        auth_rejected,
    })
}

async fn run(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
    run_id: i64,
    summary: &mut RunSummary,
) -> anyhow::Result<()> {
    let settings = store::settings::snapshot(pool, Some(account.id())).await?;
    summary.settings_version = Some(settings.version);
    let universe = settings
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    let spread_window = Duration::from_secs(
        settings
            .integer("spread.sample_secs")
            .map_err(anyhow::Error::msg)?
            .max(0) as u64,
    );
    let crosscheck_per_class = settings
        .integer("crosscheck.symbols_per_class")
        .map_err(anyhow::Error::msg)?
        .max(0) as usize;
    let crosscheck_days = settings
        .integer("crosscheck.days")
        .map_err(anyhow::Error::msg)?;

    let session = authorise(pool, conn, broker, account).await?;

    let listed = session.symbols_list().await?;
    let listed_symbols: Vec<ListedSymbol> = listed
        .symbol
        .iter()
        .map(|s| ListedSymbol {
            symbol_id: s.symbol_id,
            name: s.symbol_name.clone(),
            enabled: s.enabled,
        })
        .collect();
    summary.listed_symbols = Some(serde_json::Value::Array(
        listed_symbols
            .iter()
            .map(|s| json!({ "id": s.symbol_id, "name": s.name, "enabled": s.enabled }))
            .collect(),
    ));
    let resolutions = resolve_universe(&universe, &listed_symbols);
    let mut resolved: Vec<(&str, AssetClass, i64)> = Vec::new();
    for (name, class, r) in &resolutions {
        store::record_resolution(pool, run_id, account.id(), name, *class, r).await?;
        match r.symbol_id() {
            Some(id) => resolved.push((name.as_str(), *class, id)),
            None => summary.symbols_unresolved += 1,
        }
    }
    summary.symbols_resolved = resolved.len() as i32;
    if resolved.is_empty() {
        return Ok(());
    }

    let ids: Vec<i64> = resolved.iter().map(|r| r.2).collect();
    let details = session.symbols_by_id(&ids).await?;
    let by_id: HashMap<i64, ProtoOaSymbol> = details
        .symbol
        .into_iter()
        .map(|s| (s.symbol_id, s))
        .collect();
    let read_at = Utc::now();
    let mut stored: Vec<(&str, AssetClass, i64)> = Vec::new();
    for (name, class, id) in &resolved {
        let Some(sym) = by_id.get(id) else {
            summary.errors.push(format!(
                "{name}: symbol {id} missing from ProtoOASymbolByIdRes; not loaded"
            ));
            continue;
        };
        let enabled = listed_symbols
            .iter()
            .find(|s| s.symbol_id == *id)
            .and_then(|s| s.enabled);
        store::upsert_symbol(
            pool,
            &symbol_row(account.id(), name, *class, enabled, sym, read_at),
        )
        .await?;
        stored.push((name, *class, *id));
    }

    for (name, _, id) in &stored {
        if let Err(e) = load_bars(pool, &session, account.id(), *id, run_id, summary).await {
            summary.errors.push(format!("{name}: daily bars: {e}"));
        }
    }

    let stored_ids: Vec<i64> = stored.iter().map(|s| s.2).collect();
    let samples = match session.sample_spreads(&stored_ids, spread_window).await {
        Ok(s) => s,
        Err(e) => {
            summary.errors.push(format!("spread sampling: {e}"));
            HashMap::new()
        }
    };
    let measured_at = Utc::now();
    for (name, _, id) in &stored {
        let sample = samples.get(id);
        let row = cost_row(
            account.id(),
            *id,
            run_id,
            measured_at,
            spread_window,
            sample,
            by_id.get(id),
        );
        if row.spread_median.is_some() {
            summary.spreads_measured += 1;
        }
        if let Err(e) = store::insert_cost(pool, &row).await {
            summary.errors.push(format!("{name}: costs: {e}"));
        }
    }

    let mut per_class: HashMap<AssetClass, usize> = HashMap::new();
    for (name, class, id) in &stored {
        let n = per_class.entry(*class).or_default();
        if *n >= crosscheck_per_class {
            continue;
        }
        *n += 1;
        match crosscheck(pool, &session, account.id(), *id, crosscheck_days).await {
            Ok(mut v) => {
                v["name"] = json!(name);
                summary.crosscheck.push(v);
            }
            Err(e) => summary.errors.push(format!("{name}: cross-check: {e}")),
        }
    }
    Ok(())
}

/// Authorise the application and one permitted account on a connection,
/// refreshing the token when it is near expiry or rejected (once).
pub async fn authorise<'c>(
    pool: &PgPool,
    conn: &'c Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
) -> anyhow::Result<AccountSession<'c>> {
    let client_id = broker
        .client_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CTRADER_CLIENT_ID is not set"))?;
    let client_secret = broker
        .client_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("CTRADER_CLIENT_SECRET is not set"))?;
    conn.application_auth(client_id, client_secret).await?;

    let mut token = current_token(pool, broker).await?.ok_or_else(|| {
        anyhow::anyhow!("no token: oauth_tokens is empty and CTRADER_ACCESS_TOKEN / CTRADER_REFRESH_TOKEN are not set")
    })?;
    if token
        .expires_at
        .is_some_and(|t| t - Utc::now() < REFRESH_WHEN_WITHIN)
    {
        token = refresh(pool, conn, &token).await?;
    }
    let session = match conn.account_auth(account, &token.access_token).await {
        Ok(s) => s,
        Err(e) if e.is_token_rejected() => {
            tracing::warn!(account = %account, "access token rejected; refreshing once");
            token = refresh(pool, conn, &token).await?;
            conn.account_auth(account, &token.access_token).await?
        }
        Err(e) => return Err(e.into()),
    };

    Ok(session)
}

async fn load_bars(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    run_id: i64,
    summary: &mut RunSummary,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let raw = session.daily_history(symbol_id, now).await?;
    let mut decoded = Vec::with_capacity(raw.len());
    for tb in &raw {
        match decode_trendbar(tb) {
            Ok(b) => decoded.push(b),
            Err(e) => {
                summary.bars_refused += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "trendbar_refused",
                    &format!("symbol {symbol_id}"),
                    json!({ "error": e.to_string(), "raw_hex": hex(&tb.encode_to_vec()) }),
                    "not stored",
                )
                .await?;
            }
        }
    }
    let (closed, forming) = closed_daily_bars(decoded, now);
    summary.forming_bars_dropped += forming.len() as i32;
    let rows: Vec<BarRow> = closed.into_iter().map(to_row).collect();
    let u = store::upsert_daily_bars(pool, account_id, symbol_id, Some(run_id), &rows, now).await?;
    summary.add_bars(&u);
    Ok(())
}

fn to_row(b: jiaoyi_ctrader::DailyBar) -> BarRow {
    BarRow {
        bar_date: b.date,
        bar_open_utc: b.open_time,
        open: b.open,
        high: b.high,
        low: b.low,
        close: b.close,
        tick_volume: b.tick_volume,
    }
}

/// Re-read the last `days` days from the broker in a separate request and
/// compare with what is stored. Every difference is recorded.
async fn crosscheck(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    days: i64,
) -> anyhow::Result<Value> {
    let now = Utc::now();
    let (raw, _) = session
        .daily_trendbars(symbol_id, now - chrono::Duration::days(days), now)
        .await?;
    let decoded: Vec<_> = raw.iter().filter_map(|t| decode_trendbar(t).ok()).collect();
    let (closed, _) = closed_daily_bars(decoded, now);
    let stored: HashMap<_, _> = store::bars(pool, account_id, symbol_id)
        .await?
        .into_iter()
        .map(|b| (b.bar_date, b))
        .collect();
    let (mut matched, mut mismatched, mut missing) = (0, 0, 0);
    for b in closed.into_iter().map(to_row) {
        match stored.get(&b.bar_date) {
            Some(s) if *s == b => matched += 1,
            Some(s) => {
                mismatched += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "crosscheck_mismatch",
                    &format!("symbol {symbol_id} {}", b.bar_date),
                    json!({ "stored": s, "broker": b }),
                    "recorded; the next load stores the broker value",
                )
                .await?;
            }
            None => {
                missing += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "crosscheck_missing",
                    &format!("symbol {symbol_id} {}", b.bar_date),
                    json!({ "broker": b }),
                    "recorded; the next load stores it",
                )
                .await?;
            }
        }
    }
    Ok(json!({
        "symbol_id": symbol_id,
        "window_days": days,
        "matched": matched,
        "mismatched": mismatched,
        "missing_in_db": missing,
    }))
}

fn enum_name<E: TryFrom<i32> + Copy>(
    v: Option<i32>,
    name: impl Fn(E) -> &'static str,
) -> Option<String> {
    v.map(|i| match E::try_from(i) {
        Ok(e) => name(e).to_string(),
        Err(_) => format!("UNKNOWN({i})"),
    })
}

fn note_null<T>(reasons: &mut Map<String, Value>, column: &str, proto_field: &str, v: &Option<T>) {
    if v.is_none() {
        reasons.insert(
            column.into(),
            json!(format!("broker sent no {proto_field}")),
        );
    }
}

pub fn symbol_row(
    account_id: i64,
    name: &str,
    class: AssetClass,
    listed_enabled: Option<bool>,
    s: &ProtoOaSymbol,
    read_at: DateTime<Utc>,
) -> SymbolRow {
    let mut r = Map::new();
    let trading_mode = enum_name(s.trading_mode, |e: ProtoOaTradingMode| e.as_str_name());
    note_null(
        &mut r,
        "listed_enabled",
        "ProtoOALightSymbol.enabled",
        &listed_enabled,
    );
    note_null(
        &mut r,
        "trading_mode",
        "tradingMode (proto2 default ENABLED)",
        &trading_mode,
    );
    note_null(&mut r, "lot_size", "lotSize", &s.lot_size);
    note_null(&mut r, "min_volume", "minVolume", &s.min_volume);
    note_null(&mut r, "max_volume", "maxVolume", &s.max_volume);
    note_null(&mut r, "volume_step", "stepVolume", &s.step_volume);
    note_null(
        &mut r,
        "schedule_time_zone",
        "scheduleTimeZone",
        &s.schedule_time_zone,
    );
    note_null(
        &mut r,
        "measurement_units",
        "measurementUnits",
        &s.measurement_units,
    );
    r.insert(
        "value_per_point".into(),
        json!("not given by ProtoOASymbol; needs lot size, price and a conversion rate (phase 4)"),
    );
    SymbolRow {
        account_id,
        broker_symbol_id: s.symbol_id,
        name: name.to_string(),
        asset_class: class.as_str().to_string(),
        listed_enabled,
        trading_mode,
        digits: s.digits,
        pip_position: s.pip_position,
        lot_size: s.lot_size,
        min_volume: s.min_volume,
        max_volume: s.max_volume,
        volume_step: s.step_volume,
        schedule_time_zone: s.schedule_time_zone.clone(),
        measurement_units: s.measurement_units.clone(),
        null_reasons: Value::Object(r),
        raw: s.encode_to_vec(),
        read_at,
    }
}

pub fn cost_row(
    account_id: i64,
    symbol_id: i64,
    run_id: i64,
    measured_at: DateTime<Utc>,
    window: Duration,
    sample: Option<&SpreadSamples>,
    sym: Option<&ProtoOaSymbol>,
) -> CostRow {
    let mut r = Map::new();
    let spread_median = sample.and_then(|s| s.median());
    let spread_samples = sample.map(|s| s.spreads.len()).unwrap_or(0) as i32;
    if spread_median.is_none() {
        r.insert(
            "spread_median".into(),
            json!(format!(
                "no quote with both bid and ask in the {} s window (market closed or no quotes)",
                window.as_secs()
            )),
        );
    }
    r.insert(
        "commission_per_lot_side".into(),
        json!("not derived in phase 1: the broker gives preciseTradingCommissionRate in units of commissionType; money per lot needs prices and conversion rates"),
    );
    let market = match sym {
        Some(s) => market_state_of(s, measured_at),
        None => jiaoyi_core::schedule::MarketState::Unknown("symbol details not read".into()),
    };
    let mut row = CostRow {
        account_id,
        broker_symbol_id: symbol_id,
        load_run_id: Some(run_id),
        measured_at,
        market_open: market.is_open(),
        market_state_reason: market.reason().map(String::from),
        spread_median,
        spread_samples,
        spread_window_secs: window.as_secs() as i32,
        source: "ctrader ProtoOASymbol + ProtoOASpotEvent".into(),
        ..Default::default()
    };
    match sym {
        None => {
            for c in [
                "commission_type",
                "precise_trading_commission_rate",
                "commission_deprecated",
                "precise_min_commission",
                "min_commission_type",
                "min_commission_asset",
                "swap_long",
                "swap_short",
                "swap_calculation_type",
                "swap_rollover_3days",
                "swap_period_hours",
                "swap_time_minutes",
            ] {
                r.insert(c.into(), json!("symbol details not read"));
            }
        }
        Some(s) => {
            row.commission_type = enum_name(s.commission_type, |e: ProtoOaCommissionType| {
                e.as_str_name()
            });
            row.precise_trading_commission_rate = s.precise_trading_commission_rate;
            #[allow(deprecated)]
            {
                row.commission_deprecated = s.commission;
            }
            row.precise_min_commission = s.precise_min_commission;
            row.min_commission_type =
                enum_name(s.min_commission_type, |e: ProtoOaMinCommissionType| {
                    e.as_str_name()
                });
            row.min_commission_asset = s.min_commission_asset.clone();
            row.swap_long = s.swap_long;
            row.swap_short = s.swap_short;
            row.swap_calculation_type =
                enum_name(s.swap_calculation_type, |e: ProtoOaSwapCalculationType| {
                    e.as_str_name()
                });
            row.swap_rollover_3days =
                enum_name(s.swap_rollover3_days, |e: ProtoOaDayOfWeek| e.as_str_name());
            row.swap_period_hours = s.swap_period;
            row.swap_time_minutes = s.swap_time;
            note_null(
                &mut r,
                "commission_type",
                "commissionType (proto2 default USD_PER_MILLION_USD)",
                &row.commission_type,
            );
            note_null(
                &mut r,
                "precise_trading_commission_rate",
                "preciseTradingCommissionRate",
                &row.precise_trading_commission_rate,
            );
            note_null(
                &mut r,
                "commission_deprecated",
                "commission (deprecated)",
                &row.commission_deprecated,
            );
            note_null(
                &mut r,
                "precise_min_commission",
                "preciseMinCommission",
                &row.precise_min_commission,
            );
            note_null(
                &mut r,
                "min_commission_type",
                "minCommissionType (proto2 default CURRENCY)",
                &row.min_commission_type,
            );
            note_null(
                &mut r,
                "min_commission_asset",
                "minCommissionAsset (proto2 default USD)",
                &row.min_commission_asset,
            );
            note_null(&mut r, "swap_long", "swapLong", &row.swap_long);
            note_null(&mut r, "swap_short", "swapShort", &row.swap_short);
            note_null(
                &mut r,
                "swap_calculation_type",
                "swapCalculationType (proto2 default PIPS)",
                &row.swap_calculation_type,
            );
            note_null(
                &mut r,
                "swap_rollover_3days",
                "swapRollover3Days (proto2 default MONDAY)",
                &row.swap_rollover_3days,
            );
            note_null(
                &mut r,
                "swap_period_hours",
                "swapPeriod",
                &row.swap_period_hours,
            );
            note_null(
                &mut r,
                "swap_time_minutes",
                "swapTime",
                &row.swap_time_minutes,
            );
        }
    }
    row.null_reasons = Value::Object(r);
    row
}

/// Whether the market was open at `at`, from the symbol's own schedule.
pub fn market_state_of(s: &ProtoOaSymbol, at: DateTime<Utc>) -> jiaoyi_core::schedule::MarketState {
    use jiaoyi_core::schedule::{market_state, Holiday, Interval};
    let intervals: Vec<Interval> = s
        .schedule
        .iter()
        .map(|i| Interval {
            start_second: i.start_second,
            end_second: i.end_second,
        })
        .collect();
    let holidays: Vec<Holiday> = s
        .holiday
        .iter()
        .map(|h| Holiday {
            name: h.name.clone(),
            schedule_time_zone: h.schedule_time_zone.clone(),
            date_days: h.holiday_date,
            recurring: h.is_recurring,
            start_second: h.start_second,
            end_second: h.end_second,
        })
        .collect();
    market_state(&intervals, &holidays, s.schedule_time_zone.as_deref(), at)
}

/// The tradability of a stored symbol, for the verdict: listed as enabled and
/// in trading mode ENABLED (an absent tradingMode is the proto2 default ENABLED).
pub fn tradability(
    listed_enabled: Option<bool>,
    trading_mode: Option<&str>,
) -> (Option<bool>, Option<String>) {
    let mode = trading_mode.unwrap_or("ENABLED");
    match listed_enabled {
        Some(false) => (
            Some(false),
            Some("the symbol list marks it not enabled".into()),
        ),
        _ if mode != "ENABLED" => (Some(false), Some(format!("tradingMode {mode}"))),
        Some(true) => (Some(true), None),
        None => (None, None),
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Open a TLS connection for an environment.
pub async fn connect(
    broker: &BrokerConfig,
    env: jiaoyi_core::allowlist::Environment,
) -> Result<Connection, ClientError> {
    Connection::connect_tls(broker.host(env), broker.port).await
}
