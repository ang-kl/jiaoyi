#!/usr/bin/env python3
"""Frozen all-class broker-data research. No model API, orders or live promotion."""
import argparse, collections, datetime as dt, hashlib, html, json, math, os, re, subprocess, time
from pathlib import Path
from urllib.parse import urlsplit
from zoneinfo import ZoneInfo
import broker_backtest as client
from research_gates import DATA_GAP_REASONS, LCB_METHOD, data_gap_counts, lcb_net_r
ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/class-prediction-plan-2026-11-05.json'
def windows(plan,stage,symbol):
    tz,h,m=('UTC',7,0)
    if symbol in ('US500','NAS100','US30'):tz,h,m='America/New_York',9,30
    elif symbol=='GER40':tz,h='Europe/Berlin',9
    elif symbol=='UK100':tz,h='Europe/London',8
    elif symbol=='JPN225':tz,h='Asia/Tokyo',9
    elif symbol=='AUS200':tz,h='Australia/Sydney',10
    elif symbol in ('BTCUSD','ETHUSD'):h=0
    for day in plan[stage+'_dates']:
        start=dt.datetime.combine(dt.date.fromisoformat(day),dt.time(h,m),ZoneInfo(tz)).astimezone(dt.timezone.utc)
        yield day,start.isoformat().replace('+00:00','Z'),(start+dt.timedelta(hours=4)).isoformat().replace('+00:00','Z')
def request(plan,symbol,start,end):
    return dict(account=plan['account'],symbol=symbol,**{'from':start,'to':end},spread=0,commission_round_trip=plan['costs']['commission_round_trip'],slippage_bps_per_side=24,tick_replay=dict(volume=plan['native_volume'][symbol],entry_rule=plan['rules'][0]))
def score(rows,minimum,expected,symbols):
    gaps=data_gap_counts(rows)
    identities={(r['symbol'],r['from']) for r in rows};values=[];per=collections.defaultdict(list);censored=0
    for r in rows:
        t=r['study']['trade']
        if t is None:continue
        v=t['trailing_scenario_net_r']
        if v is None:censored+=1;continue
        if not math.isfinite(v) or not math.isfinite(t['entry']['initial_r']) or t['entry']['initial_r']<=0:raise ValueError('invalid outcome')
        values.append(v);per[r['symbol']].append((v,v*t['entry']['initial_r'],r['from']))
    n=len(values);wins=sum(x>0 for x in values);profit=sum(x for x in values if x>0);loss=-sum(x for x in values if x<0);reasons=[]
    if gaps:reasons.append(f'data coverage gaps ({gaps})')
    if len(rows)!=expected or len(identities)!=len(rows):reasons.append('missing or duplicate episodes')
    if n<minimum:reasons.append('insufficient closed scenarios')
    if censored:reasons.append('censored trades')
    if not n or wins/n<.5:reasons.append('net win rate below50% or unavailable')
    if not n or sum(values)<=0:reasons.append('net expectancy not positive')
    if profit<=loss:reasons.append('PF(R) not above1')
    # Unlike price units are never summed across instruments; each must survive costs.
    if any(not per[s] or sum(x[1] for x in per[s])<=0 for s in symbols):reasons.append('a symbol lacks positive net price-unit evidence')
    if any(not per[s] or sum(x[0]>0 for x in per[s])/len(per[s])<.5 for s in symbols):reasons.append('a symbol lacks50% net wins')
    interval=[None,None]
    if n:
        z=1.95996398454;p=wins/n;den=1+z*z/n;centre=(p+z*z/(2*n))/den;half=z*math.sqrt(p*(1-p)/n+z*z/(4*n*n))/den;interval=[centre-half,centre+half]
    return dict(episodes=len(rows),expected=expected,closed=n,censored=censored,wins=wins,net_win_rate=wins/n if n else None,mean_net_r=sum(values)/n if n else None,lcb_net_r=lcb_net_r(values),lcb_method=LCB_METHOD,data_gaps=gaps,net_r=sum(values) if n else None,profit_factor=profit/loss if loss else None,win_rate_wilson95=interval,passes_screen=not reasons,reasons=reasons,symbols={s:dict(closed=len(per[s]),net_price_units=sum(x[1] for x in per[s]) if per[s] else None) for s in symbols},financial_pass=False)
def select(summary):
    chosen={}
    for cls,rules in summary['classes'].items():
        eligible=[(v['lcb_net_r'] if v.get('lcb_net_r') is not None else v['mean_net_r'],v['mean_net_r'],rule) for rule,v in rules.items() if v['passes_screen']]
        if eligible:chosen[cls]=max(eligible)[2]
    return chosen

def verify_receipt(out,plan,digest):
    expected=plan.get('source_collection_plan_sha256') if plan.get('reuse_only') else digest
    if not isinstance(expected,str) or not re.fullmatch('[0-9a-f]{64}',expected):
        raise RuntimeError('valid source collection plan SHA256 required')
    if (out/'plan-sha256.txt').read_text().strip()!=expected:
        raise RuntimeError('collection plan receipt mismatch')
    if not re.fullmatch('[0-9a-f]{64}',digest):raise RuntimeError('invalid analysis plan SHA256')
    if plan.get('reuse_only'):(out/'analysis-plan-sha256.txt').write_text(digest+'\n')
    return expected

def normalise_saved_request(req):
    copy=json.loads(json.dumps(req))
    copy.get('tick_replay',{}).pop('entry_rule',None)
    return copy

def request_matches(saved,expected,reuse_only):
    if expected is None:return False
    if reuse_only:return normalise_saved_request(saved)==normalise_saved_request(expected)
    return saved==expected

def collect(a,plan,digest):
    if plan.get("reuse_only"):
        raise RuntimeError("this exploratory amendment reuses saved sources only; no new broker jobs")
    out=a.output/a.stage;out.mkdir(parents=True,exist_ok=True)
    receipt=out/'plan-sha256.txt'
    if receipt.exists() and receipt.read_text().strip()!=digest:raise RuntimeError('frozen plan changed')
    receipt.write_text(digest+'\n')
    selection=None
    if a.stage=='validation':
        prior=json.loads((a.output/'development/summary.json').read_text())
        if prior['plan_sha256']!=digest:raise RuntimeError('development plan mismatch')
        selection=select(prior)
        if not selection:raise RuntimeError('no class passed development; validation stays unopened')
    password=os.environ['JIAOYI_UI_PASSWORD'];new=0
    for cls,symbols in plan['classes'].items():
        if selection is not None and cls not in selection:continue
        for symbol in symbols:
            for day,start,end in windows(plan,a.stage,symbol):
                path=out/f'{symbol}-{day}.json';req=request(plan,symbol,start,end)
                if path.exists():
                    job=json.loads(path.read_text())
                    if job['request']!=req:raise RuntimeError('saved request differs from plan')
                    continue
                if new>=a.max_new_jobs:return
                current=client.call(a.base,password)
                if current and current['status']=='running':raise RuntimeError('existing job active; do not replace')
                initial=client.call(a.base,password,req);ident=initial['id'];deadline=time.monotonic()+275;new+=1
                print('submitted',a.stage,symbol,day,ident,flush=True)
                while time.monotonic()<deadline:
                    time.sleep(3);job=client.call(a.base,password)
                    if not job or job['id']!=ident:raise RuntimeError('job replaced/restarted; stop')
                    if job['status']!='running':break
                else:raise RuntimeError('job deadline; inspect before retry')
                path.write_text(json.dumps(job));path.chmod(0o600)
                print('finished',symbol,day,job['status'],job.get('error',''),flush=True)
                # Explicit source failures remain missing evidence. No automatic retries/tuning.

def analyse(a,plan,digest):
    out=a.output/a.stage
    collection_digest=verify_receipt(out,plan,digest)
    rows=[];failed=[];provenance=set()
    allowed={(s,start):request(plan,s,start,end) for ss in plan['classes'].values() for s in ss for _,start,end in windows(plan,a.stage,s)}
    for path in sorted(out.glob('*.json')):
        if path.name in ('summary.json','replays.json'):continue
        job=json.loads(path.read_text());req=job.get('request',{})
        if not request_matches(req,allowed.get((req.get('symbol'),req.get('from'))),plan.get('reuse_only',False)):raise RuntimeError('job outside frozen request set')
        if job['status']!='complete':failed.append(dict(symbol=req['symbol'],start=req['from'],error=job.get('error')));continue
        completed=subprocess.run([str(a.binary),str(path)],capture_output=True,text=True,check=True)
        result=json.loads(completed.stdout)
        if not set(plan['rules']).issubset(result['candidates']):raise RuntimeError('binary missing a frozen candidate')
        provenance.add((result['source_algorithm_sha256'],result['replay_algorithm_sha256']))
        for rule,study in result['candidates'].items():
            if rule not in plan['rules']:continue
            rows.append(dict(symbol=result['symbol'],asset_class=result['class'],**{'from':result['from']},rule=rule,input_sha256=result['input_sha256'],study=study))
    classes={};symbols={};selected={}
    if a.stage=='validation':
        prior=json.loads((a.output/'development/summary.json').read_text())
        if prior['plan_sha256']!=digest:raise RuntimeError('development plan mismatch')
        selected=select(prior)
    dates=len(plan[a.stage+'_dates'])
    for cls,ss in plan['classes'].items():
        classes[cls]={}
        for rule in plan['rules']:
            subset=[r for r in rows if r['asset_class']==cls and r['rule']==rule]
            result=score(subset,plan[a.stage+'_min_closed_per_class'],dates*len(ss),ss)
            if a.stage=='validation':
                if selected.get(cls)!=rule:result['reasons'].append('not the frozen development selection')
                if any(result['symbols'][s]['closed']<plan['validation_min_closed_per_symbol'] for s in ss):result['reasons'].append('insufficient per-symbol validation closes')
                weeks={dt.date.fromisoformat(r['from'][:10]).isocalendar()[:2] for r in subset if r['study']['trade'] and r['study']['trade']['trailing_scenario_net_r'] is not None}
                if len(weeks)<plan['validation_min_weeks']:result['reasons'].append('fewer than three validation weeks')
            result['passes_screen']=not result['reasons'];classes[cls][rule]=result
            for s in ss:symbols[s+':'+rule]=score([r for r in subset if r['symbol']==s],1 if a.stage=='development' else plan['validation_min_closed_per_symbol'],dates,[s])
    summary=dict(stage=a.stage,plan_sha256=digest,source_collection_plan_sha256=collection_digest,analysis_plan_sha256=digest,selection_metric=LCB_METHOD,classes=classes,symbols=symbols,failed_jobs=failed,complete_jobs=len(rows)//len(plan['rules']),expected_jobs=dates*20,selected_from_development=selected,financial_pass=False,code_provenance=[{'source_algorithm_sha256':x,'replay_algorithm_sha256':y} for x,y in sorted(provenance)],original_goal='73% /1:1.38 per strategy/account unchanged; interim50% per class plus per-symbol checks')
    (out/'replays.json').write_text(json.dumps(rows));(out/'summary.json').write_text(json.dumps(summary,indent=2))
    esc=lambda x:html.escape(str(x));tr=[]
    for cls,rs in classes.items():
        for rule,v in rs.items():tr.append('<tr>'+''.join('<td>'+esc(x)+'</td>' for x in [cls,rule,v['episodes'],v['closed'],v['censored'],'N/A' if v['net_win_rate'] is None else f"{v['net_win_rate']:.1%}",v['mean_net_r'],v['lcb_net_r'],'screen only' if v['passes_screen'] else '; '.join(v['reasons'])])+'</tr>')
    detail=[]
    for key,v in symbols.items():detail.append('<tr>'+''.join('<td>'+esc(x)+'</td>' for x in [key,v['closed'],v['wins'],'N/A' if v['net_win_rate'] is None else f"{v['net_win_rate']:.1%}",v['mean_net_r']])+'</tr>')
    reasons=collections.Counter(m['reason'] for r in rows for m in r['study']['marks'])
    body=f'''<!doctype html><html lang="en"><meta charset="utf-8"><title>Jiaoyi class prediction</title><meta name="viewport" content="width=device-width,initial-scale=1"><style>body{{font:16px/1.5 system-ui;max-width:1250px;margin:30px auto;padding:20px}}table{{border-collapse:collapse;width:100%;font-size:14px}}td,th{{text-align:left;padding:9px;border-bottom:1px solid #ddd}}.scroll{{overflow:auto}}pre{{white-space:pre-wrap}}</style><h1>All-class predictive research — {a.stage}</h1><p>№18,278 §§1–5 · {dt.datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')} · by Codex (GPT-6), effort not exposed.</p><h2>TLDR</h2><p>{sum(v['passes_screen'] for rs in classes.values() for v in rs.values())} class/candidate preliminary screens passed; zero qualified financial passes. {summary['complete_jobs']}/{summary['expected_jobs']} expected source episodes reproduced offline. The program tests {len(plan['rules'])} frozen first-hour variants across all20 symbols; it does not promise50% or auto-promote a strategy.</p><p>Costs retain24bp per side and executable bid/ask; commission=0 is an unverified scenario. One simulated entry at hour1 after13causal observations; variants separately test fresh scheduled versus bounded as-of features, and high-efficiency continuation/reversal versus low-efficiency reversal. Execution freshness is unchanged. Original observed range fixesR; Q1.38 and existing trailing/target exits; force-close at hour4 or retain censoring. No live policy changed.</p><h2>Class results</h2><div class="scroll"><table><tr><th>Class</th><th>Candidate</th><th>Episodes</th><th>Closed</th><th>Censored</th><th>Net wins</th><th>Mean netR</th><th>LCB ranking heuristic</th><th>Gate</th></tr>{''.join(tr)}</table></div><h2>Every symbol</h2><div class="scroll"><table><tr><th>Symbol/candidate</th><th>Closed</th><th>Wins</th><th>Net win rate</th><th>Mean netR</th></tr>{''.join(detail)}</table></div><h2>Why a candidate did not trade</h2><pre>{esc(json.dumps(reasons,indent=2))}</pre><p>Counts include both candidate arms on shared episodes; they are not independent opportunities. The corpus is bounded and samples few dates. Price-only continuation/reversal tests do not include macro surprise vintages or order flow. No failure proves that every possible strategy in a class fails.</p><h2>Next gate and target</h2><p>Only a class passing development can open its separate-time candidate evaluation; rank passing candidates by the descriptive one-sided normal90% IID lower bound, then mean netR and rule name, once without retuning. This is a variance-sensitive ranking heuristic, not a confidence guarantee or a relaxation of sample-size gates. Validation needs≥20class closes,≥5per symbol, three weeks,≥50% net wins at class and symbol level, positive expectancy/PF and positive price-unit outcomes separately per symbol. A class aggregate cannot hide a losing member. Wilson intervals, censoring and failures remain in summaryJSON. Intervals are descriptive IID approximations; correlated symbols/dates reduce effective sample size. Current contract/quote continuity and actual fees remain unqualified, so even a passing screen is not authority to deploy.</p><p>Original goal73%/1:1.38 per strategy/account remains unchanged. Plan SHA256:{digest}. No separate model API spend. Draft for review.</p></html>'''
    (out/'Jiaoyi_Class_Prediction_Report.html').write_text(body)
    print(json.dumps({'complete_jobs':summary['complete_jobs'],'classes':classes,'failed_jobs':failed},indent=2))
def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['collect','analyse']);p.add_argument('--stage',choices=['development','validation'],default='development');p.add_argument('--output',type=Path,required=True);p.add_argument('--binary',type=Path);p.add_argument('--plan',type=Path,default=PLAN);p.add_argument('--max-new-jobs',type=int,default=40);p.add_argument('--base',default='https://jiaoyi.up.railway.app');a=p.parse_args()
    u=urlsplit(a.base)
    if u.scheme!='https' or u.username or u.password or u.path not in ('','/') or u.query or u.fragment:raise SystemExit('HTTPS origin without credentials/path required')
    if not 1<=a.max_new_jobs<=40:raise SystemExit('one invocation permits1..40jobs')
    a.base=a.base.rstrip('/');plan=json.loads(a.plan.read_text());digest=hashlib.sha256(a.plan.read_bytes()).hexdigest()
    if a.action=='collect':collect(a,plan,digest)
    else:
        if a.binary is None:raise SystemExit('--binary required for offline parity')
        analyse(a,plan,digest)
if __name__=='__main__':main()
