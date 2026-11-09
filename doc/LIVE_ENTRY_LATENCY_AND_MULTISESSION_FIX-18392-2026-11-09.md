# Live-entry and multi-session repair — verified release report

№18,392 §§1–11 · 09-11-2026 10:12:25 SGT · by Codex (GPT-6; effort inherited, not independently exposed)

Repository: https://github.com/ang-kl/jiaoyi · Branch: `fix/live-entry-multisession-18392`

**Status: PR #204 merged; exact commit deployed; all four account loops verified. Current oil/index opening-data gaps and live latency remain unresolved evidence limits.**

## §1 · TLDR — deployed, with explicit data limits

[PR #204](https://github.com/ang-kl/jiaoyi/pull/204) merged as `6aa5100a5055ec786e16fc05f8069113b00d7eff` after all three CI jobs passed. Railway deployed that exact commit successfully. Two post-warm-up snapshots verified four connected, unpaused actors running the new policy, processing fresh quotes and advancing exit checks. Railway reports one running instance and the old deployment removed.
All 32 account/symbol macro contexts are ready. Only 16 of 32 current opening references are available: BTC, ETH, gold and silver have them; crude, Brent, US500 and NAS100 lack the original first-six-second two-sided opening quote on all four accounts. Those Asian-session entries remain blocked. Captured broker calendar anchors match; missing recording origin is not established.
The new actual-actor test measured **7.364 ms** with local PostgreSQL and a synthetic broker. New live dispatch below 250 ms and improved win rate remain unverified. No manual verification order was placed.

The missing cross-currency valuation and long dispatch path are concrete engineering defects. The bounce refusal and below-minimum quantity refusal followed the earlier registered rules; the owner has now authorised bounded changes to those policies. Those changes require their own evidence and do not demonstrate a trading edge.

## §2 · What happened at approximately 07:11 SGT

The owner directive reproduces the earlier №18,384 diagnostic. The following identifies the observed refusal and the repair; it does not assert that every refused entry would have won.

| Account | Observed refusal | Repair | Evidence status |
|---|---|---|---|
| 46130058 · Demo | About 5.2 seconds elapsed against a 5-second dispatch deadline. | Background prepared admission; final writer guards and durable intent retained. | Deployed; live timing unverified. |
| 43002148 · Live | Existing foreign exposure lacked a fresh verified conversion in the margin calculation. | Removed candidate-currency-only valuation restriction; broker-identified fresh FX paths cover existing exposure. | Regression passed; live account now records subscribed, fresh JPY→USD path provenance. Writer freshness remains mandatory. |
| 47790949 · Demo | Latest quote no longer satisfied the original pullback-touch rule. | Owner-approved 0.10-impulse bounce allowance after a genuine fresh touch. | New policy and focused integration passed; policy identity verified on deployed account. Natural entry outcome unverified. |
| 49236051 · Demo | Positive calculated volume below broker minimum. | Owner-approved minimum/step floor subject to existing cash-risk, aggregate-risk and margin caps. | New policy and focused integration passed; policy identity verified on deployed account. Natural entry outcome unverified. |

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

## §5 · Verification evidence

The counts below are separate suites, not a combined financial certification. Final CI on `d9e02bc` passed check, docker and engineering. The Rust job logged 773 passing tests across workspace/example steps and one existing ignored test.
| Check | Measured result | Limit |
|---|---|---|
| Actual S3 actor + local PostgreSQL + synthetic broker | Passed; 7.364 ms touch-to-write; one intent, order record and broker order; zero admission/history broker requests between touch and write | A local fixture, not measured Railway latency |
| Strategy 3 integration | 78 passed; 1 existing manual benchmark ignored | Covers new session fixtures and preserved historical position management |
| Admission evidence review | 7 passed, including six bounded-policy variants | Missing evidence stays unknown |
| FX | 3 arithmetic/freshness tests, 1 broker-path integration and 10 existing margin regressions passed | No real broker verification order |
| Atomic settings/intent/order | 2 PostgreSQL tests passed | Concurrent settings change, duplicate and rollback refusal |
| Production calendar/preparation/storage | 12 focused tests passed | Actual broker proof remains mandatory at writer |
| Broader checks | Strict Clippy, 113 research tests (1 skipped), 12 browser lifecycle tests, 191 engineering tests passed | Final required CI passed |

The wider local run found one outdated S1 fixture: it supplied USDJPY quotes without the broker asset identities now required by the conversion guard. The fixture was corrected; its original missing-rate refusal, eventual order and size assertions still pass. Production conversion freshness and risk guards were not relaxed. The final actor test includes the durable database path; an earlier 1.362 ms gate-only fixture excluded S3 consumption and is not substituted for that measurement.

The HTML report was checked at 1366 px and 390 px widths with no page overflow. Raw logs and secret values are excluded.

## §6 · Completed release gates and remaining observations

1. **Passed:** integrated Rust tests, formatting, strict Clippy, browser lifecycle, research scorecards and both deploy images on the exact final PR revision.
2. **Passed:** source/FX/calendar proof binding, settings invalidation, bounded sizing, atomic durability, duplicate refusal and historical-exit regression coverage.
3. **Passed:** PR #204 squash merge and Railway commit identity; previous deployment removed and exactly one running instance observed.
4. **Passed:** four unpaused account actors, current macro preparation 8/8 per account, fresh processed quotes and increasing exit checks across two snapshots. Initial history warm-up settled in roughly one minute.
5. **Remaining data gap:** current oil/index opening references are absent. Do not replace them with later quotes. At the next scheduled window, check actual opening capture and report readiness separately from connection health. Some late-window coefficients remain unavailable after the DST transition; their unsupported marks stay blocked.
6. **Remaining live evidence:** observe the next naturally qualifying entry for touch-to-dispatch, writer quote, fill slippage and refusal reasons. Do not manufacture a trade to meet a test.
7. **Remaining financial evidence:** evaluate completed positions independently per strategy/account against the latest 20 benchmark. No win-rate improvement or financial pass is claimed by this release.

## §7 · Preserved boundaries, goals and conclusions

The production policy is versioned as s3_macro_pullback_owner_18392_v1. Existing owned positions retain their recorded stop, target, original risk, exit policy and session deadline. An externally opened position is observed for account risk; that does not grant authority to amend or close it.
The new-entry universe remains the eight core symbols. The 48 bp impulse floor, causal five-/twenty-day price-drift alignment, peer coherence where available, structural stop, +1R breakeven transition and +1.38R gross target remain. “Macro” here means a price-drift filter, not demonstrated fundamental or news analysis. The 3.74 bp/side operating reserve is an owner-selected allowance, not a newly verified round-trip cost estimate.
The project goal remains at least 73% net wins over the latest 20 complete trades, independently for each strategy and each account, with the 1:1.38 risk:reward objective. At twenty observations, at least fifteen wins are required to exceed 73%. No new realised trading evidence in this repair establishes progress toward that goal; financial_pass is false.
Time is money. Preparation, traceable evidence and a bounded correction are the deliverable; activity alone is not success. Impulse and drift filters do not guarantee that quiet-session entries avoid losses.

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

Release evidence is recorded below and in `docs/live_entry_multisession_release_verification_18392.json`. A separate report-only PR publishes these post-deployment observations; it does not alter the deployed executable.

## §10 · Observed release receipt and current account readiness

- Code PR: [#204](https://github.com/ang-kl/jiaoyi/pull/204), merged `2026-11-09T02:05:15Z`.
- Deployed commit: `6aa5100a5055ec786e16fc05f8069113b00d7eff`.
- Railway deployment: `806ccc1d-4e9f-41d6-8b20-d36e2c7d98de`, SUCCESS.
- Old deployment: `306a5080-1f2f-401b-923b-2b545499e8c7`, REMOVED. One current running instance verified through Railway metadata.
- Health observations: `2026-11-09T02:08:44.177190+00:00` and `2026-11-09T02:09:58.056057+00:00`.
- Current policy: `s3_macro_pullback_owner_18392_v1` on all four accounts. Strategy 1 actors also report alive.

| Account | Connected / paused | Macro contexts | Opening references | Exit-check progress | Processed quote age at second snapshot |
|---|---|---|---|---|---|
| 43002148 | Connected / not paused | 8/8 | 4/8 | 40 → 65 | 0.292s |
| 46130058 | Connected / not paused | 8/8 | 4/8 | 41 → 66 | 0.243s |
| 47790949 | Connected / not paused | 8/8 | 4/8 | 41 → 66 | 0.243s |
| 49236051 | Connected / not paused | 8/8 | 4/8 | 41 → 66 | 0.243s |

All four rows lack the same current opening references: SpotCrude, SpotBrent, US500 and NAS100. This is a material entry-readiness limitation, not a claim that all eight symbols can enter now. Gold/silver and BTC/ETH still require an admissible impulse, trend/peer agreement, a retest and all risk checks. No guaranteed trade follows from readiness.

The older operations margin card reads separate legacy margin receipts and may remain stale/null for prepared entries. Selected prepared margin proof is atomically retained in strategy3_orders.detail.prepared_admission.margin; execution and stop-loss review use the retained order evidence. No extra display change was added to this repair.

## §11 · Copy-paste for Gemini review

```json
{
  "trace": "№18,392",
  "status": "deployed_and_verified_with_remaining_data_limits",
  "pr": 204,
  "merge_commit": "6aa5100a5055ec786e16fc05f8069113b00d7eff",
  "four_account_bug_fixes": {
    "live_43002148_exposure_fx_conversion_fixed": true,
    "demo_46130058_dispatch_latency_reduced": "Earlier live path about 5200 ms; revised local actor/PostgreSQL/synthetic-broker test 7.364 ms. New live dispatch latency remains unobserved.",
    "demo_47790949_pullback_tolerance_fixed": true,
    "demo_49236051_min_volume_floor_fixed": true
  },
  "multisession_schedule_enabled": {
    "crypto_24h_rolling_no_0730_blackout": true,
    "metals_energy_indices_sydney_asia_london_ny_enabled": true,
    "current_active_evaluation_windows_sgt": {
      "BTCUSD_ETHUSD": "08:00–12:00 SGT; current opening references and macro context available.",
      "XAUUSD_XAGUSD_SpotCrude_SpotBrent": "Asia to 15:00 SGT. Gold/silver opening references available; crude/Brent blocked by missing original opening quotes.",
      "US500_NAS100": "Asia 07:00–15:00 SGT; blocked by missing original opening quotes."
    }
  },
  "railway_deployment_status": "SUCCESS",
  "railway_deployment_id": "806ccc1d-4e9f-41d6-8b20-d36e2c7d98de",
  "account_loops_verified": 4,
  "single_running_instance_verified": true,
  "current_macro_contexts_ready": 32,
  "current_opening_references_available": 16,
  "current_opening_references_expected": 32,
  "live_dispatch_latency_verified": false,
  "financial_pass": false,
  "observed_at": "2026-11-09T02:09:58.056057+00:00",
  "report_file": "doc/LIVE_ENTRY_LATENCY_AND_MULTISESSION_FIX-18392-2026-11-09.md"
}
```
