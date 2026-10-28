"""Evidence-grain regressions for read-only statement/episode observations."""
import copy
import unittest

import day_review
import watches

HASH = "a" * 64


def position(account, ident, side="buy", net_r=-1, session="2026-10-28", **extra):
    row = {"ownership_verified": True, "account_id": account, "position_id": ident,
           "entry_key": "entry-" + str(ident), "symbol": "TEST", "side": side,
           "session_date": session, "prereg_hash": HASH, "position_complete": True,
           "net_r": net_r, "opened_at": session + "T10:00:00+00:00",
           "one_entry_rule_in_force": True, "rule_prereg_hash": HASH,
           "rule_effective_at": session + "T09:00:00+00:00"}
    row.update(extra)
    return row


def episode(side="buy", accounts=None, session="2026-10-28", **extra):
    accounts = [1, 2] if accounts is None else accounts
    row = {"symbol": "TEST", "side": side, "accounts": accounts,
           "copies": len(accounts), "session_date": session, "prereg_hash": HASH,
           "mark_minutes": "30", "last_closed_at": session + "T11:00:00Z"}
    row.update(extra)
    return row


class DayReviewTest(unittest.TestCase):
    def test_directions_and_copies_without_pnl_do_not_establish_losses(self):
        reading = day_review.episode_review({"episodes": {"rows": [episode(), episode("sell")]}})
        self.assertEqual(reading["outcome_state"], "unknown")
        self.assertEqual(reading["two_sided_losses"], [])
        self.assertEqual(reading["copied_losses"], [])
        self.assertEqual(reading["fundamental"], [])
        self.assertFalse(reading["pr"])
        self.assertEqual(reading["close_or_reverse"], "not_registered")

    def test_only_the_same_losing_side_and_episode_can_be_a_copied_loss(self):
        reading = day_review.statement_day([position(1, 1), position(2, 2, side="sell", net_r=2)])
        self.assertEqual(reading["copied_losses"], [])
        self.assertEqual(reading["two_sided_losses"], [])
        duplicate_account = day_review.statement_day([
            position(1, 1, mark_minutes="30"), position(1, 2, mark_minutes="30")])
        self.assertEqual(duplicate_account["copied_losses"], [])
        self.assertEqual(duplicate_account["copied_symbols"], [])
        # Explicit measured per-account outcomes establish a descriptive loss,
        # not its economic cause or a winning reverse-trade rule.
        row = episode(outcomes=[{"account_id": 1, "state": "measured", "net_r": -1},
                                {"account_id": 2, "state": "measured", "net_r": -0.5}])
        measured = day_review.episode_review({"episodes": {"rows": [row]}})
        self.assertEqual(measured["copied_losses"], ["TEST"])
        self.assertEqual(measured["kind"], "loss_pattern")
        self.assertEqual(measured["fundamental"], [])

    def test_partial_closes_and_unproven_global_version_never_create_repeat_proof(self):
        row = position(1, 1)
        partials = [dict(row, deal_id=10), dict(row, deal_id=11)]
        reading = day_review.statement_day(partials, one_entry_in_force=True)
        self.assertFalse(reading["pr"])
        self.assertEqual(reading["repeats"], [])
        raw = [{"bot": True, "account": 1, "symbol": "TEST", "side": "Buy", "net": -1,
                "day": "28 Oct 2026"}] * 2
        self.assertFalse(day_review.statement_day(raw, one_entry_in_force=True)["pr"])
        later = position(1, 2, opened_at="2026-10-28T10:30:00+00:00")
        self.assertTrue(day_review.statement_day([row, later])["pr"])
        self.assertFalse(day_review.statement_day([row, dict(later, rule_prereg_hash="b" * 64)])["pr"])
        # Manual closing channel does not override proven opening ownership.
        self.assertTrue(day_review.statement_day([row, dict(later, bot=False)])["pr"])

    def test_new_day_hash_or_outcome_changes_material_signature_without_clock_churn(self):
        source = {"episodes": {"rows": [episode()]}}
        first = day_review.episode_review(source)
        self.assertEqual(first["signature"], day_review.episode_review(copy.deepcopy(source))["signature"])
        for change in [{"session_date": "2026-10-29"}, {"prereg_hash": "b" * 64},
                       {"outcomes": [{"account_id": 1, "state": "measured", "net_r": -1}]}]:
            changed = {"episodes": {"rows": [dict(source["episodes"]["rows"][0], **change)]}}
            self.assertNotEqual(first["signature"], day_review.episode_review(changed)["signature"])
        # A stated session filter never pools another day's or hash's losses.
        rows = [position(1, 1), position(2, 2, side="sell", session="2026-10-29")]
        self.assertEqual(day_review.statement_day(rows, session_date="2026-10-28")["two_sided_losses"], [])

    def test_copy_topology_is_not_a_second_telegram_loss_alert(self):
        health = {"strategy3": [{"account_id": 1, "prereg_file": watches.V4_FILE,
                  "prereg_hash": watches.V4_HASH,
                  "performance": {"limit_reached": [{"limit": "drawdown_r"}]}, "today": {}}],
                  "episodes": {"rows": [episode()]}}
        events, state = watches.events(None, health, {"reports": []})
        self.assertNotIn("day_review", [event["event"] for event in events])
        self.assertEqual(state["day_review"]["outcome_state"], "unknown")
        again, _ = watches.events(state, health, {"reports": []})
        self.assertEqual(again, [])


if __name__ == "__main__":
    unittest.main()
