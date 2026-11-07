import datetime as dt
import json
from pathlib import Path
import tempfile
import unittest

import seasonal_pilot as pilot
import seasonal_report as report
from broker_readonly import BrokerError


class SeasonalTests(unittest.TestCase):
    def test_frozen_scope_dst_and_protected_warmup(self):
        p=pilot.load()
        self.assertEqual(len(p['jobs']),80)
        self.assertEqual(len({j['date'] for j in p['jobs']}),20)
        self.assertEqual(len(p['bar_segments']),116)
        self.assertEqual(pilot.opening('US500','2024-01-22').hour,14)
        self.assertEqual(pilot.opening('US500','2024-07-22').hour,13)
        for segment in p['bar_segments']:
            self.assertFalse(pilot.protected(pilot.instant(segment['from']),pilot.instant(segment['to'])))
        self.assertTrue(any(j['protected_warmup_omitted'] for j in p['jobs']))

    def test_bar_decoder_rejects_truncation_and_duplicate_or_future_bars(self):
        part=dict(symbol_id=1,**{'from':'2024-01-22T00:00:00Z','to':'2024-01-22T00:30:00Z'})
        self.assertNotIn('count',pilot.bar_request(part))
        self.assertEqual(pilot.bar_request(part)['toTimestamp'],int(pilot.instant(part['to']).timestamp()*1000)-1)
        at=int(pilot.instant(part['from']).timestamp()/60)
        raw=dict(utcTimestampInMinutes=at,low=100,deltaHigh=10,deltaOpen=2,deltaClose=7,volume=3)
        value=dict(period=8,symbolId=1,hasMore=False,trendbar=[raw])
        self.assertEqual(pilot.decode_bars(value,part)[0]['close_e5'],107)
        for bad in [dict(value,hasMore=True),dict(value,trendbar=[raw,raw]),dict(value,trendbar=[dict(raw,utcTimestampInMinutes=at+30)])]:
            with self.assertRaises(BrokerError):pilot.decode_bars(bad,part)

    def test_missing_closes_and_four_week_significance_are_not_zero_or_a_pass(self):
        rows=[]
        for date in ['2024-01-22','2024-07-22','2025-05-12','2025-10-06']:
            models={'orb_30m':{'study':{'trade':{'entry':{'initial_r':1}},'marks':[{'reason':'simulated_entry'}]},'sensitivities':{'trailing':{'24':1.0}}},
                    'always_long_control':{'control':{'initial_r':1,'net_by_bps':{'24':0.0}}}}
            rows.append(dict(key=date,symbol='BTCUSD',cohort=date,date=date,status='complete',replay={'result':{'models':models,'quality':{'bid_ticks':2,'ask_ticks':2}}}))
        s=report.score(rows,'orb_30m','trailing','24',10)
        self.assertEqual(s['wins'],4);self.assertFalse(s['numerical_screen_pass'])
        comparison=report.paired(rows,'orb_30m','always_long_control')
        self.assertEqual(comparison['one_sided_weekly_sign_flip_p'],.0625)
        self.assertFalse(comparison['significance_pass'])
        rows[0]['replay']['result']['models']['orb_30m']['sensitivities']['trailing']['24']=None
        s=report.score(rows,'orb_30m','trailing','24',10)
        self.assertEqual((s['episodes'],s['entries'],s['closed'],s['censored_or_invalid']),(4,4,3,1))
        empty=report.score([dict(key='missing',symbol='BTCUSD',date='2024-01-22',status='missing')],'orb_30m','trailing','24',10)
        self.assertIsNone(empty['mean_r']);self.assertIsNone(empty['win_rate'])

    def test_report_refuses_empty_receipts_for_claimed_completed_jobs(self):
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder)
            (out/'ledger.jsonl').write_text('')
            (out/'freeze.json').write_text('{}')
            result=dict(receipts=[],provenance={},execution_sha256=pilot.sha(pilot.EXECUTION),rows=[dict(key='x',status='complete')])
            with self.assertRaisesRegex(ValueError,'receipt coverage'):
                report.verify_sources(out,result,pilot.load())
