# Reconsidered review: evidence supports research implementation

№ 18,270 §§1–6 · 06-11-2026 08:17:16 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.

## §1. Revised judgement

The revised proposal deserves a corrected offline implementation. The previous response underweighted its new empirical evidence and overemphasised deployment objections even though the proposal prohibits automatic live promotion. The available fresh corpus should have been reproduced before assessing the significance of the claims.

Several accounting, provenance and documentation criticisms remain valid, but they are bounded engineering corrections, not reasons to reject the research direction. A small preliminary sample can justify an experiment without establishing a deployable edge. Research implementation, evidence qualification and trading promotion are separate decisions.

This review does not change production code, account permissions, live risk, the original 73% / 1:1.38 goal, or collection/validation boundaries.

## §2. What was independently reproduced

Main remains `64739b45a65b285595e60ae679354cc015f499fd`. The review:

- Validated all 20 fresh saved-job identities, file hashes, input hashes, side counts, tick ordering and time bounds against their collection manifest/receipts: 1,281,012 bid/ask ticks.
- Reconstructed quotes using the existing diagnostic code, with spread, six-second side freshness, causal features and strictly subsequent entry quotes unchanged.
- Computed 240 fresh diagnostic rows. Result: 222 observed outcomes and 18 missing predictors. The latter are the two peer directions at three horizons for AUS200, JPN225 and NatGas; they are not missing trade quotes.
- Confirmed usable decision, next-entry and 30/60/180-minute closing quotes for every fresh symbol at the specified checkpoints. This does not assert uninterrupted quotes throughout each four-hour window.
- Re-read all 40 original development source files, verified their frozen file hashes and reproduced all 480 original diagnostic rows exactly.
- Calculated fresh and pooled outcomes separately, preserving symbol-level results and matched controls. Costs use executable bid/ask once and the original per-side price-normalised surcharge. The 2bp and 3.74bp cases are assumptions, not calibrated fees.

This is reproduction of fixed-horizon diagnostics, not the programme's complete SL/TP/trailing/sizing replay. No new broker collection or paid model API was used.

## §3. Results that change the assessment

The following are **5 November only**, separate from development results. Basis-point means are returns per instrument exposure, not account profit percentages. Actual commission/funding remain outside these sensitivity calculations.

| Candidate | Before additional costs: wins / observations | After hypothetical 2bp/side | After 24bp/side stress | Mean after 24bp/side |
|---|---:|---:|---:|---:|
| Brent/Crude peer continuation, 180m | 2/2 | 2/2 | 2/2 | +34.35bps |
| Metals unanimous peer reversal, 30m | 2/2 | 2/2 | 0/2 | −33.65bps |
| Metals unanimous peer reversal, 180m | 2/2 | 2/2 | 0/2 | −20.32bps |
| Seven indices own momentum, 180m | 7/7 | 6/7 | 4/7 | +32.17bps |
| BTC/ETH peer continuation, 180m | 1/2 | 0/2 | 0/2 | −47.41bps |
| Six FX pairs peer reversal, 180m | 0/6 | 0/6 | 0/6 | −57.42bps |
| NatGas own momentum, 180m | 0/1 | 0/1 | 0/1 | −144.53bps |

**Oil is a credible research lead.** Across the three dates its peer rule has five observed trades, all positive at 2bp/side and four positive at 24bp/side. The source sample still includes a flat-signal no-entry. The fresh mean before additional costs is +82.55bps, closely reproducing the supplied +82.54bps. However, own momentum generates the identical two fresh trades. On the original data, the two rules' observed cohorts differ, so comparing their unpaired aggregate means cannot demonstrate peer value. Keep matched controls; do not add a peer architecture merely because its label sounds more sophisticated.

**Metals gains real supporting evidence.** The four pooled unanimous-peer observations are reproducible, rather than merely an unverified claim. At 30m, both symbols have two positive pooled results under the 2bp scenario. That resolves the literal two-observation count for the pooled dataset, although it does not retroactively turn the original development sample into a four-trade sample. A separate-date check is useful even when too small for promotion. The additional 180m selection must retain its own trial history rather than being described as the same previously selected 30m rule.

**Indices show a useful directional result.** Seven fresh gross-positive outcomes are reproduced; always-long wins only two of the same seven cases. The result is not simply a uniformly rising-market return. Nevertheless, the seven indices share market factors, so they are not seven independent regimes. At 2bp/side the fresh result is 6/7, not 7/7. UK100 and other constituent weaknesses remain visible in pooled output.

**Crypto improves direction without clearing costs on the fresh date.** Both midpoint directions were correct, but only one outcome was executable-price positive and neither survives 2bp/side. It is reasonable to retain the directional hypothesis; describing its financial edge as confirmed is unsupported.

**The revised FX conclusion is substantively better.** Reversal loses all six fresh cases, independently reproduced. That is more relevant than singling out one earlier GBPUSD loss as if it disproved all FX research. Continuation wins all six before extra costs, but has negative pooled expectancy at 2bp/side. This is evidence against a universal direction rule, not authority to flip the production strategy after one day.

Some supplied figures differ slightly from reproduction because the exact entry and cost convention matters. For example, fresh oil stress mean is +34.35bps rather than +34.26bps. NatGas's 180m executable outcome is −96.73bps rather than −92.97bps. These discrepancies require traceable calculation conventions; they do not reverse the broad conclusions.

## §4. Corrections to the earlier review

1. **Do not demand deployment-grade proof before building a research experiment.** The revised document explicitly separates promotion. The appropriate response is to implement the diagnostic hypotheses and repair their evidence gates, not wait for a proven edge to exist first.
2. **Do not treat the existing 24bp surcharge as measured universal cost.** The production source says it was inherited across classes from older conservative assumptions. Keeping it as stress is sensible; failing that stress alone does not establish absence of a realistic-cost edge. Equally, twelve selected demo-entry measurements cannot certify a 2bp round trip model for every asset/account.
3. **Distinguish document inaccuracies from algorithm defects.** Completed checkboxes remain inaccurate relative to main, but implementing those changes and updating the status is straightforward. Calling the history lookup N+1 is imprecise, not a reason to reject the Kelly optimisation. Its history telemetry must remain honest.
4. **The hashes are real and name different stages.** Repair their mapping; do not imply invented data or corrupted provenance. Restoring original request quantities is required for source reuse, while a separately declared alternative quantity would constitute a different replay.
5. **Respect the revised validation intent.** Accumulating adequate observations across weeks is an improvement over the earlier impossible three-date schedule. The executable plan still needs synchronising, but that is a plan-edit task, not an unresolvable research barrier.
6. **New-date evidence is useful even without final statistical independence.** A rule fixed before examining that date can receive a meaningful separate-date check. Pooled development and check results cannot be presented as an entirely untouched validation sample. Rules/horizons selected after inspecting November are exploratory amendments. These distinctions should be recorded without dismissing the new evidence.

## §5. Objections that remain, narrowed to their actual scope

**Realised accounting:** A helper subtracting a supplied cost is mathematically legitimate when that cost is an actual additional charge in the same units. The proposed wiring passes planned C, which includes spread and slippage already embodied in executed entry/exit prices. It therefore cannot be labelled realised net R. It can instead be a separately labelled cost-stress result. Existing official closed-trade performance reads broker-qualified monetary outcomes through `trading::closed_trades` → `outcomes::read`; `banks_a_win` in the broker-exit detail is not that authoritative calculation. Repair the ambiguous detail without replacing the established accounting path.

**Zero spread:** Existing `cost_context.quotes()` rejects `bid > ask`, so equality is already accepted. Its fresh EURUSD decision quote is two-sided and zero-spread. No code change is justified in that function; any alternate validator's defect needs to be identified specifically.

**DST:** The fresh lack of same-start AUS200/JPN225 peers is reproduced. It is an expected limitation of the frozen same-window factor, not proof that valid market quotes are absent. Own momentum remains evaluable. A different peer window would be a new causal hypothesis.

**Outcome labels:** “Promising separate-date evidence” is supported for oil and selected metal/index hypotheses. “Confirmed trading edge” remains too strong. 720 diagnostic rows reuse 60 source episodes across several rules/horizons; they are not 720 independent trades. Historical drawdowns, stops, partial fills, sizing and actual fees are not tested by these fixed-horizon results.

## §6. Revised course of action

Proceed with a bounded **offline research implementation**: provenance and missing-data gates; separate fresh/development summaries; configurable, clearly labelled cost sensitivities; matched own/peer controls; and reproducible candidate routing. Prioritise oil momentum/peer comparison and the existing 30m metals hypothesis. Keep index horizon alternatives explicitly exploratory. Keep FX reversal and NatGas separate from the oil success; keep crypto's directional improvement distinct from profitability.

Use a small set of meaningful tests for lineage, missing-versus-flat decisions, cost accounting and reproducing saved outcomes. Apply Kelly optimisation with a focused decision-parity/telemetry check. Correct the planned-C accounting wiring instead of accepting it verbatim. Preserve current production policy while research is built; research-code implementation does not require proof that the eventual strategy already wins.

Do not reduce a cost assumption or discard a losing constituent just to reach 50%. Freeze subsequent candidates and collection windows before further evidence is inspected. Continue to the required multi-week/full-strategy checks without pretending that this review completed them.

Companions: `reconsider.py` contains the reproducible read-only calculation; `fresh-reconsideration.json` contains provenance, all 240 fresh diagnostic rows, exact reproduction receipt for the 480 old rows, per-symbol summaries and controls. No production changes, PR or deployment were made during this review.
