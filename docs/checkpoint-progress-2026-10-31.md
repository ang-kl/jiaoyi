# Checkpoint follow-through: broker lifecycle evidence and stop-direction protection

**Record:** CHECKPOINT-20261031-A. **Prepared:** 31 October 2026, 09:47 SGT.
**Authority:** owner “Continue with proposed”, acknowledged №17,972 §1; progress №17,973–17,975 §§1–2. By <Codex (GPT-6), effort: not exposed>.

The owner-approved inquiry follow-through has twelve acceptance checkpoints across four phases. They constrain evidence, research progression and supported claims; they do not create a new strategy pause, risk setting or worker mutation window. The [research plan](research-validation-plan-2026-10-31.md), [inquiry findings](inquiry-findings-2026-10-31.md) and current owner contract remain the base.

## §CP.1. Reproduced execution defect and narrow correction

The existing shared stop-amendment gate refreshes the broker position's SL/TP but previously evaluated favourability using the caller's side without checking the fresh broker direction. A broker-free integration fixture reproduced an invalid acceptance: broker BUY, SL90, TP120; stale SELL command requesting SL85. The gate sent the change even though it widened protection for the actual BUY. The reciprocal SELL/BUY case has the same issue.

The gate now refuses an opposite or unrecognised broker direction before recording an amendment intent or writing an amendment frame. Matching directions still pass the existing tightening checks and preserve broker TP. This corrects an existing protection invariant; it changes no strategy geometry, original-risk definition, sizing, margin floor, owner pause policy or external-position adoption.

Three new regression functions cover both direction mismatches, unknown broker direction, and valid BUY/SELL tightening with TP preservation. All 37 `jiaoyi-ctrader` trading-gate integration tests pass on the actual worktree. Formatting and diff checks pass. Independent review found no blocking issue. Full `check`, `docker` and `engineering` CI, reviewed-head merge and deployed-source/protection readback remain release gates; this record does not anticipate them.

The test uses the existing in-memory broker, not a forced live trade. It proves the supplied state can violate protection in the earlier implementation; it does not assign a measured historical loss to that defect. A broker state change after reconciliation is outside the new guard's atomicity claim.

## §CP.2. Source evidence acquired, with claim boundaries

Read-only broker JSON WebSocket requests through the configured HTTPS proxy successfully returned current reconciliations and trader profiles for all four permitted accounts at 09:41 SGT. No order, amendment, close, cancellation or token refresh was sent. Source type, request time, response hash and bounded records are retained privately.

The previously unresolved NatGas case now has a complete returned six-deal page under one position ID. It establishes the original short, a reversal whose filled quantity both closes the short and opens a long residual, three further long increases and a final close. Per-side quantities and closing details reconcile; this is stronger than merely obtaining a net signed sum of zero. Initiating-client/intent provenance, actual management amendments and financial causation remain separate questions. A retained strategy label does not establish the originator of every later order.

The current broker also confirms the pre-Jiaoyi demo SpotCrude pending order still lacks an SL. Its owner disposition has been requested; this repair neither adopts nor cancels it. Existing broker-held stops remain in place. Public health/source freshness does not replace this order-level protection readback.

Funding-scope residuals were narrowed using the overlapping statements. The live residual already exists in earlier exports and is not a new recent trading loss; the other identified difference precedes the first represented close. Bounded broker funding/order/history reads are continuing. Missing historical cutoff valuations do not prevent reporting reconciled realised cash, but they prevent inventing equity returns. Private cash amounts and raw broker payloads remain outside Git.

## §CP.3. Concrete economic study object

Select the unchanged registered S3v6 composition as **S3V6-CURRENT-FALSIFICATION-20261031**, a candidate to test, not a profitable replacement or permission for additional risk. Its hypothesis is that a first eligible half-hour band breakout contains within-session continuation information sufficient for positive executable net returns under the entire existing exit, capacity and cost policy. Delayed information incorporation/order flow are possible mechanisms, not established causes in these accounts.

Freeze the implementation, registration and inherited exits; exact universe/symbol mappings; session clocks and decision windows; effective settings; initial Kelly, portfolio and veto/mark state; point-in-time inputs; broker units; and executable cost assumptions. This protection repair requires its own exact deployed build binding before a scored start. An opening hash alone does not identify all later management semantics.

The primary economic claim is absolute after-cost performance; the owner latest-20 goal and any comparison with bot-trade are separate claims. Direction controls must replay side-dependent prices, costs and path-dependent exits. An isolated same-time direction test and a capacity-constrained portfolio control answer different questions: overlapping control positions, asymmetric admission and adaptive state must be resolved before sealing. Do not negate final P&L or drop infeasible controls while calling them same-count.

The existing H1 controls and fixed-entry exit diagnostic do not certify a complete current-S3 evaluator. Aggregate fill/cost summaries lack sufficient registration/lifecycle dimensions to establish v6 calibration. Historical evaluation needs contemporaneous commission/spread evidence rather than today's unrestricted calibration sample. No sealed holdout, new model experiment or parameter search was run.

## §CP.4. Checkpoint status and next useful work

| Check | Status as of preparation | Completion requirement |
|---|---|---|
| C01 source/cash | Partial; exact statement checks passed | Resolve required funding scope and cutoff valuations for affected return claims |
| C02 lifecycle/original risk | Six-deal NatGas sequence confirmed; overall partial | Attribute subsequent commands; certify original risk and selected full position lifecycles |
| C03 current exposure | Current broker snapshot acquired; exception confirmed | Record owner disposition of the external pending order; preserve managed protection |
| C04 economic mechanism | Concrete falsifiable specification ready for challenge | Independent challenge; readiness is not proof of edge |
| C05 fixed policy/comparator | Draft manifest with source fingerprints | Bind effective settings/adaptive state and post-repair deployed build |
| C06 executable costs | Partial; exact assumptions and gaps documented | Point-in-time costs, feasible fills, gaps and contract constraints |
| C07 engineering proof | Reproduced defect, narrow repair, 37/37 focused tests and independent review | All required CI and actual release readback |
| C08 independent evaluator | Not certified | Valid side-dependent controls, capacity, censoring and dependence handling |
| C09 prospective protocol | Unsealed | Predeclare population, cost model, effect/precision, duration, controls, stopping and follow-up |
| C10 fixed-interval observation | No sealed experiment started | Observe full opportunity stream and disclose version/incident changes |
| C11 financial verdict | Not established | Evaluate all declared criteria at the fixed end; retain failed/insufficient outcomes |
| C12 promotion/maintenance | No financial promotion | Claim-specific verdict and authorised continuation; engineering release is distinct |

Reuse adequate feasibility evidence; require an unscored pilot only where needed. S3's registered criteria, the proposed 200-episode interpretation, sufficient week/dependence coverage and the owner goal remain distinct. Do not start scoring before sealing or extend the same experiment until it passes. Current authorised trading continues under its existing rules; no new service, paid API route, worker allowance/window or live writer is created.

Private evidence is under `/workspace/jiaoyi-checkpoint-evidence-20261031/`: broker captures and manifests, ledger scoped checks, economic candidate specification/manifest, exact failing/passing gate fixtures and independent reviews. Raw owner data belong in the private report, not this public continuation record. Release receipts are appended to the PR description when verified.
