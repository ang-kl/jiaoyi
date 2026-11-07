import copy
import unittest

from broker_readonly import BrokerError
import seasonal_pilot as pilot
import us500_prep_cost as audit


class PrepCostTests(unittest.TestCase):
    def test_session_gaps_and_dst_are_valid_but_identity_and_truncation_are_not(self):
        part = dict(symbol_id=1, **{'from':'2024-01-19T14:30:00Z','to':'2024-01-23T00:00:00Z'})
        bars = [dict(utcTimestampInMinutes=int(pilot.instant(at).timestamp()/60),
                     low=100, deltaHigh=10, deltaOpen=2, deltaClose=7, volume=3)
                for at in ['2024-01-19T14:30:00Z','2024-01-22T14:30:00Z']]
        value = dict(period=8, symbolId=1, hasMore=False, trendbar=bars)
        self.assertEqual(len(pilot.decode_bars(value,part)),2)
        self.assertNotIn('count',pilot.bar_request(part))
        self.assertEqual(pilot.opening('US500','2024-01-22').strftime('%H:%M'),'14:30')
        self.assertEqual(pilot.opening('US500','2024-07-22').strftime('%H:%M'),'13:30')
        for bad, reason in [(dict(value,period=9),'bar_period_unverified'),
                            (dict(value,symbolId=2),'bar_symbol_mismatch'),
                            (dict(value,hasMore=True),'bar_response_truncated'),
                            (dict(value,hasMore=None),'bar_completeness_unverified')]:
            with self.assertRaisesRegex(BrokerError,reason): pilot.decode_bars(bad,part)

    def test_large_unrelated_bar_count_does_not_restore_missing_opening_slots(self):
        start = 1705933800000
        unrelated = [dict(ts_ms=start-(i+1)*1800000,volume=1) for i in range(1400)]
        opening = [dict(ts_ms=start+i*1800000,volume=1) for i in range(6)]
        self.assertFalse(audit.opening_coverage(unrelated,start)['complete'])
        self.assertTrue(audit.opening_coverage(opening,start)['complete'])
        self.assertFalse(audit.opening_coverage(unrelated+opening[1:],start)['complete'])
        self.assertFalse(audit.opening_coverage([dict(b,volume=0) for b in opening],start)['complete'])

    def test_matched_costs_preserve_duplicates_losses_censoring_and_native_accounting(self):
        def row(date, gross):
            e = dict(entered_at=date,side='long',fill=100.0,initial_r=1.0,volume=1)
            o = dict(remaining_volume=int(gross is None),scenario_gross_r=gross,
                     fills=[] if gross is None else [dict(price=100+gross,volume=1)])
            net = None if gross is None else gross-(200+gross)*24/10000
            return dict(symbol='US500',key=date,protocol_flags=['retained_failure'],models={
                's3_two_stage':dict(study=dict(trade=dict(entry=e,exit_comparison=dict(candidate=o))),
                                    sensitivities=dict(trailing={'24':net}))})
        rows = [row('2024-01-22',1.38),row('2024-01-23',0),row('2024-01-24',None)]
        records, duplicates = audit.matched_trades([('a',rows),('b',[copy.deepcopy(rows[0])])],'s3_two_stage')
        self.assertEqual(duplicates,1)
        result = audit.ledgers(records)
        for key in ['ledger_a_stress_24bp','ledger_b_unverified_cost_sensitivity']:
            self.assertEqual((result[key]['closed_trades'],result[key]['wins'],result[key]['losses'],result[key]['censored']),(2,1,1,1))
            self.assertFalse(result[key]['financial_pass'])
        self.assertAlmostEqual(result['rows'][0]['ledger_a_net_r'],0.896688)
        self.assertAlmostEqual(result['rows'][0]['ledger_b_net_r'],1.339724)
        self.assertLess(result['rows'][1]['ledger_b_net_r'],0)
        self.assertIsNone(result['rows'][2]['ledger_b_net_r'])
        bad = copy.deepcopy(rows[0]);bad['models']['s3_two_stage']['sensitivities']['trailing']['24']=999
        with self.assertRaisesRegex(ValueError,'conflicting trade identity'):
            audit.matched_trades([('a',rows),('b',[bad])],'s3_two_stage')
        corrupt = copy.deepcopy(records[0]);corrupt['identity']['primary_net']=999
        with self.assertRaisesRegex(ValueError,'primary native net accounting mismatch'):audit.net_on_fills(corrupt,24)


if __name__ == '__main__': unittest.main()
