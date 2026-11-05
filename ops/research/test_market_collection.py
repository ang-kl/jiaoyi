import copy
import unittest

import market_collection as c


class MarketCollectionTests(unittest.TestCase):
    def test_collection_checks_identity_digest_and_no_replay(self):
        source = dict(bars=[], bids=[dict(ts_ms=1, price_e5=100000)],
                      asks=[dict(ts_ms=1, price_e5=101000)],
                      contract=dict(digits=2, min_volume=1, step_volume=1))
        req = dict(account=49236051, symbol="BTCUSD", **{"from": "start", "to": "end"})
        job = dict(kind="recent_market_data", request=req, status="complete", result=dict(
            input=source, input_sha256=c.source_digest(source), study=None,
            qualification="RECENT_COLLECTION_ONLY_NOT_FINANCIAL_PASS"))
        c.validate(job, req)
        # JSON key ordering is irrelevant; canonical native struct order is fixed.
        self.assertEqual(c.source_digest(dict(reversed(list(source.items())))), c.source_digest(source))
        for change in ("tick", "study", "kind"):
            bad = copy.deepcopy(job)
            if change == "tick": bad["result"]["input"]["bids"][0]["price_e5"] += 1000
            elif change == "study": bad["result"]["study"] = {}
            else: bad["kind"] = "backtest"
            with self.assertRaises(ValueError): c.validate(bad, req)
