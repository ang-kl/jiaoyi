"""Three timed, read-only acceptance journeys for the requested market report."""
import copy
import datetime as dt
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from test_worker import w
import market_validation as mv
import reporting


class MarketValidationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.start = dt.datetime.fromisoformat("2026-10-29T06:35:00+08:00")
        self.due = dt.datetime.fromisoformat("2026-10-29T09:00:00+08:00")
        self.clock = self.start
        self.c = self.coordinator(self.root)
        self.v = mv.MarketValidation(self.c)

    def tearDown(self):
        self.temp.cleanup()

    def coordinator(self, root):
        return w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": str(root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt",
            "GH_TOKEN": "fixture-gh", "RAILWAY_TOKEN": "fixture-railway",
            "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap",
            "ENGINEERING_WORK_UNTIL": "2026-10-29T07:30:00+08:00",
            "ENGINEERING_VALIDATION_REPORT_AT": self.due.isoformat(),
            "ENGINEERING_VALIDATION_CHAT_REFERENCES": "№17,743–17,744 §§1–2; by Codex, effort unreported",
            "TELEGRAM_BOT_TOKEN": "fixture-telegram", "TELEGRAM_OWNER_CHAT_ID": "313940231",
        }))

    def packets(self, *, new_trail=False, late_tokyo=False):
        at = self.clock.astimezone(dt.timezone.utc).isoformat()
        account = 43002148
        prep = []
        for symbol, opened, completed in (
                ("AUS200", "2026-10-29T07:00:00+08:00", "2026-10-29T06:44:00+08:00"),
                ("JPN225", "2026-10-29T08:00:00+08:00",
                 "2026-10-29T07:54:00+08:00" if late_tokyo else "2026-10-29T07:44:00+08:00")):
            opened = dt.datetime.fromisoformat(opened)
            completed = dt.datetime.fromisoformat(completed)
            deadline = opened - dt.timedelta(minutes=10)
            observed = self.clock >= completed
            prep.append({"symbol": symbol, "session_open": opened.isoformat(),
                "session_close": (opened + dt.timedelta(hours=6)).isoformat(),
                "deadline_at": deadline.isoformat(),
                "preparation_starts_at": (opened - dt.timedelta(minutes=20)).isoformat(),
                "completed_at": completed.isoformat() if observed else None,
                "deadline_met": completed <= deadline if observed else None,
                "state": ("prepared_late" if completed > deadline else "prepared") if observed else "scheduled",
                "history_ready": observed, "metadata_ready": observed,
                "commission_ready": observed, "spread_ready": observed})
        health = {"status": "ok", "trading": [], "strategy3": []}
        for section in ("trading", "strategy3"):
            for identity in (account, 46130058, 47790949, 49236051):
                health[section].append({"account_id": identity, "prereg_hash": "a" * 64,
                    "liveness": {"state": "alive", "fast_monitor": {
                        "state": "observing", "session_connected": True,
                        "exit_checks_completed": 10 + int((self.clock - self.start).total_seconds()),
                        "last_work_completed_at": at,
                        "last_watchdog_completed_at": at, "unresolved_open_count": 0},
                        "broker_snapshot": {"source_complete": True, "observed_at": at}},
                    "open_positions": [], "today": {"trades_today": []},
                    "preopen_preparation": {"symbols": copy.deepcopy(prep)} if section == "strategy3" else {}})
        assessment = {"state": "below_target", "history_complete": True,
            "window": {"count": 20, "wins": 5, "net_r": -1.0,
                "win_rate": .25, "realized_payoff": 2.0,
                "realized_payoff_state": "defined", "position_ids": list(range(1, 21))}}
        operations = {"as_of": at, "reports": [
            {"account_id": account, "strategy": "s3", "report": {"assessment": assessment}}],
            "margin_admission": [{"account_id": account, "configured": True, "floor_pct": 250}]}
        checks = {"owned_parent": True, "precision_known": True,
            "original_r_matches": True, "request_matches_plan": True,
            "request_uses_original_r": True, "stop_improves": True,
            "broker_acknowledged": True, "broker_stop_matches": True,
            "take_profit_preserved": True}
        trails = [{"account_id": account, "position_id": 81, "entry_key": "old-trail",
            "open_entry_key": "owned-old", "state": "passed", "checks": checks,
            "acknowledged_at": (self.start - dt.timedelta(hours=1)).isoformat()}]
        if new_trail:
            trails.append({"account_id": account, "position_id": 82, "entry_key": "new-trail",
                "open_entry_key": "owned-new", "state": "passed", "checks": checks,
                "acknowledged_at": "2026-10-29T06:44:00+08:00"})
        return {"/health": health, "/operations/reports": operations,
            "/operations/trailing-receipts": {"as_of": at, "receipts": trails},
            "/operations/stop-loss-reviews": {"as_of": at, "cases": []}}

    def fetch(self, route):
        return copy.deepcopy(self.packets()[route])

    def test_monitoring_crosses_mutation_cutoff_without_model_or_order_work(self):
        private = {"state": "available", "statement_report": {
            "report_key": "ny:2026-10-28", "status": "sent",
            "period_to": "2026-10-28T21:00:00+00:00", "text": "PRIVATE-STATEMENT-TEXT"},
            "accounts": [{"account_id": identity, "observed_at": self.start.isoformat(),
                          "balance": 1234.567, "floating": -987.654}
                         for identity in (43002148, 46130058, 47790949, 49236051)]}
        with patch.object(self.v, "_fetch", side_effect=self.fetch) as fetch, \
             patch.object(self.v, "_evidence", return_value=private), \
             patch.object(self.c, "command", side_effect=AssertionError("report must not run commands")), \
             patch.object(reporting, "send_document", side_effect=AssertionError("report is not due")):
            self.v.tick(clock=self.clock)
            first = self.v.status()
            self.assertEqual(first["observations"], 1)
            self.assertIsNotNone(first["baseline_at"])
            self.assertEqual(first["private_evidence_state"], "available")
            self.assertEqual(first["account_snapshot_count"], 4)
            self.assertTrue(first["statement_report_available"])
            self.assertEqual(first["statement_report_period"], "2026-10-28")
            public = json.dumps(first)
            for value in ("PRIVATE-STATEMENT-TEXT", "1234.567", "-987.654", "fixture-telegram"):
                self.assertNotIn(value, public, "public scheduling status must not expose private evidence")
            self.v.tick(clock=self.clock + dt.timedelta(seconds=30))
            self.assertEqual(fetch.call_count, 4, "no per-tick refetch or inference")
            self.clock = dt.datetime.fromisoformat("2026-10-29T07:40:00+08:00")
            self.assertFalse(self.c.config.mutation_allowed(self.clock))
            self.v.tick(clock=self.clock)
            later = self.v.status()
            self.assertEqual(later["observations"], 2)
            self.assertEqual(later["baseline_at"], first["baseline_at"])
            self.assertNotEqual(later["last_sample_at"], first["last_sample_at"])
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(self.c.store.jobs(), [])

        # A recoverable private-source failure is retried on the minute lane,
        # without resetting the observation baseline or waiting until09:00.
        recovering = mv.MarketValidation(self.coordinator(self.root / "recovering"))
        healthy = {"state": "available", "accounts": [{"observed_at": self.start.isoformat()}] * 4,
                   "statement_report": {"status": "sent", "report_key": "ny:2026-10-28",
                                        "period_to": "2026-10-28T21:00:00Z"}}
        with patch.object(recovering, "_fetch", side_effect=self.fetch), \
             patch.object(recovering, "_evidence", side_effect=[
                 {"state": "unknown", "reason": "private_readonly_source_CalledProcessError"}, healthy]) as evidence:
            recovering.tick(clock=self.start)
            self.assertEqual(recovering.status()["private_source_error"], "private_readonly_source_CalledProcessError")
            baseline = recovering.status()["baseline_at"]
            recovering.tick(clock=self.start + dt.timedelta(seconds=30))
            self.assertEqual(evidence.call_count, 1)
            recovering.tick(clock=self.start + dt.timedelta(seconds=60))
            self.assertEqual(evidence.call_count, 2)
            self.assertEqual(recovering.status()["baseline_at"], baseline)
            self.assertEqual(recovering.status()["private_evidence_state"], "available")
            self.assertIsNone(recovering.status()["private_source_error"])
            recovering.tick(clock=self.start + dt.timedelta(seconds=120))
            self.assertEqual(evidence.call_count, 2)

    def test_report_uses_deadline_receipts_and_new_broker_evidence_not_old_samples(self):
        with patch.object(self.v, "_fetch", side_effect=lambda route: copy.deepcopy(self.packets()[route])):
            self.v.tick(clock=self.clock)
        self.clock = dt.datetime.fromisoformat("2026-10-29T06:45:00+08:00")
        pending = self.packets(new_trail=True)
        awaiting = pending["/operations/trailing-receipts"]["receipts"][-1]
        awaiting.update(state="unknown", acknowledged_at=None,
                        decided_at="2026-10-29T06:44:00+08:00")
        awaiting["checks"]["broker_acknowledged"] = False
        with patch.object(self.v, "_fetch", side_effect=lambda route: copy.deepcopy(pending[route])):
            self.v.tick(clock=self.clock)
        self.assertEqual(self.v._state["trailing"]["unknown"], 1)
        self.clock += dt.timedelta(minutes=1)
        with patch.object(self.v, "_fetch", side_effect=lambda route: copy.deepcopy(self.packets(new_trail=True)[route])):
            self.v.tick(clock=self.clock)
        self.assertEqual(self.v._state["trailing"]["passed"], 1)
        self.assertEqual(self.v._state["trailing"]["unknown"], 0,
                         "a later acknowledgement resolves the same observed amendment")
        self.clock = self.due
        final = self.packets(new_trail=True, late_tokyo=True)
        final["/operations/trailing-receipts"]["receipts"].append({
            "account_id": 43002148, "position_id": 83, "entry_key": "missing-checks",
            "state": "passed", "checks": {}, "acknowledged_at": "2026-10-29T08:45:00+08:00"})
        with patch.object(self.v, "_fetch", side_effect=lambda route: copy.deepcopy(final[route])), \
             patch.object(reporting, "send_document", return_value=72) as post:
            self.v.tick(clock=self.clock)
        self.assertEqual(post.call_count, 1)
        self.assertEqual(self.v._state["trailing"]["passed"], 1, "the old acknowledgement is not new proof")
        self.assertEqual(self.v._state["trailing"]["unknown"], 1,
                         "an empty check object cannot certify a passed amendment")
        body = post.call_args.args[3]
        self.assertIn("PASS", body)
        self.assertIn("FAIL", body, "late receipt must not become an on-time preparation PASS")
        self.assertIn("UNKNOWN", body)
        self.assertIn("NOT EXERCISED", body.replace("_", " ").replace("-", " "))
        self.assertIn("AUS200", body)
        self.assertIn("JPN225", body)
        self.assertIn("73%", body)
        self.assertIn("SGT", body)
        self.assertIn("№", body)
        self.assertIn("§", body)
        self.assertNotIn("fixture-telegram", body)
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(self.v.status()["message_id"], 72)

    def test_delivery_restart_and_lost_ack_do_not_replay_and_source_failure_is_unknown(self):
        with patch.object(self.v, "_fetch", side_effect=self.fetch):
            self.v.tick(clock=self.clock)
        self.clock = self.due
        with patch.object(self.v, "_fetch", side_effect=self.fetch), \
             patch.object(reporting, "send_document", return_value=73) as post:
            self.v.tick(clock=self.clock)
            restarted = mv.MarketValidation(self.c)
            with patch.object(restarted, "_fetch", side_effect=AssertionError("delivered schedule must stop polling")):
                restarted.tick(clock=self.clock + dt.timedelta(minutes=5))
        self.assertEqual(post.call_count, 1)
        self.assertEqual(restarted.status()["message_id"], 73)
        self.assertEqual(restarted.status()["state"], "delivered")

        uncertain_c = self.coordinator(self.root / "uncertain")
        uncertain = mv.MarketValidation(uncertain_c)
        self.clock = self.start
        with patch.object(uncertain, "_fetch", side_effect=self.fetch):
            uncertain.tick(clock=self.start)
        self.clock = self.due
        with patch.object(uncertain, "_fetch", side_effect=OSError("source unavailable")), \
             patch.object(reporting, "send_document", side_effect=reporting.DeliveryError("lost_ack", uncertain=True)) as post:
            uncertain.tick(clock=self.due)
            after_restart = mv.MarketValidation(uncertain_c)
            after_restart.tick(clock=self.due + dt.timedelta(minutes=10))
        self.assertEqual(post.call_count, 1)
        self.assertEqual(after_restart.status()["state"], "uncertain")
        self.assertGreater(after_restart.status()["source_failure_count"], 0)
        body = post.call_args.args[3]
        self.assertIn("Monitoring: UNKNOWN", body,
                      "a healthy baseline cannot certify a later unavailable monitor source")
        self.assertNotIn("fixture-telegram", json.dumps(after_restart.status()))
        self.assertEqual(uncertain_c.store.used(), 0)


if __name__ == "__main__":
    unittest.main()
