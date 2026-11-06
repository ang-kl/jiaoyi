# Autonomous Coding Agent Prompt — PR & Merge Instructions for `ang-kl/jiaoyi`

№18,268 §§1–8 · 06-11-2026 07:36:00 SGT · Target Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Role, Serial Convention & Hard Guardrails

§1.1. You are working in `https://github.com/ang-kl/jiaoyi` on top of `main` (at or after PR #164 merge commit `7ae2e64c4e88deb2ad3d802a0a97194a8b502d5f`). Every commit message, PR body, report header, and log entry MUST carry the sequential serial number (`№18,269+`), Singapore Time timestamp (`DD-MM-YYYY HH:MM:SS SGT`, `+08:00`), and `§` paragraph numbering (`§§1–N`). Read `log/CODEX_Log.txt` and `log/GROK_Log.txt` before allocating the next serial, and record your work in your model's log under `log/` without attributing another AI's work to yourself.

§1.2. **Non-Negotiable Safety & Research Guardrails**:
- Do NOT alter live account permissions, the 17 production symbols, `PREREG_FILE` (`docs/preregistration-2026-11-01-strategy3-v7.md`), `PREREG_TEXT`, or live sizing/risk caps.
- Keep the live/demo financial goal (`73%` net wins at `REQUIRED_RR = 1.38` per strategy/account) unchanged.
- Preserve the `/owner/backtests` pre-`2025-10-24` development seal and the `/owner/market-data` post-`2026-10-24` collection-only guard from PR #164.
- Never silently drop missing quotes, censored trades, or failed jobs from denominators.

## §2. Files to Add or Update in `docs/`

§2.1. Commit the following plan and evaluation files into `docs/`:
- `docs/predictive-optimization-plan-2026-11-06.md`
- `docs/class-prediction-plan-2026-11-06.json`

## §3. Required Changes in `crates/jiaoyi/src/strategy3.rs`

§3.1. **Net-of-Cost `banks_a_win` (`CloseResult` & `broker_exit`)**:
- Update `CloseResult` to include `pub gross_r: f64`, `pub result_r: f64`, and `pub banks_a_win: bool`.
- Add `pub fn close_result_net(side: Side, entry: f64, r: f64, fill: f64, cost_in_price: f64) -> Option<CloseResult>` where:
  - `gross_move = if side == Side::Buy { fill - entry } else { entry - fill }`
  - `gross_r = gross_move / r`
  - `result_r = (gross_move - cost_in_price) / r`
  - `banks_a_win = result_r > 0.0`
- Keep `pub fn close_result(side: Side, entry: f64, r: f64, fill: f64) -> Option<CloseResult>` calling `close_result_net(side, entry, r, fill, 0.0)`.
- In `Trader::broker_exit()`, score the closing deal with `let cost = o.plan_basis.map(|b| b.cost).unwrap_or(0.0); close_result_net(side, entry, r, fill?, cost)` and record `detail["gross_r"] = json!(s.gross_r); detail["result_r"] = json!(s.result_r); detail["banks_a_win"] = json!(s.banks_a_win);`.

§3.2. **Eliminate Kelly Gate N+1 SQL Query in `Trader::entry()`**:
- Short-circuit `ts::closed_trades(self.pool, self.account_id(), "s3:").await?` in `Trader::entry()` when `keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES` (`20`) and use a `HashSet<&str>` for key filtering:
```rust
let keys =
    s3s::open_keys_for_hash(self.pool, self.account_id(), symbol_id, &self.hash).await?;
let rs: Vec<f64> = if keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES {
    Vec::new()
} else {
    let (closed, _) = ts::closed_trades(self.pool, self.account_id(), "s3:").await?;
    let key_set: std::collections::HashSet<&str> = keys.iter().map(String::as_str).collect();
    closed
        .iter()
        .filter(|t| key_set.contains(t.entry_key.as_str()))
        .map(|t| t.r)
        .collect()
};
```

§3.3. **Add Pure `peer_coherence` Helper & Unit Tests**:
- Add `pub fn peer_coherence(peer_returns_bps: &[f64]) -> Option<(f64, f64, bool)>` returning `(coherence_ratio, mean_signed_bps, unanimous_sign)` and unit tests in `strategy3::tests`.

## §4. Required Changes in `ops/research/class_research.py` and `ops/research/milestone.py`

§4.1. **Explicit Data-Gap Coverage Gate (`DATA_GAP_REASONS`)**:
- Check `DATA_GAP_REASONS` (`missing_first_hour_samples`, `missing_or_flat_preparation_range`, `missing_next_fresh_entry`, `missing_decision_quote`, `missing_close_quote`, `missing_predictor`, `zero_tick_stream`) before `if t is None: continue` so episodes with missing quotes fail `passes_screen` with `'data coverage gaps (...)'`.

§4.2. **Support `reuse_only` Exploratory Plans & 90% Lower Confidence Bound (`lcb_net_r`)**:
- Preserve `plan-sha256.txt` when `plan.get("reuse_only")` is `True` by verifying `plan["source_plan_sha256"]` and writing the exploratory digest to `analysis-plan-sha256.txt`.
- Compute `lcb_net_r = mean_net_r - 1.281551565545 * (statistics.stdev(values) / math.sqrt(n))` (`n >= 2`) and rank passing candidates in `select()` by `(lcb_net_r, mean_net_r, rule)`.

## §5. Quote-Validator & DST Peer Fixes in Offline Cost-Context / Study Code

§5.1. **Allow Zero-Spread Two-Sided Quotes (`bid <= ask`)**:
- On `2026-11-05` at `08:00:00 UTC`, `EURUSD` prints valid zero-pip spread quotes (`bid_e5 == ask_e5 == 112431`). Ensure two-sided quote reconstruction accepts `bid_e5 <= ask_e5` (`bid > 0 && ask >= bid`), rejecting only crossed books (`bid > ask`).

§5.2. **Document/Guard Southern Hemisphere DST Offset on `AUS200` vs. `JPN225`**:
- During AEDT (`UTC+11`, e.g. `2026-11-05`), `AUS200` (`10:00 Australia/Sydney` = `23:00 UTC`) reaches its Hour-1 decision mark at `00:00:00 UTC`, which is the exact opening second of `JPN225` (`09:00 Asia/Tokyo` = `00:00 UTC`), so `JPN225` has zero first-hour samples as of `00:00:00 UTC`. Ensure `missing_predictor` is explicitly reported on `AUS200` peer variants during AEDT while `own_momentum` evaluates `AUS200` normally (`+21.12 bps` at `180m` on `2026-11-05`).

## §6. Verification Gate Before Opening the PR

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile ops/research/class_research.py ops/research/milestone.py code/market_collection.py
```

## §7. Git Branch, PR Creation & Merge Instructions

```bash
git checkout -b feat/s3-predictive-regime-and-fresh-20261105-eval
git add docs/predictive-optimization-plan-2026-11-06.md \
        docs/class-prediction-plan-2026-11-06.json \
        ops/research/class_research.py \
        ops/research/milestone.py \
        crates/jiaoyi/src/strategy3.rs
git commit -m "Research & S3: data-gap gates, net banks_a_win, Kelly short-circuit, and 2026-11-05 evaluation (№18,269)"
git push -u origin feat/s3-predictive-regime-and-fresh-20261105-eval

gh pr create \
  --title "Research & S3: data-gap gates, net banks_a_win, Kelly short-circuit, and 2026-11-05 evaluation" \
  --body "## №18,269 §§1–4 · Summary
- **§1. Out-of-Sample 2026-11-05 Evaluation (1,281,012 Ticks, PR #164 Corpus)**: Evaluates the 240 fresh diagnostic rows on \`2026-11-05\` and pools across all 3 dates (\`720\` rows: \`2025-08-05\`, \`2025-08-12\`, \`2026-11-05\`). Confirms out-of-sample edge on \`energy\` oil pair (\`SpotBrent\`/\`SpotCrude\` \`peer_continuation\` @ \`180m\`: \`100%\` win rate, \`+82.54 bps\` on \`5 Nov\`, \`+65.17 bps\` pooled across \`5/5\` episodes), \`metal\` (\`XAUUSD\`/\`XAGUSD\` \`peer_reversal\` @ \`30m\`–\`180m\`: \`100%\` win rate on \`5 Nov\`, \`100%\` \`4/4\` on unanimous USD peers), and \`equity_index\` (\`own_momentum\` @ \`180m\`: \`100%\` \`7/7\` on \`5 Nov\` with \`+80.00 bps\` mean; \`73.7%\` \`14/19\` @ \`30m\` pooled), while falsifying \`fx\` \`peer_reversal\` @ \`180m\` (\`0/6\` on \`5 Nov\`).
- **§2. Data-Gap & Reuse Lineage (\`ops/research/class_research.py\`, \`ops/research/milestone.py\`)**: Enforces \`DATA_GAP_REASONS\`, preserves \`plan-sha256.txt\` under \`reuse_only\`, and ranks candidates by 90% lower confidence bound (\`lcb_net_r\`).
- **§3. Engine Fixes (\`crates/jiaoyi/src/strategy3.rs\`)**: Adds \`close_result_net\` so \`banks_a_win\` is net of cost reserve \`C\`, short-circuits \`ts::closed_trades\` when \`keys.len() < 20\`, and adds \`peer_coherence\` + unit tests.
- **§4. Verification**: Exact-head \`cargo fmt\`, \`cargo clippy -D warnings\`, \`cargo test --workspace\`, and Python checks pass; live policy and \`73% / 1:1.38\` goal unchanged."

gh pr checks --watch
gh pr merge --squash --delete-branch
```