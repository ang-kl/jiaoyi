//! Pure presentation of supplied trading evidence; no IO, policy or model calls.
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use chrono_tz::Asia::Singapore;
use serde_json::Value;

use crate::{status_page::esc, telegram_report::Period};

const UNKNOWN: &str = "unavailable";
fn number(v: &Value) -> Option<f64> {
    v.as_f64().filter(|n| n.is_finite())
}
fn count(v: &Value) -> String {
    v.as_u64()
        .map(|n| n.to_string())
        .unwrap_or_else(|| UNKNOWN.into())
}
fn value(n: Option<f64>) -> String {
    n.map(|n| {
        if n != 0.0 && n.abs() < 0.01 {
            format!("{n:.8}")
        } else {
            format!("{n:.2}")
        }
    })
    .unwrap_or_else(|| UNKNOWN.into())
}
fn pct(n: Option<f64>) -> String {
    n.map(|n| format!("{}%", value(Some(n))))
        .unwrap_or_else(|| UNKNOWN.into())
}
fn key(v: &Value) -> String {
    v.as_i64()
        .map(|n| n.to_string())
        .or_else(|| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown account".into())
}
fn text(v: &Value) -> &str {
    v.as_str().filter(|s| !s.is_empty()).unwrap_or(UNKNOWN)
}
fn stamp(v: &Value) -> Option<DateTime<Utc>> {
    v.as_str()?.parse().ok()
}
fn sgt(t: DateTime<Utc>) -> String {
    t.with_timezone(&Singapore)
        .format("%d-%m-%Y %H:%M:%S SGT")
        .to_string()
}
fn time(v: &Value) -> String {
    stamp(v).map(sgt).unwrap_or_else(|| UNKNOWN.into())
}
fn currency(a: &Value) -> Option<&str> {
    a["currency"].as_str().filter(|s| {
        (3..=12).contains(&s.len())
            && s.bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    })
}
fn money(n: Option<f64>, currency: Option<&str>) -> String {
    format!("{} {}", currency.unwrap_or("currency unverified"), value(n))
}
fn p(s: &str) -> String {
    format!("<p>{}</p>", esc(s))
}
fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::from("<div class=scroll><table><thead><tr>");
    for h in headers {
        out.push_str(&format!("<th>{}</th>", esc(h)));
    }
    out.push_str("</tr></thead><tbody>");
    for row in rows {
        out.push_str("<tr>");
        for c in row {
            out.push_str(&format!("<td>{}</td>", esc(c)));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table></div>");
    out
}
fn metric(label: &str, value: String) -> String {
    format!("<div><dt>{}</dt><dd>{}</dd></div>", esc(label), esc(&value))
}
fn risk_pct(risk: Option<f64>, equity: Option<f64>) -> Option<f64> {
    let (r, e) = (risk?, equity?);
    (r >= 0.0 && e > 0.0)
        .then_some(r / e * 100.0)
        .filter(|n| n.is_finite())
}
fn risk_text(s: &Value, field: &str, c: Option<&str>) -> String {
    let risk = number(&s[field]).filter(|r| *r >= 0.0);
    let supplemental = if field == "open_risk" {
        "open_stop_loss_before_unquoted_exit_costs"
    } else {
        "pending_stop_loss_before_unquoted_costs"
    };
    if risk.is_none() {
        if let Some(amount) = number(&s[supplemental]).filter(|r| *r >= 0.0) {
            return format!("Total-cost risk unavailable; stop-price exposure {} / {} of derived equity, excludes unquoted exit costs/gaps",money(Some(amount),c),pct(risk_pct(Some(amount),number(&s["equity"]))));
        }
    }
    format!(
        "{} / {} of derived equity",
        money(risk, c),
        pct(risk_pct(risk, number(&s["equity"])))
    )
}
fn notes(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|s| !s.is_empty())
        .take(4)
        .map(|s| s.chars().take(240).collect())
        .collect()
}
fn breach(v: &Value) -> bool {
    v.as_bool() == Some(true) || v.as_array().is_some_and(|a| !a.is_empty())
}
fn breach_text(v: &Value) -> String {
    if let Some(rows) = v.as_array() {
        if rows.is_empty() {
            return "No recorded registered breach; check definitions".into();
        }
        return rows
            .iter()
            .map(|r| {
                format!(
                    "{}: measured {}, registered threshold {}",
                    text(&r["limit"]),
                    value(number(&r["value"])),
                    value(number(&r["threshold"]))
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
    }
    if breach(v) {
        "Registered breach; owner decision".into()
    } else {
        "Breach evidence unavailable".into()
    }
}
fn cash_text(d: &Value, c: Option<&str>) -> String {
    if d["unpriced_deals"].as_u64().is_some_and(|n| n > 0) {
        if d["closing_deals"] == d["unpriced_deals"] {
            return UNKNOWN.into();
        }
        return format!(
            "Known part {}; {} unpriced deal(s)",
            money(number(&d["cash_net"]), c),
            count(&d["unpriced_deals"])
        );
    }
    format!(
        "{}{}",
        money(number(&d["cash_net"]), c),
        if d["history_complete"] == true {
            ""
        } else {
            " (history incomplete)"
        }
    )
}
fn quantity(v: &Value) -> String {
    v.as_u64()
        .map(|n| n.to_string())
        .or_else(|| v.as_array().map(|a| a.len().to_string()))
        .unwrap_or_else(|| UNKNOWN.into())
}
fn positive(v: &Value) -> bool {
    v.as_u64().is_some_and(|n| n > 0) || v.as_array().is_some_and(|a| !a.is_empty())
}
fn paused(v: &Value) -> bool {
    v.as_bool() == Some(true) || v["paused"].as_bool() == Some(true)
}
fn pause_text(v: &Value) -> String {
    if paused(v) {
        format!(
            "Owner pause; actor {}; time {}; reason {}",
            text(&v["changed_by"]),
            time(&v["changed_at"]),
            text(&v["reason"])
        )
    } else if v.as_bool() == Some(false) || v["paused"].as_bool() == Some(false) {
        "No recorded owner pause".into()
    } else {
        "Owner-pause evidence unavailable".into()
    }
}
fn limits(v: &Value) -> String {
    let mut parts = Vec::new();
    if let Some(values) = v.as_object() {
        for (k, v) in values {
            let label = k.replace(['_', '.'], " ");
            if let Some(n) = number(v) {
                parts.push(format!(
                    "{label}: {}{}",
                    value(Some(n)),
                    if k.ends_with("_pct") { "%" } else { "" }
                ));
            } else if let Some(s) = v.as_str() {
                parts.push(format!("{label}: {s}"));
            }
        }
    } else {
        parts = notes(v);
    }
    if parts.is_empty() {
        "Registered limit definitions unavailable".into()
    } else {
        parts.join("; ")
    }
}
fn alerts(accounts: &[&Value]) -> Vec<String> {
    let mut out = Vec::new();
    if accounts.is_empty() {
        out.push("Linked-account evidence unavailable; roster coverage is not established".into());
    }
    for a in accounts {
        let s = &a["snapshot"];
        let mut reasons = Vec::new();
        if s["current"] != true {
            reasons.push(
                "stale, incomplete or unverified current snapshot; excluded from headline totals"
                    .into(),
            );
        }
        if stamp(&s["observed_at"]).is_none() || number(&s["equity"]).is_none() {
            reasons.push("current account valuation unavailable".into());
        }
        if positive(&s["unprotected_pending_orders"]) {
            reasons.push(format!(
                "{} unprotected pending order(s)",
                quantity(&s["unprotected_pending_orders"])
            ));
        }
        if positive(&s["ambiguous_protection_links"]) {
            reasons.push(format!(
                "{} ambiguous protection link(s)",
                quantity(&s["ambiguous_protection_links"])
            ));
        }
        if currency(a).is_none() {
            reasons.push("account currency unverified".into());
        }
        if s["unprotected_positions"].as_u64().is_some_and(|n| n > 0) {
            reasons.push(format!(
                "{} unprotected position(s): verify broker protection",
                count(&s["unprotected_positions"])
            ));
        }
        if s["unpriced_positions"].as_u64().is_some_and(|n| n > 0) {
            reasons.push(format!(
                "{} position(s) with unpriced exposure",
                count(&s["unpriced_positions"])
            ));
        }
        if s["open_positions"].as_u64().is_some_and(|n| n > 0) && number(&s["open_risk"]).is_none()
        {
            reasons.push("open equity-to-stop risk unavailable".into());
        }
        if s["pending_orders"].as_u64().is_some_and(|n| n > 0)
            && number(&s["pending_risk"]).is_none()
        {
            reasons.push("pending-order risk unavailable".into());
        }
        if number(&s["free_margin"]).is_some_and(|n| n < 0.0) {
            reasons.push(format!(
                "negative derived free margin {}",
                money(number(&s["free_margin"]), currency(a))
            ));
        }
        if number(&a["daily"]["economic_pnl"]).is_none() {
            reasons.push("daily economic P&L incomplete".into());
        }
        for r in a["risks"].as_array().into_iter().flatten() {
            if breach(&r["limit_reached"]) {
                reasons.push(format!(
                    "{} registered limit reached; owner decision required",
                    text(&r["strategy"])
                ));
            }
        }
        for note in notes(&s["notes"]) {
            let lower = note.to_lowercase();
            if lower.contains("margin call") || lower.contains("stop-out") {
                reasons.push(format!("Broker note: {note}"));
            }
        }
        if !reasons.is_empty() {
            out.push(format!(
                "Account {}: {}",
                key(&a["account_id"]),
                reasons.join("; ")
            ));
        }
    }
    out
}
#[derive(Clone)]
struct Contribution {
    group: String,
    account: String,
    strategy: String,
    amount: f64,
}
fn contributions(daily: &[Value], accounts: &[&Value]) -> (Vec<Contribution>, usize) {
    let mut out = Vec::new();
    let mut excluded = 0;
    for row in daily {
        if row["closing_deals"].as_u64() == Some(0) {
            continue;
        }
        let a = accounts
            .iter()
            .copied()
            .find(|a| key(&a["account_id"]) == key(&row["account"]));
        let amount = number(&row["cash_net"]);
        if let (Some(a), Some(amount)) = (a, amount) {
            if let Some(c) = currency(a) {
                let env = text(&a["environment"]);
                let source_currency = row["currency"].as_str();
                if matches!(env, "live" | "demo")
                    && row["history_covers_period"] == true
                    && row["unpriced_deals"].as_u64() == Some(0)
                    && source_currency.is_none_or(|s| s == c)
                {
                    out.push(Contribution {
                        group: format!("{} · {c}", env.to_uppercase()),
                        account: key(&a["account_id"]),
                        strategy: text(&row["strategy"]).into(),
                        amount,
                    });
                    continue;
                }
            }
        }
        excluded += 1;
    }
    (out, excluded)
}
fn rankings(values: &[Contribution]) -> String {
    let mut groups: BTreeMap<&str, Vec<&Contribution>> = BTreeMap::new();
    for v in values {
        groups.entry(&v.group).or_default().push(v);
    }
    if groups.is_empty() {
        return p("Program cash attribution unavailable: no fully priced, history-covered contributions to rank.");
    }
    let mut out = String::new();
    for (group, mut rows) in groups {
        rows.sort_by(|a, b| {
            b.amount
                .total_cmp(&a.amount)
                .then(a.account.cmp(&b.account))
                .then(a.strategy.cmp(&b.strategy))
        });
        let positive: Vec<_> = rows
            .iter()
            .filter(|r| r.amount > 0.0)
            .take(2)
            .map(|r| format!("{} {}: {}", r.account, r.strategy, value(Some(r.amount))))
            .collect();
        let negative: Vec<_> = rows
            .iter()
            .rev()
            .filter(|r| r.amount < 0.0)
            .take(2)
            .map(|r| format!("{} {}: {}", r.account, r.strategy, value(Some(r.amount))))
            .collect();
        out.push_str(&p(&format!(
            "{group} — positive contributors: {}; negative contributors: {}.",
            if positive.is_empty() {
                "none observed".into()
            } else {
                positive.join("; ")
            },
            if negative.is_empty() {
                "none observed".into()
            } else {
                negative.join("; ")
            }
        )));
    }
    out
}
fn valid_assessment(a: &Value, account: &str, strategy: &str) -> bool {
    key(&a["account_id"]) == account
        && a["strategy"] == strategy
        && a["history_complete"] == true
        && a["missing_reasons"].as_array().is_some_and(Vec::is_empty)
        && matches!(
            a["state"].as_str(),
            Some("below_target" | "window_meets_numeric_reference")
        )
        && a["window"]["count"].as_u64() == Some(20)
        && a["window"]["wins"].as_u64().is_some_and(|n| n <= 20)
        && a["window"]["realized_payoff_state"] == "defined"
        && number(&a["window"]["realized_payoff"]).is_some()
}
fn previous_delta(row: &Value, account: &Value, strategy: &str) -> String {
    let a = &row["report"]["assessment"];
    let prior = &account["previous_goals"][strategy];
    let id = key(&account["account_id"]);
    let chronological = stamp(&prior["assessed_at"])
        .zip(stamp(&a["assessed_at"]))
        .is_some_and(|(p, c)| p < c);
    if row["monitor_state"] != "alive"
        || !valid_assessment(a, &id, strategy)
        || !valid_assessment(prior, &id, strategy)
        || !chronological
    {
        return "unavailable (no prior comparable valid cohort)".into();
    }
    let win = (a["window"]["wins"].as_u64().unwrap() as f64
        - prior["window"]["wins"].as_u64().unwrap() as f64)
        * 5.0;
    let payoff = number(&a["window"]["realized_payoff"]).unwrap()
        - number(&prior["window"]["realized_payoff"]).unwrap();
    format!("Win rate {win:+.2} pp; payoff {payoff:+.6}")
}
fn planned_rr(a: &Value) -> String {
    if let Some(n) = number(&a["planned_rr"]).filter(|n| *n > 0.0) {
        return format!("1:{n:.4}");
    }
    if let Some(s) = a["planned_risk_reward"].as_str() {
        if let Some((risk, reward)) = s.split_once(':') {
            if let (Ok(r), Ok(w)) = (risk.trim().parse::<f64>(), reward.trim().parse::<f64>()) {
                if r.is_finite() && w.is_finite() && r > 0.0 && w > 0.0 {
                    return format!("{r}:{w}");
                }
            }
        }
    }
    UNKNOWN.into()
}
fn daily_hit(d: &Value) -> String {
    match (
        d["positions"].as_u64(),
        d["wins"].as_u64(),
        d["unpriced_positions"].as_u64(),
    ) {
        (Some(n), Some(w), Some(0)) if n > 0 && w <= n && d["history_complete"] == true => {
            format!("{w}/{n} · {:.2}%", 100.0 * w as f64 / n as f64)
        }
        (Some(0), Some(0), Some(0)) if d["history_complete"] == true => {
            "No fully closed positions; hit ratio undefined".into()
        }
        _ => "unavailable (incomplete position outcomes)".into(),
    }
}

/// All input amounts are already normalized by the data collector. Return and
/// risk percentages supplied in account data are percentage points, not fractions.
pub fn render(period: &Period, daily: &[Value], accounts: &[Value], goals: &Value) -> String {
    let serial = format!("T-{}", period.ny_date.format("%Y%m%d"));
    let mut ordered: Vec<_> = accounts.iter().collect();
    ordered.sort_by_key(|a| {
        (
            match a["environment"].as_str() {
                Some("live") => 0,
                Some("demo") => 1,
                _ => 2,
            },
            key(&a["account_id"]),
        )
    });
    let generated = goals
        .get("generated_at")
        .filter(|v| stamp(v).is_some())
        .unwrap_or(&goals["as_of"]);
    let mut body = format!("<h1>Daily trading report · {}</h1>", period.ny_date);
    body.push_str(&p(&format!("NY17:00 accounting cutoff. SGT window [{} → {}). Generated/source projection: {}. Current stored broker snapshots below are separate from cutoff-valued performance; later marks do not establish NY-close equity.",sgt(period.from),sgt(period.to),time(generated))));
    body.push_str("<nav><a href=#performance>Performance</a> · <a href=#risk>Risk &amp; goals</a> · <a href=#context>Context</a> · <a href=#commentary>Next actions</a></nav><section id=performance><h2>1. Executive summary &amp; performance</h2>");
    body.push_str(&p("Goal per strategy/account: ≥73% net wins over the latest20 fully closed owned positions (15/20=75%), with realized payoff reference1.38. Planned R:R remains separately verified. One passing window does not establish consistency."));
    let alert_rows = alerts(&ordered);
    if !alert_rows.is_empty() {
        body.push_str("<aside class=alert><strong>What needs the owner</strong><ul>");
        for a in &alert_rows {
            body.push_str(&format!("<li>{}</li>", esc(a)));
        }
        body.push_str("</ul></aside>");
    } else {
        body.push_str(&p("No source-backed owner alert is flagged in the supplied evidence; unavailable limits or metrics are not a pass."));
    }
    let mut groups: BTreeMap<(u8, &str), Vec<&Value>> = BTreeMap::new();
    for a in &ordered {
        if let (Some(c), Some(env)) = (currency(a), a["environment"].as_str()) {
            if matches!(env, "live" | "demo") && a["snapshot"]["current"] == true {
                groups
                    .entry((u8::from(env == "demo"), c))
                    .or_default()
                    .push(a);
            }
        }
    }
    let mut totals = Vec::new();
    for ((demo, c), rows) in groups {
        let sum = |field: &str| {
            rows.iter()
                .try_fold(0.0, |sum, a| number(&a["snapshot"][field]).map(|n| sum + n))
        };
        let economic = rows.iter().try_fold(0.0, |sum, a| {
            number(&a["daily"]["economic_pnl"]).map(|n| sum + n)
        });
        totals.push(vec![
            format!(
                "{} · {c} · {} account(s)",
                if demo == 1 { "DEMO" } else { "LIVE" },
                rows.len()
            ),
            money(sum("balance"), Some(c)),
            money(sum("float"), Some(c)),
            money(sum("equity"), Some(c)),
            money(economic, Some(c)),
        ]);
    }
    if totals.is_empty() {
        body.push_str(&p("Verified-currency capital totals unavailable. No currency conversion or combined live/demo capital is inferred."));
    } else {
        body.push_str(&table(
            &[
                "Verified currency / scope",
                "Balance",
                "Net float",
                "Derived equity",
                "Daily economic P&L",
            ],
            &totals,
        ));
    }
    body.push_str(&p("Live capital and demo test capital stay separate. Only current, verified same-currency receipts are grouped; stale/incomplete snapshots are excluded, so these are observed current-account subtotals rather than a complete roster total; any missing amount makes that group total unavailable. Derived equity is balance plus net unrealized P&L, with credit/bonus treatment unverified. Net float excludes future exit commission. It is not guaranteed liquidation cash. MTD/YTD returns are not summed across accounts."));
    body.push_str("<div class=cards>");
    for a in &ordered {
        let s = &a["snapshot"];
        let d = &a["daily"];
        let c = currency(a);
        body.push_str(&format!(
            "<article class=card><h3>Account {} · {} · {}</h3>",
            esc(&key(&a["account_id"])),
            esc(text(&a["environment"])),
            esc(c.unwrap_or("currency unverified"))
        ));
        let mut badges = Vec::new();
        if number(&d["economic_pnl"]).is_some_and(|n| n < 0.0) {
            badges.push("Loss today");
        }
        if number(&s["float"]).is_some_and(|n| n < 0.0) {
            badges.push("Floating loss");
        }
        if d["underwater"] == true {
            badges.push("Underwater");
        } else if d["underwater"].is_null() {
            badges.push("Underwater status unavailable");
        }
        if s["unprotected_positions"].as_u64().is_some_and(|n| n > 0) {
            badges.push("Protection alert");
        }
        for badge in badges {
            body.push_str(&format!("<span class=badge>{}</span>", esc(badge)));
        }
        body.push_str("<dl class=metrics>");
        for (label, display) in [
            ("Balance", money(number(&s["balance"]), c)),
            ("Net float", money(number(&s["float"]), c)),
            ("Derived equity / NLV proxy", money(number(&s["equity"]), c)),
            ("Credit / bonus", money(number(&s["credit"]), c)),
            ("Daily economic P&L", money(number(&d["economic_pnl"]), c)),
            ("Recorded closing cash (not economic P&L)", cash_text(d, c)),
            ("Daily return", pct(number(&d["return_pct"]))),
            (
                "MTD / YTD return",
                format!(
                    "{} / {}",
                    pct(number(&d["mtd_return_pct"])),
                    pct(number(&d["ytd_return_pct"]))
                ),
            ),
            ("Open equity-to-stop risk", risk_text(s, "open_risk", c)),
            (
                "Open positions / pending orders",
                format!(
                    "{} / {}",
                    count(&s["open_positions"]),
                    count(&s["pending_orders"])
                ),
            ),
        ] {
            body.push_str(&metric(label, display));
        }
        body.push_str("</dl>");
        body.push_str(&p(&format!("Stored broker receipt: {} (current-source status shown by owner alerts). Daily marks: {} → {}. Economic P&L and returns remain unavailable without reconciled cutoff marks and external flows; current float is not the day's change in float.",time(&s["observed_at"]),time(&d["opening_at"]),time(&d["closing_at"]))));
        body.push_str("</article>");
    }
    body.push_str("</div><h3>Program contributors · cash attribution only</h3>");
    let (contributors, excluded) = contributions(daily, &ordered);
    body.push_str(&rankings(&contributors));
    body.push_str(&p(&format!("Rankings use owned strategy closing-deal cash within the day, including partial closes; they are not whole-account economic P&L. {excluded} source row(s) excluded for incomplete history, costs, identity or currency. No strategy/asset double counting; daily cohort net R unavailable, latest20 net R appears below.")));
    body.push_str("</section><section id=risk><h2>2. Risk, execution &amp; goal progress</h2><h3>All-account exposure and existing controls</h3>");
    for a in &ordered {
        let s = &a["snapshot"];
        let c = currency(a);
        let margin = if number(&s["used_margin"]) == Some(0.0) {
            "No margin used".into()
        } else {
            pct(number(&s["margin_level"]))
        };
        body.push_str(&format!(
            "<h3>Account {} · {}</h3>",
            esc(&key(&a["account_id"])),
            esc(text(&a["environment"]))
        ));
        body.push_str(&table(
            &[
                "Used / derived free margin",
                "Derived margin level",
                "Broker account leverage",
                "Gross leverage",
                "Pending risk",
                "Unprotected positions / pending / ambiguous links / unpriced",
            ],
            &[vec![
                format!(
                    "{} / {}",
                    money(number(&s["used_margin"]), c),
                    money(number(&s["free_margin"]), c)
                ),
                margin,
                number(&s["broker_leverage"])
                    .map(|n| format!("1:{n:.2}"))
                    .unwrap_or_else(|| UNKNOWN.into()),
                number(&s["gross_leverage"])
                    .map(|n| format!("{n:.2}x"))
                    .unwrap_or_else(|| UNKNOWN.into()),
                risk_text(s, "pending_risk", c),
                format!(
                    "{} / {} / {} / {}",
                    count(&s["unprotected_positions"]),
                    quantity(&s["unprotected_pending_orders"]),
                    quantity(&s["ambiguous_protection_links"]),
                    count(&s["unpriced_positions"])
                ),
            ]],
        ));
        let mut rows = Vec::new();
        for r in a["risks"].as_array().into_iter().flatten() {
            rows.push(vec![
                text(&r["strategy"]).into(),
                format!(
                    "{} / {}",
                    value(number(&r["current_drawdown_r"])),
                    value(number(&r["max_drawdown_r"]))
                ),
                value(number(&r["net_r"])),
                value(number(&r["profit_factor"])),
                breach_text(&r["limit_reached"]),
                pause_text(&r["owner_pause"]),
                limits(&r["limits"]),
            ]);
        }
        if rows.is_empty() {
            body.push_str(&p("Registered-limit and owner-pause evidence unavailable."));
        } else {
            body.push_str(&table(
                &[
                    "Strategy",
                    "Current / worst DD (R)",
                    "Cumulative net R",
                    "Profit factor",
                    "Existing alert",
                    "Owner pause",
                    "Registered limits",
                ],
                &rows,
            ));
        }
        for note in notes(&s["notes"]).into_iter().take(2) {
            body.push_str(&p(&format!("Source note: {note}")));
        }
        body.push_str(&p(&format!("Daily net hit ratio (fully closed positions): {}. This is separate from latest20 goal progress.",daily_hit(&a["daily"]))));
    }
    body.push_str(&p("Exposure includes linked-account manual/external positions and pending orders as supplied by broker evidence; external positions remain reporting-only. Equity-to-stop loss and compatible pending risk are separate, with source conversion/cost gaps above. Ordinary stops can slip. Broker account leverage differs from gross notional/derived equity. Free margin, margin level and gross leverage are cash-equity-derived proxies excluding unverified credit; they are not verified broker margin-call levels. No new margin floor, pause, stop, size or strategy rule is imposed."));
    body.push_str("<h3 id=goals>Latest20 per strategy/account · target ≥73% and payoff1.38</h3>");
    let mut goal_rows = Vec::new();
    for a in &ordered {
        for strategy in ["s1", "s3"] {
            let row = goals["reports"]
                .as_array()
                .and_then(|rows| {
                    rows.iter().find(|r| {
                        key(&r["account_id"]) == key(&a["account_id"]) && r["strategy"] == strategy
                    })
                })
                .unwrap_or(&Value::Null);
            let assessment = &row["report"]["assessment"];
            let w = &assessment["window"];
            let (win_gap, pay_gap) = crate::telegram_report::benchmark_gaps(row);
            let state = if row["monitor_state"] == "alive" {
                text(&assessment["state"]).to_owned()
            } else {
                format!("Unverified / {}", text(&assessment["state"]))
            };
            goal_rows.push(vec![
                format!("{} / {strategy}", key(&a["account_id"])),
                format!(
                    "{} /20; {} /{} net wins",
                    count(&w["count"]),
                    count(&w["wins"]),
                    count(&w["count"])
                ),
                format!(
                    "{}; {win_gap}",
                    pct(number(&w["win_rate"]).map(|n| n * 100.0))
                ),
                format!("Planned: {}", planned_rr(assessment)),
                format!(
                    "{}; {pay_gap}",
                    number(&w["realized_payoff"])
                        .map(|n| format!("{n:.6}"))
                        .unwrap_or_else(|| UNKNOWN.into())
                ),
                value(number(&w["net_r"])),
                previous_delta(row, a, strategy),
                format!("{state}; checked {}", time(&row["checked_at"])),
            ]);
        }
    }
    body.push_str(&table(
        &[
            "Account / strategy",
            "Valid closes / net wins",
            "Win rate / gap to73%",
            "Planned R:R",
            "Realized payoff / gap to1.38",
            "Window net R",
            "Prior comparable delta",
            "Evidence state / SGT",
        ],
        &goal_rows,
    ));
    body.push_str(&p("No composite success percentage is calculated. Missing/stale/incomplete cohorts cannot establish a target pass or an improvement; planned R:R is never copied from realized payoff. Current and prior cohorts may overlap and do not establish consistent profitability."));
    body.push_str("<h3>Execution · same-symbol measurements only</h3>");
    let mut fills = Vec::new();
    for a in &ordered {
        for e in a["execution"].as_array().into_iter().flatten() {
            if e["fill_count"].as_u64() == Some(0) {
                continue;
            }
            fills.push(vec![
                key(&a["account_id"]),
                text(&e["symbol"]).into(),
                number(&e["volume"])
                    .map(|n| {
                        format!(
                            "{} native units ({} cTrader cents)",
                            value(Some(n / 100.0)),
                            value(Some(n))
                        )
                    })
                    .unwrap_or_else(|| UNKNOWN.into()),
                count(&e["fill_count"]),
                format!(
                    "{}; coverage {}",
                    value(number(&e["slippage_mean"])),
                    count(&e["slippage_samples"])
                ),
                format!(
                    "{} / {}; coverage {}",
                    value(number(&e["latency_ms_mean"])),
                    value(number(&e["latency_ms_max"])),
                    count(&e["latency_samples"])
                ),
            ]);
        }
    }
    if fills.is_empty() {
        body.push_str(&p("No execution measurements supplied for the day; absence of this input is not proof of zero trading."));
    } else {
        body.push_str(&table(
            &[
                "Account",
                "Symbol",
                "Native volume",
                "Fill receipts",
                "Mean slippage (price)",
                "Mean / max latency(ms)",
            ],
            &fills,
        ));
    }
    body.push_str(&p("Volume and averages are never added across instruments. Recorded slippage is positive when adverse against its recorded decision/submission quote; broker-exit coverage can be unavailable. VWAP benchmark unavailable: CFD ticks are not exchange volume. 99% one-day VaR is not validated; twenty trades cannot validate that tail."));
    body.push_str("</section><section id=context><h2>3. Regime &amp; factor context</h2>");
    body.push_str(&p("Context unavailable: no validated portfolio-aligned volatility, liquidity, concentration/correlation-change or factor dataset was supplied. No market narrative, VIX proxy or event cause is inferred."));
    body.push_str("</section><section id=commentary><h2>4. Portfolio-manager commentary &amp; next actions</h2>");
    let positives = contributors.iter().filter(|v| v.amount > 0.0).count();
    let negatives = contributors.iter().filter(|v| v.amount < 0.0).count();
    body.push_str(&p(&format!("Observed: {positives} positive and {negatives} negative validated program cash contributors, separated by live/demo and currency above. These cash observations do not explain unreconciled whole-account economic P&L or prove a market cause.")));
    body.push_str(&p("Interventions and their measured effects: unavailable from the supplied daily evidence. Existing owner pauses and registered breaches are shown above; a current pause is not proof of an intervention during this day."));
    body.push_str(&p(if alert_rows.is_empty(){"Next action: retain existing rules and assess the next source-backed cohort. Owner/scheduled intervention time unavailable; verified upcoming macro-event calendar unavailable."}else{"Next action: owner reviews the source-backed account/protection and registered-limit alerts above. No action is executed by this report. Intervention deadline in SGT and a verified upcoming macro-event calendar are unavailable."}));
    body.push_str(&p("Sources: supplied broker account snapshots, period closing-deal receipts and persisted strategy assessments. Missing values remain unavailable; no raw JSON, retry logs or stacks are included. Deterministic HTML rendering uses no report-writing LLM call. Daily delivery remains NY17:15; this report neither closes positions nor retunes rules."));
    body.push_str("</section>");
    let mut numbered = String::new();
    let mut previous = 0;
    for (index, (at, _)) in body.match_indices("<p>").enumerate() {
        numbered.push_str(&body[previous..at + 3]);
        numbered.push_str(&format!("[§{serial}.{}] ", index + 1));
        previous = at + 3;
    }
    numbered.push_str(&body[previous..]);
    format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>jiaoyi {serial}</title><style>body{{font:15px system-ui;line-height:1.5;max-width:1150px;margin:1.5rem auto;padding:0 1rem;color:#162333}}h1{{font-size:1.6rem}}h2{{font-size:1.3rem;margin-top:2rem}}h3{{font-size:1.05rem}}nav{{margin:1rem 0}}a{{color:#125a9c}}.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(270px,1fr));gap:1rem}}.card{{border:1px solid #ccd6df;border-radius:10px;padding:1rem}}.metrics{{display:grid;grid-template-columns:1fr 1fr;gap:.7rem}}dt{{font-size:.78rem;color:#526170}}dd{{margin:0;font-weight:600;overflow-wrap:anywhere}}.alert{{background:#fff1df;padding:1rem;border-left:4px solid #b55209}}.badge{{display:inline-block;background:#fee8dd;padding:.15rem .4rem;margin:.2rem;border-radius:4px;font-size:.8rem}}.scroll{{overflow-x:auto}}table{{width:100%;border-collapse:collapse;font-size:.82rem}}th,td{{padding:.5rem;border-bottom:1px solid #ccd6df;text-align:left;vertical-align:top}}th{{background:#eef3f7}}section{{overflow-wrap:anywhere}}</style><body data-report-version=\"2\"><div>№{serial}</div>{numbered}</body></html>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn period() -> Period {
        crate::telegram_report::period_due("2026-10-26T21:15:00Z".parse().unwrap())
    }
    fn account(id: i64, env: &str, c: &str, balance: f64, equity: f64) -> Value {
        json!({"account_id":id,"environment":env,"currency":c,
            "snapshot":{"current":true,"observed_at":"2026-10-26T21:14:00Z","balance":balance,"float":equity-balance,"equity":equity,"credit":null,"used_margin":0,"free_margin":equity,"margin_level":null,"broker_leverage":25,"gross_leverage":0,"open_risk":0,"pending_risk":0,"open_positions":0,"pending_orders":0,"unprotected_positions":0,"unprotected_pending_orders":0,"ambiguous_protection_links":0,"unpriced_positions":0,"notes":[]},
            "daily":{"economic_pnl":5,"return_pct":2,"mtd_return_pct":3,"ytd_return_pct":4,"cash_net":7,"closing_deals":1,"positions":1,"wins":1,"unpriced_deals":0,"unpriced_positions":0,"history_complete":true,"underwater":false,"opening_at":"2026-10-25T21:00:00Z","closing_at":"2026-10-26T21:00:00Z"},"execution":[],"risks":[],"previous_goals":{}})
    }
    fn assessment(id: i64, wins: u64, payoff: f64, at: &str) -> Value {
        json!({"account_id":id,"strategy":"s3","state":"window_meets_numeric_reference","history_complete":true,"missing_reasons":[],"assessed_at":at,
            "window":{"count":20,"wins":wins,"losses":20-wins,"win_rate":wins as f64/20.0,"realized_payoff":payoff,"realized_payoff_state":"defined","net_r":12}})
    }
    fn goals(a: Value) -> Value {
        json!({"generated_at":"2026-10-26T21:15:00Z","reports":[{"account_id":1,"strategy":"s3","monitor_state":"alive","checked_at":"2026-10-26T21:05:00Z","report":{"assessment":a}}]})
    }

    #[test]
    fn four_sections_are_deterministic_numbered_sgt_and_exclude_raw_payloads() {
        let mut a = account(1, "live", "USD", 100.0, 110.0);
        a["snapshot"]["notes"] = json!(["<script>alert('x')</script>"]);
        a["raw_debug"] = json!({"secret":"RAW_PAYLOAD_MUST_NOT_RENDER"});
        let g = json!({"generated_at":"2026-10-26T21:15:00Z","reports":[]});
        let html = render(&period(), &[], &[a.clone()], &g);
        assert_eq!(html, render(&period(), &[], &[a], &g));
        assert_eq!(html.matches("<h2>").count(), 4);
        assert!(html.contains("data-report-version=\"2\""));
        assert!(html.contains("№T-20261026"));
        assert!(html.contains("[§T-20261026.1]"));
        assert!(html.contains("27-10-2026 05:00:00 SGT"));
        assert!(html.contains("27-10-2026 05:14:00 SGT"));
        assert!(html.contains("27-10-2026 05:15:00 SGT"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("RAW_PAYLOAD_MUST_NOT_RENDER"));
        assert!(!html.contains("<pre>"));
    }

    #[test]
    fn capital_groups_keep_live_demo_currency_and_stale_sources_separate() {
        let live = account(1, "live", "USD", 100.0, 110.0);
        let demo = account(2, "demo", "USD", 1000.0, 980.0);
        let mut stale = account(3, "live", "EUR", 500.0, 550.0);
        stale["snapshot"]["current"] = json!(false);
        let mut unavailable = account(4, "demo", "JPY", 0.0, 0.0);
        unavailable["snapshot"] = Value::Null;
        let html = render(
            &period(),
            &[],
            &[live, demo, stale, unavailable],
            &json!({"reports":[]}),
        );
        let headline = html.split("<div class=cards>").next().unwrap();
        assert!(headline.contains("LIVE · USD"));
        assert!(headline.contains("DEMO · USD"));
        assert!(headline.contains("USD 110.00"));
        assert!(headline.contains("USD 980.00"));
        assert!(!headline.contains("USD 1090.00"));
        assert!(!headline.contains("EUR 550.00"));
        assert!(html.contains("EUR 550.00"));
        assert_eq!(html.matches("<article class=card>").count(), 4);
        assert!(html.contains("Account 4"));
        assert!(html.contains("excluded from headline totals"));
        assert!(html.contains("Derived equity"));
        assert!(html.contains("Credit / bonus"));
        assert!(html.contains("Recorded closing cash (not economic P&amp;L)"));
        assert!(html.contains("2.00%")); // percentage-point input, not 200%.
    }

    #[test]
    fn daily_hit_and_qualified_goal_delta_are_distinct_and_invalid_evidence_stays_unknown() {
        let mut a = account(1, "live", "USD", 100.0, 110.0);
        a["previous_goals"]["s3"] = assessment(1, 14, 1.8, "2026-10-25T21:00:00Z");
        let mut g = goals(assessment(1, 15, 2.5, "2026-10-26T21:00:00Z"));
        let html = render(&period(), &[], &[a.clone()], &g);
        assert!(html.contains("1/1 · 100.00%"));
        assert!(html.contains("75.00%"));
        assert!(html.contains("Win rate +5.00 pp; payoff +0.700000"));
        assert!(html.contains("Planned: unavailable"));
        assert!(!html.contains("Planned: 1:2.5"));
        g["reports"][0]["monitor_state"] = json!("stale");
        a["daily"]["history_complete"] = json!(false);
        a["daily"]["unpriced_positions"] = json!(1);
        let unknown = render(&period(), &[], &[a], &g);
        assert!(!unknown.contains("Win rate +5.00 pp"));
        assert!(unknown.contains("no prior comparable valid cohort"));
        assert!(unknown.contains("incomplete position outcomes"));
        assert!(unknown.contains("unknown (incomplete or stale evidence)"));
    }

    #[test]
    fn real_breach_pause_and_partial_stop_exposure_use_source_definitions() {
        let mut a = account(1, "live", "USD", 100.0, 110.0);
        a["snapshot"]["open_positions"] = json!(2);
        a["snapshot"]["open_risk"] = Value::Null;
        a["snapshot"]["open_stop_loss_before_unquoted_exit_costs"] = json!(11);
        a["snapshot"]["unprotected_pending_orders"] = json!(1);
        a["snapshot"]["ambiguous_protection_links"] = json!(2);
        a["risks"] = json!([{"strategy":"s3","current_drawdown_r":12,"max_drawdown_r":15,"net_r":-3,"profit_factor":0.6,
            "limit_reached":[{"limit":"drawdown_r","value":15,"threshold":10}],"limits":{"drawdown_r":10},
            "owner_pause":{"paused":true,"changed_by":"owner","changed_at":"2026-10-26T21:10:00Z","reason":"review"}}]);
        let html = render(&period(), &[], &[a], &json!({"reports":[]}));
        assert!(html.contains("registered limit reached; owner decision required"));
        assert!(html.contains("measured 15.00, registered threshold 10.00"));
        assert!(html.contains("actor owner; time 27-10-2026 05:10:00 SGT; reason review"));
        assert!(html.contains("stop-price exposure USD 11.00 / 10.00%"));
        assert!(html.contains("excludes unquoted exit costs/gaps"));
        assert!(html.contains("1 unprotected pending order(s)"));
        assert!(html.contains("2 ambiguous protection link(s)"));
    }
}
