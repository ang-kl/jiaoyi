//! The symbol universe and its resolution on an account.
//!
//! The universe itself is the `universe.symbols` setting (seeded by migration
//! with the 20 candidates of rulebook §3); nothing here hard-codes it.
//!
//! Resolution is an exact, case-sensitive match of the broker's `symbolName`.
//! A name that is missing, or that matches more than one symbol, is recorded
//! with its reason. Nothing is aliased, trimmed, upper-cased or guessed.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    EquityIndex,
    Fx,
    Metal,
    Energy,
    Crypto,
}

impl AssetClass {
    pub fn as_str(self) -> &'static str {
        match self {
            AssetClass::EquityIndex => "equity_index",
            AssetClass::Fx => "fx",
            AssetClass::Metal => "metal",
            AssetClass::Energy => "energy",
            AssetClass::Crypto => "crypto",
        }
    }

    pub fn parse(s: &str) -> Option<AssetClass> {
        Some(match s {
            "equity_index" => AssetClass::EquityIndex,
            "fx" => AssetClass::Fx,
            "metal" => AssetClass::Metal,
            "energy" => AssetClass::Energy,
            "crypto" => AssetClass::Crypto,
            _ => return None,
        })
    }
}

/// One universe entry: an exact broker symbol name and its asset class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct UniverseEntry {
    pub name: String,
    #[serde(rename = "class")]
    pub class_name: String,
}

impl UniverseEntry {
    pub fn class(&self) -> Option<AssetClass> {
        AssetClass::parse(&self.class_name)
    }
}

/// Parse and check a `universe.symbols` value: a JSON array of
/// `{"name": ..., "class": ...}` with known classes and no duplicate or blank names.
pub fn parse_universe(v: &serde_json::Value) -> Result<Vec<(String, AssetClass)>, String> {
    let entries: Vec<UniverseEntry> = serde_json::from_value(v.clone())
        .map_err(|e| format!("not a list of {{name, class}}: {e}"))?;
    let mut out: Vec<(String, AssetClass)> = Vec::with_capacity(entries.len());
    for e in entries {
        if e.name.is_empty() || e.name.trim() != e.name {
            return Err(format!("symbol name {:?} is blank or padded", e.name));
        }
        let class = e
            .class()
            .ok_or_else(|| format!("{}: unknown class {:?}", e.name, e.class_name))?;
        if out.iter().any(|(n, _)| *n == e.name) {
            return Err(format!("{} is listed twice", e.name));
        }
        out.push((e.name, class));
    }
    if out.is_empty() {
        return Err("the universe is empty".into());
    }
    Ok(out)
}

/// What the broker's symbol list says about one symbol (from `ProtoOALightSymbol`).
#[derive(Debug, Clone, PartialEq)]
pub struct ListedSymbol {
    pub symbol_id: i64,
    /// `symbolName`; `None` when the broker sent none.
    pub name: Option<String>,
    /// `enabled`; `None` when the broker sent none.
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Resolution {
    /// Exactly one symbol carries this exact name.
    Resolved { symbol_id: i64 },
    /// No symbol carries this exact name.
    Missing { reason: String },
    /// More than one symbol carries this exact name; none is chosen.
    Ambiguous {
        symbol_ids: Vec<i64>,
        reason: String,
    },
}

impl Resolution {
    pub fn status(&self) -> &'static str {
        match self {
            Resolution::Resolved { .. } => "resolved",
            Resolution::Missing { .. } => "missing",
            Resolution::Ambiguous { .. } => "ambiguous",
        }
    }

    pub fn symbol_id(&self) -> Option<i64> {
        match self {
            Resolution::Resolved { symbol_id } => Some(*symbol_id),
            _ => None,
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Resolution::Resolved { .. } => None,
            Resolution::Missing { reason } | Resolution::Ambiguous { reason, .. } => Some(reason),
        }
    }
}

/// Words that mark a broker symbol name as a possible energy instrument
/// (owner ruling, 24-10-2026: report the broker's own names). Matched case
/// insensitively anywhere in the name; read-only, nothing is resolved by it.
pub const ENERGY_NAME_PARTS: [&str; 7] = ["crude", "brent", "oil", "wti", "gas", "xti", "xbr"];

pub fn is_energy_candidate(name: &str) -> bool {
    let n = name.to_lowercase();
    ENERGY_NAME_PARTS.iter().any(|p| n.contains(p))
}

/// Resolve one universe name against an account's symbol list.
pub fn resolve(name: &str, listed: &[ListedSymbol]) -> Resolution {
    let mut ids: Vec<i64> = listed
        .iter()
        .filter(|s| s.name.as_deref() == Some(name))
        .map(|s| s.symbol_id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    match ids.as_slice() {
        [] => {
            // Say so if a differently-cased or padded name exists, so the owner can
            // decide. It is still a miss: nothing is aliased.
            let near: Vec<&str> = listed
                .iter()
                .filter_map(|s| s.name.as_deref())
                .filter(|n| n.trim().eq_ignore_ascii_case(name) && *n != name)
                .collect();
            let reason = if near.is_empty() {
                format!("no symbol named exactly {name:?} on this account")
            } else {
                format!(
                    "no symbol named exactly {name:?} on this account; near names not used: {near:?}"
                )
            };
            Resolution::Missing { reason }
        }
        [id] => Resolution::Resolved { symbol_id: *id },
        many => Resolution::Ambiguous {
            symbol_ids: many.to_vec(),
            reason: format!(
                "{} symbols named exactly {name:?} on this account (ids {many:?}); none chosen",
                many.len()
            ),
        },
    }
}

/// Resolve the whole universe. The result keeps the universe's order.
pub fn resolve_universe(
    universe: &[(String, AssetClass)],
    listed: &[ListedSymbol],
) -> Vec<(String, AssetClass, Resolution)> {
    universe
        .iter()
        .map(|(name, class)| (name.clone(), *class, resolve(name, listed)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(id: i64, name: &str) -> ListedSymbol {
        ListedSymbol {
            symbol_id: id,
            name: Some(name.to_string()),
            enabled: Some(true),
        }
    }

    #[test]
    fn parse_universe_accepts_a_good_list_and_refuses_bad_ones() {
        use serde_json::json;
        let ok = parse_universe(&json!([
            {"name": "US500", "class": "equity_index"},
            {"name": "BTCUSD", "class": "crypto"}
        ]))
        .unwrap();
        assert_eq!(
            ok,
            vec![
                ("US500".into(), AssetClass::EquityIndex),
                ("BTCUSD".into(), AssetClass::Crypto)
            ]
        );
        assert!(parse_universe(&json!([])).unwrap_err().contains("empty"));
        assert!(parse_universe(&json!([{"name": "X", "class": "bond"}]))
            .unwrap_err()
            .contains("unknown class"));
        assert!(parse_universe(&json!([{"name": " X", "class": "fx"}]))
            .unwrap_err()
            .contains("padded"));
        assert!(parse_universe(&json!([
            {"name": "X", "class": "fx"}, {"name": "X", "class": "fx"}
        ]))
        .unwrap_err()
        .contains("twice"));
    }

    #[test]
    fn energy_candidates_match_case_insensitively() {
        for n in [
            "SpotCrude",
            "NatGas",
            "UKOIL",
            "Brent",
            "XTIUSD",
            "XBRUSD",
            "WTI.c",
            "gasoline",
        ] {
            assert!(is_energy_candidate(n), "{n}");
        }
        for n in ["EURUSD", "US500", "XAUUSD", "GER40"] {
            assert!(!is_energy_candidate(n), "{n}");
        }
    }

    #[test]
    fn exact_name_resolves() {
        let listed = vec![sym(1, "EURUSD"), sym(2, "GBPUSD")];
        assert_eq!(
            resolve("EURUSD", &listed),
            Resolution::Resolved { symbol_id: 1 }
        );
    }

    #[test]
    fn missing_name_is_recorded_with_its_reason_and_not_aliased() {
        // "EUR/USD" and "eurusd" are near names. Neither is used.
        let listed = vec![sym(1, "EUR/USD"), sym(2, "eurusd"), sym(3, "GBPUSD")];
        let r = resolve("EURUSD", &listed);
        assert_eq!(r.status(), "missing");
        assert_eq!(r.symbol_id(), None);
        let reason = r.reason().unwrap();
        assert!(
            reason.contains("no symbol named exactly \"EURUSD\""),
            "{reason}"
        );
        assert!(reason.contains("eurusd"), "{reason}");
    }

    #[test]
    fn ambiguous_name_is_recorded_and_none_chosen() {
        let listed = vec![sym(7, "XAUUSD"), sym(9, "XAUUSD")];
        let r = resolve("XAUUSD", &listed);
        assert_eq!(
            r,
            Resolution::Ambiguous {
                symbol_ids: vec![7, 9],
                reason:
                    "2 symbols named exactly \"XAUUSD\" on this account (ids [7, 9]); none chosen"
                        .into()
            }
        );
        assert_eq!(r.symbol_id(), None);
    }

    #[test]
    fn symbol_without_a_name_never_matches() {
        let listed = vec![ListedSymbol {
            symbol_id: 5,
            name: None,
            enabled: Some(true),
        }];
        assert_eq!(resolve("US500", &listed).status(), "missing");
    }

    #[test]
    fn resolve_universe_keeps_order_and_reports_every_name() {
        let universe = vec![
            ("US500".to_string(), AssetClass::EquityIndex),
            ("UKOIL".to_string(), AssetClass::Energy),
            ("ETHUSD".to_string(), AssetClass::Crypto),
        ];
        let listed = vec![sym(10, "US500"), sym(11, "ETHUSD")];
        let all = resolve_universe(&universe, &listed);
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].2.symbol_id(), Some(10));
        assert_eq!(all[1].2.status(), "missing");
        assert_eq!(all[2].2.symbol_id(), Some(11));
    }
}
