#!/usr/bin/env python3
"""Offline audit of nine saved read-only views; no broker calls or cost fitting."""
import argparse
import hashlib
import json
import math
from pathlib import Path

import cost_context

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
OIL = ('SpotCrude', 'SpotBrent')
METALS = ('XAUUSD', 'XAGUSD')


def verified_entry(row):
    value = row.get('entry_cost')
    if value is None:
        return None
    if row['kind'] != 'open' or row['side'] not in ('buy', 'sell'):
        raise ValueError('entry attribution on incompatible order')
    fields = ('decision_price', 'dispatch_price', 'fill_price', 'dispatch_spread',
              'decision_to_dispatch_bps', 'dispatch_to_fill_bps', 'decision_to_fill_bps')
    if any(not math.isfinite(value[k]) for k in fields):
        raise ValueError('non-finite attribution')
    if min(value[k] for k in ('decision_price','dispatch_price','fill_price')) <= 0:
        raise ValueError('invalid attribution price')
    if value['dispatch_spread'] < 0 or not 0 <= value['dispatch_quote_age_ms'] <= 6000:
        raise ValueError('invalid attribution spread/age')
    sign = 1 if row['side'] == 'buy' else -1
    dispatch = sign*(value['dispatch_price']-value['decision_price'])/value['dispatch_price']*10000
    execution = sign*(value['fill_price']-value['dispatch_price'])/value['dispatch_price']*10000
    if not all(math.isclose(a,b,abs_tol=1e-8) for a,b in [
        (dispatch,value['decision_to_dispatch_bps']),
        (execution,value['dispatch_to_fill_bps']),
        (dispatch+execution,value['decision_to_fill_bps']),
        (row['fill_price'],value['fill_price'])]):
        raise ValueError('attribution arithmetic mismatch')
    return value


def assess(rows, coverage, aggregates, manifest):
    identities = [(r['strategy'],r['account_id'],r['entry_key']) for r in rows]
    if len(set(identities)) != len(identities):
        raise ValueError('duplicate order identity')
    filled = [r for r in rows if r['kind']=='open' and r['fill_price'] is not None]
    measured = []
    for row in rows:
        value = verified_entry(row)
        if value is not None:
            item = {k:row[k] for k in ('strategy','account_id','entry_key','symbol','position_id',
                                      'created_at','side','requested_volume','filled_volume')}
            item['environment'] = 'live' if row['account_id']==43002148 else 'demo'
            item['entry_cost'] = value
            exits = [r for r in rows if r['account_id']==row['account_id']
                     and r['strategy']==row['strategy'] and r['position_id']==row['position_id']
                     and r['kind'] in ('close','broker_exit') and r['fill_price'] is not None]
            item['linked_exit_records'] = [{k:r[k] for k in ('entry_key','created_at','kind','order',
                'filled_volume','fill_price','slippage')} for r in exits]
            item['round_trip_qualified'] = False
            measured.append(item)
    if len(measured) != coverage['measured_entries']:
        raise ValueError('independent attribution count disagrees')
    subsets = {'all':lambda r:True, 'live':lambda r:r['account_id']==43002148,
               'demo':lambda r:r['account_id']!=43002148,
               'oil':lambda r:r['symbol'] in OIL, 'metals':lambda r:r['symbol'] in METALS}
    summary = {}
    for name, predicate in subsets.items():
        sample = [r for r in measured if predicate(r)]
        slips = [r['entry_cost']['dispatch_to_fill_bps'] for r in sample]
        summary[name] = dict(filled_opens=sum(predicate(r) for r in filled),
            measured_entries=len(sample), measured_dates_utc=sorted({r['created_at'][:10] for r in sample}),
            measured_sides=sorted({r['side'] for r in sample}),
            measured_native_volumes=sorted({r['filled_volume'] for r in sample}),
            min_dispatch_to_fill_bps=min(slips) if slips else None,
            max_dispatch_to_fill_bps=max(slips) if slips else None,
            entries_with_linked_exit_records=sum(bool(r['linked_exit_records']) for r in sample),
            verified_round_trip_samples_in_this_review=0)
    if len(rows)!=coverage['unique_rows'] or len(filled)!=coverage['filled_opens']:
        raise ValueError('independent order/open counts disagree')
    return dict(trace='№18,283 §§1–6; by Codex (GPT-6), effort not exposed',
        source_manifest=manifest, newest_order_at_utc=max((r['created_at'] for r in rows),default=None),
        scope='Eight newest500 order views plus one aggregate view; not a certified complete/unbiased cohort.',
        page_limit_reached_sources=[x['file'] for x in manifest
            if x['file'].startswith('orders-') and x['rows']>=500],
        coverage=coverage, summary=summary, attributed_entries=measured,
        aggregate_fee_diagnostics=[{k:r[k] for k in ('account_id','strategy','symbol','entries_filled',
            'commission_round_trip_per_unit_median','commission_pending')}
            for r in aggregates if r['symbol'] in OIL+METALS],
        aggregate_fee_warning='All stored commissions so far per position, not a settled closing-fee lifecycle; no quantity-completion or swap proof. Do not fit from this median or add opening commission again to broker closing commissions.',
        calibrated_per_side_bps=None, verified_round_trip_samples_in_this_review=0,
        brokerwide_complete_cost_sample_count=None, financial_pass=False,
        policy_change=False, maintained_research_reserve_bps_per_side=24,
        qualifications={'entry_arithmetic':f'PASS for {len(measured)} retained entries',
            'representative_live_entry_costs':'NOT_VERIFIABLE',
            'paired_exit_dispatch_costs':'NOT_VERIFIABLE',
            'complete_commission_swap_conversion_lifecycle':'NOT_VERIFIABLE',
            'historical_2025_cost_transfer':'NOT_VERIFIABLE'})


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--snapshots',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    manifest=json.loads((a.snapshots/'manifest.json').read_text())
    expected={f'orders-{s}-{account}.json' for account in ACCOUNTS for s in ('s1','s3')}|{'costs.json'}
    if len(manifest)!=9 or {x['file'] for x in manifest}!=expected:
        raise ValueError('exact bounded nine-source manifest required')
    for item in manifest:
        path=a.snapshots/item['file']
        if path.stat().st_size>8_000_000 or hashlib.sha256(path.read_bytes()).hexdigest()!=item['sha256']:
            raise ValueError('source snapshot hash/size mismatch')
    paths=sorted(a.snapshots/name for name in expected if name.startswith('orders-'))
    rows=[r for path in paths for r in json.loads(path.read_text())['rows']]
    if any(r['account_id'] not in ACCOUNTS for r in rows):
        raise ValueError('unexpected account')
    result=assess(rows,cost_context.receipt_coverage(paths),
                  json.loads((a.snapshots/'costs.json').read_text())['rows'],manifest)
    result['analysis_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    result['coverage_helper_sha256']=hashlib.sha256(Path(cost_context.__file__).read_bytes()).hexdigest()
    a.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    print(json.dumps(result['summary']))


if __name__=='__main__':
    main()
