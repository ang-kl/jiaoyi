# Predictive handoff: verified review and corrected implementation sequence

№ 18,268 §§1–7 · 06-11-2026 07:38:49 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.

## §1. Outcome

GitHub #147 was an issue, not a pull request. It is now closed as superseded/not planned, at 06-11-2026 07:34:31 SGT. Historical jobs 258–261/277 remain unverified; closure does not mark those jobs complete or cancel anything on Railway.

Issue: https://github.com/ang-kl/jiaoyi/issues/147
Closure: https://github.com/ang-kl/jiaoyi/issues/147#issuecomment-6672326574

All five uploaded files were read against main commit `64739b4`. The uploaded Rust file was compared with the production source. This review makes no production-code, broker-order, account-permission, worker-policy or deployment changes. No new PR was opened. The handoff contains useful research hypotheses and genuine defects worth correcting, but cannot be applied unchanged or described as already implemented and verified.

## §2. Provenance: distinguish three different hashes

| Evidence | Verified SHA-256 |
|---|---|
| Original collection plan and development collection receipt | `8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaaefda608f597e88ac` |
| Subsequent cost-context analysis plan | `300eb7750a7637a4f1682b17945aea0d035baf3b6508449b917e52763319d234` |
| Cost-context analysis code, recorded in source result | `dd4bae3893a0c23175590f5f397bee074461eb247e0a678fd086a667b8a68eed` |
| Newly uploaded class analysis plan | `0b0fa0fa5cc11952767e419003cbf63c0c4f07b267db77759d27007fd4dbad8c` |

The supplied hashes are genuine. The error is assigning the cost-context plan hash to the collection receipt. Do not overwrite the original receipt to make this comparison pass. Preserve both analysis and collection identities explicitly.

The new plan also changes native volume for AUS200, GER40, NAS100, UK100 and US30 from 10 to 100, and JPN225 from 1000 to 100. Removing `entry_rule` from a reuse comparison does not authorise these quantity changes. Fixed-horizon price diagnostics and volume-dependent strategy replay are distinct experiments.

The Markdown filename actually uploaded uses underscores: `docs/predictive_optimization_plan_2026_11_06.md`. The prompt references a different, hyphenated filename.

## §3. Production patch assessment

1. **Do not subtract planned C from executed fills and label the result realised net R.** The current cost reserve includes spread, commission and both-side slippage (`jiaoyi-core/src/strategy3/mod.rs`). Executed entry/exit prices already embody spread and realised price slippage. Deducting the full reserve again double-counts those components. Actual fees, financing and conversion require attributable broker evidence. Preserve gross fill-movement R; keep any additional stress scenario separately named; missing fees are unknown, not zero. A gross-positive fill is not proof of a qualified net win.
2. **Kelly lookup optimisation is plausible but needs accurate telemetry.** Fewer than 20 eligible entry keys cannot supply 20 distinct complete trades. Short-circuiting the expensive history query can preserve the gate decision, provided the key/trade identity relationship is verified. However, an empty synthetic sample would report zero closes even when some exist. Mark history as not evaluated/below threshold rather than asserting an observed count of zero. HashSet membership is a straightforward optimisation. The present pattern is one heavy query per assessment, not demonstrated N+1 SQL.
3. **Peer coherence belongs in the offline diagnostic path actually using it.** Cost-context evaluation currently runs in Python; the named Rust cost-context binary is not the existing implementation. An unused production Rust helper does not implement the new research rules. Check overflow as well as finite inputs when summing peer values.
4. The plan's checked boxes do not match main: `verify_receipt`, `DATA_GAP_REASONS`, `close_result_net`, the Kelly short-circuit and LCB selection are not present there. `docs/strategy3-2.rs` is a proposed source copy, not deployed code.

## §4. Recalculated research findings

Independent calculation used the existing 480 saved diagnostic rows, not new market downloads. Each reduced-cost scenario uses the original executable-price cost normalisation: `gross - (gross - stress24) * scenario_bps / 24`. The uploaded calculation instead subtracts a flat four basis points for the 2bp-per-side scenario. This small numerical difference does not rescue the qualification failures.

| Class / proposed routing | Observed trades | Wins at hypothetical 2bp/side | Mean scenario bps | Finding |
|---|---:|---:|---:|---|
| Crypto peer continuation, 180m | 4 | 3/4 | +43.1865 | Preliminary development sensitivity only; two dates, no qualified costs or unseen validation. |
| Energy peer/own fallback, 180m | 5 | 5/5 | +39.2992 | Missing decision quote; reproducing five trades requires fallback on a **missing** predictor as well as a flat one. |
| Indices own momentum, 60m | 11 | 6/11 | +8.7040 | Data gaps and losing constituents; not blocked solely by missing quotes. |
| FX unanimous peer reversal, 180m | 6 | 5/6 | +9.0758 | GBPUSD is 0/1 with −2.5880bps. Violates per-symbol gate despite positive aggregate. |
| Metals unanimous peer reversal, 30m | 2 | 2/2 | +8.2304 | Below the plan's minimum four class closes. |

These are fixed-horizon price diagnostics, not full SL/TP/trailing replay or evidence of a live edge. No class has a qualified financial PASS. Crypto's preliminary sensitivity does not establish the original 73% / 1:1.38 goal.

Cost evidence covers 12 attributable demo entry dispatch measurements from 1,079 unique order-view rows. It does not establish a paired entry/exit distribution, actual fees, live-account costs or all-symbol costs. The original result explicitly sets `calibrated_per_side_bps` to null. Label 2.0 and 3.74bp as hypothetical sensitivities, retaining the original 24bp stress. The uploaded 3.74bp column actually uses approximately 3.738317757bp, so record exact assumptions.

The original 480 rows contain 417 observed outcomes, 54 missing-data rows and nine flat-predictor skips. Calling all 63 non-observed rows missing data conflicts with the proposed distinction between genuine missing evidence and valid no-signal decisions. These are repeated diagnostic rows across variants/horizons, not 54 independent missing market episodes.

## §5. Rule and validation contradictions

- The prompt permits energy fallback when the peer signal is flat; the plan also permits fallback when unavailable. NatGas has no peer in the current economic mapping. Treat any own-signal fallback for it as an explicit separate hypothesis; do not silently erase missing-predictor evidence.
- JPN225's 180-minute close is measured after the hour-one decision: 09:00 JST + 60 + 180 minutes = 13:00 JST. The claimed 11:30–12:30 lunch-break explanation does not match that timestamp. Investigate the missing fresh quote rather than implementing the claimed session fix.
- Moving an entry window or widening close-quote freshness changes the experiment. Register such changes before looking at new outcomes; preserve the failed original result. A prior quote is not evidence of an executable future fill.
- Only three validation dates are listed. With one entry per symbol per date, five closes per symbol are impossible; two-symbol crypto/metals can generate at most six class trades, also below the 20-class minimum. Design adequate windows without lowering the gates.
- The proposed normal-approximation LCB is a descriptive ranking heuristic, not a trustworthy 90% confidence guarantee for tiny, correlated, adaptively selected samples. With n=1 it returns the mean and does not itself prevent single-trade selection. Minimum evidence gates remain essential.
- Existing `class_prediction_study` candidates and cost-context fixed-horizon rules have different schemas. Pointing the new rule names at the old binary does not implement them. Preserve original native-source verification before any new Python diagnostic transformation.
- The plan's volatility-to-cost scaling rule is not evidenced by the supplied evaluation. Do not list it as tested.

## §6. Corrected implementation sequence and acceptance checks

1. **Repair the handoff first:** retain the uploaded proposals, correct completed/checkoff claims, reference the correct filenames, split collection/analysis provenance and restore source request quantities for reuse. Record cost sensitivities as assumptions.
2. **Implement evidence gates:** detect missing-data statuses/marks and zero-sided streams before skipping `trade: null`. Cover the reason strings actually emitted by the Rust replay, including `no_next_fresh_fill_quote_before_deadline`. Preserve censoring, failures and valid no-signal skips separately. Keep all class and constituent gates.
3. **Implement reproducible offline routing in the existing diagnostic engine:** retain control arms, expose strict flat-only energy fallback separately from any explicit unavailable-peer hypothesis, evaluate FX/metals unanimity causally, use consistent executable-price cost arithmetic and exact per-symbol denominators. Keep full-strategy replay distinct.
4. **Handle production accounting separately:** distinguish gross fill movement, cost-stress estimates and broker-qualified net outcomes. Apply Kelly optimisation only with proven gate parity and honest history-status fields. Do not alter the live rule hash, size caps, permissions or goals.
5. **Run six focused acceptance checks:** (a) reject wrong collection receipts and altered request quantity/cost; (b) block a no-trade missing-data row while retaining legitimate no-signal skips; (c) block the GBPUSD and two-trade metals false PASS cases; (d) preserve missing-versus-flat fallback semantics and causal timestamps; (e) prevent planned-cost double charging/unknown-cost wins and preserve Kelly admission decisions; (f) reproduce all original 40 source identities and account for every diagnostic row. Run required fmt, Clippy, workspace tests and Python checks for the eventual implementation PR.
6. **Only then freeze and validate:** choose a sufficiently sized, separately registered validation schedule with declared prior exposure. Do not open validation to repair development gaps or optimise against its results. Merge an implementation only on green checks for the exact reviewed commit. Financial promotion requires its separate evidence gate.

## §7. What was and was not done in this review

Completed: issue closure verified through GitHub; five files read; production diff inspected; collection/analysis hashes distinguished; supplied class selections independently recalculated from saved development results; per-symbol and sample-size contradictions identified; corrected sequence recorded.

Not performed: source-code patching, compilation, a new PR, deployment, validation collection, broker orders, paid model calls or risk-policy changes. This review must not be cited as proof that the uploaded implementation exists or that a strategy is deployable.

Companion evidence: `handoff-verification.json`. Goals remain 73% net wins and risk:reward 1:1.38 per strategy/account; interim research target 50% per class and constituent. Time is Money: correct provenance and acceptance criteria before spending further runs.
