//! Orderly shutdown (review finding on #3).
//!
//! Railway stops a container with SIGTERM and, after the service's draining
//! time, SIGKILL. On SIGTERM or Ctrl-C the service stops taking HTTP requests,
//! tells every recorder and the loader to stop, and waits — up to
//! [`SHUTDOWN_GRACE`] — for the recorders to write their buffered ticks.
//! `railway.toml` sets `drainingSeconds` above this grace.

use std::time::Duration;

use tokio::sync::watch;
use tokio::task::JoinHandle;

/// How long `main` waits for the tasks to finish before it returns anyway.
pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

/// The sending half: fire once to stop everything.
#[derive(Debug)]
pub struct Trigger(watch::Sender<bool>);

/// A receiver every long-running task holds.
#[derive(Debug, Clone)]
pub struct Shutdown(watch::Receiver<bool>);

pub fn channel() -> (Trigger, Shutdown) {
    let (tx, rx) = watch::channel(false);
    (Trigger(tx), Shutdown(rx))
}

impl Trigger {
    pub fn fire(&self) {
        let _ = self.0.send(true);
    }
}

impl Shutdown {
    pub fn is_set(&self) -> bool {
        *self.0.borrow()
    }

    /// Resolves once shutdown is fired (or its trigger is gone).
    pub async fn wait(&mut self) {
        while !*self.0.borrow_and_update() {
            if self.0.changed().await.is_err() {
                return;
            }
        }
    }
}

/// Resolves on SIGTERM or Ctrl-C.
pub async fn os_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    _ = term.recv() => tracing::info!("SIGTERM received; shutting down"),
                    _ = tokio::signal::ctrl_c() => tracing::info!("Ctrl-C received; shutting down"),
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "cannot listen for SIGTERM; Ctrl-C only");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopReport {
    pub finished: usize,
    /// Tasks still running at the end of the grace period; they are aborted.
    pub aborted: usize,
}

/// Wait for every task to finish, until `grace` has passed; abort the rest.
pub async fn wait_for_tasks(handles: Vec<JoinHandle<()>>, grace: Duration) -> StopReport {
    let deadline = tokio::time::Instant::now() + grace;
    let mut report = StopReport {
        finished: 0,
        aborted: 0,
    };
    for mut h in handles {
        match tokio::time::timeout_at(deadline, &mut h).await {
            Ok(_) => report.finished += 1,
            Err(_) => {
                h.abort();
                report.aborted += 1;
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn wait_resolves_on_fire_and_reports_slow_tasks() {
        let (t, s) = channel();
        let mut waiting = s.clone();
        let task = tokio::spawn(async move { waiting.wait().await });
        assert!(!s.is_set());
        t.fire();
        assert!(s.is_set());
        let slow = tokio::spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        let r = wait_for_tasks(vec![task, slow], Duration::from_millis(200)).await;
        assert_eq!(
            r,
            StopReport {
                finished: 1,
                aborted: 1
            }
        );
    }
}
