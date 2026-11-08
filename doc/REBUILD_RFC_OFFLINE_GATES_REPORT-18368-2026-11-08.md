# Rebuild RFC — Offline engineering and pre-flight controls

№18,368 §§1–12 · 2026-11-08T17:09:50.632855+08:00 · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Offline safeguards are implemented. 4/12 original RFC gates pass, up from2/12.** The other gates have executable parsers/verifiers and fail-closed checks, but their external/economic evidence is not present. They remain blocked. This is not a claim that all12 gates passed or that the strategy is ready for live promotion.

**Collection is paused at both supported boundaries:** Python cTrader research clients and the service's owner research endpoints. A valid research request returns HTTP423 before any database/broker read. Existing trading, subscriptions, entries under existing rules and open-position protection retain their authority. No new broker requests, orders, protected-validation evaluation or live strategy changes were made by this work.

**Exact reproducibility:159 +35 +2.** All196 requested native replay outputs were run twice and matched their original receipt bytes. The merged engineering changes do not retune the frozen hypothesis or manufacture additional wins.

**Goal progress:0 new independent qualified trades; no financial improvement measured.** The account invariant remains73% net wins and1:1.38 over each strategy/account's latest20 fully closed owned trades.15/20 meets the count. Engineering progress is two additional accepted gates, not a higher win rate.

## §1. The twelve gates, verbatim from the RFC

The definitions below are parsed from §8 of `doc/NATIVE_TARGET_SIZING_AND_REBUILD_RFC-18353-2026-11-08.md`. `IMPLEMENTED_OFFLINE` describes available code; only `PASSED` counts as satisfying the original acceptance gate. A schema cannot substitute for the missing broker/economic evidence.

| Gate — original RFC definition | Status | Implemented / evidence | Remaining blocker |
| --- | --- | --- | --- |
| G1 — every result links to source, code, calendar, contract and cost versions. | PASSED | Versioned identity binds frozen hypothesis, unchanged entry/exit engine, native calendar, contract inventory, cost assumptions, spent intervals and source receipts. Unknown historical contract authority remains explicitly unknown. Code/tests: rebuild.py:build_registry/source_freeze; rebuild_preflight.Identity; unified manifest and source hashes | None for this engineering gate. |
| G2 — baseline replay reproduces frozen decisions/fills and refusing paths before a migration is accepted. | PASSED | All159+35+2 native outputs reproduce the original receipt bytes, twice, including no-entry/refusal paths. Code/tests: rebuild.py:parity/replay_bytes; parity_receipts | None for this engineering gate. |
| G3 — the runner makes zero network calls in offline mode, preserves all denominators and cannot accept arbitrary result constants. | PASSED | Single offline command; inherited Linux seccomp plus Python audit guard; native output computation only; all544 planned definitions and failures retained. Code/tests: rebuild.py; rebuild_preflight.offline_process; test_offline_restriction_is_inherited_by_native_child | None for this engineering gate. |
| G4 — an experiment cannot claim full-path qualification without verified lookback, holding-period and closing-tail coverage; interrupted receipts remain visible. | PASSED | Preflight refuses missing45-day dependencies, unspent-but-undecoded preparation, wrong native windows, stale boundaries, gaps, unverified receipts and incomplete continuation. Full-path eligibility is stricter than a recorded-scenario tier. Code/tests: rebuild_preflight.verify_plan/verify_bars; rebuild.py:preflight_file; test_preflight_blocks_pr195_defects_and_identity_overrides | None for this engineering gate. |
| G5 — targeted DST/early-close/rollover fixtures match dated authoritative examples; unknown historical terms remain unknown. | BLOCKED_EXTERNAL_EVIDENCE | Calendar layers, effective dates, DST, early-close overrides, maintenance/funding fields and unknown-term rejection are implemented. Synthetic dated fixtures pass; they do not certify the historical broker/venue schedule. Code/tests: rebuild_preflight.CalendarTerms; test_dated_calendar_dst_early_close_and_unknown_terms | Dated authoritative holiday/early-close, broker maintenance and funding/contract records for the required historical intervals. The attempted ICE primary-source retrieval returned403; no content was treated as verified. |
| G6 — quantities, fees, swaps, conversion and statement money reconcile with no double counting. | BLOCKED_EXTERNAL_EVIDENCE | Typed ProtoOADeal/dispatch parser, account/environment/contract joins, partial-volume reconciliation, Decimal moneyDigits, signed swap/conversion and closing-total commission are implemented and tested. Code/tests: rebuild_evidence.parse_proto_deal/reconcile_position; raw-deal and money tests | Matched account-scoped entry AND exit deals, dispatch quotes, historical contract point value/conversion, settled fees and statement evidence. Existing13 demo entry observations are not round trips. |
| G7 — cost estimates have held-out error/uncertainty reports and explicit sparse-bucket fallbacks. | BLOCKED_EXTERNAL_EVIDENCE | Cost model must fit frozen reconciled training samples, precede held-out observations, preserve account/environment buckets and report block errors/uncertainty or an explicit sparse fallback. Code/tests: rebuild_evidence.ExpectedExecutionCost/StressReserve/RealisedMoney/verify_calibration; cost holdout test | Reconciled round-trip training and independent held-out cost-error blocks; no calibrated historical/live cost estimate currently exists. |
| G8 — selected adversarial fixtures and recorded-event replays establish bracket priority, amend timing, gaps, partial fills, restart/reconciliation and research/live decision parity. | BLOCKED_EXTERNAL_EVIDENCE | The unchanged native two-stage kernel now has a journal-bound partition/restart adapter. Midnight/amend/bracket/gap parity and typed partial-fill/ack/restart receipt checks pass offline. Code/tests: crates/jiaoyi-core/src/research/rebuild_continuity.rs; rebuild_evidence.verify_execution_lifecycle; Rust restart and Python lifecycle tests | Recorded broker acknowledgement/amend/fill/restart sequences and shadow decisions must be reconciled against the same frozen kernel. Existing live policies are not replaced by this research candidate. |
| G9 — split/overlap and trial-budget audit passes before opening evaluation. | BLOCKED_EVIDENCE_OR_VERSIONED_TRANSITION | Split/dependency overlap, protected dates, inspected preparation, embargo and trial-budget guards are implemented. The current candidate dates cannot be called unseen after their dependencies were inspected. Code/tests: rebuild_preflight.split_budget_guard/verify_transition; trial and inspection tests; frozen160-job preflight | A reviewed reservation/transition with a complete inspection ledger and eligible independent data. Original dates are preserved; no replacement dates or increased trial budget are silently chosen. |
| G10 — net expectancy, payoff/tail metrics, controls, per-class requirements and predeclared uncertainty criteria pass; failures remain published. | BLOCKED_FINANCIAL_EVIDENCE | Receipt-linked trade evaluation separates classes/accounts, requires resolution, predeclared controls, tail and block uncertainty criteria; no aggregate win constants are accepted. Code/tests: rebuild_evidence.financial_gate; unknown-evidence test | Independent qualified outcomes under verified costs and predeclared quantitative criteria. The frozen pilot explicitly cannot grant financial promotion. |
| G11 — shadow/canary execution and money reconcile within registered tolerances. | BLOCKED_EXTERNAL_EVIDENCE | Shadow/canary receipt parser checks authorization, account/policy identity, sends, acknowledgements, partial fills, restart book, decision divergence/latency and money. Code/tests: rebuild_evidence.verify_execution_lifecycle; execution test | Actual shadow/canary decision and broker receipts within registered tolerances; no orders or canary launched here. |
| G12 — owner risk rules, monitoring, rollback and per-strategy/account scorecards work under a complete trading lifecycle. | BLOCKED_EXTERNAL_EVIDENCE | Owner policy guard retains one minimum live position,250% entry margin floor, risk/protection/rollback checks and separate latest20 scorecards per account/strategy. Code/tests: rebuild_evidence.lifecycle_scorecard; existing production protection code unchanged | Complete observed lifecycle evidence for monitoring/protection/rollback and qualified per-account/strategy outcomes. CI and a synthetic operational checklist cannot prove this. |

## §2. Failure prevention and enforced boundaries

1. **No collection authority during this rebuild.** `rebuild_authority.py` and `jiaoyi_core::research::rebuild` refuse before I/O. No environment variable or prompt text can unpause this version. A later reviewed RFC transition must install a preflight-bound acquisition capability; removing the pause alone is not sufficient.
2. **Versioned identity.** The registry hashes the original Phase4 hypothesis, unchanged entry/exit source, native calendar, contract inventory, frozen cost assumptions and complete spent interval register, including Phase5 attempts. A caller cannot relabel a fixed-window variant as the frozen candidate.
3. **Preparation before ticks.** Required45-calendar-day native-session M30 slots are generated by the Rust metadata adapter. Every slot must be covered by a declared interval. Cached references must match hashes; unspent intervals are allowed only in preparation planning. Ticks require decoded causal OHLC and complete source attribution first. An interrupted or failed request remains spent.
4. **Full-path qualification.** Evaluation additionally requires the native start/end, fresh two-sided boundary quotes, original protocol lineage, continuous coverage and declared continuation coverage. A4-hour receipt cannot claim a72-hour resolution. An M30 approximation is not a tick path.
5. **Offline means subprocesses too.** The command installs Python audit hooks and Linux seccomp rules that descendants inherit. Native execution cannot create sockets. Git publication is a separate operation outside the offline evaluation process.
6. **Historical parity is a separate capability.** Known prior inputs, native receipts and published results are pinned by SHA256. This explicitly permits reproducing historical failure paths without treating them as complete-path qualification or a fresh independent trial.

These are enforced controls for supported entry points, not a promise of mathematical immunity against all future code changes or a different deployed binary. Regression tests and source identity checks must remain in the release path.

## §3. Unified inventory and exclusions

There are **544 definitions =384 prior +160 Phase5 scheduled slots**. The latter comprise158 completed tick sources and two failed sources, all retained. There are **616 M30 sources =377 prior +239 new**, plus75 cache reuse references that do not become75 new files. All399 Phase5 collection receipts reconcile; all1041 eligible market files are hash checked again after replay.

| Classification | All definitions | Phase5 batch |
| --- | --- | --- |
| coverage_incomplete_quarantined | 49 | 16 |
| failed_tick_source | 2 | 2 |
| inherited_repeat | 20 | 0 |
| outside_candidate_universe | 64 | 0 |
| protected_quarantined | 5 | 0 |
| tier1_native_complete | 37 | 2 |
| tier2_partial_prep | 239 | 72 |
| unverified_provenance_quarantined | 60 | 0 |
| window_mismatch_quarantined | 68 | 68 |

`tier1_native_complete` means the existing native-session/M30 scenario requirements are met. It does **not** prove gap-free execution, historical broker terms, calibrated costs or independence. The strict preflight qualification barrier remains separate. Tier2 is diagnostic and is never silently substituted for the frozen candidate.

The failed slots are phase5-NAS100-2024-08-12, phase5-NAS100-2024-08-14. Protected, inherited-repeat, unverified-provenance and incomplete-path receipts are retained as exclusions, never converted to successful zero-trade sessions. Five protected definitions and a duplicate receipt account for the prior protected receipt quarantine; failed sources have a separate reason.

### All-symbol accounting

| Symbol | Definitions | Tier1 scenario | Tier2 diagnostic | Other retained exclusions |
| --- | --- | --- | --- | --- |
| ADAUSD | 2 | 0 | 0 | 2 |
| AUDJPY | 2 | 0 | 0 | 2 |
| AUDUSD | 8 | 0 | 0 | 8 |
| AUS200 | 2 | 0 | 0 | 2 |
| AVAXUSD | 2 | 0 | 0 | 2 |
| BCHUSD | 2 | 0 | 0 | 2 |
| BTCUSD | 54 | 0 | 30 | 24 |
| CADJPY | 2 | 0 | 0 | 2 |
| CHFJPY | 2 | 0 | 0 | 2 |
| DOGEUSD | 2 | 0 | 0 | 2 |
| DOTUSD | 2 | 0 | 0 | 2 |
| ETHUSD | 22 | 0 | 16 | 6 |
| EURAUD | 2 | 0 | 0 | 2 |
| EURGBP | 2 | 0 | 0 | 2 |
| EURJPY | 2 | 0 | 0 | 2 |
| EURUSD | 8 | 0 | 0 | 8 |
| GBPJPY | 2 | 0 | 0 | 2 |
| GBPUSD | 8 | 0 | 0 | 8 |
| GER40 | 2 | 0 | 0 | 2 |
| JPN225 | 2 | 0 | 0 | 2 |
| LINKUSD | 2 | 0 | 0 | 2 |
| LTCUSD | 2 | 0 | 0 | 2 |
| NAS100 | 27 | 0 | 12 | 15 |
| NZDUSD | 8 | 0 | 0 | 8 |
| NatGas | 13 | 0 | 0 | 13 |
| SOLUSD | 2 | 0 | 0 | 2 |
| SpotBrent | 82 | 8 | 30 | 44 |
| SpotCrude | 82 | 8 | 30 | 44 |
| UK100 | 2 | 0 | 0 | 2 |
| US30 | 2 | 0 | 0 | 2 |
| US500 | 71 | 9 | 47 | 15 |
| USDCAD | 8 | 0 | 0 | 8 |
| USDCHF | 8 | 0 | 0 | 8 |
| USDJPY | 8 | 0 | 0 | 8 |
| XAGUSD | 27 | 1 | 23 | 3 |
| XAUUSD | 62 | 11 | 51 | 0 |
| XLMUSD | 2 | 0 | 0 | 2 |
| XPDUSD | 2 | 0 | 0 | 2 |
| XPTUSD | 2 | 0 | 0 | 2 |
| XRPUSD | 2 | 0 | 0 | 2 |

The original frozen160-job independent plan is preserved. Its full required M30-slot arrays are generated locally and hashed in the report index to avoid duplicating hundreds of thousands of timestamp values in Git. No date/window/cost replacement was selected. The missing preparation and original-window tick requirements remain visible.

## §4. Execution continuity and money evidence

`ContinuousReplay` wraps the existing two-stage kernel. Its checkpoint binds the original input, engine version, event count and rolling event digest. Recovery replays a verified journal prefix, preserving positions, original risk and pending amendments. A partition boundary is not an exit. The midnight fixture verifies that an old stop wins before a pending breakeven amendment after restart. Gap flags and unqualified money remain visible.

The external execution verifier handles sends, acknowledgements, partial fills, rejects, restart reconciliation and decision/latency comparisons. Actual broker records are still needed. The research candidate is not wired to place orders and does not replace the currently registered production strategy. G8/G11 therefore remain blocked.

Money evidence has separate types for expected execution costs, stress reserves and realised money. Round trips require matching account/environment/contract identity, total entry/exit quantity, settled fees, signed swaps, conversion and statement reconciliation. Closing-total commission is charged once. Recorded executable spread and realised broker net P&L are not charged a second time. Missing optional fee fields remain unknown. A raw `ProtoOADeal` parser retains filled/closed-volume and authenticated-account linkage.

Cost calibration fits frozen reconciled training data and evaluates later time blocks, reporting error and uncertainty. Entry-only demo slippage cannot satisfy the round-trip schema or silently calibrate live/historical costs. The existing3.74bp sensitivity and24bp stress reserve retain their original labels; no lower cost was chosen to improve results.

## §5. Verification evidence

- Native replay: **159/159 legacy,35/35 Phase2,2/2 Phase5B**, each twice, byte-identical to frozen receipts.
- Focused Python: **11 tests passed**, exercising the demonstrated preparation/window defects, overrides, tampering, process-level network denial, cost accounting, calendar dates, raw deal fields and lifecycle reconciliation.
- Focused Rust: **3 tests passed**, including service refusal before database/broker access and state recovery across midnight.
- Full research Python suite: 92 tests passed except one existing skip.
- All included test log hashes, code/binary hashes, source receipts and parity output hashes are in the machine ledger. No hard-coded backtest return is introduced.

Required remote release checks remain `check`, `docker` and `engineering`; the PR is merged only after their success on its current head. The final PR receipt records merge identity. This report alone does not certify which Railway deployment is currently serving traffic.

## §6. How another AI reproduces or reviews this work

Read this report, `docs/rebuild_rfc_offline_gates_completion_18368.json`, `docs/rebuild_rfc_registry_18368.json` and `docs/rebuild_rfc_offline_scope_18368.json`. Then inspect `ops/research/rebuild.py`, `rebuild_preflight.py`, `rebuild_evidence.py` and the new Rust continuity/authority modules. Source hashes identify the required cache bytes; a missing local source is a blocker, not permission to download or reconstruct it.

```bash
cargo build -p jiaoyi-core --locked --example native_target_sizing_replay --example manifest_pullback_replay --example tiered_pullback_replay --example replay_inventory_metadata
python3 ops/research/rebuild.py inventory
python3 ops/research/rebuild.py parity
python3 ops/research/rebuild.py preflight --file /path/to/registered-plan.json --stage evaluation
python3 ops/research/rebuild.py verify-evidence --kind money --file /path/to/round-trip.json
```

`freeze` creates the immutable engineering scope before the official `run`; commit those inputs first. `run` refuses source drift and recomputes the inventory/parity evidence before rendering this report. The historical scripts remain forensic adapters for prior commits, not alternative collection authority. Linux libseccomp is required; an unavailable network barrier causes refusal.

## §7. The next decision and the finite checks

Keep collection paused. First obtain dated calendar/contract and matched round-trip execution evidence through a separately authorised capture routine; this task makes no such request. Feed those receipts through the typed verifiers. Review cost uncertainty and original-plan dependency inspection before declaring a new reservation. Only an explicit versioned transition can resume collection with decoded preparation, native-window and spent-interval proofs.

The release/qualification sequence is finite: **identity → complete preparation → qualified path → exact baseline parity → reconciled costs → independent split/trial audit → quantitative acceptance → shadow/canary lifecycle**. A failed gate stops promotion and records the cause. It does not launch another candidate search, consume more sessions or loosen the cost assumptions.

Raw market evidence is indexed once; the partition-store implementation uses immutable content hashes, deterministic gzip and mandatory consumer/retention references. Existing raw sources were neither duplicated into Git nor purged. The machine ledger retains the mixed530 unavailable historical references, without describing them as530 demonstrated missing broker receipts.

## §8. Limits that remain explicit

- Dated calendar test fixtures prove code semantics, not the historical CFD dealing schedule. The attempted primary ICE calendar page returned403, so G5 is not promoted on that basis.
- Journal replay proves deterministic recovery for tested paths; actual broker acknowledgements, partial execution and latency still need reconciliation.
- No unseen holdout, financial acceptance, account goal achievement or new trading edge is claimed. The protected19 August–5 September2025 interval remains sealed.
- Requested journal slot`§LOG.138` was already occupied. This work appends the next available CODEX journal entry without renumbering published history; GROK's latest retained serial remains№17,703.

Machine result SHA256: `31d6f174a0fe9821c9f83e4e91cc55acead8de2b958ee6ee3e2d3caf3b99c673`. Draft for review.
