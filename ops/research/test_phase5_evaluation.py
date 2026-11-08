import datetime as dt
import unittest
from phase5_evaluation import opening,sessions,ms,summary_values,split_bar_window,instant

class Phase5Tests(unittest.TestCase):
    def test_fixed_utc_windows_and_causal_complete_context(self):
        for date in ['2023-04-17','2023-10-16','2024-03-04','2024-08-12']:
            self.assertEqual(opening('US500',date),ms(date+'T14:30:00Z'))
            self.assertEqual(opening('SpotCrude',date),ms(date+'T13:00:00Z'))
            for symbol in ['US500','BTCUSD']:
                history=sessions(symbol,date)
                self.assertGreaterEqual(len(history),20)
                self.assertTrue(all(ms(s['close'])<opening(symbol,date) for s in history))
                self.assertTrue(all(ms(s['close'])-ms(s['open'])==14400000 for s in history))
                if symbol=='BTCUSD':self.assertEqual(len(history),45)
    def test_spent_failed_interval_is_never_recollected(self):
        a,b=instant('2024-01-01T00:00:00Z'),instant('2024-01-10T00:00:00Z')
        prior=[{'from':'2024-01-03T00:00:00Z','to':'2024-01-05T00:00:00Z','status':'failed'},
               {'from':'2024-01-05T00:00:00Z','to':'2024-01-07T00:00:00Z','status':'complete','cache':{'path':'verified','sha256':'fixture'}}]
        parts=split_bar_window(a,b,prior)
        self.assertEqual([p['mode'] for p in parts],['collect','blocked','reuse','collect'])
        self.assertTrue(all(instant(p['to'])<=instant('2024-01-03T00:00:00Z') or instant(p['from'])>=instant('2024-01-07T00:00:00Z') for p in parts if p['mode']=='collect'))
    def test_unresolved_and_flat_are_not_wins(self):
        r=summary_values([1.,-2.,0.],5,4,1)
        self.assertEqual((r['wins'],r['losses'],r['net_flat'],r['unresolved']),(1,1,1,1))
        self.assertAlmostEqual(r['net_win_rate'],1/3)
        self.assertEqual(r['profit_factor'],.5)
        self.assertIsNone(summary_values([],1,0,0)['net_win_rate'])
