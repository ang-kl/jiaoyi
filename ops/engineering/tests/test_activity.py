"""Auditable waiting without inference, fabricated success or a masked freeze."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("activity_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class ActivityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.c = w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": self.temp.name, "ENGINEERING_ENABLED": "1",
            "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "private-github-credential",
            "RAILWAY_TOKEN": "private-railway-credential", "ENGINEERING_DAILY_SESSION_LIMIT": "2",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap"}))
        self.c.phase = "idle"
        self.c.next_poll = time.monotonic() + 600

    def tearDown(self):
        self.temp.cleanup()

    def test_waiting_queued_and_unresolved_verification_are_distinct(self):
        activity = self.c.status()["activity"]
        self.assertEqual(activity["reason"], "no_queued_work_waiting_for_public_evidence_poll")
        self.assertIsNone(activity["last_public_assessment"])
        self.assertGreater(activity["next_poll_in_seconds"], 590)
        jid = self.c.store.enqueue("A private task, not a public status message")
        self.assertEqual(self.c.status()["activity"]["reason"], "queued_work_awaiting_coordinator")
        self.c.store.update(jid, "verification_pending", result=json.dumps({
            "outcome": "verification_pending", "summary": "private evidence", "tests": "private"}))
        activity = self.c.status()["activity"]
        self.assertEqual(activity["reason"], "no_queued_work_verification_pending")
        self.assertEqual(activity["pending_verification"], {"count": 1, "ids": [jid]})
        self.assertEqual(activity["last_job"]["outcome"], "verification_pending")
        # Reporting unresolved work must not silently requeue or consume it.
        with contextlib.redirect_stdout(io.StringIO()):
            self.c.activity_tick()
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(self.c.store.jobs()[0]["state"], "verification_pending")

    def test_session_allowance_is_explicit_and_never_interpreted_as_dollars(self):
        for _ in range(2):
            jid = self.c.store.enqueue("bounded work")
            self.c.store.reserve(2)
            self.c.store.update(jid, "completed", result='{"outcome":"no_change"}')
        jid = self.c.store.enqueue("work waiting for tomorrow")
        activity = self.c.status()["activity"]
        self.assertEqual(activity["reason"], "session_allowance_exhausted")
        self.assertEqual(activity["session_budget_remaining"], 0)
        self.assertEqual(activity["queued_count"], 1)
        self.assertEqual(self.c.store.jobs()[0]["id"], jid)

    def test_heartbeats_do_not_hide_stale_main_loop_or_persist_clock_churn(self):
        self.c.pulse_at = time.monotonic() - 150
        frozen = self.c.pulse_at
        output = io.StringIO()
        with contextlib.redirect_stdout(output), patch.object(self.c, "pulse", side_effect=AssertionError("observer cannot pulse")):
            self.c.activity_tick()
            self.c.next_poll -= 10
            self.c.activity_tick()
        beats = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(len(beats), 2)
        self.assertTrue(all(b["coordinator_heartbeat_stale"] for b in beats))
        self.assertTrue(all(b["reason"] == "coordinator_heartbeat_stale" for b in beats))
        self.assertEqual(self.c.pulse_at, frozen)
        self.assertEqual(self.c.store.used(), 0)
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM events WHERE kind='activity_transition'").fetchone()[0], 1)
        jid = self.c.store.enqueue("new work")
        with contextlib.redirect_stdout(io.StringIO()):
            self.c.activity_tick()
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM events WHERE kind='activity_transition'").fetchone()[0], 2)

    def test_observer_continues_every_five_seconds_without_main_loop_progress(self):
        frozen = self.c.pulse_at = time.monotonic() - 150
        waits = []
        def wait(seconds):
            waits.append(seconds)
            if len(waits) == 2:
                self.c.activity_shutdown.set()
        output = io.StringIO()
        with patch.object(self.c.activity_shutdown, "wait", side_effect=wait), \
             patch.object(self.c, "pulse", side_effect=AssertionError("observer cannot pulse")), \
             patch.object(self.c.store, "reserve", side_effect=AssertionError("observer cannot reserve")), \
             contextlib.redirect_stdout(output):
            self.c.activity_loop()
        self.assertEqual(waits, [5, 5])
        self.assertEqual(len(output.getvalue().splitlines()), 2)
        self.assertEqual(self.c.pulse_at, frozen)

    def test_quiet_child_is_separate_from_main_liveness_and_keeps_timeout(self):
        jid = self.c.store.enqueue("run meaningful work")
        self.c.store.reserve(2)
        self.c.phase = "running"
        raw = self.c.store.private / "session.raw"
        raw.write_text("Private model output must never be exposed")
        self.c.active_session = {"id": jid, "started": time.monotonic() - 160,
                                 "at": w.now(), "output": raw}
        with patch.object(w.time, "time", return_value=raw.stat().st_mtime + 150):
            activity = self.c.status()["activity"]
        self.assertEqual(activity["reason"], "active_job")
        self.assertFalse(activity["coordinator_heartbeat_stale"])
        self.assertEqual(activity["active_job"]["output_state"], "quiet")
        self.assertGreater(activity["active_job"]["remaining_seconds"], 1600)
        self.assertEqual(activity["active_job"]["timeout_seconds"], 1800)
        self.assertNotIn(str(raw), json.dumps(activity))
        self.assertNotIn("Private model output", json.dumps(activity))

    def test_public_status_and_transition_receipts_exclude_untrusted_prose(self):
        private = "private-github-credential private-railway-credential https://private.example/token"
        jid = self.c.store.enqueue(private)
        self.c.store.update(jid, "completed", result=json.dumps({"outcome": private, "summary": private, "tests": private}))
        self.c.store.event("model_report", jid, json.dumps({"outcome": "no_change", "summary": private}))
        self.c.store.event("public_evidence_assessed", jid, json.dumps({
            "decision": "unchanged_public_evidence", "source_at": private, "queued_job_id": private, "summary": private}))
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.c.activity_tick()
        public = json.dumps(self.c.status()) + output.getvalue()
        with self.c.store.connect() as db:
            public += db.execute("SELECT detail FROM events WHERE kind='activity_transition'").fetchone()[0]
        for secret in private.split():
            self.assertNotIn(secret, public)
        activity = self.c.status()["activity"]
        self.assertIsNone(activity["last_job"]["outcome"])
        self.assertEqual(activity["last_model_assessment"]["outcome"], "no_change")
        self.assertIsNone(activity["last_public_assessment"]["source_at"])
        self.assertNotIn("desired_state", public)

    def test_real_public_poll_receipt_identifies_scope_without_new_reservations(self):
        class Response:
            def __init__(self, data): self.data = data
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, _): return json.dumps(self.data).encode()
        responses = [Response({"status": "ok"}), Response({"as_of": "2026-10-27T01:00:00Z", "reports": []})]
        with patch.object(w.urllib.request, "urlopen", side_effect=responses):
            queued = self.c.enqueue_automatic_assessment()
        receipt = self.c.status()["activity"]["last_public_assessment"]
        self.assertEqual(receipt["queued_job_id"], queued)
        self.assertEqual(receipt["source_at"], "2026-10-27T01:00:00+00:00")
        self.assertEqual(receipt["decision"], "changed_public_evidence")
        self.assertEqual(self.c.store.used(), 0)


if __name__ == "__main__":
    unittest.main()
