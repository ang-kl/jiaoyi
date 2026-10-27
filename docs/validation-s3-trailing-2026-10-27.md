# S3 exit validation contract — frozen 27 October 2026, 16:10 SGT

Document VAL-S3-20261027. Owner27Oct: “how about trailing-stop methods, are they
useful; proceed: reuse verified mechanisms and complete the missing validation.”
This authorizes the bounded validation below. It does not activate a new S3
exit policy. Existing S1/S3 trading, pre-orders, protection and registered rules
remain. Earlier H1/H2 R1/R2 jobs remain paused; this is a separate S3 validation.

## Decision and immutable comparison

Question: for the same actual owned S3 entries, does a single fixed trailing
candidate improve exit outcomes after allowing for executable quote sides,
protection timing and available costs? A trail may reduce profit give-back and
also truncate winners. Its financial usefulness is unknown before comparison.

Baseline: the original entry's saved registered fixed stop, TP1 partial close,
runner/whole-position broker TP and session-end exit. Reuse current validated
S3 target arithmetic; preserve original filled volume, initial stop distance R,
price grid and planned partial geometry. No parameter search, entry change,
size change or selection based on winners. All exceptions remain visible.

Candidate `s3-initial-risk-trail-v1`: same entries/brackets; a stop R behind the
best executable bid since entry for a long, or R above the best ask for a short.
It can only tighten. R always means the original stop distance; it is never
redefined by a tightened stop. Original TP/partial/session-end rules remain.
The fixed distance uses existing original-risk evidence, not a fitted ATR or
new best-performing multiplier. This is a new unvalidated candidate policy,
not a claim that bot-trade's profitability or identical parameters transfer.

Simulation cadence: at most one trailing proposal per60seconds, following the
existing S1 operational default as an explicit simulation assumption. Client
TP1 closes and accepted trailing changes take effect no earlier than the next
recorded quote; original broker protection is checked first. Recheck that a
pending stop remains on the protective side. Fill stop gaps adversely at the
executable quote, targets conservatively at their trigger price. No same-quote
future peek or optimistic stop fills. Broker minimum-distance/latency/fill
uncertainty must be reported; this scenario cannot prove live amend acceptance.

## Frozen sample and data admission

Primary initial audit cohort: each configured account's latest20 fully closed,
program-owned S3 positions as of **27Oct2026 16:15SGT /08:15UTC**. Retain every
selected position and its qualification result. Require unique owned opening
provenance, original saved setup/brackets/grid and filled-share risk, and an
ordered recorded quote path through the needed exit horizon. Restrict initial
comparable replay to provable full opening fills; classify partial/ambiguous
cases explicitly rather than silently dropping them. Empty or censored paths,
feed restarts/gaps, unknown ordering, missing original limits/session or missing
cost inputs make the affected claim unavailable. No success by excluding losers.

Use the existing account's recorded live bid/ask quotes with session/sequence
provenance. Reconstruct sides only from past events; historical independently
paged bid/ask streams cannot establish exact within-timestamp event ordering.
Do not synthesize absent prices or copy another account's symbols/quotes. The
source-readiness audit must state whether continuity and side freshness can be
established; it cannot manufacture perfect completeness from a min/max range.

Read existing production data through a bounded trusted read-only operator.
Models receive no production broker/DB credentials. Keep only minimum summary,
source/grain/coverage receipts and comparison outputs; no new raw-tick archive,
production table or service. Preserve old quotes/history retention settings.

## Evidence and acceptance

Report per account/position: eligible/unknown reason, baseline/candidate modeled
exit and grossR, available cost provenance, modeled netR only when supportable,
paired difference and cohort/period. Bid/ask fills already include spread;
never charge it a second time. Unknown costs are not0. Counterfactual timing,
fees, swap, slippage and changed future entry opportunities are limitations,
not observed broker facts. No simulated result replaces canonical broker netR.

The comparison holds actual entries fixed, so it is **exit-management research**,
not a complete entry/portfolio/equity-path backtest or proof of causal live
benefit. Pre-existing actual outcomes have already been seen; this first cohort
is descriptive/development evidence. Unseen observations after this contract's
registration are a separate forward cohort. Do not tune this candidate to the
first cohort or open the old H1/H2 held-out seal.

Software acceptance: meaningful regressions of ratchet direction, executable
side/spread, original-risk denominator, partial/whole TP, stop gaps, pending
updates/cadence, no look-ahead, malformed/incomplete paths, and missing costs.
Compare existing S1 trail/gate/protection mechanisms with pinned bot-trade source
and actual tests. Unchanged required CI and actual executed data checks apply.

Investment acceptance remains the current S3 registration:200forward demo trades,
positive mean netR with the week-block bootstrap lower bound>0, random-direction
control comparison and PF≥1.3. Existing user goal remains separately latest20
per strategy/account,≥73% net wins (15/20=75%) and risk:reward objective1:1.38;
planned geometry and realized payoff are separate. A candidate cannot be called
useful/profitable from software tests or a selected small replay alone. Report
fail/insufficient/unknown with evidence rather than waiting for a desired verdict.

Any later proposal to change live S3 exits must show the concrete method,
comparative evidence and new registration for the owner; this validation does
not silently amend current protection. I-1..I-11, time-efficient delivery,
minimum data, allowance-as-ceiling and $40TOTAL API policy remain.
