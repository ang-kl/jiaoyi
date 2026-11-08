# Phase 3 — Coverage decomposition and expanded pullback evaluation

№18,360 §§1–12 · 08-11-2026 09:28:48 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed>

## TLDR

**263 = 35 Tier-1 sessions + 228 Phase-2 exclusions.** The exclusions comprise 196 clean-history missing-preparation definitions, 4 clean-history missing-opening-quote definitions, 20 inherited-repeat definitions and 8 unverified histories. Phase 3 qualifies **167 additional distinct Tier-2 sessions** under a separately frozen first 30-minute range rule. These are reused development observations, not new independent evidence.

**Both parity gates passed: 159/159 legacy episodes and 35/35 Phase 2 sessions.** All results below come from the native Rust replay and verified local sources. New broker requests: 0. Live changes: 0. Account qualification: 0.

**Measured forced-four-hour result:** at the strict 48 bp threshold and 24 bp admission, Tier 2's breakout lost 4/4 trades; pullback B won 0/3 with mean -0.8387R. Macro C entered 0 trades; its directional filter rejected 3 losing B trades. Avoiding these observed losses is useful development evidence, but zero trades is neither a win rate nor a profitable strategy.

At the 28 bp threshold and 3.74 bp admission, Tier-2 C won **6/8**, mean **+0.2800R**, under the same assumed cost. Repricing those exact paths at 24 bp produced **0/8**, mean **-0.9406R**. The favourable low-cost result does not survive the registered stress cost. Tier-1 strict-threshold lower-cost admission looks stronger (C: 7/7 stress-priced wins), but those are reused full-M30 sessions and cost-dependent geometry, not independent confirmation.

All 7 of those stronger Tier-1 C trades lie on just 5 dates, 2025-04-07 through 2025-04-11. Correlated oil/index trades in one selected macro week are not independent regime replications. For the strict-threshold Tier-2 B comparison, all 3 positions remain unresolved at four hours before the separate forced-close calculation; continuation resolves 2 and leaves 1 unresolved.

**Progress toward the goal:** the original 73% / 1:1.38 target applies to each strategy/account over its latest 20 trades. This research adds **0 qualifying account trades and 0 independent validation trades**. It improves coverage and identifies conditional behaviour; no target achievement or production promotion is claimed. The interim 50% research target does not override costs, representativeness or evidence gates.

At 24 bp admission, a 28–48 bp impulse normally fails the full round-trip reserve before entry. The lower admission rows deliberately expose those trades, then price the **same fills and geometry** at 24 bp stress. A lower-cost sensitivity win is not a verified broker net win.

## §1. Exact denominator and exclusions

| Primary exclusion reason (mutually exclusive) | Definitions |
| --- | --- |
| inherited_repeat_m30_interval | 20 |
| missing_fresh_open_quote | 4 |
| missing_m30_preparation_for_symbol_or_cohort | 196 |
| unverified_receipt_history | 8 |

| Overlapping reason (do not sum) | Definitions |
| --- | --- |
| failed_m30_identity_or_truncation_overlap | 24 |
| inherited_repeat_m30_interval | 20 |
| missing_fresh_open_quote | 4 |
| missing_m30_preparation_for_symbol_or_cohort | 224 |
| missing_opening_M30 | 12 |
| unverified_receipt_history | 8 |

| Symbol | Core definitions | Tier 1 | Tier 2 | Excluded or alias |
| --- | --- | --- | --- | --- |
| SpotCrude | 62 | 8 | 30 | 24 |
| SpotBrent | 62 | 8 | 30 | 24 |
| US500 | 51 | 9 | 42 | 0 |
| NAS100 | 7 | 0 | 7 | 0 |
| XAUUSD | 42 | 10 | 32 | 0 |
| XAGUSD | 7 | 0 | 6 | 1 |
| BTCUSD | 30 | 0 | 18 | 12 |
| ETHUSD | 2 | 0 | 2 | 0 |

The 6 quarantined receipts comprise 5 canonical episodes and 1 duplicate; 4 canonical BTC episodes are core, 1 NatGas episode is outside core. They were already outside 263. The two failed US500 24 January 2024 wrappers represent one failed tick episode, also outside 263. They cannot be reassigned into the 228 without double counting.

The 24 retained failed M30 receipts cover 8 intervals each for SpotCrude,SpotBrent andUS500. Their files say `bar_identity_or_truncation_unverified`; the specific count500 root cause is **not established by those receipts alone**. The JSON records interval overlaps per affected definition, including overlaps subsequently superseded by repeat requests. Two raw BTC M30 responses are not qualified decoded bars. No failed source is reclassified as a successful zero-trade replay.

Among 200 excluded definitions marked clean_first_read, qualification additionally requires the native 4h source interval, fresh open and 30-minute boundary quotes and both tick sides near the end. Duplicate account/symbol/date definitions are collapsed before outcomes, with Tier 1 priority and then longest interval/SHA256. Full exclusions and aliases are retained in §10 and JSON. Clean collection provenance does not prove a complete executable quote path.

## §2. Frozen method and costs

Tier 1 preserves Phase 2 historical 14-session sigma bands and full 45-calendar-day coded-session preparation. Tier 2 uses the maximum absolute midpoint excursion from the opening midpoint during the first 30minutes, frozen before any decision. That symmetric radius is a causal opening-range proxy, **not sigma**. Its results stay separate and cannot establish that the original sigma model works with missing data.

At half-hour marks, both tiers require a fresh decision and a strictly later fresh executable quote within 5 seconds. Tier 2 confirms bid above the upper band or ask below the lower band. Both retain the full-cost durable veto and registered 28/48 bp risk floor at decision and dispatch. The first admitted impulse is fixed; there is no retry after a failed retest or macro veto.

Arm A enters that impulse. Arm B waits for a fixed 50% retracement to the opening midpoint and rejects a first touch beyond the origin. Its structuralR includes the registered admission reserve, so changing admission cost changes both eligibility and pullback risk geometry. Arm C uses the identical B trade only when fixed 5/20-session trends align. Tier 2 computes those drifts from the exact causal endpoint bars, records interior gaps, and never substitutes a shorter lookback. A same-date clean peer, when available, must not conflict; missing peer context blocks C. An absent peer is disclosed and allowed by the conditional rule. This is technical price context, not measured fundamental or institutional information.

Every entry uses the unchanged two-stage exit engine: −1R initial stop, entry-price stop proposal at +1R applied only on a later quote, gross 1.38R target. Entry-price breakeven is usually a net loss after costs. Strict bracket closes, separate forced 4h close and conditional consecutive-M30 continuation remain separate. Continuation stops at a cache gap, ambiguous OHLC path or 72 elapsed-hour limit; short ask bars use the disclosed frozen entry-spread proxy.

Recorded spread enters executable bid/ask prices once. Extra reserve is `(entry+exit)*bps/10000/R`. Same-account demo entry benchmarks remain retrospective sensitivity only, paired with assumed exit costs; no account pooling or missing-as-zero substitution. Commission, swap, conversion, historical contracts and true round-trip slippage remain unverified.

## §3. Main comparison — forced four-hour closes

The last two columns price the same paths at their admission assumption. The stress columns always use 24 bp/side. No-entry and unresolved outcomes are retained in their own denominators. Tier/threshold/cost variants are overlapping trials, not independent extra trades.

| Tier / floor / admission | Arm | Sessions | Entries | 24bp wins/closes | Unresolved | 24bp meanR | Admission-cost wins/closes | Admission-cost meanR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | 35 | 23 | 1/23 | 0 | -1.50123 | 6/23 | -0.315023 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | 35 | 14 | 6/14 | 0 | -0.884721 | 8/14 | 0.178625 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | 35 | 9 | 4/9 | 0 | -0.700351 | 6/9 | 0.360278 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | 35 | 3 | 0/3 | 0 | -1.06404 | 0/3 | -1.06404 |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | 35 | 2 | 2/2 | 0 | 0.653291 | 2/2 | 0.653291 |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | 35 | 2 | 2/2 | 0 | 0.653291 | 2/2 | 0.653291 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | 35 | 23 | 1/23 | 0 | -1.50123 | 5/23 | -0.408842 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | 35 | 14 | 6/14 | 0 | -0.714451 | 8/14 | 0.173988 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | 35 | 9 | 4/9 | 0 | -0.485991 | 6/9 | 0.400833 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | 35 | 15 | 2/15 | 0 | -1.15212 | 2/15 | -0.392787 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | 35 | 9 | 8/9 | 0 | 0.171599 | 8/9 | 1.01895 |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | 35 | 7 | 7/7 | 0 | 0.361316 | 7/7 | 1.27828 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | 35 | 3 | 0/3 | 0 | -1.06404 | 0/3 | -1.06404 |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | 35 | 2 | 2/2 | 0 | 0.653291 | 2/2 | 0.653291 |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | 35 | 2 | 2/2 | 0 | 0.653291 | 2/2 | 0.653291 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | 35 | 15 | 2/15 | 0 | -1.15212 | 2/15 | -0.452843 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | 35 | 9 | 8/9 | 0 | 0.229211 | 8/9 | 0.95719 |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | 35 | 7 | 7/7 | 0 | 0.419593 | 7/7 | 1.20463 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | 167 | 43 | 3/43 | 0 | -1.43124 | 17/43 | -0.235313 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | 167 | 22 | 2/22 | 0 | -1.24928 | 10/22 | 0.06548 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | 167 | 8 | 0/8 | 0 | -1.10012 | 6/8 | 0.382329 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | 167 | 4 | 0/4 | 0 | -1.46086 | 0/4 | -1.46086 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | 167 | 3 | 0/3 | 0 | -0.838747 | 0/3 | -0.838747 |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | 167 | 0 | 0/0 | 0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | 167 | 43 | 3/43 | 0 | -1.43124 | 15/43 | -0.3299 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | 167 | 22 | 1/22 | 0 | -1.23968 | 8/22 | -0.145168 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | 167 | 8 | 0/8 | 0 | -0.940631 | 6/8 | 0.279986 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | 167 | 6 | 1/6 | 0 | -1.22661 | 2/6 | -0.43932 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | 167 | 4 | 0/4 | 0 | -1.95047 | 0/4 | -0.744891 |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | 167 | 0 | 0/0 | 0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | 167 | 4 | 0/4 | 0 | -1.46086 | 0/4 | -1.46086 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | 167 | 3 | 0/3 | 0 | -0.838747 | 0/3 | -0.838747 |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | 167 | 0 | 0/0 | 0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | 167 | 6 | 1/6 | 0 | -1.22661 | 2/6 | -0.501588 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | 167 | 4 | 0/4 | 0 | -1.80137 | 0/4 | -0.788873 |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | 167 | 0 | 0/0 | 0 | Unavailable | 0/0 | Unavailable |

## §4. Moderate 28–48 bp subgroup

This subgroup uses the admitted impulse risk divided by its fill price, before pullback resizing. A 28 bp rule may change the first admitted impulse and thus replace a later 48 bp entry; it is not necessarily a nested superset of the 48 bp trades. Costs remain applied even when larger than the eventual price move.

| Cell | Arm | Moderate impulses | Entries | 24bp wins/closes | 24bp meanR | Admission-cost wins/closes | Admission-cost meanR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | 19 | 19 | 0/19 | -1.67558 | 5/19 | -0.372692 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | 19 | 11 | 3/11 | -1.31705 | 5/11 | -0.133743 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | 19 | 7 | 2/7 | -1.07521 | 4/7 | 0.0869977 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | 19 | 19 | 0/19 | -1.67558 | 4/19 | -0.475738 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | 19 | 11 | 3/11 | -1.10978 | 5/11 | -0.127717 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | 19 | 7 | 2/7 | -0.811795 | 4/7 | 0.153133 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | 37 | 37 | 2/37 | -1.46442 | 15/37 | -0.202231 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | 37 | 18 | 2/18 | -1.09346 | 10/18 | 0.245562 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | 37 | 8 | 0/8 | -1.10012 | 6/8 | 0.382329 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | 37 | 37 | 2/37 | -1.46442 | 13/37 | -0.302059 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | 37 | 18 | 1/18 | -1.11486 | 8/18 | -0.00212293 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | 37 | 8 | 0/8 | -0.940631 | 6/8 | 0.279986 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | 0 | 0 | 0/0 | Unavailable | 0/0 | Unavailable |

## §5. Does Arm C remove losing B trades?

| Cell | Evaluation bp/side | B fate under C | Entries | Priced closes | B wins | B losses | Unresolved | B meanR |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=28 / admission=2 | 24 | own_trend_veto | 5 | 5 | 2 | 3 | 0 | -1.21659 |
| tier1 / floor=28 / admission=2 | 24 | retained | 9 | 9 | 4 | 5 | 0 | -0.700351 |
| tier1 / floor=28 / admission=2 | 2 | own_trend_veto | 5 | 5 | 2 | 3 | 0 | -0.148351 |
| tier1 / floor=28 / admission=2 | 2 | retained | 9 | 9 | 6 | 3 | 0 | 0.360278 |
| tier1 / floor=28 / admission=24 | 24 | retained | 2 | 2 | 2 | 0 | 0 | 0.653291 |
| tier1 / floor=28 / admission=3.74 | 24 | own_trend_veto | 5 | 5 | 2 | 3 | 0 | -1.12568 |
| tier1 / floor=28 / admission=3.74 | 24 | retained | 9 | 9 | 4 | 5 | 0 | -0.485991 |
| tier1 / floor=28 / admission=3.74 | 3.74 | own_trend_veto | 5 | 5 | 2 | 3 | 0 | -0.234332 |
| tier1 / floor=28 / admission=3.74 | 3.74 | retained | 9 | 9 | 6 | 3 | 0 | 0.400833 |
| tier1 / floor=48 / admission=2 | 24 | own_trend_veto | 2 | 2 | 1 | 1 | 0 | -0.49241 |
| tier1 / floor=48 / admission=2 | 24 | retained | 7 | 7 | 7 | 0 | 0 | 0.361316 |
| tier1 / floor=48 / admission=2 | 2 | own_trend_veto | 2 | 2 | 1 | 1 | 0 | 0.111273 |
| tier1 / floor=48 / admission=2 | 2 | retained | 7 | 7 | 7 | 0 | 0 | 1.27828 |
| tier1 / floor=48 / admission=24 | 24 | retained | 2 | 2 | 2 | 0 | 0 | 0.653291 |
| tier1 / floor=48 / admission=3.74 | 24 | own_trend_veto | 2 | 2 | 1 | 1 | 0 | -0.437125 |
| tier1 / floor=48 / admission=3.74 | 24 | retained | 7 | 7 | 7 | 0 | 0 | 0.419593 |
| tier1 / floor=48 / admission=3.74 | 3.74 | own_trend_veto | 2 | 2 | 1 | 1 | 0 | 0.0911507 |
| tier1 / floor=48 / admission=3.74 | 3.74 | retained | 7 | 7 | 7 | 0 | 0 | 1.20463 |
| tier2 / floor=28 / admission=2 | 24 | own_context_unavailable | 3 | 3 | 0 | 3 | 0 | -1.21966 |
| tier2 / floor=28 / admission=2 | 24 | own_trend_veto | 11 | 11 | 2 | 9 | 0 | -1.36584 |
| tier2 / floor=28 / admission=2 | 24 | retained | 8 | 8 | 0 | 8 | 0 | -1.10012 |
| tier2 / floor=28 / admission=2 | 2 | own_context_unavailable | 3 | 3 | 1 | 2 | 0 | 0.115461 |
| tier2 / floor=28 / admission=2 | 2 | own_trend_veto | 11 | 11 | 3 | 8 | 0 | -0.178587 |
| tier2 / floor=28 / admission=2 | 2 | retained | 8 | 8 | 6 | 2 | 0 | 0.382329 |
| tier2 / floor=28 / admission=24 | 24 | own_trend_veto | 3 | 3 | 0 | 3 | 0 | -0.838747 |
| tier2 / floor=28 / admission=3.74 | 24 | own_context_unavailable | 3 | 3 | 0 | 3 | 0 | -1.10933 |
| tier2 / floor=28 / admission=3.74 | 24 | own_trend_veto | 11 | 11 | 1 | 10 | 0 | -1.49272 |
| tier2 / floor=28 / admission=3.74 | 24 | retained | 8 | 8 | 0 | 8 | 0 | -0.940631 |
| tier2 / floor=28 / admission=3.74 | 3.74 | own_context_unavailable | 3 | 3 | 1 | 2 | 0 | 0.000491966 |
| tier2 / floor=28 / admission=3.74 | 3.74 | own_trend_veto | 11 | 11 | 1 | 10 | 0 | -0.494097 |
| tier2 / floor=28 / admission=3.74 | 3.74 | retained | 8 | 8 | 6 | 2 | 0 | 0.279986 |
| tier2 / floor=48 / admission=2 | 24 | own_trend_veto | 4 | 4 | 0 | 4 | 0 | -1.95047 |
| tier2 / floor=48 / admission=2 | 2 | own_trend_veto | 4 | 4 | 0 | 4 | 0 | -0.744891 |
| tier2 / floor=48 / admission=24 | 24 | own_trend_veto | 3 | 3 | 0 | 3 | 0 | -0.838747 |
| tier2 / floor=48 / admission=3.74 | 24 | own_trend_veto | 4 | 4 | 0 | 4 | 0 | -1.80137 |
| tier2 / floor=48 / admission=3.74 | 3.74 | own_trend_veto | 4 | 4 | 0 | 4 | 0 | -0.788873 |

`own_trend_veto` and `peer_trend_veto` are directional decisions. `own_context_unavailable` and `peer_context_unavailable` are data abstentions and cannot support an incremental-alpha claim. Evaluate rejected winners as well as losers, retained return and remaining sample size. Conditional results on reused, correlated regimes do not establish a causal benefit or future win rate. Paired episode IDs and strict/forced/continuation outcomes for every cost track are in the JSON.

## §6. Full resolution and pricing matrix

| Cell | Arm | Resolution | Evaluation bp/side | Entries | Closes | Wins | Losses | Unresolved | Win rate | MeanR | PF/status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 24 | 23 | 17 | 1 | 16 | 6 | 5.88% | -1.61435 | 0.0178878 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 2 | 23 | 17 | 4 | 13 | 6 | 23.53% | -0.390868 | 0.431522 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 3.74 | 23 | 17 | 4 | 13 | 6 | 23.53% | -0.487634 | 0.358042 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 24 | 23 | 23 | 1 | 22 | 0 | 4.35% | -1.50123 | 0.01427 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 2 | 23 | 23 | 6 | 17 | 0 | 26.09% | -0.315023 | 0.423054 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 3.74 | 23 | 23 | 5 | 18 | 0 | 21.74% | -0.408842 | 0.335392 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 24 | 23 | 18 | 2 | 16 | 5 | 11.11% | -1.50821 | 0.0284879 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 2 | 23 | 18 | 5 | 13 | 5 | 27.78% | -0.297492 | 0.541877 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 3.74 | 23 | 18 | 5 | 13 | 5 | 27.78% | -0.393249 | 0.451845 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 24 | 14 | 13 | 6 | 7 | 1 | 46.15% | -0.931563 | 0.25313 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 2 | 14 | 13 | 7 | 6 | 1 | 53.85% | 0.175928 | 1.33057 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 3.74 | 14 | 13 | 7 | 6 | 1 | 53.85% | 0.0883356 | 1.15005 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 24 | 14 | 14 | 6 | 8 | 0 | 42.86% | -0.884721 | 0.248897 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 2 | 14 | 14 | 8 | 6 | 0 | 57.14% | 0.178625 | 1.36146 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 3.74 | 14 | 14 | 8 | 6 | 0 | 57.14% | 0.0945237 | 1.17292 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 24 | 14 | 14 | 6 | 8 | 0 | 42.86% | -0.974807 | 0.231214 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 2 | 14 | 14 | 7 | 7 | 0 | 50.00% | 0.0887371 | 1.15601 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 3.74 | 14 | 14 | 7 | 7 | 0 | 50.00% | 0.00462042 | 1.0074 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 24 | 9 | 8 | 4 | 4 | 1 | 50.00% | -0.753423 | 0.298172 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 2 | 9 | 8 | 5 | 3 | 1 | 62.50% | 0.378602 | 1.86146 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 3.74 | 9 | 8 | 5 | 3 | 1 | 62.50% | 0.28907 | 1.59049 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 24 | 9 | 9 | 4 | 5 | 0 | 44.44% | -0.700351 | 0.288895 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 2 | 9 | 9 | 6 | 3 | 0 | 66.67% | 0.360278 | 1.92224 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 3.74 | 9 | 9 | 6 | 3 | 0 | 66.67% | 0.276392 | 1.63517 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 24 | 9 | 9 | 4 | 5 | 0 | 44.44% | -0.840483 | 0.25291 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 2 | 9 | 9 | 5 | 4 | 0 | 55.56% | 0.220452 | 1.43504 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 3.74 | 9 | 9 | 5 | 4 | 0 | 55.56% | 0.136542 | 1.24578 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 24 | 3 | 3 | 0 | 3 | 0 | 0.00% | -1.06404 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 2 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.490641 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 3.74 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.535992 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 24 | 23 | 17 | 1 | 16 | 6 | 5.88% | -1.61435 | 0.0178878 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 2 | 23 | 17 | 4 | 13 | 6 | 23.53% | -0.390868 | 0.431522 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 3.74 | 23 | 17 | 4 | 13 | 6 | 23.53% | -0.487634 | 0.358042 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 24 | 23 | 23 | 1 | 22 | 0 | 4.35% | -1.50123 | 0.01427 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 2 | 23 | 23 | 6 | 17 | 0 | 26.09% | -0.315023 | 0.423054 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 3.74 | 23 | 23 | 5 | 18 | 0 | 21.74% | -0.408842 | 0.335392 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 24 | 23 | 18 | 2 | 16 | 5 | 11.11% | -1.50821 | 0.0284879 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 2 | 23 | 18 | 5 | 13 | 5 | 27.78% | -0.297492 | 0.541877 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 3.74 | 23 | 18 | 5 | 13 | 5 | 27.78% | -0.393249 | 0.451845 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 24 | 14 | 13 | 6 | 7 | 1 | 46.15% | -0.749003 | 0.306993 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 2 | 14 | 13 | 7 | 6 | 1 | 53.85% | 0.253733 | 1.55885 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 3.74 | 14 | 13 | 7 | 6 | 1 | 53.85% | 0.174425 | 1.34741 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 24 | 14 | 14 | 6 | 8 | 0 | 42.86% | -0.714451 | 0.301304 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 2 | 14 | 14 | 8 | 6 | 0 | 57.14% | 0.250291 | 1.59368 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 3.74 | 14 | 14 | 8 | 6 | 0 | 57.14% | 0.173988 | 1.3732 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 24 | 14 | 14 | 6 | 8 | 0 | 42.86% | -0.803833 | 0.277084 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 2 | 14 | 14 | 7 | 7 | 0 | 50.00% | 0.161105 | 1.32474 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 3.74 | 14 | 14 | 7 | 7 | 0 | 50.00% | 0.0847875 | 1.15603 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 24 | 9 | 8 | 4 | 4 | 1 | 50.00% | -0.513581 | 0.397834 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 2 | 9 | 8 | 5 | 3 | 1 | 62.50% | 0.510929 | 2.67009 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 3.74 | 9 | 8 | 5 | 3 | 1 | 62.50% | 0.429899 | 2.23992 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 24 | 9 | 9 | 4 | 5 | 0 | 44.44% | -0.485991 | 0.382946 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 2 | 9 | 9 | 6 | 3 | 0 | 66.67% | 0.476997 | 2.75407 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 3.74 | 9 | 9 | 6 | 3 | 0 | 66.67% | 0.400833 | 2.3006 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 24 | 9 | 9 | 4 | 5 | 0 | 44.44% | -0.625029 | 0.325487 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 2 | 9 | 9 | 5 | 4 | 0 | 55.56% | 0.338264 | 1.87219 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 3.74 | 9 | 9 | 5 | 4 | 0 | 55.56% | 0.262076 | 1.61198 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 24 | 15 | 12 | 2 | 10 | 3 | 16.67% | -1.23275 | 0.0570066 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 2 | 15 | 12 | 2 | 10 | 3 | 16.67% | -0.429979 | 0.335662 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 3.74 | 15 | 12 | 2 | 10 | 3 | 16.67% | -0.493471 | 0.29447 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 24 | 15 | 15 | 2 | 13 | 0 | 13.33% | -1.15212 | 0.0492009 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 2 | 15 | 15 | 2 | 13 | 0 | 13.33% | -0.392787 | 0.30675 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 3.74 | 15 | 15 | 2 | 13 | 0 | 13.33% | -0.452843 | 0.266785 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 24 | 15 | 14 | 2 | 12 | 1 | 14.29% | -1.30744 | 0.0465807 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 2 | 15 | 14 | 2 | 12 | 1 | 14.29% | -0.520406 | 0.263529 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 3.74 | 15 | 14 | 2 | 12 | 1 | 14.29% | -0.582653 | 0.232535 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 24 | 9 | 8 | 7 | 1 | 1 | 87.50% | 0.154544 | 1.66366 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 2 | 9 | 8 | 7 | 1 | 1 | 87.50% | 1.00124 | 8.17824 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 3.74 | 9 | 8 | 7 | 1 | 1 | 87.50% | 0.934274 | 7.36129 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 24 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.171599 | 1.82901 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 2 | 9 | 9 | 8 | 1 | 0 | 88.89% | 1.01895 | 9.21832 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 3.74 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.951928 | 8.29168 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 24 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.18751 | 1.90587 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 2 | 9 | 9 | 8 | 1 | 0 | 88.89% | 1.03482 | 9.34637 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 3.74 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.967807 | 8.41331 |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 24 | 7 | 6 | 6 | 0 | 1 | 100.00% | 0.370195 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 2 | 7 | 6 | 6 | 0 | 1 | 100.00% | 1.2979 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 3.74 | 7 | 6 | 6 | 0 | 1 | 100.00% | 1.22452 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 24 | 7 | 7 | 7 | 0 | 0 | 100.00% | 0.361316 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 2 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.27828 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 3.74 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.20576 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 24 | 7 | 7 | 7 | 0 | 0 | 100.00% | 0.381773 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 2 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.29869 | no_losses |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 3.74 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.22617 | no_losses |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 24 | 3 | 3 | 0 | 3 | 0 | 0.00% | -1.06404 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 2 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.490641 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 3.74 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.535992 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.23777 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.561481 | 0 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -0.61497 | 0 |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 24 | 2 | 1 | 1 | 0 | 1 | 100.00% | 0.995041 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 2 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.34852 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 3.74 | 2 | 1 | 1 | 0 | 1 | 100.00% | 1.32056 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.653291 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.06058 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.02837 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 24 | 2 | 2 | 2 | 0 | 0 | 100.00% | 0.937333 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 2 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.344 | no_losses |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 3.74 | 2 | 2 | 2 | 0 | 0 | 100.00% | 1.31184 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 24 | 15 | 12 | 2 | 10 | 3 | 16.67% | -1.23275 | 0.0570066 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 2 | 15 | 12 | 2 | 10 | 3 | 16.67% | -0.429979 | 0.335662 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 3.74 | 15 | 12 | 2 | 10 | 3 | 16.67% | -0.493471 | 0.29447 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 24 | 15 | 15 | 2 | 13 | 0 | 13.33% | -1.15212 | 0.0492009 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 2 | 15 | 15 | 2 | 13 | 0 | 13.33% | -0.392787 | 0.30675 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 3.74 | 15 | 15 | 2 | 13 | 0 | 13.33% | -0.452843 | 0.266785 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 24 | 15 | 14 | 2 | 12 | 1 | 14.29% | -1.30744 | 0.0465807 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 2 | 15 | 14 | 2 | 12 | 1 | 14.29% | -0.520406 | 0.263529 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 3.74 | 15 | 14 | 2 | 12 | 1 | 14.29% | -0.582653 | 0.232535 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 24 | 9 | 8 | 7 | 1 | 1 | 87.50% | 0.221764 | 2.00184 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 2 | 9 | 8 | 7 | 1 | 1 | 87.50% | 1.01117 | 8.58001 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 3.74 | 9 | 8 | 7 | 1 | 1 | 87.50% | 0.948734 | 7.75947 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 24 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.229211 | 2.16491 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 2 | 9 | 9 | 8 | 1 | 0 | 88.89% | 1.01971 | 9.59955 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 3.74 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.95719 | 8.67219 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 24 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.253741 | 2.28958 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 2 | 9 | 9 | 8 | 1 | 0 | 88.89% | 1.04419 | 9.80596 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 3.74 | 9 | 9 | 8 | 1 | 0 | 88.89% | 0.98167 | 8.8684 |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 24 | 7 | 6 | 6 | 0 | 1 | 100.00% | 0.441394 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 2 | 7 | 6 | 6 | 0 | 1 | 100.00% | 1.30272 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 3.74 | 7 | 6 | 6 | 0 | 1 | 100.00% | 1.23459 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 24 | 7 | 7 | 7 | 0 | 0 | 100.00% | 0.419593 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 2 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.27205 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 3.74 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.20463 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 24 | 7 | 7 | 7 | 0 | 0 | 100.00% | 0.451131 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 2 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.30352 | no_losses |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 3.74 | 7 | 7 | 7 | 0 | 0 | 100.00% | 1.2361 | no_losses |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 24 | 43 | 29 | 3 | 26 | 14 | 10.34% | -1.63511 | 0.0153359 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 2 | 43 | 29 | 7 | 22 | 14 | 24.14% | -0.444056 | 0.407829 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | strict | 3.74 | 43 | 29 | 7 | 22 | 14 | 24.14% | -0.538257 | 0.343574 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 24 | 43 | 43 | 3 | 40 | 0 | 6.98% | -1.43124 | 0.0118578 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 2 | 43 | 43 | 17 | 26 | 0 | 39.53% | -0.235313 | 0.553433 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | forced | 3.74 | 43 | 43 | 15 | 28 | 0 | 34.88% | -0.3299 | 0.438782 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 24 | 43 | 39 | 5 | 34 | 4 | 12.82% | -1.54673 | 0.020123 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 2 | 43 | 39 | 10 | 29 | 4 | 25.64% | -0.354461 | 0.478919 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | continuation | 3.74 | 43 | 39 | 10 | 29 | 4 | 25.64% | -0.448758 | 0.401525 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 24 | 22 | 11 | 2 | 9 | 11 | 18.18% | -1.02262 | 0.0428006 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 2 | 22 | 11 | 6 | 5 | 11 | 54.55% | 0.262107 | 1.61517 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | strict | 3.74 | 22 | 11 | 6 | 5 | 11 | 54.55% | 0.160496 | 1.34097 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 24 | 22 | 22 | 2 | 20 | 0 | 9.09% | -1.24928 | 0.017972 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 2 | 22 | 22 | 10 | 12 | 0 | 45.45% | 0.06548 | 1.17806 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | forced | 3.74 | 22 | 22 | 10 | 12 | 0 | 45.45% | -0.0385057 | 0.908878 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 24 | 22 | 18 | 3 | 15 | 4 | 16.67% | -1.16097 | 0.0346218 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 2 | 22 | 18 | 9 | 9 | 4 | 50.00% | 0.122918 | 1.24233 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | continuation | 3.74 | 22 | 18 | 9 | 9 | 4 | 50.00% | 0.0213739 | 1.03845 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 24 | 8 | 4 | 0 | 4 | 4 | 0.00% | -0.846264 | 0 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 2 | 8 | 4 | 3 | 1 | 4 | 75.00% | 0.640366 | 3.16287 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | strict | 3.74 | 8 | 4 | 3 | 1 | 4 | 75.00% | 0.522787 | 2.59928 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 24 | 8 | 8 | 0 | 8 | 0 | 0.00% | -1.10012 | 0 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 2 | 8 | 8 | 6 | 2 | 0 | 75.00% | 0.382329 | 2.76046 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | forced | 3.74 | 8 | 8 | 6 | 2 | 0 | 75.00% | 0.265081 | 2.05826 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 24 | 8 | 7 | 1 | 6 | 1 | 14.29% | -0.530147 | 0.0622783 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 2 | 8 | 7 | 6 | 1 | 1 | 85.71% | 0.904907 | 6.34866 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | continuation | 3.74 | 8 | 7 | 6 | 1 | 1 | 85.71% | 0.791407 | 5.23679 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 24 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.86768 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 2 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.08263 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | strict | 3.74 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.14472 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.46086 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 2 | 4 | 4 | 1 | 3 | 0 | 25.00% | -0.711021 | 0.124332 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | forced | 3.74 | 4 | 4 | 1 | 3 | 0 | 25.00% | -0.770326 | 0.102751 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.57618 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.826594 | 0 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | continuation | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.88588 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 24 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 2 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | strict | 3.74 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 24 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.838747 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 2 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.302852 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | forced | 3.74 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.345236 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.56823 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.04735 | 0 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | continuation | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.08855 | 0 |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | strict | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | forced | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | continuation | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 24 | 43 | 29 | 3 | 26 | 14 | 10.34% | -1.63511 | 0.0153359 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 2 | 43 | 29 | 7 | 22 | 14 | 24.14% | -0.444056 | 0.407829 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | strict | 3.74 | 43 | 29 | 7 | 22 | 14 | 24.14% | -0.538257 | 0.343574 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 24 | 43 | 43 | 3 | 40 | 0 | 6.98% | -1.43124 | 0.0118578 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 2 | 43 | 43 | 17 | 26 | 0 | 39.53% | -0.235313 | 0.553433 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | forced | 3.74 | 43 | 43 | 15 | 28 | 0 | 34.88% | -0.3299 | 0.438782 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 24 | 43 | 39 | 5 | 34 | 4 | 12.82% | -1.54673 | 0.020123 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 2 | 43 | 39 | 10 | 29 | 4 | 25.64% | -0.354461 | 0.478919 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | continuation | 3.74 | 43 | 39 | 10 | 29 | 4 | 25.64% | -0.448758 | 0.401525 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 24 | 22 | 11 | 1 | 10 | 11 | 9.09% | -1.14496 | 0.0388398 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 2 | 22 | 11 | 4 | 7 | 11 | 36.36% | 0.0195031 | 1.0439 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | strict | 3.74 | 22 | 11 | 4 | 7 | 11 | 36.36% | -0.0725956 | 0.855462 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 24 | 22 | 22 | 1 | 21 | 0 | 4.55% | -1.23968 | 0.0183191 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 2 | 22 | 22 | 8 | 14 | 0 | 36.36% | -0.0511679 | 0.858819 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | forced | 3.74 | 22 | 22 | 8 | 14 | 0 | 36.36% | -0.145168 | 0.655071 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 24 | 22 | 18 | 3 | 15 | 4 | 16.67% | -1.26165 | 0.0388788 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 2 | 22 | 18 | 6 | 12 | 4 | 33.33% | -0.0978925 | 0.813144 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 3.74 | 22 | 18 | 6 | 12 | 4 | 33.33% | -0.189935 | 0.675683 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 24 | 8 | 4 | 0 | 4 | 4 | 0.00% | -0.675294 | 0 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 2 | 8 | 4 | 3 | 1 | 4 | 75.00% | 0.655371 | 3.24064 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 3.74 | 8 | 4 | 3 | 1 | 4 | 75.00% | 0.550127 | 2.71958 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 24 | 8 | 8 | 0 | 8 | 0 | 0.00% | -0.940631 | 0 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 2 | 8 | 8 | 6 | 2 | 0 | 75.00% | 0.384817 | 2.86181 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 3.74 | 8 | 8 | 6 | 2 | 0 | 75.00% | 0.279986 | 2.18611 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 24 | 8 | 7 | 2 | 5 | 1 | 28.57% | -0.566436 | 0.0936511 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 2 | 8 | 7 | 5 | 2 | 1 | 71.43% | 0.721307 | 4.85597 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 3.74 | 8 | 7 | 5 | 2 | 1 | 71.43% | 0.619458 | 3.81485 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 24 | 6 | 5 | 1 | 4 | 1 | 20.00% | -1.42385 | 0.0592591 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 2 | 6 | 5 | 1 | 4 | 1 | 20.00% | -0.607948 | 0.299985 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | strict | 3.74 | 6 | 5 | 1 | 4 | 1 | 20.00% | -0.672478 | 0.268646 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 24 | 6 | 6 | 1 | 5 | 0 | 16.67% | -1.22661 | 0.0574344 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 2 | 6 | 6 | 2 | 4 | 0 | 33.33% | -0.43932 | 0.39298 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | forced | 3.74 | 6 | 6 | 2 | 4 | 0 | 33.33% | -0.501588 | 0.345397 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 24 | 6 | 6 | 1 | 5 | 0 | 16.67% | -1.30349 | 0.0542307 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 2 | 6 | 6 | 1 | 5 | 0 | 16.67% | -0.516369 | 0.295999 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | continuation | 3.74 | 6 | 6 | 1 | 5 | 0 | 16.67% | -0.578623 | 0.262405 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 24 | 4 | 1 | 0 | 1 | 3 | 0.00% | -2.44134 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 2 | 4 | 1 | 0 | 1 | 3 | 0.00% | -1.12011 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | strict | 3.74 | 4 | 1 | 0 | 1 | 3 | 0.00% | -1.22461 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.95047 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.744891 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | forced | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.840241 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -2.31605 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.10967 | 0 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | continuation | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.20509 | 0 |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | strict | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | forced | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | continuation | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 24 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.86768 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 2 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.08263 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | strict | 3.74 | 4 | 3 | 0 | 3 | 1 | 0.00% | -1.14472 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.46086 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 2 | 4 | 4 | 1 | 3 | 0 | 25.00% | -0.711021 | 0.124332 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | forced | 3.74 | 4 | 4 | 1 | 3 | 0 | 25.00% | -0.770326 | 0.102751 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.57618 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.826594 | 0 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | continuation | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.88588 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 24 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 2 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | strict | 3.74 | 3 | 0 | 0 | 0 | 3 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 24 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.838747 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 2 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.302852 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | forced | 3.74 | 3 | 3 | 0 | 3 | 0 | 0.00% | -0.345236 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 24 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.56823 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 2 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.04735 | 0 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | continuation | 3.74 | 3 | 2 | 0 | 2 | 1 | 0.00% | -1.08855 | 0 |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | strict | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | forced | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | continuation | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 24 | 6 | 5 | 1 | 4 | 1 | 20.00% | -1.42385 | 0.0592591 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 2 | 6 | 5 | 1 | 4 | 1 | 20.00% | -0.607948 | 0.299985 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | strict | 3.74 | 6 | 5 | 1 | 4 | 1 | 20.00% | -0.672478 | 0.268646 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 24 | 6 | 6 | 1 | 5 | 0 | 16.67% | -1.22661 | 0.0574344 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 2 | 6 | 6 | 2 | 4 | 0 | 33.33% | -0.43932 | 0.39298 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | forced | 3.74 | 6 | 6 | 2 | 4 | 0 | 33.33% | -0.501588 | 0.345397 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 24 | 6 | 6 | 1 | 5 | 0 | 16.67% | -1.30349 | 0.0542307 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 2 | 6 | 6 | 1 | 5 | 0 | 16.67% | -0.516369 | 0.295999 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | continuation | 3.74 | 6 | 6 | 1 | 5 | 0 | 16.67% | -0.578623 | 0.262405 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 24 | 4 | 1 | 0 | 1 | 3 | 0.00% | -2.30686 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 2 | 4 | 1 | 0 | 1 | 3 | 0.00% | -1.1089 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | strict | 3.74 | 4 | 1 | 0 | 1 | 3 | 0.00% | -1.20365 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.80137 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.701917 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | forced | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -0.788873 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 24 | 4 | 4 | 0 | 4 | 0 | 0.00% | -2.20036 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 2 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.10003 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | continuation | 3.74 | 4 | 4 | 0 | 4 | 0 | 0.00% | -1.18706 | 0 |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | strict | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | forced | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 24 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 2 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | continuation | 3.74 | 0 | 0 | 0 | 0 | 0 | Unavailable | Unavailable | unavailable |

## §7. Demo-entry benchmark sensitivities

| Cell | Arm | Benchmark + assumed exit | Priced closes | Cost unavailable | Wins/closes | MeanR |
| --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 14 | 9 | 3/14 | -0.442522 |
| tier1 / floor=28 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 14 | 9 | 3/14 | -0.490847 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 9 | 5 | 4/9 | -0.130107 |
| tier1 / floor=28 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 9 | 5 | 4/9 | -0.174086 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 7 | 2 | 4/7 | 0.134845 |
| tier1 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 7 | 2 | 4/7 | 0.0912599 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 1 | 2 | 0/1 | -0.0226454 |
| tier1 / floor=28 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 1 | 2 | 0/1 | -0.0514064 |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 1 | 1 | 1/1 | 0.800308 |
| tier1 / floor=28 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 1 | 1 | 1/1 | 0.782145 |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 1 | 1 | 1/1 | 0.800308 |
| tier1 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 1 | 1 | 1/1 | 0.782145 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 14 | 9 | 3/14 | -0.442522 |
| tier1 / floor=28 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 14 | 9 | 3/14 | -0.490847 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 9 | 5 | 5/9 | 0.130333 |
| tier1 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 9 | 5 | 5/9 | 0.0904939 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 7 | 2 | 5/7 | 0.477421 |
| tier1 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 7 | 2 | 5/7 | 0.438068 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 10 | 5 | 1/10 | -0.328522 |
| tier1 / floor=48 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 10 | 5 | 1/10 | -0.361034 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 5 | 4 | 5/5 | 1.31839 |
| tier1 / floor=48 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 5 | 4 | 5/5 | 1.28049 |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 5 | 2 | 5/5 | 1.31839 |
| tier1 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 5 | 2 | 5/5 | 1.28049 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 1 | 2 | 0/1 | -0.0226454 |
| tier1 / floor=48 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 1 | 2 | 0/1 | -0.0514064 |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 1 | 1 | 1/1 | 0.800308 |
| tier1 / floor=48 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 1 | 1 | 1/1 | 0.782145 |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 1 | 1 | 1/1 | 0.800308 |
| tier1 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 1 | 1 | 1/1 | 0.782145 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 10 | 5 | 1/10 | -0.328522 |
| tier1 / floor=48 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 10 | 5 | 1/10 | -0.361034 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 5 | 4 | 5/5 | 1.30378 |
| tier1 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 5 | 4 | 5/5 | 1.26863 |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 5 | 2 | 5/5 | 1.30378 |
| tier1 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 5 | 2 | 5/5 | 1.26863 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 16 | 27 | 3/16 | -0.422833 |
| tier2 / floor=28 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 16 | 27 | 3/16 | -0.470315 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 10 | 12 | 4/10 | 0.00134541 |
| tier2 / floor=28 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 10 | 12 | 4/10 | -0.044044 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 2 | 6 | 2/2 | 0.278952 |
| tier2 / floor=28 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 2 | 6 | 2/2 | 0.232104 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 1 | 3 | 0/1 | -1.05409 |
| tier2 / floor=28 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 1 | 3 | 0/1 | -1.08535 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 1 | 2 | 0/1 | -0.259621 |
| tier2 / floor=28 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 1 | 2 | 0/1 | -0.279565 |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=28 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 16 | 27 | 3/16 | -0.422833 |
| tier2 / floor=28 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 16 | 27 | 3/16 | -0.470315 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 10 | 12 | 3/10 | -0.123471 |
| tier2 / floor=28 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 10 | 12 | 3/10 | -0.165043 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 2 | 6 | 2/2 | 0.253044 |
| tier2 / floor=28 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 2 | 6 | 2/2 | 0.210318 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 3 | 3 | 1/3 | -0.290548 |
| tier2 / floor=48 / admission=2 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 3 | 3 | 1/3 | -0.323768 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 2 | 2 | 0/2 | -0.637345 |
| tier2 / floor=48 / admission=2 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 2 | 2 | 0/2 | -0.683984 |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=48 / admission=2 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 1 | 3 | 0/1 | -1.05409 |
| tier2 / floor=48 / admission=24 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 1 | 3 | 0/1 | -1.08535 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 1 | 2 | 0/1 | -0.259621 |
| tier2 / floor=48 / admission=24 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 1 | 2 | 0/1 | -0.279565 |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=48 / admission=24 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_2_exit | 3 | 3 | 1/3 | -0.290548 |
| tier2 / floor=48 / admission=3.74 | arm_a_breakout_baseline | demo_entry_plus_3.74_exit | 3 | 3 | 1/3 | -0.323768 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_2_exit | 2 | 2 | 0/2 | -0.581192 |
| tier2 / floor=48 / admission=3.74 | arm_b_pure_pullback_retest | demo_entry_plus_3.74_exit | 2 | 2 | 0/2 | -0.623746 |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_2_exit | 0 | 0 | 0/0 | Unavailable |
| tier2 / floor=48 / admission=3.74 | arm_c_macro_filtered_pullback | demo_entry_plus_3.74_exit | 0 | 0 | 0/0 | Unavailable |

## §8. Verification and rebuild acceptance

- Plan SHA256:`9bed468064a81efbe78827f7e376e09254377c4477ac54fad1d5b798877139e9`.
- Result SHA256:`d35aae8b82889b8bc4168646828978195bd438becdb72898016b2220cbd000d8`.
- Frozen code commit:`07239c822f55d22437672da5588f88255df1e0ae`.
- 159 exact legacy comparisons; 35 exact Phase 2 input/output comparisons, repeated through the default policy wrapper.
- Independent cost arithmetic checks:3190.
- Source files hash-verified:756; protected receipts explicitly rejected:6.
- Focused tests, determinism and unchanged production-file verification:see `docs/rebuild_phase3_verification_18360.json`.
- Coverage selection was saved before new outcomes:`133ff30d9f2f9ac8ae172bb6debea8d5d848de4a7ab2184d5764f420b9fe1b1c`.

| RFC gate | Disposition | Evidence / remaining work |
| --- | --- | --- |
| G1 | Partial | Plan/code/source/calendar/contract identities linked; historical contract authority remains unknown. |
| G2 | Pass for bounded migration | 159 legacy plus35 Phase 2 outputs exactly reproduced; no live parity claim. |
| G3 | Pass for this offline experiment | Single manifest runner, finite policy matrix, offline guard, denominators retained, no arbitrary outcomes. Historical scripts retained as frozen compatibility evidence. |
| G4 | Partial | 263 definitions accounted, failures/quarantine/repeats/aliases retained. Tier 2 is explicitly partial-context; continuous executable history and full holding paths not certified. |
| G5 | Open | Coded session registry reused. Authoritative historical early closes, maintenance, holidays and contract changes still missing. |
| G6 | Open | 13 demo entry observations are not matched round trips; fees, financing, conversion and money reconciliation absent. |
| G7 | Open | No held-out round-trip cost calibration;2/3.74bp remain assumptions. |
| G8 | Partial | Existing two-stage exit engine reused; causal entry/dispatch tests and old outputs pass. Queue, broker acknowledgements, restarts and research/live parity remain unproved. |
| G9 | Development protocol only | Plan frozen before outcomes; tiers/repeats/protected dependencies separated. Historical cohort selection and previous data reuse prevent independent validation claims. |
| G10 | Not passed | Report all arms and costs; no financial qualification from this reused sample or unverified cost model. |
| G11 | Not performed | No shadow/canary broker comparison or live deployment. |
| G12 | Not demonstrated here | No per-strategy/account latest 20 goal qualification; existing live risk rules untouched. |

**Next decision:** retain the frozen measured arms as development evidence. Do not optimise parameters on these same dates or promote a high point win rate. Resolve the minimal historical session/contract and matched round-trip execution evidence, then register a truly independent replication with fixed costs and minimum sample requirements. Keep directional macro rejection separate from missing-context abstention. A matched-risk entry comparison is still required to isolate timing from the changed structural stop. Fresh unspent M30 collection or cTrader deal capture requires a later authorised phase; none occurs here.

The opening-range Tier-2 expansion did not reproduce the earlier full-M30 pullback result. Keep those hypotheses separate. The strict-threshold full-M30 pullback is a candidate for independent replication; the moderate-threshold result is highly cost-sensitive. This evidence does not justify weakening the production gate or replacing the live strategy with either variant.

## §9. Entered-trade and macro registers

| Episode | Tier/floor/admission | Symbol | Date | Arm | Side | Entry UTC | Fill | Risk price | Strict status | Forced status | Stress netR | Admission netR | Continuation |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cdb6706f8187637f3fe3352d | tier1/48/2 | SpotBrent | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:30:01.102Z | 75.28 | 0.46 | censored_tick | closed_tick | -0.872278 | -0.1524 | closed_m30_approximation |
| cdb6706f8187637f3fe3352d | tier1/48/3.74 | SpotBrent | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:30:01.102Z | 75.28 | 0.46 | censored_tick | closed_tick | -0.872278 | -0.209336 | closed_m30_approximation |
| cdb6706f8187637f3fe3352d | tier1/28/2 | SpotBrent | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:00:01.248Z | 75.2 | 0.334 | censored_tick | closed_tick | -0.961246 | 0.0296766 | closed_m30_approximation |
| cdb6706f8187637f3fe3352d | tier1/28/3.74 | SpotBrent | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:00:01.248Z | 75.2 | 0.334 | censored_tick | closed_tick | -0.961246 | -0.0486963 | closed_m30_approximation |
| 7e1f8994fecf745690603eeb | tier1/28/2 | SpotBrent | 2024-11-06 | arm_a_breakout_baseline | long | 2024-11-06T08:00:00.726Z | 75.092 | 0.225 | closed_tick | closed_tick | -2.62173 | -1.15551 | closed_tick |
| 7e1f8994fecf745690603eeb | tier1/28/2 | SpotBrent | 2024-11-06 | arm_b_pure_pullback_retest | long | 2024-11-06T08:01:37.635Z | 74.842 | 0.29 | closed_tick | closed_tick | -2.23636 | -1.10303 | closed_tick |
| 7e1f8994fecf745690603eeb | tier1/28/3.74 | SpotBrent | 2024-11-06 | arm_a_breakout_baseline | long | 2024-11-06T08:00:00.726Z | 75.092 | 0.225 | closed_tick | closed_tick | -2.62173 | -1.27148 | closed_tick |
| 7e1f8994fecf745690603eeb | tier1/28/3.74 | SpotBrent | 2024-11-06 | arm_b_pure_pullback_retest | long | 2024-11-06T08:01:37.635Z | 74.842 | 0.316 | closed_tick | closed_tick | -2.14707 | -1.18944 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/2 | SpotBrent | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T08:00:00.678Z | 63.44 | 0.206 | closed_tick | closed_tick | -2.50008 | -1.14281 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/2 | SpotBrent | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T08:02:03.276Z | 63.61 | 0.156 | closed_tick | closed_tick | -2.98533 | -1.18895 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/2 | SpotBrent | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T08:02:03.276Z | 63.61 | 0.156 | closed_tick | closed_tick | -2.98533 | -1.18895 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/3.74 | SpotBrent | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T08:00:00.678Z | 63.44 | 0.206 | closed_tick | closed_tick | -2.50008 | -1.25015 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/3.74 | SpotBrent | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T08:02:03.276Z | 63.61 | 0.178 | closed_tick | closed_tick | -2.7853 | -1.33512 | closed_tick |
| a92fb0ae4794ae5899abbb1d | tier1/28/3.74 | SpotBrent | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T08:02:03.276Z | 63.61 | 0.178 | closed_tick | closed_tick | -2.7853 | -1.33512 | closed_tick |
| f5233c2f0a824dfd2f998e89 | tier1/28/2 | SpotBrent | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T08:00:00.442Z | 64.155 | 0.217 | closed_tick | closed_tick | -2.43535 | -1.13229 | closed_tick |
| f5233c2f0a824dfd2f998e89 | tier1/28/2 | SpotBrent | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T08:44:54.362Z | 64.705 | 0.581 | censored_tick | closed_tick | -0.275773 | 0.21368 | closed_m30_approximation |
| f5233c2f0a824dfd2f998e89 | tier1/28/2 | SpotBrent | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T08:44:54.362Z | 64.705 | 0.581 | censored_tick | closed_tick | -0.275773 | 0.21368 | closed_m30_approximation |
| f5233c2f0a824dfd2f998e89 | tier1/28/3.74 | SpotBrent | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T08:00:00.442Z | 64.155 | 0.217 | closed_tick | closed_tick | -2.43535 | -1.23535 | closed_tick |
| f5233c2f0a824dfd2f998e89 | tier1/28/3.74 | SpotBrent | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T08:44:54.362Z | 64.705 | 0.604 | censored_tick | closed_tick | -0.265272 | 0.168306 | closed_m30_approximation |
| f5233c2f0a824dfd2f998e89 | tier1/28/3.74 | SpotBrent | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T08:44:54.362Z | 64.705 | 0.604 | censored_tick | closed_tick | -0.265272 | 0.168306 | closed_m30_approximation |
| c3a0d383d87130df509791fa | tier1/48/2 | SpotBrent | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T09:30:04.371Z | 61.09 | 0.296 | closed_tick | closed_tick | 0.394424 | 1.29948 | closed_tick |
| c3a0d383d87130df509791fa | tier1/48/3.74 | SpotBrent | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T09:30:04.371Z | 61.09 | 0.296 | closed_tick | closed_tick | 0.394424 | 1.2279 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/2 | SpotBrent | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T08:30:00.452Z | 60.91 | 0.21 | closed_tick | closed_tick | -2.39463 | -1.11622 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/2 | SpotBrent | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T08:55:42.821Z | 61.31 | 0.42 | closed_tick | closed_tick | 0.683581 | 1.32284 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/2 | SpotBrent | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T08:55:42.821Z | 61.31 | 0.42 | closed_tick | closed_tick | 0.683581 | 1.32284 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/3.74 | SpotBrent | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T08:30:00.452Z | 60.91 | 0.21 | closed_tick | closed_tick | -2.39463 | -1.21733 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/3.74 | SpotBrent | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T08:55:42.821Z | 61.31 | 0.441 | closed_tick | closed_tick | 0.716947 | 1.27748 | closed_tick |
| c3a0d383d87130df509791fa | tier1/28/3.74 | SpotBrent | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T08:55:42.821Z | 61.31 | 0.441 | closed_tick | closed_tick | 0.716947 | 1.27748 | closed_tick |
| 1f6ef75ca564ffa87d523f97 | tier1/48/2 | SpotBrent | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T09:00:00.677Z | 63.635 | 0.339 | closed_tick | closed_tick | -1.90638 | -1.07824 | closed_tick |
| 1f6ef75ca564ffa87d523f97 | tier1/48/3.74 | SpotBrent | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T09:00:00.677Z | 63.635 | 0.339 | closed_tick | closed_tick | -1.90638 | -1.14373 | closed_tick |
| 1f6ef75ca564ffa87d523f97 | tier1/28/2 | SpotBrent | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T08:00:00.292Z | 64.105 | 0.299 | closed_tick | closed_tick | -1.02911 | -0.0857592 | closed_tick |
| 1f6ef75ca564ffa87d523f97 | tier1/28/3.74 | SpotBrent | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T08:00:00.292Z | 64.105 | 0.299 | closed_tick | closed_tick | -1.02911 | -0.16037 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/2 | SpotBrent | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:00.281Z | 63.79 | 0.362 | closed_tick | closed_tick | -0.845834 | -0.0704862 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/2 | SpotBrent | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:37Z | 64.17 | 0.356 | closed_tick | closed_tick | 0.520126 | 1.3102 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/2 | SpotBrent | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:37Z | 64.17 | 0.356 | closed_tick | closed_tick | 0.520126 | 1.3102 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/3.74 | SpotBrent | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:00.281Z | 63.79 | 0.362 | closed_tick | closed_tick | -0.845834 | -0.131809 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/3.74 | SpotBrent | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:37Z | 64.17 | 0.378 | closed_tick | closed_tick | 0.56941 | 1.25449 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/48/3.74 | SpotBrent | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:37Z | 64.17 | 0.378 | closed_tick | closed_tick | 0.56941 | 1.25449 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/2 | SpotBrent | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T07:30:02.257Z | 64.23 | 0.221 | closed_tick | closed_tick | -2.43826 | -1.15719 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/2 | SpotBrent | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T07:34:33.336Z | 64.36 | 0.166 | closed_tick | closed_tick | -2.88757 | -1.17939 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/2 | SpotBrent | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T07:34:33.336Z | 64.36 | 0.166 | closed_tick | closed_tick | -2.88757 | -1.17939 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/3.74 | SpotBrent | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T07:30:02.257Z | 64.23 | 0.221 | closed_tick | closed_tick | -2.43826 | -1.25851 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/3.74 | SpotBrent | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T07:34:33.336Z | 64.36 | 0.189 | closed_tick | closed_tick | -0.250273 | 1.12675 | closed_tick |
| 70e3acf0ef22a39ed2354106 | tier1/28/3.74 | SpotBrent | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T07:34:33.336Z | 64.36 | 0.189 | closed_tick | closed_tick | -0.250273 | 1.12675 | closed_tick |
| a451c5daee55ffdac04ccd9a | tier1/48/2 | SpotCrude | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:30:01.679Z | 71.679 | 0.471 | censored_tick | closed_tick | -0.89993 | -0.230691 | closed_m30_approximation |
| a451c5daee55ffdac04ccd9a | tier1/48/3.74 | SpotCrude | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:30:01.679Z | 71.679 | 0.471 | censored_tick | closed_tick | -0.89993 | -0.283622 | closed_m30_approximation |
| a451c5daee55ffdac04ccd9a | tier1/28/2 | SpotCrude | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:00:00.328Z | 71.609 | 0.343 | censored_tick | closed_tick | -1.03119 | -0.112658 | ambiguous_m30_path |
| a451c5daee55ffdac04ccd9a | tier1/28/3.74 | SpotCrude | 2024-11-04 | arm_a_breakout_baseline | long | 2024-11-04T09:00:00.328Z | 71.609 | 0.343 | censored_tick | closed_tick | -1.03119 | -0.185306 | ambiguous_m30_path |
| 910dc7f401c979f077122690 | tier1/28/2 | SpotCrude | 2024-11-06 | arm_a_breakout_baseline | long | 2024-11-06T08:00:00.723Z | 71.59 | 0.231 | closed_tick | closed_tick | -2.52405 | -1.16272 | closed_tick |
| 910dc7f401c979f077122690 | tier1/28/2 | SpotCrude | 2024-11-06 | arm_b_pure_pullback_retest | long | 2024-11-06T08:01:34.445Z | 71.35 | 0.269 | closed_tick | closed_tick | -2.27447 | -1.10961 | closed_tick |
| 910dc7f401c979f077122690 | tier1/28/3.74 | SpotCrude | 2024-11-06 | arm_a_breakout_baseline | long | 2024-11-06T08:00:00.723Z | 71.59 | 0.231 | closed_tick | closed_tick | -2.52405 | -1.27039 | closed_tick |
| 910dc7f401c979f077122690 | tier1/28/3.74 | SpotCrude | 2024-11-06 | arm_b_pure_pullback_retest | long | 2024-11-06T08:01:34.445Z | 71.35 | 0.294 | closed_tick | closed_tick | -2.21679 | -1.23556 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/2 | SpotCrude | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T09:00:00.509Z | 59.496 | 0.287 | closed_tick | closed_tick | -2.00793 | -1.09358 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/2 | SpotCrude | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T09:09:32.322Z | 59.706 | 0.229 | closed_tick | closed_tick | 0.136122 | 1.28027 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/2 | SpotCrude | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T09:09:32.322Z | 59.706 | 0.229 | closed_tick | closed_tick | 0.136122 | 1.28027 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/3.74 | SpotCrude | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T09:00:00.509Z | 59.496 | 0.287 | closed_tick | closed_tick | -2.00793 | -1.16589 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/3.74 | SpotCrude | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T09:09:32.322Z | 59.706 | 0.25 | closed_tick | closed_tick | 0.236957 | 1.20188 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/48/3.74 | SpotCrude | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T09:09:32.322Z | 59.706 | 0.25 | closed_tick | closed_tick | 0.236957 | 1.20188 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/2 | SpotCrude | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T08:00:00.259Z | 59.596 | 0.206 | closed_tick | closed_tick | -2.45917 | -1.1839 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/2 | SpotCrude | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T08:02:03.368Z | 59.766 | 0.169 | closed_tick | closed_tick | -2.70583 | -1.14758 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/2 | SpotCrude | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T08:02:03.368Z | 59.766 | 0.169 | closed_tick | closed_tick | -2.70583 | -1.14758 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/3.74 | SpotCrude | 2025-04-07 | arm_a_breakout_baseline | short | 2025-04-07T08:00:00.259Z | 59.596 | 0.206 | closed_tick | closed_tick | -2.45917 | -1.28476 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/3.74 | SpotCrude | 2025-04-07 | arm_b_pure_pullback_retest | short | 2025-04-07T08:02:03.368Z | 59.766 | 0.19 | closed_tick | closed_tick | -2.51228 | -1.23566 | closed_tick |
| 77b10f0f75d5383bb011c4b2 | tier1/28/3.74 | SpotCrude | 2025-04-07 | arm_c_macro_filtered_pullback | short | 2025-04-07T08:02:03.368Z | 59.766 | 0.19 | closed_tick | closed_tick | -2.51228 | -1.23566 | closed_tick |
| 2b977b08318352c8c79f2cbc | tier1/48/2 | SpotCrude | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T09:00:00.339Z | 60.94 | 0.293 | closed_tick | closed_tick | -2.02468 | -1.10729 | closed_tick |
| 2b977b08318352c8c79f2cbc | tier1/48/2 | SpotCrude | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T10:21:36.559Z | 61.23 | 0.315 | censored_tick | closed_tick | 0.308038 | 1.16059 | closed_m30_approximation |
| 2b977b08318352c8c79f2cbc | tier1/48/2 | SpotCrude | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T10:21:36.559Z | 61.23 | 0.315 | censored_tick | closed_tick | 0.308038 | 1.16059 | closed_m30_approximation |
| 2b977b08318352c8c79f2cbc | tier1/48/3.74 | SpotCrude | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T09:00:00.339Z | 60.94 | 0.293 | closed_tick | closed_tick | -2.02468 | -1.17985 | closed_tick |
| 2b977b08318352c8c79f2cbc | tier1/48/3.74 | SpotCrude | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T10:21:36.559Z | 61.23 | 0.336 | censored_tick | closed_tick | 0.288786 | 1.02484 | closed_m30_approximation |
| 2b977b08318352c8c79f2cbc | tier1/48/3.74 | SpotCrude | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T10:21:36.559Z | 61.23 | 0.336 | censored_tick | closed_tick | 0.288786 | 1.02484 | closed_m30_approximation |
| 2b977b08318352c8c79f2cbc | tier1/28/2 | SpotCrude | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T07:30:00.426Z | 61.06 | 0.172 | closed_tick | closed_tick | -0.316958 | 1.242 | closed_tick |
| 2b977b08318352c8c79f2cbc | tier1/28/3.74 | SpotCrude | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T07:30:00.426Z | 61.06 | 0.172 | closed_tick | closed_tick | -0.316958 | 1.1187 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/2 | SpotCrude | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T09:00:01.727Z | 57.784 | 0.299 | closed_tick | closed_tick | -1.93339 | -1.08085 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/2 | SpotCrude | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T09:09:34.220Z | 57.974 | 0.219 | closed_tick | closed_tick | 0.116219 | 1.27795 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/2 | SpotCrude | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T09:09:34.220Z | 57.974 | 0.219 | closed_tick | closed_tick | 0.116219 | 1.27795 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/3.74 | SpotCrude | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T09:00:01.727Z | 57.784 | 0.299 | closed_tick | closed_tick | -1.93339 | -1.14828 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/3.74 | SpotCrude | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T09:09:34.220Z | 57.974 | 0.239 | closed_tick | closed_tick | 0.219736 | 1.19983 | closed_tick |
| b8444e851d499207d0449237 | tier1/48/3.74 | SpotCrude | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T09:09:34.220Z | 57.974 | 0.239 | closed_tick | closed_tick | 0.219736 | 1.19983 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/2 | SpotCrude | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T08:30:00.397Z | 57.424 | 0.217 | closed_tick | closed_tick | -2.28647 | -1.11988 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/2 | SpotCrude | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T08:54:42.668Z | 57.804 | 0.379 | closed_tick | closed_tick | 0.653822 | 1.32186 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/2 | SpotCrude | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T08:54:42.668Z | 57.804 | 0.379 | closed_tick | closed_tick | 0.653822 | 1.32186 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/3.74 | SpotCrude | 2025-04-09 | arm_a_breakout_baseline | short | 2025-04-09T08:30:00.397Z | 57.424 | 0.217 | closed_tick | closed_tick | -2.28647 | -1.21214 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/3.74 | SpotCrude | 2025-04-09 | arm_b_pure_pullback_retest | short | 2025-04-09T08:54:42.668Z | 57.804 | 0.399 | closed_tick | closed_tick | 0.68888 | 1.2731 | closed_tick |
| b8444e851d499207d0449237 | tier1/28/3.74 | SpotCrude | 2025-04-09 | arm_c_macro_filtered_pullback | short | 2025-04-09T08:54:42.668Z | 57.804 | 0.399 | closed_tick | closed_tick | 0.68888 | 1.2731 | closed_tick |
| 9e626ae09c741e2673fd8c78 | tier1/48/2 | SpotCrude | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T08:00:00.507Z | 60.808 | 0.295 | closed_tick | closed_tick | -0.989418 | -0.0824515 | closed_tick |
| 9e626ae09c741e2673fd8c78 | tier1/48/3.74 | SpotCrude | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T08:00:00.507Z | 60.808 | 0.295 | closed_tick | closed_tick | -0.989418 | -0.154184 | closed_tick |
| 9e626ae09c741e2673fd8c78 | tier1/28/2 | SpotCrude | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T07:30:00.794Z | 61.548 | 0.183 | closed_tick | closed_tick | -0.228542 | 1.24826 | closed_tick |
| 9e626ae09c741e2673fd8c78 | tier1/28/3.74 | SpotCrude | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T07:30:00.794Z | 61.548 | 0.183 | closed_tick | closed_tick | -0.228542 | 1.13146 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/24 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.793407 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/24 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | censored_tick | closed_tick | 0.31154 | 0.31154 | closed_m30_approximation |
| ce5a75c11b08e52a205e5350 | tier1/48/24 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | censored_tick | closed_tick | 0.31154 | 0.31154 | closed_m30_approximation |
| ce5a75c11b08e52a205e5350 | tier1/48/2 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.0661173 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/2 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.31 | closed_tick | closed_tick | 0.444057 | 1.3026 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/2 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.31 | closed_tick | closed_tick | 0.444057 | 1.3026 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/3.74 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.123639 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/3.74 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.331 | closed_tick | closed_tick | 0.503708 | 1.24401 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/48/3.74 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.331 | closed_tick | closed_tick | 0.503708 | 1.24401 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/24 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.793407 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/24 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | censored_tick | closed_tick | 0.31154 | 0.31154 | closed_m30_approximation |
| ce5a75c11b08e52a205e5350 | tier1/28/24 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.577 | censored_tick | closed_tick | 0.31154 | 0.31154 | closed_m30_approximation |
| ce5a75c11b08e52a205e5350 | tier1/28/2 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.0661173 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/2 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.31 | closed_tick | closed_tick | 0.444057 | 1.3026 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/2 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.31 | closed_tick | closed_tick | 0.444057 | 1.3026 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/3.74 | SpotCrude | 2025-04-11 | arm_a_breakout_baseline | short | 2025-04-11T08:30:01.057Z | 60.332 | 0.365 | closed_tick | closed_tick | -0.793407 | -0.123639 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/3.74 | SpotCrude | 2025-04-11 | arm_b_pure_pullback_retest | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.331 | closed_tick | closed_tick | 0.503708 | 1.24401 | closed_tick |
| ce5a75c11b08e52a205e5350 | tier1/28/3.74 | SpotCrude | 2025-04-11 | arm_c_macro_filtered_pullback | short | 2025-04-11T10:03:36.606Z | 60.702 | 0.331 | closed_tick | closed_tick | 0.503708 | 1.24401 | closed_tick |
| 4d04e030932184cbabc7f97b | tier1/28/2 | US500 | 2024-11-05 | arm_a_breakout_baseline | long | 2024-11-05T16:30:00.216Z | 5767 | 17.1 | censored_tick | closed_tick | -1.24544 | 0.239294 | ambiguous_m30_amend_timing |
| 4d04e030932184cbabc7f97b | tier1/28/3.74 | US500 | 2024-11-05 | arm_a_breakout_baseline | long | 2024-11-05T16:30:00.216Z | 5767 | 17.1 | censored_tick | closed_tick | -1.24544 | 0.121865 | ambiguous_m30_amend_timing |
| 8d7090e5c928d51baf5f5d28 | tier1/28/2 | US500 | 2024-11-06 | arm_a_breakout_baseline | short | 2024-11-06T16:00:00.621Z | 5891 | 17.9 | closed_tick | closed_tick | -2.58211 | -1.13184 | closed_tick |
| 8d7090e5c928d51baf5f5d28 | tier1/28/2 | US500 | 2024-11-06 | arm_b_pure_pullback_retest | short | 2024-11-06T16:27:18.449Z | 5902.2 | 13.5 | closed_tick | closed_tick | -3.11581 | -1.1899 | closed_tick |
| 8d7090e5c928d51baf5f5d28 | tier1/28/3.74 | US500 | 2024-11-06 | arm_a_breakout_baseline | short | 2024-11-06T16:00:00.621Z | 5891 | 17.9 | closed_tick | closed_tick | -2.58211 | -1.24655 | closed_tick |
| 8d7090e5c928d51baf5f5d28 | tier1/28/3.74 | US500 | 2024-11-06 | arm_b_pure_pullback_retest | short | 2024-11-06T16:27:18.449Z | 5902.2 | 15.6 | closed_tick | closed_tick | -2.86344 | -1.32827 | closed_tick |
| f35c2955c68672466ebc4e06 | tier1/28/2 | US500 | 2024-11-07 | arm_a_breakout_baseline | long | 2024-11-07T17:00:02.807Z | 5972.1 | 19.3 | censored_tick | closed_tick | -1.66103 | -0.299905 | censored_cache_gap |
| f35c2955c68672466ebc4e06 | tier1/28/3.74 | US500 | 2024-11-07 | arm_a_breakout_baseline | long | 2024-11-07T17:00:02.807Z | 5972.1 | 19.3 | censored_tick | closed_tick | -1.66103 | -0.407557 | censored_cache_gap |
| b8faf0a449b13f21ebd8db35 | tier1/28/2 | US500 | 2024-11-08 | arm_a_breakout_baseline | long | 2024-11-08T18:00:01.701Z | 5999 | 19.4 | censored_tick | closed_tick | -1.46886 | -0.10823 | censored_end_of_cache |
| b8faf0a449b13f21ebd8db35 | tier1/28/3.74 | US500 | 2024-11-08 | arm_a_breakout_baseline | long | 2024-11-08T18:00:01.701Z | 5999 | 19.4 | censored_tick | closed_tick | -1.46886 | -0.215844 | censored_end_of_cache |
| 31ac96bb6c2db5e56d25c188 | tier1/48/2 | US500 | 2025-04-07 | arm_a_breakout_baseline | long | 2025-04-07T14:30:00.343Z | 4992.4 | 27.3 | closed_tick | closed_tick | 0.499853 | 1.30753 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/48/2 | US500 | 2025-04-07 | arm_b_pure_pullback_retest | long | 2025-04-07T14:41:36.283Z | 4946.6 | 47.6 | closed_tick | closed_tick | 0.878123 | 1.33841 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/48/3.74 | US500 | 2025-04-07 | arm_a_breakout_baseline | long | 2025-04-07T14:30:00.343Z | 4992.4 | 27.3 | closed_tick | closed_tick | 0.499853 | 1.24365 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/48/3.74 | US500 | 2025-04-07 | arm_b_pure_pullback_retest | long | 2025-04-07T14:41:36.283Z | 4946.6 | 49.4 | closed_tick | closed_tick | 0.896612 | 1.30515 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/28/2 | US500 | 2025-04-07 | arm_a_breakout_baseline | long | 2025-04-07T14:30:00.343Z | 4992.4 | 27.3 | closed_tick | closed_tick | 0.499853 | 1.30753 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/28/2 | US500 | 2025-04-07 | arm_b_pure_pullback_retest | long | 2025-04-07T14:41:36.283Z | 4946.6 | 47.6 | closed_tick | closed_tick | 0.878123 | 1.33841 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/28/3.74 | US500 | 2025-04-07 | arm_a_breakout_baseline | long | 2025-04-07T14:30:00.343Z | 4992.4 | 27.3 | closed_tick | closed_tick | 0.499853 | 1.24365 | closed_tick |
| 31ac96bb6c2db5e56d25c188 | tier1/28/3.74 | US500 | 2025-04-07 | arm_b_pure_pullback_retest | long | 2025-04-07T14:41:36.283Z | 4946.6 | 49.4 | closed_tick | closed_tick | 0.896612 | 1.30515 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/24 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.68214 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/24 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | closed_tick | closed_tick | 0.995041 | 0.995041 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/24 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | closed_tick | closed_tick | 0.995041 | 0.995041 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/2 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.05685 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/2 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 41.2 | closed_tick | closed_tick | 0.77928 | 1.33092 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/2 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 41.2 | closed_tick | closed_tick | 0.77928 | 1.33092 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/3.74 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.1063 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/3.74 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 43 | closed_tick | closed_tick | 0.804938 | 1.29156 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/48/3.74 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 43 | closed_tick | closed_tick | 0.804938 | 1.29156 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/24 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.68214 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/24 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | closed_tick | closed_tick | 0.995041 | 0.995041 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/24 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 64.1 | closed_tick | closed_tick | 0.995041 | 0.995041 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/2 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.05685 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/2 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 41.2 | closed_tick | closed_tick | 0.77928 | 1.33092 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/2 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 41.2 | closed_tick | closed_tick | 0.77928 | 1.33092 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/3.74 | US500 | 2025-04-08 | arm_a_breakout_baseline | short | 2025-04-08T15:30:00.412Z | 5154.7 | 36.4 | closed_tick | closed_tick | -1.68214 | -1.1063 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/3.74 | US500 | 2025-04-08 | arm_b_pure_pullback_retest | short | 2025-04-08T15:40:52.170Z | 5193.8 | 43 | closed_tick | closed_tick | 0.804938 | 1.29156 | closed_tick |
| 5b35427f494f9de75bb0ddd1 | tier1/28/3.74 | US500 | 2025-04-08 | arm_c_macro_filtered_pullback | short | 2025-04-08T15:40:52.170Z | 5193.8 | 43 | closed_tick | closed_tick | 0.804938 | 1.29156 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/48/2 | US500 | 2025-04-09 | arm_a_breakout_baseline | long | 2025-04-09T14:30:00.387Z | 5000.5 | 31.8 | closed_tick | closed_tick | -1.75553 | -1.06584 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/48/2 | US500 | 2025-04-09 | arm_b_pure_pullback_retest | long | 2025-04-09T14:50:49.610Z | 4973.2 | 29.2 | closed_tick | closed_tick | -1.86294 | -1.11586 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/48/3.74 | US500 | 2025-04-09 | arm_a_breakout_baseline | long | 2025-04-09T14:30:00.387Z | 5000.5 | 31.8 | closed_tick | closed_tick | -1.75553 | -1.12039 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/48/3.74 | US500 | 2025-04-09 | arm_b_pure_pullback_retest | long | 2025-04-09T14:50:49.610Z | 4973.2 | 31 | closed_tick | closed_tick | -1.77086 | -1.12285 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/28/2 | US500 | 2025-04-09 | arm_a_breakout_baseline | long | 2025-04-09T14:00:00.434Z | 5009.2 | 15 | closed_tick | closed_tick | -0.226256 | 1.24615 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/28/2 | US500 | 2025-04-09 | arm_b_pure_pullback_retest | long | 2025-04-09T14:14:28.021Z | 4977.5 | 33.5 | closed_tick | closed_tick | 0.665579 | 1.32238 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/28/3.74 | US500 | 2025-04-09 | arm_a_breakout_baseline | long | 2025-04-09T14:00:00.434Z | 5009.2 | 15 | closed_tick | closed_tick | -0.226256 | 1.12969 | closed_tick |
| 7be5dc89274ab1082ba3e7ba | tier1/28/3.74 | US500 | 2025-04-09 | arm_b_pure_pullback_retest | long | 2025-04-09T14:14:28.021Z | 4977.5 | 35.3 | closed_tick | closed_tick | 0.702291 | 1.27645 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/2 | US500 | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T14:30:00.423Z | 5295.4 | 34.2 | closed_tick | closed_tick | -1.74855 | -1.06506 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/2 | US500 | 2025-04-10 | arm_b_pure_pullback_retest | short | 2025-04-10T14:31:27.206Z | 5315.6 | 22 | closed_tick | closed_tick | 0.225367 | 1.28545 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/2 | US500 | 2025-04-10 | arm_c_macro_filtered_pullback | short | 2025-04-10T14:31:27.206Z | 5315.6 | 22 | closed_tick | closed_tick | 0.225367 | 1.28545 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/3.74 | US500 | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T14:30:00.423Z | 5295.4 | 34.2 | closed_tick | closed_tick | -1.74855 | -1.11912 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/3.74 | US500 | 2025-04-10 | arm_b_pure_pullback_retest | short | 2025-04-10T14:31:27.206Z | 5315.6 | 23.8 | closed_tick | closed_tick | 0.313617 | 1.21581 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/3.74 | US500 | 2025-04-10 | arm_c_macro_filtered_pullback | short | 2025-04-10T14:31:27.206Z | 5315.6 | 23.8 | closed_tick | closed_tick | 0.313617 | 1.21581 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/2 | US500 | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T14:00:00.418Z | 5303.1 | 18.7 | closed_tick | closed_tick | -1.42555 | -0.177619 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/2 | US500 | 2025-04-10 | arm_b_pure_pullback_retest | short | 2025-04-10T14:25:08.747Z | 5319.3 | 18.3 | closed_tick | closed_tick | -0.00939454 | 1.26652 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/2 | US500 | 2025-04-10 | arm_c_macro_filtered_pullback | short | 2025-04-10T14:25:08.747Z | 5319.3 | 18.3 | closed_tick | closed_tick | -0.00939454 | 1.26652 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/3.74 | US500 | 2025-04-10 | arm_a_breakout_baseline | short | 2025-04-10T14:00:00.418Z | 5303.1 | 18.7 | closed_tick | closed_tick | -1.42555 | -0.276319 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/3.74 | US500 | 2025-04-10 | arm_b_pure_pullback_retest | short | 2025-04-10T14:25:08.747Z | 5319.3 | 20.1 | closed_tick | closed_tick | -1.27527 | -0.202929 | closed_tick |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/3.74 | US500 | 2025-04-10 | arm_c_macro_filtered_pullback | short | 2025-04-10T14:25:08.747Z | 5319.3 | 20.1 | closed_tick | closed_tick | -1.27527 | -0.202929 | closed_tick |
| acdc1e0257e10eefaf512a7f | tier1/48/24 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.716575 | censored_cache_gap |
| acdc1e0257e10eefaf512a7f | tier1/48/2 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.348961 | censored_cache_gap |
| acdc1e0257e10eefaf512a7f | tier1/48/3.74 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.378036 | censored_cache_gap |
| acdc1e0257e10eefaf512a7f | tier1/28/24 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.716575 | censored_cache_gap |
| acdc1e0257e10eefaf512a7f | tier1/28/2 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.348961 | censored_cache_gap |
| acdc1e0257e10eefaf512a7f | tier1/28/3.74 | US500 | 2025-04-11 | arm_a_breakout_baseline | long | 2025-04-11T17:00:00.269Z | 5332.1 | 63.7 | censored_tick | closed_tick | -0.716575 | -0.378036 | censored_cache_gap |
| 4c63892002b69ae4559435bd | tier2/48/24 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | -0.240423 | closed_m30_approximation |
| 4c63892002b69ae4559435bd | tier2/48/2 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | 0.403818 | closed_m30_approximation |
| 4c63892002b69ae4559435bd | tier2/48/3.74 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | 0.352864 | closed_m30_approximation |
| 4c63892002b69ae4559435bd | tier2/28/24 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | -0.240423 | closed_m30_approximation |
| 4c63892002b69ae4559435bd | tier2/28/2 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | 0.403818 | closed_m30_approximation |
| 4c63892002b69ae4559435bd | tier2/28/3.74 | BTCUSD | 2024-01-23 | arm_a_breakout_baseline | long | 2024-01-23T01:00:02.964Z | 39836 | 272.5 | censored_tick | closed_tick | -0.240423 | 0.352864 | closed_m30_approximation |
| 46b1da2b93e2935f42ee3164 | tier2/28/2 | BTCUSD | 2024-01-24 | arm_a_breakout_baseline | long | 2024-01-24T01:00:01.542Z | 40168 | 120 | closed_tick | closed_tick | -2.60432 | -1.13369 | closed_tick |
| 46b1da2b93e2935f42ee3164 | tier2/28/2 | BTCUSD | 2024-01-24 | arm_b_pure_pullback_retest | long | 2024-01-24T01:08:51.612Z | 40015 | 177.51 | closed_tick | closed_tick | -2.08239 | -1.09273 | closed_tick |
| 46b1da2b93e2935f42ee3164 | tier2/28/3.74 | BTCUSD | 2024-01-24 | arm_a_breakout_baseline | long | 2024-01-24T01:00:01.542Z | 40168 | 120 | closed_tick | closed_tick | -2.60432 | -1.25001 | closed_tick |
| 46b1da2b93e2935f42ee3164 | tier2/28/3.74 | BTCUSD | 2024-01-24 | arm_b_pure_pullback_retest | long | 2024-01-24T01:08:51.612Z | 40015 | 191.44 | closed_tick | closed_tick | -2.01945 | -1.17456 | closed_tick |
| 4a422a139782982730a96c44 | tier2/28/2 | BTCUSD | 2024-07-25 | arm_a_breakout_baseline | short | 2024-07-25T01:30:02.101Z | 64396 | 243.5 | closed_tick | closed_tick | 0.113904 | 1.27449 | closed_tick |
| 4a422a139782982730a96c44 | tier2/28/3.74 | BTCUSD | 2024-07-25 | arm_a_breakout_baseline | short | 2024-07-25T01:30:02.101Z | 64396 | 243.5 | closed_tick | closed_tick | 0.113904 | 1.1827 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/2 | BTCUSD | 2025-05-12 | arm_a_breakout_baseline | long | 2025-05-12T01:00:00.426Z | 104670 | 398.49 | closed_tick | closed_tick | -1.27725 | -0.121551 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/2 | BTCUSD | 2025-05-12 | arm_b_pure_pullback_retest | long | 2025-05-12T01:21:01.902Z | 104357 | 294.17 | closed_tick | closed_tick | -2.7429 | -1.18429 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/2 | BTCUSD | 2025-05-12 | arm_c_macro_filtered_pullback | long | 2025-05-12T01:21:01.902Z | 104357 | 294.17 | closed_tick | closed_tick | -2.7429 | -1.18429 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/3.74 | BTCUSD | 2025-05-12 | arm_a_breakout_baseline | long | 2025-05-12T01:00:00.426Z | 104670 | 398.49 | closed_tick | closed_tick | -1.27725 | -0.212956 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/3.74 | BTCUSD | 2025-05-12 | arm_b_pure_pullback_retest | long | 2025-05-12T01:21:01.902Z | 104357 | 330.49 | closed_tick | closed_tick | -2.55704 | -1.27968 | closed_tick |
| e2f8731ee1bbbd8ebec301bb | tier2/28/3.74 | BTCUSD | 2025-05-12 | arm_c_macro_filtered_pullback | long | 2025-05-12T01:21:01.902Z | 104357 | 330.49 | closed_tick | closed_tick | -2.55704 | -1.27968 | closed_tick |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/2 | BTCUSD | 2025-05-13 | arm_a_breakout_baseline | short | 2025-05-13T01:00:00.417Z | 102282 | 377.78 | closed_tick | closed_tick | -2.30415 | -1.11067 | closed_tick |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/2 | BTCUSD | 2025-05-13 | arm_b_pure_pullback_retest | short | 2025-05-13T01:04:17.293Z | 102538 | 307.83 | closed_tick | closed_tick | -0.215546 | 1.24705 | closed_tick |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/3.74 | BTCUSD | 2025-05-13 | arm_a_breakout_baseline | short | 2025-05-13T01:00:00.417Z | 102282 | 377.78 | closed_tick | closed_tick | -2.30415 | -1.20506 | closed_tick |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/3.74 | BTCUSD | 2025-05-13 | arm_b_pure_pullback_retest | short | 2025-05-13T01:04:17.293Z | 102538 | 343.51 | closed_tick | closed_tick | -1.44132 | -0.231782 | closed_tick |
| ea65ff19fabbc0109ea7c967 | tier2/28/2 | BTCUSD | 2025-05-14 | arm_a_breakout_baseline | short | 2025-05-14T02:00:00.515Z | 103688 | 363.51 | censored_tick | closed_tick | -1.24311 | 0.0116746 | closed_m30_approximation |
| ea65ff19fabbc0109ea7c967 | tier2/28/3.74 | BTCUSD | 2025-05-14 | arm_a_breakout_baseline | short | 2025-05-14T02:00:00.515Z | 103688 | 363.51 | censored_tick | closed_tick | -1.24311 | -0.0875676 | closed_m30_approximation |
| 9ce2dd036808138bac5852eb | tier2/28/2 | BTCUSD | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T02:00:01.107Z | 103001 | 355.08 | censored_tick | closed_tick | -0.71987 | 0.554994 | closed_m30_approximation |
| 9ce2dd036808138bac5852eb | tier2/28/3.74 | BTCUSD | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T02:00:01.107Z | 103001 | 355.08 | censored_tick | closed_tick | -0.71987 | 0.454164 | closed_m30_approximation |
| ae497d730dbeacf465872233 | tier2/28/2 | BTCUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.826Z | 119898 | 390.63 | closed_tick | closed_tick | -0.0966008 | 1.25695 | closed_tick |
| ae497d730dbeacf465872233 | tier2/28/3.74 | BTCUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.826Z | 119898 | 390.63 | closed_tick | closed_tick | -0.0966008 | 1.1499 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/2 | BTCUSD | 2025-10-08 | arm_a_breakout_baseline | long | 2025-10-08T01:00:00.539Z | 122003 | 505.05 | closed_tick | closed_tick | -2.16057 | -1.09989 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/2 | BTCUSD | 2025-10-08 | arm_b_pure_pullback_retest | long | 2025-10-08T01:16:24.227Z | 121680 | 361.9 | closed_tick | closed_tick | -0.237172 | 1.24526 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/2 | BTCUSD | 2025-10-08 | arm_c_macro_filtered_pullback | long | 2025-10-08T01:16:24.227Z | 121680 | 361.9 | closed_tick | closed_tick | -0.237172 | 1.24526 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/3.74 | BTCUSD | 2025-10-08 | arm_a_breakout_baseline | long | 2025-10-08T01:00:00.539Z | 122003 | 505.05 | closed_tick | closed_tick | -2.16057 | -1.18378 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/3.74 | BTCUSD | 2025-10-08 | arm_b_pure_pullback_retest | long | 2025-10-08T01:16:24.227Z | 121680 | 404.24 | closed_tick | closed_tick | -0.0681349 | 1.15435 | closed_tick |
| daaa576e731c68edb339d1a8 | tier2/28/3.74 | BTCUSD | 2025-10-08 | arm_c_macro_filtered_pullback | long | 2025-10-08T01:16:24.227Z | 121680 | 404.24 | closed_tick | closed_tick | -0.0681349 | 1.15435 | closed_tick |
| 14cfdcad3095953593d732bf | tier2/28/2 | ETHUSD | 2025-07-07 | arm_a_breakout_baseline | long | 2025-07-07T03:30:00.786Z | 2588.61 | 9.62 | closed_tick | closed_tick | -2.31514 | -1.13342 | closed_tick |
| 14cfdcad3095953593d732bf | tier2/28/2 | ETHUSD | 2025-07-07 | arm_b_pure_pullback_retest | long | 2025-07-07T03:47:11.521Z | 2579.07 | 11.79 | censored_tick | closed_tick | -1.29369 | -0.331726 | censored_end_of_cache |
| 14cfdcad3095953593d732bf | tier2/28/3.74 | ETHUSD | 2025-07-07 | arm_a_breakout_baseline | long | 2025-07-07T03:30:00.786Z | 2588.61 | 9.62 | closed_tick | closed_tick | -2.31514 | -1.22688 | closed_tick |
| 14cfdcad3095953593d732bf | tier2/28/3.74 | ETHUSD | 2025-07-07 | arm_b_pure_pullback_retest | long | 2025-07-07T03:47:11.521Z | 2579.07 | 12.68 | censored_tick | closed_tick | -1.20289 | -0.379185 | censored_end_of_cache |
| 743d69735dec5f4d37cf225b | tier2/48/2 | ETHUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.485Z | 3006.15 | 15.54 | closed_tick | closed_tick | 0.448456 | 1.30265 | closed_tick |
| 743d69735dec5f4d37cf225b | tier2/48/3.74 | ETHUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.485Z | 3006.15 | 15.54 | closed_tick | closed_tick | 0.448456 | 1.2351 | closed_tick |
| 743d69735dec5f4d37cf225b | tier2/28/2 | ETHUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.485Z | 3006.15 | 15.54 | closed_tick | closed_tick | 0.448456 | 1.30265 | closed_tick |
| 743d69735dec5f4d37cf225b | tier2/28/3.74 | ETHUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T02:30:00.485Z | 3006.15 | 15.54 | closed_tick | closed_tick | 0.448456 | 1.2351 | closed_tick |
| f3ec49598c955968c2cdec80 | tier2/28/2 | NAS100 | 2024-01-22 | arm_a_breakout_baseline | short | 2024-01-22T16:00:00.156Z | 17320.7 | 55.2 | closed_tick | closed_tick | -2.50855 | -1.12571 | closed_tick |
| f3ec49598c955968c2cdec80 | tier2/28/2 | NAS100 | 2024-01-22 | arm_b_pure_pullback_retest | short | 2024-01-22T16:22:15.390Z | 17359.8 | 46.3 | censored_tick | closed_tick | -0.7367 | 0.910711 | censored_cache_gap |
| f3ec49598c955968c2cdec80 | tier2/28/3.74 | NAS100 | 2024-01-22 | arm_a_breakout_baseline | short | 2024-01-22T16:00:00.156Z | 17320.7 | 55.2 | closed_tick | closed_tick | -2.50855 | -1.23508 | closed_tick |
| f3ec49598c955968c2cdec80 | tier2/28/3.74 | NAS100 | 2024-01-22 | arm_b_pure_pullback_retest | short | 2024-01-22T16:22:15.390Z | 17359.8 | 52.3 | censored_tick | closed_tick | -0.652184 | 0.690884 | censored_cache_gap |
| 7048b77cb373429eafea7db7 | tier2/28/2 | NAS100 | 2024-01-24 | arm_a_breakout_baseline | long | 2024-01-24T17:00:00.207Z | 17626 | 54.5 | censored_tick | closed_tick | -1.41144 | 0.011891 | censored_cache_gap |
| 7048b77cb373429eafea7db7 | tier2/28/3.74 | NAS100 | 2024-01-24 | arm_a_breakout_baseline | long | 2024-01-24T17:00:00.207Z | 17626 | 54.5 | censored_tick | closed_tick | -1.41144 | -0.100681 | censored_cache_gap |
| dae3cca41d4e71510881f096 | tier2/28/2 | NAS100 | 2024-01-25 | arm_a_breakout_baseline | short | 2024-01-25T18:00:00.205Z | 17533.2 | 63.1 | censored_tick | closed_tick | -0.962016 | 0.259769 | censored_cache_gap |
| dae3cca41d4e71510881f096 | tier2/28/3.74 | NAS100 | 2024-01-25 | arm_a_breakout_baseline | short | 2024-01-25T18:00:00.205Z | 17533.2 | 63.1 | censored_tick | closed_tick | -0.962016 | 0.163137 | censored_cache_gap |
| d2dea2acc6abeda4dd84ac5c | tier2/28/2 | SpotBrent | 2023-07-24 | arm_a_breakout_baseline | long | 2023-07-24T09:30:00.725Z | 81.43 | 0.33 | closed_tick | closed_tick | -1.22978 | -0.144148 | closed_tick |
| d2dea2acc6abeda4dd84ac5c | tier2/28/3.74 | SpotBrent | 2023-07-24 | arm_a_breakout_baseline | long | 2023-07-24T09:30:00.725Z | 81.43 | 0.33 | closed_tick | closed_tick | -1.22978 | -0.230012 | closed_tick |
| 9beae18a5071ebb3f315a848 | tier2/28/2 | SpotBrent | 2023-07-25 | arm_a_breakout_baseline | short | 2023-07-25T09:00:02.675Z | 82.423 | 0.241 | closed_tick | closed_tick | -2.66066 | -1.1536 | closed_tick |
| 9beae18a5071ebb3f315a848 | tier2/28/2 | SpotBrent | 2023-07-25 | arm_b_pure_pullback_retest | short | 2023-07-25T09:39:22.228Z | 82.663 | 0.265 | censored_tick | closed_tick | -1.7999 | -0.426722 | closed_m30_approximation |
| 9beae18a5071ebb3f315a848 | tier2/28/3.74 | SpotBrent | 2023-07-25 | arm_a_breakout_baseline | short | 2023-07-25T09:00:02.675Z | 82.423 | 0.241 | closed_tick | closed_tick | -2.66066 | -1.2728 | closed_tick |
| 9beae18a5071ebb3f315a848 | tier2/28/3.74 | SpotBrent | 2023-07-25 | arm_b_pure_pullback_retest | short | 2023-07-25T09:39:22.228Z | 82.663 | 0.293 | censored_tick | closed_tick | -1.6279 | -0.48417 | closed_m30_approximation |
| c8420ab56a4df29d24c8d1b5 | tier2/28/2 | SpotBrent | 2023-07-26 | arm_a_breakout_baseline | short | 2023-07-26T10:00:01.253Z | 82.618 | 0.296 | censored_tick | closed_tick | -1.44135 | -0.213018 | closed_m30_approximation |
| c8420ab56a4df29d24c8d1b5 | tier2/28/3.74 | SpotBrent | 2023-07-26 | arm_a_breakout_baseline | short | 2023-07-26T10:00:01.253Z | 82.618 | 0.296 | censored_tick | closed_tick | -1.44135 | -0.310167 | closed_m30_approximation |
| d7c8c21dd5d0ad94aed42504 | tier2/28/2 | SpotBrent | 2025-02-11 | arm_a_breakout_baseline | long | 2025-02-11T08:00:00.473Z | 77.379 | 0.22 | closed_tick | closed_tick | -0.309767 | 1.24085 | closed_tick |
| d7c8c21dd5d0ad94aed42504 | tier2/28/3.74 | SpotBrent | 2025-02-11 | arm_a_breakout_baseline | long | 2025-02-11T08:00:00.473Z | 77.379 | 0.22 | closed_tick | closed_tick | -0.309767 | 1.11821 | closed_tick |
| affe109b180bfeb08d170ec7 | tier2/48/2 | SpotBrent | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:04.401Z | 64.11 | 0.325 | closed_tick | closed_tick | -1.96468 | -1.09449 | closed_tick |
| affe109b180bfeb08d170ec7 | tier2/48/2 | SpotBrent | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.075Z | 64.33 | 0.211 | censored_tick | closed_tick | -2.12853 | -0.785592 | closed_m30_approximation |
| affe109b180bfeb08d170ec7 | tier2/48/3.74 | SpotBrent | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:04.401Z | 64.11 | 0.325 | closed_tick | closed_tick | -1.96468 | -1.16332 | closed_tick |
| affe109b180bfeb08d170ec7 | tier2/48/3.74 | SpotBrent | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.075Z | 64.33 | 0.234 | censored_tick | closed_tick | -1.91932 | -0.80415 | closed_m30_approximation |
| affe109b180bfeb08d170ec7 | tier2/28/2 | SpotBrent | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:04.401Z | 64.11 | 0.325 | closed_tick | closed_tick | -1.96468 | -1.09449 | closed_tick |
| affe109b180bfeb08d170ec7 | tier2/28/2 | SpotBrent | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.075Z | 64.33 | 0.211 | censored_tick | closed_tick | -2.12853 | -0.785592 | closed_m30_approximation |
| affe109b180bfeb08d170ec7 | tier2/28/3.74 | SpotBrent | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:04.401Z | 64.11 | 0.325 | closed_tick | closed_tick | -1.96468 | -1.16332 | closed_tick |
| affe109b180bfeb08d170ec7 | tier2/28/3.74 | SpotBrent | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.075Z | 64.33 | 0.234 | censored_tick | closed_tick | -1.91932 | -0.80415 | closed_m30_approximation |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/2 | SpotBrent | 2025-07-22 | arm_a_breakout_baseline | short | 2025-07-22T08:00:00.542Z | 68.729 | 0.205 | closed_tick | closed_tick | -2.63611 | -1.1587 | closed_tick |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/2 | SpotBrent | 2025-07-22 | arm_b_pure_pullback_retest | short | 2025-07-22T08:16:21.764Z | 68.949 | 0.248 | closed_tick | closed_tick | 0.0518871 | 1.27213 | closed_tick |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/3.74 | SpotBrent | 2025-07-22 | arm_a_breakout_baseline | short | 2025-07-22T08:00:00.542Z | 68.729 | 0.205 | closed_tick | closed_tick | -2.63611 | -1.27555 | closed_tick |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/3.74 | SpotBrent | 2025-07-22 | arm_b_pure_pullback_retest | short | 2025-07-22T08:16:21.764Z | 68.949 | 0.272 | closed_tick | closed_tick | -1.21675 | -0.18961 | closed_tick |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/2 | SpotBrent | 2025-10-07 | arm_a_breakout_baseline | short | 2025-10-07T09:30:00.864Z | 65.42 | 0.23 | closed_tick | closed_tick | -2.36769 | -1.11397 | closed_tick |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/2 | SpotBrent | 2025-10-07 | arm_b_pure_pullback_retest | short | 2025-10-07T10:03:15.212Z | 65.76 | 0.277 | censored_tick | closed_tick | -0.958585 | 0.0855812 | closed_m30_approximation |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/2 | SpotBrent | 2025-10-07 | arm_c_macro_filtered_pullback | short | 2025-10-07T10:03:15.212Z | 65.76 | 0.277 | censored_tick | closed_tick | -0.958585 | 0.0855812 | closed_m30_approximation |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/3.74 | SpotBrent | 2025-10-07 | arm_a_breakout_baseline | short | 2025-10-07T09:30:00.864Z | 65.42 | 0.23 | closed_tick | closed_tick | -2.36769 | -1.21313 | closed_tick |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/3.74 | SpotBrent | 2025-10-07 | arm_b_pure_pullback_retest | short | 2025-10-07T10:03:15.212Z | 65.76 | 0.3 | censored_tick | closed_tick | -0.885093 | 0.0027674 | closed_m30_approximation |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/3.74 | SpotBrent | 2025-10-07 | arm_c_macro_filtered_pullback | short | 2025-10-07T10:03:15.212Z | 65.76 | 0.3 | censored_tick | closed_tick | -0.885093 | 0.0027674 | closed_m30_approximation |
| b50a68dfbdb04a30f12e9a60 | tier2/28/2 | SpotCrude | 2023-01-25 | arm_a_breakout_baseline | short | 2023-01-25T08:30:00.618Z | 79.846 | 0.245 | closed_tick | closed_tick | -0.177336 | 1.25359 | closed_tick |
| b50a68dfbdb04a30f12e9a60 | tier2/28/2 | SpotCrude | 2023-01-25 | arm_b_pure_pullback_retest | short | 2023-01-25T09:22:05.651Z | 80.075 | 0.268 | closed_tick | closed_tick | -2.44406 | -1.12718 | closed_tick |
| b50a68dfbdb04a30f12e9a60 | tier2/28/3.74 | SpotCrude | 2023-01-25 | arm_a_breakout_baseline | short | 2023-01-25T08:30:00.618Z | 79.846 | 0.245 | closed_tick | closed_tick | -0.177336 | 1.14042 | closed_tick |
| b50a68dfbdb04a30f12e9a60 | tier2/28/3.74 | SpotCrude | 2023-01-25 | arm_b_pure_pullback_retest | short | 2023-01-25T09:22:05.651Z | 80.075 | 0.295 | closed_tick | closed_tick | -2.32231 | -1.22037 | closed_tick |
| 8f401fb18724fa4ecb2f8a36 | tier2/28/2 | SpotCrude | 2023-07-24 | arm_a_breakout_baseline | long | 2023-07-24T09:30:00.725Z | 77.537 | 0.321 | closed_tick | closed_tick | -1.22159 | -0.158912 | closed_tick |
| 8f401fb18724fa4ecb2f8a36 | tier2/28/3.74 | SpotCrude | 2023-07-24 | arm_a_breakout_baseline | long | 2023-07-24T09:30:00.725Z | 77.537 | 0.321 | closed_tick | closed_tick | -1.22159 | -0.24296 | closed_tick |
| dd362c3ee13db2ddcbd9cf99 | tier2/28/2 | SpotCrude | 2023-07-26 | arm_a_breakout_baseline | short | 2023-07-26T10:00:01.727Z | 78.854 | 0.266 | censored_tick | closed_tick | -1.7244 | -0.419389 | closed_m30_approximation |
| dd362c3ee13db2ddcbd9cf99 | tier2/28/3.74 | SpotCrude | 2023-07-26 | arm_a_breakout_baseline | short | 2023-07-26T10:00:01.727Z | 78.854 | 0.266 | censored_tick | closed_tick | -1.7244 | -0.522604 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/24 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.88928 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/24 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.533 | censored_tick | closed_tick | -0.793489 | -0.793489 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/2 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.10107 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/2 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.265 | censored_tick | closed_tick | -1.59596 | -0.582682 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/3.74 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.16341 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/3.74 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.286 | censored_tick | closed_tick | -1.47877 | -0.614155 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/24 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.88928 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/24 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.533 | censored_tick | closed_tick | -0.793489 | -0.793489 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/2 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.10107 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/2 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.265 | censored_tick | closed_tick | -1.59596 | -0.582682 | closed_m30_approximation |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/3.74 | SpotCrude | 2025-05-15 | arm_a_breakout_baseline | short | 2025-05-15T09:00:01.091Z | 60.732 | 0.34 | closed_tick | closed_tick | -1.88928 | -1.16341 | closed_tick |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/3.74 | SpotCrude | 2025-05-15 | arm_b_pure_pullback_retest | short | 2025-05-15T10:14:34.071Z | 60.962 | 0.286 | censored_tick | closed_tick | -1.47877 | -0.614155 | closed_m30_approximation |
| 8970de420891b9d784ed058b | tier2/28/2 | SpotCrude | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T08:00:00.863Z | 69.553 | 0.295 | closed_tick | closed_tick | -2.18004 | -1.14495 | closed_tick |
| 8970de420891b9d784ed058b | tier2/28/2 | SpotCrude | 2025-07-14 | arm_b_pure_pullback_retest | long | 2025-07-14T09:15:35.106Z | 69.233 | 0.358 | closed_tick | closed_tick | 0.451099 | 1.30505 | closed_tick |
| 8970de420891b9d784ed058b | tier2/28/3.74 | SpotCrude | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T08:00:00.863Z | 69.553 | 0.295 | closed_tick | closed_tick | -2.18004 | -1.22681 | closed_tick |
| 8970de420891b9d784ed058b | tier2/28/3.74 | SpotCrude | 2025-07-14 | arm_b_pure_pullback_retest | long | 2025-07-14T09:15:35.106Z | 69.233 | 0.382 | closed_tick | closed_tick | 0.508938 | 1.24612 | closed_tick |
| 35ba9e2c232645825deb093d | tier2/28/2 | SpotCrude | 2025-07-22 | arm_a_breakout_baseline | short | 2025-07-22T08:00:00.826Z | 66.335 | 0.23 | closed_tick | closed_tick | -2.43037 | -1.15905 | closed_tick |
| 35ba9e2c232645825deb093d | tier2/28/2 | SpotCrude | 2025-07-22 | arm_b_pure_pullback_retest | short | 2025-07-22T08:16:09.909Z | 66.585 | 0.287 | closed_tick | closed_tick | -1.18347 | -0.162502 | closed_tick |
| 35ba9e2c232645825deb093d | tier2/28/3.74 | SpotCrude | 2025-07-22 | arm_a_breakout_baseline | short | 2025-07-22T08:00:00.826Z | 66.335 | 0.23 | closed_tick | closed_tick | -2.43037 | -1.2596 | closed_tick |
| 35ba9e2c232645825deb093d | tier2/28/3.74 | SpotCrude | 2025-07-22 | arm_b_pure_pullback_retest | short | 2025-07-22T08:16:09.909Z | 66.585 | 0.31 | closed_tick | closed_tick | -1.09566 | -0.225203 | closed_tick |
| 5aca717f89d879a30446d2ed | tier2/28/2 | SpotCrude | 2025-10-07 | arm_a_breakout_baseline | short | 2025-10-07T09:00:01.018Z | 61.659 | 0.21 | closed_tick | closed_tick | -2.45948 | -1.16527 | closed_tick |
| 5aca717f89d879a30446d2ed | tier2/28/2 | SpotCrude | 2025-10-07 | arm_b_pure_pullback_retest | short | 2025-10-07T10:53:36.730Z | 61.859 | 0.205 | censored_tick | closed_tick | -0.95943 | 0.367202 | closed_m30_approximation |
| 5aca717f89d879a30446d2ed | tier2/28/2 | SpotCrude | 2025-10-07 | arm_c_macro_filtered_pullback | short | 2025-10-07T10:53:36.730Z | 61.859 | 0.205 | censored_tick | closed_tick | -0.95943 | 0.367202 | closed_m30_approximation |
| 5aca717f89d879a30446d2ed | tier2/28/3.74 | SpotCrude | 2025-10-07 | arm_a_breakout_baseline | short | 2025-10-07T09:00:01.018Z | 61.659 | 0.21 | closed_tick | closed_tick | -2.45948 | -1.26763 | closed_tick |
| 5aca717f89d879a30446d2ed | tier2/28/3.74 | SpotCrude | 2025-10-07 | arm_b_pure_pullback_retest | short | 2025-10-07T10:53:36.730Z | 61.859 | 0.227 | censored_tick | closed_tick | -0.866446 | 0.236858 | closed_m30_approximation |
| 5aca717f89d879a30446d2ed | tier2/28/3.74 | SpotCrude | 2025-10-07 | arm_c_macro_filtered_pullback | short | 2025-10-07T10:53:36.730Z | 61.859 | 0.227 | censored_tick | closed_tick | -0.866446 | 0.236858 | closed_m30_approximation |
| 876f6148e2ef9692e251c46d | tier2/28/2 | US500 | 2023-01-23 | arm_a_breakout_baseline | long | 2023-01-23T15:30:02.665Z | 4005.9 | 16 | closed_tick | closed_tick | 0.176165 | 1.28083 | closed_tick |
| 876f6148e2ef9692e251c46d | tier2/28/3.74 | US500 | 2023-01-23 | arm_a_breakout_baseline | long | 2023-01-23T15:30:02.665Z | 4005.9 | 16 | closed_tick | closed_tick | 0.176165 | 1.19346 | closed_tick |
| 714dcd4ea643c7abb6c67ecd | tier2/28/2 | US500 | 2023-01-24 | arm_a_breakout_baseline | long | 2023-01-24T16:00:00.571Z | 4018.3 | 12.2 | closed_tick | closed_tick | -2.59492 | -1.14794 | closed_tick |
| 714dcd4ea643c7abb6c67ecd | tier2/28/2 | US500 | 2023-01-24 | arm_b_pure_pullback_retest | long | 2023-01-24T17:01:07.816Z | 4009.6 | 10.2 | censored_tick | closed_tick | -1.13378 | 0.597512 | closed_m30_approximation |
| 714dcd4ea643c7abb6c67ecd | tier2/28/2 | US500 | 2023-01-24 | arm_c_macro_filtered_pullback | long | 2023-01-24T17:01:07.816Z | 4009.6 | 10.2 | censored_tick | closed_tick | -1.13378 | 0.597512 | closed_m30_approximation |
| 714dcd4ea643c7abb6c67ecd | tier2/28/3.74 | US500 | 2023-01-24 | arm_a_breakout_baseline | long | 2023-01-24T16:00:00.571Z | 4018.3 | 12.2 | closed_tick | closed_tick | -2.59492 | -1.26238 | closed_tick |
| 714dcd4ea643c7abb6c67ecd | tier2/28/3.74 | US500 | 2023-01-24 | arm_b_pure_pullback_retest | long | 2023-01-24T17:01:07.816Z | 4009.6 | 11.5 | censored_tick | closed_tick | -1.00561 | 0.408516 | closed_m30_approximation |
| 714dcd4ea643c7abb6c67ecd | tier2/28/3.74 | US500 | 2023-01-24 | arm_c_macro_filtered_pullback | long | 2023-01-24T17:01:07.816Z | 4009.6 | 11.5 | censored_tick | closed_tick | -1.00561 | 0.408516 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/48/24 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.85229 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/48/24 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 33.2 | censored_tick | closed_tick | -0.763822 | -0.763822 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/48/2 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.07504 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/48/2 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 15.5 | censored_tick | closed_tick | -1.63606 | -0.491177 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/48/3.74 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.13652 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/48/3.74 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 16.9 | censored_tick | closed_tick | -1.50053 | -0.533536 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/28/24 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.85229 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/28/24 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 33.2 | censored_tick | closed_tick | -0.763822 | -0.763822 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/28/2 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.07504 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/28/2 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 15.5 | censored_tick | closed_tick | -1.63606 | -0.491177 | closed_m30_approximation |
| 40cc399289d0861550320dd8 | tier2/28/3.74 | US500 | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T16:00:00.879Z | 4016.1 | 22.8 | closed_tick | closed_tick | -1.85229 | -1.13652 | closed_tick |
| 40cc399289d0861550320dd8 | tier2/28/3.74 | US500 | 2023-01-26 | arm_b_pure_pullback_retest | short | 2023-01-26T16:48:09.553Z | 4030.1 | 16.9 | censored_tick | closed_tick | -1.50053 | -0.533536 | closed_m30_approximation |
| f8992619aab40fdf9fa815cf | tier2/28/2 | US500 | 2023-01-27 | arm_a_breakout_baseline | long | 2023-01-27T15:30:00.970Z | 4069.4 | 18.8 | closed_tick | closed_tick | -2.0366 | -1.08638 | closed_tick |
| f8992619aab40fdf9fa815cf | tier2/28/2 | US500 | 2023-01-27 | arm_b_pure_pullback_retest | long | 2023-01-27T15:46:46.666Z | 4058.9 | 12.3 | closed_tick | closed_tick | -0.205164 | 1.24984 | closed_tick |
| f8992619aab40fdf9fa815cf | tier2/28/2 | US500 | 2023-01-27 | arm_c_macro_filtered_pullback | long | 2023-01-27T15:46:46.666Z | 4058.9 | 12.3 | closed_tick | closed_tick | -0.205164 | 1.24984 | closed_tick |
| f8992619aab40fdf9fa815cf | tier2/28/3.74 | US500 | 2023-01-27 | arm_a_breakout_baseline | long | 2023-01-27T15:30:00.970Z | 4069.4 | 18.8 | closed_tick | closed_tick | -2.0366 | -1.16154 | closed_tick |
| f8992619aab40fdf9fa815cf | tier2/28/3.74 | US500 | 2023-01-27 | arm_b_pure_pullback_retest | long | 2023-01-27T15:46:46.666Z | 4058.9 | 13.7 | closed_tick | closed_tick | -0.0385635 | 1.16473 | closed_tick |
| f8992619aab40fdf9fa815cf | tier2/28/3.74 | US500 | 2023-01-27 | arm_c_macro_filtered_pullback | long | 2023-01-27T15:46:46.666Z | 4058.9 | 13.7 | closed_tick | closed_tick | -0.0385635 | 1.16473 | closed_tick |
| 001f989aba50039f6db56506 | tier2/28/2 | US500 | 2024-07-24 | arm_a_breakout_baseline | short | 2024-07-24T14:30:01.663Z | 5477.8 | 25.5 | censored_tick | closed_tick | -0.370711 | 0.573029 | closed_m30_approximation |
| 001f989aba50039f6db56506 | tier2/28/3.74 | US500 | 2024-07-24 | arm_a_breakout_baseline | short | 2024-07-24T14:30:01.663Z | 5477.8 | 25.5 | censored_tick | closed_tick | -0.370711 | 0.498388 | closed_m30_approximation |
| ff90f42b6db284ffa5fee37a | tier2/28/2 | US500 | 2024-07-25 | arm_a_breakout_baseline | long | 2024-07-25T15:30:00.082Z | 5450.4 | 17.4 | closed_tick | closed_tick | -0.121825 | 1.25948 | closed_tick |
| ff90f42b6db284ffa5fee37a | tier2/28/3.74 | US500 | 2024-07-25 | arm_a_breakout_baseline | long | 2024-07-25T15:30:00.082Z | 5450.4 | 17.4 | closed_tick | closed_tick | -0.121825 | 1.15023 | closed_tick |
| 0c8d1df35ca48a63714fb422 | tier2/28/2 | US500 | 2024-11-04 | arm_a_breakout_baseline | short | 2024-11-04T17:00:00.423Z | 5706 | 18.8 | closed_tick | closed_tick | -2.46458 | -1.12692 | closed_tick |
| 0c8d1df35ca48a63714fb422 | tier2/28/2 | US500 | 2024-11-04 | arm_b_pure_pullback_retest | short | 2024-11-04T17:28:23.969Z | 5718.3 | 13.9 | censored_tick | closed_tick | -2.36409 | -0.553122 | censored_cache_gap |
| 0c8d1df35ca48a63714fb422 | tier2/28/2 | US500 | 2024-11-04 | arm_c_macro_filtered_pullback | short | 2024-11-04T17:28:23.969Z | 5718.3 | 13.9 | censored_tick | closed_tick | -2.36409 | -0.553122 | censored_cache_gap |
| 0c8d1df35ca48a63714fb422 | tier2/28/3.74 | US500 | 2024-11-04 | arm_a_breakout_baseline | short | 2024-11-04T17:00:00.423Z | 5706 | 18.8 | closed_tick | closed_tick | -2.46458 | -1.23272 | closed_tick |
| 0c8d1df35ca48a63714fb422 | tier2/28/3.74 | US500 | 2024-11-04 | arm_b_pure_pullback_retest | short | 2024-11-04T17:28:23.969Z | 5718.3 | 15.9 | censored_tick | closed_tick | -2.06672 | -0.608762 | censored_cache_gap |
| 0c8d1df35ca48a63714fb422 | tier2/28/3.74 | US500 | 2024-11-04 | arm_c_macro_filtered_pullback | short | 2024-11-04T17:28:23.969Z | 5718.3 | 15.9 | censored_tick | closed_tick | -2.06672 | -0.608762 | censored_cache_gap |
| 2c7fe293cc545089e134e08c | tier2/28/2 | US500 | 2025-02-12 | arm_a_breakout_baseline | long | 2025-02-12T16:00:01.062Z | 6035.7 | 24.4 | censored_tick | closed_tick | -0.692639 | 0.396857 | censored_cache_gap |
| 2c7fe293cc545089e134e08c | tier2/28/2 | US500 | 2025-02-12 | arm_b_pure_pullback_retest | long | 2025-02-12T16:12:41.579Z | 6020 | 18.3 | closed_tick | closed_tick | -0.199821 | 1.25065 | closed_tick |
| 2c7fe293cc545089e134e08c | tier2/28/2 | US500 | 2025-02-12 | arm_c_macro_filtered_pullback | long | 2025-02-12T16:12:41.579Z | 6020 | 18.3 | closed_tick | closed_tick | -0.199821 | 1.25065 | closed_tick |
| 2c7fe293cc545089e134e08c | tier2/28/3.74 | US500 | 2025-02-12 | arm_a_breakout_baseline | long | 2025-02-12T16:00:01.062Z | 6035.7 | 24.4 | censored_tick | closed_tick | -0.692639 | 0.310687 | censored_cache_gap |
| 2c7fe293cc545089e134e08c | tier2/28/3.74 | US500 | 2025-02-12 | arm_b_pure_pullback_retest | long | 2025-02-12T16:12:41.579Z | 6020 | 20.4 | closed_tick | closed_tick | -0.0374353 | 1.1611 | closed_tick |
| 2c7fe293cc545089e134e08c | tier2/28/3.74 | US500 | 2025-02-12 | arm_c_macro_filtered_pullback | long | 2025-02-12T16:12:41.579Z | 6020 | 20.4 | closed_tick | closed_tick | -0.0374353 | 1.1611 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/24 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.86147 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/24 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 45 | censored_tick | closed_tick | -0.95893 | -0.95893 | censored_cache_gap |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/2 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.07179 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/2 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 19.4 | closed_tick | closed_tick | -2.44134 | -1.12011 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/3.74 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.13425 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/3.74 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 21.4 | closed_tick | closed_tick | -2.30686 | -1.20365 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/24 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.86147 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/24 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 45 | censored_tick | closed_tick | -0.95893 | -0.95893 | censored_cache_gap |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/2 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.07179 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/2 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 19.4 | closed_tick | closed_tick | -2.44134 | -1.12011 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/3.74 | US500 | 2025-05-12 | arm_a_breakout_baseline | short | 2025-05-12T14:30:00.540Z | 5798.7 | 32.4 | closed_tick | closed_tick | -1.86147 | -1.13425 | closed_tick |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/3.74 | US500 | 2025-05-12 | arm_b_pure_pullback_retest | short | 2025-05-12T14:53:47.467Z | 5815.7 | 21.4 | closed_tick | closed_tick | -2.30686 | -1.20365 | closed_tick |
| cf4371537a3ed53252478781 | tier2/28/2 | US500 | 2025-05-13 | arm_a_breakout_baseline | long | 2025-05-13T14:30:00.372Z | 5886.3 | 20.5 | censored_tick | closed_tick | -0.692101 | 0.572813 | closed_m30_approximation |
| cf4371537a3ed53252478781 | tier2/28/3.74 | US500 | 2025-05-13 | arm_a_breakout_baseline | long | 2025-05-13T14:30:00.372Z | 5886.3 | 20.5 | censored_tick | closed_tick | -0.692101 | 0.472769 | closed_m30_approximation |
| f3fb5435535d98dde263ca12 | tier2/28/2 | US500 | 2025-07-21 | arm_a_breakout_baseline | long | 2025-07-21T14:30:03.372Z | 6338.5 | 20.6 | censored_tick | closed_tick | -1.44788 | -0.0939573 | closed_m30_approximation |
| f3fb5435535d98dde263ca12 | tier2/28/3.74 | US500 | 2025-07-21 | arm_a_breakout_baseline | long | 2025-07-21T14:30:03.372Z | 6338.5 | 20.6 | censored_tick | closed_tick | -1.44788 | -0.20104 | closed_m30_approximation |
| 25e98df1d7dd285726528311 | tier2/28/2 | US500 | 2025-10-09 | arm_a_breakout_baseline | short | 2025-10-09T14:30:01.394Z | 6737.3 | 19 | censored_tick | closed_tick | -0.789343 | 0.768871 | closed_m30_approximation |
| 25e98df1d7dd285726528311 | tier2/28/3.74 | US500 | 2025-10-09 | arm_a_breakout_baseline | short | 2025-10-09T14:30:01.394Z | 6737.3 | 19 | censored_tick | closed_tick | -0.789343 | 0.64563 | closed_m30_approximation |
| ed2e5b33c6527114311316f0 | tier2/28/2 | XAGUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T09:30:00.860Z | 39.135 | 0.131 | closed_tick | closed_tick | -2.43155 | -1.1193 | closed_tick |
| ed2e5b33c6527114311316f0 | tier2/28/2 | XAGUSD | 2025-07-14 | arm_b_pure_pullback_retest | long | 2025-07-14T09:53:25.013Z | 39.031 | 0.123 | censored_tick | closed_tick | -1.6286 | -0.2326 | censored_end_of_cache |
| ed2e5b33c6527114311316f0 | tier2/28/3.74 | XAGUSD | 2025-07-14 | arm_a_breakout_baseline | long | 2025-07-14T09:30:00.860Z | 39.135 | 0.131 | closed_tick | closed_tick | -2.43155 | -1.22308 | closed_tick |
| ed2e5b33c6527114311316f0 | tier2/28/3.74 | XAGUSD | 2025-07-14 | arm_b_pure_pullback_retest | long | 2025-07-14T09:53:25.013Z | 39.031 | 0.136 | censored_tick | closed_tick | -1.47292 | -0.310223 | censored_end_of_cache |
| b25d14035008c3a9fb364d7b | tier2/28/2 | XAUUSD | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T09:30:00.715Z | 1936.32 | 7.52 | censored_tick | closed_tick | -1.31859 | -0.185459 | closed_m30_approximation |
| b25d14035008c3a9fb364d7b | tier2/28/3.74 | XAUUSD | 2023-01-26 | arm_a_breakout_baseline | short | 2023-01-26T09:30:00.715Z | 1936.32 | 7.52 | censored_tick | closed_tick | -1.31859 | -0.27508 | closed_m30_approximation |
| 10d1ae4831eddd0539ba98b8 | tier2/28/2 | XAUUSD | 2024-01-23 | arm_a_breakout_baseline | short | 2024-01-23T10:30:00.232Z | 2026.94 | 7.5 | censored_tick | closed_tick | -1.07137 | 0.117275 | censored_cache_gap |
| 10d1ae4831eddd0539ba98b8 | tier2/28/3.74 | XAUUSD | 2024-01-23 | arm_a_breakout_baseline | short | 2024-01-23T10:30:00.232Z | 2026.94 | 7.5 | censored_tick | closed_tick | -1.07137 | 0.0232641 | censored_cache_gap |

| Episode | Cell | Symbol | Date | Own status | Own5 | Own20 | Peer status | Macro reason | Pullback result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cdb6706f8187637f3fe3352d | tier1/48/2 | SpotBrent | 2024-11-04 | complete | 0.00513516 | -0.0640015 | complete | own_5d20d_disagree_or_missing | no_retest |
| cdb6706f8187637f3fe3352d | tier1/48/3.74 | SpotBrent | 2024-11-04 | complete | 0.00513516 | -0.0640015 | complete | own_5d20d_disagree_or_missing | no_retest |
| cdb6706f8187637f3fe3352d | tier1/28/2 | SpotBrent | 2024-11-04 | complete | 0.00513516 | -0.0640015 | complete | own_5d20d_disagree_or_missing | no_retest |
| cdb6706f8187637f3fe3352d | tier1/28/3.74 | SpotBrent | 2024-11-04 | complete | 0.00513516 | -0.0640015 | complete | own_5d20d_disagree_or_missing | no_retest |
| 7e1f8994fecf745690603eeb | tier1/28/2 | SpotBrent | 2024-11-06 | complete | 0.0589301 | -0.0277383 | complete | own_5d20d_disagree_or_missing | entered |
| 7e1f8994fecf745690603eeb | tier1/28/3.74 | SpotBrent | 2024-11-06 | complete | 0.0589301 | -0.0277383 | complete | own_5d20d_disagree_or_missing | entered |
| a92fb0ae4794ae5899abbb1d | tier1/28/2 | SpotBrent | 2025-04-07 | complete | -0.0889527 | -0.0542017 | complete | own_and_peer_aligned | entered |
| a92fb0ae4794ae5899abbb1d | tier1/28/3.74 | SpotBrent | 2025-04-07 | complete | -0.0889527 | -0.0542017 | complete | own_and_peer_aligned | entered |
| f5233c2f0a824dfd2f998e89 | tier1/28/2 | SpotBrent | 2025-04-08 | complete | -0.142156 | -0.0720392 | complete | own_and_peer_aligned | entered |
| f5233c2f0a824dfd2f998e89 | tier1/28/3.74 | SpotBrent | 2025-04-08 | complete | -0.142156 | -0.0720392 | complete | own_and_peer_aligned | entered |
| c3a0d383d87130df509791fa | tier1/48/2 | SpotBrent | 2025-04-09 | complete | -0.167704 | -0.109467 | complete | own_and_peer_aligned | no_retest |
| c3a0d383d87130df509791fa | tier1/48/3.74 | SpotBrent | 2025-04-09 | complete | -0.167704 | -0.109467 | complete | own_and_peer_aligned | no_retest |
| c3a0d383d87130df509791fa | tier1/28/2 | SpotBrent | 2025-04-09 | complete | -0.167704 | -0.109467 | complete | own_and_peer_aligned | entered |
| c3a0d383d87130df509791fa | tier1/28/3.74 | SpotBrent | 2025-04-09 | complete | -0.167704 | -0.109467 | complete | own_and_peer_aligned | entered |
| 1f6ef75ca564ffa87d523f97 | tier1/48/2 | SpotBrent | 2025-04-10 | complete | -0.0998907 | -0.0724495 | complete | own_and_peer_aligned | no_retest |
| 1f6ef75ca564ffa87d523f97 | tier1/48/3.74 | SpotBrent | 2025-04-10 | complete | -0.0998907 | -0.0724495 | complete | own_and_peer_aligned | no_retest |
| 1f6ef75ca564ffa87d523f97 | tier1/28/2 | SpotBrent | 2025-04-10 | complete | -0.0998907 | -0.0724495 | complete | own_and_peer_aligned | no_retest |
| 1f6ef75ca564ffa87d523f97 | tier1/28/3.74 | SpotBrent | 2025-04-10 | complete | -0.0998907 | -0.0724495 | complete | own_and_peer_aligned | no_retest |
| 70e3acf0ef22a39ed2354106 | tier1/48/2 | SpotBrent | 2025-04-11 | complete | -0.0800173 | -0.0968622 | complete | own_and_peer_aligned | entered |
| 70e3acf0ef22a39ed2354106 | tier1/48/3.74 | SpotBrent | 2025-04-11 | complete | -0.0800173 | -0.0968622 | complete | own_and_peer_aligned | entered |
| 70e3acf0ef22a39ed2354106 | tier1/28/2 | SpotBrent | 2025-04-11 | complete | -0.0800173 | -0.0968622 | complete | own_and_peer_aligned | entered |
| 70e3acf0ef22a39ed2354106 | tier1/28/3.74 | SpotBrent | 2025-04-11 | complete | -0.0800173 | -0.0968622 | complete | own_and_peer_aligned | entered |
| a451c5daee55ffdac04ccd9a | tier1/48/2 | SpotCrude | 2024-11-04 | complete | 0.0101437 | -0.0656137 | complete | own_5d20d_disagree_or_missing | no_retest |
| a451c5daee55ffdac04ccd9a | tier1/48/3.74 | SpotCrude | 2024-11-04 | complete | 0.0101437 | -0.0656137 | complete | own_5d20d_disagree_or_missing | no_retest |
| a451c5daee55ffdac04ccd9a | tier1/28/2 | SpotCrude | 2024-11-04 | complete | 0.0101437 | -0.0656137 | complete | own_5d20d_disagree_or_missing | no_retest |
| a451c5daee55ffdac04ccd9a | tier1/28/3.74 | SpotCrude | 2024-11-04 | complete | 0.0101437 | -0.0656137 | complete | own_5d20d_disagree_or_missing | no_retest |
| 910dc7f401c979f077122690 | tier1/28/2 | SpotCrude | 2024-11-06 | complete | 0.0609924 | -0.0251784 | complete | own_5d20d_disagree_or_missing | entered |
| 910dc7f401c979f077122690 | tier1/28/3.74 | SpotCrude | 2024-11-06 | complete | 0.0609924 | -0.0251784 | complete | own_5d20d_disagree_or_missing | entered |
| 77b10f0f75d5383bb011c4b2 | tier1/48/2 | SpotCrude | 2025-04-07 | complete | -0.0955385 | -0.06065 | complete | own_and_peer_aligned | entered |
| 77b10f0f75d5383bb011c4b2 | tier1/48/3.74 | SpotCrude | 2025-04-07 | complete | -0.0955385 | -0.06065 | complete | own_and_peer_aligned | entered |
| 77b10f0f75d5383bb011c4b2 | tier1/28/2 | SpotCrude | 2025-04-07 | complete | -0.0955385 | -0.06065 | complete | own_and_peer_aligned | entered |
| 77b10f0f75d5383bb011c4b2 | tier1/28/3.74 | SpotCrude | 2025-04-07 | complete | -0.0955385 | -0.06065 | complete | own_and_peer_aligned | entered |
| 2b977b08318352c8c79f2cbc | tier1/48/2 | SpotCrude | 2025-04-08 | complete | -0.151367 | -0.0776305 | complete | own_and_peer_aligned | entered |
| 2b977b08318352c8c79f2cbc | tier1/48/3.74 | SpotCrude | 2025-04-08 | complete | -0.151367 | -0.0776305 | complete | own_and_peer_aligned | entered |
| 2b977b08318352c8c79f2cbc | tier1/28/2 | SpotCrude | 2025-04-08 | complete | -0.151367 | -0.0776305 | complete | own_and_peer_aligned | no_retest |
| 2b977b08318352c8c79f2cbc | tier1/28/3.74 | SpotCrude | 2025-04-08 | complete | -0.151367 | -0.0776305 | complete | own_and_peer_aligned | no_retest |
| b8444e851d499207d0449237 | tier1/48/2 | SpotCrude | 2025-04-09 | complete | -0.17592 | -0.115099 | complete | own_and_peer_aligned | entered |
| b8444e851d499207d0449237 | tier1/48/3.74 | SpotCrude | 2025-04-09 | complete | -0.17592 | -0.115099 | complete | own_and_peer_aligned | entered |
| b8444e851d499207d0449237 | tier1/28/2 | SpotCrude | 2025-04-09 | complete | -0.17592 | -0.115099 | complete | own_and_peer_aligned | entered |
| b8444e851d499207d0449237 | tier1/28/3.74 | SpotCrude | 2025-04-09 | complete | -0.17592 | -0.115099 | complete | own_and_peer_aligned | entered |
| 9e626ae09c741e2673fd8c78 | tier1/48/2 | SpotCrude | 2025-04-10 | complete | -0.101365 | -0.0729307 | complete | own_and_peer_aligned | no_retest |
| 9e626ae09c741e2673fd8c78 | tier1/48/3.74 | SpotCrude | 2025-04-10 | complete | -0.101365 | -0.0729307 | complete | own_and_peer_aligned | no_retest |
| 9e626ae09c741e2673fd8c78 | tier1/28/2 | SpotCrude | 2025-04-10 | complete | -0.101365 | -0.0729307 | complete | own_and_peer_aligned | no_retest |
| 9e626ae09c741e2673fd8c78 | tier1/28/3.74 | SpotCrude | 2025-04-10 | complete | -0.101365 | -0.0729307 | complete | own_and_peer_aligned | no_retest |
| ce5a75c11b08e52a205e5350 | tier1/48/24 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| ce5a75c11b08e52a205e5350 | tier1/48/2 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| ce5a75c11b08e52a205e5350 | tier1/48/3.74 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| ce5a75c11b08e52a205e5350 | tier1/28/24 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| ce5a75c11b08e52a205e5350 | tier1/28/2 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| ce5a75c11b08e52a205e5350 | tier1/28/3.74 | SpotCrude | 2025-04-11 | complete | -0.0838991 | -0.101332 | complete | own_and_peer_aligned | entered |
| 4d04e030932184cbabc7f97b | tier1/28/2 | US500 | 2024-11-05 | complete | -0.0178363 | -0.00151978 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4d04e030932184cbabc7f97b | tier1/28/3.74 | US500 | 2024-11-05 | complete | -0.0178363 | -0.00151978 | absent | own_5d20d_disagree_or_missing | no_retest |
| 8d7090e5c928d51baf5f5d28 | tier1/28/2 | US500 | 2024-11-06 | complete | -0.00831277 | 0.00584114 | absent | own_5d20d_disagree_or_missing | entered |
| 8d7090e5c928d51baf5f5d28 | tier1/28/3.74 | US500 | 2024-11-06 | complete | -0.00831277 | 0.00584114 | absent | own_5d20d_disagree_or_missing | entered |
| f35c2955c68672466ebc4e06 | tier1/28/2 | US500 | 2024-11-07 | complete | 0.0270298 | 0.0264611 | absent | own_aligned_peer_absent | no_retest |
| f35c2955c68672466ebc4e06 | tier1/28/3.74 | US500 | 2024-11-07 | complete | 0.0270298 | 0.0264611 | absent | own_aligned_peer_absent | no_retest |
| b8faf0a449b13f21ebd8db35 | tier1/28/2 | US500 | 2024-11-08 | complete | 0.043196 | 0.0345114 | absent | own_aligned_peer_absent | no_retest |
| b8faf0a449b13f21ebd8db35 | tier1/28/3.74 | US500 | 2024-11-08 | complete | 0.043196 | 0.0345114 | absent | own_aligned_peer_absent | no_retest |
| 31ac96bb6c2db5e56d25c188 | tier1/48/2 | US500 | 2025-04-07 | complete | -0.0821615 | -0.108965 | absent | own_5d20d_disagree_or_missing | entered |
| 31ac96bb6c2db5e56d25c188 | tier1/48/3.74 | US500 | 2025-04-07 | complete | -0.0821615 | -0.108965 | absent | own_5d20d_disagree_or_missing | entered |
| 31ac96bb6c2db5e56d25c188 | tier1/28/2 | US500 | 2025-04-07 | complete | -0.0821615 | -0.108965 | absent | own_5d20d_disagree_or_missing | entered |
| 31ac96bb6c2db5e56d25c188 | tier1/28/3.74 | US500 | 2025-04-07 | complete | -0.0821615 | -0.108965 | absent | own_5d20d_disagree_or_missing | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/48/24 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/48/2 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/48/3.74 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/28/24 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/28/2 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 5b35427f494f9de75bb0ddd1 | tier1/28/3.74 | US500 | 2025-04-08 | complete | -0.096114 | -0.0972758 | absent | own_aligned_peer_absent | entered |
| 7be5dc89274ab1082ba3e7ba | tier1/48/2 | US500 | 2025-04-09 | complete | -0.106489 | -0.1162 | absent | own_5d20d_disagree_or_missing | entered |
| 7be5dc89274ab1082ba3e7ba | tier1/48/3.74 | US500 | 2025-04-09 | complete | -0.106489 | -0.1162 | absent | own_5d20d_disagree_or_missing | entered |
| 7be5dc89274ab1082ba3e7ba | tier1/28/2 | US500 | 2025-04-09 | complete | -0.106489 | -0.1162 | absent | own_5d20d_disagree_or_missing | entered |
| 7be5dc89274ab1082ba3e7ba | tier1/28/3.74 | US500 | 2025-04-09 | complete | -0.106489 | -0.1162 | absent | own_5d20d_disagree_or_missing | entered |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/2 | US500 | 2025-04-10 | complete | -0.00491989 | -0.0270391 | absent | own_aligned_peer_absent | entered |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/48/3.74 | US500 | 2025-04-10 | complete | -0.00491989 | -0.0270391 | absent | own_aligned_peer_absent | entered |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/2 | US500 | 2025-04-10 | complete | -0.00491989 | -0.0270391 | absent | own_aligned_peer_absent | entered |
| 5ff6b8a3b787709ae2dcc9a0 | tier1/28/3.74 | US500 | 2025-04-10 | complete | -0.00491989 | -0.0270391 | absent | own_aligned_peer_absent | entered |
| acdc1e0257e10eefaf512a7f | tier1/48/24 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| acdc1e0257e10eefaf512a7f | tier1/48/2 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| acdc1e0257e10eefaf512a7f | tier1/48/3.74 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| acdc1e0257e10eefaf512a7f | tier1/28/24 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| acdc1e0257e10eefaf512a7f | tier1/28/2 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| acdc1e0257e10eefaf512a7f | tier1/28/3.74 | US500 | 2025-04-11 | complete | 0.00224796 | -0.0549498 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/48/24 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/48/2 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/48/3.74 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/28/24 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/28/2 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4c63892002b69ae4559435bd | tier2/28/3.74 | BTCUSD | 2024-01-23 | complete | -0.0753458 | -0.12145 | absent | own_5d20d_disagree_or_missing | no_retest |
| 46b1da2b93e2935f42ee3164 | tier2/28/2 | BTCUSD | 2024-01-24 | complete | -0.0341628 | -0.0696431 | absent | own_5d20d_disagree_or_missing | entered |
| 46b1da2b93e2935f42ee3164 | tier2/28/3.74 | BTCUSD | 2024-01-24 | complete | -0.0341628 | -0.0696431 | absent | own_5d20d_disagree_or_missing | entered |
| 4a422a139782982730a96c44 | tier2/28/2 | BTCUSD | 2024-07-25 | complete | -0.0201086 | 0.145655 | absent | own_5d20d_disagree_or_missing | no_retest |
| 4a422a139782982730a96c44 | tier2/28/3.74 | BTCUSD | 2024-07-25 | complete | -0.0201086 | 0.145655 | absent | own_5d20d_disagree_or_missing | no_retest |
| e2f8731ee1bbbd8ebec301bb | tier2/28/2 | BTCUSD | 2025-05-12 | complete | 0.0749959 | 0.190033 | absent | own_aligned_peer_absent | entered |
| e2f8731ee1bbbd8ebec301bb | tier2/28/3.74 | BTCUSD | 2025-05-12 | complete | 0.0749959 | 0.190033 | absent | own_aligned_peer_absent | entered |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/2 | BTCUSD | 2025-05-13 | complete | 0.0585643 | 0.0994048 | absent | own_5d20d_disagree_or_missing | entered |
| 4350fd4ccc6e7fa2e964e43c | tier2/28/3.74 | BTCUSD | 2025-05-13 | complete | 0.0585643 | 0.0994048 | absent | own_5d20d_disagree_or_missing | entered |
| ea65ff19fabbc0109ea7c967 | tier2/28/2 | BTCUSD | 2025-05-14 | complete | 0.00827615 | 0.111697 | absent | own_5d20d_disagree_or_missing | no_retest |
| ea65ff19fabbc0109ea7c967 | tier2/28/3.74 | BTCUSD | 2025-05-14 | complete | 0.00827615 | 0.111697 | absent | own_5d20d_disagree_or_missing | no_retest |
| 9ce2dd036808138bac5852eb | tier2/28/2 | BTCUSD | 2025-05-15 | complete | 0.00498519 | 0.101088 | absent | own_5d20d_disagree_or_missing | no_retest |
| 9ce2dd036808138bac5852eb | tier2/28/3.74 | BTCUSD | 2025-05-15 | complete | 0.00498519 | 0.101088 | absent | own_5d20d_disagree_or_missing | no_retest |
| ae497d730dbeacf465872233 | tier2/28/2 | BTCUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| ae497d730dbeacf465872233 | tier2/28/3.74 | BTCUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| daaa576e731c68edb339d1a8 | tier2/28/2 | BTCUSD | 2025-10-08 | complete | 0.00628445 | 0.0417494 | absent | own_aligned_peer_absent | entered |
| daaa576e731c68edb339d1a8 | tier2/28/3.74 | BTCUSD | 2025-10-08 | complete | 0.00628445 | 0.0417494 | absent | own_aligned_peer_absent | entered |
| 14cfdcad3095953593d732bf | tier2/28/2 | ETHUSD | 2025-07-07 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | entered |
| 14cfdcad3095953593d732bf | tier2/28/3.74 | ETHUSD | 2025-07-07 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | entered |
| 743d69735dec5f4d37cf225b | tier2/48/2 | ETHUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| 743d69735dec5f4d37cf225b | tier2/48/3.74 | ETHUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| 743d69735dec5f4d37cf225b | tier2/28/2 | ETHUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| 743d69735dec5f4d37cf225b | tier2/28/3.74 | ETHUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| f3ec49598c955968c2cdec80 | tier2/28/2 | NAS100 | 2024-01-22 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | entered |
| f3ec49598c955968c2cdec80 | tier2/28/3.74 | NAS100 | 2024-01-22 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | entered |
| 7048b77cb373429eafea7db7 | tier2/28/2 | NAS100 | 2024-01-24 | missing_segments | Unavailable | Unavailable | absent | own_5d20d_disagree_or_missing | no_retest |
| 7048b77cb373429eafea7db7 | tier2/28/3.74 | NAS100 | 2024-01-24 | missing_segments | Unavailable | Unavailable | absent | own_5d20d_disagree_or_missing | no_retest |
| dae3cca41d4e71510881f096 | tier2/28/2 | NAS100 | 2024-01-25 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| dae3cca41d4e71510881f096 | tier2/28/3.74 | NAS100 | 2024-01-25 | missing_segments | Unavailable | Unavailable | missing_segments | own_5d20d_disagree_or_missing | no_retest |
| d2dea2acc6abeda4dd84ac5c | tier2/28/2 | SpotBrent | 2023-07-24 | complete | 0.0254533 | 0.084108 | complete | own_and_peer_aligned | no_retest |
| d2dea2acc6abeda4dd84ac5c | tier2/28/3.74 | SpotBrent | 2023-07-24 | complete | 0.0254533 | 0.084108 | complete | own_and_peer_aligned | no_retest |
| 9beae18a5071ebb3f315a848 | tier2/28/2 | SpotBrent | 2023-07-25 | complete | 0.0551825 | 0.110581 | complete | own_5d20d_disagree_or_missing | entered |
| 9beae18a5071ebb3f315a848 | tier2/28/3.74 | SpotBrent | 2023-07-25 | complete | 0.0551825 | 0.110581 | complete | own_5d20d_disagree_or_missing | entered |
| c8420ab56a4df29d24c8d1b5 | tier2/28/2 | SpotBrent | 2023-07-26 | complete | 0.046293 | 0.14778 | complete | own_5d20d_disagree_or_missing | no_retest |
| c8420ab56a4df29d24c8d1b5 | tier2/28/3.74 | SpotBrent | 2023-07-26 | complete | 0.046293 | 0.14778 | complete | own_5d20d_disagree_or_missing | no_retest |
| d7c8c21dd5d0ad94aed42504 | tier2/28/2 | SpotBrent | 2025-02-11 | complete | 0.00630585 | -0.0467699 | complete | own_5d20d_disagree_or_missing | no_retest |
| d7c8c21dd5d0ad94aed42504 | tier2/28/3.74 | SpotBrent | 2025-02-11 | complete | 0.00630585 | -0.0467699 | complete | own_5d20d_disagree_or_missing | no_retest |
| affe109b180bfeb08d170ec7 | tier2/48/2 | SpotBrent | 2025-05-15 | complete | 0.0679605 | -0.00252936 | complete | own_5d20d_disagree_or_missing | entered |
| affe109b180bfeb08d170ec7 | tier2/48/3.74 | SpotBrent | 2025-05-15 | complete | 0.0679605 | -0.00252936 | complete | own_5d20d_disagree_or_missing | entered |
| affe109b180bfeb08d170ec7 | tier2/28/2 | SpotBrent | 2025-05-15 | complete | 0.0679605 | -0.00252936 | complete | own_5d20d_disagree_or_missing | entered |
| affe109b180bfeb08d170ec7 | tier2/28/3.74 | SpotBrent | 2025-05-15 | complete | 0.0679605 | -0.00252936 | complete | own_5d20d_disagree_or_missing | entered |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/2 | SpotBrent | 2025-07-22 | complete | 5.77492e-05 | 0.0105624 | complete | own_5d20d_disagree_or_missing | entered |
| c4de64f1ffa0ef8e3de8fc0a | tier2/28/3.74 | SpotBrent | 2025-07-22 | complete | 5.77492e-05 | 0.0105624 | complete | own_5d20d_disagree_or_missing | entered |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/2 | SpotBrent | 2025-10-07 | complete | -0.0195133 | -0.0149388 | complete | own_and_peer_aligned | entered |
| 362375ec6c45fb8d7a2b7d3f | tier2/28/3.74 | SpotBrent | 2025-10-07 | complete | -0.0195133 | -0.0149388 | complete | own_and_peer_aligned | entered |
| b50a68dfbdb04a30f12e9a60 | tier2/28/2 | SpotCrude | 2023-01-25 | complete | -0.0144435 | 0.00774979 | complete | own_5d20d_disagree_or_missing | entered |
| b50a68dfbdb04a30f12e9a60 | tier2/28/3.74 | SpotCrude | 2023-01-25 | complete | -0.0144435 | 0.00774979 | complete | own_5d20d_disagree_or_missing | entered |
| 8f401fb18724fa4ecb2f8a36 | tier2/28/2 | SpotCrude | 2023-07-24 | complete | 0.03407 | 0.104778 | complete | own_and_peer_aligned | no_retest |
| 8f401fb18724fa4ecb2f8a36 | tier2/28/3.74 | SpotCrude | 2023-07-24 | complete | 0.03407 | 0.104778 | complete | own_and_peer_aligned | no_retest |
| dd362c3ee13db2ddcbd9cf99 | tier2/28/2 | SpotCrude | 2023-07-26 | complete | 0.0522012 | 0.174288 | complete | own_5d20d_disagree_or_missing | no_retest |
| dd362c3ee13db2ddcbd9cf99 | tier2/28/3.74 | SpotCrude | 2023-07-26 | complete | 0.0522012 | 0.174288 | complete | own_5d20d_disagree_or_missing | no_retest |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/24 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/2 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8d44e2c3e3ecabd890265eb0 | tier2/48/3.74 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/24 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/2 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8d44e2c3e3ecabd890265eb0 | tier2/28/3.74 | SpotCrude | 2025-05-15 | complete | 0.075267 | 0.00214985 | complete | own_5d20d_disagree_or_missing | entered |
| 8970de420891b9d784ed058b | tier2/28/2 | SpotCrude | 2025-07-14 | complete | 0.0308137 | -0.0588668 | complete | own_5d20d_disagree_or_missing | entered |
| 8970de420891b9d784ed058b | tier2/28/3.74 | SpotCrude | 2025-07-14 | complete | 0.0308137 | -0.0588668 | complete | own_5d20d_disagree_or_missing | entered |
| 35ba9e2c232645825deb093d | tier2/28/2 | SpotCrude | 2025-07-22 | complete | 0.00146597 | 0.00380844 | complete | own_5d20d_disagree_or_missing | entered |
| 35ba9e2c232645825deb093d | tier2/28/3.74 | SpotCrude | 2025-07-22 | complete | 0.00146597 | 0.00380844 | complete | own_5d20d_disagree_or_missing | entered |
| 5aca717f89d879a30446d2ed | tier2/28/2 | SpotCrude | 2025-10-07 | complete | -0.0235708 | -0.0184639 | complete | own_and_peer_aligned | entered |
| 5aca717f89d879a30446d2ed | tier2/28/3.74 | SpotCrude | 2025-10-07 | complete | -0.0235708 | -0.0184639 | complete | own_and_peer_aligned | entered |
| 876f6148e2ef9692e251c46d | tier2/28/2 | US500 | 2023-01-23 | missing_segments | Unavailable | Unavailable | absent | own_5d20d_disagree_or_missing | no_retest |
| 876f6148e2ef9692e251c46d | tier2/28/3.74 | US500 | 2023-01-23 | missing_segments | Unavailable | Unavailable | absent | own_5d20d_disagree_or_missing | no_retest |
| 714dcd4ea643c7abb6c67ecd | tier2/28/2 | US500 | 2023-01-24 | complete | 0.00485061 | 0.0455811 | absent | own_aligned_peer_absent | entered |
| 714dcd4ea643c7abb6c67ecd | tier2/28/3.74 | US500 | 2023-01-24 | complete | 0.00485061 | 0.0455811 | absent | own_aligned_peer_absent | entered |
| 40cc399289d0861550320dd8 | tier2/48/24 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| 40cc399289d0861550320dd8 | tier2/48/2 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| 40cc399289d0861550320dd8 | tier2/48/3.74 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| 40cc399289d0861550320dd8 | tier2/28/24 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| 40cc399289d0861550320dd8 | tier2/28/2 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| 40cc399289d0861550320dd8 | tier2/28/3.74 | US500 | 2023-01-26 | complete | 0.0289161 | 0.0546586 | absent | own_5d20d_disagree_or_missing | entered |
| f8992619aab40fdf9fa815cf | tier2/28/2 | US500 | 2023-01-27 | complete | 0.0373825 | 0.0628289 | absent | own_aligned_peer_absent | entered |
| f8992619aab40fdf9fa815cf | tier2/28/3.74 | US500 | 2023-01-27 | complete | 0.0373825 | 0.0628289 | absent | own_aligned_peer_absent | entered |
| 001f989aba50039f6db56506 | tier2/28/2 | US500 | 2024-07-24 | complete | -0.00923565 | 0.0176913 | absent | own_5d20d_disagree_or_missing | no_retest |
| 001f989aba50039f6db56506 | tier2/28/3.74 | US500 | 2024-07-24 | complete | -0.00923565 | 0.0176913 | absent | own_5d20d_disagree_or_missing | no_retest |
| ff90f42b6db284ffa5fee37a | tier2/28/2 | US500 | 2024-07-25 | complete | -0.0312177 | -0.00847148 | absent | own_5d20d_disagree_or_missing | no_retest |
| ff90f42b6db284ffa5fee37a | tier2/28/3.74 | US500 | 2024-07-25 | complete | -0.0312177 | -0.00847148 | absent | own_5d20d_disagree_or_missing | no_retest |
| 0c8d1df35ca48a63714fb422 | tier2/28/2 | US500 | 2024-11-04 | complete | -0.0206769 | -0.00122096 | absent | own_aligned_peer_absent | entered |
| 0c8d1df35ca48a63714fb422 | tier2/28/3.74 | US500 | 2024-11-04 | complete | -0.0206769 | -0.00122096 | absent | own_aligned_peer_absent | entered |
| 2c7fe293cc545089e134e08c | tier2/28/2 | US500 | 2025-02-12 | complete | 0.00762547 | 0.0257572 | absent | own_aligned_peer_absent | entered |
| 2c7fe293cc545089e134e08c | tier2/28/3.74 | US500 | 2025-02-12 | complete | 0.00762547 | 0.0257572 | absent | own_aligned_peer_absent | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/24 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/2 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/48/3.74 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/24 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/2 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| 4d12cf15cd8e08baa46e7d29 | tier2/28/3.74 | US500 | 2025-05-12 | complete | 0.00262328 | 0.0365769 | absent | own_5d20d_disagree_or_missing | entered |
| cf4371537a3ed53252478781 | tier2/28/2 | US500 | 2025-05-13 | complete | 0.0448135 | 0.0799076 | absent | own_aligned_peer_absent | no_retest |
| cf4371537a3ed53252478781 | tier2/28/3.74 | US500 | 2025-05-13 | complete | 0.0448135 | 0.0799076 | absent | own_aligned_peer_absent | no_retest |
| f3fb5435535d98dde263ca12 | tier2/28/2 | US500 | 2025-07-21 | complete | 0.00685742 | 0.0543327 | absent | own_aligned_peer_absent | no_retest |
| f3fb5435535d98dde263ca12 | tier2/28/3.74 | US500 | 2025-07-21 | complete | 0.00685742 | 0.0543327 | absent | own_aligned_peer_absent | no_retest |
| 25e98df1d7dd285726528311 | tier2/28/2 | US500 | 2025-10-09 | complete | 0.00387724 | 0.0310803 | absent | own_5d20d_disagree_or_missing | no_retest |
| 25e98df1d7dd285726528311 | tier2/28/3.74 | US500 | 2025-10-09 | complete | 0.00387724 | 0.0310803 | absent | own_5d20d_disagree_or_missing | no_retest |
| ed2e5b33c6527114311316f0 | tier2/28/2 | XAGUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | complete | own_5d20d_disagree_or_missing | entered |
| ed2e5b33c6527114311316f0 | tier2/28/3.74 | XAGUSD | 2025-07-14 | missing_segments | Unavailable | Unavailable | complete | own_5d20d_disagree_or_missing | entered |
| b25d14035008c3a9fb364d7b | tier2/28/2 | XAUUSD | 2023-01-26 | complete | 0.017651 | 0.0741775 | absent | own_5d20d_disagree_or_missing | no_retest |
| b25d14035008c3a9fb364d7b | tier2/28/3.74 | XAUUSD | 2023-01-26 | complete | 0.017651 | 0.0741775 | absent | own_5d20d_disagree_or_missing | no_retest |
| 10d1ae4831eddd0539ba98b8 | tier2/28/2 | XAUUSD | 2024-01-23 | complete | -0.0143819 | -0.0208665 | missing_segments | peer_context_missing | no_retest |
| 10d1ae4831eddd0539ba98b8 | tier2/28/3.74 | XAUUSD | 2024-01-23 | complete | -0.0143819 | -0.0208665 | missing_segments | peer_context_missing | no_retest |

## §10. Complete 263-definition register

| Episode | Symbol | Date | Receipt history | Phase 3 tier | Phase 2 reasons | Phase 3 exclusion / alias | Failed M30 overlaps | Receipt SHA256 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| a3281b32cf61af3a41b4fb3a | BTCUSD | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 117c6759d19f8a34b4da145b067fcc4375b2cc1532d378c55b485e40c0f5e08b |
| ae497d730dbeacf465872233 | BTCUSD | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 5dcd99cab23cd9fc87afb55d5adc85fbcaac226d432f460cdc7ad1d50a1475ff |
| 14cfdcad3095953593d732bf | ETHUSD | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 86ffc00c6e20541e24d04c0d9f5254f7a0d2c9b653ca7e40278414b1f2d4eff3 |
| 743d69735dec5f4d37cf225b | ETHUSD | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | b85fe0567ce7bbbd2fe3094017a075851c273f98f8fef971365d031ee79bdf07 |
| 71c12956d026bd69a24d715f | NAS100 | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 1c5dae0c78981a5fb67fd4c006d9ec104227a5e551f90f7ef7f9ab71e4085fc2 |
| 00b72c8130b8d25638b1b9b8 | NAS100 | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | afa1bec8d1c7d60839b96c1f1806d22e007ead6bc52a3b364469a449cfbfbd37 |
| 835d49e718917e037ce90b18 | SpotBrent | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 93342c85bbe593171ff61bc1a7d0607d39dda63903e119d9ae6acdd4fb44eef2 |
| 48294fd137584a2725661351 | SpotBrent | 2025-07-14 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 560a4c7341a6d5a551ec7a18f3380cdfb7fd3649fbc55a19f4b89cb245f7a710 |
| a3f3c4991fe881e8160596d7 | SpotCrude | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | cef26bc31b41532c6decacdcad37e933dd8f665310c664478fbab9031e9b1932 |
| 8970de420891b9d784ed058b | SpotCrude | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 34e834b8b1871257c9f72bf83e1b7e5fdc6737f4ec395c0d2ca5ae9735743054 |
| 136bcb5c0c607b3895112788 | US500 | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c1b1ccd1216ec1e80945f7ab6f703c393a127243d4ef24faf1c59bf4578d73a6 |
| 068cb592754e6491f5762257 | US500 | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 051445d6f45c01b63f8e6521b8cf012a08ace52684c81bdc6e955ae1941f7c69 |
| 310539d714446e4a4c51da87 | XAGUSD | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 133691444396549beca81572466347d09ceb8cb9400095870a33438c6b6f1092 |
| ed2e5b33c6527114311316f0 | XAGUSD | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 0 | 64b6807ad42a2a129bce098a4b182e1cfc470040e472349fe573b8c812bd57c6 |
| 7106ba337a214ad20426e39f | XAUUSD | 2025-07-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 8de445ab80481811f8d991a8bd9dd745462c600584d2ccda38a52e7d8364d041 |
| 68f8dac888fe05b1d594711a | XAUUSD | 2025-07-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | ff676172ccb3a0dac372791b3162c9af3d496c5483a885d23f96d5ed05102443 |
| a69d98a9b0857fba4468653a | BTCUSD | 2025-09-12 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 955965b8f186e6240cd7436c5cd45c8bcff958fc883ec09ce67265cb323f81f0 |
| c891a278d9291a11a5ed214f | BTCUSD | 2025-09-13 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 9faae412646fe70d59df6607d814514513a3ca4dba41c896c1ab213b2f640a81 |
| c065a06364ed5a3f4d4daff8 | BTCUSD | 2025-09-14 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | d8514226607a2c3c95bafc199d4db3d08a59d7f6ea769f8dc04dd542808870db |
| 13413d5eeba6ae5b95025343 | BTCUSD | 2025-09-15 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 6a1da8663851665ac829b2207c0ab16669d0ca8846217e6c0bd2d689f308270a |
| 3167b31ba20ab20de427348f | BTCUSD | 2025-09-16 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | a5cc85a1eedf059f6107a49b183d2707733de5d0ec4ec09bb2f994ac0608e739 |
| 022ba7185edee7254c4d8857 | BTCUSD | 2025-09-17 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 3ee88cfa75684317d3283033623edd3aa571791c4581901f6649db8e4f2ef9b1 |
| 4565a5781bcced588063121c | BTCUSD | 2025-09-18 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 071016d656d84d9e4b5eb51d5e10ba77e4bb49d2388488340cc2b32c0e97a03b |
| bad969c61c67c207ed088029 | BTCUSD | 2025-09-19 | unverified_receipt_history | excluded | missing_m30_preparation_for_symbol_or_cohort, unverified_receipt_history | receipt_provenance_not_clean_first_read | 0 | 77680d87489328c8443f17fe1cead246ad589a07e04403ab63f6ac9043178750 |
| cdb6706f8187637f3fe3352d | SpotBrent | 2024-11-04 | clean_first_read | tier1 | qualified |  | 0 | 2c526b1cc56970fc873f773d59dc206ab0df482c8a40a495d33823c07db70e24 |
| 3609573a2a98577036cc248d | SpotBrent | 2024-11-05 | clean_first_read | tier1 | qualified |  | 0 | 407cbb2d1a50244019fdbad9e0612c175f9f40d168e2741916ca2bfc0fd681c0 |
| 7e1f8994fecf745690603eeb | SpotBrent | 2024-11-06 | clean_first_read | tier1 | qualified |  | 0 | d2c18210726a4eb66690e022ba4b3900b534dbdd123db025691f3eb08afbd15f |
| 2dc02ac0109a8eeb3762a22c | SpotBrent | 2024-11-07 | clean_first_read | excluded | missing_fresh_open_quote | missing_fresh_open_quote | 0 | c2ea4814ad7ee9ad8417195187e59e72d032e105bc0f8a83448dda4be9c4fb83 |
| d7ce2442e8c5109477cfdd52 | SpotBrent | 2024-11-08 | clean_first_read | excluded | missing_fresh_open_quote | missing_fresh_open_quote | 0 | d1235d74c4f12c79d4114fb7d1f5cf1abe196b8bea275b522f5b3d197c3fe6b3 |
| a92fb0ae4794ae5899abbb1d | SpotBrent | 2025-04-07 | clean_first_read | tier1 | qualified |  | 0 | ee4ced7cf0e2c8b999915bca5be17a35b290ed3ec0bc8d456a76becff239cadd |
| f5233c2f0a824dfd2f998e89 | SpotBrent | 2025-04-08 | clean_first_read | tier1 | qualified |  | 0 | 0219cd9ef4b31d5801673b0dbcdf53cd78a8094cf8dca6aeec54c5da30da8e5c |
| c3a0d383d87130df509791fa | SpotBrent | 2025-04-09 | clean_first_read | tier1 | qualified |  | 0 | 9447f81e8018fde261a7cc859d9aed0fa0037d44773b8f9d31d75ff6e03fc043 |
| 1f6ef75ca564ffa87d523f97 | SpotBrent | 2025-04-10 | clean_first_read | tier1 | qualified |  | 0 | 90be67edc55602d52758d5fbc630187cc729318aa53b5d887bd9dabd42a3e45e |
| 70e3acf0ef22a39ed2354106 | SpotBrent | 2025-04-11 | clean_first_read | tier1 | qualified |  | 0 | acff5e90d2efd99da0435944edfa9f5ffd6bedbee21a821ba104a1283af713f8 |
| a451c5daee55ffdac04ccd9a | SpotCrude | 2024-11-04 | clean_first_read | tier1 | qualified |  | 0 | 91ea71a536a137035064d097170567b52ee7b20ebeeeff5e585f38aaee0a9d29 |
| e060f88cb7dc74c96c07b68b | SpotCrude | 2024-11-05 | clean_first_read | tier1 | qualified |  | 0 | 5cd00dbdaf973a1391567cb93a7a3184246b09ac4b0078a804b89a5b2fd59af0 |
| 910dc7f401c979f077122690 | SpotCrude | 2024-11-06 | clean_first_read | tier1 | qualified |  | 0 | 0d1237422548e6820222c46b7d96858eb3416a2f81b52edf201049f5e94bb4e5 |
| 0f95ad7a94031afd0dea3e9d | SpotCrude | 2024-11-07 | clean_first_read | excluded | missing_fresh_open_quote | missing_fresh_open_quote | 0 | 952bfaf323a8a5eb0014ad8e90154c3ab1a7ff474ea30d89af1810016fb3948b |
| be7e37171731f34c82648fa5 | SpotCrude | 2024-11-08 | clean_first_read | excluded | missing_fresh_open_quote | missing_fresh_open_quote | 0 | c4e548da1bddabfe75635a92c284a77c6d5e15ad3d7e4a1091c791a39e30259e |
| 77b10f0f75d5383bb011c4b2 | SpotCrude | 2025-04-07 | clean_first_read | tier1 | qualified |  | 0 | 987cb0e0a519b08c3badd6b1504c7de8653e8b81e2a22009333d679816659a94 |
| 2b977b08318352c8c79f2cbc | SpotCrude | 2025-04-08 | clean_first_read | tier1 | qualified |  | 0 | 272715beb972799b23e09991e766e60bfbc31c65a98c91672b2ae9ce94afeecf |
| b8444e851d499207d0449237 | SpotCrude | 2025-04-09 | clean_first_read | tier1 | qualified |  | 0 | d74c6f509737425855971e69a4645f1c4d53a46c629a8e51bc8de8ff8d3f98a5 |
| 9e626ae09c741e2673fd8c78 | SpotCrude | 2025-04-10 | clean_first_read | tier1 | qualified |  | 0 | 2ed69f12402cb2e69e482b610f41cb6cb0c385c72b31775a181a5be917e6d702 |
| ce5a75c11b08e52a205e5350 | SpotCrude | 2025-04-11 | clean_first_read | tier1 | qualified |  | 0 | 2098bd4c155b2aa42c393e5f623be9ea20961905142e289f0190072999fa2903 |
| 0c8d1df35ca48a63714fb422 | US500 | 2024-11-04 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 7f96f1152cd42979472d511943bc60c7329b92ccb4c73b5be2ce5f72d79537ba |
| 4d04e030932184cbabc7f97b | US500 | 2024-11-05 | clean_first_read | tier1 | qualified |  | 0 | a70f0b0938d4e79afd0bee37757a84ca0993b00b0b862a03132eb97373b18707 |
| 8d7090e5c928d51baf5f5d28 | US500 | 2024-11-06 | clean_first_read | tier1 | qualified |  | 0 | e79e3f85a7217170622aaed7a7302558e476298eec3fceaaad61a28e62e756d7 |
| f35c2955c68672466ebc4e06 | US500 | 2024-11-07 | clean_first_read | tier1 | qualified |  | 0 | 56e422e4daaa07d058b04f191a0b3c221937f6b8b0e531ba13a0f02aa426ca56 |
| b8faf0a449b13f21ebd8db35 | US500 | 2024-11-08 | clean_first_read | tier1 | qualified |  | 0 | 3a70df80573c61add6e740e987c261a1b02f3cd3e17d12994d841e12c852a219 |
| 31ac96bb6c2db5e56d25c188 | US500 | 2025-04-07 | clean_first_read | tier1 | qualified |  | 0 | dd283da9afaa65022b9d8cf8449b4fbfe978e324d8652a7a9bc12bc52e88a571 |
| 5b35427f494f9de75bb0ddd1 | US500 | 2025-04-08 | clean_first_read | tier1 | qualified |  | 0 | 8f14641b7f92c7f54a1cea1e5a2af3b5df9d0dfc360d87bce65e7398cf1fb4a5 |
| 7be5dc89274ab1082ba3e7ba | US500 | 2025-04-09 | clean_first_read | tier1 | qualified |  | 0 | ae1f6a6bce0f8c5e0c39c0debb0c467fda40a7841a7d7bf08af3a28501594b46 |
| 5ff6b8a3b787709ae2dcc9a0 | US500 | 2025-04-10 | clean_first_read | tier1 | qualified |  | 0 | 08b8d4507c71c9730b0a8ce8f76f31c2f3162ba0eb8aaa01a671efcffb058289 |
| acdc1e0257e10eefaf512a7f | US500 | 2025-04-11 | clean_first_read | tier1 | qualified |  | 0 | e3564260fd0165025ff3307b1f249b4cc9da3933ed00f72316b13bc57a0a52cd |
| caca0aae14aacd9f05311212 | XAUUSD | 2024-11-04 | clean_first_read | tier1 | qualified |  | 0 | deb202c0514d0ff1951e5621838118153a8ee5f3677ae9256ed52cc98e750fb9 |
| b92b7b077dd282ac80b2c042 | XAUUSD | 2024-11-05 | clean_first_read | tier1 | qualified |  | 0 | 1d77766c2b5d89b0307e62b481353122ccd881dc721ce0a7fbc4b01b826c43df |
| 877ac3424c0551b6f7772d6a | XAUUSD | 2024-11-06 | clean_first_read | tier1 | qualified |  | 0 | 6e006e0e3b213d8d5a69ea986c1cf68c9ef2f94d03847d789b84e2f6b8d4e5d5 |
| 569183452c60779cd669e2eb | XAUUSD | 2024-11-07 | clean_first_read | tier1 | qualified |  | 0 | d05de281de7b541c3efcc4f86b1e9541261043ce3b03a552bd678929c70bf36c |
| f741e94dbeb081106be380a1 | XAUUSD | 2024-11-08 | clean_first_read | tier1 | qualified |  | 0 | 47852c7462b86a49929cfb25fda17fdeb07bbf9852c3f7e42b426cee3d9e47f0 |
| ae86f219cd744d0fe05a351e | XAUUSD | 2025-04-07 | clean_first_read | tier1 | qualified |  | 0 | 5ed344e1f8337421e551e044b6032e70875876ab8b7d0b7d185dd3370da0d740 |
| 44e4ecb72fbb53cc14fc14da | XAUUSD | 2025-04-08 | clean_first_read | tier1 | qualified |  | 0 | c06e6d65be20a216417b2b21bf3c663f366c3be065bb69f725aff6319d5f45ee |
| 3da7750ba2a191ca753fb2e9 | XAUUSD | 2025-04-09 | clean_first_read | tier1 | qualified |  | 0 | 587907b5c3888654b705113a14d660081f09f3eee600796d80eb3a086c64c1fa |
| 46371ae1ac4c72c2b4aa725b | XAUUSD | 2025-04-10 | clean_first_read | tier1 | qualified |  | 0 | 58328740525bf0419c0f751cb2711226653b45a13633fe04b9afb103cee5a3c6 |
| d9c1023f5de634fed865d7b3 | XAUUSD | 2025-04-11 | clean_first_read | tier1 | qualified |  | 0 | ec863e218a6b5dd1bdd7378ce4d95d891da4b14883e5b1d5a9e0545537564c0b |
| f3ec49598c955968c2cdec80 | NAS100 | 2024-01-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a8052e9a0149e7a79942bfa6ee2d9f5c89a97189665ee7d70328ba5242bff4c4 |
| 74608bc5524e70321ed4bed1 | NAS100 | 2024-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 4ec652a398fe0cb96383bf75ce3597744cb10a76ea01b33755e0649fdde6991f |
| 7048b77cb373429eafea7db7 | NAS100 | 2024-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 975a73ae3d2b256b6ee8983035ff0bbeea254a22a2b9f42b2435e1249cd02fcd |
| dae3cca41d4e71510881f096 | NAS100 | 2024-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 28b16ebe3ae2cd4eb34a66e0314fe4bbc8910264a178529dab5d8e61bdb4f1ed |
| 2549b6349e4927934cea6b9a | NAS100 | 2024-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 21ab2c3a6cef97d811eae6cc3eef0d99b522b4d3e468dbf8fbfd4f778eb7e9bf |
| fd538dee2dfee62bbdfef31a | XAGUSD | 2024-01-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 27422c2d2120a23e7e560515d8e051a9b7b9d462d016bdadb94b5bea066b451b |
| a7db221453c42363d5851c6b | XAGUSD | 2024-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 35f3fd493db958aa74441fd28ea9c41f50c4ebc13a8f2fd5b8053c4fa75a820c |
| 7b4c0272869b56392daed4d2 | XAGUSD | 2024-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c2204738e50458fa171017b1358fd8e4f4ad6388ce1c560567c39f594c8b379f |
| 4688021d87094d090147d2bf | XAGUSD | 2024-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0a8136d3cf4979acc7df3f607d22f5b0372216481163df12b18a58b803b49d0e |
| 9d283ed54947635561710d02 | XAGUSD | 2024-01-26 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote, missing_fresh_30m_quote | 0 | 72f08f3c4bea6d568cff6e2ec7ed12633b06ccfe936229854c9800214f81156a |
| 6381f9217d6a5d1c2c3794b4 | BTCUSD | 2024-01-22 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | c5446472c233fd4aa0421eb22ceb6a338dc6789494c1ca1033d257d6b448d789 |
| 4c63892002b69ae4559435bd | BTCUSD | 2024-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 62e8d065660ec65e8a1c64597327c45e8cfd648a2dc97ceb15288a29031761c7 |
| 46b1da2b93e2935f42ee3164 | BTCUSD | 2024-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 34a3a9b492ae2541c39e77a94188bd77f12547bbb49710a39880bada371d7c96 |
| bf44e6b3bcb58bb2906fe359 | BTCUSD | 2024-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 13ed2c9c098d3ba77d39ff7f5725206f462c35a75aa4920d3f95e8caec72ce72 |
| 83bd140279a8e96c93fb6105 | BTCUSD | 2024-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a39cf519e5f43fd78d78794feba20f2d1f60faaed838cf01619ab3e1b14215d0 |
| 8199d9cbcec7df48ffc053bd | BTCUSD | 2024-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | e83c94b9ecabacb9a1d2660b550ada25bf41f96a6fa23fe184483f68713fc7eb |
| 46dbae667d09a4839bc21925 | BTCUSD | 2024-07-23 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_tick_coverage_at_boundary | 0 | 8716efc0e9ae35f77c98bec8f6985d48431cd9924a0f6bdb3fd47297083c9751 |
| 17a309f47a71bca0f571e4d8 | BTCUSD | 2024-07-24 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_tick_coverage_at_boundary | 0 | 5de78d3f914e275765341ba0da233cabfc2f7e8dc1765d072a44528728b7154a |
| 4a422a139782982730a96c44 | BTCUSD | 2024-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 4f648912a7e2fa8ba292f68a7755976f51bdf671e33428f729c78c821cf038be |
| 8550e593edbfb863371e3be3 | BTCUSD | 2024-07-26 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 0 | d02b47088700b2ead6b462ff68e9a805115b5ff52da60b07d91c7a52e75425df |
| e2f8731ee1bbbd8ebec301bb | BTCUSD | 2025-05-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 44d2bc43268ae8f9a3a12275248d04a7109183b50b5f6215da6340643b0104eb |
| 4350fd4ccc6e7fa2e964e43c | BTCUSD | 2025-05-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1178b78fe90ca384a598d405885444f39e18f238fcc88f1e3a6ff19356c98951 |
| ea65ff19fabbc0109ea7c967 | BTCUSD | 2025-05-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 9eafa9ddb7465af7223891cd9d97a6c085f7c2d4ff73ef190d86047d11c829eb |
| 9ce2dd036808138bac5852eb | BTCUSD | 2025-05-15 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d542dc0dd92e4cae2d2a8eaf828f40634746f4d378688870473e8565a14f4100 |
| 8a9060211292afa3d94ff18b | BTCUSD | 2025-05-16 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 9444e865da337134c72f521a3f14fa0c54178309ba3a4edd99ded940d444057c |
| 39f5596f1015e6d5706ba25b | BTCUSD | 2025-10-06 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | e69fc5cc36eb26fe202e66f1b6271ced940c4d9bc82b29896ca4ef63873437c0 |
| 5faad815c6cd7484d9d45885 | BTCUSD | 2025-10-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 5b833510e132e5da5f75e9d1391929cb09226a20b6f8269c545d14c752194d8e |
| daaa576e731c68edb339d1a8 | BTCUSD | 2025-10-08 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | e8168f91c6fe750c5b0f3635583b9f36c8d8696f093ae8fc1cef643db7b26a36 |
| 7e2b39cab272c70c3468188b | BTCUSD | 2025-10-09 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | b76c297db69b083d6fa76e2d0c8c06fbf97c1264b43a490e790aa46da33ee3ef |
| aae2661c0700eeb400008256 | BTCUSD | 2025-10-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d72502fdf16b47908a15d6cc887c27d74ad7747786c421adf7819a2b59fcfe18 |
| a51722a99e186dedf667cb58 | SpotBrent | 2024-01-22 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | 645a8d7f6a832eec869d6344f818e96bf4ad21eb5c71b55bb342e7a754bf10c9 |
| 2d22569eb65ac698f7c74c93 | SpotBrent | 2024-01-23 | clean_first_read | excluded | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 7 | d3d36220d002db665426992f98f94d1d398e5004420bf78505517da77223073b |
| 413acab1546b60736c1ea52d | SpotBrent | 2024-01-24 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | 1a28aa663f34f480f59c7a09632a863c6cfec6f6973e4cb9165c491c9386e293 |
| f2ddfe7b0570504c76b992ab | SpotBrent | 2024-01-25 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | 1c729223ba22d069667067a8a759c232801668ea740be91ec8fbc3bc97bfa002 |
| 550647596835f77c4cbbcfda | SpotBrent | 2024-01-26 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | 12a7bda287bc164ff6c50b4c0599e43697e28ecfc5f072c482838338f8ab4e89 |
| 2f64878c52b286c335f65e4a | SpotBrent | 2024-07-22 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 82031a48d908f2cd948c93fbea07f2a3e9ea3cc57cd0636d80145f118ba3c786 |
| 6ebdfc6f362294e0e607bf79 | SpotBrent | 2024-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 86444c07efccefb1fdf3b6678a8ee6510022f03a552ac1efaef85bc9161bc782 |
| 1a897af29e15f58be0c69494 | SpotBrent | 2024-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 93e283b3a78d62886bd5d0df32e114c30ebd143e0dc1e1c0c13b820c99d46c80 |
| b6074168c8eb0dc0e2f8ae9a | SpotBrent | 2024-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6a543b783908ff9f0fb40c710c6b606c458f338fd0d88856966a5207c77b05aa |
| c6668faccec1c79439413775 | SpotBrent | 2024-07-26 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote, missing_tick_coverage_at_boundary | 0 | d8a664a4ed9e61fccce4644b77a5442e3de0cda0ae487177a83885d8c4ff4b18 |
| 3ddc5b55e115b2e2e1936c1c | SpotBrent | 2025-05-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | f3849f3cae519caac67d14f34246dd02aba4c3d704cd27ccbae327c4c2fae503 |
| e784ecf99abd10f7c3a1ae30 | SpotBrent | 2025-05-13 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 51940641d5ef7c9139113ab6f0bf6c613fbc967dc2af30549a9817137af1bbc4 |
| 8b39ab6c712fb7c7397fdfdb | SpotBrent | 2025-05-14 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 4e4b7af1e2d5691acbf9c286bac496b2809ebdb9ad0c8548518bca7c54e078b5 |
| affe109b180bfeb08d170ec7 | SpotBrent | 2025-05-15 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 733c776829bf9610767fc0b20897692cffca484a4e6272c50440ae2a0d5e585a |
| e77a82873dfae309f20bfbc8 | SpotBrent | 2025-05-16 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 0 | 5d3b49189d634bb3fdefeb5d92c5b41ee63b0c6a7e4d3b51c2e64f22d5d3d7a7 |
| 0d8adf49267297042d98d06a | SpotBrent | 2025-10-06 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1ac86a109457badcda1580e4abdcf9131ec011b3d43c56fbf75f3c44f4f1d999 |
| 362375ec6c45fb8d7a2b7d3f | SpotBrent | 2025-10-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | dfafce1eed0f81666d908013b34069d819388f4fe2755a21bde7ec5a72c1915f |
| a859f91f6cb95922ddcbacb1 | SpotBrent | 2025-10-08 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote, missing_fresh_30m_quote | 0 | a5ea210c4d7dada43824a27f3fd9170cdad333c0733ce18e2e9be8a8ce0131ce |
| c905518f0136641e39bad9c5 | SpotBrent | 2025-10-09 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 48c3497b593dd557931c62756cf5f89ac7b40687e4b7012706f2398c847ff827 |
| 64641fa516e0860f2ba2f3ec | SpotBrent | 2025-10-10 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | e1152e733759759453a44380401f9a53680f1984e8920393c3f8af05ac687767 |
| ab6aca1489429fe387e656d0 | SpotCrude | 2024-01-22 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | 802484c1bd97d2414448ee2576305d3f329b27987e6a49a51355f1b5f745a028 |
| 8507537493997a182445c464 | SpotCrude | 2024-01-23 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | a87cb06568bce85039e4f928aef6dc685d47ef006f6fcc9d7031d6a7325d604c |
| 4a7617b76a56942a450d6768 | SpotCrude | 2024-01-24 | clean_first_read | excluded | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 7 | 34235262c5724e1bf35e66a7957ec2b4de7a93b03b9ab706f37f17cb9fb1fcf9 |
| d1bc6b75ea34848580a14441 | SpotCrude | 2024-01-25 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | c254ed9e8db7168bfc9625ca298b16751d8a9a2c0279f3c98358caf85805b5e4 |
| 83dde1f2c6ae2b80589433bf | SpotCrude | 2024-01-26 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort |  | 7 | fc52c2eb4616116934976dda3fd37eb1de467c45898c16849a419aed2ef544a7 |
| 20abc09e536b839a6e7fe9c8 | SpotCrude | 2024-07-22 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 7a0af499b39ef32b298e61fab104c1d4d8dd42428527d686c6eac44122db513d |
| e48e81f1c2a54ffeea4d2c00 | SpotCrude | 2024-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6b77c326d4101dcbae9ba5df1f09caa565e79e92af3a39b7b1fcdaa5223c655b |
| 86d05af4f69296fddf2b69d9 | SpotCrude | 2024-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | e9ddf96bf0df79932ae1604f7dd8d352319d29c2bc50f6bb9cec35f00a6f51b2 |
| 13d1b94f806804837755c3fb | SpotCrude | 2024-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c74c522125b62314e96dda1ab915f9fe4accc945673431afb05ecdea1fa556a4 |
| 16b8f2818209bcec0d6c2761 | SpotCrude | 2024-07-26 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote, missing_tick_coverage_at_boundary | 0 | 29f4c99f149e0e99b28a8ebe4e6bdd31799b51c3aced9524b3c0a4f917adda0a |
| b29542347402bab6234839d6 | SpotCrude | 2025-05-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | f8f0ed0f6faa7e91efaff837e8e3ed22c7ab6eb686bbc01a79ec93acad092a3f |
| d0e4ba21f85ecdbfbfe40a8b | SpotCrude | 2025-05-13 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 0f69baa8d3e90184969ae465a5fbec5cde5fc206d105260541bb60e1693a0d92 |
| 9c22155b1c7571aa5a99b515 | SpotCrude | 2025-05-14 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 60aba46f1fe94cf8935ea514b8d807ea0c5051b4cfd1b39b73024178b4271227 |
| 8d44e2c3e3ecabd890265eb0 | SpotCrude | 2025-05-15 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 098f1261d1e6aa8d5c7ee6d965edc3a064021da5c4880344ed483c01a6c9c6cf |
| 27771eec2fd461a5bb8b89fc | SpotCrude | 2025-05-16 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 0 | a0888de0cb3e138bfe6bee33548371505f4c7165fc8be0223647da1f65224f62 |
| 134bb1b1c5cae3963b7a4c6c | SpotCrude | 2025-10-06 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1380ed339cb54aa11552bac0a21e37371166803610af0669bab5a6cccee1d3ed |
| 5aca717f89d879a30446d2ed | SpotCrude | 2025-10-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 3ff78383c62605fcc7ad0ad3de1333ffe373a3a3dc871cb7192daca1a90b83da |
| 7c936f35c2bb48764b26236a | SpotCrude | 2025-10-08 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0f7ed898aaf5f170da7b556c4d253654bca3f0b8a44a481857abaf899ba3ce5c |
| 5294e3691e86e4143000b304 | SpotCrude | 2025-10-09 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0a02a46d236ae8e68e8c0ff86a8081d23fc81ab2aede04b02e278797463da394 |
| 0d43b0a72fed3835f3c8e4ba | SpotCrude | 2025-10-10 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | a7f97fe1c3e8ea7550beb8431551bfa9ffcc71a3b1dedca4564297d1b87142aa |
| 4fe286efdb366d24671e8115 | US500 | 2024-01-22 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 7 | a995295de93f08f85eea305a46efa8fd97b6018810696a639b499e4c402a5aca |
| 845b548701e7627dc3f53be3 | US500 | 2024-01-23 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 7 | 6fa79b15c3914b1add3f7212084060340ec2bef1ead979d30bb863a83875c513 |
| c544f73b7b6de1593d3c133a | US500 | 2024-01-25 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 7 | 121a54b12f1dbaf72ece2d0e8d3ed3665124cc72e3df1b2c827f2f85c3080128 |
| e56cc14a79f4db75fde05223 | US500 | 2024-01-26 | clean_first_read | tier2 | failed_m30_identity_or_truncation_overlap, missing_m30_preparation_for_symbol_or_cohort, missing_opening_M30 |  | 7 | 97d4fee2560f62a5fbc6ed03a905df905669374574d6e901033b32458e0652c3 |
| 6681fcc669deff45d1f60f88 | US500 | 2024-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | bc766d85ccd6e878370deba8274139ab43a5eca557cb82d6996f4612fc338b12 |
| 04ea9a8aed2ded217967b9a0 | US500 | 2024-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 85f0d62c8e91ceac5b7c9fb02dcf272a9b2ad365234bb3e183fc55aaaf068efc |
| 001f989aba50039f6db56506 | US500 | 2024-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | ea484433f35228c42f295089dc5edeca3ffe457340fc43ccb07004e21eaae023 |
| ff90f42b6db284ffa5fee37a | US500 | 2024-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 51899a0406e2b6fcee737fe01e6adba805bbeaf7e427a47887b331a9bda6c6ed |
| 9856b173f9625f9053df2d6a | US500 | 2024-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 87941d5857bbf13c08ff16937579c3f0edb910f928ec47a5d693e2eb4e8c2deb |
| 4d12cf15cd8e08baa46e7d29 | US500 | 2025-05-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | b3ef3ee6e5564c6228f0b8148dcbe33501e4b16bb8fca1408cc41e7432f8148c |
| cf4371537a3ed53252478781 | US500 | 2025-05-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0b300f8e655a29aa61286d09e951cecc2db6032c4fc0375716d558b2bc956715 |
| c1bf02ca4edb813e89f28dd7 | US500 | 2025-05-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 94b7fd397a3517d7d8238de16a2911022ea6991e0ff7476f777d13377220263f |
| 7781d2ba777facd710ca1457 | US500 | 2025-05-15 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 473ac847610d75a180830341a3ee7bbb8dd6647fc25d740047501ef654ac9993 |
| 43eeb5e76c9e47a95bccd800 | US500 | 2025-05-16 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | f3f8d6b63ee068851e2e9a33629947dc99bedfb6fe80c3abe18ebc66664839ed |
| 9546b2e4fa6800370e6b787f | US500 | 2025-10-06 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | f4bf38085ed367c15e9978a0c2c6229b298aa9e42a61cf7c30b98fed3e76b234 |
| da807a1142c70b322b781e37 | US500 | 2025-10-07 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 17cf570ebbe93b10476e41bbb8f253c0cd77557c7c2bb1dd6ad3a7e68218d8b4 |
| 67e0c555d803a502cc15d6dd | US500 | 2025-10-08 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c53131c3217205efd2feb596249e87b91afbb157a2a25c0c464a50e84e08c79c |
| 25e98df1d7dd285726528311 | US500 | 2025-10-09 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 36568092cb3848f485e9cb4aa244f9c201c451f65fcc2acea6faeac42121dd27 |
| c47e497770890fb64a6335ad | US500 | 2025-10-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | ec726c5ca58ac3f175f8f891f5d54aa8c7555a2d8dc6ad59f531fe0c28c8fed5 |
| 7d29c206fcaad3e409105de3 | SpotBrent | 2023-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c957f10d59891b8424eda2d314e385d44c0080188b11c7c0ed0a28e5d1462010 |
| 6e18297f6eeed480dc9234ba | SpotBrent | 2023-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 939170be53d486591f5b7c5897474f112b733f383980e11a74715b4cb8bbb849 |
| d85383ce4fc5cc9e6b2a3788 | SpotBrent | 2023-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c033b1fc3c1f2c789c0ae9be40248357865a2276adeb934c50babf2f380b9d93 |
| 9d1d88c1aeae55f5d7850090 | SpotBrent | 2023-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c248912b7f1d0d0a5292bd30cb3f4da365b755e6b724f0b75452c2734a93c5ed |
| a6bc4b04896b9807e1b6e9c6 | SpotBrent | 2023-01-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0d67bbf3af0aadb7367543e5712f0996ef56df703f1483106261ae0fe854b1c2 |
| d2dea2acc6abeda4dd84ac5c | SpotBrent | 2023-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | eb270d01a1720a24c79c263f790627bc90f1da615700e7f88db30890a16dec95 |
| 9beae18a5071ebb3f315a848 | SpotBrent | 2023-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c0c789330a329896c00ffa5cdf753c5f2dd4701e336469cf2b632b1e72bb2339 |
| c8420ab56a4df29d24c8d1b5 | SpotBrent | 2023-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6365ba88cf5a85e58dbf8d853b5e7bc98e526ee2abadbc7f1a5b0e6668df6ad4 |
| 5da65ba1cd0c09e8b1e69062 | SpotBrent | 2023-07-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 8ad1eb5edefb7549b1eaeeb6beb671f4504f3dc854af3d2f24b4f79a1208ca3a |
| 6b43c186e44f7ce4adf78a84 | SpotBrent | 2023-07-28 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_tick_coverage_at_boundary | 0 | 3ec85e99ae00d26212d10683419f5c3cf75fed46b05f552dccd1585920ca5dd9 |
| 9e1b23b325aba77cbb703cee | SpotBrent | 2024-01-22 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | f1c8d903515b581e3b243d53604947eec8460aa78dfcbbbec8c31597b7809a3d |
| 58c200ae909a848ff926da74 | SpotBrent | 2024-01-23 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | 6e362fb0215319442e427904af1776f6789283e8fc4c1b8d2ae1a56217629cd2 |
| 01bf260784267d93ce76e221 | SpotBrent | 2024-01-24 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | c5bef04b57893f232fabee58729f240d9c386c14b7e0fec63a1ce51a2bedbe78 |
| 6397444a3039e1c4867dd15e | SpotBrent | 2024-01-25 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | 5194f35cad89143facf2c275a4ca9275c07e471f509bc54c85fc758ad4331cfc |
| 666c0bbe602d43cbf8baef30 | SpotBrent | 2024-01-26 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | c5b466eef033d50923f5ffc0f571eedfbb8bb5e3e0ce371b65ae9546345ee91a |
| c002e5ef4794429f5b842f26 | SpotBrent | 2024-07-22 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 9895178fb25ee2266fd11fce9430537a9f9804328d1594a907648d513825e2e3 |
| 8f979dc940d9b2ae211c6a90 | SpotBrent | 2024-07-23 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | fe5323f5f5e259bb52a51e6a54c235d4b68b7abfcba0390baf4ce2b43a5b1bc1 |
| a2e5c8f71ba017158d5202a6 | SpotBrent | 2024-07-24 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 8cef052a0707905522eb75f4df927d0571c4ecd8526b5a9264d61caae8c1127d |
| e5edfa214e564147e82e29b6 | SpotBrent | 2024-07-25 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 1edd535b4ab827cc9bbf5cbf171dae139d1bd9aae41760260f6063aa074c1e27 |
| d6e7cff7bd1aef63aa0d67a1 | SpotBrent | 2024-07-26 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 10e0fd2e99c7cd7bf45a4c3bee6f7c72c1553df21d8358177bf1841e311ddb9b |
| f5b1803cf39d6edeb396f06c | SpotBrent | 2025-02-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 46c38a1195301a8447149f1f158533726592b1c1d04400562b870388ebdc989d |
| d7c8c21dd5d0ad94aed42504 | SpotBrent | 2025-02-11 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 61c97c4ef51c1ce4c87f0298f5624f0d9fafe10b7074ceeda05284b477a898d6 |
| 73f72980deab43e63b63aa4e | SpotBrent | 2025-02-12 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote, missing_fresh_30m_quote | 0 | f47cf7423fd9a56c2544dc3ac95d3286c1865dc0ead7249e0d026e6aac710a1d |
| 4fe7cb2650e4b5c284d9356d | SpotBrent | 2025-02-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d55c5985751539b9f24f93cf73d1714250d6db81b756292692a5148dcea1da97 |
| 4d2806f05a3b115aa753655d | SpotBrent | 2025-02-14 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 235e78c348539f9e6d2a3ad1a9642d469ff3d3c9c787cea11722e81b05dfd2e9 |
| 0223cba302b7e83b96407428 | SpotBrent | 2025-07-21 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 237fd73dbdda5fa1d9c2dde52f6e0bbc3e003339fb3a269b13048d8355ef3086 |
| c4de64f1ffa0ef8e3de8fc0a | SpotBrent | 2025-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | b8c6e66d2a68dd0e526a975ea2c5163c0b3f02730c924e086227320b5921e97f |
| e876f67aef231b91b3c5c5a4 | SpotBrent | 2025-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0d7050bcb21d7352bf7b93339758b62e8adc1b5e974609e57aa55869e48736d5 |
| 8c5697c68cd1fc74ace69625 | SpotBrent | 2025-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 4e69154e34de5ebfdd5acbc1136ceda9c4bb61c02e800f1636494a5273e0eef4 |
| 258b031ea15e984f9aa6c571 | SpotBrent | 2025-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 7f9b7a033a0c9d4193ab4c156bd2d1d2c9f4f82ae7c816a7b1ee3d6287117a46 |
| c3e3b322f89aa16d477fe5d7 | SpotCrude | 2023-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 03268e06c9f87f679a6df42e8311b984e9555013e300a6e1b81bd6b5f7bb3e2d |
| 355ca1c07adee0d1aeb493fd | SpotCrude | 2023-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 98df0ecf1dcace12b7701bbace476a97010199ff3ff6a2480effab6265672bcc |
| b50a68dfbdb04a30f12e9a60 | SpotCrude | 2023-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | b3f8a2c0ef92205dc9f518669d156c9a9afd30834043089d1cf54ec70f681742 |
| c2380b0b84e765bc472a0564 | SpotCrude | 2023-01-26 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 0 | 4ea21a1e1990619054839c5a15b619d46daf2bcba89da6c19c7dd0b7793d5eb4 |
| 736039437cfe8703c32341ae | SpotCrude | 2023-01-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 98b9e12873beee0fe03cce80a30daa29b3f1c5cd74443e8a4fe86e877911b1d1 |
| 8f401fb18724fa4ecb2f8a36 | SpotCrude | 2023-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 8f155120072ac191c14374a2aa41334b324488a2f1d9f296f3d2aab0e73e9bd4 |
| a7b42dc8ec6b6738c775ba17 | SpotCrude | 2023-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 45a895905c78714f9e3f8293931cdb6bac5def9d269143641559091d27f6189e |
| dd362c3ee13db2ddcbd9cf99 | SpotCrude | 2023-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0d2dcb770c45657e0a74f5c8f3543aada50139476d0bb9aa44b1394001c6efd4 |
| 82256abfa13bcc7222b6f93d | SpotCrude | 2023-07-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0dee2ee6b826cc0a31a297c6d2c6a1dfe8aee615a9a1d99f86c36586b17200f4 |
| 09fc7226899dff17f5bcca96 | SpotCrude | 2023-07-28 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_tick_coverage_at_boundary | 0 | f752955890134a51792c2517e8ecb3c36ce5da878db8c88a65835cbf30db77d6 |
| e8eb43e304e030909039f220 | SpotCrude | 2024-01-22 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | af43b3d52c19cdbf6670f849409d6adf42862f69e43e9fc013cd2d10307f43f0 |
| 1991c7fc6471376a6cf96521 | SpotCrude | 2024-01-23 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | bd0f92b9361898c3d7c339a1c80899577bf3fb4afaf910eba9995f34db79ee49 |
| 8d0d254ad2d721727cf3c150 | SpotCrude | 2024-01-24 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | 298271a8f8642835e523ffe8cf3139ad933b9584301e73da46a9cbe930c1c7fb |
| 573e133e718b83d048cb2275 | SpotCrude | 2024-01-25 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | 97bd0f3dcdaacc396d717de24439bacbb6b98c8967626bf5eaf0b4421577df0f |
| 8b0de7c273b591bb6c13525f | SpotCrude | 2024-01-26 | inherited_repeat_interval | excluded | failed_m30_identity_or_truncation_overlap, inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 7 | 9e8cf5c18dedfd66b55b8c13aa88bead953baa48a0eb416cbe982b2bdfa83030 |
| 3e84577037ce7238ec755f2f | SpotCrude | 2024-07-22 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | e21b50bc0567c988aa20bfe39205e50fe4f71aa1a81be0ee5b24aef52a0d6ddd |
| 072399489fb15bacf30c5918 | SpotCrude | 2024-07-23 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 7a5b074019e80e96bee0aab0293a6f9634382596cf038978a7956950fdc62f24 |
| bee5886b5e233e86567cceb5 | SpotCrude | 2024-07-24 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 13cae12966689b83aa541eca478c80c6d1f4c85d96b8532ce0729d46861d6331 |
| 890b1d65a5c1e496b77e01d3 | SpotCrude | 2024-07-25 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 89bc2ddd548f8eedd82b35f61f29e343dbadcc215b77fc0ba810e5c049b4cae8 |
| 95c2666f32b2231028de74aa | SpotCrude | 2024-07-26 | inherited_repeat_interval | excluded | inherited_repeat_m30_interval, missing_m30_preparation_for_symbol_or_cohort | receipt_provenance_not_clean_first_read | 0 | 15bf5389932f2d3aa1761ec13b2badcca9a141330a2074b010516ed728227699 |
| 260d8b5043d33a0c4948f9e4 | SpotCrude | 2025-02-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a058525c2cb72aaa67a397ccc7fa5d8ef6b44df273f7b4318c8dbb141cb3ae7d |
| dfc9bcd5e87d8e9dbb62c5dc | SpotCrude | 2025-02-11 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | d28e9ffbe51ab3056e390b3844c30e8837a29da7357c386400762e247f1cd60b |
| 4e8c147c7c6baa545fddbb82 | SpotCrude | 2025-02-12 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_open_quote | 0 | a4dcc7f9767b786d7d62532b9e66c2ce362590b3775db4b796c2b1ff27c29f7a |
| 11a6a8c22334e78217761ee7 | SpotCrude | 2025-02-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a085da5581c56135b9ba6e60d4a5bbfb77dfd6694a20ed76ead09ebee434e2e5 |
| 2c4718db334d50d0069b28ca | SpotCrude | 2025-02-14 | clean_first_read | excluded | missing_m30_preparation_for_symbol_or_cohort | missing_fresh_30m_quote | 0 | 479f85d83fbc59704a9c5787bc08b06043c8005eb170770ea894239551d914f5 |
| 97d0fcd243c70f3d16342c6b | SpotCrude | 2025-07-21 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1dc882486b2cf044e4afb0aa4c6defc172c1110f170d9cfd25714fa1d89bc725 |
| 35ba9e2c232645825deb093d | SpotCrude | 2025-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 5ed5886e62cff146fe10dd5b846479e0a5040d28adf748393eb84129876239fe |
| 8157378e5e5a500e0590fb57 | SpotCrude | 2025-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 7ca0af44d6e1f52576994b9bcf8a379cc207d4010ff576fe1f7f8730228cdc73 |
| baa63ee6bb8b431a9d217332 | SpotCrude | 2025-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d73cc8e540f9b47d20ed18d9a202b5b65035f430ad9f49cc49ee795536527d47 |
| 5aff841b20cdc59bd1507911 | SpotCrude | 2025-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 54d5dff916a69971549d251372d6b0bc789a8c118cfbb99370d959c18144f5be |
| 876f6148e2ef9692e251c46d | US500 | 2023-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1b217a65b4a5ebb9aca75a541809babb7f3e50e8b1998d187684bee121d9dac6 |
| 714dcd4ea643c7abb6c67ecd | US500 | 2023-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a563d9625e25eb37569f730ed9addb02f8ca1452eb05a2c191cf7d276872b6c8 |
| 7f35dcccb70ef17e64457117 | US500 | 2023-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 4302aac7c2753119934a4d74300514c4c51e25095f8e95600d6226efa22736da |
| 40cc399289d0861550320dd8 | US500 | 2023-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | b211aa26537b6c1cfbb9f95b990b5cb95585abf69c4738e5eec481fcc5e048df |
| f8992619aab40fdf9fa815cf | US500 | 2023-01-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6bdecda65792a4cb1738670d62b09ecc78b835789b9856cc6569ef8ff8ff3da4 |
| 5acc5528cd945dc8e3fc1e27 | US500 | 2023-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 08acebbe211c13dbf2249f01cb670379d3e5a32808d864b84e1bfd5723b2a87b |
| daa7c84bfb25caac4549cc96 | US500 | 2023-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 5a87bceb0c70f950d2100eb2eb0ab357905b0bef5a44e2b26211cf6b50fcd70f |
| 7515dfa76df9ce2dd2f5c809 | US500 | 2023-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 0b9d256374713e3e1084c275c8f17b94202e215137dc5df28c47ff17b3d0285c |
| 43e781706c514672a6a96738 | US500 | 2023-07-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 5f38e7f77734210d67fd5b4bbe2b716a3fe84c6578886bdaeaa52dab4709c12e |
| a21704982a48b223406411f6 | US500 | 2023-07-28 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6b3b2309c4c1478c4a1badbf11bd055387621306369104ffc909f2b8a2a64cc5 |
| 2148bb57b5270748d1dc5857 | US500 | 2025-02-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 71de29f2775ec2e46e9bb56048fde1962dd984c0e941d6d204b4142b9efe0bd7 |
| 2fb13c0d341a2a7c4d93d305 | US500 | 2025-02-11 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 4e88f1abd2f1e6899da3105fe035700fd06cda5250008ae458893cd9c6237af3 |
| 2c7fe293cc545089e134e08c | US500 | 2025-02-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 23cdf061a359c61d4d322fdc28ee5562d2b8a2ccb90c2a6708d7dd775cc6c279 |
| d81b55b30354e68499a01af1 | US500 | 2025-02-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 01862494cd97528a41c0bd132d781549b541157f94a3fcf7a50bdee48f902537 |
| 7b74fe5bafc42e70f1ae1521 | US500 | 2025-02-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1e5469dcde7723b208fdce21588b14b3d98c4e1918b9ac43b3dd99acffd9b1a3 |
| f3fb5435535d98dde263ca12 | US500 | 2025-07-21 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 10feea1dc01815554ca09bc818a3627eb4d41f46cd911bbccb6445fba04d5b63 |
| 96de88c9fded8e055ca84165 | US500 | 2025-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 159455122bb5ca523e841e4718078231753023fff640c39041c0b7894a0abae7 |
| 3525cfcc7489cf8cd70fb13a | US500 | 2025-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 15fd4c4e06be3277de9f9f1d6b6409e4edef5a3f1b743fda0f265cfb21f4cdc5 |
| e37467317fb2c0e7cbde2e15 | US500 | 2025-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 343afa2a40a45eea51995f11dab6269bf178d4b56676c6faccad385216d888bb |
| 35aec47a684766f6889db8e4 | US500 | 2025-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 71a29c16bf5d8b2bd448af36d8d0d6bb54fc6d117d63788cf67bd7fc864a5c95 |
| a3305367b536943b295e9831 | XAUUSD | 2023-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c7612bf60cf14c5bccc3711ddc6fe24bfa8417be5fadd84814506c7e696c6ecf |
| d01543c086951e45fc2662f6 | XAUUSD | 2023-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6f47b329efb26445e2b1385e1f6f96a55eec35b18b6493c527e7da2997472617 |
| 6fcf6013c2d5308631e99a7d | XAUUSD | 2023-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 625360c277a4453b89f36aa8bfdbe93aede137bd7c20e34a5a61b6c81e18dc5b |
| b25d14035008c3a9fb364d7b | XAUUSD | 2023-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | eeac8dd134ad5eafcb8f00f4e5cfcc849c6b22ca63f1646b6a4b255eb363dd26 |
| e5c3746472e635be94b42589 | XAUUSD | 2023-01-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 57251d5fc2c730c873b82c5858fc3bd37f533ff5c7c173b7e30f7b45c1269d36 |
| 7e4329e54fa5d8930de487f8 | XAUUSD | 2023-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 820bd86b8c69d6ff5a0076a1575e7e1eb094586a500239cbce31751348463a52 |
| a5ac3d675d4a755b21393c6f | XAUUSD | 2023-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 873ce2164001b30f7091e218fe4806c9e26792e5fc436c9f25c25e63274776d9 |
| 4bf29f97bbe50d6359520302 | XAUUSD | 2023-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 06344dad2967b321f2c8e66531a496e9464dd74deb9a6ed5a2e5fffefc8eeffe |
| 301cc40952795f0f5982c6b9 | XAUUSD | 2023-07-27 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 60af852b14ff57b9df9644f03da44d8e89d2450716c742a29f1326545da87c18 |
| 00aba56cdad194a6a6e1630d | XAUUSD | 2023-07-28 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 7a9156b9c5acc0821ddc95676979c4832a12dc80c04952dd5fd0f7a7920a1a5c |
| 11bdabc3c84d460a73a8eb56 | XAUUSD | 2024-01-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 7b9e2199b5047c942e570d67acd888e108bd6fe9d5a4d86c9b7087afe2e48aed |
| 10d1ae4831eddd0539ba98b8 | XAUUSD | 2024-01-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | a95118b0d51b74b464ff9c06c44fef06713cc0ab201029c2854f1268ea80b58e |
| d35cbc2e2f670f3769850370 | XAUUSD | 2024-01-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d89ca4fffb7ad8aa3a00642c36727c5f8593a7a81e1231a3064e2db7b3ccb09d |
| 5204aa8a50ac25f4b1d6d4bb | XAUUSD | 2024-01-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 54c0a47949df1bf4797ab8f4e3f9d0ef83f8f6cd1cbf516ff218d203c5f1e2bc |
| 8cc480b98b6fa8a6e114d91d | XAUUSD | 2024-01-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | cad5ad90a6b5e4a3d9b33a6cd31a4b5377f94917f666ca17801e5db7d5b00a19 |
| 4818610971895f5197695336 | XAUUSD | 2024-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | db6c88c1b97bff263d1332f2b0ed26506df916f611ef08d1a1c2f0870024d579 |
| 1d8414a63d83d92b96f7d5c8 | XAUUSD | 2024-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 42e4b36d0a15195b7b0de47957c2c38e58878be74765e3af8577f0bf26a476b8 |
| 2fae307746859cdb6463521e | XAUUSD | 2024-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | cf8ee67433050f292c44de78ca201876f8acc41805f936bd4580f0488778ad07 |
| 78212982822d191a812f095b | XAUUSD | 2024-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 8b936523436a63b51c97919f1263e4cb0312561d5270d347e76445cb3e5cb6a7 |
| f31451969f167fdb421b9195 | XAUUSD | 2024-07-26 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 49f630faf71f5aa9acc93273d5ce82bcb1b75362493302d5ba8c112f35f8494a |
| 4ad806629abdf9d7d75ac14e | XAUUSD | 2025-02-10 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1757eb5cc7f795f3939e8f61adf0cf37c95599da8f8fca4aed895e062912a0e0 |
| 758fe8143b87051535048c5d | XAUUSD | 2025-02-11 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 6ef32b97f85b9fb768494e56ad55b87662d5ce4b386dfc3b66c87c92e4b07668 |
| e08a05e0094d27338df76a67 | XAUUSD | 2025-02-12 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d2ab8541bab2a169b3b9d1a7080fce03d0f5b192e0c60312c69cfa850c4e36ce |
| 18c4d4b660a470dcab36d784 | XAUUSD | 2025-02-13 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 632777da83845d43424cb92dd82bfb16c945369799667d9630946d4c9c2d6ef5 |
| d5f172ae04b389f87a33926f | XAUUSD | 2025-02-14 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 786ec1c5a01e616d77254006328dcf78396abf5d9e0c57a9bee33b3071b04e7a |
| 70d6f75e60e33e934c60a925 | XAUUSD | 2025-07-21 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 79de71bfd0edf161b3eac786ec243561bcb1684ac9f9b3f3a7ab23347869f218 |
| bdce29dd8539285849b9e070 | XAUUSD | 2025-07-22 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | 1b2bbccd9412c17603364c700d8b764c56d476a057c9b1ddd0e8746e3fa8dd65 |
| 196614c16843b2ea7ef2c9a0 | XAUUSD | 2025-07-23 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | c60eea7049859105176c6eede75a03483401e183f4a0ac5683ef17a95e5529de |
| b51bc946e3e8272f970a1779 | XAUUSD | 2025-07-24 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | d254554e2bb27ce3c3c6c01a71e1fa7b9406f8ea0dd9c11721064c3e84ed559a |
| 25a29bafe6cd3fdda3e5faa4 | XAUUSD | 2025-07-25 | clean_first_read | tier2 | missing_m30_preparation_for_symbol_or_cohort |  | 0 | dc86bfbb7dbcb8d97cadd10a3a4771522168af61d19ce1e1956c859c154436a0 |

## §11. Reproduce and hand over

```text
cargo build -p jiaoyi-core --example native_target_sizing_replay --example manifest_pullback_replay --example tiered_pullback_replay --example replay_inventory_metadata --locked --offline
python3 ops/research/tiered_replay.py --workdir /tmp/jiaoyi-phase3-replay --output /tmp/jiaoyi-phase3-results.json
python3 -m unittest discover -s ops/research -p test_tiered_replay.py
cargo test -p jiaoyi-core research::tiered_pullback --locked --offline
```

Use the frozen code commit for byte-for-byte reproduction. A GitHub clone contains the plan, measured outputs, report and hashes; replay additionally needs the Phase 1 manifest's original local files. Missing or changed files fail closed. Running the same committed implementation in the same work directory reuses verified native receipts and returns identical results. Later source revisions require a separate work directory.

## §12. Instructions for the independent reviewer

Read the Phase 3 plan,results and verification JSON alongside this report; compare to Phase 2 at 98fef7a. Check the 263/228 denominator, excluded provenance, Tier 2 definition, cost-admission versus fixed-path repricing, C's rejected winners and losers, and strict versus forced/continuation resolution. Do not pool policy trials or infer account qualification. Challenge the economic hypothesis and required independent evidence, not just whether CI compiles. Goals,№/§/SGT traceability, preserving proven fixes and “Time is Money” remain the invariants.
