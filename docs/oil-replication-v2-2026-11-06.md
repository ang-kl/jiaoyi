# Execute the bounded oil replication

№18,287 §§1–6 · 06-11-2026 16:13:40 SGT (+08:00).
By Codex (GPT-6); effort setting not exposed. Owner authorises execution of №18,286.

## Frozen amendment

[Plan](oil-replication-v2-2026-11-06.json), SHA256
`d35c0ff1b6379d305ee4f6294fe362a3e849121a618280c0ed269cb4d6efedfb`,
committed at `7e7d407` before collection. This is the single plan for new oil
collection. Earlier plans, supplied attachments and outcomes remain unchanged.

Primary: own momentum. Controls: unchanged peer continuation (flat peer skips;
no fallback) and always-long. Twenty episodes on ten frozen dates, two oil
symbols, native volume 100, demo account 49236051, 07:00–11:00 UTC. No index or
metals collection, validation opening, live rule or permission changes.

Admission and target geometry retain 24bp per side plus causal spread. Realised
2/3.74bp sensitivities hold entries and geometry fixed; neither is calibrated.
Additional commission is an explicit unknown modelled as zero. The native Rust
target planner and exit engine handle SL/TP/partial/runner and 30-second trailing.
This is scenario research, not qualified account returns.

## Run in order

```sh
cargo build -p jiaoyi-core --example class_prediction_study --example context_exit_replay --locked
python3 -m unittest discover -s ops/research -p 'test_*.py'

# Supply JIAOYI_UI_PASSWORD privately through the environment.
# Use one persistent evidence directory for this plan. No raw tick files in Git.
python3 ops/research/oil_replication.py collect --output /path/to/oil-evidence
python3 ops/research/oil_replication.py preflight --output /path/to/oil-evidence \
  --verifier /path/to/target/debug/examples/class_prediction_study \
  --replay /path/to/target/debug/examples/context_exit_replay
```

The first collection submits only the first date's two symbols. Technical
preflight verifies exact source requests, bounded bid/ask ticks, hashes, current
contract compatibility, original-study native parity and full-exit preparation.
Legitimate cost/flat/change skips do not fail technical preflight. Missing or
censored evidence does. Reported quote-continuity flags remain explicit and fail
the development screen; they do not alone establish a collection protocol error.

Only after `preflight.json` reports `passed: true`, repeat `collect` at most nine
more times, two jobs per invocation. The adapter requires that preflight to
match the first two source files and the adapter hash. It submits at most twenty
jobs total, one active at a time, with no failed-job retries or replacement dates.
A durable reservation is written before each POST; an uncertain response blocks
resubmission. A local exclusive lock prevents concurrent invocations in the same
evidence directory. The server also refuses competing active jobs.

```sh
python3 ops/research/oil_replication.py analyse --output /path/to/oil-evidence \
  --verifier /path/to/target/debug/examples/class_prediction_study \
  --replay /path/to/target/debug/examples/context_exit_replay
```

Analysis can report a stopped or incomplete batch: uncollected/failed episodes
stay visible. It writes `results.json` and `Oil_Replication_Report.html` with
source/binary/adapter hashes, all fills, skips, cost scenarios and per-symbol
results. Previously inspected 5/12-August results and pooled diagnostics are
labelled separately and cannot rescue the standalone screen.

## Decision and stopping rules

The primary 24bp trailing candidate needs at least five closes across three ISO
weeks **per symbol**, at least 50% wins, positive expectancy and PF above one at
class and symbol levels, positive per-symbol price-unit totals, and positive
matched R difference versus always-long at class and symbol levels. A zero-loss
sample with positive profits passes the profit/loss comparison but reports PF
as null, not a manufactured infinity. All source/coverage gaps and censoring
fail the screen. Matched controls include only episodes closed in both arms;
unmatched episodes stay visible in the full result.

Stop on source/identity/contract mismatch, failed preflight, uncertain POST,
source-size limits or budget exhaustion. Classify inadequate coverage/sample
as insufficient evidence separately from a complete losing screen. Even a
preliminary pass leaves validation closed in this command and requires separate
cost, historical-contract and prospective evidence before any promotion.

Three focused regressions cover frozen plan/source contracts, incomplete or
censored winning samples, and two-job/preflight/budget/uncertain-POST guards.
Existing native tests cover target/exit behaviour. Original 73% / 1:1.38 latest20
per strategy/account goal and interim 50% remain unproven until evidence changes.
