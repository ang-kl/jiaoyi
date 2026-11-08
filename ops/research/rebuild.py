#!/usr/bin/env python3
"""One offline RFC command: inventory, preflight, parity, evidence, report.

Legacy experiments are immutable parity adapters, never collection authority.
No candidate/date/cost overrides are accepted by this command.
"""
import argparse
import collections
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess

from replay_inventory import ROOT, CORE, canonical, read, save, sha, ms, permitted
import phase5b_audit as old
import unified_replay as u
from rebuild_preflight import offline_process, require, Refusal, Identity, verify_plan, verified_json
from rebuild_authority import STATE

WORK = Path('/workspace/jiaoyi-rebuild-offline-18368')
OUTPUT = ROOT / 'docs/rebuild_rfc_offline_gates_completion_18368.json'
REGISTRY = ROOT / 'docs/rebuild_rfc_registry_18368.json'
SCOPE = ROOT / 'docs/rebuild_rfc_offline_scope_18368.json'
NATIVE = ROOT / 'target/debug/examples'
BINS = ['native_target_sizing_replay', 'manifest_pullback_replay', 'tiered_pullback_replay', 'replay_inventory_metadata', 'rebuild_continuity_replay']
RFC = ROOT / 'doc/NATIVE_TARGET_SIZING_AND_REBUILD_RFC-18353-2026-11-08.md'


def now():
    return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()


def engine_metadata(episodes):
    items = [{'key': e['episode_id'], 'symbol': e['symbol'], 'class': e['asset_class'], 'date': e['date']} for e in episodes]
    raw = subprocess.check_output([str(NATIVE / 'replay_inventory_metadata')], input=json.dumps(items).encode(), timeout=120)
    return json.loads(raw)


def build_registry(spec, inventory, manifest):
    hypothesis = spec['frozen_candidate_hypothesis']
    spent = [dict(symbol=v['symbol'], account_id=v.get('account_id', 49236051),
                 start_ms=ms(v['from']), end_ms=ms(v['to']), role=v['kind']) for v in spec['spent_registry']['intervals']]
    for e in manifest['episodes'] + manifest['m30_segments']:
        w = e['source_window_utc']
        spent.append(dict(symbol=e['symbol'], account_id=e['account_id'], start_ms=ms(w['from']), end_ms=ms(w['to']),
                          role='ticks' if 'episode_id' in e else 'm30'))
    spent = sorted({canonical(v): v for v in spent}.values(), key=lambda v:(v['symbol'],v['account_id'],v['start_ms'],v['end_ms'],v['role']))
    result = {
        'schema': 'jiaoyi.rebuild-registry.v1', 'trace': '№18,368',
        'identity': {'candidate_id': hypothesis['id'], 'hypothesis_sha256': canonical(hypothesis),
            'code_sha256': canonical({'entry': hypothesis['entry_rule_source_sha256'], 'exit': hypothesis['exit_engine_source_sha256']}),
            'calendar_sha256': sha(ROOT / 'crates/jiaoyi-core/src/strategy3/sessions.rs'),
            'contract_sha256': canonical({s: v['contracts'] for s, v in inventory['symbols'].items()}),
            'cost_sha256': canonical(hypothesis['costs']),
            'spent_registry_sha256': canonical(spent), 'rfc_state': STATE},
        'hypothesis': hypothesis, 'transitions': [], 'collection_enabled': False,
        'horizon_ms': 4 * 3600000, 'tail_ms': 5001,
        'calendar_authority': {'strategy': 'versioned native sessions; no changes to frozen rules',
            'broker': 'unknown historical dealing/maintenance terms; cannot infer from missing bars',
            'reference_venue': 'dated source required; current strategy label is not historic venue authority',
            'funding': 'unknown until dated account/contract evidence'},
        'spent_intervals': spent,
        'frozen_independent_jobs_sha256': canonical(spec['independent_evaluation_plan']['jobs']),
        'external_evidence_required': ['account-scoped round-trip costs', 'dated broker/venue terms',
            'independent cost-error blocks', 'broker acknowledgement/shadow lifecycle', 'unseen evaluation'],
        'historical_scope': 'All existing outputs are retained diagnostics; no new independent observations.'}
    Identity.parse(result['identity'])
    return result


def classify(e, row):
    if e.get('source_status') == 'failed':
        return 'failed_tick_source'
    if not e['eligible_for_research']:
        return 'protected_quarantined'
    if e['protocol_status'] == 'inherited_repeat_interval':
        return 'inherited_repeat'
    if e['protocol_status'] != 'clean_first_read':
        return 'unverified_provenance_quarantined'
    if not permitted(ms(e['source_window_utc']['from']), ms(e['source_window_utc']['to'])):
        return 'outside_research_scope'
    if row and not row['window']['covers']:
        return 'window_mismatch_quarantined'
    if row and row.get('tier') == 'tier1':
        return 'tier1_native_complete'
    if row and row.get('tier') == 'tier2':
        return 'tier2_partial_prep'
    if e['symbol'] not in CORE:
        return 'outside_candidate_universe'
    return 'coverage_incomplete_quarantined'


def inventory():
    spec, p5, _, ends, manifest, store, meta, _, selected, register = old.sources_and_selection()
    # Independent current executable metadata must match the frozen original.
    current = engine_metadata(manifest['episodes'])
    require(meta == current, 'native_calendar_migration_drift')
    by_id = {r['episode_id']: r for r in register}
    rows = []
    for e in manifest['episodes']:
        c = by_id.get(e['episode_id'])
        row = dict(e)
        row['classification'] = classify(e, c)
        row['native_coverage'] = c
        row['qualification'] = 'recorded_scenario_only_not_complete_path_or_financial_pass'
        rows.append(row)
    require(len(rows) == 544 and len({e['episode_id'] for e in rows}) == 544, 'unified_denominator_changed')
    require(len(manifest['m30_segments']) == 616 and len(ends) == 399, 'unified_segment_denominator_changed')
    inputs = read(ROOT / 'docs/rebuild_phase1_cost_and_contract_inventory_18355.json')
    registry = build_registry(spec, inputs, manifest)
    if REGISTRY.exists():
        require(read(REGISTRY) == registry, 'registry_changed_without_transition')
    else:
        save(REGISTRY, registry)
    references = collections.defaultdict(list)
    for e in rows:
        references[e['tick_receipt_sha256']].append(e['episode_id'])
    for s in manifest['m30_segments']:
        references[s['receipt_sha256']].append(s.get('segment_id', s['source_path']))
    result = {'episodes': rows, 'm30_segments': manifest['m30_segments'],
        'failed_sources_retained': manifest['failed_sources'], 'refused_receipt_hashes': sorted(store.quarantine),
        'protected_receipt_hashes': sorted({e.get('tick_receipt_sha256',e.get('receipt_sha256')) for e in manifest['episodes']+manifest['duplicate_receipts']
                                           if not e['eligible_for_research'] and e.get('source_status') != 'failed'}),
        'verified_market_files': store.verified, 'receipt_references': dict(references),
        'retention': 'Raw evidence retained once by hash; consumer references enumerated. No purge or duplicate historical copy. New immutable partitions use bounded gzip store.',
        'summary': {'scheduled_episode_definitions': len(rows), 'prior_definitions': 384,
            'phase5_complete_ticks': sum(v['status'] == 'complete' for k, v in ends.items() if k.startswith('job-')),
            'phase5_failed_ticks': sum(v['status'] != 'complete' for k, v in ends.items() if k.startswith('job-')),
            'prior_m30_files': 377, 'phase5_new_m30_files': 239, 'all_m30_files': 616,
            'reused_slices': sum(p['mode'] == 'reuse' for p in p5['bar_segments']),
            'classifications': dict(collections.Counter(e['classification'] for e in rows)),
            'new_batch_classifications': dict(collections.Counter(e['classification'] for e in rows if e.get('origin') == 'phase5_new_batch'))},
        'source_reconciliation': {'phase5_receipts': len(ends), 'verified_market_file_count': len(store.verified),
            'all_prior_frozen_reconciliation': manifest['frozen_file_reconciliation'],
            'unavailable_historical_references': sum(x['status'] == 'unavailable_locally' for x in manifest['prior_receipt_reconciliation'])}}
    # Fully enumerate the original frozen independent plan, without substituting
    # Phase5 dates or fixed UTC windows. This remains a BLOCKED preflight.
    planned = spec['independent_evaluation_plan']['jobs']
    planned_meta = engine_metadata(planned)
    preflight_rows = []
    for j in planned:
        m = planned_meta[j['episode_id']]
        required = u.slots(m['prior_45_calendar_days'])
        bars = store.bars[(j['account_id'], j['symbol'])]
        missing = [t for t in required if t not in bars]
        covering = [e['episode_id'] for e in rows if e['account_id'] == j['account_id'] and e['symbol'] == j['symbol'] and
                    e['eligible_for_research'] and ms(e['source_window_utc']['from']) <= ms(j['from']) and ms(e['source_window_utc']['to']) >= ms(j['to'])]
        preflight_rows.append({'episode_id': j['episode_id'], 'symbol': j['symbol'], 'date': j['date'],
            'native_window': m['session'], 'required_m30_slots': required, 'missing_m30_slots': missing,
            'covering_tick_sources': covering, 'status': 'BLOCKED_COLLECTION_PAUSED_AND_EVIDENCE_INCOMPLETE'})
    result['frozen_independent_plan_preflight'] = preflight_rows
    result['historical_registry'] = {'native_calendar_version': registry['identity']['calendar_sha256'],
        'current_contracts_not_historical': True, 'unknown_holidays_never_inferred_from_absent_bars': True}
    store.finish()
    save(WORK / 'inventory.json', result)
    print(json.dumps(result['summary']), flush=True)
    return result


def source_freeze():
    files = [ROOT / n for n in ['ops/research/rebuild.py', 'ops/research/rebuild_preflight.py',
        'ops/research/rebuild_evidence.py', 'ops/research/rebuild_authority.py', 'ops/research/rebuild_report.py',
        'ops/research/test_rebuild.py', 'crates/jiaoyi-core/src/research/rebuild.rs',
        'crates/jiaoyi-core/src/research/rebuild_continuity.rs', 'crates/jiaoyi/src/broker_backtest.rs',
        'Cargo.lock']] + [REGISTRY, RFC]
    files += [ROOT/n for n in old.HELPERS+['ops/research/phase5b_audit.py']]
    files += list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))
    files += list((ROOT/'crates/jiaoyi-core/examples').glob('*.rs'))
    frozen = {'source_sha256': {str(p.relative_to(ROOT)): sha(p) for p in files},
        'native_binary_sha256': {b: sha(NATIVE / b) for b in BINS},
        'baseline_documents': {str(p.relative_to(ROOT)): sha(p) for p in [old.RESULT, old.P3,
            ROOT / 'docs/rebuild_phase2_runner_and_pullback_results_18358.json',
            ROOT / 'docs/native_target_sizing_offline_results_18353.json']},
        'operation': 'historical_parity_and_offline_engineering_only', 'new_broker_requests': 0}
    return frozen


def replay_bytes(binary, input_path, expected_path, expected_hash):
    require(sha(expected_path) == expected_hash, 'prior_native_receipt_hash_mismatch')
    command = [str(NATIVE / binary), str(input_path)]
    first = subprocess.check_output(command, timeout=120)
    second = subprocess.check_output(command, timeout=120)
    original = Path(expected_path).read_bytes()
    require(first == second == original, 'byte_parity_failed:' + str(input_path))
    return {'input_sha256': sha(input_path), 'output_sha256': hashlib.sha256(first).hexdigest(),
            'byte_identical_to_frozen': True, 'two_runs_identical': True}


def parity():
    out = {'legacy_159': [], 'phase2_35': [], 'phase5b_2': []}
    plan = read(ROOT / 'docs/native_target_sizing_offline_plan_18353.json')
    results = {r['key']: r for r in read(ROOT / 'docs/native_target_sizing_offline_results_18353.json')['rows']}
    for j in plan['jobs']:
        require(sha(j['path']) == j['sha256'], 'legacy_source_changed')
        prior = Path('/workspace/jiaoyi-native-target-18353') / ('replay-' + j['key'] + '.json')
        proof = replay_bytes('native_target_sizing_replay', j['path'], prior, results[j['key']]['native_sha256'])
        out['legacy_159'].append({'episode_id': j['key'], **proof})
        if len(out['legacy_159']) % 40 == 0:
            print('BYTE PARITY', len(out['legacy_159']), '/159', flush=True)
    rows = read(ROOT / 'docs/rebuild_phase2_runner_and_pullback_results_18358.json')['rows']
    for row in rows:
        work = Path('/workspace/jiaoyi-rebuild-phase2-18358')
        inp = work / ('input-' + row['episode_id'] + '.json')
        require(sha(inp) == row['input_sha256'], 'Phase2_input_changed')
        key = row['native_receipt_key']; expected = work / 'native' / (key + '.json')
        proof = replay_bytes('manifest_pullback_replay', inp, expected, read(work / 'native' / (key + '.receipt.json'))['output_sha256'])
        require(json.loads(expected.read_bytes()) == row['native'], 'Phase2_output_publication_mismatch')
        out['phase2_35'].append({'episode_id': row['episode_id'], **proof})
    rows = [r for r in read(old.RESULT)['rows'] if r['origin'] == 'phase5_new_batch' and r['tier'] == 'tier1']
    spec = read(old.SPEC)['frozen_candidate_hypothesis']
    for row in rows:
        inp = read(old.WORK / ('input-' + row['episode_id'] + '.json'))
        path = WORK / 'parity-input.json'
        save(path, {'input': inp, 'policy': {'tier': 'tier1', 'threshold_bps': 48.0,
                                          'admission_bps': spec['costs']['admission_bps_per_side']}})
        require(sha(path) == row['input_sha256'], 'Phase5B_input_changed')
        key = row['native_receipt_key']; expected = old.WORK / 'native' / (key + '.json')
        proof = replay_bytes('tiered_pullback_replay', path, expected, read(old.WORK / 'native' / (key + '.receipt.json'))['output_sha256'])
        require(json.loads(expected.read_bytes()) == row['native'], 'Phase5B_output_publication_mismatch')
        out['phase5b_2'].append({'episode_id': row['episode_id'], **proof})
    require(tuple(map(len, out.values())) == (159, 35, 2), 'parity_denominator_changed')
    save(WORK / 'parity.json', out)
    print('BYTE PARITY 159 + 35 + 2; each run twice, all exact', flush=True)
    return out


def evidence(kind, path):
    import rebuild_evidence as e
    value = e.read_envelope(path)
    if kind == 'money':
        return e.reconcile_position(value)
    if kind == 'cost':
        return e.verify_calibration(value['model'], value['evidence'])
    if kind == 'execution':
        return e.verify_execution_lifecycle(value)
    if kind == 'financial':
        return e.financial_gate(value['rows'], value['criteria'])
    return e.lifecycle_scorecard(value['rows'], value['policy'], value['operational_receipt'])


def continuity():
    rows = read(old.RESULT)['rows']
    selected = []
    for origin,tier in [('prior_manifest','tier1'),('phase5_new_batch','tier2')]:
        candidates = [r for r in rows if r['origin'] == origin and r['tier'] == tier
                      and r['native'].get('arms',{}).get(u.ARMS[2],{}).get('forced')]
        require(candidates, 'recorded_continuity_reference_missing')
        selected.append(sorted(candidates,key=lambda r:(r['episode_id'],r['threshold_bps']))[0])
    proofs = []
    for row in selected:
        arm = row['native']['arms'][u.ARMS[2]]['forced']
        path = WORK/'continuity-input.json'
        save(path, {'input':read(old.WORK/('input-'+row['episode_id']+'.json')),'entry':arm['entry'],
                    'engine_version':read(old.SPEC)['frozen_candidate_hypothesis']['exit_engine_source_sha256'],
                    'expected_outcome':arm['outcome']})
        raw = subprocess.check_output([str(NATIVE/'rebuild_continuity_replay'),str(path)],timeout=120)
        proof = json.loads(raw)
        require(proof['original_outcome_parity'] is True, 'recorded_continuity_failed')
        proofs.append({'episode_id':row['episode_id'],'input_sha256':sha(path),'proof':proof,
                       'output_sha256':hashlib.sha256(raw).hexdigest()})
    save(WORK/'continuity.json',proofs)
    return proofs


def preflight_file(path, stage='evaluation'):
    """Generic manifest reader, with the frozen candidate/date matrix enforced.

    Receipt-derived facts replace any caller-supplied coverage assertions. No
    supplied trade result or success constant is read or used.
    """
    from tiered_replay import CleanStore
    from replay_inventory import market, checkpoints
    plan = read(path)
    registry = read(REGISTRY)
    spec = read(old.SPEC)
    expected = {(j['episode_id'], j['account_id'], j['symbol'], j['date'], j['from'], j['to'])
                for j in spec['independent_evaluation_plan']['jobs']}
    scheduled = {(j['episode_id'], j['account_id'], j['symbol'], j['date'], j['from'], j['to'])
                 for j in plan['episodes']}
    require(scheduled == expected, 'frozen_date_matrix_changed_without_rfc_transition')
    require(canonical(spec['independent_evaluation_plan']['jobs']) == registry['frozen_independent_jobs_sha256'], 'frozen_plan_changed')
    require(sha(ROOT/'crates/jiaoyi-core/src/strategy3/sessions.rs') == registry['identity']['calendar_sha256'], 'calendar_code_changed')
    for name, key in [('manifest_pullback.rs','entry_rule_source_sha256'), ('repaired_exit.rs','exit_engine_source_sha256')]:
        require(sha(ROOT/'crates/jiaoyi-core/src/research'/name) == spec['frozen_candidate_hypothesis'][key], 'frozen_engine_changed')
    ref = plan['manifest_receipt']
    manifest = verified_json(ref['path'], ref['sha256'])
    store = CleanStore(manifest).load()
    by_id = {e['episode_id']: e for e in manifest['episodes']}
    meta = engine_metadata(plan['episodes'])
    for j in plan['episodes']:
        if stage != 'evaluation':
            continue
        require(j['episode_id'] in by_id, 'scheduled_tick_source_missing')
        e = by_id[j['episode_id']]
        require((e['symbol'],e['account_id'],e['date']) == (j['symbol'],j['account_id'],j['date']), 'episode_source_identity')
        j.update(protocol_status=e['protocol_status'],source_path=e['source_path'],tick_receipt_sha256=e['tick_receipt_sha256'])
        _, q, _, _ = market(store.source(e))
        start, end = ms(j['from']), ms(j['to'])
        ticks = {side:[t for t in q[side] if start <= t['ts_ms'] <= end+5000] for side in ('bids','asks')}
        require(all(ticks.values()), 'missing_two_sided_tick_path')
        times = sorted({t['ts_ms'] for side in ticks.values() for t in side})
        points = checkpoints(ticks['bids'], ticks['asks'], {'open':start,'end':end}, q['contract']['digits'])
        ambiguous = any(len({t['price_e5'] for t in side if t['ts_ms'] == at}) > 1
                        for side in ticks.values() for at,n in collections.Counter(t['ts_ms'] for t in side).items() if n>1)
        stale_side_gaps = sum(b['ts_ms'] - a['ts_ms'] > 6000 for side in ticks.values() for a,b in zip(side,side[1:]))
        j['path_coverage'] = {'opening_fresh':points['open'] is not None, 'closing_tail_fresh':points['end'] is not None,
            'from_ms':min(t[0]['ts_ms'] for t in ticks.values()), 'through_ms':min(t[-1]['ts_ms'] for t in ticks.values()),
            'unexpected_gaps':stale_side_gaps, 'ordering_ambiguous':ambiguous}
        # A fresh open within the declared five-second boundary is admissible.
        if points['open'] is not None:
            j['path_coverage']['from_ms'] = start
    bar_refs = collections.defaultdict(lambda: collections.defaultdict(set))
    for key, paths in store.bar_sources.items():
        for p in paths:
            decoded = market(read(p))
            if decoded:
                for b in decoded[2]:
                    bar_refs[key][b['ts_ms']].add(p)
    proof = verify_plan(plan,registry,meta,store.bars,bar_refs,store.verified,stage=stage)
    store.finish()
    return proof


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['inventory', 'preflight', 'freeze', 'parity', 'run', 'report', 'verify-evidence'])
    parser.add_argument('--stage', choices=['preparation', 'ticks', 'evaluation'], default='evaluation')
    parser.add_argument('--kind', choices=['money', 'cost', 'execution', 'financial', 'lifecycle'])
    parser.add_argument('--file', type=Path)
    args = parser.parse_args()
    network = offline_process()
    WORK.mkdir(exist_ok=True)
    if args.action == 'verify-evidence':
        require(args.kind and args.file, 'evidence_kind_and_file_required')
        print(json.dumps(evidence(args.kind, args.file), indent=2)); return
    if args.action == 'preflight':
        require(args.file is not None, 'preflight_file_required')
        print(json.dumps(preflight_file(args.file,args.stage), indent=2)); return
    if args.action == 'inventory':
        inventory(); return
    if args.action == 'parity':
        parity(); return
    if args.action == 'freeze':
        inventory()
        freeze = source_freeze()
        if SCOPE.exists():
            require(read(SCOPE) == freeze, 'frozen_scope_changed_use_versioned_transition')
        else:
            save(SCOPE, freeze)
        print('FROZEN',sha(SCOPE)); return
    if args.action == 'run':
        inv = inventory()
        freeze = source_freeze()
        require(SCOPE.exists() and read(SCOPE) == freeze, 'freeze_before_evaluation')
        for name,h in freeze['source_sha256'].items():
            committed = subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)
            require(hashlib.sha256(committed).hexdigest() == h, 'commit_frozen_code_before_evaluation:'+name)
        verified = parity()
        carried = continuity()
        require(source_freeze() == freeze, 'code_or_baseline_changed_during_run')
        for p, h in inv['verified_market_files'].items():
            require(sha(p) == h, 'market_source_changed_during_run')
        save(WORK / 'verification.json', {'network_guard': network, 'scope_sha256': sha(SCOPE),
            'parity_counts': {k: len(v) for k, v in verified.items()},
            'recorded_partition_restart_parity': carried,
            'all_sources_unchanged_after_replay': True, 'new_broker_requests': 0})
    from rebuild_report import publish
    publish()


if __name__ == '__main__':
    main()
