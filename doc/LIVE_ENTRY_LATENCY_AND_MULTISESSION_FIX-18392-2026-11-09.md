# Live-entry and multi-session repair — interim report

№18,392 §§1–7 · 09-11-2026 09:15:31 SGT · by Codex (GPT-6; effort inherited, not independently exposed)

Repository: https://github.com/ang-kl/jiaoyi · Branch: `fix/live-entry-multisession-18392`

**Status: implementation and integration testing in progress; not merged; not deployed.**

## §1 · TLDR — implementation in progress

The repair is implemented in the working branch and focused checks have passed. It is not yet merged or deployed. Production improvement, live dispatch below 250 ms and a higher win rate have not been verified. No verification order has been placed.
The missing cross-currency valuation and long dispatch path are concrete engineering defects. The bounce refusal and below-minimum quantity refusal followed the earlier registered rules; the owner has now authorised bounded changes to those policies. Those changes require their own evidence and do not demonstrate a trading edge.

## §2 · What happened at approximately 07:11 SGT

The owner directive reproduces the earlier №18,384 diagnostic. The following identifies the observed refusal and the repair; it does not assert that every refused entry would have won.

| Account | Observed refusal | Repair | Evidence status |
|---|---|---|---|
| 46130058 · Demo | About 5.2 seconds elapsed against a 5-second dispatch deadline. | Background prepared admission; final writer guards and durable intent retained. | Implemented; live timing unverified. |
| 43002148 · Live | Existing foreign exposure lacked a fresh verified conversion in the margin calculation. | Removed candidate-currency-only valuation restriction; broker-identified fresh FX paths cover existing exposure. | Unit + broker-fixture regression passed; production proof pending. |
| 47790949 · Demo | Latest quote no longer satisfied the original pullback-touch rule. | Owner-approved 0.10-impulse bounce allowance after a genuine fresh touch. | New policy implemented; final integration pending. |
| 49236051 · Demo | Positive calculated volume below broker minimum. | Owner-approved minimum/step floor subject to existing cash-risk, aggregate-risk and margin caps. | New policy implemented; final integration pending. |

## §3 · What is implemented

Prepared admission moves account, margin, sizing and plan reads into a background worker while a setup is armed. A receipt binds the exact command and policy, account, connection generation, account revision, local admission epoch and expiry. Changed account evidence invalidates reuse. The actor still owns dispatch and exits.
Durable exactly-once intent remains before the socket write. The writer rechecks the latest executable quote, costs, peer, account, expiry and FX evidence. A database durability failure still refuses an order. No synchronous broker round trip is intended on the prepared dispatch route; a production latency receipt is still required to establish its duration.
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
Observed local results: FX arithmetic and freshness unit tests 3 passed; broker integration regression 1 passed; existing margin gate regression selection 10 passed. The integration fixture exercises an external JPY-denominated holding with a USD candidate, requests the broker conversion path once, subscribes once, reuses the cache and sends zero orders.
The main agent reports research checks: 113 passed and 1 skipped; UI lifecycle checks: 12 passed. The application library compiles. The workspace-wide run had three fixture type errors being adapted at this snapshot, so the complete workspace suite and CI are not yet green. Earlier bounce/core focused tests reported 12 passed, but the final integrated run remains outstanding.

## §6 · Remaining release checks

1. Finish integrated Rust compilation and the relevant regression suites after the shared agent changes settle. Exercise receipt invalidation, missing/stale FX, real touch versus pre-touch bounce, bounded quantity floor, calendar/window identity and old-position exit compatibility.
2. Review that new prepared receipts require matching FX/calendar evidence, that changed settings invalidate reuse and that account-wide capacity and risk limits remain effective.
3. Open the PR with this chat serial, paragraph references and SGT time; require check, docker and engineering to pass before squash-merging.
4. Verify the Railway deployment commit and all four account loops. Inspect read-only FX provenance, active window and preparation status. Do not send a manual test trade.
5. Observe the next naturally qualifying entry and measure touch-to-intent/write latency, writer quote, broker fill and any refusal. The less-than-250-ms target remains unverified until measured under the deployed workload.
6. Review completed trades using the established account/strategy benchmark. Deployment and passing tests are engineering evidence, not a financial pass.

## §7 · Preserved boundaries, goals and conclusions

The production policy is versioned as s3_macro_pullback_owner_18392_v1. Existing owned positions retain their recorded stop, target, original risk, exit policy and session deadline. An externally opened position is observed for account risk; that does not grant authority to amend or close it.
The new-entry universe remains the eight core symbols. The 48 bp impulse floor, causal five-/twenty-day price-drift alignment, peer coherence where available, structural stop, +1R breakeven transition and +1.38R gross target remain. “Macro” here means a price-drift filter, not demonstrated fundamental or news analysis. The 3.74 bp/side operating reserve is an owner-selected allowance, not a newly verified round-trip cost estimate.
The project goal remains at least 73% net wins over the latest 20 complete trades, independently for each strategy and each account, with the 1:1.38 reward/risk objective. At twenty observations, at least fifteen wins are required to exceed 73%. No new realised trading evidence in this repair establishes progress toward that goal; financial_pass is false.
Time is money. Preparation, traceable evidence and a bounded correction are the deliverable; activity alone is not success.

## Reference files

- `docs/preregistration-2026-11-09-strategy3-macro-pullback-18392.md`
- `crates/jiaoyi-ctrader/src/margin/fx.rs` and `tests/exposure_conversion.rs`
- `crates/jiaoyi/src/strategy3/fast_entry.rs` and `strategy3/exposure_fx.rs`
- `crates/jiaoyi-core/src/strategy3/production_sessions.rs`

This is an interim report. PR, merged SHA, Railway deployment SHA and live latency must be appended only after observed verification.
