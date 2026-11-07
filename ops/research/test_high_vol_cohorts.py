import unittest
import high_vol_cohorts as h


class HighVolTests(unittest.TestCase):
    def test_partial_cached_overlap_is_reused_and_only_unspent_residual_is_requested(self):
        a=h.instant('2025-04-01T00:00:00Z');b=h.instant('2025-04-11T00:00:00Z')
        prior=[dict(status='complete',cache={'path':'verified','sha256':'h'},**{'from':'2025-04-04T00:00:00Z','to':'2025-04-08T00:00:00Z'})]
        parts=h.split_bar_window(a,b,prior)
        self.assertEqual([p['mode'] for p in parts],['collect','reuse','collect'])
        self.assertEqual(parts[1]['cache']['path'],'verified')
        for part in parts:
            if part['mode']=='collect':self.assertFalse(h.overlaps(part,prior[0]))
        self.assertEqual(parts[0]['from'],h.stamp(a));self.assertEqual(parts[-1]['to'],h.stamp(b))

    def test_failed_or_unreceipted_spent_interval_cannot_be_recollected(self):
        a=h.instant('2025-04-01T00:00:00Z');b=h.instant('2025-04-11T00:00:00Z')
        for status in ['failed','uncertain']:
            old=dict(status=status,**{'from':h.stamp(a),'to':h.stamp(b)})
            self.assertEqual([p['mode'] for p in h.split_bar_window(a,b,[old])],['blocked'])
        parts=h.split_bar_window(a,b,[])
        self.assertEqual(len(parts),2)
        self.assertTrue(all(h.instant(p['to'])-h.instant(p['from'])<=h.dt.timedelta(days=7) for p in parts))

    def test_reused_bars_are_sliced_causally_and_conflicting_values_fail(self):
        a=int(h.instant('2025-04-07T07:00:00Z').timestamp()*1000)
        part={'from':'2025-04-01T07:00:00Z','to':'2025-04-08T07:00:00Z'}
        bars=[dict(ts_ms=a+i*1800000,close_e5=100) for i in range(4)]
        value=dict(status='complete',bars=bars)
        self.assertEqual(len(h.assemble_bars([(part,value)],a,a+3600000)),2)
        self.assertEqual(h.assemble_bars([(part,dict(status='failed',bars=bars))],a,a+3600000),[])
        conflict=dict(status='complete',bars=[dict(bars[0],close_e5=101)])
        with self.assertRaisesRegex(ValueError,'conflicting cached bar'):
            h.assemble_bars([(part,value),(part,conflict)],a,a+3600000)


if __name__=='__main__':unittest.main()
