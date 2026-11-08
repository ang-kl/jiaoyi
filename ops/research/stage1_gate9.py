"""Owner №18,380: one frozen Gate9 cohort, preparation before every tick request.

Old collectors remain paused. This capability is local, exact-plan-bound and
never permits orders, protected dates, retuning, retries or broader collection.
"""
import argparse
import collections
import copy
import datetime as dt
import fcntl
import json
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace
from replay_inventory import ROOT, CORE, read, save, sha, canonical, ms, stamp, market, permitted
from rebuild_preflight import require, verify_plan, Interval, Refusal
from clean_reservation import merged
from rebuild_reservation import partition_slots
from rebuild import REGISTRY
import phase5b_audit as frozen
import unified_replay as u
from seasonal_pilot import bar_request, decode_bars
from class_batch import receipt

PLAN=ROOT/'docs/gate9_stage1_plan_18380.json'
RESULT=ROOT/'docs/gate9_stage1_results_18380.json'
SOURCE=ROOT/'docs/rebuild_rfc_260deals_and_clean_reservation_18372.json'
WORK=Path('/workspace/jiaoyi-monday-18380/stage1')
ACCOUNT=49236051
CODE=['ops/research/stage1_gate9.py','ops/research/broker_readonly.py','ops/research/rebuild_preflight.py','ops/research/rebuild_reservation.py','ops/research/seasonal_pilot.py']

def now():return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()

def expand(job):
    j=copy.deepcopy(job)
    j['required_m30_slots']=[t for r in j.pop('required_m30_slot_runs') for t in range(r['from_ms'],r['to_ms'],1800000)]
    require(canonical(j['required_m30_slots'])==j['required_m30_slots_sha256'],'reserved_slots_changed')
    return j

def select(res):
    choices=[]
    for w in res['selected_weeks']:
        monday=dt.date.fromisoformat(w['monday']);dates=[str(monday+dt.timedelta(days=i)) for i in range(5)]
        jobs=[expand(j) for j in res['episodes'] if j['date'] in dates]
        cache={(s['source_path'],s['sha256']) for j in jobs for s in j['preparation_segments'] if s['mode']=='verified_cache'}
        choices.append({'monday':w['monday'],'dates':dates,'verified_cache_files':len(cache),'cached_unique_slots':w['cached_unique_slots'],'jobs':jobs})
    return sorted(choices,key=lambda x:(-x['verified_cache_files'],x['monday']))[0], [{k:v for k,v in x.items() if k!='jobs'} for x in choices]

def load_cache(p,done=None):
    bars=collections.defaultdict(dict);refs=collections.defaultdict(lambda:collections.defaultdict(set));verified={}
    sources={s['source_path']:s['sha256'] for j in p['episodes'] for s in j['preparation_segments'] if s['mode']=='verified_cache'}
    for key,e in (done or {}).items():
        if key.startswith('bar-') and e['status']=='complete':sources[str(WORK/(key+'.json'))]=e['sha256']
    for path,h in sources.items():
        require(sha(path)==h,'source_hash_mismatch');raw=read(path);data=market(raw)[2]
        req=raw.get('request',{});sym=req.get('symbol')
        if not sym:
            sym=next(j['symbol'] for j in p['episodes'] if any(s.get('source_path')==path for s in j['preparation_segments']))
        key=(ACCOUNT,sym);verified[path]=h
        for bar in data:
            t=bar['ts_ms'];require(permitted(t,t+1800000),'protected_bar')
            require(t not in bars[key] or bars[key][t]==bar,'conflicting_bar')
            bars[key][t]=bar;refs[key][t].add(path)
    return bars,refs,verified

def registry(p):
    old=read(REGISTRY);r=dict(old,**p['registry'])
    return r

def preparation_proof(p,done,stage):
    bars,refs,verified=load_cache(p,done);meta=frozen.metadata({'episodes':p['episodes']});r=registry(p)
    jobs=copy.deepcopy(p['episodes'])
    if stage!='preparation':
        for j in jobs:
            key=(ACCOUNT,j['symbol'])
            j['preparation_segments']=partition_slots(j['required_m30_slots'],bars[key],refs[key],verified,
                [Interval(**x) for x in r['spent_intervals']],*key)
    proof=verify_plan({'identity':p['identity'],'scheduled_episodes':40,'episodes':jobs},r,meta,bars,refs,verified,stage=stage)
    return proof,jobs,bars,refs,verified,meta

def freeze():
    if PLAN.exists():return check_freeze()
    WORK.mkdir(parents=True,exist_ok=True)
    reservation=read(SOURCE)['reservation'];chosen,ranking=select(reservation);jobs=chosen.pop('jobs')
    require(len(jobs)==40 and len({j['episode_id'] for j in jobs})==40,'denominator')
    metadata=read(ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json')['metadata']
    future=collections.defaultdict(list)
    for j in jobs:
        m=metadata[j['symbol']];j.update(symbol_id=m['symbol_id'],native_volume=m['min_volume'],contract={k:m[k] for k in ['digits','min_volume','step_volume']})
        for s in j['preparation_segments']:
            if s['mode']=='unspent':future[j['symbol']].append((s['from_ms'],s['to_ms']))
    parts=[]
    for sym,ranges in sorted(future.items()):
        for a,b in merged(ranges):
            for start in range(a,b,7*86400000):
                parts.append({'key':f'prep-{sym}-{start}','symbol':sym,'symbol_id':metadata[sym]['symbol_id'],'from':stamp(start),'to':stamp(min(b,start+7*86400000)),'role':'causal_preparation'})
    # The frozen native adapter requires the opening M30 open field. Opening
    # references are separately declared; their close is never a prior feature.
    provisional={'episodes':jobs};bars,_,_=load_cache(provisional)
    for j in jobs:
        start=ms(j['from']);end=start+1800000
        if start in bars[(ACCOUNT,j['symbol'])] or any(s['symbol']==j['symbol'] and ms(s['from'])<=start<end<=ms(s['to']) for s in parts):continue
        require(not any(Interval(j['symbol'],ACCOUNT,start,end,'m30').overlaps(Interval(**x)) for x in read(REGISTRY)['spent_intervals']),'opening_reference_already_spent')
        parts.append({'key':f'opening-{j["episode_id"]}','symbol':j['symbol'],'symbol_id':j['symbol_id'],'from':stamp(start),'to':stamp(end),'role':'opening_reference_only'})
    previous=reservation['registry']['identity'];identity=dict(previous,rfc_state='stage1-bounded-collection-18380-v1')
    transition={'from_identity':previous,'to_identity':identity,'owner_trace':'№18,380 Parts A/B; only40 reserved episodes','reviewed_commit':subprocess.check_output(['git','rev-parse','origin/main'],cwd=ROOT,text=True).strip(),'change':'One maximum-cache cohort; scoped read capability, preparation barrier before ticks; rules unchanged'}
    reg=dict(reservation['registry'],identity=identity,transition=transition,transitions=reservation['registry']['transitions']+[transition],collection_enabled=True,frozen_independent_jobs_sha256=canonical(jobs))
    p={'schema':'jiaoyi.gate9-stage1.v1','trace':'№18,380','frozen_at_sgt':now(),'selected_week':chosen,'selection_ranking':ranking,
       'identity':identity,'transition':transition,'registry':reg,'scheduled_episodes':40,'episodes':jobs,'bar_segments':parts,
       'reservation_sha256':sha(SOURCE),'frozen_engine':frozen.frozen_identity(),'admission_bps':3.74,'stress_repricing_bps':24.0,
       'maximum_trials':1,'unchanged_candidate':'Exact archived Phase4; distinct from owner18377/18380 production policy',
       'opening_reference_policy':'Opening M30 open only; no opening close/high/low/volume as causal pre-open features',
       'cost_calibration':'3.74bp is assumed sensitivity, not verified historical round-trip cost',
       'collection_limits':{'attempts':1,'tick_episodes':40,'m30_requests':len(parts),'total_bytes':512000000,'per_file_bytes':32000000},
       'stopping':'No adaptive stopping/resampling; any incomplete preparation refuses all40 ticks; failures remain spent.',
       'validation_opened':False,'financial_pass':False}
    p['preflight_preparation']=preparation_proof(p,{},'preparation')[0]
    save(PLAN,p)
    save(WORK/'freeze.json',{'plan_sha256':sha(PLAN),'source_sha256':{n:sha(ROOT/n) for n in CODE},'frozen_engine':p['frozen_engine'],'created_at_sgt':now()})
    return p

def check_freeze():
    p=read(PLAN);f=read(WORK/'freeze.json')
    require(sha(PLAN)==f['plan_sha256'] and sha(SOURCE)==p['reservation_sha256'],'frozen_plan_changed')
    for n,h in f['source_sha256'].items():require(sha(ROOT/n)==h,'frozen_collector_changed:'+n)
    require(frozen.frozen_identity()==f['frozen_engine'],'frozen_engine_changed')
    return p

class Stage1Authority:
    def __init__(self):
        self.plan=check_freeze();self.ticks_ready=False;preparation_proof(self.plan,{},'preparation')
    def enable_ticks(self,done):
        proof,*_=preparation_proof(self.plan,done,'ticks')
        require(len(proof['episodes'])==40,'partial_preparation_not_enough')
        save(WORK/'ticks-preflight.json',proof);self.ticks_ready=True
    def authorize(self,kind=None,payload=None):
        require(sha(PLAN)==read(WORK/'freeze.json')['plan_sha256'],'capability_plan_drift')
        if kind is None:return
        payload=payload or {};require(payload.get('ctidTraderAccountId',ACCOUNT)==ACCOUNT,'capability_account')
        if kind in (2100,2102):return
        ids={j['symbol_id'] for j in self.plan['episodes']}
        if kind==2114:return
        if kind==2116:
            require(set(payload['symbolId'])<=ids,'capability_metadata_scope');return
        if kind==2137:
            require('count' not in payload and payload.get('period')==8,'interval_only_required')
            require(any(payload==bar_request(s) for s in self.plan['bar_segments']),'unreserved_m30_request');return
        if kind==2145:
            require(self.ticks_ready,'preparation_barrier_before_ticks')
            require(payload.get('type') in (1,2) and any(j['symbol_id']==payload.get('symbolId') and ms(j['from'])<=payload.get('fromTimestamp',-1)<=payload.get('toTimestamp',-1)<=ms(j['to'])+5000 for j in self.plan['episodes']),'unreserved_tick_request');return
        raise Refusal('research_message_forbidden')

def ledger():
    path=WORK/'ledger.jsonl';events=[json.loads(x) for x in path.read_text().splitlines()] if path.exists() else []
    starts=[x['key'] for x in events if x['state']=='started'];done={x['key']:x for x in events if x['state']=='finished'}
    require(len(starts)==len(set(starts)) and set(starts)==set(done),'uncertain_or_duplicate_receipt_no_retry')
    for key,e in done.items():require(sha(WORK/(key+'.json'))==e['sha256'],'receipt_changed')
    return done

def collect():
    from broker_readonly import Broker,BrokerError,ticks
    p=check_freeze();done=ledger();auth=Stage1Authority()
    def perform(key,req,operation):
        if key in done:return
        path=WORK/(key+'.json');require(not path.exists(),'unreceipted_source')
        receipt(WORK/'ledger.jsonl',{'key':key,'state':'started','at_sgt':now(),'plan_sha256':sha(PLAN),'request':req})
        budget={'pages':0,'rows':0}
        try:v=operation(budget)
        except Exception as e:v={'status':'failed','error':str(e) if isinstance(e,BrokerError) else type(e).__name__}
        v.update(request=req,budget=budget,attempts=1);raw=json.dumps(v,separators=(',',':'),allow_nan=False)+'\n'
        require(len(raw)<p['collection_limits']['per_file_bytes'],'source_byte_budget');path.write_text(raw)
        e={'key':key,'state':'finished','sha256':sha(path),'status':v['status'],'at_sgt':now()};receipt(WORK/'ledger.jsonl',e);done[key]=e
        print(json.dumps({'completed':len(done),'key':key,'status':v['status'],'error':v.get('error')}),flush=True)
        require(sum((WORK/(k+'.json')).stat().st_size for k in done)<=p['collection_limits']['total_bytes'],'batch_byte_budget')
        if v.get('error') in ['CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED','INVALID_ACCESS_TOKEN','OA_AUTH_TOKEN_EXPIRED','TypeError','KeyError','NameError','AttributeError']:raise Refusal('system_or_auth_failure_no_retry')
    with (WORK/'collection.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        with Broker(authority=auth) as broker:
            for part in p['bar_segments']:
                def fetch(budget,part=part):
                    budget['pages']+=1;raw=broker.request(2137,bar_request(part))
                    try:return {'status':'complete','bars':decode_bars(raw,part)}
                    except BrokerError as e:return {'status':'failed','error':str(e),'raw_response':raw}
                perform('bar-'+part['key'],part,fetch)
            try:auth.enable_ticks(done)
            except Refusal as e:
                save(WORK/'barrier-refusal.json',{'reason':str(e),'at_sgt':now(),'tick_requests':0,'preparation_audit':coverage(p,done)})
                print('ALL TICKS REFUSED: '+str(e),flush=True);return
            for j in p['episodes']:
                def fetch(budget,j=j):
                    end=ms(j['to'])+5000;deadline=time.monotonic()+240
                    bids=ticks(broker,j['symbol_id'],1,ms(j['from']),end,budget,deadline);asks=ticks(broker,j['symbol_id'],2,ms(j['from']),end,budget,deadline)
                    return {'status':'complete','payload':{'quotes':{'bars':[],'bids':bids,'asks':asks,'contract':j['contract']},'m30':[]}}
                perform('job-'+j['episode_id'],dict(account=ACCOUNT,symbol=j['symbol'],**{'from':j['from'],'to':j['to']}),fetch)
            save(WORK/'collection-complete.json',{'at_sgt':now(),'messages':broker.counter})

def coverage(p,done):
    bars,refs,verified=load_cache(p,done);rows=[]
    for j in p['episodes']:
        missing=[t for t in j['required_m30_slots'] if t not in bars[(ACCOUNT,j['symbol'])]]
        rows.append({'episode_id':j['episode_id'],'symbol':j['symbol'],'date':j['date'],'required_slots':len(j['required_m30_slots']),'missing_slots':len(missing),'first_missing':[stamp(t) for t in missing[:12]],'opening_reference_present':ms(j['from']) in bars[(ACCOUNT,j['symbol'])]})
    return rows

def evaluate():
    from rebuild_preflight import offline_process
    offline_process();p=check_freeze();done=ledger();cov=coverage(p,done);rows=[];excluded=[]
    bar_proof=None
    try:bar_proof,jobs,bars,refs,verified,meta=preparation_proof(p,done,'ticks')
    except Refusal as e:barrier=str(e)
    else:
        barrier=None;u.BIN=frozen.WORK/'bin'
        store=SimpleNamespace(bars=bars,source=lambda e:read(e['source_path']))
        for j in jobs:
            key='job-'+j['episode_id'];rec=done.get(key)
            if not rec or rec['status']!='complete':excluded.append({'episode_id':j['episode_id'],'reason':'missing_or_failed_tick_source'});continue
            e=dict(j,source_path=str(WORK/(key+'.json')),source_window_utc={'from':j['from'],'to':j['to']})
            peer=next(x for x in jobs if x['date']==j['date'] and x['symbol']==frozen.PEERS[j['symbol']])
            inp,qcov=u.qualified_input(e,store,meta,peer,SimpleNamespace(median_entry_slippage_bps=None))
            if inp is None:excluded.append({'episode_id':j['episode_id'],'reason':'native_input_incomplete','coverage':qcov});continue
            path=WORK/('input-'+j['episode_id']+'.json');save(path,{'input':inp,'policy':{'tier':'tier1','threshold_bps':48.0,'admission_bps':3.74}})
            n,_,key=u.cached_native('tiered_pullback_replay',path,['stage1',j['episode_id']],WORK,read(WORK/'freeze.json'));u.check_arithmetic(n)
            rows.append({'episode_id':j['episode_id'],'symbol':j['symbol'],'date':j['date'],'coverage':qcov,'native_receipt_key':key,'input_sha256':sha(path),'native':n})
    arm='arm_c_macro_filtered_pullback';scores={mode:{cost:u.score(rows,arm,mode,cost) for cost in ['3.74','24']} for mode in u.MODES}
    out={'trace':'№18,380','schema':'jiaoyi.gate9-stage1-results.v1','plan_sha256':sha(PLAN),'frozen_engine':p['frozen_engine'],'selected_week':p['selected_week'],
         'episodes_scheduled':40,'preparation_complete_episodes':sum(x['missing_slots']==0 for x in cov),'preparation_coverage':cov,'barrier_refusal':barrier,
         'tier1_native_episodes_completed':len(rows),'failed_sources':sum(e['status']!='complete' for e in done.values()),'reused_m30_segments':p['selected_week']['verified_cache_files'],
         'new_m30_segments_fetched':sum(k.startswith('bar-') for k in done),'new_tick_episodes_fetched':sum(k.startswith('job-') for k in done),
         'opening_reference_m30_requests':sum(s['role']=='opening_reference_only' for s in p['bar_segments']),
         'receipt_hashes':{k:v['sha256'] for k,v in done.items()},'rows':rows,'excluded':excluded,'candidate_scores':scores,
         'matched_arm_scores':{a:{mode:{cost:u.score(rows,a,mode,cost) for cost in ['3.74','24']} for mode in u.MODES} for a in u.ARMS},
         'financial_pass':False,'validation_opened':False,'source_scope':'Frozen Phase4 candidate; not current production owner18380. Oneweek cannot qualify multiweek financial gates.',
         'preflight_ticks_proof':bar_proof,'generated_at_sgt':now()}
    save(RESULT,out)
    summary={k:v for k,v in out.items() if k not in ['rows','matched_arm_scores','preparation_coverage','frozen_engine']}
    summary['frozen_engine_commit']=p['frozen_engine']['commit'];summary['full_result_file']=str(RESULT)
    save(WORK.parent/'stage1-summary.json',summary);print(json.dumps({k:out[k] for k in ['selected_week','preparation_complete_episodes','barrier_refusal','tier1_native_episodes_completed','failed_sources','new_m30_segments_fetched','new_tick_episodes_fetched','candidate_scores']},indent=2))

if __name__=='__main__':
    sys.modules.setdefault('stage1_gate9',sys.modules[__name__])
    a=argparse.ArgumentParser();a.add_argument('action',choices=['freeze','collect','evaluate']);args=a.parse_args()
    if args.action=='freeze':print(json.dumps(freeze()['selected_week']))
    elif args.action=='collect':collect()
    else:evaluate()
