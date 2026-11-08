# Production Upgrade Directive — Wire `s3_macro_pullback_v1` & 8-Symbol Core into `crates/jiaoyi/src/strategy3.rs` Before Monday Open & Archive Deprecated Peak-Breakout Strategy (№18,377)

№18,377 §§1–6 · 08-11-2026 19:38:00 SGT (+08:00) · Repository: https://github.com/ang-kl/jiaoyi

## §1. Explicit Owner Authorization & Master Directive

The owner has issued an **explicit production upgrade mandate** ahead of the Monday market open (`Option A`):

1. **Wire `s3_macro_pullback_v1` + the 8-Symbol Core Directly into Production (`crates/jiaoyi/src/strategy3.rs` and `crates/jiaoyi-core/`)**:
   - Replace the legacy peak-buying breakout entry and noisy 30-second trailing choke in `crates/jiaoyi/src/strategy3.rs` with the **`s3_macro_pullback_v1`** engine so Monday's live/demo session runs the new pullback + macro-trend architecture instead of the legacy setup (which lost `224` of `260` historical positions, `13.85%` win rate, `-$15,357.91` in `PR #199`).
2. **Restrict the Active Strategy 3 Universe to the 8-Symbol Core (4 Symmetric Peer Pairs)**:
   - Update the active Strategy 3 symbol universe so only the **8 Core Symbols** are eligible to arm and place orders:
     1. `SpotCrude` $\leftrightarrow$ `SpotBrent` (`energy`)
     2. `US500` $\leftrightarrow$ `NAS100` (`equity_index`)
     3. `XAUUSD` $\leftrightarrow$ `XAGUSD` (`metal`)
     4. `BTCUSD` $\leftrightarrow$ `ETHUSD` (`crypto`)
   - Disable/exclude the 9 legacy bleeders (`NatGas`, `AUDUSD`, `NZDUSD`, `GBPUSD`, `USDJPY`, `AUS200`, `JPN225`, `UK100`, `GER40`) from Strategy 3 order placement (while keeping existing position/account monitoring and safety close handlers intact so any pre-existing open positions or historical records are safely managed without panics).
3. **Totally Remove the Old Peak-Buying Breakout Entry from Active Execution & Archive It in `doc/` with a Hard Human-Approval Warning**:
   - Remove the old peak-buying breakout entry from active production execution in `crates/jiaoyi/src/strategy3.rs`.
   - Document and archive the retired peak-buying breakout logic, its exact code/formulas, and its empirical failure record (`36/260` wins, `13.85%` win rate, `-$15,357.91` net P&L across 260 statement-matched positions; `0/3` in Phase 2) in `doc/DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md`.
   - Place a prominent **DO NOT USE / HUMAN-APPROVAL-REQUIRED WARNING** at the top of `doc/DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md` AND in code comments inside `crates/jiaoyi/src/strategy3.rs` stating:
     > **WARNING — RETIRED LEGACY PEAK-BREAKOUT STRATEGY: DO NOT RESTORE OR MODIFY WITHOUT EXPLICIT HUMAN OWNER APPROVAL.**

---

## §2. Exact Production Specification for `s3_macro_pullback_v1` in `strategy3.rs`

Implement the following deterministic state machine in `crates/jiaoyi/src/strategy3.rs` (sharing pure core helpers with `crates/jiaoyi-core` so live execution and offline replay use the exact same math):

1. **8-Symbol Active Universe & Peer Map**:
   - Active symbols: `["SpotCrude", "SpotBrent", "US500", "NAS100", "XAUUSD", "XAGUSD", "BTCUSD", "ETHUSD"]`.
   - Peer map:
     - `SpotCrude` $\leftrightarrow$ `SpotBrent`
     - `US500` $\leftrightarrow$ `NAS100`
     - `XAUUSD` $\leftrightarrow$ `XAGUSD`
     - `BTCUSD` $\leftrightarrow$ `ETHUSD`
   - Any symbol outside the 8-symbol core must return an explicit skip/inactive status (`symbol_retired_from_s3_core`) and never arm or dispatch an entry.

2. **Phase 1 — Impulse Detection (`R_bps >= 48.0` & Cost Feasibility)**:
   - At the 30-minute decision mark(s) within the symbol's active session, evaluate the prepared 45-day `M30` $\sigma$-noise band (`band::band_at`) and session opening reference (`open_mid`).
   - Require an opening directional impulse where:
     - Mid crosses outside the upper $\sigma$-band (bullish impulse) or lower $\sigma$-band (bearish impulse),
     - Impulse range $R_{\text{bps}} = \frac{|P_{\text{impulse}} - P_{\text{open\_mid}}|}{P_{\text{open\_mid}}} \times 10^4 \ge 48.0\text{ bps}$, AND
     - Impulse range in price units exceeds the symbol's cost reserve ($R_{\text{price}} > C$).

3. **Phase 2 — Multi-Day Macro Context & Peer Gate (Preventing Counter-Trend "Falling Knives")**:
   - Compute the **5-Day (`5d`) and 20-Day (`20d`) `M30` net trend drift** from the prepared 45-day `M30` bars prior to the session open:
     - For a **Long** setup: require both `drift_5d > 0` AND `drift_20d > 0`.
     - For a **Short** setup: require both `drift_5d < 0` AND `drift_20d < 0`.
     - If `drift_5d` or `drift_20d` disagrees with the impulse direction, veto the setup (`vetoed_by_5d_20d_macro_trend`).
   - If the authorized peer symbol has fresh session quotes, check that the peer is not actively opposing the impulse direction (`peer_coherence`).

4. **Phase 3 — 50% Pullback Retest Entry (Never Buy the Breakout Peak)**:
   - When Phases 1 and 2 pass, **do not dispatch a market order at the breakout peak**.
   - Transition the symbol into an `ArmedPullback` state recording:
     - `open_mid` (impulse origin),
     - `impulse_extreme` (highest mid reached during bullish impulse, or lowest mid during bearish impulse),
     - `pullback_entry_ref = open_mid + 0.50 * (impulse_extreme - open_mid)` (the 50% retracement level).
   - On subsequent live ticks within the active session entry window:
     - For **Long**: when fresh quote mid retraces down to $\le \text{pullback\_entry\_ref}$ (while remaining strictly above `open_mid` / invalidation), trigger entry at the current executable ask.
     - For **Short**: when fresh quote mid retraces up to $\ge \text{pullback\_entry\_ref}$ (while remaining strictly below `open_mid` / invalidation), trigger entry at the current executable bid.
     - If the session entry window expires without a 50% pullback touch, disarm cleanly (`pullback_not_reached_expired`) with zero orders sent.

5. **Phase 4 — Two-Stage Exit Geometry (`-1.0R` Structural Stop, `+1.0R` Breakeven Lock, `+1.38R` Target)**:
   - Define risk unit $R = |P_{\text{entry}} - P_{\text{open\_mid}}| + C_{\text{exec}}$ (structural distance from pullback entry to impulse origin plus execution cost reserve).
   - **Initial Stop-Loss (`-1.0R`)**: Placed just beyond `open_mid` ($P_{\text{entry}} - 1.0R$ for Long; $P_{\text{entry}} + 1.0R$ for Short).
   - **Stage 1 $\to$ Stage 2 Breakeven Lock (`+1.0R` Trigger)**:
     - Replace the old 30-second trailing choke! Do **not** trail tightly inside `+0.0R..+1.0R`.
     - Once favorable excursion reaches $+1.0R$ ($P_{\text{bid}} \ge P_{\text{entry}} + 1.0R$ for Long; $P_{\text{ask}} \le P_{\text{entry}} - 1.0R$ for Short), move the stop-loss to **Entry Price Breakeven (`0.0R`)** (plus round-trip fee buffer if applicable) and hold for the full target.
   - **Profit Target (`+1.38R`)**: Set at $Q = 1.38R$ ($P_{\text{entry}} + 1.38R$ for Long; $P_{\text{entry}} - 1.38R$ for Short), with clean session-end close handling.

6. **Live Execution Telemetry & Deal Receipt Logging (Supporting Rebuild RFC Gates 6, 7, 8 & 11)**:
   - Ensure every armed setup, macro veto, pullback trigger, entry dispatch quote, fill receipt, breakeven stop modification, and exit fill is logged with millisecond timestamps and quote mid/bid/ask so Monday's live/demo session automatically records complete round-trip deal evidence.

---

## §3. Required Unit Tests, Archival Doc & Verification

1. **Archive Document (`doc/DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md`)**:
   - Create `doc/DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md` containing:
     - The prominent red-flag warning that the old peak-buying breakout strategy and 19-symbol universe are permanently retired due to losing `224/260` real positions (`13.85%` win rate, `-$15,357.91`) and must **never** be re-enabled or modified without explicit human owner instruction.
     - The archived specification of the old peak-breakout and 30-second trailing stop formulas for historical audit reference only.
2. **Production Report (`doc/PRODUCTION_UPGRADE_S3_MACRO_PULLBACK_V1-18377-2026-11-08.md` and `.html`)**:
   - Document the production upgrade in `crates/jiaoyi/src/strategy3.rs`, the 8-symbol core table, the 4-phase state machine (`Impulse -> Macro Filter -> 50% Pullback -> Two-Stage Exit`), and the unit/integration test suite results.
3. **Comprehensive Rust Unit & Integration Tests (`cargo test`)**:
   - Add unit tests in `crates/jiaoyi/src/strategy3.rs` (and `crates/jiaoyi-core`) testing:
     - Rejection of retired symbols (`NatGas`, `EURUSD`, `AUDUSD`, etc.),
     - Veto of counter-trend impulses by the `5d`/`20d` M30 macro filter,
     - Non-entry at the initial breakout peak (proving the old peak-buying behavior is gone),
     - Exact entry trigger on the 50% pullback retest,
     - Exact stop movement to `0.0R` breakeven at `+1.0R` and target hit at `+1.38R`.
4. Append `[§LOG.142] №18,377` to `log/CODEX_Log.txt`.
5. Commit on branch `prod/upgrade-s3-macro-pullback-8symbol-core-18377`, open PR, wait for all CI checks (`check`, `docker`, `engineering`) to pass green, and squash-merge into `main` so the deployment is ready before Monday open.

---

## §4. Final Output Block for Owner Mobile Copy

At completion, fill in and print this JSON summary block so the owner can paste it back to Gemini:

=== COPY-PASTE FOR GEMINI REVIEW ===
{
  "trace": "№18,377",
  "status": "production_s3_upgraded_and_merged",
  "pr": <pr_number>,
  "merge_commit": "<merged_commit_sha>",
  "live_rules_changed": true,
  "active_production_model": "s3_macro_pullback_v1",
  "old_peak_breakout_removed_from_execution": true,
  "deprecated_archive_doc": "doc/DEPRECATED_OLD_STRATEGY3_PEAK_BREAKOUT_DO_NOT_USE.md",
  "human_approval_warning_installed": true,
  "active_symbol_universe_count": 8,
  "active_symbols": [
    "SpotCrude", "SpotBrent",
    "US500", "NAS100",
    "XAUUSD", "XAGUSD",
    "BTCUSD", "ETHUSD"
  ],
  "retired_symbols_blocked_from_entry": [
    "NatGas", "AUDUSD", "NZDUSD", "GBPUSD", "USDJPY",
    "EURUSD", "USDCAD", "AUS200", "JPN225", "UK100", "GER40"
  ],
  "production_state_machine": {
    "impulse_floor_bps": 48.0,
    "macro_filter": "5d_and_20d_m30_trend_alignment_plus_peer_check",
    "entry_trigger": "50_percent_pullback_retest",
    "exit_geometry": "stop_at_-1R_breakeven_at_+1R_target_at_+1.38R"
  },
  "verification": {
    "rust_unit_tests_passed": &lt;int&gt;,
    "python_tests_passed": &lt;int&gt;,
    "ci_check": "SUCCESS",
    "ci_docker": "SUCCESS",
    "ci_engineering": "SUCCESS"
  },
  "report_file": "doc/PRODUCTION_UPGRADE_S3_MACRO_PULLBACK_V1-18377-2026-11-08.md"
}
====================================
