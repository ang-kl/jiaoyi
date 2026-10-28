"""A one-day review names the class. It does not close or reverse."""
import unittest

import day_review
import watches


def deal(symbol, side, account, net, day="28 Oct 2026", bot=True):
    return {"symbol": symbol, "side": side, "account": account, "net": net, "day": day, "bot": bot}


class DayReviewTest(unittest.TestCase):
    def test_both_sides_are_the_rule_and_not_a_reverse(self):
        deals = [
            deal("ETHUSD", "Buy", 1, -1), deal("ETHUSD", "Sell", 1, -1),
            deal("ETHUSD", "Buy", 2, -1), deal("ETHUSD", "Sell", 2, -1),
        ]
        reading = day_review.statement_day(deals, one_entry_in_force=False)
        self.assertEqual(reading["kind"], "fundamental")
        self.assertEqual(reading["fundamental"], ["ETHUSD"])
        self.assertFalse(reading["pr"])
        self.assertEqual(reading["close_or_reverse"], "not_registered")
        self.assertEqual(reading["sentiment"], "not_called")

    def test_a_copied_loss_is_a_trend_and_a_winner_is_not_closed(self):
        deals = [
            deal("SpotBrent", "Sell", 1, -4), deal("SpotBrent", "Sell", 2, -5),
            deal("USDJPY", "Buy", 1, 10), deal("USDJPY", "Buy", 2, 12),
        ]
        reading = day_review.statement_day(deals)
        self.assertEqual(reading["trend"], ["SpotBrent"])
        self.assertIn("USDJPY", reading["held"])
        self.assertFalse(reading["pr"])

    def test_a_repeat_is_a_pull_request_only_after_the_rule_is_running(self):
        deals = [deal("ETHUSD", "Buy", 1, -1), deal("ETHUSD", "Buy", 1, -1, day="28 Oct 2026")]
        before = day_review.statement_day(deals, one_entry_in_force=False)
        after = day_review.statement_day(deals, one_entry_in_force=True)
        self.assertFalse(before["pr"])
        self.assertTrue(after["pr"])
        self.assertEqual(after["repeats"][0]["opens"], 2)

    def test_phone_deals_are_not_the_bot(self):
        reading = day_review.statement_day([
            {"symbol": "NatGas", "side": "Buy", "account": 47790949, "net": -127.5, "day": "26 Oct 2026", "bot": False}
        ])
        self.assertEqual(reading["kind"], "none")

    def test_health_cluster_is_one_telegram_and_not_an_order(self):
        health = {
            "strategy3": [{
                "account_id": 1, "prereg_file": watches.V4_FILE, "prereg_hash": watches.V4_HASH,
                "performance": {"limit_reached": [{"limit": "drawdown_r"}]},
                "today": {},
            }],
            "episodes": {"rows": [
                {"symbol": "ETHUSD", "side": "buy", "copies": 4, "session_date": "2026-10-28"},
            ]},
        }
        events, state = watches.events(None, health, {"reports": []})
        self.assertIn("day_review", [event["event"] for event in events])
        detail = next(event["detail"] for event in events if event["event"] == "day_review")
        self.assertEqual(detail["kind"], "trend")
        self.assertFalse(detail["pr"])
        self.assertEqual(detail["close_or_reverse"], "not_registered")
        again, _ = watches.events(state, health, {"reports": []})
        self.assertNotIn("day_review", [event["event"] for event in again])


if __name__ == "__main__":
    unittest.main()
