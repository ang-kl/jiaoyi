//! A human-readable projection of durable supervisor reports; no action route.

use chrono::{DateTime, FixedOffset, Utc};
use serde_json::Value;

use crate::status_page::esc;

fn number(value: &Value, scale: f64) -> String {
    value
        .as_f64()
        .map(|n| format!("{:.2}", n * scale))
        .unwrap_or_else(|| "unknown".into())
}

fn time(value: &Value) -> String {
    value
        .as_str()
        .and_then(|s| s.parse::<DateTime<FixedOffset>>().ok())
        .map(|t| {
            t.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
                .format("%d-%m-%Y %H:%M:%S SGT")
                .to_string()
        })
        .unwrap_or_else(|| "not recorded".into())
}

pub fn page(reports: &Value) -> String {
    let mut rows = String::new();
    for row in reports["reports"].as_array().into_iter().flatten() {
        let assessment = &row["report"]["assessment"];
        let window = &assessment["window"];
        let missing = assessment["missing_reasons"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let state = assessment["state"].as_str().unwrap_or("not assessed");
        let (win_gap, payoff_gap) = crate::telegram_report::benchmark_gaps(row);
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}/{}</td><td>{}%</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&row["account_id"].to_string()), esc(row["strategy"].as_str().unwrap_or("unknown")),
            esc(row["monitor_state"].as_str().unwrap_or("unknown")), esc(state),
            esc(&window["wins"].as_u64().map(|n| n.to_string()).unwrap_or_else(|| "?".into())),
            esc(&window["count"].as_u64().map(|n| n.to_string()).unwrap_or_else(|| "?".into())),
            number(&window["win_rate"], 100.0), esc(&win_gap),
            number(&window["realized_payoff"], 1.0), esc(&payoff_gap),
            number(&window["net_r"], 1.0), esc(&time(&assessment["assessed_at"])),
            esc(&time(&row["checked_at"])),
            esc(&time(&row["report"]["last_success_at"])), esc(&missing)
        ));
    }
    let mut events = String::new();
    for event in reports["recent_events"].as_array().into_iter().flatten() {
        let report = &event["report"];
        events.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            esc(&time(&report["checked_at"])),
            esc(&event["account_id"].to_string()),
            esc(event["strategy"].as_str().unwrap_or("unknown")),
            esc(report["assessment"]["state"].as_str().unwrap_or("unknown")),
            esc(&report["milestones_newly_reached"].to_string())
        ));
    }
    let history = format!("<h2>§3 — Recent milestones and assessment changes</h2><p>Latest 100 events, excluding routine heartbeats. The full record remains in the decision history. Position-count milestones are not economic acceptance.</p><div class=\"table\"><table><thead><tr><th>Detected</th><th>Account</th><th>Strategy</th><th>Assessment</th><th>New closed-position milestones</th></tr></thead><tbody>{events}</tbody></table></div>");
    let page = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta http-equiv=\"refresh\" content=\"60\"><title>jiaoyi continuous assessment</title><style>body{{font:16px system-ui;margin:2rem;color:#17202a}}table{{border-collapse:collapse;width:100%;font-size:.9rem}}td,th{{padding:.65rem;text-align:left;border-bottom:1px solid #ccd1d1}}.table{{overflow:auto}}a{{color:#125ba0}}p{{max-width:70rem}}</style><body><h1>Continuous assessment</h1><p>№OPS-CONTINUOUS · live document, not a chat serial. Generated {}. Snapshot assembled {}.</p><h2>§1 — Goal and evidence scope</h2><p> Assessments run in the Railway service while it is running, including unattended SGT hours. The chat agents are separate from this service.</p><p>Reference: at least 15 net wins in the latest 20 fully closed positions (75%, meeting the 73% threshold). Realized net-R payoff is compared with 1.38; registered stop and target geometry stays unchanged. One passing window does not establish consistency. Unknown or stale evidence cannot establish success.</p><p>Comparable cohort deltas: unknown (no reconciled prior comparison in this view). Engineering-attributable financial movement: unknown; these assessments do not establish a causal effect of code changes.</p><h2>§2 — Current assessment and supported gaps</h2><div class=\"table\"><table><thead><tr><th>Account</th><th>Strategy</th><th>Monitor</th><th>Assessment</th><th>Wins/closes</th><th>Win rate</th><th>Win gap to 73%</th><th>Realized payoff</th><th>Payoff gap to 1.38</th><th>Window net R</th><th>Assessment source cutoff</th><th>Last persisted check</th><th>Last completed assessment</th><th>Missing evidence</th></tr></thead><tbody>{}</tbody></table></div><p>Checks run every 60 seconds; unchanged reports receive a durable heartbeat every five minutes. The last completed assessment time stays unchanged when computation fails; a completed computation can still report unknown broker evidence. Monitor liveness and broker-evidence freshness are separate. Undefined values show as unknown.</p><p>Reports and milestones are saved in the existing decision history. Telegram HTML summaries use their separate schedules; this page does not imply an immediate alert was delivered. The supervisor cannot place orders, pause trading, change stops or sizing, or tune strategies. Owner decisions remain queued for the owner's next session; existing exits continue.</p><p><a href=\"/operations/reports\">Report data</a> · <a href=\"/health\">Service health</a> · <a href=\"/\">Trading status</a> · <a href=\"https://github.com/ang-kl/jiaoyi/blob/main/docs/continuous-operations.md\">Milestones and owner decisions</a></p></body></html>",
        esc(&time(&Value::String(Utc::now().to_rfc3339()))),
        esc(&time(&reports["as_of"])), rows
    );
    page.replace("</body>", &format!("{history}</body>"))
}
