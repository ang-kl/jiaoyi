# Three-year pilot execution contract and evidence corrections

№18,324 §§1–9 · 07-11-2026 SGT (+08:00). By Codex (GPT-6), effort not exposed.
Actual freeze timestamp, plan SHA and executable SHA are recorded before new broker reads.

## §1. Authorised outcome and defects in the supplied runner

The owner explicitly requested `doc/Next_Step_Agent_Prompt_18324_2026_11_07.md`
in a fully automated loop. Its embedded JSON/Markdown are preserved verbatim in
`docs/three-year-seasonal-plan-2023-2025.json` and the corresponding Markdown.
They are proposals, not verified market evidence. This addendum supersedes their
unsupported claims and operational ambiguities before collection.

The embedded runner assigns wins by symbol/job number, fixes profit factors and
control returns as literals, ignores unsuccessful collection, never calls a replay,
never increments gaps, and declares a pass regardless. Its f-string is also broken.
Its CI loop reads `.state` where CheckRun uses `status`/`conclusion`; null checks
must never mean green. These defects are replaced, not executed as market research.
The final block uses an actual SHA256, actual merge commit and measured results.

Claims of “100% wins”, high-alpha tiers, exact spread drag, seasonality, institutional
participation and empirical slippage calibration are unverified here. No exclusion
is applied to the live universe. No summer/illiquidity causality is asserted.
One or two weeks per year are not whole-year evidence. 15/20 is 75%, exceeding the
unchanged 73% goal; reference R:R 1:1.38 remains unchanged per strategy/account.

## §2. Exact scope, sealed dates and retained prior evidence

Only Stage 1: six specified winter/summer cohorts, 30 dates, four symbols = 120
logical episode jobs. Stage 2's 480 jobs is not launched. The report serial/path is
№18,325, superseding stale №18,322 placeholders in the source JSON.
No validation data from 19 August through 5 September 2025, including warm-up.
All requests end before 24 October 2025. Use demo 49236051 read-only operations.

Reuse the ten previously spent US500 2024 winter/summer sources by frozen file and
receipt hashes: nine successful sources and one failed/uncertain job. Missing M30
from those sources stays missing. Never recollect these failures or silently repair
prior preparation. The other 110 jobs are new windows/dates/instruments. Oil's new
13:00 window differs from the previous 07:00 four-hour pilot. This new study does
not erase the earlier ORB-60 stop-tuning finding; it is an explicitly requested new
comparison, with changed window/filter scope and retained prior evidence.

## §3. Source windows and budgets

US500: 09:30 America/New_York, DST-aware, through four hours later. XAUUSD: 07–11 UTC.
Oil ORB/momentum/control window: 13–17 UTC. Native oil S3 starts 07:00 UTC: collect
07–17 UTC so its entry/veto/position state is reconstructed from the native open.
This earlier six-hour history is a declared prerequisite of the same logical job,
not an unreported second pilot. S3 keeps its real 20:00 UTC session close, so any
position still open at 17:00 is censored, not artificially closed. S3 and controls
therefore have different windows/exit policies; comparisons are conditional strategy
comparisons, not isolated direction tests or account parity certification.

45 calendar days of positive-volume M30 OHLC preparation, segmented into <=7-day
intervals without the cTrader `count` field. Components shared by symbol/cohort.
The manifest declares every segment before reads. Raw sources stay outside Git.
Limits: 120 logical jobs; per fresh job 120 tick pages, 200,000 combined updates,
240 seconds total, 32 MB; 384 MB combined source limit. Metadata reads are separate.
At most three attempts for enumerated transient read failures, within the same
page/row/time allowance. Retain every attempt, including error/status, and mark
any retried component/episode as flagged even if it later succeeds. No retries of
truncation, malformed data, unknown errors or prior spent sources. Auth failures
stop execution. In-flight uncertain receipts are never automatically repeated.

## §4. Models, causal preparation and peer limitations

S3 calls the existing v7 band/mark/durable-veto/dispatch/target/exit logic with
reconstructed first-30-minute spread and opening M30 volume. No peer gate is added
to production S3; the source description incorrectly implies one already exists.
Current minimum contract, zero extra commission and recorded quotes remain
scenarios; historical contracts, latency, fees, margin/Kelly and portfolio state
are not reconstructed. HTTP/live validators and permissions remain unchanged.

ORB-60 uses complete first-hour M30 high/low and range >1.25×the median of exactly
20 prior registered sessions at the same research opening time (13 UTC for oils).
No skipped missing prior dates, interpolation or after-outcome holiday adjustment.
At half-hour marks from hour one, first fresh two-sided quote within five seconds
must break the range; next strictly later fresh quote within that mark window
rechecks direction/cost. Original R is the range rounded to contract grid; one
entry; cost veto consumes the episode. Shared Q1.38 targets, fixed-stop and original-R
30-second trailing exits; ORB closes at the four-hour window end.

Macro-180 is a **new, frozen research hypothesis**, not replication of an unspecified
“proven 100%” rule. One decision at hour three: current midpoint minus window-opening
M30 price, with the same sign in its class peer's six completed M30 bars. Oil peers
are SpotCrude/SpotBrent; no other class peer exists in the authorised four-symbol
pilot. Gold and US500 therefore report `missing_required_class_peer`; an empty peer
set never passes unanimously. No additional symbols or jobs are invented. Original
R is the first-three-hour M30 range; same five-second dispatch, cost veto, targets,
SL/TP/trailing and four-hour close. Peer data must be available by the decision.
No future peer price or synthetic macro news is supplied.

Dynamic threshold is an additional range filter for ORB/macro, not a substitute
for fees: R > max(2.5×prepared spread, 0.15×ATR14). ATR14 is mean true range of the
last 14 completed positive-volume M30 bars plus a preceding close. Session gaps
are bridged by true range, never fabricated bars. Missing ATR blocks the candidate.
Native S3 remains unchanged. Primary admission remains cost < R at 24 bp/side.

Always-long and own-momentum controls use hour-one direction/next quote, first-hour
M30 R, primary admission and a four-hour fixed-horizon close without stops. These
are the previously tested controls. Every episode retains both, including vetoes.

## §5. Cost accounting and denominators

Spread counted once through bid/ask fills. Primary 24 bp/side. Sensitivities 2 and
3.74 bp reprice identical admitted fills/geometry; neither is certified empirical
cost, neither creates new entries, neither can override primary failure. PF is on
normalised R, not mixed-account dollars. Price-unit totals remain per symbol.
Every failed, vetoed, missing, censored or retried episode stays in coverage counts.
Win rate is wins/all closes, including breakevens; missing metrics are null.
No pooling models, fixed/trailing arms or copied observations as independent trades.

## §6. Gates and stopping

Use stricter applicable criteria: >=10 closes/symbol, >=20/class, >=4 distinct
closed cohort weeks, >=50% net wins, mean R >0.05, PF >1.25, positive price-unit P&L
per symbol and positive primary mean in each of 2023/2024/2025. Zero source,
preparation, retry or sequence flags. Explicitly report economic gates separately
from software checks. No claim that a 50% hit rate alone establishes an edge.
Matched candidate-minus-control R uses same-symbol/date closed pairs; unmatched
cases stay visible. Exact one-sided weekly sign-flip p<=0.05 versus both controls,
>=4 paired weeks; adjusted six tests (three candidates × two controls) by Bonferroni
for any overall superiority claim. This six-week pilot may lack inferential power.
A negative/nonpositive completed mean in both 2023 and 2024 stops tuning that
candidate under this plan; missing/censored results are inconclusive. It does not
prove that all possible rules lack an edge. Never auto-open validation or live gates.

## §7. Automated delivery and focused verification

Merge source plan, this contract, immutable 120-job manifest, real runner/adapter
and focused tests before collecting new prices. Runner verifies merged plan PR,
freezes sources/binary and receipts, then collects/replays/reports all jobs,
commits Markdown/HTML/JSON/journal, opens report PR, waits for explicit success of
`check`, `docker`, `engineering`, and merges the exact tested head. Empty/pending/
failed/unknown checks never pass; CI failure is preserved for repair, not bypassed.
After merge verify published artifact hashes and print the actual final review block.
No Telegram, model API, order or account mutation is part of the runner.

Three focused checks: scope/seal/cache and explicit CI success; real causal
macro/ORB/exit behavior without future-data influence; failed/missing outcomes and
independent summary/receipt arithmetic. Existing native research regressions remain.
