# TL;DR — Five bounded repairs; conditional replay, no financial qualification

№18,407 §§1–10 · Generated 09-11-2026 18:00:10 SGT (+08:00) · by Codex (runtime model/effort identity not exposed).

All five requested production repairs are implemented. The original 15 recorded demo positions lost $788.68. A conditional replay of those same original arms, under the revised rules and disclosed execution assumptions, retains two entries per account: ETHUSD wins and SpotCrude loses. SpotBrent is pair-blocked; both index setups are outside the permitted cash session. Conditional combined net cash is +20.50 USD after the modelled 3.74 bp/side reserve and recorded commission. This is not realised money or an independent backtest.

Financial PASS: false. Each account has only 1/2 modelled wins (50%), not a latest-20-trade qualifying sample. The 73% net-win / 1:1.38 project goal remains unproved. Three accounts replay the same market moves; their six closes are not six independent market experiments.

Release status at report generation: required CI, merge and Railway verification are pending. This report records implementation and focused local verification. The final PR/release evidence must identify the merged SHA, check conclusions, deployed SHA/policy and all four connected/unpaused account loops; those outcomes are not assumed here.


## §1. What failed, and exactly what changed

| Repair | Observed defect | Implemented behaviour and boundary |
| --- | --- | --- |
| 1 · Pair reservation | Both crude benchmarks and both US indices could consume risk in the same account. | One occupied leg per account across SpotCrude/SpotBrent, US500/NAS100, XAUUSD/XAGUSD and BTCUSD/ETHUSD. Broker positions, pending orders and local consuming/uncertain intents reserve the pair. A sibling arm is durably disarmed; closure does not revive stale setup history. |
| 2 · Causal extreme | The first qualifying extreme remained frozen even when the impulse extended; NAS100 later entered after a 78.51% reversal of its observed decline. | Every fresh extension updates the extreme, midpoint and range and invalidates previous preparation/touch evidence. A retracement greater than 61.8% terminates the arm. Persisted state and tombstones prevent restart revival. |
| 3 · Touch, then turn + peer hold | The old midpoint touch could sell directly into a rebound; a peer merely on the correct side of zero was sufficient. | First touch/deepest midpoint must be within 50–61.8%; a strictly later fresh quote must recover at least 8% of the impulse in its original direction. Confirmation and writer quotes must remain within 40–61.8%. The final writer requires a fresh same-direction peer move of at least 24 bp and a matching causal proof. |
| 4 · Cash-session restriction | London-session US500/NAS100 setups traded compressed overnight conditions. | New US index entries only 09:30 inclusive–16:00 exclusive America/New_York, DST aware; the writer deadline is capped at the cash close. Oil/metals retain 07:00 UTC London-labelled and 09:30 New York windows; crypto retains six UTC four-hour windows. Existing recorded exit deadlines remain intact. |
| 5 · Demo broker-minimum risk | A broker minimum admitted NAS100 at 1.2496% risk on the small demo account. | The broker minimum must fit the ordinary configured risk budget, currently 0.6% of admission equity. No 0.75% exception was added. If it cannot fit, refuse with min_volume_exceeds_trade_risk_budget; do not raise risk to force activity. |

New entry policy identity: `s3_macro_pullback_owner_18407_v1`. Existing policy 18392 positions retain their recorded two-stage exit treatment. Label ownership, yield-to-human positions/orders and the NatGas/JPN225 human reservation remain compiled protections. A recovery of 8% from the deepest observed touch does not guarantee that the next price will continue in the trade direction.

The final admission boundary uses the exact prepared broker-book snapshot, account revision, connection generation and local consuming-entry epoch. It rechecks after obtaining the socket writer lock. This prevents stale local admission and concurrent local pair sends; it cannot make an external broker/human action atomic with a network send. An uncertain send remains reserved until explicit terminal/full-fill evidence, not merely elapsed time or a flat snapshot.


## §2. Three-account actual versus conditional cash result

| Demo account | Actual original net USD | Model gross USD | Model cost deduction USD | Conditional model net USD | Model net wins | Financial PASS |
| --- | --- | --- | --- | --- | --- | --- |
| 46130058 | -386.20 | +52.22 | +43.01 | +9.21 | 1/2 (50%) | false |
| 47790949 | -386.47 | +55.25 | +44.20 | +11.05 | 1/2 (50%) | false |
| 49236051 | -16.01 | +1.71 | +1.48 | +0.24 | 1/2 (50%) | false |

Combined: actual -788.68 USD; conditional model +20.50 USD. Arithmetic difference +809.18 USD is a retrospective scenario difference, not an achieved saving. All six hypothetical entries resolve within captured coverage; no open trade is assigned a convenient exit. The live account 43002148 has no position in this 15-trade replay and is not included in these cash totals.

The cash comparison is deliberately asymmetric in evidence strength: actual totals come from broker-reconciled closed positions; hypothetical totals use recorded bid/ask plus synthetic fill timing and a slippage reserve. Modelled cash must not be posted to the real account ledger. No claim of a calibrated full round-trip cost model is made.


## §3. Account 46130058 — all five original setups

| Symbol / original broker position | Actual entry → exit | Actual net USD | Revised original-arm outcome | Conditional net USD |
| --- | --- | --- | --- | --- |
| ETHUSD / 260303252 | 2132.21 → 2116.3 | +200.47 | target_1_38r | +178.17 |
| SpotCrude / 260368257 | 90.103 → 90.498 | -154.45 | initial_stop_loss | -168.96 |
| SpotBrent / 260368313 | 99.59 → 100.076 | -144.34 | pair_risk_reservation_active | — |
| US500 / 260383624 | 7380.2 → 7408.1 | -145.08 | outside_authorized_entry_session_18407 | — |
| NAS100 / 260389631 | 26880 → 26982 | -142.80 | outside_authorized_entry_session_18407 | — |

All times in the next tables are 09 November 2026 SGT. Raw JSON retains UTC with original precision. All five historical trades and the two retained hypothetical entries are shorts.

| Symbol | Actual broker entry / exit SGT | Revised entry / exit SGT | Why the outcome differs |
| --- | --- | --- | --- |
| ETHUSD | 13:30:26.819 / 14:31:52.747 | 13:30:36.770 / 14:32:35.104 | Touch is observed first; a later turn and 37.03 bp peer hold permit entry. The target remains attainable on the captured path. |
| SpotCrude | 15:33:22.525 / 15:58:20.598 | 15:32:43.363 / 15:58:13.189 | Updated lower extreme and a valid 8% turn admit the trade with 46.55 bp peer hold; the market still reverses and hits the stop. The repairs do not remove all losses. |
| SpotBrent | 15:33:32.346 / 16:02:04.751 | No entry; pair disarm 15:32:43.363 | Crude already consumes the energy pair; the original Brent arm is discarded. |
| US500 | 16:05:24.445 / 16:43:24.003 | No entry; session refusal 15:46:24.197 | Original London arm is outside New York cash hours. No replacement NY arm is invented in this replay. |
| NAS100 | 16:20:10.483 / 16:43:21.424 | No entry; session refusal 15:36:12.720 | Session refusal occurs before any touch/turn decision. This run does not attribute the block separately to the deep-retracement guard. |

| Symbol | Origin | Updated extreme | First touch SGT / midpoint | Deepest touch SGT / midpoint | Confirm midpoint | Peer hold bp | Entry bid | Initial SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ETHUSD | 2138.72 | 2127.39 | 13:30:26.367 / 2133.24 | 13:30:33.645 / 2133.53 | 2132.03 | 37.0307 | 2131.03 | 2142.43 | 2115.29 |
| SpotCrude | 90.326 | 89.821 | 15:32:37.173 / 90.076 | 15:32:37.173 / 90.076 | 90.026 | 46.5512 | 90.007 | 90.435 | 89.416 |

ETHUSD: first touch 13:30:26.368; deepest touch 2133.53 at 13:30:33.646; confirmation midpoint 2132.03 at 13:30:36.770. The 1.50 recovery exceeds 8% × 11.33 = 0.9064. Modelled breakeven becomes effective on the next quote at 14:19:29.966; target exit is 14:32:35.105. SpotCrude: running low updates to 89.821 at 15:32:16.265, midpoint reference becomes 90.0735; touch 90.076 at 15:32:37.173, confirmation midpoint 90.026 at 15:32:43.363. The 0.050 turn exceeds 8% × 0.505 = 0.0404. Its valid entry nevertheless loses.

| Symbol | Equity proxy USD | Broker volume hundredths | Initial risk/price unit | Gross cash USD | Cost reserve + commission USD | Conditional net USD |
| --- | --- | --- | --- | --- | --- | --- |
| ETHUSD | 23937.59 | 1259 | 11.4 | +198.17 | +19.99 | +178.17 |
| SpotCrude | 24345.06 | 34100 | 0.428 | -145.95 | +23.01 | -168.96 |

Volume fields are cTrader protocol hundredths, not lots; do not read 34,100 as 34,100 lots. Geometry is rounded to the captured price grid and bounded admission rules. Cost deduction is entry/exit notional × 0.000374 each side plus the recorded round-trip commission component. Executable bid/ask already embeds spread, so spread is not deducted again. The replay uses original actual-world admission equity as a sizing proxy and does not reconstruct alternative equity, margin or manual-account trajectories.

Diagnostic attribution: with only the pair guard removed, Brent extends to 99.148, touches 99.519 at 15:33:19.922 and is invalidated at 15:33:32.448 when midpoint 99.609 exceeds 61.8% retracement. It never enters; all three account cash results remain identical. This diagnostic is not a competing strategy selected for better performance. Of the four historical losing setups per account, three do not enter under the revised original-arm replay; SpotCrude still loses.


## §4. Causal replay method and all eleven assumptions

Conditional replay of the original 15 recorded arms; no new arms after disarm, no full-day or independent backtest claim.

Source cutoff: 2026-11-09T09:30:00Z (17:30 SGT). New broker collection requests: 0. Read-only SQL extracts contain 616,910 raw rows in 24 hashed stream receipts. After exact-row deduplication, pair-enabled requests contain 616,735 quotes across the three accounts. Original summary path statistics alone cannot establish chronology; this run uses the underlying recorded quote streams.

A1. Original arm, causal macro context and peer opening reference are taken from actual audit receipts.

A2. Running-extreme/touch-turn, deep invalidation, session eligibility and brackets use shared native production helpers.

A3. All symbols of each account share one chronological quote timeline; cross-symbol same-time ties order by numeric symbol ID, never by outcome. One distinct-price BTC peer timestamp on 47790949 is replayed in both possible within-symbol orders; all full outputs must agree or the run refuses.

A4. Pair occupancy disarms the sibling original arm; closure releases occupancy but this original-arm study does not generate another arm.

A5. Prepared geometry is available from the immediately preceding fresh quote; extensions invalidate previous geometry; five-second preparation age is enforced. Actual background I/O readiness is not reconstructed.

A6. Hypothetical entries/stops fill at observed executable bid/ask; targets fill at their limits; BE becomes effective on the next quote. No actual counterfactual broker fills or transport latency are claimed.

A7. Net sensitivity subtracts recorded round-trip commission and 3.74 bp per-side slippage reserve from executable-quote cash P&L; spread is already in bid/ask and is not charged twice.

A8. Sizing is recomputed at 0.6% using each original actual-world admission equity as a proxy, not a counterfactual equity trajectory. Broker volume limits are the captured per-account original limits.

A9. S3 3% open-risk cap and pair exclusion are simulated for the supplied cohort; other historical account/manual books, margin admission and pending broker acknowledgements are not reconstructed.

A10. No post-capture exit is invented. Open hypothetical trades are censored and total counterfactual net P&L is null until resolved.

A11. This day's data motivated the repairs; improved retrospective results cannot qualify an independent trading edge.

Same-timestamp sensitivity is checked by rerunning both possible orderings of the single distinct-price within-symbol tie. Full output equality is required, not merely matching final cash. Cross-symbol ties use numeric symbol ID, fixed independently of results. Failed hash or row-count reconciliation refuses the run. Running extremes, confirmation, session eligibility and bracket geometry use shared production Rust helpers; the orchestration does not create substitute trading rules.


## §5. Verification evidence and findings corrected during review

| Focused suite | Result | Material boundary covered |
| --- | --- | --- |
| Strategy 3 account integration | 84 passed; 1 existing ignored | Entry/exit lifecycle; three human laws; sibling position/pending/uncertain-intent reservation and restart; sizing. |
| Core strategy tests | 17 passed | Long/short causal impulse and confirmation geometry; restoration/invalidation; price/risk rules. |
| cTrader trading gate integration | 50 passed | Final executable proof and peer hold, admission snapshot and consuming epoch, broker-book ownership and pair occupancy. |
| Strategy actor unit tests | 14 passed | Actor state, stored policy and bounded entry geometry. |
| Native counterfactual replay tests | 2 passed | Recorded-path lifecycle, guard and exit model. |

Additional local verification at publication: strict workspace Clippy and formatting pass; the full workspace rerun has passed all 197 jiaoyi library tests while remaining integrations are running. The research Python suite ran 115 tests: 114 passed and one existing skip. The Docker build context now explicitly includes the new embedded 18407 preregistration file; the required docker CI remains the image-build authority.

The real actor/PostgreSQL/synthetic-broker integration also passes with the new touch-then-turn sequence: 7.293 ms from confirmation to writer, one durable intent/order, no intervening admission/history request, and no order on the touch alone. This is local test latency, not a live broker latency measurement. These are focused results, not a claim that every repository test is green. Final full-suite/required CI results belong to the release receipt. The historical bounded-review fixture must retain its historical 18392 policy identity; the new policy must not be relabelled as the old midpoint-touch policy to make a test pass.


- Independent review found and addressed stale sibling arms after pair occupancy: blocked arms now retain durable invalidation, so restart/release requires a fresh impulse.
- Uncertain sends no longer release pair/risk reservations merely because a grace period elapsed and a broker snapshot is flat; explicit resolution is required.
- Index new-entry expiry is capped at the NY 16:00 cash boundary while recorded existing-position exit deadlines stay unchanged.
- Counterfactual exits now use the actual execution window; post-consumption guard/risk refusals do not retry an old original arm; open risk follows current stop distance and releases at breakeven.
- Final writer retains the mandatory causal proof, current quote freshness, actual funded price bound, cost reserve, broker calendar and peer hold. The legacy bounded variant retains its old semantics.
- Synthetic test-clock translation shifts touch/deepest/confirmation timestamps together using the explicit actor event offset. Production offset is zero; freshness and future-date checks were not relaxed.

Independent review conclusion: no unresolved production correctness defect was identified in the reviewed implementation after those corrections. This is a scoped review, not proof of universal correctness or a trading edge. The complete signed-context review is published beside this report as `FORENSIC_REPAIR_INDEPENDENT_REVIEW-18407-2026-11-09.md`.


## §6. Known limits and release acceptance


- The historical stop-loss forensic classifier does not yet classify new-policy 18407 trade admissibility from the full causal touch/turn path. It remains fail-closed/unknown for that policy. Live writer proofs and runtime guards exist; an old touch-only audit must not be substituted for them.
- Read-only causal replay is conditional on the original 15 arms. It omits new arms after invalidation, actual asynchronous preparation availability, broker acknowledgement timing, an alternative manual-order book and full counterfactual margin/equity history.
- The 3.74 bp/side reserve is an assumption; modelled target and stop fills, next-quote breakeven effectiveness and absence of transport delay are not actual broker evidence.
- Require check, docker and engineering CI conclusions to pass before merge; verify deployed source SHA and compiled policy identity after deployment.
- Confirm all four account loops connected/unpaused and existing positions still managed under their recorded policy. A deployment verification must not submit a discretionary test trade.
- Observe the next eligible sessions for arms, invalidations, turns, peer vetoes, pair occupancy, admitted risk and broker fill/SL/TP acknowledgements. Report losses and execution defects without claiming the target is met from this replay.

The nightly New York cash open on 09 November 2026 is 22:30 SGT, because New York is on standard time. The implementation uses America/New_York rather than fixing the Singapore conversion year-round. Today’s quoted 07:00 UTC London-labelled window is the retained native programme window; no separate London-DST redesign is claimed.


## §7. Source chain and reproducibility

Canonical measured result: `docs/forensic_repair_counterfactual_results_18407.json`. Exact SHA256: `ae5cb86975465eb8c0345edc35434f44120e09d8693f7b56cb01a8f5bcf13f55`. The JSON contains complete per-event timelines, original observed positions, all source receipts, six canonical request hashes, implementation hashes and the no-pair diagnostic.

Native binary SHA256: `fa751a5e2190c3767dc374c1010a69cd292c08cbe479aef2cdb75adba08f88e1`. Original forensic audit base: `9a5b7a3ec251c9aa368b3cab717f80c0660d9456`. Source/binary hashes identify this evaluated build; the PR/release receipt must separately identify the final merge/deployment SHA.

| Input audit file | SHA256 |
| --- | --- |
| evidence-refresh.json | c978f3b6721b2908144544de8102bd21fc93f6365c790249d9d5efd88f0b4f7d |
| trade-ledger.json | 2c6a656ec27cf4f911932b0755050c05ce37a74c3609898452b0d97300a1b79a |

| Evaluated source file | SHA256 |
| --- | --- |
| ops/research/forensic18407_counterfactual.py | 88f15393ff207b72dceeaf08d6445c3cfbba3388d5e0f347d5bd6437e71d4ecf |
| crates/jiaoyi-core/examples/forensic18407_replay.rs | 9985444c672c57799e5f0356957a6df93f05e811b367bb9ec5f0d929e8c28956 |
| crates/jiaoyi-core/src/strategy3/macro_pullback.rs | 9ab6cf586976363c3ffaa76a3235e3c08472096de1bcd2ab1caef839bc8433e6 |
| crates/jiaoyi-core/src/strategy3/production_sessions.rs | 6c35cc3c25599242cec2eef73438eb36f3014ac113fad09b027c2e2e079e4986 |
| crates/jiaoyi-core/src/strategy3/targets.rs | e373c96e321c9349522408249c4a545572756eb8431c10185e4948208d680c44 |
| crates/jiaoyi-core/src/strategy3/mod.rs | 8cf08123dd4b41b9fd11d62cb986fdf18aeb9121e18a9a1b44d7778b7f06c465 |
| Cargo.lock | 025a81017952f0bda10f7989938e758685e738b2a1e367b13df4944dfbfb98bf |

Reproduction requires the private captured streams and original audit receipts; public source, JSON and this report are sufficient to inspect the arithmetic and audit trail, but the private streams are not embedded in the repository. Retain their recorded hashes. No credentials are required by the offline replay itself.

```bash
cargo build -p jiaoyi-core --example forensic18407_replay --locked
python3 ops/research/forensic18407_counterfactual.py \
  --cache /workspace/jiaoyi-counterfactual-18407 \
  --audit /workspace/jiaoyi-forensic-18406 \
  --binary target/debug/examples/forensic18407_replay \
  --output /tmp/forensic18407-reproduced.json
```

Use the hashed source revision and same toolchain for binary-identical reproduction. A different build path/toolchain can change the binary hash while retaining arithmetic; compare account/event outputs and canonical request/source hashes separately. The script verifies decompressed stream SHA256 and row count before sending the request to the native adapter.


## §8. Canonical replay requests

| Account | Pair enforced | Unique quotes | Request SHA256 | Reversed within-symbol tie output |
| --- | --- | --- | --- | --- |
| 46130058 | True | 205590 | 61350b1b5a34b89a476f04d872943b0bb36f98e9f61261b960ed6fb0fdd5fdbf | identical |
| 46130058 | False | 205590 | ef0763133ac57a5b8dc39324928d0957e9ebd45e22fe81a257914772b1d6719b | identical |
| 47790949 | True | 205609 | aaf33e37ad371e6dc5a5b210deb9b2518b40f772472c4d8dc7627377d78c8d80 | identical |
| 47790949 | False | 205609 | 52a44e61c8662f3760e14516640d9ff9515aa00dbcc2fb1045b830be5a40d977 | identical |
| 49236051 | True | 205536 | 0158dff6b4d7cc3ce967ce5c55199dfd86339a9985d67050ce82e362aba75ef8 | identical |
| 49236051 | False | 205536 | 505f2afd0942c5d26d5ae9f13e407028026865e4b2b44f7f85bb8db0e1f3b04a | identical |


## §9. Every captured source receipt

| Account | Symbol | UTC window [start, exclusive end) | Raw rows | Decompressed stream SHA256 |
| --- | --- | --- | --- | --- |
| 46130058 | ETHUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 54261 | 5c21eb0a575ef8878760c40317914a1886c2cc258c3edeba2a7ceca889454db2 |
| 46130058 | ETHUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 282 | 300847cff83a314acd1352d74db4f83433cf9c6159ae17991fa6a8af8ea9f80f |
| 46130058 | BTCUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 66941 | fe90b19e0d49a7dd97dca8e0b2d3d07b1ca733e209bad6c25c9a34f7315e189e |
| 46130058 | BTCUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 310 | 23fcfa4f8b59f340316aa641ea406603b95b1a5dc118f0f5c822d4c6f83572e5 |
| 46130058 | SpotCrude | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 12798 | 9a03ab2ff960f8cb1fb1aee32028fb16aa687fa6e502780912db7ab3aa3b754e |
| 46130058 | SpotBrent | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 10615 | 8c4d555a6da872ed2bf001c4bafe9badbb0df8d0fc4442a9b75fc99ad3bb673b |
| 46130058 | US500 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 23127 | 67bb2aaa2083ab76de65707c38b704c43807b92d714e9605a9da64693ca29420 |
| 46130058 | NAS100 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 37316 | 8ee67bd5261e3aba6862d303a153db694d8f0d9f072753d703338c881daaa13b |
| 47790949 | ETHUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 54272 | c06b3bb56380d74a155a9320e955569399ce629ebfb61ba19b227837597f84b9 |
| 47790949 | ETHUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 282 | e2f881e898968dfb89244ec8fdbf4c92493dc6717340e97fefd69e4b38521339 |
| 47790949 | BTCUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 66957 | ed27dfd91e0565b8994698653eb884d6748424c6d8cdada92fd2d3913b87526e |
| 47790949 | BTCUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 311 | a072cbc22e319e8de70bf79deb99b24baa681409dd0deab657e1b16a5f64516d |
| 47790949 | SpotCrude | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 12792 | 5415db0f3901d61ac56033251a5c2423484df4cc7855f95939e028e6d99a51aa |
| 47790949 | SpotBrent | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 10610 | c2ecd0501d369eabf6533704d896b5f7cea51050a632752239f0794641d87864 |
| 47790949 | US500 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 23127 | 60268c9009e8393ab3cbf4af6c135522cacc9955f4999ccb4c43136898f05b30 |
| 47790949 | NAS100 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 37316 | 554d8ccd4340bcf1a0a9e110d2bc214c33466fad6b4b18a6ba65afda7e6411ed |
| 49236051 | ETHUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 54241 | 53f25105681a4999099a7b884b3f1d63a99703d529bd4a9c8e96df3a25e15127 |
| 49236051 | ETHUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 282 | 2c014a4aca19e84ad5c4190bee00e62f3f0f1ecf397ce8f77bbb4a2fcd43e543 |
| 49236051 | BTCUSD | 2026-11-09T04:00:00Z → 2026-11-09T08:00:00Z | 66914 | 9eee4b1cc2af4c0b9be28ad16bca1ef31b1987be1692a5f4fc43371a5290fca4 |
| 49236051 | BTCUSD | 2026-11-09T08:00:00Z → 2026-11-09T08:01:00Z | 310 | ea6fa717fe980f995ff9dcf22af6091ba97eb740469236a219e9930ed2bc9ac2 |
| 49236051 | SpotCrude | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 12792 | ad41fc24a1b97944a10787d198b11c19be4182070bbdc458c4176fe8fa3f3ee3 |
| 49236051 | SpotBrent | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 10610 | 9f3bf0c5e7179a3124b0828230c4370d712aa012b3a2256f692fed46d3caf840 |
| 49236051 | US500 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 23127 | d516dac491afd518e2263c388fbf3f762ba196991afd47d5e6d4e4be7c75f15f |
| 49236051 | NAS100 | 2026-11-09T07:00:00Z → 2026-11-09T09:30:00Z | 37317 | 4e38f3e381c667d6c61b5f91b3355c4b67afab3d128e91269a7b38147b69329c |


## §10. Review hand-off and goal status

For another AI reviewer: start with this report; inspect `docs/forensic_repair_counterfactual_results_18407.json` and the independent review; then follow the seven hashed replay/source files and the production actor/final-writer diff in the PR. Challenge causal state, stale preparation, pair race/restart handling, cost assumptions and denominator independence before interpreting the cash improvement. Verify actual CI/deployment evidence in the PR/release receipt rather than inferring it from this document.

Goal status remains financial_pass=false. The two retained modelled trades per account give one net win (50%), below the 73% project target and far short of a latest-20-trade sample. Gross target geometry of +1.38R does not imply a realised net 1:1.38 payoff after spread, slippage, commission and stop fills. No independent predictive edge or sustained improvement has been established.
