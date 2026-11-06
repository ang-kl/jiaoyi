import copy
from pathlib import Path
import tempfile
import unittest

import full_exit_research as research


class FullExitTests(unittest.TestCase):
    def test_observed_features_and_strict_next_quote_do_not_look_ahead(self):
        start = research.context.ms('2025-08-05T00:00:00Z')
        qs = [(start+m*60000, 100+m, 100.1+m) for m in range(0, 61, 5)]
        qs += [(start+3601000, 161, 161.1), (start+3602000, 900, 900.1)]
        f, reason = research.features(qs, start)
        self.assertIsNone(reason)
        self.assertAlmostEqual(f['observed_range'], 60)
        self.assertEqual(f['sample_times'][-1], start+3600000)
        job = {'request': {'from':research.iso(start), 'to':research.iso(start+14400000)},
               'result': {'input_sha256':'fixture', 'input': {'contract':
                   {'digits':2, 'min_volume':10, 'step_volume':10}}}}
        row, request = research.prepare('own_momentum','US500','equity_index',job,
                                        {('US500',start):qs},100)
        self.assertEqual(row['side'], 1)
        self.assertEqual(request['entered_at'], research.iso(start+3601000))
        self.assertEqual(request['fill'], 161.1)
        # A decision-cost veto stands even when no subsequent fill is present.
        expensive = [(at, bid*0.001+100, ask*0.001+100) for at,bid,ask in qs[:13]]
        veto, request = research.prepare('own_momentum','US500','equity_index',job,
                                         {('US500',start):expensive},100)
        self.assertEqual(veto['status'],'candidate_cost_ge_r')
        self.assertIsNone(request)
        missing = [q for q in qs if q[0] != start+300000]
        self.assertEqual(research.features(missing,start)[1], 'missing_first_hour_samples')

    def test_metals_reversal_is_causal_and_requires_unanimity(self):
        start, mark = 0, 3600000
        corpus = {('USDJPY',start):[(0,100,100),(mark,101,101)],
                  ('USDCAD',start):[(0,100,100),(mark,101,101)],
                  ('GBPUSD',start):[(0,100,100),(mark,99,99)]}
        args = ('unanimous_peer_reversal','XAUUSD','metal',start,mark,(0,100,100),(mark,101,101))
        side, peers, reason = research.signal(*args,corpus)
        self.assertEqual(side,1)  # Reverses inverse-USD proxy, not own gold return.
        self.assertIsNone(reason)
        self.assertEqual(len(peers),3)
        future = copy.deepcopy(corpus)
        future[('USDJPY',start)].append((mark+1,90,90))
        self.assertEqual(research.signal(*args,future),(side,peers,reason))
        args = (*args[:4],mark+2,*args[5:])
        self.assertEqual(research.signal(*args,future)[2],'peer_disagreement')
        del corpus[('GBPUSD',start)]
        self.assertEqual(research.signal(*args,corpus)[2],'missing_predictor')

    def test_frozen_plan_and_missing_data_cannot_be_silently_relaxed(self):
        self.assertFalse(research.frozen_plan()['limits']['new_broker_jobs'])
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder)/'changed.json'
            path.write_bytes(research.PLAN.read_bytes()+b' ')
            with self.assertRaises(ValueError):
                research.frozen_plan(path)
        missing = dict(symbol='XAUUSD', **{'from':'2025-08-05T07:00:00Z'},
                       status='missing_predictor',replay=None)
        score = research.score_rows([missing],'trailing',24,4,1,['XAUUSD'])
        self.assertFalse(score['passes_screen'])
        self.assertEqual(score['data_gaps'],{'missing_predictor':1})


if __name__ == '__main__':
    unittest.main()
