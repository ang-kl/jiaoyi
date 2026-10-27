"""Worker integration: dated permits, urgent queueing and read-only reviews."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import socket
import tempfile
import time
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("sl_integration_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class StopLossWorkerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.config = w.Config({"ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt",
            "GH_TOKEN": "fixture-gh", "RAILWAY_TOKEN": "fixture-rail",
            "ENGINEERING_DAILY_SESSION_LIMIT": "18", "ENGINEERING_EXTRA_SESSION_LIMIT": "28",
            "ENGINEERING_EXTRA_SESSION_DATE": "2026-10-27", "ENGINEERING_STOP_LOSS_REVIEWS": "1",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap"})
        self.c = w.Coordinator(self.config)

    def tearDown(self):
        self.temp.cleanup()

    def test_dated_allowance_reverts_without_restart_and_next_day_quota_is_fresh(self):
        with patch.object(w, "sgt_day", return_value="2026-10-27"):
            self.assertEqual(self.config.limit, 46)
            self.assertTrue(self.config.stop_loss_focus())
            self.assertEqual(self.c.status()["allowance"]["additional"], 28)
            for _ in range(47):
                self.c.store.enqueue("bounded authorized work")
            for _ in range(46):
                job = self.c.store.reserve(self.config.limit)
                self.assertIsNotNone(job)
                self.c.store.update(job["id"], "completed")
            self.assertIsNone(self.c.store.reserve(self.config.limit))
            self.assertEqual(self.c.store.used(), 46)
        # The same coordinator/configuration objects cross the SGT date boundary.
        with patch.object(w, "sgt_day", return_value="2026-10-28"):
            self.assertEqual(self.config.limit, 18)
            self.assertFalse(self.config.stop_loss_focus())
            status = self.c.status()
            self.assertEqual(status["daily_session_limit"], 18)
            self.assertEqual(status["allowance"]["additional"], 0)
            self.assertEqual(status["sessions_today_sgt"], 0)
            self.assertEqual(status["activity"]["session_budget_remaining"], 18)
            self.assertIsNotNone(self.c.store.reserve(self.config.limit))

    def test_reservation_order_is_acceptance_then_review_then_verification_then_default(self):
        ordinary = self.c.store.enqueue("older ordinary engineering work")
        verification = self.c.store.enqueue("verify an existing release")
        self.c.store.update(verification, "verification_queued")
        review = self.c.store.enqueue(w.stop_loss_reviews.PREFIX + " ORIGINAL SETUP REVIEW")
        acceptance = self.c.store.enqueue(w.ACCEPTANCE_TASK)
        taken = []
        for _ in range(4):
            job = self.c.store.reserve(18)
            taken.append(job["id"])
            self.c.store.update(job["id"], "completed")
        self.assertEqual(taken, [acceptance, review, verification, ordinary])
        self.assertEqual(self.c.store.used(), 4)

    def test_independent_observer_sees_exits_while_main_frozen_without_inference(self):
        frozen = self.c.pulse_at = time.monotonic() - 150
        waits = []
        def wait(seconds):
            waits.append(seconds)
            if len(waits) == 2:
                self.c.activity_shutdown.set()
        def poll(coordinator, **_):
            if not coordinator.store.jobs():
                jid = coordinator.store.enqueue(w.stop_loss_reviews.PREFIX + " observed exit")
                return {"reason": "reviews_queued", "queued_job_ids": [jid]}
            return {"reason": "no_new_review_evidence", "queued_job_ids": []}
        with patch.object(self.c.activity_shutdown, "wait", side_effect=wait), \
             patch.object(w.stop_loss_reviews, "poll", side_effect=poll) as fetch, \
             patch.object(self.c, "pulse", side_effect=AssertionError("observer cannot refresh main heartbeat")), \
             patch.object(self.c.store, "reserve", side_effect=AssertionError("observer cannot reserve inference")), \
             patch.object(w, "bounded_process", side_effect=AssertionError("observer cannot call a model")):
            self.c.stop_loss_loop()
        self.assertEqual(waits, [10, 10])
        self.assertEqual(fetch.call_count, 2)
        self.assertEqual(self.c.pulse_at, frozen)
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(len(self.c.store.jobs()), 1)
        self.assertTrue(self.c.status()["activity"]["coordinator_heartbeat_stale"])

    def test_urgent_review_skips_retained_handoff_diagnostics_and_release_wait(self):
        accepted = self.c.store.enqueue(w.ACCEPTANCE_TASK)
        self.c.store.update(accepted, "completed")
        waiting = self.c.store.enqueue("existing release")
        self.c.store.update(waiting, "ci_wait")
        urgent = self.c.store.enqueue(w.stop_loss_reviews.PREFIX + " ORIGINAL SETUP REVIEW")
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            self.config.port = listener.getsockname()[1]
        def done(*_):
            self.c.shutdown.set()
        def reviewed(job, _):
            self.assertEqual(job["id"], urgent)
            self.c.store.update(urgent, "completed")
        with patch.object(self.c, "activity_loop"), patch.object(self.c, "stop_loss_loop"), \
             patch.object(self.c.reporter, "run"), \
             patch.object(self.c.shutdown, "wait", side_effect=done), \
             patch.object(self.c, "prepare", return_value=self.root), \
             patch.object(self.c, "handoff_blockers", side_effect=AssertionError("urgent review cannot wait for handoff diagnostics")) as scan, \
             patch.object(self.c, "release", side_effect=AssertionError("urgent review cannot wait for CI polling")) as release, \
             patch.object(self.c, "run_job", side_effect=reviewed) as model:
            self.c.serve()
        scan.assert_not_called()
        release.assert_not_called()
        self.assertEqual(model.call_count, 1)
        self.assertEqual(self.c.store.used(), 1)

    def git(self, *args, cwd=None):
        return subprocess.check_output(["git", *args], cwd=cwd, text=True, stderr=subprocess.PIPE).strip()

    def review_fixture(self):
        repo = self.root / "repository"
        self.git("init", "-b", "main", str(repo))
        (repo / "docs").mkdir()
        (repo / "docs" / "original.md").write_text("Original registered setup evidence.\n")
        self.git("add", ".", cwd=repo)
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-m", "baseline", cwd=repo)
        self.git("update-ref", "refs/remotes/origin/main", "HEAD", cwd=repo)
        (self.root / "workspaces").mkdir()
        (self.root / "agent-state").mkdir()
        self.c.uid, self.c.gid = os.getuid(), os.getgid()
        jid = self.c.store.enqueue(w.stop_loss_reviews.PREFIX + " ORIGINAL SETUP REVIEW: original facts unavailable; preserve unknowns")
        job = self.c.store.reserve(18)
        self.assertEqual(job["id"], jid)
        return repo, job

    def test_readonly_review_persists_no_change_and_delivers_without_pr_or_verification(self):
        repo, job = self.review_fixture()
        baseline = self.git("rev-parse", "HEAD", cwd=repo)
        def model(args, **kwargs):
            self.assertEqual(args[0], "codex")
            self.assertEqual(args[args.index("--sandbox") + 1], "read-only")
            self.assertEqual(kwargs["timeout"], 300)
            self.assertIn("READ-ONLY STOP-LOSS REVIEW", args[-1])
            self.assertIn("Propose any proven critical repair separately", args[-1])
            Path(args[args.index("--output-last-message") + 1]).write_text(json.dumps({
                "outcome": "no_change", "summary": "Original setup evidence unknown; no proven software defect.",
                "tests": "Read-only saved evidence review; no unrelated tests.", "owner_decisions": "none", "acceptance_passed": False}))
            kwargs["logfile"].write_text("mock model; real local Git guard\n")
            return "ok"
        def delivered(coordinator, passed_job, report):
            saved = coordinator.store.jobs()[0]
            self.assertEqual(saved["state"], "completed")
            self.assertEqual(passed_job["id"], job["id"])
            self.assertEqual(report["outcome"], "no_change")
            return {"state": "confirmed", "message_id": 1}
        with patch.object(self.c, "existing_pr", return_value=None), \
             patch.object(self.c, "login_model", return_value="ok"), \
             patch.object(w, "bounded_process", side_effect=model) as run, \
             patch.object(self.c, "gh", side_effect=AssertionError("review must not create a PR")), \
             patch.object(w.stop_loss_reviews, "deliver_review", side_effect=delivered) as delivery, \
             contextlib.redirect_stdout(io.StringIO()):
            self.c.run_job(job, repo)
        saved = self.c.store.jobs()[0]
        self.assertEqual((saved["state"], saved["pr"], saved["merged"]), ("completed", None, None))
        self.assertEqual(run.call_count, 1)
        self.assertEqual(delivery.call_count, 1)
        self.assertEqual(self.c.store.used(), 1)
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=repo), baseline)
        self.assertEqual(self.git("status", "--porcelain", cwd=self.root / "workspaces" / str(job["id"])), "")
        with self.c.store.connect() as db:
            report = json.loads(db.execute("SELECT detail FROM events WHERE kind='model_report' AND job=?", (job["id"],)).fetchone()[0])
        self.assertEqual(report["outcome"], "no_change")
        self.assertIs(report["acceptance_passed"], False)

    def test_model_timeout_leaves_failed_review_and_never_delivers_it(self):
        repo, job = self.review_fixture()
        with patch.object(self.c, "existing_pr", return_value=None), \
             patch.object(self.c, "login_model", return_value="ok"), \
             patch.object(w, "bounded_process", return_value="timeout"), \
             patch.object(w.stop_loss_reviews, "deliver_review") as delivery:
            self.c.run_job(job, repo)
        self.assertEqual(self.c.store.jobs()[0]["state"], "failed")
        self.assertEqual(self.c.store.used(), 1)
        delivery.assert_not_called()


if __name__ == "__main__":
    unittest.main()
