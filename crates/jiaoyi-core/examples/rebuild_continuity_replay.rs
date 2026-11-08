use jiaoyi_core::research::{exit_replay, manifest_pullback, rebuild_continuity};
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    input: manifest_pullback::Input,
    entry: exit_replay::Input,
    engine_version: String,
    expected_outcome: serde_json::Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("verified recorded input required")?;
    let mut raw = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut raw)?;
    if raw.len() > 32_000_000 {
        return Err("source budget".into());
    }
    let r: Request = serde_json::from_slice(&raw)?;
    println!(
        "{}",
        serde_json::to_string(&rebuild_continuity::verify_recorded_path(
            &r.input,
            r.entry,
            &r.engine_version,
            &r.expected_outcome
        )?)?
    );
    Ok(())
}
