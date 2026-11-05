# Interim 50% research milestone

№18,243 §§1–5 · 5 November 2026, 14:01 SGT · by <Codex (GPT-6), effort: not exposed>.

The owner authorised building and testing towards **50% as an interim research
win-rate target**, separately for BTC and NatGas. The original per-strategy/account
73% / 1:1.38 project goal is unchanged. This is research code, not an amended live
S3 registration, a risk-policy change or automatic promotion authority.

## Economic question and frozen candidates

The prior tick proof found C/R ≈4 in both first-mark examples, followed by the
registered full-session veto. Earlier removing that veto alone increased losing
activity. A new experiment therefore changes the price-structure definition,
not costs or the target win-rate denominator. Neither hypothesis is claimed to
have a documented BTC/NatGas edge. The cited S3 equity-paper results do not
transfer automatically to these assets or this four-hour horizon.

The machine-readable [plan](research-50pct-milestone-2026-11-05.json) was written
before candidate scoring. Runtime selectors map its descriptive candidate IDs
to `opening_range_breakout2h` and `opening_range_failed_breakout2h`.

Both candidates observe the first fresh midpoint at minutes30/60/90/120 after
the existing symbol's session open. Their min/max defines a **sampled-midpoint
range**, not intrabar OHLC extremes. Missing any preparation mark prevents entry.
At minutes150/180/210:

- Breakout: enter beyond the fixed range; original stop is its opposite boundary.
- Failed breakout: after an outside mark, enter oppositely only on return inside;
  original stop is the most extreme prior outside-mark midpoint. A missing mark
  resets this excursion. No retrospective selection of a successful reversal.

Stops are planned at the decision quote; the relative distance is frozen and
the simulated bracket shifts with the later fill, as in the existing replay.

Both require C<R at each eligible mark, one simulated fill per episode, the
existing Q=1.38 partial/whole target planner and original-R trailing. Current
24bp/side reserve, BTC15 / NatGas0.004 spread scenarios and zero commission
assumption remain fixed. Neither cost calibration nor actual broker fills is
established. Entry requires a next strictly later fresh quote in the same
five-second window. The episode closes at hour4, allowing five seconds of
coverage; missing exit data stay censored. This is a new exit horizon, not
full-session S3 parity. Volume is each current contract's broker minimum.

## Evidence discipline

Development:8–19September2025. Frozen development validation:22September–17October2025.
These periods remain before the existing seal. Prior baseline study dates6–10Oct
and the6Oct admission example have already been inspected; this is not a pristine
holdout. No claim of untouched confirmation is permitted. The separately sealed
research interval remains unopened.

Development advances only with ≥5 closed scenarios, ≥50% net wins, positive mean
net R and PF>1. That low count is only a screening trigger. Validation needs ≥20
closed positions across ≥3weeks, ≥50% net wins, positive mean net R and PF>1,
separately per symbol. Every expected episode must be present; censored positions
prevent a pass. Break-even closes stay in the denominator. Wilson95% intervals
are shown; observed50% is not evidence that the true win rate is above50%.

Both hypotheses and all losses remain in the output. No tuning on validation.
If development fails, validation stays unopened; diagnose the failure before
registering another candidate. No numerical target is guaranteed by repeated
coding or testing. Scenario positions never enter the live goal window.

## Run and reproduce

The existing authenticated broker job accepts the optional `entry_rule` inside
`tick_replay`; absence keeps current S3. Candidates use ticks only, avoiding a
50-day warm-up download for every episode. Existing duration/page/row/time caps
remain; no new service, cron, SQL archive, model call or trading command.

```bash
cargo build --locked -p jiaoyi-core --example s3_candidate_study
python ops/research/milestone.py collect --stage development --output /private/evidence
python ops/research/milestone.py analyse --stage development --output /private/evidence --binary /path/to/s3_candidate_study
```

First save the two baseline M30 downloads as `btc-m30/result.json` and
`natgas-m30/result.json` under the evidence directory. The example verifies each
original source checksum and account/symbol, then combines dated warm-up bars
with each tick episode in memory. Original and combined checksums remain in the
output. Duplicate episodes and changes to the frozen costs, volume or horizon
are refused. This does not certify historical contract or account-state parity.

Collection caches unchanged complete episodes. `--legacy-fetch --max-new-jobs 4`
may prefetch a bounded prefix using the already deployed default endpoint while
new code is in CI; it adds warm-up overhead but changes no quote data. A partial
prefix cannot pass the missing-episode gate. After deployment, normal collection
resumes and uses the faster candidate fetch. Validation collection requires a
passing development candidate and the unchanged plan hash.

Engineering tests and financial evidence are separate. Compilation, CI and
synthetic path tests verify implementation behaviour. The release receipt records
the actual development results, rejected candidates and any remaining evidence
gap. No live rule is promoted merely because engineering checks turn green.
