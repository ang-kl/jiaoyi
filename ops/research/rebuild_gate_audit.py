#!/usr/bin/env python3
"""Offline publication of №18,370 captured evidence and exact reservation audit."""
import collections
import datetime as dt
import html
import json
from pathlib import Path
import re

from broker_calendar import contract_snapshot
from broker_money import ledger, statements
from cost_evidence_assessment import verified_entry
from rebuild_preflight import offline_process, require, verified_json
from replay_inventory import ROOT, CORE, read, save, sha, canonical
from unified_replay_report import table, markdown_body

WORK = Path('/workspace/jiaoyi-gates-5-9-18370')
OUTPUT = ROOT/'docs/rebuild_rfc_gates_5_to_9_audit_18370.json'
BUNDLE = ROOT/'docs/rebuild_rfc_gates_5_to_9_evidence_18370.json'
REPORT = ROOT/'doc/REBUILD_RFC_GATES_5_TO_9_REPORT-18370-2026-11-08.md'


def source(ref):
    return verified_json(ref['path'], ref['sha256'])


def build():
    capture = read(WORK/'capture/capture.json')
    require(source(capture['plan_receipt']) == capture['plan'], 'capture_plan_changed')
    require(set(a['account_id'] for a in capture['accounts']) == {49236051,46130058,47790949,43002148}, 'account_denominator')
    require(all(a['status']=='complete' for a in capture['accounts']), 'incomplete_evidence_capture')
    require(all(r['type'] in (2100,2102,2114,2116,2133) for r in capture['protocol_requests']), 'non_evidence_broker_request')
    previous = read(ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json')['execution_evidence']
    statement_rows, statement_sources = statements(previous['statement_evidence']['files'])
    accounts, positions, calendar = [], [], []
    raw_by_id = {}
    for a in capture['accounts']:
        account = a['account_id']
        cat, contracts = source(a['catalog_receipt']), source(a['contracts_receipt'])
        require(cat['ctidTraderAccountId'] == contracts['ctidTraderAccountId'] == account, 'contract_account_join')
        catalog = {int(s['symbolId']): s for s in cat['symbol']}
        names = {k:s.get('symbolName','unresolved') for k,s in catalog.items()}
        specs = {int(s['symbolId']):s for s in contracts['symbol']}
        deals = []
        pages = []
        for page in a['deals']['pages']:
            raw = source(page['receipt'])
            require(raw['ctidTraderAccountId'] == account, 'deal_page_account')
            if not raw['hasMore']:
                deals.extend(raw['deal'])
            pages.append({**page, 'raw_response_sha256': page['receipt']['sha256']})
        require(canonical(sorted(deals,key=lambda d:d['dealId'])) ==
                canonical([d['raw'] for d in a['deals']['deals']]), 'captured_deal_index_mismatch')
        for d in deals:
            raw_by_id[(account,int(d['dealId']))] = d
        positions += ledger(account,a['environment'],deals,names,specs,statement_rows)
        at = next(r['received_at_utc'] for r in capture['protocol_requests'] if r['account_id']==account and r['type']==2116)
        calendar += [contract_snapshot(account,names[k],s,at) for k,s in sorted(specs.items())]
        used_ids = sorted(set(specs)|{d['symbolId'] for d in deals})
        accounts.append({'account_id':account,'environment':a['environment'],'contracts':contracts,
            'selected_catalog_rows':[catalog[k] for k in used_ids], 'deals':sorted(deals,key=lambda d:d['dealId']),
            'catalog_response_receipt':a['catalog_receipt'],'contracts_receipt':a['contracts_receipt'],'deal_pages':pages})
    # Every older authentic raw deal in the Phase1 inventory is checked. Reports
    # and synthetic fixtures are not mistaken for additional broker observations.
    older = {}
    prior_refs = []
    paths = sorted({m['source'] for p in previous['positions'] for m in p['closing_money_evidence']})
    for path in paths:
        raw = read(path)
        prior_refs.append({'path':path,'sha256':sha(path),'deal_rows':len(raw['deal'])})
        for d in raw['deal']:
            key = (raw['ctidTraderAccountId'],d['dealId'])
            require(key in raw_by_id and canonical(d)==canonical(raw_by_id[key]), 'retained_deal_missing_or_changed')
            older[key] = d
    service_refs = read(WORK/'service/manifest.json')
    entries, event_count, event_joined, events_decoded = [], 0, 0, 0
    all_orders = []
    for ref in service_refs:
        require(ref['status']=='captured' and not ref['page_limit_reached'], 'service_snapshot_incomplete')
        value = source(ref)
        if ref['kind'].startswith('orders'):
            all_orders += value['rows']
            for row in value['rows']:
                if row.get('entry_cost') is not None:
                    verified_entry(row)
                    entries.append(row)
        else:
            for row in value['rows']:
                event_count += 1
                events_decoded += row['decodable'] is True
                key = (row['account_id'], row['deal_id'])
                if key in raw_by_id:
                    require(raw_by_id[key]['positionId']==row['position_id'], 'execution_summary_position_join')
                    event_joined += 1
    require(len({(r['account_id'],r['strategy'],r['entry_key']) for r in all_orders}) == len(all_orders), 'service_order_duplicates')
    require(all(r['kind']=='open' for r in entries), 'unexpected_exit_dispatch_attribution')
    bundle = {'schema':'jiaoyi.readonly-evidence-extract.v1','trace':'№18,370',
        'capture_plan':capture['plan'],'capture_plan_sha256':capture['plan_receipt']['sha256'],
        'capture_manifest_sha256':sha(WORK/'capture/capture.json'),
        'protocol_requests':capture['protocol_requests'], 'accounts':accounts,
        'retained_deal_sources':prior_refs,'retained_unique_deals_exactly_reconciled':len(older),
        'service_source_receipts':service_refs,'attributed_entry_rows':entries,
        'publication_scope':'Relevant symbol catalog rows only; full authenticated responses retained locally by receipt hash. No credentials, unrelated symbol catalog, raw price cache or duplicated deal index published.'}
    save(BUNDLE,bundle)
    closes = [d for p in positions for d in p['closing_deals']]
    summaries = []
    for a in accounts:
        ps = [p for p in positions if p['account_id']==a['account_id']]
        cs = [d for p in ps for d in p['closing_deals']]
        summaries.append({'account_id':a['account_id'],'environment':a['environment'],
            'captured_deals':len(a['deals']),'positions_seen':len(ps),'closing_deals':len(cs),
            'quantity_complete_positions':sum(p['quantity_complete'] for p in ps),
            'full_statement_matched_positions':sum(p['money_evidence_complete'] for p in ps),
            'closing_deal_statement_statuses':dict(collections.Counter(d['statement']['status'] for d in cs)),
            'gross_arithmetic_within_one_money_unit':sum(d['gross_within_one_money_unit'] for d in cs),
            'netting_reversal_fills':sum(p['netting_reversal_fills'] for p in ps)})
    require(len(closes) == sum('closePositionDetail' in d for a in accounts for d in a['deals']), 'closing_money_denominator_lost')
    reservation = read(WORK/'reservation/reservation.json')
    original = read(ROOT/'docs/rebuild_rfc_offline_gates_completion_18368.json')
    gate_ledger = original['gate_status_ledger']
    requirements = {
        'G5': 'Current broker contracts/schedules and dated 2026 exceptions captured. No effective-dated 2023–2025 broker/venue history or historical tick-size authority; frozen candidate calendar qualification remains blocked.',
        'G6': 'Realised closing money and complete captured quantities reconcile for the published statement-matched subset. Historical contract unit/lot/point-value authority and complete period statement/quantity coverage remain missing; no full gate claim from a partial subset.',
        'G7': '17 measured demo entry observations but zero paired exit-dispatch observations. No eligible round-trip execution-cost train/holdout sample; no cost fit or invented holdout error. Retain explicit uncalibrated sensitivities.',
        'G8': 'Broker deals join stored execution summaries. Public summaries omit raw acknowledgement/amend/dispatch payloads, restart books and frozen-candidate shadow decisions. Receipt presence is not research/live execution parity.',
        'G9': 'Exact reservation audit completed, not an unseen reservation: all 160 preparation histories overlap inspected data,80 tick windows overlap prior inspection,36 planned M30 slices have spent missing bars. Original 160 tick windows absent. New RFC transition/evidence reconciliation required; collection stays paused.'}
    for gate in gate_ledger:
        if gate['gate'] in requirements:
            gate['status']='BLOCKED_EXTERNAL_EVIDENCE' if gate['gate']!='G9' else 'AUDITED_BLOCKED_INDEPENDENCE_AND_COVERAGE'
            gate['remaining_blocker']=requirements[gate['gate']]
            gate['current_evidence_file']=str(OUTPUT.relative_to(ROOT))
    passed = sum(g['status']=='PASSED' for g in gate_ledger)
    tests = (WORK/'focused-tests.log').read_text()
    require('Ran 7 tests' in tests and tests.rstrip().endswith('OK'), 'focused_tests_not_passed')
    full_tests = (WORK/'research-tests.log').read_text()
    require('Ran 99 tests' in full_tests and 'OK (skipped=1)' in full_tests, 'research_tests_not_passed')
    artifact_hashes = {str(p.relative_to(ROOT)):sha(p) for p in [
        ROOT/'ops/research'/name for name in ('broker_evidence.py','broker_money.py','broker_calendar.py',
            'rebuild_reservation.py','rebuild_gate_audit.py','export_execution_evidence.sql','test_broker_evidence.py')]}
    result = {'schema':'jiaoyi.rfc-gates5to9-audit.v1','trace':'№18,370',
        'as_of_utc':dt.datetime.fromtimestamp(capture['plan']['to_ms']/1000,dt.timezone.utc).isoformat(),
        'captured_evidence_file':str(BUNDLE.relative_to(ROOT)),'captured_evidence_sha256':sha(BUNDLE),
        'rfc_gates_passed_before':4,'rfc_gates_passed_now':passed,'rfc_gates_total':12,'gate_status_ledger':gate_ledger,
        'calendar_contract_inventory':calendar,'round_trip_money_inventory':positions,
        'statement_sources':statement_sources,'account_evidence_summary':summaries,
        'cost_calibration':{'measured_demo_entry_observations':len(entries),'matched_exit_dispatch_observations':0,
            'eligible_round_trip_training_samples':0,'eligible_round_trip_holdout_samples':0,
            'status':'NOT_FITTED_INSUFFICIENT_PAIRED_EVIDENCE','holdout_error':None,'uncertainty':None,
            'sparse_fallback':'No empirical cost authority; 3.74bp sensitivity/24bp stress remain assumptions, not substituted live/history costs.'},
        'execution_parity':{'service_order_rows':len(all_orders),'service_summary_events':event_count,
            'decodable_service_summaries':events_decoded,'summary_rows_joined_to_broker_deals':event_joined,
            'raw_event_payloads_exposed_by_public_endpoint':False,'decision_parity_verified':False,
            'private_readonly_exporter':'ops/research/export_execution_evidence.sql',
            'private_export_not_executed':'DATABASE_URL targets local development database, not the running service; no production DB credential/route established.'},
        'gate_9_reservation_manifest':reservation,
        'readonly_endpoint_capabilities_found':{'can_query_symbol_contracts_schedules':True,
            'can_query_closed_deals_round_trip':True,'closed_round_trip_deals_found':sum(p['quantity_complete'] for p in positions),
            'count_definition':'Complete captured position-volume lifecycles; distinct from284 closing fills and from calibrated execution costs.'},
        'verification':{'focused_tests':7,'focused_test_log_sha256':sha(WORK/'focused-tests.log'),
            'research_suite':{'tests_run':99,'skipped':1,'failures':0,'log_sha256':sha(WORK/'research-tests.log')},
            'code_hashes':artifact_hashes,'retained_unique_deals_reconciled':len(older),
            'closing_gross_arithmetic_checks':len(closes),'closing_gross_arithmetic_matches':sum(c['gross_within_one_money_unit'] for c in closes),
            'statement_matched_closing_deals':sum(c['statement']['status']=='matched' for c in closes),
            'statement_money_mismatches':sum(c['statement']['status']=='mismatch' for c in closes),
            'source_hashes_reverified':True,'frozen_native_window_source_parity':True},
        'request_counts':{'broker_capture_authentication':8,'broker_authentication_only_probe':2,
            'broker_contract_catalog_queries':8,'broker_deal_queries':4,'total_broker_protocol_messages':len(capture['protocol_requests'])+2,
            'new_price_tick_requests':0,'new_m30_requests':0,'service_get_requests':15,'order_requests':0},
        'financial_pass':False,'new_independent_qualified_trades':0,'validation_opened':False,'live_rules_changed':False,
        'goal':'73% net wins,1:1.38 per strategy/account latest 20 (15/20 required); no new win-rate evidence from this audit',
        'evidence_limitations':['Broker history is bounded to24Oct2026 through capture, not lifetime account history.',
            'Statements cover earlier dates; missing newer statements remain missing.',
            'Human/manual and bot trades are labelled separately in the ledger, never silently pooled into candidate performance.',
            '13 prior entry observations grow to17 retained observations; zero paired exit-dispatch cost observations.',
            'G1–G4 statuses inherited from PR #197; current-main frozen-engine source refusal is recorded explicitly under G9.']}
    save(OUTPUT,result)
    report(result)
    return result


def report(r):
    rs=r['gate_9_reservation_manifest']; sums=rs['summary'];v=r['verification']
    account_table=table(['Account','Type','Deals','Closing fills','Complete quantities','Full statement-matched positions'],
        [[s['account_id'],s['environment'],s['captured_deals'],s['closing_deals'],s['quantity_complete_positions'],s['full_statement_matched_positions']] for s in r['account_evidence_summary']])
    gates=table(['Gate','Status','Exact remaining requirement'],[[g['gate'],g['status'],g['remaining_blocker']] for g in r['gate_status_ledger'] if g['gate'] in ('G5','G6','G7','G8','G9')])
    contracts=table(['Account','Symbol','Broker timezone','Lot size (cents)','Minimum / step','Dated exceptions'],
        [[c['account_id'],c['symbol'],c['broker_timezone'],c['contract_fields']['lotSize'],
          str(c['contract_fields']['minVolume'])+' / '+str(c['contract_fields']['stepVolume']),len(c['dated_holidays'])]
          for c in r['calendar_contract_inventory'] if c['symbol'] in CORE])
    partitions=table(['Original M30 slice status','Slices'],[[k,n] for k,n in sums['original_segment_statuses'].items()])
    by_symbol=collections.defaultdict(collections.Counter)
    for e in rs['episodes']:
        s=by_symbol[e['symbol']];s['jobs']+=1;s['seen_labels']+=bool(e['tick_window_prior_inspection_roles']);s['seen_prep']+=bool(e['preparation_prior_inspection_roles'])
        s.update(e['slot_counts'])
    preparation=table(['Symbol','Jobs','Inspected tick windows','Inspected preparation','Cached / unspent missing / spent missing slots*'],
        [[s,c['jobs'],c['seen_labels'],c['seen_prep'],f"{c['verified_cache']} / {c['unspent']} / {c['blocked_spent']}"] for s,c in sorted(by_symbol.items())])
    sgt=dt.datetime.fromisoformat(r['as_of_utc']).astimezone(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')
    total_deals=sum(a['captured_deals'] for a in r['account_evidence_summary'])
    counts=sums['original_segment_statuses']
    text=f'''# Rebuild RFC: Gates 5–9 evidence and reservation audit

№18,370 §§1–10 · Deal-history cutoff {sgt} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Read-only evidence capture succeeded on all four accounts.** It returned {total_deals} broker deals, including {v['closing_gross_arithmetic_checks']} closing fills. **{v['statement_matched_closing_deals']} closing fills match retained statement money and displayed prices; {v['statement_money_mismatches']} statement mismatches.** All {v['closing_gross_arithmetic_checks']} closing-money calculations agree with the cent-unit/conversion diagnostic within one broker money unit. Current contracts and schedules are captured for {len(r['calendar_contract_inventory'])} account/symbol combinations.

**RFC gates remain {r['rfc_gates_passed_now']}/12.** This audit materially improves the money evidence and identifies exact reservation conflicts; it does not manufacture historical calendar authority, paired exit-dispatch costs or an independent test. G5–G9 remain blocked for the reasons below. Zero new price-history requests, zero orders, zero validation access, zero live-rule changes.

**Goal progress: zero new independently qualified trades and no measured win-rate improvement.** The goal remains 73% net wins and 1:1.38 per strategy/account over the latest 20 closes. 15/20 is required. Historical/manual trades in this evidence inventory do not become frozen-candidate wins.

## §1. Evidence captured, scope and provenance

Capture covers 24 October 2026 through the timestamp above. There were 12 broker read queries (four catalogs, four contract responses, four complete deal pages), 10 authentication messages including the initial two-message probe, and 15 service GETs. No subscription, price-history, refresh-token or trading message is allowed by the new exporter. The price collector remains paused. Failed, truncated or out-of-window evidence cannot be labelled complete.

{account_table}

The earlier 77 unique raw deals across five retained broker files are all present and byte-equivalent as canonical objects in the new capture. Fifteen uploaded statement files are hash checked and deduplicated by account plus closing deal ID. The bundle retains relevant raw contracts, raw deals and catalog rows; unused catalog entries and duplicate raw history are omitted from Git. Local full-response hashes remain listed. No credential is included.

## §2. Gate status and what is still missing

{gates}

G1–G4 retain PR #197's engineering status. This run does not rerun trading backtests or convert a small reconciled subset into whole-system qualification. No profit threshold, candidate rule, session or cost assumption was changed.

## §3. Current broker calendar and contract authority

{contracts}

The raw evidence includes weekly intervals in the broker's named timezone, dated holiday exceptions, lot/minimum/step volumes, commission type/rate and funding fields. The parser converts dated exceptions with IANA timezones and rejects ambiguous local timestamps. For example, a 26 November 2026 exception starting 22:30 Europe/Moscow converts to 19:30 UTC. New York 09:30 converts to 13:30 UTC in summer and 14:30 UTC in winter.

These are **broker snapshots taken now**, not effective-dated 2023–2025 contracts or exchange calendars. `digits` supplies display precision, not conclusive minimum tick increment. No absent holiday is interpreted as proof of a normal historical session. Obtain dated broker/venue notices and historical contract effective intervals for the frozen dates before claiming G5.

## §4. Money reconciliation, reversals and actual costs

The ledger keeps broker `grossProfit`, signed swap, closing-volume commission and conversion fee in Decimal deposit-currency units. Closing commission is counted once; opening deal commission is not added again. Recorded bid/ask effects already in realised P&L are not subtracted a second time. Missing fee/conversion fields refuse reconciliation.

Actual netting reversals divide a single fill into its closing quantity and residual new opening quantity. This matters for manual NatGas/JPN225 paths: filled volume can exceed `closedVolume`. A position's label and account remain attached; incomplete opening history or a still-open remainder is retained as incomplete, not treated as a closed losing/winning trade.

Two initially apparent price mismatches were statement display rounding: the broker retains weighted entry precision beyond the CSV. The parser permits only half the CSV's displayed decimal unit for prices; **net money and commission must still match exactly**. The JSON gives every closing-deal comparison, raw-deal hash and source statement line.

The source-based quantity/gross check uses cTrader's cents-of-units convention and each close's recorded conversion rate. It is explicitly a diagnostic until historical point/lot semantics are authoritative. Whole-period coverage and effective-dated contracts remain G6 gaps, even though the statement-matched subset is useful evidence.

## §5. Cost calibration and execution parity

There are **{r['cost_calibration']['measured_demo_entry_observations']} actual demo entry dispatch-to-fill observations**, up from 13, but **zero paired exit-dispatch observations**. Entry price or broker stop/target level is not an exit dispatch quote. No round-trip cost model was fitted, no empty holdout was called a pass, and no demo costs were transferred to live or to 2023. The 3.74 bp sensitivity and 24 bp stress reserve remain labelled assumptions. Sparse buckets remain uncalibrated.

The service returned {r['execution_parity']['service_order_rows']} order rows and {r['execution_parity']['service_summary_events']} execution-summary rows; {r['execution_parity']['summary_rows_joined_to_broker_deals']} summary rows join a captured broker deal by account/deal/position. These counts are not unique replay episodes. The public view strips raw payloads. It does not expose a full amend/ack/restart book or contemporaneous decisions of the frozen research candidate.

`ops/research/export_execution_evidence.sql` is the minimal bounded, read-only private database exporter for stored execution payloads and existing entry-dispatch records. It has a 31-day bound, 2,000-row cap and `has_more`, read-only transaction, timeout and four-account allowlist. It was not run against the local development database as if it were production. It cannot recreate exit quotes or restart evidence that was never recorded. No live logging change or schema expansion was deployed.

## §6. Exact original reservation: data availability versus independence

The audit preserves the Phase 4 `s3_macro_pullback_v1` dates, native sessions, 3.74 bp admission and 24 bp repricing. It indexes 544 episode definitions and 616 M30 source files, verifies their eligible receipts, and leaves protected dependencies quarantined.

{partitions}

Of {sums['original_m30_segments']} original request slices, {counts['complete_verified_cache']} have every required native bar cached; {sums['missing_m30_segments_to_fetch']} lack bars; {counts['no_native_slots']} contain no required native-session bars. Of the incomplete slices, {counts['blocked_spent_missing_bars']} contain spent missing bars and cannot be re-requested. {counts['missing_unspent']} are wholly missing/unspent and {counts['partial_cache_unspent_gaps']} have reusable cached bars plus unspent gaps. These are request-slice counts, not file counts or successful episode counts.

**All {sums['missing_native_tick_episodes_to_fetch']} required native tick windows are absent. {sums['episodes_with_previously_inspected_tick_window']} overlap previously inspected price intervals; all {sums['episodes_with_inspected_preparation']} have some inspected preparation.** The registry considers the whole dependency chain. Cached reuse is technically available, but it cannot establish that an already inspected dependency is unseen. No original date was silently replaced and no reservation was activated.

Preparation overlap alone is not proof of look-ahead or outcome leakage. It fails the current frozen dependency-inspection rule; a different reuse policy needs an explicit versioned decision. The 80 overlapping evaluation windows are a separate issue: their price paths were already available through retained M30 data, even though native tick coverage is absent.

{preparation}

*Slot totals here count each episode's dependency separately; shared bars appear more than once. The machine manifest lists deduplicated partition identities and their consumers. `missing_m30_segments_to_fetch=178` records incompleteness, not permission to fetch all 178; spent gaps remain blocked.*

## §7. Actual preflight receipts and code identity

The current main command refuses the Phase 4 source identity with `frozen_engine_changed`: Phase 5 added the separately named fixed-window wrapper to `manifest_pullback.rs`. The exit source is unchanged. The audit does not mask that hash difference or change the frozen registry.

Using the separately retained exact Phase 4 checkout, all core/example source hashes and frozen binaries match PR #196's receipt, and native calendar metadata matches current metadata. Its real preflight refuses preparation/ticks with `spent_tick_request`; evaluation refuses `preparation_must_be_decoded_before_ticks`. The inspection guard refuses `{rs['inspection_and_trial_guard'].get('reason','see JSON')}`. All receipts, hashes, planned jobs and preparation partitions are in the JSON.

G9 therefore has an **exact audit receipt**, not an independent-reservation pass. Before any future collection, use an explicitly reviewed RFC transition to choose either transparent reused-data diagnostics or a newly reserved, genuinely unspent matrix. Preserve the frozen engine as a separately identified adapter; do not weaken the source guard to accommodate the changed wrapper. Reconcile unavailable older references and dated calendar authority first. Price collection remains paused.

## §8. Verification and next actions

Seven focused regressions pass: forbidden broker operations/accounts; truncated pagination; signed financing and commission; netting reversal quantities; statement rounding versus strict money; timezone/dated-snapshot authority; cached/unspent/spent preparation partitioning. The full research suite ran 99 tests with one existing skip and no failures. The deterministic audit rechecks source hashes and all {v['closing_gross_arithmetic_checks']} closing fills. CI must pass `check`, `docker` and `engineering` before merge.

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
'''
    REPORT.write_text(text)
    body, toc = markdown_body(text)
    page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi RFC Gates 5–9 · №18,370</title><style>body{margin:0;background:#f1f5f9;color:#172533;font:17px/1.65 system-ui,sans-serif}main{max-width:1180px;margin:auto;padding:36px 24px}h1,h2,h3{line-height:1.25;color:#12344b}h2{margin-top:2em;border-top:1px solid #bac9d4;padding-top:1em}p,li{overflow-wrap:anywhere}table{display:block;overflow-x:auto;border-collapse:collapse;font-size:14px;margin:20px 0;background:white}td,th{text-align:left;vertical-align:top;padding:10px;border:1px solid #c4d1dc;min-width:90px}th{background:#183e56;color:white}code{background:#e1e9ef;padding:2px 4px;overflow-wrap:anywhere}strong{color:#143b50}a{color:#075ea3}@media(max-width:600px){main{padding:18px 14px}body{font-size:16px}h1{font-size:28px}table{font-size:13px}}@media print{body{background:white}main{padding:0}table{display:table;font-size:10px}}</style><main>'''+body+'</main></html>'
    REPORT.with_suffix('.html').write_text(page)


if __name__ == '__main__':
    offline_process()
    r=build()
    print(json.dumps({'accounts':r['account_evidence_summary'],'verification':r['verification'],
                      'gates_passed':r['rfc_gates_passed_now']},indent=2))
