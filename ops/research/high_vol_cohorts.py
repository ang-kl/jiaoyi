#!/usr/bin/env python3
"""№18,348: fixed 40-episode collection with verified M30 reuse and native exits."""
import argparse
import collections
import copy
import datetime as dt
import fcntl
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import time

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks
from class_batch import digest, receipt, save
from seasonal_pilot import bar_request, decode_bars, instant, stamp, protected, segments
from run_three_year_pipeline import native_open, window_open
from run_repaired_offline_replay import score, without_notes, ci_state
from jan2024_peer import cumulative
from peer_collection_eval import now, net_verify, latest20

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT/'docs/high_vol_cohorts_plan_18348.json'
RESULT = ROOT/'docs/high_vol_cohorts_results_18348.json'
REPORT = ROOT/'doc/HIGH_VOL_COHORTS_REPORT-18348-2026-11-07.md'
BINARY = ROOT/'target/debug/examples/high_vol_replay'
BRANCH = 'research/high-vol-cohorts-18348'
COHORTS = {'2024_Q4_Election_Vol':[f'2024-11-{d:02}' for d in range(4,9)],
           '2025_Q2_Tariff_Vol':[f'2025-04-{d:02}' for d in range(7,12)]}
SYMBOLS = ['SpotCrude','SpotBrent','US500','XAUUSD']
CLASSES = dict(SpotCrude='energy',SpotBrent='energy',US500='equity_index',XAUUSD='metal')
MODELS = ['s3_two_stage','s3_two_stage_window_close','always_long_control','own_momentum_fixed_horizon']
PRIOR = [('docs/seasonal-execution-18315.json','/workspace/jiaoyi-seasonal-stage1-18316'),
         ('docs/three-year-execution-18324.json','/workspace/jiaoyi-three-year-pilot-18325'),
         ('docs/peer-collection-plan-18338.json','/workspace/jiaoyi-peer-18338'),
         ('docs/jan2024-peer-plan-18342.json','/workspace/jiaoyi-jan2024-peer-18342'),
         ('docs/class-batch-40-plan-18309-2026-11-07.json','/workspace/jiaoyi-class40-evidence-20261107')]
EVIDENCE = ['docs/offline-repaired-engine-results-18331.json','docs/clean_peer_collection_results_18338.json',
            'docs/jan2024_peer_parity_results_18342.json']
CODE = ['ops/research/high_vol_cohorts.py','ops/research/test_high_vol_cohorts.py','ops/research/run_high_vol_cohorts_18348.sh',
        'crates/jiaoyi-core/src/research/offline_repaired.rs','crates/jiaoyi-core/examples/high_vol_replay.rs']


def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(p): return json.loads(Path(p).read_text())
def cmd(*a): return subprocess.check_output(a,cwd=ROOT,text=True,timeout=120).strip()
def gh(*a): return json.loads(cmd('gh',*a))
def events(p): return [json.loads(x) for x in Path(p).read_text().splitlines()]
def overlaps(a,b): return instant(a['from']) < instant(b['to']) and instant(b['from']) < instant(a['to'])


def journal(section,text):
    p=ROOT/'log/CODEX_Log.txt';old=p.read_text();tag='№18,348 '+section
    if tag in old:return
    n=max(map(int,re.findall(r'\[§LOG\.(\d+)\]',old)),default=0)+1
    p.write_text(old+f'\n[§LOG.{n}] {tag} · {now()}; by <Codex (GPT-6), effort: not exposed>.\n'+text+'\n')


def split_bar_window(first,last,prior):
    """Reuse verified overlaps; never re-request a spent failed/uncertain interval."""
    bounds={first,last}
    for p in prior:
        a,b=instant(p['from']),instant(p['to'])
        if a < last and b > first: bounds.update([max(a,first),min(b,last)])
    out=[];ordered=sorted(bounds)
    for a,b in zip(ordered,ordered[1:]):
        covering=[p for p in prior if instant(p['from'])<=a and instant(p['to'])>=b]
        good=[p for p in covering if p.get('cache') and p.get('status')=='complete']
        if good:
            chosen=sorted(good,key=lambda p:p['cache']['path'])[0]
            out.append(dict(mode='reuse',cache=chosen['cache'],**{'from':stamp(a),'to':stamp(b)}))
        elif covering:
            out.append(dict(mode='blocked',reason='spent_source_unavailable',prior_sources=covering,**{'from':stamp(a),'to':stamp(b)}))
        else:
            out.extend(dict(mode='collect',**{'from':stamp(x),'to':stamp(y)}) for x,y in segments(a,b))
    return out


def prepare():
    if PLAN.exists():return read(PLAN)
    sources={};spent_ticks=[];spent_bars=[];prior_hashes={}
    for name,folder in PRIOR:
        path=ROOT/name;p=read(path);prior_hashes[name]=sha(path);folder=Path(folder)
        ledger=folder/'ledger.jsonl';es=events(ledger);sources[str(ledger)]=sha(ledger)
        started={e['key'] for e in es if e['state']=='started'};done={e['key']:e for e in es if e['state']=='finished'}
        for j in p['jobs']:
            if j.get('mode')=='missing':continue
            req=j.get('request',j);end=instant(req['to'])+dt.timedelta(seconds=5,milliseconds=1)
            spent_ticks.append(dict(symbol_id=j['symbol_id'],**{'from':j.get('source_from',req['from']),'to':stamp(end)}))
        for part in p.get('bar_segments',[]):
            key='bar-'+part['key']
            if key not in started:continue
            item=dict(part,status='uncertain');file=folder/(key+'.json');e=done.get(key)
            if e and file.exists():
                h=sha(file)
                if e.get('file_sha256',e.get('sha256'))!=h:raise ValueError('cached receipt mismatch: '+str(file))
                value=read(file);item['status']=value['status']
                if value['status']=='complete':
                    if value['request']['symbol_id']!=part['symbol_id'] or any(value['request'][k]!=part[k] for k in ['from','to']):raise ValueError('cached interval identity mismatch')
                    item['cache']=dict(path=str(file),sha256=h,receipt=e,ledger=str(ledger))
            spent_bars.append(item)
        # January peers use a single M30 interval per tick job, separately recorded in that job.
        if name.endswith('18342.json'):
            for j in p['jobs']:
                if 'peer-'+j['key'] in started:
                    spent_bars.append(dict(j,status='spent_peer_component',**{'from':j['from'],'to':j['to']}))
    meta=read(ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json')['metadata'];jobs=[];parts=[]
    for cohort,dates in COHORTS.items():
        for symbol in SYMBOLS:
            m=meta[symbol];first=native_open(symbol,dates[0])-dt.timedelta(days=45)
            last=window_open(symbol,dates[-1])+dt.timedelta(hours=4)
            relevant=[p for p in spent_bars if p['symbol_id']==m['symbol_id']]
            keys=[]
            for i,part in enumerate(split_bar_window(first,last,relevant)):
                key=f'{cohort}-{symbol}-{i}';keys.append(key)
                part.update(key=key,symbol=symbol,symbol_id=m['symbol_id']);parts.append(part)
                if protected(instant(part['from']),instant(part['to'])):raise ValueError('sealed bar interval')
                if part['mode']=='reuse':sources[part['cache']['path']]=part['cache']['sha256']
            for date in dates:
                a=window_open(symbol,date);b=a+dt.timedelta(hours=4);start=native_open(symbol,date)
                window=dict(symbol_id=m['symbol_id'],**{'from':stamp(start),'to':stamp(b+dt.timedelta(seconds=5,milliseconds=1))})
                if any(x['symbol_id']==m['symbol_id'] and overlaps(window,x) for x in spent_ticks):raise ValueError('spent tick interval: '+symbol+' '+date)
                if protected(start,b+dt.timedelta(seconds=5)) or b>=instant('2025-10-24T00:00:00Z'):raise ValueError('research seal')
                jobs.append(dict(key=symbol+'-'+date,symbol=symbol,symbol_id=m['symbol_id'],cohort=cohort,date=date,
                    asset_class=CLASSES[symbol],source_from=stamp(start),bar_segments=keys,
                    contract={k:m[k] for k in ['digits','min_volume','step_volume']},
                    request=dict(account=ACCOUNT,symbol=symbol,**{'from':stamp(a),'to':stamp(b)},spread=0.,commission_round_trip=0.,
                                 slippage_bps_per_side=24.,tick_replay=dict(volume=m['min_volume'],entry_rule='current_s3'))))
    if len(jobs)!=40 or len({j['key'] for j in jobs})!=40:raise ValueError('wrong fixed scope')
    p=dict(trace='№18,348 §§1–6',frozen_at_sgt=now(),cohorts=COHORTS,symbols=SYMBOLS,jobs=jobs,bar_segments=parts,
           prior_manifests=prior_hashes,source_sha256=sources,evidence_sha256={n:sha(ROOT/n) for n in EVIDENCE},
           collector_mode='interval_only_no_count',tick_windows_unspent=True,clean_validation=False,
           selection='Owner-selected retrospective high-volatility events; M30 overlap reused. Fresh tick reads do not establish untouched or representative validation.',
           models=MODELS,primary_cost_reserve_bps_per_side=24.,admission='Unchanged strict full C<R, including spread; implies R_bps>48 with zero extra commission. Verified on native entries.',
           exit_interpretation='Legacy s3_two_stage retains native session end. Owner-requested four-hour-window close is separately labelled s3_two_stage_window_close; original entry/SL/TP preserved, missing tail quote censored.',
           windows='US500 09:30 New York; gold 07:00 UTC; oil evaluation 13:00–17:00 UTC with native S3 preparation/entry quotes from 07:00 UTC. Controls use their existing four-hour-window rules; S3 oil may enter earlier.',
           limits=dict(logical_jobs=40,attempts_per_component=1,tick_pages_per_job=120,ticks_per_job=200000,seconds_per_job=240,
                       new_bar_requests=sum(x['mode']=='collect' for x in parts),reused_bar_slices=sum(x['mode']=='reuse' for x in parts),
                       blocked_bar_slices=sum(x['mode']=='blocked' for x in parts),bytes_per_file=32000000,total_source_bytes=512000000),
           stopping_rule='Complete each fixed slot once; never extend until target or positive performance.',
           cumulative_rule='Show requested five-close baseline plus fresh legacy closes AND full all-prior deduplicated union, retaining May loss/censoring/protocol flags. Four-hour variant standalone; mixed-rule illustrative comparison labelled.',
           cost_calibration='unverified; spread in executable quotes once; zero additional commission assumption retained',
           validation_opened=False,production_changed=False,financial_pass=False)
    PLAN.write_text(json.dumps(p,indent=2)+'\n')
    journal('§§1–6','Frozen 40 slots and receipt-based M30 partition. '+json.dumps(p['limits'])+'\nLegacy end-of-session model retained; requested four-hour close separately labelled. All prior losses/censoring retained; no financial PASS.')
    return p


def provenance(out,p):
    for name,h in {**p['source_sha256'],**p['prior_manifests'],**p['evidence_sha256']}.items():
        path=Path(name) if name.startswith('/') else ROOT/name
        if sha(path)!=h:raise ValueError('frozen source changed: '+name)
    files=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',PLAN,ROOT/'crates/jiaoyi-core/examples/high_vol_replay.rs']
    files += [ROOT/'ops/research'/n for n in ['high_vol_cohorts.py','test_high_vol_cohorts.py','run_high_vol_cohorts_18348.sh','broker_readonly.py','class_batch.py','seasonal_pilot.py','run_three_year_pipeline.py','run_repaired_offline_replay.py','jan2024_peer.py','peer_collection_eval.py']]
    hashes={str(f.relative_to(ROOT)):sha(f) for f in files}
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit frozen code before reads')
    current=dict(plan_sha256=sha(PLAN),source_sha256=hashes,binary_sha256=sha(BINARY))
    target=out/'freeze.json'
    if target.exists():
        old=read(target)
        if any(old[k]!=v for k,v in current.items()):raise ValueError('frozen execution drift')
        return old
    if (out/'ledger.jsonl').exists():raise ValueError('receipts without freeze')
    current.update(commit=cmd('git','rev-parse','HEAD'),frozen_at_sgt=now());save(target,current);return current


def completed(out):
    ledger=out/'ledger.jsonl';es=events(ledger) if ledger.exists() else []
    starts=[x['key'] for x in es if x['state']=='started'];ends={x['key']:x for x in es if x['state']=='finished'}
    if len(starts)!=len(set(starts)) or len(ends)!=sum(x['state']=='finished' for x in es) or set(starts)!=set(ends):raise ValueError('uncertain receipt; never retry')
    for key,e in ends.items():
        if sha(out/(key+'.json'))!=e['sha256']:raise ValueError('receipt checksum mismatch')
    return ends


def assemble_bars(parts,first,last):
    selected={}
    for part,value in parts:
        if value['status']!='complete':continue
        a,b=int(instant(part['from']).timestamp()*1000),int(instant(part['to']).timestamp()*1000)
        for bar in value['bars']:
            at=bar['ts_ms']
            if not (a<=at<b and first<=at and at+1800000<=last):continue
            if at in selected and selected[at]!=bar:raise ValueError('conflicting cached bar')
            selected[at]=bar
    return [selected[k] for k in sorted(selected)]


def collect(out,p):
    done=completed(out);expected={'job-'+j['key'] for j in p['jobs']}|{'bar-'+s['key'] for s in p['bar_segments'] if s['mode']=='collect'}
    if not set(done)<=expected:raise ValueError('unexpected collection receipt')
    if set(done)==expected:return
    def perform(key,operation):
        file=out/(key+'.json')
        if key in done:return read(file)
        if file.exists():raise ValueError('unreceipted source; no repeat')
        receipt(out/'ledger.jsonl',dict(key=key,state='started',at_sgt=now(),plan_sha256=sha(PLAN)))
        budget=dict(pages=0,rows=0)
        try:value=operation(budget)
        except Exception as error:value=dict(status='failed',error=str(error) if isinstance(error,BrokerError) else type(error).__name__)
        value.update(attempts=1,budget_used=budget)
        size=len(json.dumps(value,separators=(',',':')).encode())
        used=sum(f.stat().st_size for pat in ['job-*.json','bar-*.json'] for f in out.glob(pat))
        if size>32000000 or used+size>p['limits']['total_source_bytes']:raise ValueError('source-byte budget; receipt remains uncertain')
        save(file,value);end=dict(key=key,state='finished',status=value['status'],sha256=sha(file),at_sgt=now())
        receipt(out/'ledger.jsonl',end);done[key]=end
        print(json.dumps(dict(component=key,status=value['status'],error=value.get('error'),pages=budget['pages'])),flush=True)
        if value.get('error') in {'TypeError','KeyError','ValueError','AttributeError','NameError','AssertionError','CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED','INVALID_ACCESS_TOKEN','OA_AUTH_TOKEN_EXPIRED'}:raise ValueError('unexpected/auth failure retained; stop')
        return value
    with Broker() as broker:
        catalog=broker.request(2114,dict(ctidTraderAccountId=ACCOUNT,includeArchivedSymbols=False))
        details=broker.request(2116,dict(ctidTraderAccountId=ACCOUNT,symbolId=sorted({j['symbol_id'] for j in p['jobs']})))
        for symbol in SYMBOLS:
            j=next(x for x in p['jobs'] if x['symbol']==symbol)
            names=[x for x in catalog['symbol'] if x['symbolName']==symbol]
            c=next(x for x in details['symbol'] if x['symbolId']==j['symbol_id'])
            if len(names)!=1 or names[0]['symbolId']!=j['symbol_id'] or any(c[k]!=j['contract'][v] for k,v in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('metadata changed')
        save(out/('metadata-resume.json' if (out/'metadata.json').exists() else 'metadata.json'),dict(at_sgt=now(),details=details))
        available={}
        for part in p['bar_segments']:
            if part['mode']=='reuse':value=read(part['cache']['path'])
            elif part['mode']=='blocked':value=dict(status='failed',error=part['reason'])
            else:
                def fetch(_budget,part=part):
                    raw=broker.request(2137,bar_request(part))
                    try:return dict(status='complete',request=part,bars=decode_bars(raw,part))
                    except BrokerError as error:return dict(status='failed',error=str(error),request=part,raw_response=raw)
                value=perform('bar-'+part['key'],fetch)
            available[part['key']]=(part,value)
        for j in p['jobs']:
            def fetch(budget,j=j):
                req=j['request'];start=instant(j['source_from']);end=instant(req['to']);deadline=time.monotonic()+240
                bids=ticks(broker,j['symbol_id'],1,int(start.timestamp()*1000),int(end.timestamp()*1000)+5000,budget,deadline)
                asks=ticks(broker,j['symbol_id'],2,int(start.timestamp()*1000),int(end.timestamp()*1000)+5000,budget,deadline)
                parts=[available[k] for k in j['bar_segments']]
                bars=assemble_bars(parts,int((start-dt.timedelta(days=45)).timestamp()*1000),int(end.timestamp()*1000))
                payload=dict(quotes=dict(bars=[],bids=bids,asks=asks,contract=j['contract']),m30=bars,
                             source_from=j['source_from'],peer_symbol=None,peer_m30=[])
                return dict(status='complete',request=req,payload=payload,payload_sha256=digest(payload),
                            bar_component_failures=sum(v['status']!='complete' for _,v in parts),
                            bar_sources=[dict(key=part['key'],mode=part['mode'],status=v['status'],
                                sha256=part['cache']['sha256'] if part['mode']=='reuse' else sha(out/('bar-'+part['key']+'.json')) if part['mode']=='collect' else None) for part,v in parts])
            perform('job-'+j['key'],fetch)


def baseline_parity(out):
    old={r['key']:r for r in read(ROOT/EVIDENCE[0])['rows']};checked=[]
    for key in ['SpotCrude-2024-01-22','US500-2023-01-23']:
        path=Path('/workspace/jiaoyi-three-year-pilot-18325')/('job-'+key+'.json')
        if sha(path)!=old[key]['file_sha256']:raise ValueError('baseline source changed')
        result=json.loads(subprocess.check_output([str(BINARY),str(path)],text=True,timeout=90))['result']
        if without_notes(result['models']['s3_two_stage'])!=without_notes(old[key]['models']['s3_two_stage']):raise ValueError('legacy model changed')
        checked.append(dict(key=key,source_sha256=sha(path)))
    save(out/'baseline-parity.json',checked)


def replay(out,p):
    done=completed(out);rows=[];checks=0
    for j in p['jobs']:
        path=out/('job-'+j['key']+'.json');source=read(path)
        row={k:j[k] for k in ['key','symbol','date','cohort','asset_class']}
        row.update(status=source['status'],models={},file_sha256=sha(path),protocol_flags=[])
        if source['status']!='complete':row['error']=source.get('error')
        else:
            proc=subprocess.run([str(BINARY),str(path)],text=True,capture_output=True,timeout=90)
            if proc.returncode:row.update(status='replay_failed',error=proc.stderr[:400])
            else:
                native=json.loads(proc.stdout);result=native['result'];models=result['models']
                # Existing accounting checker verifies actual fill volumes and all cost sensitivities.
                checks+=net_verify(dict(s3_two_stage=models['s3_two_stage'],macro_180m_two_stage=models['s3_two_stage_window_close']))
                for model in ['s3_two_stage','s3_two_stage_window_close']:
                    trade=models[model].get('study',{}).get('trade')
                    if trade and trade['entry']['initial_r']/trade['entry']['fill']*10000<48-1e-8:raise ValueError('48bp admission invariant violated')
                row.update(models=models,quality=result['quality'],window_quality=result['window_quality'],
                           m30_bars=len(source['payload']['m30']),bar_component_failures=source['bar_component_failures'])
                save(out/('replay-'+j['key']+'.json'),native);row['replay_sha256']=sha(out/('replay-'+j['key']+'.json'))
        rows.append(row);print(json.dumps(dict(replayed=len(rows),key=j['key'],status=row['status'])),flush=True)
    groups={'overall':rows}
    for cid in COHORTS:groups[cid]=[r for r in rows if r['cohort']==cid]
    for s in SYMBOLS:groups[s]=[r for r in rows if r['symbol']==s]
    summary={g:{m:score(rs,m,'24') for m in MODELS} for g,rs in groups.items()}
    base,may,jan=[read(ROOT/n)['rows'] for n in EVIDENCE]
    requested=cumulative([('№18,331',base),('№18,348',rows)])
    full=cumulative([('№18,331',base),('№18,338',may),('№18,342',jan),('№18,348',rows)])
    illustrative=copy.deepcopy(rows)
    for row in illustrative:row['models']['s3_two_stage']=row['models'].get('s3_two_stage_window_close',{})
    mixed=cumulative([('№18,331',base),('№18,348 four-hour variant',illustrative)])
    mixed['qualification']='Descriptive requested five-prior-close bridge only; exit rules differ. Not a homogeneous model milestone or account qualification.'
    result=dict(trace='№18,348',generated_at_sgt=now(),plan_sha256=sha(PLAN),provenance=read(out/'freeze.json'),
        rows=rows,summary=summary,requested_baseline_cumulative_legacy=requested,full_cumulative_legacy=full,
        requested_five_prior_plus_four_hour_variant_mixed_rules=mixed,
        new_batch_latest20={m:latest20(rows,m) for m in MODELS[:2]},
        collection=dict(tick_jobs_attempted=len(rows),tick_jobs_complete=sum(read(out/('job-'+j['key']+'.json'))['status']=='complete' for j in p['jobs']),
                        m30_requests=sum(s['mode']=='collect' for s in p['bar_segments']),reused_m30_slices=sum(s['mode']=='reuse' for s in p['bar_segments']),
                        new_component_statuses=dict(collections.Counter(e['status'] for e in done.values())),
                        source_bytes=sum(f.stat().st_size for pat in ['job-*.json','bar-*.json'] for f in out.glob(pat))),
        verification=dict(native_replays=sum(r['status']=='complete' for r in rows),independent_net_values=checks,baseline_parity=read(out/'baseline-parity.json')),
        receipts=done,validation_opened=False,financial_pass=False,production_changed=False,stage2_authorized=False,
        limitations=['Retrospectively selected macro-event weeks; reused M30 values may have been seen. Not untouched/representative validation.',
                     'Bid/ask spread included once; 24bp/side remains an assumption and extra commission is zero/unverified.',
                     'Four-hour exit is a distinct offline variant; earlier session-end results must not be silently mixed.',
                     'Native oil S3 may enter from 07:30 UTC, before its 13:00–17:00 control window. Controls share dates/costs, not necessarily exposure duration.'])
    save(RESULT,result);report(result)
    journal('§§7–12','Completed fixed fresh tick scope; '+json.dumps(result['collection'])+'\nStandalone: '+json.dumps(summary['overall'])+'\nFull legacy cumulative: '+json.dumps(full['summary'])+'\nFour-hour close separately reported. Previous May loss and censored trades retained; no financial PASS or live change.')
    return result


def report(r):
    def fmt(v):return 'unavailable' if v is None else f'{v:.6f}'
    overall=r['summary']['overall'];full=r['full_cumulative_legacy']['summary'];requested=r['requested_baseline_cumulative_legacy']['summary']
    lines=['# Fresh macro-event cohorts: measured native two-stage replay','',
        f'№18,348 §§1–12 · {r["generated_at_sgt"]} · by Codex (GPT-6), effort not exposed.','',
        '## §1. TLDR','',
        f'Completed the fixed **40 episode slots**, with **{r["collection"]["tick_jobs_complete"]} complete tick sources** and **{r["verification"]["native_replays"]} successful native replays**. Results below are measured from broker receipts and compiled Rust paths. No financial PASS, account qualification or production activation is inferred.','',
        '| Model / exit policy | Entries | Closed | Wins | Losses | Censored | Net win rate | Mean net R | PF(R) |',
        '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for m in MODELS:
        s=overall[m];lines.append(f'| {m} | {s["entries"]} | {s["closed_trades"]} | {s["wins"]} | {s["losses"]} | {s["censored_or_unavailable"]} | {fmt(s["win_rate"])} | {fmt(s["mean_net_r"])} | {fmt(s["profit_factor"])} |')
    lines += ['', f'**Full like-for-like legacy milestone: {full["closed_trades"]}/20 unique closes**, {full["wins"]} wins, mean {fmt(full["mean_net_r"])}R, PF(R) {fmt(full["profit_factor"])}. This includes the later May loss and censored entries. The original **73% net win rate / reference 1:1.38 risk:reward on the latest 20 trades per strategy/account** remains unqualified by pooled historical research.','',
        '## §2. Frozen scope and model interpretation','',
        'Election week: 4–8 November 2024. Tariff week: 7–11 April 2025. Symbols: SpotCrude, SpotBrent, US500 and XAUUSD. Every registered prior tick window was checked before reads. Requested M30 intervals were partitioned into unspent reads and verified cached slices. The plan, code and native binary hashes were recorded before broker calls; failures and uncertain receipts cannot silently trigger retries.','',
        'The inherited `s3_two_stage` engine closes at its native session end, which can lie beyond the observation boundary. The owner-requested four-hour-window close is therefore explicitly named `s3_two_stage_window_close`. It uses identical entry, original risk, original stop, +1R next-quote entry-price amendment and gross +1.38R target, but its exit deadline is the evaluation-window end. A valid quote must arrive within the five-second closing tail; otherwise the observation remains censored. Existing broker SL/TP has priority. No synthetic boundary fill is inserted.','',
        'US500 evaluation begins at 09:30 America/New_York; gold begins at 07:00 UTC. Oil evaluation is 13:00–17:00 UTC, with the inherited S3 quote/entry horizon starting at 07:00 UTC. Controls retain their existing four-hour-window entry rules. They match dates, data and costs, but oil S3 can have earlier exposure; this is not a perfectly exposure-matched alpha attribution.','',
        'Primary admission remains strict full C<R (24 bp per side plus prepared spread, zero extra commission assumption), with entry R checked against the 48 bp floor. Bid/ask spread is already reflected in executable fills and is not charged twice. The 24 bp reserve and historical fee/contract terms remain uncalibrated. An entry-price stop is gross-flat, not net breakeven.','',
        '## §3. Cohort and instrument results','',
        '| Group | Model | Closes | Wins | Mean net R | PF(R) |',
        '| --- | --- | ---: | ---: | ---: | ---: |']
    for g in list(COHORTS)+SYMBOLS:
        for m in MODELS[:2]:
            s=r['summary'][g][m];lines.append(f'| {g} | {m} | {s["closed_trades"]} | {s["wins"]} | {fmt(s["mean_net_r"])} | {fmt(s["profit_factor"])} |')
    lines += ['', '## §4. Cumulative accounting and evidence limits','',
        f'The specifically requested №18,331 five-close baseline plus the new **legacy** results gives {requested["wins"]}/{requested["closed_trades"]} wins, mean {fmt(requested["mean_net_r"])}R. It is shown for comparison, not substituted for the complete history. The full cumulative ledger also includes №18,338 and №18,342, deduplicates repeated identities, preserves prior losses and source-protocol flags, and retains censoring. Three of the earlier five baseline closes carry inherited source-protocol flags.','',
        'The JSON also supplies the requested five-prior-close bridge to the new four-hour variant, labelled **mixed exit rules**. It is descriptive only and cannot qualify a homogeneous strategy. New-variant latest-20 metrics are separately reported; no previously censored trade is silently turned into a close.','',
        'These two macro-event weeks were selected retrospectively for volatility. Some M30 values were already present in prior preparation caches. Unspent tick reads therefore do not make this a representative or untouched validation sample. Confidence and transfer to accounts remain unproven even if point estimates exceed a target.','',
        '## §5. Collection, validation and next decision','',
        'Measured collection: `'+json.dumps(r['collection'],sort_keys=True)+'`. Per-slot source status, raw/native hashes, quality counters, censoring and receipts are in the JSON. Raw prices remain outside Git and PostgreSQL.','',
        f'Validation: interval reuse/failure guards, causal bar assembly, native boundary-close/bracket tests, exact unchanged-model parity on two prior sources, and {r["verification"]["independent_net_values"]} independent net-cost checks. The 19 August–5 September 2025 holdout remains sealed, and all requested intervals precede 24 October 2025. No live orders, live strategy changes, paid model calls or automatic Stage 2 expansion occur.','',
        'Next: assess the standalone net outcomes against both controls and across the two cohorts. If entry counts remain low, report the native veto and missing-input reasons instead of weakening the gate to fill a quota. If performance differs under the four-hour deadline, treat that as an exit-policy hypothesis requiring a separately frozen, genuinely untouched evaluation. Calibrate actual execution costs before any deployment decision.','',
        '[Frozen plan](../docs/high_vol_cohorts_plan_18348.json) · [Measured JSON](../docs/high_vol_cohorts_results_18348.json)','']
    REPORT.write_text('\n'.join(lines))


def publish(out):
    if cmd('git','branch','--show-current')!=BRANCH:raise ValueError('wrong publication branch')
    r=read(RESULT);provenance(out,read(PLAN))
    paths=CODE+[str(x.relative_to(ROOT)) for x in [PLAN,RESULT,REPORT]]+['log/CODEX_Log.txt']
    if not set(cmd('git','diff','--name-only','HEAD').splitlines())<=set(paths):raise ValueError('unrelated edits')
    cmd('git','add','--',*paths)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:
        cmd('git','commit','-m','research: measured high-volatility cohorts and complete cumulative ledger (№18,348 §§1–12)')
    head=cmd('git','rev-parse','HEAD');hashes={n:sha(ROOT/n) for n in paths}
    cmd('git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',BRANCH)
    body=out/'pr-body.md'
    body.write_text(f'№18,348 §§1–12 · {now()} · by Codex (GPT-6), effort not exposed.\n\nCollects the fixed 40 election/tariff-week episodes with receipt-verified M30 reuse, then uses native Rust replay. The requested observation-window close is a labelled variant alongside the unchanged session-end model. The complete cumulative ledger retains the later loss, censored entries and earlier protocol flags.\n\nNo fabricated results, financial PASS or live change. Focused interval/censoring/exit tests, two-source native parity and independent fill-cost checks completed.\n\nReport: `{REPORT.relative_to(ROOT)}`.\n')
    prior=gh('pr','list','--repo','ang-kl/jiaoyi','--head',BRANCH,'--state','open','--json','url')
    url=prior[0]['url'] if prior else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',BRANCH,'--title','Research: fresh high-volatility cohorts with native exit comparison (№18,348 §§1–12)','--body-file',str(body))
    print('REPORT_PR '+url,flush=True);number=url.rsplit('/',1)[-1];save(out/'publication.json',dict(url=url,head=head,artifact_sha256=hashes))
    deadline=time.monotonic()+1800
    while True:
        pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup')
        if pr['state']!='OPEN' or pr['headRefOid']!=head:raise ValueError('PR changed')
        state=ci_state(pr['statusCheckRollup']);print('CI '+state,flush=True)
        if state=='failed':raise ValueError('CI failed; no merge')
        if state=='success':break
        if time.monotonic()>deadline:raise ValueError('CI timeout; no merge')
        time.sleep(15)
    cmd('gh','pr','merge',number,'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,mergeCommit')
    if pr['state']!='MERGED':raise ValueError('merge unconfirmed')
    cmd('git','fetch','origin','main')
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('main artifact mismatch')
    block=dict(trace='№18,348',status='high_vol_cohorts_completed',pr_url=url,pr_merge_commit=pr['mergeCommit']['oid'],
        plan_sha256=sha(PLAN),episodes=40,collection=r['collection'],standalone=r['summary']['overall'],
        requested_five_prior_cumulative_legacy=r['requested_baseline_cumulative_legacy']['summary'],
        full_cumulative_legacy=r['full_cumulative_legacy']['summary'],
        four_hour_exit_variant_separate=True,financial_pass=False,validation_opened=False,
        report_file=str(REPORT.relative_to(ROOT)),verified_at_sgt=now())
    save(out/'review-block.json',block);update=out/'pr-update.json'
    save(update,dict(body=body.read_text()+'\nVerified on main:\n```json\n'+json.dumps(block,indent=2)+'\n```\n'))
    cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(update),'--jq','.html_url')
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('action',choices=['prepare','run','publish','all'])
    ap.add_argument('--output',type=Path,default=Path('/workspace/jiaoyi-high-vol-18348'));args=ap.parse_args()
    if cmd('git','branch','--show-current')!=BRANCH:
        if RESULT.exists():print('Already recorded; read '+str(REPORT.relative_to(ROOT))+'. No collection repeated.');return
        if cmd('git','diff','--name-only','HEAD'):raise ValueError('preserve tracked edits')
        cmd('git','fetch','origin','main');cmd('git','switch','-c',BRANCH,'origin/main')
    out=args.output;out.mkdir(parents=True,exist_ok=True)
    if args.action in ['prepare','all']:
        p=prepare();print('PLAN '+json.dumps(p['limits']),flush=True)
        if args.action=='prepare':return
        subprocess.run(['python3','-m','unittest','discover','-s','ops/research','-p','test_high_vol_cohorts.py'],cwd=ROOT,check=True)
        subprocess.run(['cargo','test','-p','jiaoyi-core','research::offline_repaired::tests','--locked'],cwd=ROOT,check=True)
        subprocess.run(['cargo','build','-p','jiaoyi-core','--example','high_vol_replay','--locked'],cwd=ROOT,check=True)
        paths=CODE+[str(PLAN.relative_to(ROOT)),'log/CODEX_Log.txt']
        if not set(cmd('git','diff','--cached','--name-only').splitlines())<=set(paths):raise ValueError('unrelated staged edits')
        cmd('git','add','--',*paths)
        if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:cmd('git','commit','-m','research: freeze 40 high-volatility slots and explicit end-window exit variant (№18,348 §§1–6)')
    if args.action in ['run','all']:
        p=read(PLAN)
        with (out/'run.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);provenance(out,p);baseline_parity(out);collect(out,p);replay(out,p);provenance(out,p)
    if args.action in ['publish','all']:publish(out)


if __name__=='__main__':main()
