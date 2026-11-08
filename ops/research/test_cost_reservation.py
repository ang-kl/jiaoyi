import copy
import unittest

from broker_money import ledger
from cost_components import position_cost, exit_level_observation, calibrate
from calendar_authority import verify_current, verify_historical
from clean_reservation import eligibility, merged, verify_reuse_reservation
from rebuild_preflight import Refusal
from replay_inventory import canonical, ms
from test_broker_evidence import deal, money
from test_rebuild import preparation


class CostReservationTests(unittest.TestCase):
    def test_component_money_and_dated_quantity_are_not_total_execution_cost(self):
        ds=[deal(1,1,100,100),deal(2,2,100,101,money(100,100,100,commission=10,swap=-5))]
        statement={(49236051,2):{'row':dict(symbol='US500',net='0.85',commission='-0.10',
            entry_price='100',exit_price='101',closing_quantity='1 Lots'),'sources':[]}}
        p=ledger(49236051,'demo',ds,{1:'US500'},{1:{'digits':2}},statement)[0]
        r=position_cost(p,{(49236051,d['dealId']):d for d in ds},{'lotSize':100})
        self.assertEqual(r['net_pnl_money'],'0.85')
        self.assertEqual(r['recorded_fee_financing_drag_bps'],'15.0000')
        self.assertTrue(r['dated_quantity_checks'][0]['agrees_with_current_snapshot'])
        self.assertIsNone(r['complete_round_trip_execution_cost_bps'])

    def test_exit_level_proxy_is_signed_by_exit_side_and_never_trigger_proof(self):
        d=deal(2,2,100,99,money(100,100,-100))
        r=dict(account_id=49236051,position_id=10,symbol='US500',kind='broker_exit',
            decision={'reason':'stop_loss','signal':{'reason':'stop_loss','level':100}},
            fill_price=99,filled_volume=100,slippage=1)
        v=exit_level_observation(r,d)
        self.assertEqual(v['value_bps'],'100.00')
        self.assertFalse(v['broker_trigger_proven'])
        d['tradeSide']=1;r['slippage']=-1
        self.assertEqual(exit_level_observation(r,d)['value_bps'],'-100.00')
        d['executionPrice']=98
        with self.assertRaisesRegex(Refusal,'fill_join'):exit_level_observation(r,d)

    def test_temporal_fit_cannot_learn_holdout_or_pool_accounts(self):
        obs=[dict(id=str(i),account_id=49236051,environment='demo',symbol='XAUUSD',purpose='fee',
                  at_ms=ms(f'2024-01-0{i}T12:00:00Z'),value_bps=str(i)) for i in range(1,8)]
        obs[4]['start_ms']=ms('2024-01-01T00:00:00Z')
        cfg={'minimum_training_observations':3,'minimum_holdout_observations':2}
        a=calibrate(obs,'fees',cfg);m=a['models'][0]
        self.assertEqual(m['frozen_training_mean_bps'],'2.5')
        self.assertEqual(m['boundary_excluded_ids'],['5'])
        self.assertEqual(m['held_out_ids'],['6','7'])
        other=copy.deepcopy(obs);other[-1]['value_bps']='7000'
        self.assertEqual(calibrate(other,'fees',cfg)['models'][0]['frozen_training_mean_bps'],'2.5')
        extra=copy.deepcopy(obs[-1]);extra.update(id='live',account_id=43002148,environment='live')
        b=calibrate(obs+[extra],'fees',cfg)
        live=next(v for v in b['models'] if v['environment']=='live')
        self.assertEqual(live['status'],'SPARSE_NO_MODEL')
        self.assertIsNone(live['frozen_training_mean_bps'])

    def test_calendar_tier_b_requires_real_boundaries_and_keeps_unknown_history(self):
        f=preparation();meta=f[2]['e'];start=ms(meta['session']['open'])
        a=verify_current(1,'US500',{'symbolId':1,'digits':2,'scheduleTimeZone':'America/New_York',
            'schedule':[{'startSecond':100,'endSecond':200}]},1,'2026-11-08T00:00:00Z')
        ep=dict(account_id=1,symbol='US500',episode_id='e',source_window_utc={'from':'2024-03-11T13:30:00Z','to':'2024-03-11T17:30:00Z'})
        q=lambda t:dict(quote_ms=t+1,side_max_age_ms=0)
        v=verify_historical(a,ep,meta,f[3][(1,'US500')],q(start),q(start+14400000))
        self.assertEqual(v['status'],'QUALIFIED_PROXY_REPLAY')
        self.assertFalse(v['historical_broker_authority'])
        v=verify_historical(a,ep,meta,{},q(start),None)
        self.assertEqual(v['status'],'REFUSED_PROXY_REPLAY')
        self.assertIn('native_m30_preparation_incomplete',v['reasons'])
        with self.assertRaises(Refusal):verify_current(2,'US500',a['facts'],1,'now')

    def test_search_rejects_spent_gap_and_seen_label_but_reuses_causal_bars(self):
        f=preparation();meta=f[2]['e'];start=ms(meta['session']['open']);t=min(f[3][(1,'US500')])
        seen=merged([(t,t+3600000)])
        self.assertEqual(eligibility({},meta,f[3][(1,'US500')],seen,[])['reasons'],[])
        self.assertIn('spent_missing_preparation',eligibility({},meta,{},seen,[])['reasons'])
        self.assertIn('evaluation_price_interval_already_inspected',eligibility({},meta,f[3][(1,'US500')],[(start,start+1)],[])['reasons'])

    def test_versioned_reservation_does_not_weaken_old_guard_or_allow_changed_jobs(self):
        plan,old,meta,bars,refs,verified=preparation()
        plan['episodes'][0]['tail_through']='2024-03-11T17:30:05Z'
        t=min(bars[(1,'US500')]);old['spent_intervals']=[dict(symbol='US500',account_id=1,start_ms=t,end_ms=t+3600000,role='m30')]
        new=copy.deepcopy(old);new['identity']['rfc_state']='reservation-only-18372-v1';plan['identity']=new['identity']
        transition={'from_identity':old['identity'],'to_identity':new['identity'],
            'owner_trace':'№18,372 Part C','reviewed_commit':'fixture'}
        new.update(transition=transition,transitions=[transition],frozen_independent_jobs_sha256=canonical(plan['episodes']))
        proof=verify_reuse_reservation(plan,new,meta,bars,refs,verified,old,[])
        self.assertEqual(proof['refusal_count'],0)
        self.assertFalse(proof['collection_authority'])
        self.assertFalse(proof['evaluation_ready'])
        broken=copy.deepcopy(plan);broken['episodes'][0]['date']='changed'
        with self.assertRaisesRegex(Refusal,'jobs_changed'):
            verify_reuse_reservation(broken,new,meta,bars,refs,verified,old,[])
        new['transitions']=[]
        with self.assertRaisesRegex(Refusal,'unregistered'):
            verify_reuse_reservation(plan,new,meta,bars,refs,verified,old,[])


if __name__=='__main__':unittest.main()
