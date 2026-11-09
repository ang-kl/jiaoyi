//! Deterministic, read-only backstop for the strategy actors. It observes
//! progress independently; it never calls an entry/exit handler or the broker.

use std::future::Future;
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_ctrader::Connection;
use serde_json::{json, Value};
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::shutdown::Shutdown;
use crate::trading::{beat, LivenessMap};

pub const INTERVAL: Duration = Duration::from_secs(3);
pub const MAX_QUOTE_AGE: Duration = Duration::from_secs(6);
/// An observation threshold only: this never pauses or changes a trade.
const OVERDUE: Duration = Duration::from_secs(6);

#[derive(Debug, Clone, PartialEq)]
struct Work {
    stage: &'static str,
    started_at: DateTime<Utc>,
    started: Instant,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    generation: u64,
    connected: bool,
    work: Option<Work>,
    last_work_completed_at: Option<DateTime<Utc>>,
    last_work_duration_ms: Option<u128>,
    last_quote_received_at: Option<DateTime<Utc>>,
    last_quote_processed_receipt_at: Option<DateTime<Utc>>,
    quote_receiver_lagged: u64,
    actor_quote_receiver_lagged: u64,
    actor_execution_receiver_lagged: u64,
    last_execution_processed_receipt_at: Option<DateTime<Utc>>,
    watchdog_completed_at: Option<DateTime<Utc>>,
    watchdog_completed: Option<Instant>,
    sampled_state: Option<&'static str>,
    last_exit_check_at: Option<DateTime<Utc>>,
    exit_checks: u64,
    owned_positions: Option<usize>,
    unresolved_opens: Option<usize>,
}

impl Progress {
    fn state(&self, now: Instant) -> &'static str {
        if self.watchdog_completed.is_none() {
            return "starting";
        }
        if self
            .watchdog_completed
            .is_some_and(|t| now.duration_since(t) > OVERDUE)
        {
            return "watchdog_stalled";
        }
        if !self.connected {
            return "disconnected";
        }
        if self
            .work
            .as_ref()
            .is_some_and(|w| now.duration_since(w.started) > OVERDUE)
        {
            return "actor_work_overdue";
        }
        "observing"
    }

    pub fn health(&self) -> Value {
        let now = Instant::now();
        json!({
            "state": self.state(now),
            "mode": "watchdog_read_only",
            "actor_backstop": "registered_exits_only",
            "interval_secs": INTERVAL.as_secs(),
            "overdue_after_secs": OVERDUE.as_secs(),
            "session_generation": self.generation,
            "session_connected": self.connected,
            "last_watchdog_completed_at": self.watchdog_completed_at,
            "watchdog_completion_age_ms": self.watchdog_completed.map(|t| now.duration_since(t).as_millis()),
            "active_stage": self.work.as_ref().map(|w| w.stage),
            "stage_started_at": self.work.as_ref().map(|w| w.started_at),
            "stage_elapsed_ms": self.work.as_ref().map(|w| now.duration_since(w.started).as_millis()),
            "last_work_completed_at": self.last_work_completed_at,
            "last_work_duration_ms": self.last_work_duration_ms,
            "last_quote_received_at": self.last_quote_received_at,
            "last_quote_processed_receipt_at": self.last_quote_processed_receipt_at,
            "quote_receiver_lagged": self.quote_receiver_lagged,
            "quote_receiver_lagged_scope": "independent_observer",
            "actor_quote_receiver_lagged": self.actor_quote_receiver_lagged,
            "actor_execution_receiver_lagged": self.actor_execution_receiver_lagged,
            "actor_receiver_lagged_scope": "delivery_gaps_since_reconnect_not_unique_events",
            "last_execution_processed_receipt_at": self.last_execution_processed_receipt_at,
            "last_exit_check_completed_at": self.last_exit_check_at,
            "exit_checks_completed": self.exit_checks,
            "owned_position_count": self.owned_positions,
            "unresolved_open_count": self.unresolved_opens,
            "unresolved_exit_count": null,
            "evidence_scope": "actor progress, not a protection or financial acceptance certificate",
        })
    }
}

/// The observer owns no trading state and creates no broker subscription.
/// Dropping the session aborts its receiver so reconnects cannot accumulate
/// orphan observers or reuse old receipt times as fresh-session evidence.
pub struct SessionObserver {
    handle: JoinHandle<()>,
    live: LivenessMap,
    account: i64,
}

impl SessionObserver {
    pub fn start(conn: &Connection, live: &LivenessMap, account: i64, mut stop: Shutdown) -> Self {
        beat(live, account, |l| {
            let generation = l.monitor.generation.saturating_add(1);
            let completed_at = l.monitor.watchdog_completed_at;
            let completed = l.monitor.watchdog_completed;
            l.monitor = Progress {
                generation,
                connected: true,
                watchdog_completed_at: completed_at,
                watchdog_completed: completed,
                ..Progress::default()
            };
        });
        let mut quotes = conn.spot_events();
        let observed = live.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop.wait() => return,
                    event = quotes.recv() => match event {
                        Ok(q) if q.event.ctid_trader_account_id == account => beat(&observed, account, |l| {
                            l.monitor.last_quote_received_at = Some(q.received_at);
                        }),
                        Ok(_) => {},
                        Err(RecvError::Lagged(n)) => beat(&observed, account, |l| {
                            l.monitor.quote_receiver_lagged = l.monitor.quote_receiver_lagged.saturating_add(n);
                        }),
                        Err(RecvError::Closed) => {
                            beat(&observed, account, |l| l.monitor.connected = false);
                            return;
                        },
                    }
                }
            }
        });
        Self {
            handle,
            live: live.clone(),
            account,
        }
    }
}

impl Drop for SessionObserver {
    fn drop(&mut self) {
        self.handle.abort();
        beat(&self.live, self.account, |l| {
            l.monitor.connected = false;
            l.monitor.work = None;
        });
    }
}

/// A completion is recorded only after the awaited operation returns.
/// Cancellation clears the active stage without manufacturing a completion.
pub async fn observe<F: Future>(
    live: &LivenessMap,
    account: i64,
    stage: &'static str,
    f: F,
) -> F::Output {
    let guard = WorkGuard::start(live, account, stage);
    let result = f.await;
    guard.complete();
    result
}

pub struct WorkGuard {
    live: LivenessMap,
    account: i64,
    completed: bool,
    parent: Option<Work>,
}

impl WorkGuard {
    pub fn start(live: &LivenessMap, account: i64, stage: &'static str) -> Self {
        let mut parent = None;
        beat(live, account, |l| {
            parent = l.monitor.work.take();
            l.monitor.work = Some(Work {
                stage,
                started_at: Utc::now(),
                started: Instant::now(),
            })
        });
        Self {
            live: live.clone(),
            account,
            completed: false,
            parent,
        }
    }

    pub fn complete(mut self) {
        beat(&self.live, self.account, |l| {
            l.monitor.last_work_duration_ms = l
                .monitor
                .work
                .as_ref()
                .map(|w| w.started.elapsed().as_millis());
            l.monitor.last_work_completed_at = Some(Utc::now());
            l.monitor.work = self.parent.take();
        });
        self.completed = true;
    }
}

impl Drop for WorkGuard {
    fn drop(&mut self) {
        if !self.completed {
            beat(&self.live, self.account, |l| {
                l.monitor.work = self.parent.take()
            });
        }
    }
}

pub fn quote_processed(live: &LivenessMap, account: i64, received_at: DateTime<Utc>) {
    beat(live, account, |l| {
        l.monitor.last_quote_processed_receipt_at = Some(
            l.monitor
                .last_quote_processed_receipt_at
                .map_or(received_at, |old| old.max(received_at)),
        )
    });
}

pub fn quote_is_fresh(at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    at.is_some_and(|at| at <= now && now - at <= chrono::Duration::from_std(MAX_QUOTE_AGE).unwrap())
}

pub fn exit_check_completed(live: &LivenessMap, account: i64, owned: usize, pending: usize) {
    beat(live, account, |l| {
        l.monitor.last_exit_check_at = Some(Utc::now());
        l.monitor.exit_checks = l.monitor.exit_checks.saturating_add(1);
        l.monitor.owned_positions = Some(owned);
        l.monitor.unresolved_opens = Some(pending);
    });
}

pub fn execution_processed(live: &LivenessMap, account: i64, received_at: DateTime<Utc>) {
    beat(live, account, |l| {
        l.monitor.last_execution_processed_receipt_at = Some(received_at)
    });
}

/// Actor receivers have their own cursors. A healthy independent observer
/// cannot establish that the trading actor has consumed every event.
pub fn actor_quote_lagged(live: &LivenessMap, account: i64, missed: u64) {
    beat(live, account, |l| {
        l.monitor.actor_quote_receiver_lagged =
            l.monitor.actor_quote_receiver_lagged.saturating_add(missed);
    });
}

pub fn actor_execution_lagged(live: &LivenessMap, account: i64, missed: u64) {
    beat(live, account, |l| {
        l.monitor.actor_execution_receiver_lagged = l
            .monitor
            .actor_execution_receiver_lagged
            .saturating_add(missed);
    });
}

fn pulse(live: &LivenessMap, strategy: &str) {
    let now = Instant::now();
    for (account, l) in live.lock().unwrap().iter_mut() {
        l.monitor.watchdog_completed = Some(now);
        l.monitor.watchdog_completed_at = Some(Utc::now());
        let state = l.monitor.state(now);
        if l.monitor.sampled_state != Some(state) {
            tracing::info!(account, strategy, state, "position watchdog state changed");
            l.monitor.sampled_state = Some(state);
        }
    }
}

/// No broker/DB/model work is awaited here. This task is independent from
/// both strategy actors and from their historical/reconciliation requests.
pub async fn run(s1: LivenessMap, s3: LivenessMap, mut stop: Shutdown) {
    let mut timer = tokio::time::interval(INTERVAL);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = stop.wait() => return,
            _ = timer.tick() => {
                pulse(&s1, "s1");
                pulse(&s3, "s3");
            }
        }
    }
}

/// Read progress after the HTTP handler's DB awaits. Deriving a six-second
/// watchdog state from its earlier cloned snapshot could falsely report a
/// stall when the health query itself was slow.
pub fn attach(report: &mut Value, s1: &LivenessMap, s3: &LivenessMap) {
    for (name, live) in [("trading", s1), ("strategy3", s3)] {
        let live = live.lock().unwrap();
        if let Some(accounts) = report[name].as_array_mut() {
            for account in accounts {
                let progress = account["account_id"].as_i64().and_then(|id| live.get(&id));
                account["liveness"]["fast_monitor"] =
                    progress.map(|l| l.monitor.health()).unwrap_or(Value::Null);
            }
        }
    }
}

/// Reuse the existing report collector; do not add another broker poll.
/// This is a bounded observation, not an atomic current account read.
pub async fn broker_snapshot(pool: &jiaoyi_store::sqlx::PgPool, account: i64) -> Value {
    use jiaoyi_store::sqlx::{self, Row};
    match sqlx::query("SELECT observed_at, detail->'source_complete' AS source_complete, detail->'book_stable' AS book_stable FROM report_account_snapshots WHERE account_id=$1 AND sample_key='latest'")
        .bind(account).fetch_optional(pool).await {
        Ok(Some(row)) => {
            let at: DateTime<Utc> = row.get("observed_at");
            json!({"observed_at":at,"source_complete":row.get::<Option<Value>,_>("source_complete").and_then(|v| v.as_bool()),"book_stable":row.get::<Option<Value>,_>("book_stable").and_then(|v| v.as_bool()),
                "observation_age_seconds":(Utc::now()-at).num_seconds(),"source":"existing_report_snapshot","freshness_is_not_protection_acceptance":true})
        },
        _ => json!({"observed_at":null,"source_complete":null,"book_stable":null,"state":"unknown"}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shutdown;

    #[test]
    fn http_progress_uses_the_current_map_instead_of_a_pre_query_snapshot() {
        let live = LivenessMap::default();
        beat(&live, 7, |l| l.monitor.connected = true);
        pulse(&live, "test");
        let mut report = json!({"trading":[{"account_id":7,"liveness":{"fast_monitor":{"state":"watchdog_stalled"}}}],"strategy3":[]});
        attach(&mut report, &live, &LivenessMap::default());
        assert_eq!(
            report["trading"][0]["liveness"]["fast_monitor"]["state"],
            "observing"
        );
    }

    #[tokio::test]
    async fn stalled_work_does_not_refresh_its_completion_when_the_watchdog_runs() {
        let live = LivenessMap::default();
        beat(&live, 7, |l| l.monitor.connected = true);
        let blocked = observe(&live, 7, "held_history_read", std::future::pending::<()>());
        tokio::pin!(blocked);
        tokio::select! { _ = &mut blocked => unreachable!(), _ = tokio::task::yield_now() => {} }
        beat(&live, 7, |l| {
            l.monitor.work.as_mut().unwrap().started -= Duration::from_secs(7)
        });
        pulse(&live, "test");
        let p = live.lock().unwrap()[&7].monitor.clone();
        assert_eq!(p.health()["state"], "actor_work_overdue");
        assert!(p.watchdog_completed_at.is_some());
        assert!(p.last_work_completed_at.is_none());
        let mut stale = p;
        stale.watchdog_completed = Some(Instant::now() - Duration::from_secs(7));
        assert_eq!(stale.health()["state"], "watchdog_stalled");
    }

    #[tokio::test]
    async fn cancellation_is_not_a_completed_check_and_shutdown_finishes_the_watchdog() {
        let live = LivenessMap::default();
        {
            let blocked = observe(&live, 7, "quote", std::future::pending::<()>());
            tokio::pin!(blocked);
            tokio::select! { _ = &mut blocked => unreachable!(), _ = tokio::task::yield_now() => {} }
        }
        assert!(live.lock().unwrap()[&7]
            .monitor
            .last_work_completed_at
            .is_none());
        assert!(live.lock().unwrap()[&7].monitor.work.is_none());
        let (trigger, stop) = shutdown::channel();
        let task = tokio::spawn(run(live.clone(), LivenessMap::default(), stop));
        tokio::time::sleep(Duration::from_millis(10)).await;
        trigger.fire();
        tokio::time::timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap();
        assert!(live.lock().unwrap()[&7]
            .monitor
            .watchdog_completed_at
            .is_some());
    }

    #[tokio::test]
    async fn independent_task_keeps_completing_while_an_actor_future_is_pending() {
        let live = LivenessMap::default();
        beat(&live, 7, |l| l.monitor.connected = true);
        let actor_live = live.clone();
        let actor = tokio::spawn(async move {
            observe(&actor_live, 7, "held_read", std::future::pending::<()>()).await;
        });
        let (trigger, stop) = shutdown::channel();
        let watchdog = tokio::spawn(run(live.clone(), LivenessMap::default(), stop));
        tokio::time::sleep(Duration::from_millis(10)).await;
        let first = live.lock().unwrap()[&7]
            .monitor
            .watchdog_completed_at
            .unwrap();
        tokio::time::sleep(INTERVAL + Duration::from_millis(50)).await;
        let p = live.lock().unwrap()[&7].monitor.clone();
        assert!(p.watchdog_completed_at.unwrap() > first);
        assert_eq!(p.work.as_ref().unwrap().stage, "held_read");
        assert!(p.last_work_completed_at.is_none());
        actor.abort();
        let _ = actor.await;
        trigger.fire();
        watchdog.await.unwrap();
    }

    #[tokio::test]
    async fn reconnect_resets_receipts_and_the_observer_never_sends_trading_frames() {
        use jiaoyi_core::{allowlist::Allowlist, secret::Secret};
        use jiaoyi_ctrader::testkit::FakeBroker;
        use std::collections::HashMap;
        let (conn, log) = FakeBroker {
            account_id: 7,
            quotes: HashMap::from([(22, vec![(Some(100), Some(101))])]),
            ..Default::default()
        }
        .spawn();
        conn.application_auth("id", &Secret::new("test"))
            .await
            .unwrap();
        let list = Allowlist::parse(Some("7")).unwrap();
        let account = list.permit(7).unwrap();
        let session = conn
            .account_auth(&account, &Secret::new("tok"))
            .await
            .unwrap();
        let (_trigger, stop) = shutdown::channel();
        let live = LivenessMap::default();
        let observer = SessionObserver::start(&conn, &live, 7, stop.clone());
        session.subscribe_spots(&[22]).await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if live.lock().unwrap()[&7]
                    .monitor
                    .last_quote_received_at
                    .is_some()
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(live.lock().unwrap()[&7]
            .monitor
            .last_quote_processed_receipt_at
            .is_none());
        actor_quote_lagged(&live, 7, 1024);
        actor_execution_lagged(&live, 7, 17);
        drop(observer);
        assert!(!live.lock().unwrap()[&7].monitor.connected);
        let observer = SessionObserver::start(&conn, &live, 7, stop);
        let p = live.lock().unwrap()[&7].monitor.clone();
        assert_eq!(p.generation, 2);
        assert!(p.last_quote_received_at.is_none());
        assert!(p.last_work_completed_at.is_none());
        assert_eq!(p.actor_quote_receiver_lagged, 0);
        assert_eq!(p.actor_execution_receiver_lagged, 0);
        drop(observer);
        let log = log.lock().unwrap();
        assert!(log.orders.is_empty());
        assert!(log.amends.is_empty());
        assert!(log.closes.is_empty());
    }
}
