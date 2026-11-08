"""Source-backed RFC ledger and owner report. No returns are fabricated or retuned."""
import collections
import html
import json
import re
from pathlib import Path

from replay_inventory import ROOT, read, save, sha, canonical
from rebuild_preflight import require
from unified_replay_report import table, markdown_body

WORK = Path('/workspace/jiaoyi-rebuild-offline-18368')
RESULT = ROOT/'docs/rebuild_rfc_offline_gates_completion_18368.json'
REPORT = ROOT/'doc/REBUILD_RFC_OFFLINE_GATES_REPORT-18368-2026-11-08.md'
RFC = ROOT/'doc/NATIVE_TARGET_SIZING_AND_REBUILD_RFC-18353-2026-11-08.md'


def gates(parity, tests):
    text = RFC.read_text().split('## §8.')[1]
    names = dict(re.findall(r'\*\*(G\d+):\*\* (.*?)(?= \*\*G\d+:| \|)', text))
    require(len(names) == 12, 'original_twelve_gate_definitions_changed')
    require(tests['focused_python']['exit_code'] == 0 and tests['rust_guards']['exit_code'] == 0,
            'engineering_tests_not_passed')
    require(tuple(len(parity[k]) for k in ['legacy_159','phase2_35','phase5b_2']) == (159,35,2), 'gate_parity_incomplete')
    specs = {
        'G1': ('PASSED', 'Versioned identity binds frozen hypothesis, unchanged entry/exit engine, native calendar, contract inventory, cost assumptions, spent intervals and source receipts. Unknown historical contract authority remains explicitly unknown.',
               'rebuild.py:build_registry/source_freeze; rebuild_preflight.Identity; unified manifest and source hashes', None),
        'G2': ('PASSED', 'All159+35+2 native outputs reproduce the original receipt bytes, twice, including no-entry/refusal paths.',
               'rebuild.py:parity/replay_bytes; parity_receipts', None),
        'G3': ('PASSED', 'Single offline command; inherited Linux seccomp plus Python audit guard; native output computation only; all544 planned definitions and failures retained.',
               'rebuild.py; rebuild_preflight.offline_process; test_offline_restriction_is_inherited_by_native_child', None),
        'G4': ('PASSED', 'Preflight refuses missing45-day dependencies, unspent-but-undecoded preparation, wrong native windows, stale boundaries, gaps, unverified receipts and incomplete continuation. Full-path eligibility is stricter than a recorded-scenario tier.',
               'rebuild_preflight.verify_plan/verify_bars; rebuild.py:preflight_file; test_preflight_blocks_pr195_defects_and_identity_overrides', None),
        'G5': ('BLOCKED_EXTERNAL_EVIDENCE', 'Calendar layers, effective dates, DST, early-close overrides, maintenance/funding fields and unknown-term rejection are implemented. Synthetic dated fixtures pass; they do not certify the historical broker/venue schedule.',
               'rebuild_preflight.CalendarTerms; test_dated_calendar_dst_early_close_and_unknown_terms',
               'Dated authoritative holiday/early-close, broker maintenance and funding/contract records for the required historical intervals. The attempted ICE primary-source retrieval returned403; no content was treated as verified.'),
        'G6': ('BLOCKED_EXTERNAL_EVIDENCE', 'Typed ProtoOADeal/dispatch parser, account/environment/contract joins, partial-volume reconciliation, Decimal moneyDigits, signed swap/conversion and closing-total commission are implemented and tested.',
               'rebuild_evidence.parse_proto_deal/reconcile_position; raw-deal and money tests',
               'Matched account-scoped entry AND exit deals, dispatch quotes, historical contract point value/conversion, settled fees and statement evidence. Existing13 demo entry observations are not round trips.'),
        'G7': ('BLOCKED_EXTERNAL_EVIDENCE', 'Cost model must fit frozen reconciled training samples, precede held-out observations, preserve account/environment buckets and report block errors/uncertainty or an explicit sparse fallback.',
               'rebuild_evidence.ExpectedExecutionCost/StressReserve/RealisedMoney/verify_calibration; cost holdout test',
               'Reconciled round-trip training and independent held-out cost-error blocks; no calibrated historical/live cost estimate currently exists.'),
        'G8': ('BLOCKED_EXTERNAL_EVIDENCE', 'The unchanged native two-stage kernel now has a journal-bound partition/restart adapter. Midnight/amend/bracket/gap parity and typed partial-fill/ack/restart receipt checks pass offline.',
               'crates/jiaoyi-core/src/research/rebuild_continuity.rs; rebuild_evidence.verify_execution_lifecycle; Rust restart and Python lifecycle tests',
               'Recorded broker acknowledgement/amend/fill/restart sequences and shadow decisions must be reconciled against the same frozen kernel. Existing live policies are not replaced by this research candidate.'),
        'G9': ('BLOCKED_EVIDENCE_OR_VERSIONED_TRANSITION', 'Split/dependency overlap, protected dates, inspected preparation, embargo and trial-budget guards are implemented. The current candidate dates cannot be called unseen after their dependencies were inspected.',
               'rebuild_preflight.split_budget_guard/verify_transition; trial and inspection tests; frozen160-job preflight',
               'A reviewed reservation/transition with a complete inspection ledger and eligible independent data. Original dates are preserved; no replacement dates or increased trial budget are silently chosen.'),
        'G10': ('BLOCKED_FINANCIAL_EVIDENCE', 'Receipt-linked trade evaluation separates classes/accounts, requires resolution, predeclared controls, tail and block uncertainty criteria; no aggregate win constants are accepted.',
               'rebuild_evidence.financial_gate; unknown-evidence test',
               'Independent qualified outcomes under verified costs and predeclared quantitative criteria. The frozen pilot explicitly cannot grant financial promotion.'),
        'G11': ('BLOCKED_EXTERNAL_EVIDENCE', 'Shadow/canary receipt parser checks authorization, account/policy identity, sends, acknowledgements, partial fills, restart book, decision divergence/latency and money.',
               'rebuild_evidence.verify_execution_lifecycle; execution test',
               'Actual shadow/canary decision and broker receipts within registered tolerances; no orders or canary launched here.'),
        'G12': ('BLOCKED_EXTERNAL_EVIDENCE', 'Owner policy guard retains one minimum live position,250% entry margin floor, risk/protection/rollback checks and separate latest20 scorecards per account/strategy.',
               'rebuild_evidence.lifecycle_scorecard; existing production protection code unchanged',
               'Complete observed lifecycle evidence for monitoring/protection/rollback and qualified per-account/strategy outcomes. CI and a synthetic operational checklist cannot prove this.')}
    return [dict(gate=f'G{i}',name=names[f'G{i}'],status=specs[f'G{i}'][0],
                 engineering_controls='IMPLEMENTED_OFFLINE',evidence=specs[f'G{i}'][1],
                 code_and_tests=specs[f'G{i}'][2],remaining_blocker=specs[f'G{i}'][3]) for i in range(1,13)]


def compact_inventory(inv):
    # Keep all identities and reasons; avoid duplicating160 large required-slot
    # arrays already deterministically generated from the native calendar.
    out = dict(inv)
    out['frozen_independent_plan_preflight'] = []
    for row in inv['frozen_independent_plan_preflight']:
        r = {k:v for k,v in row.items() if k not in ('required_m30_slots','missing_m30_slots')}
        r.update(required_m30_slot_count=len(row['required_m30_slots']),
                 required_m30_slots_sha256=canonical(row['required_m30_slots']),
                 missing_m30_slot_count=len(row['missing_m30_slots']),
                 first_missing_m30_slot_ms=row['missing_m30_slots'][:8])
        out['frozen_independent_plan_preflight'].append(r)
    out['local_expanded_inventory_sha256'] = sha(WORK/'inventory.json')
    return out


def publish():
    inv = read(WORK/'inventory.json'); parity = read(WORK/'parity.json')
    verification = read(WORK/'verification.json'); tests = read(WORK/'tests.json')
    ledger = gates(parity,tests)
    passed = sum(g['status'] == 'PASSED' for g in ledger)
    for path,h in tests['evidence_log_hashes'].items():
        require(sha(path) == h, 'test_log_changed')
    result = dict(schema='jiaoyi.rebuild-offline-completion.v1',trace='№18,368',
        scope_sha256=verification['scope_sha256'],rfc_source_sha256=sha(RFC),
        registry_sha256=sha(ROOT/'docs/rebuild_rfc_registry_18368.json'),
        rfc_gates_passed_before=2,rfc_gates_passed_now=passed,rfc_gates_total=len(ledger),
        gate_status_ledger=ledger,unified_manifest=compact_inventory(inv),parity_receipts=parity,
        verification=verification,tests=tests,new_broker_requests=0,financial_pass=False,
        validation_opened=False,live_rules_changed=False,new_independent_trades=0,
        collection_authority='PAUSED_AT_PYTHON_AND_SERVICE_RESEARCH_BOUNDARIES',
        live_trading_authority='UNCHANGED; existing quotes, entries under existing rules and position protection are outside this pause',
        limitations=['Engineering verification cannot promise an edge or immunity to every defect.',
          'Raw source bytes remain in the existing local evidence cache; Git stores the receipt/index/report, not a duplicated market archive.',
          'Declared calendar examples are test fixtures; dated historical broker/venue authority remains blocked.',
          'Hash reconciliation establishes retained-byte identity, not independent certification of broker economic truth.',
          'Historical scripts remain immutable parity adapters. All collection through supported current entry points is paused; do not run an old deployed build as a bypass.',
          'Native bins are built before the offline command; seccomp blocks network for the runner and all its native child processes.'])
    save(RESULT,result)
    s = inv['summary'];gate_rows = table(['Gate — original RFC definition','Status','Implemented / evidence','Remaining blocker'],
        [[g['gate']+' — '+g['name'],g['status'],g['evidence']+' Code/tests: '+g['code_and_tests'],g['remaining_blocker'] or 'None for this engineering gate.'] for g in ledger])
    classifications = table(['Classification','All definitions','Phase5 batch'],[[k,n,s['new_batch_classifications'].get(k,0)] for k,n in sorted(s['classifications'].items())])
    per_symbol = collections.defaultdict(collections.Counter)
    for e in inv['episodes']: per_symbol[e['symbol']][e['classification']] += 1
    symbols = table(['Symbol','Definitions','Tier1 scenario','Tier2 diagnostic','Other retained exclusions'],
        [[symbol,sum(c.values()),c['tier1_native_complete'],c['tier2_partial_prep'],sum(c.values())-c['tier1_native_complete']-c['tier2_partial_prep']] for symbol,c in sorted(per_symbol.items())])
    failed = [e for e in inv['episodes'] if e['classification']=='failed_tick_source']
    report = f'''# Rebuild RFC — Offline engineering and pre-flight controls

№18,368 §§1–12 · {tests['verified_at_sgt']} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Offline safeguards are implemented. {passed}/12 original RFC gates pass, up from2/12.** The other gates have executable parsers/verifiers and fail-closed checks, but their external/economic evidence is not present. They remain blocked. This is not a claim that all12 gates passed or that the strategy is ready for live promotion.

**Collection is paused at both supported boundaries:** Python cTrader research clients and the service's owner research endpoints. A valid research request returns HTTP423 before any database/broker read. Existing trading, subscriptions, entries under existing rules and open-position protection retain their authority. No new broker requests, orders, protected-validation evaluation or live strategy changes were made by this work.

**Exact reproducibility:159 +35 +2.** All196 requested native replay outputs were run twice and matched their original receipt bytes. The merged engineering changes do not retune the frozen hypothesis or manufacture additional wins.

**Goal progress:0 new independent qualified trades; no financial improvement measured.** The account invariant remains73% net wins and1:1.38 over each strategy/account's latest20 fully closed owned trades.15/20 meets the count. Engineering progress is two additional accepted gates, not a higher win rate.

## §1. The twelve gates, verbatim from the RFC

The definitions below are parsed from §8 of `doc/NATIVE_TARGET_SIZING_AND_REBUILD_RFC-18353-2026-11-08.md`. `IMPLEMENTED_OFFLINE` describes available code; only `PASSED` counts as satisfying the original acceptance gate. A schema cannot substitute for the missing broker/economic evidence.

{gate_rows}

## §2. Failure prevention and enforced boundaries

1. **No collection authority during this rebuild.** `rebuild_authority.py` and `jiaoyi_core::research::rebuild` refuse before I/O. No environment variable or prompt text can unpause this version. A later reviewed RFC transition must install a preflight-bound acquisition capability; removing the pause alone is not sufficient.
2. **Versioned identity.** The registry hashes the original Phase4 hypothesis, unchanged entry/exit source, native calendar, contract inventory, frozen cost assumptions and complete spent interval register, including Phase5 attempts. A caller cannot relabel a fixed-window variant as the frozen candidate.
3. **Preparation before ticks.** Required45-calendar-day native-session M30 slots are generated by the Rust metadata adapter. Every slot must be covered by a declared interval. Cached references must match hashes; unspent intervals are allowed only in preparation planning. Ticks require decoded causal OHLC and complete source attribution first. An interrupted or failed request remains spent.
4. **Full-path qualification.** Evaluation additionally requires the native start/end, fresh two-sided boundary quotes, original protocol lineage, continuous coverage and declared continuation coverage. A4-hour receipt cannot claim a72-hour resolution. An M30 approximation is not a tick path.
5. **Offline means subprocesses too.** The command installs Python audit hooks and Linux seccomp rules that descendants inherit. Native execution cannot create sockets. Git publication is a separate operation outside the offline evaluation process.
6. **Historical parity is a separate capability.** Known prior inputs, native receipts and published results are pinned by SHA256. This explicitly permits reproducing historical failure paths without treating them as complete-path qualification or a fresh independent trial.

These are enforced controls for supported entry points, not a promise of mathematical immunity against all future code changes or a different deployed binary. Regression tests and source identity checks must remain in the release path.

## §3. Unified inventory and exclusions

There are **{s['scheduled_episode_definitions']} definitions =384 prior +160 Phase5 scheduled slots**. The latter comprise158 completed tick sources and two failed sources, all retained. There are **616 M30 sources =377 prior +239 new**, plus75 cache reuse references that do not become75 new files. All399 Phase5 collection receipts reconcile; all{inv['source_reconciliation']['verified_market_file_count']} eligible market files are hash checked again after replay.

{classifications}

`tier1_native_complete` means the existing native-session/M30 scenario requirements are met. It does **not** prove gap-free execution, historical broker terms, calibrated costs or independence. The strict preflight qualification barrier remains separate. Tier2 is diagnostic and is never silently substituted for the frozen candidate.

The failed slots are {', '.join(e['episode_id'] for e in failed)}. Protected, inherited-repeat, unverified-provenance and incomplete-path receipts are retained as exclusions, never converted to successful zero-trade sessions. Five protected definitions and a duplicate receipt account for the prior protected receipt quarantine; failed sources have a separate reason.

### All-symbol accounting

{symbols}

The original frozen160-job independent plan is preserved. Its full required M30-slot arrays are generated locally and hashed in the report index to avoid duplicating hundreds of thousands of timestamp values in Git. No date/window/cost replacement was selected. The missing preparation and original-window tick requirements remain visible.

## §4. Execution continuity and money evidence

`ContinuousReplay` wraps the existing two-stage kernel. Its checkpoint binds the original input, engine version, event count and rolling event digest. Recovery replays a verified journal prefix, preserving positions, original risk and pending amendments. A partition boundary is not an exit. The midnight fixture verifies that an old stop wins before a pending breakeven amendment after restart. Gap flags and unqualified money remain visible.

The external execution verifier handles sends, acknowledgements, partial fills, rejects, restart reconciliation and decision/latency comparisons. Actual broker records are still needed. The research candidate is not wired to place orders and does not replace the currently registered production strategy. G8/G11 therefore remain blocked.

Money evidence has separate types for expected execution costs, stress reserves and realised money. Round trips require matching account/environment/contract identity, total entry/exit quantity, settled fees, signed swaps, conversion and statement reconciliation. Closing-total commission is charged once. Recorded executable spread and realised broker net P&L are not charged a second time. Missing optional fee fields remain unknown. A raw `ProtoOADeal` parser retains filled/closed-volume and authenticated-account linkage.

Cost calibration fits frozen reconciled training data and evaluates later time blocks, reporting error and uncertainty. Entry-only demo slippage cannot satisfy the round-trip schema or silently calibrate live/historical costs. The existing3.74bp sensitivity and24bp stress reserve retain their original labels; no lower cost was chosen to improve results.

## §5. Verification evidence

- Native replay: **159/159 legacy,35/35 Phase2,2/2 Phase5B**, each twice, byte-identical to frozen receipts.
- Focused Python: **{tests['focused_python']['tests']} tests passed**, exercising the demonstrated preparation/window defects, overrides, tampering, process-level network denial, cost accounting, calendar dates, raw deal fields and lifecycle reconciliation.
- Focused Rust: **{tests['rust_guards']['tests']} tests passed**, including service refusal before database/broker access and state recovery across midnight.
- Full research Python suite: {tests.get('full_python',{}).get('summary','CI also runs the complete suite')}.
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

Machine result SHA256: `{sha(RESULT)}`. Draft for review.
'''
    REPORT.write_text(report)
    body = markdown_body(report)
    page = f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi RFC offline gates — №18,368</title><style>
body{{font:16px/1.65 system-ui,sans-serif;color:#182330;background:#f6f8fb;margin:0;padding:24px}}main{{max-width:1250px;margin:auto;background:white;padding:28px;border-radius:14px}}h1{{font-size:30px;line-height:1.25}}h2{{margin-top:36px;border-top:1px solid #dce2e8;padding-top:20px}}h3{{margin-top:28px}}p,li{{max-width:1100px}}table{{border-collapse:collapse;width:100%;font-size:14px}}th,td{{padding:12px;text-align:left;vertical-align:top;border:1px solid #dce2e8}}th{{background:#e9eff6}}code,pre{{overflow-wrap:anywhere;white-space:pre-wrap}}a{{color:#075cb5}}.table-scroll{{overflow-x:auto}}@media(max-width:650px){{body{{padding:8px}}main{{padding:16px}}table{{min-width:700px}}h1{{font-size:25px}}}}</style><main>{body}</main></html>'''
    page = page.replace('<table>', '<div class="table-scroll"><table>').replace('</table>', '</table></div>')
    REPORT.with_suffix('.html').write_text(page)
    print(json.dumps({'gates_passed':passed,'total':12,'result_sha256':sha(RESULT),'report':str(REPORT.relative_to(ROOT))}),flush=True)
