# Stage 1 execution contract and corrections

№18,315 §§1–8 · 7 November 2026 · SGT (+08:00).
By Codex (GPT-6), effort not exposed. Actual freeze time and hashes are in the
execution manifest and collection provenance; the supplied 02:55 UTC timestamp
is the reviewer's document timestamp, not evidence that this implementation existed then.

## §1. Authority and exact source preservation

The owner explicitly requested end-to-end execution of
`doc/Next_Step_Agent_Prompt_18314_2026_11_07.md`. Its embedded JSON and Markdown
are preserved verbatim under the requested `docs/` paths. This addendum resolves
operational ambiguities before any Stage 1 outcome is read; it supersedes factual
claims and unspecified implementation choices, not the original goals or budget.

The source lists **eight**, not seven, symbols; a hypothetical full cross-product
is 320 episodes, not 280. Only Stage 1's four symbols × 20 dates = **80 logical
episode jobs** is authorised here. Stage 2 does not run. Northern-Hemisphere season
names are cohort labels, not verified market regimes. A week per cohort is not a
complete calendar-year study. Summer illiquidity is a hypothesis, not an established
cause of the prior losses. Earlier oil work also used more than two dates.
Same-millisecond bid/ask ambiguity can generate `gap_before_quote`; this does not
prove illiquidity or missing ticks. Range expansion does not prove institutional
participation. Failure rejects the tested candidate under its assumptions, not
all possible edges. 15/20 is 75%, the integer threshold exceeding the unchanged 73% goal.

## §2. Data, sessions, and budget

Use exact demo 49236051 symbols and their frozen current contract metadata.
US500 opens at 09:30 America/New_York (DST-aware), BTCUSD at 00:00 UTC, both oils
at their existing 07:00 UTC session open. The alternative 13:00 oil window is not
added: that would change the declared 80-job experiment. Each quote window is
four hours plus five seconds. Historical metadata remains unverified.

Each logical episode includes bid/ask history and up to 45 calendar days of M30
OHLC/tick-volume preparation. Shared seven-day-or-shorter M30 segments are fetched
once per symbol/cohort and reused, with their own pre-read/finished receipts.
Pages and logical jobs are distinct; segment count is in the manifest. Limits:
80 episodes, 120 tick pages / 200,000 combined updates / 240 seconds per episode,
one request per bar segment (at most 336 bars), 32 MB per episode and 256 MB total
raw source. No retry of failed/uncertain reads, replacement dates or budget reset.

No request may overlap **19 August–5 September 2025 inclusive UTC**, including
warm-up requests. October's nominal 45-day view is split around that interval;
the omission is recorded. No interpolation or fabricated history replaces it.
Fourteen prior sessions are required by S3; the ORB median needs the exact previous
20 registered sessions with valid opening bars. Insufficient history blocks that
model while independent tick controls may still be evaluated. No validation is opened.

## §3. Production-rule track and explicit parity limits

The offline adapter calls the existing native S3 band/mark/veto/dispatch-cost/target
and exit implementation with v7 dispatch checking. US500 is enabled only inside
this offline bounded adapter; HTTP validators and live strategy permissions stay
unchanged. Prepare from positive-volume opening M30 and the median observed spread
in the first 30 minutes, available before the first decision. This is a reconstructed
cost/preparation scenario, not an authenticated historical pre-open receipt.

Production S3 keeps its actual session end. A four-hour observation boundary
**censors** any remaining position; it must not invent a four-hour production close.
ORB and fixed-horizon controls close at hour four. This horizon difference is explicit
and prevents calling the study a complete production/account parity certification.
Account margin, sizing history, ownership, latency, real amendments and fees are absent.

## §4. Frozen ORB hypotheses

Two candidates: 30-minute and 60-minute opening high/low from complete broker M30
bars (single-price OHLC proxies, not executable midpoint extrema). Both require
positive volume and a positive range. Compare with the median opening range of
exactly the preceding 20 registered sessions; require current range **>1.25×median**.
Never skip missing prior sessions to cherry-pick 20 substitutes. Only information
available by the opening-range end is used.

At each half-hour mark starting at the range end, take the first fresh two-sided
quote within five seconds. Buy above the high, sell below the low. Fill at the next
strictly later fresh quote within the same mark window, rechecking direction/cost.
One entry per episode. A cost veto ends the candidate's episode; no later reassessment.
Original R is the range rounded to the contract grid, stop 1R from fill, Q=1.38
through the shared target planner and minimum native quantity. Shared fixed-stop
and 30-second original-R trailing exits are both reported. No exit parameter tuning.

## §5. Controls, costs and denominators

Every episode also evaluates always-long and own-momentum controls at hour one:
first fresh quote in its five-second mark window, then a strictly later fresh fill.
Momentum direction is that decision midpoint minus the session-opening M30 price;
zero change is no signal. Original R is that day's M30 first-hour high/low range.
Both use the same primary cost admission and hold to the first fresh quote at/after
hour four (at most five seconds), without stops or targets. Missing close is censored.

Report matched candidate-minus-control R on the same symbol/date with both complete
outcomes; record unmatched episodes. These are comparisons of complete strategies,
not a pure directional-effect estimate: entry times and exits can differ. Never pool
ORB-30 with ORB-60, competing directions, or fixed/trailing arms as independent trades.

Primary admission and target geometry remain **24 bp per side**. Reprice only those
same admitted fills at 2 and 3.74 bp per side; do not create sensitivity-only entries.
Also report how many primary cost vetoes would satisfy the lower arithmetic cost
threshold, without pretending those are trades. Prior repository evidence does not
certify either lower value as historical broker costs: both are **uncalibrated
sensitivities**. Spread is charged once via bid/ask fills; zero additional commission
is an explicit incomplete scenario, not evidence of no fees.

Closed-trade win rate = positive net closes / all closes, including breakevens.
Every failed, vetoed, unentered, or censored episode remains in the episode denominator
and coverage table. Also show wins / all scheduled episodes. Missing results are null,
not zero wins or zero returns. No price-unit sums across different instruments.

## §6. Gates and stopping

Use the stricter JSON threshold: ten closes per symbol, twenty per class, **four**
weeks, >=50% wins, strictly positive mean net R and price-unit result, PF>1, zero
coverage/sequence flags. Evaluate each model independently; sensitivity cannot rescue
a failed primary screen. Qualified broker/account performance remains unverified.

Matched deltas use a predeclared exact one-sided sign-flip test over weekly mean
paired deltas, p<=0.05 versus both controls, with at least four matched weeks. With
only four weeks its smallest possible p is 0.0625: Stage 1 alone cannot pass this
statistical gate. That is a pilot limitation, not a reason to loosen the test.
The 50% interim and 73% / 1:1.38 account goals are not automatically certified.
Negative means in both years halt tuning of that candidate under this plan;
missing/censored evidence is an inconclusive result. No automatic Stage 2 or validation.

## §7. Freeze, implementation and verification

Merge this contract, the exact source plan, generated execution manifest, adapter,
collector and focused tests before collection. Store commit, actual source hashes,
plan hashes and UTC freeze time before broker reads. Keep source and segment receipts;
replay checks all source identities and manifests. Any missing prerequisite is a
reported failure, never a fabricated “80 verified”. Reuse the core native exit engine.

## §8. Deliverables

`docs/seasonal-pilot-results-2024-2025.json`, the requested `doc/` Markdown report,
an offline HTML report, and `log/CODEX_Log.txt`. Report source/hash coverage separately
from economic qualification. Final Gemini block uses actual counts and null metrics;
its cohort breakdown is explicitly the primary S3 trailing arm, and ORB-30/60 are
separate in the overall summary. Reports are merged after CI. No ZIP or raw quote dump.
