"""Tests for the five watches. No network, no broker."""
import unittest

import watches


def health(file=watches.V4_FILE, digest=watches.V4_HASH, decisions=None, trades=None, limit=None):
    return {
        "status": "ok",
        "strategy3": [{
            "account_id": 46130058,
            "prereg_file": file,
            "prereg_hash": digest,
            "today": {
                "recent_decisions": decisions or [],
                "trades_today": trades or [],
            },
            "performance": {"limit_reached": limit},
        }],
    }


def operations(wins=6, count=20, payoff=0.4, diagnostics=None):
    return {
        "reports": [{
            "account_id": 46130058,
            "strategy": "s3",
            "report": {"assessment": {"window": {
                "wins": wins, "count": count, "realized_payoff": payoff,
            }}},
        }],
        "account_source_diagnostics": diagnostics if diagnostics is not None else [{
            "account_id": 47790949,
            "pnl_coverage_diagnostics": {"extra_pnl_position_count": 1},
            "pnl_coverage_complete": False,
            "book_stable": True,
        }],
    }


class WatchesTest(unittest.TestCase):
    def test_same_length_wrong_hash_is_a_mismatch(self):
        self.assertTrue(watches.hash_watch(health(digest="b"*64))["mismatch"])

    def test_real_source_diagnostic_uses_counts_and_keeps_missing_evidence_unknown(self):
        row={"account_id":47790949,"pnl_coverage_diagnostics":{"extra_pnl_position_count":1},"pnl_coverage_complete":False,"book_stable":True}
        self.assertTrue(watches.account_47790949_watch(operations(diagnostics=[row]))["still_open"])
        row={"account_id":47790949,"pnl_coverage_diagnostics":{}}
        self.assertIsNone(watches.account_47790949_watch(operations(diagnostics=[row]))["still_open"])
        row={"account_id":47790949,"pnl_coverage_diagnostics":{"extra_pnl_position_count":0},"pnl_coverage_complete":True,"book_stable":True}
        self.assertFalse(watches.account_47790949_watch(operations(diagnostics=[row]))["still_open"])

    def test_changed_measurement_does_not_repeat_a_limit_transition(self):
        _,state=watches.events(None,health(limit=[{"limit":"drawdown_r","value":11.0}]),operations())
        emitted,_=watches.events(state,health(limit=[{"limit":"drawdown_r","value":12.0}]),operations())
        self.assertNotIn("limit_changed",[e["event"] for e in emitted])

    def test_hash_matches_v3_file_and_full_digest(self):
        reading = watches.hash_watch(health())
        self.assertFalse(reading["mismatch"])
        self.assertTrue(reading["rows"][0]["matches_v3"])

    def test_hash_mismatch_on_old_file_or_short_digest(self):
        self.assertTrue(watches.hash_watch(health(file="docs/preregistration-2026-10-25-strategy3.md"))["mismatch"])
        self.assertTrue(watches.hash_watch(health(digest="abc"))["mismatch"])
        self.assertTrue(watches.hash_watch({"strategy3": []})["mismatch"])

    def test_latest20_names_the_gap_and_does_not_invent_a_pass(self):
        reading = watches.latest20_watch(operations())
        gap = reading["gaps"][0]
        self.assertEqual(gap["wins_short"], 9)
        self.assertAlmostEqual(gap["payoff_short"], 0.98)
        self.assertTrue(gap["sufficient"])
        short = watches.latest20_watch(operations(count=7))["gaps"][0]
        self.assertFalse(short["sufficient"])

    def test_cost_gate_counts_only_the_reserve_reason(self):
        decisions = [
            {"symbol": "ETHUSD", "reason": "cost reserve C (1.2) >= R (0.4): skipped"},
            {"symbol": "ETHUSD", "reason": "cost reserve C (1.1) >= R (0.5): skipped"},
            {"symbol": "EURUSD", "reason": "paused"},
        ]
        tally = watches.cost_gate_watch(health(decisions=decisions))["skips_by_symbol"]
        self.assertEqual(tally, {"ETHUSD": 2})

    def test_recovery_reads_health_and_does_not_invent_a_path(self):
        missing = watches.recovery_watch({})
        self.assertFalse(missing["available"])
        self.assertFalse(missing["threshold_crossed"])
        present = watches.recovery_watch({"recovery": {
            "available": True, "stops": 6, "score": 0.2, "threshold_crossed": True,
            "path": "later owned close on the same UTC date, not a quote path",
        }})
        self.assertTrue(present["available"])
        self.assertTrue(present["threshold_crossed"])
        self.assertIn("not a quote path", present["path"])

    def test_47790949_closes_when_excluded_from_the_owned_window(self):
        row = {"account_id": 47790949, "pnl_coverage_diagnostics": {"extra_pnl_position_count": 1},
               "unmatched_excluded_from_owned_window": True}
        reading = watches.account_47790949_watch(operations(diagnostics=[row]))
        self.assertFalse(reading["still_open"])
        self.assertTrue(reading["excluded_from_owned_window"])

    def test_episode_watch_names_copies_and_leaves_the_account_window(self):
        reading = watches.episode_watch({"episodes": {
            "available": True, "episodes": 4, "latest20_episodes": 4, "copied": 2,
            "note": "one signal copied across accounts is one episode",
        }})
        self.assertTrue(reading["available"])
        self.assertEqual(reading["copied"], 2)

    def test_47790949_stays_open_when_unmatched_is_named(self):
        reading = watches.account_47790949_watch(operations())
        self.assertTrue(reading["present"])
        self.assertTrue(reading["still_open"])

    def test_events_fire_once_on_change_and_not_on_the_same_poll(self):
        sent = health(trades=[{"kind": "open", "outcome": "filled", "symbol": "EURUSD", "side": "buy", "at": "t1"}])
        first, state = watches.events(None, sent, operations())
        self.assertIn("entry_sent", [event["event"] for event in first])
        again, _ = watches.events(state, sent, operations())
        self.assertEqual([event["event"] for event in again], [])

    def test_limit_change_and_hash_mismatch_are_events(self):
        limited = health(limit=[{"limit": "drawdown_r"}])
        events, state = watches.events(None, limited, operations())
        self.assertIn("limit_changed", [event["event"] for event in events])
        old = health(file="docs/preregistration-2026-10-25-strategy3.md")
        events, _ = watches.events(state, old, operations())
        self.assertIn("hash_mismatch", [event["event"] for event in events])


if __name__ == "__main__":
    unittest.main()
