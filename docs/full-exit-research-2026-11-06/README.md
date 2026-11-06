# Full SL/TP/trailing research: oil, metals and indices

№18,280 §§1–8 · 06-11-2026 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.

## TLDR

**None of the three frozen candidates passes development. Validation remains
unopened. No live trading rule or account permission changes.**

| Frozen candidate | Primary trailing outcome, 24bp/side | Decision |
| --- | --- | --- |
| Oil peer continuation | 2/2 closed scenarios win; mean +0.3264R | Only one winning date, 5 August; fewer than four class closes. Own momentum makes identical matched trades. No incremental peer value demonstrated. |
| Metals unanimous peer reversal | No admitted trades | Two peer disagreements and two cost vetoes. Profitability cannot be estimated. |
| Index own momentum | 1/6 closed scenarios wins; mean −0.7670R; PF 0.0233 | One additional censored position; one missing-fill episode and six cost vetoes. Fails expectancy and coverage. |

At the optimistic 2bp/side cost, indices reach 3/6 wins but still lose an average
0.0501R, with PF 0.80. The result identifies an entry/return-distribution problem
in this small sample, beyond the cost assumption. It does not prove that all
momentum strategies fail. The two oil wins are correlated same-day observations,
not independent evidence across market regimes.

## Files

- [HTML report](Jiaoyi_Full_Exit_Research.html): concise interpretation, every
  class/control/cost scenario, individual symbols and failure-prevention gates.
- [Complete derived JSON](full-exit-results.json): all 52 rule-episode rows,
  features, peer timestamps, admitted entries, native target plans, actual
  simulated exit fills, censoring, cost sensitivities, controls and hashes.
- [Verification receipt](verification.json).
- [Frozen plan](../full-exit-research-plan-2026-11-06.json), committed as
  `9384ccf` before calculating this experiment's outcomes. SHA256:
  `b202c9a5c9f4603576f48c9ea9dafc374a422581177beb7b32114a5edf14464b`.

No raw quote archive or ZIP is included. The result contains only the derived
evidence needed to inspect and reproduce this bounded experiment.

## What was actually tested

The original 40 historical source jobs were verified against their frozen file
manifest, collection receipt, native source checksum and original-study parity.
They cover 5 and 12 August 2025. Eleven target symbols produce 22 target episodes;
FX sources supply the causal metals peer features. Fifty-two candidate/control
rows, two exit arms and three cost assumptions do **not** create independent
observations. Previously seen development dates are not out-of-sample evidence.

Oil follows the other crude benchmark. Metals reverse the inverse-USD peer
factor only when at least three USD-oriented peers have unanimous nonzero signs.
Indices follow their own first-hour move. Own momentum and always-long controls
are retained. Thirteen causal observations establish readiness and spread; the
full observed range through the decision fixes original R. No efficiency filter
or fitted threshold is added.

The Rust adapter calls the existing target planner and exit engine. It preserves
Q=1.38, native quantities, whole-position/minimum handling, TP1 partial and runner
logic, fixed stops, 30-second original-R trailing cadence, next-quote amendment
acknowledgement and old-bracket priority. The hour-four deadline requires an
executable quote within the five-second tail; absent quotes remain censored.

Across both exit arms, this corpus produced 27 session-end fills, four stops and
five gap-through stops. **No TP1/runner target fired in the observed corpus.**
The partial/runner path is exercised by the native regression fixture, not
claimed as empirically validated by these historical trades.

The primary geometry/admission reserve stays at 24bp per side plus sampled
spread. Cost vetoes precede fill searches and are checked again by Rust at the
decision/fill prices. A reporting-order correction made during verification
changed skip classifications, but all 20 admitted candidate/control entries and
their outcomes remained identical. No rule or threshold was retuned.

Sensitivities at 2, 3.74 and 24bp hold entries, targets and exits fixed. Spread is
already in executable fills; post-trade charges use entry plus volume-weighted
partial/final exits. Zero additional commission is an unknown-fee assumption,
not an asserted broker tariff. Qualified net returns remain null.

## Verification and limitations

Five checks bound this delivery:

1. Frozen plan, source receipts and original-study parity.
2. Three focused Python checks: causal features/strict next quote, peer unanimity,
   immutable plan/data-gap gates; includes cost-veto precedence.
3. Two native adapter tests: target/exit/cost/censoring/seal behaviour, and costs
   applied once across TP1 partial plus runner. Existing core suite also passes.
4. Independent arithmetic verifies 108 closed-arm cost results against fill
   prices/volumes; native/Python initial-risk calculations agree on every entry.
5. Required CI on the PR head before merge; journal and merge evidence in the PR.

Local results: 113 core tests, two adapter tests and 21 Python tests pass; fmt,
denied-warning workspace Clippy and diff checks pass. The Python run includes
the optional raw-corpus reproduction; ordinary CI skips that one group when its
local datasets are absent. CI explicitly runs the new native adapter tests.

Historical contract qualification, quote continuity, actual round-trip fees,
liquidity/rejections, financing, account sizing, margin and portfolio conflicts
remain unverified. Reported drawdown is a per-symbol sequence of research R,
not an account drawdown or a combined portfolio equity curve.

## How to proceed

Preserve this rejected/insufficient result. Oil is the only candidate with
positive primary outcomes, but needs additional independently dated development
evidence; the peer adds no demonstrated information over own momentum. Metals
first need representative cost/entry-feasibility evidence. Indices need a new
economically justified predictor or regime hypothesis before another frozen
experiment; adjusting exits or fees to this sample would be tuning to noise.

The existing separate evaluation dates, 19 August–5 September 2025, are frozen
in the plan. Only a primary trailing candidate passing development can advance.
Validation requires at least 20 class closes, five per symbol across three weeks,
positive expectancy/PF and per-symbol outcomes, with no missing/censored evidence.
Some baseline bars on those dates were previously visible; this is separate-time
candidate evaluation, not a pristine holdout. The development command cannot
collect or open validation and does not replay recent collection-only data.

The 50% interim target remains unproven. The original 73% / 1:1.38 goal applies
per strategy and account and is unchanged. This research adds zero qualifying
account trades. No separate model API spending or Telegram message is involved.

## Reproduce

```bash
cargo build -p jiaoyi-core --locked \
  --example class_prediction_study --example context_exit_replay
python3 ops/research/full_exit_research.py \
  --sources /path/to/original/development \
  --verifier /path/to/target/debug/examples/class_prediction_study \
  --replay /path/to/target/debug/examples/context_exit_replay \
  --output /path/to/new-empty-output
```

The original source files and collection receipt are required separately. This
command makes no network or broker requests and refuses changed frozen plans.
Output JSON retains all relevant source, analysis and executable hashes.

Draft for review.
