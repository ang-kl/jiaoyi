#!/usr/bin/env python3
"""№18,351: frozen cached-only paired experiment. No broker client import."""
import argparse
import collections
import datetime as dt
import hashlib
import html
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT/'docs/option_a_vs_b_offline_plan_18351.json'
RESULT = ROOT/'docs/option_a_vs_b_offline_results_18351.json'
REPORT = ROOT/'doc/OPTION_A_VS_B_OFFLINE_REPORT-18351-2026-11-08.md'
BINARY = ROOT/'target/debug/examples/option_ab_replay'
OUT = Path('/workspace/jiaoyi-option-ab-18351')
SOURCES = [('docs/three-year-execution-18324.json','docs/offline-repaired-engine-results-18331.json','/workspace/jiaoyi-three-year-pilot-18325'),
           ('docs/high_vol_cohorts_plan_18348.json','docs/high_vol_cohorts_results_18348.json','/workspace/jiaoyi-high-vol-18348')]
MODELS = ['s3_breakout_4h','option_a_fade_4h','option_b_breakout_72h','option_ab_fade_72h','s3_breakout_4h_mtm','option_a_fade_4h_mtm']
CODE = ['ops/research/option_a_vs_b_offline.py','ops/research/test_option_a_vs_b_offline.py','ops/research/run_option_a_vs_b_offline_18351.sh',
        'crates/jiaoyi-core/src/research/option_ab.rs','crates/jiaoyi-core/src/research/mod.rs','crates/jiaoyi-core/examples/option_ab_replay.rs']

def read(p): return json.loads(Path(p).read_text())
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,x): Path(p).write_text(json.dumps(x,indent=2,allow_nan=False)+'\n')
def now(): return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')
def instant(t): return dt.datetime.fromisoformat(t.replace('Z','+00:00'))
def millis(t): return int(instant(t).timestamp()*1000)
def protected(t): return millis('2025-08-19T00:00:00Z')<=t<millis('2025-09-06T00:00:00Z') or t>=millis('2025-10-24T00:00:00Z')
def cmd(*a): return subprocess.check_output(a,cwd=ROOT,text=True,timeout=120).strip()
def journal(section,text):
    p=ROOT/'log/CODEX_Log.txt';s=p.read_text();tag='№18,351 '+section
    if tag in s:return
    n=max(map(int,re.findall(r'\[§LOG\.(\d+)\]',s)),default=0)+1
    p.write_text(s+f'\n[§LOG.{n}] {tag} · {now()}; by <Codex (GPT-6), effort: not exposed>.\n{text}\n')

def verify_receipts(folder):
    events=[json.loads(s) for s in (folder/'ledger.jsonl').read_text().splitlines()]
    starts=[e['key'] for e in events if e['state']=='started'];ends=[e for e in events if e['state']=='finished']
    if len(starts)!=len(set(starts)) or len(ends)!=len(starts) or set(starts)!={e['key'] for e in ends}:raise ValueError('uncertain/duplicate receipts')
    for e in ends:
        if sha(folder/(e['key']+'.json'))!=e.get('sha256',e.get('file_sha256')):raise ValueError('receipt hash mismatch')
    return {e['key']:e for e in ends}

def merge_bars(index,symbol,bars):
    target=index.setdefault(symbol,{})
    for b in bars:
        t=b['ts_ms']
        if protected(t):raise ValueError('sealed future bar')
        if t in target and target[t]!=b:raise ValueError('conflicting cached M30')
        target[t]=b

def prepare():
    if PLAN.exists():raise ValueError('plan already frozen; use run')
    jobs=[];excluded=[];bars=[];hashes={};seen=set()
    for manifest,result,folder in SOURCES:
        folder=Path(folder);m=read(ROOT/manifest);prior=read(ROOT/result);rows={r['key']:r for r in prior['rows']}
        done=verify_receipts(folder)
        for p in [ROOT/manifest,ROOT/result,folder/'ledger.jsonl',folder/'freeze.json']:hashes[str(p)]=sha(p)
        for j in m['jobs']:
            key=j['key'];path=folder/('job-'+key+'.json');h=sha(path);r=rows[key]
            if h!=r['file_sha256']:raise ValueError('prior evidence changed')
            value=read(path)
            if value['status']=='complete' and value['request']!=j['request']:raise ValueError('source request identity changed')
            if key in seen:raise ValueError('duplicate episode')
            seen.add(key);hashes[str(path)]=h
            item=dict(key=key,symbol=j['symbol'],date=j['date'],cohort=j['cohort'],path=str(path),sha256=h,
                      previous_result=result,protocol_flags=r.get('protocol_flags',[]),status=value['status'])
            if value['status']=='complete':jobs.append(item)
            else:excluded.append(item)
        for part in m['bar_segments']:
            path=Path(part['cache']['path']) if part.get('mode')=='reuse' else folder/('bar-'+part['key']+'.json')
            if part.get('mode')=='blocked':continue
            value=read(path);h=sha(path)
            if part.get('mode')=='reuse' and h!=part['cache']['sha256']:raise ValueError('reuse receipt mismatch')
            hashes[str(path)]=h
            if value['status']=='complete':
                if value['request']['symbol_id']!=part['symbol_id']:raise ValueError('wrong bar symbol')
                bars.append(dict(path=str(path),sha256=h,symbol=part['symbol'],
                                 first=part['from'],last=part['to']))
    if len(jobs)!=159 or len(excluded)!=1:raise ValueError('159 completed episode scope changed')
    plan=dict(trace='№18,351',frozen_at_sgt=now(),jobs=jobs,excluded_sources=excluded,bar_sources=bars,source_sha256=hashes,
        models=MODELS,new_broker_requests=0,primary_cost_bps_per_side=24.,sensitivity_bps_per_side=[2.,3.74],
        admission='Original native 24bp/side full C<R and >=48bp R/fill; first admitted entry only, same timestamp and R in every arm. Sensitivities retain primary entries.',
        entry='Use same next executable fresh quote as baseline, never an earlier decision quote. Fade buy at ask and sell at bid. Original quantity and R retained.',
        exit='Original -1R stop; +1R next-quote entry-price amend; gross1.38R rounded outward. No partial exit or further trailing.',
        horizons='4h means existing registered observation boundary, not four hours after fill. Oil inherited native quotes start07:00 before13:00-17:00 window. 72 elapsed hours from fill, not three trading sessions; weekend hours count.',
        baseline='No native session clock close before the tick boundary; unresolved ticks censored. Additional explicit MTM arm uses first fresh boundary quote within5s, broker bracket first.',
        continuation='Only subsequent contiguous M30 bars starting exactly at tick boundary; stop at first absent bar, uncertain intrabar order, partial deadline bar or end of cache. Never infer holiday/weekend continuity.',
        bar_model='M30 interpreted as bid OHLC; short ask exits use frozen recorded entry spread proxy. Existing bracket first at bar open; adverse gaps use observed open, TP capped at target. Stop/target or +1R-and-return ambiguity stays unresolved. No synthetic OHLC tick path.',
        bar_amend='After trigger with no ambiguous return, a distinct favourable closing price proves a later quote; equal extreme/close leaves amend timing ambiguous and unresolved. Short proxy and intrabar execution remain unqualified.',
        costs='Bid/ask tick fills include spread once. M30 short asks use entry-spread proxy once. Charge (entry+exit)*bps/10000/R. Zero extra commission inherited, financing/overnight swap and historical spread unavailable; rates uncalibrated.',
        old_censor_reconciliation='Reconcile all five old censored trade identities. Four May peer episodes outside159 remain excluded, not replaced or silently evaluated.',
        evidence='Previously observed development cache, retrospectively chosen hypotheses, protocol flags retained. Not untouched validation or account qualification.',
        validation_opened=False,financial_pass=False,production_changed=False)
    # Validate the union for conflicts before freezing; never use it for entry.
    bar_index(plan)
    save(PLAN,plan);journal('§§1–6','Frozen159 cached episodes; zero broker requests. Matched entries, 24/2/3.74bp cost ledgers. Separate tick censor/MTM and conditional contiguous M30 continuation; out-of-scope prior censoring retained. Requested LOG130 already occupied; append unique log index.')
    print('FROZEN_PLAN '+sha(PLAN));return plan

def bar_index(plan):
    index={}
    for src in plan['bar_sources']:
        if sha(src['path'])!=src['sha256']:raise ValueError('cached M30 hash changed')
        a,b=millis(src['first']),millis(src['last'])
        merge_bars(index,src['symbol'],[x for x in read(src['path'])['bars'] if a<=x['ts_ms'] and x['ts_ms']+1800000<=b])
    return index

def verify_plan(plan):
    for path,h in plan['source_sha256'].items():
        if sha(path)!=h:raise ValueError('frozen source changed: '+path)

def freeze(plan):
    verify_plan(plan)
    files=sorted((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',PLAN]+[ROOT/n for n in CODE]
    hashes={str(p.relative_to(ROOT)):sha(p) for p in files}
    for path,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+path],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit code/plan before evaluation')
    record=dict(plan_sha256=sha(PLAN),code_sha256=hashes,binary_sha256=sha(BINARY),commit=cmd('git','rev-parse','HEAD'))
    f=OUT/'freeze.json'
    if f.exists():
        old=read(f)
        if any(old[k]!=v for k,v in record.items()):raise ValueError('execution freeze changed')
        return old
    record['frozen_at_sgt']=now();save(f,record);return record

def without_notes(v):
    if isinstance(v,dict):return {k:without_notes(x) for k,x in v.items() if k!='notes'}
    if isinstance(v,list):return [without_notes(x) for x in v]
    return v

def score(rows,model,bps):
    values=[];states=collections.Counter();gross=[]
    for row in rows:
        m=row.get('models',{}).get(model)
        if not m:continue
        states[m['status']]+=1
        if m.get('metrics'):
            values.append(m['metrics']['net_by_bps'][bps]);gross.append(m['metrics']['gross_r'])
    pos=sum(v for v in values if v>0);neg=-sum(v for v in values if v<0)
    return dict(episodes=len(rows),entries=sum(states.values()),closed=len(values),wins=sum(v>1e-12 for v in values),
        losses=sum(v < -1e-12 for v in values),net_flat=sum(abs(v)<=1e-12 for v in values),
        unresolved=sum(states.values())-len(values),statuses=dict(states),win_rate=sum(v>1e-12 for v in values)/len(values) if values else None,
        mean_gross_r=statistics.mean(gross) if gross else None,mean_net_r=statistics.mean(values) if values else None,
        profit_factor=pos/neg if neg else None,profit_factor_status='finite' if neg else 'no_losses' if pos else 'unavailable',financial_pass=False)

def accounting_check(result):
    checks=0
    for model in result.get('models',{}).values():
        if not model.get('metrics'):continue
        i=model['entry'];side=1 if i['side']=='long' else -1
        price=model['exit']['price'] if model['status']=='closed_m30_approximation' else model['outcome']['fills'][0]['price']
        gross=side*(price-i['fill'])/i['initial_r']
        if not math.isclose(gross,model['metrics']['gross_r'],abs_tol=1e-10):raise ValueError('gross accounting mismatch')
        for bps,value in model['metrics']['net_by_bps'].items():
            wanted=gross-(i['fill']+price)*float(bps)/10000/i['initial_r']
            if not math.isclose(wanted,value,abs_tol=1e-10):raise ValueError('net accounting mismatch')
            checks+=1
    return checks

def run():
    def no_network(event,args):
        if event in ['socket.connect','socket.getaddrinfo']:raise RuntimeError('offline evaluator prohibits network')
    sys.addaudithook(no_network)
    OUT.mkdir(exist_ok=True);plan=read(PLAN);provenance=freeze(plan);index=bar_index(plan)
    priors={name:{r['key']:r for r in read(ROOT/name)['rows']} for _,name,_ in SOURCES}
    rows=[];checks=0
    for n,j in enumerate(plan['jobs'],1):
        source=read(j['path']);a=millis(source['request']['to']);b=a+72*3600000
        future=[v for t,v in sorted(index[j['symbol']].items()) if a<=t and t+1800000<=b and not protected(t)]
        fp=OUT/('future-'+j['key']+'.json');save(fp,future)
        raw=subprocess.check_output([str(BINARY),j['path'],str(fp)],cwd=ROOT,text=True,timeout=90)
        native=json.loads(raw);r=native['result'];old=priors[j['previous_result']][j['key']]
        if without_notes(r['legacy'])!=without_notes(old['models']['s3_two_stage']):raise ValueError('unchanged native admission/exit parity failed: '+j['key'])
        checks+=accounting_check(r)
        out=OUT/('replay-'+j['key']+'.json');out.write_text(raw)
        row={k:j[k] for k in ['key','symbol','date','cohort','protocol_flags']}
        row.update(status=r['status'],models=r['models'],source_sha256=j['sha256'],future_sha256=sha(fp),
                   future_bars=len(future),native_sha256=sha(out),native_legacy_parity=True,
                   legacy_entry=r['legacy'].get('study',{}).get('trade',{}),
                   admission_reasons=r['legacy'].get('study',{}).get('marks',[]))
        # Do not duplicate full legacy exit structures in every result row.
        row['legacy_entry']=row['legacy_entry']['entry'] if row['legacy_entry'] else None
        rows.append(row)
        if n%20==0 or n==len(plan['jobs']):print(f'REPLAY {n}/{len(plan["jobs"])}',flush=True)
    groups={'overall':rows}
    for s in sorted({r['symbol'] for r in rows}):groups[s]=[r for r in rows if r['symbol']==s]
    for c in sorted({r['cohort'] for r in rows}):groups[c]=[r for r in rows if r['cohort']==c]
    summary={g:{m:{bps:score(rs,m,bps) for bps in ['24','2','3.74']} for m in MODELS} for g,rs in groups.items()}
    old=read(ROOT/'docs/high_vol_cohorts_results_18348.json')['full_cumulative_legacy']['unique_trades']
    bykey={r['key']:r for r in rows};reconcile=[]
    for t in old:
        if t['net_r'] is not None:continue
        item=dict(key=t['key'],previous_traces=t['traces'],included_in_159=t['key'] in bykey)
        if item['included_in_159']:
            r=bykey[t['key']];item['option_b']=r['models']['option_b_breakout_72h'];item['option_ab']=r['models']['option_ab_fade_72h']
        else:item['status']='outside_frozen_159_episode_scope'
        reconcile.append(item)
    verify_plan(plan)
    result=dict(trace='№18,351',generated_at_sgt=now(),provenance=provenance,plan_sha256=sha(PLAN),rows=rows,summary=summary,
                previous_five_censored_reconciliation=reconcile,excluded_sources=plan['excluded_sources'],
                verification=dict(unchanged_legacy_parity_episodes=len(rows),independent_cost_checks=checks,source_hashes_reverified=True),
                new_broker_requests=0,validation_opened=False,financial_pass=False,production_changed=False,
                interpretation={k:plan[k] for k in ['admission','entry','horizons','baseline','continuation','bar_model','bar_amend','costs','evidence']})
    save(RESULT,result);report(result)
    journal('§§7–12','Completed159 offline native replays with exact old-model parity; '+str(checks)+' independent cost checks. Zero broker calls. '+json.dumps(summary['overall'])+'\nPrior five censoring identities reconciled; four outside fixed sample retained as excluded. Validation sealed, no account qualification or live change.')
    print(json.dumps(summary['overall'],indent=2))

def report(r):
    def fmt(x):return 'unavailable' if x is None else f'{x:.6f}'
    def table(bps):
        lines=['| Model | Entries | Closed | Wins | Unresolved | Win rate | Mean net R | PF(R) |','| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
        for m in MODELS:
            s=r['summary']['overall'][m][bps];lines.append(f'| {m} | {s["entries"]} | {s["closed"]} | {s["wins"]} | {s["unresolved"]} | {fmt(s["win_rate"])} | {fmt(s["mean_net_r"])} | {fmt(s["profit_factor"])} |')
        return lines
    s=r['summary']['overall'];lines=['# Option A vs B: matched-entry offline comparison','',
        f'№18,351 §§1–12 · {r["generated_at_sgt"]} · by Codex (GPT-6), effort not exposed.','',
        '## §1. TLDR','',
        f'Completed **{len(r["rows"])} cached episodes** with **zero new broker requests** and exact prior native-model parity on all159. Each arm uses the same {s[MODELS[0]]["24"]["entries"]} admitted setups. Results are conditional research outcomes, not realised account trades or a financial PASS.','',
        'Original invariant: **73% net wins / reference1:1.38 risk:reward on latest20 trades per strategy/account**. This reused pooled sample cannot qualify that goal. No new independent trade evidence was collected.','',
        '## §2. Primary24bp/side plus spread','']+table('24')+['',
        'The four-hour censor and MTM rows are deliberately separate. Bar continuation closes are conditional OHLC approximations; unresolved or ambiguous paths remain in the denominator as unresolved, not zero-return trades. Closed-only win rates can be selection-biased.','',
        '## §3. Same-entry cost sensitivities','',
        'These change cost deductions only. They do not reopen vetoed setups or recalibrate broker costs.','',
        '### 2bp/side','']+table('2')+['','### 3.74bp/side','']+table('3.74')+['',
        '## §4. Entry-by-entry outcomes at24bp','',
        '| Episode | Baseline4h | Fade4h | Breakout72h | Fade72h |','| --- | --- | --- | --- | --- |']
    for row in r['rows']:
        if not row['models']:continue
        cells=[]
        for m in MODELS[:4]:
            v=row['models'][m];cells.append(fmt(v['metrics']['net_by_bps']['24'])+'R ('+v['status']+')' if v.get('metrics') else v['status'])
        lines.append('| '+row['key']+' | '+' | '.join(cells)+' |')
    lines+=['','## §5. What happened to the five previously censored trades?','',
        '| Episode | In159? | Continuation result |','| --- | --- | --- |']
    for v in r['previous_five_censored_reconciliation']:
        lines.append(f'| {v["key"]} | {v["included_in_159"]} | {v["option_b"]["status"] if v["included_in_159"] else v["status"]} |')
    lines+=['','Four May peer trades are outside the explicitly frozen119+40 corpus. They were neither dropped from history nor added to improve this comparison. The failed US50024January2024 source also remains documented outside the159 completed episodes.','',
        '## §6. Interpretation and evidence boundaries','']
    for k,v in r['interpretation'].items():lines.append(f'- **{k}**: {v}')
    lines+=['','## §7. Assessment and next decision','',
        'Compare the paired directions before attributing any change to prediction skill. A gross-flat exit remains a loss after costs; a lower assumed charge cannot turn a negative gross fill into a winner. M30 continuation cannot establish a reliable multi-day edge without executable bid/ask paths, unambiguous order sequencing and financing.','',
        'Any apparent winner here is a hypothesis selected on previously seen data. Preserve losses, protocol flags and unresolved trades. Freeze one justified candidate and realistic cost specification before a genuinely untouched evaluation; do not tune until a target is reached. Live rules remain unchanged and the19August–5September2025 holdout remains sealed.','',
        '## §8. Reproducibility','',
        f'Plan SHA256: `{r["plan_sha256"]}`. Native binary and source hashes, per-episode results, bar coverage and accounting checks are in the results JSON. {r["verification"]["independent_cost_checks"]} independent net-cost calculations checked. Prior cached-source protocol flags are retained per episode.','',
        '[Frozen plan](../docs/option_a_vs_b_offline_plan_18351.json) · [Measured results](../docs/option_a_vs_b_offline_results_18351.json)','']
    REPORT.write_text('\n'.join(lines))
    # Small dependency-free HTML renderer for the report's known Markdown subset.
    body=[];in_table=False;in_list=False
    for line in lines:
        if line.startswith('|'):
            if not in_table:body.append('<div class="scroll"><table>');in_table=True
            cells=[x.strip() for x in line.strip('|').split('|')]
            if all(re.fullmatch(r'[-: ]+',x) for x in cells):continue
            body.append('<tr>'+''.join('<td>'+html.escape(x)+'</td>' for x in cells)+'</tr>');continue
        if in_table:body.append('</table></div>');in_table=False
        if line.startswith('- '):
            if not in_list:body.append('<ul>');in_list=True
            body.append('<li>'+html.escape(line[2:])+'</li>');continue
        if in_list:body.append('</ul>');in_list=False
        if not line:continue
        if line.startswith('#'):
            n=len(line)-len(line.lstrip('#'));body.append(f'<h{n}>'+html.escape(line[n:].strip())+f'</h{n}>')
        else:body.append('<p>'+html.escape(line)+'</p>')
    page='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi Option A vs B —18351</title><style>body{font:16px/1.6 system-ui,sans-serif;color:#172638;background:#f4f7fa;margin:0}main{max-width:1200px;margin:auto;padding:30px;background:white}h1{font-size:30px}h2{margin-top:36px;border-bottom:2px solid #dce5ee}table{border-collapse:collapse;min-width:720px;width:100%;font-size:14px}td{padding:10px;border:1px solid #ccd8e3}tr:first-child{font-weight:700;background:#e9f1f8}tr:nth-child(even){background:#f7fafc}.scroll{overflow-x:auto}li{margin:9px 0}p,li{overflow-wrap:anywhere}</style><main>'+''.join(body)+'<p><a href="../docs/option_a_vs_b_offline_results_18351.json">Measured JSON</a> · <a href="../docs/option_a_vs_b_offline_plan_18351.json">Frozen plan</a></p></main></html>'
    REPORT.with_suffix('.html').write_text(page)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=['prepare','run']);a=p.parse_args()
    prepare() if a.action=='prepare' else run()
