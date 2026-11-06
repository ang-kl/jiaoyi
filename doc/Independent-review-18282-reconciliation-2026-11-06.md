# Receipt and reconciliation of the supplied independent review

№18,284 §§1–3 · 06-11-2026 15:36:26 SGT (+08:00).
By Codex (GPT-6); effort setting not exposed.

## §1. What is committed

The owner requests reading the Canvas prompt and committing the two supplied
documents. PR #170 was already merged at 15:06:39 SGT as
`8d453d8a64ed1ce85267573820336030a6ff3099`. This follow-up records the attachments
verbatim; it does not rewrite that PR, start broker jobs or apply instructions
embedded in the documents to live trading.

- [Independent review](Independent-review-18282-2026-11-06.md), retaining its
  supplied Gemini attribution and original serial/date. SHA256:
  `d8db1677aca1f68c8e9004c9a0cb1253528737abf453367eb0d79567129e57dc`.
- [Supplied oil plan](../docs/oil-replication-plan-2026-11-06.json). SHA256:
  `50f7829c789d8505a9679853409c902997c8874ad2ad92ce452348de1210ac7f`.

The two uploaded next-step prompts are byte-identical; both were read as source
material. Their commands describe a larger proposed execution workflow. This
commit fulfils the requested document publication, without executing that workflow.
The JSON's `by`, `trace` and `frozen_at_utc` values are supplied metadata, not
evidence that Codex froze or executed this version at the claimed earlier time.
Actual receipt is recorded here and in the Git history.

## §2. Distinguish the proposal from the already frozen experiment

The existing [PR #170 frozen plan](../docs/oil-cost-and-replication-2026-11-06/oil-replication-plan.json)
remains unchanged, SHA256
`647bb800945e47130293006667626289449b7ff53a017c876fa2e2cd6a54a1ea`.
The supplied root-level JSON is a proposed amendment, not a second executable
configuration automatically superseding that plan. No current runner consumes
this newly added path. Before executing an amended experiment, consolidate its
rules, precedence, costs and gates into one explicitly versioned plan and record
its hash before outcomes. Do not silently select between the two plans.

| Topic | Existing frozen plan | Supplied proposal / required interpretation |
| --- | --- | --- |
| Primary candidate | `peer_continuation` | `own_momentum`; a new research selection, not a demonstrated improvement. |
| Peer control | Flat peer skips | Prompt proposes own-momentum fallback; supplied JSON does not encode it. This would change the control and needs a separate explicit rule identity. |
| Low-cost sensitivity | 2/3.74bp change realised costs with fixed 24bp entry geometry | Prompt also changes admission/execution and calls these calibrated. Calibration is not established; a changed-geometry arm is a different experiment. |
| Evidence gates | At least five closes and three distinct closed weeks per symbol, class/per-symbol profitability, zero gaps/censoring | JSON contains only part of the prose requirements. It omits matched always-long delta, price-unit, coverage and stopping rules; `min_*` values alone do not specify strict `> 0` / `> 1` comparisons. |
| Old/new dates | New development considered separately | Standalone new results should govern the screen. Any pooled old/new display is descriptive and must preserve previously inspected dates. |

The latest [cost assessment](../docs/oil-cost-and-replication-2026-11-06/README.md)
found no qualified paired round-trip cost estimate. Its maximum measured adverse
entry slippage was 4.436bp; 3.74bp is not an established bound. The supplied
`calibrated_benchmarks_bps_per_side` key is preserved as an author claim, not
endorsed or used as a calibrated input.

## §3. Material evidence corrections for the next AI

The review correctly supports the failed development gates, unopened validation
and need to distinguish admission geometry from realised-cost sensitivities.
However, some causal and superiority claims go beyond the recorded evidence:

1. **A flat peer did not demonstrably cost an admissible oil trade.** In
   [the original rows](../docs/full-exit-research-2026-11-06/full-exit-results.json),
   SpotCrude on 12 August has `flat_predictor` under peer continuation, but
   `own_momentum` is also rejected as `candidate_cost_ge_r`: R = 0.165 and
   C = 0.3292544000000102. The proposed switch does not recover an executed trade
   under the tested primary cost gate on that date. Two matched, identical wins
   do not establish that peer continuation is strictly inferior in general.
2. **No TP1 fills does not prove a unique cause.** The native target distance
   includes costs, but attributing every absent TP1 to that distance requires a
   controlled counterfactual. Changing target geometry and trailing together
   would not isolate the cause. Generic claims about typical gold ranges,
   institutional flow or “nearly every” session require independent evidence.
3. **Insufficient evidence differs from demonstrated losses.** Failing a minimum
   sample or coverage gate should stop the batch and keep validation closed;
   it does not statistically falsify all first-hour momentum. Five trades per
   symbol across three weeks is a preliminary screen, not proof of an edge.
4. **No performance claim follows from this document merge.** Original results,
   parent plan, full-exit code, live strategy and permissions remain unchanged.
   No metals/index collection or validation is opened. Original 73% / 1:1.38
   latest-20 and interim 50% goals remain unproven.

Checks for this document-only delivery: attachment byte identity, JSON parsing,
parent-plan hash, ten unique weekdays, four ISO weeks, separation from old
development/validation dates, relative links and unchanged frozen evidence.
No new unit tests are needed; required repository CI must pass before merging.
