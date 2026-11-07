import time
import unittest
from broker_readonly import Broker, BrokerError, decode, ticks


class ReadOnlyTests(unittest.TestCase):
    def test_decoder_signs_ties_and_invalid_window(self):
        expected = [dict(ts_ms=10, price_e5=100), dict(ts_ms=8, price_e5=95)]
        self.assertEqual(decode([dict(timestamp=10, tick=100), dict(timestamp=-2, tick=-5)], 0, 10), expected)
        self.assertEqual(decode([dict(timestamp=10, tick=100), dict(timestamp=2, tick=5)], 0, 10), expected)
        self.assertEqual(len(decode([dict(timestamp=10, tick=100), dict(timestamp=0, tick=1)], 0, 10)), 2)
        with self.assertRaises(BrokerError): decode([dict(timestamp=11, tick=100)], 0, 10)

    def test_bisection_uses_disjoint_complete_leaves_and_budget(self):
        class Fake:
            def __init__(self): self.windows = []
            def request(self, _, p):
                a, b = p['fromTimestamp'], p['toTimestamp']
                self.windows.append((a, b))
                return dict(hasMore=b-a > 1, tickData=[dict(timestamp=b, tick=100)])
        f = Fake(); budget = dict(pages=0, rows=0)
        result = ticks(f, 1, 1, 0, 3, budget, time.monotonic()+5)
        self.assertEqual(f.windows, [(0, 3), (0, 1), (2, 3)])
        self.assertEqual([x['ts_ms'] for x in result], [1, 3])
        self.assertEqual(budget, dict(pages=3, rows=2))
        with self.assertRaises(BrokerError): ticks(f, 1, 1, 0, 3, dict(pages=120, rows=0), time.monotonic()+5)

    def test_order_and_wrong_account_refused_before_transport(self):
        broker = Broker()
        with self.assertRaises(BrokerError): broker.request(2106, {})
        with self.assertRaises(BrokerError): broker.request(2145, dict(ctidTraderAccountId=43002148))
