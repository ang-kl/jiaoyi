"""A green published head must include current main before unattended merge."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("release_base_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class ReleaseBaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": self.temp.name}))
        self.jid = self.c.store.enqueue("retain and verify this published repair")
        self.head, self.base, self.new_main = "a" * 40, "b" * 40, "c" * 40
        self.c.store.update(self.jid, "ci_wait", pr=42, head=self.head)
        self.calls, self.merges = [], []
        self.main_reads = [self.base, self.base]
        self.comparison = {"base_commit": {"sha": self.base}, "merge_base_commit": {"sha": self.base},
                           "status": "ahead", "behind_by": 0}
        self.merged = False

    def tearDown(self):
        self.temp.cleanup()

    def job(self):
        return next(j for j in self.c.store.jobs() if j["id"] == self.jid)

    def gh(self, *args, **options):
        self.calls.append((args, options))
        if options.get("write"):
            self.assertEqual(args[:3], ("api", "--method", "PUT"))
            self.assertIn("sha=" + self.head, args)
            self.merges.append(args)
            return json.dumps({"merged": True, "sha": "d" * 40})
        path = args[-1]
        if path.endswith("/pulls/42"):
            return json.dumps({"state": "closed" if self.merged else "open", "merged": self.merged,
                "merge_commit_sha": "d" * 40, "base": {"ref": "main", "sha": self.base},
                "head": {"sha": self.head, "ref": "engineering/job-" + str(self.jid),
                         "repo": {"full_name": w.REPO}}})
        if "check-runs" in path:
            return json.dumps({"check_runs": [{"name": name, "status": "completed", "conclusion": "success",
                "app": {"slug": "github-actions"}} for name in ("check", "docker", "engineering")]})
        if path.endswith("/git/ref/heads/main"):
            return json.dumps({"object": {"sha": self.main_reads.pop(0)}})
        if "/compare/" in path:
            self.assertEqual(path, f"repos/{w.REPO}/compare/{self.base}...{self.head}")
            return json.dumps(self.comparison)
        self.fail("Unexpected GitHub operation: " + repr(args))

    def release(self):
        with patch.object(self.c, "gh", side_effect=self.gh):
            self.c.release(self.job())

    def assert_handoff(self):
        self.assertEqual(self.merges, [])
        saved = self.job()
        self.assertEqual((saved["state"], saved["pr"], saved["head"]), ("owner_blocked", 42, self.head))
        self.assertIsNone(saved["merged"])
        self.assertIn("integration", saved["result"])
        self.assertEqual(self.c.store.used(), 0)
        self.assertEqual(len(self.c.store.jobs()), 1)

    def test_main_advanced_beyond_green_branch_is_retained_for_integration(self):
        # CI tested H from B. Main is now B2 with a separate commit: H is
        # still green, but its tests did not cover B2 + H together.
        self.comparison.update(merge_base_commit={"sha": "0" * 40}, status="diverged", behind_by=1)
        self.release()
        self.assert_handoff()

    def test_main_moving_after_ancestry_check_prevents_merge(self):
        self.main_reads = [self.base, self.new_main]
        self.release()
        self.assert_handoff()

    def test_same_current_base_with_all_exact_head_checks_merges_once(self):
        self.release()
        self.assertEqual(len(self.merges), 1)
        self.assertEqual((self.job()["state"], self.job()["merged"]), ("deployment_wait", "d" * 40))
        self.assertEqual(self.main_reads, [])
        self.assertEqual(self.c.store.used(), 0)

    def test_inconsistent_comparison_is_not_ancestry_evidence(self):
        for comparison in (
                {**self.comparison, "base_commit": {"sha": self.new_main}},
                {**self.comparison, "merge_base_commit": {"sha": self.new_main}},
                {**self.comparison, "behind_by": 1},
                {**self.comparison, "behind_by": False},
                {**self.comparison, "status": "behind"},
                {}):
            with self.subTest(comparison=comparison):
                self.c.store.update(self.jid, "ci_wait")
                self.main_reads = [self.base, self.base]
                self.comparison = comparison
                self.release()
                self.assert_handoff()

    def test_unknown_main_read_keeps_existing_job_without_merge_or_model_retry(self):
        original = self.gh
        def unavailable(*args, **options):
            if args[-1].endswith("/git/ref/heads/main"):
                raise RuntimeError("GitHub unavailable")
            return original(*args, **options)
        with patch.object(self.c, "gh", side_effect=unavailable):
            with self.assertRaisesRegex(RuntimeError, "GitHub unavailable"):
                self.c.release(self.job())
        self.assertEqual(self.merges, [])
        self.assertEqual((self.job()["state"], self.job()["pr"], self.job()["head"]), ("ci_wait", 42, self.head))
        self.assertEqual(self.c.store.used(), 0)

    def test_missing_main_identity_cannot_be_used_as_a_compare_ref(self):
        self.main_reads = [None]
        with self.assertRaisesRegex(RuntimeError, "current main identity unavailable"):
            self.release()
        self.assertEqual(self.merges, [])
        self.assertFalse(any("/compare/" in args[-1] for args, _ in self.calls))
        self.assertEqual(self.job()["state"], "ci_wait")
        self.assertEqual(self.c.store.used(), 0)

    def test_already_merged_receipt_recovers_without_rechecking_later_main(self):
        self.merged = True
        self.main_reads = []
        self.release()
        self.assertEqual(self.merges, [])
        self.assertEqual(len(self.calls), 1)
        self.assertEqual((self.job()["state"], self.job()["merged"]), ("deployment_wait", "d" * 40))
        self.assertEqual(self.c.store.used(), 0)

    def test_owner_window_still_prevents_all_release_reads_and_writes(self):
        with patch.object(self.c.config, "mutation_allowed", return_value=False):
            self.release()
        self.assertEqual(self.calls, [])
        self.assertEqual(self.job()["state"], "ci_wait")
        self.assertEqual(self.c.phase, "mutation_window_closed")
        self.assertEqual(self.c.store.used(), 0)


if __name__ == "__main__":
    unittest.main()
