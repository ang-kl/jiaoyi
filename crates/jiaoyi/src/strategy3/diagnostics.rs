//! Read-only explanations from the same actor state used by admission. No I/O,
//! broker requests, order authority, or new trading rules live here.
use super::*;

fn gate(code: &str, state: &str, message: String) -> Value {
    json!({"code":code,"state":state,"message":message})
}

fn preparation_gate(
    prepared: bool,
    source: Option<&str>,
    last_refusal: Option<&str>,
) -> Option<Value> {
    if prepared {
        return None;
    }
    let message = if let Some(reason) = source {
        format!("Blocked: prepared admission source invalid: {reason}")
    } else if let Some(reason) = last_refusal {
        format!("Blocked: prepared admission unavailable; last refusal: {reason}")
    } else {
        "Preparing: background entry admission is not ready yet".into()
    };
    Some(gate(
        "prepared_admission_unavailable",
        if source.is_some() || last_refusal.is_some() {
            "blocked"
        } else {
            "preparing"
        },
        message,
    ))
}

impl Trader<'_, '_> {
    pub fn trigger_status(&self, id: i64, now: DateTime<Utc>) -> Value {
        let sym = &self.syms[&id];
        let mut result = trigger_distance(sym, now);
        result["production_policy"] = json!(mp::POLICY);
        result["entry_preparation"] = json!({
            "prepared_now":sym.pullback.is_some_and(|arm|self.fast_plan(id,arm).is_some()),
            "last_refusal":self.fast_cache.errors.get(&id),
            "source_refusal":fast_entry::source_refusal(sym),
            "evidence_scope":"background admission; final writer checks remain mandatory"
        });
        let session = sym.rule.and_then(|r| sessions::session_at(&r, now));
        result["entry_session_enabled"] =
            json!(session.is_some_and(|s| prod::symbol_entry_window(&sym.name, &s, now).is_ok()));
        if let Some(Ok(day)) = &sym.day {
            result["opening_reference"] = json!({"session_open":day.session.open,"session_close":day.session.close,
                "mid":day.open.as_ref().ok(),"quote":day.opening_quote,
                "macro_context":day.macro_context,"prepared_at":day.prepared_at});
        }
        let latest = sym
            .last_assessment
            .as_ref()
            .filter(|(s, at, _)| Some(*s) == session && *at <= now);
        result["last_assessment"] =
            json!(latest.map(|(s, at, code)| json!({"session_open":s.open,"at":at,"code":code})));
        let mut gates = Vec::new();
        if let Some(reason) = self.human_entry_refusal(id) {
            gates.push(gate(reason, "blocked", format!("Blocked: {reason}")));
        }
        if self.pair_entry_refusal(id).is_some() {
            gates.push(gate(
                "pair_risk_reservation_active",
                "blocked",
                format!(
                    "Blocked: pair reservation held by sibling symbol {}",
                    mp::peer(&sym.name).unwrap_or("unknown")
                ),
            ));
        }
        if self.own.values().any(|p| p.symbol_id == id)
            || self.pending.iter().any(|p| p.symbol_id == id)
        {
            gates.push(gate(
                "own_position_or_pending_entry",
                "blocked",
                "Managing existing position or pending entry; no additional entry".into(),
            ));
        }
        if result["entry_session_enabled"] != true {
            gates.push(gate(
                "outside_native_broker_session",
                "blocked",
                "Blocked: outside active native broker session".into(),
            ));
        }
        let arm = sym
            .pullback
            .filter(|a| now < prod::arm_expiry(&sym.name, &a.session, a.armed_at));
        if arm.is_none()
            && latest.is_some_and(|(_, _, code)| code == "invalidated_deep_retracement_gt_61_8pct")
        {
            gates.push(gate(
                "invalidated_deep_retracement_gt_61_8pct",
                "invalidated",
                "Invalidated: retraced > 61.8% without turn confirmation".into(),
            ));
        }
        if session.is_some_and(|s| sym.cost_skip_session == Some(s.open)) {
            gates.push(gate(
                "session_cost_veto",
                "blocked",
                "Blocked: recorded session cost reserve C >= R".into(),
            ));
        }
        let quote = fresh_s3_quote(sym, now);
        if quote.is_none() {
            gates.push(gate(
                "stale_or_missing_quote",
                "blocked",
                "Blocked: fresh two-sided quote unavailable".into(),
            ));
        }
        let day = sym
            .day
            .as_ref()
            .and_then(|d| d.as_ref().ok())
            .filter(|d| Some(d.session) == session);
        let direction = arm.map(|a| a.long).or_else(|| {
            day.and_then(|d| {
                let open = *d.open.as_ref().ok()?;
                let (bid, ask) = quote?;
                let b = prod::band_at(&d.table, &d.session, open, now).ok()?;
                band::mark_decision(&b, Some(bid), Some(ask)).map(|s| s == Dir::Buy)
            })
        });
        let impulse = arm.map(|a| a.impulse_bps).or_else(|| {
            day.and_then(|d| {
                quote
                    .zip(d.open.as_ref().ok())
                    .map(|((b, a), o)| ((b + a) / 2.0 - o).abs() / o * 10000.0)
            })
        });
        result["impulse_bps"] = json!(impulse);
        let evaluated = arm.is_some()
            || direction.is_some()
            || impulse.is_some_and(|r| r >= mp::IMPULSE_FLOOR_BPS);
        if evaluated {
            let long = direction.or_else(|| {
                day.and_then(|d| {
                    quote
                        .zip(d.open.as_ref().ok())
                        .map(|((b, a), o)| (b + a) / 2.0 > *o)
                })
            });
            let context = arm.map(|a| (a.macro_context, a.session.open)).or_else(|| {
                day.and_then(|d| d.macro_context.as_ref().ok().map(|c| (*c, d.session.open)))
            });
            if let Some(((context, open), long)) = context.zip(long) {
                result["macro_drift_5_pct"] = json!(context.drift_5 * 100.0);
                result["macro_drift_20_pct"] = json!(context.drift_20 * 100.0);
                if !mp::aligns(context, long, open) {
                    gates.push(gate(
                        "vetoed_by_5d_20d_macro_trend",
                        "vetoed",
                        format!(
                            "Vetoed: 5d/20d macro drift opposes impulse (5d: {:.3}%, 20d: {:.3}%)",
                            context.drift_5 * 100.0,
                            context.drift_20 * 100.0
                        ),
                    ));
                }
                let (movement, _) = self.peer_evidence(&sym.name, now);
                let signed = movement.map(|m| m * 10000.0 * if long { 1.0 } else { -1.0 });
                result["peer_signed_bps"] = json!(signed);
                if signed.is_none_or(|m| !m.is_finite() || m < 24.0) {
                    gates.push(gate(
                        "peer_hold_below_24bps_or_opposing",
                        "vetoed",
                        format!(
                            "Vetoed: peer {} < 24 bps or opposing ({}); mandatory at dispatch",
                            mp::peer(&sym.name).unwrap_or("unknown"),
                            signed
                                .map(|v| format!("{v:.2} signed bps"))
                                .unwrap_or_else(|| "missing or stale quote/preparation".into())
                        ),
                    ));
                }
            } else {
                gates.push(gate(
                    "macro_preparation_unavailable",
                    "blocked",
                    "Blocked: causal macro preparation unavailable".into(),
                ));
            }
        }
        if let Some(a) = arm {
            if let Some(preparation) = preparation_gate(
                result["entry_preparation"]["prepared_now"].as_bool() == Some(true),
                result["entry_preparation"]["source_refusal"].as_str(),
                result["entry_preparation"]["last_refusal"].as_str(),
            ) {
                gates.push(preparation);
            }
            result["impulse_extreme"] = json!(a.extreme);
            gates.push(gate(
                "armed_pullback",
                "armed",
                format!(
                    "Armed: waiting 50%–61.8% pullback + 8% turn (ref: {}, extreme: {}); {}",
                    a.level,
                    a.extreme,
                    if a.confirmation.touched_at.is_some() {
                        "touch recorded; later turn required"
                    } else {
                        "touch not yet recorded"
                    }
                ),
            ));
        } else if session.is_some_and(|s| sym.entered_session == Some(s.open)) {
            gates.push(gate(
                "session_already_entered",
                "blocked",
                "Blocked: one entry already recorded for this symbol/window".into(),
            ));
        } else if session.is_some_and(|s| {
            prod::decision_mark(&s, now)
                .is_some_and(|m| sym.last_decision_mark.is_some_and(|seen| seen >= m))
        }) {
            gates.push(gate(
                "decision_mark_consumed",
                "waiting",
                "Waiting: current decision mark already consumed".into(),
            ));
        } else if let Some((_, at, reason)) =
            latest.filter(|(_, at, _)| now - *at <= chrono::Duration::seconds(6))
        {
            if !gates.iter().any(|g| g["code"] == *reason) && reason != "armed_pullback" {
                gates.push(gate(
                    reason,
                    "waiting",
                    format!("Latest assessment: {reason} ({at})"),
                ));
            }
        }
        if gates.is_empty() {
            let reason = result["reason"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    if impulse.is_some_and(|r| r < mp::IMPULSE_FLOOR_BPS) {
                        "Waiting: opening impulse below 48 bps".into()
                    } else {
                        "Waiting: qualifying band crossing and admission checks".into()
                    }
                });
            gates.push(gate("waiting_for_qualified_setup", "waiting", reason));
        }
        result["entry_gate"] = gates[0].clone();
        result["reason"] = gates[0]["message"].clone();
        result["entry_gates"] = json!(gates);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_admission_discloses_margin_or_source_refusal_and_clears_after_success() {
        let margin = "margin buffer: projected margin level 209.46% below floor 250.00%";
        let blocked = preparation_gate(false, None, Some(margin)).unwrap();
        assert_eq!(blocked["state"], "blocked");
        assert!(blocked["message"].as_str().unwrap().contains(margin));
        assert!(blocked["message"]
            .as_str()
            .unwrap()
            .contains("last refusal"));
        assert!(
            preparation_gate(true, None, Some(margin)).is_none(),
            "old refusal must not override a current valid plan"
        );
        let source = preparation_gate(
            false,
            Some("original-window evidence unavailable"),
            Some(margin),
        )
        .unwrap();
        assert!(source["message"]
            .as_str()
            .unwrap()
            .contains("source invalid"));
        let waiting = preparation_gate(false, None, None).unwrap();
        assert_eq!(waiting["state"], "preparing");
        assert!(!waiting["message"].as_str().unwrap().contains("margin"));
    }
}
