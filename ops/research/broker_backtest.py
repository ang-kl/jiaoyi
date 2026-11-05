#!/usr/bin/env python3
"""Submit/poll a bounded Jiaoyi broker-data study and save JSON + HTML. No model API."""
import argparse,html,json,os,time,urllib.request,urllib.error
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self,*args,**kwargs):
        raise RuntimeError('Redirect refused; credentials were not forwarded')

def call(base,password,body=None):
    payload=None if body is None else json.dumps(body).encode()
    headers={'Authorization':'Bearer '+password,'Cache-Control':'no-cache'}
    if payload is not None:headers['Content-Type']='application/json'
    req=urllib.request.Request(base+'/owner/backtests',data=payload,headers=headers)
    with urllib.request.build_opener(NoRedirect).open(req,timeout=25) as res:raw=res.read(32_000_001)
    if len(raw)>32_000_000:raise RuntimeError('Response exceeds 32MB')
    return json.loads(raw)

def report(job):
    if job["request"].get("tick_replay") is not None:
        return tick_report(job)
    result=job.get('result') or {};study=result.get('study') or {};req=job['request'];groups=defaultdict(list)
    for signal in study.get('signals',[]):
        for x in signal['outcomes']:
            if x['net_bps'] is not None:groups[x['minutes']].append(x)
    rows=[]
    for minutes,out in sorted(groups.items()):
        n=len(out);gross=sum(x['gross_bps'] for x in out)/n;net=sum(x['net_bps'] for x in out)/n;wins=sum(x['net_bps']>0 for x in out)
        rows.append(f'<tr><td>{minutes}</td><td>{n}</td><td>{gross:.3f}</td><td>{net:.3f}</td><td>{wins/n:.1%}</td></tr>')
    esc=lambda x:html.escape(str(x))
    notes=''.join('<li>'+esc(n)+'</li>' for n in study.get('notes',[]))
    return f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi {esc(req['symbol'])} broker-data study</title><style>body{{font:16px/1.6 system-ui;max-width:1050px;margin:30px auto;padding:0 20px}}table{{border-collapse:collapse;width:100%}}td,th{{padding:8px;text-align:left;border-bottom:1px solid #ccc}}pre{{white-space:pre-wrap}}.note{{padding:16px;background:#eef3f8}}</style><h1>{esc(req['symbol'])}: broker-data signal study</h1><p>№18,238 / study {esc(job["id"])} · §1–4 · {datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M SGT')} · by &lt;Codex (GPT-6), effort: not exposed&gt;</p><h2>§1. TLDR</h2><p class="note">{esc(job['status'])}. This study measures the original S3 signal at fixed horizons using broker M30 bars. It does not simulate live order execution or certify an edge. Goal status: NOT VERIFIED; overlapping signal outcomes are not the latest 20 qualified account trades.</p><p>Account {esc(req['account'])}; {esc(req['from'])} to {esc(req['to'])}. Strategy session: {esc(study.get('session','unavailable'))}.</p><h2>§2. Results</h2><table><tr><th>Horizon minutes</th><th>Complete outcomes</th><th>Mean gross bp</th><th>Mean net bp</th><th>Positive net outcomes</th></tr>{''.join(rows)}</table><p>Session-end horizons are included and deduplicated when equal to 30/60/120 minutes. All qualifying marks are included, not just cost-admitted entries. Horizons overlap.</p><h2>§3. Coverage and assumptions</h2><p>Bars: {esc(study.get('input_bars'))}; eligible marks: {esc(study.get('eligible_marks'))}; unavailable signal/band marks: {esc(study.get('missing_bands'))}; missing session opens: {esc(study.get('missing_session_opens'))}.</p><pre>{esc(json.dumps(req,indent=2))}</pre><ul>{notes}</ul><h2>§4. Reproducibility</h2><p>Input SHA-256: {esc(result.get('input_sha256','unavailable'))}. Broker symbol ID: {esc(result.get('symbol_id'))}. Full input bars and per-signal outcomes are retained in the accompanying JSON. Failure: {esc(job.get('error','none'))}.</p><p>Draft for review.</p></html>'''

def tick_report(job):
    esc=lambda x:html.escape(str(x))
    result=job.get('result') or {};study=result.get('study') or {};req=job['request'];trade=study.get('trade')
    notes=''.join('<li>'+esc(n)+'</li>' for n in study.get('notes',[]))
    rows=''.join('<tr>'+''.join('<td>'+esc(m.get(k))+'</td>' for k in ['at','reason','side','r','cost'])+'</tr>' for m in study.get('marks',[]))
    arms=[]
    if trade:
        for key,label,net in [('baseline','Fixed stop',trade.get('fixed_stop_scenario_net_r')),('candidate','Current S3 trailing',trade.get('trailing_scenario_net_r'))]:
            arm=trade['exit_comparison'][key]
            arms.append('<h3>'+label+'</h3><p>Scenario net R: '+esc(net)+'; remaining native volume: '+esc(arm['remaining_volume'])+'. Qualified net R: '+esc(arm['net_r'])+'.</p><pre>'+esc(json.dumps(arm['fills'],indent=2))+'</pre>')
    return f'''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{esc(req['symbol'])} bounded tick replay</title><style>body{{font:16px/1.6 system-ui;max-width:1100px;margin:30px auto;padding:0 20px}}table{{border-collapse:collapse;width:100%}}td,th{{padding:8px;text-align:left;border-bottom:1px solid #ccc}}pre{{white-space:pre-wrap;overflow-wrap:anywhere}}.note{{padding:16px;background:#eef3f8}}.scroll{{overflow:auto;max-height:500px}}</style><h1>{esc(req['symbol'])}: bounded tick replay</h1><p>№18,241 · §1–5 · {datetime.now(ZoneInfo('Asia/Singapore')).strftime('%d-%m-%Y %H:%M SGT')} · by &lt;Codex (GPT-6), effort: not exposed&gt;</p><h2>§1. TLDR</h2><p class="note">{esc(job['status'])}. {'One simulated entry.' if trade else 'No simulated entry.'} This is a bounded development quote-path scenario. Goal: risk:reward 1:1.38 and ≥73% net wins per strategy/account over 20 qualified positions. Goal progress: NOT VERIFIABLE; this run adds zero qualifying account trades.</p><p>Account {esc(req['account'])}; {esc(req['from'])} to {esc(req['to'])}. Failure: {esc(job.get('error','none'))}.</p><h2>§2. Admission and exits</h2><div class="scroll"><table><tr><th>UTC mark</th><th>Reason</th><th>Side</th><th>R distance</th><th>Cost reserve</th></tr>{rows}</table></div><p>Null outcomes mean incomplete or unavailable, never zero profit.</p>{''.join(arms)}<h2>§3. Data quality</h2><pre>{esc(json.dumps(study.get('quality',{}),indent=2))}</pre><p>Even a complete API fetch cannot prove executable fills or cross-side sequencing. Bid/ask spread is included in fills; scenario commission and adverse slippage are deducted separately.</p><h2>§4. Assumptions</h2><pre>{esc(json.dumps(req,indent=2))}</pre><ul>{notes}</ul><h2>§5. Reproducibility</h2><p>Job {esc(job['id'])}; broker symbol {esc(result.get('symbol_id'))}; tick requests {esc(result.get('tick_pages'))}; input SHA-256 {esc(result.get('input_sha256'))}; algorithm SHA-256 {esc(result.get('algorithm_sha256'))}.</p><p>The JSON contains source bars, both tick streams, current contract snapshot and every outcome. No live rule was changed. Draft for review.</p></html>'''

def main():
    p=argparse.ArgumentParser();p.add_argument('request',type=Path);p.add_argument('--output',type=Path,required=True);p.add_argument('--base',default='https://jiaoyi.up.railway.app');a=p.parse_args()
    from urllib.parse import urlsplit
    url=urlsplit(a.base)
    if url.scheme!='https' or url.username or url.password or url.query or url.fragment or url.path not in ('','/'):
        raise SystemExit('Use an HTTPS origin without credentials, path, query or fragment')
    password=os.environ.get('JIAOYI_UI_PASSWORD')
    if not password:raise SystemExit('JIAOYI_UI_PASSWORD is required; never put it in the request file')
    req=json.loads(a.request.read_text());base=a.base.rstrip('/');start=call(base,password,req);identifier=start['id'];deadline=time.monotonic()+270
    print('Submitted job',identifier,flush=True)
    while time.monotonic()<deadline:
        time.sleep(2);job=call(base,password)
        if not job or job.get('id')!=identifier:raise RuntimeError('Job lost/replaced or service restarted; no result accepted')
        if job['status']!='running':break
    else:raise RuntimeError('Polling deadline exceeded; check latest job before retrying')
    a.output.mkdir(parents=True,exist_ok=True)
    for name,content in [('result.json',json.dumps(job,indent=2)),('report.html',report(job))]:
        target=a.output/name;target.write_text(content);target.chmod(0o600)
    print('Status:',job['status'],'; saved',a.output/'report.html')
    if job['status']!='complete':raise SystemExit('Research failed: '+job.get('error','unknown'))
if __name__=='__main__':
    try:main()
    except urllib.error.HTTPError as exc:raise SystemExit(f'HTTP {exc.code}: request refused; no credentials logged')
