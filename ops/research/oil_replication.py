#!/usr/bin/env python3
"""Frozen oil collection and native full-exit replay; no orders or promotion."""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import html
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

import broker_backtest as client
import full_exit_research as replay
import market_collection
from research_gates import DATA_GAP_REASONS

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / 'docs/oil-replication-v2-2026-11-06.json'
PLAN_SHA = 'd35c0ff1b6379d305ee4f6294fe362a3e849121a618280c0ed269cb4d6efedfb'
BASE = 'https://jiaoyi.up.railway.app'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_plan(path=PLAN):
    if digest(path) != PLAN_SHA:
        raise ValueError('frozen oil amendment changed')
    plan = json.loads(path.read_text())
    for name, expected in plan['reference_code_sha256'].items():
        if digest(ROOT/name) != expected:
            raise ValueError('frozen replay dependency changed: '+name)
    if digest(ROOT/plan['parent_plan_path']) != plan['parent_plan_sha256']:
        raise ValueError('parent plan changed')
    return plan


def save(path, value):
    raw = json.dumps(value, separators=(',', ':'), allow_nan=False)
    tmp = path.with_suffix(path.suffix+'.tmp')
    with tmp.open('w') as stream:
        os.chmod(tmp, 0o600)
        stream.write(raw+'\n')
        stream.flush()
        os.fsync(stream.fileno())
    tmp.replace(path)


def filename(request):
    return request['symbol']+'-'+request['from'][:10]+'.json'


def validate_job(job, request, plan):
    if job.get('request') != request or job.get('status') != 'complete':
        raise ValueError('source incomplete or request mismatch')
    result = job['result']
    if result['environment'] != 'demo' or result['qualification'] != 'DEVELOPMENT_SCENARIO_NOT_FINANCIAL_PASS':
        raise ValueError('wrong source environment/qualification')
    if not isinstance(result['symbol_id'], int) or result['symbol_id'] <= 0:
        raise ValueError('missing exact broker symbol identity')
    source, limits = result['input'], plan['limits']
    if market_collection.source_digest(source) != result['input_sha256']:
        raise ValueError('source checksum mismatch')
    contract = source['contract']; volume = request['tick_replay']['volume']
    if (not 0 <= contract['digits'] <= 5 or contract['min_volume'] <= 0
            or contract['step_volume'] <= 0 or volume < contract['min_volume']
            or volume % contract['step_volume']):
        raise ValueError('requested native volume/grid incompatible with current contract')
    if len(source['bids'])+len(source['asks']) > limits['max_ticks_per_job_combined_bid_ask']:
        raise ValueError('tick budget exceeded')
    if not 1 <= result['tick_pages'] <= limits['max_tick_pages_per_job']:
        raise ValueError('page budget exceeded')
    start, end = replay.context.ms(request['from']), replay.context.ms(request['to'])+5000
    for side in ('bids','asks'):
        if not source[side]:
            raise ValueError('missing tick side')
        previous = start-1
        for tick in source[side]:
            at, price = tick['ts_ms'], tick['price_e5']
            if (type(at) is not int or type(price) is not int or price <= 0
                    or not previous <= at < end or at < start
                    or price % (10**(5-contract['digits']))):
                raise ValueError('invalid, out-of-window or unordered tick')
            previous = at


def state_at(out):
    path = out/'ledger.json'
    state = json.loads(path.read_text()) if path.exists() else dict(plan_sha256=PLAN_SHA, attempts=[])
    if state['plan_sha256'] != PLAN_SHA:
        raise ValueError('output belongs to another plan')
    return state


def reserve(out, state, request, plan):
    attempts = state['attempts']
    if (len(attempts) >= plan['limits']['max_new_jobs_total']
            or any(a['request'] == request for a in attempts)):
        raise ValueError('budget exhausted or request already attempted; no retry')
    attempts.append(dict(request=request, status='reserved_before_post', job_id=None))
    save(out/'ledger.json', state)
    return attempts[-1]


def collect(out, plan, password, api=client.call, sleep=time.sleep, clock=time.monotonic):
    out.mkdir(parents=True, exist_ok=True)
    state = state_at(out)
    expected = plan['collection_requests']
    attempts = state['attempts']
    if [x['request'] for x in attempts] != expected[:len(attempts)]:
        raise ValueError('ledger is not the frozen request prefix')
    if any(x['status'] != 'complete' for x in attempts):
        raise ValueError('prior failed/uncertain attempt; inspect, never auto-retry')
    for item in attempts:
        path = out/filename(item['request'])
        if not path.exists() or digest(path) != item['file_sha256']:
            raise ValueError('saved source changed/missing')
    if len(attempts) >= 2:
        check = json.loads((out/'preflight.json').read_text())
        if (check['plan_sha256'] != PLAN_SHA or not check['passed']
                or check['source_sha256'] != {filename(x['request']):x['file_sha256'] for x in attempts[:2]}
                or check['adapter_sha256'] != digest(Path(__file__))):
            raise ValueError('matching successful two-episode preflight required')
    count = 0
    for request in expected[len(attempts):]:
        if count >= plan['limits']['max_new_jobs_per_invocation']:
            break
        current = api(BASE,password)
        if current and current.get('status') == 'running':
            raise ValueError('another job is active; do not replace')
        if sum(p.stat().st_size for p in out.glob('Spot*.json'))+32_000_000 > plan['limits']['max_total_source_bytes']:
            raise ValueError('source storage budget exhausted')
        item = reserve(out,state,request,plan)
        try:
            initial = api(BASE,password,request)
            item.update(job_id=initial['id'], status='submitted')
            save(out/'ledger.json',state)
            print('submitted',filename(request),item['job_id'],flush=True)
            deadline = clock()+plan['limits']['poll_deadline_seconds_per_job']
            while clock() < deadline:
                sleep(3)
                job = api(BASE,password)
                if not job or job.get('id') != item['job_id'] or job.get('request') != request:
                    raise ValueError('job replaced/restarted or request mismatch')
                if job['status'] != 'running':
                    break
            else:
                raise ValueError('poll deadline; do not resubmit')
            path = out/filename(request)
            if path.exists():
                raise ValueError('refuse to overwrite source')
            raw = json.dumps(job,separators=(',',':'),allow_nan=False).encode()
            if len(raw)+1 > plan['limits']['max_source_bytes_per_job']:
                raise ValueError('source storage size limit')
            save(path,job)
            item['file_sha256'] = digest(path)
            validate_job(job,request,plan)
            item['status'] = 'complete'
            save(out/'ledger.json',state)
            count += 1
            print('complete',filename(request),job['result']['tick_pages'],'pages',flush=True)
        except Exception as error:
            # No exception text is stored: HTTP diagnostics may contain credentials.
            item['status'] = 'failed_or_uncertain'
            item['error_type'] = type(error).__name__
            save(out/'ledger.json',state)
            raise
    return count


def value(row, arm='trailing', bps=24):
    if row['status'] != 'simulated_entry' or not row.get('replay'):
        return None
    return next(s['scenario_net_r'] for s in row['replay']['sensitivities']
                if s['arm']==arm and s['bps_per_side']==bps)


def evidence_gaps(rows, arm='trailing'):
    result = []
    for row in rows:
        if row['status'] in DATA_GAP_REASONS or row['status'] in ('source_missing','source_failed'):
            result.append(dict(symbol=row['symbol'],date=row['from'],reason=row['status']))
        if row['status']=='simulated_entry':
            actual = row['replay']['comparison']['candidate' if arm=='trailing' else 'baseline']
            for reason in actual.get('data_issues',[]):
                result.append(dict(symbol=row['symbol'],date=row['from'],reason=reason))
    return result


def score(rows, plan, arm='trailing', bps=24, dates=None):
    dates = plan['development_dates'] if dates is None else dates
    symbols = plan['symbols']
    result = replay.score_rows(rows,arm,bps,10,len(dates)*len(symbols),symbols)
    result['per_symbol'] = {}
    for symbol in symbols:
        sample = [r for r in rows if r['symbol']==symbol]
        item = replay.score_rows(sample,arm,bps,5,len(dates),[symbol])
        weeks = {dt.date.fromisoformat(r['from'][:10]).isocalendar()[:2]
                 for r in sample if value(r,arm,bps) is not None}
        item['distinct_closed_iso_weeks'] = len(weeks)
        if len(weeks) < 3:
            item['reasons'].append('fewer than three closed ISO weeks')
        item['passes_screen'] = not item['reasons']
        if not item['passes_screen']:
            result['reasons'].append(symbol+': '+'; '.join(item['reasons']))
        result['per_symbol'][symbol] = item
    result['coverage_flags'] = evidence_gaps(rows,arm)
    if result['coverage_flags']:
        result['reasons'].append('reported source/quote coverage gaps')
    result['passes_screen'] = not result['reasons']
    return result


def controls(rows, symbols):
    index = {(r['symbol'],r['from'],r['rule']):r for r in rows}
    results = []
    for control in ('peer_continuation','always_long'):
        for group in [symbols]+[[s] for s in symbols]:
            pairs = []
            for row in rows:
                if row['rule']!='own_momentum' or row['symbol'] not in group:
                    continue
                other = index[(row['symbol'],row['from'],control)]
                a,b = value(row),value(other)
                if a is not None and b is not None:
                    pairs.append((a-b,row.get('side')!=other.get('side')))
            results.append(dict(control=control,symbols=group,matched=len(pairs),
                mean_delta_r=statistics.mean(p[0] for p in pairs) if pairs else None,
                changed_direction=sum(p[1] for p in pairs)))
    return results


def analyse(out, plan, verifier, binary, preflight=False):
    state = state_at(out)
    requests = plan['collection_requests'][:2] if preflight else plan['collection_requests']
    jobs, corpus, provenance = {}, {}, []
    attempts = {filename(x['request']):x for x in state['attempts']}
    for request in requests:
        name = filename(request); path = out/name
        if name not in attempts or attempts[name]['status']!='complete':
            continue
        if path.stat().st_size>32_000_000 or digest(path)!=attempts[name]['file_sha256']:
            raise ValueError('immutable source file changed')
        job=json.loads(path.read_text());validate_job(job,request,plan)
        proof=json.loads(subprocess.run([str(verifier),str(path)],check=True,
            capture_output=True,text=True,timeout=90).stdout)
        if proof['input_sha256']!=job['result']['input_sha256']:
            raise ValueError('native source parity mismatch')
        key=(request['symbol'],replay.context.ms(request['from']))
        jobs[key]=job;corpus[key]=replay.context.quotes(job['result']['input'])
        provenance.append(dict(file=name,file_sha256=digest(path),input_sha256=proof['input_sha256'],
            source_algorithm_sha256=proof['source_algorithm_sha256'],
            replay_algorithm_sha256=proof['replay_algorithm_sha256'],
            native_source_quality=job['result']['study']['quality']))
    rows=[]
    for request in requests:
        key=(request['symbol'],replay.context.ms(request['from']))
        for rule in ['own_momentum']+plan['controls']:
            if key not in jobs:
                rows.append(dict(symbol=request['symbol'],asset_class='oil',rule=rule,
                    **{'from':request['from']},
                    status='source_failed' if filename(request) in attempts else 'source_missing',replay=None))
                continue
            row, native = replay.prepare(rule,request['symbol'],'oil',jobs[key],corpus,100)
            if native:
                row['replay']=json.loads(subprocess.run([str(binary)],input=json.dumps(native),
                    check=True,capture_output=True,text=True,timeout=90).stdout)
                row['status']=row['replay']['status']
            rows.append(row)
    if preflight:
        problems=[r['status'] for r in rows if r['status'] in DATA_GAP_REASONS
                  or r['status'] in ('source_missing','source_failed')
                  or (r['status']=='simulated_entry' and value(r) is None)]
        receipt=dict(plan_sha256=PLAN_SHA,passed=len(jobs)==2 and not problems,problems=problems,
            source_sha256={p['file']:p['file_sha256'] for p in provenance},
            adapter_sha256=digest(Path(__file__)),binary_sha256=dict(verifier=digest(verifier),replay=digest(binary)),
            rows=rows,coverage_flags=evidence_gaps(rows),financial_pass=False)
        save(out/'preflight.json',receipt)
        print(json.dumps(dict(preflight_passed=receipt['passed'],problems=problems,
                             statuses=dict(collections.Counter(r['status'] for r in rows)))))
        return receipt
    summaries=[]
    for rule in ['own_momentum']+plan['controls']:
        subset=[r for r in rows if r['rule']==rule]
        for arm in ('fixed_stop','trailing'):
            for bps in (2.0,3.74,24.0):
                summaries.append(dict(rule=rule,arm=arm,bps=bps,**score(subset,plan,arm,bps)))
    primary=next(s for s in summaries if s['rule']=='own_momentum' and s['arm']=='trailing' and s['bps']==24)
    matched=controls(rows,plan['symbols'])
    for match in matched:
        if match['control']=='always_long' and (match['mean_delta_r'] is None or match['mean_delta_r']<=0):
            primary['reasons'].append('no positive matched always-long delta: '+','.join(match['symbols']))
    primary['passes_screen']=not primary['reasons']
    insufficient=(len(jobs)!=20 or primary['censored'] or primary['coverage_flags']
                  or any(x['closed']<5 or x['distinct_closed_iso_weeks']<3 for x in primary['per_symbol'].values()))
    decision='PASS_PRELIMINARY_SCREEN' if primary['passes_screen'] else ('INSUFFICIENT_EVIDENCE' if insufficient else 'REJECT')
    old_path=ROOT/'docs/full-exit-research-2026-11-06/full-exit-results.json'
    old=json.loads(old_path.read_text())
    prior=[r for r in old['rows'] if r['asset_class']=='oil' and r['rule']=='own_momentum']
    own=[r for r in rows if r['rule']=='own_momentum']
    result=dict(trace='№18,287 · '+dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT')+' · by Codex (GPT-6), effort not exposed',
        plan_sha256=PLAN_SHA,adapter_sha256=digest(Path(__file__)),
        binary_sha256=dict(verifier=digest(verifier),replay=digest(binary)),
        complete_jobs=len(jobs),attempted_jobs=len(state['attempts']),expected_jobs=20,
        primary=primary,decision=decision,summary=summaries,matched_controls=matched,
        rows=rows,provenance=provenance,ledger=state,
        prior_development=dict(source_sha256=digest(old_path),dates=plan['prior_development_dates_tracked_separately'],
            primary=replay.score_rows(prior,'trailing',24,10,4,plan['symbols'])),
        pooled_descriptive_only=replay.score_rows(prior+own,'trailing',24,10,24,plan['symbols']),
        validation_opened=False,financial_pass=False,qualified_net_win_rate=None,
        interim_target=0.5,account_goal_progress='NOT_VERIFIABLE; zero qualifying account trades added')
    save(out/'results.json',result)
    report(result,out/'Oil_Replication_Report.html')
    print(json.dumps({k:result[k] for k in ('decision','complete_jobs','attempted_jobs','primary')}))
    return result


def report(result,path):
    esc=lambda x:html.escape(str(x))
    def display(key,value):
        if value is None:
            return 'unavailable'
        if isinstance(value,float):
            return f'{value:.1%}' if key=='net_win_rate' else f'{value:.4f}'
        return str(value)
    def table(rows,keys):
        return '<table><tr>'+''.join('<th>'+esc(k)+'</th>' for k in keys)+'</tr>'+''.join(
            '<tr>'+''.join('<td>'+esc(display(k,row.get(k)))+'</td>' for k in keys)+'</tr>' for row in rows)+'</table>'
    p=result['primary']
    symbols=[dict(symbol=s,**v) for s,v in p['per_symbol'].items()]
    path.write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Oil replication</title><style>body{font:16px/1.5 system-ui;max-width:1200px;margin:30px auto;padding:16px}table{border-collapse:collapse;width:100%}td,th{padding:8px;border-bottom:1px solid #ddd;text-align:left}pre{white-space:pre-wrap;overflow-wrap:anywhere}.scroll{overflow:auto}</style><h1>Oil own-momentum replication</h1><p>'+esc(result['trace'])+'</p><h2>TLDR</h2><p><strong>'+esc(result['decision'])+'</strong>. '+esc(p['wins'])+'/'+esc(p['closed'])+' closed scenarios win; mean net R '+esc(p['mean_net_r'])+'. '+esc(result['complete_jobs'])+'/20 source episodes complete. No validation opened or live policy changed.</p><p>Interim target: 50% per symbol plus positive expectancy. Original goal: 73% / 1:1.38 over latest20 per strategy/account. Qualified progress: Not Verifiable; zero account trades added.</p><h2>Every symbol: primary 24bp trailing scenario</h2><div class="scroll">'+table(symbols,['symbol','closed','wins','net_win_rate','mean_net_r','profit_factor','distinct_closed_iso_weeks','passes_screen'])+'</div><h2>Why this decision</h2><pre>'+esc(json.dumps(p['reasons'],indent=2))+'</pre><h2>Matched controls</h2>'+table(result['matched_controls'],['control','symbols','matched','changed_direction','mean_delta_r'])+'<h2>All scenarios</h2><div class="scroll">'+table(result['summary'],['rule','arm','bps','closed','wins','net_win_rate','mean_net_r','profit_factor','passes_screen'])+'</div><h2>Interpretation and limits</h2><p>24bp/side remains primary admission/target geometry. 2/3.74bp sensitivities hold geometry fixed and are not calibrated. Executable spread is charged once; additional commission is unknown and modelled as zero. Historical contracts, execution, fees and quote continuity remain unqualified. Price-unit results are per symbol, never pooled across assets. Controls reuse the same dates. Reported gaps/censoring fail the screen; small samples do not establish an edge.</p><h2>Prior and pooled diagnostics</h2><p>Previously inspected5/12August dates cannot rescue standalone gates.</p><pre>'+esc(json.dumps(dict(prior=result['prior_development'],pooled=result['pooled_descriptive_only']),indent=2))+'</pre><h2>Next decision</h2><p>Stop at the frozen batch. Insufficient evidence is distinct from demonstrated losses. Do not lower costs, retune exits, replace dates or collect a second batch to obtain a pass. Even a preliminary pass needs separate-date validation and representative paired costs before promotion.</p><p>Plan SHA256: '+esc(result['plan_sha256'])+'. Source hashes, source failures, native fills and every outcome are in results.json. Draft for review.</p></html>')


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('action',choices=['collect','preflight','analyse'])
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--verifier',type=Path)
    p.add_argument('--replay',type=Path)
    a=p.parse_args();plan=load_plan();a.output.mkdir(parents=True,exist_ok=True)
    with (a.output/'.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        if a.action=='collect':
            collect(a.output,plan,os.environ['JIAOYI_UI_PASSWORD'])
        else:
            if not a.verifier or not a.replay:
                p.error('native --verifier and --replay binaries required')
            result=analyse(a.output,plan,a.verifier,a.replay,a.action=='preflight')
            if a.action=='preflight' and not result['passed']:
                raise SystemExit('Technical preflight failed; remaining collection stopped')


if __name__=='__main__':
    main()
