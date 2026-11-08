"""Versioned reservation-only policy permitting verified causal warm-up reuse.

Only the owner-authorised №18,372 transition uses this policy. It leaves the
old strict guard and every collection boundary untouched. Planning PASS never
means data is decoded, the broker will supply it, or collection is authorised.
"""
import bisect
import collections
import datetime as dt
from pathlib import Path

import phase5b_audit as old
from rebuild import engine_metadata, REGISTRY
from rebuild_preflight import require, Interval, verify_transition, verify_plan, Refusal
from replay_inventory import ROOT, CORE, canonical, read, save, sha, ms, stamp, permitted, market
from tiered_replay import CleanStore
from unified_replay import slots
from rebuild_reservation import partition_slots
from calendar_authority import verify_current, verify_historical

PLAN=ROOT/'docs/rebuild_rfc_260deals_analysis_plan_18372.json'
WORK=Path('/workspace/jiaoyi-cost-reservation-18372')
CLASSES={'SpotCrude':'energy','SpotBrent':'energy','US500':'equity_index','NAS100':'equity_index',
         'XAUUSD':'metal','XAGUSD':'metal','BTCUSD':'crypto','ETHUSD':'crypto'}


def merged(intervals):
    output=[]
    for a,b in sorted(intervals):
        if output and a<=output[-1][1]:output[-1]=(output[-1][0],max(b,output[-1][1]))
        else:output.append((a,b))
    return output


def overlaps(ranges,a,b):
    i=bisect.bisect_left(ranges,(b,))-1
    return i>=0 and ranges[i][1]>a


def eligibility(job, meta, bars, spent_ranges, embargo_ranges):
    s=meta['session']
    if not s:return {'reasons':['no_native_session']}
    start=ms(s['open']);end=start+14400000+5001
    required=slots(meta['prior_45_calendar_days'])
    if not permitted(start,end) or any(not permitted(t,t+1800000) for t in required):
        return {'reasons':['protected_dependency_or_cutoff']}
    reasons=[]
    if overlaps(spent_ranges,start,end):reasons.append('evaluation_price_interval_already_inspected')
    if overlaps(embargo_ranges,start,end):reasons.append('development_label_embargo')
    missing=[t for t in required if t not in bars]
    blocked=[t for t in missing if overlaps(spent_ranges,t,t+1800000)]
    if blocked:reasons.append('spent_missing_preparation')
    return {'reasons':reasons,'required':required,'missing':missing,'blocked_slots':len(blocked),
        'from':stamp(start),'to':stamp(start+14400000),'tail_through':stamp(end-1)}


def verify_reuse_reservation(plan, registry, metadata, bars, refs, verified, old_registry, trials):
    """Layer split/trial/receipt checks around the existing preparation verifier."""
    verify_transition(old_registry['identity'],plan['identity'],registry['transition'],registry['transitions'])
    require(plan['identity']==registry['identity'],'reservation_identity')
    require(canonical(plan['episodes'])==registry['frozen_independent_jobs_sha256'],'reservation_jobs_changed')
    require(len(trials)<=1 and all(t['candidate_id']==plan['identity']['candidate_id'] for t in trials),'reservation_trial_budget')
    require(all(t.get('trial_id') and t.get('status') in ('running','passed','failed','unresolved') for t in trials),'reservation_trial_record')
    inspected=collections.defaultdict(list)
    for s in old_registry['spent_intervals']:inspected[(s['account_id'],s['symbol'])].append((s['start_ms'],s['end_ms']))
    for job in plan['episodes']:
        key=(job['account_id'],job['symbol']);start,end=ms(job['from']),ms(job['tail_through'])+1
        require(not overlaps(merged(inspected[key]),start,end),'reservation_seen_evaluation_window')
        required=set(job['required_m30_slots'])
        for segment in job['preparation_segments']:
            if segment['mode']=='verified_cache':
                for t in required:
                    if segment['from_ms']<=t<t+1800000<=segment['to_ms']:
                        require(t in bars[key] and segment['source_path'] in refs[key].get(t,()),'reuse_cache_not_decoded')
    # Existing verifier still rejects spent gaps, protected dates, wrong native
    # windows, incomplete partitions and changed identity. Only preparation
    # inspection reuse is changed by the explicit transition above.
    proof=verify_plan(plan,registry,metadata,bars,refs,verified,stage='preparation')
    return {'status':'PASSED_LOCAL_RESERVATION_ONLY','refusal_count':0,'proof':proof,
        'collection_authority':False,'evaluation_ready':False,'external_registry_completeness':'local retained collection ledger scope'}


def run():
    cfg=read(PLAN);scope=cfg['reservation'];registry=read(REGISTRY)
    p,_,ends=old.phase5_receipts();manifest=old.combined_manifest(p,ends)
    store=CleanStore(manifest).load();refs=collections.defaultdict(lambda:collections.defaultdict(set))
    for key,paths in store.bar_sources.items():
        for path in paths:
            for bar in market(read(path))[2]:refs[key][bar['ts_ms']].add(path)
    raw_ranges=collections.defaultdict(list)
    for s in registry['spent_intervals']:raw_ranges[(s['account_id'],s['symbol'])].append((s['start_ms'],s['end_ms']))
    spent={k:merged(v) for k,v in raw_ranges.items()}
    prior=read(old.RESULT);used={r['episode_id'] for r in prior['rows']}
    embargo=collections.defaultdict(list)
    for e in manifest['episodes']:
        if e['episode_id'] in used:
            embargo[(e['account_id'],e['symbol'])].append((ms(e['source_window_utc']['from']),ms(e['source_window_utc']['to'])+14400000+5001))
    embargo={k:merged(v) for k,v in embargo.items()}
    inventory=read(ROOT/'docs/rebuild_rfc_offline_gates_completion_18368.json')['unified_manifest']
    old_meta=engine_metadata(manifest['episodes'])
    require(old_meta==old.metadata({'episodes':manifest['episodes']}),'frozen_inventory_calendar_drift')
    capture=read(ROOT/'docs/rebuild_rfc_gates_5_to_9_evidence_18370.json')
    authorities={}
    for a in capture['accounts']:
        names={s['symbolId']:s['symbolName'] for s in a['selected_catalog_rows']}
        for raw in a['contracts']['symbol']:
            name=names[raw['symbolId']]
            if name in CORE:authorities[(a['account_id'],name)]=verify_current(a['account_id'],name,raw,a['contracts']['ctidTraderAccountId'],stamp(capture['capture_plan']['to_ms']))
    on_disk=[]
    for e in inventory['episodes']:
        if e['classification']!='tier1_native_complete':continue
        cov=e['native_coverage']['tier1_coverage']
        proof=verify_historical(authorities[(e['account_id'],e['symbol'])],e,old_meta[e['episode_id']],
            store.bars[(e['account_id'],e['symbol'])],cov['opening_quote'],cov['closing_tail_quote'])
        _,quotes,_,_=market(store.source(e))
        start=ms(old_meta[e['episode_id']]['session']['open']);end=start+14400000
        sides={side:[q for q in quotes[side] if start<=q['ts_ms']<=end+5000] for side in ('bids','asks')}
        gaps=sum(b['ts_ms']-a['ts_ms']>6000 for side in sides.values() for a,b in zip(side,side[1:]))
        ambiguous=0
        for side in sides.values():
            grouped=collections.defaultdict(set)
            for q in side:grouped[q['ts_ms']].add(q['price_e5'])
            ambiguous+=sum(len(v)>1 for v in grouped.values())
        through=bool(all(sides.values())) and min(side[-1]['ts_ms'] for side in sides.values())>=end
        path_qualified=proof['status']=='QUALIFIED_PROXY_REPLAY' and gaps==0 and ambiguous==0 and through
        on_disk.append({'episode_id':e['episode_id'],'symbol':e['symbol'],'date':e['date'],
            'used_by_candidate':e['episode_id'] in used,'authority':proof,
            'full_execution_path_qualified':path_qualified,
            'observed_path_checks':{'side_gaps_over6s':gaps,'ambiguous_side_timestamps':ambiguous,'both_sides_through_end':through},
            'path_claim_scope':'existing G4 observed-quote criterion only; does not prove broker fill/execution parity'})
    start=dt.date.fromisoformat(scope['search_first_date']);end=dt.date.fromisoformat(scope['last_date_exclusive'])
    days=[start+dt.timedelta(days=i) for i in range((end-start).days) if (start+dt.timedelta(days=i)).weekday()<5]
    jobs=[dict(episode_id=f'{s}-{d}',symbol=s,account_id=49236051,asset_class=CLASSES[s],date=str(d)) for d in days for s in CORE]
    meta=engine_metadata(jobs)
    require(meta==old.metadata({'episodes':jobs}),'frozen_search_calendar_drift')
    by_date=collections.defaultdict(list);candidates=[];rejections=collections.Counter()
    for job in jobs:
        key=(49236051,job['symbol']);v=eligibility(job,meta[job['episode_id']],store.bars[key],spent.get(key,[]),embargo.get(key,[]))
        by_date[job['date']].append((job,v))
        rejections.update(v['reasons'])
    for monday in [d for d in days if d.weekday()==0 and d+dt.timedelta(days=4)<end]:
        rows=[r for i in range(5) for r in by_date[str(monday+dt.timedelta(days=i))]]
        if len(rows)!=40 or any(v['reasons'] for _,v in rows):continue
        missing={(j['symbol'],t) for j,v in rows for t in v['missing']}
        required={(j['symbol'],t) for j,v in rows for t in v['required']}
        candidates.append({'monday':str(monday),'rows':rows,'missing_unique_slots':len(missing),
            'cached_unique_slots':len(required)-len(missing),'prepare_from':min(t for _,t in required),
            'evaluation_end':max(ms(v['tail_through'])+1 for _,v in rows)})
    chosen=[]
    for week in sorted(candidates,key=lambda w:(w['missing_unique_slots'],w['monday'])):
        # The frozen plan requires non-overlapping CALENDAR weeks, not invented
        # 45-day separation. Shared causal warm-up is disclosed below. All arms
        # stay frozen for the whole batch; no tuning on earlier test weeks.
        first=ms(week['monday']+'T00:00:00Z')
        if any(abs(first-ms(w['monday']+'T00:00:00Z'))<7*86400000 for w in chosen):continue
        chosen.append(week)
        if len(chosen)==scope['target_weeks']:break
    save(WORK/'search-diagnostic.json',{'candidate_weeks':[{k:v for k,v in c.items() if k!='rows'} for c in candidates],
        'selected':[w['monday'] for w in chosen],'reasons':dict(rejections),
        'range_examples':{s:spent.get((49236051,s),[]) for s in CORE},
        'first_date_reasons':{j['symbol']:v['reasons'] for j,v in by_date[str(start)]}})
    require(len(chosen)==scope['target_weeks'],'insufficient_clean_weeks_no_forced_reservation')
    chosen.sort(key=lambda w:w['monday']);selected=[]
    for week in chosen:
        for job,v in week['rows']:
            key=(49236051,job['symbol'])
            parts=partition_slots(v['required'],store.bars[key],refs[key],store.verified,
                [Interval(job['symbol'],49236051,a,b,'price') for a,b in spent.get(key,[])],*key)
            require(all(p['mode']!='blocked_spent' for p in parts),'generator_selected_spent_gap')
            selected.append(dict(job,**{k:v[k] for k in ('from','to','tail_through')},
                required_m30_slots=v['required'],preparation_segments=parts,tick_mode='unspent'))
    identity=dict(registry['identity'],rfc_state=scope['state'])
    transition={'from_identity':registry['identity'],'to_identity':identity,'owner_trace':'№18,372 Part C',
        'reviewed_commit':cfg['base_commit'],'policy_sha256':sha(PLAN),
        'change':'Verified causal preparation reuse permitted; labels unseen; same frozen candidate engine/native sessions/costs; reservation only'}
    new_registry=dict(registry,identity=identity,transition=transition,transitions=[transition],
        collection_enabled=False,frozen_independent_jobs_sha256=canonical(selected),authority_policy=cfg['calendar_authority'])
    plan={'identity':identity,'scheduled_episodes':len(selected),'episodes':selected,'collection_enabled':False}
    proof=verify_reuse_reservation(plan,new_registry,meta,store.bars,refs,store.verified,registry,[])
    ticks_refusal=None
    try:verify_plan(plan,new_registry,meta,store.bars,refs,store.verified,stage='ticks')
    except Refusal as ex:ticks_refusal=str(ex)
    # Deduplicate future missing requests; merge contiguous native slots only,
    # never bridge a spent/failed/no-required-slot interval for convenience.
    future=collections.defaultdict(list);cache=set()
    for j in selected:
        for part in j['preparation_segments']:
            if part['mode']=='unspent':future[j['symbol']].append((part['from_ms'],part['to_ms']))
            else:cache.add((j['symbol'],part['source_path'],part['sha256']))
    requests=[]
    for sym,ranges in sorted(future.items()):
        for first,last in merged(ranges):
            for a in range(first,last,7*86400000):
                b=min(last,a+7*86400000)
                requests.append({'account_id':49236051,'symbol':sym,'from':stamp(a),'to':stamp(b),
                    'period':'M30','request_mode':'interval_only_no_count'})
    frozen=old.frozen_identity();require(frozen==read(old.RESULT)['frozen_phase4'],'frozen_engine_changed')
    store.finish()
    save(WORK/'expanded-reservation-plan.json',plan)
    out={'transition':transition,'registry':{k:v for k,v in new_registry.items() if k not in ('spent_intervals','hypothesis')},
        'proof':proof,'ticks_stage_refusal':ticks_refusal,'search':{'from':str(start),'to_exclusive':str(end),
            'symbol_date_definitions':len(jobs),'clean_complete_weeks':len(candidates),'excluded_reason_counts':dict(rejections),
            'selection':'minimum distinct missing native M30 slots, then date; non-overlapping calendar weeks'},
        'selected_weeks':[{k:v for k,v in w.items() if k!='rows'} for w in chosen],
        'episodes':[dict(j,required_m30_slots_sha256=canonical(j['required_m30_slots'])) for j in selected],
        'new_m30_request_intervals':requests,'reused_m30_sources':[{'symbol':s,'path':p,'sha256':h} for s,p,h in sorted(cache)],
        'summary':{'refusal_count':proof['refusal_count'],'planned_episodes':len(selected),
            'reused_verified_m30_segments':len(cache),'new_m30_segments_needed':len(requests),'new_tick_episodes_needed':len(selected)},
        'on_disk_complete_native_scenarios':on_disk,'untouched_complete_tier1_episodes_already_on_disk':sum(not r['used_by_candidate'] for r in on_disk),
        'full_execution_path_qualified_on_disk':sum(r['full_execution_path_qualified'] for r in on_disk),
        'frozen_engine_receipt':frozen,'verified_market_source_files':len(store.verified),'new_broker_requests':0,
        'not_a_collection_permit':True,'historical_calendar_authority':False,
        'intrabatch_dependence':{'causal_preparation_may_include_earlier_selected_week':True,
            'constraint':'No refitting, candidate selection or adaptive stopping after any week. Freeze and collect the entire batch before reporting any arm outcomes.',
            'uncertainty':'Four calendar weeks need not be four independent market regimes; account for clustered outcomes.'},
        'registry_scope_limit':'Local retained request/inspection ledger; older unavailable non-ledger references remain disclosed. No unseen price outcomes read.'}
    save(WORK/'reservation.json',out)
    return out


if __name__=='__main__':
    from rebuild_preflight import offline_process
    import json
    offline_process();r=run();print(json.dumps({'summary':r['summary'],'weeks':r['selected_weeks'],'on_disk':len(r['on_disk_complete_native_scenarios']),'untouched':r['untouched_complete_tier1_episodes_already_on_disk']},indent=2))
