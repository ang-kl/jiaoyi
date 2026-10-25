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
    // Reviewed settings changes (config/settings-changes.json), each once.
    jiaoyi::trading::apply_seed(&pool, jiaoyi::trading::SEED_FILE).await?;

    let loading = Arc::new(AtomicBool::new(false));
    let recorders = jiaoyi::recorder::StatusMap::default();
    let history = jiaoyi::history::HistoryStatusMap::default();
    let trading_loops = jiaoyi::trading::LivenessMap::default();
    let strategy3_loops = jiaoyi::trading::LivenessMap::default();
    let app = router(AppState {
        pool: pool.clone(),
        loading: loading.clone(),
        recorders: recorders.clone(),
        history: history.clone(),
        trading_loops: trading_loops.clone(),
        strategy3_loops: strategy3_loops.clone(),
        allowed_accounts: Arc::new(cfg.plan.permitted.iter().map(|(a, _)| a.id()).collect()),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.http_port)).await?;
    tracing::info!(
        port = cfg.http_port,
        "serving /health, /phase1/report, /research/inventory, /research/bars, /research/jobs and /research/trials"
    );

    let (trigger, stop) = shutdown::channel();
    let trigger = Arc::new(trigger);
    let mut tasks = service::start_recorders(&pool, &cfg, &recorders, &stop);
    tasks.extend(service::start_history(&pool, &cfg, &history, &stop));
    // The research engine: the pre-registered jobs, one at a time.
    tasks.push(tokio::spawn(jiaoyi::edge::runner::run(
        pool.clone(),
        cfg.broker.clone(),
        cfg.plan.permitted.clone(),
        stop.clone(),
    )));
    // Strategy 1 (tick-triggered trend): one loop per account; demo trades at
    // once, live sends nothing until the edge gate passes.
    tasks.push(tokio::spawn(jiaoyi::trading::run(
        pool.clone(),
        cfg.clone(),
        trading_loops,
        stop.clone(),
    )));
    // Strategy 3 (intraday momentum, tick-triggered on the noise band): one
    // loop per account, demo and live (owner, 25-10-2026: "Live too, from now").
    tasks.push(tokio::spawn(jiaoyi::strategy3::run(
        pool.clone(),
        cfg.clone(),
        strategy3_loops,
        stop.clone(),
    )));
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
