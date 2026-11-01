# All-trades assessment: implementation and release evidence

**Trace:** owner “execute the plan”, following №18,092; progress №18,093–18,114. **Prepared:** 1 November 2026, 19:30 SGT. **By:** <Codex (GPT-6), effort: not exposed>.

This record maps the [approved plan](all-trades-assessment-fix-plan-2026-11-01.md) to the implementation. Exact CI results, merged/deployed source and the bounded production observation belong to the implementation PR's final receipt; source inspection and local tests alone are not a deployed PASS.

## §REL.1 — Implemented behaviour

- The existing history read sessions observe every broker position and pending entry, independently of strategy ownership. Protective closing orders are counted separately. External instrument/conversion subscriptions do not grant management authority.
- A deterministic one-second evaluator reports named checks and evidence ages. Explicit new-entry permission checks distinguish owner pause, session cost veto and capacity from current-position protection. Bounded deadline/source/disconnection episodes feed the existing coordinator; these are separate from profitability or closed-market quote availability. Manual changes, execution gaps, broker trailing-stop changes and account/margin updates invalidate affected evidence. Snapshot/event races and connection generations cannot restore old state as current.
- Pending capacity uses broker expected-margin evidence at current conditions with the existing numerical margin equations and stop reserves. Future trigger-price margin remains explicitly uncertain; passing the projection is not a complete entry-gate certificate.
- Bounded private persistence keeps one latest row per account plus at most 64 material transitions. Ordinary ticks/timestamps do not create per-second history. Disappeared objects retain their last assessment without manufacturing a close fill or cash result.
- Final entry validation follows asynchronous ledger/account work and writer-lock acquisition. Original signal/side, current quotes, account revision and dispatch deadline are checked. A guard refusal before any write is durably distinguished from partial or uncertain transmission. New intent keys prevent unsent attempts from poisoning the session/day or being resent.
- S3 v7 adds the versioned dispatch condition; inherited exits/risk are unchanged. Same-session predecessor v6 cost vetoes remain effective, while the new Kelly cohort is not retroactively filled with older trades.

## §REL.2 — Owner views and privacy

| Route | Access / purpose |
|---|---|
| `/operations/setup-assessments` | Public coverage, timing and source-status aggregates; no position IDs, prices, quantities or cash |
| `/owner/assessments` | Authenticated HTML view of current trade checks; five-second page refresh over the one-second evaluator |
| `/owner/setup-assessments` | Authenticated detailed snapshot and bounded transitions |
| `/owner/quote-coverage` | Authenticated aggregate inventory for one permitted account/symbol and at most 24 hours; no raw ticks/prices or generic SQL |

Owner authentication uses the already-configured `JIAOYI_UI_PASSWORD`: browser Basic authentication with username `owner`, or a Bearer header for a private client. Never put the password in a URL or report. Missing configuration denies access. Private responses are not shared-cacheable; the public route ignores attempts to request private detail.

Coverage inventory uses a read-only transaction, one concurrent request, statement/overall timeouts and a per-source row cap. Observed gaps, source timestamps, historical fetch availability and recorded decision marks are distinct; counts do not prove an unbroken broker stream or a profitable strategy.

## §REL.3 — Verification mapping

| Plan checks | Evidence implementation |
|---|---|
| T01–T08 | Pure evaluator and account-collector tests: external/manual trades, protective orders, revisions, stale/partial data, out-of-order events and reconnects |
| T09, T14 | Scheduled deadline/skipped-tick accounting and bounded incident episodes; fake broker with continuously arriving quotes, held history future and blocked database access; independent completions, bounded transitions and zero trading frames |
| T10 | Both sides, profitable trailing protection, S1 optional TP, unknown manual strategy, residual quantity and broker units/lot conversion |
| T11–T13 | Guard/client/store tests: frozen midpoint direction, registered costs, deadline after ledger/lock waits, proven-unsent keys/reservations, partial-write uncertainty and unchanged exits |
| Owner privacy | Authentication/configuration failures, read-only routes, non-cacheable detail, HTML escaping and public/private separation |
| Economic source inventory | Request validation and PostgreSQL fixtures for scoped live/history counts, missing/capped data and calendar/recorded-mark distinctions |
| Strategy regression | Existing S1 and S3 suites with real wire quotes feeding the independent guard cache; original trading/exit assertions retained |
| T15 | Required CI, exact deployment identity, authenticated/unauthenticated probes, all-account source/coverage reconciliation, completion counters and bounded observation; record actual results in PR closure |

The old S3 unit-style fixtures supplied quotes only to the actor. They were corrected to feed the fake broker wire before invoking the actor; the new guard was not bypassed to obtain green tests. An independent review also challenged stale-book passes, pending-order attribution, account-update events, transport expiry and pending-margin coverage. Integrated tests exposed a session-end cancellation race: the observer could finish before history recorded its interrupted window. A bounded five-second cleanup grace now preserves that receipt; shutdown remains immediately cancellable. Both exact regressions and the full 16-test history suite pass with their original counter assertions.

## §REL.4 — Release and remaining evidence

Before merge: required formatting, Clippy, Rust/PostgreSQL, coordinator and image checks must pass on the integrated commit. Trading's existing Railway watch paths deploy the merged implementation. The Rust observer runs independently of model sessions. The narrowly changed coordinator incident consumer must also be released at its exact source so missed deadlines and source failures reach the existing bounded incident path. Its session ceiling, expired mutation window and authentication route remain unchanged.

After merge: verify exactly one active trading deployment at the merged source, new registration, eight existing strategy actors, four assessment loops, protected detail, aggregate privacy and current broker-book coverage. Retain a bounded 30-minute observation with completion intervals, source-age/unknown states and any real failures; do not force a trade to produce a success sample. Preserve broker-side exits and use one authoritative writer through handover.

Economic evidence remains separate: selected lifecycle accounting/stop changes are reproduced, one research-only candidate is frozen, and production interval coverage must be measured before any alternative-return claim. See [the economic disposition](assessment-calibration-evidence-2026-11-01.md). Missing future outcomes are not a reason to claim an edge, reset losses or use extra sessions. No financial-goal improvement is asserted by this implementation record.

A final plan audit required explicit permission reporting, bounded missed-deadline incidents, pre-activation latency evidence and component recovery instructions; those are part of this release, not assumed from the first green build. See [the dispatch benchmark](entry-dispatch-benchmark-2026-11-01.md).

The missing exact text of №18,083 remains an explicit scope item. Known work proceeds, but “all requested scope completed” cannot include an unrecovered instruction.

## §REL.5 — Component recovery without removing exit protection

Use a reviewed source correction and the same CI/deployment checks; there is no runtime bypass that continues to claim unchanged v7 semantics. Do not deploy the old pre-release binary after new ledger event kinds have been recorded.

1. **Observer-only disable:** remove the `setup_assessment::run_session` integration from `history::run_with` and await the existing `maintain_account` directly. Keep the new database table and stored evidence, all strategy actors and the transport/accounting changes. The observer owns its subscriptions on the history read connection; this change does not unsubscribe a strategy's stream or cancel a broker order. Make the owner routes explicitly report disabled/unavailable rather than an old successful snapshot.
2. **Entry-guard rollback:** revert only guard construction/use in the S1/S3 entry paths and publish a new registration explicitly stating the restored dispatch semantics. Retain `REFUSED_BEFORE_SEND` and `ENTRY_VALIDATED_BEFORE_SEND` parsing/accounting, unique intent keys, uncertainty/recovery and every existing close/amend/session-exit path. Historical registration hashes and receipts remain unchanged. A policy rollback must not relabel older outcomes or remove inherited same-session cost vetoes.
3. **Verify the recovery build:** rerun the relevant observer/history tests or transport/proven-unsent/recovery tests with a normal entry and an exit control, then formatting, Clippy, the integrated suite and all required CI. Read back the replacement registration, exact source, one active writer, existing broker protection and progressing exit actors. Never run a second trading instance as a recovery test.
4. **Coordinator parser rollback:** revert only the new incident projection/recovery branch, retain existing incident records and permissions, and redeploy the matching coordinator component after its tests. This does not disable trading or grant an engineering mutation window.

These are precise source recovery procedures, not a pre-built alternate artifact or permission to pause accounts, cancel manual orders, widen stops or weaken unrelated risk limits. Until a verified replacement is ready, broker-held protection and the current authorised management actors remain in place.

## §REL.6 — Incident capacity and historical queue boundaries

The read-only pre-release audit at 1 November 2026, 19:18 SGT found **32/32 retained incidents and no closed slot** (28 missing evidence, three repair-held, one analysis-pending). Six jobs were ready, none active, with 36/36 daily sessions used. The new observer consumer therefore needs bounded overflow receipts: at most three known incident kinds per eight bounded account rows, separate from the existing 32 normal incident records. Overflow reports facts and capacity blockage without scheduling analysis, evicting unresolved records or spending sessions. Unchanged episode counters/times cannot create repeated work. Fresh explicit recovery is distinct from verified repair closure; later admission requires a legitimately available normal slot.

Issue147 was reconciled to the evidence available: PR144–146 receipts exist, but historical job258–261/277 records are outside the latest20 public job projection. An existing read-only SSH attempt failed before remote execution because no SSH keys were configured. Their exact task/disposition therefore remains unverified; generic jobs are not marked complete merely because related PRs merged. No job was dispatched, key registered or mutation window renewed to manufacture closure. Current capacity and source provenance are inspectable independently of that historical gap.
