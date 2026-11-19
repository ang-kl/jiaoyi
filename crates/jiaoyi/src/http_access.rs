//! Optional private machine HTTP profile. No login, actor locks, database calls,
//! external requests or trading authority are introduced here.
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Json, Response};

use crate::owner_assessments::{private_response, OwnerAccess};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Profile {
    /// Compatibility stage only; this is not a secured cutover.
    Legacy,
    Private,
    /// Unknown/invalid configuration denies data but preserves liveness.
    Disabled,
}

pub(crate) const AUDIT_ROUTES: &[&str] = &[
    "/health",
    "/phase1/report",
    "/research/inventory",
    "/research/bars",
    "/research/jobs",
    "/research/trials",
    "/trading/costs",
    "/trading/orders",
    "/trading/executions",
    "/operations/reports",
    "/operations/stock-preparation",
    "/operations/stop-loss-reviews",
    "/operations/trailing-receipts",
    "/operations/setup-assessments",
    "/owner/setup-assessments",
    "/owner/live-positions",
    "/owner/quote-coverage",
];

pub(crate) const RESEARCH_ROUTES: &[&str] = &["/owner/backtests", "/owner/market-data"];

/// Route policy is checked before extractors, handler state and body decoding.
/// A saved audit credential cannot acquire research authority via method changes.
pub(crate) async fn guard(
    State(access): State<OwnerAccess>,
    request: Request,
    next: Next,
) -> Response {
    if access.profile() == Profile::Legacy {
        return next.run(request).await;
    }
    let head = request.method() == Method::HEAD;
    let read = request.method() == Method::GET || head;
    let path = request.uri().path();
    let mut response =
        if path == "/health" && read && !request.headers().contains_key("authorization") {
            // This branch is intentionally before all application handlers. In
            // particular, /health normally acquires actor maps and the shared pool.
            Json(serde_json::json!({"status":"ok"})).into_response()
        } else if AUDIT_ROUTES.contains(&path) {
            if !read {
                error(StatusCode::METHOD_NOT_ALLOWED)
            } else {
                match access.authorise(request.headers()) {
                    Ok(()) => next.run(request).await,
                    Err(status) => error(status),
                }
            }
        } else if RESEARCH_ROUTES.contains(&path) {
            if !read && request.method() != Method::POST {
                error(StatusCode::METHOD_NOT_ALLOWED)
            } else {
                match access.authorise_research(request.headers()) {
                    Ok(()) => next.run(request).await,
                    Err(status) => error(status),
                }
            }
        } else {
            // HTML/JS pages and unknown future routes are never implicitly allowed.
            error(StatusCode::NOT_FOUND)
        };
    if head {
        *response.body_mut() = Body::empty();
    }
    response.headers_mut().insert(
        "x-jiaoyi-http-profile",
        match access.profile() {
            Profile::Private => "private",
            _ => "disabled",
        }
        .parse()
        .unwrap(),
    );
    private_response(response)
}

fn error(status: StatusCode) -> Response {
    let code = match status {
        StatusCode::SERVICE_UNAVAILABLE => "http_access_not_configured",
        StatusCode::UNAUTHORIZED => "machine_credential_required",
        StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
        _ => "not_found",
    };
    (status, Json(serde_json::json!({"error":code}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, Request as HttpRequest};
    use axum::routing::any;
    use axum::Router;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    const AUDIT: &str = "audit-test-credential-000000000000000000000001";
    const RESEARCH: &str = "research-test-credential-000000000000000000002";

    fn access(profile: Option<&str>, audit: Option<&str>, research: Option<&str>) -> OwnerAccess {
        OwnerAccess::with_profile(profile, Some("old-owner-password"), audit, research)
    }

    fn fake_router(access: OwnerAccess) -> Router {
        let mut router = Router::new();
        for path in AUDIT_ROUTES.iter().chain(RESEARCH_ROUTES).chain([
            &"/",
            &"/status-page.js",
            &"/operations",
            &"/owner/assessments",
            &"/owner/assessments.mjs",
            &"/future-unreviewed-route",
        ]) {
            router = router.route(path, any(|| async { StatusCode::NO_CONTENT }));
        }
        router.layer(axum::middleware::from_fn_with_state(access, guard))
    }

    async fn call(router: Router, method: &str, path: &str, auth: Option<&str>) -> Response {
        let mut request = HttpRequest::builder().method(method).uri(path);
        if let Some(auth) = auth {
            request = request.header("authorization", auth);
        }
        router
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn private_profile_method_and_principal_matrix() {
        let router = fake_router(access(Some("private"), Some(AUDIT), Some(RESEARCH)));
        let audit = format!("Bearer {AUDIT}");
        let research = format!("Bearer {RESEARCH}");
        for path in AUDIT_ROUTES {
            for method in ["GET", "HEAD"] {
                for auth in [
                    None,
                    Some(audit.as_str()),
                    Some(research.as_str()),
                    Some("Basic b3duZXI6b2xkLW93bmVyLXBhc3N3b3Jk"),
                ] {
                    let response = call(router.clone(), method, path, auth).await;
                    let expected = if auth == Some(audit.as_str()) {
                        StatusCode::NO_CONTENT
                    } else if *path == "/health" && auth.is_none() {
                        StatusCode::OK
                    } else {
                        StatusCode::UNAUTHORIZED
                    };
                    assert_eq!(response.status(), expected, "{method} {path}");
                    assert!(response.headers().get("www-authenticate").is_none());
                    assert_eq!(response.headers()["x-jiaoyi-http-profile"], "private");
                    if method == "HEAD" {
                        assert!(response
                            .into_body()
                            .collect()
                            .await
                            .unwrap()
                            .to_bytes()
                            .is_empty());
                    }
                }
            }
            for method in ["POST", "PUT", "DELETE", "OPTIONS"] {
                assert_eq!(
                    call(router.clone(), method, path, Some(&audit))
                        .await
                        .status(),
                    StatusCode::METHOD_NOT_ALLOWED
                );
            }
        }
        for path in RESEARCH_ROUTES {
            for method in ["GET", "HEAD", "POST"] {
                for auth in [
                    None,
                    Some(audit.as_str()),
                    Some(research.as_str()),
                    Some("Bearer old-owner-password"),
                ] {
                    let expected = if auth == Some(research.as_str()) {
                        StatusCode::NO_CONTENT
                    } else {
                        StatusCode::UNAUTHORIZED
                    };
                    assert_eq!(
                        call(router.clone(), method, path, auth).await.status(),
                        expected,
                        "{method} {path}"
                    );
                }
            }
            for method in ["PUT", "DELETE", "OPTIONS"] {
                assert_eq!(
                    call(router.clone(), method, path, Some(&research))
                        .await
                        .status(),
                    StatusCode::METHOD_NOT_ALLOWED
                );
            }
        }
        for path in [
            "/",
            "/?owner=1",
            "/status-page.js",
            "/operations",
            "/owner/assessments",
            "/owner/assessments.mjs",
            "/future-unreviewed-route",
        ] {
            for auth in [None, Some(audit.as_str()), Some(research.as_str())] {
                assert_eq!(
                    call(router.clone(), "GET", path, auth).await.status(),
                    StatusCode::NOT_FOUND
                );
            }
        }
    }

    #[tokio::test]
    async fn invalid_private_configuration_preserves_only_minimal_liveness() {
        for access in [
            access(Some("private"), None, Some(RESEARCH)),
            access(Some("private"), Some(AUDIT), None),
            access(Some("private"), Some(AUDIT), Some(AUDIT)),
            access(Some("private"), Some("short"), Some(RESEARCH)),
            access(
                Some("private"),
                Some("invalid credential with spaces and enough length"),
                Some(RESEARCH),
            ),
            access(Some("typo"), Some(AUDIT), Some(RESEARCH)),
            access(Some(""), Some(AUDIT), Some(RESEARCH)),
        ] {
            let router = fake_router(access);
            let response = call(router.clone(), "GET", "/health", None).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .as_ref(),
                b"{\"status\":\"ok\"}"
            );
            for path in ["/health", "/owner/setup-assessments", "/owner/backtests"] {
                assert_eq!(
                    call(
                        router.clone(),
                        "GET",
                        path,
                        Some(&format!("Bearer {AUDIT}"))
                    )
                    .await
                    .status(),
                    StatusCode::SERVICE_UNAVAILABLE
                );
            }
            assert_eq!(
                call(router, "GET", "/", None).await.status(),
                StatusCode::NOT_FOUND
            );
        }
    }

    #[test]
    fn private_profile_rejects_duplicate_headers_and_never_uses_basic_or_owner_password() {
        let access = access(Some("private"), Some(AUDIT), Some(RESEARCH));
        let mut headers = HeaderMap::new();
        headers.append("authorization", format!("Bearer {AUDIT}").parse().unwrap());
        headers.append("authorization", format!("Bearer {AUDIT}").parse().unwrap());
        assert_eq!(access.authorise(&headers), Err(StatusCode::UNAUTHORIZED));
        for credential in [
            "Bearer old-owner-password",
            "Basic b3duZXI6b2xkLW93bmVyLXBhc3N3b3Jk",
        ] {
            headers.clear();
            headers.insert("authorization", credential.parse().unwrap());
            assert_eq!(access.authorise(&headers), Err(StatusCode::UNAUTHORIZED));
            assert_eq!(
                access.authorise_research(&headers),
                Err(StatusCode::UNAUTHORIZED)
            );
        }
    }

    #[tokio::test]
    async fn legacy_profile_is_an_explicit_compatibility_stage_not_a_security_claim() {
        for profile in [None, Some("legacy")] {
            let access = access(profile, Some(AUDIT), Some(RESEARCH));
            let mut headers = HeaderMap::new();
            headers.insert(
                "authorization",
                "Bearer old-owner-password".parse().unwrap(),
            );
            assert_eq!(access.authorise(&headers), Ok(()));
            assert_eq!(access.authorise_research(&headers), Ok(()));
            headers.insert("authorization", format!("Bearer {AUDIT}").parse().unwrap());
            assert_eq!(access.authorise(&headers), Err(StatusCode::UNAUTHORIZED));
            let response = call(fake_router(access), "GET", "/", None).await;
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
            assert!(response.headers().get("x-jiaoyi-http-profile").is_none());
        }
    }
}
