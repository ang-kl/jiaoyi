//! Owner-triggered bounded research jobs. No token refresh, trading writer or DB writes.
use crate::{
    http::AppState,
    owner_assessments::{private_response, refusal, OwnerAccess},
};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Extension, Json};
use chrono::{Duration, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::research::{
    s3_signal::{self, Bar, Request},
    s3_ticks,
};
use jiaoyi_ctrader::{decode_bar, messages::ProtoOaTrendbarPeriod};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Jobs(pub Arc<Mutex<Value>>);
fn response(status: StatusCode, v: Value) -> Response {
    private_response((status, Json(v)).into_response())
}
pub async fn latest(
    Extension(access): Extension<OwnerAccess>,
    Extension(jobs): Extension<Jobs>,
    headers: HeaderMap,
) -> Response {
    if let Err(s) = access.authorise(&headers) {
        return refusal(s);
    }
    response(StatusCode::OK, jobs.0.lock().unwrap().clone())
}
pub async fn submit(
    State(state): State<AppState>,
    Extension(access): Extension<OwnerAccess>,
    Extension(jobs): Extension<Jobs>,
    headers: HeaderMap,
    Json(req): Json<Request>,
) -> Response {
    if let Err(s) = access.authorise(&headers) {
        return refusal(s);
    }
    if !state.allowed_accounts.contains(&req.account) {
        return response(
            StatusCode::FORBIDDEN,
            json!({"error":"account_not_allowed"}),
        );
    }
    if let Err(e) = req.validate(Utc::now()) {
        return response(StatusCode::BAD_REQUEST, json!({"error":e}));
    }
    let id = Utc::now().timestamp_micros().to_string();
    {
        let mut job = jobs.0.lock().unwrap();
        if job["status"] == "running" {
            return response(
                StatusCode::CONFLICT,
                json!({"error":"one_research_job_at_a_time"}),
            );
        }
        *job = json!({"id":id,"status":"running","phase":"starting","request":req,"started_at":Utc::now()});
    }
    let task_id = id.clone();
    tokio::spawn(async move {
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(240),
            run(&state, &jobs, &req),
        )
        .await;
        let mut job = jobs.0.lock().unwrap();
        job["finished_at"] = json!(Utc::now());
        match outcome {
            Ok(Ok(result)) => {
                job["status"] = json!("complete");
                job["result"] = result;
            }
            Ok(Err(stage)) => {
                job["status"] = json!("failed");
                job["error"] = json!(stage);
            }
            Err(_) => {
                job["status"] = json!("failed");
                job["error"] = json!("research_timeout_240s");
            }
        }
        job["id"] = json!(task_id);
    });
    response(
        StatusCode::ACCEPTED,
        json!({"id":id,"status":"running","poll":"/owner/backtests"}),
    )
}
fn phase(jobs: &Jobs, name: &str) {
    jobs.0.lock().unwrap()["phase"] = json!(name);
}
async fn run(state: &AppState, jobs: &Jobs, req: &Request) -> Result<Value, &'static str> {
    let config = crate::config::Config::from_env().map_err(|_| "configuration_unavailable")?;
    let account = config
        .allowlist
        .permit(req.account)
        .map_err(|_| "account_not_allowed")?;
    let env = jiaoyi_store::account_environment(&state.pool, req.account)
        .await
        .map_err(|_| "account_environment_unavailable")?
        .ok_or("account_environment_unknown")?;
    phase(jobs, "authorising_read_only");
    let conn = crate::loader::connect(&config.broker, env)
        .await
        .map_err(|_| "broker_connection_failed")?;
    conn.application_auth(
        config
            .broker
            .client_id
            .as_deref()
            .ok_or("client_id_missing")?,
        config
            .broker
            .client_secret
            .as_ref()
            .ok_or("client_secret_missing")?,
    )
    .await
    .map_err(|_| "application_auth_failed")?;
    // Read current shared token only. Never replace the seed or refresh it here.
    let token = jiaoyi_store::load_token(&state.pool)
        .await
        .map_err(|_| "token_read_failed")?
        .ok_or("current_token_unavailable")?;
    let grant = conn
        .account_list(&token.access_token)
        .await
        .map_err(|_| "token_grant_read_failed_no_refresh_attempted")?;
    if grant.account(req.account).and_then(|a| a.is_live) != Some(env == Environment::Live) {
        return Err("broker_grant_environment_mismatch");
    }
    let session = conn
        .account_auth(&account, &token.access_token)
        .await
        .map_err(|_| "account_auth_failed_no_refresh_attempted")?;
    let list = session
        .symbols_list()
        .await
        .map_err(|_| "symbols_read_failed")?;
    let candidates: Vec<_> = list
        .symbol
        .iter()
        .filter(|s| s.symbol_name.as_deref() == Some(&req.symbol))
        .collect();
    if candidates.len() != 1 {
        return Err("exact_symbol_missing_or_ambiguous");
    }
    let id = candidates[0].symbol_id;
    let details = session
        .symbols_by_id(&[id])
        .await
        .map_err(|_| "contract_read_failed")?;
    let contract = details
        .symbol
        .iter()
        .find(|s| s.symbol_id == id)
        .ok_or("contract_missing")?;
    phase(jobs, "fetching_m30");
    let candidate = req
        .tick_replay
        .as_ref()
        .is_some_and(|o| !o.entry_rule.is_current());
    // Candidate preparation uses only its tick marks, not 50 days of S3 bands.
    let mut cursor = if candidate {
        req.to
    } else {
        req.history_from()
    };
    let mut bars = std::collections::BTreeMap::new();
    let mut pages = 0;
    while cursor < req.to {
        let end = (cursor + Duration::days(1)).min(req.to);
        let page = session
            .research_m30_page(id, cursor, end - Duration::milliseconds(1))
            .await
            .map_err(|_| "m30_fetch_failed_or_truncated")?;
        for raw in page {
            let decoded =
                decode_bar(&raw, ProtoOaTrendbarPeriod::M30).map_err(|_| "m30_decode_failed")?;
            if decoded.open_time < cursor
                || decoded.open_time >= end
                || decoded.open_time + Duration::minutes(30) > req.to
            {
                return Err("broker_bar_outside_requested_closed_window");
            }
            let bar = Bar {
                at: decoded.open_time,
                open: decoded
                    .open
                    .to_string()
                    .parse()
                    .map_err(|_| "price_decode_failed")?,
                close: decoded
                    .close
                    .to_string()
                    .parse()
                    .map_err(|_| "price_decode_failed")?,
            };
            if let Some(old) = bars.insert(bar.at, bar.clone()) {
                if old != bar {
                    return Err("conflicting_duplicate_broker_bar");
                }
            }
        }
        // Keep research below one historical request/second on its own connection.
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
        pages += 1;
        cursor = end;
        jobs.0.lock().unwrap()["fetched_pages"] = json!(pages);
    }
    let bars: Vec<_> = bars.into_values().collect();
    if bars.is_empty() && !candidate {
        return Err("no_broker_bars_returned");
    }
    if req.tick_replay.is_some() {
        phase(jobs, "fetching_bounded_ticks");
        let mut budget = TickBudget::default();
        let mut source = s3_ticks::Source {
            bars,
            bids: vec![],
            asks: vec![],
            contract: s3_ticks::Contract {
                digits: contract.digits,
                min_volume: contract.min_volume.ok_or("minimum_volume_missing")?,
                step_volume: contract.step_volume.ok_or("volume_step_missing")?,
            },
        };
        for side in [
            jiaoyi_ctrader::tickdata::QuoteSide::Bid,
            jiaoyi_ctrader::tickdata::QuoteSide::Ask,
        ] {
            let ticks = bounded_ticks(
                &session,
                id,
                side,
                req.from.timestamp_millis(),
                req.to.timestamp_millis() + 5000,
                &mut budget,
            )
            .await?;
            if side == jiaoyi_ctrader::tickdata::QuoteSide::Bid {
                source.bids = ticks;
            } else {
                source.asks = ticks;
            }
            jobs.0.lock().unwrap()["tick_pages"] = json!(budget.pages);
        }
        phase(jobs, "replaying_ticks");
        let study = s3_ticks::study(req, &source).map_err(|_| "tick_replay_input_invalid")?;
        let digest = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&source).map_err(|_| "source_encoding_failed")?)
        );
        return Ok(
            json!({"source":"cTrader Open API separate BID/ASK tick history + M30 warm-up",
            "symbol_id":id,"environment":env.as_str(),"input":source,"study":study,
            "input_sha256":digest,"m30_pages":pages,"tick_pages":budget.pages,
            "qualification":"DEVELOPMENT_SCENARIO_NOT_FINANCIAL_PASS", "algorithm_version":"bounded-s3-tick-replay-v1",
            "algorithm_sha256":tick_algorithm_sha256()}),
        );
    }
    phase(jobs, "analysing");
    let study = s3_signal::study(req, &bars).map_err(|_| "study_input_invalid")?;
    let bytes = serde_json::to_vec(&bars).map_err(|_| "source_encoding_failed")?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    Ok(
        json!({"source":"cTrader Open API M30","symbol_id":id,"environment":env.as_str(),
        "contract":{"digits":contract.digits,"lot_size":contract.lot_size,"min_volume":contract.min_volume,"step_volume":contract.step_volume},
        "input_sha256":digest,"fetched_pages":pages,"history_from":req.history_from(),"bars":bars,"study":study,
        "qualification":"DIAGNOSTIC_ONLY_NOT_FINANCIAL_PASS","algorithm_version":"s3-mark-followthrough-v1", "algorithm_sha256":format!("{:x}",Sha256::digest(include_bytes!("../../jiaoyi-core/src/research/s3_signal.rs")))}),
    )
}

/// Bisection discards truncated responses and fetches disjoint complete leaves.
/// Never skip an oldest timestamp or quietly accept an empty hasMore response.
#[derive(Default)]
pub struct TickBudget {
    pub pages: usize,
    pub rows: usize,
}
pub async fn bounded_ticks(
    session: &jiaoyi_ctrader::AccountSession<'_>,
    id: i64,
    side: jiaoyi_ctrader::tickdata::QuoteSide,
    from: i64,
    to: i64,
    budget: &mut TickBudget,
) -> Result<Vec<s3_ticks::Tick>, &'static str> {
    if from < 0 || to < from || to - from > 86_405_000 {
        return Err("tick_window_invalid");
    }
    let mut pending = vec![(from, to)];
    let mut out = vec![];
    while let Some((a, b)) = pending.pop() {
        if budget.pages >= s3_ticks::MAX_PAGES {
            return Err("tick_page_budget_exceeded_no_result");
        }
        budget.pages += 1;
        let (ticks, more) = session
            .tick_page(id, side, a, b)
            .await
            .map_err(|_| "tick_fetch_or_decode_failed")?;
        if more {
            if ticks.is_empty() || a == b {
                return Err("tick_truncation_not_resolvable_no_result");
            }
            let mid = a + (b - a) / 2;
            pending.push((mid + 1, b));
            pending.push((a, mid));
        } else {
            budget.rows += ticks.len();
            if budget.rows > s3_ticks::MAX_TICKS {
                return Err("tick_row_budget_exceeded_no_result");
            }
            out.extend(ticks.into_iter().rev().map(|t| s3_ticks::Tick {
                ts_ms: t.ts_ms,
                price_e5: t.price_e5,
            }));
        }
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
    }
    Ok(out)
}
fn tick_algorithm_sha256() -> String {
    let mut h = Sha256::new();
    for b in [
        include_bytes!("../../jiaoyi-core/src/research/s3_ticks.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/research/opening_range.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/research/exit_replay.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/strategy3/band.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/strategy3/targets.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/strategy3/sessions.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/strategy3/mod.rs").as_slice(),
    ] {
        h.update(b);
    }
    format!("{:x}", h.finalize())
}
