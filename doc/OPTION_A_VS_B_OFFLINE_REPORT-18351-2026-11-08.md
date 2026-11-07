# Option A vs B: matched-entry offline comparison

№18,351 §§1–12 · 08-11-2026 05:22:44 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

Completed **159 cached episodes** with **zero new broker requests** and exact prior native-model parity on all 159. Each arm uses the same 8 admitted setups. Results are conditional research outcomes, not realised account trades or a financial PASS.

Original invariant: **73% net wins / reference 1:1.38 risk:reward on latest 20 trades per strategy/account**. This reused pooled sample cannot qualify that goal. No new independent trade evidence was collected.

**Decision:** the simple fade performed worse. The72-hour extension remains inconclusive because it resolved no additional trade from cached continuation bars. The frozen experiment is complete; the strategy has not qualified.

## §2. Primary 24 bp/side plus spread

| Model | Entries | Closed | Wins | Unresolved | Win rate | Mean net R | PF(R) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| s3_breakout_4h | 8 | 7 | 3 | 1 | 42.86% | -0.265185 | 0.517485 |
| option_a_fade_4h | 8 | 7 | 0 | 1 | 0.00% | -1.582254 | 0.000000 |
| option_b_breakout_72h | 8 | 7 | 3 | 1 | 42.86% | -0.265185 | 0.517485 |
| option_ab_fade_72h | 8 | 7 | 0 | 1 | 0.00% | -1.582254 | 0.000000 |
| s3_breakout_4h_mtm | 8 | 8 | 3 | 0 | 37.50% | -0.321609 | 0.436232 |
| option_a_fade_4h_mtm | 8 | 8 | 0 | 0 | 0.00% | -1.396729 | 0.000000 |

The four-hour censor and MTM rows are deliberately separate. Bar continuation closes are conditional OHLC approximations; unresolved or ambiguous paths remain in the denominator as unresolved, not zero-return trades. Closed-only win rates can be selection-biased.

## §3. Same-entry cost sensitivities

These change cost deductions only. They do not reopen vetoed setups or recalibrate broker costs.

### 2bp/side

| Model | Entries | Closed | Wins | Unresolved | Win rate | Mean net R | PF(R) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| s3_breakout_4h | 8 | 7 | 3 | 1 | 42.86% | 0.389032 | 3.194238 |
| option_a_fade_4h | 8 | 7 | 0 | 1 | 0.00% | -0.929066 | 0.000000 |
| option_b_breakout_72h | 8 | 7 | 3 | 1 | 42.86% | 0.389032 | 3.194238 |
| option_ab_fade_72h | 8 | 7 | 0 | 1 | 0.00% | -0.929066 | 0.000000 |
| s3_breakout_4h_mtm | 8 | 8 | 3 | 0 | 37.50% | 0.296783 | 2.493209 |
| option_a_fade_4h_mtm | 8 | 8 | 1 | 0 | 12.50% | -0.779238 | 0.041449 |

### 3.74bp/side

| Model | Entries | Closed | Wins | Unresolved | Win rate | Mean net R | PF(R) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| s3_breakout_4h | 8 | 7 | 3 | 1 | 42.86% | 0.337290 | 2.631451 |
| option_a_fade_4h | 8 | 7 | 0 | 1 | 0.00% | -0.980727 | 0.000000 |
| option_b_breakout_72h | 8 | 7 | 3 | 1 | 42.86% | 0.337290 | 2.631451 |
| option_ab_fade_72h | 8 | 7 | 0 | 1 | 0.00% | -0.980727 | 0.000000 |
| s3_breakout_4h_mtm | 8 | 8 | 3 | 0 | 37.50% | 0.247874 | 2.086433 |
| option_a_fade_4h_mtm | 8 | 8 | 1 | 0 | 12.50% | -0.828076 | 0.035031 |

## §4. Entry-by-entry outcomes at 24 bp

| Episode | Baseline4h | Fade4h | Breakout72h | Fade72h |
| --- | --- | --- | --- | --- |
| SpotCrude-2023-01-26 | -0.545150R (closed_tick) | -1.547370R (closed_tick) | -0.545150R (closed_tick) | -1.547370R (closed_tick) |
| SpotBrent-2023-01-26 | 0.536708R (closed_tick) | -1.881437R (closed_tick) | 0.536708R (closed_tick) | -1.881437R (closed_tick) |
| SpotCrude-2024-01-22 | 0.754547R (closed_tick) | -1.636534R (closed_tick) | 0.754547R (closed_tick) | -1.636534R (closed_tick) |
| SpotBrent-2024-01-22 | 0.699575R (closed_tick) | -1.696429R (closed_tick) | 0.699575R (closed_tick) | -1.696429R (closed_tick) |
| SpotCrude-2024-07-23 | -0.826426R (closed_tick) | -1.826409R (closed_tick) | -0.826426R (closed_tick) | -1.826409R (closed_tick) |
| SpotCrude-2025-04-11 | -0.793407R (closed_tick) | -1.805067R (closed_tick) | -0.793407R (closed_tick) | -1.805067R (closed_tick) |
| US500-2025-04-08 | -1.682141R (closed_tick) | -0.682534R (closed_tick) | -1.682141R (closed_tick) | -0.682534R (closed_tick) |
| US500-2025-04-11 | censored_tick | censored_tick | censored_end_of_cache | censored_end_of_cache |

## §5. What happened to the five previously censored trades?

| Episode | In159? | Continuation result |
| --- | --- | --- |
| BTCUSD-2024-05-14 | False | outside_frozen_159_episode_scope |
| ETHUSD-2024-05-16 | False | outside_frozen_159_episode_scope |
| NAS100-2025-05-15 | False | outside_frozen_159_episode_scope |
| ETHUSD-2025-05-16 | False | outside_frozen_159_episode_scope |
| US500-2025-04-11 | True | censored_end_of_cache |

Four May peer trades are outside the explicitly frozen 119 + 40 corpus. They were neither dropped from history nor added to improve this comparison. The failed US500 24 January 2024 source also remains documented outside the 159 completed episodes.

## §6. Interpretation and evidence boundaries

- **admission**: Original native 24bp/side full C<R and >=48bp R/fill; first admitted entry only, same timestamp and R in every arm. Sensitivities retain primary entries.
- **entry**: Use same next executable fresh quote as baseline, never an earlier decision quote. Fade buy at ask and sell at bid. Original quantity and R retained.
- **horizons**: 4h means existing registered observation boundary, not four hours after fill. Oil inherited native quotes start07:00 before13:00-17:00 window. 72 elapsed hours from fill, not three trading sessions; weekend hours count.
- **baseline**: No native session clock close before the tick boundary; unresolved ticks censored. Additional explicit MTM arm uses first fresh boundary quote within 5s, broker bracket first.
- **continuation**: Only subsequent contiguous M30 bars starting exactly at tick boundary; stop at first absent bar, uncertain intrabar order, partial deadline bar or end of cache. Never infer holiday/weekend continuity.
- **bar_model**: M30 interpreted as bid OHLC; short ask exits use frozen recorded entry spread proxy. Existing bracket first at bar open; adverse gaps use observed open, TP capped at target. Stop/target or +1R-and-return ambiguity stays unresolved. No synthetic OHLC tick path.
- **bar_amend**: After trigger with no ambiguous return, a distinct favourable closing price proves a later quote; equal extreme/close leaves amend timing ambiguous and unresolved. Short proxy and intrabar execution remain unqualified.
- **costs**: Bid/ask tick fills include spread once. M30 short asks use entry-spread proxy once. Charge (entry+exit)*bps/10000/R. Zero extra commission inherited, financing/overnight swap and historical spread unavailable; rates uncalibrated.
- **evidence**: Previously observed development cache, retrospectively chosen hypotheses, protocol flags retained. Not untouched validation or account qualification.

## §7. Measured assessment and next decision

At 24 bp/side, baseline breakout returned 3/7 wins and −0.265185R mean, versus fade 0/7 and −1.582254R. At 2/3.74 bp, breakout mean becomes +0.389032R/+0.337290R, but the win rate remains 42.86%. Fade stays negative. These are cost sensitivities, not verified broker economics.

The US500 11 April 2025 fade MTM has +0.302983 grossR but −0.098051 netR at 24 bp/side. It turns positive only at the lower assumed charges. Neither direction provides a qualifying primary-cost result.

Only eight setups were admitted: four SpotCrude, two SpotBrent and two US500; no XAUUSD setup. Four older US500 tick episodes have zero preparation bars after prior collection failures; all 159 tick episodes are retained, but they are not159 complete signal-preparation cases. Among the remaining nonentries, 124 episodes recorded cost-gate vetoes. Reason counts can overlap.

Twenty source episodes retain historical protocol flags; three admitted trades are flagged, including two of the three breakout winners. This experiment does not repair that historical evidence.

The prior full like-for-like research ledger remains **8/20 closes, 3 wins and 5 unresolved entries**. The 42.86% figure here omits the May peer cohort by the requested sample definition; it is not an improvement over the earlier 37.5%. Replaying observed trades adds **zero independent evidence** toward the account goal.

**Option B is unresolved, not disproved.** All seven resolved breakout trades had already exited on ticks. US500 11 April 2025 has zero cached M30 bars after 17:30 UTC, so the only in-scope open trade remains unclosed. The other four old unclosed trades belong to the excluded May peer corpus. No trade has been invented or silently closed.

**Next:** establish actual round-trip execution costs and an explicitly authorised continuation data scope before judging 72-hour holding. Do not deploy the generic fade or relax admission to fill the 20-trade quota.

Compare the paired directions before attributing any change to prediction skill. A gross-flat exit remains a loss after costs; a lower assumed charge cannot turn a negative gross fill into a winner. M30 continuation cannot establish a reliable multi-day edge without executable bid/ask paths, unambiguous order sequencing and financing.

Any apparent winner here is a hypothesis selected on previously seen data. Preserve losses, protocol flags and unresolved trades. Freeze one justified candidate and realistic cost specification before a genuinely untouched evaluation; do not tune until a target is reached. Live rules remain unchanged and the 19 August–5 September2025 holdout remains sealed.

## §8. Reproducibility

Plan SHA256: `0a8ce4af1e0317919b76380c582555846574c9297afb2482267c5f02f57f0703`. Native binary and source hashes, per-episode results, bar coverage and accounting checks are in the results JSON. 132 independent net-cost calculations checked. Prior cached-source protocol flags are retained per episode.

[Frozen plan](../docs/option_a_vs_b_offline_plan_18351.json) · [Measured results](../docs/option_a_vs_b_offline_results_18351.json)
