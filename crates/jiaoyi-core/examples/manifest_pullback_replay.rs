//! One bounded offline normalised input from the verified manifest runner.
use jiaoyi_core::research::manifest_pullback::{self, Input};
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("normalised manifest input required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("source budget".into());
    }
    let input: Input = serde_json::from_slice(&bytes)?;
    println!(
        "{}",
        serde_json::to_string(&manifest_pullback::study(&input)?)?
    );
    Ok(())
}
