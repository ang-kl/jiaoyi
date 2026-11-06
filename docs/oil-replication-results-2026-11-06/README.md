# Oil replication: completed batch, failed development screen

№18,287 §§1–12 · 06-11-2026 16:26:08 SGT (+08:00).
By Codex (GPT-6); effort setting not exposed.

## TLDR

**Do not promote this candidate.** All 20 frozen source episodes completed and
reproduced through the native verifier. Primary own-momentum trailing results:
**3/9 wins (33.3%), mean −0.7256R, PF(R) 0.1964**. Eleven episodes failed the
cost gate. Both symbols have negative expectancy and fail the 50% win target.

The observed scenario win-rate gap to the interim target is **16.7 percentage
points for each symbol and the class**. Qualified account-goal progress remains
Not Verifiable; these backtests add zero qualified account trades. Original
73% / 1:1.38 latest20 per strategy/account target is unchanged.

The formal result is `INSUFFICIENT_EVIDENCE` because Brent has too few closes/weeks
and replay paths carry continuity flags. **That does not erase the observed
losses:** the profitability screen also fails. No validation was opened, no
second batch collected and no live rule changed.

[Download HTML report](Oil_Replication_Report.html) · [Full JSON](results.json) ·
[Verification receipt](verification.json) · [Frozen plan](../oil-replication-v2-2026-11-06.json) ·
[Adapter/run instructions](../oil-replication-v2-2026-11-06.md).

## Standalone new-date results

| Primary 24bp trailing | Closes / wins | Win rate | Mean net R | PF(R) | Closed ISO weeks |
| --- | ---: | ---: | ---: | ---: | ---: |
| SpotCrude | 6 / 2 | 33.3% | −0.5569 | 0.3096 | 3 |
| SpotBrent | 3 / 1 | 33.3% | −1.0629 | 0.0297 | 2 |
| Class | 9 / 3 | 33.3% | −0.7256 | 0.1964 | Per-symbol gate applies |

There are 60 rule/episode rows (three rules on 20 shared episodes), 26 admitted
rule-episodes and 156 closed-arm/cost calculations. Repeated controls and
sensitivities do not increase the independent sample. No admitted outcome is
censored. All nine primary replay paths carry `gap_before_quote`; complete API
responses do not establish continuous executable quotes between observations.
These flags do not establish that the broker omitted records, but prevent a
qualified continuity claim. Original native source-quality counters are retained.

Primary trailing exits: three session-end closes, three stops, two gap-through
stops and one broker take-profit. At native quantity 100 this experiment exercises
the whole-position target path; it does not establish empirical partial-runner
performance. Original native synthetic tests still cover partial exits.

## What this isolates

- **Cost feasibility:** 11/20 primary episodes are vetoed, including all first-date
  preflight scenarios. A valid cost veto is an outcome, not a collection failure.
- **Predictive/execution outcome:** at the optimistic 2bp trailing sensitivity,
  results remain 4/9 wins (44.4%), mean −0.0911R and PF(R) 0.8048. This does not
  support reducing costs as a sufficient remedy for the tested candidate.
- **Trailing:** fixed-stop at 24bp gives 4/9 wins and mean −0.6274R. At 2bp it
  reaches mean +0.0068R/PF(R) 1.0137, but only 44.4% class wins and negative Brent
  expectancy. These controls do not qualify a trailing-off replacement.
- **Peer dependency:** own and peer momentum have identical returns on all eight
  matched closes. The peer changes direction before fill for SpotCrude on 1 August,
  so that control skips one otherwise simulated own-momentum entry. No incremental
  peer value is demonstrated on matched episodes.
- **Always-long comparison:** own momentum's class mean matched advantage is only
  +0.0064R. Crude is +0.3353R, while Brent is −0.6514R. An aggregate would conceal
  a failing constituent; the per-symbol control gate correctly fails.

No unique causal explanation is proven from this sample. Price-only first-hour
direction with these exits and cost geometry has not demonstrated a deployable
edge. Counts, fills and returns are evidence about this experiment, not proof
that all oil trading or quantitative trading fails.

## Execution and verification

Plan frozen at `7e7d407`, SHA256
`d35c0ff1b6379d305ee4f6294fe362a3e849121a618280c0ed269cb4d6efedfb`.
All 20 new jobs used the fixed demo account/symbols/dates/volume, one active job
and two submissions per invocation. Two-episode technical preflight passed
before the remaining18. No retries, substitute dates or parameter search.
Raw source storage is 4,297,306 bytes, kept outside Git and the production DB.

GitHub temporarily returned HTTP401 after PR creation. The authorised bounded
research ran from committed, locally tested code while merging was blocked.
GitHub access subsequently recovered. CI must pass on the final reviewed commit
before merge; the PR closure records that later status.

Review caught an overly strict rejection of equal-millisecond same-side ticks.
A regression failed before the fix and passed after. Validation now accepts
nondecreasing timestamps, matching the native collector, and retains native
ambiguity/quality counters. None of these20sources has a same-side timestamp tie,
so the repair changes no collected source or frozen feature/exit rule. Preflight
and analysis were rechecked locally without any new broker collection.

All20source hashes and original deployed/offline study equality were verified.
An independent calculation from native fills agrees with all156 scenario net-R
values, including volume-weighted exit costs. Local suite:26passed/1optional
earlier-corpus reproduction skipped. Both native binaries built. No Rust/live
strategy or permission change. Exact raw reproduction requires the saved source
files; published derived JSON alone is not a complete independent raw replay.

## Stopping decision

Park this oil candidate for promotion and keep validation closed. Preserve the
frozen negative evidence. Any future investigation needs an independently
justified mechanism and prospective evidence; changing costs, stops or dates to
rescue this batch is outside this plan. Prior inspected dates and pooled results
remain descriptive and cannot override the standalone failure.
