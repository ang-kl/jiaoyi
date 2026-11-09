//! Account-wide FX preparation runs independently from entry/exit handlers.
//! It only reads broker state, subscribes conversion quotes and records changes.
use super::*;

pub(super) async fn run(
    pool: &PgPool,
    session: &AccountSession<'_>,
    mut shutdown: Shutdown,
) -> anyhow::Result<()> {
    let account = session.account().id();
    let conn = session.connection();
    let mut timer = tokio::time::interval(Duration::from_secs(2));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_revision = None;
    let mut last_attempt = tokio::time::Instant::now() - Duration::from_secs(30);
    let mut last_error = None;
    let mut decisions = Decisions::new(pool, account);
    loop {
        tokio::select! {
            _ = shutdown.wait() => return Ok(()),
            _ = timer.tick() => {}
        }
        let revision = conn.entry_revision(account);
        if last_revision != Some(revision)
            || last_attempt.elapsed()
                >= Duration::from_secs(if last_error.is_some() { 6 } else { 30 })
        {
            last_attempt = tokio::time::Instant::now();
            let warm = async {
                let (trader, book) = tokio::join!(session.trader(), session.margin_book());
                jiaoyi_ctrader::margin::warm_account_conversions(
                    session,
                    &trader.map_err(|e| e.to_string())?,
                    &book.map_err(|e| e.to_string())?,
                )
                .await
            };
            let result = tokio::select! {
                _ = shutdown.wait() => return Ok(()),
                result = warm => result,
            };
            last_error = result.err();
            last_revision = Some(revision);
        }
        let mut status = conn.conversion_status(account);
        status["warmup_error"] = json!(last_error);
        // Prices/timestamps change every quote; persist only path/availability
        // changes, with the contemporaneous full provenance in the detail.
        let states: Vec<_> = status["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                json!({"from_asset":p["from_asset"],"deposit_asset":p["deposit_asset"],
                "legs":p["legs"],"state":p["state"],"subscribed":p["subscribed"]})
            })
            .collect();
        let reason = json!({"paths":states,"warmup_error":last_error}).to_string();
        decisions.note(None, "exposure_fx", &reason, status).await;
    }
}

pub(super) async fn health(pool: &PgPool, account: i64) -> Value {
    match ts::latest_decision_detail(pool, account, "s3_exposure_fx").await {
        Ok(Some((at, detail))) => json!({"last_state_change_at":at,"recorded_provenance":detail,
            "freshness_policy":"Each entry revalidates every conversion leg at the writer; this record is not a live quote."}),
        Ok(None) => json!({"state":"not_observed"}),
        Err(_) => json!({"state":"unavailable"}),
    }
}
