# Checkpoint follow-through: broker lifecycle evidence and stop-direction protection

**Record:** CHECKPOINT-20261031-A. **Prepared:** 31 October 2026, 09:47 SGT. **Updated:** 10:14 SGT, №17,982–17,983 §§1–2.
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

The 09:41 broker snapshot confirmed a pre-Jiaoyi demo SpotCrude pending order without an SL. The owner subsequently explicitly authorised cancelling that specific order. At 10:01:42 SGT, one scoped cancellation received ORDER_CANCELLED; the fresh detail showed CANCELLED, zero executed volume, absence from pending orders and an unchanged empty position book. No other order was cancelled, adopted or amended. The private receipt preserves the account/order identity and before/after payload fingerprints. Public health/source freshness does not replace order-level protection readback.

Subsequent broker funding reads cover 124 contiguous windows and 18 returned operations from registration for the two affected accounts. They reconcile the statement-summary versus selected-period funding differences. The earlier non-funding change precedes the represented trading period; its exact trade/strategy attribution is not inferred. All 80 fixed-cohort S3 lifecycles now have verified opening-order joins, terminal quantities and cash. Original stop geometry agrees; only 14 original monetary-risk/FX inputs are independently archived, leaving 66 unverified. Missing historical cutoff valuations still prevent certified equity returns. Private cash amounts and raw broker payloads remain outside Git.

## §CP.3. Concrete economic study object

Select the unchanged registered S3v6 composition as **S3V6-CURRENT-FALSIFICATION-20261031**, a candidate to test, not a profitable replacement or permission for additional risk. Its hypothesis is that a first eligible half-hour band breakout contains within-session continuation information sufficient for positive executable net returns under the entire existing exit, capacity and cost policy. Delayed information incorporation/order flow are possible mechanisms, not established causes in these accounts.

Freeze the implementation, registration and inherited exits; exact universe/symbol mappings; session clocks and decision windows; effective settings; initial Kelly, portfolio and veto/mark state; point-in-time inputs; broker units; and executable cost assumptions. This protection repair requires its own exact deployed build binding before a scored start. An opening hash alone does not identify all later management semantics.

The primary economic claim is absolute after-cost performance; the owner latest-20 goal and any comparison with bot-trade are separate claims. Direction controls must replay side-dependent prices, costs and path-dependent exits. An isolated same-time direction test and a capacity-constrained portfolio control answer different questions: overlapping control positions, asymmetric admission and adaptive state must be resolved before sealing. Do not negate final P&L or drop infeasible controls while calling them same-count.

The existing H1 controls and fixed-entry exit diagnostic do not certify a complete current-S3 evaluator. Aggregate fill/cost summaries lack sufficient registration/lifecycle dimensions to establish v6 calibration. Historical evaluation needs contemporaneous commission/spread evidence rather than today's unrestricted calibration sample. No sealed holdout, new model experiment or parameter search was run.

## §CP.4. Checkpoint status and next useful work

| Check | Status as of preparation | Completion requirement |
|---|---|---|
| C01 source/cash | Statement/cash/funding-scope checks passed; valuation claims partial | Historical cutoff equity/float remains required for equity-return claims |
| C02 lifecycle/original risk | NatGas sequence and 80 S3 terminal/cash joins verified; overall partial | Attribute later commands and acquire 66 missing original monetary-risk/FX inputs |
| C03 current exposure | Snapshot/protection checked; specific owner-authorised cancellation verified | Continue current-book readback at release; no general external-position adoption |
| C04 economic mechanism | Concrete falsifiable specification ready for challenge | Independent challenge; readiness is not proof of edge |
| C05 fixed policy/comparator | Draft manifest with source fingerprints | Bind effective settings/adaptive state and post-repair deployed build |
| C06 executable costs | Partial; exact assumptions and gaps documented | Point-in-time costs, feasible fills, gaps and contract constraints |
| C07 engineering proof | PR141 merged/deployed with 18 release checks; second qualification repair tested | Second repair independent review, all required CI and actual release readback |
| C08 independent evaluator | Not certified | Valid side-dependent controls, capacity, censoring and dependence handling |
| C09 prospective protocol | Unsealed | Predeclare population, cost model, effect/precision, duration, controls, stopping and follow-up |
| C10 fixed-interval observation | No sealed experiment started | Observe full opportunity stream and disclose version/incident changes |
| C11 financial verdict | Not established | Evaluate all declared criteria at the fixed end; retain failed/insufficient outcomes |
| C12 promotion/maintenance | No financial promotion | Claim-specific verdict and authorised continuation; engineering release is distinct |

Reuse adequate feasibility evidence; require an unscored pilot only where needed. S3's registered criteria, the proposed 200-episode interpretation, sufficient week/dependence coverage and the owner goal remain distinct. Do not start scoring before sealing or extend the same experiment until it passes. Current authorised trading continues under its existing rules; no new service, paid API route, worker allowance/window or live writer is created.

Private evidence is under `/workspace/jiaoyi-checkpoint-evidence-20261031/`: broker captures and manifests, ledger scoped checks, economic candidate specification/manifest, exact failing/passing gate fixtures and independent reviews. Raw owner data belong in the private report, not this public continuation record. Release receipts are appended to the PR description when verified.


## §CP.5. Verified first release and risk-qualification correction

PR141 merged as `97ffc6971a3edee624f5fef9e159096d976eee89` after all three required checks. Railway trading deployment `e3dbefa6-7f2e-419e-902d-923d992145a7` ran that exact source; the previous deployment was removed. Eighteen release assertions passed, including all eight connected/progressing strategy actors, unchanged registration/settings/margin/owner-pause policy, unchanged engineering component and no extra worker sessions. Broker protection readback preserved existing positions and stops. These observations establish the bounded engineering release, not economic acceptance.

The raw reversal shape independently reproduced a second defect: legacy risk performance accepted closing-filled quantity greater than proven opening quantity and capped expanded or missing original-request risk at a full share. This manufactured a measured whole-position R for a mixed lifecycle whose original strategy attribution was not certified. The reader now requires equal closing/opening quantities and a positive request covering the opening history. Valid partial fills still scale original risk; valid partial exits are measured only when complete. Cash and all underlying deal rows remain intact.

The associated limits/report repair marks incomplete evidence explicitly, suppresses fictitious zero-R recovery/headroom, and retains previously reached alert names without presenting stale numbers as current. Incomplete evidence cannot write a limit-cleared decision. Fully qualified evidence can still clear it, and a previously valid clear is not resurrected. Owner pause keys, risk thresholds and broker actions are untouched.

Focused verification: 15 store goals integration tests, 12 limits integration tests and 8 status-page tests pass. Fixtures cover the actual mixed quantity/cash shape, missing/zero/negative/undersized requests, valid full/partial fills, partial exits, retained alerts and valid later clearance. Formatting/diff checks pass. This paragraph records local verification; the PR closure must record CI, merge and deployed behaviour separately.

The shared closed-trade reader also feeds S3's account/symbol/current-hash Kelly sample. This is not presentation-only: excluding an invalid outcome can introduce or remove a Kelly veto, including falling below its existing 20-trade activation threshold. No formula, threshold or sizing setting changes. All 80 independently inspected S3 positions satisfy the new predicates, including the three directly identified current-v6 examples. This does not prove all possible current-hash populations are unaffected. The observed mixed NatGas incident belongs to S1 and does not enter S3 Kelly.

**Traceability:** owner continuation and specific cancellation authority; chat №17,976–17,983 §§1–2, 31 October 2026; record updated 10:14 SGT, by <Codex (GPT-6), effort: not exposed>. `log/GROK_Log.txt` checked on current main through №17,703, unchanged. Private receipts are retained under the checkpoint evidence root; no account cash or broker identifiers are added here.

The performance `evidence_state` describes the shared reader's measured/unmeasured classification. It is not a certificate of complete or fresh broker-history acquisition; the stricter goal report retains those separate checks. A missing positive closing history/opening join must remain unmeasured. The independent reviewer identified that additional legacy omission before release, and its correction is included in the same patch.


## §CP.6. Dated release closure and subsequent implementation audit

**Appended 31 October 2026, 13:23 SGT; chat №17,984–17,997 §§1–3 (prior closure) and №18,000–18,005 §§1–3 (new audit), by <Codex (GPT-6), effort: not exposed>.** Read `log/GROK_Log.txt` through №17,703, unchanged. This entry supersedes the preparation-time pending second-release statements in §CP.4/§CP.5; those paragraphs remain dated evidence.

PR142 merged as `bd40fd06b3f3d6419d94b341265bdc70fd7ec601`; [CI41159695026](https://github.com/ang-kl/jiaoyi/actions/runs/41159695026) passed all three required jobs. Trading deployment `9f3ea646-59ea-41e1-b586-cd5ac2c7c16b` ran the exact source, with 22 runtime and 14 broker-book/protection assertions at 31 October 10:24 SGT. The 35 focused prior tests and these release receipts close that bounded repair release. They do not certify all lifecycle states, the complete proposal or a financial edge.

The subsequent [implementation audit](proposal-implementation-audit-2026-10-31.md) adds three reproduced gaps: historical ownership can permit management of a reversed/enlarged same-ID lifecycle (P1); shared accounting omits receipt/history contradictions which strict goals reject (P2); worker release handling indefinitely waits for a trading deployment even when documentation scope intentionally skips one (P2). Seventeen focused acceptance checks are specified as 8 + 6 + 3 required checks, not already passing repairs. A malformed-precision counterexample is retained as deferred P3 hardening, with no observed occurrence in the fixed 80-position packet. Earlier statements that incomplete evidence cannot clear alerts apply only where the shared reader classified that evidence as incomplete; the newly reproduced omission can evade that guard.

Research remains partial: all 24 source fingerprints match, but the private 15-case S3 adapter is not a complete deployed evaluator; all 13 prospective fields remain unsealed. Four-account exact settings/universe input, original monetary-risk gaps, historical costs/paths and statistical design remain separately tracked. Current authorised trading is not a financial PASS. All 16 advertised worker source digests match main despite its older release label; at 12:05 SGT it used 5/36 sessions with the global 29 October 07:30 SGT mutation deadline expired. This record changes neither that deadline nor strategy/risk authority.
