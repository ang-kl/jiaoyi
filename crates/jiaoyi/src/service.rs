//! Start-up and the daily load loop.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use jiaoyi_core::allowlist::{ALLOWLIST_VAR, ENVIRONMENTS_VAR};
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::{self as store};

use crate::config::Config;
use crate::loader;
use crate::recorder::{self, StatusMap};
use crate::shutdown::Shutdown;
use tokio::task::JoinHandle;

/// Record every refusal and every permitted account. Run once at start-up,
/// after migrations.
pub async fn register_accounts(pool: &PgPool, cfg: &Config) -> store::Result<()> {
    for r in &cfg.plan.refused {
        tracing::warn!(account = %r.account_id, reason = %r.reason, "account refused");
        store::record_refusal(pool, &r.account_id, &r.reason, "startup").await?;
    }
    for (account, env) in &cfg.plan.permitted {
        store::upsert_account(pool, account.id(), *env).await?;
    }
    tracing::info!(
        permitted = ?cfg.plan.permitted.iter().map(|(a, e)| format!("{a}:{}", e.as_str())).collect::<Vec<_>>(),
        refused = cfg.plan.refused.len(),
        "accounts from {ALLOWLIST_VAR} and {ENVIRONMENTS_VAR}"
    );
    Ok(())
}

/// How a failed load is retried (settings `load.auth_retry_attempts`,
/// `load.auth_retry_secs`, `load.retry_minutes`; owner deploy review, 24-10-2026).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub auth_retry_attempts: u32,
    pub auth_retry_secs: u64,
    pub retry_minutes: u64,
    pub interval_hours: u64,
}

impl RetryPolicy {
    pub async fn read(pool: &PgPool) -> anyhow::Result<RetryPolicy> {
        let s = store::settings::snapshot(pool, None).await?;
        let get = |k: &str| s.integer(k).map_err(anyhow::Error::msg);
        Ok(RetryPolicy {
            auth_retry_attempts: get("load.auth_retry_attempts")?.max(0) as u32,
            auth_retry_secs: get("load.auth_retry_secs")?.max(0) as u64,
            retry_minutes: get("load.retry_minutes")?.max(1) as u64,
            interval_hours: get("load.interval_hours")?.max(1) as u64,
        })
    }

    /// The wait before the next load of an account after this outcome: the
    /// daily interval after `ok`, the short retry after `partial` or `failed`.
    pub fn next_wait(&self, status: &str) -> Duration {
        if status == "ok" {
            Duration::from_secs(self.interval_hours * 3600)
        } else {
            Duration::from_secs(self.retry_minutes * 60)
        }
    }
}

/// Load one account. When the run fails because the broker rejected the
/// token, and the token in `oauth_tokens` was refreshed during that run (by
/// this load or another task), the load is tried again at once, on a new
/// connection, up to `auth_retry_attempts` times, waiting `auth_retry_secs`
/// and then twice as long each time. Every attempt is its own recorded run.
pub async fn load_with_retries<C, F>(
    pool: &PgPool,
    connect: C,
    broker: &crate::config::BrokerConfig,
    account: &jiaoyi_core::allowlist::PermittedAccount,
    policy: &RetryPolicy,
    shutdown: &Shutdown,
) -> store::Result<loader::LoadOutcome>
where
    C: Fn() -> F,
    F: std::future::Future<
        Output = Result<jiaoyi_ctrader::Connection, jiaoyi_ctrader::ClientError>,
    >,
{
    let mut attempt = 0u32;
    loop {
        let started = chrono::Utc::now();
        let outcome = match connect().await {
            Ok(conn) => loader::load_account(pool, &conn, broker, account).await?,
            Err(e) => {
                tracing::error!(account = %account, error = %e, "cannot connect");
                record_connect_failure(pool, account.id(), &e.to_string()).await?
            }
        };
        let refreshed = outcome.auth_rejected
            && store::load_token(pool)
                .await?
                .is_some_and(|t| t.source == "refresh" && t.obtained_at >= started);
        if outcome.status == "failed"
            && refreshed
            && attempt < policy.auth_retry_attempts
            && !shutdown.is_set()
        {
            attempt += 1;
            let wait = Duration::from_secs(
                policy
                    .auth_retry_secs
                    .saturating_mul(1 << (attempt - 1).min(16)),
            );
            tracing::warn!(
                account = %account,
                attempt,
                wait_secs = wait.as_secs(),
                "load failed on authorisation after a token refresh; trying again now"
            );
            tokio::time::sleep(wait).await;
            continue;
        }
        return Ok(outcome);
    }
}

async fn record_connect_failure(
    pool: &PgPool,
    account_id: i64,
    error: &str,
) -> store::Result<loader::LoadOutcome> {
    let run = store::start_run(pool, account_id).await?;
    let summary = store::RunSummary {
        errors: vec![format!("connect: {error}")],
        ..Default::default()
    };
    store::finish_run(pool, run, "failed", &summary).await?;
    Ok(loader::LoadOutcome {
        run_id: run,
        status: "failed",
        summary,
        auth_rejected: false,
    })
}

/// Run until shutdown. Each account has its own next-load time: the daily
/// interval after a good load, `load.retry_minutes` after a failed or partial
/// one (owner deploy review, 24-10-2026), so one account's failure is retried
/// in minutes, not after 24 hours.
pub async fn run_loop(
    pool: PgPool,
    cfg: Arc<Config>,
    loading: Arc<AtomicBool>,
    mut shutdown: Shutdown,
) {
    let mut due: HashMap<i64, tokio::time::Instant> = HashMap::new();
    loop {
        if shutdown.is_set() {
            return;
        }
        let policy = match RetryPolicy::read(&pool).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "load settings unreadable; retrying in 5 minutes");
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(300)) => continue,
                    _ = shutdown.wait() => return,
                }
            }
        };
        let now = tokio::time::Instant::now();
        loading.store(true, Ordering::Relaxed);
        for (account, env) in &cfg.plan.permitted {
            if shutdown.is_set() {
                break;
            }
            if due.get(&account.id()).is_some_and(|t| *t > now) {
                continue;
            }
            let broker = cfg.broker.clone();
            let env = *env;
            let result = load_with_retries(
                &pool,
                || {
                    let b = broker.clone();
                    async move { loader::connect(&b, env).await }
                },
                &cfg.broker,
                account,
                &policy,
                &shutdown,
            )
            .await;
            let wait = match result {
                Ok(o) => policy.next_wait(o.status),
                Err(e) => {
                    tracing::error!(account = %account, error = %e, "load could not be recorded");
                    policy.next_wait("failed")
                }
            };
            tracing::info!(account = %account, next_in_secs = wait.as_secs(), "next load scheduled");
            due.insert(account.id(), tokio::time::Instant::now() + wait);
        }
        loading.store(false, Ordering::Relaxed);
        let next = due
            .values()
            .min()
            .copied()
            .unwrap_or_else(|| tokio::time::Instant::now() + policy.next_wait("ok"));
        tokio::select! {
            _ = tokio::time::sleep_until(next) => {}
            _ = shutdown.wait() => return,
        }
    }
}

/// Start one tick recorder per permitted account. The handles are kept so
/// that shutdown can wait for their final flush.
pub fn start_recorders(
    pool: &PgPool,
    cfg: &Config,
    status: &StatusMap,
    shutdown: &Shutdown,
) -> Vec<JoinHandle<()>> {
    cfg.plan
        .permitted
        .iter()
        .map(|(account, env)| {
            tokio::spawn(recorder::run(
                pool.clone(),
                cfg.broker.clone(),
                account.clone(),
                *env,
                status.clone(),
                shutdown.clone(),
            ))
        })
        .collect()
}

/// Start one history sync per permitted account (deals, ticks, probe).
pub fn start_history(
    pool: &PgPool,
    cfg: &Config,
    status: &crate::history::HistoryStatusMap,
    shutdown: &Shutdown,
) -> Vec<JoinHandle<()>> {
    cfg.plan
        .permitted
        .iter()
        .map(|(account, env)| {
            tokio::spawn(crate::history::run(
                pool.clone(),
                cfg.broker.clone(),
                account.clone(),
                *env,
                status.clone(),
                shutdown.clone(),
            ))
        })
        .collect()
}
