//! The account allowlist, rulebook §2.
//!
//! `JIAOYI_ALLOWED_ACCOUNTS=49236051,43002148` names the cTrader trader-account
//! ids (`ctidTraderAccountId`) this program may touch. A missing or empty list
//! means no account is permitted, and the service refuses to start.
//!
//! The only way to obtain a [`PermittedAccount`] is [`Allowlist::permit`]. The
//! cTrader client's account authorisation takes a `&PermittedAccount`, so an
//! account that is not on the list cannot be authorised, and therefore cannot
//! be read, through any code path.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

/// The environment variable that holds the allowlist.
pub const ALLOWLIST_VAR: &str = "JIAOYI_ALLOWED_ACCOUNTS";
/// The environment variable that says which cTrader environment each account
/// lives on, e.g. `49236051:demo,43002148:live`.
pub const ENVIRONMENTS_VAR: &str = "JIAOYI_ACCOUNT_ENVIRONMENTS";

/// Why the allowlist could not be built. Each one stops start-up.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AllowlistError {
    #[error("{ALLOWLIST_VAR} is not set: no account is permitted, refusing to start")]
    Missing,
    #[error("{ALLOWLIST_VAR} is empty: no account is permitted, refusing to start")]
    Empty,
    #[error("{ALLOWLIST_VAR} entry {0:?} is not a positive integer account id")]
    Invalid(String),
}

/// The set of permitted cTrader trader-account ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allowlist {
    ids: BTreeSet<i64>,
}

/// An account that passed the allowlist. It cannot be built any other way.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct PermittedAccount {
    id: i64,
}

impl PermittedAccount {
    /// The cTrader `ctidTraderAccountId`.
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl fmt::Display for PermittedAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// An account that was refused, with the reason to record.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("account {account_id} refused: {reason}")]
pub struct Refusal {
    pub account_id: String,
    pub reason: String,
}

impl Allowlist {
    /// Parse the raw value of `JIAOYI_ALLOWED_ACCOUNTS` (`None` when unset).
    pub fn parse(raw: Option<&str>) -> Result<Self, AllowlistError> {
        let raw = raw.ok_or(AllowlistError::Missing)?;
        let mut ids = BTreeSet::new();
        for part in raw.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            match part.parse::<i64>() {
                Ok(id) if id > 0 => {
                    ids.insert(id);
                }
                _ => return Err(AllowlistError::Invalid(part.to_string())),
            }
        }
        if ids.is_empty() {
            return Err(AllowlistError::Empty);
        }
        Ok(Allowlist { ids })
    }

    /// Read the allowlist from the process environment.
    pub fn from_env() -> Result<Self, AllowlistError> {
        let raw = std::env::var(ALLOWLIST_VAR).ok();
        Self::parse(raw.as_deref())
    }

    /// The single gate. An id not on the list is refused.
    pub fn permit(&self, account_id: i64) -> Result<PermittedAccount, Refusal> {
        if self.ids.contains(&account_id) {
            Ok(PermittedAccount { id: account_id })
        } else {
            Err(Refusal {
                account_id: account_id.to_string(),
                reason: format!("not on {ALLOWLIST_VAR}"),
            })
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = i64> + '_ {
        self.ids.iter().copied()
    }
}

/// Which cTrader server an account lives on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Demo,
    Live,
}

impl Environment {
    pub fn as_str(self) -> &'static str {
        match self {
            Environment::Demo => "demo",
            Environment::Live => "live",
        }
    }
}

/// One entry of `JIAOYI_ACCOUNT_ENVIRONMENTS`, before the allowlist is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredAccount {
    pub account_id: i64,
    pub environment: Environment,
}

/// The outcome of matching the declared environments against the allowlist.
#[derive(Debug, Clone, Default)]
pub struct AccountPlan {
    /// Permitted accounts with a declared environment: these are loaded.
    pub permitted: Vec<(PermittedAccount, Environment)>,
    /// Everything refused, with its reason. Each one is recorded.
    pub refused: Vec<Refusal>,
}

/// Build the per-account plan.
///
/// - an entry that does not parse is refused;
/// - a declared account not on the allowlist is refused;
/// - an allowlisted account with no declared environment is refused (its
///   server is not guessed).
pub fn plan_accounts(allowlist: &Allowlist, environments_raw: Option<&str>) -> AccountPlan {
    let mut plan = AccountPlan::default();
    let mut declared: Vec<DeclaredAccount> = Vec::new();
    for part in environments_raw.unwrap_or("").split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let parsed = part.split_once(':').and_then(|(id, env)| {
            let id = id.trim().parse::<i64>().ok().filter(|id| *id > 0)?;
            let env = match env.trim().to_ascii_lowercase().as_str() {
                "demo" => Environment::Demo,
                "live" => Environment::Live,
                _ => return None,
            };
            Some(DeclaredAccount {
                account_id: id,
                environment: env,
            })
        });
        match parsed {
            Some(d) => declared.push(d),
            None => plan.refused.push(Refusal {
                account_id: part.to_string(),
                reason: format!("{ENVIRONMENTS_VAR} entry is not <account id>:demo|live"),
            }),
        }
    }

    for d in &declared {
        if declared
            .iter()
            .filter(|o| o.account_id == d.account_id)
            .any(|o| o.environment != d.environment)
        {
            if !plan
                .refused
                .iter()
                .any(|r| r.account_id == d.account_id.to_string())
            {
                plan.refused.push(Refusal {
                    account_id: d.account_id.to_string(),
                    reason: format!("{ENVIRONMENTS_VAR} names both demo and live for this account"),
                });
            }
            continue;
        }
        match allowlist.permit(d.account_id) {
            Ok(p) => {
                if !plan.permitted.iter().any(|(q, _)| *q == p) {
                    plan.permitted.push((p, d.environment));
                }
            }
            Err(r) => plan.refused.push(r),
        }
    }

    for id in allowlist.ids() {
        let declared_here = declared.iter().any(|d| d.account_id == id);
        if !declared_here {
            plan.refused.push(Refusal {
                account_id: id.to_string(),
                reason: format!(
                    "no environment declared in {ENVIRONMENTS_VAR}; the server is not guessed"
                ),
            });
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_allowlist_refuses_start() {
        assert_eq!(Allowlist::parse(None), Err(AllowlistError::Missing));
    }

    #[test]
    fn empty_allowlist_refuses_start() {
        assert_eq!(Allowlist::parse(Some("")), Err(AllowlistError::Empty));
        assert_eq!(Allowlist::parse(Some(" , ,")), Err(AllowlistError::Empty));
    }

    #[test]
    fn malformed_allowlist_entry_refuses_start() {
        assert_eq!(
            Allowlist::parse(Some("49236051,abc")),
            Err(AllowlistError::Invalid("abc".into()))
        );
        assert_eq!(
            Allowlist::parse(Some("-5")),
            Err(AllowlistError::Invalid("-5".into()))
        );
    }

    #[test]
    fn account_on_allowlist_is_permitted() {
        let list = Allowlist::parse(Some("49236051, 43002148")).unwrap();
        assert_eq!(list.permit(49236051).unwrap().id(), 49236051);
        assert_eq!(list.permit(43002148).unwrap().id(), 43002148);
    }

    #[test]
    fn account_not_on_allowlist_is_refused() {
        let list = Allowlist::parse(Some("49236051,43002148")).unwrap();
        let refusal = list.permit(12345678).unwrap_err();
        assert_eq!(refusal.account_id, "12345678");
        assert!(refusal.reason.contains(ALLOWLIST_VAR), "{}", refusal.reason);
        // A near miss is still a miss.
        assert!(list.permit(4923605).is_err());
    }

    #[test]
    fn plan_refuses_and_records_undeclared_and_unlisted_accounts() {
        let list = Allowlist::parse(Some("49236051,43002148")).unwrap();
        let plan = plan_accounts(&list, Some("49236051:demo,99999999:live,garbage"));
        assert_eq!(plan.permitted.len(), 1);
        assert_eq!(plan.permitted[0].0.id(), 49236051);
        assert_eq!(plan.permitted[0].1, Environment::Demo);
        let refused: Vec<_> = plan.refused.iter().map(|r| r.account_id.as_str()).collect();
        assert!(refused.contains(&"99999999"), "{refused:?}");
        assert!(refused.contains(&"garbage"), "{refused:?}");
        assert!(refused.contains(&"43002148"), "{refused:?}");
    }

    #[test]
    fn plan_refuses_contradictory_environment() {
        let list = Allowlist::parse(Some("49236051")).unwrap();
        let plan = plan_accounts(&list, Some("49236051:demo,49236051:live"));
        assert!(plan.permitted.is_empty());
        assert!(plan.refused.iter().any(|r| r.reason.contains("both")));
    }
}
