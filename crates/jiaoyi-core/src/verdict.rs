//! The phase-1 keep/drop checks of rulebook §3.
//!
//! A symbol is dropped when it fails any check:
//! - not tradable on the account;
//! - less daily history at the broker than the minimum (rulebook: 10 years;
//!   crypto: 6 years);
//! - median spread greater than a fraction of one daily ATR (rulebook: 10% of ATR(20)).
//!
//! The thresholds are settings ([`Thresholds`]), read at each report.
//!
//! A check whose input is unknown is neither passed nor failed. If nothing
//! failed but something is unknown, the verdict is `undecided`, never `keep`.

use serde::Serialize;

use crate::universe::AssetClass;

/// The verdict thresholds, from the settings
/// `verdict.min_history_years`, `verdict.min_history_years_crypto` and
/// `verdict.max_spread_to_atr`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    pub min_history_years: f64,
    pub min_history_years_crypto: f64,
    pub max_spread_to_atr: f64,
}

impl Thresholds {
    fn min_years(&self, class: AssetClass) -> f64 {
        match class {
            AssetClass::Crypto => self.min_history_years_crypto,
            _ => self.min_history_years,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Keep,
    Drop,
    Undecided,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Verdict {
    pub decision: Decision,
    /// Every failed or unknown check, in the rulebook's order. Empty for `keep`.
    pub reasons: Vec<String>,
}

/// What is known about one universe name on one account.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Evidence {
    /// `None` when the name did not resolve; carries the resolution reason.
    pub unresolved_reason: Option<String>,
    /// `Some(true)` tradable, `Some(false)` not, `None` unknown.
    pub tradable: Option<bool>,
    pub not_tradable_reason: Option<String>,
    /// Years between the first and last closed daily bar. `None` if no bars.
    pub history_years: Option<f64>,
    /// Latest ATR. `None` with too few bars.
    pub atr20: Option<f64>,
    /// Median spread in price units. `None` if never measured.
    pub spread_median: Option<f64>,
    pub spread_missing_reason: Option<String>,
}

pub fn judge(class: AssetClass, e: &Evidence, t: &Thresholds) -> Verdict {
    if let Some(reason) = &e.unresolved_reason {
        return Verdict {
            decision: Decision::Drop,
            reasons: vec![format!("not on the account: {reason}")],
        };
    }
    let mut failed = Vec::new();
    let mut unknown = Vec::new();

    match e.tradable {
        Some(true) => {}
        Some(false) => failed.push(format!(
            "not tradable: {}",
            e.not_tradable_reason
                .as_deref()
                .unwrap_or("the broker marks it not tradable")
        )),
        None => unknown.push("tradability unknown".to_string()),
    }

    let need = t.min_years(class);
    match e.history_years {
        Some(y) if y >= need => {}
        Some(y) => failed.push(format!("{y:.2} years of daily history, needs {need:.0}")),
        None => failed.push(format!("no daily history, needs {need:.0} years")),
    }

    match (e.spread_median, e.atr20) {
        (Some(s), Some(a)) if a > 0.0 => {
            let ratio = s / a;
            if ratio > t.max_spread_to_atr {
                failed.push(format!(
                    "median spread is {:.2}% of ATR, limit {:.2}%",
                    ratio * 100.0,
                    t.max_spread_to_atr * 100.0
                ));
            }
        }
        (Some(_), Some(_)) => unknown.push("ATR is zero; spread ratio undefined".into()),
        (None, _) => unknown.push(format!(
            "median spread unknown: {}",
            e.spread_missing_reason.as_deref().unwrap_or("not measured")
        )),
        (Some(_), None) => unknown.push("ATR unknown: too few closed bars".into()),
    }

    if !failed.is_empty() {
        failed.extend(unknown);
        Verdict {
            decision: Decision::Drop,
            reasons: failed,
        }
    } else if !unknown.is_empty() {
        Verdict {
            decision: Decision::Undecided,
            reasons: unknown,
        }
    } else {
        Verdict {
            decision: Decision::Keep,
            reasons: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rulebook §3 values, as seeded by the migration.
    const T: Thresholds = Thresholds {
        min_history_years: 10.0,
        min_history_years_crypto: 6.0,
        max_spread_to_atr: 0.10,
    };

    fn good() -> Evidence {
        Evidence {
            tradable: Some(true),
            history_years: Some(15.0),
            atr20: Some(0.0100),
            spread_median: Some(0.0001),
            ..Default::default()
        }
    }

    #[test]
    fn all_checks_pass_keeps() {
        let v = judge(AssetClass::Fx, &good(), &T);
        assert_eq!(v.decision, Decision::Keep);
        assert!(v.reasons.is_empty());
    }

    #[test]
    fn unresolved_drops_with_reason() {
        let e = Evidence {
            unresolved_reason: Some("no symbol named exactly \"UKOIL\"".into()),
            ..Default::default()
        };
        let v = judge(AssetClass::Energy, &e, &T);
        assert_eq!(v.decision, Decision::Drop);
        assert!(v.reasons[0].contains("UKOIL"));
    }

    #[test]
    fn short_history_drops_and_crypto_needs_six_years() {
        let mut e = good();
        e.history_years = Some(9.5);
        assert_eq!(judge(AssetClass::Fx, &e, &T).decision, Decision::Drop);
        assert_eq!(judge(AssetClass::Crypto, &e, &T).decision, Decision::Keep);
        e.history_years = Some(5.9);
        let v = judge(AssetClass::Crypto, &e, &T);
        assert_eq!(v.decision, Decision::Drop);
        assert!(v.reasons[0].contains("needs 6"), "{:?}", v.reasons);
    }

    #[test]
    fn spread_above_ten_percent_of_atr_drops_and_exactly_ten_keeps() {
        let mut e = good();
        e.atr20 = Some(1.0);
        e.spread_median = Some(0.10);
        assert_eq!(judge(AssetClass::Metal, &e, &T).decision, Decision::Keep);
        e.spread_median = Some(0.1001);
        let v = judge(AssetClass::Metal, &e, &T);
        assert_eq!(v.decision, Decision::Drop);
        assert!(
            v.reasons[0].contains("10.01% of ATR, limit 10.00%"),
            "{:?}",
            v.reasons
        );
    }

    #[test]
    fn unknown_spread_is_undecided_never_keep() {
        let mut e = good();
        e.spread_median = None;
        e.spread_missing_reason = Some("no quotes in the 60 s window".into());
        let v = judge(AssetClass::Fx, &e, &T);
        assert_eq!(v.decision, Decision::Undecided);
        assert!(v.reasons[0].contains("no quotes"));
    }

    #[test]
    fn not_tradable_drops() {
        let mut e = good();
        e.tradable = Some(false);
        e.not_tradable_reason = Some("tradingMode CLOSE_ONLY_MODE".into());
        let v = judge(AssetClass::EquityIndex, &e, &T);
        assert_eq!(v.decision, Decision::Drop);
        assert!(v.reasons[0].contains("CLOSE_ONLY_MODE"));
    }
}
