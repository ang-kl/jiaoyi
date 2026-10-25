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
    /// `open`, `tp1` or `session_end`.
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

/// An order's answer.
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

/// Whether an order of `kind` was already decided for a position.
pub async fn order_for_position(
    pool: &PgPool,
    account_id: i64,
    position_id: i64,
    kind: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM strategy3_orders
                        WHERE account_id = $1 AND position_id = $2 AND kind = $3)",
    )
    .bind(account_id)
    .bind(position_id)
    .bind(kind)
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

/// `GET /trading/costs`: Strategy 3's actual costs and order-to-fill times,
/// per account and symbol, from its own fills. Prices and milliseconds only:
/// the commission is in price units (deposit money ÷ the order's recorded
/// quote → deposit rate), never money.
pub async fn costs(pool: &PgPool) -> Result<Value, sqlx::Error> {
    let fills = sqlx::query(
        "SELECT f.account_id, f.symbol, f.kind, f.outcome, f.latency_ms, f.slippage, f.spread,
                f.decision_price, f.position_id, f.volume, o.detail
         FROM strategy3_fills f JOIN strategy3_orders o ON o.entry_key = f.entry_key
         ORDER BY f.id",
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
    type Key = (i64, String);
    #[derive(Default)]
    struct Acc {
        entries: u64,
        exits: u64,
        not_filled: u64,
        entry_latency: Vec<f64>,
        exit_latency: Vec<f64>,
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
        let key = (a, r.get::<String, _>("symbol"));
        let e = by.entry(key).or_default();
        let kind: String = r.get("kind");
        let outcome: String = r.get("outcome");
        let lat: Option<i64> = r.get("latency_ms");
        let slip: Option<f64> = r.get("slippage");
        let spread: Option<f64> = r.get("spread");
        let price: Option<f64> = r.get("decision_price");
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
            e.exit_latency.extend(lat.map(|v| v as f64));
            e.exit_slippage.extend(slip);
            e.spread_exit.extend(spread);
        }
    }
    let rows: Vec<Value> = by
        .into_iter()
        .map(|((account_id, symbol), e)| {
            json!({
                "account_id": account_id,
                "symbol": symbol,
                "entries_filled": e.entries,
                "exits_filled": e.exits,
                "orders_not_filled": e.not_filled,
                "entry_order_to_fill_ms_median": median(e.entry_latency),
                "exit_order_to_fill_ms_median": median(e.exit_latency),
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
        "strategy": "Strategy 3: intraday momentum",
        "units": {
            "prices": "price units of the symbol",
            "slippage": "fill price against the quote at the decision (the ask for a buy, the bid for a sell); positive is adverse",
            "order_to_fill_ms": "from the frame's send to the broker's fill answer",
            "commission": "the broker's deal commissions of the position (all deals so far) per unit of the opening volume, in price units (deposit money ÷ the order's quote → deposit rate); pending until history sync stores the deals",
        },
        "rows": rows,
    }))
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
