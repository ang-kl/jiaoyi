use jiaoyi_core::research::{
    manifest_pullback::{self, Input},
    tiered_pullback::Policy,
};
use serde::Deserialize;
use std::io::Read;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    input: Input,
    policy: Policy,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("offline input required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("source budget".into());
    }
    let e: Envelope = serde_json::from_slice(&bytes)?;
    println!(
        "{}",
        serde_json::to_string(&manifest_pullback::study_policy(&e.input, &e.policy)?)?
    );
    Ok(())
}
