# Stage 1 multi-year seasonal pilot — review report

№18,316 §§1–8 · 2026-11-07T11:57:04.463123+08:00 · Codex (GPT-6), effort not exposed.

## §1. TLDR

**79/80 episode sources completed and replayed**; **80/80 episodes
carry a missing-data, sequence, preparation or censoring flag in at least one model**.
This is a fixed 20-date, four-symbol pilot across four seasonal cohort weeks.
It does not establish the interim 50% goal or the original 73% / 1:1.38 account goal.
No validation opened, no strategy promoted, no live account or production universe changed.
Two-year cohort labels are not proof of seasonality or evidence of a complete-year test.

### §1.1. Actual completion and decision

**80/80 logical jobs reached a terminal receipt: 79 successful tick sources and 1 failed job.**
All 79 successful sources passed hash checks and native replay. This is **not 80 fully verified trading episodes**.
The 116 shared M30 component requests produced 91 usable components and 25 unusable components;
the interrupted tick job is `US500-2024-01-24`. Every failure remains visible.

All three candidate development gates fail. The original goal has not been demonstrated, and this pilot does not justify deployment or opening validation.
No percentage-point improvement towards 50% or 73% can be established for real accounts from this pilot.
The useful advance is a frozen, inspectable comparison across 20 dates with explicit reasons for non-entry and incomplete evidence.

Primary 24 bp/side results below use trailing exits for the three candidates and fixed-horizon exits for controls:

| Model | Entries | Closed | Wins / closed | Net mean R | PF(R) |
| --- | --- | --- | --- | --- | --- |
| production_s3 | 0 | 0 | N/A | N/A | N/A |
| orb_30m | 3 | 3 | 1/3 | -0.52094 | 0.26882 |
| orb_60m | 7 | 7 | 0/7 | -0.78270 | 0.00000 |
| always_long_control | 26 | 26 | 6/26 | -1.04779 | 0.04863 |
| own_momentum_fixed_horizon | 26 | 26 | 5/26 | -1.13135 | 0.03293 |


### §1.2. What the experiment actually found

- **S3 non-entry:** 45 of 80 episodes reached a signal whose primary cost reserve was at least its original risk distance; the durable session veto then prevented entries. Fourteen successful tick sources lacked opening M30 preparation, one job failed, and the remaining 20 episodes produced no admitted signal. S3 therefore has **no win-rate estimate**, not a 0% win rate. A concrete example is SpotCrude on 12 May 2025: R = 0.165 price units versus cost reserve = 0.332568. US500 that day had R = 19 versus cost = 28.2592. All 45 S3 vetoes would pass the lower-cost arithmetic threshold at either sensitivity, but no such entries were simulated.
- **ORB-30:** primary trailing results are 1/3 wins, mean −0.520943 R, PF 0.268822. At 2 bp/side the same three trades become 2/3 wins, mean +0.203063 R and PF 1.619034. That tiny, conditional, uncalibrated subset is not a qualified edge or a new-entry backtest.
- **ORB-60:** primary trailing results are 0/7 wins, mean −0.782700 R. At 2 bp/side, 4/7 win (57.14%) but mean R remains **−0.185810** and PF **0.489119**. Reaching a 50% hit rate alone would not make this candidate profitable. Its primary mean is negative in both sampled years, triggering the frozen stop-tuning rule for this candidate; this is not a theorem that no ORB edge can exist.
- **Preparation versus signal:** each ORB variant lacks the exact prior 20 opening ranges in 20 episodes: BTC's five winter dates and the 15 spring oil/US500 dates. For representative 22 January BTC preparation, the absent opening bar is 7 January 2024; for 12 May oil/US500 preparation it is 18 April 2025. The current session registry omits exchange holidays. Missing history and broker calendar closures need separate verification; this run neither skipped the absent sessions nor changed the 20-session definition after seeing outcomes.
- **Controls:** both ORB variants have negative mean paired deltas against each control. ORB-30 has only three pairs across two weeks (p=0.75 against both); ORB-60 has seven pairs across two weeks (p=1.0 against always-long, p=0.75 against own-momentum). There is no demonstrated matched-control advantage.

### §1.3. Recommended next decision, not an automatic new run

1. **Verify economics before retuning entries:** reconcile independently observed broker spread, realised slippage, fees and native quantity mapping by asset/session. Freeze any replacement cost assumptions before another experiment. Do not adopt 2 or 3.74 bp merely because the results improve.
2. **Resolve data semantics:** verify the holiday/session calendar and missing opening bars, then state any revised prior-session rule in a new contract. Assess how bid/ask ordering uncertainty affects fills; a successful download cannot certify tick-path continuity. Keep this failed evidence intact.
3. **Decide whether a new hypothesis earns a new budget:** stop tuning this ORB-60 candidate under the frozen rule. S3 remains economically blocked under the primary assumption, and ORB-30 has too few closes. If a new experiment is justified, use prospectively frozen new development dates, sufficient independent weeks, and explicit fees/full-session exits where relevant. Do not retry these failed jobs, open validation, or expand Stage 2 automatically.

## §2. Frozen method and corrections

The owner's source JSON and Markdown are preserved verbatim. Read the
[execution addendum](../docs/seasonal-execution-addendum-18315.md) for every
pre-outcome interpretation: DST-aware US500; 07:00 UTC oil window; 00:00 UTC BTC;
45-day nominal M30 view excluding protected 19 August–5 September 2025 data;
ORB-30/60 high/low >1.25 times the exact prior-20-session median; four-hour quotes.
The supplied full universe contains eight symbols, not seven. Stage 2 did not run.

S3 shares native band, mark, cost veto, dispatch recheck, targets and exits.
Its reconstructed opening spread/volume is not an authentic historical pre-open
receipt. Its actual session end is retained: an open trade at hour four is censored.
ORB exits close at hour four; controls at hour one hold to hour four. Account risk,
fees, historical contracts and broker execution are not reconstructed. These are
recorded-path scenarios, not production-account parity or qualified financial returns.

Primary admission/geometry remains 24 bp per side. 2 and 3.74 bp are **uncalibrated
sensitivities on identical admitted fills**, not newly admitted trades. Bid/ask
spread is counted once; zero extra commission remains an incomplete assumption.
Win rate uses all closed trades including breakevens. Failures, vetoes, missing data
and censored episodes stay in the 80-episode table and coverage denominator.
N/A is not zero return. PF(R) uses normalised R, not combined account dollars.

### §2.1. Collector incident and its effect on conclusions

The initial collector supplied `count=500` alongside an interval. The broker selected count-based M30 history and returned a truncated 500-bar response; the decoder correctly rejected it.
The collector was paused after **24 failed M30 segments**. An in-flight, already-reserved US500 job was marked failed/uncertain rather than silently retried.
Two unspent, already-registered BTC winter segments confirmed the transport diagnosis: count mode returned 500 bars with `hasMore=true`; interval-only mode returned 326 bars with `hasMore=false`.
The former remains unusable. The latter was decoded from its saved response without another request.

[PR #179](https://github.com/ang-kl/jiaoyi/pull/179) removed `count`, retained raw failure responses, and made unexpected/truncated responses stop the collector immediately.
It merged at **07-11-2026 11:49:31 SGT**, commit `67d21e6b75ae030c30a7e50bcc6c62fce22fec0d`, after all three CI checks passed.
Collection then continued with only the unspent jobs. The original freeze, date set, native binary, signals, exit rules and costs were unchanged.
This was an engineering data-collection failure, not evidence that the market or a strategy failed.
Winter oil/US500 preparation is missing. Consequently, the 2024 and 2025 comparisons have unequal usable coverage and cannot establish a causal seasonal effect.

The resulting evidence is 96.85 MB against the 256 MB raw-source cap, including 1,824,274 bid/ask updates and 466 successful tick pages.
The interrupted job's exact page count is unknown and remains charged to its reserved 120-page cap. No raw source was deleted and no failed job was retried.
Flag categories overlap: `{"always_long_control:censored_primary": 0, "failed_M30_component": 19, "failed_episode": 1, "long_intervals": 75, "missing_side": 15, "orb_30m:censored_primary": 0, "orb_60m:censored_primary": 0, "own_momentum_fixed_horizon:censored_primary": 0, "production_s3:censored_primary": 0, "same_millisecond_groups": 79, "stale_side": 39}`. Same-millisecond quotes do not, by themselves, establish missing ticks or poor liquidity.

## §3. Overall results — each model and exit kept separate

| Model | Exit | bp/side | Entries | Closed | Wins | Win rate | Mean R | PF(R) | Censored |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| production_s3 | trailing | 24 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| production_s3 | trailing | 2 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| production_s3 | trailing | 3.74 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| production_s3 | fixed | 24 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| production_s3 | fixed | 2 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| production_s3 | fixed | 3.74 | 0 | 0 | 0 | N/A | N/A | N/A | 0 |
| orb_30m | trailing | 24 | 3 | 3 | 1 | 0.33333 | -0.52094 | 0.26882 | 0 |
| orb_30m | trailing | 2 | 3 | 3 | 2 | 0.66667 | 0.20306 | 1.61903 | 0 |
| orb_30m | trailing | 3.74 | 3 | 3 | 2 | 0.66667 | 0.14580 | 1.41683 | 0 |
| orb_30m | fixed | 24 | 3 | 3 | 1 | 0.33333 | -0.55132 | 0.25783 | 0 |
| orb_30m | fixed | 2 | 3 | 3 | 2 | 0.66667 | 0.17275 | 1.48209 | 0 |
| orb_30m | fixed | 3.74 | 3 | 3 | 2 | 0.66667 | 0.11549 | 1.30383 | 0 |
| orb_60m | trailing | 24 | 7 | 7 | 0 | 0.00000 | -0.78270 | 0.00000 | 0 |
| orb_60m | trailing | 2 | 7 | 7 | 4 | 0.57143 | -0.18581 | 0.48912 | 0 |
| orb_60m | trailing | 3.74 | 7 | 7 | 3 | 0.42857 | -0.23302 | 0.39503 | 0 |
| orb_60m | fixed | 24 | 7 | 7 | 0 | 0.00000 | -0.78270 | 0.00000 | 0 |
| orb_60m | fixed | 2 | 7 | 7 | 4 | 0.57143 | -0.18581 | 0.48912 | 0 |
| orb_60m | fixed | 3.74 | 7 | 7 | 3 | 0.42857 | -0.23302 | 0.39503 | 0 |
| always_long_control | fixed_horizon | 24 | 26 | 26 | 6 | 0.23077 | -1.04779 | 0.04863 | 0 |
| always_long_control | fixed_horizon | 2 | 26 | 26 | 12 | 0.46154 | -0.44220 | 0.33241 | 0 |
| always_long_control | fixed_horizon | 3.74 | 26 | 26 | 12 | 0.46154 | -0.49009 | 0.29134 | 0 |
| own_momentum_fixed_horizon | fixed_horizon | 24 | 26 | 26 | 5 | 0.19231 | -1.13135 | 0.03293 | 0 |
| own_momentum_fixed_horizon | fixed_horizon | 2 | 26 | 26 | 11 | 0.42308 | -0.52576 | 0.25206 | 0 |
| own_momentum_fixed_horizon | fixed_horizon | 3.74 | 26 | 26 | 11 | 0.42308 | -0.57366 | 0.21662 | 0 |

### §3.1. Per-symbol and asset-class primary results

No price-unit P&L is pooled across instruments. These are conditional scenario statistics, not account returns.

| Group | Model | Episodes | Entries | Closed | Wins | Win rate | Mean R | PF(R) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| symbol:BTCUSD | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| symbol:BTCUSD | orb_30m | 20 | 1 | 1 | 1 | 1.00000 | 0.57458 | N/A |
| symbol:BTCUSD | orb_60m | 20 | 2 | 2 | 0 | 0.00000 | -0.95947 | 0.00000 |
| symbol:BTCUSD | always_long_control | 20 | 10 | 10 | 1 | 0.10000 | -1.57537 | 0.02766 |
| symbol:BTCUSD | own_momentum_fixed_horizon | 20 | 10 | 10 | 2 | 0.20000 | -1.46896 | 0.03520 |
| symbol:SpotBrent | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| symbol:SpotBrent | orb_30m | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| symbol:SpotBrent | orb_60m | 20 | 1 | 1 | 0 | 0.00000 | -1.30482 | 0.00000 |
| symbol:SpotBrent | always_long_control | 20 | 4 | 4 | 1 | 0.25000 | -0.58624 | 0.03373 |
| symbol:SpotBrent | own_momentum_fixed_horizon | 20 | 4 | 4 | 1 | 0.25000 | -0.65010 | 0.03052 |
| symbol:SpotCrude | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| symbol:SpotCrude | orb_30m | 20 | 1 | 1 | 0 | 0.00000 | -1.80918 | 0.00000 |
| symbol:SpotCrude | orb_60m | 20 | 1 | 1 | 0 | 0.00000 | -1.34973 | 0.00000 |
| symbol:SpotCrude | always_long_control | 20 | 6 | 6 | 1 | 0.16667 | -1.31464 | 0.01681 |
| symbol:SpotCrude | own_momentum_fixed_horizon | 20 | 6 | 6 | 1 | 0.16667 | -1.33638 | 0.01654 |
| symbol:US500 | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| symbol:US500 | orb_30m | 20 | 1 | 1 | 0 | 0.00000 | -0.32823 | 0.00000 |
| symbol:US500 | orb_60m | 20 | 3 | 3 | 0 | 0.00000 | -0.30180 | 0.00000 |
| symbol:US500 | always_long_control | 20 | 6 | 6 | 3 | 0.50000 | -0.20933 | 0.36686 |
| symbol:US500 | own_momentum_fixed_horizon | 20 | 6 | 6 | 1 | 0.16667 | -0.68449 | 0.05717 |
| asset_class:crypto | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| asset_class:crypto | orb_30m | 20 | 1 | 1 | 1 | 1.00000 | 0.57458 | N/A |
| asset_class:crypto | orb_60m | 20 | 2 | 2 | 0 | 0.00000 | -0.95947 | 0.00000 |
| asset_class:crypto | always_long_control | 20 | 10 | 10 | 1 | 0.10000 | -1.57537 | 0.02766 |
| asset_class:crypto | own_momentum_fixed_horizon | 20 | 10 | 10 | 2 | 0.20000 | -1.46896 | 0.03520 |
| asset_class:energy | production_s3 | 40 | 0 | 0 | 0 | N/A | N/A | N/A |
| asset_class:energy | orb_30m | 40 | 1 | 1 | 0 | 0.00000 | -1.80918 | 0.00000 |
| asset_class:energy | orb_60m | 40 | 2 | 2 | 0 | 0.00000 | -1.32728 | 0.00000 |
| asset_class:energy | always_long_control | 40 | 10 | 10 | 2 | 0.20000 | -1.02328 | 0.02074 |
| asset_class:energy | own_momentum_fixed_horizon | 40 | 10 | 10 | 2 | 0.20000 | -1.06187 | 0.02000 |
| asset_class:equity_index | production_s3 | 20 | 0 | 0 | 0 | N/A | N/A | N/A |
| asset_class:equity_index | orb_30m | 20 | 1 | 1 | 0 | 0.00000 | -0.32823 | 0.00000 |
| asset_class:equity_index | orb_60m | 20 | 3 | 3 | 0 | 0.00000 | -0.30180 | 0.00000 |
| asset_class:equity_index | always_long_control | 20 | 6 | 6 | 3 | 0.50000 | -0.20933 | 0.36686 |
| asset_class:equity_index | own_momentum_fixed_horizon | 20 | 6 | 6 | 1 | 0.16667 | -0.68449 | 0.05717 |

## §4. Every symbol and cohort: primary trailing scenarios

| Cohort | Symbol | Model | Reason(s) | Entries | Closed | Wins | Mean R |
|---|---|---|---|---:|---:|---:|---:|
| 2024_Q1_Winter | BTCUSD | production_s3 | {"cost_ge_r_session_veto": 4, "inside_band_or_spread_too_wide": 4, "no_fresh_quote_in_mark_window": 3, "session_cost_veto": 4} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | BTCUSD | orb_30m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | BTCUSD | orb_60m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | BTCUSD | always_long_control | {"simulated_entry": 3, "no_fresh_quote_in_mark_window": 2} | 3 | 3 | 0 | -1.63454 |
| 2024_Q1_Winter | BTCUSD | own_momentum_fixed_horizon | {"simulated_entry": 3, "no_fresh_quote_in_mark_window": 2} | 3 | 3 | 0 | -1.63454 |
| 2024_Q1_Winter | SpotBrent | production_s3 | {"missing_opening_volume_or_prepared_spread": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotBrent | orb_30m | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotBrent | orb_60m | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotBrent | always_long_control | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotBrent | own_momentum_fixed_horizon | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotCrude | production_s3 | {"missing_opening_volume_or_prepared_spread": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotCrude | orb_30m | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotCrude | orb_60m | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotCrude | always_long_control | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | SpotCrude | own_momentum_fixed_horizon | {"missing_opening_range": 5} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | US500 | production_s3 | {"missing_opening_volume_or_prepared_spread": 4, "failed": 1} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | US500 | orb_30m | {"missing_opening_range": 4, "failed": 1} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | US500 | orb_60m | {"missing_opening_range": 4, "failed": 1} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | US500 | always_long_control | {"missing_opening_range": 4, "failed": 1} | 0 | 0 | 0 | N/A |
| 2024_Q1_Winter | US500 | own_momentum_fixed_horizon | {"missing_opening_range": 4, "failed": 1} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | BTCUSD | production_s3 | {"inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 1, "cost_ge_r_session_veto": 3, "session_cost_veto": 3} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | BTCUSD | orb_30m | {"range_expansion_veto": 4, "inside_opening_range": 1, "simulated_entry": 1} | 1 | 1 | 1 | 0.57458 |
| 2024_Q3_Summer | BTCUSD | orb_60m | {"range_expansion_veto": 4, "inside_opening_range": 1, "simulated_entry": 1} | 1 | 1 | 0 | -0.22197 |
| 2024_Q3_Summer | BTCUSD | always_long_control | {"simulated_entry": 3, "cost_ge_r_session_veto": 1, "no_next_fresh_fill_quote_before_deadline": 1} | 3 | 3 | 1 | -1.99593 |
| 2024_Q3_Summer | BTCUSD | own_momentum_fixed_horizon | {"simulated_entry": 3, "cost_ge_r_session_veto": 1, "no_next_fresh_fill_quote_before_deadline": 1} | 3 | 3 | 1 | -1.99593 |
| 2024_Q3_Summer | SpotBrent | production_s3 | {"cost_ge_r_session_veto": 3, "inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 3, "session_cost_veto": 3} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotBrent | orb_30m | {"cost_ge_r_session_veto": 3, "inside_opening_range": 3, "no_fresh_quote_in_mark_window": 2, "range_expansion_veto": 2} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotBrent | orb_60m | {"range_expansion_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotBrent | always_long_control | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotBrent | own_momentum_fixed_horizon | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotCrude | production_s3 | {"inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 3, "cost_ge_r_session_veto": 3, "session_cost_veto": 3} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotCrude | orb_30m | {"cost_ge_r_session_veto": 2, "inside_opening_range": 2, "no_fresh_quote_in_mark_window": 1, "range_expansion_veto": 3} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotCrude | orb_60m | {"range_expansion_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotCrude | always_long_control | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | SpotCrude | own_momentum_fixed_horizon | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | US500 | production_s3 | {"inside_band_or_spread_too_wide": 2, "cost_ge_r_session_veto": 4, "session_cost_veto": 4} | 0 | 0 | 0 | N/A |
| 2024_Q3_Summer | US500 | orb_30m | {"cost_ge_r_session_veto": 3, "inside_opening_range": 4, "simulated_entry": 1, "range_expansion_veto": 1} | 1 | 1 | 0 | -0.32823 |
| 2024_Q3_Summer | US500 | orb_60m | {"inside_opening_range": 5, "cost_ge_r_session_veto": 1, "simulated_entry": 3} | 3 | 3 | 0 | -0.30180 |
| 2024_Q3_Summer | US500 | always_long_control | {"simulated_entry": 4, "cost_ge_r_session_veto": 1} | 4 | 4 | 2 | -0.26770 |
| 2024_Q3_Summer | US500 | own_momentum_fixed_horizon | {"simulated_entry": 4, "cost_ge_r_session_veto": 1} | 4 | 4 | 1 | -0.55644 |
| 2025_Q2_Spring | BTCUSD | production_s3 | {"cost_ge_r_session_veto": 5, "session_cost_veto": 5, "inside_band_or_spread_too_wide": 4} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | BTCUSD | orb_30m | {"range_expansion_veto": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | BTCUSD | orb_60m | {"range_expansion_veto": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | BTCUSD | always_long_control | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -1.69593 |
| 2025_Q2_Spring | BTCUSD | own_momentum_fixed_horizon | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 1 | -0.79151 |
| 2025_Q2_Spring | SpotBrent | production_s3 | {"cost_ge_r_session_veto": 2, "session_cost_veto": 2, "inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 3} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotBrent | orb_30m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotBrent | orb_60m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotBrent | always_long_control | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 1 | -0.44043 |
| 2025_Q2_Spring | SpotBrent | own_momentum_fixed_horizon | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 1 | -0.44043 |
| 2025_Q2_Spring | SpotCrude | production_s3 | {"cost_ge_r_session_veto": 2, "session_cost_veto": 2, "inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 2} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotCrude | orb_30m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotCrude | orb_60m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | SpotCrude | always_long_control | {"simulated_entry": 3, "cost_ge_r_session_veto": 2} | 3 | 3 | 1 | -1.26093 |
| 2025_Q2_Spring | SpotCrude | own_momentum_fixed_horizon | {"simulated_entry": 3, "cost_ge_r_session_veto": 2} | 3 | 3 | 1 | -1.26093 |
| 2025_Q2_Spring | US500 | production_s3 | {"cost_ge_r_session_veto": 3, "session_cost_veto": 3, "inside_band_or_spread_too_wide": 4} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | US500 | orb_30m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | US500 | orb_60m | {"missing_exact_20_prior_ranges": 5} | 0 | 0 | 0 | N/A |
| 2025_Q2_Spring | US500 | always_long_control | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 1 | -0.09258 |
| 2025_Q2_Spring | US500 | own_momentum_fixed_horizon | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -0.94058 |
| 2025_Q4_PreHoldout | BTCUSD | production_s3 | {"cost_ge_r_session_veto": 5, "session_cost_veto": 5, "inside_band_or_spread_too_wide": 1} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | BTCUSD | orb_30m | {"inside_opening_range": 2, "range_expansion_veto": 3, "cost_ge_r_session_veto": 1} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | BTCUSD | orb_60m | {"inside_opening_range": 2, "range_expansion_veto": 3, "simulated_entry": 1} | 1 | 1 | 0 | -1.69698 |
| 2025_Q4_PreHoldout | BTCUSD | always_long_control | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -0.73524 |
| 2025_Q4_PreHoldout | BTCUSD | own_momentum_fixed_horizon | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -1.10759 |
| 2025_Q4_PreHoldout | SpotBrent | production_s3 | {"inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 3, "cost_ge_r_session_veto": 3, "session_cost_veto": 3} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | SpotBrent | orb_30m | {"cost_ge_r_session_veto": 1, "inside_opening_range": 1, "range_expansion_veto": 4} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | SpotBrent | orb_60m | {"inside_opening_range": 1, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 1, "range_expansion_veto": 4} | 1 | 1 | 0 | -1.30482 |
| 2025_Q4_PreHoldout | SpotBrent | always_long_control | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -0.73206 |
| 2025_Q4_PreHoldout | SpotBrent | own_momentum_fixed_horizon | {"simulated_entry": 2, "cost_ge_r_session_veto": 3} | 2 | 2 | 0 | -0.85978 |
| 2025_Q4_PreHoldout | SpotCrude | production_s3 | {"inside_band_or_spread_too_wide": 3, "no_fresh_quote_in_mark_window": 2, "cost_ge_r_session_veto": 3, "session_cost_veto": 3} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | SpotCrude | orb_30m | {"inside_opening_range": 1, "simulated_entry": 1, "range_expansion_veto": 4} | 1 | 1 | 0 | -1.80918 |
| 2025_Q4_PreHoldout | SpotCrude | orb_60m | {"inside_opening_range": 1, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 1, "range_expansion_veto": 4} | 1 | 1 | 0 | -1.34973 |
| 2025_Q4_PreHoldout | SpotCrude | always_long_control | {"simulated_entry": 3, "cost_ge_r_session_veto": 2} | 3 | 3 | 0 | -1.36835 |
| 2025_Q4_PreHoldout | SpotCrude | own_momentum_fixed_horizon | {"simulated_entry": 3, "cost_ge_r_session_veto": 2} | 3 | 3 | 0 | -1.41183 |
| 2025_Q4_PreHoldout | US500 | production_s3 | {"cost_ge_r_session_veto": 5, "session_cost_veto": 5, "inside_band_or_spread_too_wide": 3} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | US500 | orb_30m | {"cost_ge_r_session_veto": 3, "inside_opening_range": 3, "range_expansion_veto": 2} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | US500 | orb_60m | {"cost_ge_r_session_veto": 3, "inside_opening_range": 3, "range_expansion_veto": 2} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | US500 | always_long_control | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |
| 2025_Q4_PreHoldout | US500 | own_momentum_fixed_horizon | {"cost_ge_r_session_veto": 5} | 0 | 0 | 0 | N/A |

## §5. Matched controls and uncertainty

Compare on the same symbol/date only when both candidate and control close;
unmatched cases remain reported. Entry times and exit rules differ, so this is a
strategy comparison rather than an isolated directional effect. The predeclared
one-sided weekly sign-flip test has minimum p=0.0625 with four weeks, so Stage 1
cannot satisfy p<=0.05 by itself. No parameter or statistical gate was loosened.

```json
{
  "production_s3": [
    {
      "control": "always_long_control",
      "matched": 0,
      "unmatched_episodes": 80,
      "matched_weeks": 0,
      "mean_delta_r": null,
      "weekly_mean_delta_r": null,
      "one_sided_weekly_sign_flip_p": null,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": []
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 0,
      "unmatched_episodes": 80,
      "matched_weeks": 0,
      "mean_delta_r": null,
      "weekly_mean_delta_r": null,
      "one_sided_weekly_sign_flip_p": null,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": []
    }
  ],
  "orb_30m": [
    {
      "control": "always_long_control",
      "matched": 3,
      "unmatched_episodes": 77,
      "matched_weeks": 2,
      "mean_delta_r": -0.16591350272512248,
      "weekly_mean_delta_r": -0.48467717249838593,
      "one_sided_weekly_sign_flip_p": 0.75,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": [
        {
          "key": "US500-2024-07-24",
          "delta_r": 0.8167567449956341
        },
        {
          "key": "BTCUSD-2024-07-26",
          "delta_r": 0.12647092864717474
        },
        {
          "key": "SpotCrude-2025-10-06",
          "delta_r": -1.4409681818181763
        }
      ]
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 3,
      "unmatched_episodes": 77,
      "matched_weeks": 2,
      "mean_delta_r": -0.26393205508949463,
      "weekly_mean_delta_r": -0.4071619201049988,
      "one_sided_weekly_sign_flip_p": 0.75,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": [
        {
          "key": "US500-2024-07-24",
          "delta_r": -0.08141557876414746
        },
        {
          "key": "BTCUSD-2024-07-26",
          "delta_r": 0.12647092864717474
        },
        {
          "key": "SpotCrude-2025-10-06",
          "delta_r": -0.8368515151515112
        }
      ]
    }
  ],
  "orb_60m": [
    {
      "control": "always_long_control",
      "matched": 7,
      "unmatched_episodes": 73,
      "matched_weeks": 2,
      "mean_delta_r": -0.47556668686714054,
      "weekly_mean_delta_r": -0.5104449714424972,
      "one_sided_weekly_sign_flip_p": 1.0,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": [
        {
          "key": "US500-2024-07-24",
          "delta_r": 0.5595613577023357
        },
        {
          "key": "US500-2024-07-25",
          "delta_r": -0.5568888888888976
        },
        {
          "key": "US500-2024-07-26",
          "delta_r": -0.3977777777777892
        },
        {
          "key": "BTCUSD-2024-07-26",
          "delta_r": -0.6700826086956522
        },
        {
          "key": "SpotCrude-2025-10-06",
          "delta_r": -0.9815166666666613
        },
        {
          "key": "SpotBrent-2025-10-06",
          "delta_r": -0.9606333333333454
        },
        {
          "key": "BTCUSD-2025-10-08",
          "delta_r": -0.3216288904099738
        }
      ]
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 7,
      "unmatched_episodes": 73,
      "matched_weeks": 2,
      "mean_delta_r": -0.1201093859825798,
      "weekly_mean_delta_r": -0.14386844427961695,
      "one_sided_weekly_sign_flip_p": 0.75,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "pairs": [
        {
          "key": "US500-2024-07-24",
          "delta_r": -0.33861096605744584
        },
        {
          "key": "US500-2024-07-25",
          "delta_r": 1.4962512077294596
        },
        {
          "key": "US500-2024-07-26",
          "delta_r": -0.3977777777777892
        },
        {
          "key": "BTCUSD-2024-07-26",
          "delta_r": -0.6700826086956522
        },
        {
          "key": "SpotCrude-2025-10-06",
          "delta_r": -0.3773999999999962
        },
        {
          "key": "SpotBrent-2025-10-06",
          "delta_r": -0.23151666666666104
        },
        {
          "key": "BTCUSD-2025-10-08",
          "delta_r": -0.3216288904099738
        }
      ]
    }
  ]
}
```

## §6. Failure prevention and next decision

1. Keep source receipts and distinguish successful downloads from complete history,
   complete simulated positions, and verified real fills. No failed job is retried
   and no date replaced to improve performance. Same-millisecond flags are sequencing
   uncertainty, not proof that the instrument is illiquid.
2. Review each model's missing-band/range/quote reasons before proposing changes.
   A four-hour censored S3 position is not a losing close; a cost veto is not a trade.
3. Calibrate costs independently with representative paired broker fills/fees;
   lower-cost sensitivity gains cannot rescue primary admission or certify an edge.
4. Stop tuning a candidate whose completed primary scenarios have nonpositive means
   in both years under this plan. Missing or censored evidence is inconclusive.
   Any broader full-session collection or new hypothesis needs a new frozen budget.
5. No automatic Stage 2 or validation. Before any promotion require sufficient
   independent weeks/closes, per-symbol and class economics, matched-control evidence,
   intact coverage, cost stress and account/execution parity. Software tests are separate.

Lower-cost arithmetic on rejected marks (not trades): `{"2": 134, "3.74": 134}`. Full rejected-mark calculations are in JSON.
Both-year stopping flags: `{"production_s3": false, "orb_30m": false, "orb_60m": true}`.

## §7. Reproducibility, checks and publication

Plan SHA256: `2801e7267d90875bc1bafef20f7cee3469a98fe6c89ddeb8bba2ef79f7777d4e`.
Execution SHA256: `d6e354759d50213236aefe167c01c5cca8a2be167d1289c666a2672d82caf3b3`.
Collection/replay commit: `5d712c5146b75d396b90311877abb106e3ed741a`.
Actual pre-read freeze: `2026-11-07T03:37:23.395515Z`.
Binary SHA256: `ce9fc93a960ecfc045674941bd5270c07a0fe09a310bc059935b525581f81919`.

Raw data and append-only segment/episode receipts remain at `/workspace/jiaoyi-seasonal-stage1-18316` in this
workspace, not an archival service. Repository JSON includes all native outcomes,
source hashes, receipts, code hashes and per-symbol/class/cohort/cost scores;
raw quotes are not duplicated into Git. Independent raw reproduction requires
that retained corpus. The JSON records separately the post-replay narrative/source-accounting enrichment.
Run `seasonal_pilot.py replay` then `seasonal_report.py`
against it with the frozen native `seasonal_replay` binary. Plan and report PRs
carry the required CI and merge receipts.

Focused checks cover protected-date segmentation, DST, M30 decoding, omitted
receipts, exact prior-session ranges, causal ORB preparation, same-entry sensitivities,
HTTP scope preservation and S3 regression compatibility.

### §7.1. Independent post-replay verification

- All 196 component/episode receipt pairs reconcile; every raw file hash matches its receipt. All 80 planned episodes remain in the report.
- All replayed quote and M30 timestamps exclude the protected validation interval. Frozen binary identity matches the original pre-read hash.
- Native exit-cost sensitivity values and every aggregate's closed count, wins and mean R were independently recomputed from the retained outcomes.
- Plan PR #178 and transport repair #179 passed `check`, `docker` and `engineering`; native regression and focused Python checks are recorded in the journal. Report publication is subject to its own required CI.

These are engineering and accounting checks. They do not repair missing historical preparation or qualify financial performance. The JSON embeds the verification receipt.

## §8. Independent AI review and copy-paste block

Read this report, the addendum, native result JSON and frozen input plan. Challenge
coverage, historical contract and fee assumptions, the conditional entry denominators,
the four-hour S3 censoring, and paired-comparison selection. Do not pool models or
infer an edge from a small winning subset. Propose only the smallest evidence-backed
next experiment, preserving losses and failed gates.

The final chat block adds the actual report PR merge commit after publication.
Full results: [JSON](../docs/seasonal-pilot-results-2024-2025.json).
