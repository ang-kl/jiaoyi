"""Exact Phase-4 reservation audit; no new dates, price calls or evaluation.

The result is a dependency inventory, NOT a collection permit. Cached/decode
coverage, past inspection, and a genuinely unspent request are separate facts.
"""
import collections
from pathlib import Path

import phase5b_audit as old
from rebuild import REGISTRY, engine_metadata, preflight_file
from rebuild_preflight import Refusal, Interval, require, split_budget_guard, verify_plan
from replay_inventory import ROOT, read, save, sha, canonical, ms, market
from tiered_replay import CleanStore
from unified_replay import slots

STEP = 1_800_000


def partition_slots(required, bars, refs, verified, spent, account, symbol):
    """Disjoint half-open runs. A spent missing bar can never become a request."""
    rows = []
    for t in required:
        interval = Interval(symbol, account, t, t+STEP, 'm30')
        interval.validate()
        paths = sorted(p for p in refs.get(t, ()) if p in verified)
        if t in bars and paths:
            mode, path = 'verified_cache', paths[0]
            digest = verified[path]
        else:
            mode = 'blocked_spent' if any(interval.overlaps(i) for i in spent) else 'unspent'
            path = digest = None
        row = {'from_ms': t, 'to_ms': t+STEP, 'mode': mode,
               'source_path': path, 'sha256': digest}
        if rows and rows[-1]['to_ms'] == t and all(rows[-1][k] == row[k] for k in ('mode', 'source_path', 'sha256')):
            rows[-1]['to_ms'] = t+STEP
        else:
            rows.append(row)
    require(sum((r['to_ms']-r['from_ms'])//STEP for r in rows) == len(required), 'reservation_partition_drift')
    return rows


def run(work):
    work.mkdir(parents=True, exist_ok=True)
    spec, registry = read(old.SPEC), read(REGISTRY)
    plan5, _, ends = old.phase5_receipts()
    manifest = old.combined_manifest(plan5, ends)
    require(len(manifest['episodes']) == 544 and len(manifest['m30_segments']) == 616, 'source_denominator_changed')
    jobs = spec['independent_evaluation_plan']['jobs']
    require(canonical(jobs) == registry['frozen_independent_jobs_sha256'], 'original_independent_plan_changed')
    meta = engine_metadata(jobs)
    frozen_engine = old.frozen_identity()
    require(frozen_engine == read(old.RESULT)['frozen_phase4'], 'prior_frozen_engine_receipt_changed')
    original_meta = old.metadata({'episodes': jobs})
    require(meta == original_meta, 'original_native_calendar_disagrees')
    store = CleanStore(manifest).load()
    refs = collections.defaultdict(lambda: collections.defaultdict(set))
    for key, paths in store.bar_sources.items():
        for path in paths:
            for bar in market(read(path))[2]:
                refs[key][bar['ts_ms']].add(path)
    spent = [Interval(**v) for v in registry['spent_intervals']]
    by_key = collections.defaultdict(list)
    for s in spent:
        by_key[(s.account_id, s.symbol)].append(s)
    plan = {'identity': registry['identity'], 'scheduled_episodes': len(jobs), 'episodes': []}
    episode_rows, global_parts = [], {}
    for job in jobs:
        key = (job['account_id'], job['symbol'])
        required = slots(meta[job['episode_id']]['prior_45_calendar_days'])
        parts = partition_slots(required, store.bars[key], refs[key], store.verified, by_key[key], *key)
        tick = Interval(job['symbol'], job['account_id'], ms(job['from']), ms(job['tail_through'])+1, 'ticks')
        tick.validate()
        overlaps = [s for s in by_key[key] if tick.overlaps(s)]
        sources = [e for e in manifest['episodes'] if e['account_id'] == key[0] and e['symbol'] == key[1]
                   and e['eligible_for_research'] and e['protocol_status'] == 'clean_first_read'
                   and ms(e['source_window_utc']['from']) <= tick.start_ms
                   and ms(e['source_window_utc']['to']) >= tick.end_ms-1]
        j = dict(job, required_m30_slots=required, preparation_segments=parts, tick_mode='unspent')
        if sources:
            e = min(sources, key=lambda v: v['tick_receipt_sha256'])
            j.update(tick_mode='verified_cache', source_path=e['source_path'], tick_receipt_sha256=e['tick_receipt_sha256'])
        plan['episodes'].append(j)
        ids = []
        for part in parts:
            r = dict(part, account_id=key[0], symbol=key[1])
            identity = canonical(r)
            global_parts.setdefault(identity, dict(r, id=identity, consumers=[]))['consumers'].append(job['episode_id'])
            ids.append(identity)
        counts = collections.Counter()
        for part in parts:
            counts[part['mode']] += (part['to_ms']-part['from_ms'])//STEP
        dependency = [Interval(job['symbol'],job['account_id'],p['from_ms'],p['to_ms'],'m30') for p in parts]
        inspected = {s for p in dependency for s in by_key[key] if p.overlaps(s)}
        episode_rows.append({'episode_id': job['episode_id'], 'symbol': job['symbol'], 'date': job['date'],
            'native_window': {'from': job['from'], 'to': job['to'], 'tail_through': job['tail_through']},
            'required_m30_slots': len(required), 'required_m30_slots_sha256': canonical(required),
            'slot_counts': dict(counts), 'preparation_partition_ids': ids,
            'covering_native_tick_sources': [e['episode_id'] for e in sources],
            'tick_window_prior_inspection_roles': dict(collections.Counter(s.role for s in overlaps)),
            'preparation_prior_inspection_roles': dict(collections.Counter(s.role for s in inspected))})
    source_path = work/'unified-source-manifest.json'
    save(source_path, manifest)
    plan['manifest_receipt'] = {'path': str(source_path), 'sha256': sha(source_path)}
    plan_path = work/'expanded-preflight-plan.json'
    save(plan_path, plan)
    stage_results, current_command_results = {}, {}
    for stage in ('preparation', 'ticks', 'evaluation'):
        try:
            proof = preflight_file(plan_path, stage)
            current_command_results[stage] = {'status': 'PASSED', 'proof': proof}
        except Refusal as exc:
            current_command_results[stage] = {'status': 'REFUSED', 'reason': str(exc)}
        # Audit the exact frozen engine as well. Current main includes the owner-
        # amended fixed-window wrapper; do NOT disguise its different source hash
        # as the frozen candidate or weaken the canonical command's refusal.
        try:
            proof = verify_plan(plan, registry, original_meta, store.bars, refs, store.verified, stage=stage)
            stage_results[stage] = {'status': 'PASSED', 'proof': proof}
        except (Refusal, KeyError) as exc:
            stage_results[stage] = {'status': 'REFUSED', 'reason': str(exc)}
    # Original 256 request slices are not interchangeable with decoded bars or
    # newly partitioned gap requests. Report their exclusive status explicitly.
    original_segments = []
    for s in spec['independent_evaluation_plan']['m30_segments']:
        consumers = [j for j in jobs if j['symbol'] == s['symbol'] and
                     j['date'] in next(w['dates'] for w in spec['independent_evaluation_plan']['weeks'] if w['dates'][0] == s['week_start'])]
        required = sorted({t for j in consumers for t in slots(meta[j['episode_id']]['prior_45_calendar_days'])
                           if ms(s['from']) <= t and t+STEP <= ms(s['to'])})
        key = (49236051, s['symbol'])
        available = [t for t in required if t in store.bars[key] and refs[key].get(t)]
        missing = sorted(set(required)-set(available))
        spent_missing = [t for t in missing if any(Interval(s['symbol'],49236051,t,t+STEP,'m30').overlaps(i) for i in by_key[key])]
        status = ('no_native_slots' if not required else 'complete_verified_cache' if not missing else
                  'blocked_spent_missing_bars' if spent_missing else 'partial_cache_unspent_gaps' if available else 'missing_unspent')
        original_segments.append({**s, 'required_native_slots': len(required), 'cached_native_slots': len(available),
            'missing_native_slots': len(missing), 'spent_missing_native_slots': len(spent_missing), 'status': status})
    dependencies = [dict(symbol=j['symbol'],account_id=j['account_id'],start_ms=ms(j['from']),
                         end_ms=ms(j['tail_through'])+1,role='ticks') for j in jobs]
    dependencies += [dict(symbol=p['symbol'],account_id=p['account_id'],start_ms=p['from_ms'],end_ms=p['to_ms'],role='m30')
                     for p in global_parts.values()]
    reservation = {'candidate_id': registry['identity']['candidate_id'], 'maximum_trials': 1,
        'registered_candidates': [registry['identity']['candidate_id']], 'dependencies': dependencies,
        'embargo_ms': 4*3600000+5001, 'maximum_holding_ms': 4*3600000+5001, 'development_labels': []}
    try:
        split_budget_guard(registry['identity'], reservation, registry['spent_intervals'], [])
        inspection = {'status': 'PASSED', 'note': 'local retained ledger only; unavailable older references still block certification'}
    except Refusal as exc:
        inspection = {'status': 'REFUSED', 'reason': str(exc)}
    store.finish()
    out = {'frozen_candidate': registry['identity']['candidate_id'], 'source_spec_sha256': sha(old.SPEC),
        'registry_sha256': sha(REGISTRY), 'source_manifest_sha256': sha(source_path), 'expanded_plan_sha256': sha(plan_path),
        'original_jobs_sha256': canonical(jobs), 'planned_independent_episodes': len(jobs),
        'indexed_episode_definitions': len(manifest['episodes']), 'indexed_m30_sources': len(manifest['m30_segments']),
        'verified_local_source_files': len(store.verified), 'source_hashes_reverified': True,
        'original_m30_segments': original_segments, 'episodes': episode_rows,
        'preparation_partitions': list(global_parts.values()), 'preflight': stage_results,
        'current_main_command_preflight': current_command_results, 'exact_frozen_engine_receipt': frozen_engine,
        'inspection_and_trial_guard': inspection, 'maximum_candidate_trials': 1,
        'new_trials_opened': 0, 'trial_ledger_completeness': 'older missing reference audit prevents complete historical trial certification',
        'reservation_status': 'AUDITED_NOT_RESERVED_COLLECTION_PAUSED', 'collection_authority': False,
        'summary': {'original_m30_segments': len(original_segments),
            'original_segment_statuses': dict(collections.Counter(s['status'] for s in original_segments)),
            'already_cached_m30_segments': sum(s['status']=='complete_verified_cache' for s in original_segments),
            'missing_m30_segments_to_fetch': sum(bool(s['missing_native_slots']) for s in original_segments),
            'missing_segment_count_is_not_permission_to_refetch_spent_gaps': True,
            'missing_native_tick_episodes_to_fetch': sum(not e['covering_native_tick_sources'] for e in episode_rows),
            'episodes_with_previously_inspected_tick_window': sum(bool(e['tick_window_prior_inspection_roles']) for e in episode_rows),
            'episodes_with_inspected_preparation': sum(bool(e['preparation_prior_inspection_roles']) for e in episode_rows)},
        'new_price_requests': 0, 'validation_opened': False}
    save(work/'reservation.json', out)
    return out


if __name__ == '__main__':
    import json
    from rebuild_preflight import offline_process
    offline_process()
    result = run(Path('/workspace/jiaoyi-gates-5-9-18370/reservation'))
    print(json.dumps({'summary': result['summary'], 'preflight': result['preflight']}, indent=2))
