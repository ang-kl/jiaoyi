//! Offline files only; no transport, broker request or order route.
use jiaoyi_core::research::{native_target_sizing, s3_signal::Request, three_year::Payload};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("cached job JSON required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("source too large".into());
    }
    let job: Value = serde_json::from_slice(&bytes)?;
    let req: Request = serde_json::from_value(job["request"].clone())?;
    let payload: Payload = serde_json::from_value(job["payload"].clone())?;
    let h = format!("{:x}", Sha256::digest(serde_json::to_vec(&payload)?));
    if job["status"] != "complete" || job["payload_sha256"] != h {
        return Err("source status/hash mismatch".into());
    }
    println!(
        "{}",
        json!({"payload_sha256":h,"result":native_target_sizing::study(&req,&payload)?})
    );
    Ok(())
}
