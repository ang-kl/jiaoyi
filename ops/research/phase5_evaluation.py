#!/usr/bin/env python3
"""№18,364 bounded collection, immutable receipts, native fixed-window evaluation."""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
from dataclasses import asdict
from replay_inventory import ROOT, CORE, read, save, sha, canonical, ms, stamp, permitted, market, checkpoints
import unified_replay as u
from tiered_replay import CleanStore, paired
from phase4_forensics import intersect, DAY
from high_vol_cohorts import split_bar_window
from seasonal_pilot import instant, bar_request, decode_bars
from class_batch import receipt

PLAN=ROOT/'docs/rebuild_phase5_independent_evaluation_plan_18364.json'
RESULT=ROOT/'docs/rebuild_phase5_independent_evaluation_results_18364.json'
WORK=Path('/workspace/jiaoyi-rebuild-phase5-18364')
MANIFEST=ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json'
PHASE4=ROOT/'docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json'
ACCOUNT=49236051
CANDIDATE='s3_macro_pullback_fixed_windows_v1'
COHORTS={'2023_Q2_Spring':'2023-04-17','2023_Q4_Autumn':'2023-10-16','2024_Q1_Spring':'2024-03-04','2024_Q3_Summer':'2024-08-12'}
WINDOWS={'SpotCrude':'13:00','SpotBrent':'13:00','US500':'14:30','NAS100':'14:30','XAUUSD':'07:00','XAGUSD':'07:00','BTCUSD':'00:00','ETHUSD':'00:00'}
CLASSES={'SpotCrude':'energy','SpotBrent':'energy','US500':'equity_index','NAS100':'equity_index','XAUUSD':'metal','XAGUSD':'metal','BTCUSD':'crypto','ETHUSD':'crypto'}
PEERS={a:b for a,b in [('SpotCrude','SpotBrent'),('SpotBrent','SpotCrude'),('US500','NAS100'),('NAS100','US500'),('XAUUSD','XAGUSD'),('XAGUSD','XAUUSD'),('BTCUSD','ETHUSD'),('ETHUSD','BTCUSD')]}
CODE=['ops/research/phase5_evaluation.py','ops/research/broker_readonly.py','ops/research/seasonal_pilot.py','ops/research/high_vol_cohorts.py','ops/research/unified_replay.py','ops/research/tiered_replay.py','ops/research/replay_inventory.py']

def now():return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()
def opening(symbol,date):return ms(date+'T'+WINDOWS[symbol]+':00Z')
def sessions(symbol,date):
    day=dt.date.fromisoformat(date);out=[]
    for n in range(45,0,-1):
        d=day-dt.timedelta(days=n)
        if CLASSES[symbol]!='crypto' and d.weekday()>4:continue
        at=opening(symbol,d.isoformat());out.append({'open':stamp(at),'close':stamp(at+14400000),'lunch':None})
    return out

def verify_plan(p):
    for name,h in p['input_hashes'].items():
        if sha(ROOT/name)!=h:raise ValueError('frozen input changed:'+name)
    if len(p['jobs'])!=160 or len({j['key'] for j in p['jobs']})!=160:raise ValueError('wrong denominator')
    for j in p['jobs']:
        a,b=ms(j['from']),ms(j['to'])
        if a!=opening(j['symbol'],j['date']) or b-a!=14400000 or not permitted(a-45*DAY,b+72*3600000):raise ValueError('unregistered/protected job')
    for s in p['bar_segments']:
        if not permitted(ms(s['from']),ms(s['to'])):raise ValueError('protected M30 dependency')
        if s['mode']=='reuse' and sha(s['cache']['path'])!=s['cache']['sha256']:raise ValueError('cached receipt changed')
    return p

def prepare():
    if PLAN.exists():return verify_plan(read(PLAN))
    WORK.mkdir(exist_ok=True);old=read(PHASE4);manifest=read(MANIFEST);store=CleanStore(manifest).load()
    meta=read(ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json')['metadata']
    prior=[];registry=old['spent_registry']
    for v in registry['intervals']:
        if v['kind'].startswith('m30'):
            prior.append({'symbol':v['symbol'],'from':v['from'],'to':v['to'],'status':'spent_or_reserved_unavailable'})
    # Only immutable clean sources can replace a previously requested interval.
    for e in manifest['m30_segments']:
        if e['source_path'] not in store.verified:continue
        prior.append({'symbol':e['symbol'],**e['source_window_utc'],'status':'complete','cache':{'path':e['source_path'],'sha256':e['receipt_sha256']}})
    for e in manifest['episodes']:
        if e['source_path'] not in store.verified or not e.get('m30_embedded_count'):continue
        _,_,bars,_=market(store.source(e))
        if not bars:continue
        # Actual retained bar span, not a guessed45-day source interval.
        prior.append({'symbol':e['symbol'],'from':stamp(min(b['ts_ms'] for b in bars)),'to':stamp(max(b['ts_ms'] for b in bars)+1800000),
            'status':'complete','cache':{'path':e['source_path'],'sha256':e['tick_receipt_sha256']}})
    jobs=[];parts=[];overlaps=[]
    for cohort,monday in COHORTS.items():
        dates=[(dt.date.fromisoformat(monday)+dt.timedelta(days=n)).isoformat() for n in range(5)]
        for symbol in CORE:
            start=opening(symbol,dates[0])-45*DAY;end=opening(symbol,dates[-1])+4*3600000+72*3600000
            cuts=split_bar_window(instant(stamp(start)),instant(stamp(end)),[v for v in prior if v['symbol']==symbol])
            keys=[]
            for n,cut in enumerate(cuts):
                key=f'{cohort}-{symbol}-{n}';keys.append(key)
                parts.append(cut|{'key':key,'symbol':symbol,'symbol_id':meta[symbol]['symbol_id']})
            for date in dates:
                a=opening(symbol,date);b=a+14400000
                seen=[v for v in registry['intervals'] if v['symbol']==symbol and intersect(a,b+5001,ms(v['from']),ms(v['to']))]
                if seen:overlaps.append({'symbol':symbol,'date':date,'intervals':seen})
                jobs.append({'key':symbol+'-'+date,'symbol':symbol,'date':date,'cohort':cohort,'asset_class':CLASSES[symbol],
                    'from':stamp(a),'to':stamp(b),'symbol_id':meta[symbol]['symbol_id'],'bar_segments':keys,
                    'contract':{k:meta[symbol][k] for k in ['digits','min_volume','step_volume']},'native_volume':meta[symbol]['min_volume']})
    if overlaps:raise ValueError('evaluation windows already spent')
    store.finish()
    missing=[v for v in manifest['prior_receipt_reconciliation'] if v['status']=='unavailable_locally']
    hashes={str(p.relative_to(ROOT)):sha(p) for p in [MANIFEST,PHASE4,ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json']}
    p={'schema':'jiaoyi.phase5-fixed-window-plan.v1','trace':'№18,364','frozen_at_sgt':now(),'candidate_id':CANDIDATE,
       'owner_amendment':'Owner explicitly chose fixed windows as a changed candidate. Original s3_macro_pullback_v1 remains unchanged.',
       'hypothesis':'Trend-aligned50% retracement following a historical sigma-band impulse predicts remaining-window continuation after costs.',
       'dates':COHORTS,'symbols':CORE,'jobs':jobs,'bar_segments':parts,'input_hashes':hashes,
       'policy':{'threshold_bps':48.0,'admission_bps':3.74,'primary_pricing_bps':24.0,'sensitivity_bps':[2.0,3.74],
            'entry':'Existing14-session sigma at30-minute marks, fixed UTC opening reference; durable C<R veto and strictly later fresh quote.50% retracement; one attempt.',
            'context':'45 calendar days; prior fixed four-hour windows, weekdays for noncrypto and every day for crypto. Strict complete5/20-window trends and peer agreement. No opening-range fallback.',
            'risk':'Preserve cost-dependent structural R: distance to origin plus touch spread plus twice3.74bp; gross1.38R TP, entry-stop after+1R on a later quote.',
            'primary_horizon':'Forced window end on a fresh quote within5s; strict exits and72h-from-entry M30 continuation reported separately.',
            'marks':'30-minute marks through3h30m; pullback touches before4h. Fixed window end is an experiment boundary.',
            'controls':'Original always_long and own_momentum1h-entry/4h-close controls retain their24bp admission; A breakout/B unfiltered pullback use the candidate3.74bp admission and are the causal matched controls.',
            'costs':'Recorded spread once,24bp stress and2/3.74bp assumptions. Fees,swap,conversion and historical round-trip execution not calibrated; no empirical-cost claim.',
            'historical_calendar':'Coded fixed UTC windows only; historical broker calendars/contracts unverified. Missing slots exclude a session before outcomes; no holiday bars fabricated or outcome-based exclusions.'},
       'independence':{'retained_window_overlaps':0,'shared_preparation_reused':True,'unavailable_prior_references':len(missing),
          'missing_collection_ledger_references':sum(v['kind']=='collection_ledger' for v in missing),
          'limitation':'530 missing references are mixed code/reports/sources and unhashed hints, not530 demonstrated missing market receipts. No unavailable collection-ledger reference. Retained-catalog independence only; historical backtest is not chronological forward validation.'},
       'limits':{'episodes':160,'new_m30_jobs':sum(x['mode']=='collect' for x in parts),'reused_m30_slices':sum(x['mode']=='reuse' for x in parts),
          'blocked_m30_slices':sum(x['mode']=='blocked' for x in parts),'tick_pages_per_job':120,'ticks_per_job':200000,'seconds_per_tick_job':240,'attempts':1,'total_source_bytes':1280000000},
       'stopping':'One fixed batch; retain failures/censoring; no resampling, retuning or extending until a win-rate target.',
       'pilot_gate':'Per class:20 closed trades across>=3 weeks,win>=50%,positive24bp mean and positive C-minus-B return per eligible opportunity; otherwise fail/insufficient. Original73%/1:1.38 latest20 per account unchanged.',
       'validation_opened':False,'financial_pass':False,'live_rules_changed':False}
    verify_plan(p);save(PLAN,p);print(json.dumps(p['limits']),flush=True);return p

def freeze_code(p):
    files=[ROOT/n for n in CODE]+list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',ROOT/'crates/jiaoyi-core/examples/phase5_fixed_window_replay.rs',PLAN]
    hashes={str(x.relative_to(ROOT)):sha(x) for x in files}
    for n,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+n],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit before collection:'+n)
    freeze={'plan_sha256':sha(PLAN),'source_hashes':hashes,'binary_sha256':sha(u.BIN/'phase5_fixed_window_replay'),'commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()}
    path=WORK/'freeze.json'
    if path.exists():
        if read(path)!=freeze:raise ValueError('frozen code drift')
    else:save(path,freeze)
    return freeze

def ledger():
    p=WORK/'ledger.jsonl';es=[json.loads(s) for s in p.read_text().splitlines()] if p.exists() else []
    starts=[e['key'] for e in es if e['state']=='started'];ends={e['key']:e for e in es if e['state']=='finished'}
    if len(starts)!=len(set(starts)) or set(starts)!=set(ends):raise ValueError('uncertain/spent receipt; never retry')
    for key,e in ends.items():
        if sha(WORK/(key+'.json'))!=e['sha256']:raise ValueError('receipt mismatch')
    return ends

def collect(p):
    from broker_readonly import Broker,BrokerError,ticks
    freeze_code(p);done=ledger()
    expected={'bar-'+x['key'] for x in p['bar_segments'] if x['mode']=='collect'}|{'job-'+j['key'] for j in p['jobs']}
    if set(done)==expected:return
    if not set(done)<=expected:raise ValueError('unexpected receipt')
    used=sum((WORK/(k+'.json')).stat().st_size for k in done)
    def perform(key,req,op):
        nonlocal used
        if key in done:return
        path=WORK/(key+'.json')
        if path.exists():raise ValueError('unreceipted source')
        receipt(WORK/'ledger.jsonl',{'key':key,'state':'started','at_sgt':now(),'plan_sha256':sha(PLAN),'request':req})
        budget={'pages':0,'rows':0}
        try:value=op(budget)
        except Exception as error:value={'status':'failed','error':str(error) if isinstance(error,BrokerError) else type(error).__name__}
        value.update(request=req,budget=budget,attempts=1)
        raw=json.dumps(value,separators=(',',':'),allow_nan=False)
        if len(raw)>32000000 or used+len(raw)>p['limits']['total_source_bytes']:raise ValueError('byte budget; uncertain source retained')
        path.write_text(raw+'\n');used+=path.stat().st_size
        e={'key':key,'state':'finished','sha256':sha(path),'status':value['status'],'at_sgt':now()}
        receipt(WORK/'ledger.jsonl',e);done[key]=e
        print(json.dumps({'finished':len(done),'of':len(expected),'component':key,'status':value['status'],'error':value.get('error'),'pages':budget['pages']}),flush=True)
        if value.get('error') in ['TypeError','KeyError','ValueError','AttributeError','NameError','AssertionError','CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED','INVALID_ACCESS_TOKEN','OA_AUTH_TOKEN_EXPIRED']:raise ValueError('unexpected/auth failure retained; stop')
    with (WORK/'collection.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        with Broker() as broker:
            catalog=broker.request(2114,{'ctidTraderAccountId':ACCOUNT,'includeArchivedSymbols':False})
            details=broker.request(2116,{'ctidTraderAccountId':ACCOUNT,'symbolId':sorted({j['symbol_id'] for j in p['jobs']})})
            for s in CORE:
                j=next(j for j in p['jobs'] if j['symbol']==s);names=[x for x in catalog['symbol'] if x['symbolName']==s]
                c=next(x for x in details['symbol'] if x['symbolId']==j['symbol_id'])
                if len(names)!=1 or names[0]['symbolId']!=j['symbol_id'] or any(c[a]!=j['contract'][b] for a,b in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('current contract mismatch')
            save(WORK/'broker-specs.json',{'captured_at_sgt':now(),'account':ACCOUNT,'details':details,'historical_terms_verified':False})
            for part in p['bar_segments']:
                if part['mode']!='collect':continue
                def fetch(budget,part=part):
                    budget['pages']+=1;raw=broker.request(2137,bar_request(part))
                    try:return {'status':'complete','bars':decode_bars(raw,part)}
                    except BrokerError as e:return {'status':'failed','error':str(e),'raw_response':raw}
                perform('bar-'+part['key'],part,fetch)
            for j in p['jobs']:
                def fetch(budget,j=j):
                    a,b=ms(j['from']),ms(j['to'])+5000;deadline=time.monotonic()+240
                    bids=ticks(broker,j['symbol_id'],1,a,b,budget,deadline);asks=ticks(broker,j['symbol_id'],2,a,b,budget,deadline)
                    return {'status':'complete','payload':{'quotes':{'bars':[],'bids':bids,'asks':asks,'contract':j['contract']},'m30':[]}}
                req={'account':ACCOUNT,'symbol':j['symbol'],'from':j['from'],'to':j['to'],'spread':0.,'commission_round_trip':0.,'slippage_bps_per_side':24.,'tick_replay':{'volume':j['native_volume']}}
                perform('job-'+j['key'],req,fetch)
            save(WORK/'collection-complete.json',{'broker_messages_this_connection':broker.counter,'ledger_components':len(done),'new_m30_requests':sum(k.startswith('bar-') for k in done),'at_sgt':now()})


def assembled(p):
    bars=collections.defaultdict(dict);hashes={};failed=[]
    for part in p['bar_segments']:
        if part['mode']=='blocked':failed.append({'key':part['key'],'reason':part['reason']});continue
        path=Path(part['cache']['path']) if part['mode']=='reuse' else WORK/('bar-'+part['key']+'.json')
        hashes[str(path)]=sha(path);v=read(path)
        if v['status']!='complete':failed.append({'key':part['key'],'reason':v.get('error')});continue
        found=market(v);data=found[2] if found else []
        a,b=ms(part['from']),ms(part['to'])
        for row in data:
            t=row['ts_ms']
            if not a<=t<b:continue
            if not permitted(t,t+1800000):raise ValueError('protected bar')
            old=bars[part['symbol']].get(t)
            if old is not None and old!=row:raise ValueError('conflicting M30')
            bars[part['symbol']][t]=row
    return bars,hashes,failed

def qualify(j,bars,done):
    path=WORK/('job-'+j['key']+'.json');x=read(path);start,end=ms(j['from']),ms(j['to']);sym=j['symbol'];own=bars[sym]
    row={k:j[k] for k in ['key','symbol','date','cohort','asset_class']};row.update(episode_id=j['key'],source_sha256=sha(path),source_status=x['status'])
    required=u.slots(sessions(sym,j['date']));missing=[t for t in required if t not in own];reasons=[]
    if missing:reasons.append('missing_45day_fixed_window_M30')
    if start not in own or own[start]['volume']<=0:reasons.append('missing_opening_M30')
    if x['status']!='complete':reasons.append('tick_source_failed')
    cov={'reasons':reasons,'required_prep_slots':len(required),'missing_prep_slots':len(missing),'first_missing':[stamp(t) for t in missing[:8]],'historical_calendar_verified':False}
    row['coverage']=cov
    if reasons:return None,row
    q=x['payload']['quotes'];points=checkpoints(q['bids'],q['asks'],{'open':start,'end':end},q['contract']['digits']);cov['quote_checkpoints']=points
    if points['open'] is None or any(not q[k] or q[k][-1]['ts_ms']<end-6000 for k in ['bids','asks']):
        reasons.append('missing_fresh_tick_boundary');return None,row
    ownctx=u.causal_context(sessions(sym,j['date']),own,start)
    peer=PEERS[sym];peerctx=u.causal_context(sessions(peer,j['date']),bars[peer],start)
    peer_key='job-'+peer+'-'+j['date']
    if done[peer_key]['status']!='complete':peerctx={'status':'planned_peer_source_failed','drift_5':None,'drift_20':None,'latest_bar_end':None}
    prior=[b for t,b in sorted(own.items()) if start-45*DAY<=t and t+1800000<=start]
    source=q|{'bars':[{'at':stamp(b['ts_ms']),'open':b['open_e5']/100000,'close':b['close_e5']/100000} for b in prior+[own[start]]]}
    future=[b for t,b in sorted(own.items()) if end<=t<end+72*3600000]
    inp={'input':{'request':x['request'],'source':source,'future':future,'own_context':ownctx,'peer_context':peerctx,'peer_available':True,'demo_entry_bps':None},
         'control_bars':[b for t,b in sorted(own.items()) if start<=t<t+1800000<=start+3600000]}
    cov.update(continuation_bars=len(future),continuation_starts_at_boundary=bool(future and future[0]['ts_ms']==end),peer_status=peerctx['status'])
    row.update(own_context=ownctx,peer_context=peerctx);return inp,row

def control_score(rows,name,cost):
    vals=[];unresolved=entries=0
    for r in rows:
        c=r['native']['controls'][name].get('control')
        if not c:continue
        entries+=1
        if c['censored']:unresolved+=1
        else:vals.append(c['net_by_bps'][cost])
    return summary_values(vals,len(rows),entries,unresolved)

def summary_values(v,episodes,entries,unresolved):
    gains=sum(x for x in v if x>0);losses=-sum(x for x in v if x<0)
    return {'episodes':episodes,'entries':entries,'priced_closed':len(v),'wins':sum(x>1e-12 for x in v),'losses':sum(x< -1e-12 for x in v),'net_flat':sum(abs(x)<=1e-12 for x in v),
        'unresolved':unresolved,'net_win_rate':sum(x>1e-12 for x in v)/len(v) if v else None,'mean_net_r':sum(v)/len(v) if v else None,'profit_factor':gains/losses if losses else None,'profit_factor_status':'finite' if losses else 'no_losses' if gains else 'unavailable','financial_pass':False}

def evaluate(p):
    sys.addaudithook(u.deny_network);freeze=freeze_code(p);done=ledger();bars,hashes,failed=assembled(p)
    chosen=[];excluded=[]
    for j in p['jobs']:
        inp,row=qualify(j,bars,done)
        if inp is None:excluded.append(row)
        else:chosen.append((inp,row))
    coverage={'scheduled':160,'tick_sources_completed':sum(done['job-'+j['key']]['status']=='complete' for j in p['jobs']),
        'qualified_before_outcomes':len(chosen),'excluded':excluded,'m30_component_failures':failed,'source_hashes':hashes}
    frozen_coverage=WORK/'coverage-before-outcomes.json'
    if frozen_coverage.exists() and read(frozen_coverage)!=coverage:raise ValueError('coverage decisions changed')
    save(frozen_coverage,coverage);rows=[];checks=0
    (WORK/'native').mkdir(exist_ok=True)
    for inp,row in chosen:
        path=WORK/('input-'+row['key']+'.json');save(path,inp)
        n,hit,key=u.cached_native('phase5_fixed_window_replay',path,row['key'],WORK,freeze)
        checks+=u.check_arithmetic(n)
        row.update(native=n,input_sha256=sha(path),native_receipt_key=key);rows.append(row)
        print('REPLAY '+row['key']+' '+n['status'],flush=True)
    groups={'all':rows}|{s:[r for r in rows if r['symbol']==s] for s in CORE}|{c:[r for r in rows if r['asset_class']==c] for c in sorted(set(CLASSES.values()))}|{c:[r for r in rows if r['cohort']==c] for c in COHORTS}
    scores={g:{a:{m:{c:u.score(rs,a,m,c) for c in u.BPS} for m in u.MODES} for a in u.ARMS} for g,rs in groups.items()}
    controls={g:{a:{c:control_score(rs,a,c) for c in u.BPS} for a in ['always_long','own_momentum']} for g,rs in groups.items()}
    comparisons={g:{c:paired(rs,'forced',c) for c in u.BPS} for g,rs in groups.items()}
    for path,h in hashes.items():
        if sha(path)!=h:raise ValueError('bar source changed during replay')
    verify_plan(p);candidate=u.ARMS[2];metrics=scores['all'][candidate]['forced']['24']
    result={'schema':'jiaoyi.phase5-results.v1','trace':'№18,364','candidate_id':CANDIDATE,'original_candidate_id':'s3_macro_pullback_v1','candidate_changed_with_owner_authorization':True,
        'plan_sha256':sha(PLAN),'code_freeze':freeze,'coverage':coverage,'rows':rows,'summary':scores,'controls':controls,'paired_macro':comparisons,
        'new_m30_requests':sum(k.startswith('bar-') for k in done),'cached_m30_reused':sum(p['mode']=='reuse' for p in p['bar_segments']),
        'collection_receipt_hashes':{k:v['sha256'] for k,v in done.items()},'verification':{'native_arithmetic_checks':checks,'coverage_before_outcomes_sha256':sha(frozen_coverage),'network_guard':True},
        'milestone_20_trades_reached':metrics['priced_closed']>=20,'account_goal_73_pct_satisfied':False,'financial_pass':False,'validation_opened':False,'live_rules_changed':False,
        'rfc_gates_passed':2,'rfc_gates_total':12,'independence':p['independence'],'cost_status':'3.74bp is sensitivity, not empirically verified round-trip cost'}
    save(RESULT,result);print(json.dumps({'coverage':{k:v for k,v in coverage.items() if k not in ['excluded','source_hashes','m30_component_failures']},'candidate':scores['all'][candidate],'sha256':sha(RESULT)},indent=2),flush=True)
    return result

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('action',choices=['prepare','collect','evaluate']);args=parser.parse_args();WORK.mkdir(exist_ok=True)
    p=prepare()
    if args.action=='collect':collect(p)
    elif args.action=='evaluate':evaluate(p)
