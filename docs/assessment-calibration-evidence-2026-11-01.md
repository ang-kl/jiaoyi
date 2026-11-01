# Assessment calibration: evidence and frozen research amendment

**Prepared:** 1 November 2026, 18:32 SGT. **Trace:** owner “execute the plan”, following №18,092 and [§PLAN.6–7](all-trades-assessment-fix-plan-2026-11-01.md). **By:** <Codex (GPT-6), effort: not exposed>. **Status:** offline evidence checks completed; one calibration definition frozen; no new experiment, trading policy or automated schedule activated.

## §CAL.1 — What was verified

A fresh offline verification reproduced **25 source fingerprints, four selected complete S3 lifecycles, 57 successful stop amendments and 20 sampled broker acknowledgements** from the saved 1 November investigation. It independently joined program opening receipts to original broker orders and final broker deals, checked quantity balance and signed net cash, derived the original stop from the fill and broker relative distance, checked monotonically tighter stop requests, preserved TP acknowledgements and original v5/v6 registration hashes.

The source capture is dated approximately 1 November **08:58–09:05 SGT**. Running this verification later does not create a fresh account observation. The four cases are selected historical examples; they are not a random sample, an out-of-sample result, or proof that every lifecycle is complete. The source order extracts are below their row cap; broker replies report no further page within the requested interval.

Those records establish that protection was adjusted. They do **not** establish that changing direction, quantity or TP would have improved returns. The inspected packet contains opening/fill/stop evidence and sparse management observations, not a complete timestamped executable bid/ask path. The gaps between recorded management observations reach tens of minutes. Interpolating those gaps would invent the sequence needed to decide which stop, target or reversal filled first.

The saved health snapshot reports historical tick downloads enabled on two accounts and disabled on two. Aggregate stored-tick counters do not establish coverage of any particular trade interval, and disabled historical downloading does not prove that live recording is absent. The next data step is a bounded inventory of existing quote intervals, availability timestamps and gaps—not a new raw-tick warehouse or a forced two-week wait.

## §CAL.2 — E1–E4 dispositions

| Gate | Result now | Exact remaining evidence / next checkpoint |
|---|---|---|
| **E1: reconstruct** | **PARTIAL.** Selected actual cash, terminal quantity, stop changes and versions reproduced. | Existing interval-level bid/ask paths; original decision/band and availability times; contemporaneous cost/conversion inputs; the complete opportunity stream including veto/no-signal marks. Request only the specific missing intervals after a coverage inventory. |
| **E2: specify** | **FROZEN DEFINITION; ACTIVATION PENDING.** One minimal exit candidate and its comparison/preflight rules are fixed below. | Bind the exact deployed/replay source, settings, initial state and validated cost/latency adapter. The parent study's inference duration/effect/dependence design is not invented from four selected closes. |
| **E3: evaluate** | **INSUFFICIENT** for alternative-policy performance. No counterfactual returns calculated. | Reproduce the registered baseline and candidate on complete retained intervals with executable-side, partial, gap and tie controls; then use the existing study's sealed fresh-data design. The first relevant complete session supplies missing operational evidence only if retained intervals cannot. |
| **E4: decide** | **NO PROMOTION.** No evidence-based reversal, resizing or TP change is approved by this result. | Independent, claim-specific after-cost verdict under the parent study. A completed engineering release or passing latest-20 window does not satisfy this gate. |

Known engineering work can continue while affected economic claims remain unverified. The invariant 1:1.38 reference and at least 73% net wins over the latest 20 qualified closed positions remain per strategy/account. External and manually mixed lifecycles stay visible in account economics while unqualified for strategy inference. No new financial-goal delta is claimed here.

## §CAL.3 — Why this single candidate

The existing [S3v6 falsification study](checkpoint-progress-2026-10-31.md#cp3-concrete-economic-study-object) remains the parent. This amendment specifies **close the remaining position on an opposite registered mark signal** as a conditional diagnostic, with the original policy as control. It does not add an immediate reversal, new entry, position increase, new stop geometry or TP optimisation.

This tests the owner's specific concern—whether a position remained open after its own directional evidence contradicted it—without assuming that every losing trade should have been reversed. “Opposite” means the existing midpoint/band decision at the next eligible half-hour mark, not any adverse tick, a return inside the band or an LLM opinion. The first eligible quote and the original data-availability rules remain explicit. The candidate was selected after historical losses were viewed; it is a development hypothesis, not a discovery on untouched data.

The paired comparison fixes entries and opening quantity. It answers a conditional exit question; it does not claim a portfolio improvement from capacity released by earlier exits. A later capacity-constrained comparison must use the parent methodology and full opportunities. Human trades are monitored by the new assessment system but do not silently inherit this research policy.

The baseline source below is the inspected PR146 implementation with S3v6 semantics. Any new entry guard must be applied identically to both arms and bound to an exact version before comparison; otherwise changing entry eligibility is a confounder. A new registration/hash must not retroactively relabel the historical v5/v6 examples.

## §CAL.4 — Frozen definition and verification

The JSON below is the portable source of the frozen definition. Its SHA-256 is computed from UTF-8 JSON with sorted keys, separators `(',', ':')`, and `ensure_ascii=False`:

`3573f08261353ba2c8e845770e9a11af5b370070e3614555c34ea3e03503da26`

Changing the candidate, comparison or preflight rules creates a new amendment/hash and retains this version. Recording an activation receipt does not change these rules. This hash seals the definition, **not a claim that missing source inputs, a complete evaluator or future observations already exist**.

```json
{
  "protocol_id": "S3V6-CURRENT-FALSIFICATION-20261031-CALIBRATION-A1",
  "status": "frozen_definition_not_activated",
  "parent_study": "S3V6-CURRENT-FALSIFICATION-20261031",
  "baseline_source": "95e631adf316fe6c90a0986d88c9673175f82809",
  "baseline_registration": "d54344b9d463532c3b1e2d81bd07d671ed0d2fceae9ee923f1c8988b18b2ab32",
  "baseline_policy": "S3v6 with inherited v5 exits, original risk, one entry and one durable cost veto per symbol/session",
  "candidate_policy": {
    "name": "close_on_opposite_registered_mark",
    "population": "same complete qualified program-opened S3 positions as baseline; external/manual mixed lifecycles excluded from strategy inference and retained in accounting",
    "trigger": "while residual position remains open, first eligible two-sided quote at a registered half-hour decision mark, within the existing five-second quote window and registered entry_window; current mark_decision midpoint yields the opposite of the original position side",
    "band": "the original registered time-of-day table and current session opening input, evaluated at that mark using only information available then",
    "action": "one shadow close of remaining quantity at executable bid for a long or ask for a short, subject to the qualified common cost/latency model",
    "tie_rule": "a broker-confirmed baseline SL/TP/session closure before or at trigger time takes precedence; ambiguous ordering is unavailable, not best-case",
    "other_rules": "unchanged original R, entry, quantity, TP1/TP2, trailing and session exit until trigger; no reverse, new entry, size increase or target optimisation"
  },
  "comparison": {
    "type": "paired fixed-entry exit diagnostic, not a capacity-constrained portfolio or executable edge claim",
    "controls": "baseline original exit policy; both arms use identical archived inputs, initial quantity, executable-side convention and qualified cost/latency assumptions",
    "primary_future_diagnostic": "mean paired net original-R difference, one mean across qualified account copies within each original signal episode; report per-account outcomes separately",
    "coverage_denominator": "all baseline opening opportunities and mark observations in fixed interval including no signal/veto/missing inputs; paired returns require complete qualified paths and original risk",
    "censoring": "carry-ins separate; open or incomplete lifecycles retained as unavailable/right-censored, never dropped as wins/losses",
    "historical_role": "all currently viewed 26-31 October outcomes are development evidence, never a holdout",
    "claims": "feasibility and descriptive diagnostic only; no inferential edge, superiority or owner-goal PASS"
  },
  "availability_protocol": {
    "scope": "bounded data-availability preflight under the existing study, not a new two-week waiting period or experiment",
    "start_rule": "inspect already retained complete intervals and control fixtures immediately; use the first relevant complete session only for evidence absent from retained sources",
    "completion": "required data dimensions and replay controls verified for the intended claim; return exact missing intervals/fields promptly rather than wait for a calendar quota",
    "followup": "retain any open lifecycle as censored; do not force closes or extend observation until results improve",
    "data": "reuse bounded existing quote/decision/intent/receipt evidence; enumerate intervals and gaps before exporting raw ticks; no new tick warehouse",
    "outputs": "coverage by symbol/account/interval, eligible unique episodes, version changes, effective costs/latency evidence, missing original risk, replay disagreements, and named next checkpoint",
    "source_change": "material source/semantics change or broken evidence continuity marks affected interval invalid; retain failure and explicitly version any replacement",
    "ai_cost": "no model calls are scheduled by this protocol"
  },
  "activation_prerequisites": [
    "exact deployed baseline/adapter source and unchanged exit semantics bound; any new entry guard applied identically to both arms and versioned",
    "effective account settings, symbol/conversion metadata and initial adaptive/capacity state bound with timestamps",
    "information-time bid/ask and decision-band coverage export verified, including no-trade marks and gaps",
    "actual as-of cost and latency model documented; registration reserves clearly separated from measured fills",
    "fixed-entry evaluator independently reproduces controls including partials, stops, gaps, ties and unavailable evidence",
    "collector/output path and retention bounds verified; no additional broker writer",
    "before any scored start, the existing parent study has a timestamped immutable receipt binding hashes, fulfilled prerequisites and its separately justified future T0/Tend; preflight is not activation"
  ],
  "future_inference_requirements": "reuse research-validation-plan-2026-10-31 §§P5–P7; bind meaningful effect/precision, sufficient dependence blocks, count interpretation, common controls, trial family, costs, seeds and fixed duration before a distinct fresh scored start; feasibility observations excluded from scoring",
  "deferred_variants": "automatic reversal, dynamic resizing and TP optimisation; require a separate frozen hypothesis and sufficient path evidence, not retrospective tuning"
}
```

## §CAL.5 — Reproduction and limits

The private offline runner is `/workspace/jiaoyi-assessment-execution-20261101/economics/validate_saved_calibration_evidence.py`; its bounded result is `saved-evidence-validation.json`. Original source hashes and selected joins remain in `/workspace/jiaoyi-pretrade-review-20261101/accounting/evidence.json`. No broker, network, database, model or order call is made by this runner. Raw account identity and cash are intentionally omitted from this repository document.

Existing [research gates](research-validation-plan-2026-10-31.md#p7-m4m5-acceptance-and-stopping-rules) continue to govern edge claims. A measured alternative needs an adequately justified fixed horizon, costs, control semantics, dependence handling and trial history before fresh scoring. No model should negate actual net losses to simulate reversing, treat current spread as historical cost, reconstruct missing ticks from stop receipts, or stop the study at its first favourable window.

**Next checkpoint:** source completeness for the four verified lifecycle intervals is enumerated below. After the bounded owner route in §CAL.7 is deployed and verified, obtain the four interval aggregates and reuse complete intervals/control fixtures immediately. Add the exact implementation/adapter/settings binding once the engineering release is known. No calendar quota, extra paid model route or forced live trade is required.

## §CAL.6 — Exact interval inventory, 1 November 18:37 SGT

The bounded inventory maps the four selected lifecycles to **17 expected half-hour mark windows**, including their four opening marks and **13 subsequent candidate-assessment marks**. Each mark window is `[mark, mark + 5 seconds)`. This is an expected calendar inventory, not a claim those marks produced a signal or a recorded decision. The private manifest retains exact account/symbol/position IDs and UTC bounds.

| Case | Instrument | Lifecycle, SGT | Saved program order rows | Successful stop amendments / sampled acknowledgements | Expected opening + later marks | Largest gap between recorded management observations |
|---|---|---|---:|---:|---:|---:|
| A | BTCUSD | 30 Oct 05:30:04.969–06:24:36.655 | 24 | 22 / 5 | 1 + 1 | 837.610 seconds |
| B | BTCUSD | 29 Oct 15:00:04.809–17:08:19.013 | 18 | 16 / 5 | 1 + 4 | 4,711.527 seconds |
| C | BTCUSD | 28 Oct 18:00:05.064–20:05:26.864 | 11 | 9 / 5 | 1 + 4 | 4,681.998 seconds |
| D | ETHUSD | 29 Oct 20:30:04.055–22:30:55.896 | 12 | 10 / 5 | 1 + 4 | 3,756.438 seconds |

Required subsequent mark starts are A: **06:00**; B: **15:30, 16:00, 16:30, 17:00**; C: **18:30, 19:00, 19:30, 20:00**; D: **21:00, 21:30, 22:00, 22:30**, on the corresponding dates above. No complete raw bid/ask path or durable mark ledger for those windows was retrieved in this saved packet. These are **unknown production row counts**, not zero stored ticks or proven missed decisions. Management-observation gaps are not measured quote-stream gaps.

Case C retains its original **v5** opening registration; A, B and D retain **v6**. This interval inventory is for data feasibility and historical attribution, not a pooled frozen-v6 performance cohort. Opening registration alone also does not certify the implementation governing later management.

The same saved health evidence positively shows live recording on **all four accounts**, each with 17 subscribed instruments and roughly 246,000 ticks since its then-current restart. Those recorder sessions began on **31 October around 16:04 SGT**, after all four selected lifecycles closed. Their counters therefore cannot certify the earlier intervals. Historical downloads being disabled on two accounts must not be reported as all live quote recording being disabled.

The next bounded read is specified for each exact lifecycle interval from **six seconds before its opening mark through its closing fill**. It aggregates the existing `ticks`, `ticks_history` and `history_fetches` tables: side-update counts, first/last times, recorder sessions and completed/failed fetch coverage with availability times. Counts/endpoints are only a first pass; populated intervals then require per-side gap and 17-mark coverage aggregates. Original `strategy3_orders.detail` and the corresponding decision/band source establish information-time inputs separately; the public order projection does not expose them completely.

At the **18:37 SGT** inventory checkpoint, private `bounded_interval_inventory_readonly.sql` specified four fixed intervals, a read-only transaction, ten-second statement timeout and no raw rows or broad history download. That standalone query was **not executed**. The inherited database URL identified local development, and no existing authorised private production tick-query route was then available. Development data were not treated as production evidence and no production secrets were sought. The subsequently implemented owner route is recorded separately below.

Reproduce this inventory with private `inventory_selected_intervals.py`; output is `selected-interval-inventory.json`. The frozen §CAL.4 JSON and its hash are unchanged. E1 remains partial for the precise missing interval/path inputs above; known engineering work proceeds independently.

## §CAL.7 — Bounded private inventory implementation, 1 November 19:10 SGT

Execution of the approved fix plan now includes an authenticated **`/owner/quote-coverage`** route backed by [quote_coverage.rs](../crates/jiaoyi/src/quote_coverage.rs). It accepts one permitted account/symbol and a positive interval of at most 24 hours. It returns stored live/historical side counts, endpoints, observed gaps, historical fetch availability, fully contained UTC half-hour windows and separately identified recorded S3 mark counts. It does not return raw prices, accept SQL input, collect ticks or call the broker.

The implementation uses a read-only repeatable-read transaction, an eight-second SQL timeout, a ten-second overall deadline, one concurrent request and explicit row caps. Missing symbol mappings and truncated results remain visible. Three local request-validation/SQL-fixture tests passed; this is implementation evidence, **not a deployed-route or production-coverage result**. Full release verification is owned by the main agent.

The four exact owner query payloads are ready in private `owner-coverage-queries.json`; their exclusive upper bounds extend one millisecond beyond the recorded closes so the closing timestamp is included. After release, retain their actual responses and hashes, check all 17 calendar windows, and update E1 using the returned counts/gaps. When the source is adequate, proceed with the next bounded replay/data check immediately. When it is not, name the affected side, interval or mark rather than impose an arbitrary waiting period. The frozen candidate/protocol hash remains unchanged.
