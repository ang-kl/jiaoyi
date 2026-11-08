#!/usr/bin/env python3
"""Canonical manifest-driven offline replay; no broker, database or model API imports."""
import argparse
import collections
from dataclasses import dataclass, asdict
import datetime as dt
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import sys

from replay_inventory import (ROOT, CORE, SEAL_START, SEAL_END, RESEARCH_CUTOFF, read, sha, canonical,
                              save, ms, stamp, market, permitted, load_verified_episode, checkpoints)

PLAN=ROOT/'docs/rebuild_phase2_runner_and_pullback_plan_18358.json'
RESULT=ROOT/'docs/rebuild_phase2_runner_and_pullback_results_18358.json'
DEFAULT_WORK=Path('/workspace/jiaoyi-rebuild-phase2-18358')
ARMS=['arm_a_breakout_baseline','arm_b_pure_pullback_retest','arm_c_macro_filtered_pullback']
MODES=['strict','forced','continuation']
BPS=['24','2','3.74']
BIN=ROOT/'target/debug/examples'

def deny_network(event,args):
    if event in ['socket.connect','socket.getaddrinfo']:raise RuntimeError('offline replay prohibits network')

def verify_inputs(plan):
    for name,h in plan['input_hashes'].items():
        if sha(ROOT/name)!=h:raise ValueError('frozen input mismatch: '+name)

def immutable_read(path,digest,quarantine):
    if digest in quarantine:raise ValueError('quarantined source receipt')
    if sha(path)!=digest:raise ValueError('source receipt mismatch: '+str(path))
    return read(path)

def slots(sessions):
    times=[]
    for s in sessions:
        a,b=ms(s['open']),ms(s['close']);lunch=s.get('lunch')
        times.extend(t for t in range(a,b,1800000) if t+1800000<=b and not(lunch and ms(lunch[0])<=t<ms(lunch[1])))
    return times

def causal_context(sessions,bars,at):
    required=slots(sessions)
    missing=[t for t in required if t not in bars]
    out={'status':'missing_segments','drift_5':None,'drift_20':None,'latest_bar_end':None}
    if missing or len(sessions)<20:return out
    if any(t+1800000>at or not permitted(t,t+1800000) for t in required):raise ValueError('context is not causal/protected')
    for n in [5,20]:
        use=slots(sessions[-n:]);first=bars[use[0]];last=bars[use[-1]]
        out['drift_'+str(n)]=last['close_e5']/first['open_e5']-1.0
    out.update(status='complete',latest_bar_end=stamp(required[-1]+1800000))
    return out

@dataclass(frozen=True)
class CostKey:
    account_id:int
    symbol:str

@dataclass
class CostEvidence:
    key:CostKey
    symbol_ids:list
    observed_entry_samples:list
    median_entry_slippage_bps:object
    position_money_evidence:list
    spread_evidence:str='recorded bid/ask from this episode; charged once in executable fills'
    round_trip_model_status:str='unverified_requires_broker_spec_and_matched_dispatch_receipts'
    benchmark_status:str='later demo entry benchmark only; not historical/live cost calibration'

def cost_joins(inventory,manifest):
    entries=inventory['execution_evidence']['attributed_entries'];keys={CostKey(e['account_id'],e['symbol']) for e in manifest['episodes']}
    keys.update(CostKey(e['account_id'],e['symbol']) for e in entries)
    ids=collections.defaultdict(set)
    for symbol,s in inventory['symbols'].items():
        for c in s['contracts']:ids[CostKey(c['account_id'],symbol)].add(c['symbol_id'])
        for c in s['recorded_broker_specs']:ids[CostKey(c['account_id'],symbol)].add(c['record']['symbolId'])
    out={}
    for key in sorted(keys,key=lambda k:(k.account_id,k.symbol)):
        observed=[e for e in entries if e['account_id']==key.account_id and e['symbol']==key.symbol and e['environment']=='demo']
        positions=[p for p in inventory['execution_evidence']['positions'] if p['account_id']==key.account_id and set(p['symbol_ids'])&ids[key]]
        median=statistics.median(e['slippage_bps'] for e in observed) if observed else None
        out[key]=CostEvidence(key,sorted(ids[key]),observed,median,positions)
    return out

class ManifestStore:
    def __init__(self,manifest):
        self.manifest=manifest
        self.quarantine={e.get('receipt_sha256',e.get('tick_receipt_sha256')) for e in manifest['episodes']+manifest['duplicate_receipts'] if not e['eligible_for_research']}
        self.episodes={e['episode_id']:e for e in manifest['episodes']}
        self.by_hash={e['tick_receipt_sha256']:e for e in manifest['episodes']}
        for d in manifest['duplicate_receipts']:self.by_hash[d['receipt_sha256']]=self.episodes[d['canonical_episode_id']]
        self.bars=collections.defaultdict(dict);self.bar_sources=collections.defaultdict(set);self.verified={}
    def merge_bars(self,account,symbol,bars,path):
        for b in bars:
            t=b['ts_ms']
            if t%1800000 or not permitted(t,t+1800000):raise ValueError('protected/off-grid M30')
            if b['volume']<0 or not 0<b['low_e5']<=min(b['open_e5'],b['close_e5'])<=max(b['open_e5'],b['close_e5'])<=b['high_e5']:raise ValueError('invalid M30 OHLC')
            key=(account,symbol);old=self.bars[key].get(t)
            if old is not None and old!=b:raise ValueError('conflicting M30 evidence')
            self.bars[key][t]=b;self.bar_sources[key].add(path)
    def source(self,e):
        if e['tick_receipt_sha256'] in self.quarantine or not e['eligible_for_research']:raise ValueError('quarantined episode')
        x=load_verified_episode(self.manifest,e['episode_id'])
        if x.get('request',{}).get('account',e['account_id'])!=e['account_id']:raise ValueError('account identity mismatch')
        self.verified[e['source_path']]=e['tick_receipt_sha256'];return x
    def load(self):
        for e in self.manifest['episodes']:
            if not e['eligible_for_research']:continue
            x=self.source(e);_,q,bars,_=market(x)
            self.merge_bars(e['account_id'],e['symbol'],bars,e['source_path'])
        for s in self.manifest['m30_segments']:
            if not s['eligible_for_research']:continue
            x=immutable_read(s['source_path'],s['receipt_sha256'],self.quarantine)
            self.verified[s['source_path']]=s['receipt_sha256']
            self.merge_bars(s['account_id'],s['symbol'],x['bars'],s['source_path'])
        return self
    def finish(self):
        for p,h in self.verified.items():
            if sha(p)!=h:raise ValueError('source changed during evaluation')

def native_metadata(manifest):
    items=[{'key':e['episode_id'],'symbol':e['symbol'],'class':e['asset_class'],'date':e['date']} for e in manifest['episodes'] if e['eligible_for_research']]
    return json.loads(subprocess.check_output([str(BIN/'replay_inventory_metadata')],input=json.dumps(items),text=True,timeout=120))

def freeze_code(plan_path,work):
    names=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',plan_path,
        ROOT/'ops/research/unified_replay.py',ROOT/'ops/research/test_unified_replay.py',ROOT/'ops/research/replay_inventory.py',
        ROOT/'crates/jiaoyi-core/examples/manifest_pullback_replay.rs',ROOT/'crates/jiaoyi-core/examples/native_target_sizing_replay.rs',ROOT/'crates/jiaoyi-core/examples/replay_inventory_metadata.rs']
    source={str(p.relative_to(ROOT)):sha(p) for p in names}
    for n,h in source.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+n],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit frozen source before evaluating: '+n)
    value={'source_hashes':source,'binary_hashes':{n:sha(BIN/n) for n in ['manifest_pullback_replay','native_target_sizing_replay','replay_inventory_metadata']},'plan_sha256':sha(plan_path),'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()}
    path=work/'code-freeze.json'
    if path.exists() and read(path)!=value:raise ValueError('run code/plan changed; use a separate registered run')
    if not path.exists():save(path,value)
    return value

def cached_native(binary,input_path,identity,work,freeze):
    key=canonical([binary,identity,sha(input_path),freeze])
    p=work/'native'/f'{key}.json';receipt=work/'native'/f'{key}.receipt.json'
    if p.exists() or receipt.exists():
        if not(p.exists() and receipt.exists()) or read(receipt)!={'cache_key':key,'output_sha256':sha(p)}:raise ValueError('incomplete/tampered native receipt')
        return read(p),True,key
    raw=subprocess.check_output([str(BIN/binary),str(input_path)],cwd=ROOT,text=True,timeout=120)
    value=json.loads(raw);p.parent.mkdir(exist_ok=True);tmp=p.with_suffix('.tmp');tmp.write_text(raw);tmp.replace(p)
    save(receipt,{'cache_key':key,'output_sha256':sha(p)})
    return value,False,key

def parity(plan,store,work,freeze):
    old_plan=read(ROOT/'docs/native_target_sizing_offline_plan_18353.json')
    old={r['key']:r for r in read(ROOT/'docs/native_target_sizing_offline_results_18353.json')['rows']}
    results=[];hits=0
    for j in old_plan['jobs']:
        if j['sha256'] in store.quarantine or j['sha256'] not in store.by_hash:raise ValueError('baseline source not eligible in manifest')
        store.source(store.by_hash[j['sha256']]);immutable_read(j['path'],j['sha256'],store.quarantine)
        v,hit,key=cached_native('native_target_sizing_replay',j['path'],j['sha256'],work,freeze);hits+=hit
        a=v['result']['arms']['arm_1_baseline_24bp_geometry'];b=old[j['key']]['arms']['arm_1_baseline_24bp_geometry']
        for field in ['marks','strict','forced']:
            if a[field]!=b[field]:raise ValueError('baseline parity failed: '+j['key']+' '+field)
        results.append({'key':j['key'],'file_sha256':j['sha256'],'native_receipt_key':key,'parity':True})
        if len(results)%40==0:print(f'PARITY {len(results)}/159',flush=True)
    if len(results)!=plan['baseline_parity']['required_episodes']:raise ValueError('baseline denominator changed')
    save(work/'parity.json',results);print('PARITY 159/159 exact',flush=True)
    return results,hits

def qualified_input(e,store,meta,peer,join):
    m=meta[e['episode_id']];s=m['session'];reasons=[]
    if not s:return None,{'status':'no_coded_session','missing':[]}
    start=ms(s['open']);end=start+4*3600000
    bars=store.bars[(e['account_id'],e['symbol'])];required=slots(m['prior_45_calendar_days']);missing=[t for t in required if t not in bars]
    if not required or missing:reasons.append('missing_45day_session_preparation')
    if not permitted(start,end+5001):reasons.append('protected_window')
    if ms(e['source_window_utc']['from'])>start or ms(e['source_window_utc']['to'])<end:reasons.append('tick_interval_does_not_cover_native_four_hours')
    if start not in bars or bars[start]['volume']<=0:reasons.append('missing_opening_M30')
    coverage={'status':'qualified_recorded_scenario' if not reasons else 'excluded','reasons':reasons,'required_prep_slots':len(required),'missing_prep_slots':len(missing),'first_missing_utc':[stamp(t) for t in missing[:8]],'window':{'from':stamp(start),'to':stamp(end)},'historical_calendar_verified':False,'continuous_executable_path_verified':False}
    if reasons:return None,coverage
    x=store.source(e);_,q,_,_=market(x)
    source={'contract':q['contract'],'bids':[t for t in q['bids'] if start<=t['ts_ms']<=end+5000],
            'asks':[t for t in q['asks'] if start<=t['ts_ms']<=end+5000]}
    points=checkpoints(source['bids'],source['asks'],{'open':start,'end':end},source['contract']['digits'])
    coverage['opening_quote']=points['open'];coverage['closing_tail_quote']=points['end']
    if points['open'] is None:reasons.append('missing_fresh_open_quote')
    if any(not source[side] or source[side][-1]['ts_ms']<end-6000 for side in ['bids','asks']):reasons.append('missing_tick_coverage_at_boundary')
    if reasons:coverage['status']='excluded';return None,coverage
    prior=[b for t,b in sorted(bars.items()) if start-45*86400000<=t and t+1800000<=start]
    source['bars']=[{'at':stamp(b['ts_ms']),'open':b['open_e5']/100000,'close':b['close_e5']/100000} for b in prior+[bars[start]]]
    own=causal_context(m['prior_45_calendar_days'],bars,start)
    peer_ctx=None
    if peer:
        pm=meta[peer['episode_id']]
        peer_ctx=causal_context(pm['prior_45_calendar_days'],store.bars[(peer['account_id'],peer['symbol'])],start)
    future=[]
    for t,b in sorted(bars.items()):
        if t<end:continue
        if t>=end+72*3600000 or not permitted(t,t+1800000):break
        future.append(b)
    coverage.update(continuation_bars_available=len(future),continuation_starts_at_boundary=bool(future and future[0]['ts_ms']==end),peer_episode_id=peer['episode_id'] if peer else None)
    request={'account':e['account_id'],'symbol':e['symbol'],'from':stamp(start),'to':stamp(end),'spread':0.,'commission_round_trip':0.,'slippage_bps_per_side':24.,'tick_replay':{'volume':e['native_volume']}}
    return {'request':request,'source':source,'future':future,'own_context':own,'peer_context':peer_ctx,'peer_available':peer is not None,'demo_entry_bps':join.median_entry_slippage_bps},coverage

def score(rows,arm,mode,cost):
    states=collections.Counter();values=[];gross=[]
    for row in rows:
        a=row['native'].get('arms',{}).get(arm,{})
        o=a.get(mode)
        if not o:states['no_entry']+=1;continue
        metrics=o.get('metrics')
        if not metrics:states['unresolved']+=1;continue
        if cost in BPS:v=metrics['net_by_bps'][cost]
        else:
            benchmark=metrics.get('demo_entry_benchmark')
            v=benchmark.get(cost) if benchmark else None
        if v is None:states['cost_unavailable']+=1;continue
        values.append(v);gross.append(metrics['gross_r'])
        states['wins' if v>1e-12 else 'losses' if v< -1e-12 else 'net_flat']+=1
    gain=sum(v for v in values if v>0);loss=-sum(v for v in values if v<0)
    return {'episodes':len(rows),'entries':len(rows)-states['no_entry'],'priced_closed':len(values),
        **{k:states[k] for k in ['wins','losses','net_flat','no_entry','unresolved','cost_unavailable']},
        'net_win_rate':states['wins']/len(values) if values else None,'mean_net_r':statistics.mean(values) if values else None,
        'mean_gross_r':statistics.mean(gross) if gross else None,'profit_factor':gain/loss if loss else None,
        'profit_factor_status':'finite' if loss else 'no_losses' if gain else 'unavailable',
        'financial_pass':False}

def check_arithmetic(native):
    from option_a_vs_b_offline import accounting_check
    count=0
    for arm in native.get('arms',{}).values():
        count+=accounting_check({'models':{k:arm[k] for k in MODES if arm.get(k)}})
        for mode in MODES:
            v=arm.get(mode)
            if not v or not v.get('metrics'):continue
            i=v['entry'];p=v['exit']['price'] if v['status']=='closed_m30_approximation' else v['outcome']['fills'][0]['price']
            if i['entered_at']>=i['session_end']:raise ValueError('invalid entry deadline')
            if v['metrics'].get('demo_entry_benchmark'):
                vals=v['metrics']['demo_entry_benchmark'];delta=vals['demo_entry_plus_2_exit']-vals['demo_entry_plus_3.74_exit']
                if abs(delta-p*1.74/10000/i['initial_r'])>1e-9:raise ValueError('benchmark exit accounting mismatch')
                count+=1
    return count

def run(plan_path,output,work):
    sys.addaudithook(deny_network);work.mkdir(parents=True,exist_ok=True)
    plan=read(plan_path);verify_inputs(plan);freeze=freeze_code(plan_path,work)
    manifest=read(ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json')
    inventory=read(ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json')
    store=ManifestStore(manifest).load();meta=native_metadata(manifest);joins=cost_joins(inventory,manifest)
    parity_rows,parity_hits=parity(plan,store,work,freeze)
    prior_hashes={r['file_sha256'] for r in parity_rows}
    # All choices are based on coverage and provenance, before any candidate outcomes.
    peers={a:b for pair in plan['peer_pairs'] for a,b in [pair,list(reversed(pair))]}
    eligible=[e for e in manifest['episodes'] if e['eligible_for_research']]
    peer_by_day=collections.defaultdict(list)
    for e in eligible:peer_by_day[(e['account_id'],e['symbol'],e['date'])].append(e)
    selections=[];excluded=[];duplicates=[]
    for e in eligible:
        if e['symbol'] not in plan['symbols']:
            excluded.append({'episode_id':e['episode_id'],'symbol':e['symbol'],'date':e['date'],'status':'outside_core_scope'});continue
        options=peer_by_day.get((e['account_id'],peers[e['symbol']],e['date']),[])
        peer=sorted(options,key=lambda q:q['episode_id'])[0] if options else None
        inp,coverage=qualified_input(e,store,meta,peer,joins[CostKey(e['account_id'],e['symbol'])])
        record={k:e[k] for k in ['episode_id','account_id','symbol','date','cohort','source_path','tick_receipt_sha256','protocol_status']}
        record.update(coverage=coverage,prior159_member=e['tick_receipt_sha256'] in prior_hashes)
        if inp is None:excluded.append(record);continue
        selections.append((e,inp,record))
    selected={}
    for e,inp,record in sorted(selections,key=lambda x:(-(ms(x[0]['source_window_utc']['to'])-ms(x[0]['source_window_utc']['from'])),x[0]['tick_receipt_sha256'])):
        key=(e['account_id'],e['symbol'],e['date'])
        if key in selected:
            duplicates.append({'episode_id':e['episode_id'],'selected_episode_id':selected[key][0]['episode_id'],'reason':'same_account_symbol_session'});continue
        selected[key]=(e,inp,record)
    print('QUALIFIED '+str(len(selected))+' unique sessions',flush=True)
    rows=[];checks=0;hits=0
    for key,(e,inp,row) in sorted(selected.items()):
        path=work/('input-'+e['episode_id']+'.json');save(path,inp)
        native,hit,receipt=cached_native('manifest_pullback_replay',path,e['episode_id'],work,freeze);hits+=hit
        checks+=check_arithmetic(native)
        row.update(native=native,input_sha256=sha(path),native_receipt_key=receipt,
            own_context=inp['own_context'],peer_context=inp['peer_context'],
            cost_join_key=asdict(joins[CostKey(e['account_id'],e['symbol'])].key),
            native_volume=e['native_volume'])
        rows.append(row)
        if len(rows)%10==0:print(f'EXPERIMENT {len(rows)}/{len(selected)}',flush=True)
    groups={'full_qualified_core':rows,'prior159_qualified_subset':[r for r in rows if r['prior159_member']],
        'clean_receipt_history':[r for r in rows if r['protocol_status']=='clean_first_read']}
    for symbol in plan['symbols']:groups[symbol]=[r for r in rows if r['symbol']==symbol]
    costs=BPS+['demo_entry_plus_2_exit','demo_entry_plus_3.74_exit']
    summary={g:{a:{mode:{cost:score(rs,a,mode,cost) for cost in costs} for mode in MODES} for a in ARMS} for g,rs in groups.items()}
    continuation=collections.Counter()
    for r in rows:
        for a,v in r['native'].get('arms',{}).items():
            if v.get('strict') and not v['strict'].get('metrics'):continuation[(a,v['continuation']['status'])]+=1
    store.finish();verify_inputs(plan)
    result={'schema':'jiaoyi.unified-offline-results.v1','trace':plan['trace'],'plan_sha256':sha(plan_path),'code_freeze':freeze,
        'rows':rows,'excluded':excluded,'same_session_aliases':duplicates,'quarantine_receipt_hashes':sorted(store.quarantine),
        'parity':{'episodes':len(parity_rows),'exact':True,'rows':parity_rows},'cost_evidence_joins':[asdict(v) for _,v in sorted(joins.items(),key=lambda kv:(kv[0].account_id,kv[0].symbol))],
        'summary':summary,'continuation_from_censored_ticks':[{'arm':a,'status':s,'count':n} for (a,s),n in sorted(continuation.items())],
        'coverage':{'manifest_episode_definitions':len(manifest['episodes']),'eligible_definitions':len(eligible),
            'quarantined_receipts':len(store.quarantine),'m30_source_files':len(manifest['m30_segments']),
            'core_eligible_definitions':sum(e['symbol'] in plan['symbols'] for e in eligible),'qualified_unique_core_sessions':len(rows),
            'excluded_definitions':len(excluded),'duplicate_qualified_definitions':len(duplicates),'verified_local_files':len(store.verified)},
        'verification':{'baseline_parity_episodes':len(parity_rows),'independent_cost_checks':checks,
            'source_hashes_reverified':True,'all_quarantined_receipts_rejected':len(store.quarantine),'no_network_guard':True},
        'new_broker_requests':0,'validation_opened':False,'financial_pass':False,'live_rules_changed':False,
        'goal':plan['goal'],'evidence_scope':'Reused development data, selected macro weeks; partial-cost scenarios and unverified historical calendars. No independent holdout or per-account qualification.'}
    save(output,result)
    runtime={'parity_cache_hits':parity_hits,'experiment_cache_hits':hits,'result_sha256':sha(output)}
    save(work/'last-run.json',runtime)
    print(json.dumps({'coverage':result['coverage'],'summary':summary['full_qualified_core'],'runtime':runtime},indent=2),flush=True)
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--plan',type=Path,default=PLAN);p.add_argument('--output',type=Path,default=RESULT);p.add_argument('--workdir',type=Path,default=DEFAULT_WORK)
    a=p.parse_args();run(a.plan,a.output,a.workdir)
