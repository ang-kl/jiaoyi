# Monday live readiness and Stage 1 preparation audit

№18,380 §§1–9 · 2026-11-08T21:17:07.490426+08:00 · by <Codex (GPT-6), effort: not exposed>.

## TLDR

All four accounts were already unpaused. This change replaces the new-entry 24bp/side allowance with the owner-selected 3.74bp/side allowance, repairs late US500 recording subscriptions, and warms current/next native sessions immediately. It also prevents incomplete tomorrow context from replacing today's preparation. These are measured engineering repairs, not proof of a trading edge.

**Stage 1 could not reach tick collection or replay.** All 112 new M30 requests decoded successfully, but only 30/40 episodes have complete required preparation. Both crypto symbols lack 77–96 expected M30 bars per episode. The mandatory all-40 preparation barrier correctly refused every tick request. There are **zero evaluated trades, no win rate and no measured improvement toward the goal**. This is a data-qualification failure, not a strategy-loss result.

**Production activation verified:** PR #201 merged as `0f3b89c700a33f98891b47827b99df550bbe0cea` after all three CI checks passed. Railway runs that commit. Two subsequent snapshots verify all four accounts unpaused and progressing, **32/32 core subscriptions and 32/32 complete policy preparation contexts**, and 3.74 bp/side on every account. Coverage is the current crypto sessions and Monday's six non-crypto sessions; the next crypto session finalises at its boundary. See §10 for the measured release evidence. This successful deployment does not turn the historical preparation failure into a completed backtest.

## §1. Authority and two distinct policies

The authoritative source is the owner's Combined Directive №18,380, 8 November 2026. It authorises switch-on if needed, the 3.74bp production assumption, genuine warm-up/veto repairs and one reserved 40-episode historical cohort. No manual verification orders or Telegram messages were sent.

Production family remains `s3_macro_pullback_v1`, with exact revised policy `s3_macro_pullback_owner_18380_v1`. The independent test remains the **original Phase-4** candidate at `a0d7aea148e73550cc63993228b4ef9b368f0c43`. The live owner revision uses observed impulse displacement and midpoint retest; the frozen research candidate uses its own unchanged admission/context rules. Results from one cannot certify the other.

The 3.74bp allowance is an explicit owner choice, **not a verified full round-trip cost calibration**. Earlier 7/7 development results were reused observations under different rules/assumptions; they are not independent evidence for this deployment. The 24bp stress repricing remains in research.

## §2. Account switches and confirmed subscription defect

| Account | S1 paused | S3 paused | Loop | Recorder core coverage before fix |
| --- | --- | --- | --- | --- |
| 43002148 | False | False | alive | 7/8; US500 ID10013 absent |
| 46130058 | False | False | alive | 7/8; US500 ID10013 absent |
| 47790949 | False | False | alive | 7/8; US500 ID10013 absent |
| 49236051 | False | False | alive | 7/8; US500 ID10013 absent |

No unpause write was needed. Existing risk, margin, ownership and live one-broker-minimum-position constraints remain. S2 is not newly activated by this change.

The loader resolved US500 after recorder startup, while recorder discovery previously waited for hourly maintenance. The recorder now checks every 30 seconds, subscribes only new IDs on the existing connection, and records the request/acknowledgement times. It preserves buffered quotes and sequence continuity and does not backdate new-symbol coverage. All core names being configured is distinct from acknowledged subscriptions and from market-open fresh quotes.

## §3. Cost, entry and existing-position invariants

- Keep the observed impulse floor at **48bp**, macro direction and optional fresh-peer agreement.
- Admission and structural risk use **current bid/ask spread + existing recorded/fallback commission inputs + 2 × 3.74bp × executable price**.
- The final writer refuses stale retests, moved admitted prices and widened spreads that invalidate cost feasibility. The new spread recheck is scoped to pullback signals; it does not rewrite S1 policy.
- New positions retain structural −1R, breakeven only after +1R, full +1.38R gross target and session-end handling.
- Existing 18,377 positions retain their original R and two-stage exit policy. Legacy positions retain their own recorded exits. New registration-scoped state cannot reuse the previous stress-cost veto.

A smaller execution allowance reduces the assumed cost component of R. It does not remove actual spread, fees, financing, slippage or the broker-minimum/live risk checks. Realised net reward and gross target geometry remain different measures.

## §4. Warm-up, weekend gaps and peers

Current and nearest upcoming native sessions are warmed on startup, with bounded retries for incomplete current/T−20 preparation. The current session is selected deterministically before tomorrow's cached context. Macro readiness and the precise preparation error are now visible in health and trigger-distance evidence.

Non-crypto weekends were already excluded by the native calendar. A regression confirms that weekends are skipped but a missing active-session M30 bar still refuses preparation. No bar is fabricated or forward-filled. Future crypto context cannot be complete until the last causal prior-session bar exists; an early snapshot is not a certificate of tomorrow's readiness.

Missing or stale peers remain explicitly unavailable and optional under the owner's policy. A fresh opposing peer can veto. Missing subscriptions are repaired at their source rather than being relabelled as directional disagreement.

Current captured broker crypto schedules show daily two-minute New York maintenance (six minutes for live Friday). None covers a whole M30 slot. They cannot justify removing the much longer missing 2024 intervals, nor do 2026 specifications certify historical 2024 hours.

## §5. Frozen Stage 1 selection and requests

Selected dates: **11–15 November 2024**. This reserved week maximises distinct verified cached source-file reuse: **29 files**, containing 2,786 reusable required slots across the cohort. Selection used coverage, not returns. All 40 episodes and source hashes were frozen before requests. Plan SHA256: `84ba8fb36dcad6a14ed6293d3559374580e33f3c6ea4e015af3371c127998497`.

Collection used 104 interval-only M30 preparation requests plus eight separately declared Friday opening-reference requests required by the unchanged native adapter. Opening references supply the open field only; their later high/low/close are not prior features. No `count=500` mode was used. The normal research collection pause remains; only the exact owner-authorised local runner received a plan-bound read capability. Default callers and order messages remain refused.

All 112 receipts and 29 cached hashes reconcile (141 files); every request followed the freeze and matched its declared interval. The append-only runner ledger prevents retrying spent or uncertain intervals. The capability assumes the trusted runner invokes it; it is not a sandbox against arbitrary code modifying local files.

## §6. Preparation results and why there is no return table

| Symbol | Complete episodes | Scheduled | Missing slots by Mon→Fri | Opening references |
| --- | --- | --- | --- | --- |
| BTCUSD | 0 | 5 | 96, 94, 81, 77, 77 | 5 |
| ETHUSD | 0 | 5 | 96, 94, 81, 77, 77 | 5 |
| NAS100 | 5 | 5 | 0, 0, 0, 0, 0 | 5 |
| SpotBrent | 5 | 5 | 0, 0, 0, 0, 0 | 5 |
| SpotCrude | 5 | 5 | 0, 0, 0, 0, 0 | 5 |
| US500 | 5 | 5 | 0, 0, 0, 0, 0 | 5 |
| XAGUSD | 5 | 5 | 0, 0, 0, 0, 0 | 5 |
| XAUUSD | 5 | 5 | 0, 0, 0, 0, 0 | 5 |

Each crypto symbol has 96 distinct absent slots over the combined required history, clustered around historical Friday/Saturday and some Sunday periods. Successful transport and decoding do not make absent bars present. All 40 opening references exist; the failure is the causal historical preparation, not an absent session opening.

The exact barrier returned `preparation_must_be_decoded_before_ticks`. Counts: **112 successful M30 receipts; zero failed network/decoding sources; 10 incomplete preparation episodes; zero tick requests; zero native replay episodes; zero evaluated trades**. Strict wins, forced-close wins, expectancy and profit factor are unavailable, not zero-performance estimates. No partial 30-episode sample, replacement dates, calendar retuning or extra trial was silently substituted.

## §7. Verification and release

Focused checks: nine shared macro/pullback tests, five pre-open tests, seventeen stop-loss review tests, seven entry-guard tests and 78 S3 integration cases passed (one existing manual benchmark ignored). Recorder provenance and flush-setting regressions passed. Research tests cover default collection pause, forbidden order/account/interval requests, interval-only M30 and refusal of ticks before the complete preparation proof. Exact receipt reconciliation retained all gaps and failures.

The full Rust workspace and the required `check`, `docker` and `engineering` CI jobs are release gates. Post-deploy verification must match the deployed commit and new registration, observe all four unpaused account loops progressing, inspect actual recorder IDs and each symbol's preparation reason. Successful HTTP health alone is insufficient. CI and deployment results are retained with the PR, including any unresolved readiness evidence.

## §8. Goal progress and next decision

The invariant is at least **15 net wins in the latest 20 completed positions**, separately per strategy/account, alongside the 1:1.38 reward/risk goal. This run adds **0 qualified independent trades**. Win-rate delta is **not measurable**; financial pass remains false. No RFC economic gate is promoted merely because the program compiles or the price requests succeeded.

To complete this particular frozen test, obtain authoritative dated trading-session/data-availability evidence for the missing 2024 crypto intervals. If it proves the frozen 24/7 assumption wrong, record a separately versioned calendar/candidate amendment before another trial; do not rewrite this failed preflight receipt. If the data should exist, establish an authorised non-duplicative recovery plan with explicit provenance. A reduced 30-episode test or replacement week is a new scope decision, not a completed 40-episode run.

Production follow-through is different: the deployed early warm-up and explicit errors expose whether Monday's actual required bars exist. Observe real arm, retest, dispatch, fill, stop amendment and exit receipts; reconcile the first completed new-policy position and the latest-20 window per account. No assumption of a continuous chat agent or invented profitability is needed.

## §9. Inspectable artifacts

- [Combined audit JSON](../docs/monday_live_audit_and_gate10_stage1_18380.json)
- [Frozen Stage 1 plan](../docs/gate9_stage1_plan_18380.json)
- [Full Stage 1 results and receipt hashes](../docs/gate9_stage1_results_18380.json)
- [Exact missing-slot audit](../docs/gate9_stage1_gap_audit_18380.json)
- [New production registration](../docs/preregistration-2026-11-08-strategy3-macro-pullback-18380.md)
- [Previous production report](PRODUCTION_UPGRADE_S3_MACRO_PULLBACK_V1-18377-2026-11-08.md)

Raw market files remain in the bounded local evidence cache, not duplicated into Git. The committed JSON retains identities, missing timestamps and hashes for review. No ZIP or new paid model route was added.

## §10. Verified activation after merge

№18,380 §§14–18 · 08-11-2026 21:31 SGT · by <Codex (GPT-6), effort: not exposed>.

Railway trading deployment `306a5080-1f2f-401b-923b-2b545499e8c7` reports SUCCESS at the production merge SHA. The previous deployment is REMOVED. Public homepage returns HTTP 200. Registration and exact production policy match on all four accounts. No verification order or account unpause mutation was required.

| Account | Owner pause | Live reserve bp/side | Core subscriptions | Prepared contexts | Exit-check progress |
| --- | --- | --- | --- | --- | --- |
| 43002148 | False | 3.74 | 8/8 | 8/8 | 11 → 28 |
| 46130058 | False | 3.74 | 8/8 | 8/8 | 11 → 28 |
| 47790949 | False | 3.74 | 8/8 | 8/8 | 11 → 28 |
| 49236051 | False | 3.74 | 8/8 | 8/8 | 11 → 28 |

Snapshots at **21:29:40 and 21:30:24 SGT** show advancing quote and watchdog times, exit-check counts rising from 11 to 28 on every account, and healthy connected account loops. All eight broker IDs are subscribed, including US500 10013. The live account retains one broker-minimum position at a time.

For each account, Monday's six non-crypto preparations are `prepared_on_time`; current BTC/ETH are `prepared_late` after the authorised restart, with complete band/macro inputs. These crypto labels describe the current session; future history cannot be final before the boundary. Complete policy context means the prepared sigma table, causal macro history, metadata, spread and commission inputs passed their existing gates. It is not a claim of authoritative historical holidays, guaranteed future quotes, sufficient margin or profitable signals.

Local Rust workspace: 743 passed, one existing ignored. The final narrow preparation retry refinement passed its five focused cases, then all required GitHub checks passed on `42962eae6c3a78056fee0b53262e054c07241b66`. The combined JSON retains both deployment snapshots and public-health source hashes. This documentation follow-up changes no trading executable or research rule.
