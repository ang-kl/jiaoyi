//! Reproduce an exported broker backtest JSON without broker/database access.
use jiaoyi_core::research::s3_signal::{study, Bar, Request};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    io::stdin().take(32_000_001).read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("input exceeds 32MB".into());
    }
    let saved: serde_json::Value = serde_json::from_slice(&bytes)?;
    let req: Request = serde_json::from_value(saved["request"].clone())?;
    if req.tick_replay.is_some() {
        let source: jiaoyi_core::research::s3_ticks::Source =
            serde_json::from_value(saved["result"]["input"].clone())?;
        let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&source)?));
        if saved["result"]["input_sha256"].as_str() != Some(&digest) {
            return Err("input checksum mismatch".into());
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&jiaoyi_core::research::s3_ticks::study(&req, &source)?)?
        );
        return Ok(());
    }
    let bars: Vec<Bar> = serde_json::from_value(saved["result"]["bars"].clone())?;
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&bars)?));
    if saved["result"]["input_sha256"].as_str() != Some(&digest) {
        return Err("input checksum mismatch".into());
    }
    let result = study(&req, &bars)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
