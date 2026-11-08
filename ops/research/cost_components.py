"""Offline retrospective component calibration, never a fabricated all-in cost.

Recorded fees, executable spread, entry dispatch slippage and inferred exit-level
deviation have different benchmarks. They are not interchangeable or added twice
to broker realised P&L. Buckets never transfer demo/account evidence to live.
"""
import collections
import datetime as dt
from decimal import Decimal
import math
import statistics

from rebuild_evidence import number as D
from rebuild_preflight import require
from replay_inventory import canonical, ms, stamp


def position_cost(position, deals, contract):
    account = position['account_id']
    records = [deals[(account, i)] for i in position['deal_ids']]
    filled = [d for d in records if d['dealStatus'] in (2,3)]
    closes = position['closing_deals']
    gross = net = commission = swap = conversion_fee = Decimal(0)
    notional = quote_cost = units = Decimal(0)
    quantity_checks = []
    for c in closes:
        m = c['source_money']; raw = deals[(account,m['deal_id'])]
        require(canonical(raw) == m['receipt_sha256'], 'raw_closing_deal_changed')
        for key in ('gross','net','commission','swap','conversion_fee','conversion_rate'):
            D(m[key])
        qty = D(m['closed_volume'])/100
        rate = D(m['conversion_rate'])
        gross += D(m['gross']); net += D(m['net']); commission += D(m['commission'])
        swap += D(m['swap']); conversion_fee += D(m['conversion_fee'])
        notional += D(raw['closePositionDetail']['entryPrice'])*qty*rate
        quote_cost += (D(m['gross'])-D(m['net']))/rate
        units += qty
        if c['statement']['status'] == 'matched':
            text = c['statement']['row']['closing_quantity'].split()
            require(len(text)==2 and text[1]=='Lots', 'statement_quantity_unit_unrecognised')
            lots = D(text[0].replace(',',''))
            require(lots>0, 'statement_quantity_nonpositive')
            implied = D(m['closed_volume'])/lots
            quantity_checks.append({'deal_id':m['deal_id'],'statement_lots':str(lots),
                'broker_closed_cents':m['closed_volume'],'dated_observed_lot_size_cents':str(implied),
                'agrees_with_current_snapshot':implied==D(contract['lotSize'])})
    require(gross+commission+swap+conversion_fee==net, 'fee_money_does_not_reconcile')
    require(notional>0 and units>0, 'cost_notional_or_quantity')
    first=min(d['executionTimestamp'] for d in filled);last=max(d['executionTimestamp'] for d in filled)
    return {'account_id':account,'environment':position['environment'],'symbol':position['symbol'],
        'position_id':position['position_id'],'strategy_labels':position['labels'],
        'statement_matched':position['money_evidence_complete'],
        'entry_time_utc':stamp(first),'exit_time_utc':stamp(last),'start_ms':first,'at_ms':last,
        'win':net>0,'breakeven':net==0,'gross_pnl_money':str(gross),'commission_money':str(commission),
        'swap_money':str(swap),'conversion_fee_money':str(conversion_fee),'net_pnl_money':str(net),
        'recorded_fee_financing_drag_money':str(gross-net),
        'recorded_fee_financing_drag_price_units':str(quote_cost/units),
        'recorded_fee_financing_drag_bps':str((gross-net)/notional*10000),
        'reference_notional_money':str(notional),'closed_native_units':str(units),
        'notional_convention':'sum(closing entryPrice * closed cents/100 * recorded closing quote/deposit conversion)',
        'complete_round_trip_execution_cost_bps':None,
        'cost_limit':'gross-minus-net identifies booked fees/financing only; spread and execution shortfall are already inside fill-to-fill gross P&L',
        'dated_quantity_checks':quantity_checks,'source_money_sha256':canonical(position),
        'netting_reversal_fills':position['netting_reversal_fills']}


def summarise(rows):
    if not rows:
        return {'positions':0,'win_rate':None}
    fields=('gross_pnl_money','commission_money','swap_money','conversion_fee_money','net_pnl_money','recorded_fee_financing_drag_money')
    out={k:str(sum((D(r[k]) for r in rows),Decimal(0))) for k in fields}
    out.update(positions=len(rows),wins=sum(r['win'] for r in rows),breakevens=sum(r['breakeven'] for r in rows),
        win_rate=sum(r['win'] for r in rows)/len(rows),
        first_entry_utc=min(r['entry_time_utc'] for r in rows),last_exit_utc=max(r['exit_time_utc'] for r in rows),
        median_recorded_fee_financing_drag_bps=float(statistics.median(D(r['recorded_fee_financing_drag_bps']) for r in rows)),
        aggregate_notional_weighted_fee_drag_bps=float(sum(D(r['recorded_fee_financing_drag_money']) for r in rows)/sum(D(r['reference_notional_money']) for r in rows)*10000),
        symbols=sorted({r['symbol'] for r in rows}))
    if len(out['symbols'])==1:
        out['median_recorded_fee_financing_drag_price_units']=str(statistics.median(D(r['recorded_fee_financing_drag_price_units']) for r in rows))
    return out


def exit_level_observation(row, raw):
    """The public broker_exit label is a nearest-level inference, not trigger proof."""
    if row['kind']!='broker_exit' or not row.get('decision'):
        return None
    signal=row['decision'].get('signal',{});level=signal.get('level')
    if level is None or row.get('fill_price') is None or 'closePositionDetail' not in raw:
        return None
    require(row['account_id']>0 and row['position_id']==raw['positionId'],'exit_level_position_join')
    require(row['filled_volume']==raw['filledVolume'],'exit_level_quantity_join')
    require(abs(D(row['fill_price'])-D(raw['executionPrice']))<=Decimal('0.00000001'),'exit_level_fill_join')
    require(row['decision']['reason']==signal.get('reason'), 'exit_reason_disagreement')
    level=D(level);fill=D(raw['executionPrice']);require(level>0,'nonpositive_order_level')
    signed=(fill-level)*(1 if raw['tradeSide']==1 else -1)
    if row['slippage'] is not None:
        require(abs(D(row['slippage'])-signed)<Decimal('0.00000001'),'stored_level_slippage_arithmetic')
    return {'id':f"{row['account_id']}:{raw['dealId']}",'account_id':row['account_id'],
        'environment':'live' if row['account_id']==43002148 else 'demo','symbol':row['symbol'],
        'purpose':signal['reason'],'position_id':raw['positionId'],'deal_id':raw['dealId'],
        'at_ms':raw['executionTimestamp'],'level':str(level),'fill':str(fill),
        'signed_price_deviation':str(signed),'absolute_price_deviation':str(abs(fill-level)),
        'value_bps':str(signed/level*10000),'absolute_deviation_bps':str(abs(fill-level)/level*10000),
        'reference_type':'stored_nearest_SL_TP_level_execution_report_or_local_fallback',
        'broker_trigger_proven':False,'exit_dispatch_observation':False,
        'order_row_sha256':canonical(row),'raw_deal_sha256':canonical(raw)}


def calibrate(observations, component, config):
    """Freeze one cutoff per component using date metadata, never values."""
    require(len({r['id'] for r in observations})==len(observations),'duplicate_component_observation')
    dates=sorted({stamp(r['at_ms'])[:10] for r in observations})
    if len(dates)<2:
        return {'component':component,'status':'INSUFFICIENT_DATES','observations':len(observations),'models':[],'split_completed':False}
    count=min(len(dates)-1,max(1,math.floor(len(dates)*.7)))
    cutoff=ms(dates[count]+'T00:00:00Z')
    groups=collections.defaultdict(list)
    for row in observations:
        require(row['environment']==('live' if row['account_id']==43002148 else 'demo'),'environment_join')
        groups[(row['account_id'],row['environment'],row['symbol'],row['purpose'])].append(row)
    models=[]
    for key,rows in sorted(groups.items()):
        rows.sort(key=lambda r:(r['at_ms'],r['id']))
        train=[r for r in rows if r['at_ms']<cutoff]
        test=[r for r in rows if r['at_ms']>=cutoff and r.get('start_ms',r['at_ms'])>=cutoff]
        boundary=[r for r in rows if r not in train and r not in test]
        require(len(train)+len(test)+len(boundary)==len(rows),'split_denominator')
        enough=len(train)>=config['minimum_training_observations'] and len(test)>=config['minimum_holdout_observations']
        prediction=sum((D(r['value_bps']) for r in train),Decimal(0))/len(train) if enough else None
        errors=[D(r['value_bps'])-prediction for r in test] if enough else []
        blocks=collections.defaultdict(list)
        for row,error in zip(test,errors):blocks[stamp(row['at_ms'])[:10]].append(error)
        block_errors=[float(sum(es)/len(es)) for es in blocks.values()]
        half=1.96*statistics.stdev(block_errors)/math.sqrt(len(block_errors)) if len(block_errors)>=2 else None
        models.append({'account_id':key[0],'environment':key[1],'symbol':key[2],'purpose':key[3],
            'status':'COMPONENT_TEMPORAL_DIAGNOSTIC' if enough else 'SPARSE_NO_MODEL',
            'training_ids':[r['id'] for r in train],'held_out_ids':[r['id'] for r in test],
            'boundary_excluded_ids':[r['id'] for r in boundary],
            'frozen_training_mean_bps':str(prediction) if prediction is not None else None,
            'held_out_mae_bps':float(sum(map(abs,errors))/len(errors)) if errors else None,
            'held_out_bias_bps':float(sum(errors)/len(errors)) if errors else None,
            'held_out_error_range_bps':[float(min(errors)),float(max(errors))] if errors else None,
            'daily_error_means_bps':{d:float(sum(es)/len(es)) for d,es in blocks.items()},
            'normal_95_halfwidth_bps_diagnostic':half,
            'uncertainty_limit':'daily blocks may be dependent; few blocks and retrospective inspection prevent population confidence or live authority',
            'fallback':'no empirical prediction for sparse bucket; retain explicitly labelled research stress assumptions',
            'holdout_predictions':[{'id':r['id'],'observed_bps':r['value_bps'],'error_bps':str(e)} for r,e in zip(test,errors)]})
    return {'component':component,'cutoff_utc':stamp(cutoff),'date_metadata':dates,
        'split_completed':True,'retrospective':True,'models':models,'observations':len(observations),
        'trained_buckets':sum(m['status']=='COMPONENT_TEMPORAL_DIAGNOSTIC' for m in models),
        'full_round_trip_cost_authority':False,'source_observations_sha256':canonical(observations)}
