//! Service configuration from the environment: identity, connection and
//! credentials only. Every tunable number is a setting in the database.
//!
//! Secrets (client secret, tokens) are held as [`Secret`] and never logged.

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::Environment;
use jiaoyi_core::allowlist::{
    plan_accounts, AccountPlan, Allowlist, AllowlistError, ENVIRONMENTS_VAR,
};
use jiaoyi_core::secret::Secret;

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
    /// empty list stops start-up before anything else happens.
    pub fn from_env() -> Result<Config, ConfigError> {
        let allowlist = Allowlist::from_env()?;
        let plan = plan_accounts(&allowlist, var(ENVIRONMENTS_VAR).as_deref());
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
