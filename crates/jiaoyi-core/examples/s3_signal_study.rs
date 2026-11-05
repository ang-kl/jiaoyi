//! Reproduce an exported broker backtest JSON without broker/database access.
use jiaoyi_core::research::s3_signal::{study, Bar, Request};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    io::stdin().take(5_000_001).read_to_end(&mut bytes)?;
    if bytes.len() > 5_000_000 {
        return Err("input exceeds 5MB".into());
    }
    let saved: serde_json::Value = serde_json::from_slice(&bytes)?;
    let req: Request = serde_json::from_value(saved["request"].clone())?;
    let bars: Vec<Bar> = serde_json::from_value(saved["result"]["bars"].clone())?;
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&bars)?));
    if saved["result"]["input_sha256"].as_str() != Some(&digest) {
        return Err("input checksum mismatch".into());
    }
    let result = study(&req, &bars)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
