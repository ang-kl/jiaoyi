#!/usr/bin/env python3
"""№18,362: audit frozen paths and reserve unseen dates. No broker or model client."""
import argparse
import collections
import datetime as dt
from decimal import Decimal
import itertools
import json
from pathlib import Path
import re
import subprocess
import sys
import unified_replay as u
from tiered_replay import CleanStore
from replay_inventory import ROOT, read, save, sha, canonical, ms, stamp, permitted, market

RESULT=ROOT/'docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json'
SCOPE=ROOT/'docs/rebuild_phase4_scope_18362.json'
PHASE3=ROOT/'docs/rebuild_phase3_coverage_and_pullback_results_18360.json'
MANIFEST=ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json'
WORK=Path('/workspace/jiaoyi-rebuild-phase4-18362')
DAY=86400000
SPECS=[('tier1_7_of_7','tier1',48,3.74,u.ARMS[2],7),('tier2_6_of_8','tier2',28,3.74,u.ARMS[2],8),('vetoed_3','tier2',48,24,u.ARMS[1],3)]

def dec(x):return Decimal(str(x))
def intersect(a,b,c,d):return a<d and c<b
def sign(side):return 1 if side=='long' else -1

def exit_class(path,strict):
    """Use actual bracket state, not net P&L, to distinguish a breakeven stop."""
    i=path['entry'];o=path['outcome'];fills=o['fills']
    if len(fills)!=1:raise ValueError('expected one whole-position exit')
    f=fills[0];reason=f['reason'];eps=Decimal(1).scaleb(-i['digits'])/2
    if reason=='broker_take_profit':
        if not strict.get('metrics'):raise ValueError('boundary target is not a strict target')
        if abs(dec(f['price'])-(dec(i['fill'])+sign(i['side'])*dec(i['broker_tp_distance'])))>eps:raise ValueError('target price mismatch')
        if dec(i['broker_tp_distance']) < dec(i['initial_r'])*dec('1.38'):raise ValueError('target below1.38R')
        return 'strict_target_1_38r'
    if reason in ['stop','gap_through_stop']:
        return 'breakeven_stop_after_1r' if o['trail_applied']>0 and abs(dec(o['active_stop'])-dec(i['fill']))<=eps else 'initial_stop_loss'
    if reason=='session_end':
        if strict.get('metrics'):raise ValueError('forced close falsely counted as strict')
        return 'forced_window_close'
    raise ValueError('unrecognised exit:'+reason)

def path_arithmetic(path):
    i=path['entry'];f=path['outcome']['fills'][0]
    entry,exit,risk=map(dec,[i['fill'],f['price'],i['initial_r']])
    if risk<=0 or f['volume']!=i['volume'] or ms(f['at'])<ms(i['entered_at']):raise ValueError('invalid execution identity')
    gross=sign(i['side'])*(exit-entry)/risk
    if abs(float(gross)-path['metrics']['gross_r'])>1e-9:raise ValueError('gross arithmetic mismatch')
    net={}
    for cost in ['3.74','24']:
        net[cost]=float(gross-(entry+exit)*dec(cost)/10000/risk)
        if abs(net[cost]-path['metrics']['net_by_bps'][cost])>1e-9:raise ValueError('net arithmetic mismatch')
    return {'gross_r':float(gross),'net_r_at_3_74bp':net['3.74'],'net_r_at_24bp':net['24'],
        'symmetric_cost_break_even_bps':float(sign(i['side'])*(exit-entry)*10000/(entry+exit))}

def quote_at(source,at):
    target=ms(at);quotes={};ages={}
    for side in ['bids','asks']:
        prior=[q for q in source[side] if q['ts_ms']<=target]
        if not prior:raise ValueError('missing side at fill')
        q=prior[-1];quotes[side]=dec(q['price_e5'])/100000;ages[side]=target-q['ts_ms']
    if max(ages.values())>6000 or quotes['asks']<quotes['bids']:raise ValueError('stale/crossed fill quote')
    return quotes,ages

def audit_trade(row,arm,group,store,scope):
    n=row['native'];paths=n['arms'][arm];f=paths['forced'];i=f['entry'];exit=f['outcome']['fills'][0]
    kind=exit_class(f,paths['strict']);math=path_arithmetic(f)
    e=store.episodes[row['episode_id']];raw=store.source(e);_,quotes,_,_=market(raw)
    eq,ea=quote_at(quotes,i['entered_at']);xq,xa=quote_at(quotes,exit['at'])
    ep=eq['asks' if i['side']=='long' else 'bids'];xp=xq['bids' if i['side']=='long' else 'asks']
    if ep!=dec(i['fill']):raise ValueError('entry does not match recorded executable quote')
    if kind=='strict_target_1_38r':
        if sign(i['side'])*(xp-dec(exit['price']))<0:raise ValueError('target not crossed by quote')
    elif xp!=dec(exit['price']):raise ValueError('exit does not match recorded executable quote')
    # The frozen native receipt links these paths to the Phase3 code/input policy.
    receipt=Path(scope['phase3_native_cache'])/(row['native_receipt_key']+'.receipt.json')
    output=receipt.with_name(row['native_receipt_key']+'.json');proof=read(receipt)
    if proof!={'cache_key':row['native_receipt_key'],'output_sha256':sha(output)} or read(output)!=n:raise ValueError('native receipt mismatch')
    peer=row['peer_context'];own=row['own_context'];signum=sign(i['side'])
    for c in [own,peer]:
        if c and c['latest_bar_end'] and ms(c['latest_bar_end'])>ms(e['session_window_utc']['open']):raise ValueError('future trend context')
    own_veto=own['status']=='complete' and not all(own[k] is not None and signum*own[k]>0 for k in ['drift_5','drift_20'])
    peer_conflict=peer is not None and peer['status']=='complete' and any(peer[k] is not None and signum*peer[k]<0 for k in ['drift_5','drift_20'])
    return {k:row[k] for k in ['episode_id','account_id','symbol','date','cohort','tier','threshold_bps','admission_bps','tick_receipt_sha256']}|{
        'group':group,'executed_arm':arm,'counterfactual_under_C':group=='vetoed_3','direction':i['side'],
        'impulse_r_bps':n['impulse']['initial_r']/n['impulse']['fill']*10000,
        'macro_5d_drift':own['drift_5'],'macro_20d_drift':own['drift_20'],'macro_context':own,
        'peer_status':'absent' if peer is None else 'missing_context' if peer['status']!='complete' else 'disagrees' if peer_conflict else 'non_conflicting',
        'peer_context':peer,'native_macro_reason':n['macro']['reason'],'own_trend_would_veto':own_veto,
        'peer_would_conflict':peer_conflict,'effective_veto':'own_5d_20d' if own_veto else 'peer_disagreement' if peer_conflict else None,
        'entry_time_utc':i['entered_at'],'exit_time_utc':exit['at'],'entry_price':i['fill'],'exit_price':exit['price'],
        'initial_r_price':i['initial_r'],'gross_target_distance':i['broker_tp_distance'],'native_volume':i['volume'],
        'exit_reason':kind,'raw_exit_reason':exit['reason'],'strict_status':paths['strict']['status'],
        'strict_closed':bool(paths['strict'].get('metrics')),'continuation_status':paths['continuation']['status'],
        'trail_applied':f['outcome']['trail_applied'],'path_data_issues':f['outcome']['data_issues'],
        'entry_quote_age_ms':ea,'exit_quote_age_ms':xa,'native_receipt_key':row['native_receipt_key'],
        'native_output_sha256':proof['output_sha256'],'source_path':e['source_path'],**math}

def anatomy(rows):
    exits=collections.Counter(x['exit_reason'] for x in rows)
    return {'trades':len(rows),'strict_target_1_38r_hits':exits['strict_target_1_38r'],
        'breakeven_after_1r_exits':exits['breakeven_stop_after_1r'],'initial_stop_exits':exits['initial_stop_loss'],
        'forced_window_closes':exits['forced_window_close'],
        'profitable_forced_window_closes':sum(x['exit_reason']=='forced_window_close' and x['net_r_at_3_74bp']>0 for x in rows),
        'losing_trades':sum(x['net_r_at_3_74bp']<0 for x in rows),
        'wins_at_3_74bp':sum(x['net_r_at_3_74bp']>0 for x in rows),'wins_at_24bp':sum(x['net_r_at_24bp']>0 for x in rows),
        'mean_net_r_at_3_74bp':sum(x['net_r_at_3_74bp'] for x in rows)/len(rows),
        'mean_net_r_at_24bp':sum(x['net_r_at_24bp'] for x in rows)/len(rows),
        'symbols_represented':sorted({x['symbol'] for x in rows}),'unique_dates_count':len({x['date'] for x in rows})}

def metadata_nodes(x,kind='metadata'):
    if isinstance(x,dict):
        yield kind,x
        for k,v in x.items():
            if k in ['bids','asks','bars','m30','quotes','payload','input','result']:continue
            yield from metadata_nodes(v,'m30' if k=='bar_segments' else kind)
    elif isinstance(x,list):
        for v in x:yield from metadata_nodes(v,kind)

def date_literals(x):
    if isinstance(x,str):
        if re.fullmatch(r'20\d\d-\d\d-\d\d',x):yield x
    elif isinstance(x,dict):
        for k,v in x.items():
            if k not in ['bids','asks','bars','m30','quotes','payload','input','result']:yield from date_literals(v)
    elif isinstance(x,list):
        for v in x:yield from date_literals(v)

def embedded_m30_window(e):
    # №18,342 peer collector passes each four-hour job directly to bar_request;
    # its eight intraday bars are not evidence of a45-day preparation request.
    if '/jiaoyi-jan2024-peer-18342/' in e['source_path']:
        return e['source_window_utc']['from'],e['source_window_utc']['to'],'verified_intraday_m30_collector_window'
    at=ms(e['session_window_utc']['open'])
    return stamp(at-45*DAY),e['source_window_utc']['to'],'conservative_embedded_preparation_envelope'

def registry(manifest,scope):
    intervals=[];reserved_dates=set();metadata_hashes={}
    def add(symbol,a,b,kind,origin,authority):
        try:a,b=ms(a),ms(b)
        except (ValueError,TypeError,AttributeError):return
        if a>=b:raise ValueError('invalid inventory interval')
        intervals.append({'symbol':symbol,'from':stamp(a),'to':stamp(b),'kind':kind,'origin':origin,'authority':authority})
    for e in manifest['episodes']:
        w=e['source_window_utc'];add(e['symbol'],w['from'],w['to'],'ticks',e['source_path'],'verified_manifest')
        if e.get('m30_embedded_count',0):
            a,b,authority=embedded_m30_window(e);add(e['symbol'],a,b,'m30_envelope',e['source_path'],authority)
    for e in manifest['m30_segments']:
        w=e['source_window_utc'];add(e['symbol'],w['from'],w['to'],'m30',e['source_path'],'verified_manifest')
    for e in manifest['failed_sources']:
        w=e.get('request_window_utc',{});add(e['symbol'],w.get('from'),w.get('to'),'m30_failed' if 'bar-' in e['path'] else 'ticks_failed',e['path'],'failed_request_retained')
    for f in scope['metadata_files']:
        if sha(f['path'])!=f['sha256']:raise ValueError('metadata changed:'+f['path'])
        x=read(f['path']);metadata_hashes[f['path']]=f['sha256'];reserved_dates.update(date_literals(x))
        for kind,n in metadata_nodes(x):
            req=n.get('request',n)
            if not isinstance(req,dict):continue
            symbol=req.get('symbol',n.get('symbol'))
            if isinstance(symbol,str) and req.get('from') and req.get('to'):
                add(symbol,n.get('source_from',req['from']),req['to'],'m30_reserved' if kind=='m30' else 'interval_reserved',f['path'],'frozen_plan_conservative_reservation')
    intervals=list({canonical(x):x for x in intervals}.values())
    intervals.sort(key=lambda x:(x['from'],x['to'],x['symbol'],x['origin']))
    return {'intervals':intervals,'reserved_dates':sorted(reserved_dates),'metadata_hashes':metadata_hashes,
        'status':'retained_catalog_only_not_universal_broker_history','unavailable_historical_references':manifest['summary']['prior_reference_statuses'].get('unavailable_locally',0)}

def missing_runs(times):
    out=[]
    for t in sorted(times):
        if out and out[-1][1]==t:out[-1][1]=t+1800000
        else:out.append([t,t+1800000])
    return [{'from':stamp(a),'to':stamp(b),'m30_slots':(b-a)//1800000} for a,b in out]

def upgrade_audit(previous,store,meta,reg):
    ids={r['episode_id'] for r in previous['rows'] if r['tier']=='tier2'};rows=[]
    for identity in sorted(ids):
        e=store.episodes[identity];at=ms(meta[identity]['session']['open']);required=u.slots(meta[identity]['prior_45_calendar_days'])
        bars=store.bars[(e['account_id'],e['symbol'])];missing=[t for t in required if t not in bars]
        required_with_open=required+([at] if at not in bars or bars[at]['volume']<=0 else [])
        missing_with_open=missing+([at] if at in required_with_open and (at not in bars or bars[at]['volume']<=0) else [])
        symbol_intervals=[(ms(v['from']),ms(v['to'])) for v in reg['intervals'] if v['symbol']==e['symbol'] and v['kind'].startswith('m30')]
        spent=[t for t in missing_with_open if any(intersect(t,t+1800000,a,b) for a,b in symbol_intervals)]
        untouched=[t for t in missing_with_open if t not in set(spent)]
        protected=any(not permitted(t,t+1800000) for t in required+[at])
        local=sum(t in bars for t in required)
        category='protected_45day_dependency' if protected else 'tier1_ready_local' if not missing_with_open else 'needs_spent_gap_or_calendar_reconciliation' if spent else 'only_unspent_history_missing'
        rows.append({'episode_id':identity,'symbol':e['symbol'],'date':e['date'],'account_id':e['account_id'],
            'after_cutoff':at>=u.RESEARCH_CUTOFF,'required_prep_slots':len(required),'verified_local_prep_slots':local,
            'missing_prep_slots':len(missing),'opening_M30_present':at in bars and bars[at]['volume']>0,
            'local_preparation_status':'none_in_required_lookback' if not local else 'partial' if missing else 'complete',
            'has_any_local_M30_file_for_symbol':bool(store.bar_sources[(e['account_id'],e['symbol'])]),
            'overlaps_protected_preparation':protected,'category':category,'missing_in_spent_or_reserved_intervals':len(spent),
            'missing_unspent_under_retained_catalog':len(untouched),'spent_missing_runs':missing_runs(spent),'unspent_missing_runs':missing_runs(untouched),
            'tier1_upgrade_does_not_create_independent_evidence':True})
    if len(rows)!=167:raise ValueError('Tier2 denominator changed')
    return {'episodes':167,'categories':dict(collections.Counter(x['category'] for x in rows)),
        'local_preparation_status':dict(collections.Counter(x['local_preparation_status'] for x in rows)),
        'after_cutoff':sum(x['after_cutoff'] for x in rows),'with_any_local_symbol_M30':sum(x['has_any_local_M30_file_for_symbol'] for x in rows),
        'rows':rows,'calendar_warning':'Missing coded weekday slots can be legitimate holidays/maintenance. A spent gap is not permission to refetch; validate authoritative dated calendars or retained raw decoding first.'}

def independent_weeks(reg):
    """Metadata-only chronological choice, no price/return selection; disjoint45d bundles."""
    blocks=[(ms(x['from']),ms(x['to'])) for x in reg['intervals']]
    reserved=set(reg['reserved_dates']);chosen=[];rejections=collections.Counter();first=dt.date(2023,1,2);last_end=None
    for k in range(140):
        monday=first+dt.timedelta(days=7*k);dates=[(monday+dt.timedelta(days=d)).isoformat() for d in range(5)]
        a=ms(dates[0]+'T00:00:00Z');b=a+5*DAY;prep=a-45*DAY
        if not permitted(prep,b):rejections['protected_or_cutoff']+=1;continue
        if last_end is not None and prep<last_end:rejections['overlaps_prior_reserved_test_bundle']+=1;continue
        if any(d in reserved for d in dates):rejections['previous_plan_date']+=1;continue
        if any(intersect(prep,b,c,d) for c,d in blocks):rejections['existing_market_or_request_interval']+=1;continue
        chosen.append({'week_start':dates[0],'dates':dates,'preparation_envelope_from':stamp(prep),'envelope_to':stamp(b),'known_interval_overlaps':0})
        last_end=b
        if len(chosen)==4:break
    if len(chosen)!=4:raise ValueError('cannot establish four unspent bundles under the frozen registry')
    return chosen,dict(rejections)

def preparation_start(metadata):
    """Cover every required dated session, including an earlier UTC open across DST."""
    return min([ms(metadata['session']['open'])-45*DAY]+
        [ms(s['open']) for s in metadata['prior_45_calendar_days']])

def gates():
    return [
        {'gate':'G1','status':'partial','reason':'Source/code/calendar/contract/cost identities linked; historical authority remains unverified.'},
        {'gate':'G2','status':'passed','reason':'Inherited verified159/35 parity; replay code unchanged in Phase4, Phase3 hashes retained.'},
        {'gate':'G3','status':'passed','reason':'Offline manifest workflow, zero broker requests, deterministic audit arithmetic and complete denominators.'},
        {'gate':'G4','status':'partial','reason':'Exact167 gap map; protected and spent gaps cannot be silently filled. Future full paths not collected.'},
        {'gate':'G5','status':'open','reason':'Dated historical holidays, maintenance and contract validity required before collection/qualification.'},
        {'gate':'G6','status':'open','reason':'Matched broker round-trip deals, fees, swaps and conversion not available.'},
        {'gate':'G7','status':'open','reason':'No out-of-sample calibrated round-trip cost error bounds;3.74bp remains assumption.'},
        {'gate':'G8','status':'partial','reason':'18 cached native paths and recorded quote prices reconciled; real execution/restart parity unproved.'},
        {'gate':'G9','status':'partial','reason':'Single candidate and locally unspent four-week plan frozen. Missing historical references and future collection/execution checks prevent universal independence certification.'},
        {'gate':'G10','status':'not_passed','reason':'Selected historical paths are development evidence; no independent result generated.'},
        {'gate':'G11','status':'not_performed','reason':'No broker shadow/canary evaluation.'},
        {'gate':'G12','status':'not_demonstrated','reason':'No per-strategy/account operational goal qualification.'}]

def validate_gate_progress(result):
    by_id={g['gate']:g['status'] for g in result['rfc_gates']}
    if set(by_id)!={f'G{i}' for i in range(1,13)} or len(result['rfc_gates'])!=12:raise ValueError('gate register must contain12 distinct gates')
    passed=sum(v=='passed' for v in by_id.values())
    if passed!=result['rfc_gates_passed']:raise ValueError('gate count mismatch')
    if result['new_independent_observations']==0 and by_id['G10']=='passed':raise ValueError('no independent financial evidence')
    if result['independent_evaluation_plan']['reservation_status']=='frozen_not_executed' and by_id['G9']=='passed':raise ValueError('planned is not executed qualification')
    if result['financial_pass'] or result['validation_opened'] or result['new_broker_requests']!=0:raise ValueError('outside Phase4 offline scope')
    return passed

def build():
    sys.addaudithook(u.deny_network);scope=read(SCOPE)
    for path,digest in scope['input_hashes'].items():
        if sha(ROOT/path)!=digest:raise ValueError('frozen input mismatch:'+path)
    manifest=read(MANIFEST);previous=read(PHASE3);reg=registry(manifest,scope)
    # No reading of quarantined prices: store source guard checks every dependency first.
    store=CleanStore(manifest).load();meta=u.native_metadata(manifest);ledgers={};allrows=[]
    for group,tier,floor,cost,arm,count in SPECS:
        selected=[x for x in previous['rows'] if (x['tier'],x['threshold_bps'],x['admission_bps'])==(tier,floor,cost) and x['native'].get('arms',{}).get(arm,{}).get('forced')]
        if len(selected)!=count:raise ValueError('selected ledger denominator changed:'+group)
        rows=[audit_trade(x,arm,group,store,scope) for x in selected];ledgers[group]={'anatomy':anatomy(rows),'trades':rows};allrows+=rows
    upgrades=upgrade_audit(previous,store,meta,reg);weeks,rejections=independent_weeks(reg)
    candidate=scope['frozen_candidate'];symbols=candidate['symbols'];items=[]
    classes={'SpotCrude':'energy','SpotBrent':'energy','US500':'equity_index','NAS100':'equity_index','XAUUSD':'metal','XAGUSD':'metal','BTCUSD':'crypto','ETHUSD':'crypto'}
    for w in weeks:
        for date in w['dates']:
            for symbol in symbols:items.append({'key':symbol+'-'+date,'symbol':symbol,'class':classes[symbol],'date':date})
    metadata=json.loads(subprocess.check_output([str(u.BIN/'replay_inventory_metadata')],input=json.dumps(items),text=True,timeout=120))
    jobs=[]
    for x in items:
        detail=metadata[x['key']];s=detail['session'];a=ms(s['open']);b=a+4*3600000;prep=preparation_start(detail)
        if not permitted(prep,b+5001):raise ValueError('protected future plan')
        if any(intersect(prep,b+5001,ms(z['from']),ms(z['to'])) for z in reg['intervals']):raise ValueError('unspent planned dependency overlaps retained history')
        jobs.append({'episode_id':x['key'],'date':x['date'],'symbol':x['symbol'],'asset_class':x['class'],'account_id':49236051,
            'from':stamp(a),'to':stamp(b),'tail_through':stamp(b+5000),'prep_from':stamp(prep),'prep_to':stamp(a),
            'native_volume':next(e['native_volume'] for e in manifest['episodes'] if e['symbol']==x['symbol'] and e['account_id']==49236051),
            'historical_calendar_verified':False,'collection_status':'not_requested'})
    # One deduplicated interval-only M30 budget per symbol/week, rather than repeated per-day45d requests.
    prep_jobs=[]
    for w in weeks:
        for symbol in symbols:
            use=[j for j in jobs if j['symbol']==symbol and j['date'] in w['dates']];a=min(ms(j['prep_from']) for j in use);b=max(ms(j['from']) for j in use)+1800000
            t=a
            while t<b:
                end=min(t+7*DAY,b);prep_jobs.append({'symbol':symbol,'week_start':w['week_start'],'from':stamp(t),'to':stamp(end),'period':'M30','request_mode':'interval_only_no_count','collection_status':'not_requested'});t=end
    tests={'ledger_rows':len(allrows),'independent_net_arithmetic_checks':len(allrows)*2,'native_cache_receipts_verified':len(allrows),
        'recorded_entry_and_exit_quote_checks':len(allrows)*2,'future_jobs_overlap_checks':len(jobs),'new_broker_requests':0}
    store.finish()
    output={'schema':'jiaoyi.phase4-forensic-freeze.v1','trace':'№18,362','frozen_at_sgt':scope['frozen_at_sgt'],
        'scope_sha256':sha(SCOPE),'phase3_commit':scope['phase3_commit'],'builder_sha256':sha(Path(__file__)),
        'input_hashes':scope['input_hashes'],'ledgers':ledgers,'tier2_upgrade_audit':upgrades,
        'frozen_candidate_hypothesis':candidate,'independent_evaluation_plan':scope['evaluation_policy']|{
            'weeks':weeks,'unspent_weeks_count':len(weeks),'unspent_dates_count':sum(len(w['dates']) for w in weeks),
            'symbols':symbols,'total_new_episodes_planned':len(jobs),'maximum_tick_jobs':len(jobs),'maximum_m30_jobs':len(prep_jobs),
            'jobs':jobs,'m30_segments':prep_jobs,'date_selection_rejections':rejections,'reservation_status':'frozen_not_executed',
            'independence_scope':'No overlap in retained price/request catalog; chronological forward independence is NOT claimed. Old unavailable references require reconciliation before execution.'},
        'spent_registry':reg,'rfc_gates':gates(),'rfc_gates_passed':sum(g['status']=='passed' for g in gates()),'rfc_gates_total':12,
        'verification':tests,'new_broker_requests':0,'validation_opened':False,'financial_pass':False,'live_rules_changed':False,'new_independent_observations':0}
    validate_gate_progress(output)
    save(RESULT,output);print(json.dumps({'anatomy':{k:v['anatomy'] for k,v in ledgers.items()},'upgrades':upgrades['categories'],'local':upgrades['local_preparation_status'],'weeks':[w['week_start'] for w in weeks],'tick_jobs':len(jobs),'m30_jobs':len(prep_jobs),'result_sha256':sha(RESULT)},indent=2))

if __name__=='__main__':build()
