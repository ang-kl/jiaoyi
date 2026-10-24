//! The background runner: one pending job at a time, in run order.
//!
//! - A job is attempted only when its inputs exist: every universe symbol's
//!   June spread samples taken (development years only), and the deposit
//!   currency established. Until then it waits; it is not attempted.
//! - Each attempt is recorded with its start time and the deployed commit.
//!   An attempt interrupted by a stop wrote nothing (the trial and the job's
//!   end are one transaction); the job goes back to pending and is attempted
//!   again, and its trial lists every attempt.
//! - The held-out job is REFUSED, with the reason, unless the development
//!   default met PASS rules 1–3 and at least 6 of the 9 development cells
//!   have a profit factor above 1.0, all under the same pre-registration hash.
//! - A job registered under another hash than the one compiled in is refused.

use std::time::Duration;

use jiaoyi_core::allowlist::{Environment, PermittedAccount};
use jiaoyi_core::research::stats::RStats;
use jiaoyi_core::research::verdict::{development_gate, Rules};
use jiaoyi_ctrader::{AccountSession, Connection};
use jiaoyi_store::research::{self as rs, JobRow};
use jiaoyi_store::sqlx::PgPool;
use serde_json::{json, Value};

use super::data::{Data, Stage};
use super::h1job::{self, Deposit, Setting};
use super::prereg;
use super::recheck;
use super::sampler::{self, FIRST_SAMPLE_YEAR, LAST_SAMPLE_YEAR};
use crate::config::BrokerConfig;
use crate::loader;
use crate::shutdown::Shutdown;

/// A sample week that failed this many times is not asked again; its year
/// takes the nearest measured spread, labelled estimated.
pub const MAX_SAMPLE_FAILURES: i64 = 3;

#[derive(Debug, Clone)]
pub struct RunnerOptions {
    /// Pause when nothing can be attempted.
    pub idle: Duration,
}

impl Default for RunnerOptions {
    fn default() -> Self {
        RunnerOptions {
            idle: Duration::from_secs(600),
        }
    }
}

/// The deployed commit, recorded on every attempt.
pub fn deployed_commit() -> String {
    std::env::var("RAILWAY_GIT_COMMIT_SHA")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "unknown".into())
}

/// Run over TLS until shutdown.
pub async fn run(
    pool: PgPool,
    broker: BrokerConfig,
    permitted: Vec<(PermittedAccount, Environment)>,
    shutdown: Shutdown,
) {
    run_with(
        pool,
        broker.clone(),
        permitted.clone(),
        move |account: i64| {
            let b = broker.clone();
            let env = permitted
                .iter()
                .find(|(a, _)| a.id() == account)
                .map(|(_, e)| *e);
            async move {
                let env =
                    env.ok_or_else(|| anyhow::anyhow!("account {account} is not permitted"))?;
                loader::connect(&b, env).await.map_err(anyhow::Error::from)
            }
        },
        RunnerOptions::default(),
        shutdown,
    )
    .await
}

/// What the runner did with the job at the head of the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Nothing pending.
    Idle,
    /// The job's inputs are not ready; it waits, not attempted.
    Waiting(String),
    /// The job reached a final status.
    Finished(String),
}

pub async fn run_with<C, F>(
    pool: PgPool,
    broker: BrokerConfig,
    permitted: Vec<(PermittedAccount, Environment)>,
    connect: C,
    opts: RunnerOptions,
    mut shutdown: Shutdown,
) where
    C: Fn(i64) -> F,
    F: std::future::Future<Output = anyhow::Result<Connection>>,
{
    match rs::release_interrupted_jobs(&pool).await {
        Ok(n) if n > 0 => tracing::warn!(
            jobs = n,
            "research jobs interrupted by a stop go back to pending"
        ),
        Ok(_) => {}
        Err(e) => {
            tracing::error!(error = %e, "research jobs: release of interrupted attempts failed")
        }
    }
    let mut last_wait: Option<String> = None;
    loop {
        if shutdown.is_set() {
            return;
        }
        let watcher = shutdown.clone();
        let step = tokio::select! {
            s = step(&pool, &broker, &permitted, &connect, &watcher) => s,
            _ = shutdown.wait() => return,
        };
        let pause = match step {
            Ok(Step::Finished(what)) => {
                tracing::info!(%what, "research job finished");
                last_wait = None;
                continue;
            }
            Ok(Step::Idle) => opts.idle,
            Ok(Step::Waiting(why)) => {
                if last_wait.as_deref() != Some(why.as_str()) {
                    tracing::info!(reason = %why, "research job waiting for its inputs");
                    last_wait = Some(why);
                }
                opts.idle
            }
            Err(e) => {
                tracing::warn!(error = %format!("{e:#}"), "research runner step failed; trying again later");
                opts.idle
            }
        };
        tokio::select! {
            _ = tokio::time::sleep(pause) => {}
            _ = shutdown.wait() => return,
        }
    }
}

/// One step: look at the next pending job and take it as far as it goes.
pub async fn step<C, F>(
    pool: &PgPool,
    broker: &BrokerConfig,
    permitted: &[(PermittedAccount, Environment)],
    connect: &C,
    shutdown: &Shutdown,
) -> anyhow::Result<Step>
where
    C: Fn(i64) -> F,
    F: std::future::Future<Output = anyhow::Result<Connection>>,
{
    let Some(job) = rs::next_pending_job(pool).await? else {
        return Ok(Step::Idle);
    };
    let hash = prereg::hash();
    if job.prereg_hash != hash {
        return finish_without_trial(
            pool,
            &job,
            "refused",
            &format!(
                "registered under pre-registration {}, but the service runs {hash}; a changed rule is a new pre-registration",
                job.prereg_hash
            ),
        )
        .await;
    }
    let Some(setting) = Setting::from_json(&job.setting) else {
        return finish_without_trial(pool, &job, "failed", "the job's setting is unreadable").await;
    };
    let Some((account, _)) = permitted.iter().find(|(a, _)| a.id() == setting.account_id) else {
        return Ok(Step::Waiting(format!(
            "account {} is not on the allowlist of this service",
            setting.account_id
        )));
    };
    if job.hypothesis != "H1" {
        return Ok(Step::Waiting(format!(
            "no engine for {} in this release",
            job.hypothesis
        )));
    }
    let Some(stage) = Stage::parse(&job.stage) else {
        return finish_without_trial(pool, &job, "failed", "unknown stage").await;
    };
    match stage {
        Stage::Development => {
            // Inputs first: the spread samples (broker), then the deposit currency.
            if let Some(missing) = missing_samples(pool, setting.account_id).await? {
                let conn = connect(setting.account_id).await?;
                let session = loader::authorise(pool, &conn, broker, account).await?;
                let data = Data::new(pool, Stage::Development);
                for (id, name, have) in missing {
                    if shutdown.is_set() {
                        return Ok(Step::Waiting("shutdown".into()));
                    }
                    sampler::sample_symbol(
                        &data,
                        &session,
                        setting.account_id,
                        id,
                        &name,
                        &have,
                        FIRST_SAMPLE_YEAR..=LAST_SAMPLE_YEAR,
                    )
                    .await?;
                }
                if let Some(still) = missing_samples(pool, setting.account_id).await? {
                    return Ok(Step::Waiting(format!(
                        "{} symbols still miss June spread samples",
                        still.len()
                    )));
                }
            }
            let deposit = match h1job::deposit_currency(pool, setting.account_id).await? {
                Deposit::Usd(v) => v,
                Deposit::NotReady(why) => return Ok(Step::Waiting(why)),
                Deposit::Failed(why) => {
                    return attempt_and(pool, &job, |_| async move { Err(anyhow::anyhow!(why)) })
                        .await;
                }
            };
            attempt_and(pool, &job, |attempt| {
                run_trial(
                    pool,
                    job.clone(),
                    setting,
                    Stage::Development,
                    deposit,
                    None::<&AccountSession<'_>>,
                    attempt,
                )
            })
            .await
        }
        Stage::HeldOut => {
            let (met, why) = development_gate_under(pool, &hash).await?;
            if !met {
                return attempt_and_refuse(pool, &job, &why).await;
            }
            let deposit = match h1job::deposit_currency(pool, setting.account_id).await? {
                Deposit::Usd(v) => v,
                Deposit::NotReady(why) => return Ok(Step::Waiting(why)),
                Deposit::Failed(why) => {
                    return attempt_and(pool, &job, |_| async move { Err(anyhow::anyhow!(why)) })
                        .await;
                }
            };
            let conn = connect(setting.account_id).await?;
            let session = loader::authorise(pool, &conn, broker, account).await?;
            let session = &session;
            attempt_and(pool, &job, |attempt| {
                run_trial(
                    pool,
                    job.clone(),
                    setting,
                    Stage::HeldOut,
                    deposit,
                    Some(session),
                    attempt,
                )
            })
            .await
        }
    }
}

/// Claim the job (recording the attempt) and close it without a trial.
async fn finish_without_trial(
    pool: &PgPool,
    job: &JobRow,
    status: &str,
    reason: &str,
) -> anyhow::Result<Step> {
    if rs::claim_job(pool, job.id, &deployed_commit())
        .await?
        .is_none()
    {
        return Ok(Step::Waiting("taken by another runner".into()));
    }
    rs::close_job(pool, job.id, status, reason).await?;
    Ok(Step::Finished(format!("job {} {status}: {reason}", job.id)))
}

async fn attempt_and_refuse(pool: &PgPool, job: &JobRow, why: &str) -> anyhow::Result<Step> {
    finish_without_trial(pool, job, "refused", why).await
}

/// Claim the job, run `f` with the attempt number; a trial is written by `f`
/// (job done); an error fails the job with its reason.
async fn attempt_and<G, Fut>(pool: &PgPool, job: &JobRow, f: G) -> anyhow::Result<Step>
where
    G: FnOnce(i32) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<i64>>,
{
    let Some(attempt) = rs::claim_job(pool, job.id, &deployed_commit()).await? else {
        return Ok(Step::Waiting("taken by another runner".into()));
    };
    match f(attempt).await {
        Ok(trial) => Ok(Step::Finished(format!(
            "job {} done: trial {trial}",
            job.id
        ))),
        Err(e) => {
            let why = format!("{e:#}");
            rs::close_job(pool, job.id, "failed", &why).await?;
            Ok(Step::Finished(format!("job {} failed: {why}", job.id)))
        }
    }
}

/// Universe symbols on the account that still miss a June sample, with the
/// years they have.
async fn missing_samples(
    pool: &PgPool,
    account_id: i64,
) -> anyhow::Result<Option<Vec<(i64, String, Vec<i32>)>>> {
    let snap = jiaoyi_store::settings::snapshot(pool, Some(account_id)).await?;
    let universe = snap
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    let samples = rs::spread_samples(pool, account_id).await?;
    let given_up = rs::sample_weeks_failed(pool, account_id, MAX_SAMPLE_FAILURES).await?;
    let mut out = Vec::new();
    for (name, _) in universe {
        let ids = jiaoyi_store::symbol_ids_by_name(pool, account_id, &name).await?;
        let [id] = ids.as_slice() else {
            continue; // not resolved on the account: refused later with the reason
        };
        // A week is done when measured or without ticks, or when it failed
        // MAX_SAMPLE_FAILURES times (its year then takes an estimated spread).
        let have: Vec<i32> = samples
            .iter()
            .filter(|s| s.broker_symbol_id == *id)
            .map(|s| s.year)
            .chain(given_up.iter().filter(|(s, _)| s == id).map(|(_, y)| *y))
            .collect();
        if (FIRST_SAMPLE_YEAR..=LAST_SAMPLE_YEAR).any(|y| !have.contains(&y)) {
            out.push((*id, name, have));
        }
    }
    Ok((!out.is_empty()).then_some(out))
}

/// The development gate from the stored development trials under `hash`.
pub async fn development_gate_under(pool: &PgPool, hash: &str) -> anyhow::Result<(bool, String)> {
    let trials: Vec<rs::TrialRow> = rs::job_trials(pool)
        .await?
        .into_iter()
        .filter(|t| {
            t.hypothesis == "H1"
                && t.sample == "development"
                && t.prereg_hash.as_deref() == Some(hash)
        })
        .collect();
    let default = trials
        .iter()
        .find(|t| t.params["setting"]["default"].as_bool() == Some(true));
    let Some(default) = default else {
        return Ok((
            false,
            "no demonstrated edge: the default (50, 3) has no development trial under this pre-registration".into(),
        ));
    };
    let rules: Rules = serde_json::from_value(
        default
            .results
            .as_ref()
            .map(|r| r["rules"].clone())
            .unwrap_or(Value::Null),
    )?;
    let cells: Vec<Option<f64>> = trials
        .iter()
        .map(|t| {
            t.results
                .as_ref()
                .and_then(|r| serde_json::from_value::<RStats>(r["r"].clone()).ok())
                .and_then(|s| s.pf())
        })
        .collect();
    Ok(development_gate(&rules, &cells))
}

/// Run one job to its trial (done) or an error (failed by the caller).
async fn run_trial(
    pool: &PgPool,
    job: JobRow,
    setting: Setting,
    stage: Stage,
    deposit: Value,
    session: Option<&AccountSession<'_>>,
    _attempt: i32,
) -> anyhow::Result<i64> {
    let data = Data::new(pool, stage);
    let u = h1job::prepare(&data, setting.account_id, deposit).await?;
    if u.symbols.is_empty() {
        anyhow::bail!("no symbol could be tested: {:?}", u.refused);
    }
    let (per_symbol, notes) = match stage {
        Stage::Development => (h1job::simulate_all(&u, &setting, 1.0), Vec::new()),
        Stage::HeldOut => {
            let session = session
                .ok_or_else(|| anyhow::anyhow!("no broker session for the tick re-check"))?;
            let mut all = Vec::new();
            let mut notes = Vec::new();
            for x in &u.symbols {
                let (t, n) =
                    recheck::simulate_with_ticks(&data, session, x, &setting.params(1.0)).await?;
                all.push(t);
                notes.extend(n);
            }
            (all, notes)
        }
    };
    let (mut results, eval) = h1job::evaluate(&u, &setting, &per_symbol, stage);
    let slippage = h1job::slippage_table(&u, &setting);
    results["slippage_daily_bars"] = slippage.clone();
    results["evidence_against"]["slippage_2x"] = slippage["2x"].clone();
    // Estimated- vs measured-spread trades (owner, 24-10-2026): older
    // spreads were probably wider, so estimated years are flattered.
    let (est, meas): (Vec<f64>, Vec<f64>) = {
        let all: Vec<&jiaoyi_core::research::h1::Trade> = per_symbol.iter().flatten().collect();
        (
            all.iter()
                .filter(|t| t.spread_estimated)
                .map(|t| t.net_r)
                .collect(),
            all.iter()
                .filter(|t| !t.spread_estimated)
                .map(|t| t.net_r)
                .collect(),
        )
    };
    let est_stats = jiaoyi_core::research::stats::r_stats(&est);
    let meas_stats = jiaoyi_core::research::stats::r_stats(&meas);
    results["spread_split"] = json!({
        "estimated_spread_trades": est_stats,
        "measured_spread_trades": meas_stats,
        "note": "a trade is 'estimated' when its entry or exit year had no June spread measurement of its own",
    });
    if eval.r.total_r > 0.0 && est_stats.total_r > 0.5 * eval.r.total_r {
        results["evidence_against"]["edge_mostly_from_estimated_spread_years"] = json!(format!(
            "{:.2} of the {:.2} total R comes from trades in years whose spread is estimated",
            est_stats.total_r, eval.r.total_r
        ));
    }
    if stage == Stage::HeldOut {
        let diffs: Vec<f64> = notes.iter().filter_map(|n| n.diff_r).collect();
        results["recheck"] = json!({
            "counted": "tick",
            "checked": notes.len(),
            "no_tick_trade": notes.iter().filter(|n| n.tick.is_none()).count(),
            "mean_diff_r": jiaoyi_core::research::stats::mean(&diffs),
            "trades": notes,
        });
    }
    let attempts = rs::job_attempts(pool, job.id).await?;
    let commits: std::collections::BTreeSet<&str> =
        attempts.iter().map(|a| a.commit.as_str()).collect();
    if commits.len() > 1 {
        results["evidence_against"]["engine_changed_between_attempts"] = json!(commits);
    }
    results["attempts"] = json!(attempts);
    let hash = prereg::hash();
    let (params, range) = h1job::inputs(&u, &setting, stage, &hash);
    let verdict = eval.verdict.clone();
    let trial = rs::finish_job_with_trial(
        pool,
        &rs::JobTrial {
            job_id: job.id,
            prereg_hash: &hash,
            hypothesis: "H1",
            sample: stage.as_str(),
            params: &params,
            data_range: &range,
            results: &results,
            verdict: &verdict,
            settings_version: u.settings_version,
        },
    )
    .await?;
    Ok(trial)
}
