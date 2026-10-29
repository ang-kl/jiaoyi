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
    /// `open`, `tp1`, `session_end`, `broker_exit` or `trail`.
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

/// The registered session cost veto survives reconnects and process restarts.
/// Legacy v6 decisions have the same scope but predate the typed skip code.
pub async fn cost_skipped_in_session(
    pool: &PgPool,
    account_id: i64,
    symbol: &str,
    session_date: NaiveDate,
    prereg_hash: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM trading_decisions
         WHERE account_id=$1 AND symbol=$2 AND kind='s3_skip'
           AND detail->>'session_date'=$3 AND detail->>'prereg_hash'=$4
           AND detail->>'label'='jiaoyi-s3'
           AND (detail->>'skip_code'='cost_ge_r'
                OR reason LIKE 'cost reserve C (%) >= R (%): barrier too far for a no-drift process; skipped'))",
    )
    .bind(account_id)
    .bind(symbol)
    .bind(session_date.to_string())
    .bind(prereg_hash)
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

/// Open entry keys for this account, symbol and pre-registration hash.
/// Kelly reads only closes under the hash it is trading.
pub async fn open_keys_for_hash(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    prereg_hash: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT entry_key FROM strategy3_orders
         WHERE account_id = $1 AND symbol_id = $2 AND prereg_hash = $3 AND kind = 'open'",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(prereg_hash)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.get("entry_key")).collect())
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

// Codex · № 17,217 26-Oct (ordered: O-1, account-owned correlated closes).
/// Durable evidence of a Strategy 3 opening fill, including after reconcile.
pub async fn has_open_fill(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM strategy3_fills
        WHERE account_id = $1 AND position_id = $2 AND kind = 'open'
          AND outcome IN ('filled', 'partly_filled', 'reconciled_fill', 'reconciled_partial_fill') AND volume > 0)",
    )
    .bind(account_id)
    .bind(position_id)
    .fetch_one(pool)
    .await
}

/// An exact broker deal received for a local close, even before its summary.
/// Client ids are connection-local and must not serve as durable identity.
pub async fn is_local_close_deal(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
    deal_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trading_intents i
         JOIN trading_events e ON e.entry_key=i.entry_key AND e.account_id=i.account_id
         WHERE i.account_id=$1 AND i.position_id=$2 AND i.kind='close'
           AND e.deal_id=$3 AND e.position_id=$2
           AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL'))",
    )
    .bind(account_id)
    .bind(position_id)
    .bind(deal_id)
    .fetch_one(pool)
    .await
}

/// Exact local deal identity and unsettled close accounting in one snapshot.
/// Missing summaries and receipt-volume gaps remain unknown; replaying a deal
/// receipt must not make a partially persisted summary look fully covered.
/// A concurrent receipt/summary commit cannot fall between these decisions.
pub async fn close_accounting(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
    deal_id: i64,
) -> Result<(bool, bool), sqlx::Error> {
    sqlx::query_as(
        "WITH local_closes AS (
            SELECT entry_key FROM trading_intents
            WHERE account_id=$1 AND position_id=$2 AND kind='close'
            UNION
            SELECT entry_key FROM strategy3_orders
            WHERE account_id=$1 AND position_id=$2 AND kind IN ('tp1','session_end')
            UNION
            SELECT entry_key FROM strategy3_fills
            WHERE account_id=$1 AND position_id=$2 AND kind IN ('tp1','session_end'))
         SELECT EXISTS (
            SELECT 1 FROM trading_intents i
            JOIN trading_events e ON e.entry_key=i.entry_key AND e.account_id=i.account_id
            WHERE i.account_id=$1 AND i.position_id=$2 AND i.kind='close'
              AND e.deal_id=$3 AND e.position_id=$2
              AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL')) AS local_deal,
         EXISTS (
            SELECT 1 FROM local_closes c
            LEFT JOIN LATERAL (
                SELECT f.outcome,f.volume FROM strategy3_fills f
                WHERE f.account_id=$1 AND f.position_id=$2 AND f.entry_key=c.entry_key
                  AND f.kind IN ('tp1','session_end')
                ORDER BY f.id DESC LIMIT 1
            ) summary ON true
            WHERE summary.outcome IS NULL
               OR summary.outcome NOT IN ('filled','partly_filled','rejected')
               OR (summary.outcome IN ('partly_filled','rejected') AND NOT EXISTS (
                    SELECT 1 FROM trading_events e
                    WHERE e.account_id=$1 AND e.entry_key=c.entry_key
                      AND e.kind IN ('ORDER_FILLED','ORDER_REJECTED','ORDER_CANCELLED',
                                     'ORDER_EXPIRED','ORDER_ERROR','ERROR_RES')))
               OR (summary.outcome IN ('filled','partly_filled') AND (
                    COALESCE(summary.volume,0) <= 0
                    OR summary.volume > COALESCE((
                        SELECT SUM(receipt.volume) FROM (
                            SELECT DISTINCT ON (e.deal_id) e.volume
                            FROM trading_events e
                            WHERE e.account_id=$1 AND e.position_id=$2
                              AND e.entry_key=c.entry_key AND e.deal_id IS NOT NULL
                              AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL')
                            ORDER BY e.deal_id,e.received_at DESC,e.id DESC
                        ) receipt WHERE receipt.volume > 0
                    ),0))))",
    )
    .bind(account_id)
    .bind(position_id)
    .bind(deal_id)
    .fetch_one(pool)
    .await
}

// Codex · № 17,225 26-Oct (ordered: O-1, recover from broker history).
pub struct HistoryExit {
    pub deal_id: i64,
    pub order_id: i64,
    pub position_id: i64,
    pub symbol_id: i64,
    pub symbol: String,
    pub side: Option<String>,
    pub entry_key: String,
    pub prereg_hash: String,
    pub session_date: Option<NaiveDate>,
    pub executed_at: DateTime<Utc>,
    pub fill_price: Option<f64>,
    pub closed_volume: Option<i64>,
    pub filled_volume: i64,
}

/// Absence is necessary, but a confirmed account-owned closing deal is required.
pub async fn history_exits(
    pool: &PgPool,
    account_id: i64,
    broker_positions: &[i64],
) -> Result<Vec<HistoryExit>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT d.deal_id,d.order_id,d.position_id,d.broker_symbol_id,
                d.executed_at,d.execution_price,d.close_closed_volume,d.filled_volume,
                opened.entry_key,opened.prereg_hash,opened.symbol,opened.side,opened.session_date
         FROM deals d
         JOIN LATERAL (SELECT f.entry_key,f.prereg_hash,f.symbol,f.side,o.session_date
                       FROM strategy3_fills f JOIN strategy3_orders o ON o.entry_key=f.entry_key
                       WHERE f.account_id=d.account_id AND o.account_id=d.account_id
                         AND f.position_id=d.position_id AND o.symbol_id=d.broker_symbol_id
                         AND f.kind='open' AND f.volume > 0
                         AND f.outcome IN ('filled','partly_filled','reconciled_fill','reconciled_partial_fill')
                       ORDER BY f.id LIMIT 1) opened ON true
         WHERE d.account_id=$1 AND d.position_id <> ALL($2)
           AND d.is_closing AND d.deal_status IN ('FILLED','PARTIALLY_FILLED') AND d.filled_volume > 0
           AND NOT EXISTS (SELECT 1 FROM strategy3_orders recorded
                           WHERE recorded.account_id=d.account_id
                             AND recorded.entry_key='s3-broker:'||d.account_id||':'||d.position_id||':'||d.deal_id)
         ORDER BY d.executed_at,d.deal_id"
    ).bind(account_id).bind(broker_positions).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| HistoryExit {
            deal_id: r.get("deal_id"),
            order_id: r.get("order_id"),
            position_id: r.get("position_id"),
            symbol_id: r.get("broker_symbol_id"),
            symbol: r.get("symbol"),
            side: r.get("side"),
            entry_key: r.get("entry_key"),
            prereg_hash: r.get("prereg_hash"),
            session_date: r.get("session_date"),
            executed_at: r.get("executed_at"),
            fill_price: r.get("execution_price"),
            closed_volume: r.get("close_closed_volume"),
            filled_volume: r.get("filled_volume"),
        })
        .collect())
}
