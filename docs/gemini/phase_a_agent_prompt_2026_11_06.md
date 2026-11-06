# Trimmed Phase-A Coding Agent Prompt for `ang-kl/jiaoyi`

№18,275 §§1–6 · 06-11-2026 10:53:45 SGT (+08:00) · Target Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Scope, Attribution & Hard Boundaries

§1.1. Implement **only** the agreed **Phase A** offline research and non-behavioral engine improvements plus the **5 acceptance tests** reconciled across Codex (`№18,268–18,273`) and the Independent Review (`№18,274`, `Independent_Review_of_18270.md`).

§1.2. **Mandatory Traceability & Safety Rules**:
- Check `log/CODEX_Log.txt` and `log/GROK_Log.txt` before allocating the next chat/PR serial (`№18,276+`), use `SGT (+08:00)` timestamps and `§` paragraph numbering (`§§1–N`), and log your work in your own model's log under `log/`.
- **Do NOT modify**:
  - `close_result()` or `broker_exit()` cost subtraction in `crates/jiaoyi/src/strategy3.rs` (executable fills already include realized spread and slippage; subtracting `PlanBasis.cost` double-counts costs).
  - `ops/research/cost_context.py` zero-spread quote reconstruction (`sides[0][1] > sides[1][1]` on line 41 already accepts `bid == ask`).
  - Live account permissions, the 17 production symbols, `native_volume` defaults (`AUS200/GER40/NAS100/UK100/US30/US500 = 10`, `JPN225 = 100`), `PREREG_FILE` (`docs/preregistration-2026-11-01-strategy3-v7.md`), or the `73%` net wins / `1:1.38` R:R account goal.
  - The `/owner/backtests` pre-`2025-10-24` seal or `/owner/market-data` post-`2026-10-24` collection-only guard.

## §2. Minimal Changes in `crates/jiaoyi/src/strategy3.rs`

§2.1. **Kelly Gate SQL Short-Circuit (`Trader::entry()`)**:
- When `keys.len() < jiaoyi_core::strategy3::kelly::MIN_CLOSES` (`20`), skip calling `ts::closed_trades(self.pool, self.account_id(), "s3:").await?` because `rs.len()` is bounded above by `keys.len()` and `kelly::read` requires at least `MIN_CLOSES` (`20`) closes. When `keys.len() >= 20`, filter using a `HashSet<&str>`:
```rust
pub fn should_query_closed_trades_for_kelly(open_keys_count: usize) -> bool {
    open_keys_count >= jiaoyi_core::strategy3::kelly::MIN_CLOSES
}
```
```rust
let keys =
    s3s::open_keys_for_hash(self.pool, self.account_id(), symbol_id, &self.hash).await?;
let rs: Vec<f64> = if !should_query_closed_trades_for_kelly(keys.len()) {
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

§2.2. **Pure `peer_coherence` Helper**:
- Add a pure helper function in `crates/jiaoyi/src/strategy3.rs` for offline research candidate evaluation:
```rust
/// Computes peer-basket coherence |sum(r_i)| / sum(|r_i|) in [0.0, 1.0],
/// mean signed peer return in bps, and strict sign unanimity.
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

## §3. Minimal Changes in `ops/research/class_research.py` and `ops/research/milestone.py`

§3.1. **Explicit Data-Gap Failure Gate (`DATA_GAP_REASONS`)**:
- Define:
```python
DATA_GAP_REASONS = frozenset({
    'missing_first_hour_samples',
    'missing_or_flat_preparation_range',
    'missing_next_fresh_entry',
    'missing_decision_quote',
    'missing_close_quote',
    'missing_predictor',
    'zero_tick_stream',
})
```
- In both `ops/research/class_research.py` (`score()`) and `ops/research/milestone.py` (`score()`), inspect `r['study'].get('status')`, `r['study'].get('marks', ())`, and `r['study'].get('quality', {})` (`bid_ticks == 0 or ask_ticks == 0`) BEFORE `if t is None: continue`.
- Record `data_gaps` in the returned dictionary and append `'data coverage gaps (...)'` to `reasons` whenever any episode has a data gap, while leaving legitimate non-entry signal skips (`peer_disagreement`, `flat_predictor`, `no_breakout`) as valid skips.

§3.2. **Distinct Collection vs. Analysis Receipt Verification (`reuse_only`)**:
- In `ops/research/class_research.py`, add `verify_receipt(out, plan, digest)`:
  - When `plan.get('reuse_only')` is `True`, verify that `(out / 'plan-sha256.txt').read_text().strip() == plan['source_collection_plan_sha256']` (where the original 40-episode collection receipt is `8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaae648e642cb3a2d20`, never confused with the cost-context plan hash `300eb7750a7637a4f1682b17945aea0d035baf3b6508449b917e52763319d234`), write `digest` to `out / 'analysis-plan-sha256.txt'`, and do not overwrite `plan-sha256.txt`.

§3.3. **90% Lower Confidence Bound (`lcb_net_r`) Selection**:
- In `score()`, compute `lcb_net_r = mean_net_r - 1.281551565545 * (statistics.stdev(values) / math.sqrt(n))` when `n >= 2` (else `mean_net_r`).
- In `class_research.py` `select(summary)`, rank passing development candidates per class by `(v.get('lcb_net_r') if v.get('lcb_net_r') is not None else v['mean_net_r'], v['mean_net_r'], rule)`.

## §4. Five Consequential Acceptance Tests

§4.1. Add and run the following 5 focused tests:
1. **`test_kelly_short_circuit_under_min_closes` (`crates/jiaoyi/src/strategy3.rs`)**: Asserts `should_query_closed_trades_for_kelly(0)` and `should_query_closed_trades_for_kelly(19)` are `false`, `should_query_closed_trades_for_kelly(20)` is `true`, and `jiaoyi_core::strategy3::kelly::skip(&jiaoyi_core::strategy3::kelly::read(&[]))` is `false`.
2. **`test_peer_coherence_unanimity` (`crates/jiaoyi/src/strategy3.rs`)**: Asserts `peer_coherence(&[-9.39, -18.46, -0.20, -5.92, -3.37, -10.63])` returns `Some((1.0, _, true))`, mixed signs return `Some((coh, _, false))` with `coh < 1.0`, and `&[]`, `&[0.0, 0.0]`, or `&[f64::NAN]` return `None`.
3. **`test_score_fails_on_data_gap_when_trade_is_none` (Python test in `ops/research/`)**: Asserts `class_research.score()` and `milestone.score()` return `passes_screen == False` with `'data coverage gaps'` in `reasons` when an episode has `trade: None` and mark/status `missing_next_fresh_entry` or `missing_close_quote`, while `flat_predictor` or `peer_disagreement` with `trade: None` does not add a data-gap failure reason.
4. **`test_reuse_only_preserves_collection_receipt` (Python test in `ops/research/`)**: Asserts `verify_receipt()` rejects a mismatched `source_collection_plan_sha256`, accepts `8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaae648e642cb3a2d20`, leaves `plan-sha256.txt` untouched, and writes the new digest to `analysis-plan-sha256.txt`.
5. **`test_reproduce_review_exact_match` (Python check)**: If the local corpora are present, runs `docs/ai-review-2026-11-06/scripts/reproduce_review.py` and asserts `development_rows_reproduced_exactly == 480`, `tick_count == 1281012`, and `fresh_statuses == {'observed': 222, 'missing_predictor': 18}`.

## §5. Pre-PR Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 -m py_compile ops/research/class_research.py ops/research/milestone.py
```

## §6. Git Branch, PR Creation & Merge Commands

```bash
git checkout -b feat/phase-a-research-gates-and-kelly-short-circuit
git add crates/jiaoyi/src/strategy3.rs \
        ops/research/class_research.py \
        ops/research/milestone.py
git commit -m "Research & S3: Phase A data-gap gates, reuse receipt lineage, Kelly short-circuit, and peer_coherence (№18,276)"
git push -u origin feat/phase-a-research-gates-and-kelly-short-circuit

gh pr create \
  --title "Research & S3: Phase A data-gap gates, reuse receipt lineage, Kelly short-circuit, and peer_coherence" \
  --body "## №18,276 §§1–4 · Summary (Reconciled Phase A from №18,270 & №18,274)
- **§1. Engine (\`crates/jiaoyi/src/strategy3.rs\`)**: Short-circuits \`ts::closed_trades\` in \`Trader::entry()\` when \`keys.len() < MIN_CLOSES\` (\`20\`) and adds pure helper \`peer_coherence()\`. Leaves \`close_result()\` untouched so realized fills never double-count \`PlanBasis.cost\`.
- **§2. Research Gates (\`ops/research/class_research.py\`, \`ops/research/milestone.py\`)**: Enforces \`DATA_GAP_REASONS\` when \`trade is None\`, preserves \`plan-sha256.txt\` (\`8c8cb376...\`) under \`reuse_only\` via \`analysis-plan-sha256.txt\`, and ranks passing development candidates by 90% lower confidence bound (\`lcb_net_r\`).
- **§3. Acceptance Tests**: Adds the 5 Phase-A acceptance tests covering Kelly short-circuiting, \`peer_coherence\`, data-gap failure gates, \`reuse_only\` receipt preservation, and review reproduction.
- **§4. Boundaries**: Zero changes to live trading permissions, 17 production symbols, \`native_volume\`, \`cost_context.py\`, or the \`73% / 1:1.38\` goal."

gh pr checks --watch
gh pr merge --squash --delete-branch
```
