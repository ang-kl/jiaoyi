//! `/research/inventory`: what data exists for the edge verification, per
//! account and symbol, and what is missing (owner-approved plan, step 1).
//!
//! Counts and dates only: no money figure is shown on this unauthenticated
//! route.

use chrono::{DateTime, NaiveDate, Utc};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use jiaoyi_store::{self as store};
use serde::Serialize;

use crate::history::{grid, GRID_ORIGIN_MS, WEEK_MS};

#[derive(Debug, Serialize)]
pub struct Inventory {
    pub generated_at: DateTime<Utc>,
    pub settings_version: i64,
    pub notes: Vec<&'static str>,
    pub accounts: Vec<AccountInventory>,
}

#[derive(Debug, Serialize)]
pub struct AccountInventory {
    pub account_id: i64,
    pub environment: String,
    pub registration_at: Option<DateTime<Utc>>,
    pub trader_read_at: Option<DateTime<Utc>>,
    pub deals: DealCoverage,
    pub symbols: Vec<SymbolInventory>,
    pub missing: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct DealCoverage {
    pub deals: i64,
    pub positions: i64,
    pub first_executed_at: Option<DateTime<Utc>>,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub windows_ok: i64,
    pub windows_failed: i64,
    pub last_fetched_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SymbolInventory {
    pub name: String,
    pub asset_class: String,
    pub resolution: Option<String>,
    pub broker_symbol_id: Option<i64>,
    pub daily_bars: Coverage,
    pub live_ticks: Coverage,
    pub history_bid: SideCoverage,
    pub history_ask: SideCoverage,
    pub probe: ProbeResult,
    pub overlap_check: Option<OverlapCheck>,
    pub deals: i64,
    pub deals_first_at: Option<DateTime<Utc>>,
    pub deals_last_at: Option<DateTime<Utc>>,
    pub missing: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct Coverage {
    pub count: i64,
    pub first: Option<DateTime<Utc>>,
    pub last: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Default)]
pub struct SideCoverage {
    pub ticks_stored: i64,
    pub first_tick: Option<DateTime<Utc>>,
    pub last_tick: Option<DateTime<Utc>>,
    /// The download range the settings ask for.
    pub requested_from: Option<NaiveDate>,
    pub weeks_requested: usize,
    pub weeks_complete: usize,
    /// Weeks fetched completely that held no tick at all.
    pub weeks_empty: Vec<NaiveDate>,
    /// Weeks in the requested range not yet fetched completely.
    pub weeks_missing: Vec<NaiveDate>,
    pub weeks_failed: Vec<NaiveDate>,
    /// The longest silences between consecutive stored ticks (weekends included).
    pub largest_gaps: Vec<Gap>,
    pub delta_signs_seen: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Gap {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub hours: f64,
}

#[derive(Debug, Serialize, Default)]
pub struct ProbeResult {
    pub probed_at: Option<DateTime<Utc>>,
    pub weeks_probed: i64,
    /// The earliest week whose bid history held ticks.
    pub earliest_week_with_ticks: Option<NaiveDate>,
    /// The latest probed week before that with none.
    pub latest_empty_week_before: Option<NaiveDate>,
    pub method: &'static str,
}

/// Recorded live bids in the overlap of both records, checked against the
/// broker's history: same price within ±2 s.
#[derive(Debug, Serialize)]
pub struct OverlapCheck {
    pub sampled: i64,
    pub matched: i64,
}

pub async fn build(pool: &PgPool) -> store::Result<Inventory> {
    let accounts = sqlx::query(
        "SELECT account_id, environment FROM accounts WHERE enabled ORDER BY account_id",
    )
    .fetch_all(pool)
    .await?;
    let global = store::settings::snapshot(pool, None).await?;
    let mut out = Vec::new();
    for a in accounts {
        let account_id: i64 = a.get("account_id");
        out.push(account_inventory(pool, account_id, a.get("environment")).await?);
    }
    Ok(Inventory {
        generated_at: Utc::now(),
        settings_version: global.version,
        notes: vec![
            "History windows are Monday-to-Monday UTC weeks (the broker's one-week limit per tick request).",
            "A week fetched completely with no ticks is listed as empty; a closed market (holidays) shows that way too.",
            "The probe assumes the broker's tick history is continuous from its first week; it binary-searches the earliest week with bid ticks.",
            "largest_gaps include weekends and session breaks; they are not errors by themselves.",
        ],
        accounts: out,
    })
}

async fn account_inventory(
    pool: &PgPool,
    account_id: i64,
    environment: String,
) -> store::Result<AccountInventory> {
    let s = store::settings::snapshot(pool, Some(account_id)).await?;
    let days_back = s
        .integer("history.tick_days_back")
        .map_err(store::StoreError::Settings)?;
    let universe = s
        .universe("universe.symbols")
        .map_err(store::StoreError::Settings)?;
    let snap = sqlx::query(
        "SELECT registration_at, read_at FROM account_snapshots WHERE account_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    let d = sqlx::query(
        "SELECT count(*) AS n, count(DISTINCT position_id) AS p, min(executed_at) AS f, max(executed_at) AS l
         FROM deals WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let w = sqlx::query(
        "SELECT count(*) FILTER (WHERE status = 'ok') AS ok, count(*) FILTER (WHERE status = 'failed') AS failed,
                max(fetched_at) AS last
         FROM (SELECT DISTINCT ON (window_from) status, fetched_at FROM deal_fetches
               WHERE account_id = $1 ORDER BY window_from, fetched_at DESC) x",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let deals = DealCoverage {
        deals: d.get("n"),
        positions: d.get("p"),
        first_executed_at: d.get("f"),
        last_executed_at: d.get("l"),
        windows_ok: w.get("ok"),
        windows_failed: w.get("failed"),
        last_fetched_at: w.get("last"),
    };
    let resolutions = store::latest_resolutions(pool, account_id).await?;
    let now = Utc::now();
    let mut symbols = Vec::new();
    for (name, class) in &universe {
        let r = resolutions.iter().find(|r| &r.name == name);
        let id = r.and_then(|r| r.broker_symbol_id);
        symbols.push(
            symbol_inventory(
                pool,
                account_id,
                name,
                class.as_str(),
                r,
                id,
                days_back,
                now,
            )
            .await?,
        );
    }
    let mut missing = Vec::new();
    if snap.is_none() {
        missing.push("trader record not read yet: no history sync has run".to_string());
    }
    if deals.windows_ok == 0 {
        missing.push("deal history not fetched yet".to_string());
    }
    if deals.windows_failed > 0 {
        missing.push(format!(
            "{} deal-history windows failed",
            deals.windows_failed
        ));
    }
    Ok(AccountInventory {
        account_id,
        environment,
        registration_at: snap.as_ref().and_then(|r| r.get("registration_at")),
        trader_read_at: snap.as_ref().map(|r| r.get("read_at")),
        deals,
        symbols,
        missing,
    })
}

#[allow(clippy::too_many_arguments)]
async fn symbol_inventory(
    pool: &PgPool,
    account_id: i64,
    name: &str,
    class: &str,
    resolution: Option<&store::ResolutionRow>,
    id: Option<i64>,
    days_back: i64,
    now: DateTime<Utc>,
) -> store::Result<SymbolInventory> {
    let mut inv = SymbolInventory {
        name: name.to_string(),
        asset_class: class.to_string(),
        resolution: resolution.map(|r| r.status.clone()),
        broker_symbol_id: id,
        daily_bars: Coverage::default(),
        live_ticks: Coverage::default(),
        history_bid: SideCoverage::default(),
        history_ask: SideCoverage::default(),
        probe: ProbeResult {
            method: "binary search over weeks for the earliest with bid ticks",
            ..Default::default()
        },
        overlap_check: None,
        deals: 0,
        deals_first_at: None,
        deals_last_at: None,
        missing: vec![],
    };
    let Some(id) = id else {
        inv.missing.push(match resolution {
            None => "not resolved yet: no data load has run".into(),
            Some(r) => format!(
                "not on the account: {}",
                r.reason.clone().unwrap_or_else(|| r.status.clone())
            ),
        });
        return Ok(inv);
    };

    let b = sqlx::query(
        "SELECT count(*) AS n, min(bar_open_utc) AS f, max(bar_open_utc) AS l FROM daily_bars
         WHERE account_id = $1 AND broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(pool)
    .await?;
    inv.daily_bars = Coverage {
        count: b.get("n"),
        first: b.get("f"),
        last: b.get("l"),
    };

    let t = sqlx::query(
        "SELECT count(*) AS n, min(received_at) AS f, max(received_at) AS l FROM ticks
         WHERE account_id = $1 AND broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(pool)
    .await?;
    inv.live_ticks = Coverage {
        count: t.get("n"),
        first: t.get("f"),
        last: t.get("l"),
    };

    let from_ms = now.timestamp_millis() - days_back.max(1) * 86_400_000;
    let weeks = grid(from_ms, now.timestamp_millis(), GRID_ORIGIN_MS, WEEK_MS);
    inv.history_bid = side_coverage(pool, account_id, id, "bid", &weeks, from_ms).await?;
    inv.history_ask = side_coverage(pool, account_id, id, "ask", &weeks, from_ms).await?;

    let p = sqlx::query(
        "SELECT max(fetched_at) AS at, count(*) AS n,
                min(window_from) FILTER (WHERE status = 'ok' AND ticks > 0) AS earliest
         FROM history_fetches WHERE account_id = $1 AND broker_symbol_id = $2 AND kind = 'probe'",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(pool)
    .await?;
    inv.probe.probed_at = p.get("at");
    inv.probe.weeks_probed = p.get("n");
    let earliest: Option<DateTime<Utc>> = p.get("earliest");
    inv.probe.earliest_week_with_ticks = earliest.map(|d| d.date_naive());
    if let Some(e) = earliest {
        let before: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT max(window_from) FROM history_fetches
             WHERE account_id = $1 AND broker_symbol_id = $2 AND kind = 'probe'
               AND status = 'ok' AND ticks = 0 AND window_from < $3",
        )
        .bind(account_id)
        .bind(id)
        .bind(e)
        .fetch_one(pool)
        .await?;
        inv.probe.latest_empty_week_before = before.map(|d| d.date_naive());
    }

    // Overlap: up to 200 recorded live bids inside the history's span.
    if let (Some(hf), Some(hl)) = (inv.history_bid.first_tick, inv.history_bid.last_tick) {
        let o = sqlx::query(
            "WITH s AS (
                SELECT received_at, bid_e5 FROM ticks
                WHERE account_id = $1 AND broker_symbol_id = $2 AND bid_e5 IS NOT NULL
                  AND received_at BETWEEN $3 AND $4
                ORDER BY received_at LIMIT 200)
             SELECT count(*) AS sampled,
                    count(*) FILTER (WHERE EXISTS (
                        SELECT 1 FROM ticks_history h
                        WHERE h.account_id = $1 AND h.broker_symbol_id = $2 AND h.side = 'bid'
                          AND h.price_e5 = s.bid_e5
                          AND h.ts BETWEEN s.received_at - interval '2 seconds'
                                       AND s.received_at + interval '2 seconds')) AS matched
             FROM s",
        )
        .bind(account_id)
        .bind(id)
        .bind(hf)
        .bind(hl)
        .fetch_one(pool)
        .await?;
        let sampled: i64 = o.get("sampled");
        if sampled > 0 {
            inv.overlap_check = Some(OverlapCheck {
                sampled,
                matched: o.get("matched"),
            });
        }
    }

    let dl = sqlx::query(
        "SELECT count(*) AS n, min(executed_at) AS f, max(executed_at) AS l FROM deals
         WHERE account_id = $1 AND broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(id)
    .fetch_one(pool)
    .await?;
    inv.deals = dl.get("n");
    inv.deals_first_at = dl.get("f");
    inv.deals_last_at = dl.get("l");

    let m = &mut inv.missing;
    if inv.daily_bars.count == 0 {
        m.push("no daily bars stored".into());
    }
    for (label, side) in [("bid", &inv.history_bid), ("ask", &inv.history_ask)] {
        if side.weeks_complete == 0 {
            m.push(format!("no {label} tick history fetched yet"));
        } else if !side.weeks_missing.is_empty() {
            m.push(format!(
                "{} {label} weeks not fetched yet",
                side.weeks_missing.len()
            ));
        }
        if !side.weeks_failed.is_empty() {
            m.push(format!("{} {label} weeks failed", side.weeks_failed.len()));
        }
    }
    if inv.probe.probed_at.is_none() {
        m.push("tick-history depth not probed yet".into());
    } else if inv.probe.earliest_week_with_ticks.is_none() {
        m.push("probe found no bid ticks in any week".into());
    }
    if inv.live_ticks.count == 0 {
        m.push("no live ticks recorded".into());
    }
    if let Some(o) = &inv.overlap_check {
        if o.matched < o.sampled {
            m.push(format!(
                "history and live bids disagree: {} of {} sampled live bids have no history tick at the same price within 2 s",
                o.sampled - o.matched,
                o.sampled
            ));
        }
    }
    if inv.deals == 0 {
        m.push("no deals on this account for this symbol".into());
    }
    Ok(inv)
}

async fn side_coverage(
    pool: &PgPool,
    account_id: i64,
    id: i64,
    side: &str,
    weeks: &[(i64, i64)],
    from_ms: i64,
) -> store::Result<SideCoverage> {
    let rows = sqlx::query(
        "SELECT DISTINCT ON (window_from) window_from, status, complete, ticks
         FROM history_fetches
         WHERE account_id = $1 AND broker_symbol_id = $2 AND side = $3 AND kind = 'download'
         ORDER BY window_from, (status = 'ok' AND complete) DESC, fetched_at DESC",
    )
    .bind(account_id)
    .bind(id)
    .bind(side)
    .fetch_all(pool)
    .await?;
    let date = |ms: i64| {
        DateTime::<Utc>::from_timestamp_millis(ms)
            .unwrap_or_default()
            .date_naive()
    };
    let mut c = SideCoverage {
        requested_from: Some(date(from_ms)),
        weeks_requested: weeks.len(),
        ..Default::default()
    };
    for (start, _) in weeks {
        let at = DateTime::<Utc>::from_timestamp_millis(*start).unwrap_or_default();
        match rows
            .iter()
            .find(|r| r.get::<DateTime<Utc>, _>("window_from") == at)
        {
            Some(r) if r.get::<String, _>("status") == "ok" && r.get::<bool, _>("complete") => {
                c.weeks_complete += 1;
                if r.get::<i32, _>("ticks") == 0 {
                    c.weeks_empty.push(date(*start));
                }
            }
            Some(r) if r.get::<String, _>("status") == "failed" => {
                c.weeks_failed.push(date(*start));
                c.weeks_missing.push(date(*start));
            }
            _ => c.weeks_missing.push(date(*start)),
        }
    }
    let agg = sqlx::query(
        "SELECT coalesce(sum(stored), 0)::bigint AS stored, min(first_tick) AS f, max(last_tick) AS l,
                array_remove(array_agg(DISTINCT s), NULL) AS signs
         FROM history_fetches LEFT JOIN LATERAL unnest(delta_signs) AS s ON TRUE
         WHERE account_id = $1 AND broker_symbol_id = $2 AND side = $3 AND kind = 'download' AND status = 'ok'",
    )
    .bind(account_id)
    .bind(id)
    .bind(side)
    .fetch_one(pool)
    .await?;
    c.ticks_stored = agg.get("stored");
    c.first_tick = agg.get("f");
    c.last_tick = agg.get("l");
    c.delta_signs_seen = agg
        .get::<Option<Vec<String>>, _>("signs")
        .unwrap_or_default();
    if let (Some(f), Some(l)) = (c.first_tick, c.last_tick) {
        let gaps = sqlx::query(
            "SELECT prev, ts FROM (
                SELECT ts, lag(ts) OVER (ORDER BY ts, dup) AS prev FROM ticks_history
                WHERE account_id = $1 AND broker_symbol_id = $2 AND side = $3 AND ts BETWEEN $4 AND $5
             ) g WHERE prev IS NOT NULL ORDER BY ts - prev DESC LIMIT 3",
        )
        .bind(account_id)
        .bind(id)
        .bind(side)
        .bind(f)
        .bind(l)
        .fetch_all(pool)
        .await?;
        c.largest_gaps = gaps
            .into_iter()
            .map(|r| {
                let (a, b): (DateTime<Utc>, DateTime<Utc>) = (r.get("prev"), r.get("ts"));
                Gap {
                    from: a,
                    to: b,
                    hours: (b - a).num_milliseconds() as f64 / 3_600_000.0,
                }
            })
            .collect();
    }
    Ok(c)
}
