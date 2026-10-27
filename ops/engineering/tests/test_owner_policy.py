"""Approved policy activates durably without preempting current model work."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import threading
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("policy_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class OwnerPolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = w.Store(self.root)

    def tearDown(self):
        self.temp.cleanup()

    def schedule(self, after=8):
        metadata = json.dumps({"due_at": "2026-10-26T23:20:00+00:00", "after_job": after})
        with self.store.connect() as c:
            return c.execute("INSERT INTO jobs(task,state,created,updated,result) VALUES(?,'scheduled',?,?,?)",
                             ("Owner-approved policy; $40 ceiling, not a spend target", w.now(), w.now(), metadata)).lastrowid

    def test_due_time_queues_once_and_survives_restart(self):
        jid = self.schedule()
        self.assertEqual(self.store.activate_scheduled("2026-10-26T23:19:59+00:00"), [])
        self.assertIsNone(self.store.reserve(18))
        self.assertEqual(self.store.activate_scheduled("2026-10-26T23:20:00+00:00"), [jid])
        self.assertEqual(w.Store(self.root).activate_scheduled("2026-10-26T23:21:00+00:00"), [])
        with self.store.connect() as c:
            events = c.execute("SELECT detail FROM events WHERE kind='owner_policy_active'").fetchall()
            self.assertEqual(len(events), 1)
            self.assertIn("$40 ceiling", events[0][0])
            self.assertEqual(c.execute("SELECT state FROM jobs WHERE id=?", (jid,)).fetchone()[0], "queued")
        self.assertEqual(self.store.used(), 0)

    def test_current_completion_activates_earlier_but_pending_verification_does_not(self):
        current = self.store.enqueue("current release")
        self.store.update(current, "verification_pending")
        policy = self.schedule(after=current)
        self.assertEqual(self.store.activate_scheduled("2026-10-26T23:15:00+00:00"), [])
        self.store.update(current, "completed")
        self.assertEqual(self.store.activate_scheduled("2026-10-26T23:16:00+00:00"), [policy])
        self.assertEqual(self.store.used(), 0)

    def test_awaiting_policy_suppresses_auto_without_activating_in_model_heartbeat(self):
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture",
            "RAILWAY_TOKEN": "fixture", "ENGINEERING_DAILY_SESSION_LIMIT": "18",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap", "PORT": str(port)}))
        accepted = self.store.enqueue(w.ACCEPTANCE_TASK)
        self.store.update(accepted, "completed")
        self.schedule()
        with patch.object(c.store, "activate_scheduled") as activate:
            c.pulse()
            activate.assert_not_called()
        timer = threading.Timer(0.2, c.shutdown.set)
        timer.start()
        try:
            with patch.object(c.reporter, "run"), patch.object(c.store, "activate_scheduled", return_value=[]), \
                 patch.object(c, "enqueue_automatic_assessment") as automatic, patch.object(c, "prepare") as prepare, \
                 patch.object(c.store, "reserve") as reserve:
                c.serve()
                automatic.assert_not_called()
                prepare.assert_not_called()
                reserve.assert_not_called()
        finally:
            timer.cancel()
        self.assertEqual(c.phase, "idle")
        self.assertEqual(self.store.used(), 0)
