//! `/`: a read-only status page, plain HTML rendered by the binary.
//!
//! It is drawn only from what the public JSON routes already return:
//! `/health` (service, phase, settings version, token grants, last loads,
//! tick recorders), `/research/jobs` and `/research/trials` (statuses,
//! verdicts, counts and R; no money). It writes nothing. It does not show
//! token fingerprints or hosts, and every text taken from the database is
//! HTML-escaped.

use chrono::{DateTime, Utc};
use serde_json::Value;

/// The JSON routes, linked from the page.
pub const ROUTES: [(&str, &str); 6] = [
    (
        "/health",
        "service health, token grants, last loads, tick recorders",
    ),
    (
        "/phase1/report",
        "the phase-1 data report per account and symbol",
    ),
    (
        "/research/inventory",
        "the stored data inventory for the edge test",
    ),
    (
        "/research/jobs",
        "the pre-registered research jobs and their attempts",
    ),
    (
        "/research/trials",
        "trial verdicts, counts and R (no money)",
    ),
    (
        "/research/bars?account=…&symbol=…&from=…&to=…",
        "development-period daily bars (parameters required)",
    ),
];

/// HTML-escape a text.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// A JSON value as escaped text: strings without quotes, null as a dash.
fn txt(v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::String(s) => esc(s),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => esc(&other.to_string()),
    }
}

/// A number with `digits` decimals, or a dash.
fn num(v: &Value, digits: usize) -> String {
    v.as_f64()
        .map(|x| format!("{x:.digits$}"))
        .unwrap_or_else(|| "—".into())
}

/// At most `n` characters, with an ellipsis when cut.
fn short(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}

fn pill(state: &str) -> String {
    let kind = match state {
        "ok" | "listed" | "done" | "measured" | "on" => "good",
        "partial" | "pending" | "running" | "unread" => "warn",
        "failed" | "refused" | "not_listed" | "environment_mismatch" => "bad",
        _ => "neutral",
    };
    format!(r#"<span class="pill {kind}">{}</span>"#, esc(state))
}

fn rows(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

/// The page, from the `/health`, `/research/jobs` and `/research/trials`
/// values.
pub fn page(health: &Value, jobs: &Value, trials: &Value, now: DateTime<Utc>) -> String {
    let mut h = String::new();
    h.push_str(HEAD);
    h.push_str(r#"<main><header><h1>jiaoyi</h1><p class="sub">Service status · read-only</p>"#);
    h.push_str(r#"<dl class="facts">"#);
    for (k, v) in [
        ("Service", txt(&health["service"])),
        (
            "Status",
            pill(health["status"].as_str().unwrap_or("unknown")),
        ),
        ("Phase", txt(&health["phase"])),
        ("Settings version", txt(&health["settings_version"])),
        (
            "Load in progress",
            if health["load_in_progress"].as_bool() == Some(true) {
                "yes"
            } else {
                "no"
            }
            .into(),
        ),
        (
            "Account refusals recorded",
            txt(&health["account_refusals_recorded"]),
        ),
    ] {
        h.push_str(&format!("<div><dt>{k}</dt><dd>{v}</dd></div>"));
    }
    h.push_str("</dl></header>");

    // Accounts: grant state and last load.
    h.push_str(r#"<section><h2>Accounts</h2><div class="scroll"><table><thead><tr><th>Account</th><th>Environment</th><th>Grant</th><th>Last load</th><th>Status</th><th>Finished</th><th>Errors</th></tr></thead><tbody>"#);
    let grants = rows(&health["token_grants"]);
    let loads = rows(&health["last_loads"]);
    let mut ids: Vec<i64> = grants
        .iter()
        .chain(loads)
        .filter_map(|r| r["account_id"].as_i64())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        h.push_str(r#"<tr><td colspan="7" class="empty">No account recorded yet.</td></tr>"#);
    }
    for id in ids {
        let g = grants.iter().find(|r| r["account_id"].as_i64() == Some(id));
        let l = loads.iter().find(|r| r["account_id"].as_i64() == Some(id));
        let grant = match g {
            Some(g) => {
                let reason = g["reason"]
                    .as_str()
                    .map(|r| format!(r#"<div class="note">{}</div>"#, esc(&short(r, 160))));
                format!(
                    "{}{}",
                    pill(g["state"].as_str().unwrap_or("unread")),
                    reason.unwrap_or_default()
                )
            }
            None => "—".into(),
        };
        let env = g
            .map(|g| txt(&g["environment"]))
            .unwrap_or_else(|| "—".into());
        let (run, status, finished, errors) = match l {
            Some(l) => {
                let errs = rows(&l["errors"]);
                let first = errs
                    .first()
                    .and_then(|e| e.as_str())
                    .map(|e| format!(r#"<div class="note">{}</div>"#, esc(&short(e, 160))));
                (
                    txt(&l["run_id"]),
                    pill(l["status"].as_str().unwrap_or("unknown")),
                    txt(&l["finished_at"]),
                    format!("{}{}", errs.len(), first.unwrap_or_default()),
                )
            }
            None => ("—".into(), "—".into(), "—".into(), "—".into()),
        };
        h.push_str(&format!(
            "<tr><td class=\"mono\">{id}</td><td>{env}</td><td>{grant}</td><td class=\"mono\">{run}</td><td>{status}</td><td class=\"mono\">{finished}</td><td>{errors}</td></tr>"
        ));
    }
    h.push_str("</tbody></table></div></section>");

    // Tick recorders.
    h.push_str(r#"<section><h2>Tick recorders</h2><div class="scroll"><table><thead><tr><th>Account</th><th class="num">Written (this run)</th><th class="num">Last hour (stored)</th><th>Last tick received</th><th class="num">Symbols</th><th class="num">Sessions</th><th>Last flush</th><th>Last error</th></tr></thead><tbody>"#);
    let recorders = health["tick_recorders"].as_object();
    let ticks = rows(&health["ticks"]);
    let mut rec_ids: Vec<i64> = recorders
        .map(|m| m.keys().filter_map(|k| k.parse().ok()).collect())
        .unwrap_or_default();
    rec_ids.extend(ticks.iter().filter_map(|t| t["account_id"].as_i64()));
    rec_ids.sort_unstable();
    rec_ids.dedup();
    if rec_ids.is_empty() {
        h.push_str(r#"<tr><td colspan="8" class="empty">No recorder running yet.</td></tr>"#);
    }
    for id in rec_ids {
        let r = recorders.and_then(|m| m.get(&id.to_string()));
        let t = ticks.iter().find(|t| t["account_id"].as_i64() == Some(id));
        let get = |k: &str| r.map(|r| txt(&r[k])).unwrap_or_else(|| "—".into());
        let symbols = r
            .and_then(|r| r["subscribed_symbols"].as_array())
            .map(|a| a.len().to_string())
            .unwrap_or_else(|| "—".into());
        let error = r
            .and_then(|r| r["last_error"].as_str())
            .map(|e| esc(&short(e, 160)))
            .unwrap_or_else(|| "—".into());
        h.push_str(&format!(
            "<tr><td class=\"mono\">{id}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"mono\">{}</td><td class=\"num\">{symbols}</td><td class=\"num\">{}</td><td class=\"mono\">{}</td><td>{error}</td></tr>",
            get("ticks_written"),
            t.map(|t| txt(&t["last_hour"])).unwrap_or_else(|| "—".into()),
            t.map(|t| txt(&t["last_received_at"])).unwrap_or_else(|| "—".into()),
            get("sessions_started"),
            get("last_flush_at"),
        ));
    }
    h.push_str("</tbody></table></div></section>");

    // Trading: each account's switch, edge gate, last decision and the
    // positions it found that it did not open (no money).
    h.push_str(r#"<section><h2>Trading</h2><div class="scroll"><table><thead><tr><th>Account</th><th>Switch</th><th>Edge gate</th><th>Minimum volume</th><th>Last decision</th><th class="num">External positions</th></tr></thead><tbody>"#);
    let trading = rows(&health["trading"]);
    if trading.is_empty() {
        h.push_str(r#"<tr><td colspan="6" class="empty">No account recorded yet.</td></tr>"#);
    }
    for t in trading {
        let switch = match t["enabled"].as_bool() {
            Some(true) => pill("on"),
            Some(false) => pill("off"),
            None => "—".into(),
        };
        let decision = t["last_decision"]
            .as_object()
            .map(|d| {
                format!(
                    r#"{}<div class="note">{} {}</div>"#,
                    esc(d.get("kind").and_then(|v| v.as_str()).unwrap_or("")),
                    esc(&short(
                        d.get("reason").and_then(|v| v.as_str()).unwrap_or(""),
                        160
                    )),
                    txt(d.get("at").unwrap_or(&Value::Null)),
                )
            })
            .unwrap_or_else(|| "—".into());
        let external = t["last_reconcile"]["external_positions"]
            .as_array()
            .map(|a| a.len().to_string())
            .unwrap_or_else(|| "—".into());
        h.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{switch}</td><td class=\"note\">{}</td><td>{}</td><td>{decision}</td><td class=\"num\">{external}</td></tr>",
            txt(&t["account_id"]),
            esc(&short(t["edge_gate"].as_str().unwrap_or("—"), 160)),
            match t["allow_min_volume"].as_bool() {
                Some(true) => "allowed",
                Some(false) => "skip",
                None => "—",
            },
        ));
    }
    h.push_str("</tbody></table></div></section>");

    // Research jobs, with their trial's verdict.
    h.push_str(r#"<section><h2>Research jobs</h2>"#);
    h.push_str(&format!(
        r#"<p class="sub">Pre-registration <span class="mono">{}</span>, hash <span class="mono">{}</span>.</p>"#,
        txt(&jobs["prereg_file"]),
        esc(&short(jobs["prereg_hash_running"].as_str().unwrap_or("—"), 12)),
    ));
    // Claude · № 16,901 25-Oct (ordered № 16,900; R1b's engine guard on the page)
    h.push_str(&format!(
        r#"<p class="sub">Engine version {} running; the held-out job waits for engine {} or later.</p>"#,
        txt(&jobs["engine_version_running"]),
        txt(&jobs["held_out_min_engine"]),
    ));
    let sampling = &jobs["spread_sampling"];
    if sampling.is_object() {
        h.push_str(&format!(
            r#"<p class="sub">Spread sampling: {} weeks measured, {} without ticks, {} failed; {} ticks downloaded ({} per second).</p>"#,
            txt(&sampling["weeks_measured"]),
            txt(&sampling["weeks_without_ticks"]),
            txt(&sampling["weeks_failed"]),
            txt(&sampling["ticks_downloaded"]),
            num(&sampling["ticks_per_second"], 1),
        ));
    }
    h.push_str(r#"<div class="scroll"><table><thead><tr><th class="num">Job</th><th>Stage</th><th>Setting</th><th>Status</th><th>Reason</th><th class="num">Attempts</th><th>Verdict</th><th class="num">Trades</th><th class="num">Mean R</th><th class="num">PF</th></tr></thead><tbody>"#);
    let trial_rows = rows(&trials["trials"]);
    let job_rows = rows(&jobs["jobs"]);
    if job_rows.is_empty() {
        h.push_str(r#"<tr><td colspan="10" class="empty">No research job registered.</td></tr>"#);
    }
    for j in job_rows {
        let t = trial_rows
            .iter()
            .find(|t| t["job_id"].as_i64().is_some() && t["job_id"] == j["id"]);
        let setting = format!(
            "breakout {} · stop {}×ATR{}",
            txt(&j["setting"]["breakout"]),
            txt(&j["setting"]["stop_atr"]),
            if j["setting"]["default"].as_bool() == Some(true) {
                " · default"
            } else {
                ""
            }
        );
        let reason = j["reason"]
            .as_str()
            .map(|r| esc(&short(r, 200)))
            .unwrap_or_else(|| "—".into());
        let r = t.map(|t| &t["results"]["r"]);
        h.push_str(&format!(
            "<tr><td class=\"num mono\">{}</td><td>{}</td><td>{setting}</td><td>{}</td><td class=\"note\">{reason}</td><td class=\"num\">{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
            txt(&j["id"]),
            txt(&j["stage"]),
            pill(j["status"].as_str().unwrap_or("unknown")),
            rows(&j["attempts"]).len(),
            t.map(|t| txt(&t["verdict"])).unwrap_or_else(|| "—".into()),
            r.map(|r| txt(&r["n"])).unwrap_or_else(|| "—".into()),
            r.map(|r| num(&r["mean_r"], 3)).unwrap_or_else(|| "—".into()),
            r.map(|r| {
                if r["pf_infinite"].as_bool() == Some(true) {
                    "∞".to_string()
                } else {
                    num(&r["profit_factor"], 2)
                }
            })
            .unwrap_or_else(|| "—".into()),
        ));
    }
    h.push_str("</tbody></table></div></section>");

    // Links to the JSON routes.
    h.push_str(r#"<section><h2>JSON routes</h2><ul class="routes">"#);
    for (path, what) in ROUTES {
        let href = path.split('?').next().unwrap_or(path);
        h.push_str(&format!(
            r#"<li><a class="mono" href="{}">{}</a> <span class="note">{}</span></li>"#,
            esc(href),
            esc(path),
            esc(what)
        ));
    }
    h.push_str("</ul></section>");
    h.push_str(&format!(
        r#"<footer>Generated {} UTC. Read-only: this page and the routes it links write nothing.</footer></main></body></html>"#,
        now.format("%Y-%m-%d %H:%M:%S")
    ));
    h
}

const HEAD: &str = r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>jiaoyi status</title>
<style>
:root{--bg:#f7f7f5;--fg:#1d1d1b;--muted:#6b6b66;--line:#deded8;--card:#ffffff;--good:#1f7a43;--warn:#946200;--bad:#b3261e;--good-bg:#e5f3ea;--warn-bg:#fbf0d9;--bad-bg:#fbe4e2;--link:#1f5fa8;}
@media (prefers-color-scheme: dark){:root:not([data-theme="light"]){--bg:#151514;--fg:#ececea;--muted:#a0a09a;--line:#33332f;--card:#1d1d1b;--good:#6fcf97;--warn:#f2c94c;--bad:#f28b82;--good-bg:#173323;--warn-bg:#3a2f12;--bad-bg:#3d1c1a;--link:#8ab4f8;}}
:root[data-theme="dark"]{--bg:#151514;--fg:#ececea;--muted:#a0a09a;--line:#33332f;--card:#1d1d1b;--good:#6fcf97;--warn:#f2c94c;--bad:#f28b82;--good-bg:#173323;--warn-bg:#3a2f12;--bad-bg:#3d1c1a;--link:#8ab4f8;}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:1100px;margin:0 auto;padding:24px 16px 48px}
h1{font-size:28px;margin:0}
h2{font-size:18px;margin:32px 0 8px}
.sub,.note,footer{color:var(--muted)}
.note{font-size:13px}
.mono{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:13px}
.facts{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:8px;margin:16px 0 0}
.facts div{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:8px 12px}
dt{color:var(--muted);font-size:12px}dd{margin:2px 0 0;font-weight:600}
.scroll{overflow-x:auto;background:var(--card);border:1px solid var(--line);border-radius:8px}
table{border-collapse:collapse;width:100%;min-width:640px}
th,td{text-align:left;padding:8px 12px;border-bottom:1px solid var(--line);vertical-align:top}
th{font-size:12px;color:var(--muted);font-weight:600}
tr:last-child td{border-bottom:0}
.num{text-align:right;font-variant-numeric:tabular-nums}
.empty{color:var(--muted);text-align:center}
.pill{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:600;background:var(--line)}
.pill.good{color:var(--good);background:var(--good-bg)}.pill.warn{color:var(--warn);background:var(--warn-bg)}.pill.bad{color:var(--bad);background:var(--bad-bg)}
.routes{padding-left:18px}.routes li{margin:4px 0}
a{color:var(--link)}
footer{margin-top:40px;font-size:13px}
</style></head><body>
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn health() -> Value {
        json!({
            "status": "ok", "service": "jiaoyi", "phase": 1, "load_in_progress": false,
            "settings_version": 30, "account_refusals_recorded": 0,
            "token_grants": [{"account_id": 43002148, "environment": "live", "state": "listed",
                "reason": null, "token_fingerprint": "fp-DO-NOT-SHOW", "host": "live.ctraderapi.com"}],
            "last_loads": [{"account_id": 43002148, "run_id": 7, "status": "partial",
                "finished_at": "2026-10-24T10:00:00Z", "errors": ["<script>alert(1)</script> asset list: refused"]}],
            "tick_recorders": {"43002148": {"subscribed_symbols": [1, 2, 3], "sessions_started": 2,
                "ticks_written": 1234, "last_flush_at": null, "last_error": null}},
            "ticks": [{"account_id": 43002148, "last_received_at": "2026-10-24T10:01:00Z", "last_hour": 980}],
        })
    }

    fn jobs() -> Value {
        json!({
            "prereg_file": "docs/preregistration-2026-10-24.md",
            "prereg_hash_running": "9105c5eb812ff6b3d267627a53e64d924018c799753d70e598830381a909098f",
            "engine_version_running": 2,
            "held_out_min_engine": 2,
            "spread_sampling": {"weeks_measured": 26, "weeks_without_ticks": 4, "weeks_failed": 0,
                "ticks_downloaded": 5000, "ticks_per_second": 812.5},
            "jobs": [
                {"id": 1, "stage": "development", "setting": {"breakout": 50, "stop_atr": 3.0, "default": true},
                 "status": "done", "reason": null, "attempts": [{"attempt": 1}]},
                {"id": 10, "stage": "held_out", "setting": {"breakout": 50, "stop_atr": 3.0, "default": true},
                 "status": "pending", "reason": null, "attempts": []},
            ],
        })
    }

    fn trials() -> Value {
        json!({"trials": [{"job_id": 1, "verdict": "development: PASS rules 1-3 not met",
            "results": {"r": {"n": 120, "mean_r": 0.0412, "profit_factor": 1.18, "pf_infinite": false}}}]})
    }

    #[test]
    fn the_page_shows_each_section_and_escapes_what_it_shows() {
        let now = "2026-10-24T12:00:00Z".parse().unwrap();
        let p = page(&health(), &jobs(), &trials(), now);
        for want in [
            "<title>jiaoyi status</title>",
            "Settings version</dt><dd>30</dd>",
            "Accounts",
            ">listed<",
            ">partial<",
            "Tick recorders",
            ">1234<",
            ">980<",
            "Research jobs",
            "breakout 50 · stop 3.0×ATR · default",
            "development: PASS rules 1-3 not met",
            ">120<",
            ">0.041<",
            ">1.18<",
            "812.5 per second",
            "Engine version 2 running; the held-out job waits for engine 2 or later.",
            r#"href="/research/trials""#,
            r#"href="/research/bars""#,
            r#"href="/health""#,
            "Generated 2026-10-24 12:00:00 UTC",
        ] {
            assert!(p.contains(want), "missing {want:?}");
        }
        // Database text is escaped, never markup.
        assert!(!p.contains("<script>alert(1)</script>"));
        assert!(p.contains("&lt;script&gt;alert(1)&lt;/script&gt; asset list: refused"));
        // No token fingerprint or broker host on the page.
        assert!(!p.contains("fp-DO-NOT-SHOW"));
        assert!(!p.contains("ctraderapi"));
        // No write: no form, no POST.
        assert!(!p.contains("<form") && !p.to_lowercase().contains("post"));
    }

    #[test]
    fn the_trading_section_shows_switch_gate_and_external_positions() {
        let now = "2026-10-25T05:00:00Z".parse().unwrap();
        let h = json!({"trading": [
            {"account_id": 43002148, "enabled": true, "allow_min_volume": true,
             "edge_gate": "not met: no H1 held-out trial under 9105c5eb yet",
             "last_decision": {"kind": "signal_held", "reason": "edge gate <b>not</b> met", "at": "2026-10-25T04:59:00Z"},
             "last_reconcile": {"external_positions": []}},
            {"account_id": 46130058, "enabled": false, "allow_min_volume": false, "edge_gate": "not required",
             "last_decision": null,
             "last_reconcile": {"external_positions": [{"position_id": 77, "symbol_id": 22, "side": "sell", "volume": 300}]}}
        ]});
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(p.contains("<h2>Trading</h2>"));
        assert!(p.contains(r#"<span class="pill good">on</span>"#));
        assert!(p.contains(r#"<span class="pill neutral">off</span>"#));
        assert!(p.contains("not met: no H1 held-out trial"));
        assert!(p.contains("edge gate &lt;b&gt;not&lt;/b&gt; met"));
        assert!(
            p.contains(r#"<td class="num">1</td>"#),
            "one external position on 46130058"
        );
    }

    #[test]
    fn an_empty_service_says_so() {
        let now = "2026-10-24T12:00:00Z".parse().unwrap();
        let p = page(&json!({}), &json!({}), &json!({}), now);
        assert!(p.contains("No account recorded yet."));
        assert!(p.contains("No recorder running yet."));
        assert!(p.contains("No research job registered."));
    }

    #[test]
    fn escaping_covers_the_five_characters() {
        assert_eq!(
            esc(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }
}
