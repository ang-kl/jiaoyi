# Handover to the next AI: assessment and proposed next work

№18,281 §§1–10 · 06-11-2026 13:28:34 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.
Evidence baseline: main `6eb26067799a77cf1f8b7bd297260924527a6491`, merged PR #168.
This file is in `/doc` as requested. The original reports remain in `/docs` and
are linked below; no duplicate dataset or ZIP is uploaded.

## §1. TLDR for the reviewing AI

**No candidate has demonstrated a deployable edge. Zero of the three candidates
passed the frozen development screen. Validation remains unopened.**

The useful result is a separation of three different problems:

- Oil has encouraging but insufficient outcomes; the peer signal adds no
  demonstrated value over own momentum on the matched sample.
- Metals have an admission/feasibility problem under the frozen assumptions;
  there are no executed candidate trades from which to estimate profitability.
- Index momentum loses money in this sample even under the optimistic cost
  sensitivity. Reaching 50% wins alone would not repair its expectancy.

The next work should obtain independent evidence where warranted and challenge
the mechanism where it fails. It should not optimise stops, fees or thresholds
against these same outcomes. This handover proposes work; it does not itself
start collection, reopen validation, alter live trading or queue worker jobs.

## §2. Read these sources before forming a conclusion

1. [Frozen experiment plan](../docs/full-exit-research-plan-2026-11-06.json).
   SHA256 `b202c9a5c9f4603576f48c9ea9dafc374a422581177beb7b32114a5edf14464b`;
   committed as `9384ccf` before this experiment's outcomes were calculated.
2. [Interpretation and reproduction instructions](../docs/full-exit-research-2026-11-06/README.md).
3. [HTML report](../docs/full-exit-research-2026-11-06/Jiaoyi_Full_Exit_Research.html).
4. [Complete derived JSON](../docs/full-exit-research-2026-11-06/full-exit-results.json):
   inspect `summary`, `symbols`, `matched_controls`, `rows`, source and code hashes.
5. [Local verification receipt](../docs/full-exit-research-2026-11-06/verification.json)
   and [PR #168 release closure](https://github.com/ang-kl/jiaoyi/pull/168#issuecomment-6677606365).
   The receipt records CI as pending **at commit**; the closure records the later
   green CI, merge and successful Railway deployment.
6. [Python orchestration](../ops/research/full_exit_research.py),
   [Rust offline adapter](../crates/jiaoyi-core/examples/context_exit_replay.rs),
   [existing target planner](../crates/jiaoyi-core/src/strategy3/targets.rs),
   [existing exit engine](../crates/jiaoyi-core/src/research/exit_replay.rs),
   and [journal](../log/CODEX_Log.txt), §LOG.103.

The raw source jobs are not included in this handover. A full independent replay
requires the original 40 files and collection receipt. Inspecting the JSON or
rerunning synthetic tests alone is not an independent raw-data reproduction.
If those inputs are unavailable, say so explicitly.

## §3. What the evidence actually says

Primary results below use trailing exits and 24bp slippage per side. Additional
commission is modelled as zero with its true value unknown; all qualified net
returns remain null. These are research scenarios, not account performance.

| Candidate | Closed / wins | Mean scenario net R | Other episodes | Assessment |
| --- | --- | --- | --- | --- |
| Oil peer continuation | 2 / 2 | +0.3264 | One flat predictor; one cost veto | Both wins occur on 5 August. Identical to matched own-momentum control. Insufficient closes and independent dates. |
| Metals unanimous peer reversal | 0 / 0 | Unavailable | Two peer disagreements; two cost vetoes | No admitted trades. Neither success nor failure of the executed return distribution is established. |
| Index own momentum | 6 / 1 | −0.7670 | One censored position, one missing fill, six cost vetoes | Fails profitability, per-symbol and coverage gates. |

At 2bp per side, index momentum has **3/6 wins, mean −0.0501R and PF 0.80**.
At 3.74bp, it has 2/6 wins and mean −0.1068R. The optimistic sensitivities retain
the same 24bp admission/target geometry; they do not test a new low-cost entry
policy. A critic must preserve that distinction.

The corpus has two already-inspected dates, 5 and 12 August 2025. Eleven target
symbols yield 22 target episodes; 52 candidate/control rows include 20 admitted
entries. Repeated controls, cost scenarios and exit arms do not increase the
independent sample size. Correlated instruments on one date are particularly
weak evidence of repeatability.

## §4. Assessment: facts, inference and unresolved causes

**Supported facts:** the frozen candidates do not pass. Oil's peer and own
momentum make the same matched decisions. Metals are skipped. Index expectancy
is negative even at 2bp under the fixed geometry. Missing and censored episodes
are retained rather than converted into imaginary closes.

**Provisional inference:** oil own momentum deserves a bounded replication before
adding peer complexity. For metals, representative cost/entry-feasibility data
is more useful than more exit tuning. For indices, the current first-hour
direction rule lacks demonstrated predictive value; a different, economically
justified feature or regime hypothesis would need a new frozen experiment.

**Not established:** a fundamental edge, calibrated historical or live costs,
robustness across regimes, live account profitability, or a general failure of
AI-assisted coding or quantitative trading. These results concern specified
rules, geometry, costs and dates. They do not identify a unique universal cause.

The hypotheses were selected after earlier diagnostics on these development
dates. Freezing this run before its full-exit outcomes prevents within-run
retuning; it does **not** make previously seen dates an untouched holdout.

## §5. What the engineering checks establish—and their limits

The adapter reuses native Q=1.38 targets, original-R stops, TP1/runner logic,
minimum-volume handling and the existing 30-second trailing cadence. Executable
bid/ask spread is charged once. Slippage scenarios use entry plus volume-weighted
exit prices. The original source checksum/study parity is checked before replay.

Local checks passed: 113 core tests, two adapter tests and 21 Python tests,
including optional raw-corpus reproduction; 108 independent cost arithmetic
checks; formatting and denied-warning workspace Clippy. CI passed all three
jobs and 694 workspace tests, with one existing manual benchmark ignored. CI
also ran the two adapter tests; the raw-corpus Python group was explicitly
skipped there because its datasets were unavailable.

The actual replay produced session closes, stops and gap-through stops, **no
TP1/runner exits**. Synthetic tests exercise that branch; they do not establish
its empirical trading performance. Historical contract/coverage, actual fees,
financing, liquidity/rejections, account sizing, margin and portfolio conflicts
remain unverified. Engineering correctness cannot substitute for an edge.

Railway was verified successful at merge `6eb2606`, with eight connected monitoring
loops and advancing exit checks at 13:18:28–13:19:05 SGT on 6 November. Those are
dated release observations, not a claim about current runtime health. Production
strategy rules, live permissions, worker authority and Telegram policy were
unchanged by PR #168.

## §6. Proposed next work, in priority order

### A. Reconcile the evidence before coding

Check the committed source/hash receipts and independently recalculate the primary
table and matched oil comparison. Report any material discrepancy with the row
identity and formula. Do not rerun the complete CI suite merely to repeat its
known result; test a specific changed or disputed behaviour.

### B. Establish cost and data feasibility

Use authorised, representative broker execution evidence to assess entry **and**
exit slippage, commission and financing separately by symbol, account environment
and size. Earlier selected demo entry receipts cannot calibrate a live round trip.
If paired evidence is unavailable, retain the unknown rather than lowering the
24bp assumption to produce a pass. Check historical quote availability and tail
coverage before allocating a larger collection budget.

### C. Replicate oil with fixed rules on additional development dates

Keep peer continuation and own momentum side by side, with always-long as the
control. Add no coefficients or thresholds. Preserve the original plan/result;
create a new, parent-linked collection/analysis plan before new outcomes are seen.

A concrete **proposed, not activated** first batch is both SpotCrude and SpotBrent
on 21, 23, 25, 28 and 30 July; and 1, 4, 6, 8 and 11 August 2025: **20 four-hour
episodes**, using the same 07:00 UTC window and five-second tail. All precede the
earliest reserved validation date and exclude the two original development dates.
Confirm session availability before freezing. Do not replace a losing or missing
episode after inspection. Use existing per-job tick/page bounds and retain only
the source/receipt/derived evidence needed for reproduction.

In addition to the existing primary development gates, propose requiring at least
five closed scenarios per oil symbol across three distinct weeks before opening
validation. This is a proposed stricter evidence requirement, not an implemented
change to the old gate. If the bounded batch is insufficient, report that outcome;
do not keep collecting until a desired percentage appears.

### D. Treat metals and indices as separate research decisions

- **Metals:** establish whether the entry is feasible at evidenced costs. If the
  admission geometry is subsequently changed, register it as a new candidate;
  the existing 2/3.74bp payout sensitivities do not test that change. A different
  macro feature must have as-of historical data, not revised/backfilled hindsight.
- **Indices:** request at most one falsifiable, class-specific mechanism supported
  by research and available causal inputs. Specify why it should beat own
  momentum/always-long after costs, what would falsify it, and which data are
  missing. Do not assume an LLM-generated narrative is an explanatory feature.
  Without a supported mechanism/data path, park this candidate.

Do not select individual winning indices or shorten the stop after inspecting
these six closes and call the result validation.

## §7. Six checkpoints for subsequent work

1. **Identity:** immutable plan/source/code receipts, exact episode set and costs.
2. **Mechanism and availability:** causal feature, economic rationale, native
   quantity/session mapping and representative cost coverage.
3. **Focused implementation checks:** timing/look-ahead, cost/veto precedence,
   executable exits and censoring. Keep proven code unchanged.
4. **Development:** frozen primary candidate and controls; adequate independent
   dates; positive expectancy/PF and per-symbol outcomes; missing/censored failures
   visible. Win percentage alone is insufficient.
5. **Separate-time evaluation:** only after development passes; retain the reserved
   19 August–5 September dates, at least 20 class closes, five per symbol over
   three weeks, and no retuning. Previously seen baseline bars limit holdout purity.
6. **Operational qualification:** paired actual costs/coverage and prospective demo
   shadow evidence before any promotion decision under the project contract.

A failed checkpoint produces a short evidence-backed disposition: fix an actual
implementation defect, obtain a specified missing input, or reject/park the
hypothesis. It is not a reason to spend unused AI allowance or create a cosmetic PR.

## §8. Requested independent review from the other AI

Use the [ready-to-paste review prompt](AI-review-prompt-tested-18280-2026-11-06.md)
for a standalone brief covering the tested rules, results and qualification work.

Assess this handover critically; agreement is not the objective. Return:

1. A short verdict: which conclusions are supported, conditional or unsupported?
2. A table of disputed claims, source row/file, independent calculation and impact.
3. Whether the 24bp geometry versus lower payout-cost sensitivities changes the
   interpretation; distinguish infeasibility, absent signal and losing execution.
4. Whether the proposed oil batch/controls and stronger sample gate are appropriate;
   identify any prior exposure, session issue or selection bias before collection.
5. At most one justified amendment per class. Specify the mechanism, causal data,
   rejection criterion, costs and exact next deliverable—not just more tuning.
6. What was independently reproduced versus merely read from Codex's receipt.

Do not claim success from compilation, same-sample fit, simulated zero fees or a
50% win ratio with negative expectancy. Cite relevant `№`, `§`, file and commit
references. If raw files are missing, mark raw replay **Not Verifiable** rather
than asserting reproduction. Do not treat this proposed plan as an already queued
or running autonomous job.

## §9. Invariants and collaboration

- Original goal: 73% net wins and 1:1.38 reference risk/reward over the latest
  20 trades, separately per strategy/account; interim research target 50% per
  class with per-symbol checks. Neither goal is established by this experiment.
  With exactly 20 binary outcomes, at least 15 wins are needed to meet 73%.
- Preserve serial `№`, paragraph `§`, SGT date/time and model/effort attribution
  in journals and PRs. Check both [Codex](../log/CODEX_Log.txt) and
  [Grok](../log/GROK_Log.txt) logs before assigning a new serial. A different AI
  should maintain its own log under `/log` and cross-reference this one.
- Retain proven fixes, original receipts and rejected results. Keep live risk,
  permissions, historical seal and recent collection-only boundaries intact.
  No Telegram incident reports or new automation is introduced here.
- “Remember that Time is Money.” “It's not the will to win that matters—everyone
  has that. It's the will to prepare to win that matters.” Allowance is a ceiling,
  not a workload target; preparation means relevant evidence and bounded decisions.

## §10. Delivery boundary

This PR adds a handover and journal entry only. It links to the existing HTML,
JSON and code. Proposed dates, stricter sample gate, cost work and new hypotheses
have **not** been executed or silently applied. The next AI should report its
review first, then implement only the next specifically scoped, authorised work.

Draft for independent review.
