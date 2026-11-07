# Three-year seasonal pilot — real-data review

№18,325 §§1–8 · 07-11-2026 14:23:10 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**120 logical episodes accounted for; 119 successful native replays; 1 failed or unavailable.**
120/120 have at least one source, preparation, retry, sequencing or censoring flag.
No candidate receives a financial PASS. The interim 50% / positive-economics target and
original 73% net wins / reference R:R 1:1.38 over latest20 per strategy/account remain unproven.
This is a 30-date, six-week pilot, not three complete years or proof of seasonal causality.
No live change, validation opening, automatic Stage2, Telegram or model API call.

The supplied runner fabricated outcomes using symbols/job numbers and fixed PF/control values.
It was replaced before execution. No manufactured results were used as market evidence.
Full method: [frozen execution addendum](../docs/three-year-execution-addendum-18324.md).

## §2. What was tested and retained

Four symbols: SpotCrude, SpotBrent, US500 and XAUUSD; winter/summer weeks in 2023–2025.
Ten prior US500 sources were reused by hash, including the prior failed job; missing prior
M30 remains missing. New sources use read-only broker calls and append-only receipts.
All failed, vetoed, missing, censored and retried episodes remain in the 120 denominator.
Closed win rate uses all closes including breakevens. Missing metrics are null, not zero returns.

Native S3 runs from registered open (07 UTC oils/gold, DST-aware09:30 New York US500).
Oil quotes include07–17 UTC to reconstruct S3 state; ORB, macro and controls use13–17 UTC.
S3 retains its actual session close; open trades at the observation boundary are censored.
Others close at hour four. Gold/US500 Macro-180 lack an authorised class peer and are
explicitly unevaluable; no empty-peer “unanimity” or synthetic macro information is invented.
The Macro-180 rule is a newly frozen price-momentum hypothesis, not a proven macro strategy.

ORB-60 exact-prior20 range expansion and Macro-180 use the causal dynamic range filter
R > max(2.5×spread,0.15×ATR14), plus unchanged primary24bp/side cost admission.
2/3.74bp sensitivities reprice identical admitted fills; they do not create new entries
and are not calibrated costs. Shared native Q1.38 targets, SL/TP and original-R trail;
zero extra fees/current contracts/assumed next-quote execution limit account parity.
Bid/ask sequence flags do not by themselves prove missing ticks or illiquidity.

## §3. Overall scenarios — do not pool models or exits

| Model | Exit | bp/side | Entries | Closed | Wins | Win rate | Mean R | PF(R) | Censored | Flagged episodes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| production_s3 | trailing | 24 | 5 | 3 | 0 | 0.00000 | -0.87249 | 0.00000 | 2 | 120 |
| production_s3 | trailing | 2 | 5 | 3 | 2 | 0.66667 | -0.19818 | 0.18017 | 2 | 120 |
| production_s3 | trailing | 3.74 | 5 | 3 | 2 | 0.66667 | -0.25151 | 0.04010 | 2 | 120 |
| production_s3 | fixed | 24 | 5 | 3 | 1 | 0.33333 | -0.65670 | 0.42050 | 2 | 120 |
| production_s3 | fixed | 2 | 5 | 3 | 1 | 0.33333 | 0.01497 | 1.02102 | 2 | 120 |
| production_s3 | fixed | 3.74 | 5 | 3 | 1 | 0.33333 | -0.03815 | 0.94884 | 2 | 120 |
| orb_60m | trailing | 24 | 11 | 10 | 1 | 0.10000 | -0.38061 | 0.14285 | 1 | 120 |
| orb_60m | trailing | 2 | 11 | 10 | 7 | 0.70000 | 0.21398 | 3.88522 | 1 | 120 |
| orb_60m | trailing | 3.74 | 11 | 10 | 6 | 0.60000 | 0.16695 | 2.84630 | 1 | 120 |
| orb_60m | fixed | 24 | 11 | 10 | 1 | 0.10000 | -0.38061 | 0.14285 | 1 | 120 |
| orb_60m | fixed | 2 | 11 | 10 | 7 | 0.70000 | 0.21398 | 3.88522 | 1 | 120 |
| orb_60m | fixed | 3.74 | 11 | 10 | 6 | 0.60000 | 0.16695 | 2.84630 | 1 | 120 |
| macro_180m | trailing | 24 | 55 | 54 | 11 | 0.20370 | -0.36004 | 0.04218 | 1 | 120 |
| macro_180m | trailing | 2 | 55 | 54 | 27 | 0.50000 | -0.04319 | 0.72873 | 1 | 120 |
| macro_180m | trailing | 3.74 | 55 | 54 | 26 | 0.48148 | -0.06825 | 0.60601 | 1 | 120 |
| macro_180m | fixed | 24 | 55 | 54 | 11 | 0.20370 | -0.36085 | 0.04209 | 1 | 120 |
| macro_180m | fixed | 2 | 55 | 54 | 27 | 0.50000 | -0.04400 | 0.72503 | 1 | 120 |
| macro_180m | fixed | 3.74 | 55 | 54 | 26 | 0.48148 | -0.06906 | 0.60318 | 1 | 120 |
| always_long_control | fixed_horizon | 24 | 53 | 52 | 18 | 0.34615 | -0.38462 | 0.35966 | 1 | 120 |
| always_long_control | fixed_horizon | 2 | 53 | 52 | 31 | 0.59615 | 0.20676 | 1.72166 | 1 | 120 |
| always_long_control | fixed_horizon | 3.74 | 53 | 52 | 30 | 0.57692 | 0.15999 | 1.52388 | 1 | 120 |
| own_momentum_fixed_horizon | fixed_horizon | 24 | 52 | 51 | 16 | 0.31373 | -0.52755 | 0.24447 | 1 | 120 |
| own_momentum_fixed_horizon | fixed_horizon | 2 | 52 | 51 | 28 | 0.54902 | 0.06110 | 1.16936 | 1 | 120 |
| own_momentum_fixed_horizon | fixed_horizon | 3.74 | 52 | 51 | 28 | 0.54902 | 0.01454 | 1.03804 | 1 | 120 |

## §4. Symbol, class and cohort primary results

| Group | Model | Episodes | Entries | Closed | Wins | Mean R | Reasons (overlap) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| symbol:SpotBrent | production_s3 | 30 | 2 | 1 | 0 | -1.49470 | {"cost_ge_r_session_veto": 25, "inside_band_or_spread_too_wide": 16, "no_fresh_quote_in_mark_window": 6, "one_entry_already_consumed": 2, "session_cost_veto": 25, "simulated_entry": 2} |
| symbol:SpotBrent | orb_60m | 30 | 3 | 3 | 1 | -0.15979 | {"inside_opening_range": 3, "missing_exact_20_prior_ranges": 10, "range_expansion_veto": 17, "simulated_entry": 3} |
| symbol:SpotBrent | macro_180m | 30 | 27 | 27 | 5 | -0.38939 | {"no_fresh_quote_in_mark_window": 1, "no_next_fresh_fill_quote_before_deadline": 1, "peer_or_direction_veto": 1, "simulated_entry": 27} |
| symbol:SpotBrent | always_long_control | 30 | 19 | 19 | 7 | -0.43434 | {"cost_ge_r_session_veto": 10, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 19} |
| symbol:SpotBrent | own_momentum_fixed_horizon | 30 | 19 | 19 | 6 | -0.62120 | {"cost_ge_r_session_veto": 10, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 19} |
| symbol:SpotCrude | production_s3 | 30 | 3 | 2 | 0 | -0.56139 | {"cost_ge_r_session_veto": 25, "inside_band_or_spread_too_wide": 14, "no_fresh_quote_in_mark_window": 6, "one_entry_already_consumed": 3, "session_cost_veto": 25, "simulated_entry": 3} |
| symbol:SpotCrude | orb_60m | 30 | 3 | 2 | 0 | -0.38522 | {"inside_opening_range": 4, "missing_exact_20_prior_ranges": 10, "range_expansion_veto": 16, "simulated_entry": 3} |
| symbol:SpotCrude | macro_180m | 30 | 28 | 27 | 6 | -0.33069 | {"no_fresh_quote_in_mark_window": 1, "peer_or_direction_veto": 1, "simulated_entry": 28} |
| symbol:SpotCrude | always_long_control | 30 | 21 | 20 | 7 | -0.35410 | {"cost_ge_r_session_veto": 8, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 21} |
| symbol:SpotCrude | own_momentum_fixed_horizon | 30 | 20 | 19 | 8 | -0.39718 | {"cost_ge_r_session_veto": 8, "direction_changed_before_simulated_fill": 1, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 20} |
| symbol:US500 | production_s3 | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 19, "failed": 1, "inside_band_or_spread_too_wide": 13, "missing_opening_volume_or_prepared_spread": 4, "no_fresh_quote_in_mark_window": 2, "session_cost_veto": 19} |
| symbol:US500 | orb_60m | 30 | 5 | 5 | 0 | -0.51126 | {"cost_ge_r_session_veto": 1, "failed": 1, "inside_opening_range": 9, "missing_exact_20_prior_ranges": 5, "missing_opening_range": 4, "no_fresh_quote_in_mark_window": 1, "range_expansion_veto": 11, "simulated_entry": 5} |
| symbol:US500 | macro_180m | 30 | 0 | 0 | 0 | N/A | {"failed": 1, "missing_opening_range": 4, "missing_required_class_peer": 25} |
| symbol:US500 | always_long_control | 30 | 11 | 11 | 4 | -0.24224 | {"cost_ge_r_session_veto": 14, "failed": 1, "missing_opening_range": 4, "simulated_entry": 11} |
| symbol:US500 | own_momentum_fixed_horizon | 30 | 11 | 11 | 2 | -0.62300 | {"cost_ge_r_session_veto": 14, "failed": 1, "missing_opening_range": 4, "simulated_entry": 11} |
| symbol:XAUUSD | production_s3 | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 19, "inside_band_or_spread_too_wide": 22, "session_cost_veto": 18} |
| symbol:XAUUSD | orb_60m | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 5, "inside_opening_range": 7, "missing_exact_20_prior_ranges": 10, "range_expansion_veto": 13} |
| symbol:XAUUSD | macro_180m | 30 | 0 | 0 | 0 | N/A | {"missing_required_class_peer": 30} |
| symbol:XAUUSD | always_long_control | 30 | 2 | 2 | 0 | -1.00058 | {"cost_ge_r_session_veto": 28, "simulated_entry": 2} |
| symbol:XAUUSD | own_momentum_fixed_horizon | 30 | 2 | 2 | 0 | -0.35141 | {"cost_ge_r_session_veto": 28, "simulated_entry": 2} |
| asset_class:energy | production_s3 | 60 | 5 | 3 | 0 | -0.87249 | {"cost_ge_r_session_veto": 50, "inside_band_or_spread_too_wide": 30, "no_fresh_quote_in_mark_window": 12, "one_entry_already_consumed": 5, "session_cost_veto": 50, "simulated_entry": 5} |
| asset_class:energy | orb_60m | 60 | 6 | 5 | 1 | -0.24997 | {"inside_opening_range": 7, "missing_exact_20_prior_ranges": 20, "range_expansion_veto": 33, "simulated_entry": 6} |
| asset_class:energy | macro_180m | 60 | 55 | 54 | 11 | -0.36004 | {"no_fresh_quote_in_mark_window": 2, "no_next_fresh_fill_quote_before_deadline": 1, "peer_or_direction_veto": 2, "simulated_entry": 55} |
| asset_class:energy | always_long_control | 60 | 40 | 39 | 14 | -0.39319 | {"cost_ge_r_session_veto": 18, "no_fresh_quote_in_mark_window": 1, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 40} |
| asset_class:energy | own_momentum_fixed_horizon | 60 | 39 | 38 | 14 | -0.50919 | {"cost_ge_r_session_veto": 18, "direction_changed_before_simulated_fill": 1, "no_fresh_quote_in_mark_window": 1, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 39} |
| asset_class:equity_index | production_s3 | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 19, "failed": 1, "inside_band_or_spread_too_wide": 13, "missing_opening_volume_or_prepared_spread": 4, "no_fresh_quote_in_mark_window": 2, "session_cost_veto": 19} |
| asset_class:equity_index | orb_60m | 30 | 5 | 5 | 0 | -0.51126 | {"cost_ge_r_session_veto": 1, "failed": 1, "inside_opening_range": 9, "missing_exact_20_prior_ranges": 5, "missing_opening_range": 4, "no_fresh_quote_in_mark_window": 1, "range_expansion_veto": 11, "simulated_entry": 5} |
| asset_class:equity_index | macro_180m | 30 | 0 | 0 | 0 | N/A | {"failed": 1, "missing_opening_range": 4, "missing_required_class_peer": 25} |
| asset_class:equity_index | always_long_control | 30 | 11 | 11 | 4 | -0.24224 | {"cost_ge_r_session_veto": 14, "failed": 1, "missing_opening_range": 4, "simulated_entry": 11} |
| asset_class:equity_index | own_momentum_fixed_horizon | 30 | 11 | 11 | 2 | -0.62300 | {"cost_ge_r_session_veto": 14, "failed": 1, "missing_opening_range": 4, "simulated_entry": 11} |
| asset_class:metal | production_s3 | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 19, "inside_band_or_spread_too_wide": 22, "session_cost_veto": 18} |
| asset_class:metal | orb_60m | 30 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 5, "inside_opening_range": 7, "missing_exact_20_prior_ranges": 10, "range_expansion_veto": 13} |
| asset_class:metal | macro_180m | 30 | 0 | 0 | 0 | N/A | {"missing_required_class_peer": 30} |
| asset_class:metal | always_long_control | 30 | 2 | 2 | 0 | -1.00058 | {"cost_ge_r_session_veto": 28, "simulated_entry": 2} |
| asset_class:metal | own_momentum_fixed_horizon | 30 | 2 | 2 | 0 | -0.35141 | {"cost_ge_r_session_veto": 28, "simulated_entry": 2} |
| cohort:2023-Q1-WINTER | production_s3 | 20 | 2 | 2 | 0 | -0.96104 | {"cost_ge_r_session_veto": 15, "inside_band_or_spread_too_wide": 11, "one_entry_already_consumed": 2, "session_cost_veto": 14, "simulated_entry": 2} |
| cohort:2023-Q1-WINTER | orb_60m | 20 | 0 | 0 | 0 | N/A | {"missing_exact_20_prior_ranges": 20} |
| cohort:2023-Q1-WINTER | macro_180m | 20 | 10 | 10 | 5 | -0.20747 | {"missing_required_class_peer": 10, "simulated_entry": 10} |
| cohort:2023-Q1-WINTER | always_long_control | 20 | 11 | 11 | 4 | -0.67955 | {"cost_ge_r_session_veto": 9, "simulated_entry": 11} |
| cohort:2023-Q1-WINTER | own_momentum_fixed_horizon | 20 | 10 | 10 | 3 | -0.49595 | {"cost_ge_r_session_veto": 9, "direction_changed_before_simulated_fill": 1, "simulated_entry": 10} |
| cohort:2023-Q3-SUMMER | production_s3 | 20 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 18, "inside_band_or_spread_too_wide": 7, "session_cost_veto": 18} |
| cohort:2023-Q3-SUMMER | orb_60m | 20 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 1, "inside_opening_range": 3, "range_expansion_veto": 17} |
| cohort:2023-Q3-SUMMER | macro_180m | 20 | 10 | 10 | 2 | -0.14545 | {"missing_required_class_peer": 10, "simulated_entry": 10} |
| cohort:2023-Q3-SUMMER | always_long_control | 20 | 8 | 8 | 5 | 0.33301 | {"cost_ge_r_session_veto": 11, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 8} |
| cohort:2023-Q3-SUMMER | own_momentum_fixed_horizon | 20 | 8 | 8 | 3 | -0.25741 | {"cost_ge_r_session_veto": 11, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 8} |
| cohort:2024-Q1-WINTER | production_s3 | 20 | 2 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 9, "failed": 1, "inside_band_or_spread_too_wide": 12, "missing_opening_volume_or_prepared_spread": 4, "one_entry_already_consumed": 2, "session_cost_veto": 9, "simulated_entry": 2} |
| cohort:2024-Q1-WINTER | orb_60m | 20 | 0 | 0 | 0 | N/A | {"failed": 1, "missing_exact_20_prior_ranges": 15, "missing_opening_range": 4} |
| cohort:2024-Q1-WINTER | macro_180m | 20 | 10 | 10 | 4 | -0.26906 | {"failed": 1, "missing_opening_range": 4, "missing_required_class_peer": 5, "simulated_entry": 10} |
| cohort:2024-Q1-WINTER | always_long_control | 20 | 7 | 7 | 4 | 0.08217 | {"cost_ge_r_session_veto": 8, "failed": 1, "missing_opening_range": 4, "simulated_entry": 7} |
| cohort:2024-Q1-WINTER | own_momentum_fixed_horizon | 20 | 7 | 7 | 2 | -0.49212 | {"cost_ge_r_session_veto": 8, "failed": 1, "missing_opening_range": 4, "simulated_entry": 7} |
| cohort:2024-Q3-SUMMER | production_s3 | 20 | 1 | 1 | 0 | -0.69539 | {"cost_ge_r_session_veto": 16, "inside_band_or_spread_too_wide": 12, "no_fresh_quote_in_mark_window": 7, "one_entry_already_consumed": 1, "session_cost_veto": 16, "simulated_entry": 1} |
| cohort:2024-Q3-SUMMER | orb_60m | 20 | 5 | 4 | 1 | -0.06777 | {"cost_ge_r_session_veto": 2, "inside_opening_range": 9, "range_expansion_veto": 11, "simulated_entry": 5} |
| cohort:2024-Q3-SUMMER | macro_180m | 20 | 10 | 9 | 0 | -0.42015 | {"missing_required_class_peer": 10, "simulated_entry": 10} |
| cohort:2024-Q3-SUMMER | always_long_control | 20 | 14 | 13 | 5 | -0.23739 | {"cost_ge_r_session_veto": 6, "simulated_entry": 14} |
| cohort:2024-Q3-SUMMER | own_momentum_fixed_horizon | 20 | 14 | 13 | 3 | -0.93666 | {"cost_ge_r_session_veto": 6, "simulated_entry": 14} |
| cohort:2025-Q1-WINTER | production_s3 | 20 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 15, "inside_band_or_spread_too_wide": 12, "no_fresh_quote_in_mark_window": 4, "session_cost_veto": 15} |
| cohort:2025-Q1-WINTER | orb_60m | 20 | 4 | 4 | 0 | -0.78556 | {"cost_ge_r_session_veto": 2, "inside_opening_range": 6, "range_expansion_veto": 14, "simulated_entry": 4} |
| cohort:2025-Q1-WINTER | macro_180m | 20 | 7 | 7 | 0 | -0.46674 | {"missing_required_class_peer": 10, "no_fresh_quote_in_mark_window": 1, "peer_or_direction_veto": 2, "simulated_entry": 7} |
| cohort:2025-Q1-WINTER | always_long_control | 20 | 4 | 4 | 0 | -0.98309 | {"cost_ge_r_session_veto": 16, "simulated_entry": 4} |
| cohort:2025-Q1-WINTER | own_momentum_fixed_horizon | 20 | 4 | 4 | 1 | -0.21585 | {"cost_ge_r_session_veto": 16, "simulated_entry": 4} |
| cohort:2025-Q3-SUMMER | production_s3 | 20 | 0 | 0 | 0 | N/A | {"cost_ge_r_session_veto": 15, "inside_band_or_spread_too_wide": 11, "no_fresh_quote_in_mark_window": 3, "session_cost_veto": 15} |
| cohort:2025-Q3-SUMMER | orb_60m | 20 | 2 | 2 | 0 | -0.19640 | {"cost_ge_r_session_veto": 1, "inside_opening_range": 5, "no_fresh_quote_in_mark_window": 1, "range_expansion_veto": 15, "simulated_entry": 2} |
| cohort:2025-Q3-SUMMER | macro_180m | 20 | 8 | 8 | 0 | -0.77174 | {"missing_required_class_peer": 10, "no_fresh_quote_in_mark_window": 1, "no_next_fresh_fill_quote_before_deadline": 1, "simulated_entry": 8} |
| cohort:2025-Q3-SUMMER | always_long_control | 20 | 9 | 9 | 0 | -0.97179 | {"cost_ge_r_session_veto": 10, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 9} |
| cohort:2025-Q3-SUMMER | own_momentum_fixed_horizon | 20 | 9 | 9 | 4 | -0.37796 | {"cost_ge_r_session_veto": 10, "no_fresh_quote_in_mark_window": 1, "simulated_entry": 9} |

## §5. Matched controls and year evidence

Same-symbol/date closed pairs only, with unmatched episodes preserved. S3 and control
windows/exit policies differ, so paired deltas do not isolate a directional cause.
Six-week sign-flip tests have limited resolution; Bonferroni adjustment for six candidate
comparisons is reported. No threshold is relaxed after seeing outcomes.

{
  "production_s3": [
    {
      "control": "always_long_control",
      "matched": 3,
      "unmatched_episodes": 117,
      "matched_weeks": 2,
      "mean_delta_r": 0.36645104266033907,
      "weekly_mean_delta_r": 0.4064580849962702,
      "one_sided_weekly_sign_flip_p": 0.25,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 3,
      "unmatched_episodes": 117,
      "matched_weeks": 2,
      "mean_delta_r": -0.07486351133027021,
      "weekly_mean_delta_r": -0.25551374598964377,
      "one_sided_weekly_sign_flip_p": 0.75,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    }
  ],
  "orb_60m": [
    {
      "control": "always_long_control",
      "matched": 10,
      "unmatched_episodes": 110,
      "matched_weeks": 3,
      "mean_delta_r": 0.14108841929922555,
      "weekly_mean_delta_r": 0.2784535980932702,
      "one_sided_weekly_sign_flip_p": 0.375,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 10,
      "unmatched_episodes": 110,
      "matched_weeks": 3,
      "mean_delta_r": 0.1009653475132467,
      "weekly_mean_delta_r": 0.026287546208128232,
      "one_sided_weekly_sign_flip_p": 0.5,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    }
  ],
  "macro_180m": [
    {
      "control": "always_long_control",
      "matched": 36,
      "unmatched_episodes": 84,
      "matched_weeks": 6,
      "mean_delta_r": 0.04539826588106826,
      "weekly_mean_delta_r": 0.2528648303908339,
      "one_sided_weekly_sign_flip_p": 0.234375,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    },
    {
      "control": "own_momentum_fixed_horizon",
      "matched": 35,
      "unmatched_episodes": 85,
      "matched_weeks": 6,
      "mean_delta_r": 0.19104737510238337,
      "weekly_mean_delta_r": 0.0935074804332421,
      "one_sided_weekly_sign_flip_p": 0.328125,
      "significance_pass": false,
      "caveat": "Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.",
      "bonferroni_p": 1
    }
  ]
}

Year means/closes: `{"production_s3": {"2023": {"closed": 2, "wins": 0, "mean_r": -0.9610364695340502}, "2024": {"closed": 1, "wins": 0, "mean_r": -0.6953934640522875}, "2025": {"closed": 0, "wins": 0, "mean_r": null}}, "orb_60m": {"2023": {"closed": 0, "wins": 0, "mean_r": null}, "2024": {"closed": 4, "wins": 1, "mean_r": -0.06776806559489106}, "2025": {"closed": 6, "wins": 0, "mean_r": -0.5891731558245897}}, "macro_180m": {"2023": {"closed": 20, "wins": 7, "mean_r": -0.17645871921006356}, "2024": {"closed": 19, "wins": 4, "mean_r": -0.3406289323348344}, "2025": {"closed": 15, "wins": 0, "mean_r": -0.6294091143327517}}}`.

## §6. Failure prevention and next decision

1. Preserve evidence and keep engineering PASS separate from economic qualification.
2. Diagnose actual reasons: missing prior bars/peers, cost veto, lack of signal, and open
   censored positions are different. Never label them losing closes or fabricate returns.
3. Independently calibrate costs, historical quantities and execution assumptions before a
   replacement-cost trial. A lower-cost winning subset does not rescue primary failure.
4. Respect the frozen stopping rule; do not tune to a target hit rate. A new hypothesis needs
   an independently justified mechanism, a new freeze and sufficient untouched development
   dates/weeks. Prior reused dates are development evidence, not fresh out-of-sample evidence.
5. Keep validation sealed and Stage2 inactive. Only verified evidence can justify another run.

Stopping flags (nonpositive completed means in both2023/2024): `{"production_s3": true, "orb_60m": false, "macro_180m": true}`. Missing evidence is inconclusive.

## §7. Reproducibility and verification

Plan SHA256: `a7b09cde60c398a2c8502aed5cf401c24ad53df0ecdd123218510cff6b5256ff`. Manifest SHA256: `942e548d708857fea2469022976c81d81b809c5fda1a12ff494dd13910eb9b95`.
Plan merge: `6cd8e98a25913dd9cc45daba5b33d96726f0c1c6`.
Binary SHA256: `8dd57c65fd723e0165fce0efa1cf42a7f3cff7dc1630b145de1df86c1812151b`.
Pre-read freeze: `2026-11-07T06:08:54.478770Z`.
Verification: `{"verified_at": "2026-11-07T06:23:10.118342Z", "receipt_pairs": 296, "quote_updates": 3796741, "fresh_tick_pages": 842, "checks": ["all source receipts/hashes", "all 120 episode identities", "all five model outputs per replay", "protected quote exclusion", "independent score counts/wins/means"], "financial_pass": false}`.
Raw sources: `/workspace/jiaoyi-three-year-pilot-18325`; retained workspace corpus, not permanent archival storage.
Repository JSON includes all native outcomes, receipts, source hashes and grouped scores;
raw quotes are not copied into Git. Reproduction requires that corpus and frozen binary.
Report publication uses explicit successful required CI and verifies main-file hashes.

## §8. Independent AI review

Read this report, the addendum, source plan and [full JSON](../docs/three-year-pilot-results-2023-2025.json).
Challenge chronology, reused/failed evidence, actual costs, per-class/individual symbol
sample size, peer availability, tick ordering and matched-control selection. Do not
repeat the source prompt's fixed winning numbers or claim a full backtest was completed
where evidence is missing. Propose the smallest evidence-backed next action.
