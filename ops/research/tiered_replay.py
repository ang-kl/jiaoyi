#!/usr/bin/env python3
"""№18,360. Manifest-only coverage audit and finite tiered experiment; zero collection."""
import argparse
import collections
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import unified_replay as u
from replay_inventory import ROOT, read, save, sha, ms, stamp, market, permitted, checkpoints, canonical

PLAN=ROOT/'docs/rebuild_phase3_coverage_and_pullback_plan_18360.json'
RESULT=ROOT/'docs/rebuild_phase3_coverage_and_pullback_results_18360.json'
PHASE2=ROOT/'docs/rebuild_phase2_runner_and_pullback_results_18358.json'
WORK=Path('/workspace/jiaoyi-rebuild-phase3-18360')
COSTS=u.BPS+['demo_entry_plus_2_exit','demo_entry_plus_3.74_exit']

def partial_context(sessions,bars,at):
    """Fixed5/20 completed-session endpoint drift, not a shorter lookback fallback."""
    result={'status':'missing_segments','drift_5':None,'drift_20':None,'latest_bar_end':None}
    if len(sessions)<20:return result
    sets=[u.slots(sessions[-n:]) for n in (5,20)]
    if any(not s for s in sets):return result
    times={t for s in sets for t in [s[0],s[-1]]}
    if any(t+1800000>at or not permitted(t,t+1800000) for t in times):raise ValueError('noncausal/protected endpoint')
    if any(t not in bars for t in times):return result
    for n,s in zip((5,20),sets):result['drift_'+str(n)]=bars[s[-1]]['close_e5']/bars[s[0]]['open_e5']-1
    result.update(status='complete',latest_bar_end=stamp(max(times)+1800000))
    return result

class CleanStore(u.ManifestStore):
    def load(self):
        self.excluded_bar_sources=[]
        repeated={(e['symbol'],e['cohort']) for e in self.manifest['episodes'] if e['protocol_status']=='inherited_repeat_interval'}
        for e in self.manifest['episodes']:
            if not e['eligible_for_research'] or e['protocol_status']!='clean_first_read':continue
            _,_,bars,_=market(self.source(e))
            self.merge_bars(e['account_id'],e['symbol'],bars,e['source_path'])
        for s in self.manifest['m30_segments']:
            repeat=any(s['symbol']==sym and 'three-year-pilot' in s['source_path'] and ('bar-'+cohort+'-'+sym+'-') in s['source_path'] for sym,cohort in repeated)
            ledger=any(r['kind']=='collection_ledger' for r in s['receipt_sources'])
            if not s['eligible_for_research'] or repeat or not ledger:
                self.excluded_bar_sources.append({'path':s['source_path'],'sha256':s['receipt_sha256'],'reason':'inherited_repeat_m30_interval' if repeat else 'no_first_collection_ledger_or_protected'})
                continue
            x=u.immutable_read(s['source_path'],s['receipt_sha256'],self.quarantine)
            self.verified[s['source_path']]=s['receipt_sha256']
            self.merge_bars(s['account_id'],s['symbol'],x['bars'],s['source_path'])
        return self

def tier2_input(e,store,meta,peer,join):
    reasons=[];s=meta[e['episode_id']]['session']
    if not s:return None,{'reasons':['no_coded_session']}
    start=ms(s['open']);end=start+14400000
    if e['protocol_status']!='clean_first_read':reasons.append('receipt_provenance_not_clean_first_read')
    if not permitted(start,end+5001):reasons.append('protected_or_cutoff_window')
    if ms(e['source_window_utc']['from'])>start or ms(e['source_window_utc']['to'])<end:reasons.append('tick_interval_does_not_cover_native_four_hours')
    coverage={'reasons':reasons,'window':{'from':stamp(start),'to':stamp(end)},'continuous_executable_path_verified':False}
    if reasons:return None,coverage
    _,q,_,_=market(store.source(e))
    source={'contract':q['contract'],'bars':[],**{side:[t for t in q[side] if start<=t['ts_ms']<=end+5000] for side in ['bids','asks']}}
    points=checkpoints(source['bids'],source['asks'],{'open':start,'warmup_end':start+1800000,'end':end},source['contract']['digits'])
    coverage['quote_checkpoints']=points
    if points['open'] is None:reasons.append('missing_fresh_open_quote')
    if points['warmup_end'] is None:reasons.append('missing_fresh_30m_quote')
    if any(not source[side] or source[side][-1]['ts_ms']<end-6000 for side in ['bids','asks']):reasons.append('missing_tick_coverage_at_boundary')
    if reasons:return None,coverage
    bars=store.bars[(e['account_id'],e['symbol'])];sessions=meta[e['episode_id']]['prior_45_calendar_days']
    own=partial_context(sessions,bars,start);peer_context=None
    if peer:peer_context=partial_context(meta[peer['episode_id']]['prior_45_calendar_days'],store.bars[(peer['account_id'],peer['symbol'])],start)
    required=u.slots(sessions)
    coverage.update(status='qualified_partial_context_scenario',missing_45day_slots=sum(t not in bars for t in required),own_endpoint_status=own['status'],peer_endpoint_status=peer_context['status'] if peer_context else 'absent',peer_episode_id=peer['episode_id'] if peer else None)
    future=[]
    for t,b in sorted(bars.items()):
        if t<end:continue
        if t>=end+72*3600000 or not permitted(t,t+1800000):break
        future.append(b)
    request={'account':e['account_id'],'symbol':e['symbol'],'from':stamp(start),'to':stamp(end),'spread':0.,'commission_round_trip':0.,'slippage_bps_per_side':24.,'tick_replay':{'volume':e['native_volume']}}
    return {'request':request,'source':source,'future':future,'own_context':own,'peer_context':peer_context,'peer_available':peer is not None,'demo_entry_bps':join.median_entry_slippage_bps},coverage

def decomposition(manifest,previous):
    selected={r['episode_id'] for r in previous['rows']};old={r['episode_id']:r for r in previous['excluded'] if r['symbol'] in u.CORE}
    rows=[]
    for e in manifest['episodes']:
        if e['symbol'] not in u.CORE or not e['eligible_for_research']:continue
        reasons=[];p=e['protocol_status'];cov=old.get(e['episode_id'],{}).get('coverage',{})
        if p=='inherited_repeat_interval':reasons.append('inherited_repeat_m30_interval')
        elif p!='clean_first_read':reasons.append('unverified_receipt_history')
        for r in cov.get('reasons',[]):
            reasons.append('missing_m30_preparation_for_symbol_or_cohort' if r=='missing_45day_session_preparation' else r)
        failed=[]
        start=ms(e['session_window_utc']['open'])
        for f in manifest['failed_sources']:
            if f['symbol']!=e['symbol'] or f['reason']!='bar_identity_or_truncation_unverified':continue
            a,b=(ms(f['request_window_utc'][k]) for k in ['from','to'])
            if a<start and b>start-45*86400000:failed.append({'path':f['path'],'sha256':f['sha256'],'from':stamp(a),'to':stamp(b)})
        if failed and 'missing_m30_preparation_for_symbol_or_cohort' in reasons:reasons.append('failed_m30_identity_or_truncation_overlap')
        is_selected=e['episode_id'] in selected
        rows.append({k:e[k] for k in ['episode_id','account_id','symbol','date','cohort','source_path','tick_receipt_sha256','protocol_status','protocol_flags']}|{
            'tier1_selected':is_selected,'phase2_exclusion_reasons':sorted(set(reasons)),
            'primary_exclusion_reason':None if is_selected else reasons[0] if reasons else 'unclassified',
            'phase2_coverage':cov,'failed_m30_interval_overlaps':failed,
            'truncated_m30_count500_defect':'not_proven_by_retained_failed_receipt' if failed else 'no_linked_failed_segment'})
    excluded=[r for r in rows if not r['tier1_selected']]
    if len(rows)!=263 or len(selected)!=35 or len(excluded)!=228 or any(r['primary_exclusion_reason']=='unclassified' for r in excluded):raise ValueError('denominator drift')
    return {'core_definitions':263,'phase2_qualified':35,'phase2_excluded':228,'rows':rows,
        'excluded_provenance':dict(collections.Counter(r['protocol_status'] for r in excluded)),
        'exclusive_primary_reasons':dict(collections.Counter(r['primary_exclusion_reason'] for r in excluded)),
        'overlapping_reasons':dict(collections.Counter(k for r in excluded for k in r['phase2_exclusion_reasons'])),
        'quarantined_outside263':[e for e in manifest['episodes'] if not e['eligible_for_research']],
        'failed_sources_outside263':manifest['failed_sources'],
        'quarantined_duplicate_receipts':[e for e in manifest['duplicate_receipts'] if not e['eligible_for_research']]}

def paired(rows,mode,cost):
    groups=collections.defaultdict(list);detail=[]
    for r in rows:
        n=r['native'];arms=n.get('arms',{});b=arms.get(u.ARMS[1],{}).get(mode)
        if not b:continue
        reason=n['macro']['reason']
        if n['macro']['pass']:category='retained'
        elif reason=='own_5d20d_disagree_or_missing':category='own_trend_veto' if r['own_context']['status']=='complete' else 'own_context_unavailable'
        elif reason=='peer_conflict':category='peer_trend_veto'
        else:category='peer_context_unavailable'
        groups[category].append(r)
        detail.append({'episode_id':r['episode_id'],'symbol':r['symbol'],'date':r['date'],'category':category,'b_net_r':b.get('metrics',{}).get('net_by_bps',{}).get(cost) if b.get('metrics') else None})
    return {'categories':{k:u.score(v,u.ARMS[1],mode,cost) for k,v in sorted(groups.items())},'pairs':detail,
        'interpretation':'Directional veto and missing-data abstention separated; development-only conditional association, not incremental alpha proof.'}

def freeze(work):
    work.mkdir(parents=True,exist_ok=True);base=work/'base';base.mkdir(exist_ok=True)
    value=u.freeze_code(PLAN,base)
    names=['ops/research/tiered_replay.py','ops/research/test_tiered_replay.py','crates/jiaoyi-core/examples/tiered_pullback_replay.rs']
    for n in names:
        h=sha(ROOT/n)
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+n],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit before evaluation:'+n)
        value['source_hashes'][n]=h
    value['binary_hashes']['tiered_pullback_replay']=sha(u.BIN/'tiered_pullback_replay')
    path=work/'code-freeze.json'
    if path.exists() and read(path)!=value:raise ValueError('frozen implementation changed')
    if not path.exists():save(path,value)
    return value

def run(work=WORK,output=RESULT):
    sys.addaudithook(u.deny_network);plan=read(PLAN);u.verify_inputs(plan);frozen=freeze(work)
    manifest=read(ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json');previous=read(PHASE2)
    audit=decomposition(manifest,previous);store=u.ManifestStore(manifest).load();meta=u.native_metadata(manifest)
    joins=u.cost_joins(read(ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json'),manifest)
    legacy,hits=u.parity(plan,store,work,frozen)
    peers={a:b for pair in plan['peer_pairs'] for a,b in [pair,list(reversed(pair))]}
    eligible=[e for e in manifest['episodes'] if e['eligible_for_research']]
    def peer_for(e,clean=False):
        options=[p for p in eligible if p['account_id']==e['account_id'] and p['symbol']==peers[e['symbol']] and p['date']==e['date'] and (not clean or p['protocol_status']=='clean_first_read')]
        return min(options,key=lambda x:x['episode_id']) if options else None
    selected=[];phase2_parity=[];checks=0
    for old in previous['rows']:
        e=store.episodes[old['episode_id']];inp,cov=u.qualified_input(e,store,meta,peer_for(e),joins[u.CostKey(e['account_id'],e['symbol'])])
        path=work/('input-'+e['episode_id']+'.json');save(path,inp)
        if sha(path)!=old['input_sha256']:raise ValueError('Phase2 normalized input changed')
        n,hit,key=u.cached_native('manifest_pullback_replay',path,e['episode_id'],work,frozen);hits+=hit
        if n!=old['native']:raise ValueError('Phase2 native parity failed:'+e['episode_id'])
        phase2_parity.append({'episode_id':e['episode_id'],'exact':True,'receipt_key':key})
        selected.append((e,inp,cov,'tier1'))
    print('PARITY Phase2 35/35 exact',flush=True)
    clean=CleanStore(manifest).load();candidates=[];tier2_excluded=[];aliases=[]
    for row in audit['rows']:
        if row['tier1_selected']:continue
        e=store.episodes[row['episode_id']]
        if e['protocol_status']!='clean_first_read':
            tier2_excluded.append({'episode_id':e['episode_id'],'reasons':['receipt_provenance_not_clean_first_read']});continue
        inp,cov=tier2_input(e,clean,meta,peer_for(e,True),joins[u.CostKey(e['account_id'],e['symbol'])])
        if inp is None:tier2_excluded.append({'episode_id':e['episode_id'],**cov});continue
        candidates.append((e,inp,cov,'tier2'))
    used={(e['account_id'],e['symbol'],e['date']):e['episode_id'] for e,_,_,_ in selected}
    for item in sorted(candidates,key=lambda x:(-(ms(x[0]['source_window_utc']['to'])-ms(x[0]['source_window_utc']['from'])),x[0]['tick_receipt_sha256'])):
        e=item[0];k=(e['account_id'],e['symbol'],e['date'])
        if k in used:aliases.append({'episode_id':e['episode_id'],'canonical_episode_id':used[k]});continue
        used[k]=e['episode_id'];selected.append(item)
    print('COVERAGE '+json.dumps(dict(collections.Counter(t for _,_,_,t in selected))),flush=True)
    audit.update(tier2_selected=sum(t=='tier2' for _,_,_,t in selected),tier2_excluded=tier2_excluded,same_session_aliases=aliases,excluded_m30_sources=clean.excluded_bar_sources)
    decisions={e['episode_id']:{'tier':t,'coverage':cov} for e,_,cov,t in selected}
    for r in audit['rows']:r['phase3_qualification']=decisions.get(r['episode_id'],{'tier':None})
    save(work/'coverage-before-outcomes.json',audit)
    rows=[]
    for e,inp,cov,tier in sorted(selected,key=lambda v:(v[3],v[0]['symbol'],v[0]['date'])):
        for threshold in plan['thresholds_bps']:
            for admission in plan['admission_bps_per_side']:
                policy={'tier':tier,'threshold_bps':threshold,'admission_bps':admission}
                path=work/'variant-input.json';save(path,{'input':inp,'policy':policy})
                identity=[e['episode_id'],policy]
                n,hit,key=u.cached_native('tiered_pullback_replay',path,identity,work,frozen);hits+=hit
                if tier=='tier1' and threshold==48 and admission==24:
                    old=next(r for r in previous['rows'] if r['episode_id']==e['episode_id'])
                    if n!=old['native']:raise ValueError('policy default parity failed')
                checks+=u.check_arithmetic(n)
                rows.append({k:e[k] for k in ['episode_id','account_id','symbol','date','cohort','tick_receipt_sha256','protocol_status']}|{
                    **policy,'input_sha256':sha(path),'native_receipt_key':key,'native':n,'own_context':inp['own_context'],'peer_context':inp['peer_context'],
                    'moderate_28_to48_impulse':bool(n.get('impulse') and 28<=n['impulse']['initial_r']/n['impulse']['fill']*10000<48)})
        if len(rows)%60==0:print('EXPERIMENT '+str(len(rows))+'/'+str(6*len(selected)),flush=True)
    summary={};paired_results={}
    for tier in ['tier1','tier2']:
        for threshold in plan['thresholds_bps']:
            for admission in plan['admission_bps_per_side']:
                cell=f'{tier}|floor={threshold:g}|admission={admission:g}'
                rs=[r for r in rows if r['tier']==tier and r['threshold_bps']==threshold and r['admission_bps']==admission]
                groups={'all':rs,'moderate_28_to48':[r for r in rs if r['moderate_28_to48_impulse']]}
                groups.update({sym:[r for r in rs if r['symbol']==sym] for sym in plan['symbols']})
                summary[cell]={g:{a:{m:{c:u.score(v,a,m,c) for c in COSTS} for m in u.MODES} for a in u.ARMS} for g,v in groups.items()}
                paired_results[cell]={m:{c:paired(rs,m,c) for c in u.BPS} for m in u.MODES}
    for s in [store,clean]:s.finish()
    u.verify_inputs(plan)
    result={'schema':'jiaoyi.tiered-offline-results.v1','trace':plan['trace'],'plan_sha256':sha(PLAN),'code_freeze':frozen,
        'coverage':audit,'parity':{'legacy_exact':len(legacy)==159,'legacy_rows':legacy,'phase2_exact':len(phase2_parity)==35,'phase2_rows':phase2_parity},
        'rows':rows,'summary':summary,'paired_macro':paired_results,'cost_evidence_joins':[asdict(v) for v in joins.values()],
        'verification':{'cost_arithmetic_checks':checks,'verified_source_files':len(set(store.verified)|set(clean.verified)),
            'coverage_frozen_before_new_outcomes_sha256':sha(work/'coverage-before-outcomes.json')},
        'new_broker_requests':0,'live_rules_changed':False,'validation_opened':False,'financial_pass':False,
        'new_independent_observations':0,'goal':plan['goal'],'historical_holdout_status':manifest['historical_holdout_status']}
    save(output,result);save(work/'last-run.json',{'native_cache_hits':hits,'variant_evaluations':len(rows),'result_sha256':sha(output)})
    print('DONE '+str(output)+' SHA256 '+sha(output),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--workdir',type=Path,default=WORK);p.add_argument('--output',type=Path,default=RESULT);args=p.parse_args();run(args.workdir,args.output)
