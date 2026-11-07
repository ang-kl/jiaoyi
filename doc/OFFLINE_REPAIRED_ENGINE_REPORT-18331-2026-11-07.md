# Offline repaired-engine experiment

№18,331 §§1–8 · 07-11-2026 15:53:10 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**119/119 cached native replays completed. All 120 scheduled rows remain, including the prior failed source.** Zero new broker requests. Primary outcomes: s3_dynamic_two_stage: 6/83 wins, mean -2.961114 R; macro_180m_repaired: 11/54 wins, mean -0.360854 R.

**No financial qualification or demonstrated progress towards the per-strategy/account 73% / reference R:R 1:1.38 latest-20 goal.** These are reused development dates, uncalibrated cost scenarios, and an inherited protocol failure affecting20 episodes. A 50% hit rate alone is insufficient. Validation stays sealed; no production entry, exit, registration, account or worker configuration changed.

### §1.1. What the measured changes explain

- **Two-stage exits alone:** unchanged five S3 entries, five closes, three wins (60%), mean +0.123851R and PF1.451491 at the primary costs. The original arm closed only three positions and censored two. The two newly closed winners both have inherited failed-interval protocol flags. Three of the five closes have protocol flags overall. The two closes without this particular flag have one win and mean −0.004221R; other quote-quality limitations remain. This is an exploratory lead, not a qualified improvement or proof that trailing caused the losses.
- **Dynamic gate alone:** entries rise from5 to93 (88 added), with83 closes, only one net winner and mean −3.035711R. At entry,86/93 have the old cost reserve at least as large as R; median reserve is2.824972R. On the83 closes, mean gross return is already −0.199246R and modelled costs subtract another2.836465R. The volatility filter does not replace an economic cost check.
- **Dynamic gate plus two-stage exits:** six wins from83 closes (7.23%), mean −2.961114R. Mean gross return remains −0.138772R. Even the uncalibrated2bp sensitivity has26/83 wins and mean −0.373967R. This combination does not solve the entry problem under either stated scenario.
- **Macro:**11/54 wins remain20.37%; mean shifts from −0.360042R to −0.360854R. There are no additional entries, because NAS100/silver observations are absent. Mean gross return is −0.015200R before the modelled costs. Authorisation is implemented and tested; missing data still prevents the requested index/gold evaluation.

**Decision:** retain the original production cost screen and trading rules. Reject the dynamic-gate substitution and combined candidate under these tested assumptions. Keep the five-entry exit-only result as a small, protocol-sensitive research observation; do not promote it or claim the50% per-class target. Any next exit study needs a separately frozen, representative dataset and defensible costs. Do not tune this sample further or automatically collect peer data.

## §2. What the supplied scripts actually did

Both attachments loaded the previous result JSON but did not use it to replay ticks. They assigned 96 macro trades/52 wins and48 S3 trades/31 wins as constants. Their per-symbol S3 wins sum to32, conflicting with31 overall. The Bash review block's means/PFs also disagree with its own arithmetic. Its CI reader uses `.state` for CheckRuns and can treat null as green; its quoted heredoc prints the literal merge command. Those outputs were not executed or published as market evidence.

This implementation executes native Rust on verified cached quote files and checks exact original-model parity for every successful episode. The Markdown directive's №18,331 governs; the companion Bash's №18,332 is not a separate experiment. Original attachment hashes and the pre-run interpretation are in the [frozen contract](../docs/offline-repair-contract-18331.md).

## §3. Frozen interpretation and limits

- Two-stage: original −1R stop until executable bid(long)/ask(short) reaches +1R; propose stop at entry, acknowledge on a strictly later quote only if still valid. Old broker bracket wins first. No further trailing or partial exit. Whole-position **gross** target1.38R rounded outward to the price grid. This changes target geometry as well as trailing; it cannot establish that trailing alone caused losses. An entry-price stop is gross-flat and usually net-negative after fees/slippage.
- Dynamic S3 admission: R must exceed max(2.5×current executable spread,0.15×ATR14) in **price units**, at decision and dispatch. ATR uses14 completed positive-volume M30 ranges plus the preceding close. Missing ATR blocks entry; failed numeric gate consumes the session. This replaces the old admission comparison only. The old rule was spread+commission+two-sided slippage relative to R, not a universal48bp threshold.
- Primary accounting retains24bp/side and executable bid/ask fills; extra commission remains the frozen zero-cost assumption. No double spread charge. Sensitivities2/3.74bp reprice the same fills; no empirical calibration claimed.
- Peer mappings authorise US500/NAS100, XAUUSD/XAGUSD and Crude/Brent. The119 cached datasets have oil peers only. US500 and Gold remain unavailable for macro; authorisation does not manufacture six completed peer bars.
- S3 original window/close stays intact: oil source07–17UTC reconstructs the07UTC start and censors at17 while native close remains20. ORB/macro/control window13–17 for oil. Index uses DST-aware NY open; Gold07–11UTC. No synthetic session-end close at a truncated boundary.
- Three S3 arms isolate two-stage geometry, dynamic admission, and both together. Macro applies authorised peers/two-stage geometry while retaining original admission. Controls and every baseline are preserved. No additional parameter search follows the outcomes.

## §4. Overall scenarios

Win rates use closed trades; nonentries, missing data and censored positions remain explicit in the120-row denominator. Profit factor is based on normalised R, not mixed-account dollars.

| Model | bp/side | Entries | Closes | Wins | Net win rate | Mean net R | PF(R) | Gross flat | Censored |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| production_s3_baseline | 24 | 5 | 3 | 0 | 0.000000 | -0.872489 | 0.000000 | 0 | 2 |
| production_s3_baseline | 2 | 5 | 3 | 2 | 0.666667 | -0.198179 | 0.180171 | 0 | 2 |
| production_s3_baseline | 3.74 | 5 | 3 | 2 | 0.666667 | -0.251511 | 0.040105 | 0 | 2 |
| s3_two_stage | 24 | 5 | 5 | 3 | 0.600000 | 0.123851 | 1.451491 | 1 | 0 |
| s3_two_stage | 2 | 5 | 5 | 3 | 0.600000 | 0.769237 | 33.562394 | 1 | 0 |
| s3_two_stage | 3.74 | 5 | 5 | 3 | 0.600000 | 0.718193 | 17.528835 | 1 | 0 |
| s3_dynamic_gate | 24 | 93 | 83 | 1 | 0.012048 | -3.035711 | 0.005578 | 0 | 10 |
| s3_dynamic_gate | 2 | 93 | 83 | 25 | 0.301205 | -0.435618 | 0.315226 | 0 | 10 |
| s3_dynamic_gate | 3.74 | 93 | 83 | 20 | 0.240964 | -0.641262 | 0.187096 | 0 | 10 |
| s3_dynamic_two_stage | 24 | 93 | 83 | 6 | 0.072289 | -2.961114 | 0.010911 | 5 | 10 |
| s3_dynamic_two_stage | 2 | 93 | 83 | 26 | 0.313253 | -0.373967 | 0.489902 | 5 | 10 |
| s3_dynamic_two_stage | 3.74 | 93 | 83 | 26 | 0.313253 | -0.578587 | 0.337331 | 5 | 10 |
| macro_180m_baseline | 24 | 55 | 54 | 11 | 0.203704 | -0.360042 | 0.042180 | 0 | 1 |
| macro_180m_baseline | 2 | 55 | 54 | 27 | 0.500000 | -0.043191 | 0.728734 | 0 | 1 |
| macro_180m_baseline | 3.74 | 55 | 54 | 26 | 0.481481 | -0.068251 | 0.606012 | 0 | 1 |
| macro_180m_repaired | 24 | 55 | 54 | 11 | 0.203704 | -0.360854 | 0.042089 | 0 | 1 |
| macro_180m_repaired | 2 | 55 | 54 | 27 | 0.500000 | -0.044005 | 0.725028 | 0 | 1 |
| macro_180m_repaired | 3.74 | 55 | 54 | 26 | 0.481481 | -0.069065 | 0.603179 | 0 | 1 |
| always_long_control | 24 | 53 | 52 | 18 | 0.346154 | -0.384622 | 0.359664 | 0 | 1 |
| always_long_control | 2 | 53 | 52 | 31 | 0.596154 | 0.206765 | 1.721663 | 0 | 1 |
| always_long_control | 3.74 | 53 | 52 | 30 | 0.576923 | 0.159991 | 1.523880 | 0 | 1 |
| own_momentum_fixed_horizon | 24 | 52 | 51 | 16 | 0.313725 | -0.527552 | 0.244466 | 0 | 1 |
| own_momentum_fixed_horizon | 2 | 52 | 51 | 28 | 0.549020 | 0.061098 | 1.169356 | 0 | 1 |
| own_momentum_fixed_horizon | 3.74 | 52 | 51 | 28 | 0.549020 | 0.014541 | 1.038042 | 0 | 1 |

## §5. Per-symbol / class / year primary results

| Group | Model | Episodes | Entries | Closes | Wins | Win rate | Mean R | PF(R) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| symbol:SpotBrent | s3_two_stage | 30 | 2 | 2 | 2 | 1.000000 | 0.618142 | unavailable |
| symbol:SpotBrent | s3_dynamic_gate | 30 | 27 | 26 | 0 | 0.000000 | -2.630607 | 0.000000 |
| symbol:SpotBrent | s3_dynamic_two_stage | 30 | 27 | 27 | 4 | 0.148148 | -2.449101 | 0.024879 |
| symbol:SpotBrent | macro_180m_repaired | 30 | 27 | 27 | 5 | 0.185185 | -0.389957 | 0.026157 |
| symbol:SpotCrude | s3_two_stage | 30 | 3 | 3 | 1 | 0.333333 | -0.205676 | 0.550131 |
| symbol:SpotCrude | s3_dynamic_gate | 30 | 28 | 26 | 0 | 0.000000 | -2.278118 | 0.000000 |
| symbol:SpotCrude | s3_dynamic_two_stage | 30 | 28 | 26 | 2 | 0.076923 | -2.209343 | 0.017518 |
| symbol:SpotCrude | macro_180m_repaired | 30 | 28 | 27 | 6 | 0.222222 | -0.331750 | 0.060161 |
| symbol:US500 | s3_two_stage | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| symbol:US500 | s3_dynamic_gate | 30 | 19 | 15 | 1 | 0.066667 | -2.872620 | 0.031758 |
| symbol:US500 | s3_dynamic_two_stage | 30 | 19 | 14 | 0 | 0.000000 | -2.974352 | 0.000000 |
| symbol:US500 | macro_180m_repaired | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| symbol:XAUUSD | s3_two_stage | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| symbol:XAUUSD | s3_dynamic_gate | 30 | 19 | 16 | 0 | 0.000000 | -5.077991 | 0.000000 |
| symbol:XAUUSD | s3_dynamic_two_stage | 30 | 19 | 16 | 0 | 0.000000 | -5.035181 | 0.000000 |
| symbol:XAUUSD | macro_180m_repaired | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| asset_class:energy | s3_two_stage | 60 | 5 | 5 | 3 | 0.600000 | 0.123851 | 1.451491 |
| asset_class:energy | s3_dynamic_gate | 60 | 55 | 52 | 0 | 0.000000 | -2.454363 | 0.000000 |
| asset_class:energy | s3_dynamic_two_stage | 60 | 55 | 53 | 6 | 0.113208 | -2.331484 | 0.021471 |
| asset_class:energy | macro_180m_repaired | 60 | 55 | 54 | 11 | 0.203704 | -0.360854 | 0.042089 |
| asset_class:equity_index | s3_two_stage | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| asset_class:equity_index | s3_dynamic_gate | 30 | 19 | 15 | 1 | 0.066667 | -2.872620 | 0.031758 |
| asset_class:equity_index | s3_dynamic_two_stage | 30 | 19 | 14 | 0 | 0.000000 | -2.974352 | 0.000000 |
| asset_class:equity_index | macro_180m_repaired | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| asset_class:metal | s3_two_stage | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| asset_class:metal | s3_dynamic_gate | 30 | 19 | 16 | 0 | 0.000000 | -5.077991 | 0.000000 |
| asset_class:metal | s3_dynamic_two_stage | 30 | 19 | 16 | 0 | 0.000000 | -5.035181 | 0.000000 |
| asset_class:metal | macro_180m_repaired | 30 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| year:2023 | s3_two_stage | 40 | 2 | 2 | 1 | 0.500000 | -0.004221 | 0.984514 |
| year:2023 | s3_dynamic_gate | 40 | 35 | 29 | 1 | 0.034483 | -3.297392 | 0.014565 |
| year:2023 | s3_dynamic_two_stage | 40 | 35 | 30 | 1 | 0.033333 | -3.356246 | 0.005302 |
| year:2023 | macro_180m_repaired | 40 | 20 | 20 | 7 | 0.350000 | -0.176459 | 0.075834 |
| year:2024 | s3_two_stage | 40 | 3 | 3 | 2 | 0.666667 | 0.209232 | 1.759530 |
| year:2024 | s3_dynamic_gate | 40 | 28 | 25 | 0 | 0.000000 | -2.733493 | 0.000000 |
| year:2024 | s3_dynamic_two_stage | 40 | 28 | 27 | 5 | 0.185185 | -2.356385 | 0.033050 |
| year:2024 | macro_180m_repaired | 40 | 20 | 19 | 4 | 0.210526 | -0.340629 | 0.080498 |
| year:2025 | s3_two_stage | 40 | 0 | 0 | 0 | unavailable | unavailable | unavailable |
| year:2025 | s3_dynamic_gate | 40 | 30 | 29 | 0 | 0.000000 | -3.034562 | 0.000000 |
| year:2025 | s3_dynamic_two_stage | 40 | 30 | 26 | 0 | 0.000000 | -3.133181 | 0.000000 |
| year:2025 | macro_180m_repaired | 40 | 15 | 15 | 0 | 0.000000 | -0.632332 | 0.000000 |

## §6. Matched comparison and failure prevention

```json
{
  "s3_two_stage": {
    "baseline": "production_s3_baseline",
    "matched_closes": 3,
    "unmatched_episodes": 117,
    "added_entry_episodes": 0,
    "removed_entry_episodes": 0,
    "mean_delta_r": 0.594199441281881,
    "caveat": "Conditional same-episode closed pairs; changed target/admission changes selection. No causal alpha or significance claim."
  },
  "s3_dynamic_gate": {
    "baseline": "production_s3_baseline",
    "matched_closes": 3,
    "unmatched_episodes": 117,
    "added_entry_episodes": 88,
    "removed_entry_episodes": 0,
    "mean_delta_r": 0.0,
    "caveat": "Conditional same-episode closed pairs; changed target/admission changes selection. No causal alpha or significance claim."
  },
  "s3_dynamic_two_stage": {
    "baseline": "production_s3_baseline",
    "matched_closes": 3,
    "unmatched_episodes": 117,
    "added_entry_episodes": 88,
    "removed_entry_episodes": 0,
    "mean_delta_r": 0.594199441281881,
    "caveat": "Conditional same-episode closed pairs; changed target/admission changes selection. No causal alpha or significance claim."
  },
  "macro_180m_repaired": {
    "baseline": "macro_180m_baseline",
    "matched_closes": 54,
    "unmatched_episodes": 66,
    "added_entry_episodes": 0,
    "removed_entry_episodes": 0,
    "mean_delta_r": -0.0008120472120481464,
    "caveat": "Conditional same-episode closed pairs; changed target/admission changes selection. No causal alpha or significance claim."
  }
}
```

The matched differences describe this reused sample only. Added entries, altered censoring and changed gross targets make aggregate win-rate comparisons insufficient to prove causality or edge. Preserve all losses and failed rows. The20 source-protocol flags remain; offline reuse does not erase the earlier broker rereads. Same-millisecond ordering, gaps, missing preparation, current-only contract metadata and uncalibrated execution costs still limit the evidence.

Next: compare the actual primary expectancy and per-symbol results before any further hypothesis. Independently establish broker cost/contract assumptions; do not choose cheaper costs because they look better. NAS100/XAGUSD collection requires a separately bounded and frozen source plan. A repaired collector/protocol and untouched development evidence are prerequisites to a credible next stage. Do not open Stage2, validation, or live promotion from this experiment.

## §7. Verification and reproducibility

```json
{
  "source_components": 296,
  "baseline_parity_episodes": 119,
  "independent_cost_values": 1314,
  "native_replays_completed": 119
}
```

Plan SHA256: `195e24b3312f0aeba7a0919c277db9453be28ea750e371856a81814c39b03e17`. Pre-run freeze commit: `56aa6f131ff3527a93272cabe4fa0eea51cea331`. Binary SHA256: `a95ac9a7a87a5e6d1d3713cbef2e68a8e1fccd86229b91d60fb0e7e59f2cc4e1`.

All original source receipt hashes and the baseline were checked before/after replay. Raw quotes remain in the existing workspace corpus, outside Git; the corpus is required to reproduce the run. The full JSON retains source hashes, native fills, per-episode nonentries, peer gaps and inherited protocol flags.

## §8. Independent AI review

Read this report, the frozen contract, the [measured JSON](../docs/offline-repaired-engine-results-18331.json) and PR #182 source audit. Verify costs, entry/exit chronology, changed-target attribution, missing peers and denominators. Treat supplied hard-coded positives as invalid. Propose the smallest evidence-backed next action; distinguish software PASS from financial qualification.
