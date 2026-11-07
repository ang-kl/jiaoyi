# Fresh macro-event cohorts: measured native two-stage replay

№18,348 §§1–12 · 07-11-2026 23:18:24 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

Completed the fixed **40 episode slots**, with **40 complete tick sources** and **40 successful native replays**. Results below are measured from broker receipts and compiled Rust paths. No financial PASS, account qualification or production activation is inferred.

| Model / exit policy | Entries | Closed | Wins | Losses | Censored | Net win rate | Mean net R | PF(R) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| s3_two_stage | 3 | 2 | 0 | 2 | 1 | 0.000000 | -1.237774 | 0.000000 |
| s3_two_stage_window_close | 3 | 3 | 0 | 3 | 0 | 0.000000 | -1.064041 | 0.000000 |
| always_long_control | 30 | 30 | 11 | 19 | 0 | 0.366667 | -0.499312 | 0.373664 |
| own_momentum_fixed_horizon | 30 | 30 | 7 | 23 | 0 | 0.233333 | -0.950518 | 0.158005 |

**Full like-for-like legacy milestone: 8/20 unique closes**, 3 wins, mean -0.451137R, PF(R) 0.355510. This includes the later May loss and censored entries. The original **73% net win rate / reference 1:1.38 risk:reward on the latest 20 trades per strategy/account** remains unqualified by pooled historical research.

## §2. Frozen scope and model interpretation

Election week: 4–8 November 2024. Tariff week: 7–11 April 2025. Symbols: SpotCrude, SpotBrent, US500 and XAUUSD. Every registered prior tick window was checked before reads. Requested M30 intervals were partitioned into unspent reads and verified cached slices. The plan, code and native binary hashes were recorded before broker calls; failures and uncertain receipts cannot silently trigger retries.

The inherited `s3_two_stage` engine closes at its native session end, which can lie beyond the observation boundary. The owner-requested four-hour-window close is therefore explicitly named `s3_two_stage_window_close`. It uses identical entry, original risk, original stop, +1R next-quote entry-price amendment and gross +1.38R target, but its exit deadline is the evaluation-window end. A valid quote must arrive within the five-second closing tail; otherwise the observation remains censored. Existing broker SL/TP has priority. No synthetic boundary fill is inserted.

US500 evaluation begins at 09:30 America/New_York; gold begins at 07:00 UTC. Oil evaluation is 13:00–17:00 UTC, with the inherited S3 quote/entry horizon starting at 07:00 UTC. Controls retain their existing four-hour-window entry rules. They match dates, data and costs, but oil S3 can have earlier exposure; this is not a perfectly exposure-matched alpha attribution.

Primary admission remains strict full C<R (24 bp per side plus prepared spread, zero extra commission assumption), with entry R checked against the 48 bp floor. Bid/ask spread is already reflected in executable fills and is not charged twice. The 24 bp reserve and historical fee/contract terms remain uncalibrated. An entry-price stop is gross-flat, not net breakeven.

## §3. Cohort and instrument results

| Group | Model | Closes | Wins | Mean net R | PF(R) |
| --- | --- | ---: | ---: | ---: | ---: |
| 2024_Q4_Election_Vol | s3_two_stage | 0 | 0 | unavailable | unavailable |
| 2024_Q4_Election_Vol | s3_two_stage_window_close | 0 | 0 | unavailable | unavailable |
| 2025_Q2_Tariff_Vol | s3_two_stage | 2 | 0 | -1.237774 | 0.000000 |
| 2025_Q2_Tariff_Vol | s3_two_stage_window_close | 3 | 0 | -1.064041 | 0.000000 |
| SpotCrude | s3_two_stage | 1 | 0 | -0.793407 | 0.000000 |
| SpotCrude | s3_two_stage_window_close | 1 | 0 | -0.793407 | 0.000000 |
| SpotBrent | s3_two_stage | 0 | 0 | unavailable | unavailable |
| SpotBrent | s3_two_stage_window_close | 0 | 0 | unavailable | unavailable |
| US500 | s3_two_stage | 1 | 0 | -1.682141 | 0.000000 |
| US500 | s3_two_stage_window_close | 2 | 0 | -1.199358 | 0.000000 |
| XAUUSD | s3_two_stage | 0 | 0 | unavailable | unavailable |
| XAUUSD | s3_two_stage_window_close | 0 | 0 | unavailable | unavailable |

## §4. Cumulative accounting and evidence limits

The specifically requested №18,331 five-close baseline plus the new **legacy** results gives 3/7 wins, mean -0.265185R. It is shown for comparison, not substituted for the complete history. The full cumulative ledger also includes №18,338 and №18,342, deduplicates repeated identities, preserves prior losses and source-protocol flags, and retains censoring. Three of the earlier five baseline closes carry inherited source-protocol flags.

The JSON also supplies the requested five-prior-close bridge to the new four-hour variant, labelled **mixed exit rules**. It is descriptive only and cannot qualify a homogeneous strategy. New-variant latest-20 metrics are separately reported; no previously censored trade is silently turned into a close.

These two macro-event weeks were selected retrospectively for volatility. Some M30 values were already present in prior preparation caches. Unspent tick reads therefore do not make this a representative or untouched validation sample. Confidence and transfer to accounts remain unproven even if point estimates exceed a target.

## §5. Collection, validation and next decision

Measured collection: `{"m30_requests": 55, "new_component_statuses": {"complete": 95}, "reused_m30_slices": 9, "source_bytes": 102437362, "tick_jobs_attempted": 40, "tick_jobs_complete": 40}`. Per-slot source status, raw/native hashes, quality counters, censoring and receipts are in the JSON. Raw prices remain outside Git and PostgreSQL.

Validation: interval reuse/failure guards, causal bar assembly, native boundary-close/bracket tests, exact unchanged-model parity on two prior sources, and 30 independent net-cost checks. The 19 August–5 September 2025 holdout remains sealed, and all requested intervals precede 24 October 2025. No live orders, live strategy changes, paid model calls or automatic Stage 2 expansion occur.

Next: assess the standalone net outcomes against both controls and across the two cohorts. If entry counts remain low, report the native veto and missing-input reasons instead of weakening the gate to fill a quota. If performance differs under the four-hour deadline, treat that as an exit-policy hypothesis requiring a separately frozen, genuinely untouched evaluation. Calibrate actual execution costs before any deployment decision.

[Frozen plan](../docs/high_vol_cohorts_plan_18348.json) · [Measured JSON](../docs/high_vol_cohorts_results_18348.json)

## §6. Post-run assessment: what this batch establishes

**This high-volatility selection did not establish an edge.** All 40 source jobs and all preparation components completed, but the election cohort admitted no S3 trade. Across the full batch, 36 of 40 episodes recorded a strict cost-gate veto; those episodes had no S3 entry. The remaining non-entry was XAUUSD on 8 November 2024. Selecting a volatile week does not guarantee that the model's causal risk distance exceeds its full cost reserve at the decision mark.

| Trade | Direction | Gross R | Assumed cost R | Net R | Exit |
| --- | --- | ---: | ---: | ---: | --- |
| SpotCrude-2025-04-11 | short | -0.000000 | 0.793407 | -0.793407 | stop |
| US500-2025-04-08 | short | -1.000000 | 0.682141 | -1.682141 | stop |
| US500-2025-04-11 | long | -0.315542 | 0.401034 | -0.716575 | session_end |

The oil short reached the +1R amendment trigger and subsequently exited at entry price: gross-flat but −0.793407R after assumed costs. The 8 April US500 short hit its original stop. The 11 April US500 long was still open at the observation boundary in the legacy model; the explicit window-close variant exited it at 17:30:00.201 UTC for a gross loss. These are measured path outcomes, not inferred broker fills.

For the three window-close trades, mean gross return was -0.438514R and mean assumed cost was 0.625527R. **All three gross returns were non-positive.** Reducing nonnegative costs on these identical fills cannot turn them into winners. A cheaper-cost admission policy could choose different trades, but that was not this experiment and must not be inferred from it.

Always-long and own-momentum controls also had negative mean net returns at 24 bp/side. Their different admissions and oil exposure windows prevent a clean causal alpha comparison; the results provide no justification for deployment. The sample is too small and event-selected to prove universal strategy failure.

The next investigation should distinguish the model's opening/mark risk distance and durable cost veto from whole-week volatility, and separate actual round-trip execution costs from the stress reserve. Any altered admission or cost-aware exit rule needs its own frozen hypothesis. Do not collect additional dates merely to reach 20 closes or erase these losses. No source, model, fill, score or cumulative outcome was changed by this assessment.
