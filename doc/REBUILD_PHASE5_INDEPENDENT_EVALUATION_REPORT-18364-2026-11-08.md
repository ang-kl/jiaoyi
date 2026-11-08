# Phase 5 — Fixed-window macro pullback: measured historical evaluation

№18,364 §§1–11 · 08-11-2026 14:33:24 SGT · by <Codex (GPT-6), effort: not exposed>

## TLDR

**This tests the owner-approved changed candidate `s3_macro_pullback_fixed_windows_v1`, not the frozen Phase 4 candidate.** The requested fixed UTC windows alter the opening reference, historical sigma and causal trend context. There are no live rule changes.

**Primary four-hour result at 24 bp/side:** 1/2 wins (50.00%), mean net R **-1.00689**, profit factor **0.0993783**. At the uncalibrated 3.74 bp sensitivity, the same chosen entries produce 1/2 wins and mean 0.00263576R. Before the extra cost scenario, mean executable-price gross R is 0.188995; the 24 bp scenario subtracts an average 1.19589R. Recorded spread is included once; fees, swap, conversion and true round-trip slippage remain unverified.

**Coverage:** 158/160 tick sources completed; **62 sessions were eligible for native evaluation**, with 98 excluded before outcomes. 239 new M30 requests, 75 cached slices reused, 18 spent slices left blocked. A completed download is not a qualified trade.

Both candidate closes carry quote-gap flags. The reported returns are replay scenarios; native execution-qualified returns remain unavailable.

**Goal progress:** 2 pooled priced candidate closes; 18 more would be needed even to reach a pooled 20-close count. This is not the per-strategy/account milestone. No account has qualified for 73% wins / 1:1.38 risk:reward; the gross target remains 1.38R, and realised net payoff may differ. Per-class 50% screens are below. No automatic extra sampling is authorised by a shortfall.

**Uncertainty:** the descriptive 95% Wilson interval for the pooled primary win rate is 9.45% to 90.55%. This treats closes as independent Bernoulli observations; correlated peers and weeks weaken that assumption. It is not an account-level confidence guarantee.

**Decision:** no financial promotion. Keep the candidate and this batch immutable. Coverage/calendar, cost calibration and class-level evidence must be assessed before any separately registered next experiment. All failures and censored paths remain in the report.

## §1. What was frozen, and what changed

The owner explicitly selected fixed UTC windows after the mismatch with Phase 4 was identified. Energy uses 13:00–17:00, indices 14:30–18:30, metals 07:00–11:00 and crypto 00:00–04:00. All are UTC; indices intentionally do not move with New York daylight saving in this changed candidate.

Each historical sigma reference and 5/20-session trend uses the same four-hour clock window: weekdays for noncrypto and every day for crypto. The existing 14-session sigma, 48 bp floor, durable full-cost veto, later fresh-quote dispatch, 50% retracement, structural risk calculation, gross 1.38R target and next-quote entry-stop amendment are retained. Last entry decision mark is 3h30m; a retest must occur before 4h. This is a disclosed change in session/context definition, not an exact replication of the 7/7 Phase 4 sample.

Primary admission remains 3.74bp/side and primary outcome pricing 24bp/side; changing admission and repricing fixed entries are different operations. The structural risk includes the admission reserve, so a cheap admission assumption affects both selection and geometry. No parameter is retuned after this batch.

## §2. Provenance and independence

Plan SHA256: `e3645201bac9598759b7986a225341ca794e900aeed666e038e2c61f83a2115b`. Code freeze commit: `b7842fc8fcbd3165c8ce07e35d023886d2b8b7a7`. Results SHA256: `6b770318ea0fb58339068acc2b1b56b91a5cb0c239bb7479ca5ea155a394dd1a`. Broker reads use demo account 49236051 and only the existing read-only message allowlist. No orders, subscriptions, live-account reads, model API calls or Telegram messages are required.

The 20 owner-selected dates have no overlap with retained development evaluation intervals. Some 45-day preparation overlaps earlier requests and is reused by verified receipt. This is frozen retrospective historical evaluation, not chronological forward validation. The 530 unavailable Phase 1 references are mixed source/code/report revisions; none is an unavailable collection-ledger reference. They must not be described as 530 proven missing tick receipts, nor does their absence prove universal broker/chat independence.

Historical broker trading calendars and contract terms remain unverified. Fixed weekday slots are a coded scenario. Missing bars may indicate a holiday, maintenance, unavailable history or a defect; they are not fabricated or removed after seeing outcomes. Failed or uncertain requests remain spent. The protected 19 August–5 September 2025 interval was excluded from every dependency.

## §3. Eligibility and data limitations

| Exclusion reason (overlapping counts) | Episodes |
| --- | --- |
| missing_45day_fixed_window_M30 | 88 |
| missing_fresh_tick_boundary | 10 |
| tick_source_failed | 2 |

All eligible/excluded decisions were recorded before any new candidate outcome. Own preparation must be complete across 45 calendar days, with a valid opening M30 bar and fresh recorded quote boundaries. Planned peer failures/missing context cause macro abstention, never a convenient peer-absent pass. Macro-data abstentions are separated from directional vetoes below.

The two failed tick jobs were NAS100 on 12 and 14 August 2024. Both exceeded the frozen 200,000-row budget; neither was retried. They also lacked required preparation, so exclusion-reason totals overlap. All 239 newly requested M30 segments returned complete receipts, but existing blocked slices and missing expected calendar slots still prevent full preparation coverage. A successful request does not imply complete historical coverage.

Continuous executable liquidity is not established by a complete downloaded interval: separate bid/ask streams can contain gaps, stale updates and unknown same-millisecond ordering. Raw quality flags are retained in the JSON. Current contract snapshots cannot certify historical minimum lots, precision changes or dollar P&L. Profit factors aggregate R-normalised returns, not statement dollars; different symbols' price units are never summed as portfolio cash.

## §4. Candidate and matched A/B controls

| Arm | Mode | Cost bp/side | Entries | Priced closes | Wins | Losses | Flat | Unresolved | Win rate | Mean net R | PF |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| arm_a_breakout_baseline | strict | 24 | 9 | 6 | 1 | 5 | 0 | 3 | 16.67% | -1.15179 | 0.0573737 |
| arm_a_breakout_baseline | strict | 3.74 | 9 | 6 | 1 | 5 | 0 | 3 | 16.67% | -0.416649 | 0.329938 |
| arm_a_breakout_baseline | strict | 2 | 9 | 6 | 1 | 5 | 0 | 3 | 16.67% | -0.353513 | 0.380095 |
| arm_a_breakout_baseline | forced | 24 | 9 | 9 | 1 | 8 | 0 | 0 | 11.11% | -0.919098 | 0.0483895 |
| arm_a_breakout_baseline | forced | 3.74 | 9 | 9 | 3 | 6 | 0 | 0 | 33.33% | -0.242538 | 0.485334 |
| arm_a_breakout_baseline | forced | 2 | 9 | 9 | 3 | 6 | 0 | 0 | 33.33% | -0.184432 | 0.571078 |
| arm_a_breakout_baseline | continuation | 24 | 9 | 9 | 3 | 6 | 0 | 0 | 33.33% | -0.793257 | 0.222877 |
| arm_a_breakout_baseline | continuation | 3.74 | 9 | 9 | 3 | 6 | 0 | 0 | 33.33% | -0.116441 | 0.784552 |
| arm_a_breakout_baseline | continuation | 2 | 9 | 9 | 3 | 6 | 0 | 0 | 33.33% | -0.0583141 | 0.883188 |
| arm_b_pure_pullback_retest | strict | 24 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.914905 | 0.0748955 |
| arm_b_pure_pullback_retest | strict | 3.74 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.0362107 | 0.916962 |
| arm_b_pure_pullback_retest | strict | 2 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | 0.0392547 | 1.10102 |
| arm_b_pure_pullback_retest | forced | 24 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.914905 | 0.0748955 |
| arm_b_pure_pullback_retest | forced | 3.74 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.0362107 | 0.916962 |
| arm_b_pure_pullback_retest | forced | 2 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | 0.0392547 | 1.10102 |
| arm_b_pure_pullback_retest | continuation | 24 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.914905 | 0.0748955 |
| arm_b_pure_pullback_retest | continuation | 3.74 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | -0.0362107 | 0.916962 |
| arm_b_pure_pullback_retest | continuation | 2 | 3 | 3 | 1 | 2 | 0 | 0 | 33.33% | 0.0392547 | 1.10102 |
| arm_c_macro_filtered_pullback | strict | 24 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | -1.00689 | 0.0993783 |
| arm_c_macro_filtered_pullback | strict | 3.74 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.00263576 | 1.00441 |
| arm_c_macro_filtered_pullback | strict | 2 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.0893375 | 1.16172 |
| arm_c_macro_filtered_pullback | forced | 24 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | -1.00689 | 0.0993783 |
| arm_c_macro_filtered_pullback | forced | 3.74 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.00263576 | 1.00441 |
| arm_c_macro_filtered_pullback | forced | 2 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.0893375 | 1.16172 |
| arm_c_macro_filtered_pullback | continuation | 24 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | -1.00689 | 0.0993783 |
| arm_c_macro_filtered_pullback | continuation | 3.74 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.00263576 | 1.00441 |
| arm_c_macro_filtered_pullback | continuation | 2 | 2 | 2 | 1 | 1 | 0 | 0 | 50.00% | 0.0893375 | 1.16172 |

`strict` counts only bracket resolution inside the tick window; open paths remain unresolved. `forced` is the primary score and uses a fresh executable boundary quote within 5s. `continuation` resumes only unresolved strict paths on recorded M30 bars, capped at 72 hours from entry. M30 is a bid-price proxy; short exits use the frozen entry spread proxy. Intrabar ambiguity, missing bars, overnight/weekend gaps and partial deadline bars remain unresolved. Continuation returns never replace primary four-hour results.

Exit anatomy of the primary candidate: `{'strict_target': 1, 'initial_stop': 1}`. Stop amendments are not automatically breakeven exits, and breakeven at entry can still lose after costs.

## §5. Per-class, symbol and week results

| Class | Eligible episodes | Closes | Wins | Win rate at24bp | Mean net R | Weeks with closes | C minus B R/opportunity | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| crypto | 22 | 2 | 1 | 50.00% | -1.00689 | 1 | 0.0332242 | insufficient/fail |
| energy | 10 | 0 | 0 | Unavailable | Unavailable | 0 | 0 | insufficient/fail |
| equity_index | 2 | 0 | 0 | Unavailable | Unavailable | 0 | 0 | insufficient/fail |
| metal | 28 | 0 | 0 | Unavailable | Unavailable | 0 | 0 | insufficient/fail |

| Symbol | Eligible | Closes | Wins at24bp | Win rate at24bp | Mean R at24bp | Wins at3.74bp | Mean R at3.74bp |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 5 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| SpotBrent | 5 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| US500 | 1 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| NAS100 | 1 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| XAUUSD | 15 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| XAGUSD | 13 | 0 | 0 | Unavailable | Unavailable | 0 | Unavailable |
| BTCUSD | 11 | 1 | 1 | 100.00% | 0.222209 | 1 | 1.19958 |
| ETHUSD | 11 | 1 | 0 | 0.00% | -2.23599 | 0 | -1.19431 |

| Week | Eligible | Closes | Wins | Mean R at24bp | Mean R at3.74bp |
| --- | --- | --- | --- | --- | --- |
| 2023_Q2_Spring | 0 | 0 | 0 | Unavailable | Unavailable |
| 2023_Q4_Autumn | 24 | 0 | 0 | Unavailable | Unavailable |
| 2024_Q1_Spring | 18 | 2 | 1 | -1.00689 | 0.00263576 |
| 2024_Q3_Summer | 20 | 0 | 0 | Unavailable | Unavailable |

The frozen per-class screen requires at least 20 closed trades across three weeks, win rate≥50%, positive stress-cost expectancy and positive C-minus-B return per eligible opportunity. Unresolved comparative paths or peer-data abstentions prevent a clean incremental-filter claim. Small weekly samples and dependence between peers limit inference. A pooled score cannot pass a class or account.

## §6. Requested always-long / own-momentum controls

| Auxiliary control | Cost bp/side | Eligible | Entries | Closes | Wins | Unresolved | Mean R | PF |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| always_long | 24 | 62 | 34 | 34 | 5 | 0 | -0.675454 | 0.148804 |
| always_long | 3.74 | 62 | 34 | 34 | 13 | 0 | -0.229765 | 0.503298 |
| always_long | 2 | 62 | 34 | 34 | 14 | 0 | -0.191488 | 0.565319 |
| own_momentum | 24 | 62 | 34 | 34 | 6 | 0 | -0.671973 | 0.167247 |
| own_momentum | 3.74 | 62 | 34 | 34 | 11 | 0 | -0.226284 | 0.517843 |
| own_momentum | 2 | 62 | 34 | 34 | 12 | 0 | -0.188006 | 0.576381 |

Both auxiliary controls use the same qualified sessions, first-hour range and next fresh quote, then a four-hour close. They retain their original 24bp admission and have no candidate bracket exits; therefore they are descriptive benchmarks, not perfectly matched entry/exit ablations. A versus B versus C is the stronger controlled comparison for entry and macro-filter effects. No control result is selected as a replacement candidate.

| B path disposition under C | Entries | Priced closes | Wins | Losses | B mean net R24 |
| --- | --- | --- | --- | --- | --- |
| own_trend_veto | 1 | 1 | 0 | 1 | -0.730932 |
| retained | 2 | 2 | 1 | 1 | -1.00689 |

## §7. Every candidate trade

| Episode | Side | Entry UTC | Exit UTC | Forced-path exit | Initial R price | NetR24 | NetR3.74 | Strict status | 72h M30 status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| BTCUSD-2024-03-04 | long | 2024-03-04T02:09:49.881Z | 2024-03-04T02:31:54Z | broker_take_profit | 263.39 | 0.222209 | 1.19958 | closed_tick | closed_tick |
| ETHUSD-2024-03-07 | long | 2024-03-07T01:03:17.942Z | 2024-03-07T01:03:50.300Z | gap_through_stop | 14.87 | -2.23599 | -1.19431 | closed_tick | closed_tick |

The JSON retains full entry/exit state, context, preparation quality, source hashes and native receipt identities. Positive boundary drift is not relabelled as a target hit. Profits from M30 approximations are not represented as broker executions.

The BTC long on 4 March 2024 entered at 63,350 with an initial risk distance of 263.39 (41.58 bp). It reached the gross target, but its stress-priced result was only +0.222209R. The ETH long on 7 March entered at 3,830.20 with an initial risk distance of 14.87 (38.82 bp), then crossed its stop; its stress-priced loss was −2.235992R. The 48 bp floor applies to the opening impulse, not the smaller post-retracement structural risk. This explains the large stress-cost burden without changing the frozen rules after observing results.

Both candidate paths contain `gap_before_quote` flags. Native qualified gross/net results remain unavailable; the reported numbers are explicitly scenario calculations from the recorded quotes. Both exits occur inside four hours, so the 72-hour continuation adds no candidate resolution. These two trades cannot establish a repeatable edge, even before unverified fees, financing and execution effects.

## §8. RFC gates and failure prevention

| Gate | Status | Evidence / gap |
| --- | --- | --- |
| G1 | partial | Source/code/calendar/contract/cost identities linked; historical authority remains unverified. |
| G2 | passed | 159 legacy and 35 Phase 2 outputs reproduced exactly before collection; original session defaults retained. |
| G3 | passed | Frozen bounded collection separated from network-denied native evaluation; failures and all 160 slots retained. |
| G4 | partial | 62 sessions qualified under coded fixed windows; 98 excluded before outcomes. Missing/ambiguous continuations are censored. |
| G5 | open | Dated historical holidays, maintenance and contract validity required before collection/qualification. |
| G6 | open | Matched broker round-trip deals, fees, swaps and conversion not available. |
| G7 | open | No out-of-sample calibrated round-trip cost error bounds; 3.74 bp remains assumption. |
| G8 | partial | 18 cached native paths and recorded quote prices reconciled; real execution/restart parity unproved. |
| G9 | partial | One owner-amended candidate and untouched retained evaluation windows frozen before collection. Preparation reused; retrospective selection and unavailable historical references limit independence claims. |
| G10 | not_passed | See fixed per-class screens; selected point estimates do not establish calibrated financial qualification. |
| G11 | not_performed | No broker shadow/canary evaluation. |
| G12 | not_demonstrated | No per-strategy/account operational goal qualification. |

Exactly 2/12 gates retain bounded engineering passes. CI verifies software behaviour; it cannot establish an economic edge. Prevent recurrence by keeping separate: frozen identity and rules; source/calendar eligibility; native replay and arithmetic; costs; independent per-class evidence; authorised shadow/live operations. No future repair may overwrite this result or replay spent broker requests as fresh evidence.

Before any next batch, address the actual excluded dependencies from the register, establish dated calendars/contracts and capture matched account-scoped round-trip costs. If the fully specified candidate fails the registered class screen, retain that failure and formulate a new economic hypothesis rather than repeatedly tuning these dates. If evidence is insufficient, publish that fact without extending until a favourable sample appears.

## §9. Verification and reproduction

- 159 legacy and 35 Phase 2 replay outputs reproduced exactly before collection.
- Two complete offline runs produced byte-identical result JSON; all 399 collection receipt hashes reconciled.
- Focused fixed-window and collection/accounting tests; full research tests and CI recorded in verification and PR.
- Native cost arithmetic checks: 117.
- Coverage freeze SHA256: `a07fbf2b353090f5e45788334e013109096c6c166333949dab9e89e40ca18264`.
- Protected validation unopened; live rules unchanged; evaluation network guard enabled.

The original sequence was `prepare`, commit the plan/code, `collect`, then `evaluate`. For offline reproduction, use a separate checkout of the frozen commit `b7842fc8fcbd3165c8ce07e35d023886d2b8b7a7` with the retained cache and exact frozen binary, then run `python3 ops/research/phase5_evaluation.py evaluate`. The runner deliberately rejects a different commit or source hash. Generate the report using `python3 ops/research/phase5_report.py` from this publication commit. No new collection is needed for reproduction.

Collection is resumable only with matching immutable receipts and code freeze; uncertain attempts fail closed. Reproduction requires retained raw files and the frozen binary, not merely the GitHub JSON. Do not execute collection again to manufacture a new independent sample. Before any future collection, union this Phase 5 plan and its receipt register with the Phase 1/4 inventories; the frozen Phase 4 registry alone is now incomplete.

## §10. Complete 160-episode register

| Episode | Cohort | Tick source | Required M30 | Missing M30 | Eligibility / macro decision | Candidate status | Tick receipt SHA256 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 797dd1b0050d11837eb0a477716e5862c8791f9924bde02db2e65b28e90e580c |
| SpotCrude-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | e73f37e24a6e8cfddcea526639764800f948cba4744f12bd9c821afd5153551d |
| SpotCrude-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | 5d5054e9d015dd02b5db25287e72d6efc138897abc3d910a13d0a1cc342d383f |
| SpotCrude-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | edd91d6fe4e41282640687549d139840782c2a9187864cf8f409a0e7ff829a5c |
| SpotCrude-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | d73910a7398373455bda111d5af61ff2ceb5b266d09a07a72d7782d9f3a4ae1f |
| SpotBrent-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | ab1d7210a74018b544b0f78e5ec3ecba87932d53e050ea46af9bf171db4f484f |
| SpotBrent-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 52c6409cce23540498e78301d3b8bf12989544e4449ba6170bc1e5a6f6369725 |
| SpotBrent-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | 81699260f9a450a60a3a9f69524ca946aec4c45c29a9a21cadc955c8d77bdf3f |
| SpotBrent-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 41e8c476f448e851a289bd92c38da9560c7ab37a81abe3313c4173ad0dfde0bb |
| SpotBrent-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 40253a8e80950a4166b6e68f4098fbe0dbcb95453e4541d3832944593250a6ad |
| US500-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 89ac2697297dd53584bc36bda53270a1db747acb3a74fb8b4fd9f3a0e8e02ea5 |
| US500-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 979dee673d23b704c50aca965aa655d7b5892e7d194e1b7201706031cdf47442 |
| US500-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | c1b3de6c2cf2e8709fdd2ae22e8274fcc30bdd9807ee7627622959269007314b |
| US500-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | efcc0bf06c63359535faac0e1e3b56aea860e86b618bb0d46f0e40284f80ef9f |
| US500-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | d381bbdaee873cbf63549b2085bb7e1cfc23de9128b3a099fff00950d1d0e55d |
| NAS100-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 31b17ffc1a6430213c579d6c933cad219c18cc73b74bbb04318dc111984080d7 |
| NAS100-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | ad207fe0da5081ceaca039bafc68f9f5881e964773e3a94afdefee1c51ea1f45 |
| NAS100-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | e427089f896ea17f58143645ee7d080239c9436e2387a0541a8169d1765b4359 |
| NAS100-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | ba272ab17f06f449dedbc97f80fec65b9e8ca68f4c3d45b381b4d166d5b9351b |
| NAS100-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | a4c34ee8cb45650b1c5b2d6cc78601133da4bbbab094df1fc581c36f95fe4727 |
| XAUUSD-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 37e46b294002f7c66f94cc4c8a0b93668f5558ddba064f9ed8da033b25fb1d76 |
| XAUUSD-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 31d48a859280d3e7afe58744c10564006d0b1e730cc9f151e0d44090ca41c60d |
| XAUUSD-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | befd52164f3fedf25c55f1a2d9b438168454d501b3edbc58e3a7fb3ba0f49190 |
| XAUUSD-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 441db229c1d294ad42fc6cb062355c9d7962d72f9d31085fb33bb13384b8f0b2 |
| XAUUSD-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 1fa27343cd7a41aeaf91a359dd35210cdbb6d420083e31f3dc583f45a43bee74 |
| XAGUSD-2023-04-17 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 85f16ed3982f25bd6650bbb3c468828ff687632511edd8586dab5d9a1aa1d73c |
| XAGUSD-2023-04-18 | 2023_Q2_Spring | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | cb19f5fd9e5ae05596c2cbc65c6362dd8af2aaced708c24c6d9c819e27a9e0e8 |
| XAGUSD-2023-04-19 | 2023_Q2_Spring | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | 6e307364412a0b26ccbfe22740e8e8e000f4c5862ed444b30c9c5f2130abed19 |
| XAGUSD-2023-04-20 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 49fedae57e219261366650ecf6f164e6091d7a81b0a2a8323bc130713d23f7e3 |
| XAGUSD-2023-04-21 | 2023_Q2_Spring | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | db1a37a6c79317033006333948e853eafcf1093e03c02a79f6cfbcd0e10fbdfb |
| BTCUSD-2023-04-17 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | e8ab232f03e6442c3302b621492017ff0670d279110f49d60b859a8a7ff99365 |
| BTCUSD-2023-04-18 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | fefe7a6149830b649d928eb2e104c0c7f3bae128a2c7f9a2beea92d9f80ae8ea |
| BTCUSD-2023-04-19 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | e739adb5c05e6778e327194a1d2c6021b35bc5159dce6329b312b97a4e633799 |
| BTCUSD-2023-04-20 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 6dd515abcbed15034cd803a75381651323cf97271c166f5cab1a7e6a4f5d1472 |
| BTCUSD-2023-04-21 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | cf65d22ffa8f7ff52a7b3f7e840cb05252b972a0136b7a4165b2782142008a6e |
| ETHUSD-2023-04-17 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 7f538412e323236ef05510df7e759159c7337e5eb117ae83b8287b5cecd8913f |
| ETHUSD-2023-04-18 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 571cb28506b925c43ddab3841ea92cec8335b98462435815b7243ba42689080a |
| ETHUSD-2023-04-19 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 8d84619a4a68156f4acc9844b2c1352e8b98cadb5dd7a24c5ad053e77df9f033 |
| ETHUSD-2023-04-20 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 33810e71705b7c7f8c558d0f0f910aba42a27db7f3d86f449022be3f1c8d1960 |
| ETHUSD-2023-04-21 | 2023_Q2_Spring | complete | 360 | 14 | missing_45day_fixed_window_M30 | not_entered | 1b5344531eacd77f28605a78d93919a8489a1c5a4c7101cad8668ba4452111b2 |
| SpotCrude-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | f2fa71a7be9b64f6eb8fac3792d3bfc0e0254801dff905dccbc86b30342ca7c2 |
| SpotCrude-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | 4d89d092ddd5786b068b5e2a7658bfc9f960d9246a3d3057aa1b20b04aa1c37c |
| SpotCrude-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 0 | no_admitted_impulse | not_entered | 2e28e3cb567a8adfe04663a9bdd194187e202a2ac1a19cfadcd0f2e7395376ec |
| SpotCrude-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | eeca7908c1a42f3f44accfe144c2a97975ea850d8275140841e467d54012ea4c |
| SpotCrude-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | b860dda1a3e9cc5a6934ae98e1c48644f87bc15d34f86be62da5d206d1451710 |
| SpotBrent-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | ff74880614ec6514c16cadb00a062245d1e4d09d215316bfe459c322ddc85281 |
| SpotBrent-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | dd141fc09d57937a733c82cb614dd668f8ce0eabee8990b9b508dc8f77736581 |
| SpotBrent-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 0 | no_admitted_impulse | not_entered | f44fee0b42c60ba07ef0667abbfc5b0db5762bf715fa4b97e0a90bd319f87e68 |
| SpotBrent-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | 7134d653f50cc867ed01689b5ce440110d02d3bdd497e8cd42a9b641b4cbba24 |
| SpotBrent-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | 571e8de6bf90dc5546c5f8ee63d37fa029e500f471fdce2d8050a622fdcc9cf0 |
| US500-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 3 | missing_45day_fixed_window_M30 | not_entered | 33628f598f55ebbc5a9748edb25d6dff9679a78d0321b51b04d9dbc77976c792 |
| US500-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 3 | missing_45day_fixed_window_M30 | not_entered | afed1ec30a5cec5d7a6385cba54d9df0900518384e5d257da3ebc910ee83ee48 |
| US500-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 3 | missing_45day_fixed_window_M30 | not_entered | f76f66c64f2e050f43e2bf272eb30dfe9e698fccae2048235c27dd6b510e5fd9 |
| US500-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 3 | missing_45day_fixed_window_M30 | not_entered | dc7a4eb09a63a9aff4f668d4f1753e7d13d3d70db468882b197fbb4426d64877 |
| US500-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | 84670b5286733340dc21081af5429d85731317f52965e335e4dda55e9afaf9ff |
| NAS100-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 3 | missing_45day_fixed_window_M30 | not_entered | eb0ec8b13d3874a18e9ff5d264a6f78aed90e5aa386d881cf471969946111ebf |
| NAS100-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 3 | missing_45day_fixed_window_M30 | not_entered | 622a8df00f688aa4bd496b0df076ad22f1ec6d2709b024b2ef866653d8c8b97d |
| NAS100-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 3 | missing_45day_fixed_window_M30 | not_entered | 50b4c0f6d27b9c5899e95313b20adc5fd8fa2846002bb152d7f80e79057a7580 |
| NAS100-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 3 | missing_45day_fixed_window_M30 | not_entered | 0e392f0afbb138378d923efc9408b2b3d4c70447bfda2c8701064a4a31167048 |
| NAS100-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | 678ba2d27d7d98f54d0d1d3943bd84be3d9b55a0964a231c79a25ee7b65d92bf |
| XAUUSD-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | c840cdfef22e33d8cbcb98c1b78c26b1a85142181782e24715887f72052340cb |
| XAUUSD-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | 90788a2ec86a60200a19fb8b68c8e7e2b0b316656e566fc48493adfab70333b8 |
| XAUUSD-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 0 | no_admitted_impulse | not_entered | 5578b66f1b89b7654e7938f53893b342816cec5c024643fbe8774ab53b1143cd |
| XAUUSD-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | 33c0dc9868082174a7b8977aa781c3d96fe9f2a9942fbda9a25988580f5d71cc |
| XAUUSD-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | 261f9f6f5d5c900079a22bc0638231049be209f297eb6c7531bd70f56e019e87 |
| XAGUSD-2023-10-16 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | ec68c851144240f4f4c78fe0d73fd672901f72ad46d622c9852d04eba18edee6 |
| XAGUSD-2023-10-17 | 2023_Q4_Autumn | complete | 248 | 0 | no_admitted_impulse | not_entered | cf3185130bf0ef3ee534a2399d280da49fceca21bcca28dc70ec29a24daef54d |
| XAGUSD-2023-10-18 | 2023_Q4_Autumn | complete | 256 | 0 | no_admitted_impulse | not_entered | 4f71be32560f89a5a3931be97601675f441ee777f4e655cb27cda4f563fd979d |
| XAGUSD-2023-10-19 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | db8e529f433268d15b727285519af408203bc52f3dbc9c46a0a4d0aa0a975dde |
| XAGUSD-2023-10-20 | 2023_Q4_Autumn | complete | 264 | 0 | no_admitted_impulse | not_entered | e17764c0b632d7987bd625541fd11ab5157e3eb1cd4d085ecd500a1af51d2146 |
| BTCUSD-2023-10-16 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | 986f60df26925b3da893a04221c65583193b0631b3d04184337049d39b0ce425 |
| BTCUSD-2023-10-17 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | df80401ad63c6ba35940e2d44898d074c811588fead4dd06bed5f2eb50a30ff8 |
| BTCUSD-2023-10-18 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | df5d9b92cae040ce0a6d3fddaad16f619ede7d3f557c4d0f9f2520cdc7772018 |
| BTCUSD-2023-10-19 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | 521cf013e43deccfdb4fb4e920e33ecb16066baec23420ec895137d1a312b4c5 |
| BTCUSD-2023-10-20 | 2023_Q4_Autumn | complete | 360 | 0 | no_admitted_impulse | not_entered | b17b1ac38ab6a8a9abf281299aa06de1eeecdf015eebe54eae1d055b13242199 |
| ETHUSD-2023-10-16 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | a3a6b907e0464c32a74f03819516bcbe3552499ea049424175ee683af3a61f54 |
| ETHUSD-2023-10-17 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | 2c0f864df1de18f43fbf5bd455a2e807dcd1748d6e245e44004d403198ea1947 |
| ETHUSD-2023-10-18 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | 8a6e1e25331a295a627e1327d0fb86ec28b9b04db53d42680d27f2fff968f165 |
| ETHUSD-2023-10-19 | 2023_Q4_Autumn | complete | 360 | 0 | missing_fresh_tick_boundary | not_entered | 3626c8a72e510a13c3a38afd39801c26513976fc50129ff2492cade506c2cb65 |
| ETHUSD-2023-10-20 | 2023_Q4_Autumn | complete | 360 | 0 | no_admitted_impulse | not_entered | f40521e2906091197de9930b0e62b7a66729343a2ce6e1e7c6a11935e26d8e56 |
| SpotCrude-2024-03-04 | 2024_Q1_Spring | complete | 248 | 48 | missing_45day_fixed_window_M30 | not_entered | 2418002409cb019a77987cad8dac1c6021b1972d62b73677b1581037b5880c5a |
| SpotCrude-2024-03-05 | 2024_Q1_Spring | complete | 248 | 40 | missing_45day_fixed_window_M30 | not_entered | 1440e70cbb111d328d36cec4fcb30c45364f321b76ab258e33a24c1c1f7caed4 |
| SpotCrude-2024-03-06 | 2024_Q1_Spring | complete | 256 | 40 | missing_45day_fixed_window_M30 | not_entered | 4cf98c8813617b131df2b930e8523177d8265fecdebac8c17305d236cc45560c |
| SpotCrude-2024-03-07 | 2024_Q1_Spring | complete | 264 | 40 | missing_45day_fixed_window_M30 | not_entered | 0ee0197b8b02f22b78967c5978c41f17928f0adbd2290c2325e03c1bdc104521 |
| SpotCrude-2024-03-08 | 2024_Q1_Spring | complete | 264 | 32 | missing_45day_fixed_window_M30 | not_entered | 64c6e1ba7c109b7b3b716b1c53d72fe3b14637057277d0fd5e6ab1618a98eb03 |
| SpotBrent-2024-03-04 | 2024_Q1_Spring | complete | 248 | 48 | missing_45day_fixed_window_M30 | not_entered | 0a428ead1cfba7e134f108a9c1b19f5b1e0f97c406e074d379130defeaff3d88 |
| SpotBrent-2024-03-05 | 2024_Q1_Spring | complete | 248 | 40 | missing_45day_fixed_window_M30 | not_entered | 1f9b8dbfcb754346778cd351252e776e3e06e3ddd3b3f8d92708d70b8af3f6f5 |
| SpotBrent-2024-03-06 | 2024_Q1_Spring | complete | 256 | 40 | missing_45day_fixed_window_M30 | not_entered | 80994052238d6441ba5766ceb7c478f3a50f3e0bfbea687241f0d7d85b2a8155 |
| SpotBrent-2024-03-07 | 2024_Q1_Spring | complete | 264 | 40 | missing_45day_fixed_window_M30 | not_entered | f6bf5d688216f3b5c2e16788a48fad81dfcac8a39145e0a4a321277b2f07d2e3 |
| SpotBrent-2024-03-08 | 2024_Q1_Spring | complete | 264 | 32 | missing_45day_fixed_window_M30 | not_entered | 972eaf7f3039d3513d8963837ba52af985e1e341e8d4bbce8bea20342d291b07 |
| US500-2024-03-04 | 2024_Q1_Spring | complete | 248 | 49 | missing_45day_fixed_window_M30 | not_entered | 023b5cd03570eaaf27bcbf0afde8b4b504fbd293b0e770a026dba49797400afa |
| US500-2024-03-05 | 2024_Q1_Spring | complete | 248 | 41 | missing_45day_fixed_window_M30 | not_entered | 6d394dfcc8112dae92cc87c71f873942573f37e1dd50cd3d07729a822b2e92b9 |
| US500-2024-03-06 | 2024_Q1_Spring | complete | 256 | 41 | missing_45day_fixed_window_M30 | not_entered | 78571656b9fce32f6b72ff814dcd793489b04932ab606e3ec5f62113786ee281 |
| US500-2024-03-07 | 2024_Q1_Spring | complete | 264 | 41 | missing_45day_fixed_window_M30 | not_entered | 2a3c5c355e8574509cb71c23d632b5e0af31f81f55d82afe7f32cacbd5335bac |
| US500-2024-03-08 | 2024_Q1_Spring | complete | 264 | 33 | missing_45day_fixed_window_M30 | not_entered | 3a5faa5460a186cb5079b8eb93685a98fb223dc87cffe7dfbd9a4009362068e6 |
| NAS100-2024-03-04 | 2024_Q1_Spring | complete | 248 | 1 | missing_45day_fixed_window_M30 | not_entered | 665b5b3b85a37d27800eda86a8add3e5cffad642b71b2fb3bb7288e210a084d2 |
| NAS100-2024-03-05 | 2024_Q1_Spring | complete | 248 | 1 | missing_45day_fixed_window_M30 | not_entered | 9cffa07b799901b4a89a1e5a119291680053135c7cb90a69dbc237b8b4963ece |
| NAS100-2024-03-06 | 2024_Q1_Spring | complete | 256 | 1 | missing_45day_fixed_window_M30 | not_entered | df2b5f4d6f34bcdd90af1f469f66dccb7c435a7c88e502b1c74f81d3a72f91b9 |
| NAS100-2024-03-07 | 2024_Q1_Spring | complete | 264 | 1 | missing_45day_fixed_window_M30 | not_entered | 362826a6d48c2bc8751c9d7d0eec634ad23aad2c86d604a80e0f5b2f5a0fe3c3 |
| NAS100-2024-03-08 | 2024_Q1_Spring | complete | 264 | 1 | missing_45day_fixed_window_M30 | not_entered | 578afd7e143b32154ca40408c64bc0540c83efee45258ef1509008129f4cc0bf |
| XAUUSD-2024-03-04 | 2024_Q1_Spring | complete | 248 | 0 | no_admitted_impulse | not_entered | 9afdefc3796a60dd006e26a9d5a88342350971249d9f71b6756110e248e5aa40 |
| XAUUSD-2024-03-05 | 2024_Q1_Spring | complete | 248 | 0 | no_admitted_impulse | not_entered | f9164e31121c81d75d8153359c705f2dc469e12521c16f37dffdce1767161c55 |
| XAUUSD-2024-03-06 | 2024_Q1_Spring | complete | 256 | 0 | no_admitted_impulse | not_entered | 1abe68983f1d4dbd1e123662d75549a84bd8c92582f1c0a2cd5bf8ab8582496a |
| XAUUSD-2024-03-07 | 2024_Q1_Spring | complete | 264 | 0 | no_admitted_impulse | not_entered | 67529885380868782d6c1c2c8b53e030bf77bc4434f8411bfda453087144bd7c |
| XAUUSD-2024-03-08 | 2024_Q1_Spring | complete | 264 | 0 | no_admitted_impulse | not_entered | e87b5d53d03be94a4edcb2eaaed9ee37e8648ff086f7279e2375d893e111bb42 |
| XAGUSD-2024-03-04 | 2024_Q1_Spring | complete | 248 | 0 | no_admitted_impulse | not_entered | 68c77d341c3201b328908442dd7b4fc9d1f24074e10b138b8d5819692d9e4ab5 |
| XAGUSD-2024-03-05 | 2024_Q1_Spring | complete | 248 | 0 | no_admitted_impulse | not_entered | 50ab223a5c0e04ce7df70f29db25c0cd47fffa4bdbe8828a079d587bc97b3e3c |
| XAGUSD-2024-03-06 | 2024_Q1_Spring | complete | 256 | 0 | missing_fresh_tick_boundary | not_entered | 2ffeb84dd322b0a0964bf1be84a3958871277a8728bf3e57472a0b157533bfa3 |
| XAGUSD-2024-03-07 | 2024_Q1_Spring | complete | 264 | 0 | no_admitted_impulse | not_entered | 0496976989099751c77733606613231221e805eb25bb2c7f00d0a7245a5647ae |
| XAGUSD-2024-03-08 | 2024_Q1_Spring | complete | 264 | 0 | missing_fresh_tick_boundary | not_entered | 1d4f7883199888c6d6c9595973d0d687c72b97fdc20bb5d4a17ce34fb5051ffe |
| BTCUSD-2024-03-04 | 2024_Q1_Spring | complete | 360 | 0 | own_and_peer_aligned | admitted | 960f9be13352e6bee833f693277a418bff24e9a532f87c0ca2a479dbdcb07470 |
| BTCUSD-2024-03-05 | 2024_Q1_Spring | complete | 360 | 0 | no_admitted_impulse | not_entered | 40dd8bc8bc40b7dfc713ab88fd5452cd0ed9a0615160cb4d767ba098b0d95c5e |
| BTCUSD-2024-03-06 | 2024_Q1_Spring | complete | 360 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | 1fd160f3e57b7a3ac1f4b9913b5e581304e051c879208546572583b8b2b8e449 |
| BTCUSD-2024-03-07 | 2024_Q1_Spring | complete | 360 | 0 | no_admitted_impulse | not_entered | 48c8bffa38305edb4aa0b33739fe114bb2f8a458e3be32aeb0c3b89896b9599a |
| BTCUSD-2024-03-08 | 2024_Q1_Spring | complete | 360 | 0 | no_admitted_impulse | not_entered | c1be171f6ac724863805016bd7c08b31f1bd1db83eeb81d1c3c43fcb947a182f |
| ETHUSD-2024-03-04 | 2024_Q1_Spring | complete | 360 | 0 | no_admitted_impulse | not_entered | 721071e02dbec33ca8ddb318aa7e2f6c4bfd24b59abdfac93d4c46c1166f4b49 |
| ETHUSD-2024-03-05 | 2024_Q1_Spring | complete | 360 | 0 | no_admitted_impulse | not_entered | 103308a0e11d5a39c805078cef0fe9cddb431f86a6aa5b1b6d02034b66acdf53 |
| ETHUSD-2024-03-06 | 2024_Q1_Spring | complete | 360 | 0 | own_5d20d_disagree_or_missing | own_5d20d_disagree_or_missing | 88b52af313ebc93bc2f0defcb2ecfc96398a83480d68a1f1f6fb0c233dbc6024 |
| ETHUSD-2024-03-07 | 2024_Q1_Spring | complete | 360 | 0 | own_and_peer_aligned | admitted | b2ce18a0756b8408ef26e268e60a66bf87225f1567668e14df6c0ef61d7affe6 |
| ETHUSD-2024-03-08 | 2024_Q1_Spring | complete | 360 | 0 | own_and_peer_aligned | no_retest | b1a9a5b1cfb1745d26fec4f5f841821fbdf402de5b0d24afe80f24ecc917a9a8 |
| SpotCrude-2024-08-12 | 2024_Q3_Summer | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | b36d5207474836de541899a309d070c177bc19b7122abc6e19436de374f0e99d |
| SpotCrude-2024-08-13 | 2024_Q3_Summer | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 22016a36c7f6ad9900fe96ed2f2f45546e339cc7b56efe9823fbe859cb6ec83c |
| SpotCrude-2024-08-14 | 2024_Q3_Summer | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | 5172ec059d566a2ed11852c43e2620c4c771d4ea0514576711acfcd7df939e7d |
| SpotCrude-2024-08-15 | 2024_Q3_Summer | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 19af25d0e1884687f2fcd37324016e9b350d7499ab4a0e8cbd5998f65bd7e8e3 |
| SpotCrude-2024-08-16 | 2024_Q3_Summer | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | a31225758ccd34f89bc3a6c9274069bc2a7ddaecb2e9bc27553f2ef999e2002e |
| SpotBrent-2024-08-12 | 2024_Q3_Summer | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | bbbcf70c6a6b1fc6b173a4a9c64ffbd13d97ca05466f68e6d2534e6b0399ad64 |
| SpotBrent-2024-08-13 | 2024_Q3_Summer | complete | 248 | 8 | missing_45day_fixed_window_M30 | not_entered | 40ef39d0d857ac780532f463874434bfb589cda60ede3c8509519fd6930e8961 |
| SpotBrent-2024-08-14 | 2024_Q3_Summer | complete | 256 | 8 | missing_45day_fixed_window_M30 | not_entered | 142f1c50ce8573598950d857a956ec1a0036365e053af9c833295781e46a503a |
| SpotBrent-2024-08-15 | 2024_Q3_Summer | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | 9fe15f367e5382f4d380b54841b84d38b81bc7fc2c8abee9e768b9755148695a |
| SpotBrent-2024-08-16 | 2024_Q3_Summer | complete | 264 | 8 | missing_45day_fixed_window_M30 | not_entered | c90f5154f4c2d367a5f29aae6ddea29003747a8d5163a0c526f647fe976e1f10 |
| US500-2024-08-12 | 2024_Q3_Summer | complete | 248 | 5 | missing_45day_fixed_window_M30 | not_entered | 93d28103b4532cfd193a69f9dc3723f46b497691b7e7d936500fb715673d7ea3 |
| US500-2024-08-13 | 2024_Q3_Summer | complete | 248 | 5 | missing_45day_fixed_window_M30 | not_entered | 9310997c7df6fb22b1e5a5cdb5f0076680a1dda406f0af4ca92e0dc4192c3c8b |
| US500-2024-08-14 | 2024_Q3_Summer | complete | 256 | 5 | missing_45day_fixed_window_M30 | not_entered | da8d9c200cbc15e84a56d421005d2a20f7cb8732107e6f5d0f97a87b3b859f7d |
| US500-2024-08-15 | 2024_Q3_Summer | complete | 264 | 5 | missing_45day_fixed_window_M30 | not_entered | 3b1aaffbadd820ba448580d71dcbee65c9eaeaf8bd10592a72c7ea5a986c9cb6 |
| US500-2024-08-16 | 2024_Q3_Summer | complete | 264 | 5 | missing_45day_fixed_window_M30 | not_entered | 26930363aa6f50dd234195f9b17cd8f923fd75854b8946bfc18a037e5fb4d2d0 |
| NAS100-2024-08-12 | 2024_Q3_Summer | failed | 248 | 5 | missing_45day_fixed_window_M30; tick_source_failed | not_entered | a11ce68367ae5af31c0d3f5e444e5e08c5090c7d4d80519a16313c6032e092c0 |
| NAS100-2024-08-13 | 2024_Q3_Summer | complete | 248 | 5 | missing_45day_fixed_window_M30 | not_entered | b75514af219df1fa23034d98442bd287b7da4da88b15d2ea094f068f3b870361 |
| NAS100-2024-08-14 | 2024_Q3_Summer | failed | 256 | 5 | missing_45day_fixed_window_M30; tick_source_failed | not_entered | a76fb6ede3de8f29fa16eaeb9e7c42aeb52b9eb19a575d9b28554dc7c88fa1b7 |
| NAS100-2024-08-15 | 2024_Q3_Summer | complete | 264 | 5 | missing_45day_fixed_window_M30 | not_entered | 68af27a4fdc6b551c561e27c8961d11a87a2b91d5d27d4a5ee06ec6e60852ad7 |
| NAS100-2024-08-16 | 2024_Q3_Summer | complete | 264 | 5 | missing_45day_fixed_window_M30 | not_entered | 7d74005a181b020bfbffcd947d5d3f13ee998a31e6f1d3b75862723a2583681f |
| XAUUSD-2024-08-12 | 2024_Q3_Summer | complete | 248 | 0 | no_admitted_impulse | not_entered | 9212d876ef7da979c4471c1c0f27685e8ce24de12aa42ea0af1980b2049cb821 |
| XAUUSD-2024-08-13 | 2024_Q3_Summer | complete | 248 | 0 | no_admitted_impulse | not_entered | ab949e2eba01d652d7ff3291802a832b23e67d59ceba4e1642a64e553a5f325f |
| XAUUSD-2024-08-14 | 2024_Q3_Summer | complete | 256 | 0 | no_admitted_impulse | not_entered | 4fe174cbea982de8976095a8d96bf112aa4a01b935315301c79bfff27d8554e7 |
| XAUUSD-2024-08-15 | 2024_Q3_Summer | complete | 264 | 0 | no_admitted_impulse | not_entered | bc223c5605ee54cae6d44376c46dbf0a46a9feedd645aaf85ae69f8d84b0d8e5 |
| XAUUSD-2024-08-16 | 2024_Q3_Summer | complete | 264 | 0 | no_admitted_impulse | not_entered | fc874cffb0ad40adad35e13a3efa6511fd58077d114ad6f6f648ef09675a636c |
| XAGUSD-2024-08-12 | 2024_Q3_Summer | complete | 248 | 0 | no_admitted_impulse | not_entered | 255a4e31c922901f566eeb5326783eface6de80878e2f396ffd63c1e1eb3c79d |
| XAGUSD-2024-08-13 | 2024_Q3_Summer | complete | 248 | 0 | no_admitted_impulse | not_entered | 029cd9cb79754e4b9caac4387b3bed89d4f493c88d35c2ee9d24bd0b1ecd556c |
| XAGUSD-2024-08-14 | 2024_Q3_Summer | complete | 256 | 0 | no_admitted_impulse | not_entered | 5d04f8561d877c8388c9592a19b35c1180fd3771c9f56058a0ddd91688794e2f |
| XAGUSD-2024-08-15 | 2024_Q3_Summer | complete | 264 | 0 | no_admitted_impulse | not_entered | addcba80d2923a78af97352e9776463f6c146385f027c8b9e6ca26451c0736b3 |
| XAGUSD-2024-08-16 | 2024_Q3_Summer | complete | 264 | 0 | no_admitted_impulse | not_entered | 831842fd9093d6f74158f26a04de3ebbdd4558de0916cf425c1ddc1468cd07d9 |
| BTCUSD-2024-08-12 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 02e43b67d8ee6be81720ce9cb28c534ddba9f3f078776abebc7216ce3ea6d494 |
| BTCUSD-2024-08-13 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 1eeb5b32449b4143dbc341f8065f8dcf41a487d46f8521c7e952e3e2096ef7cc |
| BTCUSD-2024-08-14 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | a10f1b392009d45ae5bb0b7c116a110b145a9e31a994169f1a9ad1a043035d39 |
| BTCUSD-2024-08-15 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 15610ce195d5b27cba6cc32a2ed4f5e8e31fdf2fff6b6dc98d7a135689ce72fb |
| BTCUSD-2024-08-16 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | d079f9bb058b4eb5f99131a0f31345d6488f3767b6e1c9143743011ef98c4da2 |
| ETHUSD-2024-08-12 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | fa160d488bdacc27cdaea3fb9b9c1b80228ba45ad7469213ce2d9d8e26d3ade4 |
| ETHUSD-2024-08-13 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | c773d7e35996274e946bb7e5f8c35bd2bf5f7f05d757852a3c65dfcf3e37dfcc |
| ETHUSD-2024-08-14 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 2fbbf22592707394944d191a48908882166bdf3f7a79bb3185448db07f881385 |
| ETHUSD-2024-08-15 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 01ce58c4051b915b3bfe5e50f311fd5b871ab091c149de6c84998c1bbf4cc701 |
| ETHUSD-2024-08-16 | 2024_Q3_Summer | complete | 360 | 0 | no_admitted_impulse | not_entered | 1db36b6110335e75a60ac7326a70a292b45f8a63c2f55fb36a089e62a42f18f6 |

## §11. Instructions for independent AI review

Read this report with the Phase 5 plan, results and verification JSON under `docs/`, and Phase 4's frozen original. Check the owner-authorised session change, complete 160 denominator, cached/spent interval handling, strict versus forced versus M30 accounting, per-class comparisons and cost sensitivity. Challenge any purported edge against the measured evidence. Preserve original goals, serial/paragraph numbering, SGT timestamps, proven fixes and the “Time is Money” invariant.
