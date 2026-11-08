# Phase 5B — Exact Phase-4 candidate and exclusion audit

№18,366 §§1–11 · 08-11-2026 15:17:33 SGT · by <Codex (GPT-6), effort: not exposed>

## TLDR

**The exact frozen candidate was replayed on 2 eligible new-batch episodes.** It generated no entries: both eligible sessions had no admitted impulse. There is therefore no new realised win rate or mean return for the exact candidate. All reported trade returns are recorded-quote scenarios with unverified execution costs.

**The original independent evaluation remains incomplete.** None of Phase 5's 20 dates matches the frozen Phase 4 date matrix, and 0/160 original planned windows have covering cached ticks. Replaying the original rules on a supported subset does not complete its original test plan.

**The 96 exclusions in PR #195 reconcile:** 86 completed sources lacked fixed-window M30 preparation; 10 lacked a fresh tick boundary. Two failed Nasdaq sources are outside those 96. All 160 scheduled slots, 158 completed tick receipts, 239 new M30 receipts and 75 reused slices are retained in the audit.

**Tier 2 is a different diagnostic model.** The new 72-episode Tier 2 track at 28 bp produced 6 closes, 0 wins, mean -1.02587R at 3.74 bp; repriced at 24 bp it has 0 wins and mean -2.01138R. It uses opening-range/partial context and cannot be relabelled `s3_macro_pullback_v1`.

**Goal progress:** 0/20 pooled exact-candidate closes in this new batch, with no per-class/account qualification. The 73% / 1:1.38 latest-20 goal remains unchanged. RFC engineering gates remain 2/12; financial pass is false. No broker calls or live-rule changes occurred.

## §1. Exact specification and executable identity

The candidate is loaded unchanged from Phase 4 commit `a0d7aea148e73550cc63993228b4ef9b368f0c43`. Rust adapters were compiled from a separate checkout at that commit; source hashes are compared against Git objects and the frozen entry/exit hashes. All 202 relevant Phase 3 reference paths were then reproduced exactly. The Phase 5 fixed-window adapter is not used.

Frozen admission is 3.74 bp/side, the Tier 1 impulse threshold is 48 bp, entry is a later executable 50% retracement, structural risk includes the original cost reserve, entry-price stop follows +1R, and gross target is +1.38R. The primary horizon is native open +4h. The 5/20 trend context uses complete native sessions across 45 calendar days. Recorded spread is included once. Fees, swap, conversion, historical contract terms and round-trip slippage remain unverified.

The “macro” input is causal historical price drift, not fundamental/news data or observed institutional flow. The exact frozen economic hypothesis remains a hypothesis.

## §2. Dates and windows: two distinct mismatches

| Frozen Phase 4 week | Dates |
| --- | --- |
| 2023-03-20 | 2023-03-20, 2023-03-21, 2023-03-22, 2023-03-23, 2023-03-24 |
| 2023-05-22 | 2023-05-22, 2023-05-23, 2023-05-24, 2023-05-25, 2023-05-26 |
| 2023-09-18 | 2023-09-18, 2023-09-19, 2023-09-20, 2023-09-21, 2023-09-22 |
| 2023-11-20 | 2023-11-20, 2023-11-21, 2023-11-22, 2023-11-23, 2023-11-24 |

These dates are preserved verbatim. The Phase 5 April/October 2023 and March/August 2024 labels are a diagnostic dataset requested for this audit; they do not replace the frozen independent dates. Their price outcomes have already been examined in Phase 5, which limits independence.

Native oil sessions open at 07:00 UTC, versus the cached 13:00–17:00 oil windows. US indices open at 09:30 New York, translating to 13:30 UTC during daylight saving and 14:30 UTC during standard time. Phase 5 used 14:30 UTC throughout. Metals 07:00–11:00 and crypto 00:00–04:00 tick windows align. Full historical M30 context still differs: native metal/oil days span 07:00–20:00, US cash sessions 09:30–16:00 New York, and crypto days 00:00–24:00 UTC, rather than four-hour context.

| Symbol | Completed sources | Cover native 4h | Wrong window | Failed source |
| --- | --- | --- | --- | --- |
| BTCUSD | 20 | 20 | 0 | 0 |
| ETHUSD | 20 | 20 | 0 | 0 |
| NAS100 | 18 | 5 | 13 | 2 |
| SpotBrent | 20 | 0 | 20 | 0 |
| SpotCrude | 20 | 0 | 20 | 0 |
| US500 | 20 | 5 | 15 | 0 |
| XAGUSD | 20 | 20 | 0 | 0 |
| XAUUSD | 20 | 20 | 0 | 0 |

Of 158 completed sources, 90 cover the native four-hour label window and 68 do not: 40 oil and 28 index sources. The two failed Nasdaq jobs are counted separately. Missing opening hours are not reconstructed from M30 bars or shifted to a later session.

## §3. Exact reconciliation of 96 excluded completed sources

| PR195 completed-source exclusion | Count |
| --- | --- |
| missing_45day_fixed_window_M30 | 86 |
| missing_fresh_tick_boundary | 10 |

The arithmetic is 160 scheduled = 62 previously evaluated + 96 completed but excluded + 2 failed. The old report's 98 total exclusions include both failures; 96 refers only to completed-source exclusions. Both Nasdaq failures hit the frozen 200,000-row budget and remain failed/spent.

All 399 Phase 5 collection receipt hashes were reconciled. The 75 reused M30 slices refer to 39 distinct verified source files. A successful M30 request can still contain fewer bars than the coded calendar requires; no missing bar is invented and no historical holiday is assumed from absence alone.

Native-rule exclusions below can overlap. They are not interchangeable with PR #195's mutually exclusive 86 + 10 reasons.

| Track | Overlapping reason | Count |
| --- | --- | --- |
| Tier 1 | missing_45day_session_preparation | 152 |
| Tier 1 | tick_interval_does_not_cover_native_four_hours | 68 |
| Tier 2 fallback | tick_interval_does_not_cover_native_four_hours | 68 |
| Tier 2 fallback | missing_fresh_30m_quote | 8 |
| Tier 2 fallback | missing_tick_coverage_at_boundary | 9 |
| Tier 2 fallback | missing_fresh_open_quote | 7 |

## §4. Eligibility and evidence tiers

The combined catalogue contains 544 episode definitions and 616 M30 source files. Of these, 427 episode definitions concern the frozen eight symbols; 117 concern other symbols and are outside this candidate's scope. The register preserves failed, protected, repeated and otherwise ineligible core definitions alongside the selected sessions.

| Dataset | Exact Tier 1 | Tier 2 diagnostic |
| --- | --- | --- |
| phase5_new_batch | 2 | 72 |
| prior_manifest | 35 | 167 |

Tier 1 is the exact frozen sigma candidate. Tier 2 is used only for otherwise excluded clean native-window sources, with the original Phase 3 opening-range and endpoint-context rules; its 48 bp and 28 bp variants remain separate diagnostics. Missing planned peer data causes abstention rather than an absent-peer pass. Protected receipts and inherited repeat intervals remain outside clean tiers; duplicate account/symbol/session identities are not double counted.

The exact new Tier 1 episodes are XAUUSD and XAGUSD on 20 October 2023. Their inclusion follows the frozen full-session coverage rules, not their outcomes. Prior-manifest counts of 35 Tier 1 and 167 Tier 2 remain unchanged. Combined totals are 37 Tier 1 and 239 Tier 2 episodes; they do not constitute a new independent sample.

## §5. Measured candidate and diagnostic results

| Dataset | Track | Episodes | Closes | Wins3.74 | Losses3.74 | Unresolved | Win%3.74 | MeanR3.74 | PF3.74 | Wins24 | MeanR24 | PF24 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| new_batch | tier1_r48 | 2 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| new_batch | tier2_r48 | 72 | 1 | 0 | 1 | 0 | 0.00% | -0.120175 | 0 | 0 | -0.757751 | 0 |
| new_batch | tier2_r28 | 72 | 6 | 0 | 6 | 0 | 0.00% | -1.02587 | 0 | 0 | -2.01138 | 0 |
| prior_manifest | tier1_r48 | 35 | 7 | 7 | 0 | 0 | 100.00% | 1.20463 | Unavailable | 7 | 0.419593 | Unavailable |
| prior_manifest | tier2_r48 | 167 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| prior_manifest | tier2_r28 | 167 | 8 | 6 | 2 | 0 | 75.00% | 0.279986 | 2.18611 | 0 | -0.940631 | 0 |
| combined | tier1_r48 | 37 | 7 | 7 | 0 | 0 | 100.00% | 1.20463 | Unavailable | 7 | 0.419593 | Unavailable |
| combined | tier2_r48 | 239 | 1 | 0 | 1 | 0 | 0.00% | -0.120175 | 0 | 0 | -0.757751 | 0 |
| combined | tier2_r28 | 239 | 14 | 6 | 8 | 0 | 42.86% | -0.279668 | 0.513239 | 0 | -1.39952 | 0 |

| Track | Class | Episodes | Closes | Wins3.74 | Losses3.74 | Unresolved | Win%3.74 | MeanR3.74 | PF3.74 | Wins24 | MeanR24 | PF24 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1_r48 | energy | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | equity_index | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | metal | 2 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | crypto | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | energy | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | equity_index | 10 | 2 | 0 | 2 | 0 | 0.00% | -1.21885 | 0 | 0 | -2.35723 | 0 |
| tier2_r28 | metal | 36 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | crypto | 26 | 4 | 0 | 4 | 0 | 0.00% | -0.929387 | 0 | 0 | -1.83846 | 0 |

| Track | Symbol | Episodes | Closes | Wins3.74 | Losses3.74 | Unresolved | Win%3.74 | MeanR3.74 | PF3.74 | Wins24 | MeanR24 | PF24 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1_r48 | SpotCrude | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | SpotBrent | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | US500 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | NAS100 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | XAUUSD | 1 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | XAGUSD | 1 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | BTCUSD | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier1_r48 | ETHUSD | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | SpotCrude | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | SpotBrent | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | US500 | 5 | 1 | 0 | 1 | 0 | 0.00% | -1.23906 | 0 | 0 | -2.47032 | 0 |
| tier2_r28 | NAS100 | 5 | 1 | 0 | 1 | 0 | 0.00% | -1.19864 | 0 | 0 | -2.24415 | 0 |
| tier2_r28 | XAUUSD | 19 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | XAGUSD | 17 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | Unavailable | 0 | Unavailable | Unavailable |
| tier2_r28 | BTCUSD | 12 | 2 | 0 | 2 | 0 | 0.00% | -0.663991 | 0 | 0 | -1.46129 | 0 |
| tier2_r28 | ETHUSD | 14 | 2 | 0 | 2 | 0 | 0.00% | -1.19478 | 0 | 0 | -2.21562 | 0 |

MeanR and profit factor aggregate normalised research outcomes, not account dollars. A gross 1.38R target is not a realised 1.38 net reward-to-risk ratio. Flat/no-entry/unresolved paths remain distinct. Empty samples have unavailable means and rates, never fabricated zeros.

## §6. Strict versus forced versus continuation

| Dataset | Track | Mode | Cost | Closes | Wins | Losses | Unresolved | MeanR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| new_batch | tier1_r48 | strict | 3.74 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier1_r48 | strict | 24 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier1_r48 | forced | 3.74 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier1_r48 | forced | 24 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier1_r48 | continuation | 3.74 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier1_r48 | continuation | 24 | 0 | 0 | 0 | 0 | Unavailable |
| new_batch | tier2_r28 | strict | 3.74 | 6 | 0 | 6 | 0 | -1.02587 |
| new_batch | tier2_r28 | strict | 24 | 6 | 0 | 6 | 0 | -2.01138 |
| new_batch | tier2_r28 | forced | 3.74 | 6 | 0 | 6 | 0 | -1.02587 |
| new_batch | tier2_r28 | forced | 24 | 6 | 0 | 6 | 0 | -2.01138 |
| new_batch | tier2_r28 | continuation | 3.74 | 6 | 0 | 6 | 0 | -1.02587 |
| new_batch | tier2_r28 | continuation | 24 | 6 | 0 | 6 | 0 | -2.01138 |
| combined | tier1_r48 | strict | 3.74 | 6 | 6 | 0 | 1 | 1.23459 |
| combined | tier1_r48 | strict | 24 | 6 | 6 | 0 | 1 | 0.441394 |
| combined | tier1_r48 | forced | 3.74 | 7 | 7 | 0 | 0 | 1.20463 |
| combined | tier1_r48 | forced | 24 | 7 | 7 | 0 | 0 | 0.419593 |
| combined | tier1_r48 | continuation | 3.74 | 7 | 7 | 0 | 0 | 1.2361 |
| combined | tier1_r48 | continuation | 24 | 7 | 7 | 0 | 0 | 0.451131 |
| combined | tier2_r28 | strict | 3.74 | 10 | 3 | 7 | 4 | -0.395473 |
| combined | tier2_r28 | strict | 24 | 10 | 0 | 10 | 4 | -1.47695 |
| combined | tier2_r28 | forced | 3.74 | 14 | 6 | 8 | 0 | -0.279668 |
| combined | tier2_r28 | forced | 24 | 14 | 0 | 14 | 0 | -1.39952 |
| combined | tier2_r28 | continuation | 3.74 | 13 | 5 | 8 | 1 | -0.139926 |
| combined | tier2_r28 | continuation | 24 | 13 | 2 | 11 | 1 | -1.23333 |

Forced native 4h close is the frozen primary score. Strict counts only bracket-resolved paths; unresolved paths remain visible. 72h M30 continuation is a separate non-confirmatory diagnostic already available in the engine and is not selected as the winning horizon. Recorded M30 ambiguity, gaps and executable-price proxies retain their limitations.

## §7. Controls and every candidate path

| Dataset | Track | Arm | Closes | Wins3.74 | MeanR3.74 | Wins24 | MeanR24 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| new_batch | tier1_r48 | arm_a_breakout_baseline | 0 | 0 | Unavailable | 0 | Unavailable |
| new_batch | tier1_r48 | arm_b_pure_pullback_retest | 0 | 0 | Unavailable | 0 | Unavailable |
| new_batch | tier1_r48 | arm_c_macro_filtered_pullback | 0 | 0 | Unavailable | 0 | Unavailable |
| new_batch | tier2_r28 | arm_a_breakout_baseline | 23 | 6 | -0.683952 | 2 | -1.56619 |
| new_batch | tier2_r28 | arm_b_pure_pullback_retest | 17 | 3 | -0.773567 | 1 | -1.65152 |
| new_batch | tier2_r28 | arm_c_macro_filtered_pullback | 6 | 0 | -1.02587 | 0 | -2.01138 |
| combined | tier1_r48 | arm_a_breakout_baseline | 15 | 2 | -0.452843 | 2 | -1.15212 |
| combined | tier1_r48 | arm_b_pure_pullback_retest | 9 | 8 | 0.95719 | 8 | 0.229211 |
| combined | tier1_r48 | arm_c_macro_filtered_pullback | 7 | 7 | 1.20463 | 7 | 0.419593 |
| combined | tier2_r28 | arm_a_breakout_baseline | 66 | 21 | -0.453282 | 5 | -1.47827 |
| combined | tier2_r28 | arm_b_pure_pullback_retest | 39 | 11 | -0.419086 | 2 | -1.4192 |
| combined | tier2_r28 | arm_c_macro_filtered_pullback | 14 | 6 | -0.279668 | 0 | -1.39952 |

| Dataset | Tier | Floor | Symbol | Date | Episode | Side | Entry UTC | Exit UTC | Exit reason | NetR3.74 | NetR24 | Quote issues |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| prior_manifest | tier1 | 48 | SpotCrude | 2025-04-11 | ce5a75c11b08e52a205e5350 | short | 2025-04-11T10:03:36.606Z | 2025-04-11T10:50:18.330Z | broker_take_profit | 1.24401 | 0.503708 | gap_before_quote |
| prior_manifest | tier1 | 48 | SpotCrude | 2025-04-08 | 2b977b08318352c8c79f2cbc | short | 2025-04-08T10:21:36.559Z | 2025-04-08T11:00:00.219Z | session_end | 1.02484 | 0.288786 | gap_before_quote |
| prior_manifest | tier1 | 48 | SpotCrude | 2025-04-07 | 77b10f0f75d5383bb011c4b2 | short | 2025-04-07T09:09:32.322Z | 2025-04-07T09:21:41.276Z | broker_take_profit | 1.20188 | 0.236957 | gap_before_quote |
| prior_manifest | tier1 | 48 | SpotBrent | 2025-04-11 | 70e3acf0ef22a39ed2354106 | short | 2025-04-11T10:03:37Z | 2025-04-11T10:51:42.127Z | broker_take_profit | 1.25449 | 0.56941 | gap_before_quote |
| prior_manifest | tier1 | 48 | SpotCrude | 2025-04-09 | b8444e851d499207d0449237 | short | 2025-04-09T09:09:34.220Z | 2025-04-09T09:28:50.574Z | broker_take_profit | 1.19983 | 0.219736 | gap_before_quote |
| prior_manifest | tier1 | 48 | US500 | 2025-04-10 | 5ff6b8a3b787709ae2dcc9a0 | short | 2025-04-10T14:31:27.206Z | 2025-04-10T14:58:15.046Z | broker_take_profit | 1.21581 | 0.313617 | gap_before_quote |
| prior_manifest | tier1 | 48 | US500 | 2025-04-08 | 5b35427f494f9de75bb0ddd1 | short | 2025-04-08T15:40:52.170Z | 2025-04-08T16:23:00.163Z | broker_take_profit | 1.29156 | 0.804938 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | ETHUSD | 2024-03-05 | phase5-ETHUSD-2024-03-05 | long | 2024-03-05T02:03:01.965Z | 2024-03-05T02:16:59.064Z | gap_through_stop | -1.19457 | -2.22209 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | ETHUSD | 2024-08-16 | phase5-ETHUSD-2024-08-16 | short | 2024-08-16T01:35:56.388Z | 2024-08-16T02:00:31.048Z | gap_through_stop | -1.195 | -2.20916 | gap_before_quote |
| prior_manifest | tier2 | 28 | US500 | 2025-02-12 | 2c7fe293cc545089e134e08c | long | 2025-02-12T16:12:41.579Z | 2025-02-12T17:00:57.222Z | broker_take_profit | 1.1611 | -0.0374353 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | US500 | 2024-03-08 | phase5-US500-2024-03-08 | long | 2024-03-08T16:03:03.865Z | 2024-03-08T16:28:13.060Z | gap_through_stop | -1.23906 | -2.47032 | gap_before_quote |
| prior_manifest | tier2 | 28 | SpotCrude | 2025-10-07 | 5aca717f89d879a30446d2ed | short | 2025-10-07T10:53:36.730Z | 2025-10-07T11:00:00.434Z | session_end | 0.236858 | -0.866446 | gap_before_quote |
| prior_manifest | tier2 | 28 | BTCUSD | 2025-05-12 | e2f8731ee1bbbd8ebec301bb | long | 2025-05-12T01:21:01.902Z | 2025-05-12T01:34:54.288Z | gap_through_stop | -1.27968 | -2.55704 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | NAS100 | 2024-03-08 | phase5-NAS100-2024-03-08 | long | 2024-03-08T15:39:27.498Z | 2024-03-08T16:05:42.185Z | gap_through_stop | -1.19864 | -2.24415 | gap_before_quote |
| prior_manifest | tier2 | 28 | US500 | 2023-01-27 | f8992619aab40fdf9fa815cf | long | 2023-01-27T15:46:46.666Z | 2023-01-27T17:16:17.598Z | broker_take_profit | 1.16473 | -0.0385635 | gap_before_quote |
| prior_manifest | tier2 | 28 | US500 | 2024-11-04 | 0c8d1df35ca48a63714fb422 | short | 2024-11-04T17:28:23.969Z | 2024-11-04T18:30:00.215Z | session_end | -0.608762 | -2.06672 | gap_before_quote |
| phase5_new_batch | tier2 | 48 | BTCUSD | 2024-03-04 | phase5-BTCUSD-2024-03-04 | long | 2024-03-04T02:01:31.425Z | 2024-03-04T03:39:28.901Z | gap_through_stop | -0.120175 | -0.757751 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | BTCUSD | 2024-03-04 | phase5-BTCUSD-2024-03-04 | long | 2024-03-04T02:01:31.425Z | 2024-03-04T03:39:28.901Z | gap_through_stop | -0.120175 | -0.757751 | gap_before_quote |
| prior_manifest | tier2 | 28 | US500 | 2023-01-24 | 714dcd4ea643c7abb6c67ecd | long | 2023-01-24T17:01:07.816Z | 2023-01-24T18:30:00.811Z | session_end | 0.408516 | -1.00561 | gap_before_quote |
| phase5_new_batch | tier2 | 28 | BTCUSD | 2024-08-16 | phase5-BTCUSD-2024-08-16 | short | 2024-08-16T01:39:56.370Z | 2024-08-16T02:25:21.224Z | gap_through_stop | -1.20781 | -2.16483 | gap_before_quote |
| prior_manifest | tier2 | 28 | SpotBrent | 2025-10-07 | 362375ec6c45fb8d7a2b7d3f | short | 2025-10-07T10:03:15.212Z | 2025-10-07T11:00:00.440Z | session_end | 0.0027674 | -0.885093 | gap_before_quote |
| prior_manifest | tier2 | 28 | BTCUSD | 2025-10-08 | daaa576e731c68edb339d1a8 | long | 2025-10-08T01:16:24.227Z | 2025-10-08T02:21:59.944Z | broker_take_profit | 1.15435 | -0.0681349 | gap_before_quote |

The JSON retains A/B/C native outputs, exact source/input/receipt hashes, paired macro decisions and quote-quality flags. A positive replay does not erase `qualified_gross_r`/`net_r` unavailability in the native engine. Prior wins remain reused development evidence. Directional vetoes and data-driven abstention are separate in the paired tables in JSON.

In the new 28 bp diagnostic, all six retained trades already have negative executable-price gross returns: mean −0.843949R before the extra slippage scenario. Five lose around their original stop; the BTC long on 4 March exits close to its amended entry-price stop and becomes a larger net loss after costs. Every path carries `gap_before_quote`. Thus cost calibration is necessary, but reducing the assumed charge alone would not repair this observed entry/exit performance.

The macro direction filter vetoes nine unfiltered pullback paths: three winners and six losers at 3.74 bp. Another two losing paths are blocked by unavailable own context. The six retained paths all lose. Avoiding some losing opportunities is not evidence that the retained entries have positive expectancy; missing-data abstentions must not be credited as macro foresight.

## §8. What would be needed next

1. Preserve the original dates, native windows and full-session context as a single immutable specification generated from the frozen native calendar.
2. Reconcile historical holidays, maintenance and actual missing preparation before collecting again. Full coded weekday slots are not authoritative historical broker calendars.
3. Produce a metadata-only gap/overlap inventory for the original 160 windows. It must include Phase 5's new M30 requests: parts of the original planned dates now lie within previously read preparation. Do not call those prices untouched or refetch spent intervals.
4. Any later authorised collection must acquire missing native tick windows and necessary history under an explicit revised independence assessment. This phase makes no such requests.
5. Calibrate matched account-scoped round-trip costs before interpreting 3.74 bp sensitivity as executable economics. Then assess the predeclared class screens and per-account goals without tuning these already inspected dates.

Completing unsupported windows requires different data, not changing the original model's clocks or replacing missing ticks with synthetic evidence.

## §9. Verification and RFC gates

| Gate | Status | Evidence / remaining work |
| --- | --- | --- |
| G1 | partial | Source/code/calendar/contract/cost identities linked; historical authority remains unverified. |
| G2 | passed | All 202 original Phase 3 Tier 1-r48 / Tier 2-r28 paths at 3.74 bp reproduced exactly using Rust built from the literal Phase 4 commit. |
| G3 | passed | Zero broker calls; immutable source and eligibility freeze; deterministic native receipts and cost arithmetic. |
| G4 | partial | New sources:2 exact Tier 1,72 Tier 2 diagnostics;68 completed sources have wrong native windows. Full denominators retained. |
| G5 | open | Dated historical holidays, maintenance and contract validity required before collection/qualification. |
| G6 | open | Matched broker round-trip deals, fees, swaps and conversion not available. |
| G7 | open | No out-of-sample calibrated round-trip cost error bounds;3.74 bp remains assumption. |
| G8 | partial | 18 cached native paths and recorded quote prices reconciled; real execution/restart parity unproved. |
| G9 | partial | Exact candidate identity restored for supported cached windows; original independent date matrix remains unexecuted; Phase 5 dates already exposed. |
| G10 | not_passed | No independently qualified class/account result. Reused development results cannot qualify the candidate. |
| G11 | not_performed | No broker shadow/canary evaluation. |
| G12 | not_demonstrated | No per-strategy/account operational goal qualification. |

- Exact Phase 4 source/binary provenance and202 original native path comparisons.
- 2591 independent native cost arithmetic checks.
- 1041 verified market source files; all399 Phase 5 receipts reconciled.
- Three focused regression tests; full research test outcome and deterministic rerun proof in verification JSON.
- Protected validation unopened, offline network guard active, zero new broker requests.
- Coverage SHA256:`da8a5a990cf185318df3ae6db59bf71d4d2db2d1e7f0663d55b0b627313d5094`; result SHA256:`2f3cf05f4ae50e033e6f8e07770848774cac6995a4ae90a9303b4ce61bd657c7`.

Reproduction requires the retained raw caches and frozen Rust adapters. Run `python3 ops/research/phase5b_audit.py prepare` and `evaluate` only with matching scope/source/binary hashes; source changes require a separately registered audit. The report is generated by `python3 ops/research/phase5b_report.py`. GitHub artifacts contain the audit and native outputs, not the complete raw tick cache.

## §10. All 160 Phase 5 source dispositions

| Episode | PR195 state | PR195 reasons | Native open UTC | Window covered | Native M30 missing | Phase 5B track | Native eligibility reasons | Tick receipt SHA256 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T07:00:00Z | False | 38 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 797dd1b0050d11837eb0a477716e5862c8791f9924bde02db2e65b28e90e580c |
| SpotCrude-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | e73f37e24a6e8cfddcea526639764800f948cba4744f12bd9c821afd5153551d |
| SpotCrude-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 5d5054e9d015dd02b5db25287e72d6efc138897abc3d910a13d0a1cc342d383f |
| SpotCrude-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | edd91d6fe4e41282640687549d139840782c2a9187864cf8f409a0e7ff829a5c |
| SpotCrude-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | d73910a7398373455bda111d5af61ff2ceb5b266d09a07a72d7782d9f3a4ae1f |
| SpotBrent-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T07:00:00Z | False | 38 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | ab1d7210a74018b544b0f78e5ec3ecba87932d53e050ea46af9bf171db4f484f |
| SpotBrent-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 52c6409cce23540498e78301d3b8bf12989544e4449ba6170bc1e5a6f6369725 |
| SpotBrent-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 81699260f9a450a60a3a9f69524ca946aec4c45c29a9a21cadc955c8d77bdf3f |
| SpotBrent-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 41e8c476f448e851a289bd92c38da9560c7ab37a81abe3313c4173ad0dfde0bb |
| SpotBrent-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T07:00:00Z | False | 26 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 40253a8e80950a4166b6e68f4098fbe0dbcb95453e4541d3832944593250a6ad |
| US500-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 89ac2697297dd53584bc36bda53270a1db747acb3a74fb8b4fd9f3a0e8e02ea5 |
| US500-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 979dee673d23b704c50aca965aa655d7b5892e7d194e1b7201706031cdf47442 |
| US500-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | c1b3de6c2cf2e8709fdd2ae22e8274fcc30bdd9807ee7627622959269007314b |
| US500-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | efcc0bf06c63359535faac0e1e3b56aea860e86b618bb0d46f0e40284f80ef9f |
| US500-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | d381bbdaee873cbf63549b2085bb7e1cfc23de9128b3a099fff00950d1d0e55d |
| NAS100-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 31b17ffc1a6430213c579d6c933cad219c18cc73b74bbb04318dc111984080d7 |
| NAS100-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | ad207fe0da5081ceaca039bafc68f9f5881e964773e3a94afdefee1c51ea1f45 |
| NAS100-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | e427089f896ea17f58143645ee7d080239c9436e2387a0541a8169d1765b4359 |
| NAS100-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | ba272ab17f06f449dedbc97f80fec65b9e8ca68f4c3d45b381b4d166d5b9351b |
| NAS100-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T13:30:00Z | False | 13 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | a4c34ee8cb45650b1c5b2d6cc78601133da4bbbab094df1fc581c36f95fe4727 |
| XAUUSD-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 37e46b294002f7c66f94cc4c8a0b93668f5558ddba064f9ed8da033b25fb1d76 |
| XAUUSD-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 31d48a859280d3e7afe58744c10564006d0b1e730cc9f151e0d44090ca41c60d |
| XAUUSD-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | befd52164f3fedf25c55f1a2d9b438168454d501b3edbc58e3a7fb3ba0f49190 |
| XAUUSD-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 441db229c1d294ad42fc6cb062355c9d7962d72f9d31085fb33bb13384b8f0b2 |
| XAUUSD-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 1fa27343cd7a41aeaf91a359dd35210cdbb6d420083e31f3dc583f45a43bee74 |
| XAGUSD-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 85f16ed3982f25bd6650bbb3c468828ff687632511edd8586dab5d9a1aa1d73c |
| XAGUSD-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | cb19f5fd9e5ae05596c2cbc65c6362dd8af2aaced708c24c6d9c819e27a9e0e8 |
| XAGUSD-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 6e307364412a0b26ccbfe22740e8e8e000f4c5862ed444b30c9c5f2130abed19 |
| XAGUSD-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | 49fedae57e219261366650ecf6f164e6091d7a81b0a2a8323bc130713d23f7e3 |
| XAGUSD-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T07:00:00Z | True | 26 | tier2 | missing_45day_session_preparation /  | db1a37a6c79317033006333948e853eafcf1093e03c02a79f6cfbcd0e10fbdfb |
| BTCUSD-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T00:00:00Z | True | 99 | tier2 | missing_45day_session_preparation /  | e8ab232f03e6442c3302b621492017ff0670d279110f49d60b859a8a7ff99365 |
| BTCUSD-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T00:00:00Z | True | 97 | excluded | missing_45day_session_preparation / missing_fresh_30m_quote; missing_tick_coverage_at_boundary | fefe7a6149830b649d928eb2e104c0c7f3bae128a2c7f9a2beea92d9f80ae8ea |
| BTCUSD-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T00:00:00Z | True | 91 | excluded | missing_45day_session_preparation / missing_tick_coverage_at_boundary | e739adb5c05e6778e327194a1d2c6021b35bc5159dce6329b312b97a4e633799 |
| BTCUSD-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T00:00:00Z | True | 87 | tier2 | missing_45day_session_preparation /  | 6dd515abcbed15034cd803a75381651323cf97271c166f5cab1a7e6a4f5d1472 |
| BTCUSD-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T00:00:00Z | True | 87 | tier2 | missing_45day_session_preparation /  | cf65d22ffa8f7ff52a7b3f7e840cb05252b972a0136b7a4165b2782142008a6e |
| ETHUSD-2023-04-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-17T00:00:00Z | True | 99 | tier2 | missing_45day_session_preparation /  | 7f538412e323236ef05510df7e759159c7337e5eb117ae83b8287b5cecd8913f |
| ETHUSD-2023-04-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-18T00:00:00Z | True | 97 | excluded | missing_45day_session_preparation / missing_fresh_30m_quote | 571cb28506b925c43ddab3841ea92cec8335b98462435815b7243ba42689080a |
| ETHUSD-2023-04-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-19T00:00:00Z | True | 91 | tier2 | missing_45day_session_preparation /  | 8d84619a4a68156f4acc9844b2c1352e8b98cadb5dd7a24c5ad053e77df9f033 |
| ETHUSD-2023-04-20 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-20T00:00:00Z | True | 87 | tier2 | missing_45day_session_preparation /  | 33810e71705b7c7f8c558d0f0f910aba42a27db7f3d86f449022be3f1c8d1960 |
| ETHUSD-2023-04-21 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-04-21T00:00:00Z | True | 87 | tier2 | missing_45day_session_preparation /  | 1b5344531eacd77f28605a78d93919a8489a1c5a4c7101cad8668ba4452111b2 |
| SpotCrude-2023-10-16 | evaluated |  | 2023-10-16T07:00:00Z | False | 15 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | f2fa71a7be9b64f6eb8fac3792d3bfc0e0254801dff905dccbc86b30342ca7c2 |
| SpotCrude-2023-10-17 | evaluated |  | 2023-10-17T07:00:00Z | False | 3 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 4d89d092ddd5786b068b5e2a7658bfc9f960d9246a3d3057aa1b20b04aa1c37c |
| SpotCrude-2023-10-18 | evaluated |  | 2023-10-18T07:00:00Z | False | 3 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 2e28e3cb567a8adfe04663a9bdd194187e202a2ac1a19cfadcd0f2e7395376ec |
| SpotCrude-2023-10-19 | evaluated |  | 2023-10-19T07:00:00Z | False | 3 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | eeca7908c1a42f3f44accfe144c2a97975ea850d8275140841e467d54012ea4c |
| SpotCrude-2023-10-20 | evaluated |  | 2023-10-20T07:00:00Z | False | 0 | excluded | tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | b860dda1a3e9cc5a6934ae98e1c48644f87bc15d34f86be62da5d206d1451710 |
| SpotBrent-2023-10-16 | evaluated |  | 2023-10-16T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | ff74880614ec6514c16cadb00a062245d1e4d09d215316bfe459c322ddc85281 |
| SpotBrent-2023-10-17 | evaluated |  | 2023-10-17T07:00:00Z | False | 5 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | dd141fc09d57937a733c82cb614dd668f8ce0eabee8990b9b508dc8f77736581 |
| SpotBrent-2023-10-18 | evaluated |  | 2023-10-18T07:00:00Z | False | 5 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | f44fee0b42c60ba07ef0667abbfc5b0db5762bf715fa4b97e0a90bd319f87e68 |
| SpotBrent-2023-10-19 | evaluated |  | 2023-10-19T07:00:00Z | False | 5 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 7134d653f50cc867ed01689b5ce440110d02d3bdd497e8cd42a9b641b4cbba24 |
| SpotBrent-2023-10-20 | evaluated |  | 2023-10-20T07:00:00Z | False | 0 | excluded | tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 571e8de6bf90dc5546c5f8ee63d37fa029e500f471fdce2d8050a622fdcc9cf0 |
| US500-2023-10-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-16T13:30:00Z | False | 8 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 33628f598f55ebbc5a9748edb25d6dff9679a78d0321b51b04d9dbc77976c792 |
| US500-2023-10-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-17T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | afed1ec30a5cec5d7a6385cba54d9df0900518384e5d257da3ebc910ee83ee48 |
| US500-2023-10-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-18T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | f76f66c64f2e050f43e2bf272eb30dfe9e698fccae2048235c27dd6b510e5fd9 |
| US500-2023-10-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-19T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | dc7a4eb09a63a9aff4f668d4f1753e7d13d3d70db468882b197fbb4426d64877 |
| US500-2023-10-20 | evaluated |  | 2023-10-20T13:30:00Z | False | 0 | excluded | tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 84670b5286733340dc21081af5429d85731317f52965e335e4dda55e9afaf9ff |
| NAS100-2023-10-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-16T13:30:00Z | False | 8 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | eb0ec8b13d3874a18e9ff5d264a6f78aed90e5aa386d881cf471969946111ebf |
| NAS100-2023-10-17 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-17T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 622a8df00f688aa4bd496b0df076ad22f1ec6d2709b024b2ef866653d8c8b97d |
| NAS100-2023-10-18 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-18T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 50b4c0f6d27b9c5899e95313b20adc5fd8fa2846002bb152d7f80e79057a7580 |
| NAS100-2023-10-19 | completed_but_excluded | missing_45day_fixed_window_M30 | 2023-10-19T13:30:00Z | False | 6 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 0e392f0afbb138378d923efc9408b2b3d4c70447bfda2c8701064a4a31167048 |
| NAS100-2023-10-20 | evaluated |  | 2023-10-20T13:30:00Z | False | 0 | excluded | tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 678ba2d27d7d98f54d0d1d3943bd84be3d9b55a0964a231c79a25ee7b65d92bf |
| XAUUSD-2023-10-16 | evaluated |  | 2023-10-16T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | c840cdfef22e33d8cbcb98c1b78c26b1a85142181782e24715887f72052340cb |
| XAUUSD-2023-10-17 | evaluated |  | 2023-10-17T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 90788a2ec86a60200a19fb8b68c8e7e2b0b316656e566fc48493adfab70333b8 |
| XAUUSD-2023-10-18 | evaluated |  | 2023-10-18T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 5578b66f1b89b7654e7938f53893b342816cec5c024643fbe8774ab53b1143cd |
| XAUUSD-2023-10-19 | evaluated |  | 2023-10-19T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 33c0dc9868082174a7b8977aa781c3d96fe9f2a9942fbda9a25988580f5d71cc |
| XAUUSD-2023-10-20 | evaluated |  | 2023-10-20T07:00:00Z | True | 0 | tier1 |  /  | 261f9f6f5d5c900079a22bc0638231049be209f297eb6c7531bd70f56e019e87 |
| XAGUSD-2023-10-16 | evaluated |  | 2023-10-16T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | ec68c851144240f4f4c78fe0d73fd672901f72ad46d622c9852d04eba18edee6 |
| XAGUSD-2023-10-17 | evaluated |  | 2023-10-17T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | cf3185130bf0ef3ee534a2399d280da49fceca21bcca28dc70ec29a24daef54d |
| XAGUSD-2023-10-18 | evaluated |  | 2023-10-18T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 4f71be32560f89a5a3931be97601675f441ee777f4e655cb27cda4f563fd979d |
| XAGUSD-2023-10-19 | evaluated |  | 2023-10-19T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | db8e529f433268d15b727285519af408203bc52f3dbc9c46a0a4d0aa0a975dde |
| XAGUSD-2023-10-20 | evaluated |  | 2023-10-20T07:00:00Z | True | 0 | tier1 |  /  | e17764c0b632d7987bd625541fd11ab5157e3eb1cd4d085ecd500a1af51d2146 |
| BTCUSD-2023-10-16 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-16T00:00:00Z | True | 56 | excluded | missing_45day_session_preparation / missing_fresh_open_quote; missing_tick_coverage_at_boundary | 986f60df26925b3da893a04221c65583193b0631b3d04184337049d39b0ce425 |
| BTCUSD-2023-10-17 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-17T00:00:00Z | True | 51 | excluded | missing_45day_session_preparation / missing_fresh_open_quote | df80401ad63c6ba35940e2d44898d074c811588fead4dd06bed5f2eb50a30ff8 |
| BTCUSD-2023-10-18 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-18T00:00:00Z | True | 51 | excluded | missing_45day_session_preparation / missing_fresh_open_quote; missing_fresh_30m_quote; missing_tick_coverage_at_boundary | df5d9b92cae040ce0a6d3fddaad16f619ede7d3f557c4d0f9f2520cdc7772018 |
| BTCUSD-2023-10-19 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-19T00:00:00Z | True | 47 | excluded | missing_45day_session_preparation / missing_fresh_open_quote; missing_fresh_30m_quote; missing_tick_coverage_at_boundary | 521cf013e43deccfdb4fb4e920e33ecb16066baec23420ec895137d1a312b4c5 |
| BTCUSD-2023-10-20 | evaluated |  | 2023-10-20T00:00:00Z | True | 47 | excluded | missing_45day_session_preparation / missing_fresh_30m_quote | b17b1ac38ab6a8a9abf281299aa06de1eeecdf015eebe54eae1d055b13242199 |
| ETHUSD-2023-10-16 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-16T00:00:00Z | True | 56 | excluded | missing_45day_session_preparation / missing_tick_coverage_at_boundary | a3a6b907e0464c32a74f03819516bcbe3552499ea049424175ee683af3a61f54 |
| ETHUSD-2023-10-17 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-17T00:00:00Z | True | 51 | excluded | missing_45day_session_preparation / missing_fresh_open_quote | 2c0f864df1de18f43fbf5bd455a2e807dcd1748d6e245e44004d403198ea1947 |
| ETHUSD-2023-10-18 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-18T00:00:00Z | True | 51 | excluded | missing_45day_session_preparation / missing_fresh_open_quote; missing_fresh_30m_quote; missing_tick_coverage_at_boundary | 8a6e1e25331a295a627e1327d0fb86ec28b9b04db53d42680d27f2fff968f165 |
| ETHUSD-2023-10-19 | completed_but_excluded | missing_fresh_tick_boundary | 2023-10-19T00:00:00Z | True | 47 | excluded | missing_45day_session_preparation / missing_fresh_open_quote | 3626c8a72e510a13c3a38afd39801c26513976fc50129ff2492cade506c2cb65 |
| ETHUSD-2023-10-20 | evaluated |  | 2023-10-20T00:00:00Z | True | 47 | excluded | missing_45day_session_preparation / missing_fresh_30m_quote | f40521e2906091197de9930b0e62b7a66729343a2ce6e1e7c6a11935e26d8e56 |
| SpotCrude-2024-03-04 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-04T07:00:00Z | False | 151 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 2418002409cb019a77987cad8dac1c6021b1972d62b73677b1581037b5880c5a |
| SpotCrude-2024-03-05 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-05T07:00:00Z | False | 125 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 1440e70cbb111d328d36cec4fcb30c45364f321b76ab258e33a24c1c1f7caed4 |
| SpotCrude-2024-03-06 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-06T07:00:00Z | False | 125 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 4cf98c8813617b131df2b930e8523177d8265fecdebac8c17305d236cc45560c |
| SpotCrude-2024-03-07 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-07T07:00:00Z | False | 125 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 0ee0197b8b02f22b78967c5978c41f17928f0adbd2290c2325e03c1bdc104521 |
| SpotCrude-2024-03-08 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-08T07:00:00Z | False | 99 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 64c6e1ba7c109b7b3b716b1c53d72fe3b14637057277d0fd5e6ab1618a98eb03 |
| SpotBrent-2024-03-04 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-04T07:00:00Z | False | 150 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 0a428ead1cfba7e134f108a9c1b19f5b1e0f97c406e074d379130defeaff3d88 |
| SpotBrent-2024-03-05 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-05T07:00:00Z | False | 124 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 1f9b8dbfcb754346778cd351252e776e3e06e3ddd3b3f8d92708d70b8af3f6f5 |
| SpotBrent-2024-03-06 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-06T07:00:00Z | False | 124 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 80994052238d6441ba5766ceb7c478f3a50f3e0bfbea687241f0d7d85b2a8155 |
| SpotBrent-2024-03-07 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-07T07:00:00Z | False | 124 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | f6bf5d688216f3b5c2e16788a48fad81dfcac8a39145e0a4a321277b2f07d2e3 |
| SpotBrent-2024-03-08 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-08T07:00:00Z | False | 98 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 972eaf7f3039d3513d8963837ba52af985e1e341e8d4bbce8bea20342d291b07 |
| US500-2024-03-04 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-04T14:30:00Z | True | 79 | tier2 | missing_45day_session_preparation /  | 023b5cd03570eaaf27bcbf0afde8b4b504fbd293b0e770a026dba49797400afa |
| US500-2024-03-05 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-05T14:30:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | 6d394dfcc8112dae92cc87c71f873942573f37e1dd50cd3d07729a822b2e92b9 |
| US500-2024-03-06 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-06T14:30:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | 78571656b9fce32f6b72ff814dcd793489b04932ab606e3ec5f62113786ee281 |
| US500-2024-03-07 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-07T14:30:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | 2a3c5c355e8574509cb71c23d632b5e0af31f81f55d82afe7f32cacbd5335bac |
| US500-2024-03-08 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-08T14:30:00Z | True | 53 | tier2 | missing_45day_session_preparation /  | 3a5faa5460a186cb5079b8eb93685a98fb223dc87cffe7dfbd9a4009362068e6 |
| NAS100-2024-03-04 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-04T14:30:00Z | True | 6 | tier2 | missing_45day_session_preparation /  | 665b5b3b85a37d27800eda86a8add3e5cffad642b71b2fb3bb7288e210a084d2 |
| NAS100-2024-03-05 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-05T14:30:00Z | True | 6 | tier2 | missing_45day_session_preparation /  | 9cffa07b799901b4a89a1e5a119291680053135c7cb90a69dbc237b8b4963ece |
| NAS100-2024-03-06 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-06T14:30:00Z | True | 6 | tier2 | missing_45day_session_preparation /  | df2b5f4d6f34bcdd90af1f469f66dccb7c435a7c88e502b1c74f81d3a72f91b9 |
| NAS100-2024-03-07 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-07T14:30:00Z | True | 6 | tier2 | missing_45day_session_preparation /  | 362826a6d48c2bc8751c9d7d0eec634ad23aad2c86d604a80e0f5b2f5a0fe3c3 |
| NAS100-2024-03-08 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-03-08T14:30:00Z | True | 6 | tier2 | missing_45day_session_preparation /  | 578afd7e143b32154ca40408c64bc0540c83efee45258ef1509008129f4cc0bf |
| XAUUSD-2024-03-04 | evaluated |  | 2024-03-04T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 9afdefc3796a60dd006e26a9d5a88342350971249d9f71b6756110e248e5aa40 |
| XAUUSD-2024-03-05 | evaluated |  | 2024-03-05T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | f9164e31121c81d75d8153359c705f2dc469e12521c16f37dffdce1767161c55 |
| XAUUSD-2024-03-06 | evaluated |  | 2024-03-06T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 1abe68983f1d4dbd1e123662d75549a84bd8c92582f1c0a2cd5bf8ab8582496a |
| XAUUSD-2024-03-07 | evaluated |  | 2024-03-07T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 67529885380868782d6c1c2c8b53e030bf77bc4434f8411bfda453087144bd7c |
| XAUUSD-2024-03-08 | evaluated |  | 2024-03-08T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | e87b5d53d03be94a4edcb2eaaed9ee37e8648ff086f7279e2375d893e111bb42 |
| XAGUSD-2024-03-04 | evaluated |  | 2024-03-04T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 68c77d341c3201b328908442dd7b4fc9d1f24074e10b138b8d5819692d9e4ab5 |
| XAGUSD-2024-03-05 | evaluated |  | 2024-03-05T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 50ab223a5c0e04ce7df70f29db25c0cd47fffa4bdbe8828a079d587bc97b3e3c |
| XAGUSD-2024-03-06 | completed_but_excluded | missing_fresh_tick_boundary | 2024-03-06T07:00:00Z | True | 1 | excluded | missing_45day_session_preparation / missing_tick_coverage_at_boundary | 2ffeb84dd322b0a0964bf1be84a3958871277a8728bf3e57472a0b157533bfa3 |
| XAGUSD-2024-03-07 | evaluated |  | 2024-03-07T07:00:00Z | True | 1 | tier2 | missing_45day_session_preparation /  | 0496976989099751c77733606613231221e805eb25bb2c7f00d0a7245a5647ae |
| XAGUSD-2024-03-08 | completed_but_excluded | missing_fresh_tick_boundary | 2024-03-08T07:00:00Z | True | 1 | excluded | missing_45day_session_preparation / missing_tick_coverage_at_boundary | 1d4f7883199888c6d6c9595973d0d687c72b97fdc20bb5d4a17ce34fb5051ffe |
| BTCUSD-2024-03-04 | evaluated |  | 2024-03-04T00:00:00Z | True | 77 | tier2 | missing_45day_session_preparation /  | 960f9be13352e6bee833f693277a418bff24e9a532f87c0ca2a479dbdcb07470 |
| BTCUSD-2024-03-05 | evaluated |  | 2024-03-05T00:00:00Z | True | 75 | tier2 | missing_45day_session_preparation /  | 40dd8bc8bc40b7dfc713ab88fd5452cd0ed9a0615160cb4d767ba098b0d95c5e |
| BTCUSD-2024-03-06 | evaluated |  | 2024-03-06T00:00:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | 1fd160f3e57b7a3ac1f4b9913b5e581304e051c879208546572583b8b2b8e449 |
| BTCUSD-2024-03-07 | evaluated |  | 2024-03-07T00:00:00Z | True | 62 | tier2 | missing_45day_session_preparation /  | 48c8bffa38305edb4aa0b33739fe114bb2f8a458e3be32aeb0c3b89896b9599a |
| BTCUSD-2024-03-08 | evaluated |  | 2024-03-08T00:00:00Z | True | 62 | excluded | missing_45day_session_preparation / missing_fresh_30m_quote | c1be171f6ac724863805016bd7c08b31f1bd1db83eeb81d1c3c43fcb947a182f |
| ETHUSD-2024-03-04 | evaluated |  | 2024-03-04T00:00:00Z | True | 77 | tier2 | missing_45day_session_preparation /  | 721071e02dbec33ca8ddb318aa7e2f6c4bfd24b59abdfac93d4c46c1166f4b49 |
| ETHUSD-2024-03-05 | evaluated |  | 2024-03-05T00:00:00Z | True | 75 | tier2 | missing_45day_session_preparation /  | 103308a0e11d5a39c805078cef0fe9cddb431f86a6aa5b1b6d02034b66acdf53 |
| ETHUSD-2024-03-06 | evaluated |  | 2024-03-06T00:00:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | 88b52af313ebc93bc2f0defcb2ecfc96398a83480d68a1f1f6fb0c233dbc6024 |
| ETHUSD-2024-03-07 | evaluated |  | 2024-03-07T00:00:00Z | True | 62 | tier2 | missing_45day_session_preparation /  | b2ce18a0756b8408ef26e268e60a66bf87225f1567668e14df6c0ef61d7affe6 |
| ETHUSD-2024-03-08 | evaluated |  | 2024-03-08T00:00:00Z | True | 62 | tier2 | missing_45day_session_preparation /  | b1a9a5b1cfb1745d26fec4f5f841821fbdf402de5b0d24afe80f24ecc917a9a8 |
| SpotCrude-2024-08-12 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-12T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | b36d5207474836de541899a309d070c177bc19b7122abc6e19436de374f0e99d |
| SpotCrude-2024-08-13 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-13T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 22016a36c7f6ad9900fe96ed2f2f45546e339cc7b56efe9823fbe859cb6ec83c |
| SpotCrude-2024-08-14 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-14T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 5172ec059d566a2ed11852c43e2620c4c771d4ea0514576711acfcd7df939e7d |
| SpotCrude-2024-08-15 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-15T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 19af25d0e1884687f2fcd37324016e9b350d7499ab4a0e8cbd5998f65bd7e8e3 |
| SpotCrude-2024-08-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-16T07:00:00Z | False | 17 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | a31225758ccd34f89bc3a6c9274069bc2a7ddaecb2e9bc27553f2ef999e2002e |
| SpotBrent-2024-08-12 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-12T07:00:00Z | False | 19 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | bbbcf70c6a6b1fc6b173a4a9c64ffbd13d97ca05466f68e6d2534e6b0399ad64 |
| SpotBrent-2024-08-13 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-13T07:00:00Z | False | 19 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 40ef39d0d857ac780532f463874434bfb589cda60ede3c8509519fd6930e8961 |
| SpotBrent-2024-08-14 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-14T07:00:00Z | False | 19 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 142f1c50ce8573598950d857a956ec1a0036365e053af9c833295781e46a503a |
| SpotBrent-2024-08-15 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-15T07:00:00Z | False | 19 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 9fe15f367e5382f4d380b54841b84d38b81bc7fc2c8abee9e768b9755148695a |
| SpotBrent-2024-08-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-16T07:00:00Z | False | 19 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | c90f5154f4c2d367a5f29aae6ddea29003747a8d5163a0c526f647fe976e1f10 |
| US500-2024-08-12 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-12T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 93d28103b4532cfd193a69f9dc3723f46b497691b7e7d936500fb715673d7ea3 |
| US500-2024-08-13 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-13T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 9310997c7df6fb22b1e5a5cdb5f0076680a1dda406f0af4ca92e0dc4192c3c8b |
| US500-2024-08-14 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-14T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | da8d9c200cbc15e84a56d421005d2a20f7cb8732107e6f5d0f97a87b3b859f7d |
| US500-2024-08-15 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-15T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 3b1aaffbadd820ba448580d71dcbee65c9eaeaf8bd10592a72c7ea5a986c9cb6 |
| US500-2024-08-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-16T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 26930363aa6f50dd234195f9b17cd8f923fd75854b8946bfc18a037e5fb4d2d0 |
| NAS100-2024-08-12 | tick_source_failed | missing_45day_fixed_window_M30; tick_source_failed | 2024-08-12T13:30:00Z | False | Unavailable | excluded |  /  | a11ce68367ae5af31c0d3f5e444e5e08c5090c7d4d80519a16313c6032e092c0 |
| NAS100-2024-08-13 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-13T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | b75514af219df1fa23034d98442bd287b7da4da88b15d2ea094f068f3b870361 |
| NAS100-2024-08-14 | tick_source_failed | missing_45day_fixed_window_M30; tick_source_failed | 2024-08-14T13:30:00Z | False | Unavailable | excluded |  /  | a76fb6ede3de8f29fa16eaeb9e7c42aeb52b9eb19a575d9b28554dc7c88fa1b7 |
| NAS100-2024-08-15 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-15T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 68af27a4fdc6b551c561e27c8961d11a87a2b91d5d27d4a5ee06ec6e60852ad7 |
| NAS100-2024-08-16 | completed_but_excluded | missing_45day_fixed_window_M30 | 2024-08-16T13:30:00Z | False | 11 | excluded | missing_45day_session_preparation; tick_interval_does_not_cover_native_four_hours / tick_interval_does_not_cover_native_four_hours | 7d74005a181b020bfbffcd947d5d3f13ee998a31e6f1d3b75862723a2583681f |
| XAUUSD-2024-08-12 | evaluated |  | 2024-08-12T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 9212d876ef7da979c4471c1c0f27685e8ce24de12aa42ea0af1980b2049cb821 |
| XAUUSD-2024-08-13 | evaluated |  | 2024-08-13T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | ab949e2eba01d652d7ff3291802a832b23e67d59ceba4e1642a64e553a5f325f |
| XAUUSD-2024-08-14 | evaluated |  | 2024-08-14T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 4fe174cbea982de8976095a8d96bf112aa4a01b935315301c79bfff27d8554e7 |
| XAUUSD-2024-08-15 | evaluated |  | 2024-08-15T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | bc223c5605ee54cae6d44376c46dbf0a46a9feedd645aaf85ae69f8d84b0d8e5 |
| XAUUSD-2024-08-16 | evaluated |  | 2024-08-16T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | fc874cffb0ad40adad35e13a3efa6511fd58077d114ad6f6f648ef09675a636c |
| XAGUSD-2024-08-12 | evaluated |  | 2024-08-12T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 255a4e31c922901f566eeb5326783eface6de80878e2f396ffd63c1e1eb3c79d |
| XAGUSD-2024-08-13 | evaluated |  | 2024-08-13T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 029cd9cb79754e4b9caac4387b3bed89d4f493c88d35c2ee9d24bd0b1ecd556c |
| XAGUSD-2024-08-14 | evaluated |  | 2024-08-14T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 5d04f8561d877c8388c9592a19b35c1180fd3771c9f56058a0ddd91688794e2f |
| XAGUSD-2024-08-15 | evaluated |  | 2024-08-15T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | addcba80d2923a78af97352e9776463f6c146385f027c8b9e6ca26451c0736b3 |
| XAGUSD-2024-08-16 | evaluated |  | 2024-08-16T07:00:00Z | True | 3 | tier2 | missing_45day_session_preparation /  | 831842fd9093d6f74158f26a04de3ebbdd4558de0916cf425c1ddc1468cd07d9 |
| BTCUSD-2024-08-12 | evaluated |  | 2024-08-12T00:00:00Z | True | 69 | tier2 | missing_45day_session_preparation /  | 02e43b67d8ee6be81720ce9cb28c534ddba9f3f078776abebc7216ce3ea6d494 |
| BTCUSD-2024-08-13 | evaluated |  | 2024-08-13T00:00:00Z | True | 67 | tier2 | missing_45day_session_preparation /  | 1eeb5b32449b4143dbc341f8065f8dcf41a487d46f8521c7e952e3e2096ef7cc |
| BTCUSD-2024-08-14 | evaluated |  | 2024-08-14T00:00:00Z | True | 57 | tier2 | missing_45day_session_preparation /  | a10f1b392009d45ae5bb0b7c116a110b145a9e31a994169f1a9ad1a043035d39 |
| BTCUSD-2024-08-15 | evaluated |  | 2024-08-15T00:00:00Z | True | 53 | tier2 | missing_45day_session_preparation /  | 15610ce195d5b27cba6cc32a2ed4f5e8e31fdf2fff6b6dc98d7a135689ce72fb |
| BTCUSD-2024-08-16 | evaluated |  | 2024-08-16T00:00:00Z | True | 53 | tier2 | missing_45day_session_preparation /  | d079f9bb058b4eb5f99131a0f31345d6488f3767b6e1c9143743011ef98c4da2 |
| ETHUSD-2024-08-12 | evaluated |  | 2024-08-12T00:00:00Z | True | 68 | tier2 | missing_45day_session_preparation /  | fa160d488bdacc27cdaea3fb9b9c1b80228ba45ad7469213ce2d9d8e26d3ade4 |
| ETHUSD-2024-08-13 | evaluated |  | 2024-08-13T00:00:00Z | True | 66 | tier2 | missing_45day_session_preparation /  | c773d7e35996274e946bb7e5f8c35bd2bf5f7f05d757852a3c65dfcf3e37dfcc |
| ETHUSD-2024-08-14 | evaluated |  | 2024-08-14T00:00:00Z | True | 56 | tier2 | missing_45day_session_preparation /  | 2fbbf22592707394944d191a48908882166bdf3f7a79bb3185448db07f881385 |
| ETHUSD-2024-08-15 | evaluated |  | 2024-08-15T00:00:00Z | True | 52 | tier2 | missing_45day_session_preparation /  | 01ce58c4051b915b3bfe5e50f311fd5b871ab091c149de6c84998c1bbf4cc701 |
| ETHUSD-2024-08-16 | evaluated |  | 2024-08-16T00:00:00Z | True | 52 | tier2 | missing_45day_session_preparation /  | 1db36b6110335e75a60ac7326a70a292b45f8a63c2f55fb36a089e62a42f18f6 |

## §11. Instructions for Gemini

Read this report with `docs/rebuild_phase5b_exact_frozen_candidate_audit_18366.json`, the Phase 5B scope/verification JSON and the exact Phase 4 specification. Verify the 86 + 10 completed exclusions,68 incompatible completed windows,2 exact new Tier 1 episodes, separate Tier 2 models, unchanged independent dates and source provenance. Do not call a Tier 2 result an exact frozen-candidate win, or a combined development result independent validation. Preserve the 73% /1:1.38 account goal, serial/paragraph references, SGT timestamps and “Time is Money” invariant.
