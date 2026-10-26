import concurrent.futures
import contextlib
import io
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class Fixture:
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.store = w.Store(self.root)

    def tearDown(self):
        self.temp.cleanup()

    def config(self, **extra):
        return w.Config({"ENGINEERING_STATE_DIR": str(self.root), **extra})


class WorkerTests(Fixture, unittest.TestCase):
    def test_disabled_and_explicit_activation_inputs(self):
        self.assertEqual(self.config().blocked(), "disabled")
        config = self.config(ENGINEERING_ENABLED="1")
        self.assertIn("OPENAI_API_KEY", config.blocked())
        self.assertIn("ENGINEERING_DAILY_SESSION_LIMIT", config.blocked())
        config = self.config(ENGINEERING_ENABLED="1", OPENAI_API_KEY="secret", GH_TOKEN="secret2",
                             RAILWAY_TOKEN="secret3", ENGINEERING_MODEL="owner-selected-model",
                             ENGINEERING_DAILY_SESSION_LIMIT="2",
                             ENGINEERING_BUDGET_ACK="session-limits-are-not-a-dollar-cap")
        self.assertIsNone(config.blocked())
        with self.assertRaises(ValueError):
            self.config(ENGINEERING_SESSION_TIMEOUT_SECONDS="99999")
        with self.assertRaises(ValueError):
            self.config(ENGINEERING_ISOLATION_MODE="unrestricted-root")
        self.assertEqual(self.config().isolation, "codex")
        self.assertEqual(self.config(ENGINEERING_ISOLATION_MODE="uid-container").isolation, "uid-container")

    def test_disabled_status_exposes_missing_names_and_source_without_values(self):
        c = w.Coordinator(self.config(OPENAI_API_KEY="private-key"))
        s = c.status()
        self.assertEqual(s["state"], "disabled")
        self.assertNotIn("OPENAI_API_KEY", s["missing_activation_inputs"])
        self.assertIn("GH_TOKEN", s["missing_activation_inputs"])
        self.assertNotIn("private-key", json.dumps(s))
        self.assertEqual(s["execution_boundary"], {"mode": "codex", "verified": None})
        self.assertEqual(len(s["worker_source_sha256"]), 64)

    def test_chatgpt_activation_uses_plan_default_without_api_key(self):
        config = self.config(ENGINEERING_AUTH_MODE="chatgpt", ENGINEERING_ENABLED="1",
            GH_TOKEN="gh", RAILWAY_TOKEN="rail", ENGINEERING_DAILY_SESSION_LIMIT="1",
            ENGINEERING_BUDGET_ACK="session-limits-are-not-a-dollar-cap")
        self.assertIsNone(config.blocked())
        c = w.Coordinator(config)
        with patch.object(w.urllib.request, "urlopen", side_effect=AssertionError("API discovery")):
            c.discover_models()
        self.assertEqual(c.status()["model_auth_mode"], "chatgpt")
        self.assertIn("ENGINEERING_MODEL", self.config(ENGINEERING_AUTH_MODE="chatgpt", ENGINEERING_MODEL="bad model").missing())
        with self.assertRaises(ValueError):
            self.config(ENGINEERING_AUTH_MODE="automatic-fallback")

    def test_chatgpt_login_reuses_only_worker_home_and_rejects_api_provider(self):
        c = w.Coordinator(self.config(ENGINEERING_AUTH_MODE="chatgpt"))
        c.uid = os.getuid()
        home = self.root / "codex-chatgpt"
        home.mkdir(mode=0o700)
        attempt = self.root / "attempt"
        attempt.mkdir()
        env = w.clean_environment(c.config, self.root / "work", attempt, c.uid)
        common = dict(env=env, cwd=self.root, stop=lambda: False)
        def status(args, **kwargs):
            self.assertEqual(args, ["codex", "login", "status"])
            self.assertIsNone(kwargs["input_text"])
            self.assertEqual(kwargs["env"]["HOME"], str(attempt))
            self.assertEqual(kwargs["env"]["CODEX_HOME"], str(home))
            self.assertNotIn("OPENAI_API_KEY", kwargs["env"])
            kwargs["logfile"].write_text("Logged in using ChatGPT\n")
            return "ok"
        with patch.object(w, "bounded_process", side_effect=status):
            self.assertEqual(c.login_model(1, common), "ok")
        with patch.object(w, "bounded_process", return_value="exit_1"):
            self.assertEqual(c.login_model(1, common), "exit_1")
        (c.store.private / "1-login.log").write_text("Logged in using an API key\n")
        with patch.object(w, "bounded_process", return_value="ok"):
            self.assertEqual(c.login_model(1, common), "wrong_auth_provider")

    def test_chatgpt_login_refuses_missing_public_or_symlink_directory(self):
        c = w.Coordinator(self.config(ENGINEERING_AUTH_MODE="chatgpt"))
        c.uid = os.getuid()
        home = self.root / "codex-chatgpt"
        with patch.object(w, "bounded_process") as launch:
            with self.assertRaises(FileNotFoundError):
                c.login_model(1, {"env": {}})
            home.mkdir(mode=0o755)
            home.chmod(0o755)
            with self.assertRaises(RuntimeError):
                c.login_model(1, {"env": {}})
            home.rmdir()
            home.symlink_to(self.store.private, target_is_directory=True)
            with self.assertRaises(RuntimeError):
                c.login_model(1, {"env": {}})
            launch.assert_not_called()

    def test_model_discovery_only_reads_metadata_and_keeps_key_out_of_logs(self):
        c = w.Coordinator(self.config(OPENAI_API_KEY="private-model-key"))
        class Response:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, _): return b'{"data":[{"id":"gpt-coding-model"}]}'
        def request(req, timeout):
            self.assertEqual(req.full_url, "https://api.openai.com/v1/models")
            self.assertEqual(req.get_method(), "GET")
            self.assertEqual(req.get_header("Authorization"), "Bearer private-model-key")
            return Response()
        log = io.StringIO()
        with patch.object(w.urllib.request, "urlopen", side_effect=request), contextlib.redirect_stdout(log):
            c.discover_models()
        self.assertIn("gpt-coding-model", log.getvalue())
        self.assertNotIn("private-model-key", log.getvalue())
        self.assertEqual(c.catalog_state, "available_in_private_logs")

    def test_recovery_preserves_quota_and_selects_ci_repair(self):
        first = self.store.enqueue("reproduce failure")
        second = self.store.enqueue("repair existing PR")
        self.store.reserve(2)
        self.store.reserve(2)
        self.store.update(second, "running", pr=5)
        self.store.recover()
        rows = self.store.jobs(("queued", "repair_queued"))
        self.assertEqual([(r["id"], r["state"]) for r in rows], [(first, "queued"), (second, "repair_queued")])
        self.assertEqual(self.store.used(), 2)
        self.assertIsNone(self.store.reserve(2))
        self.assertEqual(w.Store(self.root).jobs()[0]["pr"], 5)

    def test_interrupted_verification_is_resumed_as_verification(self):
        jid = self.store.enqueue("verify deployed fix")
        self.store.update(jid, "running", pr=10, head="a" * 40, merged="b" * 40)
        self.store.recover()
        self.assertEqual(self.store.jobs()[0]["state"], "verification_queued")
        self.assertEqual(self.store.reserve(1)["state"], "verification_queued")

    def test_quota_reservation_is_atomic_across_connections(self):
        for i in range(5):
            self.store.enqueue(str(i))
        with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
            results = list(pool.map(lambda _: self.store.reserve(1), range(5)))
        self.assertEqual(sum(r is not None for r in results), 1)
        self.assertEqual(self.store.used(), 1)

    def test_singleton_refuses_competing_process_and_releases_after_exit(self):
        code = ("import importlib.util,sys; s=importlib.util.spec_from_file_location('w',sys.argv[1]);"
                "m=importlib.util.module_from_spec(s);s.loader.exec_module(m);"
                "c=m.Store(sys.argv[2]).singleton();c.__enter__()")
        with self.store.singleton():
            p = subprocess.run([sys.executable, "-c", code, str(MODULE), str(self.root)], capture_output=True)
            self.assertNotEqual(p.returncode, 0)
            self.assertIn(b"another coordinator", p.stderr)
        with self.store.singleton():
            pass

    def test_model_environment_excludes_supervisor_and_production_secrets(self):
        config = self.config(GH_TOKEN="gh", RAILWAY_TOKEN="rail", OPENAI_API_KEY="model",
                             CTRADER_ACCESS_TOKEN="broker", DATABASE_URL="production", PATH="/usr/bin")
        env = w.clean_environment(config, self.root / "work", self.root / "auth", 10001)
        self.assertFalse({"GH_TOKEN", "RAILWAY_TOKEN", "OPENAI_API_KEY", "CTRADER_ACCESS_TOKEN"} & env.keys())
        self.assertEqual(env["DATABASE_URL"], "postgres://engineering@127.0.0.1:15432/postgres")

    def test_actual_process_timeout_and_secret_redaction(self):
        log = self.root / "session.log"
        result = w.bounded_process([sys.executable, "-u", "-c", "import time; print('private-key'); time.sleep(20)"],
                                  env=os.environ.copy(), cwd=self.root, logfile=log, timeout=0.3,
                                  stop=lambda: False, redact=("private-key",))
        self.assertEqual(result, "timeout")
        self.assertIn("[REDACTED]", log.read_text())
        self.assertNotIn("private-key", log.read_text())
        self.assertFalse(log.with_suffix(".raw").exists())

    def test_stop_terminates_running_session_and_retains_log(self):
        stop = threading.Event()
        timer = threading.Timer(0.3, stop.set)
        timer.start()
        try:
            result = w.bounded_process([sys.executable, "-u", "-c", "import time; print('started'); time.sleep(20)"],
                env=os.environ.copy(), cwd=self.root, logfile=self.root / "stop.log", timeout=10, stop=stop.is_set)
        finally:
            timer.cancel()
        self.assertEqual(result, "interrupted")
        self.assertIn("started", (self.root / "stop.log").read_text())

    def test_stop_file_prevents_remote_mutations(self):
        c = w.Coordinator(self.config())
        (self.root / "STOP").touch()
        with self.assertRaisesRegex(RuntimeError, "stopped"):
            c.command(["unreachable-command"], write=True)
        self.assertEqual(c.status()["state"], "stopped")

    def test_untrusted_results_and_git_pointers_reject_fifo_symlink_and_oversize(self):
        regular = self.root / "result"
        regular.write_text("valid")
        self.assertEqual(w.bounded_text(regular, 5), "valid")
        with self.assertRaises(RuntimeError):
            w.bounded_text(regular, 4)
        symlink = self.root / "link"
        symlink.symlink_to(regular)
        with self.assertRaises(OSError):
            w.bounded_text(symlink, 10)
        fifo = self.root / "fifo"
        os.mkfifo(fifo)
        started = time.monotonic()
        with self.assertRaises(RuntimeError):
            w.bounded_text(fifo, 10)
        self.assertLess(time.monotonic() - started, 1)

    def test_disabled_server_never_starts_model_and_reopens_state(self):
        c = w.Coordinator(self.config(PORT="18947"))
        with patch.object(c, "prepare", side_effect=AssertionError("model work while disabled")):
            t = threading.Thread(target=c.serve)
            t.start()
            time.sleep(0.25)
            c.shutdown.set()
            t.join(timeout=3)
            self.assertFalse(t.is_alive())
        self.assertEqual(self.store.used(), 0)
        self.assertEqual(self.store.jobs(), [])

    def test_incomplete_readonly_report_cannot_unlock_routine_engineering(self):
        c = w.Coordinator(self.config())
        c.uid, c.gid = os.getuid(), os.getgid()
        repo = self.root / "repository"
        (self.root / "agent-state").mkdir()
        class Response:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, _): return b'{}'
        for passed in (None, False, True):
            with self.subTest(acceptance_passed=passed):
                jid = self.store.enqueue(w.ACCEPTANCE_TASK)
                job = self.store.reserve(10)
                workspace = self.root / "workspaces" / str(jid)
                workspace.mkdir(parents=True)
                (workspace / ".git").write_text("gitdir: " + str(repo / ".git" / "worktrees" / str(jid)) + "\n")
                report = {"outcome": "no_change", "summary": "Read-only session; failed delegation unless explicitly passed",
                          "tests": "Actual report regression: a successful main session alone is insufficient",
                          "owner_decisions": "none"}
                if passed is not None:
                    report["acceptance_passed"] = passed
                def session(*_, **kwargs):
                    (Path(kwargs["env"]["HOME"]) / "result.json").write_text(json.dumps(report))
                    return "ok"
                with patch.object(c, "existing_pr", return_value=None), patch.object(c, "login_model", return_value="ok"), \
                     patch.object(c, "changes", return_value=[]), patch.object(w.urllib.request, "urlopen", return_value=Response()), \
                     patch.object(w, "bounded_process", side_effect=session), contextlib.redirect_stdout(io.StringIO()):
                    c.run_job(job, repo)
                self.assertEqual(self.store.jobs()[0]["state"], "completed" if passed is True else "owner_blocked")

    def test_first_enabled_job_is_bounded_acceptance_and_consumes_one_reservation(self):
        c = w.Coordinator(self.config(PORT="18948", ENGINEERING_ENABLED="1", OPENAI_API_KEY="fake",
            GH_TOKEN="fake", RAILWAY_TOKEN="fake", ENGINEERING_MODEL="fake-model",
            ENGINEERING_DAILY_SESSION_LIMIT="1", ENGINEERING_BUDGET_ACK="session-limits-are-not-a-dollar-cap"))
        def model(job, _):
            self.assertEqual(job["task"], w.ACCEPTANCE_TASK)
            c.store.update(job["id"], "completed", result="simulated acceptance")
            c.shutdown.set()
        with patch.object(c, "prepare", return_value=self.root), patch.object(c, "run_job", side_effect=model) as launch:
            c.serve()
        self.assertEqual(launch.call_count, 1)
        self.assertEqual(self.store.used(), 1)

    def test_failed_acceptance_prevents_routine_model_work(self):
        jid = self.store.enqueue(w.ACCEPTANCE_TASK)
        self.store.update(jid, "failed", result="capability missing")
        self.store.enqueue("later engineering task")
        c = w.Coordinator(self.config(PORT="18949", ENGINEERING_ENABLED="1", OPENAI_API_KEY="fake",
            GH_TOKEN="fake", RAILWAY_TOKEN="fake", ENGINEERING_MODEL="fake-model",
            ENGINEERING_DAILY_SESSION_LIMIT="2", ENGINEERING_BUDGET_ACK="session-limits-are-not-a-dollar-cap"))
        timer = threading.Timer(0.25, c.shutdown.set)
        timer.start()
        try:
            with patch.object(c, "prepare") as prepare, patch.object(c, "run_job") as model:
                c.serve()
                prepare.assert_not_called()
                model.assert_not_called()
        finally:
            timer.cancel()
        self.assertEqual(c.phase, "acceptance_blocked")
        self.assertEqual(self.store.used(), 0)


class ReleaseTests(Fixture, unittest.TestCase):
    def checks(self, conclusion="success"):
        return {"check_runs": [{"name": name, "status": "completed", "conclusion": conclusion,
                               "app": {"slug": "github-actions"}} for name in ("check", "docker", "engineering")]}

    def job(self):
        jid = self.store.enqueue("test")
        self.store.update(jid, "ci_wait", pr=8, head="a" * 40)
        return self.store.jobs()[0]

    def pr(self, **extra):
        return {"state": "open", "merged": False, "base": {"ref": "main"},
                "head": {"sha": "a" * 40, "ref": "engineering/job-1", "repo": {"full_name": w.REPO}}, **extra}

    def test_merge_requires_all_gates_on_exact_head(self):
        c = w.Coordinator(self.config())
        job = self.job()
        calls = []
        def gh(*args, **kwargs):
            calls.append((args, kwargs))
            if kwargs.get("write"):
                return json.dumps({"merged": True, "sha": "b" * 40})
            return json.dumps(self.checks() if "check-runs" in args[-1] else self.pr())
        with patch.object(c, "gh", side_effect=gh):
            c.release(job)
        writes = [args for args, kw in calls if kw.get("write")]
        self.assertEqual(len(writes), 1)
        self.assertIn("sha=" + "a" * 40, writes[0])
        self.assertEqual(self.store.jobs()[0]["state"], "deployment_wait")

    def test_wrong_head_or_fork_or_base_cannot_merge(self):
        c = w.Coordinator(self.config())
        job = self.job()
        for field in ("head", "fork", "base"):
            pr = self.pr()
            if field == "head":
                pr["head"]["sha"] = "c" * 40
            elif field == "fork":
                pr["head"]["repo"]["full_name"] = "someone/jiaoyi"
            else:
                pr["base"]["ref"] = "other"
            with patch.object(c, "gh", return_value=json.dumps(pr)) as gh:
                c.release(job)
                self.assertEqual(gh.call_count, 1)
            self.assertEqual(self.store.jobs()[0]["state"], "owner_blocked")

    def test_missing_gate_or_pending_duplicate_prevents_merge(self):
        c = w.Coordinator(self.config())
        job = self.job()
        for mode in ("missing", "pending_duplicate"):
            checks = self.checks()
            if mode == "missing":
                checks["check_runs"].pop()
            else:
                checks["check_runs"].append({"name": "check", "status": "in_progress", "conclusion": None,
                                             "app": {"slug": "github-actions"}})
            with patch.object(c, "gh", side_effect=[json.dumps(self.pr()), json.dumps(checks)]) as gh:
                c.release(job)
                self.assertEqual(gh.call_count, 2)
            self.assertEqual(self.store.jobs()[0]["state"], "ci_wait")

    def test_failed_ci_queues_repair_instead_of_merging(self):
        c = w.Coordinator(self.config())
        job = self.job()
        with patch.object(c, "gh", side_effect=[json.dumps(self.pr()), json.dumps(self.checks("failure"))]) as gh:
            c.release(job)
            self.assertEqual(gh.call_count, 2)
        self.assertEqual(self.store.jobs()[0]["state"], "repair_queued")

    def test_restart_finds_existing_pr_without_new_model_session(self):
        c = w.Coordinator(self.config())
        jid = self.store.enqueue("continue")
        job = self.store.reserve(2)
        with patch.object(c, "existing_pr", return_value={"state": "OPEN", "number": 42, "headRefOid": "a" * 40}):
            c.run_job(job, self.root / "unused")
        saved = self.store.jobs()[0]
        self.assertEqual((saved["id"], saved["state"], saved["pr"]), (jid, "ci_wait", 42))

    def test_exact_deployment_queues_behavior_verification(self):
        c = w.Coordinator(self.config(RAILWAY_TOKEN="test"))
        job = self.job()
        self.store.update(job["id"], "deployment_wait", merged="b" * 40)
        job = self.store.jobs()[0]
        class Response:
            def __init__(self, data):
                self.data = json.dumps(data).encode()
            def __enter__(self):
                return self
            def __exit__(self, *_):
                pass
            def read(self):
                return self.data
        deployed = {"data": {"serviceInstance": {"latestDeployment": {
            "id": "deployment", "status": "SUCCESS", "meta": {"commitHash": "c" * 40}}}}}
        with patch.object(w.urllib.request, "urlopen", return_value=Response(deployed)):
            c.release(job)
        self.assertEqual(self.store.jobs()[0]["state"], "deployment_wait")
        deployed["data"]["serviceInstance"]["latestDeployment"]["meta"]["commitHash"] = "b" * 40
        with patch.object(w.urllib.request, "urlopen", side_effect=[Response(deployed), Response({"status": "ok"})]):
            c.release(job)
        self.assertEqual(self.store.jobs()[0]["state"], "verification_queued")


class ScopeTests(Fixture, unittest.TestCase):
    def repo(self):
        repo = self.root / "work"
        repo.mkdir()
        subprocess.run(["git", "init", "-q", str(repo)], check=True)
        (repo / "journal").mkdir()
        (repo / "journal" / "today.md").write_text("existing evidence\n")
        (repo / "docs").mkdir()
        (repo / "docs" / "factory-charter.md").write_text("registered contract\n")
        subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
        subprocess.run(["git", "-C", str(repo), "-c", "user.name=test", "-c", "user.email=test@test",
                        "commit", "-qm", "base"], check=True)
        return repo

    def test_contract_changes_and_missing_journal_are_refused(self):
        repo = self.repo()
        c = w.Coordinator(self.config())
        (repo / "docs" / "factory-charter.md").write_text("changed\n")
        with self.assertRaisesRegex(RuntimeError, "protected path"):
            c.changes(repo)
        (repo / "docs" / "factory-charter.md").write_text("registered contract\n")
        (repo / "docs" / "fix.md").write_text("fix\n")
        with self.assertRaisesRegex(RuntimeError, "journal"):
            c.changes(repo)
        (repo / "journal" / "today.md").write_text("existing evidence\nnew evidence\n")
        self.assertEqual(c.changes(repo), ["docs/fix.md", "journal/today.md"])

    def test_historical_journal_rewrite_is_refused(self):
        repo = self.repo()
        (repo / "journal" / "today.md").write_text("replacement\n")
        with self.assertRaisesRegex(RuntimeError, "preserve"):
            w.Coordinator(self.config()).changes(repo)


if __name__ == "__main__":
    unittest.main()
