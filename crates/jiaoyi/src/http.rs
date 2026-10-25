//! HTTP on $PORT: `/` (the status page), `/health`, `/phase1/report`,
//! `/research/inventory`, `/research/bars`, `/research/jobs`,
//! `/research/trials`, `/trading/costs` and `/trading/orders`. Read-only: no
//! route writes.

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
    /// Each account's Strategy 3 loop, the same way.
    pub strategy3_loops: crate::trading::LivenessMap,
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
        .route("/trading/costs", get(trading_costs))
        .route("/trading/orders", get(trading_orders))
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
    let s3_loops = s.strategy3_loops.lock().unwrap().clone();
    match crate::report::health(
        &s.pool,
        s.loading.load(Ordering::Relaxed),
        recorders,
        loops,
        s3_loops,
    )
    .await
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
    let s3_loops = s.strategy3_loops.lock().unwrap().clone();
    let values = async {
        let health = crate::report::health(
            &s.pool,
            s.loading.load(Ordering::Relaxed),
            recorders,
            loops,
            s3_loops,
        )
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

/// `/trading/costs`: each strategy's actual costs and order-to-fill times
/// from its own fills. Prices and milliseconds only, no money.
async fn trading_costs(State(s): State<AppState>) -> Response {
    match jiaoyi_store::trading::costs(&s.pool).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => unavailable(e),
    }
}

// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
/// `/trading/orders` query parameters, as given.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct OrdersParams {
    pub account: Option<String>,
    pub strategy: Option<String>,
    pub since: Option<String>,
    pub limit: Option<String>,
}

/// The order list's default and largest page.
pub const ORDERS_DEFAULT: i64 = 100;
pub const ORDERS_MAX: i64 = 500;

/// Check `/trading/orders`' parameters: an account id, `s1` or `s3`, an
/// RFC 3339 time and a limit of 1–500. Anything else is refused.
pub fn orders_query(p: &OrdersParams) -> Result<jiaoyi_store::trading::OrdersQuery, String> {
    let account_id = match p.account.as_deref() {
        None | Some("") => None,
        Some(a) => Some(
            a.parse::<i64>()
                .map_err(|_| format!("account must be an account id, not {a:?}"))?,
        ),
    };
    let strategy = match p.strategy.as_deref() {
        None | Some("") => None,
        Some(s @ ("s1" | "s3")) => Some(s.to_string()),
        Some(s) => return Err(format!("strategy must be s1 or s3, not {s:?}")),
    };
    let since = match p.since.as_deref() {
        None | Some("") => None,
        Some(t) => Some(
            chrono::DateTime::parse_from_rfc3339(t)
                .map_err(|_| format!("since must be an RFC 3339 time, not {t:?}"))?
                .with_timezone(&chrono::Utc),
        ),
    };
    let limit = match p.limit.as_deref() {
        None | Some("") => ORDERS_DEFAULT,
        Some(l) => match l.parse::<i64>() {
            Ok(n) if (1..=ORDERS_MAX).contains(&n) => n,
            _ => return Err(format!("limit must be 1 to {ORDERS_MAX}, not {l:?}")),
        },
    };
    Ok(jiaoyi_store::trading::OrdersQuery {
        account_id,
        strategy,
        since,
        limit,
    })
}

/// The body of `/trading/orders`: the orders, newest first, with what each
/// field means. Prices, volumes and milliseconds only, no money.
pub fn orders_body(rows: &[jiaoyi_store::trading::OrderView]) -> serde_json::Value {
    json!({
        "strategies": {
            "s1": crate::trading::STRATEGY,
            "s3": crate::strategy3::STRATEGY,
        },
        "units": {
            "volume": "cents of units, as the broker counts them",
            "prices": "price units of the symbol",
            "outcome": "the last execution message recorded for the order (ORDER_FILLED, ORDER_CANCELLED, RECONCILED_FILL, ...); filled for an exit the broker executed itself",
            "filled_volume": "the order's deals, each once, or the position reconcile adopted, whichever is larger",
            "fill_price": "the deals' volume-weighted price, or the adopted position's",
            "latency_ms": "from the intent's record, just before the frame, to the broker's fill answer",
            "slippage": "the fill against the quote at the decision (the ask for a buy, the bid for a sell), or a broker exit against the level it executed; positive is adverse",
            "decision": "the newest decision recorded with the order's entry key: its kind, reason, time and the signal's inputs",
        },
        "count": rows.len(),
        "rows": rows,
    })
}

/// `/trading/orders?account=&strategy=s1|s3&since=&limit=`: every order of
/// Strategy 1 and Strategy 3, newest first, with its outcome, fill, position,
/// costs and the decision it was placed under.
async fn trading_orders(State(s): State<AppState>, Query(p): Query<OrdersParams>) -> Response {
    let q = match orders_query(&p) {
        Ok(q) => q,
        Err(reason) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "status": "refused", "reason": reason })),
            )
                .into_response()
        }
    };
    match jiaoyi_store::trading::orders(&s.pool, &q).await {
        Ok(rows) => Json(orders_body(&rows)).into_response(),
        Err(e) => unavailable(e),
    }
}

async fn trials(State(s): State<AppState>) -> Response {
    match crate::edge::routes::trials(&s.pool).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => unavailable(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
    fn params(account: &str, strategy: &str, since: &str, limit: &str) -> OrdersParams {
        let o = |s: &str| (!s.is_empty()).then(|| s.to_string());
        OrdersParams {
            account: o(account),
            strategy: o(strategy),
            since: o(since),
            limit: o(limit),
        }
    }

    #[test]
    fn the_order_list_takes_an_account_s1_or_s3_a_time_and_a_limit_of_1_to_500() {
        let q = orders_query(&params("", "", "", "")).unwrap();
        assert_eq!(
            (q.account_id, q.strategy, q.since, q.limit),
            (None, None, None, 100)
        );
        let q = orders_query(&params("49236051", "s1", "2026-10-25T00:00:00Z", "500")).unwrap();
        assert_eq!(q.account_id, Some(49236051));
        assert_eq!(q.strategy.as_deref(), Some("s1"));
        assert_eq!(
            q.since.map(|t| t.to_rfc3339()).as_deref(),
            Some("2026-10-25T00:00:00+00:00")
        );
        assert_eq!(q.limit, 500);
        assert_eq!(orders_query(&params("", "s3", "", "1")).unwrap().limit, 1);
        for bad in [
            params("", "", "", "0"),
            params("", "", "", "501"),
            params("", "", "", "-5"),
            params("", "", "", "ten"),
            params("", "s2", "", ""),
            params("", "S1", "", ""),
            params("x", "", "", ""),
            params("", "", "yesterday", ""),
        ] {
            assert!(orders_query(&bad).is_err(), "{bad:?}");
        }
    }
}
