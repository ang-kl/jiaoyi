# Rebuild RFC: Gates 5–9 evidence and reservation audit

№18,370 §§1–10 · Deal-history cutoff 08-11-2026 17:57:51 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Read-only evidence capture succeeded on all four accounts.** It returned 572 broker deals, including 284 closing fills. **266 closing fills match retained statement money and displayed prices; 0 statement mismatches.** All 284 closing-money calculations agree with the cent-unit/conversion diagnostic within one broker money unit. Current contracts and schedules are captured for 68 account/symbol combinations.

**RFC gates remain 4/12.** This audit materially improves the money evidence and identifies exact reservation conflicts; it does not manufacture historical calendar authority, paired exit-dispatch costs or an independent test. G5–G9 remain blocked for the reasons below. Zero new price-history requests, zero orders, zero validation access, zero live-rule changes.

**Goal progress: zero new independently qualified trades and no measured win-rate improvement.** The goal remains 73% net wins and 1:1.38 per strategy/account over the latest 20 closes. 15/20 is required. Historical/manual trades in this evidence inventory do not become frozen-candidate wins.

## §1. Evidence captured, scope and provenance

Capture covers 24 October 2026 through the timestamp above. There were 12 broker read queries (four catalogs, four contract responses, four complete deal pages), 10 authentication messages including the initial two-message probe, and 15 service GETs. No subscription, price-history, refresh-token or trading message is allowed by the new exporter. The price collector remains paused. Failed, truncated or out-of-window evidence cannot be labelled complete.

| Account | Type | Deals | Closing fills | Complete quantities | Full statement-matched positions |
| --- | --- | --- | --- | --- | --- |
| 49236051 | demo | 146 | 74 | 72 | 68 |
| 46130058 | demo | 180 | 89 | 87 | 85 |
| 47790949 | demo | 170 | 85 | 83 | 81 |
| 43002148 | live | 76 | 36 | 29 | 26 |

The earlier 77 unique raw deals across five retained broker files are all present and byte-equivalent as canonical objects in the new capture. Fifteen uploaded statement files are hash checked and deduplicated by account plus closing deal ID. The bundle retains relevant raw contracts, raw deals and catalog rows; unused catalog entries and duplicate raw history are omitted from Git. Local full-response hashes remain listed. No credential is included.

## §2. Gate status and what is still missing

| Gate | Status | Exact remaining requirement |
| --- | --- | --- |
| G5 | BLOCKED_EXTERNAL_EVIDENCE | Current broker contracts/schedules and dated 2026 exceptions captured. No effective-dated 2023–2025 broker/venue history or historical tick-size authority; frozen candidate calendar qualification remains blocked. |
| G6 | BLOCKED_EXTERNAL_EVIDENCE | Realised closing money and complete captured quantities reconcile for the published statement-matched subset. Historical contract unit/lot/point-value authority and complete period statement/quantity coverage remain missing; no full gate claim from a partial subset. |
| G7 | BLOCKED_EXTERNAL_EVIDENCE | 17 measured demo entry observations but zero paired exit-dispatch observations. No eligible round-trip execution-cost train/holdout sample; no cost fit or invented holdout error. Retain explicit uncalibrated sensitivities. |
| G8 | BLOCKED_EXTERNAL_EVIDENCE | Broker deals join stored execution summaries. Public summaries omit raw acknowledgement/amend/dispatch payloads, restart books and frozen-candidate shadow decisions. Receipt presence is not research/live execution parity. |
| G9 | AUDITED_BLOCKED_INDEPENDENCE_AND_COVERAGE | Exact reservation audit completed, not an unseen reservation: all 160 preparation histories overlap inspected data,80 tick windows overlap prior inspection,36 planned M30 slices have spent missing bars. Original 160 tick windows absent. New RFC transition/evidence reconciliation required; collection stays paused. |

G1–G4 retain PR #197's engineering status. This run does not rerun trading backtests or convert a small reconciled subset into whole-system qualification. No profit threshold, candidate rule, session or cost assumption was changed.

## §3. Current broker calendar and contract authority

| Account | Symbol | Broker timezone | Lot size (cents) | Minimum / step | Dated exceptions |
| --- | --- | --- | --- | --- | --- |
| 49236051 | XAUUSD | America/New_York | 10000 | 100 / 100 | 2 |
| 49236051 | XAGUSD | America/New_York | 500000 | 5000 / 5000 | 2 |
| 49236051 | US500 | America/New_York | 100 | 10 / 10 | 2 |
| 49236051 | NAS100 | America/New_York | 100 | 10 / 10 | 2 |
| 49236051 | BTCUSD | America/New_York | 100 | 1 / 1 | 1 |
| 49236051 | ETHUSD | America/New_York | 100 | 1 / 1 | 1 |
| 49236051 | SpotBrent | America/New_York | 10000 | 100 / 100 | 2 |
| 49236051 | SpotCrude | America/New_York | 10000 | 100 / 100 | 2 |
| 46130058 | XAUUSD | America/New_York | 10000 | 100 / 100 | 2 |
| 46130058 | XAGUSD | America/New_York | 500000 | 5000 / 5000 | 2 |
| 46130058 | US500 | America/New_York | 100 | 10 / 10 | 2 |
| 46130058 | NAS100 | America/New_York | 100 | 10 / 10 | 2 |
| 46130058 | BTCUSD | America/New_York | 100 | 1 / 1 | 1 |
| 46130058 | ETHUSD | America/New_York | 100 | 1 / 1 | 1 |
| 46130058 | SpotBrent | America/New_York | 10000 | 100 / 100 | 2 |
| 46130058 | SpotCrude | America/New_York | 10000 | 100 / 100 | 2 |
| 47790949 | XAUUSD | America/New_York | 10000 | 100 / 100 | 2 |
| 47790949 | XAGUSD | America/New_York | 500000 | 5000 / 5000 | 2 |
| 47790949 | US500 | America/New_York | 100 | 10 / 10 | 2 |
| 47790949 | NAS100 | America/New_York | 100 | 10 / 10 | 2 |
| 47790949 | BTCUSD | America/New_York | 100 | 1 / 1 | 1 |
| 47790949 | ETHUSD | America/New_York | 100 | 1 / 1 | 1 |
| 47790949 | SpotBrent | America/New_York | 10000 | 100 / 100 | 2 |
| 47790949 | SpotCrude | America/New_York | 10000 | 100 / 100 | 2 |
| 43002148 | XAUUSD | America/New_York | 10000 | 100 / 100 | 2 |
| 43002148 | XAGUSD | America/New_York | 500000 | 5000 / 5000 | 2 |
| 43002148 | US500 | America/New_York | 100 | 10 / 10 | 2 |
| 43002148 | NAS100 | America/New_York | 100 | 10 / 10 | 2 |
| 43002148 | BTCUSD | America/New_York | 100 | 1 / 1 | 1 |
| 43002148 | ETHUSD | America/New_York | 100 | 1 / 1 | 1 |
| 43002148 | SpotBrent | America/New_York | 10000 | 100 / 100 | 2 |
| 43002148 | SpotCrude | America/New_York | 10000 | 100 / 100 | 2 |

The raw evidence includes weekly intervals in the broker's named timezone, dated holiday exceptions, lot/minimum/step volumes, commission type/rate and funding fields. The parser converts dated exceptions with IANA timezones and rejects ambiguous local timestamps. For example, a 26 November 2026 exception starting 22:30 Europe/Moscow converts to 19:30 UTC. New York 09:30 converts to 13:30 UTC in summer and 14:30 UTC in winter.

These are **broker snapshots taken now**, not effective-dated 2023–2025 contracts or exchange calendars. `digits` supplies display precision, not conclusive minimum tick increment. No absent holiday is interpreted as proof of a normal historical session. Obtain dated broker/venue notices and historical contract effective intervals for the frozen dates before claiming G5.

## §4. Money reconciliation, reversals and actual costs

The ledger keeps broker `grossProfit`, signed swap, closing-volume commission and conversion fee in Decimal deposit-currency units. Closing commission is counted once; opening deal commission is not added again. Recorded bid/ask effects already in realised P&L are not subtracted a second time. Missing fee/conversion fields refuse reconciliation.

Actual netting reversals divide a single fill into its closing quantity and residual new opening quantity. This matters for manual NatGas/JPN225 paths: filled volume can exceed `closedVolume`. A position's label and account remain attached; incomplete opening history or a still-open remainder is retained as incomplete, not treated as a closed losing/winning trade.

Two initially apparent price mismatches were statement display rounding: the broker retains weighted entry precision beyond the CSV. The parser permits only half the CSV's displayed decimal unit for prices; **net money and commission must still match exactly**. The JSON gives every closing-deal comparison, raw-deal hash and source statement line.

The source-based quantity/gross check uses cTrader's cents-of-units convention and each close's recorded conversion rate. It is explicitly a diagnostic until historical point/lot semantics are authoritative. Whole-period coverage and effective-dated contracts remain G6 gaps, even though the statement-matched subset is useful evidence.

## §5. Cost calibration and execution parity

There are **17 actual demo entry dispatch-to-fill observations**, up from 13, but **zero paired exit-dispatch observations**. Entry price or broker stop/target level is not an exit dispatch quote. No round-trip cost model was fitted, no empty holdout was called a pass, and no demo costs were transferred to live or to 2023. The 3.74 bp sensitivity and 24 bp stress reserve remain labelled assumptions. Sparse buckets remain uncalibrated.

The service returned 1233 order rows and 1364 execution-summary rows; 456 summary rows join a captured broker deal by account/deal/position. These counts are not unique replay episodes. The public view strips raw payloads. It does not expose a full amend/ack/restart book or contemporaneous decisions of the frozen research candidate.

`ops/research/export_execution_evidence.sql` is the minimal bounded, read-only private database exporter for stored execution payloads and existing entry-dispatch records. It has a 31-day bound, 2,000-row cap and `has_more`, read-only transaction, timeout and four-account allowlist. It was not run against the local development database as if it were production. It cannot recreate exit quotes or restart evidence that was never recorded. No live logging change or schema expansion was deployed.

## §6. Exact original reservation: data availability versus independence

The audit preserves the Phase 4 `s3_macro_pullback_v1` dates, native sessions, 3.74 bp admission and 24 bp repricing. It indexes 544 episode definitions and 616 M30 source files, verifies their eligible receipts, and leaves protected dependencies quarantined.

| Original M30 slice status | Slices |
| --- | --- |
| blocked_spent_missing_bars | 36 |
| complete_verified_cache | 46 |
| missing_unspent | 128 |
| no_native_slots | 32 |
| partial_cache_unspent_gaps | 14 |

Of 256 original request slices, 46 have every required native bar cached; 178 lack bars; 32 contain no required native-session bars. Of the incomplete slices, 36 contain spent missing bars and cannot be re-requested. 128 are wholly missing/unspent and 14 have reusable cached bars plus unspent gaps. These are request-slice counts, not file counts or successful episode counts.

**All 160 required native tick windows are absent. 80 overlap previously inspected price intervals; all 160 have some inspected preparation.** The registry considers the whole dependency chain. Cached reuse is technically available, but it cannot establish that an already inspected dependency is unseen. No original date was silently replaced and no reservation was activated.

Preparation overlap alone is not proof of look-ahead or outcome leakage. It fails the current frozen dependency-inspection rule; a different reuse policy needs an explicit versioned decision. The 80 overlapping evaluation windows are a separate issue: their price paths were already available through retained M30 data, even though native tick coverage is absent.

| Symbol | Jobs | Inspected tick windows | Inspected preparation | Cached / unspent missing / spent missing slots* |
| --- | --- | --- | --- | --- |
| BTCUSD | 20 | 10 | 20 | 15874 / 26800 / 526 |
| ETHUSD | 20 | 10 | 20 | 15874 / 26800 / 526 |
| NAS100 | 20 | 10 | 20 | 3037 / 5240 / 43 |
| SpotBrent | 20 | 10 | 20 | 6009 / 10580 / 51 |
| SpotCrude | 20 | 10 | 20 | 6019 / 10580 / 41 |
| US500 | 20 | 10 | 20 | 3037 / 5240 / 43 |
| XAGUSD | 20 | 10 | 20 | 6019 / 10580 / 41 |
| XAUUSD | 20 | 10 | 20 | 6019 / 10580 / 41 |

*Slot totals here count each episode's dependency separately; shared bars appear more than once. The machine manifest lists deduplicated partition identities and their consumers. `missing_m30_segments_to_fetch=178` records incompleteness, not permission to fetch all 178; spent gaps remain blocked.*

## §7. Actual preflight receipts and code identity

The current main command refuses the Phase 4 source identity with `frozen_engine_changed`: Phase 5 added the separately named fixed-window wrapper to `manifest_pullback.rs`. The exit source is unchanged. The audit does not mask that hash difference or change the frozen registry.

Using the separately retained exact Phase 4 checkout, all core/example source hashes and frozen binaries match PR #196's receipt, and native calendar metadata matches current metadata. Its real preflight refuses preparation/ticks with `spent_tick_request`; evaluation refuses `preparation_must_be_decoded_before_ticks`. The inspection guard refuses `already_inspected_m30`. All receipts, hashes, planned jobs and preparation partitions are in the JSON.

G9 therefore has an **exact audit receipt**, not an independent-reservation pass. Before any future collection, use an explicitly reviewed RFC transition to choose either transparent reused-data diagnostics or a newly reserved, genuinely unspent matrix. Preserve the frozen engine as a separately identified adapter; do not weaken the source guard to accommodate the changed wrapper. Reconcile unavailable older references and dated calendar authority first. Price collection remains paused.

## §8. Verification and next actions

Seven focused regressions pass: forbidden broker operations/accounts; truncated pagination; signed financing and commission; netting reversal quantities; statement rounding versus strict money; timezone/dated-snapshot authority; cached/unspent/spent preparation partitioning. The full research suite ran 99 tests with one existing skip and no failures. The deterministic audit rechecks source hashes and all 284 closing fills. CI must pass `check`, `docker` and `engineering` before merge.

1. Obtain historical broker/venue contract and calendar effective-date evidence for the intended candidate dates.
2. Supply newer account statements for complete bounded-period reconciliation; preserve incomplete/netting paths in denominators.
3. Export the existing private execution event records from the actual service database, if available; collect missing both-leg dispatch and restart/shadow evidence only under a specifically approved subsequent live instrumentation change.
4. Freeze the independent cost split and uncertainty protocol once paired cost evidence exists. Do not lower research costs to make outcomes pass.
5. Review a versioned candidate/engine and reservation transition. Do not fetch the 178 incomplete slices blindly or call the 160 original episodes unspent.

## §9. Instructions for Gemini or another reviewer

Read this report first, then `docs/rebuild_rfc_gates_5_to_9_audit_18370.json` and `docs/rebuild_rfc_gates_5_to_9_evidence_18370.json`. Inspect `broker_evidence.py`, `broker_money.py`, `broker_calendar.py`, `rebuild_reservation.py` and `test_broker_evidence.py` in `ops/research/`. Compare gate definitions with the original RFC, not the user's hoped-for gate count.

Challenge the money and quantity joins, snapshot-versus-historical authority, absent paired exit costs, and preparation inspection ledger. Do not interpret money reconciliation, CI, or high win rates from old development dates as financial qualification. All original 73%/1:1.38/account boundaries remain in force.

## §10. Trace and publication

Source snapshot and code hashes are in the machine ledger. Requested journal slot §LOG.139 was already occupied; the append-only journal uses the next free slot. Some chat progress timestamps ran ahead of the actual clock; §7 corrected them to 18:08 SGT. Artifact timestamps derive from captured UTC timestamps, not those chat labels.

This report is the completed audit; the final PR comment/body and owner review block record the actual CI results and merge SHA. It does not certify a Railway runtime deployment. No live program files changed.
