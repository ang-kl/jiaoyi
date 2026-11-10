#!/usr/bin/env python3
"""Frozen, offline exit-only audit. Native replay; no broker calls or policy tuning."""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from forensic18407_counterfactual import stream
import unified_replay as u
from tiered_replay import CleanStore, tier2_input
from replay_inventory import read, save, sha, market, ms, stamp, canonical, permitted

ROOT=Path(__file__).resolve().parents[2]
ARM='arm_c_macro_filtered_pullback'
COSTS=[0.,3.74,24.]

def iso(us):
    return datetime.fromtimestamp(us/1000000,timezone.utc).isoformat()

def micros(s):
    return round(datetime.fromisoformat(s.replace('Z','+00:00')).timestamp()*1000000)

def native(binary, value, work, tag):
    p=work/'case.json'; save(p,value)
    raw=subprocess.check_output([str(binary),str(p)],timeout=90)
    return json.loads(raw),{'identity':tag,'input_sha256':sha(p),'output_sha256':hashlib.sha256(raw).hexdigest()}

def metrics(values):
    # A deterministic closed-trade R curve, not an intratrade or portfolio VaR estimate.
    good=[v for v in values if v is not None]
    pos=sum(v for v in good if v>1e-12); neg=-sum(v for v in good if v< -1e-12)
    cum=peak=dd=0.
    for v in good:
        cum+=v; peak=max(peak,cum);dd=max(dd,peak-cum)
    return {'entries':len(values),'closed':len(good),'unresolved':len(values)-len(good),
            'wins':sum(v>1e-12 for v in good),'losses':sum(v< -1e-12 for v in good),
            'net_flat':sum(abs(v)<=1e-12 for v in good),'win_rate':sum(v>1e-12 for v in good)/len(good) if good else None,
            'net_r':sum(good) if good else None,'mean_net_r':sum(good)/len(good) if good else None,
            'profit_factor':pos/neg if neg else None,'profit_factor_status':'defined' if neg else 'no_losses' if good else 'no_trades',
            'closed_trade_max_drawdown_r':dd if good else None}

def net(outcome,entry,bps):
    gross=outcome['scenario_gross_r']
    return None if gross is None else gross-(entry['fill']+outcome['fills'][0]['price'])*bps/10000/entry['initial_r']

def augment(pair,entry):
    for name in ('baseline','candidate'):
        pair[name]['net_by_bps']={str(b):net(pair[name],entry,b) for b in COSTS}
    return pair

def observed(work,binary):
    ledgers=[('daytime_15',Path('/workspace/jiaoyi-forensic-18406'),Path('/workspace/jiaoyi-counterfactual-18407'),'evidence-refresh.json'),
             ('overnight_6',Path('/workspace/jiaoyi-followup-18411'),work/'overnight','evidence.json')]
    result=[];receipts=[]
    for cohort,audit,cache,ef in ledgers:
        evidence=read(audit/ef); ledger=read(audit/'trade-ledger.json')
        for r in ledger:
            acc=r['account_id'];sym=r['symbol'];risk=r.get('risk_price',r.get('risk_distance'));entry=r['entry_fill']
            ds=[d for d in evidence['deals'] if d['account_id']==acc and d['position_id']==r['position_id']]
            opening=[d for d in ds if not d['is_closing']];closing=[d for d in ds if d['is_closing']]
            assert len(opening)==len(closing)==1 and opening[0]['filled_volume']==closing[0]['close_closed_volume']
            d=closing[0];digits=d['close_money_digits'];scale=10**digits
            costs=sum(d[k] for k in ('close_swap','close_commission','close_pnl_conversion_fee'))/scale
            money=(d['close_gross_profit']/scale)+costs
            expected=r.get('net_pnl_usd',r.get('net_pnl'));assert abs(money-expected)<1e-9
            assert opening[0]['execution_price']==entry and d['execution_price']==r['exit_fill']
            volume=opening[0]['filled_volume'];units=volume/100
            # All scoped contracts quote and settle in USD; compare, do not infer cash scale.
            assert abs((entry-r['exit_fill'])*units-d['close_gross_profit']/scale)<=0.011
            assert opening[0]['commission']==0,'unhandled opening commission: do not double count or assume zero'
            rows,proof=stream(cache,acc,sym);receipts.extend(proof)
            begin=micros(r['entry_time']);end=micros(r.get('window_close') or ('2026-11-09T20:00:00Z' if sym=='ETHUSD' else '2026-11-09T21:59:00Z'))
            bid=ask=None;bt=at=None;quotes=[];stale=0;prior=None;gaps=[]
            for us,_,b,a in rows:
                if b is not None:bid,bt=b/100000,us
                if a is not None:ask,at=a/100000,us
                if us<begin:continue
                if bid is None or ask is None or ask<bid or max(us-bt,us-at)>6000000:
                    stale+=1;continue
                if prior is not None and us<=prior:continue
                if prior is not None and us-prior>6000000:gaps.append((us-prior)/1e6)
                quotes.append({'at':iso(us),'bid':bid,'ask':ask,'gap_before':False});prior=us
            assert quotes and micros(quotes[0]['at'])>=begin
            target=abs(r['initial_tp']-entry)
            inp={'entered_at':r['entry_time'],'fill':entry,'initial_r':risk,'side':'short','volume':volume,
                 'tp1_close_volume':volume,'tp1_distance':round(target,5),'broker_tp_distance':round(target,5),
                 'session_end':iso(end),'digits':3 if sym in ('SpotCrude','SpotBrent') else 2,'cadence':'s3_thirty_seconds'}
            case={'entry':inp,'through':quotes[-1]['at'],'quotes':quotes,'request':None,'source':None,'forced':True}
            pair,rec=native(binary,case,work,f'{cohort}:{acc}:{r["position_id"]}');receipts.append(rec)
            alternate, _=stream(cache,acc,sym,True)
            if alternate!=rows:raise ValueError('same-timestamp path needs explicit ordering sensitivity')
            baseline=pair['baseline'];bf=baseline['fills'][0] if baseline['fills'] else None
            observed_kind=r.get('exit_kind',r.get('exit_reason_classified'))
            result.append({'cohort':cohort,'account_id':acc,'symbol':sym,'position_id':r['position_id'],
              'historical_policy':r.get('policy','s3_macro_pullback_owner_18392_v1'),'entry':inp,
              'broker_exit':{'at':r['exit_time'],'price':r['exit_fill'],'kind':observed_kind,'net_usd':money,'net_r':money/(risk*units),'costs_usd':costs},
              'broker_receipt_reconciliation':True,'exact_quote_fill_parity':bool(bf and bf['price']==r['exit_fill'] and micros(bf['at'])==micros(r['exit_time'])),
              'baseline_price_difference_from_broker':bf['price']-r['exit_fill'] if bf else None,
              'baseline_time_difference_ms':(micros(bf['at'])-micros(r['exit_time']))/1000 if bf else None,
              'recorded_path':{'quotes':len(quotes),'stale_or_crossed_updates_omitted':stale,'intervals_over6s':len(gaps),'largest_interval_seconds':max(gaps,default=0),'continuous_broker_path_verified':False},
              'paired':augment(pair,inp)})
            for arm in ('baseline','candidate'):
                out=result[-1]['paired'][arm];out['estimated_cash_usd']=round(out['scenario_gross_r']*risk*units+costs,2) if out['scenario_gross_r'] is not None else None
    return result,receipts

def historical(work,binary):
    old=read(ROOT/'docs/rebuild_phase3_coverage_and_pullback_results_18360.json')
    manifest=read(ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json')
    store=u.ManifestStore(manifest);clean=CleanStore(manifest).load();meta=u.native_metadata(manifest)
    joins=u.cost_joins(read(ROOT/'docs/rebuild_phase1_cost_and_contract_inventory_18355.json'),manifest)
    peers={a:b for a,b in [('SpotCrude','SpotBrent'),('SpotBrent','SpotCrude'),('US500','NAS100'),('NAS100','US500'),('XAUUSD','XAGUSD'),('XAGUSD','XAUUSD'),('BTCUSD','ETHUSD'),('ETHUSD','BTCUSD')]}
    inputs={};out=[];proofs=[];parity=0
    for index,row in enumerate(old['rows']):
        eid=row['episode_id'];e=store.episodes[eid]
        if eid not in inputs:
            if row['tier']=='tier1': inp=read(Path('/workspace/jiaoyi-rebuild-phase3-18360')/f'input-{eid}.json')
            else:
                options=[p for p in manifest['episodes'] if p['eligible_for_research'] and p['protocol_status']=='clean_first_read' and p['account_id']==e['account_id'] and p['symbol']==peers[e['symbol']] and p['date']==e['date']]
                peer=min(options,key=lambda p:p['episode_id']) if options else None
                inp,cov=tier2_input(e,clean,meta,peer,joins[u.CostKey(e['account_id'],e['symbol'])]);assert inp is not None
            inputs[eid]=inp
        inp=inputs[eid];policy={k:row[k] for k in ('tier','threshold_bps','admission_bps')}
        variant={'input':inp,'policy':policy};p=work/'variant.json';save(p,variant)
        assert sha(p)==row['input_sha256'],f'normalized source changed {eid}'
        n,proof=native(binary.parent/'tiered_pullback_replay',variant,work,f'phase3:{eid}:{policy}')
        if n!=row['native']:raise ValueError('historical native parity failed '+eid)
        parity+=1;proofs.append(proof)
        arm=n.get('arms',{}).get(ARM,{})
        if arm.get('status')!='admitted':continue
        for mode in ('strict','forced'):
            path=arm[mode];entry=path['entry'];through=ms(inp['request']['to'])+(5000 if mode=='forced' else 0)
            pair,rec=native(binary,{'entry':entry,'through':stamp(through),'quotes':[],
                                   'request':inp['request'],'source':inp['source'],'forced':mode=='forced'},work,f'phase3:{eid}:{policy}:{mode}')
            if pair['baseline']!=path['outcome']:raise ValueError('exit baseline parity failed '+eid)
            out.append({k:row[k] for k in ('episode_id','symbol','date','tier','threshold_bps','admission_bps')}|{'cohort':'phase3','mode':mode,'entry':entry,'paired':augment(pair,entry)})
            proofs.append(rec)
        if index%60==0:print('historical parity',parity,flush=True)
    stage=read(ROOT/'docs/gate10_stage1_2024_11_11_results_18382.json');stage_parity=0
    for row in stage['rows']:
        p=Path('/workspace/jiaoyi-stage1-18382')/f'input-{row["episode_id"]}.json'
        assert sha(p)==row['input_sha256']
        n,proof=native(binary.parent/'tiered_pullback_replay',read(p),work,'stage1:'+row['episode_id'])
        if n!=row['native']:raise ValueError('stage1 parity failed '+row['episode_id'])
        if n.get('arms',{}).get(ARM,{}).get('status')=='admitted':raise ValueError('Stage1 unexpected entry; comparison adapter required')
        stage_parity+=1;proofs.append(proof)
    clean.finish()
    return out,proofs,{'phase3_variants_exact':parity,'phase3_distinct_sessions':len(inputs),
        'tier1_sessions':35,'tier2_sessions':167,'stage1_episodes_exact':stage_parity,'stage1_candidate_entries':0,
        'stage1_scheduled':stage['episodes_scheduled'],'stage1_exclusions':stage['episodes_scheduled']-stage_parity}

def summary(actual,hist):
    out={}
    for cohort in ('daytime_15','overnight_6'):
        rows=[r for r in actual if r['cohort']==cohort]
        cell={'broker_net_usd':round(sum(r['broker_exit']['net_usd'] for r in rows),2),
              'broker':metrics([r['broker_exit']['net_r'] for r in sorted(rows,key=lambda r:(r['broker_exit']['at'],r['account_id']))])}
        for cost in COSTS:
            key=str(cost);cell[key]={}
            for arm in ('baseline','candidate'):
                ordered=sorted(rows,key=lambda r:(r['paired'][arm]['fills'][0]['at'] if r['paired'][arm]['fills'] else '',r['account_id']))
                cell[key][arm]=metrics([r['paired'][arm]['net_by_bps'][key] for r in ordered])
        cell['recorded_cost_cash_estimate']={arm:round(sum(r['paired'][arm]['estimated_cash_usd'] or 0 for r in rows),2) for arm in ('baseline','candidate')}
        cell['per_account']={str(acc):{arm:metrics([r['paired'][arm]['net_by_bps']['0.0'] for r in sorted(rows,key=lambda r:r['broker_exit']['at']) if r['account_id']==acc]) for arm in ('baseline','candidate')} for acc in sorted({r['account_id'] for r in rows})}
        out[cohort]=cell
    groups=defaultdict(list)
    for r in hist:groups[f'{r["tier"]}|floor={r["threshold_bps"]:g}|admission={r["admission_bps"]:g}|{r["mode"]}'].append(r)
    out['historical']={}
    for group,rows in sorted(groups.items()):
        out['historical'][group]={str(cost):{arm:metrics([r['paired'][arm]['net_by_bps'][str(cost)] for r in sorted(rows,key=lambda r:(r['paired'][arm]['fills'][0]['at'] if r['paired'][arm]['fills'] else '',r['episode_id']))]) for arm in ('baseline','candidate')} for cost in COSTS}
    return out

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--work',type=Path,default=Path('/workspace/jiaoyi-repair-18412'));p.add_argument('--binary',type=Path,default=ROOT/'target/debug/examples/profit_lock_replay');args=p.parse_args()
    sys.addaudithook(u.deny_network)
    actual,ap=observed(args.work,args.binary);save(args.work/'observed-results.json',actual);print('21 broker receipts reconciled; observed exit replay complete',flush=True)
    hist,hp,parity=historical(args.work,args.binary)
    result={'trace':'№18,412','new_broker_requests':0,'validation_opened':False,'candidate_in_production':False,
            'plan_sha256':sha(ROOT/'docs/readiness_profit_lock_plan_18412.json'),'binary_sha256':sha(args.binary),
            'observed':actual,'historical':hist,'parity':parity,'receipts':ap+hp,'summary':summary(actual,hist),
            'independent_unseen_qualification':False,'costs':'Recorded executable bid/ask includes spread; 0,3.74,24bp per side are additional slippage sensitivities. Actual broker cash separately reconciled. NetR and cash replay are estimates, no guaranteed fills.'}
    save(args.work/'replay-results.json',result)
    print(json.dumps({'parity':parity,'summary':result['summary']},indent=2),flush=True)
if __name__=='__main__':main()
