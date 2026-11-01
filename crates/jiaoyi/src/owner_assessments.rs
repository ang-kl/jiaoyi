//! Authenticated owner projection. The evaluator never grants order authority.
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct OwnerAccess {
    digest: Option<[u8; 32]>,
}

impl OwnerAccess {
    pub fn from_env() -> Self {
        Self::new(std::env::var("JIAOYI_UI_PASSWORD").ok().as_deref())
    }

    pub fn new(password: Option<&str>) -> Self {
        Self {
            digest: password
                .filter(|s| !s.trim().is_empty())
                .map(|s| Sha256::digest(s.as_bytes()).into()),
        }
    }

    pub fn authorise(&self, headers: &HeaderMap) -> Result<(), StatusCode> {
        let expected = self.digest.ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .filter(|h| h.len() <= 4096)
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let (scheme, credentials) = header.split_once(' ').ok_or(StatusCode::UNAUTHORIZED)?;
        let password = if scheme.eq_ignore_ascii_case("Bearer") {
            credentials.as_bytes().to_vec()
        } else if scheme.eq_ignore_ascii_case("Basic") {
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(credentials)
                .map_err(|_| StatusCode::UNAUTHORIZED)?;
            decoded
                .strip_prefix(b"owner:")
                .ok_or(StatusCode::UNAUTHORIZED)?
                .to_vec()
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        };
        let actual: [u8; 32] = Sha256::digest(&password).into();
        // Compare fixed-length digests without stopping at the first mismatch.
        let difference = expected
            .iter()
            .zip(actual)
            .fold(0_u8, |difference, (a, b)| difference | (a ^ b));
        if difference == 0 {
            Ok(())
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub fn private_response(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-store".parse().unwrap());
    headers.insert("vary", "Authorization".parse().unwrap());
    headers.insert("referrer-policy", "no-referrer".parse().unwrap());
    headers.insert("x-robots-tag", "noindex, nofollow".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        "content-security-policy",
        "default-src 'none'; script-src 'self'; connect-src 'self'; style-src 'unsafe-inline'; base-uri 'none'; frame-ancestors 'none'"
            .parse()
            .unwrap(),
    );
    response
}

pub fn refusal(status: StatusCode) -> Response {
    let message = if status == StatusCode::SERVICE_UNAVAILABLE {
        "Owner access is not configured."
    } else {
        "Owner authentication required."
    };
    let mut response = (status, message).into_response();
    if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            "www-authenticate",
            "Basic realm=\"Jiaoyi owner\", charset=\"UTF-8\""
                .parse()
                .unwrap(),
        );
    }
    private_response(response)
}

fn text(value: &Value) -> String {
    use crate::status_page::esc;
    match value {
        Value::Null => "unknown".into(),
        Value::String(s) => esc(s),
        _ => esc(&value.to_string()),
    }
}

fn checks(value: &Value) -> String {
    let mut out = String::new();
    for check in value.as_array().into_iter().flatten() {
        out.push_str(&format!(
            "<div><b>{}: {}</b> — {}</div>",
            text(&check["name"]),
            text(&check["state"]),
            text(&check["reason"])
        ));
    }
    if out.is_empty() {
        "Awaiting assessment".into()
    } else {
        out
    }
}

fn incidents(value: &Value) -> String {
    let Some(rows) = value.as_array() else {
        return "Observer incident state unavailable".into();
    };
    if rows.is_empty() {
        return "No observer incident reported".into();
    }
    rows.iter()
        .take(3)
        .map(|item| {
            format!(
                "<div><b>{}: {}</b> — {} <small>(opened {})</small></div>",
                text(&item["kind"]),
                text(&item["state"]),
                text(&item["reason"]),
                text(&item["opened_at"])
            )
        })
        .collect()
}

pub const SCRIPT: &str = include_str!("owner_assessments.mjs");

fn position_row(account: &Value, object: &Value) -> String {
    let units = object["volume"]
        .as_f64()
        .map(|raw| format!("{:.2}", raw / 100.0))
        .unwrap_or_else(|| "unknown".into());
    let attributes = if object["kind"] == "position" {
        format!(
            " data-position-key=\"{}:{}\" data-live-eligible=\"{}\" data-market-open-until=\"{}\"",
            text(&account["account_id"]),
            text(&object["id"]),
            object["live_update"]["eligible"].as_bool().unwrap_or(false),
            text(&object["live_update"]["market_open_until"])
        )
    } else {
        String::new()
    };
    format!(
        "<tr{attributes}><td>{}<br><small>{} · {}<br>As of {} · <span data-update-status>Snapshot</span></small></td><td>{}<br>{} units</td><td>{}</td><td>{}<br>{}</td><td>{}</td></tr>",
        text(&object["symbol"]), text(&object["kind"]), text(&object["id"]), text(&account["assessed_at"]),
        text(&object["side"]), units, text(&object["entry"]), text(&object["stop_loss"]),
        text(&object["take_profit"]), checks(&object["checks"]))
}

/// Cheap authenticated browser projection: no pending-entry payloads, account
/// totals, history, database access or broker requests on the refresh path.
pub fn live_positions(report: &Value) -> Value {
    let now = chrono::Utc::now();
    let accounts: Vec<Value> = report["accounts"].as_array().into_iter().flatten().map(|account| {
        let fresh = account["assessed_at"].as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .is_some_and(|at| { let age = now.signed_duration_since(at); age.num_milliseconds() >= 0 && age.num_seconds() < 15 });
        let complete = fresh && account["evidence_complete"] == true;
        let positions: Vec<&Value> = account["objects"].as_array().into_iter().flatten()
            .filter(|o| o["kind"] == "position").collect();
        let ids: Vec<Value> = positions.iter().map(|o| o["id"].clone()).collect();
        let updates: Vec<Value> = positions.into_iter().map(|object| {
            let mut eligibility = object["live_update"].clone();
            let valid = complete && eligibility["eligible"] == true
                && eligibility["valid_until"].as_str().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .is_some_and(|until| until > now);
            if !valid && eligibility.is_object() { eligibility["eligible"] = Value::Bool(false); }
            let mut row = serde_json::json!({"id":object["id"],"live_update":eligibility});
            if valid { row["html"] = Value::String(position_row(account, object)); }
            row
        }).collect();
        serde_json::json!({"account_id":account["account_id"], "assessed_at":account["assessed_at"],
            "evidence_complete":complete, "position_ids":ids, "positions":updates})
    }).collect();
    serde_json::json!({"accounts":accounts})
}

pub fn page(report: &Value) -> String {
    let generated = chrono::Utc::now()
        .with_timezone(&chrono_tz::Asia::Singapore)
        .format("%d-%m-%Y %H:%M:%S SGT")
        .to_string();
    let mut sections = String::new();
    for account in report["accounts"].as_array().into_iter().flatten() {
        let mut rows = String::new();
        for object in account["objects"].as_array().into_iter().flatten() {
            rows.push_str(&position_row(account, object));
        }
        if rows.is_empty() {
            rows.push_str("<tr><td colspan=5>No objects in this snapshot. Use Recheck for a fresh inventory.</td></tr>");
        }
        sections.push_str(&format!(
            "<section><h2>Account {}</h2><p>Summary snapshot: {} · Broker book: {} · Evidence complete: {} · Revision: {}</p><p>Positions: {} · Pending entries: {} · Protective orders: {} · Breaches: {} · Blocked new-entry checks: {} · Unknown checks: {}</p><div class=checks>{}</div><div class=checks>{}</div><div class=scroll><table><thead><tr><th>Instrument / object</th><th>Side / quantity</th><th>Entry / trigger</th><th>SL / TP</th><th>Checks and reasons</th></tr></thead><tbody id=\"positions-{}\">{}</tbody></table></div></section>",
            text(&account["account_id"]), text(&account["assessed_at"]), text(&account["book_at"]),
            text(&account["evidence_complete"]), text(&account["revision"]), text(&account["position_count"]),
            text(&account["pending_entry_count"]), text(&account["protective_order_count"]),
            text(&account["breach_count"]), text(&account["blocked_entry_check_count"]), text(&account["unknown_count"]),
            checks(&account["account_checks"]), incidents(&account["incidents"]), text(&account["account_id"]), rows));
    }
    format!(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content=\"width=device-width, initial-scale=1\"><title>Jiaoyi — trade assessments</title><style>body{{font:15px system-ui,sans-serif;max-width:1400px;margin:2rem auto;padding:0 1rem;color:#17252f;background:#f7fafb}}section{{background:white;border:1px solid #ccd7dc;border-radius:8px;padding:1rem;margin:1rem 0}}table{{width:100%;border-collapse:collapse}}th,td{{text-align:left;padding:.65rem;border-bottom:1px solid #dde5e8;vertical-align:top}}small{{color:#53636d}}.scroll{{overflow:auto}}.checks{{padding:.5rem 0}}a{{color:#075985}}</style><h1>All-trades assessment</h1><p>№SETUP-ASSESSMENT · {generated} · plan №18,092</p><h2>§1 — Goal and evidence</h2><p>Goal: risk:reward 1:1.38 and ≥73% net wins over the latest 20 qualified closed positions, separately per strategy/account. <b>Financial progress is not measured by these setup checks.</b></p><p id=live-update-status role=status>Updates paused until this page is visible and focused.</p><p>Only existing open-position rows in known open markets receive live updates, every five seconds while this page is active. Pending entries, closed-market rows and account summaries retain their displayed snapshot times. <a href=\"/owner/assessments\">Recheck all snapshots</a> after a market opens or when evidence is unavailable. Background trade assessment and protection continue independently of this page. A passed check does not predict profit; this page cannot place or change orders.</p><noscript><p>JavaScript is disabled; use Recheck for current snapshots.</p></noscript><h2>§2 — Current account assessments</h2>{sections}<p><a href=\"/owner/setup-assessments\">Full assessment data</a> · <a href=\"/operations\">Performance and goal evidence</a></p><script type=module src=\"/owner/assessments.mjs\"></script></html>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", value.parse().unwrap());
        headers
    }

    #[test]
    fn missing_configuration_and_invalid_credentials_never_authorise() {
        assert_eq!(
            OwnerAccess::new(None).authorise(&headers("Bearer secret")),
            Err(StatusCode::SERVICE_UNAVAILABLE)
        );
        assert_eq!(
            OwnerAccess::new(Some(" ")).authorise(&HeaderMap::new()),
            Err(StatusCode::SERVICE_UNAVAILABLE)
        );
        let access = OwnerAccess::new(Some("secret"));
        for header in [
            "Bearer wrong",
            "Bearer secret-more",
            "Basic broken",
            "Basic YWRtaW46c2VjcmV0",
        ] {
            assert_eq!(
                access.authorise(&headers(header)),
                Err(StatusCode::UNAUTHORIZED)
            );
        }
        assert_eq!(
            access.authorise(&HeaderMap::new()),
            Err(StatusCode::UNAUTHORIZED)
        );
        assert_eq!(access.authorise(&headers("Bearer secret")), Ok(()));
        assert_eq!(access.authorise(&headers("Basic b3duZXI6c2VjcmV0")), Ok(()));
    }

    #[test]
    fn private_responses_cannot_be_shared_cached_and_errors_do_not_echo_secrets() {
        let response = refusal(StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(response.headers()["vary"], "Authorization");
        assert!(response.headers().contains_key("www-authenticate"));
    }

    #[test]
    fn live_projection_updates_only_fresh_positions_and_never_refreshes_static_payloads() {
        let now = chrono::Utc::now();
        let until = (now + chrono::Duration::seconds(5)).to_rfc3339();
        let open = json!({"eligible":true,"valid_until":until,"market_open_until":until,"reason":"observed_open"});
        let report = json!({"accounts":[{"account_id":7,"assessed_at":now,"evidence_complete":true,
            "private_account_totals":"not in row refresh", "objects":[
            {"kind":"position","id":1,"symbol":"<img src=x onerror=alert(1)>","live_update":open},
            {"kind":"position","id":2,"symbol":"closed","live_update":{"eligible":false,"reason":"market_closed"}},
            {"kind":"pending_entry","id":3,"symbol":"pending","live_update":open}]}]});
        let result = live_positions(&report);
        let a = &result["accounts"][0];
        assert_eq!(a["position_ids"], json!([1, 2]));
        assert_eq!(a["positions"].as_array().unwrap().len(), 2);
        assert!(a["positions"][0]["html"]
            .as_str()
            .unwrap()
            .contains("&lt;img"));
        assert!(a["positions"][1].get("html").is_none());
        assert!(!result.to_string().contains("not in row refresh"));
        assert!(!result.to_string().contains("pending"));
        let mut stale = report;
        stale["accounts"][0]["assessed_at"] = json!(now - chrono::Duration::seconds(16));
        let result = live_positions(&stale);
        assert_eq!(result["accounts"][0]["evidence_complete"], false);
        assert!(result["accounts"][0]["positions"][0].get("html").is_none());
    }

    #[test]
    fn owner_html_escapes_external_values_and_does_not_hide_unknowns() {
        let report = json!({"accounts":[{"account_id":7,"objects":[{"symbol":"<script>bad</script>","kind":"position","id":3,"volume":11,"checks":[{"name":"direction","state":"unknown","reason":"manual <unassigned>"}]}]}]});
        let html = page(&report);
        assert!(!html.contains("<script>bad</script>"));
        assert!(html.contains("&lt;script&gt;bad&lt;/script&gt;"));
        assert!(html.contains("direction: unknown"));
        assert!(html.contains("0.11 units"));
        assert!(html.contains("Financial progress is not measured"));
        assert!(!html.contains("http-equiv"));
        assert!(html.contains("type=module"));
        assert!(html.contains("Recheck all snapshots"));
    }
}
