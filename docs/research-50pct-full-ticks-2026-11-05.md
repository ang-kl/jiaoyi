# Full-tick preparation follow-up and economic rejection

№18,244 §§1–5 ·5November2026,14:28SGT ·by <Codex (GPT-6), effort: not exposed>.

The owner confirmed50% as an interim research target. The original live goal
remains73% net wins and1:1.38 per strategy/account. PR155 implemented the first
[frozen experiment](research-50pct-milestone-2026-11-05.md). This follow-up changes
research preparation, not live execution rules.

## Why a second cycle was necessary

The first cycle covered12 BTC and10 NatGas four-hour development episodes,
8–19September2025. Sampled-range breakout produced BTC1/2 wins (50%) but
-0.834211R, PF0.040518. That is neither enough evidence nor a profitable result.
NatGas lacked one or more of the four precise preparation marks in9/10 episodes,
although its windows contained other broker ticks. Failed-breakout candidates
produced no entries. None passed development; validation remained unopened.

A separate [amendment](research-50pct-full-ticks-2026-11-05.json) was frozen before
scoring the new variants. It retains the failed variants and their trial count.
The new pair uses every fresh, noncrossed, grouped midpoint within the first
120minutes, requiring an observation in each30-minute block. This captures
observed between-mark extremes and avoids requiring four exact-time quotes.
It is still an observed-path range, not certified complete broker history.

Entry marks, C<R,24bp/side costs, Q1.38, minimum fixed volume, one entry,
partial/whole target planning, original-R trailing and the four-hour close are
unchanged. Costs or targets were not reduced. The same cached22 episodes were
used; no additional broker download was needed. Both old hypotheses remain
visible; all66 old baseline/sampled episode results reproduce exactly.

The scorecard additionally requires positive total net price-unit P&L. A wider
stop changes the denominator of R and can otherwise make risk-normalised results
look better while larger monetary losses remain. PF in both units is reported.
Neither unit is a certified account return; no historical account/margin state
or currency conversion is reconstructed.

## Results under the frozen assumptions

| Candidate | BTC | NatGas |
|---|---|---|
| Sampled-range breakout |1/2 wins; -0.834211R; PF0.040518|No entries|
| Sampled-range failed breakout |No entries|No entries|
| Full-tick-range breakout |1/3 wins (33.3%); -1.360327R; PF0.022980|0/3 wins; -1.980942R; PF0|
| Full-tick-range failed breakout |No entries|No entries|

All four candidates fail development; no validation was opened. Alternative
strategies share the same episodes and are not independent observations. The
original S3 baseline had no entries in these partial-session windows; this is
not a full-day S3 performance result. Goal progress from simulated trades is0.

For the six full-tick breakout entries, a post-mortem inspected the best recorded
executable-side exit through the episode end while retaining the frozen fees.
Even this optimistic, hindsight-only bound was nonpositive for2/3 BTC and3/3
NatGas entries. It ignores stop/target constraints and does not prove missing
quote coverage. It is not a trading strategy. Conditional on the observed path,
entry, horizon and cost scenario, changing exits alone cannot make those five
cases net winners. Five of six simulated exits were session-end closes; one
NatGas trade gapped through its stop.

The live cost endpoint contains only one live S3 BTC entry (reported entry
slippage3.5147bp); NatGas S3 rows are demos. Entry/exit reference types and sparse
live coverage do not establish a calibrated replacement for24bp/side. The24bp
figure is a stress/reserve scenario, not an assertion of an actual broker fee.
No cheaper assumption is substituted to manufacture50%.

## Next evidence needed

The acute research gap is cost relative to the available move and holding horizon,
plus asset/session-specific predictive information. A follow-up should first
calibrate entry/exit execution costs with consistent reference quotes and adequate
live coverage, and examine liquidity/range by asset and session without selecting
windows by their eventual P&L. Only then preregister a new entry/horizon hypothesis.
Repeated SL/TP tweaks on these outcomes do not establish an edge. Validation and
the separately sealed period remain protected from repeated tuning.

Engineering checks: new sparse-preparation/between-mark-extreme/missing-block test;
existing long/short causal and censored-exit tests; scorecard regression preventing
variable-R concealment of losses; exact parity for66 previous results. Required
CI and deployed receipt are recorded on the PR. Financial target: NOT MET.
