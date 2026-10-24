//! HTTP on $PORT: `/health`, `/phase1/report` and `/research/inventory`.
//! Read-only: no route writes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use axum::extract::State;
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
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/phase1/report", get(report))
        .route("/research/inventory", get(inventory))
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
