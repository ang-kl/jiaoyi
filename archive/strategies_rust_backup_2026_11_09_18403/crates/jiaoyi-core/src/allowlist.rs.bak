//! The account allowlist, rulebook §2.
//!
//! `JIAOYI_ALLOWED_ACCOUNTS=49236051,43002148` names the cTrader trader-account
//! ids (`ctidTraderAccountId`) this program may touch. A missing or empty list
//! means no account is permitted, and the service refuses to start.
//!
//! Demo accounts added by the owner on 25-10-2026 ("Code reads the new
//! variables"): each `JIAOYI_DEMO_ACCOUNT_ID` and `JIAOYI_DEMO_ACCOUNT_ID_<N>`
//! variable (N a positive integer) names one more `ctidTraderAccountId`,
//! permitted and declared demo, in addition to the pair above, which stays as
//! it was. A malformed variable is refused and recorded under its name; it
//! does not stop start-up. An account named only by such a variable does not
//! download tick history (owner, 25-10-2026: "Skip their tick history").
//!
//! The only way to obtain a [`PermittedAccount`] is [`Allowlist::permit`] (or
//! the plan, for an account a demo variable names). The cTrader client's
//! account authorisation takes a `&PermittedAccount`, so an account that is
//! not on the list cannot be authorised, and therefore cannot be read, through
//! any code path.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;

/// The environment variable that holds the allowlist.
pub const ALLOWLIST_VAR: &str = "JIAOYI_ALLOWED_ACCOUNTS";
/// The environment variable that says which cTrader environment each account
/// lives on, e.g. `49236051:demo,43002148:live`.
pub const ENVIRONMENTS_VAR: &str = "JIAOYI_ACCOUNT_ENVIRONMENTS";
/// The variables that each add one demo account: this name alone, or this
/// name followed by `_<N>`, N a positive integer (owner, 25-10-2026).
pub const DEMO_ACCOUNT_VAR: &str = "JIAOYI_DEMO_ACCOUNT_ID";

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
    /// Accounts named by `JIAOYI_DEMO_ACCOUNT_ID*` variables.
    demo: DemoAccounts,
}

/// The accounts named by `JIAOYI_DEMO_ACCOUNT_ID*` variables, read by
/// [`demo_accounts`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DemoAccounts {
    /// Each account id with the variables that name it, in variable order
    /// (the unsuffixed name, then `_1`, `_2`, ...). Duplicates collapse here.
    pub accounts: BTreeMap<i64, Vec<String>>,
    /// Variables refused, in variable name order: a name that is not the
    /// pattern, or a value that is not a positive integer. Each is recorded;
    /// none stops start-up.
    pub refused: Vec<Refusal>,
}

/// The position of a demo account variable: 0 for the unsuffixed name, N for
/// `_<N>`. `None` when the name carries the prefix but is not the pattern
/// (`_0`, `_X`, `_`, `S`, ...).
fn demo_var_index(name: &str) -> Option<u64> {
    let rest = name.strip_prefix(DEMO_ACCOUNT_VAR)?;
    if rest.is_empty() {
        return Some(0);
    }
    let n = rest.strip_prefix('_')?;
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    n.parse::<u64>().ok().filter(|n| *n > 0)
}

/// A refused value as recorded: trimmed and cut to 40 characters.
fn shown(value: &str) -> String {
    let v = value.trim();
    if v.chars().count() > 40 {
        format!("{}…", v.chars().take(40).collect::<String>())
    } else {
        v.to_string()
    }
}

/// Read the demo account variables from `(name, value)` pairs, e.g. the
/// process environment. Only names that start with [`DEMO_ACCOUNT_VAR`] are
/// looked at; every other pair is skipped unread.
///
/// - `JIAOYI_DEMO_ACCOUNT_ID` and `JIAOYI_DEMO_ACCOUNT_ID_<N>` (N a positive
///   integer, any count) each name one account;
/// - a name with the prefix that is not that pattern (`_0`, `_X`,
///   `JIAOYI_DEMO_ACCOUNT_IDS`) is refused, so a typo is recorded, not lost;
/// - a value that is not a positive integer (empty, `abc`, `0`, `-5`, a
///   list) is refused with the variable's name as the reason;
/// - two variables naming one account collapse to one account.
pub fn demo_accounts<I, K, V>(vars: I) -> DemoAccounts
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut named: Vec<(u64, String, String)> = Vec::new();
    let mut refused: Vec<(String, Refusal)> = Vec::new();
    for (k, v) in vars {
        let name = k.as_ref();
        if !name.starts_with(DEMO_ACCOUNT_VAR) {
            continue;
        }
        let value = v.as_ref();
        match demo_var_index(name) {
            Some(i) => named.push((i, name.to_string(), value.to_string())),
            None => refused.push((
                name.to_string(),
                Refusal {
                    account_id: shown(value),
                    reason: format!(
                        "{name} is not {DEMO_ACCOUNT_VAR} or {DEMO_ACCOUNT_VAR}_<N> with N a positive integer; not read"
                    ),
                },
            )),
        }
    }
    named.sort();
    let mut out = DemoAccounts::default();
    for (_, name, value) in named {
        match value.trim().parse::<i64>() {
            Ok(id) if id > 0 => out.accounts.entry(id).or_default().push(name),
            _ => refused.push((
                name.clone(),
                Refusal {
                    account_id: shown(&value),
                    reason: format!("{name} is not a positive integer account id"),
                },
            )),
        }
    }
    refused.sort_by(|a, b| a.0.cmp(&b.0));
    out.refused = refused.into_iter().map(|(_, r)| r).collect();
    out
}

/// Whether an account downloads tick history (history sync step 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum TickHistory {
    On,
    Off { reason: String },
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
        Ok(Allowlist {
            ids,
            demo: DemoAccounts::default(),
        })
    }

    /// Add the accounts the demo variables name ([`demo_accounts`]). The
    /// list parsed from `JIAOYI_ALLOWED_ACCOUNTS` is unchanged.
    pub fn with_demo_accounts(mut self, demo: DemoAccounts) -> Self {
        self.demo = demo;
        self
    }

    /// The accounts the demo variables name, and the variables refused.
    pub fn demo_accounts(&self) -> &DemoAccounts {
        &self.demo
    }

    /// Read the allowlist from the process environment.
    pub fn from_env() -> Result<Self, AllowlistError> {
        let raw = std::env::var(ALLOWLIST_VAR).ok();
        Self::parse(raw.as_deref())
    }

    /// The single gate. An id not on the list, and named by no demo
    /// variable, is refused.
    pub fn permit(&self, account_id: i64) -> Result<PermittedAccount, Refusal> {
        if self.ids.contains(&account_id) || self.demo.accounts.contains_key(&account_id) {
            Ok(PermittedAccount { id: account_id })
        } else {
            self.permit_listed(account_id)
        }
    }

    /// The gate of the pair alone: `JIAOYI_ALLOWED_ACCOUNTS`.
    fn permit_listed(&self, account_id: i64) -> Result<PermittedAccount, Refusal> {
        if self.ids.contains(&account_id) {
            Ok(PermittedAccount { id: account_id })
        } else {
            Err(Refusal {
                account_id: account_id.to_string(),
                reason: format!("not on {ALLOWLIST_VAR}"),
            })
        }
    }

    /// The ids on `JIAOYI_ALLOWED_ACCOUNTS` (not the demo variables).
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
    /// Permitted accounts that do not download tick history, with the
    /// reason: those named only by a demo variable (owner, 25-10-2026).
    pub tick_history_off: BTreeMap<i64, String>,
}

impl AccountPlan {
    /// Whether this account downloads tick history (history sync step 3).
    pub fn tick_history(&self, account_id: i64) -> TickHistory {
        match self.tick_history_off.get(&account_id) {
            Some(reason) => TickHistory::Off {
                reason: reason.clone(),
            },
            None => TickHistory::On,
        }
    }
}

/// Build the per-account plan.
///
/// - an entry that does not parse is refused;
/// - a declared account not on the allowlist is refused;
/// - an allowlisted account with no declared environment is refused (its
///   server is not guessed);
/// - an account a demo variable names is permitted as demo, unless
///   `JIAOYI_ACCOUNT_ENVIRONMENTS` declares it live: then the demo variable is
///   refused as contradictory, and the pair's own handling of that account
///   stands (a typo in a demo variable cannot stop a live account);
/// - an account the pair alone does not permit, and a demo variable does,
///   downloads no tick history (`tick_history_off`);
/// - each refused demo variable is recorded.
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

    // The demo variables' accounts, less those the environments declare live.
    let mut demo_claims: BTreeMap<i64, &Vec<String>> = BTreeMap::new();
    for (id, names) in &allowlist.demo.accounts {
        if declared
            .iter()
            .any(|d| d.account_id == *id && d.environment == Environment::Live)
        {
            plan.refused.push(Refusal {
                account_id: id.to_string(),
                reason: format!(
                    "both demo and live for this account: {} names it demo, {ENVIRONMENTS_VAR} live; the demo variable is not read",
                    names.join(", ")
                ),
            });
        } else {
            demo_claims.insert(*id, names);
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
        match allowlist.permit_listed(d.account_id) {
            Ok(p) => {
                if !plan.permitted.iter().any(|(q, _)| *q == p) {
                    plan.permitted.push((p, d.environment));
                }
            }
            // A demo variable permits it (declared demo here): below.
            Err(_) if demo_claims.contains_key(&d.account_id) => {}
            Err(r) => plan.refused.push(r),
        }
    }

    for id in allowlist.ids() {
        let declared_here = declared.iter().any(|d| d.account_id == id);
        // A demo variable declares its environment: below.
        if !declared_here && !demo_claims.contains_key(&id) {
            plan.refused.push(Refusal {
                account_id: id.to_string(),
                reason: format!(
                    "no environment declared in {ENVIRONMENTS_VAR}; the server is not guessed"
                ),
            });
        }
    }

    for (id, names) in demo_claims {
        // Permitted by the pair too (declared demo there): the full set.
        if plan.permitted.iter().any(|(p, _)| p.id() == id) {
            continue;
        }
        plan.permitted
            .push((PermittedAccount { id }, Environment::Demo));
        plan.tick_history_off.insert(
            id,
            format!(
                "named by {} only; tick history download off (owner, 25-10-2026: Postgres disk)",
                names.join(", ")
            ),
        );
    }
    plan.refused.extend(allowlist.demo.refused.iter().cloned());
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

    // Claude · № 16,973 25-Oct (ordered № 16,972·C; demo accounts by variable)
    fn vars(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// The owner's pair as deployed, plus the owner's two demo variables.
    fn owners_plan(demo: &[(&str, &str)]) -> AccountPlan {
        let list = Allowlist::parse(Some("49236051,43002148"))
            .unwrap()
            .with_demo_accounts(demo_accounts(vars(demo)));
        plan_accounts(&list, Some("49236051:demo,43002148:live"))
    }

    #[test]
    fn demo_variables_unsuffixed_and_numbered_each_name_one_account() {
        let d = demo_accounts(vars(&[
            ("JIAOYI_DEMO_ACCOUNT_ID_2", "46130058"),
            ("CTRADER_CLIENT_SECRET", "s3cret-not-read"),
            ("JIAOYI_DEMO_ACCOUNT_ID_3", " 47790949 "),
            ("JIAOYI_DEMO_ACCOUNT_ID", "11111111"),
            ("JIAOYI_DEMO_ACCOUNT_ID_10", "22222222"),
            ("PATH", "/usr/bin"),
        ]));
        assert_eq!(
            d.accounts,
            BTreeMap::from([
                (11111111, vec!["JIAOYI_DEMO_ACCOUNT_ID".to_string()]),
                (22222222, vec!["JIAOYI_DEMO_ACCOUNT_ID_10".to_string()]),
                (46130058, vec!["JIAOYI_DEMO_ACCOUNT_ID_2".to_string()]),
                (47790949, vec!["JIAOYI_DEMO_ACCOUNT_ID_3".to_string()]),
            ])
        );
        assert!(d.refused.is_empty(), "{:?}", d.refused);
        // Other variables are not read: nothing of theirs is kept.
        let kept = format!("{d:?}");
        assert!(
            !kept.contains("s3cret") && !kept.contains("/usr/bin"),
            "{kept}"
        );
    }

    #[test]
    fn a_prefixed_name_that_is_not_the_pattern_is_refused_with_its_name() {
        let names = [
            "JIAOYI_DEMO_ACCOUNT_ID_0",
            "JIAOYI_DEMO_ACCOUNT_ID_X",
            "JIAOYI_DEMO_ACCOUNT_ID_",
            "JIAOYI_DEMO_ACCOUNT_ID_2A",
            "JIAOYI_DEMO_ACCOUNT_IDS",
        ];
        let d = demo_accounts(names.iter().map(|n| (*n, "46130058")));
        assert!(d.accounts.is_empty(), "{:?}", d.accounts);
        assert_eq!(d.refused.len(), names.len(), "{:?}", d.refused);
        for n in names {
            assert!(
                d.refused
                    .iter()
                    .any(|r| r.reason.starts_with(&format!("{n} is not"))
                        && r.account_id == "46130058"),
                "{n}: {:?}",
                d.refused
            );
        }
    }

    #[test]
    fn a_malformed_value_is_refused_with_the_variable_name_not_a_stop() {
        let d = demo_accounts(vars(&[
            ("JIAOYI_DEMO_ACCOUNT_ID", ""),
            ("JIAOYI_DEMO_ACCOUNT_ID_2", "4613005x"),
            ("JIAOYI_DEMO_ACCOUNT_ID_3", "0"),
            ("JIAOYI_DEMO_ACCOUNT_ID_4", "-47790949"),
            ("JIAOYI_DEMO_ACCOUNT_ID_5", "46130058,47790949"),
            ("JIAOYI_DEMO_ACCOUNT_ID_6", "47790949"),
        ]));
        assert_eq!(
            d.accounts,
            BTreeMap::from([(47790949, vec!["JIAOYI_DEMO_ACCOUNT_ID_6".to_string()])])
        );
        let reasons: Vec<&str> = d.refused.iter().map(|r| r.reason.as_str()).collect();
        assert_eq!(
            reasons,
            vec![
                "JIAOYI_DEMO_ACCOUNT_ID is not a positive integer account id",
                "JIAOYI_DEMO_ACCOUNT_ID_2 is not a positive integer account id",
                "JIAOYI_DEMO_ACCOUNT_ID_3 is not a positive integer account id",
                "JIAOYI_DEMO_ACCOUNT_ID_4 is not a positive integer account id",
                "JIAOYI_DEMO_ACCOUNT_ID_5 is not a positive integer account id",
            ]
        );
        assert_eq!(d.refused[1].account_id, "4613005x");
        // The plan records them and still runs the pair and the good one.
        let plan = owners_plan(&[
            ("JIAOYI_DEMO_ACCOUNT_ID_2", "4613005x"),
            ("JIAOYI_DEMO_ACCOUNT_ID_3", "47790949"),
        ]);
        let ids: Vec<i64> = plan.permitted.iter().map(|(a, _)| a.id()).collect();
        assert_eq!(ids, vec![49236051, 43002148, 47790949]);
        assert!(plan
            .refused
            .iter()
            .any(|r| r.reason == "JIAOYI_DEMO_ACCOUNT_ID_2 is not a positive integer account id"));
    }

    #[test]
    fn the_owners_demo_accounts_are_permitted_as_demo_without_tick_history() {
        let plan = owners_plan(&[
            ("JIAOYI_DEMO_ACCOUNT_ID_2", "46130058"),
            ("JIAOYI_DEMO_ACCOUNT_ID_3", "47790949"),
        ]);
        let got: Vec<(i64, Environment)> =
            plan.permitted.iter().map(|(a, e)| (a.id(), *e)).collect();
        assert_eq!(
            got,
            vec![
                (49236051, Environment::Demo),
                (43002148, Environment::Live),
                (46130058, Environment::Demo),
                (47790949, Environment::Demo),
            ]
        );
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);
        assert_eq!(plan.tick_history(49236051), TickHistory::On);
        assert_eq!(plan.tick_history(43002148), TickHistory::On);
        assert_eq!(
            plan.tick_history(46130058),
            TickHistory::Off {
                reason: "named by JIAOYI_DEMO_ACCOUNT_ID_2 only; tick history download off (owner, 25-10-2026: Postgres disk)".into()
            }
        );
        assert!(
            matches!(plan.tick_history(47790949), TickHistory::Off { reason } if reason.contains("JIAOYI_DEMO_ACCOUNT_ID_3"))
        );
        // The pair's own gate is unchanged: a demo variable's account passes
        // the allowlist, an unnamed one still does not.
        let list = Allowlist::parse(Some("49236051,43002148"))
            .unwrap()
            .with_demo_accounts(demo_accounts(vars(&[(
                "JIAOYI_DEMO_ACCOUNT_ID_2",
                "46130058",
            )])));
        assert_eq!(list.permit(46130058).unwrap().id(), 46130058);
        assert!(list.permit(47790949).is_err());
        assert_eq!(list.ids().collect::<Vec<_>>(), vec![43002148, 49236051]);
    }

    #[test]
    fn two_variables_naming_one_account_collapse_to_one() {
        let plan = owners_plan(&[
            ("JIAOYI_DEMO_ACCOUNT_ID_3", "46130058"),
            ("JIAOYI_DEMO_ACCOUNT_ID_2", "46130058"),
        ]);
        assert_eq!(
            plan.permitted
                .iter()
                .filter(|(a, _)| a.id() == 46130058)
                .count(),
            1
        );
        assert!(
            matches!(plan.tick_history(46130058), TickHistory::Off { reason }
            if reason.starts_with("named by JIAOYI_DEMO_ACCOUNT_ID_2, JIAOYI_DEMO_ACCOUNT_ID_3 only"))
        );
    }

    #[test]
    fn a_demo_variable_naming_a_live_account_is_refused_and_the_live_account_stays() {
        let plan = owners_plan(&[("JIAOYI_DEMO_ACCOUNT_ID_4", "43002148")]);
        assert!(plan
            .permitted
            .iter()
            .any(|(a, e)| a.id() == 43002148 && *e == Environment::Live));
        assert_eq!(plan.permitted.len(), 2);
        assert_eq!(plan.tick_history(43002148), TickHistory::On);
        let r: Vec<&Refusal> = plan.refused.iter().collect();
        assert_eq!(r.len(), 1, "{r:?}");
        assert_eq!(r[0].account_id, "43002148");
        assert!(
            r[0].reason.contains("both") && r[0].reason.contains("JIAOYI_DEMO_ACCOUNT_ID_4"),
            "{}",
            r[0].reason
        );
        // Declared live and not on the list: refused twice, never permitted.
        let list = Allowlist::parse(Some("49236051"))
            .unwrap()
            .with_demo_accounts(demo_accounts(vars(&[(
                "JIAOYI_DEMO_ACCOUNT_ID",
                "55555555",
            )])));
        let plan = plan_accounts(&list, Some("49236051:demo,55555555:live"));
        assert!(plan.permitted.iter().all(|(a, _)| a.id() != 55555555));
        assert_eq!(
            plan.refused
                .iter()
                .filter(|r| r.account_id == "55555555")
                .count(),
            2,
            "{:?}",
            plan.refused
        );
    }

    #[test]
    fn an_account_the_pair_also_permits_keeps_the_full_set() {
        let plan = owners_plan(&[("JIAOYI_DEMO_ACCOUNT_ID", "49236051")]);
        assert_eq!(plan.permitted.len(), 2);
        assert_eq!(plan.tick_history(49236051), TickHistory::On);
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);
    }

    #[test]
    fn a_demo_variable_answers_the_pairs_missing_half() {
        // On the list, no environment declared: the variable declares it demo.
        let list = Allowlist::parse(Some("49236051,46130058"))
            .unwrap()
            .with_demo_accounts(demo_accounts(vars(&[(
                "JIAOYI_DEMO_ACCOUNT_ID_2",
                "46130058",
            )])));
        let plan = plan_accounts(&list, Some("49236051:demo"));
        assert!(plan
            .permitted
            .iter()
            .any(|(a, e)| a.id() == 46130058 && *e == Environment::Demo));
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);
        assert!(matches!(
            plan.tick_history(46130058),
            TickHistory::Off { .. }
        ));
        // Declared demo, not on the list: the variable permits it.
        let list = Allowlist::parse(Some("49236051"))
            .unwrap()
            .with_demo_accounts(demo_accounts(vars(&[(
                "JIAOYI_DEMO_ACCOUNT_ID_2",
                "46130058",
            )])));
        let plan = plan_accounts(&list, Some("49236051:demo,46130058:demo"));
        assert_eq!(plan.permitted.len(), 2);
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);
        assert!(matches!(
            plan.tick_history(46130058),
            TickHistory::Off { .. }
        ));
    }

    #[test]
    fn without_demo_variables_the_plan_is_the_pairs() {
        let plan = owners_plan(&[]);
        assert_eq!(plan.permitted.len(), 2);
        assert!(plan.refused.is_empty() && plan.tick_history_off.is_empty());
    }

    #[test]
    fn plan_refuses_contradictory_environment() {
        let list = Allowlist::parse(Some("49236051")).unwrap();
        let plan = plan_accounts(&list, Some("49236051:demo,49236051:live"));
        assert!(plan.permitted.is_empty());
        assert!(plan.refused.iter().any(|r| r.reason.contains("both")));
    }
}
