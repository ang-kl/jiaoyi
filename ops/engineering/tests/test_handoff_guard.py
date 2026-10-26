"""Retained proposals block fresh inference, without stopping release polling."""
import importlib.util
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("handoff_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class HandoffGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        self.c = w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture",
            "RAILWAY_TOKEN": "fixture", "ENGINEERING_DAILY_SESSION_LIMIT": "18",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap", "PORT": str(port)}))
        accepted = self.c.store.enqueue(w.ACCEPTANCE_TASK)
        self.c.store.update(accepted, "completed", result="accepted")

    def tearDown(self):
        self.temp.cleanup()

    def retained_proposal(self):
        def git(*args, cwd=None):
            return subprocess.check_output(["git", *args], cwd=cwd, text=True, stderr=subprocess.PIPE).strip()
        seed = self.root / "seed"
        git("init", "-b", "main", str(seed))
        (seed / "crates").mkdir()
        (seed / "crates" / "example.rs").write_text("// base\n")
        (seed / "journal").mkdir()
        (seed / "journal" / "base.md").write_text("prior evidence\n")
        git("add", ".", cwd=seed)
        git("-c", "user.name=test", "-c", "user.email=test@example.invalid", "commit", "-m", "base", cwd=seed)
        remote = self.root / "remote.git"
        git("clone", "--bare", str(seed), str(remote))
        repo = self.root / "repository"
        git("clone", str(remote), str(repo))
        jid = self.c.store.enqueue("legacy report whose publication failed")
        workspace = self.root / "workspaces" / str(jid)
        workspace.parent.mkdir()
        git("worktree", "add", "-b", "engineering/job-" + str(jid), str(workspace), "origin/main", cwd=repo)
        (workspace / "crates" / "example.rs").write_text("// retained tested fix\n")
        (workspace / "journal" / "new.md").write_text("actual prior regression evidence\n")
        git("add", ".", cwd=workspace)
        git("-c", "user.name=test", "-c", "user.email=test@example.invalid", "commit", "-m", "retained proposal", cwd=workspace)
        self.c.store.update(jid, "failed", result="RuntimeError")
        return jid, workspace, git("rev-parse", "HEAD", cwd=workspace)

    def run_briefly(self):
        timer = threading.Timer(0.2, self.c.shutdown.set)
        timer.start()
        try:
            self.c.serve()
        finally:
            timer.cancel()

    def test_eighteen_is_allowance_not_a_reason_to_repeat_retained_failed_work(self):
        jid, workspace, head = self.retained_proposal()
        self.c.store.enqueue(w.DEFAULT_TASK)
        with patch.object(self.c.reporter, "run"), patch.object(self.c, "prepare") as prepare, \
             patch.object(self.c, "run_job") as model, patch.object(self.c.store, "reserve", wraps=self.c.store.reserve) as reserve, \
             patch.object(self.c, "_dry_run_push", side_effect=AssertionError("extra remote probe")):
            self.run_briefly()
            prepare.assert_not_called()
            model.assert_not_called()
            reserve.assert_not_called()
        self.assertEqual(self.c.phase, "handoff_blocked")
        self.assertEqual(self.c.store.used(), 0)
        self.assertTrue(self.c.config.enabled)
        self.assertEqual(self.c.config.limit, 18)
        self.assertEqual((workspace / "crates" / "example.rs").read_text(), "// retained tested fix\n")
        self.assertEqual(self.c.git("rev-parse", "HEAD", cwd=workspace), head)
        self.assertEqual(next(row for row in self.c.store.jobs() if row["id"] == jid)["state"], "failed")

    def test_handoff_scan_is_cached_until_failed_job_state_changes(self):
        jid, _, _ = self.retained_proposal()
        with patch.object(self.c, "diagnose_job", wraps=self.c.diagnose_job) as diagnose:
            self.assertEqual(self.c.handoff_blockers(), [jid])
            self.assertEqual(self.c.handoff_blockers(), [jid])
            self.assertEqual(diagnose.call_count, 1)
            self.assertFalse(diagnose.call_args.kwargs["probe_write"])
            self.c.store.update(jid, "completed", result="owner resolved retained handoff")
            self.assertEqual(self.c.handoff_blockers(), [])

    def test_ci_polling_continues_while_failed_proposals_exist(self):
        self.retained_proposal()
        published = self.c.store.enqueue("already published task")
        self.c.store.update(published, "ci_wait", pr=42)
        def release(job):
            self.assertEqual(job["id"], published)
            self.c.shutdown.set()
        with patch.object(self.c.reporter, "run"), patch.object(self.c, "release", side_effect=release) as poll, \
             patch.object(self.c, "handoff_blockers", side_effect=AssertionError("release polling blocked")), \
             patch.object(self.c, "prepare") as prepare, patch.object(self.c, "run_job") as model:
            self.c.serve()
            poll.assert_called_once()
            prepare.assert_not_called()
            model.assert_not_called()
        self.assertEqual(self.c.store.used(), 0)

    def test_idle_wait_does_not_prepare_database_or_reserve_a_model(self):
        self.c.next_poll = time.monotonic() + 3600
        with patch.object(self.c.reporter, "run"), patch.object(self.c, "prepare") as prepare, \
             patch.object(self.c.store, "reserve", wraps=self.c.store.reserve) as reserve:
            self.run_briefly()
            prepare.assert_not_called()
            reserve.assert_not_called()
        self.assertEqual(self.c.phase, "idle")
        self.assertEqual(self.c.store.used(), 0)
