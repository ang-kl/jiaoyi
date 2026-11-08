"""Owner №18,382: frozen 30/10 partition of the already prepared Stage1 cohort.

No production import, deployment route, order route or new M30 request exists.
"""
import argparse
import collections
import copy
import fcntl
import json
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace
import stage1_gate9 as prior
from replay_inventory import ROOT, read, save, sha, canonical, ms, stamp, checkpoints, market
from rebuild_preflight import require, verify_plan, Interval, Refusal, offline_process
from rebuild_reservation import partition_slots
from class_batch import receipt
import phase5b_audit as frozen
import unified_replay as u

WORK=Path('/workspace/jiaoyi-stage1-18382')
PLAN=ROOT/'docs/gate10_stage1_2024_11_11_plan_18382.json'
RESULT=ROOT/'docs/gate10_stage1_2024_11_11_results_18382.json'
ACCOUNT=prior.ACCOUNT
CODE=['ops/research/stage1_completion.py','ops/research/stage1_gate9.py','ops/research/broker_readonly.py','ops/research/rebuild_preflight.py','ops/research/unified_replay.py','ops/research/stage1_controls.py']

def inputs():
    p=prior.check_freeze();done=prior.ledger();bars,refs,verified=prior.load_cache(p,done)
    require(len(verified)==141 and len(done)==112 and all(v['status']=='complete' for v in done.values()),'prior_source_denominator')
    require(all(k.startswith('bar-') for k in done),'prior_ticks_already_spent')
    return p,done,bars,refs,verified

def partition(p,bars):
    eligible=[];blocked=[];gap_proofs={}
    for job in p['episodes']:
        missing=[t for t in job['required_m30_slots'] if t not in bars[(ACCOUNT,job['symbol'])]]
        if not missing:eligible.append(copy.deepcopy(job));continue
        proof=[]
        for t in missing:
            covering=[s['key'] for s in p['bar_segments'] if s['symbol']==job['symbol'] and ms(s['from'])<=t and t+1800000<=ms(s['to'])]
            require(covering,'unspent_crypto_gap_requires_new_plan')
            proof.append({'from_ms':t,'completed_prior_requests':covering})
        gap_proofs.setdefault(job['symbol'],{}).update({str(x['from_ms']):x for x in proof})
        blocked.append({'episode_id':job['episode_id'],'symbol':job['symbol'],'date':job['date'],'reason':'missing_bars_inside_successfully_fetched_intervals','missing_m30_slots':len(missing)})
    require(len(eligible)==30 and len(blocked)==10,'partition_denominator_changed')
    require({j['symbol'] for j in blocked}=={'BTCUSD','ETHUSD'},'unexpected_blocked_symbol')
    return eligible,blocked,gap_proofs

def prep(p,stage='ticks',evaluation_jobs=None):
    old,done,bars,refs,verified=inputs();jobs=copy.deepcopy(evaluation_jobs or p['episodes']);meta=frozen.metadata({'episodes':jobs})
    for j in jobs:
        key=(ACCOUNT,j['symbol'])
        j['preparation_segments']=partition_slots(j['required_m30_slots'],bars[key],refs[key],verified,[Interval(**x) for x in p['registry']['spent_intervals']],*key)
    proof=verify_plan({'identity':p['identity'],'scheduled_episodes':len(jobs),'episodes':jobs},p['registry'],meta,bars,refs,verified,stage=stage)
    return proof,jobs,bars,refs,verified,meta

def freeze():
    if PLAN.exists():return check_freeze()
    import stage1_controls as controls
    WORK.mkdir(parents=True,exist_ok=True)
    old,done,bars,refs,verified=inputs();jobs,blocked,gaps=partition(old,bars)
    previous=old['identity'];identity=dict(previous,rfc_state='stage1-traditional-partition-18382-v1')
    transition={'from_identity':previous,'to_identity':identity,'owner_trace':'№18,382 §1 steps2A/2B','reviewed_commit':subprocess.check_output(['git','rev-parse','origin/main'],cwd=ROOT,text=True).strip(),'change':'Partition the40scheduled denominator by preparation availability beforeticks;30unchangedtraditionalepisodes and10explicitblockedcrypto; no retuning/resampling'}
    registry=dict(prior.registry(old),identity=identity,transition=transition,transitions=old['registry']['transitions']+[transition],frozen_independent_jobs_sha256=canonical(jobs))
    p={'schema':'jiaoyi.stage1-completion.v1','trace':'№18,382','frozen_at_sgt':prior.now(),'identity':identity,'registry':registry,'transition':transition,'episodes_scheduled':40,'collection_episode_count':30,'episodes':jobs,'blocked_episodes':blocked,
       'crypto_diagnosis':{'new_unspent_missing_m30_intervals':0,'all_missing_slots_already_requested':True,'missing_slot_receipts':gaps,'conclusion':'Bothcrypto96distinctmissing slots are inside successful112-request priorcollection; unavailable in retainedbrokerhistory,not uncoveredintervals. Historicalclosurecauseunverified; owner18382authorises30/10partition.'},
       'source_plan_sha256':sha(prior.PLAN),'source_result_sha256':sha(prior.RESULT),'reused_source_hashes':verified,'source_ledger_sha256':sha(prior.WORK/'ledger.jsonl'),
       'frozen_engine':frozen.frozen_identity(),'controls':controls.identity(),'admission_bps_per_side':3.74,'stress_repricing_bps_per_side':24.0,'threshold_bps':48.0,'tier':'tier1',
       'coverage_policy':'Full-path Gate3 proof remains mandatory forcertifiedmetrics. Prepandfreshboundary-completefiles may additionally produce explicitlylabelledrecorded-pathdiagnostics; gaps/ambiguity nevercertifyexecutionorfinancialpass.',
       'maximum_trials':1,'collection_limits':{'m30_requests':0,'tick_episodes':30,'attempts':1,'total_bytes':512000000,'per_file_bytes':32000000,'pages_per_episode':120,'rows_per_episode':200000,'seconds_per_episode':240},
       'live_production_untouched':True,'validation_opened':False,'financial_pass':False}
    p['preflight_before_ticks']=prep(p)[0];save(PLAN,p)
    save(WORK/'freeze.json',{'plan_sha256':sha(PLAN),'source_sha256':{name:sha(ROOT/name) for name in CODE},'frozen_engine':p['frozen_engine'],'controls':p['controls'],'at_sgt':prior.now()})
    return p

def check_freeze():
    import stage1_controls as controls
    p=read(PLAN);f=read(WORK/'freeze.json');require(sha(PLAN)==f['plan_sha256'],'plan_changed')
    for path,h in f['source_sha256'].items():require(sha(ROOT/path)==h,'code_changed:'+path)
    require(sha(prior.PLAN)==p['source_plan_sha256'] and sha(prior.RESULT)==p['source_result_sha256'] and sha(prior.WORK/'ledger.jsonl')==p['source_ledger_sha256'],'prior_artifact_changed')
    require(frozen.frozen_identity()==p['frozen_engine'] and controls.identity()==p['controls'],'native_identity_changed')
    return p

class CompletionAuthority(prior.Stage1Authority):
    def __init__(self):
        self.plan=check_freeze();self.current_job=None;self.proof=prep(self.plan)[0]
        require(len(self.proof['episodes'])==30,'partial_preparation_not_enough')
    def authorize(self,kind=None,payload=None):
        require(sha(PLAN)==read(WORK/'freeze.json')['plan_sha256'],'capability_plan_drift')
        if kind is None:return
        payload=payload or {};require(payload.get('ctidTraderAccountId',ACCOUNT)==ACCOUNT,'capability_account')
        if kind in (2100,2102):return
        require(kind==2145,'only_reserved_ticks_authorised')
        require(self.current_job is not None,'no_active_receipted_episode')
        job=self.current_job
        require(job in self.plan['episodes'],'unreserved_episode')
        events=[json.loads(x) for x in (WORK/'ledger.jsonl').read_text().splitlines()]
        require(events[-1]['state']=='started' and events[-1]['key']=='job-'+job['episode_id'],'episode_not_currently_started')
        require(payload.get('type') in (1,2) and payload.get('symbolId')==job['symbol_id'] and ms(job['from'])<=payload.get('fromTimestamp',-1)<=payload.get('toTimestamp',-1)<=ms(job['to'])+5000,'unreserved_tick_interval')

def ledger():
    path=WORK/'ledger.jsonl';events=[json.loads(x) for x in path.read_text().splitlines()] if path.exists() else []
    starts=[e['key'] for e in events if e['state']=='started'];done={e['key']:e for e in events if e['state']=='finished'}
    require(len(starts)==len(set(starts)) and set(starts)==set(done),'uncertain_or_duplicate_receipt_no_retry')
    for key,e in done.items():require(sha(WORK/(key+'.json'))==e['sha256'],'receipt_changed')
    return done

def collect():
    from broker_readonly import Broker,BrokerError,ticks
    p=check_freeze();done=ledger();auth=CompletionAuthority()
    with (WORK/'collection.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        with Broker(authority=auth) as broker:
            for j in p['episodes']:
                key='job-'+j['episode_id']
                if key in done:continue
                path=WORK/(key+'.json');require(not path.exists(),'unreceipted_source')
                req={'account':ACCOUNT,'symbol':j['symbol'],'from':j['from'],'to':j['to'],'spread':0.,'commission_round_trip':0.,'slippage_bps_per_side':24.,'tick_replay':{'volume':j['native_volume']}}
                receipt(WORK/'ledger.jsonl',{'key':key,'state':'started','request':req,'plan_sha256':sha(PLAN),'at_sgt':prior.now()});auth.current_job=j
                budget={'pages':0,'rows':0}
                try:
                    a,b=ms(j['from']),ms(j['to'])+5000;deadline=time.monotonic()+240
                    bids=ticks(broker,j['symbol_id'],1,a,b,budget,deadline);asks=ticks(broker,j['symbol_id'],2,a,b,budget,deadline)
                    v={'status':'complete','payload':{'quotes':{'bars':[],'bids':bids,'asks':asks,'contract':j['contract']},'m30':[]}}
                except Exception as e:v={'status':'failed','error':str(e) if isinstance(e,BrokerError) else type(e).__name__}
                v.update(request=req,budget=budget,attempts=1);raw=json.dumps(v,separators=(',',':'),allow_nan=False)+'\n';require(len(raw)<32000000,'source_byte_budget');path.write_text(raw)
                e={'key':key,'state':'finished','sha256':sha(path),'status':v['status'],'at_sgt':prior.now()};receipt(WORK/'ledger.jsonl',e);done[key]=e;auth.current_job=None
                print(json.dumps({'completed':len(done),'key':key,'status':v['status'],'error':v.get('error'),'pages':budget['pages'],'rows':budget['rows']}),flush=True)
                require(sum((WORK/(k+'.json')).stat().st_size for k in done)<512000000,'batch_byte_budget')
                if v.get('error') in ['CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED','INVALID_ACCESS_TOKEN','OA_AUTH_TOKEN_EXPIRED','TypeError','KeyError','NameError','AttributeError']:raise Refusal('system_or_auth_failure_no_retry')
            save(WORK/'collection-complete.json',{'at_sgt':prior.now(),'broker_messages':broker.counter,'tick_episodes':len(done),'new_m30_requests':0})

def path_coverage(job,raw):
    _,q,_,_=market(raw);start,end=ms(job['from']),ms(job['to']);sides={k:[t for t in q[k] if start<=t['ts_ms']<=end+5000] for k in ['bids','asks']}
    require(all(sides.values()),'missing_two_sided_tick_path')
    points=checkpoints(sides['bids'],sides['asks'],{'open':start,'end':end},q['contract']['digits'])
    ambiguous=any(len({t['price_e5'] for t in rows if t['ts_ms']==at})>1 for rows in sides.values() for at,n in collections.Counter(t['ts_ms'] for t in rows).items() if n>1)
    gaps=sum(b['ts_ms']-a['ts_ms']>6000 for rows in sides.values() for a,b in zip(rows,rows[1:]))
    return {'opening_fresh':points['open'] is not None,'closing_tail_fresh':points['end'] is not None,'from_ms':start if points['open'] else min(x[0]['ts_ms'] for x in sides.values()),'through_ms':min(x[-1]['ts_ms'] for x in sides.values()),'unexpected_gaps':gaps,'ordering_ambiguous':ambiguous}

def strict_exit_classification(strict):
    outcome=strict.get('outcome') or {};fills=outcome.get('fills',[])
    if not fills:return 'unresolved'
    reason=fills[0].get('reason');entry=strict['entry']
    if reason in ('stop','gap_through_stop'):
        if outcome.get('trail_applied',0)>0 and abs(outcome['active_stop']-entry['fill'])<1e-8:
            return 'breakeven_stop_after_1r'
        return 'initial_stop_loss'
    return 'strict_target_1_38r' if reason=='take_profit' else reason

def result_summary(rows):
    arm='arm_c_macro_filtered_pullback';scores={mode:{cost:u.score(rows,arm,mode,cost) for cost in ['3.74','24']} for mode in u.MODES}
    reasons=collections.Counter();classes=collections.Counter();ledger=[]
    for row in rows:
        a=row['native'].get('arms',{}).get(arm,{})
        if not a.get('strict'):continue
        strict=a['strict'];forced=a.get('forced');out=strict.get('outcome',{});fills=out.get('fills',[])
        reason=fills[0].get('reason') if fills else strict.get('status');reasons[reason]+=1;classes[strict_exit_classification(strict)]+=1
        ledger.append({'episode_id':row['episode_id'],'symbol':row['symbol'],'date':row['date'],'full_path_qualified':row['full_path_qualified'],'entry':strict.get('entry'),'strict':strict,'forced':forced})
    return {'scores':scores,'strict_exit_reasons':dict(reasons),'strict_exit_classifications':dict(classes),'trade_ledger':ledger}

def evaluate():
    import stage1_controls as controls
    offline_process();p=check_freeze();done=ledger();require(len(done)==30,'collection_not_finished')
    proof,jobs,bars,refs,verified,meta=prep(p);store=SimpleNamespace(bars=bars,source=lambda e:read(e['source_path']));decisions=[];prepared=[]
    for j in jobs:
        key='job-'+j['episode_id'];path=WORK/(key+'.json');rec=done[key]
        if rec['status']!='complete':decisions.append({'episode_id':j['episode_id'],'reason':'failed_tick_source','error':read(path).get('error')});continue
        e=dict(j,source_path=str(path),source_window_utc={'from':j['from'],'to':j['to']});peer=next(x for x in jobs if x['date']==j['date'] and x['symbol']==frozen.PEERS[j['symbol']])
        inp,cov=u.qualified_input(e,store,meta,peer,SimpleNamespace(median_entry_slippage_bps=None))
        if inp is None:decisions.append({'episode_id':j['episode_id'],'reason':'native_input_incomplete','coverage':cov});continue
        pc=path_coverage(j,read(path));v=copy.deepcopy(j);v.update(path_coverage=pc,protocol_status='clean_first_read',source_path=str(path),tick_receipt_sha256=sha(path))
        src=dict(verified);src[str(path)]=sha(path)
        try:strict_proof=verify_plan({'identity':p['identity'],'scheduled_episodes':1,'episodes':[v]},p['registry'],meta,bars,refs,src,stage='evaluation');refusal=None
        except Refusal as err:strict_proof=None;refusal=str(err)
        cov.update(path_coverage=pc,full_path_qualified=strict_proof is not None,gate3_evaluation_refusal=refusal)
        prepared.append((j,inp,cov,strict_proof));decisions.append({'episode_id':j['episode_id'],'reason':'recorded_scenario_ready','coverage':cov})
    coverage={'scheduled':40,'blocked_crypto':p['blocked_episodes'],'tick_sources':len(done),'completed_tick_sources':sum(x['status']=='complete' for x in done.values()),'decisions':decisions}
    cp=WORK/'coverage-before-outcomes.json'
    if cp.exists():require(read(cp)==coverage,'coverage_changed_after_outcomes')
    else:save(cp,coverage)
    rows=[];u.BIN=frozen.WORK/'bin';f=read(WORK/'freeze.json');checks=0
    for j,inp,cov,gproof in prepared:
        path=WORK/('input-'+j['episode_id']+'.json');save(path,{'input':inp,'policy':{'tier':'tier1','threshold_bps':48.,'admission_bps':3.74}})
        native,_,key=u.cached_native('tiered_pullback_replay',path,['stage1-18382',j['episode_id']],WORK,f);checks+=u.check_arithmetic(native)
        control=controls.evaluate(inp,j,bars,WORK,f)
        rows.append({'episode_id':j['episode_id'],'symbol':j['symbol'],'date':j['date'],'tick_receipt_sha256':done['job-'+j['episode_id']]['sha256'],'input_sha256':sha(path),'native_receipt_key':key,'coverage':cov,'full_path_qualified':gproof is not None,'gate3_evaluation_proof':gproof,'native':native,'controls':control})
    summary=result_summary(rows);certified=[r for r in rows if r['full_path_qualified']]
    out={'schema':'jiaoyi.stage1-completed.v1','trace':'№18,382','status':'stage1_partition_evaluated','generated_at_sgt':prior.now(),'plan_sha256':sha(PLAN),'frozen_engine':p['frozen_engine'],'controls_identity':p['controls'],'live_production_untouched':True,'cohort_week':'2024-11-11 through2024-11-15','episodes_scheduled':40,'preparation_complete_episodes':30,'preparation_complete_episodes_evaluated':len(rows),'crypto_preparation_blocked_episodes':p['blocked_episodes'],'crypto_diagnosis':p['crypto_diagnosis'],'new_m30_segments_fetched':0,'reused_m30_source_files':141,'new_tick_episodes_fetched':len(done),'failed_tick_sources':sum(x['status']!='complete' for x in done.values()),'full_path_qualified_episodes':len(certified),'diagnostic_recorded_path_episodes':len(rows)-len(certified),'coverage_before_outcomes':coverage,'coverage_before_outcomes_sha256':sha(cp),'rows':rows,'candidate':summary,'certified_subset':result_summary(certified),'matched_arm_scores':{arm:{mode:{cost:u.score(rows,arm,mode,cost) for cost in ['3.74','24']} for mode in u.MODES} for arm in u.ARMS},'passive_controls':controls.summary(rows),'collection_receipt_hashes':{k:v['sha256'] for k,v in done.items()},'verification':{'native_arithmetic_checks':checks,'frozen_input_hashes_verified':len(verified),'coverage_frozen_before_outcomes':True,'new_m30_requests':0},'financial_pass':False,'validation_opened':False}
    save(RESULT,out);compact={k:v for k,v in out.items() if k not in ['rows','frozen_engine','controls_identity','crypto_diagnosis','coverage_before_outcomes','collection_receipt_hashes']};compact['candidate']={k:v for k,v in summary.items() if k!='trade_ledger'};save(WORK/'summary.json',compact)
    print(json.dumps({k:out[k] for k in ['preparation_complete_episodes_evaluated','failed_tick_sources','full_path_qualified_episodes','candidate']},indent=2))

if __name__=='__main__':
    a=argparse.ArgumentParser();a.add_argument('action',choices=['freeze','collect','evaluate']);args=a.parse_args()
    if args.action=='freeze':print(json.dumps(freeze()['preflight_before_ticks']))
    elif args.action=='collect':collect()
    else:evaluate()
