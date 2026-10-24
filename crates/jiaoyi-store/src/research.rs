//! Storage for the edge test's research engine (migration 0009): the
//! pre-registered jobs, their trials and the spread samples.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobRow {
    pub id: i64,
    pub hypothesis: String,
    pub stage: String,
    pub setting: Value,
    pub run_order: i32,
    pub prereg_hash: String,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

fn job(r: sqlx::postgres::PgRow) -> JobRow {
    JobRow {
        id: r.get("id"),
        hypothesis: r.get("hypothesis"),
        stage: r.get("stage"),
        setting: r.get("setting"),
        run_order: r.get("run_order"),
        prereg_hash: r.get("prereg_hash"),
        status: r.get("status"),
        reason: r.get("reason"),
        created_at: r.get("created_at"),
        started_at: r.get("started_at"),
        finished_at: r.get("finished_at"),
    }
}

const JOB_COLS: &str = "id, hypothesis, stage, setting, run_order, prereg_hash, status, reason,
                        created_at, started_at, finished_at";

pub async fn jobs(pool: &PgPool) -> crate::Result<Vec<JobRow>> {
    Ok(sqlx::query(&format!(
        "SELECT {JOB_COLS} FROM research_jobs ORDER BY run_order"
    ))
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(job)
    .collect())
}

/// The pending job that runs next (lowest run order).
pub async fn next_pending_job(pool: &PgPool) -> crate::Result<Option<JobRow>> {
    Ok(sqlx::query(&format!(
        "SELECT {JOB_COLS} FROM research_jobs WHERE status = 'pending' ORDER BY run_order LIMIT 1"
    ))
    .fetch_optional(pool)
    .await?
    .map(job))
}

/// Take a pending job (pending → running) and record the attempt with the
/// deployed commit. `None` when it is no longer pending.
pub async fn claim_job(pool: &PgPool, id: i64, commit: &str) -> crate::Result<Option<i32>> {
    let mut tx = pool.begin().await?;
    let taken = sqlx::query(
        "UPDATE research_jobs SET status = 'running', started_at = now(), reason = NULL
         WHERE id = $1 AND status = 'pending'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if taken != 1 {
        return Ok(None);
    }
    let attempt: i32 = sqlx::query_scalar(
        "INSERT INTO research_job_attempts (job_id, attempt, commit)
         SELECT $1, coalesce(max(attempt), 0) + 1, $2 FROM research_job_attempts WHERE job_id = $1
         RETURNING attempt",
    )
    .bind(id)
    .bind(commit)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(attempt))
}

/// One attempt at a job.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Attempt {
    pub attempt: i32,
    pub started_at: DateTime<Utc>,
    pub commit: String,
}

pub async fn job_attempts(pool: &PgPool, job_id: i64) -> crate::Result<Vec<Attempt>> {
    let rows = sqlx::query(
        "SELECT attempt, started_at, commit FROM research_job_attempts WHERE job_id = $1 ORDER BY attempt",
    )
    .bind(job_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Attempt {
            attempt: r.get("attempt"),
            started_at: r.get("started_at"),
            commit: r.get("commit"),
        })
        .collect())
}

/// Close a running job without a trial: `refused` or `failed`, with the reason.
pub async fn close_job(pool: &PgPool, id: i64, status: &str, reason: &str) -> crate::Result<()> {
    sqlx::query(
        "UPDATE research_jobs SET status = $2, reason = $3, finished_at = now()
         WHERE id = $1 AND status = 'running'",
    )
    .bind(id)
    .bind(status)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Jobs left `running` by a stopped service wrote nothing (a trial and the
/// job's end are one transaction). They go back to pending, with the reason,
/// and are attempted again. Returns how many.
pub async fn release_interrupted_jobs(pool: &PgPool) -> crate::Result<u64> {
    Ok(sqlx::query(
        "UPDATE research_jobs SET status = 'pending', started_at = NULL,
                reason = 'interrupted: the service stopped before the attempt wrote a result; attempted again'
         WHERE status = 'running'",
    )
    .execute(pool)
    .await?
    .rows_affected())
}

/// One trial of a pre-registered job.
pub struct JobTrial<'a> {
    pub job_id: i64,
    pub prereg_hash: &'a str,
    pub hypothesis: &'a str,
    pub sample: &'a str,
    pub params: &'a Value,
    pub data_range: &'a Value,
    pub results: &'a Value,
    pub verdict: &'a str,
    pub settings_version: i64,
}

/// Write a job's trial and mark the job done, in one transaction. A job has
/// at most one trial (unique `job_id`).
pub async fn finish_job_with_trial(pool: &PgPool, t: &JobTrial<'_>) -> crate::Result<i64> {
    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO research_trials (hypothesis, params, data_range, sample, results_json, verdict,
                                      settings_version, prereg_hash, job_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING trial_id",
    )
    .bind(t.hypothesis)
    .bind(t.params)
    .bind(t.data_range)
    .bind(t.sample)
    .bind(t.results)
    .bind(t.verdict)
    .bind(t.settings_version)
    .bind(t.prereg_hash)
    .bind(t.job_id)
    .fetch_one(&mut *tx)
    .await?;
    let done = sqlx::query(
        "UPDATE research_jobs SET status = 'done', reason = NULL, finished_at = now()
         WHERE id = $1 AND status = 'running'",
    )
    .bind(t.job_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if done != 1 {
        return Err(crate::StoreError::Settings(format!(
            "job {} is not running; its trial is not written",
            t.job_id
        )));
    }
    tx.commit().await?;
    Ok(id)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrialRow {
    pub trial_id: i64,
    pub job_id: Option<i64>,
    pub prereg_hash: Option<String>,
    pub hypothesis: String,
    pub sample: String,
    pub params: Value,
    pub data_range: Value,
    pub results: Option<Value>,
    pub verdict: Option<String>,
    pub settings_version: i64,
    pub run_at: DateTime<Utc>,
}

/// Trials written by pre-registered jobs, in job order.
pub async fn job_trials(pool: &PgPool) -> crate::Result<Vec<TrialRow>> {
    let rows = sqlx::query(
        "SELECT t.trial_id, t.job_id, t.prereg_hash, t.hypothesis, t.sample, t.params, t.data_range,
                t.results_json, t.verdict, t.settings_version, t.run_at
         FROM research_trials t JOIN research_jobs j ON j.id = t.job_id
         ORDER BY j.run_order",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| TrialRow {
            trial_id: r.get("trial_id"),
            job_id: r.get("job_id"),
            prereg_hash: r.get("prereg_hash"),
            hypothesis: r.get("hypothesis"),
            sample: r.get("sample"),
            params: r.get("params"),
            data_range: r.get("data_range"),
            results: r.get("results_json"),
            verdict: r.get("verdict"),
            settings_version: r.get("settings_version"),
            run_at: r.get("run_at"),
        })
        .collect())
}

/// One spread sample: a week of a symbol's broker ticks, streamed, never
/// stored raw.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpreadSample {
    pub account_id: i64,
    pub broker_symbol_id: i64,
    pub symbol: String,
    pub year: i32,
    pub week_from: DateTime<Utc>,
    pub week_to: DateTime<Utc>,
    /// `measured`, `no_ticks` or `failed`.
    pub status: String,
    pub median_spread: Option<Decimal>,
    pub samples: i64,
    pub bid_ticks: i64,
    pub ask_ticks: i64,
    pub download_ms: i64,
    pub basis: Option<Value>,
    pub error: Option<String>,
    pub prereg_hash: String,
}

pub async fn record_spread(pool: &PgPool, s: &SpreadSample) -> crate::Result<()> {
    sqlx::query(
        "INSERT INTO research_spreads (account_id, broker_symbol_id, symbol, year, week_from, week_to,
            status, median_spread, samples, bid_ticks, ask_ticks, download_ms, basis, error, prereg_hash)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
    )
    .bind(s.account_id)
    .bind(s.broker_symbol_id)
    .bind(&s.symbol)
    .bind(s.year)
    .bind(s.week_from)
    .bind(s.week_to)
    .bind(&s.status)
    .bind(s.median_spread)
    .bind(s.samples)
    .bind(s.bid_ticks)
    .bind(s.ask_ticks)
    .bind(s.download_ms)
    .bind(&s.basis)
    .bind(&s.error)
    .bind(&s.prereg_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// The spread samples of an account that are not failed, by symbol and year.
pub async fn spread_samples(pool: &PgPool, account_id: i64) -> crate::Result<Vec<SpreadSample>> {
    let rows = sqlx::query(
        "SELECT account_id, broker_symbol_id, symbol, year, week_from, week_to, status, median_spread,
                samples, bid_ticks, ask_ticks, download_ms, basis, error, prereg_hash
         FROM research_spreads WHERE account_id = $1 AND status <> 'failed'
         ORDER BY symbol, year",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SpreadSample {
            account_id: r.get("account_id"),
            broker_symbol_id: r.get("broker_symbol_id"),
            symbol: r.get("symbol"),
            year: r.get("year"),
            week_from: r.get("week_from"),
            week_to: r.get("week_to"),
            status: r.get("status"),
            median_spread: r.get("median_spread"),
            samples: r.get("samples"),
            bid_ticks: r.get("bid_ticks"),
            ask_ticks: r.get("ask_ticks"),
            download_ms: r.get("download_ms"),
            basis: r.get("basis"),
            error: r.get("error"),
            prereg_hash: r.get("prereg_hash"),
        })
        .collect())
}

/// The commission query. Commission is a cost parameter, not a price: the
/// live deals it reads (21-10-2025 → 16-01-2026, the source the
/// pre-registration names) are mostly after the held-out seal, so it reads
/// ONLY the class, the commission, its money digits and the filled volume —
/// never a price or rate column (pinned by a test).
pub const COMMISSION_SQL: &str =
    "SELECT s.asset_class, d.commission, d.money_digits, d.filled_volume
         FROM deals d JOIN symbols s
           ON s.account_id = d.account_id AND s.broker_symbol_id = d.broker_symbol_id
         WHERE d.account_id = $1 AND d.commission IS NOT NULL AND d.money_digits IS NOT NULL
           AND d.filled_volume > 0";

/// Commission samples from an account's deals: |commission| ÷ filled units
/// per deal, in deposit money, with the symbol's class.
pub async fn commission_per_unit(
    pool: &PgPool,
    account_id: i64,
) -> crate::Result<Vec<(String, f64)>> {
    let rows = sqlx::query(COMMISSION_SQL)
        .bind(account_id)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let commission: i64 = r.get("commission");
            let digits: i32 = r.get("money_digits");
            let cents: i64 = r.get("filled_volume");
            let money = commission.abs() as f64 / 10f64.powi(digits);
            (r.get("asset_class"), money / (cents as f64 / 100.0))
        })
        .collect())
}

/// The quote → deposit rates of the closing deals on the symbols whose
/// quote asset, in the given load run's broker symbol list, is `asset_id`.
/// The filter is in the query, so no other symbol's rate is read: a deal on
/// a symbol quoted in another currency carries a price of that currency,
/// and a development job must not read one dated in the sealed period
/// (owner, 24-10-2026). Any date is read here, because a rate of a symbol
/// quoted in the deposit currency itself is 1 by definition, not a price.
pub const QUOTED_IN_SQL: &str = "SELECT d.close_quote_to_deposit_rate
     FROM deals d
     WHERE d.account_id = $1 AND d.is_closing AND d.close_quote_to_deposit_rate IS NOT NULL
       AND d.broker_symbol_id IN (
         SELECT (e->>'id')::bigint
         FROM load_runs r, jsonb_array_elements(r.listed_symbols) e
         WHERE r.id = $2 AND r.account_id = $1
           AND jsonb_typeof(e->'quote_asset_id') = 'number'
           AND (e->>'quote_asset_id')::bigint = $3)";

pub async fn closing_rates_quoted_in(
    pool: &PgPool,
    account_id: i64,
    load_run_id: i64,
    asset_id: i64,
) -> crate::Result<Vec<f64>> {
    Ok(sqlx::query_scalar(QUOTED_IN_SQL)
        .bind(account_id)
        .bind(load_run_id)
        .bind(asset_id)
        .fetch_all(pool)
        .await?)
}

/// (executed at, quote → deposit rate) of one symbol's closing deals
/// executed before `before`: the filter is in the query, so a deal at or
/// after it is not read (a development job passes the seal).
pub async fn closing_rates_of(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    before: DateTime<Utc>,
) -> crate::Result<Vec<(DateTime<Utc>, f64)>> {
    Ok(sqlx::query_as(
        "SELECT executed_at, close_quote_to_deposit_rate FROM deals
         WHERE account_id = $1 AND broker_symbol_id = $2 AND is_closing
           AND close_quote_to_deposit_rate IS NOT NULL AND executed_at < $3
         ORDER BY executed_at, deal_id",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(before)
    .fetch_all(pool)
    .await?)
}

/// The broker's current swap terms and volume limits for a symbol: the
/// latest cost row and the symbol row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolTerms {
    pub digits: i32,
    pub pip_position: i32,
    pub min_volume: Option<i64>,
    pub volume_step: Option<i64>,
    pub swap_long: Option<f64>,
    pub swap_short: Option<f64>,
    pub swap_calculation_type: Option<String>,
    pub swap_rollover_3days: Option<String>,
    pub swap_period_hours: Option<i32>,
    pub swap_read_at: Option<DateTime<Utc>>,
}

pub async fn symbol_terms(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
) -> crate::Result<Option<SymbolTerms>> {
    let r = sqlx::query(
        "SELECT s.digits, s.pip_position, s.min_volume, s.volume_step,
                c.swap_long, c.swap_short, c.swap_calculation_type, c.swap_rollover_3days,
                c.swap_period_hours, c.measured_at
         FROM symbols s
         LEFT JOIN LATERAL (
             SELECT * FROM symbol_costs c
             WHERE c.account_id = s.account_id AND c.broker_symbol_id = s.broker_symbol_id
             ORDER BY c.measured_at DESC LIMIT 1) c ON true
         WHERE s.account_id = $1 AND s.broker_symbol_id = $2",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|r| SymbolTerms {
        digits: r.get("digits"),
        pip_position: r.get("pip_position"),
        min_volume: r.get("min_volume"),
        volume_step: r.get("volume_step"),
        swap_long: r.get("swap_long"),
        swap_short: r.get("swap_short"),
        swap_calculation_type: r.get("swap_calculation_type"),
        swap_rollover_3days: r.get("swap_rollover_3days"),
        swap_period_hours: r.get("swap_period_hours"),
        swap_read_at: r.get("measured_at"),
    }))
}

/// A symbol's stored daily bars that END (open + 1 day) by `end`, in date order.
pub async fn daily_bars_ending_by(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
    end: DateTime<Utc>,
) -> crate::Result<Vec<crate::BarRow>> {
    let rows = sqlx::query(
        "SELECT bar_date, bar_open_utc, open, high, low, close, tick_volume
         FROM daily_bars
         WHERE account_id = $1 AND broker_symbol_id = $2
           AND bar_open_utc + interval '1 day' <= $3
         ORDER BY bar_date",
    )
    .bind(account_id)
    .bind(symbol_id)
    .bind(end)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| crate::BarRow {
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

/// The deposit asset id the broker stated in the account's latest trader
/// record (ProtoOATraderRes).
pub async fn deposit_asset_id(pool: &PgPool, account_id: i64) -> crate::Result<Option<i64>> {
    Ok(sqlx::query_scalar(
        "SELECT deposit_asset_id FROM account_snapshots WHERE account_id = $1 ORDER BY id DESC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?)
}

/// The latest load run of the account that ended `ok` with the symbol
/// resolved in it. Such a run read the symbol's whole daily history and
/// stored it without an error (the loader stores a symbol's row before its
/// bars, and any failed bar load makes the run partial), so every stored bar
/// of a sampled week is there. `None`: no such run yet.
pub async fn complete_bar_load(
    pool: &PgPool,
    account_id: i64,
    symbol_id: i64,
) -> crate::Result<Option<i64>> {
    Ok(sqlx::query_scalar(
        "SELECT r.id FROM load_runs r
         JOIN universe_resolutions u ON u.load_run_id = r.id
         WHERE r.account_id = $1 AND r.status = 'ok' AND r.finished_at IS NOT NULL
           AND u.account_id = $1 AND u.broker_symbol_id = $2 AND u.status = 'resolved'
         ORDER BY r.id DESC LIMIT 1",
    )
    .bind(account_id)
    .bind(symbol_id)
    .fetch_optional(pool)
    .await?)
}

/// (broker symbol id, year) of the sample weeks that failed at least
/// `times` times.
pub async fn sample_weeks_failed(
    pool: &PgPool,
    account_id: i64,
    times: i64,
) -> crate::Result<Vec<(i64, i32)>> {
    Ok(sqlx::query_as(
        "SELECT broker_symbol_id, year FROM research_spreads
         WHERE account_id = $1 AND status = 'failed'
         GROUP BY broker_symbol_id, year HAVING count(*) >= $2",
    )
    .bind(account_id)
    .bind(times)
    .fetch_all(pool)
    .await?)
}

/// Spread sampling so far: rows by status, ticks downloaded and wall time
/// (the download rate with nothing stored).
pub async fn sampling_summary(pool: &PgPool) -> crate::Result<Value> {
    let r = sqlx::query(
        "SELECT count(*) FILTER (WHERE status = 'measured') AS measured,
                count(*) FILTER (WHERE status = 'no_ticks') AS no_ticks,
                count(*) FILTER (WHERE status = 'failed') AS failed,
                coalesce(sum(bid_ticks + ask_ticks), 0)::bigint AS ticks,
                coalesce(sum(download_ms), 0)::bigint AS ms
         FROM research_spreads",
    )
    .fetch_one(pool)
    .await?;
    let ticks: i64 = r.get("ticks");
    let ms: i64 = r.get("ms");
    Ok(serde_json::json!({
        "weeks_measured": r.get::<i64, _>("measured"),
        "weeks_without_ticks": r.get::<i64, _>("no_ticks"),
        "weeks_failed": r.get::<i64, _>("failed"),
        "ticks_downloaded": ticks,
        "download_seconds": ms as f64 / 1000.0,
        "ticks_per_second": (ms > 0).then(|| ticks as f64 * 1000.0 / ms as f64),
        "note": "broker ticks streamed for the June spread samples and never stored; the rate includes the broker's paging and the 250 ms pacing between historical requests",
    }))
}

#[cfg(test)]
mod tests {
    use super::COMMISSION_SQL;

    /// Owner, 24-10-2026: the commission read touches no price column.
    #[test]
    fn the_commission_query_reads_no_price() {
        let select = COMMISSION_SQL
            .split_once("FROM")
            .unwrap()
            .0
            .trim_start_matches("SELECT");
        let cols: Vec<&str> = select.split(',').map(str::trim).collect();
        assert_eq!(
            cols,
            vec![
                "s.asset_class",
                "d.commission",
                "d.money_digits",
                "d.filled_volume"
            ]
        );
        let lower = COMMISSION_SQL.to_lowercase();
        for banned in ["price", "rate", "profit", "balance", "swap", "*"] {
            assert!(!lower.contains(banned), "{banned} in {COMMISSION_SQL}");
        }
    }
}
