//! HTTP on $PORT: `/` (the status page), `/health`, `/phase1/report`,
//! `/research/inventory`, `/research/bars`, `/research/jobs` and
//! `/research/trials`. Read-only: no route writes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::get;
use axum::Router;
use jiaoyi_store::sqlx::PgPool;
use serde_json::json;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub loading: Arc<AtomicBool>,
    pub recorders: crate::recorder::StatusMap,
    pub history: crate::history::HistoryStatusMap,
    /// Each account's Strategy 1 loop: its reconcile runs and last quote.
    pub trading_loops: crate::trading::LivenessMap,
    /// The account allowlist (`JIAOYI_ALLOWED_ACCOUNTS`) as permitted at start-up.
    pub allowed_accounts: Arc<std::collections::BTreeSet<i64>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(status_page))
        .route("/health", get(health))
        .route("/phase1/report", get(report))
        .route("/research/inventory", get(inventory))
        .route("/research/bars", get(bars))
        .route("/research/jobs", get(jobs))
        .route("/research/trials", get(trials))
        .with_state(state)
}

fn unavailable(e: impl std::fmt::Display) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "status": "error", "error": e.to_string() })),
    )
        .into_response()
}

async fn health(State(s): State<AppState>) -> Response {
    let recorders = s.recorders.lock().unwrap().clone();
    let history = s.history.lock().unwrap().clone();
    let loops = s.trading_loops.lock().unwrap().clone();
    match crate::report::health(&s.pool, s.loading.load(Ordering::Relaxed), recorders, loops).await
    {
        Ok(mut v) => {
            v["history_sync"] = serde_json::to_value(history).unwrap_or_default();
            Json(v).into_response()
        }
        Err(e) => unavailable(e),
    }
}

/// `/`: the read-only status page, drawn from the same values as `/health`,
/// `/research/jobs` and `/research/trials`.
async fn status_page(State(s): State<AppState>) -> Response {
    let recorders = s.recorders.lock().unwrap().clone();
    let loops = s.trading_loops.lock().unwrap().clone();
    let values = async {
        let health =
            crate::report::health(&s.pool, s.loading.load(Ordering::Relaxed), recorders, loops)
                .await?;
        let jobs = crate::edge::routes::jobs(&s.pool).await?;
        let trials = crate::edge::routes::trials(&s.pool).await?;
        Ok::<_, jiaoyi_store::StoreError>((health, jobs, trials))
    };
    match values.await {
        Ok((health, jobs, trials)) => {
            Html(crate::status_page::page(&health, &jobs, &trials, chrono::Utc::now()))
                .into_response()
        }
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Html(format!(
                "<!doctype html><meta charset=\"utf-8\"><title>jiaoyi status</title><p>The status is unavailable: {}</p><p><a href=\"/health\">/health</a></p>",
                crate::status_page::esc(&e.to_string())
            )),
        )
            .into_response(),
    }
}

async fn inventory(State(s): State<AppState>) -> Response {
    match crate::research::build(&s.pool).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => unavailable(e),
    }
}

async fn report(State(s): State<AppState>) -> Response {
    match crate::report::build(&s.pool).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => unavailable(e),
    }
}

async fn bars(State(s): State<AppState>, Query(q): Query<crate::research::BarsQuery>) -> Response {
    use crate::research::BarsRefusal;
    match crate::research::bars(&s.pool, &s.allowed_accounts, &q).await {
        Ok(v) => Json(v).into_response(),
        Err(BarsRefusal::Store(e)) => unavailable(e),
        Err(e) => {
            let status = match e {
                BarsRefusal::BadRequest(_) => StatusCode::BAD_REQUEST,
                BarsRefusal::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::NOT_FOUND,
            };
            (
                status,
                Json(json!({ "status": "refused", "reason": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn jobs(State(s): State<AppState>) -> Response {
    match crate::edge::routes::jobs(&s.pool).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => unavailable(e),
    }
}

async fn trials(State(s): State<AppState>) -> Response {
    match crate::edge::routes::trials(&s.pool).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => unavailable(e),
    }
}
