# PR #214 engineering release; exit experiment rejected

№18,414 §§1–6 · 10-11-2026 09:13 SGT · by <Codex (GPT-6), effort: not exposed>.

The owner authorised the engineering-only release after rejecting the exit experiment in №18,412. PR #214 now repairs session readiness, dispatch-evidence parsing and goal assessment consistency. It removes the executable profit-lock experiment and preserves its measured rejection record. This approval supersedes the earlier combined-release hold; it does not approve the experimental exit policy.

## Retained repairs

| Repair | Behaviour and verification |
| --- | --- |
| P1 `load_day` readiness | Verified causal preparation plus the original first-six-second quote can initialise the session without waiting for its first M30 candle to close. Missing original quotes still refuse entry. The PostgreSQL/fake-broker regression covers all eight core symbols at open +5 seconds. |
| P1 dispatch evidence | Explicit historical policy schemas recognise the six recorded `owner_18409_v1` confirmed-turn receipts. Identity, side, chronology and finite-price checks remain mandatory; unknown policies remain unknown. Touch-to-guard and confirmation-to-guard timings are separate. |
| P2 goal assessment | Outcomes, settings and history share one read-only PostgreSQL repeatable-read snapshot. Concurrent commits become visible on the next poll. Genuine missing, stale or future history remains unknown. |

## Trading policy retained

- The actual production policy identifier remains `s3_macro_pullback_owner_18409_v1`; this release does not introduce a new trading hypothesis or registration hash.
- Baseline exits remain +1.00R favourable excursion → stop at entry, with +1.38R gross target. Crypto uses four-hour execution windows; traditional instruments retain their native session deadlines and existing positions retain their recorded deadlines.
- Pair reservations, running-extreme/61.8% invalidation, touch-then-turn confirmation, peer checks, account risk limits and the Three Laws remain unchanged.
- No pause, risk, credential, allowance or environment-variable changes are needed. No discretionary broker order is used to verify the release.

## Rejected experiment and audit provenance

Removed `crates/jiaoyi-core/src/research/profit_lock.rs`, its native example and `ops/research/profit_lock_18412.py`. The remaining diff introduces no `jiaoyi-core` strategy change. The +0.65R breakeven / +0.80R → +0.50R experiment is not an executable release feature.

The [№18,412 measured audit](READINESS_PARSER_AND_PROFIT_LOCK_REPAIR-18412-2026-11-10.md), HTML, JSON and frozen plan remain historical evidence. Reproducing that rejected experiment requires checking out immutable commit `9720251ee8cf45ab900ebfe4cb6ea141b4b202d4`, which contains the referenced replay tools. Its earlier “draft/undeployed” wording describes the conditional audit at that time; this release note records the later engineering-only approval. The experiment failed the historical comparison, including negative Tier-1 stress expectancy. No financial pass is claimed.

## Verification and deployment receipt

Local verification after removal: workspace formatting and all-target compilation pass; Strategy 3 integration tests 85 passed / 1 existing ignored benchmark; snapshot concurrency 1 passed; goal integration tests 15 passed; dispatch parser tests 4 passed; strict all-target workspace Clippy passes.

The release requires fresh `check`, `docker` and `engineering` CI success on the final PR head. Both services need the merged code: Rust repairs in `jiaoyi`, Python parser repairs in `jiaoyi-engineering`. The latter uses its existing file-upload deployment route, with a clean bundle from the merged commit and a runtime parser hash check.

**The authoritative final receipt is [PR #214 deployment verification](https://github.com/ang-kl/jiaoyi/pull/214#issuecomment-6755948964).** It is updated after merge and deployment with the actual merge SHA, CI results, both Railway deployment IDs, connected/unpaused account count, preserved original opening references and parser evidence. This document does not pre-assert successful deployment. Original +1R/+1.38R behaviour is established by unchanged strategy code and regression coverage; new live trades are not manufactured as a test.

The project goal remains at least 73% net wins and 1:1.38 risk:reward over the latest 20 trades per strategy/account. Engineering correctness does not establish goal attainment. “Remember that Time is Money”; no further optimisation is included in this release.
