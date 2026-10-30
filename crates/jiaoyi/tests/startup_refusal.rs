//! The built binary refuses to start without an account allowlist, before it
//! touches the database or the broker.

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Output};

/// Read readiness from this child's pipe. A successful connection to a
/// released ephemeral port can belong to another concurrent test process.
struct ServiceChild {
    child: Option<Child>,
    stdout: Option<std::thread::JoinHandle<Vec<u8>>>,
    ready: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl ServiceChild {
    fn new(mut child: Child) -> Self {
        let stdout = child.stdout.take().unwrap();
        let (ready, received) = tokio::sync::oneshot::channel();
        let stdout = std::thread::spawn(move || {
            let mut ready = Some(ready);
            let mut output = Vec::new();
            for line in BufReader::new(stdout).split(b'\n') {
                let Ok(line) = line else { break };
                if String::from_utf8_lossy(&line).contains("serving /health, /operations/reports") {
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(());
                    }
                }
                output.extend(line);
                output.push(b'\n');
            }
            output
        });
        Self {
            child: Some(child),
            stdout: Some(stdout),
            ready: Some(received),
        }
    }

    async fn wait_until_serving(&mut self) {
        let ready = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.ready.take().unwrap(),
        )
        .await;
        if !matches!(ready, Ok(Ok(()))) {
            self.stop();
            let stdout = self.stdout.take().unwrap().join().unwrap();
            let mut stderr = String::new();
            self.child
                .as_mut()
                .unwrap()
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut stderr)
                .unwrap();
            panic!(
                "child did not serve within 10 s: {ready:?}\n{}\n{stderr}",
                String::from_utf8_lossy(&stdout)
            );
        }
    }

    fn stop(&mut self) {
        if let Some(child) = &mut self.child {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }

    fn wait_with_output(mut self) -> std::io::Result<Output> {
        let mut output = self.child.take().unwrap().wait_with_output()?;
        output.stdout = self.stdout.take().unwrap().join().unwrap();
        Ok(output)
    }
}

impl std::ops::Deref for ServiceChild {
    type Target = Child;
    fn deref(&self) -> &Child {
        self.child.as_ref().unwrap()
    }
}
impl std::ops::DerefMut for ServiceChild {
    fn deref_mut(&mut self) -> &mut Child {
        self.child.as_mut().unwrap()
    }
}
impl Drop for ServiceChild {
    fn drop(&mut self) {
        self.stop();
        if let Some(stdout) = self.stdout.take() {
            let _ = stdout.join();
        }
    }
}

fn run(envs: &[(&str, &str)]) -> (Option<i32>, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jiaoyi"));
    cmd.env_clear().env("RUST_LOG", "info");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("binary runs");
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text)
}

#[test]
fn missing_allowlist_refuses_start_even_with_a_database() {
    let (code, text) = run(&[("DATABASE_URL", "postgres://nobody@127.0.0.1:1/none")]);
    assert_eq!(code, Some(2), "{text}");
    assert!(
        text.contains("JIAOYI_ALLOWED_ACCOUNTS is not set"),
        "{text}"
    );
    assert!(text.contains("refusing to start"), "{text}");
}

#[test]
fn empty_allowlist_refuses_start() {
    let (code, text) = run(&[
        ("JIAOYI_ALLOWED_ACCOUNTS", " , "),
        ("DATABASE_URL", "postgres://x@127.0.0.1:1/x"),
    ]);
    assert_eq!(code, Some(2), "{text}");
    assert!(text.contains("JIAOYI_ALLOWED_ACCOUNTS is empty"), "{text}");
}

/// Review finding 3 on #3: SIGTERM (how Railway stops a container) stops the
/// service, which waits for its recorders and loader before exiting.
#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn sigterm_stops_the_service_after_its_tasks_finish(pool: sqlx::PgPool) {
    let db: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let url = format!("{}/{db}", base.rsplit_once('/').unwrap().0);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jiaoyi"))
        .env_clear()
        .env("RUST_LOG", "info")
        .env("JIAOYI_ALLOWED_ACCOUNTS", "49236051")
        .env("JIAOYI_ACCOUNT_ENVIRONMENTS", "49236051:demo")
        .env("DATABASE_URL", &url)
        .env("PORT", "0")
        // Nothing listens here: loads fail fast and are recorded.
        .env("CTRADER_DEMO_HOST", "127.0.0.1")
        .env("CTRADER_PORT", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map(ServiceChild::new)
        .unwrap();
    // The child reports serving only after its account registration and bind.
    child.wait_until_serving().await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let killed = std::process::Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(killed.success());
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(12),
            "still running 12 s after SIGTERM"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    };
    let out = child.wait_with_output().unwrap();
    let text = strip_ansi(
        &(String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)),
    );
    assert!(status.success(), "{status:?}\n{text}");
    assert!(text.contains("SIGTERM received"), "{text}");
    // One recorder, one history sync, the loader, the research runner,
    // Strategy 1's trader (0012), Strategy 3's (0013), and the assessment
    // supervisor and independent position watchdog, all finished,
    // none aborted.
    assert!(text.contains("finished=8"), "{text}");
    assert!(text.contains("aborted=0"), "{text}");
}

/// Owner, 25-10-2026 ("Code reads the new variables"): the service reads
/// `JIAOYI_DEMO_ACCOUNT_ID*` from its environment. A good one adds a demo
/// account; a malformed one is recorded under its name and does not stop
/// start-up; no other variable is read into the log.
// Claude · № 16,973 25-Oct (ordered № 16,972·C; demo accounts by variable)
#[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
async fn the_service_reads_the_demo_account_variables(pool: sqlx::PgPool) {
    let db: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let url = format!("{}/{db}", base.rsplit_once('/').unwrap().0);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jiaoyi"))
        .env_clear()
        .env("RUST_LOG", "info")
        .env("JIAOYI_ALLOWED_ACCOUNTS", "49236051")
        .env("JIAOYI_ACCOUNT_ENVIRONMENTS", "49236051:demo")
        .env("JIAOYI_DEMO_ACCOUNT_ID_2", "46130058")
        .env("JIAOYI_DEMO_ACCOUNT_ID_3", "4779094x")
        .env("UNRELATED_SECRET", "do-not-read-me")
        .env("DATABASE_URL", &url)
        .env("PORT", "0")
        .env("CTRADER_DEMO_HOST", "127.0.0.1")
        .env("CTRADER_PORT", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map(ServiceChild::new)
        .unwrap();
    child.wait_until_serving().await;
    // Accounts are registered before the HTTP server starts.
    let accounts: Vec<(i64, String)> =
        sqlx::query_as("SELECT account_id, environment FROM accounts ORDER BY account_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        accounts,
        vec![
            (46130058, "demo".to_string()),
            (49236051, "demo".to_string())
        ]
    );
    let refusals: Vec<(String, String)> =
        sqlx::query_as("SELECT account_ref, reason FROM account_refusals")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        refusals,
        vec![(
            "4779094x".to_string(),
            "JIAOYI_DEMO_ACCOUNT_ID_3 is not a positive integer account id".to_string()
        )]
    );
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(std::process::Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap()
        .success());
    let started = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(started.elapsed() < std::time::Duration::from_secs(12));
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let out = child.wait_with_output().unwrap();
    let text = strip_ansi(
        &(String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)),
    );
    assert!(!text.contains("refusing to start"), "{text}");
    assert!(text.contains("tick_history_off=[46130058]"), "{text}");
    assert!(!text.contains("do-not-read-me"), "{text}");
    // Two recorders, two history syncs, the loader, the research runner,
    // Strategy 1's trader (0012), Strategy 3's (0013), the supervisor and
    // independent position watchdog.
    assert!(text.contains("finished=10"), "{text}");
    assert!(text.contains("aborted=0"), "{text}");
}

/// Remove terminal colour codes (ESC [ ... m) from log text.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
