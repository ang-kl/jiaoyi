# Autonomous Coding Agent Prompt — PR & Merge Instructions for `ang-kl/jiaoyi`

№18,267 §§1–8 · 06-11-2026 07:28:45 SGT · Target Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Role, Serial Convention & Hard Guardrails

§1.1. You are working in `https://github.com/ang-kl/jiaoyi`. Every commit message, PR body, report header, and chat reply MUST carry the sequential serial number (`№18,268+`), Singapore Time timestamp (`DD-MM-YYYY HH:MM:SS SGT`, `+08:00`), and `§` paragraph numbering (`§§1–N`).

§1.2. **Non-Negotiable Safety & Research Guardrails**:
- Do NOT alter live account permissions, `PREREG_FILE` (`docs/preregistration-2026-11-01-strategy3-v7.md`), `PREREG_TEXT`, or live sizing/risk caps.
- Keep the live/demo financial goal (`73%` net wins at `REQUIRED_RR = 1.38` per strategy/account) unchanged.
- Never silently drop missing quotes, censored trades, or failed jobs from denominators.
- Do not spend model API credits or trigger new paid/live broker orders during offline verification.

## §2. Files Already Uploaded or Staged in `docs/`

§2.1. Confirm that the following two plan files are present in `docs/` (or commit them from the workspace root if staged):
- `docs/predictive-optimization-plan-2026-11-06.md`
- `docs/class-prediction-plan-2026-11-06.json` (with `"reuse_only": true` and `"source_plan_sha256": "300eb7750a7637a4f1682b17945aea0d035baf3b6508449b917e52763319d234"`).

## §3. Required Changes in `crates/jiaoyi/src/strategy3.rs`

§3.1. **Net-of-Cost `banks_a_win` (`CloseResult` & `broker_exit`)**:
- Update `CloseResult` to include `pub gross_r: f64`, `pub result_r: f64`, and `pub banks_a_win: bool`.
- Add `pub fn close_result_net(side: Side, entry: f64, r: f64, fill: f64, cost_in_price: f64) -> Option<CloseResult>` where:
  - `gross_move = if side == Side::Buy { fill - entry } else { entry - fill }`
  - `gross_r = gross_move / r`
  - `result_r = (gross_move - cost_in_price) / r`
  - `banks_a_win = result_r > 0.0`
- Keep `pub fn close_result(side: Side, entry: f64, r: f64, fill: f64) -> Option<CloseResult>` calling `close_result_net(side, entry, r, fill, 0.0)` so existing callers/tests remain source-compatible.
- In `Trader::broker_exit()`, score the closing deal with `let cost = o.plan_basis.map(|b| b.cost).unwrap_or(0.0); close_result_net(side, entry, r, fill?, cost)` and record `detail["gross_r"] = json!(s.gross_r); detail["result_r"] = json!(s.result_r); detail["banks_a_win"] = json!(s.banks_a_win);`.

§3.2. **Eliminate Kelly Gate N+1 SQL Query in `Trader::entry()`**:
- Replace the unconditional `ts::closed_trades(self.pool, self.account_id(), "s3:").await?` call in `Trader::entry()` with a short-circuit check when `keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES` (`20`) and use a `HashSet<&str>` for key filtering:
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
- Add `pub fn peer_coherence(peer_returns_bps: &[f64]) -> Option<(f64, f64, bool)>` returning `(coherence_ratio, mean_signed_bps, unanimous_sign)`:
```rust
pub fn peer_coherence(peer_returns_bps: &[f64]) -> Option<(f64, f64, bool)> {
    if peer_returns_bps.is_empty() || peer_returns_bps.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let sum_signed: f64 = peer_returns_bps.iter().sum();
    let sum_abs: f64 = peer_returns_bps.iter().map(|x| x.abs()).sum();
    if sum_abs <= f64::EPSILON {
        return None;
    }
    let unanimous = peer_returns_bps.iter().all(|&x| x > 0.0)
        || peer_returns_bps.iter().all(|&x| x < 0.0);
    let coherence = (sum_signed.abs() / sum_abs).clamp(0.0, 1.0);
    let mean_bps = sum_signed / (peer_returns_bps.len() as f64);
    Some((coherence, mean_bps, unanimous))
}
```
- Add unit tests in `strategy3::tests` verifying that `close_result_net` sets `banks_a_win = false` when `0 < gross_move < cost_in_price` and that `peer_coherence(&[-9.39, -18.46, -0.20, -5.92, -3.37, -10.63])` returns `(1.0, mean_bp, true)`.

## §4. Required Changes in `ops/research/class_research.py` and `ops/research/milestone.py`

§4.1. **Explicit Data-Gap Coverage Gate (`DATA_GAP_REASONS`)**:
- Define:
```python
DATA_GAP_REASONS = frozenset({
    'missing_first_hour_samples',
    'missing_or_flat_preparation_range',
    'missing_next_fresh_entry',
    'missing_decision_quote',
    'missing_close_quote',
    'missing_predictor',
})
```
- In both `ops/research/class_research.py` (`score()`) and `ops/research/milestone.py` (`score()`), inspect `r['study'].get('status')`, `r['study'].get('marks', ())`, and `r['study'].get('quality', {})` (`bid_ticks == 0 or ask_ticks == 0`) BEFORE `if t is None: continue`.
- If any episode in a symbol/class subset has a data gap, record `data_gaps` in the score dictionary and append `'data coverage gaps (...)'` to `reasons` so `passes_screen` is `False`. Legitimate signal skips (`flat_predictor`, `peer_disagreement`, `no_breakout`) remain valid non-entries without failing data coverage.

§4.2. **Support `reuse_only` Exploratory Plans Without Breaking Receipt Verification**:
- In `ops/research/class_research.py`, add `verify_receipt(out, plan, digest)` and `normalise_saved_request(req)`:
  - When `plan.get("reuse_only")` is `True`, verify that `(out / 'plan-sha256.txt').read_text().strip() == plan['source_plan_sha256']`, write `digest` to `out / 'analysis-plan-sha256.txt'`, and ignore `tick_replay.entry_rule` differences when matching saved broker job requests against `allowed`.

§4.3. **Rank Candidates by 90% Lower Confidence Bound (`lcb_net_r`)**:
- In both scripts, compute `lcb_net_r = mean_net_r - 1.281551565545 * (statistics.stdev(values) / math.sqrt(n))` when `n >= 2` (else `mean_net_r`).
- In `class_research.py` `select(summary)`, rank passing candidates by `(v['lcb_net_r'], v['mean_net_r'], rule)` so single-trade high-variance outliers cannot win class selection.

## §5. Required Changes in Offline Study / Cost-Context Binary (`crates/jiaoyi`)

§5.1. In the offline candidate/cost-context Rust binary (`s3_candidate_study` / `s3_cost_context`):
- Register the class-polarized candidate variants specified in `docs/class-prediction-plan-2026-11-06.json`:
  - `peer_continuation_or_own_180m`: Uses `peer_continuation` at `180m`, falling back deterministically to `own_momentum` when the peer predictor is flat (`0.0 bps`).
  - `peer_reversal_unanimous_180m` & `peer_reversal_unanimous_30m`: Uses `peer_reversal` at `180m` (`fx`) or `30m` (`metal`) only when `peer_coherence` reports `unanimous == true` ($\mathcal{C}_{\text{peer}} = 1.0$); otherwise records a clean signal skip (`peer_disagreement`).
- Retain `stress_after_slippage_bps` (`24 bps/side`) alongside calibrated net-expectancy columns at `2.0 bps/side` and `3.74 bps/side` (the max measured cTrader entry dispatch slippage in `cost_coverage.groups`).

## §6. Verification Gate Before Opening the PR

§6.1. Run all of the following commands locally and fix any warnings or failures before committing:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile ops/research/class_research.py ops/research/milestone.py
```

§6.2. If the saved 40-episode development corpus is present locally, run offline verification and confirm that `cost-context-results.json` / `summary.json` reproduces the expected SHA-256 provenance and surfaces the exact data-gap counts (`AUS200`: `missing_next_fresh_entry`, `NatGas`: `missing_decision_quote`, `JPN225`/`UK100`: `missing_close_quote`).

## §7. Git Branch, PR Creation & Merge Instructions

§7.1. Create a clean feature branch, commit with a trace-numbered message, open the PR, wait for all CI checks to pass, and merge:

```bash
git checkout -b feat/s3-predictive-regime-and-data-gap-gates
git add docs/predictive-optimization-plan-2026-11-06.md \
        docs/class-prediction-plan-2026-11-06.json \
        ops/research/class_research.py \
        ops/research/milestone.py \
        crates/jiaoyi/src/strategy3.rs
git commit -m "Research & S3: enforce data-gap gates, net banks_a_win, Kelly short-circuit, and class regime routing (№18,268)"
git push -u origin feat/s3-predictive-regime-and-data-gap-gates

gh pr create \
  --title "Research & S3: enforce data-gap gates, net banks_a_win, Kelly short-circuit, and class regime routing" \
  --body "## №18,268 §§1–4 · Summary
- **§1. Data-Gap & Reuse Lineage (\`ops/research/class_research.py\`, \`ops/research/milestone.py\`)**: Enforces \`DATA_GAP_REASONS\` so missing quotes (\`AUS200\`, \`NatGas\`, \`JPN225\`, \`UK100\`) cannot silently pass coverage when \`trade is None\`; preserves \`plan-sha256.txt\` under \`reuse_only\` via \`analysis-plan-sha256.txt\`; selects development winners by 90% lower confidence bound (\`lcb_net_r\`).
- **§2. Engine Accounting & Query Optimization (\`crates/jiaoyi/src/strategy3.rs\`)**: Adds \`close_result_net\` so \`banks_a_win\` is net of planned cost reserve \`C\`; short-circuits \`ts::closed_trades\` in \`Trader::entry()\` when \`keys.len() < MIN_CLOSES\` (\`20\`); adds \`peer_coherence\` helper and unit tests.
- **§3. Preregistered Plan (\`docs/\`)**: Adds \`docs/predictive-optimization-plan-2026-11-06.md\` and \`docs/class-prediction-plan-2026-11-06.json\`.
- **§4. Verification**: \`cargo test --workspace\` and Python syntax/unit checks pass; no live trading policy or account goal (\`73% / 1:1.38\`) changed."

gh pr checks --watch
gh pr merge --squash --delete-branch
```
