import copy
import unittest
from phase4_forensics import path_arithmetic,exit_class,independent_weeks,intersect,ms,gates,validate_gate_progress,embedded_m30_window,preparation_start,DAY

class Phase4ForensicTests(unittest.TestCase):
    def test_ledger_cost_arithmetic_and_bracket_anatomy(self):
        path={'entry':{'fill':100,'initial_r':2,'side':'long','digits':2,'broker_tp_distance':2.76,'volume':100,'entered_at':'2024-01-01T00:00:00Z'},
              'outcome':{'fills':[{'at':'2024-01-01T01:00:00Z','price':102.76,'volume':100,'reason':'broker_take_profit'}],'active_stop':100,'trail_applied':1},
              'metrics':{'gross_r':1.38,'net_by_bps':{'3.74':1.34208388,'24':1.136688}}}
        self.assertAlmostEqual(path_arithmetic(path)['net_r_at_24bp'],1.136688)
        self.assertEqual(exit_class(path,{'metrics':{ 'closed':True}}),'strict_target_1_38r')
        changed=copy.deepcopy(path);changed['metrics']['net_by_bps']['24']=1.38
        with self.assertRaisesRegex(ValueError,'net arithmetic'):path_arithmetic(changed)
        changed=copy.deepcopy(path);changed['outcome']['fills'][0].update(reason='gap_through_stop',price=99.9)
        self.assertEqual(exit_class(changed,{}),'breakeven_stop_after_1r')
        changed['outcome'].update(active_stop=98,trail_applied=0)
        self.assertEqual(exit_class(changed,{}),'initial_stop_loss')
        changed['outcome']['fills'][0]['reason']='session_end'
        self.assertEqual(exit_class(changed,{}),'forced_window_close')
        with self.assertRaisesRegex(ValueError,'falsely counted'):exit_class(changed,{'metrics':{'closed':True}})
    def test_unspent_week_selection_preserves_prep_and_label_separation(self):
        blocked={'from':'2022-11-01T00:00:00Z','to':'2023-01-28T00:00:00Z'}
        reg={'intervals':[blocked],'reserved_dates':['2023-03-20']}
        weeks,_=independent_weeks(reg);self.assertEqual(len(weeks),4)
        self.assertNotIn('2023-03-20',{d for w in weeks for d in w['dates']})
        previous_end=0
        for w in weeks:
            a,b=ms(w['preparation_envelope_from']),ms(w['envelope_to'])
            self.assertGreaterEqual(a,previous_end);previous_end=b
            self.assertFalse(intersect(a,b,ms(blocked['from']),ms(blocked['to'])))
            self.assertEqual(len(w['dates']),5)
            self.assertLess(b,ms('2025-10-24T00:00:00Z'))
    def test_gate_progress_cannot_turn_a_frozen_plan_into_financial_pass(self):
        r={'rfc_gates':gates(),'rfc_gates_passed':2,'new_independent_observations':0,
           'independent_evaluation_plan':{'reservation_status':'frozen_not_executed'},
           'financial_pass':False,'validation_opened':False,'new_broker_requests':0}
        self.assertEqual(validate_gate_progress(r),2)
        changed=copy.deepcopy(r);changed['rfc_gates'][9]['status']='passed';changed['rfc_gates_passed']=3
        with self.assertRaisesRegex(ValueError,'no independent'):validate_gate_progress(changed)
        changed=copy.deepcopy(r);changed['rfc_gates'][8]['status']='passed';changed['rfc_gates_passed']=3
        with self.assertRaisesRegex(ValueError,'planned is not executed'):validate_gate_progress(changed)
    def test_intraday_peer_bars_do_not_mark45days_as_spent(self):
        e={'source_path':'/workspace/jiaoyi-jan2024-peer-18342/peer-NAS100-2024-01-24.json',
           'source_window_utc':{'from':'2024-01-24T14:30:00Z','to':'2024-01-24T18:30:00Z'}}
        a,b,authority=embedded_m30_window(e)
        self.assertEqual(ms(b)-ms(a),4*3600000)
        self.assertEqual(authority,'verified_intraday_m30_collector_window')
    def test_future_preparation_covers_earlier_utc_open_across_dst(self):
        metadata={'session':{'open':'2023-11-20T14:30:00Z'},
                  'prior_45_calendar_days':[{'open':'2023-10-06T13:30:00Z'}]}
        self.assertEqual(preparation_start(metadata),ms('2023-10-06T13:30:00Z'))
        self.assertLess(preparation_start(metadata),ms(metadata['session']['open'])-45*DAY)
