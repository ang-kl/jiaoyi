//! Owner-triggered bounded research jobs. No token refresh, trading writer or DB writes.
use crate::{
    http::AppState,
    owner_assessments::{private_response, refusal, OwnerAccess},
};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Extension, Json};
use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::research::{
    s3_signal::{self, Bar, Request},
    s3_ticks,
};
use jiaoyi_ctrader::{decode_bar, messages::ProtoOaTrendbarPeriod};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Jobs(pub Arc<Mutex<Value>>);

/// Recent, post-holdout quote acquisition. No replay parameters or trading action.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionRequest {
    pub account: i64,
    pub symbol: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}
impl CollectionRequest {
    fn validate(&self, now: DateTime<Utc>) -> Result<Request, String> {
        let first = jiaoyi_core::seal::held_out_to()
            .succ_opt()
            .ok_or("date overflow")?
            .and_hms_opt(0, 0, 0)
            .ok_or("date overflow")?
            .and_utc();
        if self.account <= 0
            || self.from < first
            || self.from >= self.to
            || self
                .to
                .checked_add_signed(Duration::seconds(5))
                .is_none_or(|end| end > now)
            || self.from.timestamp() % 1800 != 0
            || self.to.timestamp() % 1800 != 0
            || self.from.timestamp_subsec_nanos() != 0
            || self.to.timestamp_subsec_nanos() != 0
        {
            return Err("collection requires a closed post-holdout window from 2026-10-24, including its five-second tail".into());
        }
        // Reuse the registered session/window validator only. The development
        // replay's seal stays unchanged, and this request never enters a study.
        let req = Request {
            account: self.account,
            symbol: self.symbol.clone(),
            from: self.from,
            to: self.to,
            spread: 0.0,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(s3_ticks::Options {
                volume: 1,
                entry_rule: jiaoyi_core::research::opening_range::EntryRule::ClassMomentum1h,
            }),
        };
        jiaoyi_core::research::class_prediction::validate(&req)?;
        Ok(req)
    }
}
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
    if matches!(req.symbol.as_str(), "SpotCrude" | "SpotBrent")
        && req
            .tick_replay
            .as_ref()
            .is_some_and(|o| o.entry_rule.is_current())
    {
        return response(
            StatusCode::BAD_REQUEST,
            json!({"error":"prepared oil S3 requires the offline evidence adapter; no collection started"}),
        );
    }
    let envelope = json!({"request":req});
    start(state, jobs, req, envelope, false)
}

pub async fn collect(
    State(state): State<AppState>,
    Extension(access): Extension<OwnerAccess>,
    Extension(jobs): Extension<Jobs>,
    headers: HeaderMap,
    Json(collection): Json<CollectionRequest>,
) -> Response {
    if let Err(s) = access.authorise(&headers) {
        return refusal(s);
    }
    if !state.allowed_accounts.contains(&collection.account) {
        return response(
            StatusCode::FORBIDDEN,
            json!({"error":"account_not_allowed"}),
        );
    }
    let req = match collection.validate(Utc::now()) {
        Ok(req) => req,
        Err(e) => return response(StatusCode::BAD_REQUEST, json!({"error":e})),
    };
    let envelope = json!({"request":collection,"kind":"recent_market_data"});
    start(state, jobs, req, envelope, true)
}

fn start(
    state: AppState,
    jobs: Jobs,
    req: Request,
    mut envelope: Value,
    collection: bool,
) -> Response {
    let id = Utc::now().timestamp_micros().to_string();
    {
        let mut job = jobs.0.lock().unwrap();
        if job["status"] == "running" {
            return response(
                StatusCode::CONFLICT,
                json!({"error":"one_research_job_at_a_time"}),
            );
        }
        envelope["id"] = json!(id);
        envelope["status"] = json!("running");
        envelope["phase"] = json!("starting");
        envelope["started_at"] = json!(Utc::now());
        *job = envelope;
    }
    let task_id = id.clone();
    tokio::spawn(async move {
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(240),
            run(&state, &jobs, &req, collection),
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
        json!({"id":id,"status":"running","poll":if collection {"/owner/market-data"} else {"/owner/backtests"}}),
    )
}
fn phase(jobs: &Jobs, name: &str) {
    jobs.0.lock().unwrap()["phase"] = json!(name);
}
async fn run(
    state: &AppState,
    jobs: &Jobs,
    req: &Request,
    collection: bool,
) -> Result<Value, &'static str> {
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
        if collection {
            phase(jobs, "validating_collection");
            validate_collection_source(req, &source)?;
            let digest = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&source).map_err(|_| "source_encoding_failed")?)
            );
            return Ok(json!({
                "source":"cTrader Open API separate BID/ASK tick history",
                "symbol_id":id,"environment":env.as_str(),"input":source,
                "input_sha256":digest,"tick_pages":budget.pages,"m30_pages":0,
                "study":null,"qualification":"RECENT_COLLECTION_ONLY_NOT_FINANCIAL_PASS",
                "collector_sha256":format!("{:x}",Sha256::digest(include_bytes!("broker_backtest.rs"))),
                "coverage":{"bid_ticks":source.bids.len(),"ask_ticks":source.asks.len(),
                    "both_sides_present":!source.bids.is_empty() && !source.asks.is_empty(),
                    "requested_from":req.from,"requested_through":req.to + Duration::seconds(5),
                    "quote_continuity":"unproven","contract_metadata":"current_snapshot"},
                "contract_details":{"digits":contract.digits,"min_volume":contract.min_volume,
                    "step_volume":contract.step_volume,"lot_size":contract.lot_size},
                "collected_at":Utc::now()
            }));
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

fn validate_collection_source(
    req: &Request,
    source: &s3_ticks::Source,
) -> Result<(), &'static str> {
    if !(0..=5).contains(&source.contract.digits)
        || source.contract.min_volume <= 0
        || source.contract.step_volume <= 0
        || source.bids.len() + source.asks.len() > s3_ticks::MAX_TICKS
    {
        return Err("collection_contract_or_size_invalid");
    }
    let quantum = 10_i64.pow((5 - source.contract.digits) as u32);
    for ticks in [&source.bids, &source.asks] {
        let mut previous = None;
        for tick in ticks {
            if previous.is_some_and(|p| tick.ts_ms < p)
                || tick.ts_ms < req.from.timestamp_millis()
                || tick.ts_ms > req.to.timestamp_millis() + 5000
                || tick.price_e5 <= 0
                || tick.price_e5 % quantum != 0
            {
                return Err("collection_tick_invalid");
            }
            previous = Some(tick.ts_ms);
        }
    }
    Ok(())
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
        include_bytes!("../../jiaoyi-core/src/research/s3_signal.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/research/fx_diagnostic.rs").as_slice(),
        include_bytes!("../../jiaoyi-core/src/research/class_prediction.rs").as_slice(),
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

#[cfg(test)]
mod collection_tests {
    use super::*;
    fn request() -> CollectionRequest {
        CollectionRequest {
            account: 49236051,
            symbol: "BTCUSD".into(),
            from: "2026-11-05T00:00:00Z".parse().unwrap(),
            to: "2026-11-05T04:00:00Z".parse().unwrap(),
        }
    }
    #[test]
    fn recent_collection_is_closed_post_holdout_and_never_opens_development_replay() {
        let mut r = request();
        let now = "2026-11-06T00:00:00Z".parse().unwrap();
        let req = r.validate(now).unwrap();
        assert!(req.validate(now).is_err()); // Original development seal remains.
        assert!(r.validate(r.to + Duration::seconds(4)).is_err());
        assert!(r.validate(r.to + Duration::seconds(5)).is_ok());
        r.from = "2026-10-23T00:00:00Z".parse().unwrap();
        r.to = r.from + Duration::hours(4);
        assert!(r.validate(now).is_err());
        r = request();
        r.symbol = "UNREGISTERED".into();
        assert!(r.validate(now).is_err());
        r = request();
        r.to += Duration::hours(1);
        assert!(r.validate(now).is_err());
    }
    #[test]
    fn collection_rejects_bad_grid_order_and_out_of_window_ticks() {
        let r = request();
        let req = r.validate(r.to + Duration::seconds(5)).unwrap();
        let tick = s3_ticks::Tick {
            ts_ms: r.from.timestamp_millis(),
            price_e5: 100_000,
        };
        let mut src = s3_ticks::Source {
            bars: vec![],
            bids: vec![tick.clone()],
            asks: vec![tick],
            contract: s3_ticks::Contract {
                digits: 2,
                min_volume: 1,
                step_volume: 1,
            },
        };
        assert!(validate_collection_source(&req, &src).is_ok());
        src.bids[0].price_e5 += 1;
        assert!(validate_collection_source(&req, &src).is_err());
        src.bids[0] = src.asks[0].clone();
        src.bids.push(s3_ticks::Tick {
            ts_ms: r.from.timestamp_millis() - 1,
            price_e5: 100_000,
        });
        assert!(validate_collection_source(&req, &src).is_err());
        src.bids.clear();
        src.asks.clear();
        assert!(validate_collection_source(&req, &src).is_ok()); // Empty fetch is explicit coverage, not a price.
    }
}
