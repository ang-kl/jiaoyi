"""Only material goal evidence may schedule an automatic model assessment."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("automatic_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


def snapshots():
    health = {"status": "ok", "ticks": 10, "trading": [
        {"account_id": 1, "strategy": "s1", "liveness": {"state": "alive", "last_quote_at": "old"}}],
        "strategy3": []}
    operations = {"as_of": "old", "recent_events": [], "reports": [
        {"account_id": 1, "strategy": "s1", "monitor_state": "alive", "checked_at": "old",
         "report": {"assessment": {"state": "below_target", "history_complete": True,
                                    "assessed_at": "old", "history_fetched_at": "old",
                                    "window": {"position_ids": [10, 11], "count": 2, "wins": 1,
                                               "losses": 1, "net_r": 0.38}}}}]}
    return health, operations


class AutomaticEvidenceTests(unittest.TestCase):
    def test_fingerprint_ignores_heartbeats_clock_churn_and_float_roundtrip_noise(self):
        health, reports = snapshots()
        changed_health, changed_reports = copy.deepcopy((health, reports))
        changed_health["ticks"] += 50000
        changed_health["trading"][0]["liveness"]["last_quote_at"] = "new"
        changed_reports["as_of"] = "new"
        changed_reports["recent_events"] = [{"report_type": "heartbeat"}]
        changed_reports["reports"][0]["checked_at"] = "new"
        assessment = changed_reports["reports"][0]["report"]["assessment"]
        assessment["assessed_at"] = assessment["history_fetched_at"] = "new"
        assessment["window"]["net_r"] += 1e-15
        self.assertEqual(w.evidence_fingerprint(health, reports, "2026-10-27", "revision"),
                         w.evidence_fingerprint(changed_health, changed_reports, "2026-10-27", "revision"))
        for day, revision in (("2026-10-28", "revision"), ("2026-10-27", "new-revision")):
            self.assertNotEqual(w.evidence_fingerprint(health, reports, "2026-10-27", "revision"),
                                w.evidence_fingerprint(health, reports, day, revision))

    def test_persisted_fingerprint_survives_store_restart(self):
        with tempfile.TemporaryDirectory() as temp:
            store = w.Store(temp)
            self.assertIsNotNone(store.enqueue_changed_evidence(w.DEFAULT_TASK, "first-evidence"))
            self.assertIsNone(w.Store(temp).enqueue_changed_evidence(w.DEFAULT_TASK, "first-evidence"))
            self.assertEqual(len(store.jobs()), 1)
            self.assertEqual(store.used(), 0)

    def test_unknown_financial_evidence_is_not_zero_and_stalled_loop_is_material(self):
        health, reports = snapshots()
        baseline = w.evidence_fingerprint(health, reports, "2026-10-27", "revision")
        health["trading"][0]["liveness"]["state"] = "stalled"
        self.assertNotEqual(baseline, w.evidence_fingerprint(health, reports, "2026-10-27", "revision"))
        window = reports["reports"][0]["report"]["assessment"]["window"]
        window["net_r"] = None
        unknown = w.evidence_fingerprint(health, reports, "2026-10-27", "revision")
        window["net_r"] = 0
        self.assertNotEqual(unknown, w.evidence_fingerprint(health, reports, "2026-10-27", "revision"))

    def coordinator(self, root):
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": str(root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture",
            "RAILWAY_TOKEN": "fixture", "ENGINEERING_DAILY_SESSION_LIMIT": "18",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap", "PORT": str(port)}))
        accepted = c.store.enqueue(w.ACCEPTANCE_TASK)
        c.store.update(accepted, "completed", result="accepted")
        return c

    def test_three_polls_spend_only_for_initial_and_changed_cohort(self):
        with tempfile.TemporaryDirectory() as temp:
            c = self.coordinator(temp)
            first = snapshots()
            heartbeat = copy.deepcopy(first)
            heartbeat[0]["ticks"] = 100000
            heartbeat[1]["as_of"] = "next polling time"
            heartbeat[1]["reports"][0]["checked_at"] = "next heartbeat"
            heartbeat[1]["reports"][0]["report"]["assessment"]["window"]["net_r"] += 1e-15
            new_cohort = copy.deepcopy(heartbeat)
            new_cohort[1]["reports"][0]["report"]["assessment"]["window"]["position_ids"] = [11, 12]
            bodies = iter(json.dumps(data).encode() for pair in (first, heartbeat, new_cohort) for data in pair)
            class Response:
                def __enter__(self): return self
                def __exit__(self, *_): pass
                def read(self, _): return next(bodies)
            iterations = []
            def wait(_):
                iterations.append(True)
                c.next_poll = 0  # Advance the simulated polling deadline.
                if len(iterations) == 3:
                    c.shutdown.set()
            def model(job, _):
                c.store.update(job["id"], "completed", result="evaluated material evidence")
            with patch.object(c.reporter, "run"), patch.object(c, "prepare", return_value=Path(temp)) as prepare, \
                 patch.object(c, "run_job", side_effect=model) as run, patch.object(c.shutdown, "wait", side_effect=wait), \
                 patch.object(w.urllib.request, "urlopen", return_value=Response()) as fetch:
                with patch.object(c, "incident_loop"):
                    c.serve()
            self.assertEqual(fetch.call_count, 6)
            self.assertEqual(run.call_count, 2)
            self.assertEqual(prepare.call_count, 2)
            self.assertEqual(c.store.used(), 2)
            self.assertEqual(c.config.limit, 18)
            self.assertEqual(len([job for job in c.store.jobs() if job["task"] == w.DEFAULT_TASK]), 2)

    def test_failed_snapshot_waits_until_next_poll_without_queuing_or_spending(self):
        with tempfile.TemporaryDirectory() as temp:
            c = self.coordinator(temp)
            iterations = []
            def wait(_):
                iterations.append(True)
                if len(iterations) == 2:
                    c.shutdown.set()
            with patch.object(c.reporter, "run"), patch.object(c.shutdown, "wait", side_effect=wait), \
                 patch.object(w.urllib.request, "urlopen", side_effect=OSError("unavailable")) as fetch, \
                 patch.object(c, "prepare") as prepare, patch.object(c.store, "reserve") as reserve:
                with patch.object(c, "incident_loop"):
                    c.serve()
            self.assertEqual(fetch.call_count, 1)
            prepare.assert_not_called()
            reserve.assert_not_called()
            self.assertEqual(c.store.used(), 0)
            self.assertEqual(len(c.store.jobs()), 1)
