//! Start-up and the daily load loop.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use jiaoyi_core::allowlist::{ALLOWLIST_VAR, ENVIRONMENTS_VAR};
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::{self as store};

use crate::config::Config;
use crate::loader;
use crate::recorder::{self, StatusMap};

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

/// Load every permitted account once, one after the other (they share one
/// OAuth token, which a refresh replaces).
pub async fn load_all(pool: &PgPool, cfg: &Config, loading: &AtomicBool) {
    loading.store(true, Ordering::Relaxed);
    for (account, env) in &cfg.plan.permitted {
        let result = match loader::connect(&cfg.broker, *env).await {
            Ok(conn) => loader::load_account(pool, &conn, &cfg.broker, account)
                .await
                .map(|_| ()),
            Err(e) => {
                tracing::error!(account = %account, error = %e, "cannot connect");
                record_connect_failure(pool, account.id(), &e.to_string()).await
            }
        };
        if let Err(e) = result {
            tracing::error!(account = %account, error = %e, "load could not be recorded");
        }
    }
    loading.store(false, Ordering::Relaxed);
}

async fn record_connect_failure(pool: &PgPool, account_id: i64, error: &str) -> store::Result<()> {
    let run = store::start_run(pool, account_id).await?;
    let summary = store::RunSummary {
        errors: vec![format!("connect: {error}")],
        ..Default::default()
    };
    store::finish_run(pool, run, "failed", &summary).await
}

/// Run forever: load, then wait `load.interval_hours` (read after each load).
pub async fn run_loop(pool: PgPool, cfg: Arc<Config>, loading: Arc<AtomicBool>) {
    loop {
        load_all(&pool, &cfg, &loading).await;
        let hours = match jiaoyi_store::settings::snapshot(&pool, None).await {
            Ok(s) => s.integer("load.interval_hours").unwrap_or_else(|e| {
                tracing::error!(error = %e, "load.interval_hours unreadable; retrying in 1 hour");
                1
            }),
            Err(e) => {
                tracing::error!(error = %e, "settings unreadable; retrying in 1 hour");
                1
            }
        };
        let wait = std::time::Duration::from_secs(hours.max(1) as u64 * 3600);
        tracing::info!(next_in_secs = wait.as_secs(), "load cycle done");
        tokio::time::sleep(wait).await;
    }
}

/// Start one tick recorder per permitted account.
pub fn start_recorders(pool: &PgPool, cfg: &Config, status: &StatusMap) {
    for (account, env) in &cfg.plan.permitted {
        tokio::spawn(recorder::run(
            pool.clone(),
            cfg.broker.clone(),
            account.clone(),
            *env,
            status.clone(),
        ));
    }
}
