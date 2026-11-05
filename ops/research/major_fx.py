#!/usr/bin/env python3
"""Frozen major-FX market structure pilot. No signal or model API calls."""
import argparse, datetime as dt, hashlib, html, json, os, subprocess, time
from pathlib import Path
from zoneinfo import ZoneInfo
import broker_backtest as client
ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/major-fx-research-2026-11-05.json'

def requests(plan):
    for day in plan['dates']:
        for symbol in plan['symbols']:
            for window in plan['windows']:
                start=dt.datetime.combine(dt.date.fromisoformat(day),dt.time(window['hour']),ZoneInfo(window['tz'])).astimezone(dt.timezone.utc)
                req={'account':plan['account'],'symbol':symbol,'from':start.isoformat().replace('+00:00','Z'),'to':(start+dt.timedelta(hours=plan['hours'])).isoformat().replace('+00:00','Z'),'spread':0,'commission_round_trip':0,'slippage_bps_per_side':24,'tick_replay':{'volume':plan['native_volume'],'entry_rule':plan['mode']}}
                yield f"{day}/{symbol}/{window['name'].replace(' ','-')}",req

def collect(a,plan):
    password=os.environ['JIAOYI_UI_PASSWORD'];count=0
    for key,req in requests(plan):
        folder=a.output/key;folder.mkdir(parents=True,exist_ok=True);file=folder/'result.json'
        if file.exists():
            job=json.loads(file.read_text())
            if job.get('status')=='complete' and job.get('request')==req:continue
        if count>=a.max_new_jobs:break
        # Never replace another user's running job. Service also enforces single-flight.
        latest=client.call(a.base,password)
        if latest and latest.get('status')=='running':raise RuntimeError('Another broker research job is running; retry later')
        job=client.call(a.base,password,req);ident=job['id'];count+=1;deadline=time.monotonic()+270
        print('Collecting',key,ident,flush=True)
        while job['status']=='running' and time.monotonic()<deadline:
            time.sleep(2);job=client.call(a.base,password)
            if not job or job.get('id')!=ident:raise RuntimeError('Job replaced or service restarted; do not accept unrelated data')
        if job['status']=='running':raise RuntimeError('Deadline exceeded; check latest job before retrying')
        file.write_text(json.dumps(job));file.chmod(0o600)
        if job['status']!='complete':raise RuntimeError('Broker diagnostic failed: '+str(job.get('error')))
        print('Complete',key,'ticks',sum(job['result']['study']['quality'][k] for k in ('bid_ticks','ask_ticks')),flush=True)

def analyse(a,plan):
    jobs=[];expected=list(requests(plan));missing=[]
    for key,req in expected:
        file=a.output/key/'result.json'
        if not file.exists():missing.append(key);continue
        job=json.loads(file.read_text())
        if job.get('status')!='complete' or job.get('request')!=req:raise RuntimeError('Saved request/status differs from frozen plan')
        jobs.append(str(file))
    result=subprocess.run([str(a.binary)],input=json.dumps(jobs),text=True,capture_output=True,check=True)
    rows=json.loads(result.stdout)
    if len(rows)!=len(jobs):raise RuntimeError('Replay count mismatch')
    for row,path in zip(rows,jobs):
        j=json.loads(Path(path).read_text())
        if row['request']!=j['request'] or row['study']!=j['result']['study']:raise RuntimeError('Offline/deployed diagnostic mismatch')
        if row['study']['trade'] is not None or row['study']['marks']:raise RuntimeError('Diagnostic must not create trades')
    summary={'plan_sha256':hashlib.sha256(PLAN.read_bytes()).hexdigest(),'expected_jobs':len(expected),'complete_jobs':len(rows),'missing':missing,'offline_exact':True,'ticks':sum(r['study']['quality']['bid_ticks']+r['study']['quality']['ask_ticks'] for r in rows),'goal_status':'NOT TESTED: no FX strategy selected; zero qualifying account trades','rows':rows}
    a.output.mkdir(parents=True,exist_ok=True);(a.output/'summary.json').write_text(json.dumps(summary,indent=2))
    (a.output/'manifest.json').write_text(json.dumps(jobs,indent=2))
    esc=lambda x:html.escape(str(x));table=[]
    def fmt(v):return 'N/A' if v is None else f'{v:.3f}'
    for r in rows:
        d=r['study']['fx_diagnostic'];req=r['request']
        values=[req['symbol'],req['from'][:10],d['window'],f"{d['available_samples']}/49",fmt(d['sampled_spread_median_bps']),fmt(d['sampled_spread_p95_bps']),fmt(d['observed_mid_range_bps']),fmt(d['signed_move_bps']),fmt(d['five_minute_path_efficiency']),fmt(d['current_round_trip_reserve_ex_commission_bps'])]
        table.append('<tr>'+''.join('<td>'+esc(v)+'</td>' for v in values)+'</tr>')
    sources=''.join('<li><a href="'+esc(s['url'])+'">'+esc(s['title'])+'</a>: '+esc(s['use_and_limit'])+'</li>' for s in plan['sources'])
    now=dt.datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M SGT')
    content=f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Major FX: evidence before strategy</title><style>body{{font:16px/1.6 system-ui;max-width:1300px;margin:30px auto;padding:0 20px;color:#152b40}}table{{border-collapse:collapse;width:100%}}td,th{{padding:8px;border-bottom:1px solid #ccc;text-align:left}}.scroll{{overflow:auto}}.note{{padding:16px;background:#fff2de}}li{{margin:10px 0}}</style><h1>Major FX: evidence before strategy</h1><p>№18,245 §§1–7 · {now} · by &lt;Codex (GPT-6), effort: not exposed&gt;</p><h2>§1. TLDR</h2><p class="note">FX setup is unvalidated. The BTC/NatGas entry rules were not transplanted. This pilot measures seven major pairs across three local-clock windows on two dates: {len(rows)}/{len(expected)} completed, {summary['ticks']:,} bid/ask updates. No trade strategy was scored; the 50% target is NOT TESTED. Original 73% / 1:1.38 per-strategy/account goal remains; zero new qualifying account trades.</p><h2>§2. External evidence and implications</h2><ul>{sources}</ul><p>Retrieved 5 November 2026. Source titles were checked; unrelated search hits were excluded. Papers report findings in their own samples, not guaranteed contemporary CFD returns. The Breedon/Ranaldo source is the published abstract, not a reviewed full paper. No citation certifies Jiaoyi's setup.</p><h2>§3. What was wrong with assuming the existing setup</h2><ul><li>FX trades across regional centres; the current fixed 07:00 UTC S3 clock is not a universal market open. The diagnostic uses Tokyo 09:00, London 08:00 and New York 08:00 local time with DST. These windows are observation conventions, not proven optimal entry times.</li><li>24bp per side is 48bp round trip before spread and commission. It is an owner-inherited reserve, not a measured broker fee. Sparse current execution records cannot validate or replace it.</li><li>Tick history lacks consolidated volume, signed order flow and point-in-time news surprises. An LLM cannot infer those missing data reliably from a price chart.</li><li>Seven USD pairs are correlated exposures. Per-pair results cannot be added as independent portfolio bets.</li><li>Profitability requires direction, horizon, costs and execution to agree; stop/target tuning and win rate alone cannot establish an edge.</li></ul><h2>§4. Broker observations</h2><div class="scroll"><table><tr><th>Pair</th><th>Date UTC</th><th>Window</th><th>Fresh 5m samples</th><th>Median spread bp</th><th>P95 spread bp</th><th>Observed range bp</th><th>Signed move bp</th><th>Path efficiency</th><th>Current reserve bp excl. commission</th></tr>{''.join(table)}</table></div><p>Range is hindsight, not an achievable profit. Spread quantiles sample evenly in time rather than weighting busy tick periods. Missing any of 49 samples makes move/path efficiency unavailable. Native broker quantities and grid precision are checked; this is not account P&amp;L. Quiet/stale intervals and ambiguous same-millisecond order remain explicit in JSON. These two dates cannot establish session rankings, a reliable distribution or a trading edge.</p><h2>§5. Six gates before an FX strategy change</h2><ol>{''.join('<li>'+esc(g)+'</li>' for g in plan['next_gates'])}</ol><p>Research priority: first establish session/event-conditioned movement and executable costs; only then choose between continuation, mean reversion or abstention. Literature on cross-sectional momentum/carry does not establish an intraday breakout edge. Any exact thresholds or horizons remain preregistered hypotheses, not literature-certified constants.</p><h2>§6. Boundaries and remaining gaps</h2><ul>{''.join('<li>'+esc(g)+'</li>' for g in plan['limitations'])}</ul><p>Broker symbol identity is checked exactly. One bounded research job runs at a time; no new service, cron, model API, order writer or live rule change. Commission zero in the diagnostic request means excluded/unknown, never free trading. The range-to-reserve comparison describes the current assumption only.</p><h2>§7. Reproduction</h2><p>Plan SHA256: {summary['plan_sha256']}. Offline source-checksum verification and exact deployed-result comparison passed for {len(rows)} completed jobs. Missing jobs: {esc(', '.join(missing) or 'none')}. Replay with cargo example fx_session_study and manifest.json; original broker source jobs accompany the private archive. Draft for review.</p></html>'''
    (a.output/'Major_FX_Research_Report.html').write_text(content)
    print(json.dumps({k:v for k,v in summary.items() if k!='rows'},indent=2))

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['collect','analyse']);p.add_argument('--output',type=Path,required=True);p.add_argument('--max-new-jobs',type=int,default=4);p.add_argument('--binary',type=Path);p.add_argument('--base',default='https://jiaoyi.up.railway.app');a=p.parse_args()
    from urllib.parse import urlsplit
    u=urlsplit(a.base)
    if u.scheme!='https' or u.username or u.password or u.path not in ('','/') or u.query or u.fragment:raise SystemExit('Use an HTTPS origin without credentials/path/query')
    if not 1<=a.max_new_jobs<=42:raise SystemExit('max-new-jobs must be 1..42')
    a.base=a.base.rstrip('/');plan=json.loads(PLAN.read_text())
    if a.action=='collect':collect(a,plan)
    else:
        if a.binary is None:raise SystemExit('--binary is required for verified analysis')
        analyse(a,plan)
if __name__=='__main__':main()
