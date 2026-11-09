"""Bounded independent observations; all network/SQL requests are fakes."""
import datetime as dt
import io
import json
from pathlib import Path
import subprocess
import sys
import threading
import time
import unittest
import urllib.error
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from monitoring import sources

NOW = dt.datetime(2026, 11, 9, 3, tzinfo=dt.timezone.utc)
SINCE = NOW - dt.timedelta(hours=2)


class Reply(io.BytesIO):
    status = 200


class SourcesTests(unittest.TestCase):
    def test_explicit_checkpoint_budget_is_capped_without_extending_sql_or_once(self):
        http_budgets, sql_budgets = [], []

        def opened(request, timeout):
            http_budgets.append(timeout)
            return Reply(b"{}")

        def sql(env, deadline, since, observed_at):
            sql_budgets.append(deadline-time.monotonic())
            return {"status":"ok"}, {"status":"ok"}

        with patch.object(sources.urllib.request.OpenerDirector, "open", side_effect=opened), \
             patch.object(sources, "_sql", side_effect=sql):
            sources.collect_sources({"LIVE_MONITOR_DATABASE_URL":"postgresql://SECRET"}, SINCE, NOW, timeout=600)
            self.assertEqual(len(http_budgets), 2)
            self.assertTrue(all(11 < value <= 12 for value in http_budgets))
            self.assertTrue(all(0 < value <= 3.5 for value in sql_budgets))
            http_budgets.clear()
            sources.collect_sources({}, SINCE, NOW)
            self.assertEqual(len(http_budgets), 2)
            self.assertTrue(all(4 < value <= 4.2 for value in http_budgets))

    def test_parallel_sources_keep_success_and_never_share_owner_secret(self):
        owner_seen = threading.Event()
        requests = []

        def open_source(request, timeout):
            requests.append(request)
            self.assertGreater(timeout, 0)
            self.assertLessEqual(timeout, 4.2)
            if request.full_url.endswith("/health"):
                self.assertTrue(owner_seen.wait(0.5), "requests must be concurrent")
                raise urllib.error.URLError(TimeoutError("SECRET transport"))
            if request.full_url.endswith("/owner/setup-assessments"):
                owner_seen.set()
                return Reply(b'{"accounts":[]}')
            return Reply(b'{"reports":[]}')

        with patch.object(sources.urllib.request.OpenerDirector, "open", side_effect=open_source):
            result = sources.collect_sources({"JIAOYI_UI_PASSWORD": "SECRET"}, SINCE, NOW)
        self.assertIsNone(result["health"])
        self.assertEqual(result["setup_assessments"], {"accounts": []})
        self.assertEqual(result["operations_reports"], {"reports": []})
        self.assertEqual(result["source_status"]["health"]["status"], "timeout")
        self.assertEqual(result["source_status"]["sql_evidence"]["error_code"], "database_credential_missing")
        for request in requests:
            self.assertEqual(request.get_method(), "GET")
            expected = "Bearer SECRET" if "/owner/" in request.full_url else None
            self.assertEqual(request.get_header("Authorization"), expected)
        self.assertNotIn("SECRET", json.dumps(result))
        self.assertIsNone(sources.NoRedirect().redirect_request(None, None, 302, "", {}, "https://evil.invalid"))

    def test_deadline_does_not_wait_for_workers_and_slots_cannot_accumulate(self):
        release = threading.Event()
        entered = []

        def blocked(request, deadline):
            entered.append(request.full_url)
            release.wait(2)
            return {"late": True}, {"status": "ok"}

        try:
            with patch.object(sources, "_http_json", side_effect=blocked):
                start = time.monotonic()
                first = sources.collect_sources({"JIAOYI_UI_PASSWORD": "SECRET"}, SINCE, NOW, timeout=0.04)
                self.assertLess(time.monotonic()-start, 0.3)
                self.assertEqual(len(entered), 3)
                self.assertTrue(all(first[name] is None for name in sources.ROUTES))
                self.assertTrue(all(first["source_status"][name]["status"] == "timeout" for name in sources.ROUTES))
                second = sources.collect_sources({"JIAOYI_UI_PASSWORD": "SECRET"}, SINCE, NOW, timeout=0.04)
                self.assertEqual(len(entered), 3, "still-blocked sources cannot create more threads")
                self.assertTrue(all(second["source_status"][name]["status"] == "busy" for name in sources.ROUTES))
        finally:
            release.set()
            until = time.monotonic()+1
            while sources._ACTIVE and time.monotonic() < until:
                time.sleep(0.005)
        self.assertFalse(sources._ACTIVE)

    def test_once_process_exits_with_hung_daemon_sources(self):
        script = r'''
import datetime as dt, json, sys, threading
sys.path.insert(0, sys.argv[1])
from monitoring import sources
def hung(*args):
    threading.Event().wait(100)
sources._http_json = hung
now = dt.datetime.now(dt.timezone.utc)
value = sources.collect_sources({}, now, now, timeout=.04)
print(json.dumps(value["source_status"]["health"]["status"]))
'''
        run = subprocess.run([sys.executable, "-c", script, str(ROOT)], capture_output=True,
                             text=True, timeout=2, check=True)
        self.assertEqual(json.loads(run.stdout), "timeout")

    def test_body_limit_and_safe_errors_with_fixed_read_only_railway_query(self):
        seen = []
        sizes = []

        class Oversize(Reply):
            def read(self, size=-1):
                sizes.append(size)
                return b"x" * size

        def open_source(request, timeout):
            seen.append(request)
            if request.full_url == sources.RAILWAY_URL:
                body = json.loads(request.data)
                self.assertTrue(body["query"].lstrip().startswith("query("))
                self.assertNotIn("mutation", body["query"])
                self.assertEqual(body["variables"], {"s": sources.RAILWAY_SERVICE, "e": sources.RAILWAY_ENVIRONMENT})
                return Reply(json.dumps({"data":{"serviceInstance": {
                    "latestDeployment":{"id":"id","status":"SUCCESS","meta":{"commitHash":"a"*40,"secret":"SECRET"}},
                    "activeDeployments":[],
                }}}).encode())
            if request.full_url.endswith("/health"):
                return Oversize(b"")
            raise urllib.error.HTTPError("https://SECRET", 503, "SECRET body", {}, io.BytesIO(b"SECRET"))

        with patch.object(sources.urllib.request.OpenerDirector, "open", side_effect=open_source):
            result = sources.collect_sources({"RAILWAY_TOKEN": "SECRET"}, SINCE, NOW)
        self.assertEqual(sizes, [sources.MAX_BYTES+1])
        self.assertEqual(result["source_status"]["health"]["error_code"], "oversize")
        self.assertEqual(result["source_status"]["operations_reports"]["http_status"], 503)
        self.assertEqual(result["deployment"]["latestDeployment"]["meta"], {"commitHash":"a"*40})
        self.assertEqual(result["source_status"]["deployment"]["status"], "ok")
        for request in seen:
            expected = "SECRET" if request.full_url == sources.RAILWAY_URL else None
            self.assertEqual(request.get_header("Project-access-token"), expected)
        self.assertNotIn("SECRET", json.dumps(result))

    def test_sql_runs_in_parallel_and_malformed_origin_never_sends_credentials(self):
        calls = []
        def sql(env, deadline, since, observed_at):
            calls.append((deadline-time.monotonic(), since, observed_at))
            return {"status":"ok","closed_trades":[]}, {"status":"ok"}
        with patch.object(sources, "_sql", side_effect=sql), \
             patch.object(sources.urllib.request.OpenerDirector, "open", side_effect=AssertionError("invalid origin used")):
            result = sources.collect_sources({"TRADING_BASE_URL":"https://owner:SECRET@example.invalid?token=SECRET",
                "JIAOYI_UI_PASSWORD":"SECRET", "LIVE_MONITOR_DATABASE_URL":"postgresql://SECRET"}, SINCE, NOW)
        self.assertEqual(len(calls), 1)
        self.assertGreater(calls[0][0], 0)
        self.assertLessEqual(calls[0][0], 3.5)
        self.assertEqual(calls[0][1:], (SINCE.isoformat(), NOW.isoformat()))
        self.assertEqual(result["source_status"]["sql_evidence"]["status"], "ok")
        self.assertTrue(all(result["source_status"][name]["error_code"] == "invalid_base_url" for name in sources.ROUTES))
        self.assertNotIn("SECRET", json.dumps(result))


if __name__ == "__main__":
    unittest.main()
