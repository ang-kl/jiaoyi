//! Streaming JSONL adapter for the pure fixed-entry exit comparison.
//! First line: {"input":Input,"coverage":Coverage,"costs":Costs}.
//! Remaining lines: one ordered full Quote each. No defaults supply costs or
//! qualification evidence. No database, broker, live orders or tick archive.
use std::io::{self, BufRead, Read, Write};

use jiaoyi_core::research::exit_replay::{Costs, Coverage, ExitReplay, Input, Quote};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_QUOTES: usize = 400_000;
const MAX_LINE_BYTES: usize = 65_536;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    input: Input,
    coverage: Coverage,
    costs: Costs,
}

fn line(reader: &mut impl BufRead, number: usize) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = Vec::new();
    let count = reader
        .take(MAX_LINE_BYTES as u64 + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| format!("stdin read failed at line {number}: {error}"))?;
    if count > MAX_LINE_BYTES {
        return Err(format!("line {number} exceeds {MAX_LINE_BYTES} bytes"));
    }
    Ok((count > 0).then_some(bytes))
}

fn run(reader: &mut impl BufRead) -> Result<Value, String> {
    let header = line(reader, 1)?.ok_or("missing first-line request")?;
    let request: Request = serde_json::from_slice(&header)
        .map_err(|error| format!("invalid first-line request: {error}"))?;
    let mut replay = ExitReplay::new(request.input)
        .map_err(|error| format!("invalid replay configuration: {error}"))?;
    let mut quotes = 0;
    while let Some(bytes) = line(reader, quotes + 2)? {
        if quotes == MAX_QUOTES {
            return Err(format!("quote limit {MAX_QUOTES} exceeded"));
        }
        let quote: Quote = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid quote JSON at line {}: {error}", quotes + 2))?;
        replay
            .push(quote)
            .map_err(|error| format!("invalid quote evidence at line {}: {error:?}", quotes + 2))?;
        quotes += 1;
    }
    Ok(json!({
        "comparison":replay.finish(request.coverage,request.costs),
        "quote_lines":quotes,
        "quote_limit":MAX_QUOTES,
        "line_byte_limit":MAX_LINE_BYTES
    }))
}

fn main() {
    let stdin = io::stdin();
    let result = run(&mut stdin.lock());
    let failed = result.is_err();
    let output = result.unwrap_or_else(|error| json!({"comparison":null,"error":error}));
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    if serde_json::to_writer(&mut stdout, &output).is_err()
        || stdout.write_all(b"\n").is_err()
        || stdout.flush().is_err()
    {
        std::process::exit(2);
    }
    if failed {
        std::process::exit(1);
    }
}
