"""Frozen 80-episode seasonal pilot. Demo reads only; no retries, orders or validation."""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import json
from pathlib import Path
import subprocess
import time
from zoneinfo import ZoneInfo

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks
from class_batch import digest, receipt, save

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT/'docs/multi-year-seasonal-plan-2024-2025.json'
EXECUTION = ROOT/'docs/seasonal-execution-18315.json'
UTC = dt.timezone.utc
HOLD = (dt.datetime(2025,8,19,tzinfo=UTC), dt.datetime(2025,9,6,tzinfo=UTC))
MODELS = ['production_s3','orb_30m','orb_60m','always_long_control','own_momentum_fixed_horizon']


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def stamp(value): return value.isoformat().replace('+00:00','Z')
def instant(value): return dt.datetime.fromisoformat(value.replace('Z','+00:00'))
def protected(a,b): return a < HOLD[1] and b > HOLD[0]


def opening(symbol,date):
    date=dt.date.fromisoformat(date)
    if symbol=='US500': return dt.datetime.combine(date,dt.time(9,30),ZoneInfo('America/New_York')).astimezone(UTC)
    return dt.datetime.combine(date,dt.time(0 if symbol=='BTCUSD' else 7),UTC)


def segments(first,last):
    allowed=[(first,last)]
    if protected(first,last):
        allowed=[(first,min(last,HOLD[0])),(max(first,HOLD[1]),last)]
    out=[]
    for a,b in allowed:
        while a<b:
            end=min(a+dt.timedelta(days=7),b)
            out.append((a,end));a=end
    return out


def prepare():
    p=json.loads(PLAN.read_text())
    metadata=json.loads((ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json').read_text())['metadata']
    jobs=[];parts=[]
    for cohort in p['stage_1_pilot_cohorts']:
        dates=p['seasonal_cohorts'][cohort]
        for symbol in p['stage_1_pilot_symbols']:
            starts=[opening(symbol,date) for date in dates]
            pieces=[]
            for i,(a,b) in enumerate(segments(min(starts)-dt.timedelta(days=45), max(starts)+dt.timedelta(hours=4))):
                key=f'{cohort}-{symbol}-{i}'
                part=dict(key=key,symbol=symbol,symbol_id=metadata[symbol]['symbol_id'],**{'from':stamp(a),'to':stamp(b)})
                parts.append(part);pieces.append(key)
            for date,start in zip(dates,starts):
                if protected(start,start+dt.timedelta(hours=4,seconds=5)) or start.date()>=dt.date(2025,10,24):raise ValueError('protected/sealed episode')
                req=dict(account=ACCOUNT,symbol=symbol,**{'from':stamp(start),'to':stamp(start+dt.timedelta(hours=4))},
                         spread=0.0,commission_round_trip=0.0,slippage_bps_per_side=24.0,
                         tick_replay=dict(volume=p['native_volume'][symbol],entry_rule='current_s3'))
                jobs.append(dict(key=symbol+'-'+date,cohort=cohort,date=date,
                                 asset_class=next(k for k,v in p['classes'].items() if symbol in v),
                                 symbol_id=metadata[symbol]['symbol_id'],
                                 contract={k:metadata[symbol][k] for k in ('digits','min_volume','step_volume')},
                                 request=req,bar_segments=pieces,
                                 protected_warmup_omitted=protected(start-dt.timedelta(days=45),start)))
    value=dict(trace='№18,315 §§1–8; Codex (GPT-6), effort not exposed',
               prepared_at_utc=stamp(dt.datetime.now(UTC)),plan_path=str(PLAN.relative_to(ROOT)),plan_sha256=sha(PLAN),
               addendum_sha256=sha(ROOT/'docs/seasonal-execution-addendum-18315.md'),
               collection_only_after_plan_pr_merge=True,account=ACCOUNT,models=MODELS,
               cost_sensitivity_bps=[24.0,2.0,3.74],cost_calibration='unverified',
               limits=dict(jobs=80,tick_pages_per_job=120,ticks_per_job=200000,tick_seconds_per_job=240,
                           bar_requests=len(parts),bar_rows_per_segment=336,source_bytes_per_job=32000000,total_source_bytes=256000000),
               validation_opened=False,financial_pass=False,jobs=jobs,bar_segments=parts)
    assert len(jobs)==80 and len({j['key'] for j in jobs})==80
    if EXECUTION.exists():raise ValueError('execution manifest already exists; never rewrite a freeze')
    EXECUTION.write_text(json.dumps(value,indent=2)+'\n')
    print(json.dumps({'jobs':len(jobs),'bar_segments':len(parts),'plan_sha256':sha(PLAN),'execution_sha256':sha(EXECUTION)}))


def load():
    p=json.loads(EXECUTION.read_text())
    if sha(PLAN)!=p['plan_sha256'] or sha(ROOT/'docs/seasonal-execution-addendum-18315.md')!=p['addendum_sha256']:
        raise ValueError('frozen plan/addendum changed')
    if len(p['jobs'])!=80 or len({j['key'] for j in p['jobs']})!=80:raise ValueError('wrong job scope')
    for j in p['jobs']:
        a,b=instant(j['request']['from']),instant(j['request']['to'])
        if b-a!=dt.timedelta(hours=4) or protected(a,b+dt.timedelta(seconds=5)) or b>=dt.datetime(2025,10,24,tzinfo=UTC):raise ValueError('protected/sealed job')
    for s in p['bar_segments']:
        a,b=instant(s['from']),instant(s['to'])
        if not dt.timedelta(0)<b-a<=dt.timedelta(days=7) or protected(a,b) or b>=dt.datetime(2025,10,24,tzinfo=UTC):raise ValueError('protected/sealed warmup')
    return p


def freeze(out,p,binary):
    paths=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',ROOT/'crates/jiaoyi-core/examples/seasonal_replay.rs']
    paths += [ROOT/'ops/research'/x for x in ['broker_readonly.py','class_batch.py','seasonal_pilot.py','seasonal_report.py']]
    hashes={str(f.relative_to(ROOT)):sha(f) for f in sorted(paths)}
    target=out/'freeze.json'
    if target.exists():
        prior=json.loads(target.read_text())
        if prior['execution_sha256']!=sha(EXECUTION) or prior['binary_sha256']!=sha(binary):raise ValueError('manifest/binary changed since freeze')
        if prior['source_sha256']!=hashes:
            amendment_path=ROOT/'docs/seasonal-collector-amendment-18316.json'
            amendment=json.loads(amendment_path.read_text())
            changed={name for name in set(prior['source_sha256'])|set(hashes) if prior['source_sha256'].get(name)!=hashes.get(name)}
            expected=amendment['changed_sources']
            if (sha(target)!=amendment['prior_freeze_sha256'] or amendment['execution_sha256']!=sha(EXECUTION)
                    or changed!=set(expected) or any(expected[name]!={'before':prior['source_sha256'][name],'after':hashes[name]} for name in changed)):
                raise ValueError('unapproved source drift')
            phase=out/'collector-amendment.json'
            value=dict(amendment_sha256=sha(amendment_path),amendment=amendment,source_sha256=hashes,
                       commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
            if phase.exists() and json.loads(phase.read_text())!=value:raise ValueError('collector phase changed')
            save(phase,value)
        return
    if (out/'ledger.jsonl').exists():raise ValueError('cannot reconstruct missing pre-read provenance')
    save(target,dict(frozen_at_utc=stamp(dt.datetime.now(UTC)),commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        source_sha256=hashes,plan_sha256=p['plan_sha256'],execution_sha256=sha(EXECUTION),binary_sha256=sha(binary)))


def bar_request(part):
    # cTrader count mode can supersede fromTimestamp. Use the bounded interval only.
    return dict(ctidTraderAccountId=ACCOUNT,symbolId=part['symbol_id'],period=8,
                fromTimestamp=int(instant(part['from']).timestamp()*1000),
                toTimestamp=int(instant(part['to']).timestamp()*1000)-1)


def decode_bars(result,part):
    # Keep the same acceptance rules, but retain the actual rejected field.
    # A truncation/identity failure is not evidence of a market-session gap.
    if result.get('period') not in (8,'M30'):
        raise BrokerError('bar_period_unverified')
    if result.get('symbolId',part['symbol_id'])!=part['symbol_id']:
        raise BrokerError('bar_symbol_mismatch')
    if result.get('hasMore') is True:
        raise BrokerError('bar_response_truncated')
    if result.get('hasMore') is not False:
        raise BrokerError('bar_completeness_unverified')
    a,b=instant(part['from']),instant(part['to'])
    rows=[];seen=set()
    for raw in result.get('trendbar',[]):
        at=int(raw['utcTimestampInMinutes'])*60000
        low=int(raw['low']); op=low+int(raw.get('deltaOpen',0));hi=low+int(raw.get('deltaHigh',0));cl=low+int(raw.get('deltaClose',0))
        if (at in seen or at%1800000 or not int(a.timestamp()*1000)<=at<int(b.timestamp()*1000)
                or low<=0 or not low<=op<=hi or not low<=cl<=hi or int(raw['volume'])<0):raise BrokerError('invalid_m30')
        seen.add(at)
        rows.append(dict(ts_ms=at,open_e5=op,high_e5=hi,low_e5=low,close_e5=cl,volume=int(raw['volume'])))
    if len(rows)>336:raise BrokerError('bar_row_budget')
    return sorted(rows,key=lambda x:x['ts_ms'])


def events(out):
    path=out/'ledger.jsonl'
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


def collect(out,p,binary):
    out.mkdir(parents=True,exist_ok=True)
    with (out/'collect.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        freeze(out,p,binary)
        save(out/'execution.json',p)
        es=events(out);starts=[e['key'] for e in es if e['state']=='started'];ends={e['key']:e for e in es if e['state']=='finished'}
        if len(starts)!=len(set(starts)) or len(ends)!=sum(e['state']=='finished' for e in es) or set(starts)!=set(ends):raise ValueError('duplicate/uncertain ledger; never retry')
        for key,e in ends.items():
            if sha(out/(key+'.json'))!=e['file_sha256']:raise ValueError('saved source mismatch')
        expected={'job-'+j['key'] for j in p['jobs']}|{'bar-'+s['key'] for s in p['bar_segments']}
        if not set(starts)<=expected:raise ValueError('unexpected receipt')
        part_map={s['key']:s for s in p['bar_segments']}
        def run(key,operation):
            path=out/(key+'.json')
            if key in ends:
                value=json.loads(path.read_text())
                if value['status']=='captured_raw':
                    try:return dict(status='complete',request=value['request'],bars=decode_bars(value['raw_response'],value['request']))
                    except BrokerError as error:return dict(status='failed',error=str(error))
                return value
            if path.exists() or key in starts:raise ValueError('unreceipted/spent source')
            receipt(out/'ledger.jsonl',dict(key=key,state='started',at=stamp(dt.datetime.now(UTC)),execution_sha256=sha(EXECUTION)))
            starts.append(key)
            print('started',key,flush=True)
            try:value=operation()
            except BrokerError as error:value=dict(status='failed',error=str(error))
            except Exception as error:value=dict(status='failed',error=type(error).__name__)
            raw=json.dumps(value,separators=(',',':')).encode()
            total=sum(f.stat().st_size for f in out.glob('job-*.json'))+sum(f.stat().st_size for f in out.glob('bar-*.json'))
            if len(raw)>32000000 or total+len(raw)>256000000:raise ValueError('byte budget; leave unresolved receipt, no retry')
            save(path,value)
            e=dict(key=key,state='finished',status=value['status'],file_sha256=sha(path))
            receipt(out/'ledger.jsonl',e);ends[key]=e
            print('finished',key,value['status'],value.get('error',''),flush=True)
            allowed_failures={'tick_row_budget','page_or_time_budget','tick_decode_failed',
                'unresolvable_tick_truncation','missing_truncation_flag',
                'invalid_m30','bar_row_budget'}
            if value['status']=='failed' and value.get('error') not in allowed_failures:
                raise RuntimeError('transport/auth/unexpected source failure; preserve receipts, no retry')
            return value
        with Broker() as broker:
            symbols={j['request']['symbol']:j for j in p['jobs']}
            catalog=broker.request(2114,dict(ctidTraderAccountId=ACCOUNT,includeArchivedSymbols=False))
            details=broker.request(2116,dict(ctidTraderAccountId=ACCOUNT,symbolId=[j['symbol_id'] for j in symbols.values()]))
            by_name=collections.defaultdict(list)
            for s in catalog['symbol']:by_name[s['symbolName']].append(s)
            by_id={s['symbolId']:s for s in details['symbol']}
            for name,j in symbols.items():
                if len(by_name[name])!=1 or by_name[name][0]['symbolId']!=j['symbol_id'] or not by_name[name][0].get('enabled',True):raise ValueError('symbol identity changed')
                c=by_id[j['symbol_id']]
                if any(int(c[k])!=j['contract'][v] for k,v in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('contract changed')
            metadata=out/('metadata-resume.json' if (out/'metadata.json').exists() else 'metadata.json')
            save(metadata,dict(at=stamp(dt.datetime.now(UTC)),symbols=[by_name[n][0] for n in symbols],details=details))
            for j in p['jobs']:
                key='job-'+j['key']
                if key in ends:continue
                # Bar requests are separately receipted components of these same 80 logical jobs.
                parts=[]
                for name in j['bar_segments']:
                    part=part_map[name]
                    def fetch(part=part):
                        r=broker.request(2137,bar_request(part))
                        try:return dict(status='complete',request=part,bars=decode_bars(r,part))
                        except BrokerError as error:return dict(status='failed',error=str(error),request=part,raw_response=r)
                    parts.append(run('bar-'+name,fetch))
                def fetch_job(j=j,parts=parts):
                    req=j['request'];a=instant(req['from']);b=instant(req['to'])
                    budget=dict(pages=0,rows=0);deadline=time.monotonic()+240
                    bids=ticks(broker,j['symbol_id'],1,int(a.timestamp()*1000),int(b.timestamp()*1000)+5000,budget,deadline)
                    asks=ticks(broker,j['symbol_id'],2,int(a.timestamp()*1000),int(b.timestamp()*1000)+5000,budget,deadline)
                    bars=[row for part in parts if part['status']=='complete' for row in part['bars']
                          if int((a-dt.timedelta(days=45)).timestamp()*1000)<=row['ts_ms'] and row['ts_ms']+1800000<=int(b.timestamp()*1000)]
                    bars.sort(key=lambda x:x['ts_ms'])
                    payload=dict(quotes=dict(bars=[],bids=bids,asks=asks,contract=j['contract']),m30=bars)
                    return dict(status='complete',request=req,payload=payload,payload_sha256=digest(payload),
                                bar_component_failures=sum(x['status']!='complete' for x in parts),budget_used=budget,
                                segment_hashes={name:ends['bar-'+name]['file_sha256'] for name in j['bar_segments']},
                                execution_sha256=sha(EXECUTION))
                run(key,fetch_job)


def replay(out,p,binary):
    frozen=json.loads((out/'freeze.json').read_text())
    if frozen['binary_sha256']!=sha(binary) or frozen['execution_sha256']!=sha(EXECUTION):raise ValueError('replay identity mismatch')
    es=events(out);started=[e['key'] for e in es if e['state']=='started'];finished=[e for e in es if e['state']=='finished']
    ends={e['key']:e for e in finished}
    if len(started)!=len(set(started)) or len(ends)!=len(finished) or set(started)!=set(ends):raise ValueError('uncertain/duplicate receipts')
    if any(e['execution_sha256']!=sha(EXECUTION) for e in es if e['state']=='started'):raise ValueError('receipt manifest mismatch')
    for key,e in ends.items():
        if sha(out/(key+'.json'))!=e['file_sha256']:raise ValueError('source receipt mismatch')
    rows=[]
    for j in p['jobs']:
        key='job-'+j['key'];row={k:j[k] for k in ('key','cohort','date','asset_class','protected_warmup_omitted')}
        row['symbol']=j['request']['symbol'];row['status']='missing'
        if key in ends:
            job=json.loads((out/(key+'.json')).read_text());row['status']=job['status'];row['file_sha256']=ends[key]['file_sha256']
            if job['status']=='complete':
                if job['request']!=j['request'] or job['payload_sha256']!=digest(job['payload']) or job['execution_sha256']!=sha(EXECUTION):raise ValueError('job identity mismatch')
                if job['segment_hashes']!={name:ends['bar-'+name]['file_sha256'] for name in j['bar_segments']}:raise ValueError('warmup identity mismatch')
                proc=subprocess.run([str(binary),str(out/(key+'.json'))],text=True,capture_output=True,timeout=90)
                if proc.returncode:row.update(status='replay_failed',error=proc.stderr[:200])
                else:row.update(replay=json.loads(proc.stdout),budget_used=job['budget_used'],bar_component_failures=job['bar_component_failures'],bar_count=len(job['payload']['m30']))
            else:row['error']=job['error']
        rows.append(row)
    result=dict(trace='№18,316 §§1–8',plan_sha256=p['plan_sha256'],execution_sha256=sha(EXECUTION),provenance=frozen,
                models=MODELS,rows=rows,receipts=es,financial_pass=False,validation_opened=False)
    if (out/'collector-amendment.json').exists():result['collector_amendment']=json.loads((out/'collector-amendment.json').read_text())
    save(out/'replay-results.json',result)
    print(json.dumps(dict(status=dict(collections.Counter(r['status'] for r in rows)),models=len(MODELS))))


if __name__=='__main__':
    a=argparse.ArgumentParser(description=__doc__)
    a.add_argument('action',choices=['prepare','collect','replay']);a.add_argument('--output',type=Path)
    a.add_argument('--binary',type=Path,default=ROOT/'target/debug/examples/seasonal_replay')
    args=a.parse_args()
    if args.action=='prepare':prepare()
    elif args.action=='collect':collect(args.output,load(),args.binary.resolve())
    else:replay(args.output,load(),args.binary.resolve())
