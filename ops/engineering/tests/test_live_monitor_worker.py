"""Monitor integration stays independent of model sessions and existing reports."""
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import threading
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("live_monitor_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class FakeMonitor:
    def __init__(self, env, root):
        self.root = root
        self.started = threading.Event()
        self.stopped = threading.Event()
        self.requests = []

    def run(self, shutdown):
        self.started.set()
        shutdown.wait(2)
        self.stopped.set()

    def status(self):
        return {"enabled": True, "checkpoint_count": 0, "model_calls": 0}

    def owner_response(self, path, headers):
        self.requests.append(path)
        if headers.get("Authorization") != "Bearer owner-fixture":
            return 401, "text/plain", b"Owner authentication required."
        if path.endswith(".jsonl"):
            return 200, "application/x-ndjson", b'{"checkpoint":0}\n'
        return 200, "application/json", b'{"checkpoints":[]}'


class LiveMonitorWorkerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

    def coordinator(self, **extra):
        return w.Coordinator(w.Config({"ENGINEERING_STATE_DIR": self.temp.name, **extra}))

    def serve_once(self, coordinator):
        captured = {}

        class Server:
            def __init__(self, address, handler):
                captured["handler"] = handler
            def serve_forever(self): pass
            def shutdown(self): pass
            def server_close(self): pass

        def stop_after_start():
            if coordinator.monitor is not None:
                self.assertTrue(coordinator.monitor.started.wait(1),
                                "monitor must start independently before model discovery")
            coordinator.shutdown.set()

        with contextlib.ExitStack() as stack:
            stack.enter_context(patch.object(w, "ThreadingHTTPServer", Server))
            for method in ("activity_loop", "stop_loss_loop", "incident_loop"):
                stack.enter_context(patch.object(coordinator, method))
            reporter = stack.enter_context(patch.object(coordinator.reporter, "run"))
            reserve = stack.enter_context(patch.object(coordinator.store, "reserve",
                                                      side_effect=AssertionError("model reservation")))
            model = stack.enter_context(patch.object(coordinator, "run_job",
                                                    side_effect=AssertionError("model work")))
            stack.enter_context(patch.object(coordinator, "discover_models", side_effect=stop_after_start))
            coordinator.serve()
            self.assertEqual(reporter.call_count, 1, "existing reporter startup is preserved")
            reserve.assert_not_called()
            model.assert_not_called()
        return captured["handler"]

    @staticmethod
    def request(handler_type, path, headers=None):
        handler = object.__new__(handler_type)
        handler.path = path
        handler.headers = headers or {}
        handler.wfile = io.BytesIO()
        response = {"headers": {}}
        handler.send_response = lambda status: response.update(status=status)
        handler.send_header = lambda name, value: response["headers"].update({name: value})
        handler.end_headers = lambda: None
        handler.do_GET()
        response["body"] = handler.wfile.getvalue()
        return response

    def test_disabled_monitor_has_no_instance_or_private_evidence_route(self):
        with patch.object(w, "Monitor") as monitor:
            c = self.coordinator(JIAOYI_UI_PASSWORD="owner-fixture")
            monitor.assert_not_called()
            self.assertEqual(c.status()["live_monitor"], {"enabled": False})
            for relative in ("monitoring/runtime.py", "monitoring/sql_evidence.sql"):
                self.assertEqual(c.status()["source_files_sha256"][relative],
                                 hashlib.sha256((MODULE.parent / relative).read_bytes()).hexdigest(),
                                 "deployment provenance must include nested monitor code and SQL")
            handler = self.serve_once(c)
        response = self.request(handler, "/owner/live-monitor")
        self.assertEqual(response["status"], 404)
        self.assertEqual(response["headers"]["Cache-Control"], "no-store")

    def test_enabled_monitor_runs_and_stops_without_model_work_and_owner_routes_are_private(self):
        with patch.object(w, "Monitor", FakeMonitor):
            c = self.coordinator(LIVE_MONITOR_ENABLED="1", JIAOYI_UI_PASSWORD="owner-fixture")
        self.assertEqual(c.monitor.root, c.store.private / "monitor-18393")
        self.assertEqual(c.status()["live_monitor"]["model_calls"], 0)
        self.assertNotIn("owner-fixture", json.dumps(c.status()))
        handler = self.serve_once(c)
        self.assertTrue(c.monitor.stopped.is_set())
        self.assertEqual(c.store.used(), 0)

        refused = self.request(handler, "/owner/live-monitor")
        self.assertEqual(refused["status"], 401)
        self.assertIn("Basic realm=", refused["headers"]["WWW-Authenticate"])
        for path, content_type in (("/owner/live-monitor", "application/json"),
                                   ("/owner/live-monitor.jsonl", "application/x-ndjson"),
                                   ("/owner/live-monitor/once", "application/json")):
            response = self.request(handler, path, {"Authorization": "Bearer owner-fixture"})
            self.assertEqual(response["status"], 200)
            self.assertEqual(response["headers"]["Content-Type"], content_type)
            self.assertEqual(response["headers"]["Cache-Control"], "no-store")
            self.assertEqual(response["headers"]["Vary"], "Authorization")
            self.assertNotIn("owner-fixture", response["body"].decode())
        with patch.object(c.monitor, "owner_response", side_effect=RuntimeError("owner-fixture")):
            unavailable = self.request(handler, "/owner/live-monitor")
            self.assertEqual(unavailable["status"], 503)
            self.assertNotIn("owner-fixture", unavailable["body"].decode())


if __name__ == "__main__":
    unittest.main()
