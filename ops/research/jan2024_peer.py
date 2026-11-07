#!/usr/bin/env python3
"""Exactly ten January2024 peer jobs; cached primaries; no model tuning/orders."""
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
import statistics
import subprocess
import time

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks
from class_batch import digest, receipt, save
from seasonal_pilot import bar_request, decode_bars, instant, stamp, protected
from run_repaired_offline_replay import score, without_notes, ci_state
from peer_collection_eval import completed, interval_overlap, latest20, net_verify, now

ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/jan2024-peer-plan-18342.json'
RESULT=ROOT/'docs/jan2024_peer_parity_results_18342.json'
REPORT=ROOT/'doc/JAN2024_PEER_PARITY_REPORT-18342-2026-11-07.md'
BASE=ROOT/'docs/offline-repaired-engine-results-18331.json'
MAY=ROOT/'docs/clean_peer_collection_results_18338.json'
MANIFEST=ROOT/'docs/three-year-execution-18324.json'
CORPUS=Path('/workspace/jiaoyi-three-year-pilot-18325')
BINARY=ROOT/'target/debug/examples/offline_repaired_replay'
BRANCH='eval/jan2024-peer-parity-18342'
DATES=[f'2024-01-{d}' for d in range(22,27)]
PRIMARY=['US500','XAUUSD','SpotCrude']
PEERS={'US500':'NAS100','XAUUSD':'XAGUSD','SpotCrude':'SpotBrent'}
MODELS=['s3_two_stage','macro_180m_two_stage','always_long_control','own_momentum_fixed_horizon']
CODE=['ops/research/jan2024_peer.py','ops/research/test_jan2024_peer.py','ops/research/run_jan2024_peer_runner.sh']


def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def cmd(*a):return subprocess.check_output(a,cwd=ROOT,text=True,timeout=120).strip()
def gh(*a):return json.loads(cmd('gh',*a))


def journal(section,message):
    path=ROOT/'log/CODEX_Log.txt';text=path.read_text();tag=f'№18,342 {section}'
    if tag in text:return
    n=max(map(int,re.findall(r'\[§LOG\.(\d+)\]',text)),default=0)+1
    path.write_text(text+f'\n[§LOG.{n}] {tag} · {now()}; by <Codex (GPT-6), effort: not exposed>.\n'+message+'\n')


def prepare():
    if cmd('git','branch','--show-current')!=BRANCH:raise ValueError('create task branch before preparing')
    if PLAN.exists():return json.loads(PLAN.read_text())
    baseline=json.loads(BASE.read_text());meta=json.loads((ROOT/'docs/class-batch-40-plan-18309-2026-11-07.json').read_text())['metadata']
    manifest=json.loads(MANIFEST.read_text());original={j['key']:j for j in manifest['jobs']}
    rows={r['key']:r for r in baseline['rows']};sources={};jobs=[];evaluations=[]
    for date in DATES:
        for symbol in PRIMARY:
            key=symbol+'-'+date;j=original[key];row=rows[key];path=CORPUS/('job-'+key+'.json')
            if sha(path)!=row['file_sha256']:raise ValueError('primary checksum differs from baseline')
            sources[str(path)]=sha(path)
            evaluations.append(dict(key=key,symbol=symbol,date=date,asset_class=j['asset_class'],
                                    request=j['request'],source_path=str(path),status=row['status'],protocol_flags=row.get('protocol_flags',[])))
            if symbol!='SpotCrude':
                peer=PEERS[symbol];m=meta[peer]
                jobs.append(dict(key=peer+'-'+date,symbol=peer,primary=key,date=date,symbol_id=m['symbol_id'],
                    contract={k:m[k] for k in ['digits','min_volume','step_volume']},
                    **{'from':j['request']['from'],'to':j['request']['to']}))
    prior_manifests={};spent_ticks=[];spent_bars=[]
    for name in ['docs/seasonal-execution-18315.json','docs/three-year-execution-18324.json',
                 'docs/class-batch-40-plan-18309-2026-11-07.json','docs/peer-collection-plan-18338.json']:
        path=ROOT/name;prior_manifests[name]=sha(path);p=json.loads(path.read_text())
        for j in p['jobs']:
            if j.get('mode')=='missing':continue
            req=j['request'];spent_ticks.append(dict(symbol_id=j['symbol_id'],**{'from':j.get('source_from',req['from']),'to':stamp(instant(req['to'])+dt.timedelta(seconds=5,milliseconds=1))}))
        spent_bars.extend(p.get('bar_segments',[]))
    for j in jobs:
        window=dict(j,**{'to':stamp(instant(j['to'])+dt.timedelta(seconds=5,milliseconds=1))})
        if any(interval_overlap(window,p) for p in spent_ticks) or any(interval_overlap(j,p) for p in spent_bars):raise ValueError('peer interval was already spent')
        if protected(instant(j['from']),instant(window['to'])):raise ValueError('sealed validation')
    p=dict(trace='№18,342 §§1–6',prepared_at_sgt=now(),by='Codex (GPT-6), effort not exposed',dates=DATES,
           source_attachment_sha256='bedd4bf3a0589eb11e9f0c5e108a0e57eb596d366d817a76e29b326c38b2ac68',
           jobs=jobs,evaluations=evaluations,source_sha256=sources,prior_manifests=prior_manifests,
           evidence_sha256={str(p.relative_to(ROOT)):sha(p) for p in [BASE,MAY]},
           limits=dict(peer_jobs=10,primary_slots=15,pages_per_job=120,rows_per_job=200000,seconds_per_job=240,
                       m30_requests=10,attempts_per_job=1,total_source_bytes=96000000,bytes_per_file=32000000),
           rules='Existing offline_repaired model unchanged. Peer confirmation affects macro only; S3 parity required.',
           costs=dict(primary_bps_per_side=24,extra_commission=0,calibration='unverified',spread='executable bid/ask, not double charged'),
           cumulative='Union prior №18,331 and №18,338 plus current S3 identities; conflicts stop publication; no duplicate January wins',
           known_failed_primary='US500-2024-01-24',known_source_protocol_flags='All five Crude sources inherit prior failed-M30 refetch flags',
           observations='Previously studied development dates; equal date labels do not prove quote continuity or exact timestamp parity',
           validation_opened=False,financial_pass=False,production_changed=False)
    PLAN.write_text(json.dumps(p,indent=2)+'\n')
    journal('§§1–6','Frozen10 NAS100/XAGUSD jobs, each tick sides plus boundedM30;15 cached primary slots.\nUS50024Jan failure and Crude protocol flags retained. Existing Rust rules unchanged; cumulative S3 deduplicated.\nAttachment truncated and hard-coded outcomes rejected. No simulated_ok, primary reread or financial PASS.')
    return p


def provenance(out,p):
    if len(p['jobs'])!=10 or len({j['key'] for j in p['jobs']})!=10 or len(p['evaluations'])!=15:raise ValueError('scope changed')
    for name,h in {**p['source_sha256'],**p['prior_manifests'],**p['evidence_sha256']}.items():
        path=Path(name) if name.startswith('/') else ROOT/name
        if sha(path)!=h:raise ValueError('frozen evidence changed')
    files=list((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',PLAN,ROOT/'crates/jiaoyi-core/examples/offline_repaired_replay.rs']
    files += [ROOT/x for x in CODE]+[ROOT/'ops/research'/x for x in ['broker_readonly.py','class_batch.py','seasonal_pilot.py','run_repaired_offline_replay.py','peer_collection_eval.py']]
    hashes={str(f.relative_to(ROOT)):sha(f) for f in files}
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit before broker reads')
    current=dict(plan_sha256=sha(PLAN),source_sha256=hashes,binary_sha256=sha(BINARY))
    target=out/'freeze.json'
    if target.exists():
        old=json.loads(target.read_text())
        if any(old[k]!=v for k,v in current.items()):raise ValueError('frozen code changed')
        return old
    if (out/'ledger.jsonl').exists():raise ValueError('receipts without pre-read freeze')
    current.update(commit=cmd('git','rev-parse','HEAD'),frozen_at_sgt=now());save(target,current);return current


def collect(out,p):
    done=completed(out);expected={'peer-'+j['key'] for j in p['jobs']}
    if not set(done)<=expected:raise ValueError('unknown receipt')
    if set(done)==expected:return
    with Broker() as broker:
        catalog=broker.request(2114,dict(ctidTraderAccountId=ACCOUNT,includeArchivedSymbols=False))
        details=broker.request(2116,dict(ctidTraderAccountId=ACCOUNT,symbolId=sorted({j['symbol_id'] for j in p['jobs']})))
        for j in p['jobs']:
            names=[s for s in catalog['symbol'] if s['symbolName']==j['symbol']]
            c=next(s for s in details['symbol'] if s['symbolId']==j['symbol_id'])
            if len(names)!=1 or names[0]['symbolId']!=j['symbol_id'] or any(c[k]!=j['contract'][v] for k,v in [('digits','digits'),('minVolume','min_volume'),('stepVolume','step_volume')]):raise ValueError('broker metadata changed')
        if not (out/'metadata.json').exists():save(out/'metadata.json',dict(at_sgt=now(),details=details))
        for j in p['jobs']:
            key='peer-'+j['key'];path=out/(key+'.json')
            if key in done:continue
            if path.exists():raise ValueError('unreceipted data; no repeat')
            receipt(out/'ledger.jsonl',dict(key=key,state='started',at_sgt=now(),plan_sha256=sha(PLAN)))
            budget=dict(pages=0,rows=0)
            try:
                a=int(instant(j['from']).timestamp()*1000);b=int(instant(j['to']).timestamp()*1000)+5000;deadline=time.monotonic()+240
                bids=ticks(broker,j['symbol_id'],1,a,b,budget,deadline);asks=ticks(broker,j['symbol_id'],2,a,b,budget,deadline)
                raw=broker.request(2137,bar_request(j));bars=decode_bars(raw,j)
                value=dict(status='complete',bids=bids,asks=asks,bars=bars)
            except Exception as error:
                value=dict(status='failed',error=str(error) if isinstance(error,BrokerError) else type(error).__name__)
            value.update(request=j,budget_used=budget,attempts=1)
            raw=json.dumps(value,separators=(',',':')).encode();used=sum(f.stat().st_size for f in out.glob('peer-*.json'))
            if len(raw)>32000000 or used+len(raw)>96000000:raise ValueError('byte budget; uncertain receipt retained')
            save(path,value);receipt(out/'ledger.jsonl',dict(key=key,state='finished',status=value['status'],sha256=sha(path),at_sgt=now()))
            print(json.dumps(dict(job=j['key'],status=value['status'],error=value.get('error'),pages=budget['pages'])),flush=True)
            if value.get('error') in {'TypeError','KeyError','ValueError','AttributeError','NameError','AssertionError','CH_ACCESS_TOKEN_INVALID','CH_TOKEN_EXPIRED','NOT_AUTHORIZED'}:raise ValueError('unexpected failure retained; stop')


def overlay(source,peer):
    """Only the peer fields change; primary preparation, quotes and request are frozen."""
    result=copy.deepcopy(source);req=result['request'];start=int(instant(req['from']).timestamp()*1000)
    if peer['request']['from']!=req['from'] or peer['request']['to']!=req['to']:raise ValueError('peer/primary interval mismatch')
    result['payload']['peer_symbol']=peer['request']['symbol']
    result['payload']['peer_m30']=[b for b in peer.get('bars',[]) if start<=b['ts_ms'] and b['ts_ms']+1800000<=start+10800000] if peer['status']=='complete' else []
    result['payload_sha256']=digest(result['payload']);return result


def cumulative(groups):
    unique={};duplicates=0
    for trace,rows in groups:
        for row in rows:
            m=row.get('models',{}).get('s3_two_stage',{});trade=m.get('study',{}).get('trade')
            if not trade:continue
            entry=trade['entry'];outcome=trade['exit_comparison']['candidate'];net=m['sensitivities']['trailing']['24']
            key=(row['symbol'],entry['entered_at'],entry['side'])
            fingerprint=digest(without_notes(dict(entry=entry,outcome=outcome,net=net)))
            if key in unique:
                if unique[key]['fingerprint']!=fingerprint:raise ValueError('conflicting outcomes for same S3 trade identity')
                unique[key]['traces'].append(trace);duplicates+=1;continue
            unique[key]=dict(fingerprint=fingerprint,traces=[trace],row=row,net=net)
    rows=[v['row'] for v in unique.values()];summary=score(rows,'s3_two_stage','24')
    return dict(summary=summary,latest20=latest20(rows,'s3_two_stage'),duplicate_observations_removed=duplicates,
                unique_trades=[dict(symbol=v['row']['symbol'],key=v['row']['key'],traces=v['traces'],net_r=v['net'],
                                    protocol_flags=v['row'].get('protocol_flags',[])) for v in unique.values()],
                qualification='pooled reused development evidence, not the per-strategy/account goal')


def replay(out,p):
    baseline=json.loads(BASE.read_text());previous={r['key']:r for r in baseline['rows']};rows=[];checks=parity=0
    for j in p['evaluations']:
        row={k:j[k] for k in ['key','symbol','date','asset_class','status','protocol_flags']};row['models']={}
        source=json.loads(Path(j['source_path']).read_text());row['primary_source_sha256']=sha(Path(j['source_path']))
        if source['status']!='complete':row['error']=source.get('error','retained_failed_primary')
        else:
            req=source['request']
            if req!=j['request'] or source['payload_sha256']!=digest(source['payload']):raise ValueError('primary payload identity mismatch')
            if j['symbol']!='SpotCrude':
                path=out/('peer-'+PEERS[j['symbol']]+'-'+j['date']+'.json');peer=json.loads(path.read_text())
                source=overlay(source,peer);row['peer_source_sha256']=sha(path);row['peer_source_status']=peer['status']
            work=out/'native-input.json';save(work,source)
            proc=subprocess.run([str(BINARY),str(work)],capture_output=True,text=True,timeout=90);work.unlink()
            if proc.returncode:row.update(status='replay_failed',error=proc.stderr[:250])
            else:
                native=json.loads(proc.stdout)
                if native['payload_sha256']!=source['payload_sha256']:raise ValueError('native hash mismatch')
                result=native['result'];models=result['models'];old=previous[j['key']]['models']['s3_two_stage']
                if without_notes(models['s3_two_stage'])!=without_notes(old):raise ValueError('S3 changed when only peer data were added')
                parity+=1
                row['models']=dict(s3_two_stage=models['s3_two_stage'],macro_180m_two_stage=models['macro_180m_repaired'],
                    always_long_control=result['baseline']['models']['always_long_control'],
                    own_momentum_fixed_horizon=result['baseline']['models']['own_momentum_fixed_horizon'])
                checks+=net_verify(row['models']);row['peer']=result['peer'];row['quality']=result['baseline']['quality']
                row['window_quality']=result['baseline']['window_quality'];row['payload_sha256']=source['payload_sha256']
                save(out/('replay-'+j['key']+'.json'),native);row['replay_sha256']=sha(out/('replay-'+j['key']+'.json'))
        rows.append(row);print(json.dumps(dict(evaluated=len(rows),key=j['key'],status=row['status'])),flush=True)
    peer_files=[out/('peer-'+j['key']+'.json') for j in p['jobs']];sources=[json.loads(path.read_text()) for path in peer_files]
    peer_quality=[]
    for value in sources:
        j=value['request'];q=dict(key=j['key'],status=value['status'],primary=j['primary'])
        if value['status']=='complete':
            a=int(instant(j['from']).timestamp()*1000)
            q.update(bid_ticks=len(value['bids']),ask_ticks=len(value['asks']),m30_bars=len(value['bars']),
                six_required_bar_times_present=all(a+i*1800000 in {b['ts_ms'] for b in value['bars'] if b['volume']>0} for i in range(6)),
                max_bid_gap_ms=max((b['ts_ms']-a['ts_ms'] for a,b in zip(value['bids'],value['bids'][1:])),default=None),
                max_ask_gap_ms=max((b['ts_ms']-a['ts_ms'] for a,b in zip(value['asks'],value['asks'][1:])),default=None))
        peer_quality.append(q)
    groups={'overall':rows}
    for s in PRIMARY:groups[s]=[r for r in rows if r['symbol']==s]
    summary={g:{m:score(rs,m,'24') for m in MODELS} for g,rs in groups.items()}
    old_rows=[]
    for j in p['evaluations']:
        old=copy.deepcopy(previous[j['key']]);old.setdefault('models',{})['macro_180m_two_stage']=old.get('models',{}).get('macro_180m_repaired',{})
        old_rows.append(old)
    prior_macro=score(old_rows,'macro_180m_two_stage','24')
    cumulative_s3=cumulative([('№18,331',baseline['rows']),('№18,338',json.loads(MAY.read_text())['rows']),('№18,342',rows)])
    result=dict(trace='№18,342',generated_at_sgt=now(),provenance=json.loads((out/'freeze.json').read_text()),dates=DATES,rows=rows,
        collection=dict(peer_jobs_attempted=len(sources),peer_jobs_complete=sum(s['status']=='complete' for s in sources),
            peer_jobs_with_both_tick_sides=sum(bool(s.get('bids')) and bool(s.get('asks')) for s in sources),
            tick_page_requests=sum(s['budget_used']['pages'] for s in sources),m30_requests=sum(s['status']=='complete' for s in sources),
            source_bytes=sum(path.stat().st_size for path in peer_files)),
        peer_quality=peer_quality,summary=summary,prior_macro_same_slots=prior_macro,cumulative_s3=cumulative_s3,
        verification=dict(s3_unchanged_slots=parity,independent_net_values=checks,primary_slots=15,
            primary_failed_slots=sum(r['status']=='failed' for r in rows),source_protocol_flagged_slots=sum(bool(r['protocol_flags']) for r in rows)),
        temporal_parity='Requested windows matched; actual bar availability and quote gaps reported per slot. No zero-gap or identical-tick-timestamp claim.',
        financial_pass=False,validation_opened=False,production_changed=False,stage2_authorized=False,receipts=completed(out))
    save(RESULT,result);report(result)
    journal('§§7–10','Completed '+json.dumps(result['collection'])+'\nS3 unchanged in'+str(parity)+' replayed slots; primary failure and all source flags retained.\nMeasured January macro: '+json.dumps(summary['overall']['macro_180m_two_stage'])+'\nDeduplicated cumulative S3: '+json.dumps(cumulative_s3['summary'])+'\nNo financial PASS, production activation or validation opening; no fixed-count wins invented.')
    return result


def report(r):
    macro=r['summary']['overall']['macro_180m_two_stage'];cum=r['cumulative_s3']['summary'];collection=r['collection']
    def fmt(v):return 'unavailable' if v is None else f'{v:.6f}'
    text=f'''# January 2024 peer collection and measured evaluation

№18,342 §§1–10 · {r['generated_at_sgt']} · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**January macro: {macro['wins']}/{macro['closed_trades']} net wins, mean {fmt(macro['mean_net_r'])}R, PF(R) {fmt(macro['profit_factor'])}.**

**Cumulative S3 after deduplication: {cum['closed_trades']}/20 closed-trade milestone; {cum['wins']} wins, {cum['losses']} losses, mean {fmt(cum['mean_net_r'])}R.** This pooled reused-development count is not the per-strategy/account73% / reference risk:reward1:1.38 goal. Qualified progress towards that goal remains unproven. No financial PASS or Stage2 promotion.

Ten peer jobs attempted; {collection['peer_jobs_complete']} completed, {collection['peer_jobs_with_both_tick_sides']} contain both tick sides. {r['verification']['s3_unchanged_slots']}/15 primary slots replayed. **US50024January remains a retained failed source**; it was not recollected. All five Crude slots retain inherited failed-M30-refetch protocol flags. Existing primary prices, preparation and Rust strategy rules were unchanged.

## §2. Measured pair results at24bp/side

| Primary / peer | Slots | Entries | Closes | Wins | Losses | Censored | Mean net R | PF(R) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
'''
    for s in PRIMARY:
        m=r['summary'][s]['macro_180m_two_stage'];text+='| '+' | '.join(str(v) for v in [s+'/'+PEERS[s],m['episodes'],m['entries'],m['closed_trades'],m['wins'],m['losses'],m['censored_or_unavailable'],fmt(m['mean_net_r']),fmt(m['profit_factor'])])+' |\n'
    text+='\n### Matched controls and prior macro\n\n| Model | Entries | Closed | Wins | Mean net R | PF(R) |\n| --- | --- | --- | --- | --- | --- |\n'
    for n,m in [('prior_macro_same_slots',r['prior_macro_same_slots']),*r['summary']['overall'].items()]:
        text+=f"| {n} | {m['entries']} | {m['closed_trades']} | {m['wins']} | {fmt(m['mean_net_r'])} | {fmt(m['profit_factor'])} |\n"
    text+=f'''
## §3. What changed, and what did not

Only the missing NAS100 and XAGUSD peer observations were added to US500/gold replay payloads. Existing Crude/Brent inputs were reused. Macro requires six completed M30 bars in the same causal window; S3 has no peer filter. Exact S3 model-output parity passed for all{r['verification']['s3_unchanged_slots']} replayed primary slots. Adding peers therefore did **not** manufacture new S3 trades.

The submitted text was truncated mid-review-block and assigned all wins, losses, class rows and returns as constants. Its collector arguments were unsupported; failure became`simulated_ok`. Those statements were replaced by actual sources and native fills. A1.024R loss is not a24bp-per-side model: cost/R depends on entry/exit prices and original risk. An entry-price stop is gross-flat, usually net-negative after costs.

The existing strict C<R screen, +1R next-valid-quote stop amendment, gross+1.38R target, broker bracket priority, native deadlines and censoring remain. Bid/ask includes spread once; assumed24bp per side is charged to actual fills, with the existing zero additional commission assumption. These costs and historical contract terms remain uncalibrated; no claim of realistic verified total costs is made.

## §4. Data continuity and cumulative accounting

The10 peer jobs align requested windows to the primary windows (US indices14:30UTC in January; metals07:00UTC). Each job performs bounded bid/ask pagination and one bounded M30 read. Ten logical jobs are not ten API messages. Matching dates/windows does not mean identical tick timestamps or zero quote gaps. The JSON retains actual peer bar coverage, bid/ask counts, maximum within-side gaps, native quality counts, failed primary slots and receipts.

Cumulative S3 combines the measured №18,331 and №18,338 results with this run, keyed by symbol, entry time and side. Repeated identities require identical entry/exit/net outcomes; disagreements stop publication. **{r['cumulative_s3']['duplicate_observations_removed']} duplicate observation(s) removed.** Earlier losses and protocol flags remain. There is no automatic addition of the two trades asserted in the supplied script.

## §5. Reproduction and next decision

[Frozen plan](../docs/jan2024-peer-plan-18342.json) · [Measured JSON](../docs/jan2024_peer_parity_results_18342.json). The JSON contains source/code/binary hashes, per-slot models, source failures, cumulative identities, controls and timing evidence. Raw sources remain outside Git and PostgreSQL. There were no new primary reads, paid model calls, live orders, risk changes or worker changes. Validation19August–5September2025 stays sealed.

Assess the measured before/after macro results and net costs before proposing another bounded experiment. Peer availability is an engineering improvement, not proof of an entry edge. Do not extrapolate a small pooled sample to the account goal or expand sampling until a favourable win count appears.
'''
    REPORT.write_text(text)


def publish(out):
    if cmd('git','branch','--show-current')!=BRANCH:raise ValueError('publication requires task branch')
    r=json.loads(RESULT.read_text());paths=CODE+[str(x.relative_to(ROOT)) for x in [PLAN,RESULT,REPORT]]+['log/CODEX_Log.txt']
    if not set(cmd('git','diff','--cached','--name-only').splitlines())<=set(paths):raise ValueError('unrelated staged files')
    cmd('git','diff','--check');cmd('git','add','--',*paths)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:cmd('git','commit','-m','research: measured January peer evaluation and deduplicated S3 progress (№18,342 §§1–10)')
    head=cmd('git','rev-parse','HEAD');hashes={p:sha(ROOT/p) for p in paths}
    cmd('git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',BRANCH)
    body=out/'pr-body.md';m=r['summary']['overall']['macro_180m_two_stage'];c=r['cumulative_s3']['summary']
    body.write_text(f'№18,342 §§1–10 · {now()} · by Codex (GPT-6), effort not exposed.\n\nThe supplied script hard-coded profits and converted failed collection into success. This runner collects exactly10 January2024 NAS100/XAGUSD jobs, reuses primary sources and measures the unchanged native model. US50024Jan failure and Crude protocol flags remain. It adds no live strategy change.\n\nActual macro: {m["wins"]}/{m["closed_trades"]} wins, mean {m["mean_net_r"]}R, PF(R) {m["profit_factor"]}. Deduplicated cumulative S3: {c["wins"]}/{c["closed_trades"]} wins, mean {c["mean_net_r"]}R. No financial qualification or Stage2.\n\nValidation: focused deduplication/conflict/interval-overlay tests, unchanged existing native peer/exit tests,14-slot S3 parity and independent cost arithmetic. Required CI must pass on this head.\n\nReport: `{REPORT.relative_to(ROOT)}`.\n')
    prs=gh('pr','list','--repo','ang-kl/jiaoyi','--head',BRANCH,'--state','open','--json','url')
    url=prs[0]['url'] if prs else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',BRANCH,'--title','Research: measured January peer evaluation (№18,342 §§1–10)','--body-file',str(body))
    print('REPORT_PR '+url,flush=True);number=url.rsplit('/',1)[-1];deadline=time.monotonic()+1800
    save(out/'publication.json',dict(url=url,head=head,artifact_sha256=hashes))
    while True:
        pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup')
        if pr['state']!='OPEN' or pr['headRefOid']!=head:raise ValueError('PR changed')
        state=ci_state(pr['statusCheckRollup']);print('CI '+state,flush=True)
        if state=='failed':raise ValueError('CI failed; no merge')
        if state=='success':break
        if time.monotonic()>deadline:raise ValueError('CI timeout; no merge')
        time.sleep(15)
    cmd('gh','pr','merge',number,'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr=gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,mergeCommit,mergedAt')
    if pr['state']!='MERGED':raise ValueError('merge unconfirmed')
    cmd('git','fetch','origin','main')
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('main hash mismatch')
    block=dict(trace='№18,342',status='jan2024_peer_evaluation_completed',pr_url=url,pr_merge_commit=pr['mergeCommit']['oid'],
        frozen_plan_sha256=sha(PLAN),collection=r['collection'],macro_180m_jan2024=m,s3_two_stage_cumulative=c,
        source_limits=r['verification'],duplicate_s3_observations_removed=r['cumulative_s3']['duplicate_observations_removed'],
        financial_pass=False,validation_opened=False,report_file=str(REPORT.relative_to(ROOT)),verified_at_sgt=now())
    save(out/'review-block.json',block);update=out/'pr-update.json'
    save(update,dict(body=body.read_text()+'\nVerified on main:\n```json\n'+json.dumps(block,indent=2)+'\n```\n'))
    cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(update),'--jq','.html_url')
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('action',choices=['prepare','run','publish','all']);ap.add_argument('--output',type=Path,default=Path('/workspace/jiaoyi-jan2024-peer-18342'));args=ap.parse_args()
    if cmd('git','branch','--show-current')!=BRANCH:
        if RESULT.exists():print('This experiment is already recorded. Read '+str(REPORT.relative_to(ROOT))+'; no collection repeated.');return
        if cmd('git','diff','--name-only','HEAD'):raise ValueError('preserve tracked edits before creating task branch')
        cmd('git','fetch','origin','main');cmd('git','switch','-c',BRANCH,'origin/main')
    out=args.output;out.mkdir(parents=True,exist_ok=True)
    if args.action in ['prepare','all']:
        p=prepare();print('PLAN '+json.dumps(p['limits']),flush=True)
        if args.action=='prepare':return
        subprocess.run(['python3','-m','unittest','discover','-s','ops/research','-p','test_jan2024_peer.py'],cwd=ROOT,check=True)
        subprocess.run(['cargo','build','-p','jiaoyi-core','--example','offline_repaired_replay','--locked'],cwd=ROOT,check=True)
        files=CODE+[str(PLAN.relative_to(ROOT)),'log/CODEX_Log.txt']
        if not set(cmd('git','diff','--cached','--name-only').splitlines())<=set(files):raise ValueError('unrelated staged edits')
        cmd('git','add','--',*files)
        if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:cmd('git','commit','-m','research: freeze ten January peer jobs and unchanged model (№18,342 §§1–6)')
    if args.action in ['run','all']:
        p=json.loads(PLAN.read_text())
        with (out/'run.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);provenance(out,p);collect(out,p);completed(out);replay(out,p);provenance(out,p)
    if args.action in ['publish','all']:publish(out)


if __name__=='__main__':main()
