#!/usr/bin/env python3
"""Offline bridge to unchanged Phase-4 native seasonal controls (№18,382).

The old three-year *adapter* fixes oil at 13:00 UTC and excludes peers. This
bridge changes only module visibility: every Rust computation is read from the
frozen archive, while its +1h entry/+4h exit and 24bp admission remain intact.
It does not use the later fixed-window candidate adapter or make network calls.
"""
from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path

WORK = Path("/workspace/jiaoyi-stage1-18382")
FROZEN = Path("/workspace/jiaoyi-frozen-phase4-18366")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def build_bridge(frozen_root, work):
    """Compile unchanged archived modules; return an auditable binary receipt."""
    frozen = Path(frozen_root).resolve()
    work = Path(work).resolve()
    src = frozen / "crates/jiaoyi-core/src"
    modules = re.findall(r"^(?:pub )?mod (\w+);$", (src / "research/mod.rs").read_text(), re.M)
    crate = work / "control-bridge"
    (crate / "src").mkdir(parents=True, exist_ok=True)
    paths = sorted(src.rglob("*.rs")) + [frozen / "Cargo.lock"]
    sources = {str(p.relative_to(frozen)): sha(p) for p in paths}
    source_hash = hashlib.sha256(canonical(sources).encode()).hexdigest()
    imports = "\n".join(
        '#[path = ' + json.dumps(str(src / "research" / (m + ".rs"))) + '] pub mod ' + m + ';'
        for m in modules
    )
    main = '''#![allow(dead_code)]
pub use jiaoyi_core::*;
mod research {
IMPORTS
    pub fn controls(e: &super::Envelope) -> Result<serde_json::Value,String> {
        if e.request.to != e.request.from + chrono::Duration::hours(4) {
            return Err("control requires four-hour window".into());
        }
        let (quotes, quality) = s3_ticks::snapshots(&e.request, &e.source)?;
        Ok(serde_json::json!({
            "always_long": seasonal::control(&e.request,&e.source,&e.control_bars,&quotes,true)?,
            "own_momentum": seasonal::control(&e.request,&e.source,&e.control_bars,&quotes,false)?,
            "quality": quality,
            "admission_bps_per_side":24.0,
            "entry_offset_minutes":60,
            "close_offset_minutes":240,
            "sl_tp":false
        }))
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    request: research::s3_signal::Request,
    source: research::s3_ticks::Source,
    control_bars: Vec<research::seasonal::M30>,
}
fn main() -> Result<(),Box<dyn std::error::Error>> {
    use std::io::Read;
    let path=std::env::args().nth(1).ok_or("input required")?;
    let mut bytes=vec![];
    std::fs::File::open(path)?.take(32_000_001).read_to_end(&mut bytes)?;
    if bytes.len()>32_000_000 {return Err("source budget".into());}
    let e:Envelope=serde_json::from_slice(&bytes)?;
    println!("{}",research::controls(&e)?);
    Ok(())
}
'''.replace("IMPORTS", imports)
    (crate / "src/main.rs").write_text(main)
    (crate / "Cargo.toml").write_text('''[package]
name = "stage1-frozen-controls"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
chrono = { version = "0.4", default-features = false, features = ["clock", "std", "serde"] }
chrono-tz = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
thiserror = "2"
libm = "0.2"
jiaoyi-core = { path = ''' + json.dumps(str(frozen / "crates/jiaoyi-core")) + " }\n")
    if not (crate / "Cargo.lock").exists():
        shutil.copyfile(frozen / "Cargo.lock", crate / "Cargo.lock")
    env = dict(__import__("os").environ)
    env["PATH"] = "/workspace/.jiaoyi-env/cargo/bin:" + env.get("PATH", "")
    env["CARGO_HOME"] = "/workspace/.jiaoyi-env/cargo"
    cmd = ["cargo", "build", "--offline", "--manifest-path", str(crate / "Cargo.toml")]
    with (work / "control-build.log").open("w") as f:
        subprocess.run(cmd, env=env, stdout=f, stderr=subprocess.STDOUT, check=True, timeout=300)
    binary = crate / "target/debug/stage1-frozen-controls"
    receipt = dict(binary=str(binary), binary_sha256=sha(binary), frozen_sources_sha256=source_hash,
                   frozen_sources=sources, bridge_source_sha256=sha(crate / "src/main.rs"),
                   cargo_lock_sha256=sha(crate / "Cargo.lock"),
                   seasonal_source_sha256=sha(src / "research/seasonal.rs"),
                   helper_sha256=sha(__file__), new_broker_requests=0,
                   description="Unchanged Phase-4 seasonal controls; 24bp admission; +1h entry/+4h close; no SL/TP")
    if sources != {str(p.relative_to(frozen)): sha(p) for p in paths}:
        raise ValueError("Frozen source changed during bridge compilation")
    (work / "control-bridge-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return receipt


def run_controls(receipt, native_input, bars, output_path):
    """Run frozen controls on the same quote path and recorded opening M30 bars.

    native_input is the inner Input object used by tiered_pullback_replay.
    Missing opening-hour bars produce explicit native no-entry statuses.
    """
    binary = Path(receipt["binary"])
    if sha(binary) != receipt["binary_sha256"]:
        raise ValueError("Control binary hash changed")
    output = Path(output_path)
    output.parent.mkdir(parents=True, exist_ok=True)
    input_path = output.with_suffix(".input.json")
    payload = {"request": native_input["request"], "source": native_input["source"],
               "control_bars": bars}
    input_path.write_text(canonical(payload) + "\n")
    completed = subprocess.run([str(binary), str(input_path)], capture_output=True, text=True,
                               check=True, timeout=120)
    value = json.loads(completed.stdout)
    output.write_text(canonical(value) + "\n")
    return dict(controls=value, input_sha256=sha(input_path), output_sha256=sha(output),
                binary_sha256=receipt["binary_sha256"])


def control_score(rows, name, cost):
    """Summarise executed fixed-horizon controls without hiding vetoes/censors."""
    values = []
    statuses = {}
    entries = censored = 0
    for row in rows:
        control = row["controls"][name]
        status = control["status"]
        statuses[status] = statuses.get(status, 0) + 1
        trade = control.get("control")
        if trade is None:
            continue
        entries += 1
        if trade["censored"]:
            censored += 1
        elif str(cost) in trade["net_by_bps"]:
            values.append(trade["net_by_bps"][str(cost)])
        elif str(float(cost)).removesuffix(".0") in trade["net_by_bps"]:
            values.append(trade["net_by_bps"][str(float(cost)).removesuffix(".0")])
        else:
            raise ValueError("Missing requested control cost sensitivity")
    gain = sum(v for v in values if v > 0)
    loss = -sum(v for v in values if v < 0)
    return dict(episodes=len(rows), statuses=statuses, entries=entries, closed=len(values),
                censored=censored, wins=sum(v > 0 for v in values),
                net_flat=sum(v == 0 for v in values), mean_net_r=sum(values)/len(values) if values else None,
                profit_factor=gain/loss if loss else None, net_r_values=values,
                admission_bps_per_side=24.0, repricing_bps_per_side=cost,
                risk_geometry="opening-hour range; fixed window close; no SL/TP")


def identity():
    """Identity frozen by the parent plan before any new tick collection."""
    path = WORK / "control-bridge-receipt.json"
    receipt = json.loads(path.read_text())
    if sha(receipt["binary"]) != receipt["binary_sha256"]:
        raise ValueError("Frozen control binary changed")
    if sha(__file__) != receipt["helper_sha256"]:
        raise ValueError("Control helper differs from pre-collection receipt")
    if any(sha(FROZEN / p) != h for p, h in receipt["frozen_sources"].items()):
        raise ValueError("Frozen control computation source changed")
    return {k: v for k, v in receipt.items() if k != "frozen_sources"}


def evaluate(inp, job, bars, work, freeze_receipt):
    """Parent runner API; cached outcomes are checked against their input hash."""
    ident = identity()
    if inp["request"]["symbol"] != job["symbol"]:
        raise ValueError("Control symbol mismatch")
    series = bars[(inp["request"]["account"], job["symbol"])]
    from datetime import datetime
    start = int(datetime.fromisoformat(inp["request"]["from"].replace("Z", "+00:00")).timestamp()*1000)
    opening = [series[t] for t in (start, start+1_800_000) if t in series]
    path = Path(work) / ("controls-" + job["episode_id"] + ".json")
    receipt_path = path.with_suffix(".receipt.json")
    expected = {"request": inp["request"], "source": inp["source"], "control_bars": opening}
    input_hash = hashlib.sha256((canonical(expected)+"\n").encode()).hexdigest()
    if path.exists() or receipt_path.exists():
        receipt = json.loads(receipt_path.read_text())
        if receipt["input_sha256"] != input_hash or receipt["output_sha256"] != sha(path) or receipt["binary_sha256"] != ident["binary_sha256"]:
            raise ValueError("Cached control result identity mismatch")
        values = json.loads(path.read_text())
    else:
        output = run_controls(ident, inp, opening, path)
        values = output.pop("controls")
        receipt = output
        receipt_path.write_text(canonical(receipt)+"\n")
    return {**values, "receipt": receipt,
            "comparison_scope": "Supplementary fixed-horizon controls; 24bp admission, +1h entry, no SL/TP. Entry timing and risk geometry differ from candidate."}


def summary(rows):
    return {name: {str(cost): control_score(rows, name, cost) for cost in (3.74, 24)}
            for name in ("always_long", "own_momentum")}
