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
pub const ROUTES: [(&str, &str); 8] = [
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
        "/trading/costs",
        "each strategy's actual costs and order-to-fill times from its fills (no money)",
    ),
    (
        "/trading/orders?account=…&strategy=s1|s3&since=…&limit=…",
        "every Strategy 1 and Strategy 3 order: outcome, fill, position, costs and its decision (no money)",
    ),
    (
        "/research/bars?account=…&symbol=…&from=…&to=…",
        "development-period daily bars (parameters required)",
    ),
];

// Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
/// One trade row of a strategy's trades table: the time, symbol, order,
/// side, outcome, price, volume, order-to-fill ms and slippage, after the
/// account where the table spans accounts.
fn trade_row(account: Option<&Value>, t: &Value) -> String {
    let account = account
        .map(|a| format!(r#"<td class="mono">{}</td>"#, txt(a)))
        .unwrap_or_default();
    format!(
        "<tr>{account}<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
        txt(&t["at"]),
        txt(&t["symbol"]),
        txt(&t["kind"]),
        txt(&t["side"]),
        txt(&t["outcome"]),
        txt(&t["price"]),
        txt(&t["volume"]),
        txt(&t["order_to_fill_ms"]),
        num(&t["slippage"], 2),
    )
}

/// The link from a strategy's section to every order it placed.
fn orders_link(strategy: &str) -> String {
    format!(
        r#"<p class="note">Every order, with its outcome, fill, position and the decision it was placed under: <a class="mono" href="/trading/orders?strategy={s}">/trading/orders?strategy={s}</a></p>"#,
        s = esc(strategy)
    )
}

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
        "ok" | "listed" | "done" | "measured" | "trading" | "alive" => "good",
        // A live account trading with its edge gate lifted, and by whom.
        s if s.starts_with("trading (") => "good",
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
        "within limits" => "good",
        s if s.starts_with("paused by the owner") => "warn",
        "PAUSED by the owner" => "warn",
        "LIMIT REACHED: owner's decision needed" => "bad",
        "partial" | "pending" | "running" | "unread" | "starting" => "warn",
        "failed" | "refused" | "not_listed" | "environment_mismatch" | "stalled" => "bad",
        _ => "neutral",
    };
    format!(r#"<span class="pill {kind}">{}</span>"#, esc(state))
}

/// A loop's liveness: its state, and when its reconcile last ran and it
/// last processed a quote (Strategy 1's and Strategy 3's alike).
fn loop_cell(live: &Value) -> String {
    match live["state"].as_str() {
        Some(state) => format!(
            r#"{}<div class="note">reconcile {} · quote {}</div>"#,
            pill(state),
            txt(&live["last_reconcile_run_at"]),
            txt(&live["last_quote_at"]),
        ),
        None => "—".into(),
    }
}

fn rows(v: &Value) -> &[Value] {
    v.as_array().map(Vec::as_slice).unwrap_or(&[])
}

/// Each account's results in R, its limit alert and the owner's pause, for
/// a strategy's section (Strategy 1's and Strategy 3's alike). The limits
/// alert; only the owner pauses. R only, no money.
// Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
fn performance_table(accounts: &[Value]) -> String {
    let mut h = String::from(
        r#"<h3>Performance and limits</h3><div class="scroll"><table><thead><tr><th>Account</th><th class="num">Closed trades</th><th class="num">Net R</th><th class="num">Profit factor</th><th class="num">Drawdown R (largest · now)</th><th>Limits</th><th>Owner's pause</th></tr></thead><tbody>"#,
    );
    if accounts.is_empty() {
        h.push_str(r#"<tr><td colspan="7" class="empty">No account recorded yet.</td></tr>"#);
    }
    for a in accounts {
        let p = &a["performance"];
        let limits = if let Some(e) = p["error"].as_str() {
            format!(
                r#"{}<div class="note">{}</div>"#,
                pill("unread"),
                esc(&short(e, 160))
            )
        } else if p["owner_decision_needed"].as_bool() == Some(true) {
            let what = rows(&p["limit_reached"])
                .iter()
                .map(|x| {
                    format!(
                        "{} {} (limit {})",
                        txt(&x["limit"]),
                        num(&x["value"], 2),
                        txt(&x["threshold"])
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            format!(
                r#"{}<div class="note">{what}. Trading continues; only the owner pauses.</div>"#,
                pill("LIMIT REACHED: owner's decision needed")
            )
        } else if p.is_object() {
            let pf = match p["to_limits"]["pf_margin"].as_f64() {
                Some(m) => format!("profit factor {m:.2} above its floor"),
                None => format!(
                    "profit factor judged after {} more closed trade(s)",
                    txt(&p["to_limits"]["pf_trades_to_judge"])
                ),
            };
            format!(
                r#"{}<div class="note">{} R of drawdown room; {pf}</div>"#,
                pill("within limits"),
                num(&p["to_limits"]["drawdown_room_r"], 2),
            )
        } else {
            "—".into()
        };
        let o = &a["owner_pause"];
        let pause = match o["paused"].as_bool() {
            Some(true) => format!(
                r#"{}<div class="note">by {} at {}: {}</div>"#,
                pill("PAUSED by the owner"),
                txt(&o["changed_by"]),
                txt(&o["changed_at"]),
                esc(&short(o["reason"].as_str().unwrap_or("—"), 160)),
            ),
            Some(false) => "not paused".into(),
            None => "—".into(),
        };
        let unmeasured = match p["unmeasured"].as_u64() {
            Some(n) if n > 0 => format!(r#"<div class="note">{n} not measured</div>"#),
            _ => String::new(),
        };
        h.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td class=\"num\">{}{unmeasured}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{} · {}</td><td>{limits}</td><td>{pause}</td></tr>",
            txt(&a["account_id"]),
            txt(&p["closed_trades"]),
            num(&p["net_r"], 2),
            num(&p["profit_factor"], 2),
            num(&p["max_drawdown_r"], 2),
            num(&p["current_drawdown_r"], 2),
        ));
    }
    h.push_str("</tbody></table></div>");
    h
}

/// A distance to a trigger, or why there is none.
fn distance_cell(d: &Value, reason: &Value) -> String {
    match d.as_f64() {
        Some(_) => num(d, 2),
        None => format!(r#"—<div class="note">{}</div>"#, txt(reason)),
    }
}

/// Strategy 1's distance to each symbol's trigger at its last quote: the
/// 50-day level on the side the EMA filter allows, and the quote's distance
/// to it in ATR (negative once through it).
fn s1_trigger_table(accounts: &[Value]) -> String {
    let mut h = String::from(
        r#"<h3>Distance to trigger</h3><p class="note">At each symbol's last quote: the 50-day level on the side the EMA filter allows, and how far the ask (long) or bid (short) is from it in ATR. Negative means through it.</p><div class="scroll"><table><thead><tr><th>Account</th><th>Symbol</th><th>Filter</th><th class="num">50-day level</th><th class="num">ATR</th><th class="num">Quote</th><th class="num">Distance (ATR)</th><th>At</th></tr></thead><tbody>"#,
    );
    let mut any = false;
    for a in accounts {
        for (symbol, t) in a["trigger_distance"].as_object().into_iter().flatten() {
            any = true;
            h.push_str(&format!(
                "<tr><td class=\"mono\">{}</td><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"mono\">{}</td></tr>",
                txt(&a["account_id"]),
                esc(symbol),
                txt(&t["filter"]),
                txt(&t["level"]),
                num(&t["atr"], 5),
                txt(&t["quote"]),
                distance_cell(&t["distance_atr"], &t["reason"]),
                txt(&t["at"]),
            ));
        }
    }
    if !any {
        h.push_str(r#"<tr><td colspan="8" class="empty">No quote processed yet.</td></tr>"#);
    }
    h.push_str("</tbody></table></div>");
    h
}

/// Strategy 3's distance to each symbol's trigger at its last quote: the
/// noise band then, the bid and ask, and the nearer side's distance to its
/// edge in half-widths (open × σ), negative once through it; or why there
/// is none (outside the entry window, no band).
fn s3_trigger_table(a: &Value) -> String {
    let mut h = String::from(
        r#"<div class="scroll"><table><thead><tr><th>Symbol</th><th class="num">Band lower</th><th class="num">Band upper</th><th class="num">σ</th><th class="num">Bid</th><th class="num">Ask</th><th>Nearer edge</th><th class="num">Distance (half-widths)</th><th>At</th></tr></thead><tbody>"#,
    );
    let mut any = false;
    for (symbol, t) in a["trigger_distance"].as_object().into_iter().flatten() {
        any = true;
        h.push_str(&format!(
            "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"mono\">{}</td></tr>",
            esc(symbol),
            num(&t["lower"], 5),
            num(&t["upper"], 5),
            num(&t["sigma"], 5),
            txt(&t["bid"]),
            txt(&t["ask"]),
            txt(&t["nearer_edge"]),
            distance_cell(&t["distance_half_widths"], &t["reason"]),
            txt(&t["at"]),
        ));
    }
    if !any {
        h.push_str(r#"<tr><td colspan="9" class="empty">No quote processed yet.</td></tr>"#);
    }
    h.push_str("</tbody></table></div>");
    h
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

    // Strategy 1 per account: whether it trades (demo at once, live once the
    // edge gate passes or the owner lifts it), the gate, the minimum-volume rule, the open-risk
    // budget, the last decision, whether the loop is alive (its reconcile
    // runs and last quote, recorded or not) and the positions it found that
    // it did not open. No money.
    h.push_str(r#"<section><h2>Strategy 1: tick-triggered trend</h2><div class="scroll"><table><thead><tr><th>Account</th><th>Environment</th><th>Trades</th><th>Edge gate</th><th>Minimum volume</th><th class="num">Open-risk budget</th><th>Last decision</th><th>Loop</th><th class="num">External positions</th></tr></thead><tbody>"#);
    let trading = rows(&health["trading"]);
    if trading.is_empty() {
        h.push_str(r#"<tr><td colspan="9" class="empty">No account recorded yet.</td></tr>"#);
    }
    for t in trading {
        let trades = match t["trades"].as_str() {
            Some("yes") => pill("trading"),
            Some(other) => pill(other),
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
        // Claude · № 17,020 25-Oct (production read-back 04:26Z)
        let looping = loop_cell(&t["liveness"]);
        let external = t["last_reconcile"]["external_positions"]
            .as_array()
            .map(|a| a.len().to_string())
            .unwrap_or_else(|| "—".into());
        let budget = t["trend_open_risk_pct"]
            .as_f64()
            .map(|p| format!("{p}%"))
            .unwrap_or_else(|| "—".into());
        h.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td>{trades}</td><td class=\"note\">{}</td><td>{}</td><td class=\"num\">{budget}</td><td>{decision}</td><td>{looping}</td><td class=\"num\">{external}</td></tr>",
            txt(&t["account_id"]),
            txt(&t["environment"]),
            esc(&short(t["edge_gate"].as_str().unwrap_or("—"), 160)),
            match t["allow_min_volume"].as_bool() {
                Some(true) => "allowed",
                Some(false) => "skip",
                None => "—",
            },
        ));
    }
    h.push_str("</tbody></table></div>");
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
    h.push_str(&performance_table(trading));
    h.push_str(&s1_trigger_table(trading));
    // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
    // Strategy 1's trades today, the same columns as Strategy 3's, and every order.
    h.push_str(r#"<h3>Trades today</h3><div class="scroll"><table><thead><tr><th>Account</th><th>Time</th><th>Symbol</th><th>Order</th><th>Side</th><th>Outcome</th><th class="num">Price</th><th class="num">Volume</th><th class="num">Order to fill (ms)</th><th class="num">Slippage</th></tr></thead><tbody>"#);
    let mut any = false;
    for t in rows(&health["trading"]) {
        for x in rows(&t["trades_today"]) {
            any = true;
            h.push_str(&trade_row(Some(&t["account_id"]), x));
        }
    }
    if !any {
        h.push_str(r#"<tr><td colspan="10" class="empty">No trade today.</td></tr>"#);
    }
    h.push_str("</tbody></table></div>");
    h.push_str(&orders_link("s1"));
    h.push_str("</section>");

    // Claude · № 17,020 25-Oct (owner's Strategy 3 v2: a section per account)
    // Strategy 3 per account: its open positions, today's decisions and
    // today's trades. Prices and volumes only, no money.
    h.push_str(r#"<section><h2>Strategy 3: intraday momentum</h2>"#);
    let s3 = rows(&health["strategy3"]);
    if s3.is_empty() {
        h.push_str(r#"<p class="sub">No account recorded yet.</p>"#);
    }
    if let Some(first) = s3.first() {
        h.push_str(&format!(
            r#"<p class="sub">Pre-registration <span class="mono">{}</span>, hash <span class="mono">{}</span>; label <span class="mono">{}</span>. Entries at a 30-minute mark on the mid; skipped when C is at least R.</p>"#,
            txt(&first["prereg_file"]),
            esc(&short(first["prereg_hash"].as_str().unwrap_or("—"), 12)),
            txt(&first["label"]),
        ));
    }
    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
    if !s3.is_empty() {
        h.push_str(&performance_table(s3));
    }
    for a in s3 {
        let positions = rows(&a["open_positions"]);
        let pos = if positions.is_empty() {
            "none".to_string()
        } else {
            positions
                .iter()
                .map(|p| {
                    format!(
                        "{} {} {} at {} (stop {}, take-profit {})",
                        txt(&p["symbol"]),
                        txt(&p["side"]),
                        txt(&p["volume"]),
                        txt(&p["entry"]),
                        txt(&p["stop"]),
                        txt(&p["take_profit"]),
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let counts = rows(&a["today"]["decisions_today"])
            .iter()
            .map(|c| format!("{} {}", txt(&c["kind"]), txt(&c["count"])))
            .collect::<Vec<_>>()
            .join(", ");
        // Claude · № 17,074 25-Oct (ordered: owner "merge when green and
        // deploy"; Strategy 3's heartbeat, as #16 gave Strategy 1)
        h.push_str(&format!(
            r#"<h3 class="mono">{} <span class="sub">{} · {}</span></h3><div class="note">Loop: {}</div><p class="note">Open: {pos}</p><p class="note">Decisions today: {}</p>"#,
            txt(&a["account_id"]),
            txt(&a["environment"]),
            txt(&a["sizing"]),
            loop_cell(&a["liveness"]),
            if counts.is_empty() { "none".to_string() } else { counts },
        ));
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
        h.push_str(r#"<p class="note">Distance to trigger, at each symbol's last quote (negative means through the band's edge):</p>"#);
        h.push_str(&s3_trigger_table(a));
        h.push_str(r#"<div class="scroll"><table><thead><tr><th>Time</th><th>Symbol</th><th>Order</th><th>Side</th><th>Outcome</th><th class="num">Price</th><th class="num">Volume</th><th class="num">Order to fill (ms)</th><th class="num">Slippage</th></tr></thead><tbody>"#);
        let trades = rows(&a["today"]["trades_today"]);
        if trades.is_empty() {
            h.push_str(r#"<tr><td colspan="9" class="empty">No trade today.</td></tr>"#);
        }
        for t in trades {
            h.push_str(&trade_row(None, t));
        }
        h.push_str("</tbody></table></div>");
    }
    h.push_str(&orders_link("s3"));
    h.push_str("</section>");

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
    fn the_strategy_3_section_shows_positions_decisions_and_trades_without_money() {
        let now = "2026-10-25T05:00:00Z".parse().unwrap();
        let h = json!({"strategy3": [
            {"account_id": 49236051, "environment": "demo", "label": "jiaoyi-s3",
             "sizing": "risk_pct of fresh equity",
             "prereg_file": "docs/preregistration-2026-10-25-strategy3.md",
             "prereg_hash": "35a3c9071cb73d6d09c72ea492af0ba2b7c7ebee",
             "liveness": {"state": "stalled", "last_reconcile_run_at": "2026-10-25T04:02:28Z", "last_quote_at": "2026-10-25T04:03:00Z"},
             "open_positions": [{"symbol": "BTCUSD", "side": "buy", "volume": 8, "entry": 60462.0,
                                  "stop": 60012.0, "take_profit": 62563.21}],
             "today": {"decisions_today": [{"kind": "s3_sent", "count": 1}],
                       "trades_today": [{"at": "2026-10-25T00:45:00Z", "symbol": "BTCUSD", "kind": "open",
                                          "side": "buy", "outcome": "filled", "price": 60462.0, "volume": 13,
                                          "order_to_fill_ms": 182, "slippage": 2.0}]}}
        ]});
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(p.contains("<h2>Strategy 3: intraday momentum</h2>"));
        // The loop's liveness, whether or not a decision changed (as Strategy 1's).
        assert!(
            p.contains(r#"Loop: <span class="pill bad">stalled</span><div class="note">reconcile 2026-10-25T04:02:28Z · quote 2026-10-25T04:03:00Z</div>"#),
            "{p}"
        );
        assert!(p.contains("BTCUSD buy 8 at 60462.0 (stop 60012.0, take-profit 62563.21)"));
        assert!(p.contains("s3_sent 1"));
        assert!(p.contains(">182<") && p.contains(">2.00<"));
        assert!(p.contains("jiaoyi-s3"));
        assert!(!p.to_lowercase().contains("balance") && !p.to_lowercase().contains("equity "));
    }

    // Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
    #[test]
    fn strategy_1_shows_its_trades_today_and_both_strategies_link_their_orders() {
        let now = "2026-10-25T05:00:00Z".parse().unwrap();
        let h = json!({
            "trading": [{"account_id": 46130058, "environment": "demo", "trades": "yes",
                "trades_today": [{"at": "2026-10-25T04:10:00Z", "symbol": "ETHUSD", "kind": "broker_exit",
                                   "side": "buy", "outcome": "filled", "price": 3489.0, "volume": 5,
                                   "order_to_fill_ms": null, "slippage": 2.0},
                                  {"at": "2026-10-25T04:00:00Z", "symbol": "ETHUSD", "kind": "open",
                                   "side": "buy", "outcome": "filled", "price": 3611.0, "volume": 5,
                                   "order_to_fill_ms": 141, "slippage": 0.0}]}],
            "strategy3": [{"account_id": 46130058, "environment": "demo",
                "today": {"trades_today": []}}],
        });
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(p.contains("<h3>Trades today</h3>"), "{p}");
        assert!(
            p.contains(r#"<td class="mono">46130058</td><td>2026-10-25T04:10:00Z</td><td>ETHUSD</td><td>broker_exit</td>"#),
            "{p}"
        );
        assert!(p.contains(">141<") && p.contains(">2.00<"), "{p}");
        assert!(p.contains(r#"href="/trading/orders?strategy=s1""#), "{p}");
        assert!(p.contains(r#"href="/trading/orders?strategy=s3""#), "{p}");
        assert!(
            p.contains(r#"<a class="mono" href="/trading/orders">"#),
            "{p}"
        );
        let none = page(
            &json!({"trading": [{"account_id": 1, "trades_today": []}]}),
            &json!({}),
            &json!({}),
            now,
        );
        assert!(none.contains(r#"<td colspan="10" class="empty">No trade today.</td>"#));
    }

    #[test]
    fn the_strategy_section_shows_trading_state_gate_and_external_positions() {
        let now = "2026-10-25T05:00:00Z".parse().unwrap();
        let h = json!({"trading": [
            {"account_id": 43002148, "environment": "live", "trades": "armed: waits for the edge gate",
             "allow_min_volume": true, "trend_open_risk_pct": 3.0,
             "edge_gate": "not met: no H1 held-out trial under 9105c5eb yet",
             "last_decision": {"kind": "signal_held", "reason": "edge gate <b>not</b> met", "at": "2026-10-25T04:59:00Z"},
             "last_reconcile": {"external_positions": []},
             "liveness": {"state": "stalled", "last_reconcile_run_at": "2026-10-25T04:02:28Z", "last_quote_at": null}},
            {"account_id": 46130058, "environment": "demo", "trades": "yes", "allow_min_volume": false,
             "trend_open_risk_pct": 3.0, "edge_gate": "not required (demo)", "last_decision": null,
             "last_reconcile": {"external_positions": [{"position_id": 77, "symbol_id": 22, "side": "sell", "volume": 300}]},
             "liveness": {"state": "alive", "last_reconcile_run_at": "2026-10-25T04:58:00Z", "last_quote_at": "2026-10-25T04:59:59Z"}}
        ]});
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(p.contains("<h2>Strategy 1: tick-triggered trend</h2>"));
        // The loop's liveness, whether or not a decision changed.
        assert!(p.contains(r#"<span class="pill bad">stalled</span><div class="note">reconcile 2026-10-25T04:02:28Z · quote —</div>"#), "{p}");
        assert!(p.contains(r#"<span class="pill good">alive</span><div class="note">reconcile 2026-10-25T04:58:00Z · quote 2026-10-25T04:59:59Z</div>"#));
        assert!(p.contains(r#"<span class="pill good">trading</span>"#));
        assert!(p.contains("armed: waits for the edge gate"));
        assert!(p.contains("not met: no H1 held-out trial"));
        assert!(p.contains("edge gate &lt;b&gt;not&lt;/b&gt; met"));
        assert!(p.contains("3%"));
        assert!(
            p.contains(r#"<td class="num">1</td>"#),
            "one external position on 46130058"
        );
        // Live with the edge gate lifted by the owner (25-10): trading, and by whom.
        let h = json!({"trading": [
            {"account_id": 43002148, "environment": "live",
             "trades": "trading (edge gate lifted by the owner 25-10)",
             "edge_gate": "edge gate lifted by the owner 25-10 (trading.live_requires_edge_pass 0)"}
        ]});
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(
            p.contains(
                r#"<span class="pill good">trading (edge gate lifted by the owner 25-10)</span>"#
            ),
            "{p}"
        );
    }

    // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
    #[test]
    fn both_strategies_show_results_limit_alerts_the_owners_pause_and_trigger_distance() {
        let now = "2026-10-25T08:00:00Z".parse().unwrap();
        let hit = json!({"closed_trades": 50, "unmeasured": 1, "net_r": -12.5, "profit_factor": 0.7,
            "max_drawdown_r": 11.5, "current_drawdown_r": 11.0,
            "to_limits": {"drawdown_room_r": -1.5, "pf_trades_to_judge": 0, "pf_margin": -0.1},
            "limit_reached": [{"limit": "drawdown_r", "value": 11.5, "threshold": 10.0},
                              {"limit": "profit_factor", "value": 0.7, "threshold": 0.8}],
            "owner_decision_needed": true});
        let ok = json!({"closed_trades": 3, "unmeasured": 0, "net_r": 1.2, "profit_factor": 2.0,
            "max_drawdown_r": 1.0, "current_drawdown_r": 0.0,
            "to_limits": {"drawdown_room_r": 9.0, "pf_trades_to_judge": 47, "pf_margin": null},
            "limit_reached": null, "owner_decision_needed": false});
        let h = json!({
            "trading": [{"account_id": 46130058, "environment": "demo",
                "trades": "paused by the owner: no new entry", "performance": hit,
                "owner_pause": {"paused": true, "changed_by": "owner",
                    "changed_at": "2026-10-25T07:00:00Z", "reason": "owner: pause <b>S1</b>"},
                "trigger_distance": {
                    "BTCUSD": {"at": "2026-10-25T07:59:00Z", "filter": "buy", "level": 68000.0,
                        "atr": 1500.0, "quote": 67250.0, "distance_atr": 0.5, "reason": null},
                    "XAUUSD": {"at": "2026-10-25T07:59:30Z", "distance_atr": null,
                        "reason": "no levels: fewer than 120 completed days"}}}],
            "strategy3": [{"account_id": 49236051, "environment": "demo", "performance": ok,
                "owner_pause": {"paused": false},
                "trigger_distance": {
                    "BTCUSD": {"at": "2026-10-25T07:58:00Z", "lower": 59550.0, "upper": 60450.0,
                        "sigma": 0.0075, "bid": 60435.0, "ask": 60440.0, "nearer_edge": "upper",
                        "distance_half_widths": 0.0222, "reason": null},
                    "ETHUSD": {"at": "2026-10-25T07:58:00Z", "distance_half_widths": null,
                        "reason": "the lunch break"}},
                "today": {"trades_today": []}}],
        });
        let p = page(&h, &json!({}), &json!({}), now);
        assert!(p.contains("<h3>Performance and limits</h3>"), "{p}");
        // The alert, its numbers, and that only the owner pauses.
        assert!(
            p.contains(
                r#"<span class="pill bad">LIMIT REACHED: owner&#39;s decision needed</span>"#
            ),
            "{p}"
        );
        assert!(p.contains("drawdown_r 11.50 (limit 10.0); profit_factor 0.70 (limit 0.8). Trading continues; only the owner pauses."), "{p}");
        assert!(
            p.contains(">50<div class=\"note\">1 not measured</div>"),
            "{p}"
        );
        assert!(
            p.contains(">-12.50<") && p.contains(">11.50 · 11.00<"),
            "{p}"
        );
        // The owner's pause, by whom and why, escaped.
        assert!(p.contains(r#"<span class="pill warn">PAUSED by the owner</span><div class="note">by owner at 2026-10-25T07:00:00Z: owner: pause &lt;b&gt;S1&lt;/b&gt;</div>"#), "{p}");
        assert!(
            p.contains(r#"<span class="pill warn">paused by the owner: no new entry</span>"#),
            "{p}"
        );
        // Within the limits: the room left and when the profit factor is judged.
        assert!(p.contains(r#"<span class="pill good">within limits</span><div class="note">9.00 R of drawdown room; profit factor judged after 47 more closed trade(s)</div>"#), "{p}");
        assert!(p.contains("not paused"));
        // The distance to each trigger, or why there is none.
        assert!(p.contains("<h3>Distance to trigger</h3>"));
        assert!(p.contains(r#"<td>BTCUSD</td><td>buy</td><td class="num">68000.0</td><td class="num">1500.00000</td><td class="num">67250.0</td><td class="num">0.50</td>"#), "{p}");
        assert!(
            p.contains("no levels: fewer than 120 completed days"),
            "{p}"
        );
        assert!(p.contains(r#"<td>BTCUSD</td><td class="num">59550.00000</td><td class="num">60450.00000</td><td class="num">0.00750</td><td class="num">60435.0</td><td class="num">60440.0</td><td>upper</td><td class="num">0.02</td>"#), "{p}");
        assert!(
            p.contains(r#"—<div class="note">the lunch break</div>"#),
            "{p}"
        );
        assert!(!p.to_lowercase().contains("balance"));
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
