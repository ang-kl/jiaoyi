//! Storage for the order path (migration 0012): intents written before each
//! frame, every execution message, the trader's decisions, and the settings
//! changes applied from `config/settings-changes.json`. And what reads them
//! back (migration 0014): Strategy 1's fills, both strategies' costs, and the
//! order list of `GET /trading/orders`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
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
/// trader both receive each answer) is kept once: `false`.
pub async fn record_event(pool: &PgPool, e: &EventRow) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
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
    Ok(r.rows_affected() == 1)
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
              AND e.kind IN ('ORDER_FILLED', 'ORDER_PARTIAL_FILL', 'RECONCILED_FILL',
                             'RECONCILED_PARTIAL_FILL'))",
    )
    .bind(account_id)
    .bind(position_id)
    .fetch_one(pool)
    .await
}

/// Event kinds that fill an open intent, in part or in full.
/// `RECONCILED_PARTIAL_FILL`: reconcile found part of an unresolved open at
/// the broker (Codex round 4 on #14); its remainder stays unresolved.
pub const FILL_KINDS: &[&str] = &[
    "ORDER_FILLED",
    "ORDER_PARTIAL_FILL",
    "RECONCILED_FILL",
    "RECONCILED_PARTIAL_FILL",
];
/// Event kinds that fill an open intent in full. A partial fill does not:
/// its remainder may still fill (Codex round 3 on #14).
pub const FULL_FILL_KINDS: &[&str] = &["ORDER_FILLED", "RECONCILED_FILL"];
/// Event kinds that end an open intent's remainder without a fill.
/// `RECONCILED_ABSENT`: reconcile found it neither filled nor pending at the
/// broker after its grace.
pub const END_KINDS: &[&str] = &[
    "ORDER_REJECTED",
    "ORDER_CANCELLED",
    "ORDER_EXPIRED",
    "ORDER_ERROR",
    "ERROR_RES",
    "RECONCILED_ABSENT",
];

/// An open intent with neither a full fill nor an end recorded: an order the
/// broker may still fill, in whole or in its remainder (accepted only, no
/// answer at all, or partly filled).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnresolvedIntent {
    pub entry_key: String,
    pub client_msg_id: String,
    pub symbol_id: Option<i64>,
    /// The volume requested, in cents.
    pub volume: Option<i64>,
    /// The volume filled so far, in cents, as a lower bound: the larger of
    /// its recorded deals' sum and the broker position reconcile adopted it
    /// at. That position is a snapshot that can include a deal whose own
    /// event is recorded after it, so the two are never added (Codex on #16).
    pub filled: i64,
    /// The position its partial fills went into, if any.
    pub position_id: Option<i64>,
    /// The risk the gate recomputed for the whole volume, in the deposit currency.
    pub risk: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// An account's unresolved open intents: reconcile matches them by label and
/// symbol, the open-risk budget holds their outstanding risk, they are never
/// resent.
pub async fn open_intents_unresolved(
    pool: &PgPool,
    account_id: i64,
) -> Result<Vec<UnresolvedIntent>, sqlx::Error> {
    let done: Vec<&str> = FULL_FILL_KINDS.iter().chain(END_KINDS).copied().collect();
    // Claude · № 17,044 25-Oct (ordered: owner "merge when green and deploy";
    // Codex P2 on #16, a reconciled snapshot and its late deal counted twice)
    let rows = sqlx::query(
        "SELECT i.entry_key, i.client_msg_id, i.symbol_id, i.volume, i.risk, i.created_at,
                GREATEST(
                    COALESCE((SELECT SUM(e.volume) FROM trading_events e
                              WHERE e.entry_key = i.entry_key
                                AND e.kind IN ('ORDER_PARTIAL_FILL', 'ORDER_FILLED')), 0),
                    COALESCE((SELECT MAX(e.volume) FROM trading_events e
                              WHERE e.entry_key = i.entry_key
                                AND e.kind = 'RECONCILED_PARTIAL_FILL'), 0))::BIGINT AS filled,
                (SELECT MAX(e.position_id) FROM trading_events e
                 WHERE e.entry_key = i.entry_key
                   AND e.kind IN ('ORDER_PARTIAL_FILL', 'RECONCILED_PARTIAL_FILL')) AS position_id
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
            volume: r.get("volume"),
            filled: r.get("filled"),
            position_id: r.get("position_id"),
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
         FROM trading_decisions WHERE kind NOT LIKE 's3\\_%' ORDER BY account_id, id DESC",
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

// ------------------------------------------------------------- the instrument
// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")

/// An order's answer, as `strategy1_fills` and `strategy3_fills` record it:
/// when it was sent and answered, the fill against the quote at the
/// decision (slippage, positive = adverse), the spread then, the volume.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FillRow {
    pub account_id: i64,
    pub entry_key: String,
    pub label: String,
    pub prereg_hash: String,
    pub kind: String,
    pub symbol: String,
    pub position_id: Option<i64>,
    pub side: Option<String>,
    pub outcome: String,
    pub sent_at: DateTime<Utc>,
    pub answered_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<i64>,
    pub decision_price: Option<f64>,
    pub fill_price: Option<f64>,
    pub slippage: Option<f64>,
    pub spread: Option<f64>,
    pub volume: Option<i64>,
    pub detail: Value,
}

/// Record a Strategy 1 order's answer (`strategy1_fills`), once per entry
/// key. `false`: already recorded (a broker exit's deal seen twice).
pub async fn record_s1_fill(pool: &PgPool, f: &FillRow) -> Result<bool, sqlx::Error> {
    let r = sqlx::query(
        "INSERT INTO strategy1_fills (account_id, entry_key, label, prereg_hash, kind, symbol,
            position_id, side, outcome, sent_at, answered_at, latency_ms, decision_price,
            fill_price, slippage, spread, volume, detail)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
         ON CONFLICT (entry_key) DO NOTHING",
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
    Ok(r.rows_affected() == 1)
}

/// Strategy 1's trades today (UTC day) on an account, newest first, from its
/// fills: the same columns as Strategy 3's. No money.
pub async fn s1_trades_today(pool: &PgPool, account_id: i64) -> Result<Value, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT recorded_at, symbol, kind, side, outcome, fill_price, volume, latency_ms, slippage
         FROM strategy1_fills
         WHERE account_id = $1 AND recorded_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
         ORDER BY id DESC LIMIT 50",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(json!(rows
        .iter()
        .map(|r| json!({
            "at": r.get::<DateTime<Utc>, _>("recorded_at"),
            "symbol": r.get::<String, _>("symbol"),
            "kind": r.get::<String, _>("kind"),
            "side": r.get::<Option<String>, _>("side"),
            "outcome": r.get::<String, _>("outcome"),
            "price": r.get::<Option<f64>, _>("fill_price"),
            "volume": r.get::<Option<i64>, _>("volume"),
            "order_to_fill_ms": r.get::<Option<i64>, _>("latency_ms"),
            "slippage": r.get::<Option<f64>, _>("slippage"),
        }))
        .collect::<Vec<_>>()))
}

fn median(mut v: Vec<f64>) -> Option<f64> {
    v.retain(|x| x.is_finite());
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    })
}

/// The program's strategies whose orders are read back: code, label.
pub const STRATEGIES: [(&str, &str); 2] = [("s1", "jiaoyi-s1"), ("s3", "jiaoyi-s3")];

fn label_of(strategy: &str) -> &'static str {
    STRATEGIES
        .iter()
        .find(|(s, _)| *s == strategy)
        .map_or("", |(_, l)| l)
}

/// `GET /trading/costs`: each strategy's actual costs and order-to-fill
/// times, per strategy, account and symbol, from its own fills. Prices and
/// milliseconds only: the commission is in price units (deposit money ÷ the
/// order's recorded quote → deposit rate), never money.
pub async fn costs(pool: &PgPool) -> Result<Value, sqlx::Error> {
    // Strategy 3's quote → deposit rate is on its order; Strategy 1's on its fill.
    let fills = sqlx::query(
        "SELECT 's1' AS strategy, f.account_id, f.symbol, f.kind, f.outcome, f.latency_ms,
                f.slippage, f.spread, f.decision_price, f.position_id, f.volume, f.detail
         FROM strategy1_fills f
         UNION ALL
         SELECT 's3', f.account_id, f.symbol, f.kind, f.outcome, f.latency_ms, f.slippage,
                f.spread, f.decision_price, f.position_id, f.volume, o.detail
         FROM strategy3_fills f JOIN strategy3_orders o ON o.entry_key = f.entry_key",
    )
    .fetch_all(pool)
    .await?;
    // Commission per position from the broker's deals: |commission| in
    // deposit money, and the volume of the opening deal.
    let deals = sqlx::query(
        "SELECT d.account_id, d.position_id,
                sum(abs(d.commission)::float8 / power(10, d.money_digits)) AS commission
         FROM deals d
         WHERE d.commission IS NOT NULL AND d.money_digits IS NOT NULL
           AND (d.account_id, d.position_id) IN
               (SELECT account_id, position_id FROM strategy3_fills
                WHERE kind = 'open' AND position_id IS NOT NULL
                UNION
                SELECT account_id, position_id FROM strategy1_fills
                WHERE kind = 'open' AND position_id IS NOT NULL)
         GROUP BY d.account_id, d.position_id",
    )
    .fetch_all(pool)
    .await?;
    let commission = |a: i64, p: i64| -> Option<f64> {
        deals
            .iter()
            .find(|d| d.get::<i64, _>("account_id") == a && d.get::<i64, _>("position_id") == p)
            .and_then(|d| d.get::<Option<f64>, _>("commission"))
    };
    type Key = (String, i64, String);
    #[derive(Default)]
    struct Acc {
        entries: u64,
        exits: u64,
        broker_exits: u64,
        not_filled: u64,
        stop_amends: u64,
        entry_latency: Vec<f64>,
        exit_latency: Vec<f64>,
        amend_latency: Vec<f64>,
        slippage: Vec<f64>,
        slippage_bps: Vec<f64>,
        exit_slippage: Vec<f64>,
        spread_entry: Vec<f64>,
        spread_exit: Vec<f64>,
        commission_per_unit: Vec<f64>,
        commission_pending: u64,
    }
    let mut by: std::collections::BTreeMap<Key, Acc> = std::collections::BTreeMap::new();
    for r in &fills {
        let a: i64 = r.get("account_id");
        let key = (
            r.get::<String, _>("strategy"),
            a,
            r.get::<String, _>("symbol"),
        );
        let e = by.entry(key).or_default();
        let kind: String = r.get("kind");
        let outcome: String = r.get("outcome");
        let lat: Option<i64> = r.get("latency_ms");
        let slip: Option<f64> = r.get("slippage");
        let spread: Option<f64> = r.get("spread");
        let price: Option<f64> = r.get("decision_price");
        // A trailing-stop amend moves a stop; it fills nothing.
        if kind == "trail" {
            e.stop_amends += 1;
            e.amend_latency.extend(lat.map(|v| v as f64));
            continue;
        }
        if outcome != "filled" {
            e.not_filled += 1;
            continue;
        }
        if kind == "open" {
            e.entries += 1;
            e.entry_latency.extend(lat.map(|v| v as f64));
            e.slippage.extend(slip);
            if let (Some(s), Some(p)) = (slip, price) {
                e.slippage_bps.push(s / p * 10_000.0);
            }
            e.spread_entry.extend(spread);
            let detail: Value = r.get("detail");
            let rate = detail["quote_to_deposit"].as_f64();
            let volume: Option<i64> = r.get("volume");
            match (r.get::<Option<i64>, _>("position_id"), rate, volume) {
                (Some(p), Some(rate), Some(v)) if rate > 0.0 && v > 0 => match commission(a, p) {
                    Some(c) => e.commission_per_unit.push(c / rate / (v as f64 / 100.0)),
                    None => e.commission_pending += 1,
                },
                _ => e.commission_pending += 1,
            }
        } else {
            e.exits += 1;
            if kind == "broker_exit" {
                e.broker_exits += 1;
            }
            e.exit_latency.extend(lat.map(|v| v as f64));
            e.exit_slippage.extend(slip);
            e.spread_exit.extend(spread);
        }
    }
    let rows: Vec<Value> = by
        .into_iter()
        .map(|((strategy, account_id, symbol), e)| {
            json!({
                "strategy": strategy,
                "label": label_of(&strategy),
                "account_id": account_id,
                "symbol": symbol,
                "entries_filled": e.entries,
                "exits_filled": e.exits,
                "broker_exits_filled": e.broker_exits,
                "orders_not_filled": e.not_filled,
                "stop_amends": e.stop_amends,
                "entry_order_to_fill_ms_median": median(e.entry_latency),
                "exit_order_to_fill_ms_median": median(e.exit_latency),
                "stop_amend_ms_median": median(e.amend_latency),
                "entry_slippage_median": median(e.slippage),
                "entry_slippage_bps_median": median(e.slippage_bps),
                "exit_slippage_median": median(e.exit_slippage),
                "spread_at_entry_median": median(e.spread_entry),
                "spread_at_exit_median": median(e.spread_exit),
                "commission_round_trip_per_unit_median": median(e.commission_per_unit),
                "commission_pending": e.commission_pending,
            })
        })
        .collect();
    Ok(json!({
        "strategies": {
            "s1": "Strategy 1: tick-triggered trend (label jiaoyi-s1)",
            "s3": "Strategy 3: intraday momentum (label jiaoyi-s3)",
        },
        "units": {
            "prices": "price units of the symbol",
            "slippage": "fill price against the quote at the decision (the ask for a buy, the bid for a sell); for a broker exit, against the stop or take-profit level it executed; positive is adverse",
            "order_to_fill_ms": "from the intent's record, just before the frame, to the broker's fill answer (none for a broker exit: no order was sent)",
            "broker_exits": "stops and take-profits the broker executed, one per deal, counted in exits_filled; the spread is the last quote's when the event arrived",
            "stop_amends": "Strategy 1's trailing-stop amends: counted apart with their order-to-answer ms; they fill nothing",
            "commission": "the broker's deal commissions of the position (all deals so far) per unit of the opening volume, in price units (deposit money ÷ the order's quote → deposit rate); pending until history sync stores the deals",
        },
        "rows": rows,
    }))
}

/// `GET /trading/orders`: what to list.
#[derive(Debug, Clone, PartialEq)]
pub struct OrdersQuery {
    pub account_id: Option<i64>,
    /// `s1` or `s3`; `None` is both.
    pub strategy: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: i64,
}

/// The decision an order was placed under: the newest decision recorded
/// with the order's entry key.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderDecision {
    pub kind: String,
    pub reason: String,
    pub at: DateTime<Utc>,
    /// The signal's inputs at the decision, where recorded (prices, ATR,
    /// EMAs, the quote, the sizing mode): an allowlist, never money.
    pub signal: Value,
}

/// One order of `GET /trading/orders`: an intent of Strategy 1 or 3, or an
/// exit the broker executed itself. Prices, volumes and milliseconds only.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderView {
    pub entry_key: String,
    /// `s1` or `s3`.
    pub strategy: String,
    pub label: String,
    pub account_id: i64,
    pub symbol: Option<String>,
    /// The gate's kind (`open`, `amend_stop`, `close`), or `broker_exit`.
    pub kind: String,
    /// What the order was for, from the strategy's fills (`open`, `trail`,
    /// `tp1`, `session_end`, `broker_exit`), when recorded.
    pub order: Option<String>,
    pub side: Option<String>,
    /// The volume requested, in cents (none for an amend or a broker exit).
    pub requested_volume: Option<i64>,
    pub created_at: DateTime<Utc>,
    /// The last execution message's kind (`ORDER_FILLED`, `ORDER_CANCELLED`,
    /// `RECONCILED_FILL`…), or `filled` for a broker exit; none without an answer.
    pub outcome: Option<String>,
    /// Filled so far, in cents: the deals' sum (each deal once), or the
    /// position reconcile adopted, whichever is larger; the requested volume
    /// when the open was found filled in full at the broker. None for an amend.
    pub filled_volume: Option<i64>,
    /// The deals' volume-weighted price, or the adopted position's price.
    pub fill_price: Option<f64>,
    pub position_id: Option<i64>,
    pub latency_ms: Option<i64>,
    pub slippage: Option<f64>,
    pub spread: Option<f64>,
    pub decision: Option<OrderDecision>,
}

/// The decision detail's keys an order's `signal` shows: none is money.
pub const SIGNAL_KEYS: [&str; 12] = [
    "level",
    "atr",
    "ema_fast",
    "ema_slow",
    "bid",
    "ask",
    "after",
    "side",
    "relative_stop",
    "relative_take_profit",
    "sizing",
    "reason",
];

fn signal_of(detail: Option<Value>) -> Value {
    let mut out = serde_json::Map::new();
    if let Some(Value::Object(m)) = detail {
        for k in SIGNAL_KEYS {
            if let Some(v) = m.get(k) {
                out.insert(k.to_string(), v.clone());
            }
        }
    }
    Value::Object(out)
}

/// `GET /trading/orders`: the newest `limit` orders of Strategy 1 and
/// Strategy 3 (intents and broker exits), newest first.
pub async fn orders(pool: &PgPool, q: &OrdersQuery) -> Result<Vec<OrderView>, sqlx::Error> {
    let intents = sqlx::query(
        "SELECT i.account_id, i.entry_key, i.kind, i.side, i.volume, i.created_at,
                i.position_id AS intent_position,
                COALESCE(i.symbol_id, sym.symbol_id) AS symbol_id,
                ev.last_kind, dl.filled, dl.vwap, dl.deal_position,
                rc.volume AS rec_volume, rc.price AS rec_price, rc.position_id AS rec_position,
                EXISTS (SELECT 1 FROM trading_events e WHERE e.entry_key = i.entry_key
                        AND e.kind IN ('ORDER_FILLED', 'RECONCILED_FILL')) AS full,
                f.order_kind, f.symbol AS fill_symbol, f.latency_ms, f.slippage, f.spread,
                d.kind AS d_kind, d.reason AS d_reason, d.at AS d_at, d.detail AS d_detail,
                (SELECT s.name FROM symbols s WHERE s.account_id = i.account_id
                   AND s.broker_symbol_id = COALESCE(i.symbol_id, sym.symbol_id)) AS symbol_name
         FROM trading_intents i
         LEFT JOIN LATERAL (SELECT e.symbol_id FROM trading_events e
                            WHERE e.entry_key = i.entry_key AND e.symbol_id IS NOT NULL
                            ORDER BY e.id LIMIT 1) sym ON true
         LEFT JOIN LATERAL (SELECT e.kind AS last_kind FROM trading_events e
                            WHERE e.entry_key = i.entry_key ORDER BY e.id DESC LIMIT 1) ev ON true
         LEFT JOIN LATERAL (
             SELECT SUM(x.volume)::BIGINT AS filled,
                    SUM(x.price * x.volume) / NULLIF(SUM(x.volume)::float8, 0) AS vwap,
                    MAX(x.position_id) AS deal_position
             FROM (SELECT DISTINCT ON (e.deal_id) e.volume, e.price, e.position_id
                   FROM trading_events e
                   WHERE e.entry_key = i.entry_key
                     AND e.kind IN ('ORDER_FILLED', 'ORDER_PARTIAL_FILL')
                     AND e.deal_id IS NOT NULL
                   ORDER BY e.deal_id, e.id) x) dl ON true
         LEFT JOIN LATERAL (SELECT e.volume, e.price, e.position_id FROM trading_events e
                            WHERE e.entry_key = i.entry_key
                              AND e.kind IN ('RECONCILED_FILL', 'RECONCILED_PARTIAL_FILL')
                              AND e.volume IS NOT NULL
                            ORDER BY e.id DESC LIMIT 1) rc ON true
         LEFT JOIN LATERAL (SELECT * FROM (
                SELECT kind AS order_kind, symbol, latency_ms, slippage, spread
                FROM strategy1_fills WHERE entry_key = i.entry_key
                UNION ALL
                SELECT kind, symbol, latency_ms, slippage, spread
                FROM strategy3_fills WHERE entry_key = i.entry_key) u LIMIT 1) f ON true
         LEFT JOIN LATERAL (SELECT d.kind, d.reason, d.at, d.detail FROM trading_decisions d
                            WHERE d.account_id = i.account_id AND d.detail ? 'entry_key'
                              AND d.detail->>'entry_key' = i.entry_key
                            ORDER BY d.id DESC LIMIT 1) d ON true
         WHERE (i.entry_key LIKE 's1:%' OR i.entry_key LIKE 's1-%'
                OR i.entry_key LIKE 's3:%' OR i.entry_key LIKE 's3-%')
           AND ($1::BIGINT IS NULL OR i.account_id = $1)
           AND ($2::TEXT IS NULL OR i.entry_key LIKE $2 || ':%' OR i.entry_key LIKE $2 || '-%')
           AND ($3::TIMESTAMPTZ IS NULL OR i.created_at >= $3)
         ORDER BY i.created_at DESC, i.id DESC
         LIMIT $4",
    )
    .bind(q.account_id)
    .bind(q.strategy.as_deref())
    .bind(q.since)
    .bind(q.limit)
    .fetch_all(pool)
    .await?;
    let mut out: Vec<OrderView> = intents
        .into_iter()
        .map(|r| {
            let key: String = r.get("entry_key");
            let strategy = if key.starts_with("s1") { "s1" } else { "s3" };
            let kind: String = r.get("kind");
            let requested: Option<i64> = r.get("volume");
            let deals: i64 = r.get::<Option<i64>, _>("filled").unwrap_or(0);
            let rec: Option<i64> = r.get("rec_volume");
            let full: bool = r.get("full");
            // An amend fills nothing.
            let filled_volume = (kind != "amend_stop").then(|| {
                let f = deals.max(rec.unwrap_or(0));
                // Filled in full: at least the requested volume, also when the
                // full fill was found by reconcile with no position read after
                // earlier partial deals (Codex P2 on #18).
                // Claude · № 17,120 25-Oct (ordered: owner "merge when green
                // and deploy"; Codex P2 on #18)
                if full {
                    f.max(requested.unwrap_or(0))
                } else {
                    f
                }
            });
            let vwap: Option<f64> = r.get("vwap");
            let decision = r.get::<Option<String>, _>("d_kind").map(|k| OrderDecision {
                kind: k,
                reason: r.get("d_reason"),
                at: r.get("d_at"),
                signal: signal_of(r.get("d_detail")),
            });
            OrderView {
                entry_key: key.clone(),
                strategy: strategy.into(),
                label: label_of(strategy).into(),
                account_id: r.get("account_id"),
                symbol: r
                    .get::<Option<String>, _>("fill_symbol")
                    .or(r.get("symbol_name")),
                kind,
                order: r.get("order_kind"),
                side: r.get("side"),
                requested_volume: requested,
                created_at: r.get("created_at"),
                outcome: r.get("last_kind"),
                filled_volume,
                fill_price: vwap.or(r.get("rec_price")),
                position_id: r
                    .get::<Option<i64>, _>("deal_position")
                    .or(r.get("rec_position"))
                    .or(r.get("intent_position")),
                latency_ms: r.get("latency_ms"),
                slippage: r.get("slippage"),
                spread: r.get("spread"),
                decision,
            }
        })
        .collect();
    // The broker's own exits: no intent, one fill row per deal.
    let exits = sqlx::query(
        "SELECT * FROM (
             SELECT 's1' AS strategy, account_id, entry_key, symbol, side, position_id, volume,
                    fill_price, slippage, spread, sent_at, detail
             FROM strategy1_fills WHERE kind = 'broker_exit'
             UNION ALL
             SELECT 's3', account_id, entry_key, symbol, side, position_id, volume,
                    fill_price, slippage, spread, sent_at, detail
             FROM strategy3_fills WHERE kind = 'broker_exit') x
         WHERE ($1::BIGINT IS NULL OR account_id = $1)
           AND ($2::TEXT IS NULL OR strategy = $2)
           AND ($3::TIMESTAMPTZ IS NULL OR sent_at >= $3)
         ORDER BY sent_at DESC LIMIT $4",
    )
    .bind(q.account_id)
    .bind(q.strategy.as_deref())
    .bind(q.since)
    .bind(q.limit)
    .fetch_all(pool)
    .await?;
    for r in exits {
        let strategy: String = r.get("strategy");
        let at: DateTime<Utc> = r.get("sent_at");
        let detail: Value = r.get("detail");
        out.push(OrderView {
            entry_key: r.get("entry_key"),
            label: label_of(&strategy).into(),
            strategy,
            account_id: r.get("account_id"),
            symbol: Some(r.get("symbol")),
            kind: "broker_exit".into(),
            order: Some("broker_exit".into()),
            side: r.get("side"),
            requested_volume: None,
            created_at: at,
            outcome: Some("filled".into()),
            filled_volume: r.get("volume"),
            fill_price: r.get("fill_price"),
            position_id: r.get("position_id"),
            latency_ms: None,
            slippage: r.get("slippage"),
            spread: r.get("spread"),
            decision: Some(OrderDecision {
                kind: "broker_exit".into(),
                reason: detail["reason"].as_str().unwrap_or("other").to_string(),
                at,
                signal: signal_of(Some(detail.clone())),
            }),
        });
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    out.truncate(q.limit.max(0) as usize);
    Ok(out)
}

// ---------------------------------------------------------------- performance

// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)

/// One closed position of a strategy, in R: the broker's net money for it
/// over the risk the gate recorded for its open.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClosedTrade {
    pub entry_key: String,
    pub position_id: i64,
    pub closed_at: DateTime<Utc>,
    pub r: f64,
}

/// A position's deal, as the history sync stores it (table `deals`).
#[derive(Debug, Clone, PartialEq)]
pub struct DealMoney {
    pub is_closing: bool,
    pub filled_volume: i64,
    pub executed_at: DateTime<Utc>,
    pub money_digits: Option<i32>,
    pub close_gross_profit: Option<i64>,
    pub close_swap: Option<i64>,
    pub close_commission: Option<i64>,
    pub close_pnl_conversion_fee: Option<i64>,
    pub close_money_digits: Option<i32>,
}

/// A position's net money in the deposit currency from its deals, when it
/// is closed (the closing deals' volume equals the opening deals'): each
/// closing deal's gross profit + swap − |commission| − |P&L conversion fee|,
/// at its close money digits.
///
/// The closing deal's commission is both sides of the closed volume: on the
/// owner's broker the balance moves by the closing deal's net and not at the
/// open (measured on 47790949, bot-trade `commission-terms.js`
/// `CLOSING_DEAL_SIDES`), so the opening deal's commission is not taken
/// again. Commissions and fees are costs whatever their sign in the feed (as
/// `research::commission_per_unit` reads them). Returns (opened volume,
/// closed at, net money), or `Err(why)` when it is open or a money field is
/// missing or the closing quantity exceeds proven opening history.
pub fn position_money(deals: &[DealMoney]) -> Result<(i64, DateTime<Utc>, f64), String> {
    let opened: i64 = deals
        .iter()
        .filter(|d| !d.is_closing)
        .map(|d| d.filled_volume)
        .sum();
    let closed: i64 = deals
        .iter()
        .filter(|d| d.is_closing)
        .map(|d| d.filled_volume)
        .sum();
    if opened <= 0 {
        return Err("no opening deal stored yet".into());
    }
    if closed < opened {
        return Err("still open".into());
    }
    if closed > opened {
        // A reversal can contain both a close and a new open in one deal.
        // This closing/opening split cannot certify that mixed lifecycle.
        return Err("closing history exceeds proven opening quantity".into());
    }
    let mut net = 0.0;
    let mut closed_at = None;
    for d in deals {
        if d.is_closing {
            let digits = d
                .close_money_digits
                .or(d.money_digits)
                .ok_or("a closing deal has no money digits")?;
            if !(0..=18).contains(&digits) {
                return Err("closing monetary precision is outside 0..18".into());
            }
            let unit = 10f64.powi(digits);
            let gross = d
                .close_gross_profit
                .ok_or("a closing deal has no gross profit")?;
            // Absence is unknown, including an omitted optional broker fee.
            // Only an explicit zero proves that a charge was zero.
            let swap = d.close_swap.ok_or("a closing deal has no swap")?;
            let commission = d
                .close_commission
                .ok_or("a closing deal has no commission")?;
            let conversion_fee = d
                .close_pnl_conversion_fee
                .ok_or("a closing deal has no P&L conversion fee")?;
            net += (gross as f64 + swap as f64) / unit;
            net -= commission.unsigned_abs() as f64 / unit;
            net -= conversion_fee.unsigned_abs() as f64 / unit;
            closed_at = closed_at.max(Some(d.executed_at));
        }
    }
    if !net.is_finite() {
        return Err("closing net money is not finite".into());
    }
    Ok((opened, closed_at.ok_or("no closing deal")?, net))
}

/// A strategy's closed positions on an account, oldest close first, in net
/// R, and how many candidate lifecycles lack complete evidence (including
/// contradictory receipts, missing original risk or closing money). `prefix`: the strategy's open entry keys
/// (`s1:` or `s3:`). A position belongs to the open whose fill events name
/// it; its risk is the gate's for the whole requested volume, scaled by the
/// filled share.
pub async fn closed_trades(
    pool: &PgPool,
    account_id: i64,
    prefix: &str,
) -> Result<(Vec<ClosedTrade>, usize), sqlx::Error> {
    let evidence =
        crate::outcomes::read(pool, account_id, prefix.trim_end_matches(':'), Utc::now()).await?;
    Ok((evidence.trades, evidence.unmeasured))
}

/// A strategy's results on an account, in R only (no money).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Performance {
    pub trades: usize,
    /// Unresolved candidate lifecycles, excluded from the measured closes.
    pub unmeasured: usize,
    pub net_r: f64,
    /// Σ winning R ÷ |Σ losing R|; `None` with no losing trade.
    pub profit_factor: Option<f64>,
    /// The largest peak-to-trough fall of cumulative R (the peak starts at 0).
    pub max_drawdown_r: f64,
    /// The fall from the highest cumulative R so far to the last.
    pub current_drawdown_r: f64,
    pub last_closed_at: Option<DateTime<Utc>>,
}

/// The results of closed trades, oldest first.
pub fn summarize(trades: &[ClosedTrade], unmeasured: usize) -> Performance {
    let (mut cum, mut peak, mut max_dd) = (0.0f64, 0.0f64, 0.0f64);
    let (mut win, mut loss) = (0.0f64, 0.0f64);
    for t in trades {
        cum += t.r;
        peak = peak.max(cum);
        max_dd = max_dd.max(peak - cum);
        if t.r > 0.0 {
            win += t.r;
        } else {
            loss += -t.r;
        }
    }
    Performance {
        trades: trades.len(),
        unmeasured,
        net_r: cum,
        profit_factor: (loss > 0.0).then(|| win / loss),
        max_drawdown_r: max_dd,
        current_drawdown_r: peak - cum,
        last_closed_at: trades.last().map(|t| t.closed_at),
    }
}

/// A strategy's results on an account (`strategy`: `s1` or `s3`).
pub async fn performance(
    pool: &PgPool,
    account_id: i64,
    strategy: &str,
) -> Result<Performance, sqlx::Error> {
    let (trades, unmeasured) = closed_trades(pool, account_id, &format!("{strategy}:")).await?;
    Ok(summarize(&trades, unmeasured))
}

/// The latest limit decision of a strategy on an account (`kinds`: its
/// reached and cleared kinds): its kind and detail.
pub async fn latest_limit_decision(
    pool: &PgPool,
    account_id: i64,
    kinds: &[&str],
) -> Result<Option<(String, Value)>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT kind, detail FROM trading_decisions
         WHERE account_id = $1 AND kind = ANY($2) ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(kinds)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.get("kind"), r.get("detail"))))
}
