//! The built binary refuses to start without an account allowlist, before it
//! touches the database or the broker.

use std::process::Command;

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
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_jiaoyi"))
        .env_clear()
        .env("RUST_LOG", "info")
        .env("JIAOYI_ALLOWED_ACCOUNTS", "49236051")
        .env("JIAOYI_ACCOUNT_ENVIRONMENTS", "49236051:demo")
        .env("DATABASE_URL", &url)
        .env("PORT", port.to_string())
        // Nothing listens here: loads fail fast and are recorded.
        .env("CTRADER_DEMO_HOST", "127.0.0.1")
        .env("CTRADER_PORT", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // Wait for /health.
    let mut up = false;
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            up = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(up, "service did not start");
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
    // One recorder, one history sync and the loader, all finished, none aborted.
    assert!(text.contains("finished=3"), "{text}");
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
