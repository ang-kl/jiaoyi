import json
from fractions import Fraction
from pathlib import Path
import tempfile
import unittest

from replay_inventory import (
    SEAL_START, SEAL_END, RESEARCH_CUTOFF, checkpoints, quantiles,
    permitted, protected_dependency, reconcile, sha, stamp, execution_audit,
)


class ReplayInventoryTests(unittest.TestCase):
    def test_every_frozen_receipt_must_match_and_missing_is_not_zero(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'receipt.json';p.write_text('{"status":"complete"}\n')
            scope={'files':[{'path':str(p),'sha256':sha(p)}]}
            self.assertEqual(reconcile(scope)[0]['status'],'match')
            p.write_text('{"status":"failed"}\n')
            with self.assertRaisesRegex(ValueError,'mismatch'):reconcile(scope)
            p.unlink()
            with self.assertRaisesRegex(ValueError,'missing'):reconcile(scope)

    def test_exact_quantiles_and_grouped_checkpoint_spread_units(self):
        bids=[{'ts_ms':1000,'price_e5':10000000},{'ts_ms':3000,'price_e5':10000000}]
        asks=[{'ts_ms':1000,'price_e5':10001000},{'ts_ms':3000,'price_e5':10003000}]
        points=checkpoints(bids,asks,{'strategy:0':0,'window:0':0,'later':2000,'absent':9000},2)
        self.assertEqual(points['strategy:0'],points['window:0'])
        self.assertIsNone(points['absent'])
        a=points['strategy:0'];b=points['later']
        self.assertEqual((a['spread_e5'],b['spread_e5']),(1000,3000))
        prices=quantiles([Fraction(a['spread_e5'],100000),Fraction(b['spread_e5'],100000)])
        self.assertEqual(prices['exact']['p50_median'],'1/50')
        self.assertEqual(prices['exact']['p10'],'3/250')
        vals=[Fraction(20000*x['spread_e5'],x['bid_e5']+x['ask_e5']) for x in [a,b]]
        self.assertEqual(Fraction(quantiles(vals)['exact']['p25']),vals[0]+(vals[1]-vals[0])/4)
        self.assertIsNone(quantiles([])['values']['min'])
        stale=checkpoints(bids,[{'ts_ms':-10000,'price_e5':10001000}],{'x':0},2)
        self.assertIsNone(stale['x'])

    def test_protected_dependencies_and_tails_fail_closed(self):
        self.assertTrue(permitted(SEAL_START-60000,SEAL_START))
        self.assertFalse(permitted(SEAL_START-60000,SEAL_START+1))
        self.assertFalse(permitted(SEAL_START,SEAL_END))
        self.assertFalse(permitted(SEAL_END-1,SEAL_END+1))
        self.assertTrue(permitted(SEAL_END,SEAL_END+60000))
        self.assertFalse(permitted(RESEARCH_CUTOFF-5000,RESEARCH_CUTOFF+1))
        self.assertTrue(protected_dependency(SEAL_END,SEAL_END+60000,[{'ts_ms':SEAL_START}]))
        self.assertTrue(protected_dependency(SEAL_END,SEAL_END+60000,[],{'bids':[{'ts_ms':SEAL_START}],'asks':[]}))
        self.assertTrue(protected_dependency(SEAL_END,SEAL_END+60000,[],{'bids':[],'asks':[],
            'bars':[{'at':stamp(SEAL_START)}]}))

    def test_deal_ids_are_account_scoped_and_fees_do_not_prove_slippage(self):
        deal={'dealId':1,'positionId':2,'symbolId':10054,'filledVolume':10}
        data={'a':{'ctidTraderAccountId':43002148,'deal':[deal]},
              'b':{'ctidTraderAccountId':49236051,'deal':[dict(deal)]},
              'tests/fixture':{'ctidTraderAccountId':43002148,'deal':[dict(deal,dealId=99)]}}
        out=execution_audit(data)['report']
        self.assertEqual(out['raw_deal_records'],2)
        self.assertEqual(len(out['positions']),2)
        self.assertEqual(out['verified_round_trip_cost_samples'],0)


if __name__=='__main__':unittest.main()
