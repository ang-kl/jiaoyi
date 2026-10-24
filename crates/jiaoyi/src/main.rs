//! The jiaoyi service (Railway service `jiaoyi`).
//!
//! Start-up order: the account allowlist is checked first (missing or empty:
//! refuse to start), then migrations run, then accounts are recorded, then the
//! HTTP server, the tick recorders and the daily load loop start.
//!
//! On SIGTERM or Ctrl-C: the HTTP server stops, every recorder and the loader
//! are told to stop, and `main` waits up to `SHUTDOWN_GRACE` for the
//! recorders' final flushes before it returns.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use jiaoyi::config::Config;
use jiaoyi::http::{router, AppState};
use jiaoyi::service;
use jiaoyi::shutdown::{self, SHUTDOWN_GRACE};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = match Config::from_env() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!(error = %e, "refusing to start");
            std::process::exit(2);
        }
    };

    let pool = jiaoyi_store::connect(cfg.database_url.expose()).await?;
    jiaoyi_store::migrate(&pool).await?;
    tracing::info!("migrations applied");
    service::register_accounts(&pool, &cfg).await?;

    let loading = Arc::new(AtomicBool::new(false));
    let recorders = jiaoyi::recorder::StatusMap::default();
    let app = router(AppState {
        pool: pool.clone(),
        loading: loading.clone(),
        recorders: recorders.clone(),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.http_port)).await?;
    tracing::info!(port = cfg.http_port, "serving /health and /phase1/report");

    let (trigger, stop) = shutdown::channel();
    let trigger = Arc::new(trigger);
    let mut tasks = service::start_recorders(&pool, &cfg, &recorders, &stop);
    tasks.push(tokio::spawn(service::run_loop(
        pool,
        cfg.clone(),
        loading,
        stop.clone(),
    )));

    let on_signal = trigger.clone();
    let served = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown::os_signal().await;
            on_signal.fire();
        })
        .await;
    // Whether the server stopped on a signal or failed, stop the tasks too.
    trigger.fire();
    let report = shutdown::wait_for_tasks(tasks, SHUTDOWN_GRACE).await;
    tracing::info!(
        finished = report.finished,
        aborted = report.aborted,
        "shutdown complete"
    );
    served?;
    Ok(())
}
