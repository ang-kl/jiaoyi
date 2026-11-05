import json
import tempfile
import unittest
from pathlib import Path

import cost_context as c


class CostContextTests(unittest.TestCase):
    def test_causal_quotes_exclude_future_stale_crossed_and_missing_sides(self):
        source = {"bids": [{"ts_ms": 0, "price_e5": 100000},
                           {"ts_ms": 7000, "price_e5": 100000},
                           {"ts_ms": 9000, "price_e5": 120000}],
                  "asks": [{"ts_ms": 1, "price_e5": 110000},
                           {"ts_ms": 8000, "price_e5": 110000}]}
        qs = c.quotes(source)
        self.assertEqual([q[0] for q in qs], [1, 8000])
        self.assertEqual(c.prior(qs, 7999)[0], 1)
        self.assertIsNone(c.prior(qs, 70001))
        self.assertIsNone(c.first(qs, 8001, 9000))
        self.assertIsNone(c.first(qs, 2, 8000))  # exclusive deadline

    def test_factor_excludes_target_and_later_window_and_orients_usd(self):
        start, end = 0, 3600000
        def series(last):
            return [(0, 100, 100), (end, last, last), (end + 1, 1, 1)]
        corpus = {(s, start): series(110 if s in ("USDJPY", "USDCAD") else 90)
                  for s in c.USD}
        corpus[("EURUSD", start)] = series(10000)  # target must not influence factor
        value, parts = c.factor("EURUSD", "fx", start, end, corpus, {})
        self.assertLess(value, 0)
        self.assertEqual(len(parts), 5)
        self.assertNotIn("EURUSD", [x["symbol"] for x in parts])
        self.assertTrue(all(x["last_at"] <= end for x in parts))
        later = {(s, 1): q for (s, _), q in corpus.items()}
        self.assertIsNone(c.factor("XAUUSD", "metal", start, end, later, {})[0])
        self.assertIsNone(c.factor("NatGas", "energy", start, end, corpus, {})[0])

    def test_executable_costs_charge_spread_once_and_keep_break_even_negative(self):
        entry, close = (0, 99, 101), (1, 109, 111)
        long = c.outcome(entry, close, 1, 24)
        self.assertEqual(long["executable_before_fees_bps"], 800)
        self.assertAlmostEqual(long["stress_after_slippage_bps"], 749.6)
        self.assertAlmostEqual(long["break_even_per_side_bps"], 8 / 210 * 10000)
        short = c.outcome(entry, close, -1, 0)
        self.assertEqual(short["executable_before_fees_bps"], -1200)
        self.assertLess(short["break_even_per_side_bps"], 0)
        self.assertIsNone(long["qualified_net_bps"])

    def test_receipts_deduplicate_preserve_unknown_and_reject_conflicts(self):
        row = dict(strategy="s3", account_id=43002148, entry_key="key", kind="open",
                   symbol="BTCUSD", fill_price=100, entry_cost=None)
        with tempfile.TemporaryDirectory() as directory:
            a, b = Path(directory) / "a.json", Path(directory) / "b.json"
            a.write_text(json.dumps({"rows": [row]}))
            b.write_text(a.read_text())
            result = c.receipt_coverage([a, b])
            self.assertEqual(result["filled_opens"], 1)
            self.assertEqual(result["measured_entries"], 0)
            self.assertIsNone(result["calibrated_per_side_bps"])
            b.write_text(json.dumps({"rows": [{**row, "fill_price": 101}]}))
            with self.assertRaisesRegex(ValueError, "conflicting receipt"):
                c.receipt_coverage([a, b])

    def test_comparisons_use_identical_observed_episodes(self):
        rows = []
        for start in (0, 1):
            for variant in ("own_momentum", "always_long", "peer_continuation"):
                value = None if start == 1 and variant == "peer_continuation" else {"stress_after_slippage_bps": 100 if start else -10}
                rows.append(dict(symbol="BTCUSD", start_ms=start, horizon_minutes=30,
                                 asset_class="crypto", variant=variant, side=1, outcome=value))
        paired = c.matched_controls(rows)
        self.assertEqual(len(paired), 2)
        self.assertTrue(all(r["matched"] == 1 and r["mean_stress_delta_bps"] == 0 for r in paired))


if __name__ == "__main__":
    unittest.main()
