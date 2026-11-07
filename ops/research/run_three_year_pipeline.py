#!/usr/bin/env python3
"""Frozen 120-job real-data pilot; collect/replay/report/publish without invented outcomes."""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import html
import json
from pathlib import Path
import subprocess
import time
from zoneinfo import ZoneInfo

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks
from class_batch import digest, receipt, save
from seasonal_pilot import bar_request, decode_bars, segments, protected, instant, stamp
from seasonal_report import score as base_score, paired, table, fmt, reasons

ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/three-year-seasonal-plan-2023-2025.json'
ADDENDUM=ROOT/'docs/three-year-execution-addendum-18324.md'
MANIFEST=ROOT/'docs/three-year-execution-18324.json'
PRIOR=Path('/workspace/jiaoyi-seasonal-stage1-18316')
MODELS=['production_s3','orb_60m','macro_180m','always_long_control','own_momentum_fixed_horizon']
ARMS={m:['trailing','fixed'] if m in MODELS[:3] else ['fixed_horizon'] for m in MODELS}
BPS=['24','2','3.74'];UTC=dt.timezone.utc
BINARY=ROOT/'target/debug/examples/three_year_replay'
RESULT=ROOT/'docs/three-year-pilot-results-2023-2025.json'
REPORT=ROOT/'doc/THREE_YEAR_PILOT_REPORT-18325-2026-11-07.md'
TRANSIENT={'response_deadline','TimeoutError','REQUEST_FREQUENCY_EXCEEDED','TIMEOUT_ERROR'}
REQUIRED={'check','docker','engineering'}


def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def now():return stamp(dt.datetime.now(UTC))
def sgt():return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')
def cmd(*args):return subprocess.check_output(args,cwd=ROOT,text=True).strip()
def gh(*args):return json.loads(cmd('gh',*args))
def native_open(symbol,date):
    d=dt.date.fromisoformat(date)
    return dt.datetime.combine(d,dt.time(9,30),ZoneInfo('America/New_York')).astimezone(UTC) if symbol=='US500' else dt.datetime.combine(d,dt.time(7),UTC)
def window_open(symbol,date):return native_open(symbol,date)+(dt.timedelta(hours=6) if symbol in ('SpotCrude','SpotBrent') else dt.timedelta())


def prepare():
    if MANIFEST.exists():raise ValueError('never rewrite a frozen manifest')
    p=json.loads(PLAN.read_text());stage=p['execution_stages'][0]
    metadata=json.loads((ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json').read_text())['metadata']
    co={c['id']:c for c in p['cohorts']};jobs=[];parts=[]
    old_events=[json.loads(s) for s in (PRIOR/'ledger.jsonl').read_text().splitlines()]
    old_done={r['key']:r for r in old_events if r['state']=='finished'}
    for cid in stage['cohort_ids']:
        dates=co[cid]['dates']
        for symbol in stage['symbols']:
            reuse=symbol=='US500' and cid in ('2024-Q1-WINTER','2024-Q3-SUMMER')
            keys=[]
            if not reuse:
                for i,(a,b) in enumerate(segments(native_open(symbol,dates[0])-dt.timedelta(days=45),window_open(symbol,dates[-1])+dt.timedelta(hours=4))):
                    key=f'{cid}-{symbol}-{i}';keys.append(key)
                    parts.append(dict(key=key,symbol=symbol,symbol_id=metadata[symbol]['symbol_id'],**{'from':stamp(a),'to':stamp(b)}))
            for date in dates:
                a=window_open(symbol,date);b=a+dt.timedelta(hours=4);m=metadata[symbol]
                j=dict(key=symbol+'-'+date,cohort=cid,date=date,symbol=symbol,asset_class=next(u['asset_class'] for u in p['universe_active'] if u['symbol']==symbol),symbol_id=m['symbol_id'],
                    source_from=stamp(native_open(symbol,date)),contract={k:m[k] for k in ('digits','min_volume','step_volume')},bar_segments=keys,
                    request=dict(account=ACCOUNT,symbol=symbol,**{'from':stamp(a),'to':stamp(b)},spread=0.0,commission_round_trip=0.0,slippage_bps_per_side=24.0,tick_replay=dict(volume=m['min_volume'],entry_rule='current_s3')))
                if reuse:
                    path=PRIOR/('job-'+j['key']+'.json');h=sha(path)
                    if old_done['job-'+j['key']]['file_sha256']!=h:raise ValueError('prior source receipt mismatch')
                    j['reuse']=dict(path=str(path),file_sha256=h,receipt=old_done['job-'+j['key']])
                jobs.append(j)
    manifest=dict(trace='№18,324 §§1–9',prepared_at_utc=now(),plan_sha256=sha(PLAN),addendum_sha256=sha(ADDENDUM),
        prior_ledger_sha256=sha(PRIOR/'ledger.jsonl'),prior_freeze_sha256=sha(PRIOR/'freeze.json'),models=MODELS,jobs=jobs,bar_segments=parts,
        limits=dict(logical_jobs=120,fresh_tick_jobs=110,reused_jobs=10,bar_components=len(parts),pages_per_job=120,updates_per_job=200000,seconds_per_job=240,attempts=3,bytes_per_component=32000000,total_source_bytes=384000000),validation_opened=False)
    MANIFEST.write_text(json.dumps(manifest,indent=2)+'\n');load();print(json.dumps(manifest['limits']))


def load():
    p=json.loads(MANIFEST.read_text())
    if p['plan_sha256']!=sha(PLAN) or p['addendum_sha256']!=sha(ADDENDUM):raise ValueError('plan/addendum changed')
    if len(p['jobs'])!=120 or len({j['key'] for j in p['jobs']})!=120 or len({j['date'] for j in p['jobs']})!=30:raise ValueError('scope mismatch')
    for j in p['jobs']:
        a,b=instant(j['source_from']),instant(j['request']['to'])+dt.timedelta(seconds=5)
        if protected(a,b) or b>=dt.datetime(2025,10,24,tzinfo=UTC) or a.year<2023 or j['request']['account']!=ACCOUNT:raise ValueError('scope/seal/account refused')
        if a!=native_open(j['symbol'],j['date']) or instant(j['request']['from'])!=window_open(j['symbol'],j['date']):raise ValueError('session drift')
    for x in p['bar_segments']:
        a,b=instant(x['from']),instant(x['to'])
        if protected(a,b) or not dt.timedelta(0)<b-a<=dt.timedelta(days=7):raise ValueError('bar bounds')
    return p


def check_ci(checks):
    # Duplicate required names, absent checks and pending/unknown conclusions fail closed.
    relevant=[c for c in checks if c.get('name',c.get('context')) in REQUIRED]
    if len(relevant)!=3 or {c.get('name',c.get('context')) for c in relevant}!=REQUIRED:return 'pending'
    for c in relevant:
        status=c.get('status');end=c.get('conclusion') if status is not None else c.get('state')
        if end in ('FAILURE','ERROR','CANCELLED','TIMED_OUT','ACTION_REQUIRED','STALE','STARTUP_FAILURE'):return 'failed'
        if status is not None and status!='COMPLETED':return 'pending'
        if end!='SUCCESS':return 'pending'
    return 'success'


def sources():
    files=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',ROOT/'crates/jiaoyi-core/examples/three_year_replay.rs']
    files += [ROOT/'ops/research'/n for n in ['run_three_year_pipeline.py','broker_readonly.py','class_batch.py','seasonal_pilot.py','seasonal_report.py']]
    return {str(f.relative_to(ROOT)):sha(f) for f in sorted(files)}


def freeze(out,plan_pr):
    p=gh('pr','view',str(plan_pr),'--repo','ang-kl/jiaoyi','--json','state,mergeCommit,statusCheckRollup')
    if p['state']!='MERGED' or check_ci(p['statusCheckRollup'])!='success':raise ValueError('plan PR must be merged green')
    merged=p['mergeCommit']['oid'];subprocess.run(['git','merge-base','--is-ancestor',merged,'HEAD'],cwd=ROOT,check=True)
    current=dict(plan_sha256=sha(PLAN),manifest_sha256=sha(MANIFEST),source_sha256=sources(),binary_sha256=sha(BINARY),plan_pr=plan_pr,plan_merge_commit=merged)
    f=out/'freeze.json'
    if f.exists():
        old=json.loads(f.read_text())
        if any(old[k]!=v for k,v in current.items()):raise ValueError('frozen provenance changed')
    else:
        if (out/'ledger.jsonl').exists():raise ValueError('missing pre-read freeze')
        save(f,dict(current,at=now(),commit=cmd('git','rev-parse','HEAD')))
    for name,h in current['source_sha256'].items():
        raw=subprocess.check_output(['git','show',merged+':'+name],cwd=ROOT)
        if hashlib.sha256(raw).hexdigest()!=h:raise ValueError('source differs from merged plan')
    return current


def ledger(out):
    f=out/'ledger.jsonl';return [json.loads(s) for s in f.read_text().splitlines()] if f.exists() else []

def verify_ledger(out,p,complete=False):
    es=ledger(out);starts=[x['key'] for x in es if x['state']=='started'];done=[x for x in es if x['state']=='finished'];ends={x['key']:x for x in done}
    if len(starts)!=len(set(starts)) or len(done)!=len(ends) or set(starts)!=set(ends):raise ValueError('uncertain/duplicate receipt; no automatic repeat')
    expected={'job-'+j['key'] for j in p['jobs']}|{'bar-'+s['key'] for s in p['bar_segments']}
    if not set(ends)<=expected or (complete and set(ends)!=expected):raise ValueError('receipt scope/completeness mismatch')
    for key,x in ends.items():
        if sha(out/(key+'.json'))!=x['file_sha256']:raise ValueError('source hash mismatch')
    if any(x.get('manifest_sha256')!=sha(MANIFEST) for x in es if x['state']=='started'):raise ValueError('receipt manifest drift')
    return ends


def reject_spent_bar_reads(plan, prior_plan, prior_events, current_ends):
    """Prior component failures must not disappear behind new cohort/job names."""
    prior_keys={e['key'] for e in prior_events if e['state']=='started'}
    intervals={(x['symbol_id'],x['from'],x['to']) for x in prior_plan['bar_segments'] if 'bar-'+x['key'] in prior_keys}
    repeated=[x['key'] for x in plan['bar_segments'] if 'bar-'+x['key'] not in current_ends and (x['symbol_id'],x['from'],x['to']) in intervals]
    if repeated:
        raise ValueError('prior spent M30 interval requires explicit cached-source lineage; refuse new read: '+','.join(repeated))


def collect(out,p,plan_pr):
    out.mkdir(parents=True,exist_ok=True)
    with (out/'collect.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);freeze(out,plan_pr);ends=verify_ledger(out,p)
        if sha(PRIOR/'ledger.jsonl')!=p['prior_ledger_sha256'] or sha(PRIOR/'freeze.json')!=p['prior_freeze_sha256']:raise ValueError('prior evidence changed')
        reject_spent_bar_reads(p,json.loads((PRIOR/'execution.json').read_text()),ledger(PRIOR),ends)
        def perform(key,operation):
            path=out/(key+'.json')
            if key in ends:return json.loads(path.read_text())
            if path.exists():raise ValueError('unreceipted file')
            receipt(out/'ledger.jsonl',dict(key=key,state='started',at=now(),manifest_sha256=sha(MANIFEST)))
            attempts=[];budget=dict(pages=0,rows=0);deadline=time.monotonic()+240;value=None
            for attempt in range(1,4):
                try:value=operation(budget,deadline);attempts.append(dict(number=attempt,status=value['status']));break
                except Exception as error:
                    reason=str(error) if isinstance(error,BrokerError) else type(error).__name__
                    attempts.append(dict(number=attempt,status='failed',error=reason))
                    value=dict(status='failed',error=reason)
                    if reason not in TRANSIENT or attempt==3 or time.monotonic()+attempt>=deadline:break
                    time.sleep(attempt)
            value.update(attempts=attempts,budget_used=budget)
            raw=json.dumps(value,separators=(',',':')).encode()
            used=sum(f.stat().st_size for pattern in ['job-*.json','bar-*.json'] for f in out.glob(pattern))
            if len(raw)>32000000 or used+len(raw)>384000000:raise ValueError('source byte budget; uncertain receipt retained')
            save(path,value);e=dict(key=key,state='finished',status=value['status'],file_sha256=sha(path));receipt(out/'ledger.jsonl',e);ends[key]=e
            print(json.dumps(dict(key=key,status=value['status'],error=value.get('error'),attempts=len(attempts))),flush=True)
            if value.get('error') in {'TypeError','KeyError','AttributeError','NameError','AssertionError','ValueError','bar_identity_or_truncation_unverified'}:
                raise RuntimeError('unexpected collection defect; receipt retained; stop before spending more jobs')
            if value.get('error') in {'CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','INVALID_ACCESS_TOKEN','NOT_AUTHORIZED','OA_AUTH_TOKEN_EXPIRED'}:raise ValueError('authentication failed; receipt retained')
            return value
        with Broker() as broker:
            symbols={j['symbol']:j for j in p['jobs']}
            catalog=broker.request(2114,dict(ctidTraderAccountId=ACCOUNT,includeArchivedSymbols=False))
            details=broker.request(2116,dict(ctidTraderAccountId=ACCOUNT,symbolId=[j['symbol_id'] for j in symbols.values()]))
            for name,j in symbols.items():
                names=[s for s in catalog['symbol'] if s['symbolName']==name]
                if len(names)!=1 or names[0]['symbolId']!=j['symbol_id']:raise ValueError('symbol identity changed')
                c=next(s for s in details['symbol'] if s['symbolId']==j['symbol_id'])
                if any(c[k]!=j['contract'][v] for k,v in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('contract drift')
            metadata=out/('metadata-resume.json' if (out/'metadata.json').exists() else 'metadata.json');save(metadata,dict(at=now(),details=details))
            components={}
            for part in p['bar_segments']:
                def fetch(_,__,part=part):
                    raw=broker.request(2137,bar_request(part))
                    try:return dict(status='complete',request=part,bars=decode_bars(raw,part))
                    except BrokerError as error:return dict(status='failed',error=str(error),request=part,raw_response=raw)
                components[part['key']]=perform('bar-'+part['key'],fetch)
            for j in p['jobs']:
                def fetch_job(budget,deadline,j=j):
                    req=j['request'];a=instant(j['source_from']);b=instant(req['to']);reuse=j.get('reuse')
                    part_names=list(j['bar_segments']);old_flags=0
                    if reuse:
                        f=Path(reuse['path'])
                        if sha(f)!=reuse['file_sha256']:raise ValueError('prior job hash drift')
                        old=json.loads(f.read_text())
                        if old['status']!='complete':return dict(status='failed',error='retained_prior_failed_job',prior_source=reuse)
                        if old['request']!=req or digest(old['payload'])!=old['payload_sha256']:raise ValueError('prior payload identity changed')
                        source=old['payload']['quotes'];bars=old['payload']['m30'];old_flags=old['bar_component_failures']
                    else:
                        bids=ticks(broker,j['symbol_id'],1,int(a.timestamp()*1000),int(b.timestamp()*1000)+5000,budget,deadline)
                        asks=ticks(broker,j['symbol_id'],2,int(a.timestamp()*1000),int(b.timestamp()*1000)+5000,budget,deadline)
                        source=dict(bars=[],bids=bids,asks=asks,contract=j['contract'])
                        bars=sorted([x for n in part_names if components[n]['status']=='complete' for x in components[n]['bars'] if int((a-dt.timedelta(days=45)).timestamp()*1000)<=x['ts_ms'] and x['ts_ms']+1800000<=int(b.timestamp()*1000)],key=lambda x:x['ts_ms'])
                    peer={'SpotCrude':'SpotBrent','SpotBrent':'SpotCrude'}.get(j['symbol']);peer_bars=[]
                    if peer:
                        peer_job=next(x for x in p['jobs'] if x['cohort']==j['cohort'] and x['symbol']==peer and x['date']==j['date'])
                        part_names+=peer_job['bar_segments'];start=int(instant(req['from']).timestamp()*1000)
                        peer_bars=sorted([x for n in peer_job['bar_segments'] if components[n]['status']=='complete' for x in components[n]['bars'] if start<=x['ts_ms'] and x['ts_ms']+1800000<=start+10800000],key=lambda x:x['ts_ms'])
                    payload=dict(quotes=source,m30=bars,source_from=j['source_from'],peer_symbol=peer,peer_m30=peer_bars)
                    flags=sum(components[n]['status']!='complete' or any(t['status']=='failed' for t in components[n]['attempts']) for n in set(part_names))+old_flags
                    return dict(status='complete',request=req,payload=payload,payload_sha256=digest(payload),bar_component_failures=flags,
                        segment_hashes={n:ends['bar-'+n]['file_sha256'] for n in sorted(set(part_names))},prior_source=reuse,manifest_sha256=sha(MANIFEST))
                perform('job-'+j['key'],fetch_job)
        verify_ledger(out,p,True)


def replay(out,p):
    frozen=json.loads((out/'freeze.json').read_text());ends=verify_ledger(out,p,True)
    if frozen['binary_sha256']!=sha(BINARY) or frozen['source_sha256']!=sources():raise ValueError('replay code/binary changed')
    rows=[]
    for j in p['jobs']:
        key='job-'+j['key'];path=out/(key+'.json');raw=json.loads(path.read_text());row={k:j[k] for k in ['key','symbol','date','cohort','asset_class']}
        row.update(status=raw['status'],file_sha256=ends[key]['file_sha256'],reused='reuse' in j,attempts=raw['attempts'])
        if raw['status']=='complete':
            if raw['request']!=j['request'] or raw['payload_sha256']!=digest(raw['payload']) or raw['manifest_sha256']!=sha(MANIFEST):raise ValueError('replay source mismatch')
            for name,h in raw['segment_hashes'].items():
                if h!=ends['bar-'+name]['file_sha256']:raise ValueError('bar lineage mismatch')
            proc=subprocess.run([str(BINARY),str(path)],capture_output=True,text=True,timeout=90)
            if proc.returncode:row.update(status='replay_failed',error=proc.stderr[:250])
            else:row.update(replay=json.loads(proc.stdout),bar_component_failures=raw['bar_component_failures']+sum(a['status']=='failed' for a in raw['attempts']),budget_used=raw['budget_used'])
        else:row['error']=raw['error']
        rows.append(row)
    result=dict(trace='№18,325',plan_sha256=sha(PLAN),manifest_sha256=sha(MANIFEST),provenance=frozen,receipts=ledger(out),rows=rows,models=MODELS,financial_pass=False,validation_opened=False)
    save(out/'replay-results.json',result);print('replayed',dict(collections.Counter(r['status'] for r in rows)),flush=True)


def score(rows,name,arm,bps,min_closed):
    s=base_score(rows,name,arm,bps,min_closed)
    if s['mean_r'] is None or s['mean_r']<=.05:s['rejection_reasons'].append('mean_not_above_0_05')
    if s['pf'] is None or s['pf']<=1.25:s['rejection_reasons'].append('pf_not_above_1_25_or_unavailable')
    s['numerical_screen_pass']=not s['rejection_reasons'];return s


def summarize(rows):
    if len(rows)!=120 or len({r['key'] for r in rows})!=120:raise ValueError('missing/duplicate denominator')
    groups={'overall':rows}
    for field in ('symbol','asset_class','cohort'):
        for v in sorted({r[field] for r in rows}):groups[field+':'+v]=[r for r in rows if r[field]==v]
    scores={k:{m:{a:{b:score(rs,m,a,b,10 if k.startswith('symbol:') else 20) for b in BPS} for a in ARMS[m]} for m in MODELS} for k,rs in groups.items()}
    by_year={m:{str(y):score([r for r in rows if r['date'].startswith(str(y))],m,'trailing','24',10) for y in (2023,2024,2025)} for m in MODELS[:3]}
    comparisons={m:[paired(rows,m,c) for c in MODELS[3:]] for m in MODELS[:3]}
    gates={};stops={}
    for m in MODELS[:3]:
        for c in comparisons[m]:c['bonferroni_p']=min(1,6*c['one_sided_weekly_sign_flip_p']) if c['one_sided_weekly_sign_flip_p'] is not None else None
        failures=[k for k in groups if k.startswith(('symbol:','asset_class:')) and not scores[k][m]['trailing']['24']['numerical_screen_pass']]
        if any(x['mean_r'] is None or x['mean_r']<=0 for x in by_year[m].values()):failures.append('positive_each_year_unproven')
        if any(c['matched_weeks']<4 or c['bonferroni_p'] is None or c['bonferroni_p']>.05 for c in comparisons[m]):failures.append('matched_control_advantage_unproven')
        gates[m]=dict(passes_development_screen=not failures,reasons=failures,financial_pass=False)
        stops[m]=all(by_year[m][y]['mean_r'] is not None and by_year[m][y]['mean_r']<=0 for y in ['2023','2024'])
    return dict(scores=scores,by_year=by_year,matched_controls=comparisons,development_gates=gates,stop_tuning_nonpositive_2023_2024=stops)


def independent_verify(out,p,result,summary):
    ends=verify_ledger(out,p,True)
    if result['receipts']!=ledger(out) or result['provenance']!=json.loads((out/'freeze.json').read_text()):raise ValueError('provenance changed')
    if {r['key'] for r in result['rows']}!={j['key'] for j in p['jobs']}:raise ValueError('job coverage')
    ticks_count=pages=0
    for row in result['rows']:
        key='job-'+row['key'];raw=json.loads((out/(key+'.json')).read_text())
        if ends[key]['file_sha256']!=row['file_sha256']:raise ValueError('row hash')
        if row['status']!='complete':continue
        if set(row['replay']['result']['models'])!=set(MODELS):raise ValueError('model coverage')
        for side in ['bids','asks']:
            ts=raw['payload']['quotes'][side];ticks_count+=len(ts)
            if any(protected(dt.datetime.fromtimestamp(t['ts_ms']/1000,UTC),dt.datetime.fromtimestamp((t['ts_ms']+1)/1000,UTC)) for t in ts):raise ValueError('held quote')
        pages+=raw['budget_used']['pages']
    for group,models in summary['scores'].items():
        rs=result['rows'] if group=='overall' else [r for r in result['rows'] if str(r[group.split(':',1)[0]])==group.split(':',1)[1]]
        for m,arms in models.items():
            for arm,costs in arms.items():
                for b,s in costs.items():
                    values=[]
                    for row in rs:
                        model=row.get('replay',{}).get('result',{}).get('models',{}).get(m,{})
                        value=(model.get('sensitivities',{}).get(arm,{}) if arm!='fixed_horizon' else (model.get('control') or {}).get('net_by_bps',{})).get(b)
                        if value is not None:values.append(value)
                    if s['closed']!=len(values) or s['wins']!=sum(v>0 for v in values):raise ValueError('score count mismatch')
                    if values and abs(s['mean_r']-sum(values)/len(values))>1e-10:raise ValueError('mean arithmetic mismatch')
    return dict(verified_at=now(),receipt_pairs=len(ends),quote_updates=ticks_count,fresh_tick_pages=pages,
        checks=['all source receipts/hashes','all 120 episode identities','all five model outputs per replay','protected quote exclusion','independent score counts/wins/means'],financial_pass=False)


def report(out,p):
    result=json.loads((out/'replay-results.json').read_text());rows=result['rows'];summary=summarize(rows)
    result['verification']=independent_verify(out,p,result,summary);result['summary']=summary;result['generated_at_sgt']=sgt()
    for row in rows:
        for m in row.get('replay',{}).get('result',{}).get('models',{}).values():m.get('study',{}).pop('notes',None)
    actual=collections.Counter(r['status'] for r in rows)
    overall={m:{'closed_trades':s['closed'],'wins':s['wins'],'win_rate':s['win_rate'],'mean_net_r':s['mean_r'],'profit_factor':s['pf']} for m in MODELS for s in [summary['scores']['overall'][m][ARMS[m][0]]['24']]}
    gaps=sum(any(score([r],m,ARMS[m][0],'24',1)['data_gap_or_flag_episodes'] for m in MODELS) for r in rows)
    block=dict(trace='№18,325',plan_sha256=sha(PLAN),pr_merge_commit=None,total_jobs_attempted=120,total_jobs_completed=actual['complete'],
        source_statuses=dict(actual),reused_jobs=sum(r['reused'] for r in rows),overall_summary=overall,data_gap_count=gaps,
        data_gap_definition='Episodes with source/preparation/retry/sequence/censoring flags; not a count of lost ticks.',report_file=str(REPORT.relative_to(ROOT)),financial_pass=False,validation_opened=False)
    result['review_block']=block;RESULT.write_text(json.dumps(result,separators=(',',':'),allow_nan=False)+'\n');save(out/'review-block.json',block)
    header=['Model','Exit','bp/side','Entries','Closed','Wins','Win rate','Mean R','PF(R)','Censored','Flagged episodes']
    cells=[]
    for m in MODELS:
        for arm in ARMS[m]:
            for b,s in summary['scores']['overall'][m][arm].items():cells.append([m,arm,b,s['entries'],s['closed'],s['wins'],fmt(s['win_rate']),fmt(s['mean_r']),fmt(s['pf']),s['censored_or_invalid'],s['data_gap_or_flag_episodes']])
    def mdtable(h,rs):return '| '+' | '.join(h)+' |\n| '+' | '.join('---' for _ in h)+' |\n'+''.join('| '+' | '.join(map(str,r))+' |\n' for r in rs)
    details=[]
    for group,model_scores in summary['scores'].items():
        if group=='overall':continue
        for m in MODELS:
            s=model_scores[m][ARMS[m][0]]['24'];details.append([group,m,s['episodes'],s['entries'],s['closed'],s['wins'],fmt(s['mean_r']),json.dumps(s['reasons'],sort_keys=True)])
    dh=['Group','Model','Episodes','Entries','Closed','Wins','Mean R','Reasons (overlap)']
    narrative=f'''# Three-year seasonal pilot — real-data review

№18,325 §§1–8 · {sgt()} · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**120 logical episodes accounted for; {actual['complete']} successful native replays; {120-actual['complete']} failed or unavailable.**
{gaps}/120 have at least one source, preparation, retry, sequencing or censoring flag.
No candidate receives a financial PASS. The interim 50% / positive-economics target and
original 73% net wins / reference R:R 1:1.38 over latest20 per strategy/account remain unproven.
This is a 30-date, six-week pilot, not three complete years or proof of seasonal causality.
No live change, validation opening, automatic Stage2, Telegram or model API call.

The supplied runner fabricated outcomes using symbols/job numbers and fixed PF/control values.
It was replaced before execution. No manufactured results were used as market evidence.
Full method: [frozen execution addendum](../docs/three-year-execution-addendum-18324.md).

## §2. What was tested and retained

Four symbols: SpotCrude, SpotBrent, US500 and XAUUSD; winter/summer weeks in 2023–2025.
Ten prior US500 sources were reused by hash, including the prior failed job; missing prior
M30 remains missing. New sources use read-only broker calls and append-only receipts.
All failed, vetoed, missing, censored and retried episodes remain in the 120 denominator.
Closed win rate uses all closes including breakevens. Missing metrics are null, not zero returns.

Native S3 runs from registered open (07 UTC oils/gold, DST-aware09:30 New York US500).
Oil quotes include07–17 UTC to reconstruct S3 state; ORB, macro and controls use13–17 UTC.
S3 retains its actual session close; open trades at the observation boundary are censored.
Others close at hour four. Gold/US500 Macro-180 lack an authorised class peer and are
explicitly unevaluable; no empty-peer “unanimity” or synthetic macro information is invented.
The Macro-180 rule is a newly frozen price-momentum hypothesis, not a proven macro strategy.

ORB-60 exact-prior20 range expansion and Macro-180 use the causal dynamic range filter
R > max(2.5×spread,0.15×ATR14), plus unchanged primary24bp/side cost admission.
2/3.74bp sensitivities reprice identical admitted fills; they do not create new entries
and are not calibrated costs. Shared native Q1.38 targets, SL/TP and original-R trail;
zero extra fees/current contracts/assumed next-quote execution limit account parity.
Bid/ask sequence flags do not by themselves prove missing ticks or illiquidity.

## §3. Overall scenarios — do not pool models or exits

'''+mdtable(header,cells)+'\n## §4. Symbol, class and cohort primary results\n\n'+mdtable(dh,details)+'''
## §5. Matched controls and year evidence

Same-symbol/date closed pairs only, with unmatched episodes preserved. S3 and control
windows/exit policies differ, so paired deltas do not isolate a directional cause.
Six-week sign-flip tests have limited resolution; Bonferroni adjustment for six candidate
comparisons is reported. No threshold is relaxed after seeing outcomes.

'''+json.dumps({m:[{k:v for k,v in c.items() if k!='pairs'} for c in cs] for m,cs in summary['matched_controls'].items()},indent=2)+'\n\nYear means/closes: `'+json.dumps({m:{y:{k:v[k] for k in ['closed','wins','mean_r']} for y,v in years.items()} for m,years in summary['by_year'].items()})+'`.\n\n'+'''## §6. Failure prevention and next decision

1. Preserve evidence and keep engineering PASS separate from economic qualification.
2. Diagnose actual reasons: missing prior bars/peers, cost veto, lack of signal, and open
   censored positions are different. Never label them losing closes or fabricate returns.
3. Independently calibrate costs, historical quantities and execution assumptions before a
   replacement-cost trial. A lower-cost winning subset does not rescue primary failure.
4. Respect the frozen stopping rule; do not tune to a target hit rate. A new hypothesis needs
   an independently justified mechanism, a new freeze and sufficient untouched development
   dates/weeks. Prior reused dates are development evidence, not fresh out-of-sample evidence.
5. Keep validation sealed and Stage2 inactive. Only verified evidence can justify another run.

'''+f"Stopping flags (nonpositive completed means in both2023/2024): `{json.dumps(summary['stop_tuning_nonpositive_2023_2024'])}`. Missing evidence is inconclusive.\n\n"+f'''## §7. Reproducibility and verification

Plan SHA256: `{sha(PLAN)}`. Manifest SHA256: `{sha(MANIFEST)}`.
Plan merge: `{result['provenance']['plan_merge_commit']}`.
Binary SHA256: `{result['provenance']['binary_sha256']}`.
Pre-read freeze: `{result['provenance']['at']}`.
Verification: `{json.dumps(result['verification'])}`.
Raw sources: `{out}`; retained workspace corpus, not permanent archival storage.
Repository JSON includes all native outcomes, receipts, source hashes and grouped scores;
raw quotes are not copied into Git. Reproduction requires that corpus and frozen binary.
Report publication uses explicit successful required CI and verifies main-file hashes.

## §8. Independent AI review

Read this report, the addendum, source plan and [full JSON](../docs/three-year-pilot-results-2023-2025.json).
Challenge chronology, reused/failed evidence, actual costs, per-class/individual symbol
sample size, peer availability, tick ordering and matched-control selection. Do not
repeat the source prompt's fixed winning numbers or claim a full backtest was completed
where evidence is missing. Propose the smallest evidence-backed next action.
'''
    REPORT.write_text(narrative)
    page='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Three-year pilot №18,325</title><style>body{font:16px/1.55 system-ui;max-width:1300px;margin:auto;padding:24px;background:#f7f9fc;color:#172738}h1,h2{line-height:1.2}.notice{padding:18px;background:#fff0c4}.scroll{overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}th,td{padding:8px;border-bottom:1px solid #cbd5e1;text-align:left;vertical-align:top}th{background:#e6edf5}pre{white-space:pre-wrap;overflow-wrap:anywhere}details{padding:16px;background:white;margin:18px 0;border:1px solid #dbe4ef}summary{cursor:pointer}a{color:#1450a0}@media(max-width:600px){body{padding:14px}h1{font-size:27px}}</style><h1>Three-year seasonal pilot</h1><p>№18,325 §§1–8 · '+html.escape(sgt())+' · by Codex (GPT-6), effort not exposed.</p><p class="notice"><b>§1. TLDR:</b> '+str(actual['complete'])+'/120 native replays; '+str(120-actual['complete'])+' failed/unavailable; '+str(gaps)+'/120 flagged episodes. No financial qualification. Interim50% and original73% / R:R1:1.38 latest20 per strategy/account remain unproven.</p><p>The supplied runner hard-coded wins and PF. This report uses real broker sources and native replay. Costs2/3.74bp are uncalibrated sensitivities on identical fills. Ten US500 episodes reuse prior evidence, including a failure. Gold/US500 Macro-180 cannot run without an authorised class peer. S3 oil needs earlier07UTC state; comparisons have different horizons.</p><h2>§3. Overall cost and exit scenarios</h2>'+table(header,cells)+'<details><summary>§4. All symbol/class/cohort results</summary>'+table(dh,details)+'</details><h2>§6–8. Decision and review</h2><p>Keep failed evidence. Verify costs, calendar coverage, peer requirements and quote ordering before any new frozen experiment. No automatic Stage2 or validation.</p><p><a href="'+REPORT.name+'">Complete assessment, methods and verification</a> · <a href="../docs/three-year-pilot-results-2023-2025.json">Native JSON evidence</a></p></html>'
    REPORT.with_suffix('.html').write_text(page)
    log=ROOT/'log/CODEX_Log.txt';marker='[§LOG.123] №18,325'
    if marker not in log.read_text():
        with log.open('a') as f:f.write(f'\n{marker} §§1–8 · {sgt()}; by <Codex (GPT-6), effort: not exposed>.\nExecuted frozen three-year120-job pilot; statuses {dict(actual)}; reused10 prior US500 episodes including failure.\nActual primary outcomes: {json.dumps(overall)}.\nFlagged episodes {gaps}/120. Plan SHA {sha(PLAN)}. Binary SHA {result["provenance"]["binary_sha256"]}.\nNo fabricated win/PF values, Stage2, validation, live change, Telegram or model API.\nAll source receipts/hashes, episode scope and aggregate counts/means verified; raw corpus outside Git.\nReport Markdown/HTML/JSON include limitations, by-year/class/symbol/cohort results and matched controls.\nCI/main-publication receipt belongs to report PR. Time is Money; prepare to win.\n')
    print('report generated',json.dumps(block),flush=True)


def publish(out):
    artifacts=[str(REPORT.relative_to(ROOT)),str(REPORT.with_suffix('.html').relative_to(ROOT)),str(RESULT.relative_to(ROOT)),'log/CODEX_Log.txt']
    h={f:sha(ROOT/f) for f in artifacts};branch='report/three-year-pilot-results-18325'
    if cmd('git','branch','--show-current')!=branch:cmd('git','switch','-c',branch)
    subprocess.run(['git','add',*artifacts],cwd=ROOT,check=True)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:
        subprocess.run(['git','commit','-m','research: publish measured three-year pilot and retained failures (№18,325 §§1–8)'],cwd=ROOT,check=True)
    subprocess.run(['git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',branch],cwd=ROOT,check=True)
    body=out/'report-pr-body.md';block=json.loads((out/'review-block.json').read_text())
    body.write_text(f'№18,325 §§1–8 · {sgt()} · by Codex (GPT-6), effort not exposed.\n\nPublish real broker/native replay evidence for all120 scheduled slots: {block["total_jobs_completed"]} successful replays, {120-block["total_jobs_completed"]} failed/unavailable; preserve all denominators and original sources. No fabricated outcomes or assumed financial PASS.\n\nPrimary summaries:\n```json\n'+json.dumps(block['overall_summary'],indent=2)+'\n```\n\nMarkdown/HTML/JSON and journal include scope corrections, source/receipt hashes, matched controls and independent arithmetic verification. No validation, Stage2 or live changes. Required CI must pass before merge.\n')
    prs=gh('pr','list','--repo','ang-kl/jiaoyi','--head',branch,'--state','all','--json','number,url,state')
    url=prs[0]['url'] if prs else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',branch,'--title','Research: measured three-year pilot results (№18,325 §§1–8)','--body-file',str(body))
    number=int(url.rsplit('/',1)[1]);head=cmd('git','rev-parse','HEAD');save(out/'publication.json',dict(url=url,number=number,head=head,artifact_sha256=h));print('REPORT_PR',url,flush=True)
    deadline=time.monotonic()+1800
    while True:
        pr=gh('pr','view',str(number),'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup,mergeCommit,mergedAt')
        if pr['headRefOid']!=head:raise ValueError('PR head changed')
        state=check_ci(pr['statusCheckRollup']);print('CI',state,flush=True)
        if state=='failed':raise ValueError('required CI failed; inspect and repair, never bypass')
        if state=='success':break
        if time.monotonic()>deadline:raise ValueError('CI deadline; do not treat timeout as success')
        time.sleep(10)
    if pr['state']!='MERGED':cmd('gh','pr','merge',str(number),'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr=gh('pr','view',str(number),'--repo','ang-kl/jiaoyi','--json','state,mergeCommit,mergedAt')
    if pr['state']!='MERGED':raise ValueError('merge not confirmed')
    cmd('git','fetch','origin','main')
    for f,wanted in h.items():
        actual=hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+f],cwd=ROOT)).hexdigest()
        if actual!=wanted:raise ValueError('main artifact hash mismatch')
    block['pr_merge_commit']=pr['mergeCommit']['oid'];save(out/'review-block.json',block)
    body.write_text(body.read_text()+f'\nMerged green at {pr["mergedAt"]}; merge {block["pr_merge_commit"]}. Verified all four artifact hashes on main.\n')
    update=out/'pr-body-update.json';save(update,{'body':body.read_text()})
    cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(update),'--jq','.html_url')
    save(out/'publication.json',dict(url=url,number=number,head=head,merge=pr,artifact_sha256=h))
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('action',choices=['prepare','run','replay','report','publish']);ap.add_argument('--output',type=Path);ap.add_argument('--plan-pr',type=int);ap.add_argument('--publish',action='store_true');args=ap.parse_args()
    if args.action=='prepare':prepare()
    else:
        p=load()
        if args.action=='run':collect(args.output,p,args.plan_pr)
        if args.action in ['run','replay']:replay(args.output,p)
        if args.action in ['run','replay','report']:report(args.output,p)
        if args.action=='publish' or args.publish:publish(args.output)
