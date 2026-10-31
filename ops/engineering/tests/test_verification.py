"""Bounded deployment verification can prove error paths without production faults."""
import contextlib
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import test_stop_loss_worker as fixtures

w = fixtures.w


class VerificationTests(unittest.TestCase):
    setUp = fixtures.StopLossWorkerTests.setUp
    tearDown = fixtures.StopLossWorkerTests.tearDown
    git = fixtures.StopLossWorkerTests.git

    def fixture(self, acceptance=False):
        repo, job = fixtures.StopLossWorkerTests.review_fixture(self)
        task = w.ACCEPTANCE_TASK if acceptance else "Verify the completed-clock retention fix after deployment."
        merged = self.git("rev-parse", "HEAD", cwd=repo)
        with self.c.store.connect() as db:
            db.execute("UPDATE jobs SET task=?,pr=42,head=?,merged=? WHERE id=?", (task, merged, merged, job["id"]))
        job.update(task=task, state="queued" if acceptance else "verification_queued", pr=42, head=merged, merged=merged)
        return repo, job

    def execute(self, repo, job, report, inspect, release_observation=None):
        class Response:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, _):
                return json.dumps({"status": "ok", "reports": [{"report": {"last_success_at": "2026-10-27T06:00:00Z"}}]}).encode()
        def model(args, **kwargs):
            inspect(args, kwargs)
            Path(args[args.index("--output-last-message") + 1]).write_text(json.dumps(report))
            kwargs["logfile"].write_text("mock model; actual local Git immutability checks\n")
            return "ok"
        with patch.object(self.c, "existing_pr", return_value=None), \
             patch.object(self.c.releases, "observe", side_effect=release_observation, return_value=True) as release, \
             patch.object(self.c, "login_model", return_value="ok"), \
             patch.object(w.urllib.request, "urlopen", return_value=Response()) as fetch, \
             patch.object(w, "bounded_process", side_effect=model) as run, \
             patch.object(self.c, "gh", side_effect=AssertionError("verification must not publish a PR")), \
             contextlib.redirect_stdout(io.StringIO()):
            self.c.run_job(job, repo)
        self.assertEqual(fetch.call_count, 2)
        self.assertEqual(run.call_count, 1)
        if job["state"] == "verification_queued":
            self.assertEqual(release.call_count, 2 if report["outcome"] == "verified" and report["acceptance_passed"] is False else 1)

    def report(self, outcome="verified", accepted=False):
        return {"outcome": outcome,
                "summary": "Current deployed positive behavior plus unchanged-source controlled failure-path assertions; natural production failure unobserved.",
                "tests": "Existing focused isolated-DB regression and current deployed snapshot; financial acceptance unproved.",
                "owner_decisions": "none", "acceptance_passed": accepted}

    def test_closed_work_window_allows_receipt_inspection_but_forbids_new_test_builds(self):
        repo, job = self.fixture()
        self.config.work_until = w.dt.datetime(2000, 1, 1, tzinfo=w.dt.timezone.utc)
        def inspect(args, options):
            prompt = args[-1]
            self.assertIn("OWNER WORK WINDOW CLOSED", prompt)
            self.assertIn("Do not run tests, compilers, cargo, build tools", prompt)
            self.assertIn("override any earlier permission", prompt)
            self.assertIn("Inspect cached", prompt)
            self.assertFalse(options["stop"](), "closed-window read-only inspection remains allowed")
        self.execute(repo, job, self.report(outcome="verification_pending"), inspect)
        self.assertEqual(self.c.store.jobs()[0]["state"], "verification_pending")
        self.assertEqual(self.c.store.used(), 1)

    def test_controlled_negative_and_live_positive_can_complete_single_agent_verification(self):
        repo, job = self.fixture()
        baseline = self.git("rev-parse", "HEAD", cwd=repo)
        def inspect(args, options):
            self.assertEqual(args[args.index("--sandbox") + 1], "read-only")
            self.assertEqual(args[args.index("multi_agent") - 1], "--disable")
            self.assertIn("agents.max_threads=1", args)
            self.assertEqual(options["timeout"], 300)
            self.assertFalse(options["stop"]())
            prior = self.config.work_until
            self.config.work_until = w.dt.datetime(2000, 1, 1, tzinfo=w.dt.timezone.utc)
            self.assertTrue(options["stop"](), "a pre-cutoff verifier must stop when the window closes")
            self.config.work_until = prior
            self.assertEqual(options["env"]["DATABASE_URL"], "postgres://engineering@127.0.0.1:15432/postgres")
            prompt = args[-1]
            self.assertIn("existing focused regression tests", prompt)
            self.assertIn("existing isolated DATABASE_URL", prompt)
            self.assertIn("matches the deployed source", prompt)
            self.assertIn("current deployed positive proof", prompt)
            self.assertIn("Do not withhold engineering completion solely", prompt)
            self.assertIn("Never induce a production failure", prompt)
            self.assertIn("acceptance_passed must remain false", prompt)
        self.execute(repo, job, self.report(), inspect)
        self.assertEqual(self.c.store.jobs()[0]["state"], "completed")
        self.assertEqual(self.c.store.used(), 1)
        self.assertIsNone(self.c.store.reserve(self.config.limit))
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=repo), baseline)
        self.assertEqual(self.git("status", "--porcelain", cwd=self.root / "workspaces" / str(job["id"])), "")
        with self.c.store.connect() as db:
            saved = json.loads(db.execute("SELECT detail FROM events WHERE kind='completed' AND job=?", (job["id"],)).fetchone()[0])
        self.assertIs(saved["acceptance_passed"], False)

    def test_missing_required_positive_proof_stays_pending_without_automatic_reservation(self):
        repo, job = self.fixture()
        report = self.report("verification_pending")
        report["summary"] = "Current deployed affected-path positive proof unavailable."
        self.execute(repo, job, report, lambda *_: None)
        self.assertEqual(self.c.store.jobs()[0]["state"], "verification_pending")
        self.assertIsNone(self.c.store.reserve(self.config.limit))
        self.assertEqual(self.c.store.used(), 1)

    def test_changed_active_revision_after_model_result_cannot_complete(self):
        repo, job = self.fixture()
        observed = []
        def changed(*args, **kwargs):
            observed.append(args)
            if len(observed) == 1:
                return True
            self.c.releases.handoff(job, "release_superseded", current_sha="e" * 40)
            return False
        self.execute(repo, job, self.report(), lambda *_: None, changed)
        self.assertEqual(self.c.store.jobs()[0]["state"], "verification_pending")
        self.assertIn("release_superseded", self.c.store.jobs()[0]["result"])

    def test_verifier_cannot_claim_financial_acceptance(self):
        repo, job = self.fixture()
        with self.assertRaisesRegex(RuntimeError, "verification cannot declare financial acceptance"):
            self.execute(repo, job, self.report(accepted=True), lambda *_: None)
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM events WHERE kind='completed' AND job=?", (job["id"],)).fetchone()[0], 0)

    def test_initial_acceptance_keeps_required_delegation_and_original_timeout(self):
        repo, job = self.fixture(acceptance=True)
        def inspect(args, options):
            self.assertEqual(args[args.index("multi_agent") - 1], "--enable")
            self.assertIn("agents.max_threads=4", args)
            self.assertEqual(options["timeout"], 1800)
            self.assertIn("Delegate one read-only charter check", args[-1])
        self.execute(repo, job, self.report("no_change", accepted=True), inspect)
        self.assertEqual(self.c.store.acceptance(), "completed")


if __name__ == "__main__":
    unittest.main()
