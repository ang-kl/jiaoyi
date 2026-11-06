# Independent Quantitative & Software Review of Codex Assessment `№18,270` (`docs/ai-review-2026-11-06`)

№18,274 §§1–8 · 06-11-2026 10:47:00 SGT (+08:00)
Prepared by: Independent Reviewer (Gemini 3.1 Pro)
Reviewed Corpus & Code Revision: `ang-kl/jiaoyi` `main` @ `64739b45a65b285595e60ae679354cc015f499fd`, PR #164 (`7ae2e64c4e88deb2ad3d802a0a97194a8b502d5f`), and `docs/ai-review-2026-11-06` (PR #165, `№18,273`).

## §1. TLDR & Overall Verdict

§1.1. **Verdict on Codex `№18,270` (`Response_18270.md` & `Reconsidered_Review.md`)**: **Agree (with two technical refinements on §§4.1 and 4.3)**.

§1.2. **Executive Summary**:
- **Exact Reproduction Confirmed from Raw Ticks**: I executed the verbatim `ops/research/cost_context.py` (`64739b45`) and `docs/ai-review-2026-11-06/scripts/reproduce_review.py` logic against the 480 baseline rows in `data/cost-context-results.json` and the 20 raw 5 November 2026 symbol job files (`1,281,012` bid+ask ticks). All 480 development rows and all 240 fresh 5 November rows (`222 observed`, `18 missing_predictor`) in `reviews/fresh-reconsideration.json` reproduced **to the exact floating-point digit**.
- **Where Codex (`№18,268–18,270`) Corrected Our Earlier Handoff Rightly**:
  1. **Cost Double-Counting**: Subtracting `PlanBasis.cost` (`b.cost`) from realized fills in `crates/jiaoyi/src/strategy3.rs` (`close_result_net`) **double-counts spread and planned 24 bps/side slippage**, because executable entry/exit fills already cross the bid-ask spread and incur actual slippage. Codex's rejection of `close_result_net(..., b.cost)` is 100% correct.
  2. **Provenance Hashes & `native_volume`**: The original 40-episode collection receipt (`8c8cb376...`) is distinct from the cost-context analysis plan (`300eb775...`), and `native_volume` for indices must not be silently changed from `10`/`100` to `100`/`1000`.
  3. **Zero-Spread Quotes**: `ops/research/cost_context.py` line 41 (`sides[0][1] > sides[1][1]`) **already accepts `bid == ask`** (`EURUSD` decision quote `(1793865600083, 1.12423, 1.12423)`). The strict `bid < ask` bug existed only in our first-pass ad-hoc script, not in the repository's `cost_context.py`.
  4. **Validation Schedule Math**: Three validation dates (`2025-08-19`, `2025-08-26`, `2025-09-02`) yield at most 3 episodes per symbol, which cannot mathematically satisfy `validation_min_closed_per_symbol = 5` or `validation_min_closed_per_class = 20` for 2–3 symbol classes.
- **Two Refinements to Codex `№18,270`**:
  1. **Root Cause of the 18 `missing_predictor` Rows (`Reconsidered_Review.md` `§2`)**: Codex attributed the 18 `missing_predictor` rows on 5 November to *"30/60-minute predictions for jobs started late on 5 November"*. In fact, the 18 rows are `2 peer variants × 3 horizons (30m, 60m, 180m) = 6 rows each` for **`NatGas`** (which has no peer in `cost_context.py:peers()`), **`AUS200`** (`start = 2026-11-04T23:00:00Z`), and **`JPN225`** (`start = 2026-11-05T00:00:00Z`), because `cost_context.py:factor()` line 63 looks up `corpus.get((peer, start))` with an exact `start` match and Sydney AEDT (`UTC+11`) shifts `AUS200` one hour earlier than Tokyo (`00:00 UTC`).
  2. **Matched-Control Equivalence in Oil (`§4.3`)**: While `oil_peer_180` won `2/2` on 5 November (`+82.55 bps` gross, `+34.35 bps` at 24bp/side stress) and `5/5` pooled (`+65.17 bps` gross), `matched_controls` proves that **`oil_peer_180` and `oil_own_180` had `changed_direction = 0` (`mean_stress_delta_bps = 0.0`) across all 4 matched episodes** (`2` dev + `2` fresh). Thus, the Oil 180m edge is a **180-minute first-hour continuation effect** shared equally by `own_momentum` (`5/5`, `+65.66 bps` gross) and `peer_continuation` (`5/5`, `+65.17 bps` gross). By contrast, in **Metals (`30m` & `180m`)**, `peer_reversal` genuinely outperforms both `own_momentum` (`+17.46 bps/trade` at 30m, `+31.29 bps/trade` at 180m) and `always_long` (`+10.29 bps/trade` at 30m, `+20.19 bps/trade` at 180m).

## §2. Claim-by-Claim Audit Table

| # | Claim Under Review | Source / Reference | Independent Verification Result | Verdict | Severity | Necessary Correction / Action |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | All 480 dev rows and 240 fresh 5 Nov rows (`1,281,012` ticks) reproduce with usable quotes at decision, entry, and 30/60/180m exits for all 20 symbols. | `Response_18270.md` `§2`; `Reconsidered_Review.md` `§2`; `fresh-reconsideration.json` | Verified directly from the 20 raw JSON files in `Jiaoyi_Fresh_Market_Data_2026-11-05.zip` + `data/cost-context-results.json` using `cost_context.py` (`64739b45`). `ticks = 1,281,012`; `observed = 222`, `missing_predictor = 18`, `0` quote gaps. | **Agree** | High | None to the calculation; retain `reproduce_review.py` as the canonical diagnostic check. |
| **2** | The 18 `missing_predictor` rows on 5 Nov are *"30/60-minute predictions for jobs started late on 5 November"*. | `Reconsidered_Review.md` `§2` (line 21) | Disproved by inspecting `fresh_rows` status by symbol: the 18 rows are `NatGas` (`6`), `AUS200` (`6`), and `JPN225` (`6`) across all 3 horizons (`30m`, `60m`, `180m`) on `peer_continuation` and `peer_reversal`. | **Disagree (minor prose error)** | Low | Correct explanation: `6` rows are `NatGas` (no peer in `peers()`), and `12` rows are `AUS200`/`JPN225` (`start` differs by 1h under Sydney AEDT `UTC+11` in `corpus.get((peer, start))`). |
| **3** | Subtracting `PlanBasis.cost` (`b.cost`) from realized fills in `strategy3.rs` (`close_result_net`) double-counts spread and slippage. | `Handoff_Review.md` `§3.1`; `Response_18269.md` `§2` | Confirmed in `crates/jiaoyi/src/strategy3.rs` (`lines 503–527` & `2968–2980`). `b.cost` includes planned spread + `2×24bp` slippage reserve, whereas `fill` and `entry` already include realized spread and slippage. | **Agree** | **Critical** | Drop `close_result_net(..., b.cost)` from `strategy3.rs`. Only deduct explicit non-price deal fees (`commission + swap`) if available, or leave `close_result` untouched. |
| **4** | Oil continuation at `180m` (`SpotBrent`, `SpotCrude`) won `2/2` on 5 Nov (`+82.55 bps` gross, `+78.54 bps` @ 2bp, `+34.35 bps` @ 24bp) and `5/5` pooled (`+65.17 bps` gross). | `Response_18270.md` `§2`; `fresh-reconsideration.json` (`oil_peer_180`) | Confirmed (`SpotBrent +97.69 bps`, `SpotCrude +67.42 bps` gross on 5 Nov; `5/5` pooled). However, `matched_controls` shows `oil_peer_180` and `oil_own_180` (`5/5`, `+65.66 bps` gross) had `changed_direction = 0/4` on matched episodes. | **Partly Agree** | Medium | Test both `oil_own_180` and `oil_peer_180` (with `own_momentum` fallback when peer is flat) in the offline candidate study rather than attributing the edge solely to cross-oil peer lead. |
| **5** | Metals `peer_reversal` at `30m` won `2/2` on 5 Nov (`+14.39 bps` gross, `+10.38 bps` @ 2bp) and `4/4` on pooled unanimous-peer episodes (`+13.31 bps` @ 30m, `+27.32 bps` @ 180m). | `Response_18270.md` `§2`; `fresh-reconsideration.json` (`metal_reversal_30`, `metal_unanimous_reversal_30/180`) | Confirmed (`XAGUSD +24.14 bps`, `XAUUSD +4.63 bps` @ 30m; `+39.73 bps` and `+15.79 bps` @ 180m on 5 Nov). Beats `metal_own_30` by `+17.46 bps/trade` (30m) and `+31.29 bps/trade` (180m) and beats `always_long` by `+10.29 bps/trade` (30m). | **Agree** | High | Strongest true cross-market predictor in the dataset ($N=4$ unanimous episodes is still small, so treat as a top offline research candidate, not a proven live edge). |
| **6** | Equity Index `own_momentum` at `180m` won `7/7` gross (`+80.00 bps`) and `6/7` at 2bp (`+76.02 bps`) on 5 Nov, and `13/18` (`+33.57 bps` gross) pooled. | `Response_18270.md` `§2`; `fresh-reconsideration.json` (`indices_own_180`) | Confirmed (`US30 +155.14`, `US500 +142.22`, `JPN225 +123.23`, `NAS100 +109.23`, `AUS200 +21.12`, `GER40 +6.68`, `UK100 +2.38 bps` on 5 Nov). Beats `always_long` (`2/7`, `-73.48 bps`) by `+153.48 bps/trade` on 5 Nov and beats `peer_continuation` (`-3.56 bps` delta). | **Agree** | High | Include `indices_own_30` (`14/19` gross, `+19.98 bps` pooled) and `indices_own_180` (`13/18` gross, `+33.57 bps` pooled) in offline candidate replay; note `UK100` (`-7.69 bps` pooled at 180m) remains a weak constituent. |
| **7** | Crypto `peer_continuation` at `180m` won `1/2` gross (`+0.60 bps`) and `0/2` at 2bp (`-3.40 bps`) on 5 Nov; FX `peer_reversal` at `180m` won `0/6` (`-9.43 bps` gross) on 5 Nov. | `Response_18270.md` `§2`; `fresh-reconsideration.json` | Confirmed (`BTCUSD +1.24 bps` gross / `-2.76 bps` @ 2bp; `ETHUSD -0.04 bps` gross / `-4.04 bps` @ 2bp; FX `0/6` on `fx_reversal_180` and `fx_unanimous_reversal_180`). | **Agree** | High | Keep `fx` rejected/diagnostic-only; do not claim out-of-sample confirmation for `crypto` on 5 November. |
| **8** | Existing `cost_context.py` already handles `bid == ask` zero-spread quotes (`EURUSD`). | `Response_18269.md` `§3`; `ops/research/cost_context.py` line 41 | Confirmed: line 41 checks `sides[0][1] > sides[1][1]`, which permits `bid == ask`. `EURUSD` decision quote `(1793865600083, 1.12423, 1.12423)` was properly observed. | **Agree** | Low | Remove the zero-spread patch request from the agent prompt; `cost_context.py` already handles it correctly. |

## §3. Material Calculation & Methodology Discrepancies Explained

§3.1. **Why My Turn-4 Script Reported `228 observed / 12 missing_predictor` While `cost_context.py` Reports `222 observed / 18 missing_predictor` on 5 November**:
- In `ops/research/cost_context.py` (`line 63`), `factor()` looks up a peer's quote stream using the **target symbol's** session start timestamp:
  ```python
  qs = corpus.get((peer, start))
  if qs is None:
      continue  # Do not use a different market's later opening hour.
  ```
- In August 2025 (Australian winter AEST `UTC+10`), `AUS200` (`10:00 Sydney` = `00:00 UTC`) and `JPN225` (`09:00 Tokyo` = `00:00 UTC`) shared the exact same `start` (`00:00 UTC`), so `corpus.get((peer, start))` succeeded for both.
- On `2026-11-05` (Australian summer AEDT `UTC+11`), `AUS200` opened at `2026-11-04T23:00:00Z` while `JPN225` opened at `2026-11-05T00:00:00Z`. Because their `start` keys differ, `corpus.get((peer, start))` returns `None` for **both** `AUS200` (`6` peer rows) and `JPN225` (`6` peer rows), plus `NatGas` (`6` peer rows because `peers("NatGas", "energy", ...)` returns `{}`), totaling `6 + 6 + 6 = 18` `missing_predictor` rows and `240 - 18 = 222` observed rows.

§3.2. **Entry-Mid vs. Entry-Fill Normalization (`~0.01–0.09 bps` Difference)**:
- `ops/research/cost_context.py` (`lines 72–74`) normalizes both gross return and slippage drag by `mid(entry)`:
  $$\text{gross}_{\text{bps}} = \text{side} \times \frac{b - a}{\text{mid}(\text{entry})} \times 10^4, \qquad \text{drag}_{\text{bps}} = \frac{a + b}{\text{mid}(\text{entry})} \times \text{per\_side}_{\text{bps}}$$
- For `oil_peer_180` on 5 November, `cost_context.py` yields `+82.55 bps` gross (`0bp`), `+78.54 bps` at `2bp/side`, and `+34.35 bps` at `24bp/side` (matching `Response_18270.md` and `fresh-reconsideration.json` verbatim), whereas dividing by entry fill price (`a`) yielded `+82.54 bps` gross and `+34.26 bps` at `24bp/side`. `cost_context.py`'s `mid(entry)` normalization is cleaner because it is symmetric between long and short entries at the same quote.

§3.3. **Matched Controls (`matched_controls_fresh` and Pooled) — Where Peer Signals Add Value vs. Where They Are Redundant**:

| Asset Class & Horizon | Peer Variant Tested | Matched Episodes (`Dev + Fresh = Pooled`) | Direction Changes vs. `own_momentum` | Mean Stress Delta vs. `own_momentum` (`bps`) | Direction Changes vs. `always_long` | Mean Stress Delta vs. `always_long` (`bps`) | Conclusion on Peer Mechanism |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Energy (`180m`)** | `peer_continuation` | `2 + 2 = 4` (vs `own`) / `3 + 2 = 5` (vs `long`) | **`0 / 4` (`0%`)** | **`0.00`** | `3 / 5` (`60%`) | **`+68.15`** | **Redundant with `own_momentum`**: `SpotBrent` and `SpotCrude` moved together in Hour 1 on all 4 matched episodes; edge is 180m oil momentum, not cross-oil lead-lag. |
| **Metals (`30m`)** | `peer_reversal` | `4 + 2 = 6` | **`3 / 6` (`50%`)** | **`+17.46`** | `2 / 6` (`33%`) | **`+10.29`** | **Genuine cross-market value**: USD peer-basket reversal adds `+17.46 bps/trade` over metal's own Hour-1 move at `30m`. |
| **Metals (`180m`)** | `peer_reversal` | `4 + 2 = 6` | **`3 / 6` (`50%`)** | **`+31.29`** | `2 / 6` (`33%`) | **`+20.19`** | **Genuine cross-market value**: USD peer-basket reversal adds `+31.29 bps/trade` over `own_momentum` at `180m` (`4/4` when USD peers are unanimous). |
| **Equity Index (`30m`)** | `peer_continuation` | `12 + 5 = 17` | `5 / 17` (`29%`) | `-0.11` | `10 / 17` (`59%`) | `+38.10` | **No gain over `own_momentum`**: `own_momentum` is slightly better (`+0.11 bps`) and avoids DST peer gaps on `AUS200`/`JPN225`. |
| **Equity Index (`180m`)** | `peer_continuation` | `11 + 5 = 16` | `5 / 16` (`31%`) | `-3.56` | `10 / 16` (`63%`) | `+43.11` | **Inferior to `own_momentum`**: `own_momentum` beats `peer_continuation` by `+3.56 bps/trade` on matched episodes. |
| **Crypto (`180m`)** | `peer_continuation` | `4 + 2 = 6` | `2 / 6` (`33%`) | `+3.78` | `3 / 6` (`50%`) | `+61.32` | **Inconclusive**: Identical to `own_momentum` in Aug (`0/4` changes); avoided `own_momentum` loss on 5 Nov (`+11.33 bps` delta) but still lost `-3.40 bps` at `2bp/side`. |
| **FX (`180m`)** | `peer_reversal` | `12 + 6 = 18` | `16 / 18` (`89%`) | `+1.48` | `8 / 18` (`44%`) | `+0.82` | **No edge**: Both `peer_reversal` (`-4.41 bps` @ 2bp) and `peer_continuation` (`-3.91 bps` @ 2bp) lose money after minimal costs. |

## §4. Evaluation of Both AI Systems (What Each Got Right and Wrong)

### §4.1. What Codex Got Right (`№18,268–18,270`)
- **Caught the `PlanBasis.cost` Double-Counting Bug (`Handoff_Review.md` `§3.1`)**: Correctly identified that subtracting `b.cost` from realized fill prices in `strategy3.rs` double-counts spread and planned slippage.
- **Self-Corrected Its Initial Anchoring (`Reconsidered_Review.md` `§§1–4`)**: In `№18,268` and `№18,269`, Codex initially dismissed the research direction without running the 5 November dataset. In `№18,270`, Codex ran the full 1,281,012-tick reproduction (`reproduce_review.py`), confirmed the 5 November results for Oil (`2/2`), Metals (`2/2`), and Indices (`7/7`), and appropriately pivoted from blocking research code to recommending bounded offline implementation.
- **Preserved Strict Provenance & Schedule Integrity**: Correctly separated the 3 SHA-256 hashes (`8c8cb376...` collection receipt vs. `300eb775...` cost-context plan vs. `0b0fa0fa...` new plan), flagged the unintended `native_volume` edits, and pointed out that 3 validation dates cannot yield 5–20 trades per symbol/class.

### §4.2. What Codex Got Wrong or Overlooked (`№18,268–18,270`)
- **Initial Over-Gatekeeping in `№18,268–18,269`**: Demanded multi-week live-deployment proof before allowing offline research code to be written, and initially criticized `FX` as a false PASS in `№18,269` even after our `№18,268` update had already falsified and rejected `FX`.
- **Misstated the Cause of the 18 `missing_predictor` Rows in `Reconsidered_Review.md` `§2`**: Described them as *"30/60-minute predictions for jobs started late on 5 November"* rather than `NatGas` (`6` rows, no peer) + `AUS200`/`JPN225` (`12` rows, AEDT `UTC+11` 1-hour `start` offset across all 3 horizons).
- **Did Not Highlight That `oil_peer_180` and `oil_own_180` Are Identical on Matched Episodes**: Presented `Oil continuation, 180 minutes` (`oil_peer_180`) in `Response_18270.md` without noting that `changed_direction == 0` against `own_momentum` on all 4 matched episodes.

### §4.3. What Our Earlier Proposal (`№18,265–18,268`) Got Right and Wrong
- **Got Right**: Identified the horizon-scaling effect (`180m` vs `30m` on Oil and Equity Indices), the USD peer-unanimity filter on Metals (`4/4` across the 3 dates), the `5 November` falsification of `FX` reversal (`0/6`), the `DATA_GAP_REASONS` check in `class_research.py`/`milestone.py`, and the `keys.len() < MIN_CLOSES` (`20`) SQL short-circuit in `strategy3.rs`.
- **Got Wrong**:
  1. Proposed subtracting `b.cost` in `close_result_net()` in `strategy3.rs` (double-counting spread and planned slippage).
  2. Used `300eb775...` (the cost-context plan SHA-256) as `source_plan_sha256` instead of `8c8cb376...` (the `plan-sha256.txt` receipt in the `class_research.py` development directory), and changed index `native_volume` values in `class-prediction-plan-2026-11-06.json`.
  3. Claimed a zero-spread (`bid < ask`) bug in repo code when that bug existed only in our own temporary Turn-4 script (`cost_context.py` already used `sides[0][1] > sides[1][1]`), and used overly strong wording (*"confirmed out-of-sample edge"*) for a single additional market date (`2026-11-05`).

## §5. Prioritised Next-Step Plan & Bottom-Line Decision

§5.1. **Bottom-Line Decision**: **Proceed with a small, clean offline research + non-behavioral engine PR (`Phase A` below)**, drop the erroneous `close_result_net(..., b.cost)` change, keep `FX` and `NatGas` (`07:00 UTC`) excluded from candidate promotion, and do **not** alter live Strategy 3 rules until a multi-week validation corpus ($\ge 5$ closes per symbol, $\ge 20$ per class across $\ge 3$ weeks) is evaluated under full SL/TP/trailing replay.

### Phase A — Smallest Useful Implementation (Immediate Offline PR, Zero Live Risk)

1. **`crates/jiaoyi/src/strategy3.rs` (1 Safe Optimization + 1 Pure Helper)**:
   - **Keep**: The Kelly SQL short-circuit in `Trader::entry()` (`if keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES { Vec::new() } else { ... }`) so the loop does not query `ts::closed_trades` on every 30-minute mark while `keys.len() < 20`.
   - **Keep**: Pure helper `pub fn peer_coherence(peer_returns_bps: &[f64]) -> Option<(f64, f64, bool)>` + unit test for offline study use.
   - **Drop**: Do **not** change `close_result()` or subtract `b.cost` in `broker_exit()`.
2. **`ops/research/class_research.py` & `ops/research/milestone.py` (Data-Gap & Reuse Hygiene)**:
   - **Keep**: `DATA_GAP_REASONS` check so missing quotes (`missing_next_fresh_entry`, `missing_decision_quote`, `missing_close_quote`) explicitly block `passes_screen` instead of silently shrinking `closed`.
   - **Fix Provenance Hash**: In `verify_receipt()`, validate `source_plan_sha256` against `8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaae648e642cb3a2d20` (the actual `class_research.py` development collection receipt) and preserve the original `native_volume` map (`AUS200: 10, GER40: 10, NAS100: 10, UK100: 10, US30: 10, JPN225: 100`).
3. **Focused Offline Candidate Replay (`s3_candidate_study` / `ops/research/`)**:
   - Test only the **three empirically supported hypotheses** under full Strategy 3 exit geometry (`1.38R` target / Chandelier trail / `180m` or `30m` horizon end) rather than fixed-horizon mid/quote diagnostics alone:
     - **Candidate 1 (`energy` Oil pair — `SpotBrent`, `SpotCrude`)**: `own_momentum` (and `peer_continuation` with `own_momentum` fallback) with `180m` episode close.
     - **Candidate 2 (`metal` — `XAUUSD`, `XAGUSD`)**: `peer_reversal` gated by unanimous USD-peer sign (`peer_coherence == 1.0`) at `30m` and `180m`.
     - **Candidate 3 (`equity_index`)**: `own_momentum` at `30m` and `180m`.

### Phase B — Consequential Acceptance Tests (5 Tests Required for the PR)

1. **`test_kelly_short_circuit_under_min_closes` (Rust)**: Verifies that when `open_keys_for_hash` returns `< 20` keys, `kelly::read(&[])` is called with `n = 0` and `skip` is `false` without querying `ts::closed_trades`.
2. **`test_peer_coherence_unanimity` (Rust)**: Verifies `peer_coherence` returns `Some((1.0, mean, true))` when all peers share the same non-zero sign, `Some((ratio, mean, false))` when signs are mixed, and `None` on empty/zero/NaN inputs.
3. **`test_score_fails_on_data_gap_when_trade_is_none` (Python)**: Verifies `class_research.score()` and `milestone.score()` return `passes_screen == False` with `'data coverage gaps (...)'` when an episode has `trade: None` due to `missing_next_fresh_entry` or `missing_close_quote`, while allowing legitimate signal skips (`peer_disagreement`, `flat_predictor`).
4. **`test_reuse_only_preserves_collection_receipt` (Python)**: Verifies that running `class_research.py analyse` with `reuse_only: true` checks `plan-sha256.txt == 8c8cb376...` and writes `analysis-plan-sha256.txt` without overwriting `plan-sha256.txt`.
5. **`test_reproduce_review_exact_match` (Python)**: Runs `docs/ai-review-2026-11-06/scripts/reproduce_review.py` and asserts `development_rows_reproduced_exactly == 480` and `fresh_statuses == {'observed': 222, 'missing_predictor': 18}`.

## §6. Open Uncertainties & Unexecuted Checks

§6.1. **Explicitly Verified in This Review**:
- Executed `ops/research/cost_context.py` (`64739b45`) and `scripts/reproduce_review.py` logic in Python against `data/cost-context-results.json` (`480` rows) and the 20 raw symbol files in `Jiaoyi_Fresh_Market_Data_2026-11-05.zip` (`1,281,012` ticks).

§6.2. **Unavailable / Unexecuted in This Review Environment (Marked Explicitly)**:
- **Raw 40 August 2025 Job Files**: While `data/cost-context-results.json` (`480` derived rows) was verified in our sandbox, the 40 raw August 2025 tick job JSON files (`AUDUSD-2025-08-05.json`, etc.) were not uploaded to this sandbox; their exact reproduction (`development_rows_reproduced_exactly = 480`) relies on Codex's local receipt (`reproduction-check.json`, `№18,273 §2`).
- **`cargo test --workspace` & `cargo clippy`**: The full Rust workspace (`crates/jiaoyi-core`, `crates/jiaoyi-store`, `crates/jiaoyi-ctrader`) is not checked out in this sandbox VM, so Rust compilation and workspace-wide `cargo test` were not executed here and must be verified by CI on the repository.
- **Full Strategy Replay vs. Fixed-Horizon Diagnostic**: Both `cost-context-results.json` and `fresh-reconsideration.json` measure **fixed-horizon (`30m`, `60m`, `180m`) quote returns without initial stop `R`, `1.38R` take-profit (`TP1`/`TP2`), or Chandelier trailing stops**, and without actual broker commission/swap fees. A candidate that wins on fixed 180-minute horizon quotes can still stop out intrabar if its initial stop $R$ is too tight.
