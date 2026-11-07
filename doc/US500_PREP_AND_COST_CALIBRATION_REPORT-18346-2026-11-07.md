# US500 preparation audit and matched-fill cost ledgers

№18,346 §§1–10 · 07-11-2026 22:36:01 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**No US500 episodes were restored. The proposed costs are not empirically calibrated.** The submitted script wrote predetermined profits without changing a validator or running a backtest. This corrected offline run uses retained evidence and actual native fills. It makes no financial PASS or live change.

| Evaluation | Ledger | Closes | Wins | Mean net R | PF(R) |
| --- | --- | ---: | ---: | ---: | ---: |
| Cumulative S3 | A: 24 bp/side | 6 | 3 | -0.188924 | 0.637193 |
| Cumulative S3 | B: proposed rates, unverified | 6 | 3 | 0.417874 | 2.927198 |
| January macro | A: 24 bp/side | 6 | 2 | -0.288994 | 0.167304 |
| January macro | B: proposed rates, unverified | 6 | 4 | 0.061330 | 1.375197 |

**Goal progress: 6/20 unique closed research trades**, with 4 censored entries. One repeated January winner is deduplicated; previous losses remain. The original **73% net wins / reference risk:reward 1:1.38 over the latest 20 trades for each strategy on each account** is unproven. Pooled research results and the interim 50% target do not qualify an account.

## §2. What actually failed

The 30 retained US500 slots contain 1 failed primary, 4 complete tick files with zero M30 bars, and 25 slots with all six positive-volume opening M30 bars. Eight January M30 components retain `bar_identity_or_truncation_unverified`; 24 January retains `interrupted_reserved_job_no_retry` in its original receipt.

The original collector sent `count=500` together with interval bounds. The documented diagnostic returned 500 rows with `hasMore=true`, versus 326 with `hasMore=false` in interval mode. PR #179 (`67d21e6`) already removed count mode. That diagnostic used BTC components: the discarded US500 responses do not reveal which identity/truncation predicate failed. The recorded failure is established; the exact US500 broker-response field remains **Not Verifiable**.

No validator requiring 2,160 continuous M30 bars was found. Current code validates each returned bar and requires specific opening/prior-session bars; weekend gaps between unrelated bars are allowed. US500 cash-open alignment already uses 09:30 America/New_York (14:30 UTC in January, 13:30 UTC in July). Do not encode a fixed UTC maintenance schedule across DST without dated broker evidence.

A blanket minimum of 1,400 bars cannot establish session completeness. It would also reject these nonempty sources: US500-2023-01-23, US500-2025-02-10, US500-2025-02-11. A large unrelated bar count cannot replace missing opening bars.

The implemented change splits the ambiguous decoder error into period, symbol, truncation and missing-completeness errors while preserving its acceptance rules. Existing failed files remain immutable. No bars are bridged, imputed, refetched or manufactured.

## §3. Cost evidence and assumptions

The retained November 2026 audit has 13 measured demo entries, zero attributed live entries and **zero verified round-trip cost samples**. These are partial entry diagnostics, not sufficient execution, fee and swap calibration for historical 2023–2025 trading.

| Symbol | Proposed bp/side | Measured entries | Entry slippage range (bp) | Calibrated? |
| --- | ---: | ---: | --- | --- |
| US500 | 2.0 | 0 | unavailable to unavailable | No |
| NAS100 | 2.0 | 0 | unavailable to unavailable | No |
| XAUUSD | 2.0 | 0 | unavailable to unavailable | No |
| XAGUSD | 2.0 | 0 | unavailable to unavailable | No |
| SpotCrude | 3.74 | 3 | -0.629987 to 2.474482 | No |
| SpotBrent | 3.74 | 2 | -1.377575 to 0.914905 | No |
| BTCUSD | 2.5 | 2 | -0.662320 to 0.018041 | No |
| ETHUSD | 2.5 | 1 | 4.435982 to 4.435982 | No |

Ledger A retains 24 bp per side. Ledger B uses the supplied 2 bp indices/metals, 3.74 bp oils and 2.5 bp crypto as **unverified sensitivity assumptions**. Both retain exactly the same entries, original risk, fills, SL/TP/two-stage amendments, deadlines and censoring. Admission remains frozen at 24 bp; lower rates do not admit new trades. No new native strategy replay is claimed.

`net R = signed(exit VWAP − entry fill) / original risk − (entry fill + exit VWAP) × bp/10,000 / original risk`. Executable bid/ask already incorporates spread; it is not charged again. Additional commission is zero in this sensitivity because the inherited experiment assumed zero, not because actual commission/swap was verified as zero. PF is calculated from equal-R trade results, not dollar account returns.

The supplied “0.60 bp live spread”, “80× hurdle”, restored 14 episodes, 8/12 and 18/24 win claims are unsupported by this run. Reducing assumed cost can change net arithmetic; it cannot establish a measured edge or restore missing observations.

## §4. Validation and next action

Three focused tests cover session-gap/decoder behaviour, irrelevant bar counts versus exact opening slots, and matched-fill costs/censoring/deduplication. All primary ledger values are independently checked against native results. Frozen input hashes are rechecked after analysis. The validation holdout (19 August–5 September 2025) stays sealed. Zero broker/market-data requests, database queries, paid model calls or trading changes occur.

Next: specify a separate bounded repair of the eight failed US500 M30 intervals and the single failed tick interval if complete January evaluation is required. Preserve the old failures and mark replacement provenance; do not call repaired, previously observed data a fresh holdout. To calibrate cost, obtain paired decision/dispatch/fill observations for both entry and exit, actual settled fees/swaps and contract/conversion terms, separated by account, symbol, session and order size. Freeze any resulting cost and admission experiment before evaluating untouched dates. Do not lower costs merely to reach a win target.

[Frozen plan](../docs/us500-prep-cost-plan-18346.json) · [Measured JSON](../docs/us500_prep_and_cost_calibration_18346.json) · [Prior cost evidence](../docs/oil-cost-and-replication-2026-11-06/cost-assessment.json)
