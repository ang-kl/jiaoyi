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
