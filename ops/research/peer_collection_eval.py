#!/usr/bin/env python3
"""Owner's 40 peer slots and 80 evaluation slots; fixed dates, real receipts.

Missing primary sources remain missing. Cached reads are reused, never repeated.
The twenty-close milestone is measured after the batch, not a sampling stop rule.
"""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import html
import json
import math
import os
from pathlib import Path
import subprocess
import tempfile
import time
from zoneinfo import ZoneInfo

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks
from class_batch import digest, receipt, save
from seasonal_pilot import bar_request, decode_bars, instant, protected, segments, stamp
from run_repaired_offline_replay import observation, score, ci_state

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT/'docs/peer-collection-plan-18338.json'
RESULT = ROOT/'docs/clean_peer_collection_results_18338.json'
REPORT = ROOT/'doc/CLEAN_PEER_COLLECTION_20TRADE_REPORT-18338-2026-11-07.md'
BINARY = ROOT/'target/debug/examples/peer_collection_replay'
PRIOR = Path('/workspace/jiaoyi-seasonal-stage1-18316')
BRANCH = 'eval/peer-collection-20trade-18338'
PEERS = ['NAS100', 'XAGUSD', 'BTCUSD', 'ETHUSD']
SYMBOLS = ['US500', 'XAUUSD', 'SpotCrude', 'SpotBrent'] + PEERS
DATES = [f'2024-05-{d}' for d in range(13,18)] + [f'2025-05-{d}' for d in range(12,17)]
PAIRS = dict(SpotCrude='SpotBrent', SpotBrent='SpotCrude', US500='NAS100', NAS100='US500',
             XAUUSD='XAGUSD', XAGUSD='XAUUSD', BTCUSD='ETHUSD', ETHUSD='BTCUSD')
CLASSES = {s: ('energy' if s in ['SpotCrude','SpotBrent'] else 'equity_index' if s in ['US500','NAS100']
               else 'metal' if s in ['XAUUSD','XAGUSD'] else 'crypto') for s in SYMBOLS}
MODELS = ['s3_two_stage','macro_180m_two_stage','production_s3_baseline','always_long_control','own_momentum_fixed_horizon']
UTC = dt.timezone.utc
CODE = ['ops/research/peer_collection_eval.py','ops/research/run_peer_collection_and_20trade_eval.sh',
        'ops/research/test_peer_collection_eval.py','crates/jiaoyi-core/examples/peer_collection_replay.rs',
        'crates/jiaoyi-core/src/research/peer_evaluation.rs','crates/jiaoyi-core/src/research/three_year.rs',
        'crates/jiaoyi-core/src/research/offline_repaired.rs','crates/jiaoyi-core/src/research/mod.rs']


def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def now(): return dt.datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')
def cmd(*a): return subprocess.check_output(a,cwd=ROOT,text=True,timeout=120).strip()
def gh(*a): return json.loads(cmd('gh',*a))
def events(path): return [json.loads(s) for s in path.read_text().splitlines()] if path.exists() else []


def opening(s, date):
    date=dt.date.fromisoformat(date)
    zone=ZoneInfo('America/New_York') if CLASSES[s]=='equity_index' else UTC
    at=dt.time(9,30) if CLASSES[s]=='equity_index' else dt.time(0 if CLASSES[s]=='crypto' else 7)
    return dt.datetime.combine(date,at,zone).astimezone(UTC)


def interval_overlap(a,b):
    return a['symbol_id']==b['symbol_id'] and instant(a['from'])<instant(b['to']) and instant(b['from'])<instant(a['to'])


def journal(text, section):
    path=ROOT/'log/CODEX_Log.txt'
    old=path.read_text()
    marker=f'№18,338 {section}'
    if marker in old: return
    import re
    n=max(map(int,re.findall(r'\[§LOG\.(\d+)\]',old)),default=0)+1
    path.write_text(old+f'\n[§LOG.{n}] {marker} · {now()}; by <Codex (GPT-6), effort: not exposed>.\n'+text+'\n')


def prepare():
    if PLAN.exists(): return json.loads(PLAN.read_text())
    if cmd('git','branch','--show-current')!=BRANCH: raise ValueError('use isolated task branch before writing plan')
    old=json.loads((ROOT/'docs/seasonal-execution-18315.json').read_text())
    old_by={j['key']:j for j in old['jobs']}
    ends={e['key']:e for e in events(PRIOR/'ledger.jsonl') if e['state']=='finished'}
    meta=json.loads((ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json').read_text())['metadata']
    lineage={};parts={};jobs=[]
    def cached(key):
        path=PRIOR/(key+'.json');h=sha(path)
        if ends.get(key,{}).get('file_sha256')!=h: raise ValueError('prior source lacks matching receipt: '+key)
        lineage[str(path)]=h
        return dict(path=str(path),sha256=h)
    for date in DATES:
        for s in SYMBOLS:
            a=opening(s,date);start=a+dt.timedelta(hours=6) if CLASSES[s]=='energy' else a
            end=start+dt.timedelta(hours=4);key=s+'-'+date;m=meta[s]
            j=dict(key=key,symbol=s,date=date,cohort=date[:4]+'-May',asset_class=CLASSES[s],symbol_id=m['symbol_id'],
                   source_from=stamp(a),request=dict(account=ACCOUNT,symbol=s,**{'from':stamp(start),'to':stamp(end)},
                     spread=0.0,commission_round_trip=0.0,slippage_bps_per_side=24.0,
                     tick_replay=dict(volume=m['min_volume'],entry_rule='current_s3')),
                   contract={k:m[k] for k in ['digits','min_volume','step_volume']},bar_segments=[])
            if key in old_by:
                j.update(mode='reuse',prior_source=cached('job-'+key),previously_observed=True)
                source=json.loads(Path(j['prior_source']['path']).read_text())
                j['source_to']=source['request']['to']
                j['partial_observation']=instant(j['source_to'])<end
                for name in old_by[key]['bar_segments']:
                    part=next(p for p in old['bar_segments'] if p['key']==name)
                    parts[name]=dict(part,mode='reuse',prior_source=cached('bar-'+name))
                    j['bar_segments'].append(name)
            elif s in PEERS:
                j.update(mode='collect',previously_observed=False,source_to=stamp(end),partial_observation=False)
                group=[d for d in DATES if d[:4]==date[:4]]
                for i,(left,right) in enumerate(segments(opening(s,group[0])-dt.timedelta(days=45),opening(s,group[-1])+dt.timedelta(hours=4))):
                    name=f'{date[:4]}-May-{s}-{i}'
                    parts[name]=dict(key=name,symbol=s,symbol_id=m['symbol_id'],mode='collect',**{'from':stamp(left),'to':stamp(right)})
                    j['bar_segments'].append(name)
            else:
                j.update(mode='missing',previously_observed=False,reason='primary_source_absent_outside_40_peer_collection_slots')
            jobs.append(j)
    # Audit against every committed source manifest from the earlier batches.
    spent_ticks=[];spent_bars=[];prior_manifests={}
    for name in ['docs/seasonal-execution-18315.json','docs/three-year-execution-18324.json','docs/class-batch-40-plan-18309-2026-11-07.json']:
        path=ROOT/name;prior_manifests[name]=sha(path);prior=json.loads(path.read_text())
        for j in prior['jobs']:
            req=j['request'];spent_ticks.append(dict(symbol_id=j['symbol_id'],**{'from':j.get('source_from',req['from']),'to':stamp(instant(req['to'])+dt.timedelta(seconds=5,milliseconds=1))}))
        spent_bars.extend(prior.get('bar_segments',[]))
    for j in jobs:
        if j['mode']=='collect':
            window=dict(symbol_id=j['symbol_id'],**{'from':j['source_from'],'to':stamp(instant(j['source_to'])+dt.timedelta(seconds=5,milliseconds=1))})
            if any(interval_overlap(window,x) for x in spent_ticks): raise ValueError('refuse overlapping spent tick read: '+j['key'])
    for p in parts.values():
        if p['mode']=='collect' and any(interval_overlap(p,x) for x in spent_bars): raise ValueError('refuse overlapping spent M30 read: '+p['key'])
    plan=dict(trace='№18,338 §§1–6',prepared_at_sgt=now(),by='Codex (GPT-6), effort not exposed',
        dates=DATES,symbols=SYMBOLS,peer_symbols=PEERS,peer_pairs=PAIRS,models=MODELS,jobs=jobs,bar_segments=list(parts.values()),
        prior_manifests=prior_manifests,prior_source_sha256=lineage,prior_ledger_sha256=sha(PRIOR/'ledger.jsonl'),
        architecture_sha256=sha(ROOT/'docs/s3_two_stage_architecture_lock_18334.json'),
        costs=dict(primary_bps_per_side=24,same_fill_sensitivities=[2,3.74],extra_commission=0,calibration='unverified; bid/ask includes spread once'),
        rules='Unchanged strict C<R admission and two-stage +1R trigger / gross1.38R target. Peer confirmation affects macro only.',
        scope='40 peer slots only; reuse5 BTC slots; missing primary slots retained. Previously observed 2025 dates remain development.',
        limits=dict(peer_slots=40,fresh_tick_jobs=35,reused_tick_sources=20,missing_primary_slots=25,
                    fresh_bar_components=sum(p['mode']=='collect' for p in parts.values()),max_tick_pages=120,max_tick_rows=200000,
                    seconds_per_tick_job=240,attempts=1,source_bytes_per_file=32000000,total_new_source_bytes=256000000,max_replays=55),
        validation_opened=False,financial_pass=False,stopping_rule='Complete fixed slots once; never collect until 20 closes or until a win threshold passes',
        goal='73% net wins / reference R:R1:1.38 latest20 per strategy/account; 15/20=75%; pooled historical research is not account qualification')
    PLAN.write_text(json.dumps(plan,indent=2)+'\n');validate_plan(plan)
    journal('Frozen40 peer slots:35 new tick reads,5 cached BTC sources;20 total reused sources and25 missing primaries retained in80 slots.\n2025 May dates previously observed; no clean-validation or Stage2 claim. One attempt per component; source overlaps forbidden.\nExisting cost/exit formulae retained; offline8-symbol scope and symmetric crypto peers added. No live changes.', '§§1–6')
    return plan


def validate_plan(p):
    if p['dates']!=DATES or p['symbols']!=SYMBOLS or len(p['jobs'])!=80: raise ValueError('scope drift')
    if len({j['key'] for j in p['jobs']})!=80: raise ValueError('duplicate slots')
    if collections.Counter(j['mode'] for j in p['jobs'])!={'collect':35,'reuse':20,'missing':25}: raise ValueError('source modes drift')
    for j in p['jobs']:
        req=j['request'];a,b=instant(j['source_from']),instant(req['to'])+dt.timedelta(seconds=5)
        if protected(a,b) or b>=dt.datetime(2025,10,24,tzinfo=UTC) or req['account']!=ACCOUNT: raise ValueError('seal/account violation')
    for part in p['bar_segments']:
        a,b=instant(part['from']),instant(part['to'])
        if protected(a,b) or not dt.timedelta(0)<b-a<=dt.timedelta(days=7): raise ValueError('bar bounds')
    for name,h in p['prior_source_sha256'].items():
        if sha(Path(name))!=h: raise ValueError('prior source drift')
    for name,h in p['prior_manifests'].items():
        if sha(ROOT/name)!=h: raise ValueError('prior manifest drift')
    if sha(PRIOR/'ledger.jsonl')!=p['prior_ledger_sha256']: raise ValueError('prior ledger drift')


def provenance(out,p):
    validate_plan(p)
    paths=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',PLAN]
    paths += [ROOT/x for x in CODE if not x.startswith('crates/jiaoyi-core/src/')]
    paths += [ROOT/'ops/research'/n for n in ['broker_readonly.py','class_batch.py','seasonal_pilot.py','run_repaired_offline_replay.py']]
    source={str(f.relative_to(ROOT)):sha(f) for f in paths}
    for name,h in source.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)).hexdigest()!=h: raise ValueError('commit code and plan before collection')
    frozen=dict(plan_sha256=sha(PLAN),sources=source,binary_sha256=sha(BINARY))
    path=out/'freeze.json'
    if path.exists():
        old=json.loads(path.read_text())
        if any(old[k]!=v for k,v in frozen.items()): raise ValueError('frozen inputs changed')
        return old
    if (out/'ledger.jsonl').exists(): raise ValueError('receipts without pre-read freeze')
    frozen.update(commit=cmd('git','rev-parse','HEAD'),frozen_at_sgt=now());save(path,frozen)
    return frozen


def completed(out):
    es=events(out/'ledger.jsonl');starts=[e['key'] for e in es if e['state']=='started'];ends=[e for e in es if e['state']=='finished']
    if len(starts)!=len(set(starts)) or len(ends)!=len({e['key'] for e in ends}) or set(starts)!={e['key'] for e in ends}: raise ValueError('uncertain receipt; no automatic repeat')
    for e in ends:
        if sha(out/(e['key']+'.json'))!=e['sha256']: raise ValueError('receipt hash mismatch')
    return {e['key']:e for e in ends}


def collect(out,p):
    done=completed(out)
    fresh=[('bar-'+x['key'],x) for x in p['bar_segments'] if x['mode']=='collect']
    fresh += [('tick-'+x['key'],x) for x in p['jobs'] if x['mode']=='collect']
    if not set(done)<= {k for k,_ in fresh}: raise ValueError('unknown collection receipt')
    if len(done)==len(fresh):return
    with Broker() as broker:
        catalog=broker.request(2114,dict(ctidTraderAccountId=ACCOUNT,includeArchivedSymbols=False))
        metadata=broker.request(2116,dict(ctidTraderAccountId=ACCOUNT,symbolId=sorted({j['symbol_id'] for j in p['jobs']})))
        for j in p['jobs']:
            matched=[s for s in catalog['symbol'] if s['symbolName']==j['symbol']]
            if len(matched)!=1 or matched[0]['symbolId']!=j['symbol_id']:raise ValueError('broker identity drift')
            c=next(s for s in metadata['symbol'] if s['symbolId']==j['symbol_id'])
            if any(c[k]!=j['contract'][v] for k,v in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('broker contract drift')
        if not (out/'metadata.json').exists():save(out/'metadata.json',dict(at_sgt=now(),details=metadata))
        for key,j in fresh:
            if key in done:continue
            if (out/(key+'.json')).exists():raise ValueError('unreceipted source file')
            receipt(out/'ledger.jsonl',dict(key=key,state='started',at_sgt=now(),plan_sha256=sha(PLAN)))
            budget=dict(pages=0,rows=0)
            try:
                if key.startswith('bar-'):
                    raw=broker.request(2137,bar_request(j));value=dict(status='complete',bars=decode_bars(raw,j))
                else:
                    a=int(instant(j['source_from']).timestamp()*1000);b=int(instant(j['source_to']).timestamp()*1000)+5000
                    deadline=time.monotonic()+240
                    bids=ticks(broker,j['symbol_id'],1,a,b,budget,deadline)
                    asks=ticks(broker,j['symbol_id'],2,a,b,budget,deadline)
                    value=dict(status='complete',quotes=dict(bars=[],bids=bids,asks=asks,contract=j['contract']))
            except Exception as error:
                reason=str(error) if isinstance(error,BrokerError) else type(error).__name__
                value=dict(status='failed',error=reason)
            value.update(budget_used=budget,request=j,attempts=1)
            size=len(json.dumps(value,separators=(',',':')).encode())
            used=sum(f.stat().st_size for pattern in ['tick-*.json','bar-*.json'] for f in out.glob(pattern))
            if size>32000000 or used+size>256000000:raise ValueError('source byte budget; uncertain receipt retained')
            path=out/(key+'.json');save(path,value)
            receipt(out/'ledger.jsonl',dict(key=key,state='finished',status=value['status'],sha256=sha(path),at_sgt=now()))
            print(json.dumps(dict(component=key,status=value['status'],error=value.get('error'),pages=budget['pages'])),flush=True)
            if value.get('error') in {'TypeError','KeyError','ValueError','AttributeError','NameError','AssertionError','CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED','INVALID_ACCESS_TOKEN','OA_AUTH_TOKEN_EXPIRED'}:raise RuntimeError('retained failure; stop rather than spending further reads')


def net_verify(models):
    count=0
    for name in ['s3_two_stage','macro_180m_two_stage']:
        m=models[name];trade=m.get('study',{}).get('trade')
        if not trade:continue
        entry=trade['entry'];sign=1 if entry['side']=='long' else -1
        for arm,key in [('trailing','candidate'),('fixed','baseline')]:
            o=trade['exit_comparison'][key]
            if o['remaining_volume']:
                if o['scenario_gross_r'] is not None or any(v is not None for v in m['sensitivities'][arm].values()):raise ValueError('censored outcome counted closed')
                continue
            if sum(f['volume'] for f in o['fills'])!=entry['volume']:raise ValueError('volume conservation')
            average=sum(f['price']*f['volume']/entry['volume'] for f in o['fills'])
            gross=sign*(average-entry['fill'])/entry['initial_r']
            for bps,net in m['sensitivities'][arm].items():
                expected=gross-(entry['fill']+average)*float(bps)/10000/entry['initial_r']
                if not math.isclose(net,expected,abs_tol=1e-8,rel_tol=1e-8):raise ValueError('net accounting mismatch')
                count+=1
    return count


def latest20(rows,name):
    closes=[]
    for row in rows:
        m=row.get('models',{}).get(name,{});_,net,_,_=observation(m,'24')
        if net is None:continue
        control=m.get('control')
        at=control['closed_at'] if control is not None else m['study']['trade']['exit_comparison']['candidate']['fills'][-1]['at']
        closes.append((at,row['key'],net))
    closes.sort();last=closes[-20:];wins=sum(v>0 for _,_,v in last)
    return dict(available_closes=len(closes),window_size=len(last),wins=wins,win_rate=wins/len(last) if last else None,
                enough_closes=len(last)==20,numerical_73_percent_threshold_met=len(last)==20 and wins>=15,
                account_goal_satisfied=False,scope='pooled research only; not account outcomes',trades=[dict(at=t,key=k,net_r=v) for t,k,v in last])


def replay(out,p):
    rows=[];bars={};bar_failures={};checks=0;replayed=0
    for part in p['bar_segments']:
        path=Path(part['prior_source']['path']) if part['mode']=='reuse' else out/('bar-'+part['key']+'.json')
        value=json.loads(path.read_text());bars[part['key']]=value.get('bars',[]) if value['status']=='complete' else []
        bar_failures[part['key']]=value['status']!='complete'
    jobs={j['key']:j for j in p['jobs']}
    def get_bars(job,left,right):
        result={}
        for name in job['bar_segments']:
            for b in bars[name]:
                if left<=b['ts_ms'] and b['ts_ms']+1800000<=right:
                    if b['ts_ms'] in result and result[b['ts_ms']]!=b:raise ValueError('conflicting bar sources')
                    result[b['ts_ms']]=b
        return [result[t] for t in sorted(result)]
    for j in p['jobs']:
        row={k:j[k] for k in ['key','symbol','date','cohort','asset_class','mode','previously_observed']}
        row['models']={};row['partial_observation']=j.get('partial_observation',False)
        if j['mode']=='missing':row.update(status='missing_source',error=j['reason'])
        else:
            path=Path(j['prior_source']['path']) if j['mode']=='reuse' else out/('tick-'+j['key']+'.json')
            source=json.loads(path.read_text());row.update(status=source['status'],source_sha256=sha(path))
            if source['status']!='complete':row['error']=source['error']
            else:
                quotes=source['payload']['quotes'] if j['mode']=='reuse' else source['quotes']
                a=int(instant(j['source_from']).timestamp()*1000);b=int(instant(j['request']['to']).timestamp()*1000)
                start=int(instant(j['request']['from']).timestamp()*1000)
                peer=jobs[PAIRS[j['symbol']]+'-'+j['date']]
                payload=dict(quotes=quotes,m30=get_bars(j,a-45*86400000,b),source_from=j['source_from'],
                             peer_symbol=PAIRS[j['symbol']],peer_m30=get_bars(peer,start,start+10800000))
                row['bar_component_failures']=sum(bar_failures[n] for n in set(j['bar_segments']+peer['bar_segments']))
                job=dict(status='complete',request=j['request'],payload=payload,payload_sha256=digest(payload))
                work=out/'native-input.json';save(work,job)
                run=subprocess.run([str(BINARY),str(work)],capture_output=True,text=True,timeout=90)
                work.unlink()
                if run.returncode:row.update(status='replay_failed',error=run.stderr[:200])
                else:
                    result=json.loads(run.stdout)
                    if result['payload_sha256']!=job['payload_sha256']:raise ValueError('native input identity mismatch')
                    save(out/('replay-'+j['key']+'.json'),result)
                    row.update(result['result']);row['payload_sha256']=job['payload_sha256'];row['replay_sha256']=sha(out/('replay-'+j['key']+'.json'))
                    checks+=net_verify(row['models']);replayed+=1
        rows.append(row)
        print(json.dumps(dict(evaluated=len(rows),key=j['key'],status=row['status'])),flush=True)
    groups={'overall':rows}
    for key in ['symbol','asset_class','cohort']:
        for value in sorted({r[key] for r in rows}):groups[key+':'+value]=[r for r in rows if r[key]==value]
    summary={group:{m:score(rs,m,'24') for m in MODELS} for group,rs in groups.items()}
    windows={group:{m:latest20(rs,m) for m in MODELS[:2]} for group,rs in groups.items()}
    led=completed(out);fresh=[v for k,v in led.items() if k.startswith('tick-')]
    peers=[r for r in rows if r['symbol'] in PEERS]
    result=dict(trace='№18,338',generated_at_sgt=now(),provenance=json.loads((out/'freeze.json').read_text()),
        dates=DATES,clean_validation=False,previously_observed_dates=DATES[5:],all_8_symbols=SYMBOLS,rows=rows,summary=summary,latest20=windows,
        collection=dict(peer_slots=40,new_tick_attempts=len(fresh),new_tick_complete=sum(v['status']=='complete' for v in fresh),
                        reused_peer_sources=sum(r['mode']=='reuse' for r in peers),missing_primary_slots=25,
                        new_bar_attempts=sum(k.startswith('bar-') for k in led),all_new_component_statuses=dict(collections.Counter(v['status'] for v in led.values()))),
        verification=dict(native_replays=replayed,independent_net_values=checks,prior_source_files=len(p['prior_source_sha256'])),
        primary_costs=p['costs'],financial_pass=False,validation_opened=False,account_goal_73_pct_satisfied=False,
        next_stage_authorized=False,production_changed=False,receipts=events(out/'ledger.jsonl'))
    save(RESULT,result);write_report(result)
    journal('Completed bounded peer run; actual collection: '+json.dumps(result['collection'])+'\nNative replays:'+str(replayed)+'; independent net values:'+str(checks)+'.\nActual overall scores: '+json.dumps(summary['overall'])+'\nNo financial qualification, live activation, Stage2 or holdout opening. Missing25 primary slots and prior2025 exposure retained.', '§§7–10')
    return result


def write_report(r):
    s=r['summary']['overall']['s3_two_stage'];c=r['collection']
    def f(v):return 'unavailable' if v is None else f'{v:.6f}'
    text=f'''# Peer collection and bounded twenty-close milestone evaluation

№18,338 §§1–10 · {r['generated_at_sgt']} · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**Measured S3 two-stage: {s['wins']}/{s['closed_trades']} net wins; mean {f(s['mean_net_r'])}R; PF(R) {f(s['profit_factor'])}.** Twenty closes are a milestone, never an invented denominator. Account goal achieved: **not established**. The invariant is 73% / reference risk:reward1:1.38 on latest20 per strategy/account (15/20=75%). No qualified improvement towards that account goal is demonstrated by this development run.

New peer tick reads: **{c['new_tick_complete']}/{c['new_tick_attempts']} complete**, plus {c['reused_peer_sources']} cached BTC slots. All80 eight-symbol/date slots remain, including **25 missing primary sources** outside the40-peer collection budget. Native replays: {r['verification']['native_replays']}. May2025 oil/US500/BTC dates were already studied; this is not a clean validation batch.

## §2. Corrections to the supplied script

The supplied collector invocation had an unsupported `--symbol` and omitted its required `--output`; that collector also refuses these historical dates. Failed commands became `simulated_ok`, and all trade counts, class rows and controls were hard-coded. Those values were not executed as evidence. Even the invented16/22 is72.727%, below73%, and it does not describe the latest20. Its own win/loss arithmetic gives a different mean/PF from its final block. A1.024R loss is not24bp/side cost accounting: actual cost/R depends on prices and originalR. This replacement uses native fills, retains failures and never authorises Stage2 from a fabricated PASS.

## §3. Measured models at24bp/side

| Model | Entries | Closed | Wins | Net win rate | Mean net R | PF(R) | Censored |
| --- | --- | --- | --- | --- | --- | --- | --- |
'''
    for name,s in r['summary']['overall'].items():
        text+='| '+' | '.join(str(x) for x in [name,s['entries'],s['closed_trades'],s['wins'],f(s['win_rate']),f(s['mean_net_r']),f(s['profit_factor']),s['censored_or_unavailable']])+' |\n'
    text+='\n### Per-class results\n\n| Class | Model | Closed | Wins | Mean R | PF(R) |\n| --- | --- | --- | --- | --- | --- |\n'
    for group,models in r['summary'].items():
        if group.startswith('asset_class:'):
            for name in MODELS[:2]:
                s=models[name];text+=f"| {group.split(':')[1]} | {name} | {s['closed_trades']} | {s['wins']} | {f(s['mean_net_r'])} | {f(s['profit_factor'])} |\n"
    text+='\n## §4. Latest20 research windows\n\n```json\n'+json.dumps(r['latest20']['overall'],indent=2)+'\n```\n'
    text+='''
## §5. Frozen rules and data limits

The owner-selected dates and40 peer slots are fixed before reads. Five cachedBTC slots and15 primary sources are reused by hash;35 new peer tick jobs are permitted. Missing25 primary sources are not silently added to the budget. Completed M30 warm-up is collected separately and bounded. Old oil ticks end11UTC; the locked native oil observation requires07–17UTC, so these sources remain partial and cannot supply afternoon macro execution. Missing bars/quotes block decisions and unclosed positions stay censored. No interpolation, duplicate interval collection, same-day refit or sampling until favourable results.

S3 retains the strict spread+commission+two-sided cost reserve<R at decision/dispatch;48bp alone is not equivalent. The existing exit engine retains−1R until executable+1R, acknowledges an entry-price stop on a strictly later valid quote, and targets gross+1.38R. Broker bracket priority/gaps, native session deadlines and censoring remain. Peer confirmation affects macro only. Authorised pairs are symmetric Crude/Brent, US500/NAS100, Gold/Silver and BTC/ETH; missing peer observations never pass confirmation. S3 exits are tick-replayed; macro signals use six completed M30 bars and causal quotes.

Primary accounting uses actual bid/ask fills plus the assumed24bp per side; zero additional commission remains uncalibrated. Gross-flat exits can be net losses. Historical contract changes, swaps, latency, rejections and account portfolio capacity remain unverified. Current broker-minimum contracts are not proof of historical contract terms. Trade results are R scenarios, not account cash/NLV outcomes.

## §6. Evidence and next step

The [frozen plan](../docs/peer-collection-plan-18338.json) and [measured JSON](../docs/clean_peer_collection_results_18338.json) retain all80 slots, source/replay hashes, per-symbol/class/cohort outcomes, reasons and receipts. Raw sources remain private in the existing workspace; no database data were added. The native executable and source hashes were frozen before reads. No trading-service strategy, risk, worker permission or paid model API changed. Validation19Aug–5Sep2025 stays sealed.

Next: inspect measured failures/censoring and per-class net expectancy. Any proposal to collect the25 missing primaries or choose genuinely unused evaluation dates needs a separately bounded manifest; this batch does not silently expand. Fewer than20 closes is an insufficient-sample result, not a reason to invent trades. No Stage2 expansion or financial PASS follows automatically.
'''
    REPORT.write_text(text)
    page='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Peer evaluation №18,338</title><style>body{max-width:1100px;margin:auto;padding:24px;font:16px/1.55 system-ui;color:#192b3a;background:#fafbfd}pre{white-space:pre-wrap;overflow-wrap:anywhere}a{color:#1450a0}</style><h1>Measured peer evaluation</h1><p>'+html.escape(r['generated_at_sgt'])+'</p><pre>'+html.escape(text)+'</pre></html>'
    REPORT.with_suffix('.html').write_text(page)


def publish(out):
    r=json.loads(RESULT.read_text());paths=CODE+[str(x.relative_to(ROOT)) for x in [PLAN,RESULT,REPORT,REPORT.with_suffix('.html')]]+['log/CODEX_Log.txt']
    staged=set(cmd('git','diff','--cached','--name-only').splitlines())
    if not staged<=set(paths):raise ValueError('unrelated staged changes')
    cmd('git','diff','--check');cmd('git','add','--',*paths)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:cmd('git','commit','-m','research: publish measured peer collection and evaluation (№18,338 §§1–10)')
    head=cmd('git','rev-parse','HEAD')
    if cmd('git','branch','--show-current')!=BRANCH:raise ValueError('publication branch mismatch')
    cmd('git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',BRANCH)
    body=out/'pr-body.md';s=r['summary']['overall']['s3_two_stage']
    body.write_text(f'№18,338 §§1–10 · {now()} · by Codex (GPT-6), effort not exposed.\n\nThe supplied script replaced failed collection with success and hard-coded a profitable20-trade report. This change performs bounded demo data reads and native replay with receipts, cached-source reuse and explicit gaps. Only offline scope/peer mappings expand; entry, exit and cost formulae are retained.\n\nActual collection: '+json.dumps(r['collection'])+f'\n\nActual S3: {s["wins"]}/{s["closed_trades"]} wins, mean {s["mean_net_r"]}R, PF(R) {s["profit_factor"]}. Prior2025 exposure and25 missing primaries remain explicit. No financial PASS, Stage2, live activation or holdout access.\n\nValidation: focused source/receipt/scoring tests, native scope/peer tests, existing exit/cost regressions, independent fill accounting; required check/docker/engineering CI must pass on this head.\n\nReport: `{REPORT.relative_to(ROOT)}`.\n')
    prs=gh('pr','list','--repo','ang-kl/jiaoyi','--head',BRANCH,'--state','open','--json','url')
    url=prs[0]['url'] if prs else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',BRANCH,'--title','Research: real bounded peer collection and evaluation (№18,338 §§1–10)','--body-file',str(body))
    print('REPORT_PR '+url,flush=True);number=url.rsplit('/',1)[-1];deadline=time.monotonic()+1800
    hashes={name:sha(ROOT/name) for name in paths};save(out/'publication.json',dict(url=url,head=head,artifact_sha256=hashes))
    while True:
        pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup')
        if pr['headRefOid']!=head or pr['state']!='OPEN':raise ValueError('PR changed')
        state=ci_state(pr['statusCheckRollup']);print('CI '+state,flush=True)
        if state=='failed':raise ValueError('CI failed; no merge')
        if state=='success':break
        if time.monotonic()>deadline:raise ValueError('CI timeout; no merge')
        time.sleep(15)
    cmd('gh','pr','merge',number,'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,mergeCommit,mergedAt')
    if pr['state']!='MERGED':raise ValueError('merge not confirmed')
    cmd('git','fetch','origin','main')
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('main artifact mismatch')
    block=dict(trace='№18,338',status='bounded_peer_evaluation_completed',pr_url=url,pr_merge_commit=pr['mergeCommit']['oid'],
        frozen_plan_sha256=sha(PLAN),collection=r['collection'],s3_two_stage=s,macro_180m_two_stage=r['summary']['overall']['macro_180m_two_stage'],
        financial_pass=False,account_goal_73_pct_satisfied=False,validation_opened=False,stage2_authorized=False,
        report_file=str(REPORT.relative_to(ROOT)),verified_at_sgt=now())
    save(out/'review-block.json',block)
    update=out/'pr-update.json';save(update,dict(body=body.read_text()+'\nVerified on main:\n```json\n'+json.dumps(block,indent=2)+'\n```\n'))
    cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(update),'--jq','.html_url')
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('action',choices=['prepare','run','publish','all']);ap.add_argument('--output',type=Path,default=Path('/workspace/jiaoyi-peer-18338'));args=ap.parse_args()
    out=args.output;out.mkdir(parents=True,exist_ok=True)
    if args.action in ['prepare','all']:
        p=prepare();print('PLAN '+json.dumps(p['limits']),flush=True)
        if args.action=='prepare':return
        subprocess.run(['cargo','fmt','--all','--check'],cwd=ROOT,check=True)
        subprocess.run(['python3','-m','unittest','discover','-s','ops/research','-p','test_peer_collection_eval.py'],cwd=ROOT,check=True)
        subprocess.run(['cargo','test','-p','jiaoyi-core','research::three_year::tests','--locked'],cwd=ROOT,check=True)
        subprocess.run(['cargo','build','-p','jiaoyi-core','--example','peer_collection_replay','--locked'],cwd=ROOT,check=True)
        paths=CODE+[str(PLAN.relative_to(ROOT)),'log/CODEX_Log.txt']
        if not set(cmd('git','diff','--cached','--name-only').splitlines())<=set(paths):raise ValueError('unrelated staged files')
        cmd('git','add','--',*paths)
        if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:cmd('git','commit','-m','research: freeze bounded peer collection and offline scope (№18,338 §§1–6)')
    if args.action in ['run','all']:
        p=json.loads(PLAN.read_text())
        with (out/'run.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);provenance(out,p);collect(out,p);completed(out);replay(out,p);provenance(out,p)
    if args.action in ['publish','all']:publish(out)


if __name__=='__main__':main()
