//! Offline full-exit comparison of the six existing frozen hypotheses.
use jiaoyi_core::research::{
    opening_range::EntryRule,
    s3_signal::Request,
    s3_ticks::{self, Source},
};
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
    let mut req: Request = serde_json::from_value(job["request"].clone())?;
    let src: Source = serde_json::from_value(job["result"]["input"].clone())?;
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&src)?));
    if job["status"] != "complete" || job["result"]["input_sha256"] != digest {
        return Err("source status/checksum mismatch".into());
    }
    let mut studies = serde_json::Map::new();
    for rule in [
        EntryRule::ClassMomentum1h,
        EntryRule::ClassReversion1h,
        EntryRule::ClassChopReversion1h,
        EntryRule::ClassMomentumObserved1h,
        EntryRule::ClassReversionObserved1h,
        EntryRule::ClassChopReversionObserved1h,
    ] {
        req.tick_replay
            .as_mut()
            .ok_or("tick options missing")?
            .entry_rule = rule;
        let key = serde_json::to_value(rule)?.as_str().unwrap().to_owned();
        studies.insert(key, serde_json::to_value(s3_ticks::study(&req, &src)?)?);
    }
    println!(
        "{}",
        json!({"symbol":req.symbol,"from":req.from,"to":req.to,
        "input_sha256":digest,"studies":studies,"financial_pass":false})
    );
    Ok(())
}
