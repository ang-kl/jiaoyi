# Proposal implementation and post-repair audit — 31 October 2026

**TL;DR:** the PR141/142 repairs have bounded test and release evidence. The full proposal remains incomplete. A new independent audit reproduced three further engineering gaps: changed-lifecycle management authority (P1), omitted incomplete outcomes in shared accounting (P2), and an indefinite worker release wait (P2). A malformed-precision counterexample is retained as deferred P3 hardening. These are concrete boundaries, not evidence that they caused the historical trading losses. Research acceptance and the owner's financial goal remain separate and unproved.

**Trace:** chat №18,000–18,005 §§1–3; prepared 31 October 2026, 13:23 SGT; by <Codex (GPT-6), effort: not exposed>. The №18,005 status was checked at 13:20:48 SGT; its displayed 13:21 time was rounded forward. Audited main: `bd40fd06b3f3d6419d94b341265bdc70fd7ec601`. `log/GROK_Log.txt` was read through №17,703 (30 October 04:29 SGT), with no newer entry. This public handover contains no private broker identities or cash; detailed fixtures, raw captures and source hashes remain in the private audit packet.

## §AUDIT.1. Prior release is closed; its scope is bounded

PR142 merged as `bd40fd06b3f3d6419d94b341265bdc70fd7ec601`. [CI41159695026](https://github.com/ang-kl/jiaoyi/actions/runs/41159695026) passed all three required jobs. Trading deployment `9f3ea646-59ea-41e1-b586-cd5ac2c7c16b` ran the exact source; retained release evidence includes **35 focused tests, 22 runtime assertions and 14 broker-book/protection assertions**. The relevant source observation was 31 October 10:24 SGT. Existing protection and the earlier specifically authorised pending-order disposition were preserved. No broader order-management authority or financial acceptance follows from those checks.

PR141's direction guard remains effective within its tested stop-amendment boundary. PR142 rejects its demonstrated mixed quantity/request-risk cases and retains alerts when the shared reader classifies the evidence as incomplete. The new findings below concern additional boundaries which those repairs did not close. The old pending-release text in the dated checkpoint record and CODEX log is superseded by this closure, not silently rewritten as if it had been known earlier.

The engineering component's older repository release label is not itself evidence of stale coordinator code: all **16 advertised worker source digests match audited main**. At the retained 12:05 SGT status, the observer was active, **5/36** sessions had been consumed and mutations were disabled by the separate **29 October 07:30 SGT** deadline. Unused permits do not reopen that window. No worker deadline, allowance, authentication route or strategy pause is changed by this audit.

## §AUDIT.2. Reproduced additional engineering findings

### P1 — Historical ownership can authorise management of a changed lifecycle

Source anchors at audited main: `crates/jiaoyi-store/src/trading.rs:128`, `crates/jiaoyi/src/strategy3.rs:1164`, `:1203`, `:1253`, `:1887`, and `crates/jiaoyi-ctrader/src/trading.rs:591`, `:791`. Historical proof that Jiaoyi once opened a broker position is accepted for later same-ID state. Three broker-free tests through the real S3 actor/store/gate reproduce an original plan closing a same-size opposite-side position, an enlarged same-side position, and a reversed/enlarged position after actor restart. A legitimate owned TP1/session-end control passes.

The [current ownership boundary](current-system-contract.md) does not authorise adopting external/manual changes. Updating cached side, checking only side, or checking only quantity would not resolve every reproduced case. The minimal repair must qualify original owned side and attributable residual quantity at adoption and management dispatch. Preserve valid partial openings/exits, recovery and existing broker protection. Ambiguous ownership must not silently grant management authority. S1 and same-side stop-amendment implications are source-derived; the executed counterexamples exercised S3 session-end closes. No historical actor, broker loss or currently occurring live reversal is inferred.

**Eight required acceptance checks — proposed, not yet reported as passing:**

1. Existing-actor same-size reversal cannot amend or close under the old authority.
2. Same-side enlargement does not grant authority over additional unproved quantity.
3. Restart retains the refusal for a reversed/enlarged lifecycle.
4. An ordinary owned whole close and TP1 partial remain authorised.
5. Partial opening plus later fill preserves correct residual ownership without double counting.
6. Partial close, external reduction and delayed history are qualified conservatively while retaining valid residual protection.
7. Refused or valid amendments preserve tighter existing broker SL and TP.
8. Unconfirmed requests, duplicates, reconnects and late receipts do not create duplicate closes or erase unresolved state.

### P2 — Shared accounting omits evidence that strict goals reject

Source anchors: `crates/jiaoyi-store/src/trading.rs:920`, `:983`, `:1003`, `:1039`; stricter existing checks at `crates/jiaoyi-store/src/goals.rs:450`, `:459`; consumers at `crates/jiaoyi/src/limits.rs:95`, `:135`, `:270` and `crates/jiaoyi/src/strategy3.rs:2397`.

One fixture has a confirmed opening receipt larger than the opening/closing history subset. The shared reader treats that equal subset as a fully closed measured outcome; strict goals correctly return UNKNOWN. A second fixture has a terminal receipt preceding closing-history ingestion and is incorrectly reported as measured with no unknowns. A third uses the production history-upsert API to revise a closing quantity while its terminal receipt remains; the shared reader omits the affected outcome and permits a prior limit alert to clear. The alert-clear counterexample specifically requires that source revision; it is not evidence that ordinary receipt arrival alone removes a previously recorded loss.

A controlled Kelly sample combines nineteen synthetic −1R values with the reader’s false completed outcome and crosses the existing activation-count boundary. The actor’s account/symbol/opening-hash filter was inspected in source, not exercised end to end by this probe. This proves possible function-level eligibility impact, not that a current production population did so or that lot size increased. Strict official goals correctly reject all these fixtures. No occurrence was found in the independently rechecked fixed 80-position source set.

The minimal correction is consistent receipt-versus-history qualification before admitting shared outcomes or asserting zero unknowns. Genuine open residuals remain open; terminal outcomes missing required history remain unknown. Preserve cash, original-risk attribution, batched reads, existing thresholds and owner-only pause authority.

**Six required acceptance checks — proposed, not yet reported as passing:**

1. A confirmed opening larger than equal partial opening/closing history cannot enter completed R or Kelly.
2. A terminal receipt before complete closing history cannot appear fully measured.
3. A previously reached alert survives a relevant revision or history gap.
4. Complete late history restores measurement and permits a justified later clearance.
5. Fully filled closes, partially filled requests and genuine partial exits retain correct results.
6. Current-hash Kelly consumes only qualified completed outcomes without policy changes.

### P2 — Worker release wait has no non-runtime or bounded failure disposition

Source anchors: `ops/engineering/worker.py:1062`, `:1117`, `:1461`, `:1522`, `:1525`, `:1596`; `railway.toml:7`; `ops/engineering/tests/test_trading_deploy_scope.py:44`. The worker permits documentation/journal changes but sends every merged job into a wait for the exact trading deployment. Deployment filtering deliberately avoids restarting trading for those files. A mocked invocation of the real coordinator leaves the job waiting through repeated old-success, matching skipped/failed/crashed and newer-success observations. The existing **157 Python tests pass**, demonstrating missing coverage rather than a failing baseline.

The repair must classify exact merged component scope. Documentation-only work needs repository/CI/document verification; runtime changes still require correct deployment and task-specific verification. Failed, skipped or superseded evidence needs a bounded explicit disposition, not a fabricated PASS or an unnecessary trading restart. The latest-20 public job list does not expose all older release-wait jobs; zero `verification_pending` is not proof of an empty release backlog. No actual currently stalled documentation job is claimed without its retained queue readback.

**Three required acceptance checks — proposed, not yet reported as passing:**

1. Documentation/journal-only merge completes the appropriate exact-head repository verification without requesting a trading deployment.
2. Runtime merge retains exact deployment and behaviour checks; failed/crashed/skipped/superseded observations have an explicit bounded disposition.
3. Old release jobs remain visible without starving unrelated permitted work; a closed mutation window still blocks publication while allowed observation continues.

### P3 — Malformed monetary precision, deferred hardening

`crates/jiaoyi-store/src/trading.rs:946`, `:950`, `:1065` can admit a nonfinite R from an invalid precision fixture. Strict goals already reject precision outside their supported range (`goals.rs:262`). No malformed precision or nonfinite source cash was found in the archived 80-position set. Keep this as a bounded hardening item for the same qualification surface, not a claimed production incident or separate large project. Required eventual behaviour is unknown evidence for invalid precision/nonfinite net or R, preserving explicit-zero fee semantics.

The **8 + 6 + 3** cases above are seventeen focused repair-acceptance requirements. They are not seventeen completed repair tests. A future repair still requires implementation, independent review, all required CI and affected-component release proof.

## §AUDIT.3. Twelve-checkpoint closure

| Check | Current bounded finding | Remaining closure |
|---|---|---|
| C01 source/cash | Statement, funding and fixed cash checks retained | Historical cutoff equity/float for marked-return claims |
| C02 lifecycle/original risk | 80 terminal/cash/stop-geometry checks; 14 original monetary denominators independently reproduced | 66 denominators and later-command attribution; shared-reader gap above |
| C03 current exposure | Dated protection and specifically authorised disposition verified | New releases/actions require their own current readback; management-authority repair remains open |
| C04 economic mechanism | Concrete unchanged S3v6 hypothesis independently challenged | Its economic mechanism and transferable after-cost edge are unproved |
| C05 fixed policy/comparator | 24 source fingerprints and observed PR142 build binding verified | Exact effective settings/universe, initial adaptive state and information-time provenance |
| C06 executable costs | Production formula and early veto implemented | Historical as-of costs and qualified counterfactual fills/latency/gaps/financing/conversion |
| C07 engineering proof | Prior two repairs have bounded release proof | New P1/P2 boundaries require their own repair and release checks |
| C08 independent evaluator | Reusable H1 helpers; private 15-case S3 mechanics prototype | Complete qualified S3 control/input integration, claim/capacity decision, censoring and inference |
| C09 prospective protocol | Thirteen future/design fields explicitly unsealed | Seal them before scored outcomes, including dates, execution model, dependence and stopping |
| C10 fixed-interval observation | Operational monitoring runs | No sealed prospective study has started |
| C11 financial verdict | No qualified goal or financial PASS | Valid completed evaluation; software checks cannot substitute |
| C12 promotion/maintenance | Current owner authority and release/reporting rules documented | Claim-specific financial verdict; no automatic promotion from CI |

All 24 manifest source SHA-256 and Git blob bindings match `bd40fd06`. The adapter's 15 synthetic groups reran identically; the earlier seven boundary assertions cover target/session cases. All author fixtures use zero cost reserve, so nonzero-cost and heterogeneous units require further verification before broader use. The private prototype consumes supplied broker applications and fills and explicitly does not qualify market performance. It is absent from tracked main and deployment.

The committed runner accepts only H1 (`crates/jiaoyi/src/edge/runner.rs:222`). Its random-entry-day control (`crates/jiaoyi-core/src/research/control.rs:1`) is not the same-time S3 direction control required by the plan. The older exit diagnostic expressly uses an assumed S1 60-second cadence (`crates/jiaoyi-core/src/research/exit_replay.rs:1`, `:34`), despite the example name `s3_exit_replay`. A small correctly scoped evaluator/input integration remains necessary; another framework rewrite does not follow from these gaps.

Current commission/spread readers are valid preparation-time inputs, not a historical as-of reconstruction (`crates/jiaoyi-store/src/research.rs:337`, `:346`; `crates/jiaoyi-store/src/strategy3.rs:277`). Historical replay needs original decision snapshots or a qualified event-and-ingestion cutoff. The bounded existing-state query still needs seven effective numeric settings and universe overrides across four accounts, a 32-row input requirement. Source defaults are not current state.

The owner goal remains risk:reward 1:1.38 and at least 73% net wins over the latest 20 qualified positions per strategy/account, with 15 wins the first whole-count threshold. Planned geometry, realised payoff, cash, uncertainty and consistency remain distinct. A prior loss cohort is development evidence; it cannot be renamed a fresh holdout. Current financial evidence does not justify claiming a repeatable edge or ruling out every possible edge.

## §AUDIT.4. Granular requirement inventory

These 39 rows trace the selected research proposal, not every feature ever requested. Status counts are not a completion percentage. `Verified` is bounded to the named claim; `Private prototype` is not deployed functionality; `Data-dependent` requires sources or future observations; `Specification` denotes a document/protocol, not executed acceptance. The additional engineering findings above override any broader completion inference from the earlier bounded verified rows.

| ID | Gate | Requirement | Status | Remaining limit/action |
|---|---|---|---|---|
| 01a | C01 | Reconcile statement cash and funding scope | Verified, bounded | Retained private accounting checks; this audit did not refetch or recalculate broker cash. |
| 01b | C01 | Historical cutoff equity, float and cash-flow-adjusted returns | Data-dependent | Mixed-time current float is not historical NY-close valuation. |
| 02a | C02 | Fixed 80 S3 opening/terminal quantities, cash and original stop geometry | Verified, bounded | 80/80 is a fixed mixed-version cohort, not pure-current-v6 or every historical position. |
| 02b | C02 | Recompute original monetary R for all 80 positions | Data-dependent | 14 original USD denominators reproduced; 66 lack matching risk/conversion evidence. |
| 02c | C02 | Attribute NatGas later commands and mixed lifecycle to a strategy/controller | Data-dependent | Six fills identify the reversal/scale-ins and cash, not authorship of sync/ctm commands or causation of the loss. |
| 02d | C02 | Keep invalid mixed lifecycle original R unmeasured | Verified, bounded | PR142 fixes its reproduced mixed-risk shape; new receipt/history omission remains open under §AUDIT.2. |
| 03a | C03 | Current exposure/protection and specifically authorised pending cancellation | Verified, bounded | Timestamped 31 Oct snapshot; no guarantee of future protection or exact maximum loss through gaps. |
| 04a | C04 | Concrete falsifiable economic mechanism and adverse alternatives | Specification | Plausible continuation/order-flow explanation is not an identified source of alpha; historical SPY results do not transfer to this policy. |
| 04b | C04 | Independent challenge of proposed mechanism | Verified, bounded | Independent reasoning/source challenge completed; model agreement is not market evidence. |
| 05a | C05 | Pin all relevant source and inherited registrations | Verified, bounded | This audit independently checked 24 SHA256 and git-blob bindings to bd40fd06. Not a future T0 binding. |
| 05b | C05 | Observed deployed build/public state binding | Verified, bounded | Retained PR142 10:24 projection/release bound; component reads are not an atomic full snapshot. |
| 05c | C05 | Complete effective numerical settings and universe overrides | Data-dependent | Seven numeric values plus universe across four accounts remain absent. Local SQL validation is not a production read. |
| 05d | C05 | Initial adaptive/book state, lifecycle management versions and information-time inputs | Data-dependent | Initial Kelly/veto/mark state and input availability are not fully sealed; source hash alone cannot reconstruct exit-version hybrids. |
| 05e | C05 | Historical bot-trade frozen policy/model/context provenance | Data-dependent | Full historical adapter/deployment/model context unavailable; current LLM cannot reconstruct past decisions without hindsight. |
| 06a | C06 | Production cost/R formula and early veto | Verified, bounded | Formula and refusal path exist; 24 bps reserve is a rule/assumption, not current-v6 empirical calibration. |
| 06b | C06 | Decision-time historical spread and commission availability | Missing integration | Existing calibration reads all then-stored commission deals and preparation-time recent spread; no historical as-of argument. Reusing today for earlier decisions is invalid. |
| 06c | C06 | Calibrated feasible fill/latency/gap/financing/conversion model | Missing integration | Script consumes supplied events/fills; quotes do not prove depth, market impact or counterfactual execution. |
| 06d | C06 | Qualified recorded market paths at complete opportunity times | Data-dependent | Opening receipts do not provide full intervening bid/ask/protection paths or post-actual-exit counterfactual history. |
| 07a | C07 | PR141 stop-amendment direction repair tested and deployed | Verified, bounded | 37 gate tests and 18 release checks retained. Does not prove financial causation or broker-atomic lifecycle protection. |
| 07b | C07 | PR142 lifecycle qualification/alert repair tested and deployed | Verified, bounded | Prior release verified; additional shared-reader qualification gap requires a separate correction. |
| 07c | C07 | Public checkpoint/handover closure reflects final PR142 receipts | Specification | Corrected by this dated append-only handover; no new runtime release is claimed. |
| 08a | C08 | Reusable H1 bootstrap/control/verdict components exist | Verified, bounded | Helpers implement H1/random entry-day semantics; they are not full current S3 validation. |
| 08b | C08 | S3 fixed-entry mechanical diagnostic | Private prototype | 15/15 rerun exactly; seven earlier boundary assertions. All author fixtures reserve=0; narrow unit/precision/event assumptions. Not tracked/deployed. |
| 08c | C08 | Execute valid 1,000 S3 same-time side-dependent controls | Missing integration | Committed runner explicitly rejects non-H1; H1 controls resample entry dates. Private adapter has no qualified opposite-side execution model. |
| 08d | C08 | Correctly define conditional controls versus feasible portfolio capacity | Specification | Private proposal chooses isolated conditional entries; explicitly does not discharge registered acceptance or joint capacity. |
| 08e | C08 | Complete predetermined opportunity stream and missing/veto/order outcomes | Missing integration | Operational logs/events exist but no qualified research dataset/evaluator of the entire fixed universe/opportunity stream demonstrated. |
| 08f | C08 | Rule-level shared-signal episode aggregation and common dependence | Specification | 200-episode aggregation/weights and dependence design remain null; owner per-account 20-window is a separate grain. |
| 08g | C08 | Scored censoring/follow-up and paired portfolio-calendar inference | Missing integration | Trade-entry-week helper omits no-entry weeks; private adapter labels individual censoring but no full fixed-horizon portfolio evaluator exists. |
| 08h | C08 | Research artifact provenance and repeatable integration | Private prototype | Hashes/receipts are private files, absent from tracked main/CI. This is legitimate private preparation but not deployed feature completion. |
| 09a | C09 | Immutable prospective timing/design/seed/state protocol | Specification | All 13 future fields null: T0/Tend/follow-up/calendar/effect/episode/dependence/control/multiplicity/execution/seeds/state/build. |
| 09b | C09 | Trial family, multiplicity, failed development records and untouched holdout | Specification | Known H1 losses and historical statement cohorts are development data; no complete selection-adjusted return matrix shown. |
| 10a | C10 | Operational goals and health observation continue | Verified, bounded | 60-second supervisor/qualified latest20 are implemented; operation is not a scored fixed-policy experiment. |
| 10b | C10 | Fixed-interval prospective evidence collection for sealed policy | Data-dependent | No sealed study has started; future observations cannot be created by code or more agents. |
| 11a | C11 | Owner latest20 goal correctly separated from edge claim | Verified, bounded | 3/5/4/1 fixed cash winners versus15; 14/80 independently qualified monetary denominators. No goal pass. |
| 11b | C11 | Absolute executable after-cost S3 financial acceptance | Data-dependent | No qualified positive expectancy/bootstrap/control/PF result; 15 synthetic passes cannot supply market evidence. |
| 11c | C11 | Improvement over bot-trade under comparable feasible exposure | Data-dependent | Different accounts, periods, currencies, provenance and missing historical model context prevent causal superiority claim. |
| 12a | C12 | Promotion/maintenance tied to financial verdict and authority | Specification | Current live authority is owner permission, not financial promotion evidence; no financial PASS. |
| 12b | C12 | Recurring economic readout has complete qualified metrics | Implemented, unverified | Operational/private reports exist; prospective qualified expectancy, paired uncertainty and complete trial ledger are not demonstrated as a running report. |
| 12c | C12 | Autonomous worker authority is separate from trading/research readiness | Verified, bounded | Worker observer runs, mutations remain deadline-disabled; release-wait defect is open under §AUDIT.2. |

## §AUDIT.5. Remaining scope and next useful work

The [research plan](research-validation-plan-2026-10-31.md) §§P4–P8, [checkpoint record](checkpoint-progress-2026-10-31.md) §CP.4 and [system contract](current-system-contract.md) §§CURRENT.3–CURRENT.13 govern these distinctions. The fast-monitor proposal already discloses time spent awaiting broker/database work; no new live timing outage or hard three-second execution guarantee was established by this audit.

S3 preparation is implemented, with eight retained weekend crypto preparation receipts before their deadlines. This does not certify every historical deadline. The US/Europe stock view remains cache-only and ineligible; broker identity, input acquisition and verified session calendars are unfinished. VaR, VWAP and historical marked returns remain unknown when the necessary evidence is missing. A renderer cannot manufacture them.

The useful next sequence is the three reproduced engineering corrections and their focused counterexamples, independent review, required CI and component-scoped release proof; then the bounded settings/source reads, smallest valid evaluator completion and sealed prospective design. The deferred precision item may accompany the same qualification surface if warranted. Do not loosen risk policy, increase session use, add a broker writer, start a paid route or rewrite the strategy merely to make this inventory shorter. More code cannot create the future market observations needed to judge the financial hypothesis.

Private audit artifacts retain the complete 39-row JSON matrix, 24 source checks, execution/accounting counterexamples, worker reproduction and independent counter-review under `jiaoyi-proposal-audit-evidence-20261031`. Private prior release/source artifacts remain under `jiaoyi-checkpoint-evidence-20261031`. These directory names identify retained owner evidence; they are not a public-data publication or assertion that all prerequisites have passed.
