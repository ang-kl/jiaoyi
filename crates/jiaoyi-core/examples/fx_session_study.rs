//! Verify and reproduce diagnostic source jobs offline. No broker/order access.
use jiaoyi_core::research::{
    opening_range::EntryRule,
    s3_signal::Request,
    s3_ticks::{self, Source},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::{self, Read},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(100_001).read_to_string(&mut input)?;
    if input.len() > 100_000 {
        return Err("manifest too large".into());
    }
    let files: Vec<String> = serde_json::from_str(&input)?;
    if files.len() > 100 {
        return Err("maximum 100 diagnostic jobs".into());
    }
    let mut seen = HashSet::new();
    let mut out = vec![];
    for path in files {
        let mut bytes = vec![];
        std::fs::File::open(path)?
            .take(32_000_001)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 32_000_000 {
            return Err("job exceeds 32MB".into());
        }
        let job: Value = serde_json::from_slice(&bytes)?;
        let req: Request = serde_json::from_value(job["request"].clone())?;
        let src: Source = serde_json::from_value(job["result"]["input"].clone())?;
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&src)?));
        if job["status"] != "complete"
            || job["result"]["input_sha256"] != hash
            || req
                .tick_replay
                .as_ref()
                .is_none_or(|o| o.entry_rule != EntryRule::FxSessionDiagnostic)
            || !seen.insert((req.account, req.symbol.clone(), req.from))
        {
            return Err("invalid diagnostic identity, checksum, mode or duplicate window".into());
        }
        let study = s3_ticks::study(&req, &src)?;
        out.push(json!({"request":req,"input_sha256":hash,"study":study}));
    }
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
