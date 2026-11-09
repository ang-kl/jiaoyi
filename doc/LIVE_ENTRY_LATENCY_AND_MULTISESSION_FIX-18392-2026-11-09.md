# Live-entry and multi-session repair — interim report

№18,392 §§1–9 · 09-11-2026 09:50:58 SGT · by Codex (GPT-6; effort inherited, not independently exposed)

Repository: https://github.com/ang-kl/jiaoyi · Branch: `fix/live-entry-multisession-18392`

**Status: implementation and focused integration verified; PR #204 awaiting CI; not deployed.**

## §1 · TLDR — implementation in progress

The repair is committed in [PR #204](https://github.com/ang-kl/jiaoyi/pull/204), with focused integration checks passing. It is not yet merged or deployed. Production improvement, live dispatch below 250 ms and a higher win rate have not been verified. No verification order has been placed.
The missing cross-currency valuation and long dispatch path are concrete engineering defects. The bounce refusal and below-minimum quantity refusal followed the earlier registered rules; the owner has now authorised bounded changes to those policies. Those changes require their own evidence and do not demonstrate a trading edge.

## §2 · What happened at approximately 07:11 SGT

The owner directive reproduces the earlier №18,384 diagnostic. The following identifies the observed refusal and the repair; it does not assert that every refused entry would have won.

| Account | Observed refusal | Repair | Evidence status |
|---|---|---|---|
| 46130058 · Demo | About 5.2 seconds elapsed against a 5-second dispatch deadline. | Background prepared admission; final writer guards and durable intent retained. | Implemented; live timing unverified. |
| 43002148 · Live | Existing foreign exposure lacked a fresh verified conversion in the margin calculation. | Removed candidate-currency-only valuation restriction; broker-identified fresh FX paths cover existing exposure. | Unit + broker-fixture regression passed; production proof pending. |
| 47790949 · Demo | Latest quote no longer satisfied the original pullback-touch rule. | Owner-approved 0.10-impulse bounce allowance after a genuine fresh touch. | New policy and focused integration passed; production verification pending. |
| 49236051 · Demo | Positive calculated volume below broker minimum. | Owner-approved minimum/step floor subject to existing cash-risk, aggregate-risk and margin caps. | New policy and focused integration passed; production verification pending. |

## §3 · What is implemented

Prepared admission moves account, margin, sizing and plan reads into a background worker while a setup is armed. A receipt binds the exact command and policy, account, connection generation, account revision, local admission epoch and expiry. Changed account evidence invalidates reuse. Restored arms also require their verified original opening bid/ask midpoint and matching causal macro context; an arithmetic-only state snapshot cannot authorise a prepared entry. Missing source evidence is exposed as `source_refusal` without substituting a later quote. The actor still owns dispatch and exits.
Durable exactly-once intent remains before the socket write. The writer rechecks the latest executable quote, costs, peer, account, expiry and FX evidence. A database durability failure still refuses an order. No synchronous broker round trip is intended on the prepared dispatch route; a production latency receipt is still required to establish its duration. Unused background margin receipts stay in memory; the selected proof is persisted with the actual intent/order. The actor regression verifies zero unused warm-margin rows, avoiding database growth on each refresh.
FX repair values existing positions and pending entries in their own quote currencies. Broker conversion metadata supplies asset identities; every conversion leg must have finite, positive, uncrossed bid/ask quotes no more than five seconds old. Direct and inverse conversion use conservative sides, and liability reserves are recomputed again at the writer. A missing rate is not zero and a foreign rate is not silently one. Historical bars are not accepted as executable fresh FX.
A separate read-only FX worker subscribes required conversion legs at startup and after account changes, with periodic recovery. It runs independently of entry/exit handlers. Account-specific paths and availability changes are recorded as s3_exposure_fx; /health exposes that recorded provenance and explicitly distinguishes it from a live quote.
The bounded pullback policy requires a real fresh 50% retracement touch before permitting up to 10% of the original impulse range of bounce. Sizing and brackets fund the permitted quote range. A plain market order cannot guarantee its eventual fill stays inside that range; post-dispatch slippage remains separately measured.
For demos only, a positive sub-minimum calculated quantity may be raised to the valid broker minimum/step if the existing minimum-volume cash-risk cap, aggregate open-risk cap and margin floor all pass. Live retains one broker-minimum program trade at a time.

## §4 · Multiple sessions and preparation

BTCUSD and ETHUSD use four-hour windows anchored at 00:00, 04:00, 08:00, 12:00, 16:00 and 20:00 UTC: 08:00, 12:00, 16:00, 20:00, 00:00 and 04:00 SGT. A structurally valid crypto arm may cross an adjacent window boundary until its four-hour impulse expiry; a clock reset alone does not cancel it at 07:30 SGT.
SpotCrude, SpotBrent, XAUUSD, XAGUSD, US500 and NAS100 evaluate the broker-active Asian, London and New York parts of the day. London is referenced to 07:00 UTC / 15:00 SGT; New York to 09:30 America/New_York. Actual Asian reopenings, daily breaks and holiday availability follow the broker contract calendar, including symbol-specific minute offsets. The schedule is not permission to trade during a closed market.
Window IDs use exact UTC opening timestamps so preparation, restored arms, vetoes and one-entry accounting can distinguish multiple windows on one date. Preparation uses causal 45-calendar-day M30 history and corresponding historical windows; five-day and twenty-day drift must represent prior days rather than five or twenty four-hour blocks.
The first half-hour uses the prepared first-half-hour sigma and the 48 bp impulse floor. Unarmed discovery is assessed at most once per second; an armed setup checks fresh quotes. Missing history or an unavailable fresh opening reference can still block a session. There is no claim of guaranteed immediate readiness.

## §5 · Verification so far

The counts below are separate suites, not a combined production certification. They are an interim snapshot; the main agent will update this report after the remaining checks and release verification.
| Check | Measured result | Limit |
|---|---|---|
| Actual S3 actor + local PostgreSQL + synthetic broker | Passed; 7.364 ms touch-to-write; one intent, order record and broker order; zero admission/history broker requests between touch and write | A local fixture, not measured Railway latency |
| Strategy 3 integration | 78 passed; 1 existing manual benchmark ignored | Covers new session fixtures and preserved historical position management |
| Admission evidence review | 7 passed, including six bounded-policy variants | Missing evidence stays unknown |
| FX | 3 arithmetic/freshness tests, 1 broker-path integration and 10 existing margin regressions passed | No real broker verification order |
| Atomic settings/intent/order | 2 PostgreSQL tests passed | Concurrent settings change, duplicate and rollback refusal |
| Production calendar/preparation/storage | 12 focused tests passed | Actual broker proof remains mandatory at writer |
| Broader checks | Strict Clippy, 113 research tests (1 skipped), 12 browser lifecycle tests, 191 engineering tests passed | Full CI remains the release gate |

The wider local run found one outdated S1 fixture: it supplied USDJPY quotes without the broker asset identities now required by the conversion guard. The fixture was corrected; its original missing-rate refusal, eventual order and size assertions still pass. Production conversion freshness and risk guards were not relaxed. The final actor test includes the durable database path; an earlier 1.362 ms gate-only fixture excluded S3 consumption and is not substituted for that measurement.

The HTML report was checked at 1366 px and 390 px widths with no page overflow. Raw logs and secret values are excluded.

## §6 · Remaining release checks

1. Completed locally: integrated compilation, strict Clippy and focused regressions for receipt invalidation, missing/stale FX, real touch versus pre-touch bounce, bounded quantity floor, calendar/window identity and old-position exits.
2. Completed review: prepared receipts require matching source/FX/calendar evidence, settings changes invalidate reuse, and account-wide capacity and risk limits remain effective.
3. PR #204 is open with this chat serial, paragraph references and SGT time. Require check, docker and engineering to pass on its final head before squash-merging.
4. Verify the Railway deployment commit and all four account loops. Inspect read-only FX provenance, active window and preparation status. Do not send a manual test trade.
5. Observe the next naturally qualifying entry and measure touch-to-intent/write latency, writer quote, broker fill and any refusal. The less-than-250-ms target remains unverified until measured under the deployed workload.
6. Review completed trades using the established account/strategy benchmark. Deployment and passing tests are engineering evidence, not a financial pass.

## §7 · Preserved boundaries, goals and conclusions

The production policy is versioned as s3_macro_pullback_owner_18392_v1. Existing owned positions retain their recorded stop, target, original risk, exit policy and session deadline. An externally opened position is observed for account risk; that does not grant authority to amend or close it.
The new-entry universe remains the eight core symbols. The 48 bp impulse floor, causal five-/twenty-day price-drift alignment, peer coherence where available, structural stop, +1R breakeven transition and +1.38R gross target remain. “Macro” here means a price-drift filter, not demonstrated fundamental or news analysis. The 3.74 bp/side operating reserve is an owner-selected allowance, not a newly verified round-trip cost estimate.
The project goal remains at least 73% net wins over the latest 20 complete trades, independently for each strategy and each account, with the 1:1.38 risk:reward objective. At twenty observations, at least fifteen wins are required to exceed 73%. No new realised trading evidence in this repair establishes progress toward that goal; financial_pass is false.
Time is money. Preparation, traceable evidence and a bounded correction are the deliverable; activity alone is not success.

## Reference files

- `docs/preregistration-2026-11-09-strategy3-macro-pullback-18392.md`
- `crates/jiaoyi-ctrader/src/margin/fx.rs` and `tests/exposure_conversion.rs`
- `crates/jiaoyi/src/strategy3/fast_entry.rs` and `strategy3/exposure_fx.rs`
- `crates/jiaoyi-core/src/strategy3/production_sessions.rs`

## §8 · Operational schedule in Singapore time

On 9 November, when New York is on standard time, the active crypto window is 08:00–12:00 SGT. Traditional Asian windows run from the broker reopening to 15:00; London runs 15:00–22:30; New York runs 22:30–05:59 the following day (Friday closes at 05:55 Saturday). Most Monday Asian reopenings are 07:00 SGT; metals first trade at 07:01. Brent has a later 09:00 SGT reopening on Tuesday–Friday. New York daylight saving shifts its local anchors by one hour in SGT. Actual account-specific broker closures, holidays and permissions override these evaluation windows.

A missing original opening quote or complete causal history remains a legitimate blocker. The repair retries original opening evidence within a bounded first minute instead of unnecessarily waiting five minutes after a recorder flush delay. It never invents an opening midpoint from a later quote. Two concurrent background preparations per account are bounded; eight simultaneous arms are not guaranteed to retain fresh receipts. Health exposes readiness and refusal evidence.

## §9 · Agent responsibilities and release evidence

The main agent integrated the production actor and owns the release. Six delegated roles covered prepared admission/sizing, FX exposure, session calendars/storage, bounce and exit preservation, admission review/regression fixtures, and independent safety/atomic persistence. These engineering agents do not autonomously manage broker positions; the deployed deterministic actors retain execution authority.

Before a touch: causal preparation → arm → background order/account proofs. At a touch: durable consumption → atomic settings/intent/order → fresh writer checks → broker frame. After dispatch: broker receipt, reconciliation and existing protective exits. The UI is an observer of this server workflow.

Release evidence is pending. [PR #204](https://github.com/ang-kl/jiaoyi/pull/204) contains the implementation. Its required `check`, `docker` and `engineering` outcomes, actual merge SHA, Railway SHA, four-account liveness and any observed production entry timing will be added after verification. No deployed or financial PASS is inferred from local tests.
