#!/usr/bin/env python3
"""№18,353: cached native cost-admission experiment; no broker imports."""
import argparse
import collections
import datetime as dt
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import sys
from option_a_vs_b_offline import verify_plan, accounting_check

ROOT=Path(__file__).resolve().parents[2]
PLAN=ROOT/'docs/native_target_sizing_offline_plan_18353.json'
RESULT=ROOT/'docs/native_target_sizing_offline_results_18353.json'
PRIOR_PLAN=ROOT/'docs/option_a_vs_b_offline_plan_18351.json'
PRIOR_RESULT=ROOT/'docs/option_a_vs_b_offline_results_18351.json'
OUT=Path('/workspace/jiaoyi-native-target-18353')
BINARY=ROOT/'target/debug/examples/native_target_sizing_replay'
ARMS=['arm_1_baseline_24bp_geometry','arm_2_native_3_74bp_geometry','arm_3_native_2_00bp_geometry','arm_4_symbol_mapped_native_geometry']
CODE=['ops/research/native_target_sizing.py','ops/research/test_native_target_sizing.py','ops/research/run_native_target_sizing_and_rebuild_18353.sh',
      'crates/jiaoyi-core/src/research/s3_ticks.rs','crates/jiaoyi-core/src/research/option_ab.rs','crates/jiaoyi-core/src/research/native_target_sizing.rs',
      'crates/jiaoyi-core/src/research/mod.rs','crates/jiaoyi-core/examples/native_target_sizing_replay.rs']

def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):Path(p).write_text(json.dumps(v,indent=2,allow_nan=False)+'\n')
def now():return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')
def cmd(*a):return subprocess.check_output(a,cwd=ROOT,text=True,timeout=120).strip()
def journal(section,text):
    p=ROOT/'log/CODEX_Log.txt';old=p.read_text();tag='№18,353 '+section
    if tag in old:return
    n=max(map(int,re.findall(r'\[§LOG\.(\d+)\]',old)),default=0)+1
    p.write_text(old+f'\n[§LOG.{n}] {tag} · {now()}; by <Codex (GPT-6), effort: not exposed>.\n{text}\n')

def prepare():
    if PLAN.exists():raise ValueError('plan already frozen')
    previous=read(PRIOR_PLAN);verify_plan(previous)
    prior=read(PRIOR_RESULT);basekeys=[r['key'] for r in prior['rows'] if r['models']]
    if len(previous['jobs'])!=159 or len(basekeys)!=8:raise ValueError('prior fixed scope changed')
    p=dict(trace='№18,353',frozen_at_sgt=now(),jobs=previous['jobs'],source_sha256=previous['source_sha256'],
           prior_plan_sha256=sha(PRIOR_PLAN),prior_results_sha256=sha(PRIOR_RESULT),baseline_episode_keys=basekeys,
           arms=ARMS,cost_bps=[24.,3.74,2.],mapped=dict(US500=2.,XAUUSD=2.,SpotCrude=3.74,SpotBrent=3.74),
           premise_correction='PlanBasis.r comes from rounded sigma half-width, not cost. Existing two-stage stop=-R, BE-trigger=+R, gross target=ceil-grid(1.38R), independent of C. Production TP1=ceil-grid(1.38R+C) is a different exit policy; its native distances are diagnostics, not substituted into this parity-bound experiment.',
           admission='Native cost reserve recomputed before decision and dispatch. Existing durable full-C>=R veto remains first; a separate R/entry*10000>=48 floor rejects low-range marks and rechecks dispatch without introducing a new session veto. One entry per session retained.',
           comparisons='Exact baseline parity with189 at every episode; baseline8 cohort, new episodes, same-entry versus changed-entry, loss/censor transitions and same-fill repricing control separately reported.',
           horizons='Existing observation boundaries:4h NY/gold; oil native quotes07-17UTC with evaluation13-17. Strict censor at boundary plus separate forced first fresh quote<=5s after boundary, existing SL/TP first.',
           spread_and_costs='Same causal prepared spread for admission, recorded bid/ask for fills. Spread charged once. Net subtracts (entry+exit)*arm_bps/10000/R; zero extra commission inherited and unverified. 2/3.74bp are assumptions, not calibrated truths.',
           stop_rule='Evaluate exactly159 immutable cached episodes once per frozen arm; no adaptive parameter search or new market reads.',
           flags='Keep missing preparation, same-millisecond ordering limits, censoring and prior20 protocol-flagged episodes. Reuse is not untouched validation.',
           new_broker_requests=0,validation_opened=False,live_rules_changed=False,financial_pass=False,
           rebuild_scope='Independent RFC only; no service rewrite or live migration authorised by this research plan.')
    save(PLAN,p);journal('§§1–6','Frozen159 local sources/four native-cost arms. Correct false premise: two-stage R/BE/target independent of C at fixed entry; native production target diagnostics separate. Full cost gate then48bp decision/dispatch floor. Requested LOG131 occupied; unique journal index retained.')
    print('PLAN '+sha(PLAN))

def freeze(p):
    verify_plan(p)
    if sha(PRIOR_PLAN)!=p['prior_plan_sha256'] or sha(PRIOR_RESULT)!=p['prior_results_sha256']:raise ValueError('prior evidence changed')
    paths=sorted((ROOT/'crates/jiaoyi-core/src').rglob('*.rs'))+[ROOT/'Cargo.lock',PLAN]+[ROOT/n for n in CODE]+[ROOT/'ops/research/option_a_vs_b_offline.py']
    hashes={str(x.relative_to(ROOT)):sha(x) for x in paths}
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)).hexdigest()!=h:raise ValueError('commit frozen source before evaluating')
    value=dict(plan_sha256=sha(PLAN),source_sha256=hashes,binary_sha256=sha(BINARY),commit=cmd('git','rev-parse','HEAD'))
    f=OUT/'freeze.json'
    if f.exists():
        old=read(f)
        if any(old[k]!=v for k,v in value.items()):raise ValueError('frozen run drift')
        return old
    value['frozen_at_sgt']=now();save(f,value);return value

def identity(outcome):
    if not outcome:return None
    i=outcome['entry'];return tuple(i[k] for k in ['entered_at','side','fill','initial_r','volume'])

def observation(arm,mode):
    o=arm.get(mode)
    if not o:return 'no_entry',None
    if not o.get('metrics'):return 'unresolved',None
    v=o['metrics']['net_by_bps'][f'{arm["bps"]:g}']
    return ('win' if v>1e-12 else 'loss' if v< -1e-12 else 'net_flat'),v

def score(rows,arm,mode,control=False):
    values=[];states=collections.Counter();g=[]
    for row in rows:
        a=row['arms'][arm]
        if control:a=dict(a,**{mode:row['arms'][ARMS[0]].get(mode)})
        state,v=observation(a,mode);states[state]+=1
        if v is not None:
            values.append(v);g.append(a[mode]['metrics']['gross_r'])
    gain=sum(v for v in values if v>0);loss=-sum(v for v in values if v<0)
    return dict(episodes=len(rows),entries=len(rows)-states['no_entry'],closed=len(values),wins=states['win'],losses=states['loss'],
        net_flat=states['net_flat'],unresolved=states['unresolved'],no_entry=states['no_entry'],
        win_rate=states['win']/len(values) if values else None,mean_net_r=statistics.mean(values) if values else None,
        mean_gross_r=statistics.mean(g) if g else None,profit_factor=gain/loss if loss else None,
        profit_factor_status='finite' if loss else 'no_losses' if gain else 'unavailable',financial_pass=False)

def run():
    def guard(event,args):
        if event in ['socket.connect','socket.getaddrinfo']:raise RuntimeError('offline run prohibits network')
    sys.addaudithook(guard);OUT.mkdir(exist_ok=True);p=read(PLAN);provenance=freeze(p)
    prior={r['key']:r for r in read(PRIOR_RESULT)['rows']};rows=[];checks=0
    for n,j in enumerate(p['jobs'],1):
        raw=subprocess.check_output([str(BINARY),j['path']],text=True,cwd=ROOT,timeout=90)
        f=OUT/('replay-'+j['key']+'.json');f.write_text(raw);native=json.loads(raw)['result'];arms=native['arms']
        baseline=arms[ARMS[0]];old=prior[j['key']]
        for mode,previous in [('strict','s3_breakout_4h'),('forced','s3_breakout_4h_mtm')]:
            if baseline.get(mode)!=old['models'].get(previous):raise ValueError('baseline path parity failed: '+j['key']+' '+mode)
        if baseline['marks']!=old['admission_reasons']:raise ValueError('baseline mark parity failed')
        relations={}
        for name,a in arms.items():
            checks+=accounting_check({'models':{k:a[k] for k in ['strict','forced'] if a[k]}})
            b=baseline['strict'];v=a['strict']
            relation='neither' if not b and not v else 'new_episode' if not b else 'removed_entry' if not v else 'same_entry' if identity(b)==identity(v) else 'changed_entry'
            transitions={m:observation(baseline,m)[0]+'->'+observation(a,m)[0] for m in ['strict','forced']}
            geom=None
            if relation=='same_entry':
                fields=['initial_stop_distance','breakeven_trigger_distance','two_stage_target_distance']
                geom=all(a['geometry'][k]==baseline['geometry'][k] for k in fields)
                if not geom:raise ValueError('same-entry two-stage geometry drift')
            if v and a['geometry']['r_bps']<48.-1e-10:raise ValueError('floor violated')
            relations[name]=dict(relation=relation,transitions=transitions,same_entry_geometry_unchanged=geom)
        source=read(j['path'])
        row={k:j[k] for k in ['key','symbol','date','cohort','protocol_flags']}
        row.update(arms=arms,relations=relations,quality=native['quality'],m30_bars=len(source['payload']['m30']),bar_component_failures=source.get('bar_component_failures',0),
                   baseline_cohort=j['key'] in p['baseline_episode_keys'],file_sha256=j['sha256'],native_sha256=sha(f),baseline_parity=True)
        rows.append(row)
        if n%20==0 or n==159:print(f'REPLAY {n}/159',flush=True)
    groups={'all159':rows,'baseline8':[r for r in rows if r['baseline_cohort']],'outside_baseline8':[r for r in rows if not r['baseline_cohort']]}
    for s in sorted({r['symbol'] for r in rows}):groups[s]=[r for r in rows if r['symbol']==s]
    for c in sorted({r['cohort'] for r in rows}):groups[c]=[r for r in rows if r['cohort']==c]
    summary={g:{a:{m:score(rs,a,m) for m in ['strict','forced']} for a in ARMS} for g,rs in groups.items()}
    control={a:{m:score(groups['baseline8'],a,m,True) for m in ['strict','forced']} for a in ARMS}
    comparison={a:dict(relations=dict(collections.Counter(r['relations'][a]['relation'] for r in rows)),
       transitions={m:dict(collections.Counter(r['relations'][a]['transitions'][m] for r in rows if r['baseline_cohort'])) for m in ['strict','forced']},
       minimum_admitted_r_bps=min((r['arms'][a]['geometry']['r_bps'] for r in rows if r['arms'][a]['strict']),default=None),
       admitted_keys=[r['key'] for r in rows if r['arms'][a]['strict']]) for a in ARMS}
    verify_plan(p)
    result=dict(trace='№18,353',generated_at_sgt=now(),plan_sha256=sha(PLAN),provenance=provenance,rows=rows,summary=summary,
       same_baseline_entry_repricing_control=control,comparisons=comparison,
       verification=dict(baseline_strict_and_forced_parity_episodes=len(rows),independent_cost_checks=checks,source_hashes_reverified=True),
       premise_correction=p['premise_correction'],new_broker_requests=0,validation_opened=False,financial_pass=False,live_rules_changed=False,
       goal=dict(original_win_rate=.73,reference_reward_r=1.38,window=20,scope='each strategy/account',independent_observations_added=0,account_goal_qualified=False))
    save(RESULT,result);journal('§§7–12','Completed159 native four-arm evaluations; exact189 strict/forced/mark parity, '+str(checks)+' independent cost checks. '+json.dumps(summary['all159'])+'\nAll admissions retain48bp floor. Source flags and unresolved trades retained; zero broker calls/live changes. RFC follows measured outcomes.')
    print(json.dumps({'summary':summary['all159'],'comparisons':comparison},indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=['prepare','run']);a=p.parse_args()
    prepare() if a.action=='prepare' else run()
