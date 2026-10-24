//! HTTP on $PORT: `/health`, `/phase1/report`, `/research/inventory` and
//! `/research/bars`. Read-only: no route writes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
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
    /// The account allowlist (`JIAOYI_ALLOWED_ACCOUNTS`) as permitted at start-up.
    pub allowed_accounts: Arc<std::collections::BTreeSet<i64>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/phase1/report", get(report))
        .route("/research/inventory", get(inventory))
        .route("/research/bars", get(bars))
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
    match crate::report::health(&s.pool, s.loading.load(Ordering::Relaxed), recorders).await {
        Ok(mut v) => {
            v["history_sync"] = serde_json::to_value(history).unwrap_or_default();
            Json(v).into_response()
        }
        Err(e) => unavailable(e),
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
