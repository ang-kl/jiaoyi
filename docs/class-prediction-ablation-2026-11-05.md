# Class-prediction development follow-up: separate sampling from regime selection

№18,258 §6; №18,259 §§1–5 ·05-11-2026,20:18 SGT ·by <Codex (GPT-6), effort: not exposed>.

## Trigger and frozen changes

The40broker episodes of PR160 reproduced exactly after PR161's JSON-boundary
correction. For each original candidate:22episodes failed efficiency>=0.5,
12lacked strict first-hour samples,4failed C<R,2entered. The ETH entry closed
negative after costs; JPN225 lacked closing coverage and remained censored.
Those are development observations, not evidence that all markets lack edges.

One exploratory amendment reuses those40sources without any additional broker
job, API model call, database retention or live strategy change. The original
40-episode plan and80outputs remain intact. This is an adaptive development
experiment on already-seen data, not independent confirmation or validation.

Retain original high-efficiency continuation/reversal controls. Add low-efficiency
reversal (<0.5) to test fading choppy rather than strongly directional movement.
Cross these three hypotheses with two feature sampling policies: original strict
five-second scheduled snapshots; and bounded, already-observed features. The
latter uses the first fresh snapshot within minute1, then at minutes5..55 the
last historically fresh snapshot at/before that mark, no older than60seconds.
The selected timestamps are exposed for audit. These are dated observations,
not current executable quotes. Decision at minute60 still requires a fresh quote
within5seconds, and the next fresh fill proxy must arrive before the same deadline.
Original side-age6seconds,24bp/side,cost gate,R/targets,volume and exits remain.
There is no interpolation, future feature, weakened live guard or new scheduler.

The amendment JSON freezes these six variants. Its reuse-only flag prevents
collector calls, including validation. A later independently specified evaluation
would be a separate protocol, not silently using already-seen development data.
Only its listed variants are scored; the original plan still selects only its
two controls. Source and locally compiled algorithm hashes are recorded separately.

## Actual results and decision

All40original sources reproduced; all80original candidate outcomes unchanged.
The240paired episode/variant results are not240independent trades. Bounded as-of
features reduced missing-feature episodes from12to2. That is a measurement
improvement, not a profitability improvement.

The observed-feature/choppy-reversal candidate closed6index scenarios,2energy
scenarios and1crypto scenario:0net winners in each class. FX and metals had no
entries, mostly cost refusals for this candidate. Fixed-stop comparison also
produced no net winners. Index scenarios had2/6gross winners, showing that costs
matter, while four gross losses demonstrate that fees do not explain everything.
Original coherent crypto continuation was gross positive on its one close but
net negative. No candidate passed any class screen; evaluation remains unopened.

Retain reproducible measurement and missingness accounting; reject these tested
candidates for promotion under the fixed assumptions. Do not lower fees, relabel
censored entries, count missing trades as wins or keep changing thresholds on the
same dates. Next economic work requires class-specific explanatory information
and verified costs/horizons, not another arbitrary efficiency threshold. Current
class/contract/cost evidence remains insufficient to certify50% or the original
73%/1:1.38 perstrategy/account goal. Tests do not promise either target.

## Verification

Two added Rust tests distinguish past observations from execution freshness and
check the independent low-efficiency regime. One Python guard refuses all broker
collection for the reuse-only amendment. The five class Rust checks and eight
Python research checks passed; workspace denied-warningClippy passed. Results
and source hashes live in /workspace/jiaoyi-class-ablation-evidence. Required CI,
merge and deployment receipts belong on the PR. No live registration changed.
