//! Bounded offline exit-only comparison; no transport or production route.
use jiaoyi_core::research::profit_lock::{self, Case};
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("input path required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("input budget exceeded".into());
    }
    let case: Case = serde_json::from_slice(&bytes)?;
    println!(
        "{}",
        serde_json::to_string(&profit_lock::compare_case(case)?)?
    );
    Ok(())
}
