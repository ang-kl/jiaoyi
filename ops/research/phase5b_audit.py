#!/usr/bin/env python3
"""№18,366. Exact frozen-code replay and source audit; no collection capability."""
import argparse
import collections
import copy
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from replay_inventory import ROOT,CORE,read,save,sha,canonical,ms,stamp,market,permitted
import unified_replay as u
from tiered_replay import CleanStore,tier2_input,paired
from phase4_forensics import gates,missing_runs

WORK=Path('/workspace/jiaoyi-rebuild-phase5b-18366')
CACHE5=Path('/workspace/jiaoyi-rebuild-phase5-18364')
FROZEN=Path('/workspace/jiaoyi-frozen-phase4-18366')
COMMIT='a0d7aea148e73550cc63993228b4ef9b368f0c43'
SPEC=ROOT/'docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json'
SCOPE=ROOT/'docs/rebuild_phase5b_scope_18366.json'
RESULT=ROOT/'docs/rebuild_phase5b_exact_frozen_candidate_audit_18366.json'
MANIFEST=ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json'
P3=ROOT/'docs/rebuild_phase3_coverage_and_pullback_results_18360.json'
P5PLAN=ROOT/'docs/rebuild_phase5_independent_evaluation_plan_18364.json'
P5RESULT=ROOT/'docs/rebuild_phase5_independent_evaluation_results_18364.json'
BINS=['tiered_pullback_replay','manifest_pullback_replay','native_target_sizing_replay','replay_inventory_metadata']
HELPERS=['ops/research/unified_replay.py','ops/research/tiered_replay.py','ops/research/replay_inventory.py','ops/research/phase4_forensics.py']
PEERS={a:b for pair in read(SPEC)['frozen_candidate_hypothesis']['peer_pairs'] for a,b in [pair,list(reversed(pair))]}

def now():return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()

def phase5_receipts():
    p=read(P5PLAN);r=read(P5RESULT);events=[json.loads(s) for s in (CACHE5/'ledger.jsonl').read_text().splitlines()]
    starts=[e['key'] for e in events if e['state']=='started'];ends={e['key']:e for e in events if e['state']=='finished'}
    expected={'bar-'+s['key'] for s in p['bar_segments'] if s['mode']=='collect'}|{'job-'+j['key'] for j in p['jobs']}
    if len(starts)!=len(set(starts)) or set(starts)!=set(ends) or set(ends)!=expected:raise ValueError('Phase5 ledger mismatch')
    if {k:v['sha256'] for k,v in ends.items()}!=r['collection_receipt_hashes']:raise ValueError('Phase5 publication receipts changed')
    for key,v in ends.items():
        if sha(CACHE5/(key+'.json'))!=v['sha256']:raise ValueError('Phase5 source hash mismatch')
    return p,r,ends

def exclusion_audit(p,r,ends):
    selected={e['key'] for e in r['rows']};excluded={e['key']:e for e in r['coverage']['excluded']};rows=[]
    for j in p['jobs']:
        old=excluded.get(j['key']);rec=ends['job-'+j['key']]
        state='evaluated' if j['key'] in selected else 'completed_but_excluded' if rec['status']=='complete' else 'tick_source_failed'
        rows.append({'episode_id':'phase5-'+j['key'],'key':j['key'],'symbol':j['symbol'],'date':j['date'],'cohort':j['cohort'],
            'state':state,'source_status':rec['status'],'receipt_sha256':rec['sha256'],'reasons':old['coverage']['reasons'] if old else [],'prior_coverage':old['coverage'] if old else None})
    completed_excluded=[e for e in rows if e['state']=='completed_but_excluded']
    counts=collections.Counter(tuple(e['reasons']) for e in completed_excluded)
    if len(rows)!=160 or len(selected)!=62 or len(completed_excluded)!=96 or sum(e['state']=='tick_source_failed' for e in rows)!=2:raise ValueError('Phase5 denominator mismatch')
    if any(len(k)!=1 for k in counts):raise ValueError('unexpected overlapping completed-source exclusions')
    return {'scheduled':160,'completed_tick_sources':158,'evaluated_in_pr195':62,'completed_but_excluded':96,'failed_tick_sources':2,
        'exclusive_completed_source_reasons':{k[0]:v for k,v in counts.items()},'new_m30_receipts':239,'reused_m30_slices':75,'reused_unique_files':len({s['cache']['path'] for s in p['bar_segments'] if s['mode']=='reuse'}),'rows':rows}

def combined_manifest(p,ends):
    m=copy.deepcopy(read(MANIFEST));original_ids={e['episode_id'] for e in m['episodes']}
    for j in p['jobs']:
        rec=ends['job-'+j['key']];path=CACHE5/('job-'+j['key']+'.json');raw=read(path)
        if raw['request']['account']!=49236051 or raw['request']['symbol']!=j['symbol']:raise ValueError('Phase5 account/symbol mismatch')
        e={'episode_id':'phase5-'+j['key'],'account_id':49236051,'symbol':j['symbol'],'date':j['date'],'asset_class':j['asset_class'],'cohort':j['cohort'],
            'source_path':str(path),'source_window_utc':{'from':j['from'],'to':j['to']},'tick_receipt_sha256':rec['sha256'],'receipt_sha256':rec['sha256'],
            'eligible_for_research':rec['status']=='complete','protocol_status':'clean_first_read' if rec['status']=='complete' else 'failed_tick_source',
            'protocol_flags':[],'source_status':rec['status'],'native_volume':j['native_volume'],'contract':j['contract'],'symbol_id':j['symbol_id'],'m30_embedded_count':0,
            'origin':'phase5_new_batch','independence':'already_evaluated_changed_candidate_dates_not_frozen_phase4_test_dates'}
        if e['episode_id'] in original_ids:raise ValueError('duplicate episode ID')
        m['episodes'].append(e)
    for part in p['bar_segments']:
        if part['mode']!='collect':continue
        rec=ends['bar-'+part['key']];path=CACHE5/('bar-'+part['key']+'.json')
        if rec['status']!='complete':raise ValueError('unanticipated failed M30 source')
        m['m30_segments'].append({'account_id':49236051,'symbol':part['symbol'],'symbol_id':part['symbol_id'],'source_path':str(path),
            'receipt_sha256':rec['sha256'],'eligible_for_research':True,'source_window_utc':{'from':part['from'],'to':part['to']},
            'receipt_sources':[{'kind':'collection_ledger','origin':str(CACHE5/'ledger.jsonl'),'sha256':rec['sha256']}]})
    return m

def metadata(m):
    items=[{'key':e['episode_id'],'symbol':e['symbol'],'class':e['asset_class'],'date':e['date']} for e in m['episodes']]
    return json.loads(subprocess.check_output([str(WORK/'bin/replay_inventory_metadata')],input=json.dumps(items),text=True,timeout=120))

def window_check(e,m):
    s=m['session']
    if s is None:return {'covers':False,'reason':'no_coded_session'}
    a,b=ms(e['source_window_utc']['from']),ms(e['source_window_utc']['to']);start=ms(s['open']);end=start+14400000
    return {'covers':a<=start and b>=end,'source_from':stamp(a),'source_to':stamp(b),'native_from':stamp(start),'native_to':stamp(end),
        'missing_open_minutes':max(0,min(a,end)-start)/60000,'missing_end_minutes':max(0,end-max(b,start))/60000,
        'overlap_minutes':max(0,min(b,end)-max(a,start))/60000,'native_calendar':m['profile']}

def choose_peer(e,episodes):
    opts=[x for x in episodes if x['account_id']==e['account_id'] and x['symbol']==PEERS[e['symbol']] and x['date']==e['date'] and (x['eligible_for_research'] and x['protocol_status']=='clean_first_read' or x.get('origin')=='phase5_new_batch')]
    return min(opts,key=lambda x:x['episode_id']) if opts else None

def force_planned_peer_failure(inp,peer):
    if inp is not None and peer is not None and peer.get('source_status')=='failed':
        inp['peer_context']={'status':'planned_peer_source_failed','drift_5':None,'drift_20':None,'latest_bar_end':None}
        inp['peer_available']=True
    return inp

def select(m,store,meta,joins):
    candidates=[];register=[]
    for e in m['episodes']:
        if e['symbol'] not in CORE:continue
        row={k:e[k] for k in ['episode_id','account_id','symbol','date','cohort','source_path','tick_receipt_sha256','protocol_status']}
        row['origin']=e.get('origin','prior_manifest');row['asset_class']=e['asset_class'];row['window']=window_check(e,meta[e['episode_id']])
        if not e['eligible_for_research']:
            row['excluded_reason']='failed_tick_source' if e.get('source_status')=='failed' else 'quarantined_protected_dependency';register.append(row);continue
        if e['protocol_status']!='clean_first_read':row['excluded_reason']='receipt_provenance_not_clean_first_read';register.append(row);continue
        peer=choose_peer(e,m['episodes']);join=joins[u.CostKey(e['account_id'],e['symbol'])]
        inp,cov=u.qualified_input(e,store,meta,peer,join);row['tier1_coverage']=cov
        tier='tier1'
        if inp is None:
            try:inp,cov2=tier2_input(e,store,meta,peer,join)
            except ValueError as ex:
                if 'protected endpoint' not in str(ex):raise
                inp=None;cov2={'reasons':['protected_context_endpoint']}
            row['tier2_coverage']=cov2;tier='tier2'
        if inp is None:row['excluded_reason']='no_qualified_native_window';register.append(row);continue
        inp=force_planned_peer_failure(inp,peer)
        row.update(tier=tier,peer_episode_id=peer['episode_id'] if peer else None,own_context=inp['own_context'],peer_context=inp['peer_context'])
        candidates.append((e,inp,row));register.append(row)
    selected=[];used={}
    for e,inp,row in sorted(candidates,key=lambda v:(v[2]['tier'],-(ms(v[0]['source_window_utc']['to'])-ms(v[0]['source_window_utc']['from'])),v[0]['tick_receipt_sha256'])):
        key=(e['account_id'],e['symbol'],e['date'])
        if key in used:row.update(excluded_reason='same_session_alias',canonical_episode_id=used[key]);row.pop('tier');continue
        used[key]=e['episode_id'];selected.append((e,inp,row))
    return selected,register

def immutable_save(path,value):
    if path.exists() and read(path)!=value:raise ValueError('immutable artifact changed: '+str(path))
    if not path.exists():save(path,value)

def sources_and_selection():
    u.BIN=WORK/'bin'
    spec=read(SPEC);p5,r5,ends=phase5_receipts();manifest=combined_manifest(p5,ends)
    store=CleanStore(manifest).load();meta=metadata(manifest)
    for s in p5['bar_segments']:
        if s['mode']=='reuse' and store.verified.get(s['cache']['path'])!=s['cache']['sha256']:raise ValueError('reused slice lacks clean verified source')
    joins=u.cost_joins(read(ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json'),manifest)
    selected,register=select(manifest,store,meta,joins)
    return spec,p5,r5,ends,manifest,store,meta,joins,selected,register

def frozen_identity():
    if subprocess.check_output(['git','rev-parse','HEAD'],cwd=FROZEN,text=True).strip()!=COMMIT:raise ValueError('frozen checkout moved')
    names=subprocess.check_output(['git','ls-tree','-r','--name-only',COMMIT,'crates/jiaoyi-core/src','crates/jiaoyi-core/examples','Cargo.lock'],cwd=ROOT,text=True).splitlines()
    hashes={}
    for name in names:
        original=subprocess.check_output(['git','show',COMMIT+':'+name],cwd=ROOT)
        if (FROZEN/name).read_bytes()!=original:raise ValueError('frozen source edited:'+name)
        hashes[name]=sha(FROZEN/name)
    for name in HELPERS:
        original=subprocess.check_output(['git','show',COMMIT+':'+name],cwd=ROOT)
        if (ROOT/name).read_bytes()!=original:raise ValueError('frozen helper drift:'+name)
    if (FROZEN/SPEC.relative_to(ROOT)).read_bytes()!=SPEC.read_bytes():raise ValueError('Phase4 specification changed')
    f=read(SPEC)['frozen_candidate_hypothesis']
    if hashes['crates/jiaoyi-core/src/research/manifest_pullback.rs']!=f['entry_rule_source_sha256'] or hashes['crates/jiaoyi-core/src/research/repaired_exit.rs']!=f['exit_engine_source_sha256']:raise ValueError('candidate engine mismatch')
    return {'commit':COMMIT,'source_sha256':hashes,'binary_sha256':{b:sha(WORK/'bin'/b) for b in BINS},'specification_sha256':sha(SPEC)}

def prepare():
    sys.addaudithook(u.deny_network);WORK.mkdir(exist_ok=True)
    spec,p5,r5,ends,manifest,store,meta,joins,selected,register=sources_and_selection()
    frozen=frozen_identity();index=[]
    for e,inp,row in selected:
        path=WORK/('input-'+e['episode_id']+'.json');immutable_save(path,inp)
        index.append({'episode_id':e['episode_id'],'tier':row['tier'],'input_sha256':sha(path),'origin':row['origin']})
    audit=exclusion_audit(p5,r5,ends)
    frozen_dates={j['date'] for j in spec['independent_evaluation_plan']['jobs']}
    native_jobs=spec['independent_evaluation_plan']['jobs'];source_intervals=[e for e in manifest['episodes'] if e['eligible_for_research']]
    date_audit=[]
    for j in native_jobs:
        covering=[e['episode_id'] for e in source_intervals if e['symbol']==j['symbol'] and e['account_id']==j['account_id'] and ms(e['source_window_utc']['from'])<=ms(j['from']) and ms(e['source_window_utc']['to'])>=ms(j['to'])]
        date_audit.append({'episode_id':j['episode_id'],'date':j['date'],'from':j['from'],'to':j['to'],'covering_verified_tick_sources':covering})
    coverage={'register':register,'selection_index':index,'phase5_exclusion_audit':audit,
        'frozen_independent_plan_dates':[w['dates'] for w in spec['independent_evaluation_plan']['weeks']],
        'phase5_dates_overlap_frozen_plan':len(frozen_dates&{j['date'] for j in p5['jobs']}),'frozen_independent_plan_coverage':date_audit,
        'combined_manifest_episode_definitions':len(manifest['episodes']),'core_episode_definitions':len(register),
        'outside_core_definitions':len(manifest['episodes'])-len(register),'m30_source_files':len(manifest['m30_segments'])}
    immutable_save(WORK/'coverage-before-outcomes.json',coverage)
    inputs={str(x.relative_to(ROOT)):sha(x) for x in [SPEC,MANIFEST,P3,P5PLAN,P5RESULT,ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json']}
    value={'schema':'jiaoyi.phase5b-scope.v1','trace':'№18,366','frozen_phase4':frozen,'candidate':spec['frozen_candidate_hypothesis'],
        'input_hashes':inputs,'builder_sha256':sha(Path(__file__)),'coverage_sha256':sha(WORK/'coverage-before-outcomes.json'),'verified_market_files':store.verified,
        'phase5_ledger_sha256':sha(CACHE5/'ledger.jsonl'),'collection_receipts_reconciled':len(ends),
        'tracks':{'tier1_48':'Exact frozen candidate rules; original native sessions,3.74bp admission; full45-day native-session preparation.',
            'tier2_48':'Separate Phase3 opening-range diagnostic on clean native-window sources excluded from Tier1; not the frozen candidate.',
            'tier2_28':'Separate Phase3 opening-range diagnostic,28bp floor; not the frozen candidate.'},
        'primary_score':'forced native4h scenario close; strict and72h continuation separate;24bp stress reprices identical entries;3.74bp sensitivity not calibrated cost',
        'date_policy':'Original frozen Phase4 independent dates are unchanged. Phase5 dates and prior manifest replay are explicitly reused diagnostic cohorts, not substitutes for that plan.',
        'new_broker_requests':0,'validation_opened':False,'live_rules_changed':False,'financial_pass':False}
    immutable_save(SCOPE,value);store.finish()
    print(json.dumps({'selected':dict(collections.Counter((x['origin']+':'+x['tier']) for x in index)),'window_covered_new':sum(r['origin']=='phase5_new_batch' and r['window']['covers'] and r.get('excluded_reason')!='failed_tick_source' for r in register),'coverage_sha256':sha(WORK/'coverage-before-outcomes.json'),'scope_sha256':sha(SCOPE)}),flush=True)

def phase3_parity(meta,joins,freeze):
    previous=read(P3);m=read(MANIFEST);full=u.ManifestStore(m).load();clean=CleanStore(m).load()
    by_id={e['episode_id']:e for e in m['episodes']};count=checks=0;proofs=[]
    targets=[r for r in previous['rows'] if r['admission_bps']==3.74 and (r['tier']=='tier1' and r['threshold_bps']==48 or r['tier']=='tier2' and r['threshold_bps']==28)]
    if len(targets)!=202:raise ValueError('Phase3 comparison denominator changed')
    for old in targets:
        e=by_id[old['episode_id']];opts=[p for p in m['episodes'] if p['eligible_for_research'] and p['account_id']==e['account_id'] and p['symbol']==PEERS[e['symbol']] and p['date']==e['date'] and (old['tier']=='tier1' or p['protocol_status']=='clean_first_read')]
        peer=min(opts,key=lambda x:x['episode_id']) if opts else None;join=joins[u.CostKey(e['account_id'],e['symbol'])]
        inp,cov=(u.qualified_input(e,full,meta,peer,join) if old['tier']=='tier1' else tier2_input(e,clean,meta,peer,join))
        if inp is None:raise ValueError('prior input no longer qualifies')
        policy={k:old[k] for k in ['tier','threshold_bps','admission_bps']};path=WORK/'parity-input.json';save(path,{'input':inp,'policy':policy})
        if sha(path)!=old['input_sha256']:raise ValueError('Phase3 input parity:'+e['episode_id'])
        n,hit,key=u.cached_native('tiered_pullback_replay',path,['phase3_parity',e['episode_id'],policy],WORK,freeze)
        if n!=old['native']:raise ValueError('Phase3 native output changed:'+e['episode_id'])
        checks+=u.check_arithmetic(n);proofs.append({'episode_id':e['episode_id'],'tier':old['tier'],'exact':True,'receipt_key':key});count+=1
        if count%50==0:print('FROZEN PARITY '+str(count)+'/202',flush=True)
    full.finish();clean.finish();return proofs,checks

def evaluate():
    sys.addaudithook(u.deny_network);scope=read(SCOPE)
    if sha(Path(__file__))!=scope['builder_sha256']:raise ValueError('audit code changed after scope freeze')
    for name,h in scope['input_hashes'].items():
        if sha(ROOT/name)!=h:raise ValueError('frozen publication changed')
    frozen=frozen_identity()
    if frozen!=scope['frozen_phase4']:raise ValueError('frozen engine changed')
    for name in ['ops/research/phase5b_audit.py',str(SCOPE.relative_to(ROOT))]:
        if subprocess.check_output(['git','show','HEAD:'+name],cwd=ROOT)!= (ROOT/name).read_bytes():raise ValueError('commit scope/code before native evaluation')
    spec,p5,r5,ends,manifest,store,meta,joins,selected,register=sources_and_selection()
    cov=read(WORK/'coverage-before-outcomes.json')
    if sha(WORK/'coverage-before-outcomes.json')!=scope['coverage_sha256'] or register!=cov['register'] or store.verified!=scope['verified_market_files']:raise ValueError('coverage/source drift')
    if sha(CACHE5/'ledger.jsonl')!=scope['phase5_ledger_sha256']:raise ValueError('collection ledger changed')
    freeze={'frozen_phase4':frozen,'audit_scope_sha256':sha(SCOPE)};proofs,checks=phase3_parity(meta,joins,freeze)
    rows=[]
    for e,inp,row in selected:
        for threshold in ([48.0] if row['tier']=='tier1' else [48.0,28.0]):
            policy={'tier':row['tier'],'threshold_bps':threshold,'admission_bps':spec['frozen_candidate_hypothesis']['costs']['admission_bps_per_side']}
            path=WORK/'variant-input.json';save(path,{'input':inp,'policy':policy})
            n,hit,key=u.cached_native('tiered_pullback_replay',path,['phase5b',e['episode_id'],policy],WORK,freeze);checks+=u.check_arithmetic(n)
            out={k:row[k] for k in ['episode_id','account_id','symbol','date','cohort','origin','asset_class','tier','own_context','peer_context','tick_receipt_sha256']}
            out.update(threshold_bps=threshold,admission_bps=policy['admission_bps'],native=n,input_sha256=sha(path),native_receipt_key=key,exact_frozen_candidate=row['tier']=='tier1')
            rows.append(out)
        if len(rows)%20==0:print('EVALUATED '+str(len(rows))+' policy episodes',flush=True)
    groups={'new_batch':[r for r in rows if r['origin']=='phase5_new_batch'],'prior_manifest':[r for r in rows if r['origin']=='prior_manifest'],'combined':rows}
    summary={};comparisons={}
    for group,rs in groups.items():
        summary[group]={};comparisons[group]={}
        for tier,threshold in [('tier1',48),('tier2',48),('tier2',28)]:
            chosen=[x for x in rs if x['tier']==tier and x['threshold_bps']==threshold];track=f'{tier}_r{threshold}'
            slices={'all':chosen}|{s:[r for r in chosen if r['symbol']==s] for s in CORE}|{c:[r for r in chosen if r['asset_class']==c] for c in ['energy','equity_index','metal','crypto']}
            summary[group][track]={s:{a:{mode:{cost:u.score(rr,a,mode,cost) for cost in ['3.74','24']} for mode in u.MODES} for a in u.ARMS} for s,rr in slices.items()}
            comparisons[group][track]={c:paired(chosen,'forced',c) for c in ['3.74','24']}
    window_counts={s:collections.Counter('failed_source' if r.get('excluded_reason')=='failed_tick_source' else 'covers_native_window' if r['window']['covers'] else 'missing_native_window' for r in register if r['origin']=='phase5_new_batch' and r['symbol']==s) for s in CORE}
    prior_rows=read(P3)['rows'];old_t1={r['episode_id'] for r in prior_rows if r['tier']=='tier1'};old_t2={r['episode_id'] for r in prior_rows if r['tier']=='tier2'}
    tiers={origin:{tier:sum(r['origin']==origin and r.get('tier')==tier for r in register) for tier in ['tier1','tier2']} for origin in ['phase5_new_batch','prior_manifest']}
    exact_new=tiers['phase5_new_batch']['tier1'];plan_covered=sum(bool(x['covering_verified_tick_sources']) for x in cov['frozen_independent_plan_coverage'])
    result={'schema':'jiaoyi.phase5b-exact-audit.v1','trace':'№18,366','scope_sha256':sha(SCOPE),'frozen_phase4':frozen,'frozen_candidate_hypothesis':spec['frozen_candidate_hypothesis'],
        'new_broker_requests':0,'validation_opened':False,'live_rules_changed':False,'financial_pass':False,'rfc_gates_passed':2,'rfc_gates_total':12,
        'original_frozen_candidate_tested_offline':exact_new>0,'original_frozen_candidate_tested_offline_definition':'At least one new-batch episode evaluated with exact Phase4 Tier1 rules; not a claim the independent date matrix or financial validation completed.',
        'original_candidate_rules_replayed_on_prior_manifest':any(r['tier']=='tier1' and r['origin']=='prior_manifest' for r in rows),
        'original_frozen_independent_plan_completed':plan_covered==160,'frozen_independent_plan_tick_sources_covered':plan_covered,
        'phase5_exclusion_audit':cov['phase5_exclusion_audit'],'coverage':cov,'eligible_counts':tiers,'window_alignment_by_symbol':window_counts,
        'prior_tier_changes':{'tier2_to_tier1':[r['episode_id'] for r in register if r['origin']=='prior_manifest' and r.get('tier')=='tier1' and r['episode_id'] in old_t2],
            'prior_tier1_not_selected_as_tier1':[r['episode_id'] for r in register if r['episode_id'] in old_t1 and r.get('tier')!='tier1']},
        'rows':rows,'summary':summary,'paired_macro':comparisons,'verification':{'phase3_exact_paths':len(proofs),'phase3_parity':proofs,'native_arithmetic_checks':checks,'verified_market_file_count':len(store.verified),'network_guard_enabled':True},
        'limitations':['Native calendars omit historical holidays/maintenance.','3.74bp is an assumption, not calibrated round-trip cost.','Tick discontinuities and native unqualified-return flags remain visible.','Tier2 is a separate opening-range diagnostic, not the frozen sigma candidate.','New-batch dates were already examined under a different candidate and do not match the original frozen independent weeks.','Combined prior outcomes are reused development evidence; do not count them as new independent wins.']}
    store.finish()
    for name,h in scope['input_hashes'].items():
        if sha(ROOT/name)!=h:raise ValueError('source changed while evaluating')
    save(RESULT,result);print(json.dumps({'eligible':tiers,'original_candidate_on_new_batch':exact_new>0,'original_frozen_plan_windows_covered':plan_covered,'result_sha256':sha(RESULT)},indent=2),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','evaluate']);a=p.parse_args()
    if a.action=='prepare':prepare()
    else:evaluate()
