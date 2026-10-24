//! The PASS rules, the development gate and the verdicts, exactly as the
//! pre-registration states them.
//!
//! PASS on the held-out data at the default setting, all of:
//! 1. mean net R per trade > 0, and the 95% lower bound of the week-block
//!    bootstrap (read as the 2.5th percentile) > 0;
//! 2. mean net R above the 95th percentile of the 1,000 control runs;
//! 3. profit factor ≥ 1.3;
//! 4. at least 80 trades for H1 (fewer: "insufficient evidence").
//!
//! The development gate, before the held-out data is opened: the default
//! meets rules 1–3 on development data, and at least 6 of the 9 grid cells
//! have a profit factor above 1.0. Otherwise: "no demonstrated edge".

use serde::{Deserialize, Serialize};

pub const PF_PASS: f64 = 1.3;
pub const H1_MIN_TRADES: usize = 80;
pub const GATE_MIN_CELLS: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    pub rule1_mean_positive: bool,
    pub rule1_bootstrap_lower_positive: bool,
    pub rule2_above_control_p95: bool,
    pub rule3_profit_factor: bool,
    pub rule4_min_trades: bool,
}

impl Rules {
    pub fn evaluate(
        n: usize,
        mean_r: Option<f64>,
        bootstrap_lower: Option<f64>,
        control_p95: Option<f64>,
        profit_factor: Option<f64>,
        min_trades: usize,
    ) -> Rules {
        let mean_pos = mean_r.is_some_and(|m| m > 0.0);
        Rules {
            rule1_mean_positive: mean_pos,
            rule1_bootstrap_lower_positive: bootstrap_lower.is_some_and(|b| b > 0.0),
            rule2_above_control_p95: matches!((mean_r, control_p95), (Some(m), Some(p)) if m > p),
            rule3_profit_factor: n > 0 && profit_factor.is_some_and(|pf| pf >= PF_PASS),
            rule4_min_trades: n >= min_trades,
        }
    }

    /// Rule 2 also requires `other_mean` strictly above the same control
    /// p95. Held-out (Codex P2 on #9): the controls are executed on daily
    /// bars, so the daily-bar-executed mean and the tick-counted mean must
    /// both be above it; either one alone fails rule 2.
    pub fn also_above_control_p95(
        mut self,
        other_mean: Option<f64>,
        control_p95: Option<f64>,
    ) -> Rules {
        self.rule2_above_control_p95 &=
            matches!((other_mean, control_p95), (Some(m), Some(p)) if m > p);
        self
    }

    pub fn rules_1_to_3(&self) -> bool {
        self.rule1_mean_positive
            && self.rule1_bootstrap_lower_positive
            && self.rule2_above_control_p95
            && self.rule3_profit_factor
    }
}

/// The held-out verdict: PASS, FAIL or "insufficient evidence".
pub fn held_out_verdict(r: &Rules) -> &'static str {
    if !r.rule4_min_trades {
        "insufficient evidence"
    } else if r.rules_1_to_3() {
        "PASS"
    } else {
        "FAIL"
    }
}

/// The development gate. `cells_pf`: the profit factor of each of the 9
/// grid cells (`None` for a cell without trades or not run).
pub fn development_gate(default_rules: &Rules, cells_pf: &[Option<f64>]) -> (bool, String) {
    let above = cells_pf
        .iter()
        .filter(|pf| pf.is_some_and(|v| v > 1.0))
        .count();
    let mut why = Vec::new();
    if !default_rules.rules_1_to_3() {
        why.push(format!(
            "the default (50, 3) did not meet PASS rules 1-3 on development data: {default_rules:?}"
        ));
    }
    if cells_pf.len() != 9 {
        why.push(format!(
            "{} of 9 development cells have a result",
            cells_pf.len()
        ));
    }
    if above < GATE_MIN_CELLS {
        why.push(format!(
            "{above} of 9 development cells have a profit factor above 1.0 (at least {GATE_MIN_CELLS} needed)"
        ));
    }
    if why.is_empty() {
        (
            true,
            format!("development gate met: default rules 1-3, {above} of 9 cells PF > 1.0"),
        )
    } else {
        (false, format!("no demonstrated edge: {}", why.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_as_written() {
        let pass = Rules::evaluate(
            120,
            Some(0.2),
            Some(0.01),
            Some(0.1),
            Some(1.3),
            H1_MIN_TRADES,
        );
        assert!(pass.rules_1_to_3() && pass.rule4_min_trades);
        assert_eq!(held_out_verdict(&pass), "PASS");
        // PF exactly 1.3 passes (≥); mean equal to the control p95 fails (>).
        let at_p95 = Rules::evaluate(120, Some(0.1), Some(0.01), Some(0.1), Some(2.0), 80);
        assert!(!at_p95.rule2_above_control_p95);
        assert_eq!(held_out_verdict(&at_p95), "FAIL");
        // Held-out rule 2: both means above the p95, each alone fails it.
        let both = pass.also_above_control_p95(Some(0.15), Some(0.1));
        assert!(both.rule2_above_control_p95);
        assert!(
            !pass
                .also_above_control_p95(Some(0.1), Some(0.1))
                .rule2_above_control_p95
        );
        assert!(
            !pass
                .also_above_control_p95(None, Some(0.1))
                .rule2_above_control_p95
        );
        assert!(
            !at_p95
                .also_above_control_p95(Some(0.5), Some(0.1))
                .rule2_above_control_p95
        );
        let few = Rules::evaluate(79, Some(0.2), Some(0.01), Some(0.1), Some(2.0), 80);
        assert_eq!(held_out_verdict(&few), "insufficient evidence");
        let zero_lb = Rules::evaluate(200, Some(0.2), Some(0.0), Some(0.1), Some(2.0), 80);
        assert!(!zero_lb.rule1_bootstrap_lower_positive);
        let six = [
            Some(1.1),
            Some(1.2),
            Some(0.9),
            Some(1.01),
            Some(1.5),
            Some(1.0),
            Some(2.0),
            Some(1.3),
            None,
        ];
        let (ok, why) = development_gate(&pass, &six);
        assert!(ok, "{why}");
        // PF exactly 1.0 is not above 1.0: five cells.
        let five = [
            Some(1.1),
            Some(1.2),
            Some(0.9),
            Some(1.01),
            Some(1.5),
            Some(1.0),
            Some(0.5),
            Some(1.3),
            None,
        ];
        let (ok, why) = development_gate(&pass, &five);
        assert!(
            !ok && why.starts_with("no demonstrated edge") && why.contains("5 of 9"),
            "{why}"
        );
        let (ok, _) = development_gate(&few, &six);
        assert!(ok, "rule 4 is not part of the gate");
        let (ok, why) = development_gate(&at_p95, &six);
        assert!(!ok && why.contains("rules 1-3"), "{why}");
    }
}
