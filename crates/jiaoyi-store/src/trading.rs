//! Storage for the order path (migration 0012): intents written before each
//! frame, every execution message, the trader's decisions, and the settings
//! changes applied from `config/settings-changes.json`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

/// An intent, as the order gate writes it before the frame.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IntentRow {
    pub account_id: i64,
    pub entry_key: String,
    pub client_msg_id: String,
    pub kind: String,
    pub symbol_id: Option<i64>,
    pub position_id: Option<i64>,
    pub side: Option<String>,
    pub volume: Option<i64>,
    pub relative_stop: Option<i64>,
    pub stop_price: Option<f64>,
    pub take_profit: Option<f64>,
    pub risk: Option<f64>,
    pub max_risk: f64,
    pub settings_version: Option<i64>,
}

/// Write an intent. `false`: its entry key is already recorded (never sent again).
pub async fn record_intent(pool: &PgPool, i: &IntentRow) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO trading_intents (account_id, entry_key, client_msg_id, kind, symbol_id,
            position_id, side, volume, relative_stop, stop_price, take_profit, risk, max_risk,
            settings_version)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
         ON CONFLICT (entry_key) DO NOTHING",
    )
    .bind(i.account_id)
    .bind(&i.entry_key)
    .bind(&i.client_msg_id)
    .bind(&i.kind)
    .bind(i.symbol_id)
    .bind(i.position_id)
    .bind(&i.side)
    .bind(i.volume)
    .bind(i.relative_stop)
    .bind(i.stop_price)
    .bind(i.take_profit)
    .bind(i.risk)
    .bind(i.max_risk)
    .bind(i.settings_version)
    .execute(pool)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Whether an entry key is recorded.
pub async fn intent_exists(pool: &PgPool, entry_key: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM trading_intents WHERE entry_key = $1)")
        .bind(entry_key)
        .fetch_one(pool)
        .await
}

/// One execution message.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EventRow {
    pub account_id: i64,
    pub entry_key: Option<String>,
    pub client_msg_id: Option<String>,
    pub payload_type: i32,
    pub kind: String,
    pub position_id: Option<i64>,
    pub order_id: Option<i64>,
    pub deal_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub side: Option<String>,
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

/// Record an execution message. A message already recorded (the gate and the
/// trader both receive each answer) is kept once.
pub async fn record_event(pool: &PgPool, e: &EventRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO trading_events (account_id, entry_key, client_msg_id, payload_type, kind,
            position_id, order_id, deal_id, symbol_id, side, price, volume, stop_loss, label,
            error_code, description, is_server_event, raw, received_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
         ON CONFLICT DO NOTHING",
    )
    .bind(e.account_id)
    .bind(&e.entry_key)
    .bind(&e.client_msg_id)
    .bind(e.payload_type)
    .bind(&e.kind)
    .bind(e.position_id)
    .bind(e.order_id)
    .bind(e.deal_id)
    .bind(e.symbol_id)
    .bind(&e.side)
    .bind(e.price)
    .bind(e.volume)
    .bind(e.stop_loss)
    .bind(&e.label)
    .bind(&e.error_code)
    .bind(&e.description)
    .bind(e.is_server_event)
    .bind(&e.raw)
    .bind(e.received_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Whether this program opened the position: one of its own open intents
/// was filled into it.
pub async fn owns_position(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM trading_events e JOIN trading_intents i ON i.entry_key = e.entry_key
            WHERE e.account_id = $1 AND e.position_id = $2 AND i.kind = 'open'
              AND e.kind IN ('ORDER_FILLED', 'ORDER_PARTIAL_FILL', 'RECONCILED_FILL'))",
    )
    .bind(account_id)
    .bind(position_id)
    .fetch_one(pool)
    .await
}

/// Event kinds that fill an open intent.
pub const FILL_KINDS: &[&str] = &["ORDER_FILLED", "ORDER_PARTIAL_FILL", "RECONCILED_FILL"];
/// Event kinds that end an open intent without a fill. `RECONCILED_ABSENT`:
/// reconcile found it neither filled nor pending at the broker after its grace.
pub const END_KINDS: &[&str] = &[
    "ORDER_REJECTED",
    "ORDER_CANCELLED",
    "ORDER_EXPIRED",
    "ORDER_ERROR",
    "ERROR_RES",
    "RECONCILED_ABSENT",
];

/// An open intent with neither a fill nor an end recorded: an order the
/// broker may still fill (accepted only, or no answer at all).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnresolvedIntent {
    pub entry_key: String,
    pub client_msg_id: String,
    pub symbol_id: Option<i64>,
    /// The risk the gate recomputed for it, in the deposit currency.
    pub risk: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// An account's unresolved open intents: reconcile matches them by label and
/// symbol, the open-risk budget holds their risk, they are never resent.
pub async fn open_intents_unresolved(
    pool: &PgPool,
    account_id: i64,
) -> Result<Vec<UnresolvedIntent>, sqlx::Error> {
    let done: Vec<&str> = FILL_KINDS.iter().chain(END_KINDS).copied().collect();
    let rows = sqlx::query(
        "SELECT i.entry_key, i.client_msg_id, i.symbol_id, i.risk, i.created_at
         FROM trading_intents i
         WHERE i.account_id = $1 AND i.kind = 'open'
           AND NOT EXISTS (SELECT 1 FROM trading_events e
                           WHERE e.entry_key = i.entry_key AND e.kind = ANY($2))
         ORDER BY i.id",
    )
    .bind(account_id)
    .bind(&done)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UnresolvedIntent {
            entry_key: r.get("entry_key"),
            client_msg_id: r.get("client_msg_id"),
            symbol_id: r.get("symbol_id"),
            risk: r.get("risk"),
            created_at: r.get("created_at"),
        })
        .collect())
}

/// Record a decision of the trader.
pub async fn record_decision(
    pool: &PgPool,
    account_id: i64,
    symbol: Option<&str>,
    kind: &str,
    reason: &str,
    detail: &Value,
    settings_version: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO trading_decisions (account_id, symbol, kind, reason, detail, settings_version)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(account_id)
    .bind(symbol)
    .bind(kind)
    .bind(reason)
    .bind(detail)
    .bind(settings_version)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionRow {
    pub account_id: i64,
    pub at: DateTime<Utc>,
    pub symbol: Option<String>,
    pub kind: String,
    pub reason: String,
}

/// Each account's latest decision.
pub async fn latest_decisions(pool: &PgPool) -> Result<Vec<DecisionRow>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT DISTINCT ON (account_id) account_id, at, symbol, kind, reason
         FROM trading_decisions ORDER BY account_id, id DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| DecisionRow {
            account_id: r.get("account_id"),
            at: r.get("at"),
            symbol: r.get("symbol"),
            kind: r.get("kind"),
            reason: r.get("reason"),
        })
        .collect())
}

/// The newest held-out trial's verdict for a hypothesis under a
/// pre-registration hash, if any.
pub async fn held_out_verdict(
    pool: &PgPool,
    hypothesis: &str,
    prereg_hash: &str,
) -> Result<Option<Option<String>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT verdict FROM research_trials
         WHERE hypothesis = $1 AND sample = 'held_out' AND prereg_hash = $2
         ORDER BY trial_id DESC LIMIT 1",
    )
    .bind(hypothesis)
    .bind(prereg_hash)
    .fetch_optional(pool)
    .await
}

/// Whether a settings change from the seed file was applied.
pub async fn seed_applied(pool: &PgPool, seed_id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM settings_seed_applied WHERE seed_id = $1)")
        .bind(seed_id)
        .fetch_one(pool)
        .await
}

pub async fn mark_seed_applied(
    pool: &PgPool,
    seed_id: &str,
    version: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings_seed_applied (seed_id, version) VALUES ($1, $2)
         ON CONFLICT (seed_id) DO NOTHING",
    )
    .bind(seed_id)
    .bind(version)
    .execute(pool)
    .await?;
    Ok(())
}

/// The newest decision of one kind for an account: when, and its detail.
pub async fn latest_decision_detail(
    pool: &PgPool,
    account_id: i64,
    kind: &str,
) -> Result<Option<(DateTime<Utc>, Value)>, sqlx::Error> {
    let r = sqlx::query(
        "SELECT at, detail FROM trading_decisions WHERE account_id = $1 AND kind = $2
         ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(kind)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| (r.get("at"), r.get("detail"))))
}
