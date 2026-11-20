//! Service configuration from the environment: identity, connection and
//! credentials only. Every tunable number is a setting in the database.
//!
//! Secrets (client secret, tokens) are held as [`Secret`] and never logged.

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::allowlist::{
    demo_accounts, plan_accounts, AccountPlan, Allowlist, AllowlistError, DEMO_ACCOUNT_VAR,
    ENVIRONMENTS_VAR,
};
use jiaoyi_core::secret::Secret;

/// These closed broker accounts are deliberately omitted before any worker is planned.
pub(crate) const RETIRED_ACCOUNTS: [i64; 2] = [47790949, 49236051];

fn active_accounts(allowlist: Allowlist, environments: Option<&str>) -> (Allowlist, AccountPlan) {
    let allowlist = allowlist.without_accounts(&RETIRED_ACCOUNTS);
    let environments = environments
        .unwrap_or_default()
        .split(',')
        .filter(|declaration| {
            let id = declaration
                .split(':')
                .next()
                .and_then(|v| v.trim().parse::<i64>().ok());
            !id.is_some_and(|id| RETIRED_ACCOUNTS.contains(&id))
        })
        .collect::<Vec<_>>()
        .join(",");
    let plan = plan_accounts(&allowlist, Some(&environments));
    (allowlist, plan)
}

pub const CRYPTO_DISABLED: &str = "CRYPTO_4H_DISABLED_ARM_B";
pub const LIVE_PAUSED: &str = "LIVE_EXECUTION_PAUSED_PRE_QUALIFICATION";

/// Operator switches for opening new exposure. Read once per strategy session.
#[derive(Clone, Copy, Debug, Default)]
pub struct EntryPolicy {
    pub crypto_4h: bool,
    pub live_execution: bool,
}

impl EntryPolicy {
    pub(crate) fn from_env() -> Self {
        Self::from_values(
            var("JIAOYI_S3_ENABLE_CRYPTO_4H").as_deref(),
            var("JIAOYI_ENABLE_LIVE_EXECUTION").as_deref(),
        )
    }

    fn from_values(crypto: Option<&str>, live: Option<&str>) -> Self {
        let enabled =
            |value: Option<&str>| value.is_some_and(|v| v.trim().eq_ignore_ascii_case("true"));
        Self {
            crypto_4h: enabled(crypto),
            live_execution: enabled(live),
        }
    }

    pub(crate) fn live_refusal(self, env: Environment) -> Option<&'static str> {
        (env == Environment::Live && !self.live_execution).then_some(LIVE_PAUSED)
    }

    pub(crate) fn s3_refusal(self, env: Environment, symbol: &str) -> Option<&'static str> {
        self.live_refusal(env).or_else(|| {
            (!self.crypto_4h && matches!(symbol, "BTCUSD" | "ETHUSD")).then_some(CRYPTO_DISABLED)
        })
    }
}

/// Spotware's documented Open API endpoints, protobuf port
/// (help.ctrader.com/open-api/proxies-endpoints/).
pub const DEFAULT_DEMO_HOST: &str = "demo.ctraderapi.com";
pub const DEFAULT_LIVE_HOST: &str = "live.ctraderapi.com";
pub const DEFAULT_PORT: u16 = 5035;

#[derive(Debug, Clone)]
pub struct Config {
    pub allowlist: Allowlist,
    pub plan: AccountPlan,
    pub database_url: Secret,
    pub http_port: u16,
    pub broker: BrokerConfig,
}

#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub demo_host: String,
    pub live_host: String,
    pub port: u16,
    /// `None` when unset: every load then fails and records why.
    pub client_id: Option<String>,
    pub client_secret: Option<Secret>,
    /// Seed tokens: stored when `oauth_tokens` is empty, or when they are a
    /// new seed (a fingerprint the stored token does not descend from).
    pub seed_access_token: Option<Secret>,
    pub seed_refresh_token: Option<Secret>,
    pub seed_expires_at: Option<DateTime<Utc>>,
}

impl BrokerConfig {
    pub fn host(&self, env: Environment) -> &str {
        match env {
            Environment::Demo => &self.demo_host,
            Environment::Live => &self.live_host,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Allowlist(#[from] AllowlistError),
    #[error("DATABASE_URL is not set")]
    NoDatabase,
    #[error("{0} is not valid: {1}")]
    Invalid(&'static str, String),
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// The `JIAOYI_DEMO_ACCOUNT_ID*` variables of the process environment. Only
/// names with that prefix are read; no other variable's value is touched. A
/// value that is not valid Unicode reaches the parser as replacement
/// characters and is refused there, under its name.
fn demo_account_vars() -> Vec<(String, String)> {
    std::env::vars_os()
        .filter_map(|(k, v)| {
            let k = k.to_string_lossy().into_owned();
            k.starts_with(DEMO_ACCOUNT_VAR)
                .then(|| (k, v.to_string_lossy().into_owned()))
        })
        .collect()
}

fn parse<T: std::str::FromStr>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    match var(name) {
        None => Ok(default),
        Some(v) => v
            .trim()
            .parse()
            .map_err(|e: T::Err| ConfigError::Invalid(name, e.to_string())),
    }
}

impl Config {
    /// Read the environment. The allowlist is checked first: a missing or
    /// empty list stops start-up before anything else happens. The demo
    /// account variables add accounts to it; a malformed one is refused and
    /// recorded, and does not stop start-up (owner, 25-10-2026).
    pub fn from_env() -> Result<Config, ConfigError> {
        // Claude · № 16,973 25-Oct (ordered № 16,972·C; demo accounts by variable)
        let allowlist =
            Allowlist::from_env()?.with_demo_accounts(demo_accounts(demo_account_vars()));
        let (allowlist, plan) = active_accounts(allowlist, var(ENVIRONMENTS_VAR).as_deref());
        let database_url = Secret::new(var("DATABASE_URL").ok_or(ConfigError::NoDatabase)?);
        let seed_expires_at = match var("CTRADER_ACCESS_TOKEN_EXPIRES_AT") {
            None => None,
            Some(v) => Some(
                DateTime::parse_from_rfc3339(v.trim())
                    .map_err(|e| {
                        ConfigError::Invalid("CTRADER_ACCESS_TOKEN_EXPIRES_AT", e.to_string())
                    })?
                    .with_timezone(&Utc),
            ),
        };
        Ok(Config {
            allowlist,
            plan,
            database_url,
            http_port: parse("PORT", 8080)?,
            broker: BrokerConfig {
                demo_host: var("CTRADER_DEMO_HOST").unwrap_or_else(|| DEFAULT_DEMO_HOST.into()),
                live_host: var("CTRADER_LIVE_HOST").unwrap_or_else(|| DEFAULT_LIVE_HOST.into()),
                port: parse("CTRADER_PORT", DEFAULT_PORT)?,
                client_id: var("CTRADER_CLIENT_ID"),
                client_secret: var("CTRADER_CLIENT_SECRET").map(Secret::new),
                seed_access_token: var("CTRADER_ACCESS_TOKEN").map(Secret::new),
                seed_refresh_token: var("CTRADER_REFRESH_TOKEN").map(Secret::new),
                seed_expires_at,
            },
        })
    }
}

#[cfg(test)]
mod entry_policy_tests {
    use super::*;

    #[test]
    fn retired_ids_never_enter_the_allowlist_worker_plan_or_refusal_ledger() {
        let allowlist = Allowlist::parse(Some("43002148,46130058,47790949"))
            .unwrap()
            .with_demo_accounts(demo_accounts([
                ("JIAOYI_DEMO_ACCOUNT_ID", "49236051"),
                ("JIAOYI_DEMO_ACCOUNT_ID_1", "47790949"),
            ]));
        let (allowlist, plan) = active_accounts(
            allowlist,
            Some("43002148:live,46130058:demo,47790949:demo,49236051:demo,999:demo"),
        );
        assert_eq!(
            plan.permitted
                .iter()
                .map(|(a, e)| (a.id(), *e))
                .collect::<Vec<_>>(),
            [(43002148, Environment::Live), (46130058, Environment::Demo)]
        );
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.refused[0].account_id, "999");
        for id in RETIRED_ACCOUNTS {
            assert!(allowlist.permit(id).is_err());
            assert!(!plan.tick_history_off.contains_key(&id));
        }
    }

    #[test]
    fn only_explicit_opt_in_allows_crypto_or_live_entries() {
        let default = EntryPolicy::from_values(None, None);
        for sym in [
            "XAUUSD",
            "XAGUSD",
            "SpotCrude",
            "SpotBrent",
            "US500",
            "NAS100",
        ] {
            assert_eq!(default.s3_refusal(Environment::Demo, sym), None);
            assert_eq!(
                default.s3_refusal(Environment::Live, sym),
                Some(LIVE_PAUSED)
            );
        }
        for symbol in ["BTCUSD", "ETHUSD"] {
            assert_eq!(
                default.s3_refusal(Environment::Demo, symbol),
                Some(CRYPTO_DISABLED)
            );
            assert_eq!(
                EntryPolicy::from_values(Some("true"), None).s3_refusal(Environment::Live, symbol),
                Some(LIVE_PAUSED)
            );
            assert_eq!(
                EntryPolicy::from_values(None, Some("true")).s3_refusal(Environment::Live, symbol),
                Some(CRYPTO_DISABLED)
            );
            assert_eq!(
                EntryPolicy::from_values(Some("true"), Some("true"))
                    .s3_refusal(Environment::Live, symbol),
                None
            );
        }
        assert_eq!(default.live_refusal(Environment::Demo), None);
        assert_eq!(
            EntryPolicy::from_values(None, Some("enabled")).live_refusal(Environment::Live),
            Some(LIVE_PAUSED)
        );
        assert_eq!(
            EntryPolicy::from_values(None, Some("true")).live_refusal(Environment::Live),
            None
        );
    }
}
