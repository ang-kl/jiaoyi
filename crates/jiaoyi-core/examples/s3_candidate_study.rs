//! Reproduce a corpus against baseline + both frozen candidates. No network/order access.
use jiaoyi_core::research::{
    opening_range::EntryRule,
    s3_signal::{Bar, Request},
    s3_ticks::{self, Source},
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    jobs: Vec<String>,
    bars_jobs: HashMap<String, String>,
}
fn read(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > 32_000_000 {
        return Err("source exceeds32MB".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
fn hash<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(v)?)))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(100_001).read_to_string(&mut input)?;
    if input.len() > 100_000 {
        return Err("manifest too large".into());
    }
    let m: Manifest = serde_json::from_str(&input)?;
    if m.jobs.len() > 100 {
        return Err("maximum100 episodes per corpus".into());
    }
    let mut bars = HashMap::new();
    for (symbol, path) in &m.bars_jobs {
        let j = read(path)?;
        let b: Vec<Bar> = serde_json::from_value(j["result"]["bars"].clone())?;
        if j["status"] != "complete"
            || j["request"]["symbol"] != symbol.as_str()
            || Some(hash(&b)?.as_str()) != j["result"]["input_sha256"].as_str()
        {
            return Err("M30 identity/checksum mismatch".into());
        }
        bars.insert(
            symbol.clone(),
            (
                b,
                j["result"]["input_sha256"].clone(),
                j["request"]["account"].clone(),
            ),
        );
    }
    let mut seen = HashSet::new();
    let mut results = vec![];
    for path in m.jobs {
        let j = read(&path)?;
        let mut req: Request = serde_json::from_value(j["request"].clone())?;
        let mut src: Source = serde_json::from_value(j["result"]["input"].clone())?;
        if j["status"] != "complete"
            || Some(hash(&src)?.as_str()) != j["result"]["input_sha256"].as_str()
        {
            return Err("tick checksum/status mismatch".into());
        }
        let spread = match req.symbol.as_str() {
            "BTCUSD" => 15.0,
            "NatGas" => 0.004,
            _ => return Err("unsupported symbol".into()),
        };
        if req.spread != spread
            || req.commission_round_trip != 0.0
            || req.slippage_bps_per_side != 24.0
            || req.to - req.from != chrono::Duration::hours(4)
            || req
                .tick_replay
                .as_ref()
                .is_none_or(|o| o.volume != src.contract.min_volume)
        {
            return Err("frozen cost, volume or episode settings changed".into());
        }
        if !seen.insert((req.account, req.symbol.clone(), req.from)) {
            return Err("duplicate account/symbol/episode".into());
        }
        let (b, bar_hash, account) = bars.get(&req.symbol).ok_or("matching M30 source missing")?;
        if *account != json!(req.account) {
            return Err("M30/tick account mismatch".into());
        }
        src.bars = b
            .iter()
            .filter(|b| {
                b.at >= req.history_from() && b.at + chrono::Duration::minutes(30) <= req.to
            })
            .cloned()
            .collect();
        let merged_hash = hash(&src)?;
        for rule in [
            EntryRule::CurrentS3,
            EntryRule::OpeningRangeBreakout2h,
            EntryRule::OpeningRangeFailedBreakout2h,
        ] {
            req.tick_replay
                .as_mut()
                .ok_or("tick options missing")?
                .entry_rule = rule;
            let study = s3_ticks::study(&req, &src)?;
            results.push(json!({"account":req.account,"symbol":req.symbol,"from":req.from,"to":req.to,"rule":rule,"tick_source_sha256":j["result"]["input_sha256"],"m30_source_sha256":bar_hash,"combined_source_sha256":merged_hash,"study":study}));
        }
    }
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
