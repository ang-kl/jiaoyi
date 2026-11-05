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
    let momentum = wire_value(&s3_ticks::study(&req, &src)?)?;
    if momentum != job["result"]["study"] {
        return Err("deployed/offline study mismatch".into());
    }
    let mut candidates = serde_json::Map::new();
    candidates.insert("class_momentum1h".into(), momentum);
    for rule in [
        EntryRule::ClassReversion1h,
        EntryRule::ClassChopReversion1h,
        EntryRule::ClassMomentumObserved1h,
        EntryRule::ClassReversionObserved1h,
        EntryRule::ClassChopReversionObserved1h,
    ] {
        req.tick_replay.as_mut().unwrap().entry_rule = rule;
        let name = serde_json::to_value(rule)?.as_str().unwrap().to_string();
        candidates.insert(name, serde_json::to_value(s3_ticks::study(&req, &src)?)?);
    }
    println!(
        "{}",
        json!({"symbol":req.symbol,"class":class_prediction::asset_class(&req.symbol),"from":req.from,"to":req.to,"input_sha256":digest,"source_algorithm_sha256":job["result"]["algorithm_sha256"],"replay_algorithm_sha256":replay_algorithm_sha256(),"candidates":candidates})
    );
    Ok(())
}

// Compare both studies after the same JSON wire round trip. A Value built
// directly from f64 retains the in-memory bits; the JSON parser can round its
// decimal spelling differently. No numeric tolerance or field omission is used.
fn wire_value<T: serde::Serialize>(value: &T) -> Result<Value, serde_json::Error> {
    serde_json::from_slice(&serde_json::to_vec(value)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_comparison_accepts_same_decimal_but_rejects_changed_evidence() {
        let remote: Value =
            serde_json::from_str(r#"{"return_bps":-30.756127566953722,"side":"sell"}"#).unwrap();
        let local = json!({"return_bps":-30.756127566953722_f64,"side":"sell"});
        assert_eq!(wire_value(&local).unwrap(), remote);
        assert_ne!(
            wire_value(&json!({"return_bps":-30.75612_f64,"side":"sell"})).unwrap(),
            remote
        );
        assert_ne!(
            wire_value(&json!({"return_bps":-30.756127566953722_f64,"side":"buy"})).unwrap(),
            remote
        );
    }
}

fn replay_algorithm_sha256() -> String {
    let mut h = Sha256::new();
    for bytes in [
        include_bytes!("../src/research/s3_signal.rs").as_slice(),
        include_bytes!("../src/research/fx_diagnostic.rs").as_slice(),
        include_bytes!("../src/research/class_prediction.rs").as_slice(),
        include_bytes!("../src/research/s3_ticks.rs").as_slice(),
        include_bytes!("../src/research/opening_range.rs").as_slice(),
        include_bytes!("../src/research/exit_replay.rs").as_slice(),
        include_bytes!("../src/strategy3/band.rs").as_slice(),
        include_bytes!("../src/strategy3/targets.rs").as_slice(),
        include_bytes!("../src/strategy3/sessions.rs").as_slice(),
        include_bytes!("../src/strategy3/mod.rs").as_slice(),
    ] {
        h.update(bytes);
    }
    format!("{:x}", h.finalize())
}
