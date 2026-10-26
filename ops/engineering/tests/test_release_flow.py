"""Real Git coverage of the post-model release path; no network or inference."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("release_flow_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class ReleaseFlowTests(unittest.TestCase):
    def test_changes_commit_push_and_verified_pr_use_real_git(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*args, cwd=None):
                return subprocess.check_output(["git", *args], cwd=cwd, text=True,
                                               stderr=subprocess.PIPE).strip()
            seed = root / "seed"
            git("init", "-b", "main", str(seed))
            (seed / "crates" / "example" / "src").mkdir(parents=True)
            (seed / "crates" / "example" / "src" / "lib.rs").write_text("pub fn value() -> u8 { 1 }\n")
            (seed / "journal").mkdir()
            (seed / "journal" / "base.md").write_text("prior evidence\n")
            git("add", ".", cwd=seed)
            git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-m", "base", cwd=seed)
            remote = root / "remote.git"
            git("clone", "--bare", str(seed), str(remote))
            repo = root / "repository"
            git("clone", str(remote), str(repo))
            (root / "workspaces").mkdir()
            (root / "agent-state").mkdir()
            c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": str(root), "PATH": os.environ["PATH"]}))
            c.uid, c.gid = os.getuid(), os.getgid()
            jid = c.store.enqueue("repair one authorized defect")
            job = c.store.reserve(1)
            previous = c.store.enqueue("prior completed investigation")
            c.store.update(previous, "failed", result="RuntimeError: publish blocked")
            c.store.event("model_report", previous, json.dumps({
                "outcome": "changes", "summary": "Retain the verified regression evidence; do not repeat the same fix",
                "tests": "behavioral regression red then green", "owner_decisions": "none"}))
            created = []
            def gh(*args, **kwargs):
                if args[:2] == ("pr", "list"):
                    if not created:
                        return "[]"
                    return json.dumps([{"number": 42, "state": "OPEN", "headRefOid": git("rev-parse", "refs/heads/engineering/job-1", cwd=remote),
                                        "mergeCommit": None, "baseRefName": "main", "headRepositoryOwner": {"login": "ang-kl"},
                                        "isCrossRepository": False}])
                self.assertEqual(args[:2], ("pr", "create"))
                self.assertTrue(kwargs["write"])
                self.assertIn("Meaningful repair", Path(args[args.index("--body-file") + 1]).read_text())
                created.append(True)
                return "https://github.com/ang-kl/jiaoyi/pull/42"
            def model_or_format(args, **kwargs):
                if args[0] == "codex":
                    self.assertIn("Retain the verified regression evidence", args[-1])
                    self.assertIn("RuntimeError: publish blocked", args[-1])
                    workspace = Path(kwargs["cwd"])
                    (workspace / "crates" / "example" / "src" / "lib.rs").write_text("pub fn value() -> u8 { 2 }\n")
                    (workspace / "journal" / "new.md").write_text("Meaningful repair with a regression receipt.\n")
                    Path(args[args.index("--output-last-message") + 1]).write_text(json.dumps({
                        "outcome": "changes", "summary": "Meaningful repair", "tests": "regression red then green",
                        "owner_decisions": "none", "acceptance_passed": False}))
                else:
                    self.assertEqual(args, ["cargo", "fmt", "--all", "--check"])
                kwargs["logfile"].write_text("simulated CLI only; actual Git release path\n")
                return "ok"
            with patch.object(c, "gh", side_effect=gh), patch.object(c, "login_model", return_value="ok"), \
                 patch.object(w, "bounded_process", side_effect=model_or_format), contextlib.redirect_stdout(io.StringIO()):
                c.run_job(job, repo)
            saved = next(row for row in c.store.jobs() if row["id"] == jid)
            self.assertEqual((saved["state"], saved["pr"]), ("ci_wait", 42))
            self.assertEqual(saved["head"], git("rev-parse", "refs/heads/engineering/job-1", cwd=remote))
            self.assertEqual(git("rev-list", "--count", "main..engineering/job-1", cwd=remote), "1")
            self.assertEqual(git("show", "engineering/job-1:journal/new.md", cwd=remote),
                             "Meaningful repair with a regression receipt.")
            self.assertEqual(c.store.used(), 1)
            diagnostic = c.diagnose_job(saved)
            self.assertTrue(diagnostic["git_pointer_valid"])
            self.assertEqual(diagnostic["scope_guard"], "passed")
            self.assertEqual(diagnostic["commits_ahead"], 1)
            self.assertTrue(diagnostic["fmt_log_present"])

            # A failed real push can leave a local commit but no remote ref.
            # The diagnostic handshake must not create that missing ref.
            git("update-ref", "-d", "refs/heads/engineering/job-1", cwd=remote)
            failed = {**saved, "state": "failed"}
            diagnostic = c.diagnose_job(failed)
            self.assertEqual(diagnostic["write_probe"]["result"], "dry_run_handshake_ok")
            self.assertFalse(diagnostic["write_probe"]["mutated_remote"])
            self.assertEqual(git("for-each-ref", "--format=%(refname)", "refs/heads/engineering/job-1", cwd=remote), "")
            with patch.object(c, "_dry_run_push", side_effect=AssertionError("repeat handshake")) as probe:
                self.assertEqual(c.diagnose_job(failed)["write_probe"], diagnostic["write_probe"])
                probe.assert_not_called()

            # Retained evidence pinpoints a rejected scope before a new model
            # attempt, while credential-bearing formatter output stays private.
            workspace = root / "workspaces" / str(jid)
            (workspace / "outside.tmp").write_text("outside the allowed scope")
            c.config.env["TELEGRAM_BOT_TOKEN"] = "123456:private-telegram"
            c.config.env["GH_TOKEN"] = "private-github-token"
            (c.store.private / f"{jid}-fmt.log").write_text(
                "https://api.telegram.org/bot123456:private-telegram/sendDocument private-github-token")
            diagnostic = c.diagnose_job(saved)
            self.assertEqual(diagnostic["scope_guard"], "blocked")
            self.assertIn("outside.tmp", diagnostic["scope_error"])
            self.assertNotIn("private-telegram", json.dumps(diagnostic))
            self.assertNotIn("private-github-token", json.dumps(diagnostic))
            self.assertNotIn("api.telegram.org/bot", json.dumps(diagnostic))

            (workspace / ".git").write_text("gitdir: /untrusted/location\n")
            with patch.object(c, "git", side_effect=AssertionError("untrusted Git must not execute")) as root_git:
                self.assertFalse(c.diagnose_job(saved)["git_pointer_valid"])
                root_git.assert_not_called()

    def test_write_probe_never_returns_remote_stderr_or_tokens(self):
        with tempfile.TemporaryDirectory() as temp:
            c = w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": temp, "PATH": os.environ["PATH"]}))
            error = subprocess.CompletedProcess([], 128, "", "remote: Write access to repository not granted. HTTP403 https://private-token@github.com/owner/repo")
            with patch.object(w.subprocess, "run", return_value=error) as run:
                result = c._dry_run_push(Path(temp), 17)
            self.assertEqual(result["classifications"], ["http_403", "write_access_denied"])
            self.assertNotIn("private-token", json.dumps(result))
            self.assertNotIn("github.com", json.dumps(result))
            self.assertIn("--dry-run", run.call_args.args[0])
            self.assertEqual(run.call_args.args[0][-1], "HEAD:refs/heads/engineering/job-17")

