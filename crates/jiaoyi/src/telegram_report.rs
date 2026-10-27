//! Deterministic NY-close HTML files. Telegram failures cannot mutate trading.
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use jiaoyi_core::secret::Secret;
use jiaoyi_store::sqlx::{self, PgPool, Row};
use reqwest::{multipart, Client};
use serde_json::{json, Value};

use crate::{shutdown::Shutdown, supervisor};

#[derive(Clone)]
pub struct DeliveryConfig {
    token: Secret,
    chat_id: i64,
}

impl DeliveryConfig {
    pub fn from_env() -> Option<Self> {
        if std::env::var("TELEGRAM_TRADING_REPORT_SCHEDULE")
            .ok()
            .as_deref()
            != Some("ny_close")
        {
            return None;
        }
        let token = std::env::var("TELEGRAM_BOT_TOKEN").ok()?.trim().to_owned();
        if token.is_empty()
            || !token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-:".contains(&b))
        {
            return None;
        }
        let chat_id = std::env::var("TELEGRAM_OWNER_CHAT_ID")
            .ok()?
            .trim()
            .parse()
            .ok()?;
        Some(Self {
            token: Secret::new(token),
            chat_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Period {
    pub ny_date: NaiveDate,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub deliver_after: DateTime<Utc>,
}

fn close_on(date: NaiveDate) -> DateTime<Utc> {
    New_York
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap()))
        .single()
        .expect("17:00 New York is outside DST transitions")
        .with_timezone(&Utc)
}

/// Latest calendar-day close eligible for delivery: 17:00 NY + 15 minutes.
/// Includes zero-activity/weekend periods so no instrument's activity vanishes.
pub fn period_due(now: DateTime<Utc>) -> Period {
    let mut date = now.with_timezone(&New_York).date_naive();
    let delay = chrono::Duration::minutes(15);
    if now < close_on(date) + delay {
        date = date.checked_sub_days(Days::new(1)).unwrap();
    }
    let to = close_on(date);
    Period {
        ny_date: date,
        from: close_on(date.checked_sub_days(Days::new(1)).unwrap()),
        to,
        deliver_after: to + delay,
    }
}

const DAY_SQL: &str = r#"
WITH owned AS (
 SELECT DISTINCT ON (e.position_id) e.position_id
 FROM trading_intents i JOIN trading_events e ON e.account_id=i.account_id AND e.entry_key=i.entry_key
 WHERE i.account_id=$1 AND i.kind='open' AND i.entry_key LIKE $2||':%'
   AND e.position_id IS NOT NULL AND e.kind=ANY($5) AND e.volume>0
 ORDER BY e.position_id,i.id
), p AS (
 SELECT o.position_id,
 COALESCE(SUM(d.filled_volume) FILTER(WHERE NOT d.is_closing),0) AS opened,
 COALESCE(SUM(d.filled_volume) FILTER(WHERE d.is_closing),0) AS closed,
 MAX(d.executed_at) FILTER(WHERE d.is_closing) AS closed_at,
 BOOL_AND(NOT d.is_closing OR (d.close_gross_profit IS NOT NULL AND d.close_swap IS NOT NULL
  AND d.close_commission IS NOT NULL AND d.close_pnl_conversion_fee IS NOT NULL
  AND COALESCE(d.close_money_digits,d.money_digits) IS NOT NULL
  AND COALESCE(d.close_money_digits,d.money_digits) BETWEEN 0 AND 18)) AS money_complete,
 SUM(CASE WHEN d.is_closing AND d.close_gross_profit IS NOT NULL AND d.close_swap IS NOT NULL
  AND d.close_commission IS NOT NULL AND d.close_pnl_conversion_fee IS NOT NULL
  AND COALESCE(d.close_money_digits,d.money_digits) BETWEEN 0 AND 18 THEN
  (d.close_gross_profit::double precision+d.close_swap::double precision
   -abs(d.close_commission::double precision)-abs(d.close_pnl_conversion_fee::double precision))
   /power(10::double precision,COALESCE(d.close_money_digits,d.money_digits)) END) AS net,
 COUNT(*) FILTER(WHERE d.is_closing AND d.executed_at>=$3 AND d.executed_at<$4) AS closing_deals,
 COUNT(*) FILTER(WHERE d.is_closing AND d.executed_at>=$3 AND d.executed_at<$4 AND
  (d.close_gross_profit IS NULL OR d.close_swap IS NULL OR d.close_commission IS NULL
   OR d.close_pnl_conversion_fee IS NULL OR COALESCE(d.close_money_digits,d.money_digits) IS NULL
   OR COALESCE(d.close_money_digits,d.money_digits) NOT BETWEEN 0 AND 18)) AS unpriced_deals,
 SUM(CASE WHEN d.is_closing AND d.executed_at>=$3 AND d.executed_at<$4
  AND d.close_gross_profit IS NOT NULL AND d.close_swap IS NOT NULL
  AND d.close_commission IS NOT NULL AND d.close_pnl_conversion_fee IS NOT NULL
  AND COALESCE(d.close_money_digits,d.money_digits) BETWEEN 0 AND 18 THEN
  (d.close_gross_profit::double precision+d.close_swap::double precision
   -abs(d.close_commission::double precision)-abs(d.close_pnl_conversion_fee::double precision))
   /power(10::double precision,COALESCE(d.close_money_digits,d.money_digits)) END) AS cash_net
 FROM owned o LEFT JOIN deals d ON d.account_id=$1 AND d.position_id=o.position_id
  AND d.filled_volume>0 AND d.deal_status IN ('FILLED','PARTIALLY_FILLED') GROUP BY o.position_id
)
SELECT COUNT(*) FILTER(WHERE opened>0 AND closed>=opened AND closed_at>=$3 AND closed_at<$4) AS positions,
 COUNT(*) FILTER(WHERE opened>0 AND closed>=opened AND closed_at>=$3 AND closed_at<$4 AND money_complete AND net>0) AS wins,
 COUNT(*) FILTER(WHERE opened>0 AND closed>=opened AND closed_at>=$3 AND closed_at<$4 AND NOT money_complete) AS unpriced_positions,
 COALESCE(SUM(closing_deals),0)::bigint AS closing_deals,
 COALESCE(SUM(unpriced_deals),0)::bigint AS unpriced_deals,
 COALESCE(SUM(cash_net),0)::double precision AS cash_net FROM p
"#;

pub async fn build(
    pool: &PgPool,
    accounts: &[i64],
    period: &Period,
) -> Result<String, sqlx::Error> {
    let mut daily = Vec::new();
    let generated_at = Utc::now();
    for &account in accounts {
        let currency: Option<String> =
            sqlx::query_scalar("SELECT currency FROM accounts WHERE account_id=$1")
                .bind(account)
                .fetch_optional(pool)
                .await?
                .flatten();
        let history = sqlx::query("SELECT window_from,window_to,fetched_at,status FROM deal_fetches WHERE account_id=$1 ORDER BY window_to DESC,fetched_at DESC,id DESC LIMIT 1")
            .bind(account).fetch_optional(pool).await?;
        let coverage = history.as_ref().is_some_and(|r| {
            r.get::<String, _>("status") == "ok"
                && r.get::<DateTime<Utc>, _>("window_from") <= period.from
                && r.get::<DateTime<Utc>, _>("window_to") >= period.to
                && r.get::<DateTime<Utc>, _>("window_to") <= generated_at
                && r.get::<DateTime<Utc>, _>("fetched_at") >= period.to
                && r.get::<DateTime<Utc>, _>("fetched_at") <= generated_at
        });
        for strategy in ["s1", "s3"] {
            let row = sqlx::query(DAY_SQL)
                .bind(account)
                .bind(strategy)
                .bind(period.from)
                .bind(period.to)
                .bind(jiaoyi_store::trading::FILL_KINDS)
                .fetch_one(pool)
                .await?;
            daily.push(json!({"account":account,"strategy":strategy,"currency":currency,
                "positions":row.get::<i64,_>("positions"),"wins":row.get::<i64,_>("wins"),
                "unpriced_positions":row.get::<i64,_>("unpriced_positions"),
                "closing_deals":row.get::<i64,_>("closing_deals"),"unpriced_deals":row.get::<i64,_>("unpriced_deals"),
                "cash_net":row.get::<f64,_>("cash_net"),"history_covers_period":coverage,
                "history_through":history.as_ref().map(|r|r.get::<DateTime<Utc>,_>("window_to")),
                "history_fetched_at":history.as_ref().map(|r|r.get::<DateTime<Utc>,_>("fetched_at"))}));
        }
    }
    let goals = supervisor::reports(pool, accounts).await?;
    let mut account_rows = crate::trading_report_data::collect(pool, accounts, period).await?;
    for row in &mut daily {
        if let Some(account) = account_rows
            .iter()
            .find(|a| a["account_id"] == row["account"])
        {
            row["currency"] = account["currency"].clone();
            row["history_covers_period"] = account["daily"]["history_complete"].clone();
        }
    }
    for account in &mut account_rows {
        let id = account["account_id"].as_i64().unwrap_or_default();
        let mut risks = Vec::new();
        for strategy in [&crate::limits::S1, &crate::limits::S3] {
            let mut risk = crate::limits::health(pool, id, strategy)
                .await
                .unwrap_or_else(|_| json!({"notes":["Registered risk assessment unavailable"]}));
            risk["strategy"] = json!(strategy.code);
            risk["owner_pause"] = crate::limits::owner_pause(pool, id, strategy)
                .await
                .unwrap_or(Value::Null);
            risks.push(risk);
        }
        account["risks"] = json!(risks);
    }
    Ok(render(period, &daily, &account_rows, &goals))
}

pub(crate) fn benchmark_gaps(report: &Value) -> (String, String) {
    let a = &report["report"]["assessment"];
    let w = &a["window"];
    let unknown = "unknown (incomplete or stale evidence)".to_owned();
    if report["monitor_state"] != "alive"
        || a["history_complete"] != true
        || !a["missing_reasons"].as_array().is_some_and(Vec::is_empty)
        || !matches!(
            a["state"].as_str(),
            Some("insufficient_evidence" | "below_target" | "window_meets_numeric_reference")
        )
    {
        return (unknown.clone(), unknown);
    }
    let (Some(count), Some(wins)) = (w["count"].as_u64(), w["wins"].as_u64()) else {
        return (unknown.clone(), unknown);
    };
    let window = jiaoyi_store::goals::WINDOW as u64;
    if count > window || wins > count {
        return (unknown.clone(), unknown);
    }
    if count < window {
        return (
            format!(
                "{} measured closes short of 20; win gap unknown",
                window - count
            ),
            "unknown (insufficient evidence)".into(),
        );
    }
    let win_gap = (wins as f64 / window as f64 - jiaoyi_store::goals::WIN_RATE_TARGET) * 100.0;
    let win_gap = if win_gap < 0.0 {
        format!(
            "{:.2} pp below 73%; {} net wins below 15/20",
            -win_gap,
            15 - wins
        )
    } else {
        format!("{win_gap:.2} pp above 73%; 15/20 minimum met")
    };
    let payoff_gap = match w["realized_payoff_state"].as_str() {
        Some("defined") => w["realized_payoff"]
            .as_f64()
            .filter(|p| p.is_finite() && *p >= 0.0)
            .map(|p| {
                let gap = p - jiaoyi_store::goals::REALIZED_PAYOFF_REFERENCE;
                format!(
                    "{:.6} {} 1.38",
                    gap.abs(),
                    if gap < 0.0 {
                        "below"
                    } else {
                        "above or equal to"
                    }
                )
            })
            .unwrap_or(unknown),
        Some(state @ ("no_wins" | "no_losses" | "no_wins_or_losses")) => {
            format!("unknown ({state})")
        }
        _ => unknown,
    };
    (win_gap, payoff_gap)
}

/// Private deterministic report presentation, separated from delivery state.
pub fn render(period: &Period, daily: &[Value], accounts: &[Value], goals: &Value) -> String {
    crate::trading_report_view::render(period, daily, accounts, goals)
}

#[derive(Debug)]
enum SendError {
    Rejected,
    Uncertain,
}

async fn send_document(
    client: &Client,
    origin: &str,
    config: &DeliveryConfig,
    period: &Period,
    html: &str,
    replace_message: Option<i64>,
) -> Result<i64, SendError> {
    let part = multipart::Part::bytes(html.as_bytes().to_vec())
        .file_name(format!("jiaoyi-trading-{}.html", period.ny_date))
        .mime_str("text/html")
        .map_err(|_| SendError::Rejected)?;
    let form = multipart::Form::new().text("chat_id", config.chat_id.to_string());
    let (method, form) = if let Some(message_id) = replace_message {
        ("editMessageMedia", form.text("message_id", message_id.to_string()).text(
            "media", json!({"type":"document","media":"attach://document",
                "caption":format!("jiaoyi · trading day {} · revised account report",period.ny_date)}).to_string()))
    } else {
        (
            "sendDocument",
            form.text(
                "caption",
                format!(
                    "jiaoyi · trading day {} · 17:00 New York close",
                    period.ny_date
                ),
            ),
        )
    };
    let form = form.part("document", part);
    let response = client
        .post(format!("{origin}/bot{}/{method}", config.token.expose()))
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                SendError::Rejected
            } else {
                SendError::Uncertain
            }
        })?;
    if response.content_length().is_some_and(|n| n > 1024 * 1024) {
        return Err(SendError::Uncertain);
    }
    let bytes = response.bytes().await.map_err(|_| SendError::Uncertain)?;
    if bytes.len() > 1024 * 1024 {
        return Err(SendError::Uncertain);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| SendError::Uncertain)?;
    if value["ok"] != true {
        return Err(SendError::Rejected);
    }
    value["result"]["message_id"]
        .as_i64()
        .ok_or(SendError::Uncertain)
}

/// Upgrade a delivered legacy layout in place once fresh account marks exist.
/// The same Telegram message is edited, so a restart cannot duplicate a report.
async fn refresh_legacy(
    pool: &PgPool,
    accounts: &[i64],
    config: &DeliveryConfig,
    period: &Period,
    origin: &str,
    key: &str,
) -> anyhow::Result<()> {
    if accounts.is_empty() {
        return Ok(());
    }
    let fresh: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT account_id) FROM report_account_snapshots WHERE account_id=ANY($1) AND sample_key='latest' AND observed_at BETWEEN now()-interval '3 minutes' AND now()")
        .bind(accounts).fetch_one(pool).await?;
    // Incomplete/missing accounts belong in the report with explicit unknowns;
    // they must not suppress every other account's delivered evidence.
    if fresh == 0 {
        return Ok(());
    }
    let mut tx = pool.begin().await?;
    let locked: bool =
        sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))")
            .bind(key)
            .fetch_one(&mut *tx)
            .await?;
    if !locked {
        return Ok(());
    }
    let row = sqlx::query("SELECT message_id FROM telegram_reports WHERE report_key=$1 AND status='sent' AND html NOT LIKE '%data-report-version=\"2\"%' AND attempts<5 AND attempted_at<=now()-interval '15 minutes'")
        .bind(key).fetch_optional(&mut *tx).await?;
    let Some(row) = row else {
        return Ok(());
    };
    let message_id: i64 = row.get("message_id");
    let html = build(pool, accounts, period).await?;
    let client = Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let edited = matches!(send_document(&client, origin, config, period, &html, Some(message_id)).await, Ok(id) if id == message_id);
    sqlx::query("UPDATE telegram_reports SET html=CASE WHEN $2 THEN $3 ELSE html END,attempts=attempts+1,attempted_at=now(),last_error=CASE WHEN $2 THEN NULL ELSE 'legacy_revision_not_confirmed' END WHERE report_key=$1")
        .bind(key).bind(edited).bind(html).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(period=%period.ny_date, confirmed=edited, "legacy daily report revision attempted in place");
    Ok(())
}

pub async fn deliver_due(
    pool: &PgPool,
    accounts: &[i64],
    config: &DeliveryConfig,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    deliver_due_at(pool, accounts, config, now, "https://api.telegram.org").await
}

async fn deliver_due_at(
    pool: &PgPool,
    accounts: &[i64],
    config: &DeliveryConfig,
    now: DateTime<Utc>,
    origin: &str,
) -> anyhow::Result<()> {
    let period = period_due(now);
    let key = format!("trading:ny-close:{}", period.ny_date);
    let existing = sqlx::query(
        "SELECT status,attempts,attempted_at,html FROM telegram_reports WHERE report_key=$1",
    )
    .bind(&key)
    .fetch_optional(pool)
    .await?;
    if let Some(r) = existing {
        if r.get::<String, _>("status") == "sent" {
            if r.get::<String, _>("html")
                .contains("data-report-version=\"2\"")
            {
                return Ok(());
            }
            return refresh_legacy(pool, accounts, config, &period, origin, &key).await;
        }
        if r.get::<String, _>("status") != "retry"
            || r.get::<i32, _>("attempts") >= 5
            || now
                .signed_duration_since(r.get::<DateTime<Utc>, _>("attempted_at"))
                .num_seconds()
                < 900
        {
            return Ok(());
        }
    }
    let html = build(pool, accounts, &period).await?;
    let mut tx = pool.begin().await?;
    let locked: bool =
        sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(hashtextextended($1,0))")
            .bind(&key)
            .fetch_one(&mut *tx)
            .await?;
    if !locked {
        return Ok(());
    }
    let claimed:Option<String>=sqlx::query_scalar("INSERT INTO telegram_reports(report_key,period_from,period_to,html,status) VALUES($1,$2,$3,$4,'sending') ON CONFLICT(report_key) DO UPDATE SET status='sending',attempts=telegram_reports.attempts+1,attempted_at=now(),html=EXCLUDED.html WHERE telegram_reports.status='retry' AND telegram_reports.attempts<5 AND telegram_reports.attempted_at<=now()-interval '15 minutes' RETURNING report_key")
        .bind(&key).bind(period.from).bind(period.to).bind(&html).fetch_optional(&mut *tx).await?;
    tx.commit().await?;
    if claimed.is_none() {
        return Ok(());
    }
    let client = Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    match send_document(&client, origin, config, &period, &html, None).await {
        Ok(message_id) => {
            sqlx::query("UPDATE telegram_reports SET status='sent',message_id=$2,sent_at=now(),last_error=NULL WHERE report_key=$1")
                .bind(&key).bind(message_id).execute(pool).await?;
            tracing::info!(period=%period.ny_date,message_id,"Telegram daily HTML delivered");
        }
        Err(error) => {
            let (status, reason) = match error {
                SendError::Rejected => ("retry", "telegram_rejected_or_connection_failed"),
                SendError::Uncertain => ("uncertain", "delivery_uncertain_no_automatic_resend"),
            };
            sqlx::query("UPDATE telegram_reports SET status=$2,last_error=$3 WHERE report_key=$1")
                .bind(&key)
                .bind(status)
                .bind(reason)
                .execute(pool)
                .await?;
            tracing::warn!(period=%period.ny_date,reason,"Telegram daily HTML not confirmed");
        }
    }
    Ok(())
}

pub async fn run(pool: PgPool, accounts: Vec<i64>, mut stop: Shutdown) {
    let Some(config) = DeliveryConfig::from_env() else {
        tracing::info!("Telegram daily trading report disabled or incomplete configuration");
        return;
    };
    loop {
        if stop.is_set() {
            return;
        }
        match tokio::time::timeout(
            Duration::from_secs(90),
            deliver_due(&pool, &accounts, &config, Utc::now()),
        )
        .await
        {
            Ok(Ok(())) => {}
            _ => tracing::warn!("Telegram daily report failed or timed out; trading continues"),
        }
        tokio::select! {_ = stop.wait()=>return,_=tokio::time::sleep(Duration::from_secs(60))=>{}}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn ny_close_delivery_changes_sgt_hour_with_dst_and_never_sends_early() {
        let summer = period_due(at("2026-10-26T21:15:00Z"));
        assert_eq!(summer.to, at("2026-10-26T21:00:00Z"));
        assert_eq!(summer.deliver_after, at("2026-10-26T21:15:00Z"));
        assert_eq!(
            period_due(at("2026-10-26T21:14:59Z")).ny_date,
            NaiveDate::from_ymd_opt(2026, 10, 25).unwrap()
        );
        let winter = period_due(at("2026-11-02T22:15:00Z"));
        assert_eq!(winter.to, at("2026-11-02T22:00:00Z"));
        let spring = period_due(at("2026-03-08T21:15:00Z"));
        assert_eq!((spring.to - spring.from).num_hours(), 23);
        let fall = period_due(at("2026-11-01T22:15:00Z"));
        assert_eq!((fall.to - fall.from).num_hours(), 25);
    }

    #[test]
    fn retained_benchmark_guards_require_complete_fresh_twenty_position_evidence() {
        let mut r = json!({"account_id":1,"strategy":"s3","monitor_state":"alive",
            "report":{"assessment":{"state":"below_target","history_complete":true,
                "missing_reasons":[],"window":{"count":20,"wins":6,
                    "realized_payoff":1.2,"realized_payoff_state":"defined"}}}});
        assert!(benchmark_gaps(&r).0.contains("43.00 pp below"));
        assert!(benchmark_gaps(&r).1.contains("0.180000 below"));
        r["monitor_state"] = json!("stale");
        assert!(benchmark_gaps(&r).0.starts_with("unknown"));
        r["monitor_state"] = json!("alive");
        r["report"]["assessment"]["window"]["count"] = json!(12);
        assert!(benchmark_gaps(&r).0.contains("8 measured closes short"));
        assert!(benchmark_gaps(&r).1.starts_with("unknown"));
        r["report"]["assessment"]["window"]["count"] = json!(20);
        r["report"]["assessment"]["missing_reasons"] = json!(["missing_costs"]);
        assert!(benchmark_gaps(&r).0.starts_with("unknown"));
    }

    async fn server(response: Option<&'static str>) -> (String, tokio::task::JoinHandle<Vec<u8>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let count = stream.read(&mut buf).await.unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&buf[..count]);
                if let Some(end) = request.windows(4).position(|x| x == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]).to_lowercase();
                    let length: usize = headers
                        .lines()
                        .find_map(|line| line.strip_prefix("content-length:"))
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
                assert!(request.len() < 1024 * 1024);
            }
            if let Some(body) = response {
                let reply=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                stream.write_all(reply.as_bytes()).await.unwrap();
            }
            request
        });
        (origin, task)
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn confirmed_html_attachment_is_not_sent_again_after_restart(pool: PgPool) {
        let (origin, request) = server(Some(r#"{"ok":true,"result":{"message_id":72}}"#)).await;
        let config = DeliveryConfig {
            token: Secret::new("TEST_TOKEN"),
            chat_id: 123,
        };
        let now = Utc::now();
        deliver_due_at(&pool, &[], &config, now, &origin)
            .await
            .unwrap();
        let wire = String::from_utf8(request.await.unwrap()).unwrap();
        assert!(wire.starts_with("POST /botTEST_TOKEN/sendDocument "));
        assert!(wire.contains("name=\"chat_id\""));
        assert!(wire.contains("filename=\"jiaoyi-trading-"));
        assert!(wire.contains("Content-Type: text/html"));
        assert!(wire.contains("<!doctype html>"));
        // The fake server has closed; a duplicate network send would fail.
        deliver_due_at(&pool, &[], &config, now, &origin)
            .await
            .unwrap();
        let receipt: (String, i32, Option<i64>) =
            sqlx::query_as("SELECT status,attempts,message_id FROM telegram_reports")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(receipt, ("sent".into(), 1, Some(72)));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn partial_account_evidence_does_not_block_in_place_report_revision(pool: PgPool) {
        let (origin, request) = server(Some(r#"{"ok":true,"result":{"message_id":72}}"#)).await;
        let config = DeliveryConfig {
            token: Secret::new("TEST_TOKEN"),
            chat_id: 123,
        };
        let now = Utc::now();
        let period = period_due(now);
        let key = format!("trading:ny-close:{}", period.ny_date);
        sqlx::query("INSERT INTO telegram_reports(report_key,period_from,period_to,html,status,message_id,sent_at,attempted_at) VALUES($1,$2,$3,'legacy exposure JSON','sent',72,now()-interval '1 hour',now()-interval '1 hour')")
            .bind(&key).bind(period.from).bind(period.to).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES(1,'latest',now(),$1)")
            .bind(json!({"observed_at":now,"source_complete":true,"environment":"demo","currency":"USD","balance":100.0,"float":0.0,"equity":100.0}))
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO report_account_snapshots(account_id,sample_key,observed_at,detail) VALUES(2,'latest',now(),$1)")
            .bind(json!({"observed_at":now,"source_complete":false,"environment":"demo","currency":"EUR","balance":90.0,"float":null,"equity":null}))
            .execute(&pool).await.unwrap();
        deliver_due_at(&pool, &[1, 2, 3], &config, now, &origin)
            .await
            .unwrap();
        let wire = String::from_utf8(
            tokio::time::timeout(Duration::from_secs(2), request)
                .await
                .expect("incomplete account blocked the entire revision")
                .unwrap(),
        )
        .unwrap();
        assert!(wire.starts_with("POST /botTEST_TOKEN/editMessageMedia "));
        assert!(wire.contains("name=\"message_id\""));
        assert!(wire.contains("attach://document"));
        assert!(!wire.contains("/sendDocument"));
        deliver_due_at(&pool, &[1, 2, 3], &config, now, &origin)
            .await
            .unwrap();
        let row = sqlx::query(
            "SELECT message_id,attempts,html,status FROM telegram_reports WHERE report_key=$1",
        )
        .bind(key)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.get::<i64, _>("message_id"), 72);
        assert_eq!(row.get::<i32, _>("attempts"), 2);
        assert_eq!(row.get::<String, _>("status"), "sent");
        assert!(row
            .get::<String, _>("html")
            .contains("data-report-version=\"2\""));
        let html: String = row.get("html");
        assert!(html.contains("Account 2"));
        assert!(html.contains("Account 3"));
        assert!(html.contains("unavailable"));
        assert!(!html
            .split("<div class=cards>")
            .next()
            .unwrap()
            .contains("EUR 90.00"));
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn ambiguous_delivery_is_retained_without_blind_retry(pool: PgPool) {
        let (origin, request) = server(None).await;
        let config = DeliveryConfig {
            token: Secret::new("TEST_TOKEN"),
            chat_id: 123,
        };
        let now = Utc::now();
        deliver_due_at(&pool, &[], &config, now, &origin)
            .await
            .unwrap();
        request.await.unwrap();
        deliver_due_at(&pool, &[], &config, now, &origin)
            .await
            .unwrap();
        let receipt: (String, i32, String) =
            sqlx::query_as("SELECT status,attempts,last_error FROM telegram_reports")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            receipt,
            (
                "uncertain".into(),
                1,
                "delivery_uncertain_no_automatic_resend".into()
            )
        );
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn missing_conversion_fee_or_digits_is_unpriced_not_zero(pool: PgPool) {
        let account = 1_i64;
        let now = Utc::now();
        let p = period_due(now);
        let time = p.to - chrono::Duration::hours(1);
        for id in [1_i64, 2] {
            let key = format!("s3:test:{id}");
            sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,volume,max_risk)VALUES($1,$2,$2,'open',1,100,1)")
                .bind(account).bind(&key).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,volume,raw,received_at)VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$3,100,''::bytea,$4)")
                .bind(account).bind(&key).bind(id).bind(time).execute(&pool).await.unwrap();
            for closing in [false, true] {
                sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,money_digits,is_closing,close_gross_profit,close_swap,close_commission,close_pnl_conversion_fee,raw,fetched_at) VALUES($1,$2,$2,$3,1,100,100,'BUY','FILLED',$4,$4,$5,$6,100,0,-10,$7,''::bytea,$4)")
                    .bind(account).bind(id*10+i64::from(closing)).bind(id).bind(time)
                    .bind(if id==2 {None}else{Some(2_i32)}).bind(closing)
                    .bind(if id==1 {None}else{Some(0_i64)}).execute(&pool).await.unwrap();
            }
        }
        let row = sqlx::query(DAY_SQL)
            .bind(account)
            .bind("s3")
            .bind(p.from)
            .bind(p.to)
            .bind(jiaoyi_store::trading::FILL_KINDS)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.get::<i64, _>("positions"), 2);
        assert_eq!(row.get::<i64, _>("unpriced_deals"), 2);
        assert_eq!(row.get::<i64, _>("unpriced_positions"), 2);
        assert_eq!(row.get::<i64, _>("wins"), 0);
        assert_eq!(row.get::<f64, _>("cash_net"), 0.0);
    }
}
