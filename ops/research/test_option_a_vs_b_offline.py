import unittest
from option_a_vs_b_offline import merge_bars, score, protected, millis

class OfflineComparisonTests(unittest.TestCase):
    def test_cached_bar_conflict_and_seal_fail_closed(self):
        index={};bar={'ts_ms':millis('2024-01-22T17:00:00Z'),'close_e5':100}
        merge_bars(index,'US500',[bar]);merge_bars(index,'US500',[bar])
        with self.assertRaises(ValueError):merge_bars(index,'US500',[dict(bar,close_e5=101)])
        with self.assertRaises(ValueError):merge_bars(index,'US500',[dict(bar,ts_ms=millis('2025-08-19T00:00:00Z'))])
        self.assertTrue(protected(millis('2025-10-24T00:00:00Z')))

    def test_unresolved_and_flat_exits_do_not_create_wins(self):
        def row(status,net=None):return {'models':{'x':{'status':status,'metrics':None if net is None else {'net_by_bps':{'24':net},'gross_r':0}}}}
        s=score([row('closed_tick',-.48),row('censored_end_of_cache'),row('ambiguous_m30_path')],'x','24')
        self.assertEqual((s['entries'],s['closed'],s['wins'],s['unresolved']),(3,1,0,2))
        self.assertEqual(s['mean_net_r'],-.48)

if __name__=='__main__':unittest.main()
