"""Published repairs survive stale PR projections without another AI attempt."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import test_worker as fixtures

w = fixtures.w


class PublicationHandoffTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        seed = self.root / "seed"
        self.git("init", "-b", "main", str(seed))
        (seed / "crates").mkdir()
        (seed / "crates" / "value.rs").write_text("pub fn value() -> u8 { 1 }\n")
        (seed / "journal").mkdir()
        (seed / "journal" / "base.md").write_text("Original evidence.\n")
        self.git("add", ".", cwd=seed)
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-m", "base", cwd=seed)
        self.old_head = self.git("rev-parse", "HEAD", cwd=seed)
        self.remote = self.root / "remote.git"
        self.git("clone", "--bare", str(seed), str(self.remote))
        self.repo = self.root / "repository"
        self.git("clone", str(self.remote), str(self.repo))
        (self.root / "workspaces").mkdir()
        (self.root / "agent-state").mkdir()
        self.c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"]}))
        self.c.uid, self.c.gid = os.getuid(), os.getgid()
        self.jid = self.c.store.enqueue("Repair the existing reproducible CI defect.")

    def tearDown(self):
        self.temp.cleanup()

    def git(self, *args, cwd=None):
        return subprocess.check_output(["git", *args], cwd=cwd, text=True, stderr=subprocess.PIPE).strip()

    def list_pr(self):
        return {"number": 42, "state": "OPEN", "headRefOid": self.old_head, "mergeCommit": None,
                "baseRefName": "main", "headRepositoryOwner": {"login": "ang-kl"}, "isCrossRepository": False}

    def rest_pr(self, head):
        return {"state": "open", "merged": False, "base": {"ref": "main"},
                "head": {"sha": head, "ref": "engineering/job-" + str(self.jid), "repo": {"full_name": w.REPO}}}

    def model(self, args, **options):
        if args[0] == "codex":
            workspace = Path(options["cwd"])
            (workspace / "crates" / "value.rs").write_text("pub fn value() -> u8 { 2 }\n")
            (workspace / "journal" / "fix.md").write_text("Bounded fix: regression failed then passed.\n")
            Path(args[args.index("--output-last-message") + 1]).write_text(json.dumps({
                "outcome": "changes", "summary": "Preserve a meaningful published repair.", "tests": "Behavioral red then green.",
                "owner_decisions": "none", "acceptance_passed": False}))
        else:
            self.assertEqual(args, ["cargo", "fmt", "--all", "--check"])
        options["logfile"].write_text("mock model/format; actual local Git commit and push\n")
        return "ok"

    def publish(self, repair=True, create=None):
        if repair:
            self.c.store.update(self.jid, "repair_queued", pr=42, head=self.old_head)
        job = self.c.store.reserve(5)
        def gh(*args, **options):
            if args[:2] == ("pr", "list"):
                return json.dumps([self.list_pr()] if repair else [])
            if args[:2] == ("pr", "create"):
                self.assertTrue(options["write"])
                saved = self.c.store.jobs()[0]
                self.assertEqual(saved["state"], "ci_wait")
                self.assertEqual(saved["head"], self.git("rev-parse", "refs/heads/engineering/job-1", cwd=self.remote))
                self.assertIn("meaningful published repair", saved["result"])
                return create() if create else "https://github.com/ang-kl/jiaoyi/pull/42"
            raise AssertionError("unexpected GitHub operation")
        with patch.object(self.c, "gh", side_effect=gh) as github, \
             patch.object(self.c, "login_model", return_value="ok"), \
             patch.object(w, "bounded_process", side_effect=self.model) as model, \
             contextlib.redirect_stdout(io.StringIO()):
            self.c.run_job(job, self.repo)
        return github, model

    def test_stale_graphql_head_then_fresh_rest_keeps_checkpoint_and_resumes_without_ai(self):
        github, model = self.publish()
        saved = self.c.store.jobs()[0]
        published = self.git("rev-parse", "refs/heads/engineering/job-1", cwd=self.remote)
        self.assertNotEqual(published, self.old_head)
        self.assertEqual((saved["state"], saved["pr"], saved["head"]), ("ci_wait", 42, published))
        self.assertFalse(any(call.args[:2] == ("pr", "create") for call in github.call_args_list))
        self.assertEqual(sum(call.args[0][0] == "codex" for call in model.call_args_list), 1)
        # The projection still has the stale head after the successful push.
        with patch.object(self.c, "gh", return_value=json.dumps([self.list_pr()])):
            self.assertEqual(self.c.existing_pr("engineering/job-1")["headRefOid"], self.old_head)
        self.c.store.recover()
        self.assertIsNone(self.c.store.reserve(5))
        responses = [json.dumps(self.rest_pr(published)), json.dumps({"check_runs": []})]
        with patch.object(self.c, "gh", side_effect=responses) as release, \
             patch.object(w, "bounded_process", side_effect=AssertionError("published repair cannot rerun AI")):
            self.c.release(self.c.store.jobs()[0])
        self.assertEqual(release.call_count, 2)
        self.assertEqual(self.c.store.jobs()[0]["head"], published)
        self.assertEqual(self.c.store.used(), 1)

    def test_persistent_different_rest_head_refuses_without_losing_published_head(self):
        self.publish()
        saved = self.c.store.jobs()[0]
        with patch.object(self.c, "gh", return_value=json.dumps(self.rest_pr(self.old_head))) as github:
            self.c.release(saved)
        refused = self.c.store.jobs()[0]
        self.assertEqual(refused["state"], "owner_blocked")
        self.assertEqual(refused["head"], saved["head"])
        self.assertEqual(github.call_count, 1)
        self.assertFalse(any(call.kwargs.get("write") for call in github.call_args_list))
        self.assertIsNone(self.c.store.reserve(5))
        self.assertEqual(self.c.store.used(), 1)

    def test_create_url_checkpoints_number_before_any_list_projection(self):
        github, _ = self.publish(repair=False)
        saved = self.c.store.jobs()[0]
        self.assertEqual((saved["state"], saved["pr"]), ("ci_wait", 42))
        self.assertEqual(sum(call.args[:2] == ("pr", "create") for call in github.call_args_list), 1)
        self.assertEqual(sum(call.args[:2] == ("pr", "list") for call in github.call_args_list), 1)

    def test_lost_creation_ack_resolves_existing_pr_without_duplicate_create_or_model(self):
        def lost():
            raise RuntimeError("create acknowledgement lost")
        with self.assertRaisesRegex(RuntimeError, "acknowledgement lost"):
            self.publish(repair=False, create=lost)
        saved = self.c.store.jobs()[0]
        self.assertEqual(saved["state"], "ci_wait")
        self.assertIsNone(saved["pr"])
        self.assertIn("meaningful published repair", saved["result"])
        self.c.store.recover()
        self.assertIsNone(self.c.store.reserve(5))
        replies = [json.dumps(self.rest_pr(saved["head"])), json.dumps({"check_runs": []})]
        with patch.object(self.c, "existing_pr", return_value=self.list_pr()), \
             patch.object(self.c, "gh", side_effect=replies) as github, \
             patch.object(w, "bounded_process", side_effect=AssertionError("resume cannot rerun AI")):
            self.c.release(saved)
        self.assertEqual(self.c.store.jobs()[0]["pr"], 42)
        self.assertEqual(self.c.store.used(), 1)
        self.assertFalse(any(call.kwargs.get("write") for call in github.call_args_list))

    def test_missing_pr_after_lost_ack_has_bounded_reads_and_explicit_next_action(self):
        def lost():
            raise RuntimeError("create acknowledgement lost")
        with self.assertRaises(RuntimeError):
            self.publish(repair=False, create=lost)
        saved = self.c.store.jobs()[0]
        with patch.object(self.c, "existing_pr", return_value=None) as lookup, patch.object(self.c, "gh") as github:
            self.c.release(saved)
        blocked = self.c.store.jobs()[0]
        self.assertEqual(lookup.call_count, 3)
        self.assertEqual(blocked["state"], "owner_blocked")
        self.assertIn("Inspect branch/PR creation", blocked["result"])
        self.assertEqual(blocked["head"], saved["head"])
        self.assertIsNone(self.c.store.reserve(5))
        github.assert_not_called()


if __name__ == "__main__":
    unittest.main()
