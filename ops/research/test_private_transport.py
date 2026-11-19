"""Research transport compatibility; every HTTP operation is a fake."""
import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

import broker_backtest as client
import class_research
import major_fx
import market_collection
import milestone
import oil_replication

PRIVATE = "http://jiaoyi.railway.internal:8080"
SCOPED = {
    "JIAOYI_TRADING_ACCESS_MODE": "scoped", "TRADING_BASE_URL": PRIVATE,
    "JIAOYI_TRADING_PRIVATE_ORIGIN": PRIVATE,
    "JIAOYI_RESEARCH_BEARER_TOKEN": "r" * 32,
    "JIAOYI_AUDIT_BEARER_TOKEN": "a" * 32, "JIAOYI_UI_PASSWORD": "old-owner-credential",
}


class Reply(io.BytesIO):
    status = 200


class StopBeforeCollection(Exception):
    pass


class ResearchPrivateTransportTests(unittest.TestCase):
    def test_private_poll_and_submission_preserve_payload_and_collection_authority(self):
        body = {"account": 1, "symbol": "TEST", "tick_replay": {"volume": 100}}
        for call, route in ((client.call, "/owner/backtests"), (market_collection.call, "/owner/market-data")):
            with self.subTest(route=route), patch.dict(os.environ, SCOPED, clear=True), \
                    patch("rebuild_authority.require_collection_authority") as authority:
                seen = []
                def opened(req, timeout):
                    self.assertEqual(timeout, 25)
                    seen.append(req)
                    return Reply(b'{"id":"fixture","status":"complete"}')
                with patch.object(client.trading_http, "open_request", side_effect=opened):
                    self.assertEqual(call(None, client.research_token())["id"], "fixture")
                    authority.assert_not_called()
                    self.assertEqual(call(None, client.research_token(), body)["status"], "complete")
                authority.assert_called_once_with()
                self.assertEqual([r.get_method() for r in seen], ["GET", "POST"])
                self.assertTrue(all(r.full_url == PRIVATE + route for r in seen))
                self.assertTrue(all(r.get_header("Authorization") == "Bearer " + "r" * 32 for r in seen))
                self.assertIsNone(seen[0].data)
                self.assertEqual(json.loads(seen[1].data), body)

    def test_collection_pause_still_blocks_before_transport_or_origin_handling(self):
        for call in (client.call, market_collection.call):
            with self.subTest(client=call.__module__), \
                    patch("rebuild_authority.require_collection_authority", side_effect=RuntimeError("research_collection_paused")), \
                    patch.object(client.trading_http, "request") as request, \
                    patch.object(client.trading_http, "open_request") as opened:
                with self.assertRaisesRegex(RuntimeError, "research_collection_paused"):
                    call("http://unapproved.invalid", "unused", {})
                request.assert_not_called()
                opened.assert_not_called()

    def test_missing_scoped_secret_cannot_fall_back_to_owner_password(self):
        env = dict(SCOPED)
        del env["JIAOYI_RESEARCH_BEARER_TOKEN"]
        with patch.dict(os.environ, env, clear=True), patch.object(client.trading_http, "open_request") as opened:
            with self.assertRaises(ValueError) as error:
                client.research_token()
            self.assertNotIn(env["JIAOYI_UI_PASSWORD"], str(error.exception))
            opened.assert_not_called()
        with patch.dict(os.environ, {"JIAOYI_UI_PASSWORD": "legacy-token"}, clear=True):
            self.assertEqual(client.research_token(), "legacy-token")
            self.assertEqual(client.origin(), "https://jiaoyi.up.railway.app")

    def test_origin_validation_prevents_credential_urls_and_unpinned_http(self):
        for call in (client.call, market_collection.call):
            for base in ("https://owner:SECRET@example.invalid", "http://jiaoyi.railway.internal:8080",
                         "https://example.invalid/?token=SECRET"):
                with self.subTest(client=call.__module__, base=base), patch.dict(os.environ, {}, clear=True), \
                        patch.object(client.trading_http, "open_request") as opened:
                    with self.assertRaises(ValueError) as error:
                        call(base, "credential")
                    self.assertNotIn("SECRET", str(error.exception))
                    self.assertNotIn("credential", str(error.exception))
                    opened.assert_not_called()

    def test_research_responses_remain_bounded(self):
        class Oversize(Reply):
            def read(self, size):
                self.requested = size
                return b"x" * size
        for call in (client.call, market_collection.call):
            reply = Oversize()
            with self.subTest(client=call.__module__), patch.dict(os.environ, {}, clear=True), \
                    patch.object(client.trading_http, "open_request", return_value=reply):
                with self.assertRaisesRegex(RuntimeError, "32MB"):
                    call("https://example.invalid", "credential")
                self.assertEqual(reply.requested, 32_000_001)

    def test_no_redirect_transport_is_used_for_both_research_endpoints(self):
        for call in (client.call, market_collection.call):
            with self.subTest(client=call.__module__), patch.dict(os.environ, SCOPED, clear=True), \
                    patch.object(client.trading_http.urllib.request, "build_opener") as builder:
                builder.return_value.open.return_value = Reply(b"null")
                self.assertIsNone(call(None, client.research_token()))
                handlers = builder.call_args.args
                redirects = [h for h in handlers if isinstance(h, client.trading_http.NoRedirect)]
                self.assertEqual(len(redirects), 1)
                self.assertIsNone(redirects[0].redirect_request(None, None, 302, "", {}, "https://evil.invalid"))
                proxies = [h for h in handlers if isinstance(h, client.trading_http.urllib.request.ProxyHandler)]
                self.assertEqual(len(proxies), 1)
                self.assertEqual(proxies[0].proxies, {})

    def test_cli_collect_wrappers_use_env_origin_without_changing_their_plans(self):
        for module in (class_research, major_fx, milestone):
            with self.subTest(module=module.__name__), tempfile.TemporaryDirectory() as folder, \
                    patch.dict(os.environ, SCOPED, clear=True), patch.object(module, "collect") as collect:
                args = [module.__name__, "collect", "--output", folder]
                if module is milestone:
                    args += ["--stage", "development"]
                with patch("sys.argv", args):
                    module.main()
                self.assertEqual(collect.call_args.args[0].base, PRIVATE)

    def test_collect_wrappers_select_research_token_not_audit_or_owner(self):
        with tempfile.TemporaryDirectory() as folder, patch.dict(os.environ, SCOPED, clear=True):
            args = types.SimpleNamespace(output=Path(folder), stage="development", base=PRIVATE,
                                         max_new_jobs=1, legacy_fetch=False)
            plan = {"classes": {"metal": ["XAUUSD"]}, "development_dates": ["2024-01-02"],
                    "account": 1, "costs": {"commission_round_trip": 0},
                    "native_volume": {"XAUUSD": 100}, "rules": ["fixture"],
                    "development": ["2024-01-02", "2024-01-02"]}
            for module in (class_research, major_fx, milestone):
                with self.subTest(module=module.__name__), \
                        patch.object(client, "call", side_effect=StopBeforeCollection) as call, \
                        patch.object(major_fx, "requests", return_value=iter([("fixture", {"symbol": "TEST"})])), \
                        contextlib.redirect_stdout(io.StringIO()):
                    with self.assertRaises(StopBeforeCollection):
                        if module is class_research:
                            module.collect(args, plan, "fixture-plan")
                        else:
                            module.collect(args, plan)
                    self.assertEqual(call.call_args.args[1], "r" * 32)

    def test_offline_analysis_does_not_require_transport_configuration_or_credentials(self):
        env = {"JIAOYI_TRADING_ACCESS_MODE": "invalid", "TRADING_BASE_URL": "not-an-origin"}
        for module in (class_research, major_fx, milestone, oil_replication):
            with self.subTest(module=module.__name__), tempfile.TemporaryDirectory() as folder, \
                    patch.dict(os.environ, env, clear=True), patch.object(module, "analyse", return_value={}) as analyse, \
                    patch.object(client, "origin", side_effect=AssertionError("offline origin lookup")), \
                    patch.object(client, "research_token", side_effect=AssertionError("offline credential lookup")):
                args = [module.__name__, "analyse", "--output", folder]
                if module is oil_replication:
                    args += ["--verifier", "/tmp/not-run", "--replay", "/tmp/not-run"]
                else:
                    args += ["--binary", "/tmp/not-run"]
                if module is milestone:
                    args += ["--stage", "development"]
                with patch("sys.argv", args):
                    module.main()
                analyse.assert_called_once()

    def test_oil_collection_resolves_private_origin_and_preserves_active_job_guard(self):
        with tempfile.TemporaryDirectory() as folder, patch.dict(os.environ, SCOPED, clear=True):
            seen = []
            def api(base, password, request=None):
                seen.append((base, password, request))
                return {"status": "running"}
            with self.assertRaisesRegex(ValueError, "another job is active"):
                oil_replication.collect(Path(folder), oil_replication.load_plan(), client.research_token(), api=api)
            self.assertEqual(seen, [(PRIVATE, "r" * 32, None)])


if __name__ == "__main__":
    unittest.main()
