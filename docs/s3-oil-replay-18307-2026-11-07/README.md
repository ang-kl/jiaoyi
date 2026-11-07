# Prepared S3 oil replay — first implementation

№18,307 §§1–5 · 07-11-2026 08:03 SGT.
By Codex (GPT-6), effort not exposed.

## §1. Delivered

The offline `production_s3_replay` example runs SpotCrude/SpotBrent through the
existing S3 band, target and full-exit engine. It requires explicit opening-bar
volume and preparation/cost evidence, uses the production 45-day history view,
and rechecks the registered cost reserve at the next executable quote used as
a dispatch proxy. A newly failing C >= R vetoes subsequent session marks.
The decision's target/stop payload is unchanged. Legacy BTC/NatGas and class
candidates retain their previous semantics.

The network backtest route refuses prepared current-S3 oil before collection:
its request schema cannot yet supply preparation evidence. This release is an
offline research implementation; production trading decisions are unchanged.

## §2. Usage and evidence

```sh
cargo run --locked -p jiaoyi-core --example production_s3_replay -- source.json > replay.json
```

`source.json` uses the retained broker-job shape: `status: "complete"`,
`request`, `result.input`, `result.input_sha256`, plus `preparation`:

```json
{
  "source": "immutable preparation receipt reference",
  "available_at": "2025-07-21T07:30:00Z",
  "opening_bar_tick_volume": 10,
  "spread": 0.02,
  "commission_round_trip": 0.0
}
```

These numbers illustrate the schema, not measured oil costs or permission to
fill missing evidence. Preparation must be available by the first mark, the
opening volume positive and cost values identical to the request. The reference
is caller-supplied, not independently authenticated. Input SHA-256 uses the
typed Rust source serialisation. The CLI checks status/hash, caps the source at
32 MB, preserves the research seal and emits algorithm/input fingerprints.
Native tick/page limits remain unchanged. No network or order API is used by
the executable. Current contract metadata remains an assumption unless separately
supported by historical evidence.

## §3. Verification and backtests actually run

[Verification JSON](verification.json) separates artificial fixtures from market data:

- Three new regression tests cover preparation/causal marks, final-cost veto,
  and long/short full exits with quality/censoring. All seven S3 tick tests pass.
- Synthetic Crude long and Brent short executable runs produce 24 full-session
  decision marks. Their returns are software fixtures, not financial evidence.
- Retained BTC and NatGas market replays exactly reproduce their stored complete
  studies. This checks compatibility; the old sample is not new validation.
- Core all-target Clippy passes without lint suppression. Required full CI
  accompanies the PR, including application compilation and integration tests.

## §4. Remaining specification work and honest limits

This is the first bounded implementation of the
[№18,306 specification](../s3-oil-replay-spec-18306-2026-11-07.md), not completion
of every acceptance case in its three groups. Account margin, Kelly, equity,
connection/revision changes, actual dispatch latency and acknowledgements are
not reconstructed. The next-quote proxy cannot certify broker execution.
Restart persistence and client-action freshness/acknowledgement boundary parity
still require dedicated evidence and fixtures; legacy gap flags remain intact.
The adapter does not claim those missing checks passed.

No genuine oil performance backtest was run: the old oil raw corpus is missing,
and the required causal opening/preparation evidence is unavailable in the
retained datasets. Recent stored oil ticks are outside the development seal.
Neither synthetic metadata nor a lower assumed cost may fill those gaps.
Financial qualification stays false, with qualified net fields unavailable for
unproven paths. The earlier twenty-job allowance remains spent.

## §5. Next milestone

Recover a permitted development source with the required preparation evidence,
or define a separately bounded evidence acquisition plan with explicit gaps.
Complete the remaining dispatch/account/exit parity fixtures before calling a
result production-faithful. Then freeze the development dates and assumptions
before measuring performance. New qualified trades: **0**. Interim 50% and the
original 73% / 1:1.38 latest-20 per-strategy/account goals remain unverified.
