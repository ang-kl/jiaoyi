"""Real immutable Git evidence; mocked GitHub/Railway, no model or live writes."""
import contextlib
import datetime as dt
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("component_worker", Path(__file__).resolve().parents[1] / "worker.py")
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)
r = w.releases
ROOT = Path(__file__).resolve().parents[3]


class Fixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.config = w.Config({"ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture",
            "RAILWAY_TOKEN": "fixture", "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap"})
        self.c = w.Coordinator(self.config)
        self.repo = self.root / "repository"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "test")
        self.git("config", "user.email", "test@test")
        for directory in ("docs", "journal", "crates"):
            (self.repo / directory).mkdir()
        (self.repo / "railway.toml").write_bytes((ROOT / "railway.toml").read_bytes())
        (self.repo / "docs/note.md").write_text("old evidence\n")
        (self.repo / "journal/day.md").write_text("preserved evidence\n")
        (self.repo / "crates/logic.rs").write_text("old logic\n")
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD")
        self.rules = r.patterns(r.tomllib.loads((ROOT / "railway.toml").read_text())["build"]["watchPatterns"])
        self.merged_at = (dt.datetime.now(dt.timezone.utc) - dt.timedelta(seconds=60)).isoformat()
        self.remote_merged = True
        self.active, self.latest = [], None
        self.writes, self.queries = [], []
        self.patches = [patch.object(self.c, "gh", side_effect=self.gh),
            patch.object(self.c.releases, "api", side_effect=self.api),
            patch.object(w.urllib.request, "urlopen", side_effect=AssertionError("unmocked network")),
            patch.object(w, "bounded_process", side_effect=AssertionError("release must not call a model/build"))]
        for p in self.patches:
            p.start()
        original_git = self.c.git
        def local_only(*args, **kwargs):
            if args[0] == "fetch":
                raise RuntimeError("fixture objects missing; no network fetch")
            return original_git(*args, **kwargs)
        p = patch.object(self.c, "git", side_effect=local_only)
        p.start()
        self.patches.append(p)

    def tearDown(self):
        for p in reversed(self.patches):
            p.stop()
        self.temp.cleanup()

    def git(self, *args, input_text=None):
        return subprocess.run(["git", *args], cwd=self.repo, input=input_text, text=True,
            capture_output=True, check=True).stdout.strip()

    def fixture(self, runtime=False):
        path = "crates/logic.rs" if runtime else "docs/note.md"
        (self.repo / path).write_text("new source-backed evidence\n\n")
        with (self.repo / "journal/day.md").open("a") as f:
            f.write("bounded change recorded\n")
        self.git("add", ".")
        self.git("commit", "-qm", "candidate")
        self.head = self.git("rev-parse", "HEAD")
        self.merge = self.git("commit-tree", self.head + "^{tree}", "-p", self.base, input_text="squash\n")
        self.jid = self.c.store.enqueue("deliver the declared artifact" if not runtime else "repair a proven runtime defect")
        self.c.store.update(self.jid, "deployment_wait", pr=42, head=self.head, merged=self.merge)
        return self.job()

    def job(self):
        return next(j for j in self.c.store.jobs(("ci_wait", "deployment_wait", "verification_queued", "verification_pending", "completed", "owner_blocked", "running")) if j["id"] == self.jid)

    def gh(self, *args, **options):
        if options.get("write"):
            self.writes.append(args)
            self.remote_merged = True
            return json.dumps({"merged": True, "sha": self.merge})
        path = args[-1]
        if path.endswith("/pulls/42"):
            return json.dumps({"state": "closed" if self.remote_merged else "open", "merged": self.remote_merged,
                "merge_commit_sha": self.merge, "merged_at": self.merged_at, "base": {"ref": "main"},
                "head": {"sha": self.head, "ref": "engineering/job-" + str(self.jid), "repo": {"full_name": w.REPO}}})
        if "check-runs" in path:
            return json.dumps({"check_runs": [{"name": name, "status": "completed", "conclusion": "success",
                "app": {"slug": "github-actions"}} for name in ("check", "docker", "engineering")]})
        if path.endswith("/git/ref/heads/main"):
            return json.dumps({"object": {"sha": self.base}})
        if "/compare/" in path:
            return json.dumps({"status": "ahead", "behind_by": 0, "base_commit": {"sha": self.base}, "merge_base_commit": {"sha": self.base}})
        raise AssertionError(args)

    def api(self, query, variables=None):
        self.queries.append(query)
        if "watchPatterns" in query:
            return {"serviceInstance": {"watchPatterns": self.rules}}
        if "activeDeployments" in query:
            return {"serviceInstance": {"activeDeployments": self.active, "latestDeployment": self.latest}}
        if "deployments(first:" in query:
            return {"deployments": {"edges": [], "pageInfo": {"hasNextPage": False, "endCursor": None}}}
        raise AssertionError(query)

    def deployment(self, sha=None, status="SUCCESS", seconds=-30, identity="deployment"):
        return {"id": identity, "status": status,
            "createdAt": (dt.datetime.now(dt.timezone.utc) + dt.timedelta(seconds=seconds)).isoformat(),
            "meta": {"commitHash": sha or self.merge}}

    def declare(self):
        plan, files = self.c.releases.classify(self.job(), self.base, self.merge)
        receipt = self.c.store.private / "approved-artifacts.json"
        receipt.write_text(json.dumps({"head": self.head,
            "source": "owner_approved_artifact", "criteria": "exact_artifact_content",
            "reference": "fixture approved content receipt", "files": files}))
        receipt.chmod(0o600)
        self.c.releases.accept_repository(self.jid, receipt)
        return files

    def health(self):
        return patch.object(w.urllib.request, "urlopen", side_effect=lambda *_args, **_kwargs: io.BytesIO(b'{"status":"ok"}'))


class ClassificationTests(Fixture):
    def test_docs_merge_completes_from_exact_declared_bytes_without_model_or_deployment(self):
        self.fixture()
        files = self.declare()
        self.assertEqual(files["docs/note.md"], hashlib.sha256(b"new source-backed evidence\n\n").hexdigest())
        self.remote_merged = False
        self.c.store.update(self.jid, "ci_wait", merged=None)
        self.c.release(self.job())
        self.assertEqual(len(self.writes), 1)
        self.c.release(self.job())
        self.assertEqual(self.job()["state"], "completed")
        self.assertFalse(any("activeDeployments" in q or "deployments(first:" in q for q in self.queries))
        self.assertEqual(self.c.store.used(), 0)
        self.assertFalse(json.loads(self.job()["result"])["acceptance_passed"])

    def test_model_prose_or_wrong_content_cannot_self_certify_docs(self):
        self.fixture()
        self.c.store.event("model_report", self.jid, json.dumps({"outcome": "verified", "summary": "all sources checked"}))
        self.c.release(self.job())
        self.assertEqual(self.job()["state"], "verification_pending")
        self.assertIn("repository_acceptance_missing", self.job()["result"])
        files = self.declare()
        files["docs/note.md"] = hashlib.sha256(b"new source-backed evidence").hexdigest()
        self.c.store.event("repository_acceptance", self.jid, json.dumps({"head": self.head,
            "source": "independent_review", "criteria": "exact_artifact_content", "reference": "review", "files": files}))
        self.c.releases.observe(self.job())
        self.assertEqual(self.job()["state"], "verification_pending")
        self.assertEqual(self.c.store.used(), 0)

    def test_complete_diff_includes_watched_delete_and_rename_into_docs(self):
        self.fixture()
        (self.repo / "crates/logic.rs").rename(self.repo / "docs/renamed.md")
        self.git("add", "-A")
        self.git("commit", "-qm", "rename watched input")
        renamed = self.git("rev-parse", "HEAD")
        plan, files = self.c.releases.classify(self.job(), self.base, renamed)
        self.assertEqual(plan["scope"], "trading_runtime")
        self.assertIsNone(files["crates/logic.rs"])
        self.assertIn("docs/renamed.md", files)

    def test_protected_paths_and_journal_rewrite_refuse_before_merge(self):
        self.fixture()
        (self.repo / "docs/factory-charter.md").write_text("unauthorised amendment\n")
        self.git("add", ".")
        self.git("commit", "-qm", "protected")
        with self.assertRaisesRegex(r.EvidenceError, "release_scope_protected"):
            self.c.releases.classify(self.job(), self.base, self.git("rev-parse", "HEAD"))
        self.git("checkout", "-f", self.head)
        (self.repo / "journal/day.md").write_text("erased prior evidence\n")
        self.git("add", ".")
        self.git("commit", "-qm", "rewrite")
        with self.assertRaisesRegex(r.EvidenceError, "journal_not_append_only"):
            self.c.releases.classify(self.job(), self.base, self.git("rev-parse", "HEAD"))

    def test_watch_mismatch_and_unknown_grammar_never_certify_repository_only(self):
        self.fixture()
        self.rules = list(reversed(self.rules))
        self.assertEqual(self.c.releases.classify(self.job(), self.base, self.merge)[0]["scope"], "repository_only")
        self.rules.append("/docs/**")
        with self.assertRaisesRegex(r.EvidenceError, "watch_patterns_mismatch"):
            self.c.releases.classify(self.job(), self.base, self.merge)
        self.rules = ["!/docs/**"]
        with self.assertRaises(r.EvidenceError):
            self.c.releases.classify(self.job(), self.base, self.merge)

    def test_wrong_merge_tree_and_unknown_time_are_retained_not_verified(self):
        self.fixture()
        self.merge = self.git("commit-tree", self.base + "^{tree}", "-p", self.base, input_text="wrong tree\n")
        self.c.store.update(self.jid, "deployment_wait", merged=self.merge)
        self.c.release(self.job())
        self.assertIn("merge_tree_mismatch", self.job()["result"])
        for stamp in (None, "2026-10-31T00:00:00", "2999-01-01T00:00:00Z"):
            self.merged_at = stamp
            self.c.store.update(self.jid, "deployment_wait")
            self.c.release(self.job())
            self.assertIn("release_time_unavailable", self.job()["result"])


class DeploymentTests(Fixture):
    def test_exact_active_runtime_requires_behaviour_not_just_success(self):
        self.fixture(runtime=True)
        self.active = [self.deployment()]
        self.latest = self.active[0]
        with self.health():
            self.c.release(self.job())
        self.assertEqual(self.job()["state"], "verification_queued")
        self.assertEqual(self.c.store.used(), 0)
        self.assertNotEqual(self.job()["state"], "completed")

    def test_failed_crashed_skipped_runtime_have_terminal_operational_handoff(self):
        self.fixture(runtime=True)
        self.active = [self.deployment(self.base, seconds=-300)]
        for status in ("FAILED", "CRASHED", "SKIPPED"):
            self.latest = self.deployment(status=status)
            self.c.store.update(self.jid, "deployment_wait")
            self.c.release(self.job())
            self.assertEqual(self.job()["state"], "verification_pending")
            result = json.loads(self.job()["result"])
            self.assertFalse(result["owner_action_required"])
            self.assertFalse(result["acceptance_passed"])
            self.assertEqual(result["deployment_status"], status)

    def test_newer_failed_build_does_not_supersede_the_exact_active_revision(self):
        self.fixture(runtime=True)
        self.active = [self.deployment()]
        self.latest = self.deployment("e" * 40, "FAILED", seconds=-1, identity="new-failure")
        with self.health():
            self.c.release(self.job())
        self.assertEqual(self.job()["state"], "verification_queued")
        self.assertFalse(any("deployments(first:" in q for q in self.queries))

    def test_newer_active_revision_and_verifier_identity_change_are_not_false_green(self):
        self.fixture(runtime=True)
        self.active = [self.deployment()]
        self.latest = self.active[0]
        with self.health():
            self.c.release(self.job())
        self.active = [self.deployment("e" * 40, seconds=-1, identity="replacement")]
        self.latest = self.active[0]
        self.assertFalse(self.c.releases.observe(self.job(), verifying=True))
        self.assertEqual(self.job()["state"], "verification_pending")
        self.assertIn("release_superseded", self.job()["result"])

    def test_deadline_survives_restart_and_does_not_use_mutable_updated_time(self):
        self.fixture(runtime=True)
        self.merged_at = (dt.datetime.now(dt.timezone.utc) - dt.timedelta(seconds=1900)).isoformat()
        self.active = [self.deployment(self.base, seconds=-3000)]
        self.latest = self.deployment(status="BUILDING")
        plan = self.c.releases.plan(self.job())
        self.c.store.update(self.jid, "deployment_wait")
        fresh = r.Releases(self.c, w.REPO, w.SERVICE, w.ENVIRONMENT, w.PROTECTED)
        with patch.object(fresh, "api", side_effect=self.api):
            fresh.observe(self.job())
        self.assertEqual(self.job()["state"], "verification_pending")
        self.assertIn("deployment_evidence_timeout", self.job()["result"])
        self.assertEqual(fresh.saved(self.jid, "release_scope")["wait_deadline_at"], plan["wait_deadline_at"])

    def test_unchanged_wait_has_no_per_poll_events_and_polls_at_most_every_thirty_seconds(self):
        self.fixture(runtime=True)
        self.active = [self.deployment(self.base, seconds=-300)]
        self.latest = self.deployment(status="BUILDING")
        with patch.object(r.time, "monotonic", return_value=100):
            self.c.releases.poll()
            count = len(self.queries)
            self.c.releases.poll()
            self.assertEqual(len(self.queries), count)
        with self.c.store.connect() as db:
            prior = db.execute("SELECT count(*) FROM events").fetchone()[0]
        self.c.release(self.job())
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT count(*) FROM events").fetchone()[0], prior)


class RecoveryAndAuthorityTests(Fixture):
    def test_operator_receipt_rejects_fifo_and_nonprivate_file_without_waiting(self):
        self.fixture()
        path = self.c.store.private / "pipe"
        os.mkfifo(path, 0o600)
        with self.assertRaisesRegex(r.EvidenceError, "private_acceptance_file_required"):
            self.c.releases.accept_repository(self.jid, path)
        path.unlink()
        path.write_text("{}")
        path.chmod(0o644)
        with self.assertRaisesRegex(r.EvidenceError, "private_acceptance_file_required"):
            self.c.releases.accept_repository(self.jid, path)
        self.assertEqual(self.c.store.used(), 0)

    def test_operator_cli_registers_changed_private_receipt_without_model_or_policy_change(self):
        self.fixture()
        self.c.release(self.job())
        self.assertEqual(self.job()["state"], "verification_pending")
        _, files = self.c.releases.classify(self.job(), self.base, self.merge)
        path = self.c.store.private / "approved.json"
        path.write_text(json.dumps({"head": self.head, "source": "independent_review",
            "criteria": "exact_artifact_content", "reference": "reviewed exact content", "files": files}))
        path.chmod(0o600)
        command = [sys.executable, str(Path(w.__file__)), "accept-repository", "--job", str(self.jid), "--receipt", str(path)]
        first = subprocess.run(command, env=self.config.env, capture_output=True, text=True, check=True)
        self.assertTrue(json.loads(first.stdout)["receipt_registered"])
        self.assertEqual(self.job()["state"], "deployment_wait")
        second = subprocess.run(command, env=self.config.env, capture_output=True, text=True, check=True)
        self.assertFalse(json.loads(second.stdout)["receipt_registered"])
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(self.config.limit, 36)

    def test_already_merged_after_cutoff_is_reconciled_without_any_remote_write(self):
        self.fixture()
        self.declare()
        self.config.work_until = dt.datetime(2000, 1, 1, tzinfo=dt.timezone.utc)
        self.c.store.update(self.jid, "ci_wait", merged=None)
        with patch.object(self.c, "prepare", side_effect=AssertionError("reconciliation must not prepare DB/build")):
            self.c.release(self.job())
            self.c.release(self.job())
        self.assertEqual(self.job()["state"], "completed")
        self.assertEqual(self.writes, [])
        self.assertEqual(self.c.store.used(), 0)
        self.assertFalse(self.config.mutation_allowed())

    def test_interrupted_verifier_rechecks_scope_before_any_new_reservation(self):
        self.fixture()
        self.c.store.update(self.jid, "running")
        self.c.store.recover()
        self.assertEqual(self.job()["state"], "verification_queued")
        self.c.releases.poll()
        self.assertEqual(self.job()["state"], "verification_pending")
        self.assertIsNone(self.c.store.reserve(36))
        self.assertEqual(self.c.store.used(), 0)

    def test_nonurgent_verifier_progresses_beside_closed_ci_and_retained_failed_workspace(self):
        self.fixture(runtime=True)
        self.active = [self.deployment()]
        self.latest = self.active[0]
        self.c.store.update(self.jid, "verification_queued")
        hold = self.c.store.enqueue("unrelated retained release")
        self.c.store.update(hold, "ci_wait", pr=43, head="f" * 40)
        failed = self.c.store.enqueue("unrelated retained workspace")
        self.c.store.update(failed, "failed")
        accepted = self.c.store.enqueue(w.ACCEPTANCE_TASK)
        self.c.store.update(accepted, "completed")
        self.config.work_until = dt.datetime(2000, 1, 1, tzinfo=dt.timezone.utc)
        self.c.boundary_ready = True
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            self.config.port = listener.getsockname()[1]
        def finish(job, _repo):
            self.assertEqual(job["id"], self.jid)
            self.assertEqual(job["state"], "verification_queued")
            self.c.store.update(self.jid, "completed")
            self.c.shutdown.set()
        timer = threading.Timer(2, self.c.shutdown.set)
        timer.start()
        try:
            with patch.object(self.c.reporter, "run"), patch.object(self.c, "activity_loop"), \
                    patch.object(self.c, "incident_loop"), patch.object(self.c, "stop_loss_loop"), \
                    patch.object(self.c, "discover_models"), patch.object(self.c, "prepare", return_value=self.repo), \
                    patch.object(self.c, "handoff_blockers", side_effect=AssertionError("read-only verification must not inherit mutation handoff")), \
                    patch.object(self.c, "run_job", side_effect=finish) as run, self.health():
                self.c.serve()
        finally:
            timer.cancel()
        self.assertEqual(run.call_count, 1)
        self.assertEqual(self.job()["state"], "completed")
        self.assertEqual(self.writes, [])

    def test_old_ci_hold_visible_beyond_latest_twenty_and_closed_window_reads_only(self):
        self.fixture()
        self.remote_merged = False
        self.c.store.update(self.jid, "ci_wait", merged=None)
        for _ in range(25):
            jid = self.c.store.enqueue("newer completed review")
            self.c.store.update(jid, "completed")
        self.config.work_until = dt.datetime(2000, 1, 1, tzinfo=dt.timezone.utc)
        self.c.release(self.job())
        status = self.c.status()
        self.assertNotIn(self.jid, [j["id"] for j in status["jobs"]])
        self.assertEqual(status["activity"]["release_backlog"]["items"][0]["id"], self.jid)
        self.assertEqual(status["activity"]["reason"], "owner_engineering_window_closed")
        self.assertEqual(self.writes, [])
        self.assertEqual(self.job()["state"], "ci_wait")
        with contextlib.redirect_stdout(io.StringIO()):
            self.c.activity_tick()
            self.c.activity_tick()
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT count(*) FROM events WHERE kind='activity_transition'").fetchone()[0], 1)

    def test_closed_release_hold_does_not_starve_an_eligible_read_only_review(self):
        self.fixture()
        self.remote_merged = False
        self.c.store.update(self.jid, "ci_wait", merged=None)
        accepted = self.c.store.enqueue(w.ACCEPTANCE_TASK)
        self.c.store.update(accepted, "completed")
        review = self.c.store.enqueue(w.stop_loss_reviews.PREFIX + " source-linked review")
        self.config.work_until = dt.datetime(2000, 1, 1, tzinfo=dt.timezone.utc)
        self.c.boundary_ready = True
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            self.config.port = listener.getsockname()[1]
        def finish(job, _repo):
            self.assertEqual(job["id"], review)
            self.c.store.update(review, "completed")
            self.c.shutdown.set()
        timer = threading.Timer(2, self.c.shutdown.set)
        timer.start()
        with patch.object(self.c.reporter, "run"), patch.object(self.c, "activity_loop"), \
                patch.object(self.c, "incident_loop"), patch.object(self.c, "stop_loss_loop"), \
                patch.object(self.c, "discover_models"), patch.object(self.c, "prepare", return_value=self.repo), \
                patch.object(self.c, "run_job", side_effect=finish) as run:
            self.c.serve()
        timer.cancel()
        self.assertEqual(run.call_count, 1)
        self.assertEqual(self.writes, [])
        self.assertEqual(self.job()["state"], "ci_wait")
        self.assertEqual(self.config.limit, 36)


if __name__ == "__main__":
    unittest.main()
