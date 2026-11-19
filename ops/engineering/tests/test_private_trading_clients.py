"""Private-access client integration, using fake transport and local fixtures only."""
import datetime as dt
import io
import hashlib
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import trading_http
import source_snapshots
import stop_loss_reviews
import market_validation
import incidents
from monitoring import sources
from test_component_release import Fixture, w

PRIVATE = "http://trading.railway.internal:8080"
ENV = {"TRADING_BASE_URL": PRIVATE, "JIAOYI_TRADING_PRIVATE_ORIGIN": PRIVATE,
       "JIAOYI_TRADING_ACCESS_MODE": "scoped", "JIAOYI_AUDIT_BEARER_TOKEN": "A" * 40,
       "JIAOYI_UI_PASSWORD": "OLD_OWNER_MUST_NOT_BE_USED"}
HEALTH = {"status": "ok", "broker_accounts": [], "trading": [], "strategy3": []}
NOW = dt.datetime(2026, 11, 19, 10, tzinfo=dt.timezone.utc)


class Reply(io.BytesIO):
    status = 200


class PrivateClientTests(unittest.TestCase):
    def setUp(self):
        self.requests = []
        self.bodies = {"/health": HEALTH, "/operations/reports": {"reports": []},
                       "/owner/setup-assessments": {"accounts": []},
                       "/operations/stop-loss-reviews": {"schema_version": 1,
                           "as_of": NOW.isoformat(), "cases": []},
                       "/operations/trailing-receipts": {"receipts": []}}
        self.transport = patch.object(trading_http, "open_request", side_effect=self.opened)
        self.transport.start()
        self.addCleanup(self.transport.stop)

    def opened(self, request, timeout):
        self.assertTrue(request.full_url.startswith(PRIVATE + "/"))
        self.assertEqual(request.get_header("Authorization"), "Bearer " + ENV["JIAOYI_AUDIT_BEARER_TOKEN"])
        self.assertEqual(request.get_method(), "GET")
        self.assertTrue(request._jiaoyi_private_transport)
        self.assertLessEqual(timeout, 20)
        self.requests.append(request)
        route = request.full_url.removeprefix(PRIVATE)
        return Reply(json.dumps(self.bodies[route]).encode())

    def test_persisted_task_identities_are_unchanged_but_prompts_use_supplied_evidence(self):
        expected = {
            "DEFAULT_TASK": "dfb634b7883b71af7d1e746867703748ae52ff12396bfecac0da7c2cd8144a5b",
            "ACCEPTANCE_TASK": "854fc72f40bbe07c0a447ab38f8dd7fa2e7cfdeeb16bd21d260bb90d618c85c6",
        }
        for name, digest in expected.items():
            task = getattr(w, name)
            self.assertEqual(hashlib.sha256(task.encode()).hexdigest(), digest)
            prompt = w.model_task(task)
            self.assertNotIn("https://jiaoyi.up.railway.app", prompt)
            self.assertIn("coordinator-provided", prompt)
        self.assertTrue(w.Coordinator.read_only_job({"task": w.ACCEPTANCE_TASK, "state": "queued"}))
        for state in ("queued", "repair_queued"):
            job = {"task": w.DEFAULT_TASK, "state": state}
            self.assertFalse(w.Coordinator.read_only_job(job))
            self.assertIn("coordinator-provided", w.model_task(job["task"]))
        self.assertEqual(w.model_task("custom owner instruction"), "custom owner instruction")

    def test_snapshot_and_monitor_use_scoped_token_and_preserve_payloads(self):
        health, operations, receipts = source_snapshots.fetch(env=ENV)
        self.assertEqual(health, HEALTH)
        self.assertEqual(operations, {"reports": []})
        self.assertTrue(all(r["ok"] for r in receipts.values()))
        result = sources.collect_sources(ENV, NOW - dt.timedelta(hours=2), NOW)
        self.assertEqual(result["health"], HEALTH)
        self.assertEqual(result["operations_reports"], {"reports": []})
        self.assertEqual(result["setup_assessments"], {"accounts": []})
        self.assertEqual(len(self.requests), 5)
        self.assertNotIn(ENV["JIAOYI_AUDIT_BEARER_TOKEN"], json.dumps(result))
        self.assertNotIn(ENV["JIAOYI_UI_PASSWORD"], json.dumps(receipts))

    def test_scoped_missing_audit_secret_never_falls_back_or_opens_transport(self):
        env = {k: v for k, v in ENV.items() if k != "JIAOYI_AUDIT_BEARER_TOKEN"}
        health, operations, receipts = source_snapshots.fetch(env=env)
        result = sources.collect_sources(env, NOW, NOW)
        self.assertIsNone(health)
        self.assertIsNone(operations)
        self.assertTrue(all(not r["ok"] for r in receipts.values()))
        self.assertTrue(all(result[name] is None for name in sources.ROUTES))
        self.assertEqual(self.requests, [])

    def test_minimal_liveness_is_not_account_evidence(self):
        self.bodies["/health"] = {"status": "ok"}
        health, operations, receipts = source_snapshots.fetch(env=ENV)
        self.assertIsNone(health)
        self.assertEqual(operations, {"reports": []})
        self.assertEqual(receipts["/health"]["error_code"], "invalid_shape")
        result = sources.collect_sources(ENV, NOW, NOW)
        self.assertIsNone(result["health"])
        self.assertEqual(result["source_status"]["health"]["error_code"], "invalid_shape")

    def test_market_and_stop_reviews_use_coordinator_origin_and_credential(self):
        with tempfile.TemporaryDirectory() as root:
            c = SimpleNamespace(config=SimpleNamespace(env=ENV), store=w.Store(root))
            validator = market_validation.MarketValidation(c)
            self.assertEqual(validator._fetch("/health"), HEALTH)
            result = stop_loss_reviews.poll(c, at=NOW)
            self.assertEqual(result["queued_job_ids"], [])
        self.assertEqual(len(self.requests), 2)

    def test_incident_fetch_uses_coordinator_environment_without_hardcoded_origin(self):
        manager = incidents.IncidentManager.__new__(incidents.IncidentManager)
        manager.c = SimpleNamespace(config=SimpleNamespace(env=ENV))
        manager.observe = Mock(return_value={"observed": True})
        self.assertEqual(manager.poll(), {"observed": True})
        manager.observe.assert_called_once()
        self.assertEqual(manager.observe.call_args.args, (HEALTH, {"reports": []}))
        self.assertEqual(len(self.requests), 2)

    def test_automatic_model_cache_reuses_two_reads_and_is_bounded_to_job_and_age(self):
        store = Mock()
        store.enqueue_changed_evidence.return_value = 77
        c = SimpleNamespace(config=SimpleNamespace(env=ENV), store=store,
                            bundle_digest="fixture", _emit_watch_events=Mock(), public_time=w.Coordinator.public_time,
                            _automatic_evidence=None)
        self.assertEqual(w.Coordinator.enqueue_automatic_assessment(c), 77)
        cached = w.Coordinator.automatic_snapshots(c, {"id": 77})
        self.assertEqual(len(cached), 2)
        self.assertEqual(json.loads(cached[0])["data"], HEALTH)
        self.assertEqual(len(self.requests), 2)
        self.assertNotIn(ENV["JIAOYI_AUDIT_BEARER_TOKEN"], "".join(cached))
        with self.assertRaisesRegex(ValueError, "automatic_evidence_unavailable_or_stale"):
            w.Coordinator.automatic_snapshots(c, {"id": 78})
        c._automatic_evidence["captured_at"] -= 301
        with self.assertRaisesRegex(ValueError, "automatic_evidence_unavailable_or_stale"):
            w.Coordinator.automatic_snapshots(c, {"id": 77})
        self.assertEqual(len(self.requests), 2, "cache freshness checks never add requests")


class PrivateReleaseTests(Fixture):
    def test_release_liveness_uses_private_origin_without_read_authority(self):
        self.c.config.env.update(ENV)
        self.fixture(runtime=True)
        self.active = [self.deployment()]
        self.latest = self.active[0]
        seen = []
        def opened(request, timeout):
            seen.append(request)
            self.assertEqual(request.full_url, PRIVATE + "/health")
            self.assertIsNone(request.get_header("Authorization"))
            self.assertEqual(timeout, 20)
            return Reply(b'{"status":"ok"}')
        with patch.object(trading_http, "open_request", side_effect=opened):
            self.c.release(self.job())
        self.assertEqual(len(seen), 1)
        self.assertEqual(self.job()["state"], "verification_queued")


if __name__ == "__main__":
    unittest.main()
