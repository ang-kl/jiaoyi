"""№18,372 forensic ledger and time-ordered component diagnostics, offline."""
import collections
from decimal import Decimal
from pathlib import Path

from cost_components import position_cost, summarise, exit_level_observation, calibrate
from rebuild_preflight import require, verified_json
from replay_inventory import ROOT, read, save, sha, canonical, stamp, ms, permitted
from cost_evidence_assessment import verified_entry

WORK=Path('/workspace/jiaoyi-cost-reservation-18372')
PLAN=ROOT/'docs/rebuild_rfc_260deals_analysis_plan_18372.json'


def run():
    plan=read(PLAN)
    for path,digest in plan['source_hashes'].items():require(sha(ROOT/path)==digest,'frozen_input_changed:'+path)
    audit=read(ROOT/'docs/rebuild_rfc_gates_5_to_9_audit_18370.json')
    bundle=read(ROOT/'docs/rebuild_rfc_gates_5_to_9_evidence_18370.json')
    require(sha(ROOT/'docs/rebuild_rfc_gates_5_to_9_evidence_18370.json')==audit['captured_evidence_sha256'],'broker_bundle_changed')
    deals={(a['account_id'],d['dealId']):d for a in bundle['accounts'] for d in a['deals']}
    contracts={(a['account_id'],s['symbolId']):s for a in bundle['accounts'] for s in a['contracts']['symbol']}
    complete=[p for p in audit['round_trip_money_inventory'] if p['quantity_complete']]
    require(len(complete)==271,'position_denominator_changed')
    rows=[position_cost(p,deals,contracts[(p['account_id'],p['symbol_id'])]) for p in complete]
    matched=[r for r in rows if r['statement_matched']]
    require(len(matched)==260,'statement_matched_denominator_changed')
    quantities=[q for r in matched for q in r['dated_quantity_checks']]
    quantity_conflicts=[q for q in quantities if not q['agrees_with_current_snapshot']]
    grouped={}
    all_grouped={}
    for group,keys in [('account',('account_id',)),('symbol',('symbol',)),
                       ('account_symbol',('account_id','symbol')),('account_strategy',('account_id','strategy_labels'))]:
        bins=collections.defaultdict(list)
        for r in matched:
            key=tuple(tuple(r[k]) if isinstance(r[k],list) else r[k] for k in keys)
            bins[key].append(r)
        grouped[group]=[{'key':list(k),'summary':summarise(v)} for k,v in sorted(bins.items())]
        all_bins=collections.defaultdict(list)
        for r in rows:
            key=tuple(tuple(r[k]) if isinstance(r[k],list) else r[k] for k in keys)
            all_bins[key].append(r)
        all_grouped[group]=[{'key':list(k),'summary':summarise(v)} for k,v in sorted(all_bins.items())]
    observations={}
    observations['recorded_fee_financing']=[dict(id=f"fee:{r['account_id']}:{r['position_id']}",account_id=r['account_id'],
        environment=r['environment'],symbol=r['symbol'],purpose='booked_round_trip_fees_and_financing',
        at_ms=r['at_ms'],start_ms=r['start_ms'],value_bps=r['recorded_fee_financing_drag_bps'],
        position_id=r['position_id'],source_sha256=r['source_money_sha256']) for r in matched]
    entries=[]
    for r in bundle['attributed_entry_rows']:
        v=verified_entry(r)
        entries.append({'id':r['entry_key'],'account_id':r['account_id'],
            'environment':'live' if r['account_id']==43002148 else 'demo','symbol':r['symbol'],
            'purpose':'entry_'+r['side'],'position_id':r['position_id'],'at_ms':ms(v['checked_at']),
            'value_bps':str(v['dispatch_to_fill_bps']),'price_deviation':str(Decimal(str(v['fill_price']))-Decimal(str(v['dispatch_price']))),
            'dispatch_spread':v['dispatch_spread'],'source_sha256':canonical(r),
            'in_statement_matched260':any(p['account_id']==r['account_id'] and p['position_id']==r['position_id'] for p in matched)})
    require(len(entries)==17,'entry_observation_denominator_changed')
    observations['entry_dispatch_slippage']=entries
    levels={};source_rows={};exclusions=collections.Counter()
    for ref in bundle['service_source_receipts']:
        if not ref['kind'].startswith('orders'):continue
        raw=verified_json(ref['path'],ref['sha256'])
        for r in raw['rows']:
            if r['kind']!='broker_exit':continue
            try:deal_id=int(r['entry_key'].split(':')[-1])
            except ValueError:exclusions['not_deal_key']+=1;continue
            d=deals.get((r['account_id'],deal_id))
            if not d:exclusions['deal_outside_capture']+=1;continue
            v=exit_level_observation(r,d)
            if v is None:exclusions['no_recorded_level_or_closing_detail']+=1;continue
            if v['id'] in levels:
                require(levels[v['id']]==v,'ambiguous_duplicate_level')
                continue
            v['in_statement_matched260']=any(p['account_id']==r['account_id'] and p['position_id']==r['position_id'] for p in matched)
            levels[v['id']]=v;source_rows[v['id']]=r
    observations['exit_level_deviation_proxy']=list(levels.values())
    # Retained historical checkpoints, isolated from 2026 fill/fee calibration.
    prior=read(ROOT/'docs/rebuild_phase1_unified_replay_manifest_18355.json')
    spreads={}
    for ep in prior['episodes']:
        if not ep['eligible_for_research'] or ep['protocol_status']!='clean_first_read':continue
        for key,q in ep['spread_checkpoints'].items():
            if q is None or not permitted(q['quote_ms'],q['quote_ms']+1):continue
            identity=(ep['account_id'],ep['symbol'],q['quote_ms'],q['bid_e5'],q['ask_e5'])
            if identity in spreads:continue
            require(q['ask_e5']>=q['bid_e5']>0 and q['side_max_age_ms']<=6000 and q['quote_delay_ms']<=5000,'spread_quote_invalid')
            bid,ask=Decimal(q['bid_e5']),Decimal(q['ask_e5'])
            spreads[identity]={'id':canonical(identity),'account_id':ep['account_id'],
                'environment':'live' if ep['account_id']==43002148 else 'demo','symbol':ep['symbol'],
                'purpose':'historical_checkpoint_spread','at_ms':q['quote_ms'],
                'value_bps':str(20000*(ask-bid)/(ask+bid)),'price_units':str((ask-bid)/100000),
                'episode_id':ep['episode_id'],'source_sha256':ep['tick_receipt_sha256'],
                'context':'historical checkpoint spread only; no transfer to 2026 exit fills or live accounts'}
    observations['historical_spread']=list(spreads.values())
    calibration={name:calibrate(obs,name,plan['calibration']) for name,obs in observations.items()}
    summary=summarise(matched)
    symbol_medians={r['key'][0]:r['summary']['median_recorded_fee_financing_drag_bps'] for r in grouped['symbol']}
    result={'trace':'№18,372','frozen_plan_sha256':sha(PLAN),'positions_271':rows,
        'summary_statement_matched260':summary,'summary_quantity_complete271':summarise(rows),
        'summary_other11_quantity_complete':summarise([r for r in rows if not r['statement_matched']]),
        'forensic_groups':grouped,'forensic_groups_all271':all_grouped,'dated_quantity_checks':{'closing_fills':len(quantities),
            'current_spec_agreements':sum(q['agrees_with_current_snapshot'] for q in quantities),'conflicts':quantity_conflicts,
            'authority':'observed lot conversion at recorded trades; not blanket historical contract validity'},
        'component_observations':observations,'component_calibration':calibration,
        'exit_reference_source_rows':source_rows,'exit_reference_exclusions':dict(exclusions),
        'exit_reference_provenance':{'code':'crates/jiaoyi/src/fills.rs:exit_level; strategy3.rs:broker_exit',
            'source_sha256':{'fills.rs':sha(ROOT/'crates/jiaoyi/src/fills.rs'),'strategy3.rs':sha(ROOT/'crates/jiaoyi/src/strategy3.rs')},
            'warning':'Stop/TP reason may be chosen as nearest stored level; level may fall back to local owned-position state. This is not broker-trigger provenance or pure exit-dispatch shortfall.'},
        'median_recorded_fee_financing_bps_by_symbol':symbol_medians,
        'median_round_trip_cost_bps_by_symbol':{s:None for s in summary['symbols']},
        'complete_round_trip_cost_unidentifiable':'No matched midpoint/dispatch/ack evidence for both legs and accepted resting-level provenance. Component models cannot be added to already net broker P&L.',
        'new_broker_requests':0,'financial_pass':False,'live_rules_changed':False,'validation_opened':False}
    save(WORK/'forensics.json',result)
    return result


if __name__=='__main__':
    from rebuild_preflight import offline_process
    import json
    offline_process();r=run();print(json.dumps({'matched260':r['summary_statement_matched260'],
        'quantity':r['dated_quantity_checks'],'components':{k:{'observations':v['observations'],'trained_buckets':v.get('trained_buckets'),
        'cutoff':v.get('cutoff_utc')} for k,v in r['component_calibration'].items()}},indent=2))
