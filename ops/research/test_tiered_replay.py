import unittest
import unified_replay as u
from tiered_replay import partial_context, paired

class TieredReplayTests(unittest.TestCase):
    def test_partial_endpoints_are_causal_and_do_not_impute_missing_horizons(self):
        start=u.ms('2024-01-01T00:00:00Z');sessions=[];bars={}
        for n in range(20):
            t=start+n*86400000;sessions.append({'open':u.stamp(t),'close':u.stamp(t+3600000),'lunch':None})
            bars[t]={'open_e5':100,'close_e5':101};bars[t+1800000]={'open_e5':101,'close_e5':102+n}
        bars.pop(start+1800000) # Interior gap does not change endpoint returns.
        at=start+20*86400000;c=partial_context(sessions,bars,at)
        self.assertEqual(c['status'],'complete');self.assertAlmostEqual(c['drift_20'],.21)
        self.assertEqual(u.causal_context(sessions,bars,at)['status'],'missing_segments')
        with self.assertRaisesRegex(ValueError,'noncausal'):partial_context(sessions,bars,start)
        bars.pop(start);self.assertEqual(partial_context(sessions,bars,at)['status'],'missing_segments')
    def test_missing_context_is_not_counted_as_directional_filter_success(self):
        rows=[]
        for status in ['complete','missing_segments']:
            rows.append({'episode_id':status,'symbol':'US500','date':'2024-01-01','own_context':{'status':status},'native':{
                'macro':{'reason':'own_5d20d_disagree_or_missing','pass':False},'arms':{u.ARMS[1]:{'forced':{'metrics':{'net_by_bps':{'24':-1.2},'gross_r':-1}}}}}})
        p=paired(rows,'forced','24')['categories']
        self.assertEqual(p['own_trend_veto']['losses'],1);self.assertEqual(p['own_context_unavailable']['losses'],1)
