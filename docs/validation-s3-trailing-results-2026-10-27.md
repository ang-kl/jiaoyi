# S3 trailing validation: recorded-path results

Document VAL-S3-20261027-RESULTS. Recorded 27 October 2026, 16:36 SGT.

The bounded offline audit and four-case recorded-path comparison are complete.
One illustrative live-account path improves under the candidate; the three
same-episode demo paths do not change. **No comparison qualifies as a verified
gross or net result. Trailing's financial usefulness remains unproved.**

## Frozen method and actual execution

The [contract](validation-s3-trailing-2026-10-27.md) was frozen at
`f5f21dd7831cb81a916481ec33d5f4a7e8e1e0a9`. Clarified session timing and the
implementation were pinned at `157794c8a4404a54fc10e518db48cf4b0b03fd51`,
committed at 16:33:27 SGT, before the real replay receipts at 16:35:48 SGT
(08:35:48 UTC). One fixed candidate, `s3-initial-risk-trail-v1`, was compared;
no parameter search or outcome-based retuning was performed.

The baseline retains saved entry/stop, TP1 partial/runner geometry and session
close. The candidate uses the same entries and volume, trailing one original
entry-to-stop R behind the best executable bid for a long or ask for a short.
Every modeled partial/exit is weighted by its share of original whole-position
volume. Tightening the stop never redefines R. Sixty-second proposal cadence,
next-quote TP1/amend effects, protective-side recheck, old broker bracket first,
adverse stop gaps and conservative target fills are explicit simulation
assumptions. Broker amend acceptance/latency is not demonstrated by this replay.

A trusted operator streamed existing production quotes through the pure JSONL
example, compiled under the isolated engineering UID in Railway deployment
`74d72a91-fcd7-45a0-ab14-7effab177bbd`. Source access was bounded/read-only.
No raw-tick archive, new production table, broker request, model call or live
candidate order/amend was created for this comparison. Normal engineering
startup metadata was restored afterward; the coordinator resumed running.

## Cohort and qualification

The frozen cutoff is 27 October 2026, 16:15 SGT / 08:15 UTC. The audit retains
all 75 selected owned closed positions: live 15 and 20 on each of three demos.
All 75 had unique owned opening provenance, saved order/session evidence,
provable full opening and matching closed quantity in the retained audit.
All 75 remain **strict UNKNOWN** because original cached limits/grid and
historical recorder no-loss proof were not persisted. Current symbol rules
cannot substitute for original entry-time evidence. Opening-plan/broker-TP
confirmation is also not promoted to verified by the scenario inputs.

A deterministic descriptive subset selects one fully filled case per account
whose known session ended before cutoff. All four are AUS200, across only two
market entry episodes. This subset is a source-usable illustration, not a
representative four-trade sample or four independent market trials. The 75-case
inventory and its unknown qualifications remain retained; the other cases are
not silently discarded from an alleged successful backtest.

## Conditional gross scenarios

These are **unqualified recorded-quote gross R**, rounded to six decimals.
They are neither canonical broker outcomes nor net profit estimates.

| Account | Baseline gross R | Candidate gross R | Candidate − baseline R | Simulated trail applications | Observed adjacent recording-session ID switches |
|---|---:|---:|---:|---:|---:|
| Live 43002148 | −1.023952 | +0.538922 | +1.562874 | 15 | 44 |
| Demo 46130058 | +0.878947 | +0.878947 | 0.000000 | 24 | 0 |
| Demo 47790949 | +0.878947 | +0.878947 | 0.000000 | 24 | 0 |
| Demo 49236051 | +0.878947 | +0.878947 | 0.000000 | 24 | 0 |

The live baseline and candidate close on adverse quote gaps through their
respective stops; both carry `gap_before_quote`. The source's 44 switches count
changes between adjacent ordered rows' `session_id`, **not 44 proven process
restarts or independent lost intervals**. The producer resets carried bid/ask
on each switch and marks the changed full-quote path as gapped. Overlapping
recorders/deployment draining are possible explanations, not established
causes. This makes the apparent live improvement especially conditional.

All three demo paths end at the modeled session close with the same R. Their
zero observed session-ID switches do not prove loss-free continuity. The
maximum observed carried-side age over each queried horizon ranged from
6.650975 to 6.662287 seconds. Quote age, ordered sequence and a min/max time
range cannot certify missing-message absence. Entry timing uses the observed
opening-execution receipt's received time, not a claimed exact broker matching
engine execution clock.

For all four cases and both arms, `qualified_gross_r` and `net_r` are null;
qualified and net paired deltas are also null. Quote continuity, original plan
and original grid remain unproven. Current-grid use is explicitly assumed.
Separate counterfactual cost inputs were unavailable and were not filled with
zero. Executable bid/ask already incorporates spread; a later cost model must
not charge it twice. Fees, swap, slippage and changed exit timing would need
evidence for each arm. Recorded-quote baseline fills differ from actual broker
fills; these scenarios never replace canonical broker net R.

## Checks, completion and next evidence

The prepared software has ten passing pure replay regressions, a compiled
bounded JSONL example, three passing recorder-provenance PostgreSQL fixtures,
four existing recorder checks and the passing actual S3 entry-metadata test.
Workspace/all-targets locked Clippy with warnings denied and formatting passed.
The two previously unavailable S1 SQLx checks passed after local PostgreSQL was
restored; their earlier setup failures remain recorded in the
[mechanism audit](verified-trading-mechanisms-2026-10-27.md). Independent source
review found no critical blocker to this bounded offline work. These checks
establish software behavior, not investment acceptance. Required exact-head CI
and live receipt verification for any release remain separate.

Completed: frozen-method audit, retained 75-case readiness inventory,
conditional four-case comparison and explicit qualification gaps. This does
not complete the full eight-stage quant workflow, overall handover or financial
goal. Forward entry-time limits/session metadata and recorder START/GAP evidence
are prepared to improve later source qualification; they cannot retroactively
certify the current sample. No extra raw archive or speculative live policy
change is required to make the present negative/unknown verdict concrete.

Still open: original-plan/grid and source continuity qualification, supported
arm-specific net costs, unseen forward evidence, and the registered S3 forward
acceptance tests. The existing registration requires 200 forward demo trades,
positive mean net R with week-block bootstrap lower bound above zero,
random-direction control comparison and PF at least 1.3. The separate owner
goal remains latest 20 per strategy/account, at least 73% net wins (15/20) and
risk:reward objective 1:1.38; planned geometry and realized payoff are distinct.
No winner aggregation or projected win rate is inferred from these four paths.

I-1..I-11, original entries/pre-orders, ownership, one order gate, broker
protection, size/risk/account rules and owner-only decisions remain. No live S3
trailing was activated, and old H1/H2 R1/R2 research was not resumed. A later
live exit change needs its own concrete evidence, registration and owner
instruction.

Private minimal source receipts: `s3-validation-cohort.json` and
`s3-recorded-path-results.json` under `/workspace/.jiaoyi-env/`. This document
publishes summary evidence only; raw prices and position IDs are omitted.
