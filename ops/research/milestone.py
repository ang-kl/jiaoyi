#!/usr/bin/env python3
"""Bounded preregistered collection + honest per-symbol milestone scoring. No model API."""
import argparse,datetime as dt,hashlib,html,json,math,os,subprocess,time,urllib.error
from pathlib import Path
from zoneinfo import ZoneInfo
import broker_backtest as client
from research_gates import DATA_GAP_REASONS, LCB_METHOD, data_gap_counts, lcb_net_r
ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/research-50pct-full-ticks-2026-11-05.json'
SYMBOLS={'BTCUSD':('btc',15,1,0),'NatGas':('natgas',.004,10000,7)}
RULES=('current_s3','opening_range_breakout2h','opening_range_failed_breakout2h','opening_tick_range_breakout2h','opening_tick_range_failed_breakout2h')
# Serde's rename_all snake_case keeps the numeric suffix adjacent to the word.
def dates(plan,stage,symbol):
    start,end=map(dt.date.fromisoformat,plan[stage]);out=[]
    while start<=end:
        if symbol=='BTCUSD' or start.weekday()<5:out.append(start)
        start+=dt.timedelta(days=1)
    return out

def score(rows,minimum,expected,stage):
    gaps=data_gap_counts(rows)
    trades=[r['study']['trade'] for r in rows if r['study']['trade'] is not None]
    values=[t['trailing_scenario_net_r'] for t in trades if t['trailing_scenario_net_r'] is not None]
    price_values=[t['trailing_scenario_net_r']*t['entry']['initial_r'] for t in trades if t['trailing_scenario_net_r'] is not None]
    price_profit=sum(x for x in price_values if x>0);price_loss=-sum(x for x in price_values if x<0)
    censored=len(trades)-len(values);n=len(values);wins=sum(x>0 for x in values);loss=-sum(x for x in values if x<0);profit=sum(x for x in values if x>0)
    rate=wins/n if n else None;mean=sum(values)/n if n else None
    pf=profit/loss if loss else None
    weeks=len({dt.date.fromisoformat(r['from'][:10]).isocalendar()[:2] for r in rows if r['study']['trade'] and r['study']['trade']['trailing_scenario_net_r'] is not None})
    lower=upper=None
    if n:
        z=1.95996398454;den=1+z*z/n;centre=(rate+z*z/(2*n))/den;half=z*math.sqrt(rate*(1-rate)/n+z*z/(4*n*n))/den;lower=centre-half;upper=centre+half
    reasons=[]
    if gaps:reasons.append(f'data coverage gaps ({gaps})')
    if len(rows)!=expected:reasons.append('missing episode coverage')
    if len({r['from'][:10] for r in rows})!=len(rows):reasons.append('duplicate episode dates')
    if n<minimum:reasons.append(f'fewer than {minimum} closed scenarios')
    if censored:reasons.append('censored outcomes retained; cannot pass')
    if rate is None or rate<.5:reasons.append('net win rate below 50% or unavailable')
    if mean is None or mean<=0:reasons.append('net expectancy not positive')
    if profit<=loss:reasons.append('profit factor not above 1')
    if not price_values or sum(price_values)<=0:reasons.append('total net price-unit P&L not positive')
    if stage=='validation' and weeks<3:reasons.append('fewer than three calendar weeks')
    return {'episodes':len(rows),'expected_episodes':expected,'entries':len(trades),'closed':n,'censored':censored,'wins':wins,'net_win_rate':rate,'win_rate_wilson95':[lower,upper],'net_r':sum(values) if n else None,'mean_net_r':mean,'lcb_net_r':lcb_net_r(values),'lcb_method':LCB_METHOD,'data_gaps':gaps,'profit_factor':pf,'net_price_units':sum(price_values) if price_values else None,'price_unit_profit_factor':price_profit/price_loss if price_loss else None,'profit_factor_note':'no gross losses' if n and not loss else None,'weeks':weeks,'passes_screen':not reasons,'reasons':reasons,'qualification':'research scenarios; not account-goal trades or proof of edge'}

def normalise(req):
    copy=json.loads(json.dumps(req))
    copy.get("tick_replay",{}).pop("entry_rule",None)
    return copy

def collect(a,plan):
    password=os.environ.get('JIAOYI_UI_PASSWORD')
    if not password:raise SystemExit('JIAOYI_UI_PASSWORD required; never put it in a file')
    symbols=list(SYMBOLS)
    if a.stage=='validation':
        summary=json.loads((a.output/'development'/'summary.json').read_text())
        if summary['plan_sha256']!=hashlib.sha256(PLAN.read_bytes()).hexdigest():raise SystemExit('Development plan hash mismatch')
        symbols=[s for s in symbols if any(v['passes_screen'] for k,v in summary['scores'].items() if k.startswith(s+':') and not k.endswith(':current_s3'))]
        if not symbols:raise SystemExit('No development candidate passed. Validation remains unopened; investigate before registering another trial.')
    fetched=0
    for symbol in symbols:
        name,spread,volume,hour=SYMBOLS[symbol]
        for day in dates(plan,a.stage,symbol):
            folder=a.output/a.stage/name/day.isoformat();folder.mkdir(parents=True,exist_ok=True)
            start=dt.datetime.combine(day,dt.time(hour),dt.timezone.utc)
            req={'account':49236051,'symbol':symbol,'from':start.isoformat().replace('+00:00','Z'),'to':(start+dt.timedelta(hours=4)).isoformat().replace('+00:00','Z'),'spread':spread,'commission_round_trip':0,'slippage_bps_per_side':24,'tick_replay':{'volume':volume,'entry_rule':'opening_range_breakout2h'}}
            if a.legacy_fetch:req['tick_replay'].pop('entry_rule')
            saved=folder/'result.json'
            if saved.exists():
                j=json.loads(saved.read_text())
                if j.get('status')=='complete' and normalise(j.get('request',{}))==normalise(req):print('Cached',symbol,day,flush=True);continue
            if fetched>=a.max_new_jobs:return
            fetched+=1
            print('Fetching',symbol,day,flush=True)
            try:
                initial=client.call(a.base,password,req);ident=initial['id'];deadline=time.monotonic()+270
                while time.monotonic()<deadline:
                    time.sleep(2);j=client.call(a.base,password)
                    if not j or j.get('id')!=ident:raise RuntimeError('research job replaced/restarted')
                    if j['status']!='running':break
                else:raise RuntimeError('poll deadline exceeded')
                saved.write_text(json.dumps(j));saved.chmod(0o600)
                if j['status']!='complete':raise RuntimeError(j.get('error','job failed'))
                print('Complete',symbol,day,'ticks',sum(j['result']['study']['quality'][k] for k in ['bid_ticks','ask_ticks']),flush=True)
            except (urllib.error.HTTPError,RuntimeError) as e:
                why=f'HTTP {e.code}' if isinstance(e,urllib.error.HTTPError) else str(e)
                (folder/'failure.json').write_text(json.dumps({'request':req,'failure':why}));print('Failure',symbol,day,why,flush=True)

def analyse(a,plan):
    jobs=[]
    for symbol,(name,*_) in SYMBOLS.items():
        for day in dates(plan,a.stage,symbol):
            p=a.output/a.stage/name/day.isoformat()/'result.json'
            if p.exists() and json.loads(p.read_text()).get('status')=='complete':jobs.append(str(p))
    manifest={'jobs':jobs,'bars_jobs':{s:str(a.output/(v[0]+'-m30')/'result.json') for s,v in SYMBOLS.items()}}
    out=a.output/a.stage;out.mkdir(parents=True,exist_ok=True);(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    result=subprocess.run([str(a.binary)],input=json.dumps(manifest),text=True,capture_output=True,check=True)
    rows=json.loads(result.stdout);(out/'replays.json').write_text(result.stdout)
    for row in rows:
        if row['symbol'] not in SYMBOLS or dt.date.fromisoformat(row['from'][:10]) not in dates(plan,a.stage,row['symbol']):raise RuntimeError('Replay outside frozen stage/universe')
    prior=None
    if a.stage=='validation':
        prior=json.loads((a.output/'development'/'summary.json').read_text())
        if prior['plan_sha256']!=hashlib.sha256(PLAN.read_bytes()).hexdigest():raise RuntimeError('Development plan hash mismatch')
    scores={}
    for symbol in SYMBOLS:
        for rule in RULES:
            subset=[r for r in rows if r['symbol']==symbol and r['rule']==rule]
            scores[symbol+':'+rule]=score(subset,5 if a.stage=='development' else 20,len(dates(plan,a.stage,symbol)),a.stage)
            if prior and rule!='current_s3' and not prior['scores'][symbol+':'+rule]['passes_screen']:
                scores[symbol+':'+rule]['passes_screen']=False;scores[symbol+':'+rule]['reasons'].append('development gate did not pass')
    summary={'stage':a.stage,'plan_sha256':hashlib.sha256(PLAN.read_bytes()).hexdigest(),'scores':scores,'original_goal':'73% net wins and 1:1.38 per strategy/account; unchanged','interim_goal':'50% scenario net wins plus positive expectancy/PF>1; financial goal remains unverified'}
    (out/'summary.json').write_text(json.dumps(summary,indent=2))
    esc=lambda x:html.escape(str(x));table=[]
    for key,v in scores.items():
        rate='N/A' if v['net_win_rate'] is None else f"{v['net_win_rate']:.1%}"
        table.append('<tr>'+''.join('<td>'+esc(x)+'</td>' for x in [key,v['episodes'],v['entries'],v['closed'],v['censored'],rate,v['mean_net_r'],v['profit_factor'],v['net_price_units'],v['price_unit_profit_factor'],'PASS SCREEN' if v['passes_screen'] else '; '.join(v['reasons'])])+'</tr>')
    now=dt.datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M SGT')
    content=f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi 50% research milestone</title><style>body{{font:16px/1.5 system-ui;max-width:1300px;margin:30px auto;padding:0 20px}}table{{border-collapse:collapse;width:100%}}td,th{{border-bottom:1px solid #ccd;padding:10px;text-align:left}}.scroll{{overflow:auto}}pre{{white-space:pre-wrap}}</style><h1>50% interim research target — {a.stage}</h1><p>№18,243 §§1–5 · {now} · by &lt;Codex (GPT-6), effort: not exposed&gt;</p><h2>§1. TLDR</h2><p>{'A candidate passed this preliminary screen; independent validation is still required.' if any(v['passes_screen'] for k,v in scores.items() if not k.endswith(':current_s3')) else 'No candidate passed this screen. The 50% milestone is not established.'} This is a frozen development experiment, not a live account result. No strategy was promoted.</p><h2>§2. All results</h2><div class="scroll"><table><tr><th>Symbol / rule</th><th>Episodes</th><th>Entries</th><th>Closed</th><th>Censored</th><th>Net wins</th><th>Mean net R</th><th>PF(R)</th><th>Net price units</th><th>PF(price units)</th><th>Verdict</th></tr>{''.join(table)}</table></div><h2>§3. Interpretation</h2><p>Baseline retains the current S3 session end and may remain censored at four hours. New candidates explicitly close their four-hour episode; this is a different hypothesis, not full-session baseline parity. Missing episodes and censored positions prevent a pass. Break-even closes stay in the denominator. All candidates and losses are retained.</p><p>24bp/side and spread/commission scenarios are unchanged. They are not certified historical execution costs. Quotes do not establish fills or within-millisecond ordering. Fixed minimum native volume bypasses historical account sizing and portfolio conflicts only in this isolated research scenario.</p><h2>§4. Goal and uncertainty</h2><p>The original goal remains 73% net wins and 1:1.38 per strategy/account. Research contributes zero qualifying account trades. Development uses a minimum five closed scenarios only as a screening threshold; validation requires at least20 closes across three weeks, ≥50% net wins, positive mean net R, positive total net price-unit P&L and PF>1. Wilson95% intervals, descriptive normal90% IID lower-bound ranking scores and raw outcomes are in the JSON; these are not confidence guarantees for correlated, selected samples; observed50% is not proof that the true win rate exceeds50%.</p><h2>§5. Reproducibility and next gate</h2><p>Frozen plan SHA256: {summary['plan_sha256']}. Each replay retains original tick/M30 checksums and a combined-source checksum. Validation dates remain fixed; no rule is retuned on them. Previously inspected baseline dates are declared in the plan; this is not a pristine holdout.</p><p>Draft for review.</p></html>'''
    (out/'report.html').write_text(content)
    print(json.dumps(scores,indent=2))

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['collect','analyse']);p.add_argument('--stage',choices=['development','validation'],required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--binary',type=Path);p.add_argument('--legacy-fetch',action='store_true');p.add_argument('--max-new-jobs',type=int,default=100);p.add_argument('--base',default='https://jiaoyi.up.railway.app');a=p.parse_args()
    from urllib.parse import urlsplit
    u=urlsplit(a.base)
    if u.scheme!='https' or u.username or u.password or u.query or u.fragment or u.path not in ('','/'):raise SystemExit('Use an HTTPS origin without credentials/path/query')
    if not 1<=a.max_new_jobs<=100:raise SystemExit('max-new-jobs must be1..100')
    a.base=a.base.rstrip('/');plan=json.loads(PLAN.read_text())
    if a.action=='collect':collect(a,plan)
    else:
        if a.binary is None:raise SystemExit('--binary must name the compiled s3_candidate_study example')
        analyse(a,plan)
if __name__=='__main__':main()
