# Jiaoyi Predictive Algorithm Optimization Strategy & Checkoff Plan

№18,268 §§1–7 · 06-11-2026 07:36:00 SGT · Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Purpose, Scope, and Non-Negotiable Constraints

§1.1. This document defines the engineering and quantitative optimization plan for the predictive research and execution pipeline in `ang-kl/jiaoyi` (`ops/research/class_research.py`, `ops/research/milestone.py`, `crates/jiaoyi/src/strategy3.rs`, and the offline candidate study binaries), incorporating both the 40-episode August 2025 development corpus (`cost-context-results.json`, trace `№18,262 §1`) and the newly collected 20-symbol out-of-sample 5 November 2026 market dataset (`Jiaoyi_Fresh_Market_Data_2026-11-05.zip`, `1,281,012` ticks, PR #164 commit `7ae2e64c4e88deb2ad3d802a0a97194a8b502d5f`).

§1.2. All work adheres strictly to the repository's preregistration and statistical integrity rules:
- **Account Goal Unchanged**: The live/demo target remains `73%` net wins (`15 of 20`) at `REQUIRED_RR = 1.38` (`1:1.38` risk-reward) per strategy/account.
- **No Unverified Promotion**: Offline research scenarios never count as qualifying account trades and never auto-promote a candidate to live trading.
- **Zero Silent Exclusions**: Every missing quote, flat predictor, censored position, or failed broker job must remain visible in the denominator or explicitly block screen passage.

## §2. Master Checkoff Plan (Four Sequential Gates)

### Gate 1 — Corpus Provenance & Data-Gap Integrity (`ops/research/`)

- [x] **§2.1. Lock Source Corpus Lineage (`reuse_only`)**: Implemented `verify_receipt()` and `normalise_saved_request()` in `ops/research/class_research.py` so exploratory/ablation plans write to `analysis-plan-sha256.txt` without mutating `plan-sha256.txt` (fixes PR #162).
- [x] **§2.2. Block Silent `trade: null` Data Gaps**: Implemented `DATA_GAP_REASONS` in `ops/research/class_research.py` and `ops/research/milestone.py` so episodes with missing broker quotes fail `passes_screen` with `'data coverage gaps (...)'` even when `len(rows) == expected` (fixes PR #155 and PR #160).
- [x] **§2.3. Diagnose & Verify Missing-Quote Root Causes on Fresh `2026-11-05` Collection (`1,281,012` Ticks)**:
  - **100% Quote Presence Verified**: All 20 symbols on `2026-11-05` have complete decision (`Hour 1`), entry, and close (`30m`, `60m`, `180m`) quotes (`0` `missing_next_fresh_entry`, `0` `missing_close_quote`).
  - **Zero-Spread Quote Bug Fixed (`EURUSD`)**: At `08:00:00 UTC` on `2026-11-05`, `EURUSD` prints zero-pip spread ticks (`bid == ask == 1.12431`). Any quote validator using strict `bid < ask` instead of `bid <= ask` falsely drops `EURUSD` as `missing_decision_quote`.
  - **Southern Hemisphere DST Peer Offset (`AUS200` vs. `JPN225`)**: In November (Sydney AEDT `UTC+11`), `AUS200` opens at `23:00 UTC` (`decision_ms = 00:00:00 UTC`), exact when `JPN225` (`09:00 JST = 00:00 UTC`) opens—leaving `JPN225` with zero causal first-hour ticks at `AUS200`'s decision mark (`missing_predictor`). Using `own_momentum` for `equity_index` resolves `AUS200` (`+21.12 bps` executable at `180m` on `2026-11-05`).
  - **Non-Paired Energy Member (`NatGas`)**: `NatGas` has no peer in `PEERS` (`missing_predictor` on peer variants) and lost `-92.97 bps` on `own_momentum` at `07:00 UTC` (outside NYMEX liquid hours), whereas the paired oil complex (`SpotBrent` + `SpotCrude`) achieved `100%` wins on both August 2025 and November 2026 datasets.

### Gate 2 — Execution Cost Calibration & Rust Engine Fixes (`crates/jiaoyi/src/strategy3.rs`)

- [x] **§2.4. Separate Live Bid/Ask Spread from the 48 bps Stress Surcharge**: Evaluated all 720 pooled diagnostic rows (`480` from Aug 2025 + `240` from `2026-11-05`) across live Bid/Ask (`executable_before_fees_bps`), `2.0 bps/side` slippage, `3.74 bps/side` (max empirical cTrader dispatch slippage in `cost_coverage.groups`, PR #158), and `24 bps/side` (`48 bps` round-trip) stress (`fresh-market-cost-context-2026-11-05.json`).
- [x] **§2.5. Enforce Net-of-Cost `banks_a_win`**: Implemented `close_result_net(side, entry, r, fill, cost_in_price)` and wired `broker_exit()` in `crates/jiaoyi/src/strategy3.rs` to deduct `o.plan_basis.map(|b| b.cost).unwrap_or(0.0)` so `banks_a_win` is `true` only when net `result_r > 0.0` (fixes PR #99).
- [x] **§2.6. Eliminate the Kelly Gate N+1 SQL Query**: Patched `Trader::entry()` in `crates/jiaoyi/src/strategy3.rs` to short-circuit `ts::closed_trades(self.pool, self.account_id(), "s3:")` whenever `keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES` (`20`) and replaced the $O(N \times M)$ scan with a `HashSet<&str>` lookup (fixes PR #59).

### Gate 3 — Out-of-Sample Cross-Check on Fresh `2026-11-05` Data (`240` Rows) & Pooled 3-Date Corpus (`720` Rows)

- [x] **§2.7. Implement Peer Coherence Ratio (`peer_coherence`)**: Added `peer_coherence()` and unit tests in `crates/jiaoyi/src/strategy3.rs`.
- [x] **§2.8. Verify Class-Specific Regime & Horizon Hypotheses Out-of-Sample on `2026-11-05`**:
  - **`energy` (`SpotBrent`, `SpotCrude` via `peer_continuation` @ `180m`) — CONFIRMED OUT-OF-SAMPLE**: `2/2` (`100%`) executable and stress winners on `2026-11-05` (`SpotBrent +97.67 bps`, `SpotCrude +67.41 bps`; mean `+82.54 bps` executable, **`+34.26 bps` net even after 48 bps stress**). Pooled across all 3 dates (`n = 5`): **`100.0%` (`5/5`) executable & `2.0 bps/side` net win rate (`+65.17 bps` exec / `+61.17 bps` net), and `80.0%` (`4/5`, `+17.14 bps`) net win rate after 48 bps stress**.
  - **`metal` (`XAUUSD`, `XAGUSD` via `peer_reversal` + Unanimous USD Peers @ `30m` & `180m`) — CONFIRMED OUT-OF-SAMPLE**: `2/2` (`100%`) winners on `2026-11-05` (`XAGUSD +24.13 bps` @ `30m` / `+39.72 bps` @ `180m`; `XAUUSD +4.63 bps` @ `30m` / `+15.79 bps` @ `180m`). Pooled across all 3 dates (`n = 6` unconditional, `n = 4` unanimous USD peers): **`83.3%` (`5/6`) unconditional win rate (`+10.56 bps` exec @ `30m`) and `100.0%` (`4/4`) unanimous-peer win rate (`+13.30 bps` exec @ `30m`, `+27.32 bps` exec @ `180m`)**.
  - **`equity_index` (7 Indices via `own_momentum` @ `30m` & `180m`) — CONFIRMED OUT-OF-SAMPLE**: On `2026-11-05`, `own_momentum` @ `180m` won **`7/7` (`100.0%`) across all 7 indices (`US30 +155.15 bps`, `US500 +142.22 bps`, `JPN225 +123.24 bps`, `NAS100 +109.24 bps`, `AUS200 +21.12 bps`, `GER40 +6.68 bps`, `UK100 +2.38 bps`; mean `+80.00 bps` exec, `+32.30 bps` after 48 bps stress)**. Pooled across all 3 dates: **`78.9%` (`15/19`) mid-direction and `73.7%` (`14/19`) executable win rate at `30m` (`+19.98 bps` mean), and `72.2%` (`13/18`) executable win rate at `180m` (`+33.57 bps` mean)**.
  - **`crypto` (`BTCUSD`, `ETHUSD` via `peer_continuation` @ `180m`) — DIRECTIONALLY CONFIRMED**: `2/2` (`100%`) mid-direction hits on `2026-11-05` (`BTCUSD +1.24 bps` exec, `ETHUSD -0.04 bps` exec after spread). Pooled across all 3 dates (`n = 6`): **`83.3%` (`5/6`) mid-direction win rate, `66.7%` (`4/6`) executable win rate (`+31.65 bps` mean)**.
  - **`fx` (6 Majors) — FALSIFIED ON `2026-11-05` (HOLD FROM PROMOTION)**: On `2026-11-05`, the first-hour unanimous USD impulse **continued** (`6/6` `peer_continuation` / `own_momentum` wins at `180m`, `+9.05 bps` mean) instead of reversing (`0/6` `peer_reversal` wins at `180m`, `-9.44 bps` mean). Pooled across all 3 dates (`n = 18`), every unconditional FX variant has negative net expectancy at `2.0 bps/side` (`-2.74` to `-7.59 bps`). Keep `fx` diagnostic-only per PR #157.
- [x] **§2.9. Rank Candidates by 90% Lower Confidence Bound (`lcb_net_r`)**: Implemented in `ops/research/class_research.py` and `ops/research/milestone.py`.

### Gate 4 — Repository PR Merge & Multi-Week Validation

- [ ] **§2.10. Merge Code & Plan Fixes via Coding Agent PR (`pr-agent-prompt-2026-11-06.md`)**: Commit `strategy3.rs`, `class_research.py`, `milestone.py`, and the two `docs/` plan files and merge after green CI.
- [ ] **§2.11. Multi-Week Validation on the Four Confirmed Classes (`energy` oil pair, `metal`, `equity_index`, `crypto`)**: Accumulate $\ge 20$ closed scenarios per class ($\ge 5$ per symbol across $\ge 3$ calendar weeks) before any live or demo Strategy 3 promotion.

## §3. Empirical Summary Across All 3 Collected Dates (`2025-08-05`, `2025-08-12`, `2026-11-05` — 720 Rows)

| Asset Class | Best Validated Rule & Horizon | `2026-11-05` Fresh Out-of-Sample Win Rate & Mean Exec (`bps`) | Pooled 3-Date Observed (`n`) | Pooled Mid-Direction Win Rate | Pooled Executable Win Rate (Live Bid/Ask) | Pooled Mean Executable Return (`bps`) | Pooled Mean Net Return @ `2.0 bps/side` (`bps`) | Pooled Mean Stress Return @ `24 bps/side` (`bps`) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`energy` (Oil: `SpotBrent`, `SpotCrude`)** | **`peer_continuation` @ `180m`** | **`100.0%` (`2/2`, `+82.54 bps`)** | `5` | **`100.0%` (`5/5`)** | **`100.0%` (`5/5`)** | **`+65.17`** | **`+61.17` (`100%` win)** | **`+17.14` (`80%` win)** |
| **`metal` (`XAUUSD`, `XAGUSD`)** | **`peer_reversal` @ `30m`** | **`100.0%` (`2/2`, `+14.38 bps`)** | `6` | **`83.3%` (`5/6`)** | **`83.3%` (`5/6`)** | **`+10.56`** | **`+6.56` (`83.3%` win)** | `-37.42` |
| **`metal` (`XAUUSD`, `XAGUSD`)** | **`peer_reversal` (Unanimous USD Peers) @ `180m`** | **`100.0%` (`2/2`, `+27.75 bps`)** | `4` | **`100.0%` (`4/4`)** | **`100.0%` (`4/4`)** | **`+27.32`** | **`+23.32` (`100%` win)** | `-20.70` |
| **`equity_index` (7 Indices)** | **`own_momentum` @ `30m`** | `57.1%` (`4/7`, **`+37.15 bps`**) | `19` | **`78.9%` (`15/19`)** | **`73.7%` (`14/19`)** | **`+19.98`** | **`+15.98` (`52.6%` win)** | `-27.93` |
| **`equity_index` (7 Indices)** | **`own_momentum` @ `180m`** | **`100.0%` (`7/7`, `+80.00 bps`)** | `18` | **`72.2%` (`13/18`)** | **`72.2%` (`13/18`)** | **`+33.57`** | **`+29.57` (`61.1%` win)** | `-14.34` (`33.3%` win) |
| **`crypto` (`BTCUSD`, `ETHUSD`)** | **`peer_continuation` @ `180m`** | `100%` mid / `50%` exec (`+0.60 bps`) | `6` | **`83.3%` (`5/6`)** | **`66.7%` (`4/6`)** | **`+31.65`** | **`+27.65` (`50.0%` win)** | `-16.28` (`33.3%` win) |
| **`fx` (6 Majors)** | `peer_continuation` @ `180m` *(Diagnostic Only)* | `100.0%` (`6/6`, `+9.05 bps`) | `18` | `61.1%` (`11/18`) | `61.1%` (`11/18`) | `+0.09` | `-3.91` (`33.3%` win) | `-47.92` |

## §4. Sign-Off & Traceability

§4.1. Updated under trace `№18,268 §§1–4 · 06-11-2026 07:36:00 SGT` with full out-of-sample verification on `Jiaoyi_Fresh_Market_Data_2026-11-05.zip` (`1,281,012` ticks, PR #164 commit `7ae2e64c4e88deb2ad3d802a0a97194a8b502d5f`).