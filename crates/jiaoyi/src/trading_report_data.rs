//! Read-only account accounting for a fixed New York close window.
//!
//! Broker closing cash, fully closed positions and current exposure have
//! different time bases. Keep them separate; no deposit/withdrawal assumptions
//! are made to manufacture equity returns.
use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use jiaoyi_store::sqlx::{self, PgPool, Row};
use serde_json::{json, Value};

use crate::telegram_report::Period;

const DAILY_SQL: &str = r#"
WITH opening_receipts AS (
 SELECT o.position_id,GREATEST(MAX(o.volume),COALESCE((SELECT SUM(volume) FROM (
   SELECT DISTINCT ON(e.deal_id) e.volume FROM trading_events e
   JOIN trading_intents i ON i.account_id=e.account_id AND i.entry_key=e.entry_key AND i.kind='open'
   WHERE e.account_id=$1 AND e.position_id=o.position_id AND e.volume>0
     AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
     AND e.deal_id IS NOT NULL AND e.received_at<$3 ORDER BY e.deal_id,e.id DESC
 ) d),0)) AS quantity
 FROM trading_events o JOIN trading_intents i
   ON i.account_id=o.account_id AND i.entry_key=o.entry_key AND i.kind='open'
 WHERE o.account_id=$1 AND o.position_id IS NOT NULL AND o.volume>0 AND o.received_at<$3
   AND o.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 GROUP BY o.position_id
), money AS (
 SELECT d.*, (deal_status IN ('FILLED','PARTIALLY_FILLED')
   AND close_gross_profit IS NOT NULL AND close_swap IS NOT NULL
   AND close_commission IS NOT NULL AND close_pnl_conversion_fee IS NOT NULL
   AND COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18) AS priced,
 CASE WHEN deal_status IN ('FILLED','PARTIALLY_FILLED')
   AND close_gross_profit IS NOT NULL AND close_swap IS NOT NULL
   AND close_commission IS NOT NULL AND close_pnl_conversion_fee IS NOT NULL
   AND COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18 THEN
   (close_gross_profit::float8+close_swap::float8-abs(close_commission::float8)
    -abs(close_pnl_conversion_fee::float8))
   /power(10::float8,COALESCE(close_money_digits,money_digits)) END AS net
 FROM deals d WHERE account_id=$1 AND filled_volume>0 AND executed_at<$3
), position_history AS (
 SELECT position_id,
 COALESCE(SUM(filled_volume) FILTER(WHERE NOT is_closing AND deal_status IN ('FILLED','PARTIALLY_FILLED')),0) AS history_opened,
 COALESCE(SUM(filled_volume) FILTER(WHERE is_closing AND deal_status IN ('FILLED','PARTIALLY_FILLED')),0) AS closed,
 MAX(executed_at) FILTER(WHERE is_closing) AS closed_at,
 BOOL_AND(NOT is_closing OR COALESCE(priced,false)) AS priced,
 BOOL_AND(deal_status IN ('FILLED','PARTIALLY_FILLED')) AS valid,
 SUM(net) FILTER(WHERE is_closing) AS net
 FROM money GROUP BY position_id
), positions AS (
 SELECT h.*,GREATEST(h.history_opened,COALESCE(o.quantity,0)) AS opened,
 h.history_opened>=COALESCE(o.quantity,0) AS opening_complete
 FROM position_history h LEFT JOIN opening_receipts o USING(position_id)
), cash AS (
 SELECT COUNT(*) AS closing_deals,
 COUNT(*) FILTER(WHERE NOT COALESCE(priced,false)) AS unpriced_deals,
 COALESCE(SUM(net),0)::float8 AS known_cash_net
 FROM money WHERE is_closing AND executed_at >= $2
)
SELECT c.*,
 (SELECT COUNT(*) FROM positions WHERE opened>0 AND closed=opened AND valid AND opening_complete
   AND closed_at >= $2) AS positions,
 (SELECT COUNT(*) FROM positions WHERE opened>0 AND closed=opened AND valid AND opening_complete
   AND closed_at >= $2 AND priced AND net>0) AS wins,
 (SELECT COUNT(*) FROM positions WHERE closed_at >= $2
   AND ((opened>0 AND closed>=opened AND (NOT priced OR NOT valid OR closed<>opened))
     OR opened=0 OR NOT valid OR NOT opening_complete)) AS unpriced_positions
FROM cash c
"#;

// A normal retry of the same window supersedes its previous receipt. Adjacent
// successful windows can jointly cover a report; an older backfill job cannot
// overwrite a newer coverage frontier merely by having a larger row ID.
const COVERAGE_SQL: &str = r#"
SELECT window_from,window_to,fetched_at,status,error FROM (
 SELECT DISTINCT ON(window_from,window_to) window_from,window_to,fetched_at,status,error,id
 FROM deal_fetches WHERE account_id=$1 AND window_from<$3 AND window_to>$2
 ORDER BY window_from,window_to,fetched_at DESC,id DESC
) f ORDER BY window_from,window_to
"#;

async fn history_covers(
    pool: &PgPool,
    account: i64,
    period: &Period,
    now: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query(COVERAGE_SQL)
        .bind(account)
        .bind(period.from)
        .bind(period.to)
        .fetch_all(pool)
        .await?;
    let mut through = period.from;
    for r in rows {
        let from: DateTime<Utc> = r.get("window_from");
        let to: DateTime<Utc> = r.get("window_to");
        let fetched: DateTime<Utc> = r.get("fetched_at");
        if r.get::<String, _>("status") != "ok"
            || r.get::<Option<String>, _>("error").is_some()
            || from > to
            || to > now
            || fetched > now
            || fetched < to.min(period.to)
        {
            continue;
        }
        if from <= through {
            through = through.max(to);
        }
    }
    Ok(through >= period.to)
}

const EXECUTION_SQL: &str = r#"
WITH distinct_deals AS (
 SELECT DISTINCT ON(e.deal_id) e.* FROM trading_events e
 WHERE e.account_id=$1 AND e.deal_id IS NOT NULL AND e.volume>0
   AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL')
 ORDER BY e.deal_id,e.id
), fills AS (
 SELECT * FROM distinct_deals WHERE received_at >= $2 AND received_at < $3
), telemetry AS (
 SELECT account_id,entry_key,symbol,slippage,latency_ms,kind,detail FROM strategy1_fills WHERE account_id=$1
 UNION ALL
 SELECT account_id,entry_key,symbol,slippage,latency_ms,kind,detail FROM strategy3_fills WHERE account_id=$1
), matched AS (
 SELECT e.*,t.entry_key AS telemetry_key,
 COALESCE(s.name,NULLIF(t.symbol,''),'symbol:'||e.symbol_id::text,'unknown') AS symbol
 FROM fills e LEFT JOIN symbols s ON s.account_id=$1 AND s.broker_symbol_id=e.symbol_id
 LEFT JOIN LATERAL(SELECT symbol,entry_key FROM telemetry t WHERE t.entry_key=e.entry_key
   OR (t.kind='broker_exit' AND t.detail->>'deal_id'=e.deal_id::text)
   ORDER BY (t.entry_key=e.entry_key) DESC NULLS LAST,t.entry_key LIMIT 1) t ON true
), sizes AS (
 SELECT symbol,COALESCE(SUM(volume),0)::bigint AS volume,COUNT(*) AS fill_count
 FROM matched GROUP BY symbol
), timings AS (
 SELECT m.symbol,AVG(t.slippage) FILTER(WHERE t.slippage > '-Infinity'::float8
   AND t.slippage < 'Infinity'::float8) AS slippage_mean,
 AVG(t.latency_ms) FILTER(WHERE t.latency_ms>=0)::float8 AS latency_ms_mean,
 MAX(t.latency_ms) FILTER(WHERE t.latency_ms>=0) AS latency_ms_max,
 COUNT(t.slippage) FILTER(WHERE t.slippage > '-Infinity'::float8
   AND t.slippage < 'Infinity'::float8) AS slippage_samples,
 COUNT(t.latency_ms) FILTER(WHERE t.latency_ms>=0) AS latency_samples
 FROM (SELECT DISTINCT symbol,telemetry_key FROM matched) m JOIN telemetry t ON t.entry_key=m.telemetry_key
 WHERE t.kind <> 'trail' GROUP BY m.symbol
)
SELECT s.*,t.slippage_mean,t.latency_ms_mean,t.latency_ms_max,
 COALESCE(t.slippage_samples,0)::bigint AS slippage_samples,
 COALESCE(t.latency_samples,0)::bigint AS latency_samples
FROM sizes s LEFT JOIN timings t USING(symbol) ORDER BY symbol
"#;

struct Mark {
    at: DateTime<Utc>,
    detail: Value,
}

fn timestamp(value: &Value) -> Option<DateTime<Utc>> {
    value.as_str()?.parse().ok()
}

fn close_on(date: NaiveDate) -> DateTime<Utc> {
    New_York
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap()))
        .single()
        .expect("17:00 NY has no DST ambiguity")
        .with_timezone(&Utc)
}

fn measured_equity(mark: &Mark, cutoff: DateTime<Utc>, now: DateTime<Utc>) -> Option<f64> {
    let d = &mark.detail;
    let delay = mark.at.signed_duration_since(cutoff).num_milliseconds();
    if d["source_complete"] != true
        || timestamp(&d["observed_at"]) != Some(mark.at)
        || timestamp(&d["ny_close_cutoff"]) != Some(cutoff)
        || d["close_delay_ms"].as_i64() != Some(delay)
        || !(0..=90_000).contains(&delay)
        || mark.at > now
        || d["equity_basis"] != "balance_plus_broker_net_unrealized_excluding_unverified_credit"
    {
        return None;
    }
    d["equity"].as_f64().filter(|n| n.is_finite())
}

/// P&L is between actual retained marks (up to90s after cutoff), excluding
/// unverified credit. Gross flow count, not a zero net flow, permits a return.
fn economics(
    opening: &Mark,
    closing: &Mark,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<(f64, Option<f64>)> {
    let initial = measured_equity(opening, from, now)?;
    let final_equity = measured_equity(closing, to, now)?;
    let d = &closing.detail;
    let currency = opening.detail["currency"]
        .as_str()
        .filter(|s| !s.is_empty())?;
    let external_count = d["external_flow_count"].as_u64()?;
    let net_flow = d["net_external_flow"].as_f64().filter(|n| n.is_finite())?;
    if d["currency"].as_str() != Some(currency)
        || d["cashflow_coverage_complete"] != true
        || timestamp(&d["flow_window_from"]) != Some(opening.at)
        || timestamp(&d["flow_window_to"]) != Some(closing.at)
        || d["cashflow_source"]
            != "broker_cash_flow_history_actual_marks_open_exclusive_close_inclusive"
        || (external_count == 0 && net_flow != 0.0)
    {
        return None;
    }
    let pnl = final_equity - initial - net_flow;
    if !pnl.is_finite() {
        return None;
    }
    let return_pct = (external_count == 0 && initial > 0.0)
        .then(|| 100.0 * pnl / initial)
        .filter(|r| r.is_finite());
    Some((pnl, return_pct))
}

fn linked_return(
    marks: &BTreeMap<String, Mark>,
    start: NaiveDate,
    end: NaiveDate,
    now: DateTime<Utc>,
) -> Option<f64> {
    let mut date = start;
    let mut growth = 1.0;
    while date <= end {
        let previous = date.checked_sub_days(Days::new(1))?;
        let opening = marks.get(&format!("close:{previous}"))?;
        let closing = marks.get(&format!("close:{date}"))?;
        let (_, daily_return) =
            economics(opening, closing, close_on(previous), close_on(date), now)?;
        growth *= 1.0 + daily_return? / 100.0;
        date = date.checked_add_days(Days::new(1))?;
    }
    let result = 100.0 * (growth - 1.0);
    result.is_finite().then_some(result)
}

/// Every account's own deposit currency; never sum currencies or infer cents.
/// Cash includes external/manual positions because this is account accounting,
/// while goal comparisons remain strategy-owned supervisor cohorts.
pub async fn collect(
    pool: &PgPool,
    accounts: &[i64],
    period: &Period,
) -> Result<Vec<Value>, sqlx::Error> {
    let now = Utc::now();
    let previous_key = format!(
        "close:{}",
        period.ny_date.checked_sub_days(Days::new(1)).unwrap()
    );
    let closing_key = format!("close:{}", period.ny_date);
    let year_start = NaiveDate::from_ymd_opt(period.ny_date.year(), 1, 1).unwrap();
    let month_start = period.ny_date.with_day(1).unwrap();
    let first_key = format!(
        "close:{}",
        year_start.checked_sub_days(Days::new(1)).unwrap()
    );
    let mut out = Vec::with_capacity(accounts.len());
    for &account in accounts {
        let metadata = sqlx::query("SELECT environment,currency FROM accounts WHERE account_id=$1")
            .bind(account)
            .fetch_optional(pool)
            .await?;
        let snapshots: BTreeMap<String,Mark> = sqlx::query("SELECT sample_key,observed_at,detail FROM report_account_snapshots WHERE account_id=$1 AND (sample_key='latest' OR (sample_key>=$2 AND sample_key<=$3))")
            .bind(account).bind(&first_key).bind(&closing_key).fetch_all(pool).await?.into_iter()
            .map(|r|(r.get("sample_key"),Mark{at:r.get("observed_at"),detail:r.get("detail")})).collect();
        let mut snapshot = snapshots
            .get("latest")
            .map(|r| r.detail.clone())
            .unwrap_or_else(|| json!({}));
        let current = snapshots.get("latest").is_some_and(|mark| {
            mark.detail["source_complete"] == true
                && timestamp(&mark.detail["observed_at"]) == Some(mark.at)
                && mark.at <= now
                && now.signed_duration_since(mark.at) <= chrono::Duration::seconds(180)
        });
        snapshot["current"] = json!(current);
        snapshot["stored_observed_at"] = json!(snapshots.get("latest").map(|m| m.at));
        let environment = snapshot["environment"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| metadata.as_ref().map(|r| r.get::<String, _>("environment")));
        let currency = snapshot["currency"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                metadata
                    .as_ref()
                    .and_then(|r| r.get::<Option<String>, _>("currency"))
            });
        let opening = snapshots.get(&previous_key);
        let closing = snapshots.get(&closing_key);
        let economic = opening
            .zip(closing)
            .and_then(|(o, c)| economics(o, c, period.from, period.to, now));
        let mtd = linked_return(&snapshots, month_start, period.ny_date, now);
        let ytd = linked_return(&snapshots, year_start, period.ny_date, now);
        let row = sqlx::query(DAILY_SQL)
            .bind(account)
            .bind(period.from)
            .bind(period.to)
            .fetch_one(pool)
            .await?;
        let unpriced_deals: i64 = row.get("unpriced_deals");
        let known_cash_net: f64 = row.get("known_cash_net");
        let history_complete = history_covers(pool, account, period, now).await?;
        let executions = sqlx::query(EXECUTION_SQL).bind(account).bind(period.from).bind(period.to)
            .fetch_all(pool).await?.iter().map(|r| json!({
                "symbol":r.get::<String,_>("symbol"),"volume":r.get::<i64,_>("volume"),
                "fill_count":r.get::<i64,_>("fill_count"),
                "slippage_mean":r.get::<Option<f64>,_>("slippage_mean"),
                "latency_ms_mean":r.get::<Option<f64>,_>("latency_ms_mean"),
                "latency_ms_max":r.get::<Option<i64>,_>("latency_ms_max"),
                "slippage_samples":r.get::<i64,_>("slippage_samples"),
                "latency_samples":r.get::<i64,_>("latency_samples"),
                "volume_unit":"broker native cents of instrument units; separate per symbol",
                "time_basis":"first recorded broker deal-fill event received in [from,to)",
                "slippage_definition":"existing per-order direction-aware price units: buy fill-reference; sell reference-fill; positive adverse; broker exit reference is its stop/target",
                "telemetry_basis":"arithmetic mean of available distinct-order telemetry matched to period deal fills, not volume weighted"
            })).collect::<Vec<_>>();
        let mut previous_goals = json!({"s1":null,"s3":null});
        for strategy in ["s1", "s3"] {
            let kind = format!("{strategy}_supervisor_report");
            let previous: Option<Value> = sqlx::query_scalar("SELECT COALESCE(detail->'assessment','null'::jsonb) FROM trading_decisions WHERE account_id=$1 AND kind=$2 AND at<=$3 ORDER BY at DESC,id DESC LIMIT 1")
                .bind(account).bind(kind).bind(period.from).fetch_optional(pool).await?;
            previous_goals[strategy] = previous.unwrap_or(Value::Null);
        }
        out.push(json!({"account_id":account,"environment":environment,"currency":currency,
            "snapshot":snapshot,
            "daily":{"cash_net":(unpriced_deals==0 && known_cash_net.is_finite()).then_some(known_cash_net),
                "known_cash_net":known_cash_net,"closing_deals":row.get::<i64,_>("closing_deals"),
                "positions":row.get::<i64,_>("positions"),"wins":row.get::<i64,_>("wins"),
                "unpriced_deals":unpriced_deals,"unpriced_positions":row.get::<i64,_>("unpriced_positions"),
                "history_complete":history_complete,"economic_pnl":economic.map(|(p,_)|p),"return_pct":economic.and_then(|(_,r)|r),
                "mtd_return_pct":mtd,"ytd_return_pct":ytd,"underwater":null,
                "cashflow_coverage_complete":economic.is_some(),
                "net_external_flow":closing.map(|m|m.detail["net_external_flow"].clone()),
                "external_flow_count":closing.map(|m|m.detail["external_flow_count"].clone()),
                "previous_equity":opening.map(|r|r.detail["equity"].clone()),
                "opening_at":opening.map(|r|r.at),
                "closing_at":closing.map(|r|r.at),
                "economic_basis":"actual retained marks up to90s after NY cutoff; balance plus broker net unrealized excluding unverified credit; external flows removed; percentages require no gross external flows",
                "notes":["Cash net is gross + swap - abs(closing commission) - abs(conversion fee) for all account closing deals executed in [from,to), including partial/manual closes. Missing monetary fields are unknown, not zero.",
                    "Fully closed positions and known wins use the whole stored position through cutoff, separate from partial-close cash. Missing opening history or costs is unpriced.",
                    "Equity P&L requires actual bounded close marks and independent complete external cashflows. Matching net balance changes do not prove absence of gross cashflows. Returns require zero gross external flows; MTD/YTD link a complete daily chain; underwater has no verified starting-capital baseline."]},
            "execution":executions,"risks":[],"previous_goals":previous_goals}));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn period() -> Period {
        crate::telegram_report::period_due(Utc::now() - Duration::days(2))
    }

    async fn account(pool: &PgPool, id: i64, currency: &str) {
        sqlx::query("INSERT INTO accounts(account_id,environment,login_missing_reason,currency,role_missing_reason,enabled) VALUES($1,'demo','not read',$2,'not assigned',true)")
            .bind(id).bind(currency).execute(pool).await.unwrap();
    }

    async fn snapshot(pool: &PgPool, account: i64, key: &str, at: DateTime<Utc>, detail: Value) {
        sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES($1,$2,$3,$4)")
            .bind(account).bind(key).bind(at).bind(detail).execute(pool).await.unwrap();
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn current_snapshot_requires_matching_fresh_nonfuture_complete_source(pool: PgPool) {
        let now = DateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap();
        for id in 1..=5 {
            account(&pool, id, "USD").await;
            let at = now
                + Duration::seconds(match id {
                    2 => -181,
                    3 => 60,
                    _ => -30,
                });
            let detail_at = if id == 4 {
                at - Duration::seconds(1)
            } else {
                at
            };
            snapshot(
                &pool,
                id,
                "latest",
                at,
                json!({"balance":100,"observed_at":detail_at,"source_complete":id!=5}),
            )
            .await;
        }
        let reports = collect(&pool, &[1, 2, 3, 4, 5], &period()).await.unwrap();
        for (index, r) in reports.iter().enumerate() {
            assert_eq!(r["snapshot"]["current"], index == 0);
            assert_eq!(
                r["snapshot"]["balance"], 100,
                "stale facts retain their as-of value"
            );
            assert!(r["snapshot"]["stored_observed_at"].is_string());
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn deal(
        pool: &PgPool,
        account: i64,
        id: i64,
        position: i64,
        at: DateTime<Utc>,
        volume: i64,
        closing: bool,
        net: i64,
    ) {
        sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,is_closing,close_gross_profit,close_swap,close_commission,close_pnl_conversion_fee,close_money_digits,raw,fetched_at) VALUES($1,$2,$2,$3,1,$4,$4,'BUY','FILLED',$5,$5,$6,$7,0,-10,0,2,''::bytea,now())")
            .bind(account).bind(id).bind(position).bind(volume).bind(at).bind(closing).bind(net+10)
            .execute(pool).await.unwrap();
    }

    async fn coverage(
        pool: &PgPool,
        account: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        ok: bool,
    ) {
        sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,status,complete,error) VALUES($1,$2,$3,$4,false,$5)")
            .bind(account).bind(from).bind(to).bind(if ok {"ok"} else {"failed"})
            .bind((!ok).then_some("broker failed")).execute(pool).await.unwrap();
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn account_cash_includes_manual_and_partial_closes_but_wins_use_whole_positions(
        pool: PgPool,
    ) {
        let p = period();
        account(&pool, 1, "JPY").await;
        account(&pool, 2, "EUR").await;
        snapshot(
            &pool,
            1,
            "latest",
            Utc::now(),
            json!({"currency":"SGD","environment":"demo","equity":120}),
        )
        .await;
        snapshot(
            &pool,
            1,
            &format!(
                "close:{}",
                p.ny_date.checked_sub_days(Days::new(1)).unwrap()
            ),
            p.from,
            json!({"equity":100}),
        )
        .await;
        snapshot(
            &pool,
            1,
            &format!("close:{}", p.ny_date),
            p.to,
            json!({"equity":120}),
        )
        .await;
        // No strategy intents: whole-account cash must still include these.
        for position in 1..=5 {
            deal(
                &pool,
                1,
                position * 10,
                position,
                p.from - Duration::hours(2),
                100,
                false,
                0,
            )
            .await;
        }
        // This position's first close belongs to yesterday; only the final4
        // is today's cash, while the whole position's10 determines its win.
        deal(
            &pool,
            1,
            11,
            1,
            p.from - Duration::seconds(1),
            40,
            true,
            600,
        )
        .await;
        deal(&pool, 1, 12, 1, p.from + Duration::hours(1), 60, true, 400).await;
        deal(&pool, 1, 21, 2, p.from + Duration::hours(1), 40, true, 200).await;
        deal(
            &pool,
            1,
            31,
            3,
            p.from + Duration::hours(2),
            100,
            true,
            -300,
        )
        .await;
        deal(&pool, 1, 41, 4, p.from, 100, true, 100).await;
        deal(&pool, 1, 51, 5, p.to, 100, true, 1000).await;
        deal(
            &pool,
            2,
            1,
            1,
            p.from + Duration::hours(1),
            100,
            true,
            999999,
        )
        .await;
        coverage(&pool, 1, p.from, p.to, true).await;
        let reports = collect(&pool, &[1, 2], &p).await.unwrap();
        let r = &reports[0];
        assert_eq!(r["currency"], "SGD");
        assert_eq!(reports[1]["currency"], "EUR");
        assert_eq!(r["daily"]["cash_net"], 4.0);
        assert_eq!(r["daily"]["closing_deals"], 4);
        assert_eq!(r["daily"]["positions"], 3);
        assert_eq!(r["daily"]["wins"], 2);
        assert_eq!(r["daily"]["history_complete"], true);
        assert_eq!(r["daily"]["previous_equity"], 100);
        for key in [
            "economic_pnl",
            "return_pct",
            "mtd_return_pct",
            "ytd_return_pct",
            "underwater",
        ] {
            assert!(
                r["daily"][key].is_null(),
                "unverified cashflows cannot yield {key}"
            );
        }
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn missing_money_or_opening_history_is_unknown_not_a_zero_or_a_win(pool: PgPool) {
        let p = period();
        account(&pool, 1, "USD").await;
        deal(&pool, 1, 10, 1, p.from - Duration::hours(1), 100, false, 0).await;
        deal(&pool, 1, 11, 1, p.from + Duration::hours(1), 100, true, 100).await;
        deal(&pool, 1, 21, 2, p.from + Duration::hours(1), 100, true, 200).await;
        sqlx::query(
            "UPDATE deals SET close_pnl_conversion_fee=NULL WHERE account_id=1 AND deal_id=11",
        )
        .execute(&pool)
        .await
        .unwrap();
        let reports = collect(&pool, &[1], &p).await.unwrap();
        let d = &reports[0]["daily"];
        assert!(d["cash_net"].is_null());
        assert_eq!(d["known_cash_net"], 2.0);
        assert_eq!(d["closing_deals"], 2);
        assert_eq!(d["positions"], 1);
        assert_eq!(d["wins"], 0);
        assert_eq!(d["unpriced_deals"], 1);
        assert_eq!(d["unpriced_positions"], 2);
        assert_eq!(d["history_complete"], false);
        assert_eq!(reports[0]["snapshot"]["current"], false);
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn confirmed_opening_receipt_prevents_incomplete_history_from_closing_half_a_position(
        pool: PgPool,
    ) {
        let p = period();
        account(&pool, 1, "USD").await;
        let opened = p.from - Duration::hours(1);
        sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,volume,max_risk) VALUES(1,'s1:receipt','s1:receipt','open',100,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,position_id,deal_id,volume,received_at,raw) VALUES(1,'s1:receipt',2126,'ORDER_FILLED',1,10,100,$1,''::bytea)")
            .bind(opened).execute(&pool).await.unwrap();
        deal(&pool, 1, 10, 1, opened, 50, false, 0).await;
        deal(&pool, 1, 11, 1, p.from + Duration::hours(1), 50, true, 100).await;
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["cash_net"], 1.0);
        assert_eq!(reports[0]["daily"]["positions"], 0);
        assert_eq!(reports[0]["daily"]["wins"], 0);
        assert_eq!(reports[0]["daily"]["unpriced_positions"], 1);
        sqlx::query(
            "UPDATE deals SET filled_volume=100,volume=100 WHERE account_id=1 AND deal_id=10",
        )
        .execute(&pool)
        .await
        .unwrap();
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["positions"], 0);
        assert_eq!(reports[0]["daily"]["unpriced_positions"], 0);
        deal(&pool, 1, 12, 1, p.from + Duration::hours(2), 50, true, 100).await;
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["cash_net"], 2.0);
        assert_eq!(reports[0]["daily"]["positions"], 1);
        assert_eq!(reports[0]["daily"]["wins"], 1);
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn coverage_merges_contiguous_receipts_and_previous_goal_never_uses_today(pool: PgPool) {
        let p = period();
        for id in 1..=3 {
            account(&pool, id, "USD").await;
        }
        let mid = p.from + Duration::hours(12);
        coverage(&pool, 1, p.from, mid, true).await;
        coverage(&pool, 1, mid, p.to, true).await;
        // A later archival task is irrelevant to the report's frontier.
        coverage(
            &pool,
            1,
            p.from - Duration::days(8),
            p.from - Duration::days(7),
            true,
        )
        .await;
        coverage(&pool, 2, p.from, mid - Duration::seconds(1), true).await;
        coverage(&pool, 2, mid, p.to, true).await;
        coverage(&pool, 3, p.from, p.to, true).await;
        coverage(&pool, 3, p.from, p.to, false).await;
        for (time, n) in [(p.from, 20), (p.from + Duration::seconds(1), 99)] {
            sqlx::query("INSERT INTO trading_decisions(account_id,at,kind,reason,detail) VALUES(1,$1,'s1_supervisor_report','fixture',$2)")
                .bind(time).bind(json!({"assessment":{"window":{"count":n},"sample_identity":n}}))
                .execute(&pool).await.unwrap();
        }
        let reports = collect(&pool, &[1, 2, 3], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["history_complete"], true);
        assert_eq!(reports[1]["daily"]["history_complete"], false);
        assert_eq!(reports[2]["daily"]["history_complete"], false);
        assert_eq!(reports[0]["previous_goals"]["s1"]["sample_identity"], 20);
        assert!(reports[0]["previous_goals"]["s3"].is_null());
    }

    async fn event(
        pool: &PgPool,
        account: i64,
        deal: Option<i64>,
        key: Option<&str>,
        at: DateTime<Utc>,
        volume: i64,
        kind: &str,
    ) {
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,payload_type,kind,deal_id,symbol_id,volume,received_at,raw) VALUES($1,$2,2126,$3,$4,1,$5,$6,''::bytea)")
            .bind(account).bind(key).bind(kind).bind(deal).bind(volume).bind(at).execute(pool).await.unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    async fn telemetry(
        pool: &PgPool,
        account: i64,
        key: &str,
        kind: &str,
        at: DateTime<Utc>,
        slip: f64,
        latency: Option<i64>,
        detail: Value,
    ) {
        sqlx::query("INSERT INTO strategy1_fills(account_id,entry_key,label,prereg_hash,kind,symbol,outcome,sent_at,answered_at,slippage,latency_ms,detail) VALUES($1,$2,'jiaoyi-s1','test',$3,'XAUUSD','filled',$4,$4,$5,$6,$7)")
            .bind(account).bind(key).bind(kind).bind(at).bind(slip).bind(latency).bind(detail).execute(pool).await.unwrap();
    }

    fn mark(
        cutoff: DateTime<Utc>,
        equity: f64,
        opening_at: DateTime<Utc>,
        net: f64,
        count: u64,
    ) -> Mark {
        let at = cutoff + Duration::seconds(30);
        Mark {
            at,
            detail: json!({"currency":"USD","source_complete":true,"equity":equity,
            "observed_at":at,"ny_close_cutoff":cutoff,"close_delay_ms":30_000,
            "equity_basis":"balance_plus_broker_net_unrealized_excluding_unverified_credit",
            "flow_window_from":opening_at,"flow_window_to":at,
            "net_external_flow":net,"external_flow_count":count,"cashflow_coverage_complete":true,
            "cashflow_source":"broker_cash_flow_history_actual_marks_open_exclusive_close_inclusive"}),
        }
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn economic_pnl_needs_actual_marks_and_verified_gross_cashflows_not_balance_guessing(
        pool: PgPool,
    ) {
        let p = period();
        account(&pool, 1, "USD").await;
        let opening = mark(p.from, 100.0, p.from - Duration::days(1), 0.0, 0);
        let closing = mark(p.to, 130.0, opening.at, 20.0, 1);
        snapshot(
            &pool,
            1,
            &format!(
                "close:{}",
                p.ny_date.checked_sub_days(Days::new(1)).unwrap()
            ),
            opening.at,
            opening.detail,
        )
        .await;
        snapshot(
            &pool,
            1,
            &format!("close:{}", p.ny_date),
            closing.at,
            closing.detail,
        )
        .await;
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["economic_pnl"], 10.0);
        assert!(reports[0]["daily"]["return_pct"].is_null());
        // Offset deposit/withdrawal is still a gross flow. Zero net does not
        // make a simple opening-equity denominator valid.
        sqlx::query("UPDATE report_account_snapshots SET detail=detail||'{\"net_external_flow\":0,\"external_flow_count\":2}'::jsonb WHERE account_id=1 AND sample_key=$1")
            .bind(format!("close:{}",p.ny_date)).execute(&pool).await.unwrap();
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["economic_pnl"], 30.0);
        assert!(reports[0]["daily"]["return_pct"].is_null());
        sqlx::query("UPDATE report_account_snapshots SET detail=jsonb_set(detail,'{external_flow_count}','0') WHERE account_id=1 AND sample_key=$1")
            .bind(format!("close:{}",p.ny_date)).execute(&pool).await.unwrap();
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert_eq!(reports[0]["daily"]["return_pct"], 30.0);
        sqlx::query("UPDATE report_account_snapshots SET detail=jsonb_set(detail,'{cashflow_coverage_complete}','false') WHERE account_id=1 AND sample_key=$1")
            .bind(format!("close:{}",p.ny_date)).execute(&pool).await.unwrap();
        let reports = collect(&pool, &[1], &p).await.unwrap();
        assert!(reports[0]["daily"]["economic_pnl"].is_null());
    }

    #[test]
    fn period_returns_link_complete_no_flow_days_and_reject_false_cutoff_marks() {
        let day = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let prior = day.checked_sub_days(Days::new(1)).unwrap();
        let next = day.checked_add_days(Days::new(1)).unwrap();
        let opening = mark(
            close_on(prior),
            100.0,
            close_on(prior) - Duration::days(1),
            0.0,
            0,
        );
        let first = mark(close_on(day), 110.0, opening.at, 0.0, 0);
        let second = mark(close_on(next), 121.0, first.at, 0.0, 0);
        let now = close_on(next) + Duration::hours(1);
        let mut marks = BTreeMap::from([
            (format!("close:{prior}"), opening),
            (format!("close:{day}"), first),
            (format!("close:{next}"), second),
        ]);
        assert!((linked_return(&marks, day, next, now).unwrap() - 21.0).abs() < 1e-9);
        marks.get_mut(&format!("close:{next}")).unwrap().detail["external_flow_count"] = json!(2);
        assert!(linked_return(&marks, day, next, now).is_none());
        let second = marks.get_mut(&format!("close:{next}")).unwrap();
        second.detail["external_flow_count"] = json!(0);
        second.at += Duration::minutes(5);
        assert!(linked_return(&marks, day, next, now).is_none());
        marks.remove(&format!("close:{prior}"));
        assert!(linked_return(&marks, day, next, now).is_none());
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn execution_uses_first_period_deal_receipts_native_volume_and_distinct_order_telemetry(
        pool: PgPool,
    ) {
        let p = period();
        account(&pool, 1, "USD").await;
        account(&pool, 2, "USD").await;
        let at = p.from + Duration::hours(1);
        event(
            &pool,
            1,
            Some(1),
            Some("s1:a"),
            at,
            100,
            "ORDER_PARTIAL_FILL",
        )
        .await;
        event(&pool, 1, Some(2), Some("s1:a"), at, 200, "ORDER_FILLED").await;
        event(
            &pool,
            1,
            Some(2),
            Some("s1:a"),
            at + Duration::seconds(1),
            200,
            "ORDER_FILLED",
        )
        .await;
        event(&pool, 1, Some(3), Some("s1:b"), at, 50, "ORDER_FILLED").await;
        event(&pool, 1, Some(4), None, at, 25, "ORDER_FILLED").await;
        event(
            &pool,
            1,
            Some(5),
            Some("s1:old"),
            p.from - Duration::seconds(1),
            1000,
            "ORDER_FILLED",
        )
        .await;
        event(&pool, 1, Some(5), Some("s1:old"), at, 1000, "ORDER_FILLED").await;
        event(&pool, 1, None, Some("s1:a"), at, 300, "RECONCILED_FILL").await;
        event(
            &pool,
            1,
            Some(6),
            Some("s1:trail"),
            at,
            999,
            "ORDER_REPLACED",
        )
        .await;
        event(
            &pool,
            2,
            Some(1),
            Some("s1:other"),
            at,
            9999,
            "ORDER_FILLED",
        )
        .await;
        telemetry(&pool, 1, "s1:a", "open", at, 2.0, Some(10), json!({})).await;
        telemetry(&pool, 1, "s1:b", "open", at, -1.0, Some(30), json!({})).await;
        telemetry(
            &pool,
            1,
            "s1-broker:4",
            "broker_exit",
            at,
            5.0,
            None,
            json!({"deal_id":4}),
        )
        .await;
        telemetry(
            &pool,
            1,
            "s1:old",
            "open",
            at,
            1000.0,
            Some(9999),
            json!({}),
        )
        .await;
        let reports = collect(&pool, &[1], &p).await.unwrap();
        let e = reports[0]["execution"].as_array().unwrap();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0]["symbol"], "XAUUSD");
        assert_eq!(e[0]["volume"], 375);
        assert_eq!(e[0]["fill_count"], 4);
        assert_eq!(e[0]["slippage_mean"], 2.0);
        assert_eq!(e[0]["slippage_samples"], 3);
        assert_eq!(e[0]["latency_ms_mean"], 20.0);
        assert_eq!(e[0]["latency_ms_max"], 30);
        assert_eq!(e[0]["latency_samples"], 2);
    }
}
