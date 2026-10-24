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
