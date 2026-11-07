//! Local files only. No broker client or live strategy activation.
use jiaoyi_core::research::{peer_evaluation, s3_signal::Request, three_year::Payload};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("job JSON required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("source exceeds32MB".into());
    }
    let job: Value = serde_json::from_slice(&bytes)?;
    let req: Request = serde_json::from_value(job["request"].clone())?;
    let payload: Payload = serde_json::from_value(job["payload"].clone())?;
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&payload)?));
    if job["status"] != "complete" || job["payload_sha256"] != digest {
        return Err("source checksum/status mismatch".into());
    }
    println!(
        "{}",
        json!({"payload_sha256":digest,"result":peer_evaluation::study(&req,&payload)?})
    );
    Ok(())
}
