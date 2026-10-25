//! Storage for Strategy 3 (migration 0013): each order as decided, with its
//! intent, and each order's answer. Both are append-only.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

/// An order as decided, written with its intent before the frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderRow {
    pub account_id: i64,
    pub entry_key: String,
    pub label: String,
    pub prereg_hash: String,
    /// `open`, `tp1`, `session_end` or `broker_exit`.
    pub kind: String,
    pub symbol: String,
    pub symbol_id: i64,
    pub position_id: Option<i64>,
    pub side: Option<String>,
    pub session_date: Option<NaiveDate>,
    pub decided_at: DateTime<Utc>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<i64>,
    pub detail: Value,
}

pub async fn record_order(pool: &PgPool, o: &OrderRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO strategy3_orders (account_id, entry_key, label, prereg_hash, kind, symbol,
            symbol_id, position_id, side, session_date, decided_at, bid, ask, volume, detail)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
    )
    .bind(o.account_id)
    .bind(&o.entry_key)
    .bind(&o.label)
    .bind(&o.prereg_hash)
    .bind(&o.kind)
    .bind(&o.symbol)
    .bind(o.symbol_id)
    .bind(o.position_id)
    .bind(&o.side)
    .bind(o.session_date)
    .bind(o.decided_at)
    .bind(o.bid)
    .bind(o.ask)
    .bind(o.volume)
    .bind(&o.detail)
    .execute(pool)
    .await?;
    Ok(())
}

/// An order's answer: the row shape both strategies' fills tables share.
pub use crate::trading::FillRow;

pub async fn record_fill(pool: &PgPool, f: &FillRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO strategy3_fills (account_id, entry_key, label, prereg_hash, kind, symbol,
            position_id, side, outcome, sent_at, answered_at, latency_ms, decision_price,
            fill_price, slippage, spread, volume, detail)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
    )
    .bind(f.account_id)
    .bind(&f.entry_key)
    .bind(&f.label)
    .bind(&f.prereg_hash)
    .bind(&f.kind)
    .bind(&f.symbol)
    .bind(f.position_id)
    .bind(&f.side)
    .bind(&f.outcome)
    .bind(f.sent_at)
    .bind(f.answered_at)
    .bind(f.latency_ms)
    .bind(f.decision_price)
    .bind(f.fill_price)
    .bind(f.slippage)
    .bind(f.spread)
    .bind(f.volume)
    .bind(&f.detail)
    .execute(pool)
    .await?;
    Ok(())
}

/// A broker-executed exit (a stop, a take-profit, or a close made outside
/// this program), recorded once per deal: the order row stands for the
/// broker's own order, so `costs` reads it with the program's closes.
/// Returns whether it was new.
// Claude · № 17,020 25-Oct (Codex P2 on #15): broker exits are costs too.
pub async fn record_broker_exit(
    pool: &PgPool,
    o: &OrderRow,
    f: &FillRow,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO strategy3_orders (account_id, entry_key, label, prereg_hash, kind, symbol,
            symbol_id, position_id, side, session_date, decided_at, bid, ask, volume, detail)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
         ON CONFLICT (entry_key) DO NOTHING",
    )
    .bind(o.account_id)
    .bind(&o.entry_key)
    .bind(&o.label)
    .bind(&o.prereg_hash)
    .bind(&o.kind)
    .bind(&o.symbol)
    .bind(o.symbol_id)
    .bind(o.position_id)
    .bind(&o.side)
    .bind(o.session_date)
    .bind(o.decided_at)
    .bind(o.bid)
    .bind(o.ask)
    .bind(o.volume)
    .bind(&o.detail)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if inserted {
        sqlx::query(
            "INSERT INTO strategy3_fills (account_id, entry_key, label, prereg_hash, kind, symbol,
                position_id, side, outcome, sent_at, answered_at, latency_ms, decision_price,
                fill_price, slippage, spread, volume, detail)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
        )
        .bind(f.account_id)
        .bind(&f.entry_key)
        .bind(&f.label)
        .bind(&f.prereg_hash)
        .bind(&f.kind)
        .bind(&f.symbol)
        .bind(f.position_id)
        .bind(&f.side)
        .bind(&f.outcome)
        .bind(f.sent_at)
        .bind(f.answered_at)
        .bind(f.latency_ms)
        .bind(f.decision_price)
        .bind(f.fill_price)
        .bind(f.slippage)
        .bind(f.spread)
        .bind(f.volume)
        .bind(&f.detail)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(inserted)
}

/// The plan of a filled Strategy 3 open: its decision detail, and the fill.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenPlan {
    pub entry_key: String,
    pub position_id: i64,
    pub detail: Value,
    pub fill_price: Option<f64>,
}

/// The plans of the account's filled Strategy 3 opens, by position.
pub async fn open_plans(pool: &PgPool, account_id: i64) -> Result<Vec<OpenPlan>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT o.entry_key, f.position_id, o.detail, f.fill_price
         FROM strategy3_orders o JOIN strategy3_fills f ON f.entry_key = o.entry_key
         WHERE o.account_id = $1 AND o.kind = 'open' AND f.position_id IS NOT NULL
         ORDER BY f.id",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| OpenPlan {
            entry_key: r.get("entry_key"),
            position_id: r.get("position_id"),
            detail: r.get("detail"),
            fill_price: r.get("fill_price"),
        })
        .collect())
}

/// Whether a position's TP1 close is done: one was filled, partly filled,
/// or left unresolved (no answer, or no answer recorded). Only rejected
/// attempts leave it open for another try.
// Claude · № 17,020 25-Oct (Codex P2 on #15): a rejection is not a TP1.
pub async fn tp1_done(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM strategy3_orders o
                        LEFT JOIN strategy3_fills f ON f.entry_key = o.entry_key
                        WHERE o.account_id = $1 AND o.position_id = $2 AND o.kind = 'tp1'
                          AND f.outcome IS DISTINCT FROM 'rejected')",
    )
    .bind(account_id)
    .bind(position_id)
    .fetch_one(pool)
    .await
}

/// How many orders of `kind` were decided for a position.
pub async fn orders_for_position(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
    kind: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM strategy3_orders
         WHERE account_id = $1 AND position_id = $2 AND kind = $3",
    )
    .bind(account_id)
    .bind(position_id)
    .bind(kind)
    .fetch_one(pool)
    .await
}

/// The number of Strategy 3 opens decided on a symbol in a session (the next
/// entry's number).
pub async fn opens_in_session(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    session_date: NaiveDate,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM strategy3_orders
         WHERE account_id = $1 AND symbol_id = $2 AND session_date = $3 AND kind = 'open'",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(session_date)
    .fetch_one(pool)
    .await
}

/// How many recorded quotes the median spread reads, at most.
pub const SPREAD_QUOTES: i64 = 50_000;

/// The median ask − bid of the account's last [`SPREAD_QUOTES`] recorded
/// two-sided quotes on a symbol within 7 days, in price units, and how many
/// were read. `None` without one.
pub async fn median_spread(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
) -> Result<Option<(f64, i64)>, sqlx::Error> {
    let r = sqlx::query(
        "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY s) AS m, count(*) AS n FROM (
            SELECT (ask_e5 - bid_e5)::float8 / 100000.0 AS s FROM ticks
            WHERE account_id = $1 AND broker_symbol_id = $2
              AND received_at > now() - interval '7 days'
              AND bid_e5 IS NOT NULL AND ask_e5 IS NOT NULL
            ORDER BY received_at DESC LIMIT $3) q",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(SPREAD_QUOTES)
    .fetch_one(pool)
    .await?;
    let n: i64 = r.get("n");
    let m: Option<f64> = r.get("m");
    Ok(m.filter(|_| n > 0).map(|m| (m, n)))
}

/// `/health` → `strategy3`: per account, the open plans' positions and today's
/// decisions and trades (UTC day). No money.
pub async fn today(pool: &PgPool, account_id: i64) -> Result<Value, sqlx::Error> {
    let decisions = sqlx::query(
        "SELECT at, symbol, kind, reason FROM trading_decisions
         WHERE account_id = $1 AND kind LIKE 's3\\_%' AND at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
         ORDER BY id DESC LIMIT 20",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let counts = sqlx::query(
        "SELECT kind, count(*) AS n FROM trading_decisions
         WHERE account_id = $1 AND kind LIKE 's3\\_%' AND at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
         GROUP BY kind ORDER BY kind",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let trades = sqlx::query(
        "SELECT recorded_at, symbol, kind, side, outcome, fill_price, volume, latency_ms, slippage
         FROM strategy3_fills
         WHERE account_id = $1 AND recorded_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
         ORDER BY id DESC LIMIT 50",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(json!({
        "decisions_today": counts.iter().map(|r| json!({
            "kind": r.get::<String, _>("kind"), "count": r.get::<i64, _>("n"),
        })).collect::<Vec<_>>(),
        "recent_decisions": decisions.iter().map(|r| json!({
            "at": r.get::<DateTime<Utc>, _>("at"),
            "symbol": r.get::<Option<String>, _>("symbol"),
            "kind": r.get::<String, _>("kind"),
            "reason": r.get::<String, _>("reason"),
        })).collect::<Vec<_>>(),
        "trades_today": trades.iter().map(|r| json!({
            "at": r.get::<DateTime<Utc>, _>("recorded_at"),
            "symbol": r.get::<String, _>("symbol"),
            "kind": r.get::<String, _>("kind"),
            "side": r.get::<Option<String>, _>("side"),
            "outcome": r.get::<String, _>("outcome"),
            "price": r.get::<Option<f64>, _>("fill_price"),
            "volume": r.get::<Option<i64>, _>("volume"),
            "order_to_fill_ms": r.get::<Option<i64>, _>("latency_ms"),
            "slippage": r.get::<Option<f64>, _>("slippage"),
        })).collect::<Vec<_>>(),
    }))
}
