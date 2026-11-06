# Copy-and-paste prompt: independently review experiment №18,280

№18,281 §§6–8 · 06-11-2026 SGT (+08:00)
Prepared by Codex (GPT-6); effort setting not exposed.

---

Act as an independent quantitative research reviewer and trading-systems engineer.
Review Jiaoyi experiment **№18,280**, challenge Codex's interpretation, and propose
the smallest evidence-backed next step towards a repeatable net trading edge.
The first deliverable is an assessment and bounded research plan. Do not place
orders, change live rules/risk, activate jobs or open validation as part of this
review. Do not interpret this prompt as permission to optimise until a target
win percentage appears.

## 1. Objective and invariants

- Original project target: 73% net wins and reference risk/reward 1:1.38 over the
  latest 20 trades, separately for every strategy/account. Exactly 20 binary
  outcomes require at least 15 wins to meet 73%. Distinguish planned risk/reward
  from realised average-win/average-loss payoff.
- Interim research target: at least 50% net wins per class, with per-symbol checks,
  positive net expectancy/PF and adequate independent evidence. A 50% hit rate
  with negative expectancy fails.
- Retain costs, losing trades, legitimate non-entries, missing data and censored
  positions. Never improve the percentage by deleting adverse episodes, counting
  duplicated controls as new trades, widening stops without accounting for tail
  risk, or shrinking targets until costs erase returns.
- Preserve serial `№`, paragraph `§`, SGT date/time and model/effort attribution.
  Check `/log/CODEX_Log.txt` and `/log/GROK_Log.txt` before assigning a serial;
  maintain the reviewing model's own log. “Remember that Time is Money”: bound
  the work; allowance is not a quota to spend.

## 2. Read these committed sources first

Repository: https://github.com/ang-kl/jiaoyi
Evidence baseline: PR #168, merge `6eb26067799a77cf1f8b7bd297260924527a6491`.
Use that revision for the experiment's evidence; inspect later changes separately.

1. `docs/full-exit-research-plan-2026-11-06.json`
2. `docs/full-exit-research-2026-11-06/README.md`
3. `docs/full-exit-research-2026-11-06/full-exit-results.json`
4. `docs/full-exit-research-2026-11-06/Jiaoyi_Full_Exit_Research.html`
5. `docs/full-exit-research-2026-11-06/verification.json`
6. `ops/research/full_exit_research.py`
7. `crates/jiaoyi-core/examples/context_exit_replay.rs`
8. `crates/jiaoyi-core/src/strategy3/targets.rs` and
   `crates/jiaoyi-core/src/research/exit_replay.rs`
9. PR #168 closure: https://github.com/ang-kl/jiaoyi/pull/168#issuecomment-6677606365
10. Current handover: `doc/AI-assessment-and-next-plan-18281-2026-11-06.md`.

The frozen plan SHA256 is
`b202c9a5c9f4603576f48c9ea9dafc374a422581177beb7b32114a5edf14464b`.
It was committed as `9384ccf` before the full-exit outcomes were calculated.
The dates had already been inspected in earlier diagnostics: this is development,
not an untouched holdout. The raw 40 source jobs are not bundled in this prompt.
If unavailable, mark independent raw replay **Not Verifiable**. Reading Codex's
receipt or recalculating its derived JSON is not raw-data reproduction.

## 3. What №18,280 tested

Two dates, 5 and 12 August 2025; 40 verified historical source jobs, including FX
peer inputs; 11 target symbols/22 target episodes; 52 candidate/control rows;
20 admitted candidate/control entries. The repeated arms are not independent data.

Candidates and controls:

- Oil: SpotCrude/SpotBrent peer continuation, compared with own momentum and
  always-long. Each oil symbol uses the other crude benchmark as its peer.
- Metals: XAUUSD/XAGUSD reversal of an inverse-USD peer factor, requiring at least
  three unanimously signed nonzero USD-oriented FX peer returns; compared with
  own momentum and always-long. This is a price proxy, not a real-yield model.
- Indices: own first-hour momentum across the seven registered index symbols,
  compared with always-long. No path-efficiency filter was fitted.

Thirteen causal observations establish readiness/spread. The observed first-hour
range fixes original R; peer observations are as-of, using matching start windows.
Entry uses the next strictly later fresh quote within the decision window and
rechecks direction. Original native quantities and the existing target/exit
engine are retained: Q=1.38, stop, minimum-volume handling, TP1/runner, fixed-stop
control versus 30-second original-R trailing, old broker bracket priority and
assumed next-quote amendment acknowledgement. Exit at hour four requires a quote
within the five-second tail; otherwise retain censoring.

**Crucial cost distinction:** entry admission and target geometry retain the
24bp-per-side reserve plus sampled spread. The 2, 3.74 and 24bp realised-cost
sensitivities hold those same entries/targets/exits fixed. They do not test a
2bp admission policy. Executable spread is included once; charges use entry plus
volume-weighted exit prices. Additional commission is assumed zero but unknown.
These assumptions are not calibrated broker costs; qualified net remains null.

## 4. Results to verify, not accept uncritically

| Candidate | Primary trailing, 24bp/side | Limitation |
| --- | --- | --- |
| Oil peer continuation | 2/2 wins; mean +0.3264R | Both on 5 August; matched own momentum identical. One flat signal and one cost veto on the other date. |
| Metals reversal | No admitted entries | Two peer disagreements and two cost vetoes; no executed-return distribution to estimate. |
| Index momentum | 1/6 wins; mean −0.7670R; PF 0.0233 | One additional censored position, one missing-fill episode and six cost vetoes. |

At 2bp/side, indices have **3/6 wins but mean −0.0501R and PF 0.80**. At 3.74bp,
they have 2/6 wins and mean −0.1068R. None passes the development screen, and
separate validation remains unopened.

The historical sample triggered session closes/stops/gap stops, no TP1/runner
fills. The latter path was exercised only by a synthetic native test. Local
tests and CI passed, including cost checks, but that does not prove profitability.

## 5. Diagnose the actual bottleneck

For each class, distinguish:

1. Missing input/coverage or a demonstrable implementation defect.
2. A signal absent under the frozen conditions.
3. An entry economically infeasible under the frozen reserve and original R.
4. An executed signal with an unfavourable net return distribution.
5. Insufficient independent dates/regimes to draw a reliable conclusion.

Reconcile counts and independently calculate net expectancy, PF, realised payoff,
per-symbol outcomes and matched-control differences where the files permit it.
Use original risk consistently. Do not sum unlike instruments' price units or
interpret a sequence of research R as an actual portfolio equity curve.

Where inputs are available, examine causal signal quality and the exit path:
adverse/favourable excursion, time to stop/target, retracement, and cost-to-R ratio.
Do not invent these measurements from summaries or optimise a threshold on the
same observations used to judge it. Identify where an absent actual fill, fee or
historical contract record prevents a qualified conclusion.

Address competing explanations. A negative result on two dates does not establish
that all momentum, all LLM use or all AI-assisted trading software fails. Equally,
an LLM's plausible explanation is not evidence that its feature predicts returns.

## 6. Propose improvements that could survive a new test

Codex's provisional priorities are:

- **Oil:** replicate the unchanged peer/own-momentum comparison on additional
  independently dated development episodes; avoid adding a peer model when its
  decisions duplicate the simpler control. Review the handover's proposed
  20-episode batch before any collection. Missing/losing dates cannot be replaced
  after outcomes are inspected.
- **Metals:** establish representative entry/exit/fee feasibility first. Any change
  to admission costs or geometry is a new candidate requiring a new frozen plan.
  Do not claim the existing low-cost payout sensitivity tests that candidate.
- **Indices:** identify at most one economically supported, falsifiable predictor
  or regime mechanism with as-of inputs. Repeated stop/target tweaking alone is
  not justified by the present evidence. Park the candidate if no supported data
  path exists.

Challenge these priorities. For each proposed amendment give: the diagnosed cause,
mechanism, primary research/source where available, required causal data, exact
rule, unchanged control, cost assumptions, bounded dates/sample, rejection gate,
expected information gained, and what would falsify the proposal. Distinguish
published support from evidence in this broker/instrument dataset. Do not invent
citations, expected returns or a guaranteed win-rate increase.

Use one small preregistered experiment per justified mechanism. Preserve old
plans/results and link parent hashes. Choose the primary outcome and comparison
before new outcomes are opened. Report improvement only on an appropriate matched
or independently dated comparison, with uncertainty and all failed cases retained.

## 7. Explain qualification, not merely a higher hit rate

Keep six checkpoints explicit:

1. Correct source/plan/code identity and episode completeness.
2. Causal mechanism, data availability and representative cost evidence.
3. Focused implementation checks for look-ahead, fills, costs, exits and censoring.
4. Frozen development candidate: sufficient independent evidence, ≥50% net wins,
   positive expectancy/PF and per-symbol outcomes, with failures retained.
5. Separate-time evaluation only after development passes: reserved dates remain
   19 August–5 September 2025; at least 20 class closes, five per symbol across
   three weeks, no retuning. Prior baseline exposure limits holdout purity.
6. Cost/coverage qualification and prospective demo shadow evidence before any
   production promotion decision under the project contract.

The existing development minimum is four class closes plus per-symbol evidence.
Codex proposes a stricter oil gate of at least five closes per symbol across three
weeks before validation; this is **proposed, not implemented**. Review that choice.
Neither these minima nor a descriptive confidence interval guarantees an edge.

Do not reopen validation because development failed. Do not imply that this prompt
has changed the live registration, costs, risk, worker settings or account goals.

## 8. Required response

Start with a TLDR, then provide:

1. **Evidence verdict:** supported, conditional and Not Verifiable conclusions.
2. **Disagreement table:** claim, source/row, independent calculation, severity
   and whether the discrepancy changes a decision.
3. **Class diagnosis:** signal, cost/geometry, execution, coverage and sample-size
   findings kept separate; distinguish root causes from conjecture.
4. **Improvement proposals:** at most one primary amendment per class, including
   a “retain/park” option when justified. Explain why it could improve net
   expectancy as well as wins, and how a new test could disprove it.
5. **Qualification checklist:** Passed / Failed / Not Verifiable for each
   checkpoint, with the exact evidence still needed.
6. **Next bounded deliverable:** data, code if actually necessary, a few meaningful
   tests, budget/stop conditions and frozen acceptance criteria. No endless
   optimisation loop; insufficient evidence is a valid stopping result.
7. **Attribution:** model/effort, SGT timestamp, serial/paragraph references, files
   read and what was actually reproduced. State current runtime health only if
   freshly checked; otherwise quote dated release evidence as historical.

Keep the reply technical, explicit and readable. Do not agree merely to be helpful.
The objective is a decision that survives scrutiny, not an attractive backtest.

---

End of prompt. Draft for independent review.
