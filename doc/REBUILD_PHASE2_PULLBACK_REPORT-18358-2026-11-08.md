# Phase 2 — Unified offline runner and pullback evaluation

№18,358 §§1–9 · 08-11-2026 08:10:44 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed> · Repository: `ang-kl/jiaoyi`

## §1. TL;DR and measured goal progress

**The infrastructure milestone works. Pullback is a promising development observation, not a qualified edge.** All **159** old baseline episodes reproduce exactly. The new comparison admits **35 unique sessions** out of **263 core-symbol definitions**, with the protected receipts excluded.

At the **24 bp per-side stress assumption**, the four-hour forced-close comparison is:

- **A — breakout:** 3 entries, 0/3 positive closes, mean **−1.064041R**.
- **B — pullback:** 2 entries, 2/2 positive closes, mean **+0.653291R**.
- **C — macro-filtered pullback:** the same 2 entries and outcomes as B. **No incremental filter benefit is demonstrated.**

Strict tick evidence is smaller: B/C have **one target win and one unresolved trade**. The unresolved SpotCrude trade is positive at forced window end; a subsequent **conditional M30 approximation** reaches its target. These are alternative resolutions of the same trade, never extra independent wins. PF is **undefined/no losses**, not an invented large finite value, for the two-win pullback sample.

**Goal progress:** 0 new independent trade observations; 0 account/strategy qualifications. The 73% / reference 1:1.38 goal over the latest 20 trades and interim 50% research target remain unqualified. A 2/2 point estimate on reused tariff-week data is too small to establish either target; metals and crypto have no qualifying trade evidence here. Successful CI establishes software checks, not market edge.

Read the [frozen plan](../docs/rebuild_phase2_runner_and_pullback_plan_18358.json), [full measured results](../docs/rebuild_phase2_runner_and_pullback_results_18358.json), and [verification](../docs/rebuild_phase2_verification_18358.json). No broker requests, live trades, production rule changes or paid model calls occur in this phase.

## §2. Frozen interpretation and economic hypothesis

The proposed mechanism is that waiting for a retracement can improve an entry after a short-term impulse, while preceding multi-day direction can restrict countertrend exposure. This is a hypothesis, not evidence of institutional flow. The “macro” arm uses only **technical price-trend context**; it contains no fundamental, earnings, news, positioning or macroeconomic release data.

The baseline remains the actual native S3v3 **half-hour mark** rule, with causal 14-session sigma, full 24 bp cost veto and 48 bp risk floor at decision and dispatch. The source code does not know the future peak. The impulse is the first admitted baseline event and its recorded dispatch midpoint. The fixed 50% retracement is set from that midpoint and the fresh session-opening midpoint; later highs/lows do not move it. A strictly later executable ask/bid must touch the rounded level. A touch that has already crossed the origin is rejected. Limit queue/priority is not observable, so the touch fill is a research approximation.

For B/C, structural risk is `ceil_grid(abs(entry − open_mid) + observed spread + 2×entry×24/10000)`. The stop lies beyond the origin; the original R then controls the +1R entry-price stop amendment and outward-rounded 1.38R gross target. **This changes both entry timing and risk/target geometry.** The result cannot isolate a pure entry-timing effect. The cost-dependent geometry follows the owner's requested definition; the sensitivity columns hold that geometry and all fills fixed.

Five- and 20-day drift use the last 5/20 **complete coded trading sessions**, drawn from the 45-calendar-day preparation, with every bar completed before today's open. Both own drifts must strictly agree with the impulse direction. If an eligible same-account peer episode exists that date, complete peer context is required and neither peer drift may oppose it. An absent same-date peer is explicitly allowed by the frozen conditional rule; missing context for a present peer is a veto.

The new observation window is **native session open to open + 4 hours**. Thus oil now uses 07:00–11:00UTC. The legacy parity run preserves its old 07:00–17:00 source path and 13:00–17:00 reporting window. The 35-session new comparison is deliberately separate from the 159-episode legacy performance table.

## §3. Coverage, exclusions and protected data

| Symbol | Eligible definitions | Qualified sessions | Missing 45d prep | Missing opening M30 | Missing fresh open quote | 24bp forced A: wins/closes | 24bp forced B: wins/closes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 62 | 8 | 52 | 0 | 2 | 0/1 | 1/1 |
| SpotBrent | 62 | 8 | 52 | 0 | 2 | 0/0 | 0/0 |
| US500 | 51 | 9 | 42 | 4 | 0 | 0/2 | 1/1 |
| NAS100 | 7 | 0 | 7 | 2 | 0 | 0/0 | 0/0 |
| XAUUSD | 42 | 10 | 32 | 0 | 0 | 0/0 | 0/0 |
| XAGUSD | 7 | 0 | 7 | 2 | 0 | 0/0 | 0/0 |
| BTCUSD | 30 | 0 | 30 | 2 | 0 | 0/0 | 0/0 |
| ETHUSD | 2 | 0 | 2 | 2 | 0 | 0/0 | 0/0 |

The 379 eligible manifest definitions include 263 core-symbol definitions and 116 other symbols outside this experiment. Of the 263, 224 lack complete 45-day coded-session preparation; some also lack the opening M30 bar. Of the 39 with complete preparation, four oil episodes on 7–8 November 2024 lack a fresh opening quote within 5 seconds, leaving 35. Exclusion reasons can overlap and must not be summed as independent counts. All 35 are in the original 159 source subset, so **the full qualified-core and prior159-qualified summaries are identical**. The other 124 prior episodes still passed legacy parity; they did not silently become successful new-arm evaluations.

All six quarantined receipt hashes are rejected, including the duplicate alias. No protected prices or outcomes are evaluated. The earlier discovery of protected preparation means historical untouched holdout status remains **not established**. These reused high-volatility cohorts are development evidence, not independent validation.

Complete coded-session slots are a bounded research coverage check. Historical broker calendars, holidays, quote-sequence completeness, queue priority and contract terms remain unverified. Long quote intervals and same-millisecond ordering issues remain in each native quality record. No omitted price path is interpolated into a purported actual fill.

## §4. Complete cost and resolution matrix

“Net R” below is **net of the stated partial cost scenario**. Recorded bid/ask fills include spread once. Additional adverse slippage is `(entry+exit)×bps/10000/R`. Commission, swap, conversion and other unrecorded round-trip components remain unknown; zero extra charges in the arithmetic are an omission, not a verified zero fee.

Strict mode counts actual simulated bracket exits before the four-hour boundary; surviving positions remain unresolved. Forced mode separately uses a fresh closing quote within 5 seconds, with existing brackets first. Continuation preserves the strict state and processes only consecutive cached M30 bars up to 72 elapsed hours from entry. The first gap, protected interval, uncertain OHLC ordering or partial deadline remains censored. Short ask OHLC uses the explicitly assumed frozen entry-spread proxy, and financing is not included.

| Arm | Resolution | bp/side | Entries | Priced closes | Wins | Losses | Unresolved | Net win rate | Mean net R | PF / status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| arm_a_breakout_baseline | strict | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| arm_a_breakout_baseline | strict | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| arm_a_breakout_baseline | strict | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| arm_a_breakout_baseline | forced | 24 | 3 | 3 | 0 | 3 | 0 | 0.00% | -1.06404 | 0 |
| arm_a_breakout_baseline | forced | 2 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.490641 | 0 |
| arm_a_breakout_baseline | forced | 3.74 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.535992 | 0 |
| arm_a_breakout_baseline | continuation | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| arm_a_breakout_baseline | continuation | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| arm_a_breakout_baseline | continuation | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| arm_b_pure_pullback_retest | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| arm_b_pure_pullback_retest | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| arm_b_pure_pullback_retest | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| arm_b_pure_pullback_retest | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| arm_b_pure_pullback_retest | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| arm_b_pure_pullback_retest | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| arm_b_pure_pullback_retest | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| arm_b_pure_pullback_retest | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| arm_b_pure_pullback_retest | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |
| arm_c_macro_filtered_pullback | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| arm_c_macro_filtered_pullback | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| arm_c_macro_filtered_pullback | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| arm_c_macro_filtered_pullback | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| arm_c_macro_filtered_pullback | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| arm_c_macro_filtered_pullback | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| arm_c_macro_filtered_pullback | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| arm_c_macro_filtered_pullback | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| arm_c_macro_filtered_pullback | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |

Both lower-cost columns **reprice the same entries**. They do not reopen the cost gate or search for additional favourable trades. Mean R is per priced closed trade and does not turn no-entry or censored episodes into zero-profit observations.

## §5. Individual trades and what changed

| Symbol | Date | Arm | Side | Entry UTC | Entry price | Original R price | TP distance | Strict status | Strict net R24 | Forced net R24 | Forced exit | Continuation status | Continuation net R24 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | 0.504 | closed_tick | -0.793407 | -0.793407 | stop | closed_tick | -0.793407 |
| SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | 0.797 | censored_tick | Unavailable | 0.31154 | session_end | closed_m30_approximation | 0.879624 |
| SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | 0.797 | censored_tick | Unavailable | 0.31154 | session_end | closed_m30_approximation | 0.879624 |
| US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | 50.3 | closed_tick | -1.68214 | -1.68214 | stop | closed_tick | -1.68214 |
| US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | 88.5 | closed_tick | 0.995041 | 0.995041 | broker_take_profit | closed_tick | 0.995041 |
| US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | 88.5 | closed_tick | 0.995041 | 0.995041 | broker_take_profit | closed_tick | 0.995041 |
| US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | 88 | censored_tick | Unavailable | -0.716575 | session_end | censored_cache_gap | Unavailable |

**US500, 8 April 2025:** the breakout short stopped before the later retracement. The pullback short entered after that retracement and reached its target inside the observed tick window. Its risk and target distances were also different, so the improvement is attributable to the combined frozen redesign, not proven entry timing alone.

**SpotCrude, 11 April 2025:** the breakout short moved its stop to entry and closed at gross breakeven, which is a loss after the stress reserve. The later pullback short remained open at 11:00 UTC. Its forced-close result was +0.311540R; the conditional M30 target resolution was +0.879624R. That M30 result is not an additional tick-verified win.

**US500, 11 April 2025:** the baseline long lost at forced close. Price never reached the fixed retracement before the deadline, so B took no trade. C also rejected the conflicting 5d/20d direction. Since B already abstained, this does not establish incremental value from C.

| Symbol | Date | Own5d | Own20d | Peer5d | Peer20d | Macro decision | Pullback |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 2025-04-11 | -8.39% | -10.13% | -8.00% | -9.69% | own_and_peer_aligned | entered |
| US500 | 2025-04-08 | -9.61% | -9.73% | Absent | Absent | own_aligned_peer_absent | entered |
| US500 | 2025-04-11 | 0.22% | -5.49% | Absent | Absent | own_5d20d_disagree_or_missing | no_retest |

The two pullback trades occur in the same April 2025 tariff-volatility cohort, on different dates/assets. They are not representative random samples or evidence of stable performance across regimes. No metal or crypto edge was demonstrated. There is no honest basis for advertising “100% win rate achieved” or promoting this candidate to live trading.

## §6. Typed cost evidence and empirical benchmark limits

| Account | Symbol | Account-scoped IDs | Demo entry n | Median signed entry bp | Position-money records | Round-trip status |
| --- | --- | --- | --- | --- | --- | --- |
| 46130058 | BTCUSD |  | 1 | -0.66232 | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 46130058 | SpotCrude |  | 1 | -0 | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 47790949 | BTCUSD |  | 1 | 0.0180406 | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 47790949 | SpotBrent |  | 1 | -1.37757 | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 47790949 | SpotCrude | 10054 | 1 | 2.47448 | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | BTCUSD | 10028 | 0 | Unavailable | 2 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | ETHUSD | 10029 | 1 | 4.43598 | 3 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | NAS100 | 10014 | 0 | Unavailable | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | SpotBrent | 10053 | 1 | 0.914905 | 1 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | SpotCrude | 10054 | 1 | -0.629987 | 1 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | US500 | 10013 | 0 | Unavailable | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | XAGUSD | 42 | 0 | Unavailable | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |
| 49236051 | XAUUSD | 41 | 0 | Unavailable | 0 | unverified_requires_broker_spec_and_matched_dispatch_receipts |

The complete inventory retains 13 demo entry observations. Joins require **account plus symbol**; numeric symbol IDs are not global. Current account-specific contracts and retained position-money records remain attached without inventing a round-trip cost model. Statement account totals are never attributed wholesale to a symbol. Unknown benchmarks remain null.

For research account 49236051, SpotCrude has one retained entry benchmark of approximately −0.630 bp; the US500 benchmark is absent. These observations were captured later than the historical experiment. They are used **only as retrospective sensitivity benchmarks**, never causal admission or macro features. An assumed 2 or 3.74 bp exit leg does not turn the measured entry leg into a measured round trip. Other demo accounts and live accounts are not pooled.

| Arm | Resolution | Later demo entry + assumed exit | Priced closes | Closed but benchmark missing | Unresolved | Win rate | Mean R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| arm_a_breakout_baseline | strict | demo_entry_plus_2_exit | 1 | 1 | 1 | 0.00% | -0.0226454 |
| arm_a_breakout_baseline | strict | demo_entry_plus_3.74_exit | 1 | 1 | 1 | 0.00% | -0.0514064 |
| arm_a_breakout_baseline | forced | demo_entry_plus_2_exit | 1 | 2 | 0 | 0.00% | -0.0226454 |
| arm_a_breakout_baseline | forced | demo_entry_plus_3.74_exit | 1 | 2 | 0 | 0.00% | -0.0514064 |
| arm_a_breakout_baseline | continuation | demo_entry_plus_2_exit | 1 | 1 | 1 | 0.00% | -0.0226454 |
| arm_a_breakout_baseline | continuation | demo_entry_plus_3.74_exit | 1 | 1 | 1 | 0.00% | -0.0514064 |
| arm_b_pure_pullback_retest | strict | demo_entry_plus_2_exit | 0 | 1 | 1 | Unavailable | Unavailable |
| arm_b_pure_pullback_retest | strict | demo_entry_plus_3.74_exit | 0 | 1 | 1 | Unavailable | Unavailable |
| arm_b_pure_pullback_retest | forced | demo_entry_plus_2_exit | 1 | 1 | 0 | 100.00% | 0.800308 |
| arm_b_pure_pullback_retest | forced | demo_entry_plus_3.74_exit | 1 | 1 | 0 | 100.00% | 0.782145 |
| arm_b_pure_pullback_retest | continuation | demo_entry_plus_2_exit | 1 | 1 | 0 | 100.00% | 1.36715 |
| arm_b_pure_pullback_retest | continuation | demo_entry_plus_3.74_exit | 1 | 1 | 0 | 100.00% | 1.34908 |
| arm_c_macro_filtered_pullback | strict | demo_entry_plus_2_exit | 0 | 1 | 1 | Unavailable | Unavailable |
| arm_c_macro_filtered_pullback | strict | demo_entry_plus_3.74_exit | 0 | 1 | 1 | Unavailable | Unavailable |
| arm_c_macro_filtered_pullback | forced | demo_entry_plus_2_exit | 1 | 1 | 0 | 100.00% | 0.800308 |
| arm_c_macro_filtered_pullback | forced | demo_entry_plus_3.74_exit | 1 | 1 | 0 | 100.00% | 0.782145 |
| arm_c_macro_filtered_pullback | continuation | demo_entry_plus_2_exit | 1 | 1 | 0 | 100.00% | 1.36715 |
| arm_c_macro_filtered_pullback | continuation | demo_entry_plus_3.74_exit | 1 | 1 | 0 | 100.00% | 1.34908 |

## §7. Verification, acceptance gates and next decision

- Frozen plan SHA256: `94c5f2c37131908ea6d3cd76f337003e1ff197232cea3311626ac385f149e338`.
- Result SHA256: `50b14b847796c296293a01c3dff748b5a8b64888c6054ec77206afb926fd5862`.
- Exact baseline parity: 159/159 decision-mark, strict-path and forced-path comparisons.
- Verified source files: 756; quarantined receipt hashes rejected: 6.
- Independent gross/net arithmetic checks: 58.
- Focused tests: two Rust and four Python tests, plus retained existing exit tests in CI.
- Resumed run: 159 baseline and 35 experiment cache hits; output bytes match the initial run.
- Native source and binary hashes, source receipts and plan identity are recorded. No network calls are permitted by the Python runner; native adapters are local-file programs without a broker transport route.

| Gate | Status | Evidence or limitation |
| --- | --- | --- |
| G1 | Partial | Frozen plan, source/code/binary/manifest and account identity links; inherited unresolved provenance remains. |
| G2 | Passed: legacy migration parity | All 159 original24bp baseline decision marks, strict paths and forced paths reproduce exactly before new arms. |
| G3 | Passed for this bounded offline runner | One manifest command, denied network, hashed native receipts, explicit denominators and byte-identical verified resume; older runners retained as historical reproducibility tools. |
| G4 | Partial | Per-experiment45-day/native4h coverage enforced; missing slots/tails rejected or censored; continuous execution and dated broker calendar not certified. |
| G5 | Partial | Existing DST-aware code registry reused unchanged; historical broker holidays/maintenance remain unverified. |
| G6 | Partial | Typed account/symbol cost and retained position-money joins; zero complete round-trip cost samples. |
| G7 | Not satisfied | 13 retained demo entry observations are not historical/live round-trip calibration. |
| G8 | Partial | Shared existing two-stage tick/M30 mechanics reused; live broker/restart/partial-fill parity not established. |
| G9 | Not satisfied for promotion | Protected dependencies rejected; all 35 evaluated sessions reuse development data. Phase 1 historical exposure caveat retained. |
| G10 | Not satisfied | Two correlated-regime pullback trades cannot establish durable expectancy or class/account goals. |
| G11 | Deferred | No shadow or canary deployment. |
| G12 | Deferred | Live rules untouched; full rebuilt trading lifecycle and rollback not exercised. |

**Next:** keep B unchanged as a candidate awaiting replication. Do not tune the retracement or reduce costs to improve this tiny result. First register an independent evaluation scope, resolve the known historical-holdout exposure, and identify the smallest missing preparation/contract/cost evidence. If entry timing itself is the next question, freeze a separate matched-risk control before evaluation; this run changed both timing and geometry. C needs a sample where it actually changes B's entries before its incremental value can be assessed.

No further broker collection, historical-spec retrieval, live migration or capital exposure is performed here. Within the current offline cache the qualified comparison is complete. The original goals, traceability, SGT timestamps, preserving proven fixes, and “Time is Money” remain the operating invariants.

Reproduce from the exact cached sources, with a fresh work directory for a different code revision:

```text
cargo build -p jiaoyi-core --example native_target_sizing_replay --example manifest_pullback_replay --example replay_inventory_metadata --locked --offline
python3 ops/research/unified_replay.py --workdir /tmp/jiaoyi-phase2-replay --output /tmp/jiaoyi-phase2-results.json
python3 -m unittest discover -s ops/research -p test_unified_replay.py
cargo test -p jiaoyi-core research::manifest_pullback --locked --offline
```

Running the same command again verifies the cached native receipts and returns deterministic results. Missing or changed files fail closed. A Git clone contains the report and result evidence; replay also requires the Phase 1 manifest's local source cache. Original historical scripts remain for old frozen experiments; the new comparison uses the canonical manifest command.

## §8. Complete evaluated-session register

| Symbol | Date | Episode | Prior159 | Protocol | Prepared slots | Peer | Admission | B result | C result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SpotBrent | 2024-11-04 | cdb6706f8187637f3fe3352d | True | clean_first_read | 806 | a451c5daee55ffdac04ccd9a | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2024-11-05 | 3609573a2a98577036cc248d | True | clean_first_read | 806 | e060f88cb7dc74c96c07b68b | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2024-11-06 | 7e1f8994fecf745690603eeb | True | clean_first_read | 832 | 910dc7f401c979f077122690 | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2025-04-07 | a92fb0ae4794ae5899abbb1d | True | clean_first_read | 806 | 77b10f0f75d5383bb011c4b2 | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2025-04-08 | f5233c2f0a824dfd2f998e89 | True | clean_first_read | 806 | 2b977b08318352c8c79f2cbc | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2025-04-09 | c3a0d383d87130df509791fa | True | clean_first_read | 832 | b8444e851d499207d0449237 | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2025-04-10 | 1f6ef75ca564ffa87d523f97 | True | clean_first_read | 858 | 9e626ae09c741e2673fd8c78 | no_admitted_impulse | no_impulse | no_impulse |
| SpotBrent | 2025-04-11 | 70e3acf0ef22a39ed2354106 | True | clean_first_read | 858 | ce5a75c11b08e52a205e5350 | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2024-11-04 | a451c5daee55ffdac04ccd9a | True | clean_first_read | 806 | cdb6706f8187637f3fe3352d | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2024-11-05 | e060f88cb7dc74c96c07b68b | True | clean_first_read | 806 | 3609573a2a98577036cc248d | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2024-11-06 | 910dc7f401c979f077122690 | True | clean_first_read | 832 | 7e1f8994fecf745690603eeb | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2025-04-07 | 77b10f0f75d5383bb011c4b2 | True | clean_first_read | 806 | a92fb0ae4794ae5899abbb1d | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2025-04-08 | 2b977b08318352c8c79f2cbc | True | clean_first_read | 806 | f5233c2f0a824dfd2f998e89 | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2025-04-09 | b8444e851d499207d0449237 | True | clean_first_read | 832 | c3a0d383d87130df509791fa | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2025-04-10 | 9e626ae09c741e2673fd8c78 | True | clean_first_read | 858 | 1f6ef75ca564ffa87d523f97 | no_admitted_impulse | no_impulse | no_impulse |
| SpotCrude | 2025-04-11 | ce5a75c11b08e52a205e5350 | True | clean_first_read | 858 | 70e3acf0ef22a39ed2354106 | impulse_admitted | admitted | admitted |
| US500 | 2024-11-05 | 4d04e030932184cbabc7f97b | True | clean_first_read | 403 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2024-11-06 | 8d7090e5c928d51baf5f5d28 | True | clean_first_read | 416 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2024-11-07 | f35c2955c68672466ebc4e06 | True | clean_first_read | 429 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2024-11-08 | b8faf0a449b13f21ebd8db35 | True | clean_first_read | 429 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2025-04-07 | 31ac96bb6c2db5e56d25c188 | True | clean_first_read | 403 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2025-04-08 | 5b35427f494f9de75bb0ddd1 | True | clean_first_read | 403 | Absent | impulse_admitted | admitted | admitted |
| US500 | 2025-04-09 | 7be5dc89274ab1082ba3e7ba | True | clean_first_read | 416 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2025-04-10 | 5ff6b8a3b787709ae2dcc9a0 | True | clean_first_read | 429 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| US500 | 2025-04-11 | acdc1e0257e10eefaf512a7f | True | clean_first_read | 429 | Absent | impulse_admitted | no_retest | own_5d20d_disagree_or_missing |
| XAUUSD | 2024-11-04 | caca0aae14aacd9f05311212 | True | clean_first_read | 806 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2024-11-05 | b92b7b077dd282ac80b2c042 | True | clean_first_read | 806 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2024-11-06 | 877ac3424c0551b6f7772d6a | True | clean_first_read | 832 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2024-11-07 | 569183452c60779cd669e2eb | True | clean_first_read | 858 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2024-11-08 | f741e94dbeb081106be380a1 | True | clean_first_read | 858 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2025-04-07 | ae86f219cd744d0fe05a351e | True | clean_first_read | 806 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2025-04-08 | 44e4ecb72fbb53cc14fc14da | True | clean_first_read | 806 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2025-04-09 | 3da7750ba2a191ca753fb2e9 | True | clean_first_read | 832 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2025-04-10 | 46371ae1ac4c72c2b4aa725b | True | clean_first_read | 858 | Absent | no_admitted_impulse | no_impulse | no_impulse |
| XAUUSD | 2025-04-11 | d9c1023f5de634fed865d7b3 | True | clean_first_read | 858 | Absent | no_admitted_impulse | no_impulse | no_impulse |

## §9. Complete excluded-definition register

| Symbol | Date | Episode | Reason | Missing prep slots |
| --- | --- | --- | --- | --- |
| ADAUSD | 2025-07-07 | d1a69a2dc803a87c5097b45c | outside_core_scope | Unavailable |
| ADAUSD | 2025-07-14 | 082a6549c9d765be3ea185f0 | outside_core_scope | Unavailable |
| AUDJPY | 2025-07-07 | 89062e8ea535c6e11e35605c | outside_core_scope | Unavailable |
| AUDJPY | 2025-07-14 | ff659acf6369cf424d3be3f0 | outside_core_scope | Unavailable |
| AUDUSD | 2025-07-07 | 3b5f4cd3aec1c1aeec26d1c1 | outside_core_scope | Unavailable |
| AUDUSD | 2025-07-14 | 5b27681e214647e03fd2d057 | outside_core_scope | Unavailable |
| AUS200 | 2025-07-07 | a99521db57837133c65c7bf8 | outside_core_scope | Unavailable |
| AUS200 | 2025-07-14 | 17e9233fca4846dcd3b20771 | outside_core_scope | Unavailable |
| AVAXUSD | 2025-07-07 | 6d3f76cfb8e5d6be83456b15 | outside_core_scope | Unavailable |
| AVAXUSD | 2025-07-14 | 3434d4494f742203c4176189 | outside_core_scope | Unavailable |
| BCHUSD | 2025-07-07 | 6a8ef9c5180a2a3a33e355cf | outside_core_scope | Unavailable |
| BCHUSD | 2025-07-14 | ca49f91ad05c674370c5e7d4 | outside_core_scope | Unavailable |
| BTCUSD | 2025-07-07 | a3281b32cf61af3a41b4fb3a | missing_45day_session_preparation, missing_opening_M30 | 2160 |
| BTCUSD | 2025-07-14 | ae497d730dbeacf465872233 | missing_45day_session_preparation, missing_opening_M30 | 2160 |
| CADJPY | 2025-07-07 | 0dc38d8a164908535db54ed8 | outside_core_scope | Unavailable |
| CADJPY | 2025-07-14 | 1deb2d2370a35ffba08289f7 | outside_core_scope | Unavailable |
| CHFJPY | 2025-07-07 | 9ce9f711a042156c2370ac80 | outside_core_scope | Unavailable |
| CHFJPY | 2025-07-14 | 09a99eb09e31bdaedd4c4191 | outside_core_scope | Unavailable |
| DOGEUSD | 2025-07-07 | 1f88f533575396b87ea4a2da | outside_core_scope | Unavailable |
| DOGEUSD | 2025-07-14 | c91a743316e90353453124cb | outside_core_scope | Unavailable |
| DOTUSD | 2025-07-07 | bf629eae41fd47b705d8277b | outside_core_scope | Unavailable |
| DOTUSD | 2025-07-14 | 89c4613d1d72efa008bc0499 | outside_core_scope | Unavailable |
| ETHUSD | 2025-07-07 | 14cfdcad3095953593d732bf | missing_45day_session_preparation, missing_opening_M30 | 2160 |
| ETHUSD | 2025-07-14 | 743d69735dec5f4d37cf225b | missing_45day_session_preparation, missing_opening_M30 | 2160 |
| EURAUD | 2025-07-07 | e1f06101d6a34ed9996e91d9 | outside_core_scope | Unavailable |
| EURAUD | 2025-07-14 | 0e26317ceb22c2b314ebff9f | outside_core_scope | Unavailable |
| EURGBP | 2025-07-07 | bea38c79d0e834eb4656154e | outside_core_scope | Unavailable |
| EURGBP | 2025-07-14 | f24de31428e722feeeecf67c | outside_core_scope | Unavailable |
| EURJPY | 2025-07-07 | 0cd2fa1723ea86507b689380 | outside_core_scope | Unavailable |
| EURJPY | 2025-07-14 | 9139404eb2d76b61ad71f5fc | outside_core_scope | Unavailable |
| EURUSD | 2025-07-07 | c4b169c2bf658cafad387409 | outside_core_scope | Unavailable |
| EURUSD | 2025-07-14 | 65a0e0cfc68c2fbc8e10430e | outside_core_scope | Unavailable |
| GBPJPY | 2025-07-07 | b808e8015d85275495a2e27e | outside_core_scope | Unavailable |
| GBPJPY | 2025-07-14 | be6b0c0157eb9ae776ad779a | outside_core_scope | Unavailable |
| GBPUSD | 2025-07-07 | 700f4652d66d5583480961af | outside_core_scope | Unavailable |
| GBPUSD | 2025-07-14 | cc6ade232fa2bfdfc271f767 | outside_core_scope | Unavailable |
| GER40 | 2025-07-07 | 3d77d6cd049d93cde8e6be62 | outside_core_scope | Unavailable |
| GER40 | 2025-07-14 | 15f3db9f48a3f3e9a904f7a7 | outside_core_scope | Unavailable |
| JPN225 | 2025-07-07 | 7ecde84ce8cbe6c1353333b4 | outside_core_scope | Unavailable |
| JPN225 | 2025-07-14 | 61ed57fd3eccc7f0a9b57574 | outside_core_scope | Unavailable |
| LINKUSD | 2025-07-07 | 085502ec03defd273479daa9 | outside_core_scope | Unavailable |
| LINKUSD | 2025-07-14 | 38d8626b7d50661152403e83 | outside_core_scope | Unavailable |
| LTCUSD | 2025-07-07 | 3aa3c12411e861249073242b | outside_core_scope | Unavailable |
| LTCUSD | 2025-07-14 | 65bd630ef242a0cf32c4cf98 | outside_core_scope | Unavailable |
| NAS100 | 2025-07-07 | 71c12956d026bd69a24d715f | missing_45day_session_preparation, missing_opening_M30 | 403 |
| NAS100 | 2025-07-14 | 00b72c8130b8d25638b1b9b8 | missing_45day_session_preparation, missing_opening_M30 | 403 |
| NZDUSD | 2025-07-07 | 0e2c4ca5acaccd715d6b2558 | outside_core_scope | Unavailable |
| NZDUSD | 2025-07-14 | 01778b487df4e97b892ad3f5 | outside_core_scope | Unavailable |
| NatGas | 2025-07-07 | 5f9235d027c1422bf1b936e5 | outside_core_scope | Unavailable |
| NatGas | 2025-07-14 | 2cd0e59019c7ee2b2e7efd59 | outside_core_scope | Unavailable |
| SOLUSD | 2025-07-07 | 39f0f8e2e1fd602374ca71dd | outside_core_scope | Unavailable |
| SOLUSD | 2025-07-14 | 12b930c576e5ff65ee3ed7f9 | outside_core_scope | Unavailable |
| SpotBrent | 2025-07-07 | 835d49e718917e037ce90b18 | missing_45day_session_preparation | 270 |
| SpotBrent | 2025-07-14 | 48294fd137584a2725661351 | missing_45day_session_preparation | 140 |
| SpotCrude | 2025-07-07 | a3f3c4991fe881e8160596d7 | missing_45day_session_preparation | 269 |
| SpotCrude | 2025-07-14 | 8970de420891b9d784ed058b | missing_45day_session_preparation | 139 |
| UK100 | 2025-07-07 | ab45e47ee50f64b8ee456c7a | outside_core_scope | Unavailable |
| UK100 | 2025-07-14 | d454eff7237c800488e74140 | outside_core_scope | Unavailable |
| US30 | 2025-07-07 | 76eb3c77f3dabbcf4dba6276 | outside_core_scope | Unavailable |
| US30 | 2025-07-14 | 87550782f1b702dfe5d60f74 | outside_core_scope | Unavailable |
| US500 | 2025-07-07 | 136bcb5c0c607b3895112788 | missing_45day_session_preparation | 147 |
| US500 | 2025-07-14 | 068cb592754e6491f5762257 | missing_45day_session_preparation | 82 |
| USDCAD | 2025-07-07 | b7bae7a84725303d1db2a890 | outside_core_scope | Unavailable |
| USDCAD | 2025-07-14 | 5c911d7be10e49425b40f084 | outside_core_scope | Unavailable |
| USDCHF | 2025-07-07 | e663d35316b51ee1e2aa46cb | outside_core_scope | Unavailable |
| USDCHF | 2025-07-14 | 57429c86356f6754dfc49911 | outside_core_scope | Unavailable |
| USDJPY | 2025-07-07 | 4439405cdd634f063a0783b0 | outside_core_scope | Unavailable |
| USDJPY | 2025-07-14 | 00e43213f810417444ca96a6 | outside_core_scope | Unavailable |
| XAGUSD | 2025-07-07 | 310539d714446e4a4c51da87 | missing_45day_session_preparation, missing_opening_M30 | 806 |
| XAGUSD | 2025-07-14 | ed2e5b33c6527114311316f0 | missing_45day_session_preparation, missing_opening_M30 | 806 |
| XAUUSD | 2025-07-07 | 7106ba337a214ad20426e39f | missing_45day_session_preparation | 269 |
| XAUUSD | 2025-07-14 | 68f8dac888fe05b1d594711a | missing_45day_session_preparation | 139 |
| XLMUSD | 2025-07-07 | 26525ea248404210163c4b11 | outside_core_scope | Unavailable |
| XLMUSD | 2025-07-14 | 46b412edad6dfcfa2ff86d70 | outside_core_scope | Unavailable |
| XPDUSD | 2025-07-07 | f2e972076104fd4e9dea8c3c | outside_core_scope | Unavailable |
| XPDUSD | 2025-07-14 | ec3fd68245aaf601c4fee57e | outside_core_scope | Unavailable |
| XPTUSD | 2025-07-07 | 9203b0224ae6c0554bc038b8 | outside_core_scope | Unavailable |
| XPTUSD | 2025-07-14 | 7b2e427ebc4ffbf8e1c2543c | outside_core_scope | Unavailable |
| XRPUSD | 2025-07-07 | 1844d0a38c5f6679ac288e0a | outside_core_scope | Unavailable |
| XRPUSD | 2025-07-14 | 22f81ff70d4bf24ff650f483 | outside_core_scope | Unavailable |
| BTCUSD | 2025-09-12 | a69d98a9b0857fba4468653a | missing_45day_session_preparation | 1872 |
| BTCUSD | 2025-09-13 | c891a278d9291a11a5ed214f | missing_45day_session_preparation | 1824 |
| BTCUSD | 2025-09-14 | c065a06364ed5a3f4d4daff8 | missing_45day_session_preparation | 1776 |
| BTCUSD | 2025-09-15 | 13413d5eeba6ae5b95025343 | missing_45day_session_preparation | 1728 |
| BTCUSD | 2025-09-16 | 3167b31ba20ab20de427348f | missing_45day_session_preparation | 1680 |
| BTCUSD | 2025-09-17 | 022ba7185edee7254c4d8857 | missing_45day_session_preparation | 1632 |
| BTCUSD | 2025-09-18 | 4565a5781bcced588063121c | missing_45day_session_preparation | 1584 |
| BTCUSD | 2025-09-19 | bad969c61c67c207ed088029 | missing_45day_session_preparation | 1536 |
| NatGas | 2025-09-08 | 1dde41961c5513d2e49f48f6 | outside_core_scope | Unavailable |
| NatGas | 2025-09-09 | 6992c66eff6791f4aff090ed | outside_core_scope | Unavailable |
| NatGas | 2025-09-10 | ccf51afdb044eabeb17a21c9 | outside_core_scope | Unavailable |
| NatGas | 2025-09-11 | 2f79bd5f417eb04353c3854c | outside_core_scope | Unavailable |
| NatGas | 2025-09-12 | 9032a14bc79bf8cd0e653691 | outside_core_scope | Unavailable |
| NatGas | 2025-09-15 | 9e443883551d203d2332f422 | outside_core_scope | Unavailable |
| NatGas | 2025-09-16 | 0adad7fc27bc689a1a137e02 | outside_core_scope | Unavailable |
| NatGas | 2025-09-17 | 9aff81d1ca11c7ddef52d6f5 | outside_core_scope | Unavailable |
| NatGas | 2025-09-18 | e6a8f2151786f69b9f37a433 | outside_core_scope | Unavailable |
| NatGas | 2025-09-19 | 808049d904cbddafaa9e1126 | outside_core_scope | Unavailable |
| SpotBrent | 2024-11-07 | 2dc02ac0109a8eeb3762a22c | missing_fresh_open_quote | 0 |
| SpotBrent | 2024-11-08 | d7ce2442e8c5109477cfdd52 | missing_fresh_open_quote | 0 |
| SpotCrude | 2024-11-07 | 0f95ad7a94031afd0dea3e9d | missing_fresh_open_quote | 0 |
| SpotCrude | 2024-11-08 | be7e37171731f34c82648fa5 | missing_fresh_open_quote | 0 |
| US500 | 2024-11-04 | 0c8d1df35ca48a63714fb422 | missing_45day_session_preparation | 2 |
| NAS100 | 2024-01-22 | f3ec49598c955968c2cdec80 | missing_45day_session_preparation | 403 |
| NAS100 | 2024-01-23 | 74608bc5524e70321ed4bed1 | missing_45day_session_preparation | 395 |
| NAS100 | 2024-01-24 | 7048b77cb373429eafea7db7 | missing_45day_session_preparation | 400 |
| NAS100 | 2024-01-25 | dae3cca41d4e71510881f096 | missing_45day_session_preparation | 405 |
| NAS100 | 2024-01-26 | 2549b6349e4927934cea6b9a | missing_45day_session_preparation | 397 |
| XAGUSD | 2024-01-22 | fd538dee2dfee62bbdfef31a | missing_45day_session_preparation | 806 |
| XAGUSD | 2024-01-23 | a7db221453c42363d5851c6b | missing_45day_session_preparation | 798 |
| XAGUSD | 2024-01-24 | 7b4c0272869b56392daed4d2 | missing_45day_session_preparation | 816 |
| XAGUSD | 2024-01-25 | 4688021d87094d090147d2bf | missing_45day_session_preparation | 834 |
| XAGUSD | 2024-01-26 | 9d283ed54947635561710d02 | missing_45day_session_preparation | 826 |
| AUDUSD | 2025-09-08 | 265645a3be096326aed2aedb | outside_core_scope | Unavailable |
| AUDUSD | 2025-09-08 | 29f1fdeef321960dd072b797 | outside_core_scope | Unavailable |
| AUDUSD | 2025-09-08 | dfb552bb618ed7a79347c413 | outside_core_scope | Unavailable |
| EURUSD | 2025-09-08 | 080470d042651b8c655dad9b | outside_core_scope | Unavailable |
| EURUSD | 2025-09-08 | 5b124b6cb82e8d11d58e206e | outside_core_scope | Unavailable |
| EURUSD | 2025-09-08 | af2df2b9ecb95ec99e4ba214 | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-08 | 10ccc5a2f8457ba6c63cade0 | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-08 | 88de0a62804d4051c67da23a | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-08 | 203cb97bcdb4d6aa66c41465 | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-08 | 29d67181a212c5d322ae2605 | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-08 | ff8833c6c0408e7b986fa556 | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-08 | ee47bfaaab75edf28033ae92 | outside_core_scope | Unavailable |
| USDCAD | 2025-09-08 | b7effcb8473375ad72e30a33 | outside_core_scope | Unavailable |
| USDCAD | 2025-09-08 | 4393face0ef1ff16fe668670 | outside_core_scope | Unavailable |
| USDCAD | 2025-09-08 | 8a730a4fb8297af23d51c185 | outside_core_scope | Unavailable |
| USDCHF | 2025-09-08 | 2c47602726b787377b989764 | outside_core_scope | Unavailable |
| USDCHF | 2025-09-08 | 4dd8f646786d33e103b4eaa9 | outside_core_scope | Unavailable |
| USDCHF | 2025-09-08 | 6c2a25e90c123f22ec1074ed | outside_core_scope | Unavailable |
| USDJPY | 2025-09-08 | d944ef2b2fa31478531904b6 | outside_core_scope | Unavailable |
| USDJPY | 2025-09-08 | 77ccbe44afa519c5a61a7381 | outside_core_scope | Unavailable |
| USDJPY | 2025-09-08 | a9a4144930ba0f42db95eb6d | outside_core_scope | Unavailable |
| AUDUSD | 2025-09-18 | b82bbb14c003d4093cf77eab | outside_core_scope | Unavailable |
| AUDUSD | 2025-09-18 | fcc1d48a348abfd833bc9b55 | outside_core_scope | Unavailable |
| AUDUSD | 2025-09-18 | f9f9893d28f7f5860313eda7 | outside_core_scope | Unavailable |
| EURUSD | 2025-09-18 | 55b1ac5ae47e542c569461ce | outside_core_scope | Unavailable |
| EURUSD | 2025-09-18 | 31a1e202c0e8c69e112eab89 | outside_core_scope | Unavailable |
| EURUSD | 2025-09-18 | 61c07a5a8818486dbf3defaf | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-18 | 22ce4c1bc5a5125f0f639e86 | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-18 | 7a38284e1e023c49abb24d83 | outside_core_scope | Unavailable |
| GBPUSD | 2025-09-18 | b09f25a7c4841418d340291f | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-18 | 7fab6f81eb13a87cbd74c74c | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-18 | 78d0423f69c649afdc4c2cff | outside_core_scope | Unavailable |
| NZDUSD | 2025-09-18 | 4077f769bc94792d8903123a | outside_core_scope | Unavailable |
| USDCAD | 2025-09-18 | 331ab23fa5ab236d5c9725c8 | outside_core_scope | Unavailable |
| USDCAD | 2025-09-18 | ea6052d2a879cb4641909cae | outside_core_scope | Unavailable |
| USDCAD | 2025-09-18 | 1a9e734a30e7a884b29566db | outside_core_scope | Unavailable |
| USDCHF | 2025-09-18 | 107a61488594299274e2e70f | outside_core_scope | Unavailable |
| USDCHF | 2025-09-18 | 5955503cea2dcfc86e473dba | outside_core_scope | Unavailable |
| USDCHF | 2025-09-18 | 726d63629c7dbe1067c5949d | outside_core_scope | Unavailable |
| USDJPY | 2025-09-18 | 2706d186ee9c9c2bf2cb97f8 | outside_core_scope | Unavailable |
| USDJPY | 2025-09-18 | b069a6d65fff6680a2fc4ca6 | outside_core_scope | Unavailable |
| USDJPY | 2025-09-18 | 428da4156a95703bac2255f6 | outside_core_scope | Unavailable |
| BTCUSD | 2024-01-22 | 6381f9217d6a5d1c2c3794b4 | missing_45day_session_preparation | 555 |
| BTCUSD | 2024-01-23 | 4c63892002b69ae4559435bd | missing_45day_session_preparation | 507 |
| BTCUSD | 2024-01-24 | 46b1da2b93e2935f42ee3164 | missing_45day_session_preparation | 459 |
| BTCUSD | 2024-01-25 | bf44e6b3bcb58bb2906fe359 | missing_45day_session_preparation | 411 |
| BTCUSD | 2024-01-26 | 83bd140279a8e96c93fb6105 | missing_45day_session_preparation | 363 |
| BTCUSD | 2024-07-22 | 8199d9cbcec7df48ffc053bd | missing_45day_session_preparation | 84 |
| BTCUSD | 2024-07-23 | 46dbae667d09a4839bc21925 | missing_45day_session_preparation | 82 |
| BTCUSD | 2024-07-24 | 17a309f47a71bca0f571e4d8 | missing_45day_session_preparation | 60 |
| BTCUSD | 2024-07-25 | 4a422a139782982730a96c44 | missing_45day_session_preparation | 56 |
| BTCUSD | 2024-07-26 | 8550e593edbfb863371e3be3 | missing_45day_session_preparation | 56 |
| BTCUSD | 2025-05-12 | e2f8731ee1bbbd8ebec301bb | missing_45day_session_preparation | 41 |
| BTCUSD | 2025-05-13 | 4350fd4ccc6e7fa2e964e43c | missing_45day_session_preparation | 39 |
| BTCUSD | 2025-05-14 | ea65ff19fabbc0109ea7c967 | missing_45day_session_preparation | 39 |
| BTCUSD | 2025-05-15 | 9ce2dd036808138bac5852eb | missing_45day_session_preparation | 35 |
| BTCUSD | 2025-05-16 | 8a9060211292afa3d94ff18b | missing_45day_session_preparation | 35 |
| BTCUSD | 2025-10-06 | 39f5596f1015e6d5706ba25b | missing_45day_session_preparation | 720 |
| BTCUSD | 2025-10-07 | 5faad815c6cd7484d9d45885 | missing_45day_session_preparation | 672 |
| BTCUSD | 2025-10-08 | daaa576e731c68edb339d1a8 | missing_45day_session_preparation | 624 |
| BTCUSD | 2025-10-09 | 7e2b39cab272c70c3468188b | missing_45day_session_preparation | 576 |
| BTCUSD | 2025-10-10 | aae2661c0700eeb400008256 | missing_45day_session_preparation | 528 |
| SpotBrent | 2024-01-22 | a51722a99e186dedf667cb58 | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-23 | 2d22569eb65ac698f7c74c93 | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-24 | 413acab1546b60736c1ea52d | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-25 | f2ddfe7b0570504c76b992ab | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-26 | 550647596835f77c4cbbcfda | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-07-22 | 2f64878c52b286c335f65e4a | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-23 | 6ebdfc6f362294e0e607bf79 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-24 | 1a897af29e15f58be0c69494 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-25 | b6074168c8eb0dc0e2f8ae9a | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-26 | c6668faccec1c79439413775 | missing_45day_session_preparation | 12 |
| SpotBrent | 2025-05-12 | 3ddc5b55e115b2e2e1936c1c | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-05-13 | e784ecf99abd10f7c3a1ae30 | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-05-14 | 8b39ab6c712fb7c7397fdfdb | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-05-15 | affe109b180bfeb08d170ec7 | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-05-16 | e77a82873dfae309f20bfbc8 | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-10-06 | 0d8adf49267297042d98d06a | missing_45day_session_preparation | 286 |
| SpotBrent | 2025-10-07 | 362375ec6c45fb8d7a2b7d3f | missing_45day_session_preparation | 260 |
| SpotBrent | 2025-10-08 | a859f91f6cb95922ddcbacb1 | missing_45day_session_preparation | 260 |
| SpotBrent | 2025-10-09 | c905518f0136641e39bad9c5 | missing_45day_session_preparation | 260 |
| SpotBrent | 2025-10-10 | 64641fa516e0860f2ba2f3ec | missing_45day_session_preparation | 234 |
| SpotCrude | 2024-01-22 | ab6aca1489429fe387e656d0 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-23 | 8507537493997a182445c464 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-24 | 4a7617b76a56942a450d6768 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-25 | d1bc6b75ea34848580a14441 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-26 | 83dde1f2c6ae2b80589433bf | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-07-22 | 20abc09e536b839a6e7fe9c8 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-23 | e48e81f1c2a54ffeea4d2c00 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-24 | 86d05af4f69296fddf2b69d9 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-25 | 13d1b94f806804837755c3fb | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-26 | 16b8f2818209bcec0d6c2761 | missing_45day_session_preparation | 8 |
| SpotCrude | 2025-05-12 | b29542347402bab6234839d6 | missing_45day_session_preparation | 26 |
| SpotCrude | 2025-05-13 | d0e4ba21f85ecdbfbfe40a8b | missing_45day_session_preparation | 26 |
| SpotCrude | 2025-05-14 | 9c22155b1c7571aa5a99b515 | missing_45day_session_preparation | 26 |
| SpotCrude | 2025-05-15 | 8d44e2c3e3ecabd890265eb0 | missing_45day_session_preparation | 26 |
| SpotCrude | 2025-05-16 | 27771eec2fd461a5bb8b89fc | missing_45day_session_preparation | 26 |
| SpotCrude | 2025-10-06 | 134bb1b1c5cae3963b7a4c6c | missing_45day_session_preparation | 286 |
| SpotCrude | 2025-10-07 | 5aca717f89d879a30446d2ed | missing_45day_session_preparation | 260 |
| SpotCrude | 2025-10-08 | 7c936f35c2bb48764b26236a | missing_45day_session_preparation | 260 |
| SpotCrude | 2025-10-09 | 5294e3691e86e4143000b304 | missing_45day_session_preparation | 260 |
| SpotCrude | 2025-10-10 | 0d43b0a72fed3835f3c8e4ba | missing_45day_session_preparation | 234 |
| US500 | 2024-01-22 | 4fe286efdb366d24671e8115 | missing_45day_session_preparation, missing_opening_M30 | 403 |
| US500 | 2024-01-23 | 845b548701e7627dc3f53be3 | missing_45day_session_preparation, missing_opening_M30 | 403 |
| US500 | 2024-01-25 | c544f73b7b6de1593d3c133a | missing_45day_session_preparation, missing_opening_M30 | 429 |
| US500 | 2024-01-26 | e56cc14a79f4db75fde05223 | missing_45day_session_preparation, missing_opening_M30 | 429 |
| US500 | 2024-07-22 | 6681fcc669deff45d1f60f88 | missing_45day_session_preparation | 17 |
| US500 | 2024-07-23 | 04ea9a8aed2ded217967b9a0 | missing_45day_session_preparation | 17 |
| US500 | 2024-07-24 | 001f989aba50039f6db56506 | missing_45day_session_preparation | 17 |
| US500 | 2024-07-25 | ff90f42b6db284ffa5fee37a | missing_45day_session_preparation | 17 |
| US500 | 2024-07-26 | 9856b173f9625f9053df2d6a | missing_45day_session_preparation | 17 |
| US500 | 2025-05-12 | 4d12cf15cd8e08baa46e7d29 | missing_45day_session_preparation | 13 |
| US500 | 2025-05-13 | cf4371537a3ed53252478781 | missing_45day_session_preparation | 13 |
| US500 | 2025-05-14 | c1bf02ca4edb813e89f28dd7 | missing_45day_session_preparation | 13 |
| US500 | 2025-05-15 | 7781d2ba777facd710ca1457 | missing_45day_session_preparation | 13 |
| US500 | 2025-05-16 | 43eeb5e76c9e47a95bccd800 | missing_45day_session_preparation | 13 |
| US500 | 2025-10-06 | 9546b2e4fa6800370e6b787f | missing_45day_session_preparation | 143 |
| US500 | 2025-10-07 | da807a1142c70b322b781e37 | missing_45day_session_preparation | 130 |
| US500 | 2025-10-08 | 67e0c555d803a502cc15d6dd | missing_45day_session_preparation | 130 |
| US500 | 2025-10-09 | 25e98df1d7dd285726528311 | missing_45day_session_preparation | 130 |
| US500 | 2025-10-10 | c47e497770890fb64a6335ad | missing_45day_session_preparation | 117 |
| SpotBrent | 2023-01-23 | 7d29c206fcaad3e409105de3 | missing_45day_session_preparation | 78 |
| SpotBrent | 2023-01-24 | 6e18297f6eeed480dc9234ba | missing_45day_session_preparation | 78 |
| SpotBrent | 2023-01-25 | d85383ce4fc5cc9e6b2a3788 | missing_45day_session_preparation | 78 |
| SpotBrent | 2023-01-26 | 9d1d88c1aeae55f5d7850090 | missing_45day_session_preparation | 78 |
| SpotBrent | 2023-01-27 | a6bc4b04896b9807e1b6e9c6 | missing_45day_session_preparation | 78 |
| SpotBrent | 2023-07-24 | d2dea2acc6abeda4dd84ac5c | missing_45day_session_preparation | 10 |
| SpotBrent | 2023-07-25 | 9beae18a5071ebb3f315a848 | missing_45day_session_preparation | 10 |
| SpotBrent | 2023-07-26 | c8420ab56a4df29d24c8d1b5 | missing_45day_session_preparation | 10 |
| SpotBrent | 2023-07-27 | 5da65ba1cd0c09e8b1e69062 | missing_45day_session_preparation | 10 |
| SpotBrent | 2023-07-28 | 6b43c186e44f7ce4adf78a84 | missing_45day_session_preparation | 10 |
| SpotBrent | 2024-01-22 | 9e1b23b325aba77cbb703cee | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-23 | 58c200ae909a848ff926da74 | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-24 | 01bf260784267d93ce76e221 | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-25 | 6397444a3039e1c4867dd15e | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-01-26 | 666c0bbe602d43cbf8baef30 | missing_45day_session_preparation | 52 |
| SpotBrent | 2024-07-22 | c002e5ef4794429f5b842f26 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-23 | 8f979dc940d9b2ae211c6a90 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-24 | a2e5c8f71ba017158d5202a6 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-25 | e5edfa214e564147e82e29b6 | missing_45day_session_preparation | 12 |
| SpotBrent | 2024-07-26 | d6e7cff7bd1aef63aa0d67a1 | missing_45day_session_preparation | 12 |
| SpotBrent | 2025-02-10 | f5b1803cf39d6edeb396f06c | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-02-11 | d7c8c21dd5d0ad94aed42504 | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-02-12 | 73f72980deab43e63b63aa4e | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-02-13 | 4fe7cb2650e4b5c284d9356d | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-02-14 | 4d2806f05a3b115aa753655d | missing_45day_session_preparation | 26 |
| SpotBrent | 2025-07-21 | 0223cba302b7e83b96407428 | missing_45day_session_preparation | 10 |
| SpotBrent | 2025-07-22 | c4de64f1ffa0ef8e3de8fc0a | missing_45day_session_preparation | 10 |
| SpotBrent | 2025-07-23 | e876f67aef231b91b3c5c5a4 | missing_45day_session_preparation | 10 |
| SpotBrent | 2025-07-24 | 8c5697c68cd1fc74ace69625 | missing_45day_session_preparation | 10 |
| SpotBrent | 2025-07-25 | 258b031ea15e984f9aa6c571 | missing_45day_session_preparation | 10 |
| SpotCrude | 2023-01-23 | c3e3b322f89aa16d477fe5d7 | missing_45day_session_preparation | 53 |
| SpotCrude | 2023-01-24 | 355ca1c07adee0d1aeb493fd | missing_45day_session_preparation | 53 |
| SpotCrude | 2023-01-25 | b50a68dfbdb04a30f12e9a60 | missing_45day_session_preparation | 53 |
| SpotCrude | 2023-01-26 | c2380b0b84e765bc472a0564 | missing_45day_session_preparation | 53 |
| SpotCrude | 2023-01-27 | 736039437cfe8703c32341ae | missing_45day_session_preparation | 53 |
| SpotCrude | 2023-07-24 | 8f401fb18724fa4ecb2f8a36 | missing_45day_session_preparation | 6 |
| SpotCrude | 2023-07-25 | a7b42dc8ec6b6738c775ba17 | missing_45day_session_preparation | 6 |
| SpotCrude | 2023-07-26 | dd362c3ee13db2ddcbd9cf99 | missing_45day_session_preparation | 6 |
| SpotCrude | 2023-07-27 | 82256abfa13bcc7222b6f93d | missing_45day_session_preparation | 6 |
| SpotCrude | 2023-07-28 | 09fc7226899dff17f5bcca96 | missing_45day_session_preparation | 6 |
| SpotCrude | 2024-01-22 | e8eb43e304e030909039f220 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-23 | 1991c7fc6471376a6cf96521 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-24 | 8d0d254ad2d721727cf3c150 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-25 | 573e133e718b83d048cb2275 | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-01-26 | 8b0de7c273b591bb6c13525f | missing_45day_session_preparation | 53 |
| SpotCrude | 2024-07-22 | 3e84577037ce7238ec755f2f | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-23 | 072399489fb15bacf30c5918 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-24 | bee5886b5e233e86567cceb5 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-25 | 890b1d65a5c1e496b77e01d3 | missing_45day_session_preparation | 8 |
| SpotCrude | 2024-07-26 | 95c2666f32b2231028de74aa | missing_45day_session_preparation | 8 |
| SpotCrude | 2025-02-10 | 260d8b5043d33a0c4948f9e4 | missing_45day_session_preparation | 27 |
| SpotCrude | 2025-02-11 | dfc9bcd5e87d8e9dbb62c5dc | missing_45day_session_preparation | 27 |
| SpotCrude | 2025-02-12 | 4e8c147c7c6baa545fddbb82 | missing_45day_session_preparation | 27 |
| SpotCrude | 2025-02-13 | 11a6a8c22334e78217761ee7 | missing_45day_session_preparation | 27 |
| SpotCrude | 2025-02-14 | 2c4718db334d50d0069b28ca | missing_45day_session_preparation | 27 |
| SpotCrude | 2025-07-21 | 97d0fcd243c70f3d16342c6b | missing_45day_session_preparation | 9 |
| SpotCrude | 2025-07-22 | 35ba9e2c232645825deb093d | missing_45day_session_preparation | 9 |
| SpotCrude | 2025-07-23 | 8157378e5e5a500e0590fb57 | missing_45day_session_preparation | 9 |
| SpotCrude | 2025-07-24 | baa63ee6bb8b431a9d217332 | missing_45day_session_preparation | 9 |
| SpotCrude | 2025-07-25 | 5aff841b20cdc59bd1507911 | missing_45day_session_preparation | 9 |
| US500 | 2023-01-23 | 876f6148e2ef9692e251c46d | missing_45day_session_preparation | 26 |
| US500 | 2023-01-24 | 714dcd4ea643c7abb6c67ecd | missing_45day_session_preparation | 26 |
| US500 | 2023-01-25 | 7f35dcccb70ef17e64457117 | missing_45day_session_preparation | 26 |
| US500 | 2023-01-26 | 40cc399289d0861550320dd8 | missing_45day_session_preparation | 26 |
| US500 | 2023-01-27 | f8992619aab40fdf9fa815cf | missing_45day_session_preparation | 26 |
| US500 | 2023-07-24 | 5acc5528cd945dc8e3fc1e27 | missing_45day_session_preparation | 17 |
| US500 | 2023-07-25 | daa7c84bfb25caac4549cc96 | missing_45day_session_preparation | 17 |
| US500 | 2023-07-26 | 7515dfa76df9ce2dd2f5c809 | missing_45day_session_preparation | 17 |
| US500 | 2023-07-27 | 43e781706c514672a6a96738 | missing_45day_session_preparation | 17 |
| US500 | 2023-07-28 | a21704982a48b223406411f6 | missing_45day_session_preparation | 17 |
| US500 | 2025-02-10 | 2148bb57b5270748d1dc5857 | missing_45day_session_preparation | 34 |
| US500 | 2025-02-11 | 2fb13c0d341a2a7c4d93d305 | missing_45day_session_preparation | 34 |
| US500 | 2025-02-12 | 2c7fe293cc545089e134e08c | missing_45day_session_preparation | 34 |
| US500 | 2025-02-13 | d81b55b30354e68499a01af1 | missing_45day_session_preparation | 34 |
| US500 | 2025-02-14 | 7b74fe5bafc42e70f1ae1521 | missing_45day_session_preparation | 34 |
| US500 | 2025-07-21 | f3fb5435535d98dde263ca12 | missing_45day_session_preparation | 17 |
| US500 | 2025-07-22 | 96de88c9fded8e055ca84165 | missing_45day_session_preparation | 17 |
| US500 | 2025-07-23 | 3525cfcc7489cf8cd70fb13a | missing_45day_session_preparation | 17 |
| US500 | 2025-07-24 | e37467317fb2c0e7cbde2e15 | missing_45day_session_preparation | 17 |
| US500 | 2025-07-25 | 35aec47a684766f6889db8e4 | missing_45day_session_preparation | 17 |
| XAUUSD | 2023-01-23 | a3305367b536943b295e9831 | missing_45day_session_preparation | 52 |
| XAUUSD | 2023-01-24 | d01543c086951e45fc2662f6 | missing_45day_session_preparation | 52 |
| XAUUSD | 2023-01-25 | 6fcf6013c2d5308631e99a7d | missing_45day_session_preparation | 52 |
| XAUUSD | 2023-01-26 | b25d14035008c3a9fb364d7b | missing_45day_session_preparation | 52 |
| XAUUSD | 2023-01-27 | e5c3746472e635be94b42589 | missing_45day_session_preparation | 52 |
| XAUUSD | 2023-07-24 | 7e4329e54fa5d8930de487f8 | missing_45day_session_preparation | 6 |
| XAUUSD | 2023-07-25 | a5ac3d675d4a755b21393c6f | missing_45day_session_preparation | 6 |
| XAUUSD | 2023-07-26 | 4bf29f97bbe50d6359520302 | missing_45day_session_preparation | 6 |
| XAUUSD | 2023-07-27 | 301cc40952795f0f5982c6b9 | missing_45day_session_preparation | 6 |
| XAUUSD | 2023-07-28 | 00aba56cdad194a6a6e1630d | missing_45day_session_preparation | 6 |
| XAUUSD | 2024-01-22 | 11bdabc3c84d460a73a8eb56 | missing_45day_session_preparation | 53 |
| XAUUSD | 2024-01-23 | 10d1ae4831eddd0539ba98b8 | missing_45day_session_preparation | 53 |
| XAUUSD | 2024-01-24 | d35cbc2e2f670f3769850370 | missing_45day_session_preparation | 53 |
| XAUUSD | 2024-01-25 | 5204aa8a50ac25f4b1d6d4bb | missing_45day_session_preparation | 53 |
| XAUUSD | 2024-01-26 | 8cc480b98b6fa8a6e114d91d | missing_45day_session_preparation | 53 |
| XAUUSD | 2024-07-22 | 4818610971895f5197695336 | missing_45day_session_preparation | 6 |
| XAUUSD | 2024-07-23 | 1d8414a63d83d92b96f7d5c8 | missing_45day_session_preparation | 6 |
| XAUUSD | 2024-07-24 | 2fae307746859cdb6463521e | missing_45day_session_preparation | 6 |
| XAUUSD | 2024-07-25 | 78212982822d191a812f095b | missing_45day_session_preparation | 6 |
| XAUUSD | 2024-07-26 | f31451969f167fdb421b9195 | missing_45day_session_preparation | 6 |
| XAUUSD | 2025-02-10 | 4ad806629abdf9d7d75ac14e | missing_45day_session_preparation | 27 |
| XAUUSD | 2025-02-11 | 758fe8143b87051535048c5d | missing_45day_session_preparation | 27 |
| XAUUSD | 2025-02-12 | e08a05e0094d27338df76a67 | missing_45day_session_preparation | 27 |
| XAUUSD | 2025-02-13 | 18c4d4b660a470dcab36d784 | missing_45day_session_preparation | 27 |
| XAUUSD | 2025-02-14 | d5f172ae04b389f87a33926f | missing_45day_session_preparation | 27 |
| XAUUSD | 2025-07-21 | 70d6f75e60e33e934c60a925 | missing_45day_session_preparation | 9 |
| XAUUSD | 2025-07-22 | bdce29dd8539285849b9e070 | missing_45day_session_preparation | 9 |
| XAUUSD | 2025-07-23 | 196614c16843b2ea7ef2c9a0 | missing_45day_session_preparation | 9 |
| XAUUSD | 2025-07-24 | b51bc946e3e8272f970a1779 | missing_45day_session_preparation | 9 |
| XAUUSD | 2025-07-25 | 25a29bafe6cd3fdda3e5faa4 | missing_45day_session_preparation | 9 |
