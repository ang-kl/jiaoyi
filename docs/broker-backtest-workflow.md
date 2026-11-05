# Automated broker-data research workflow

№18,238 §§1–4 · 5 November 2026, 10:13 SGT · by <Codex (GPT-6), effort: not exposed>.
Owner chose an automated broker-data workflow in Jiaoyi rather than TradingView.
The prior two-breakout experiment remains parked pending economic evidence.

## What runs

`POST /owner/backtests` starts a bounded job; `GET /owner/backtests` returns the
latest job and result. Both use the existing owner Basic/Bearer authentication,
allowlist and no-store headers. Only BTCUSD and NatGas are accepted initially.
The worker opens a separate read connection, uses the stored current token,
checks the broker grant/environment, resolves the exact broker symbol and reads
contract metadata and M30 bars. It never refreshes/replaces tokens, subscribes
quotes, sends orders, modifies strategy settings or writes research data to SQL.

One job per service process, 240-second deadline, <=14-day study window, 50-day
calendar warm-up and one-day broker pages. Research runs below one historical
request/second on its connection. Unresolved `hasMore`, wrong range, invalid
bars or conflicting duplicates fail the job. Missing bars remain missing.
Only the latest job is retained in bounded process memory; a restart loses it.
The client checks the job ID and stops on replacement/restart rather than
mistaking another result for its own. Downloaded JSON retains the input bars,
contract identity, request assumptions, input/algorithm hashes and outcomes.
No persistent tick warehouse or paid model call is introduced.

## Scope of the first backtest

This is **fixed-horizon signal attribution**, not the full live S3 trade simulator.
It reuses the Rust band/history/session functions, studies every raw qualifying
mark, uses the following M30 open as entry-price proxy, and measures 30/60/120
minute and session-end returns (duplicate horizons collapse). Each complete
outcome requires all intervening bars. Both long and short directions charge
explicit spread, round-trip commission per price unit and per-side slippage.
Costs are scenarios, not certified historical realised costs. A single-price
bar cannot reproduce executable bid/ask ticks or the five-second entry window.

Session veto, one-entry limits, Kelly, sizing, SL/TP and trailing are intentionally
not applied: those rules would obscure whether the raw signal has information.
Overlapping marks/horizons are not independent trades, cannot be summed into a
portfolio return and do not qualify as the latest20 account goal window. Reports
say DIAGNOSTIC_ONLY_NOT_FINANCIAL_PASS. No signal filter or live rule is changed.
NatGas uses the existing weekday07:00–20:00UTC strategy session; BTC uses the UTC
day. Broker contract size and tick-volume caveats remain visible.

## Data boundary

The existing research seal remains enforced: all study and warm-up bars end by
2025-10-24T00:00:00Z. Requests crossing it are refused before broker access.
The original `/research/bars` route and financial trial gates are unchanged.
Example files use 6–10 October2025 development data, not a claim about current
market performance. This connection does not silently open the sealed interval
or reclassify the already inspected2026 TradingView examples as a fresh holdout.
A separately authorised current-period study scope is not implemented here.

## Run

Use the service's existing `JIAOYI_UI_PASSWORD` securely in the caller environment;
never put it into JSON or command arguments. The service uses its existing cTrader
credentials and database token. No new account linking or API key is needed.

```sh
python ops/research/broker_backtest.py ops/research/btc.example.json --output /tmp/btc-study
python ops/research/broker_backtest.py ops/research/natgas.example.json --output /tmp/natgas-study
```

Run sequentially. Edit only the account, historical range and explicitly labelled
cost scenarios when needed. The examples name the already allowlisted demo
49236051. BTC spread15 and NatGas spread0.004 are **scenario inputs**, not validated
2025 measurements; both use the inherited24bp reserve and zero commission scenario.
The HTML and JSON files are private local artifacts, written mode0600. No Telegram
message is sent. The script makes one submission and polls until completion;
it neither schedules recurring work nor requests AI sessions.

Reproduce a downloaded result offline using the same source version:

```sh
cargo run -p jiaoyi-core --example s3_signal_study --locked < /tmp/btc-study/result.json
```

The adapter checks the input-bar hash before recomputation. Costs remain in the
saved request. A failed job contains its stage name, never broker token text.
`token_grant_read_failed_no_refresh_attempted` means the normal service token
lifecycle must restore access; the research runner does not rotate credentials.

## Focused validation

Three pure-study tests cover next-open pricing and cost arithmetic, direction,
future-data isolation, missing outcome paths, NatGas sessions, duplicate bars,
range limits and insufficient history. One HTTP test covers authentication,
account scope and the seal before any database/broker operation. One fake-broker
test confirms `hasMore` refusal and no orders/amends/closes. Clippy and required
repository CI remain release gates. Actual deployed BTC and NatGas result
receipts must be recorded separately; successful compilation is not a broker
connection or financial PASS.

## Bounded BTC / NatGas tick replay

№18,241 §§1–5 · 5 November 2026, 10:50 SGT · by <Codex (GPT-6), effort: not exposed>.

Supply `tick_replay: {"volume": ...}` in the existing owner-authenticated job.
`btc.ticks.example.json` and `natgas.ticks.example.json` start with four-hour
**development** windows. Volume is broker cents of units (BTC 1 and NatGas
10000 in the current demo metadata); it is not lots or inferred account sizing.
The normal client creates the HTML report and JSON; the same offline Rust
example verifies the source checksum and recomputes the tick study:

```bash
python ops/research/broker_backtest.py ops/research/btc.ticks.example.json --output /tmp/btc-ticks
cargo run --locked -p jiaoyi-core --example s3_signal_study < /tmp/btc-ticks/result.json
```

This runs in the existing **jiaoyi** service, using its separate read-only
broker session. No engineering worker, schedule, paid model call, account link,
order writer, token refresh, live strategy change or research SQL archive is
introduced. The previously established development-data seal remains in force.
One owner job at a time; latest job only, in memory; download before replacement
or deployment. Source ticks stay private and out of Git.

Limits: start at the registered UTC session open; maximum one session / 24
hours; 50 calendar days of M30 warm-up; at most 120 tick requests and 200,000
returned bid+ask rows; overall timeout 240 seconds. Each historical call is
followed by 750 ms pacing. Truncated tick responses are discarded and their
interval bisected into disjoint inclusive windows. A truncated single
millisecond, empty `hasMore`, decode failure, cap or timeout fails the job;
partial output is never presented as complete. Shorten the end time to fit a
budget, keeping the same session start. Never split a day into independent
fresh-state runs and then add the results.

The pure replay reuses S3 session/band, cost-reserve and target arithmetic.
It evaluates the first fresh quote within each five-second decision window,
retains C≥R for the rest of that session, and consumes one entry per session.
It models entry on the next strictly later fresh quote before the mark deadline,
with direction rechecked. This dispatch/fill timing is a stated approximation.
Current 24 bp/side reserve is fixed; spread and commission are explicit scenarios.
The original rounded R and target distances are frozen at decision time and
shift with the simulated fill, as relative brackets do.

For a fixed entry, the report compares fixed stop versus the current original-R
trailing method: a 30-second proposal interval and no new proposals in the final
30 minutes. Stops never loosen. Old server brackets precede pending amendments
and TP1 closes; acknowledgement/market-close occurs on the next snapshot.
Whole-minimum positions use TP1 as broker target; split positions use the shared
partial/runner planner. Gaps through stops fill at the observed executable side;
targets fill at their level. No real liquidity or acceptance claim is made.
Partial windows censor remaining legs; only a full-session run uses the five
seconds of extra history to attempt a session-end close.

Separate bid/ask histories cannot reconstruct cross-side event sequence.
Same-millisecond updates are grouped into one final snapshot and counted;
within-millisecond extremes may be lost. Stale sides (>6 seconds), long event
intervals, crossed quotes and initially missing sides are reported. Quiet time
is not proof of a missing broker tick. Quote continuity remains unproven even
when API pagination completes. Gross and cost-adjusted values are **scenarios**;
qualified net R stays null. Spread is already in bid/ask fills and is not deducted
twice. Slippage is a post-fill adverse cost scenario, not a path/trigger change.

Missing from account parity: historical contract metadata, actual original
volume/Kelly/margin state, portfolio conflicts, manual trades, funding/retries,
restarts, latency, slippage, broker stop-distance rejection, swaps and currency
conversion. Current contract metadata is exported explicitly. This deliverable
is an execution-path diagnostic, not an account backtest or edge certification.
It contributes **zero qualifying trades** towards ≥73% wins / 1:1.38 R:R over
20 positions per strategy/account. No candidate rule is promoted by this PR.

Verification covers both long and short partial/trailing paths, causal entry,
per-session veto, censored positions, malformed/stale/crossed input and capped
broker pagination preserving duplicate ticks without any order writes. Final
CI and deployed two-symbol receipts belong on the PR; compilation alone does
not prove historical broker availability or an economic edge.
