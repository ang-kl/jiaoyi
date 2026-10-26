//! Deterministic NY-close HTML files. Telegram failures cannot mutate trading.
use std::time::Duration;

use chrono::{DateTime, Days, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::{America::New_York, Asia::Singapore};
use jiaoyi_core::secret::Secret;
use jiaoyi_store::sqlx::{self, PgPool, Row};
use reqwest::{multipart, Client};
use serde_json::{json, Value};

use crate::{shutdown::Shutdown, status_page::esc, supervisor};

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
    let mut books = Vec::new();
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
            let kind = if strategy == "s1" {
                "reconcile"
            } else {
                "s3_reconcile"
            };
            let book = sqlx::query("SELECT at,detail FROM trading_decisions WHERE account_id=$1 AND kind=$2 ORDER BY id DESC LIMIT 1")
                .bind(account).bind(kind).fetch_optional(pool).await?;
            books.push(json!({"account":account,"strategy":strategy,"at":book.as_ref().map(|r|r.get::<DateTime<Utc>,_>("at")),
                "detail":book.as_ref().map(|r|r.get::<Value,_>("detail"))}));
        }
    }
    let goals = supervisor::reports(pool, accounts).await?;
    Ok(render(period, &daily, &books, &goals))
}

fn num(v: &Value, scale: f64) -> String {
    v.as_f64()
        .filter(|x| x.is_finite())
        .map(|x| format!("{:.2}", x * scale))
        .unwrap_or_else(|| "unknown".into())
}

fn sgt_time(at: DateTime<Utc>) -> String {
    at.with_timezone(&Singapore)
        .format("%d-%m-%Y %H:%M:%S SGT")
        .to_string()
}

fn source_time(value: &Value) -> String {
    value
        .as_str()
        .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        .map(sgt_time)
        .unwrap_or_else(|| "unknown".into())
}

pub fn render(period: &Period, daily: &[Value], books: &[Value], goals: &Value) -> String {
    let serial = format!("T-{}", period.ny_date.format("%Y%m%d"));
    let mut body=format!("<h1>Trading day ending {} · 17:00 New York</h1><p>SGT window [{} → {}). Generated {}. Delivery after 17:15 NY (05:15/06:15 SGT, DST-aware). This is an accounting cutoff, not a claim that every CFD or crypto venue is closed.</p><details><summary>Technical UTC window</summary><p>[{} → {})</p></details>",period.ny_date,sgt_time(period.from),sgt_time(period.to),sgt_time(Utc::now()),period.from,period.to);
    body.push_str("<h2>Day's executions</h2><table><tr><th>Account / strategy</th><th>Fully closed positions / known wins</th><th>Closing deals</th><th>Known cash net</th><th>Unpriced deals / positions</th><th>History coverage</th></tr>");
    for r in daily {
        let cash = if r["unpriced_deals"].as_i64().is_some_and(|n| n > 0)
            && r["closing_deals"] == r["unpriced_deals"]
        {
            "unknown".into()
        } else {
            num(&r["cash_net"], 1.0)
        };
        body.push_str(&format!("<tr><td>{} / {}</td><td>{} / {}</td><td>{}</td><td>{} {}</td><td>{} / {}</td><td>{}; through {}</td></tr>",
            esc(&r["account"].to_string()),esc(r["strategy"].as_str().unwrap_or("unknown")),r["positions"],r["wins"],r["closing_deals"],cash,esc(r["currency"].as_str().unwrap_or("deposit currency unknown")),r["unpriced_deals"],r["unpriced_positions"],if r["history_covers_period"]==true {"covers window"} else {"partial / unknown"},esc(&source_time(&r["history_through"]))));
    }
    body.push_str("</table><p>Cash net sums only owned closing deals executed in this window, including partial closes: gross + swap − |closing commission| − |conversion fee|. Missing money fields are unpriced, never zero. Fully closed positions are assigned by final close time; known wins are not a complete win rate when any position is unpriced. Account currencies are never added together. Late history can revise these observations.</p><h2>Latest-20 reference · snapshot at generation</h2><table><tr><th>Account / strategy</th><th>State / monitor</th><th>Wins / closes</th><th>Win rate</th><th>Realized payoff</th><th>Net R</th><th>Persisted check SGT</th><th>Missing evidence</th></tr>");
    for r in goals["reports"].as_array().into_iter().flatten() {
        let a = &r["report"]["assessment"];
        let w = &a["window"];
        body.push_str(&format!("<tr><td>{} / {}</td><td>{} / {}</td><td>{} / {}</td><td>{}%</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",esc(&r["account_id"].to_string()),esc(r["strategy"].as_str().unwrap_or("unknown")),esc(a["state"].as_str().unwrap_or("unknown")),esc(r["monitor_state"].as_str().unwrap_or("unknown")),w["wins"],w["count"],num(&w["win_rate"],100.0),num(&w["realized_payoff"],1.0),num(&w["net_r"],1.0),esc(&source_time(&r["checked_at"])),esc(&a["missing_reasons"].to_string())));
    }
    body.push_str("</table><p>Target is ≥73% net wins (15/20=75%) and realized payoff reference1.38, separately for each strategy/account. Registered stop/target geometry is unchanged. One passing overlapping window does not establish consistency. These latest-20 cohorts are separate from the daily cash window.</p><h2>Latest reconciled exposure</h2>");
    for b in books {
        body.push_str(&format!(
            "<details><summary>{} / {} · receipt {}</summary><pre>{}</pre></details>",
            esc(&b["account"].to_string()),
            esc(b["strategy"].as_str().unwrap_or("unknown")),
            esc(&source_time(&b["at"])),
            esc(&b["detail"].to_string())
        ));
    }
    body.push_str("<p>Exposure receipts can be stale and are not a fresh broker snapshot. Current broker-side stop prices are not reconstructed from incomplete receipts. Existing pre-orders, stops/targets, sizing, account activation and owner-only pause decisions are untouched. The report is deterministic: no additional LLM call or model tokens. <a href=\"https://jiaoyi.up.railway.app/operations\">Current assessments</a> · <a href=\"https://jiaoyi.up.railway.app/\">Trading desk</a></p>");
    let mut numbered = String::new();
    let mut previous = 0;
    for (index, (at, _)) in body.match_indices("<p>").enumerate() {
        numbered.push_str(&body[previous..at + 3]);
        numbered.push_str(&format!("[§{serial}.{}] ", index + 1));
        previous = at + 3;
    }
    numbered.push_str(&body[previous..]);
    format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>jiaoyi trading {}</title><style>body{{font:15px system-ui;max-width:1100px;margin:2rem auto;padding:0 1rem;color:#17202a}}table{{border-collapse:collapse;width:100%;font-size:.85rem}}td,th{{padding:.5rem;border-bottom:1px solid #ccc;text-align:left}}pre{{white-space:pre-wrap;overflow-wrap:anywhere}}p{{line-height:1.5}}</style><body><div>№{serial}</div>{numbered}</body></html>",period.ny_date)
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
) -> Result<i64, SendError> {
    let part = multipart::Part::bytes(html.as_bytes().to_vec())
        .file_name(format!("jiaoyi-trading-{}.html", period.ny_date))
        .mime_str("text/html")
        .map_err(|_| SendError::Rejected)?;
    let form = multipart::Form::new()
        .text("chat_id", config.chat_id.to_string())
        .text(
            "caption",
            format!(
                "jiaoyi · trading day {} · 17:00 New York close",
                period.ny_date
            ),
        )
        .part("document", part);
    let response = client
        .post(format!(
            "{origin}/bot{}/sendDocument",
            config.token.expose()
        ))
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
        "SELECT status,attempts,attempted_at FROM telegram_reports WHERE report_key=$1",
    )
    .bind(&key)
    .fetch_optional(pool)
    .await?;
    if let Some(r) = existing {
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
    match send_document(&client, origin, config, &period, &html).await {
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
    fn html_escapes_source_text_and_keeps_unknowns_separate_from_zero() {
        let p = period_due(at("2026-10-26T22:00:00Z"));
        let html = render(
            &p,
            &[
                json!({"account":1,"strategy":"<script>alert(1)</script>","cash_net":0,"currency":"USD","positions":2,"wins":0,"unpriced_positions":2,"closing_deals":2,"unpriced_deals":2,"history_covers_period":false}),
            ],
            &[],
            &json!({"reports":[]}),
        );
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("unknown USD"));
        assert!(html.contains("partial / unknown"));
        assert!(html.contains("27-10-2026 05:00:00 SGT"));
        assert!(html.contains("no additional LLM call"));
        assert!(html.contains("№T-20261026"));
        assert!(html.contains("[§T-20261026.1]"));
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
