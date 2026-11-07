//! Local files only. No broker transport or order routes.
use jiaoyi_core::research::{option_ab, s3_signal::Request, seasonal::M30, three_year::Payload};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
fn read(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("file exceeds32MB".into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("job and future-bars JSON required".into());
    }
    let job: Value = serde_json::from_slice(&read(&args[1])?)?;
    let req: Request = serde_json::from_value(job["request"].clone())?;
    let payload: Payload = serde_json::from_value(job["payload"].clone())?;
    let h = format!("{:x}", Sha256::digest(serde_json::to_vec(&payload)?));
    if job["status"] != "complete" || job["payload_sha256"] != h {
        return Err("source checksum/status mismatch".into());
    }
    let bytes = read(&args[2])?;
    let bars: Vec<M30> = serde_json::from_slice(&bytes)?;
    println!(
        "{}",
        json!({"payload_sha256":h,"future_sha256":format!("{:x}",Sha256::digest(&bytes)),"result":option_ab::study(&req,&payload,&bars)?})
    );
    Ok(())
}
