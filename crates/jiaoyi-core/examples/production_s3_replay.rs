//! Bounded offline job: {request,result:{input,input_sha256},preparation}.
use jiaoyi_core::research::{production_s3, s3_signal::Request, s3_ticks::Source};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("source JSON path required")?;
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(32_000_001)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 32_000_000 {
        return Err("source exceeds 32MB".into());
    }
    let job: Value = serde_json::from_slice(&bytes)?;
    let req: Request = serde_json::from_value(job["request"].clone())?;
    let src: Source = serde_json::from_value(job["result"]["input"].clone())?;
    let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&src)?));
    if job["status"] != "complete" || job["result"]["input_sha256"] != digest {
        return Err("source checksum/status mismatch".into());
    }
    let preparation = serde_json::from_value(job["preparation"].clone())?;
    let mut algorithm = Sha256::new();
    for source in [
        include_bytes!("../src/research/production_s3.rs").as_slice(),
        include_bytes!("../src/research/s3_ticks.rs").as_slice(),
        include_bytes!("../src/research/s3_signal.rs").as_slice(),
        include_bytes!("../src/research/exit_replay.rs").as_slice(),
        include_bytes!("../src/strategy3/band.rs").as_slice(),
        include_bytes!("../src/strategy3/targets.rs").as_slice(),
        include_bytes!("../src/strategy3/sessions.rs").as_slice(),
        include_bytes!("../src/strategy3/mod.rs").as_slice(),
    ] {
        algorithm.update(source);
    }
    let algorithm_sha256 = format!("{:x}", algorithm.finalize());
    let study = production_s3::study(&req, &src, &preparation)?;
    println!(
        "{}",
        json!({"request":req,"preparation":preparation,
        "algorithm_sha256":algorithm_sha256,"input_sha256":digest,"study":study,"financial_pass":false,
        "account_admission":"not_reconstructed","qualification":"DEVELOPMENT_SCENARIO_ONLY"})
    );
    Ok(())
}
