//! Settings: every tunable number lives in the database, not in the code.
//!
//! Each setting has a definition (key, kind, bounds, description) and a value
//! at global scope, optionally overridden per account. A change outside the
//! bounds is refused with its reason; nothing is clamped. Every change bumps
//! the settings version and is logged with old and new values (see
//! `jiaoyi-store`). This module holds the pure rules; storage is elsewhere.
//!
//! The account allowlist is NOT a setting: it is an identity boundary and stays
//! in the environment (`JIAOYI_ALLOWED_ACCOUNTS`).

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::universe::{parse_universe, AssetClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Integer,
    Number,
    SymbolList,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s {
            "integer" => Kind::Integer,
            "number" => Kind::Number,
            "symbol_list" => Kind::SymbolList,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Definition {
    pub key: String,
    pub kind: Kind,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub description: String,
}

/// Check a proposed value against its definition. Refuses, never clamps.
pub fn validate(def: &Definition, v: &Value) -> Result<(), String> {
    let num = match def.kind {
        Kind::Integer => {
            let Some(i) = v.as_i64() else {
                return Err(format!("{}: {v} is not an integer", def.key));
            };
            i as f64
        }
        Kind::Number => {
            let Some(f) = v.as_f64().filter(|f| f.is_finite()) else {
                return Err(format!("{}: {v} is not a finite number", def.key));
            };
            f
        }
        Kind::SymbolList => {
            return parse_universe(v)
                .map(|_| ())
                .map_err(|e| format!("{}: {e}", def.key))
        }
    };
    if let Some(min) = def.min {
        if num < min {
            return Err(format!("{}: {num} is below the minimum {min}", def.key));
        }
    }
    if let Some(max) = def.max {
        if num > max {
            return Err(format!("{}: {num} is above the maximum {max}", def.key));
        }
    }
    Ok(())
}

/// Where a value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope", content = "account_id")]
pub enum Scope {
    Global,
    Account(i64),
}

/// The settings in force for one account (or globally), at one version.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub version: i64,
    pub values: BTreeMap<String, (Value, Scope)>,
}

impl Snapshot {
    /// Merge global values with an account's overrides; the account wins.
    pub fn merge(
        version: i64,
        global: Vec<(String, Value)>,
        account: Vec<(String, Value, i64)>,
    ) -> Snapshot {
        let mut values: BTreeMap<String, (Value, Scope)> = global
            .into_iter()
            .map(|(k, v)| (k, (v, Scope::Global)))
            .collect();
        for (k, v, id) in account {
            values.insert(k, (v, Scope::Account(id)));
        }
        Snapshot { version, values }
    }

    fn get(&self, key: &str) -> Result<&Value, String> {
        self.values
            .get(key)
            .map(|(v, _)| v)
            .ok_or_else(|| format!("setting {key} has no value"))
    }

    pub fn integer(&self, key: &str) -> Result<i64, String> {
        let v = self.get(key)?;
        v.as_i64()
            .ok_or_else(|| format!("setting {key} = {v} is not an integer"))
    }

    pub fn number(&self, key: &str) -> Result<f64, String> {
        let v = self.get(key)?;
        v.as_f64()
            .ok_or_else(|| format!("setting {key} = {v} is not a number"))
    }

    pub fn universe(&self, key: &str) -> Result<Vec<(String, AssetClass)>, String> {
        parse_universe(self.get(key)?).map_err(|e| format!("setting {key}: {e}"))
    }

    pub fn scope_of(&self, key: &str) -> Option<Scope> {
        self.values.get(key).map(|(_, s)| *s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn def(kind: Kind, min: Option<f64>, max: Option<f64>) -> Definition {
        Definition {
            key: "k".into(),
            kind,
            min,
            max,
            description: String::new(),
        }
    }

    #[test]
    fn out_of_bounds_values_are_refused_with_the_reason_never_clamped() {
        let d = def(Kind::Integer, Some(1.0), Some(3650.0));
        assert!(validate(&d, &json!(30)).is_ok());
        assert_eq!(
            validate(&d, &json!(0)).unwrap_err(),
            "k: 0 is below the minimum 1"
        );
        assert_eq!(
            validate(&d, &json!(4000)).unwrap_err(),
            "k: 4000 is above the maximum 3650"
        );
        assert!(validate(&d, &json!(2.5))
            .unwrap_err()
            .contains("not an integer"));
        assert!(validate(&d, &json!("30"))
            .unwrap_err()
            .contains("not an integer"));
    }

    #[test]
    fn numbers_and_lists_are_checked() {
        let d = def(Kind::Number, Some(0.0), Some(1.0));
        assert!(validate(&d, &json!(0.1)).is_ok());
        assert!(validate(&d, &json!(1.01)).is_err());
        let l = def(Kind::SymbolList, None, None);
        assert!(validate(&l, &json!([{"name": "EURUSD", "class": "fx"}])).is_ok());
        assert!(validate(&l, &json!([])).is_err());
    }

    #[test]
    fn account_value_overrides_the_global_one() {
        let s = Snapshot::merge(
            7,
            vec![("a".into(), json!(30)), ("b".into(), json!(1.5))],
            vec![("a".into(), json!(10), 49236051)],
        );
        assert_eq!(s.integer("a").unwrap(), 10);
        assert_eq!(s.scope_of("a"), Some(Scope::Account(49236051)));
        assert_eq!(s.number("b").unwrap(), 1.5);
        assert_eq!(s.scope_of("b"), Some(Scope::Global));
        assert_eq!(s.version, 7);
        assert!(s.integer("missing").unwrap_err().contains("no value"));
    }
}
