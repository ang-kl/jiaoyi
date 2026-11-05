//! Reproduce one complete broker job and compare both frozen class hypotheses.
use jiaoyi_core::research::{
    class_prediction,
    opening_range::EntryRule,
    s3_signal::Request,
    s3_ticks::{self, Source},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("one source job path required")?;
    let mut bytes = Vec::new();
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
    if job["status"] != "complete"
        || job["result"]["input_sha256"] != digest
        || req
            .tick_replay
            .as_ref()
            .is_none_or(|o| o.entry_rule != EntryRule::ClassMomentum1h)
    {
        return Err("source checksum/status/request mismatch".into());
    }
    let momentum = serde_json::to_value(s3_ticks::study(&req, &src)?)?;
    if momentum != job["result"]["study"] {
        return Err("deployed/offline study mismatch".into());
    }
    req.tick_replay.as_mut().unwrap().entry_rule = EntryRule::ClassReversion1h;
    let reversion = s3_ticks::study(&req, &src)?;
    println!(
        "{}",
        json!({"symbol":req.symbol,"class":class_prediction::asset_class(&req.symbol),"from":req.from,"to":req.to,"input_sha256":digest,"algorithm_sha256":job["result"]["algorithm_sha256"],"candidates":{"class_momentum1h":momentum,"class_reversion1h":reversion}})
    );
    Ok(())
}
