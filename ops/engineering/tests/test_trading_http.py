"""Offline transport/authority boundary checks; never contact Trading/brokers."""
import io
from pathlib import Path
import sys
import unittest
import urllib.request
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import trading_http as http


class TradingHttpTests(unittest.TestCase):
    def test_legacy_defaults_preserve_existing_credential_boundaries(self):
        env = {"JIAOYI_UI_PASSWORD": "legacy password"}
        public = http.request("/health", env=env)
        self.assertEqual(public.full_url, http.DEFAULT_ORIGIN + "/health")
        self.assertIsNone(public.get_header("Authorization"))
        private = http.request("/owner/setup-assessments", env=env)
        self.assertEqual(private.get_header("Authorization"), "Bearer legacy password")
        self.assertEqual(http.research_token(env), "legacy password")

    def test_env_override_and_explicit_override_do_not_contact_a_default_origin(self):
        env = {"TRADING_BASE_URL": "https://approved.invalid:8443/"}
        self.assertEqual(http.origin(env=env), "https://approved.invalid:8443")
        self.assertEqual(http.origin("https://explicit.invalid", env), "https://explicit.invalid")
        with self.assertRaisesRegex(ValueError, "conflicting_trading_origins"):
            http.origin(env={**env, "JIAOYI_BASE_URL": "https://different.invalid"})

    def test_private_http_requires_exact_separately_pinned_origin_and_port(self):
        value = "http://trading.railway.internal:8080"
        env = {"TRADING_BASE_URL": value, "JIAOYI_TRADING_PRIVATE_ORIGIN": value}
        self.assertEqual(http.origin(env=env), value)
        for candidate, pin in [(value, None), (value, "http://other.railway.internal:8080"),
                               ("http://example.com:8080", "http://example.com:8080"),
                               ("http://trading.railway.internal", "http://trading.railway.internal"),
                               ("http://trading.railway.internal.evil:8080", "http://trading.railway.internal.evil:8080")]:
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                http.origin(candidate, {"JIAOYI_TRADING_PRIVATE_ORIGIN": pin})

    def test_origin_rejects_secret_sink_components(self):
        bad = ["https://owner:SECRET@example.invalid", "https://example.invalid/path",
               "https://example.invalid?token=SECRET", "https://example.invalid#x",
               "https://example.invalid?", "https://example.invalid#",
               "https://example.invalid:0", "https://example.invalid:65536",
               "https://example.invalid\n", "https://example.invalid\\@evil.invalid",
               "https://%65xample.invalid", "", "ftp://example.invalid"]
        for value in bad:
            with self.subTest(value=value), self.assertRaises(ValueError) as raised:
                http.origin(value, {})
            self.assertNotIn("SECRET", str(raised.exception))

    def test_scoped_mode_uses_only_the_selected_role_credential(self):
        env = {"JIAOYI_TRADING_ACCESS_MODE": "scoped", "JIAOYI_UI_PASSWORD": "legacy",
               "JIAOYI_AUDIT_BEARER_TOKEN": "a" * 32,
               "JIAOYI_RESEARCH_BEARER_TOKEN": "r" * 32}
        for path in http.AUDIT_PATHS:
            self.assertEqual(http.request(path, env=env).get_header("Authorization"), "Bearer " + "a" * 32)
        for path in http.RESEARCH_PATHS:
            for payload in (None, b"{}"):
                req = http.request(path, purpose="research", env=env, data=payload)
                self.assertEqual(req.get_header("Authorization"), "Bearer " + "r" * 32)
                self.assertEqual(req.get_method(), "GET" if payload is None else "POST")
        self.assertIsNone(http.request("/health", purpose="liveness", env=env).get_header("Authorization"))

    def test_no_fallback_to_legacy_when_scoped_credentials_missing_or_mode_invalid(self):
        env = {"JIAOYI_TRADING_ACCESS_MODE": "scoped", "JIAOYI_UI_PASSWORD": "legacy"}
        for action in [lambda: http.request("/health", env=env), lambda: http.research_token(env)]:
            with self.assertRaises(ValueError): action()
        with self.assertRaisesRegex(ValueError, "invalid_trading_access_mode"):
            http.request("/health", env={"JIAOYI_TRADING_ACCESS_MODE": "typo"})

    def test_audit_cannot_submit_or_access_research_mutation_or_monitor_once(self):
        for route, purpose, payload in [("/owner/backtests", "audit", None),
                                         ("/owner/market-data", "audit", b"{}"),
                                         ("/health", "audit", b"{}"),
                                         ("/owner/live-monitor/once", "audit", None),
                                         ("/health", "research", None),
                                         ("/operations/reports", "liveness", None)]:
            with self.subTest(route=route, purpose=purpose), self.assertRaises(ValueError):
                http.request(route, purpose=purpose, data=payload, env={})

    def test_route_cannot_escape_the_selected_origin(self):
        for route in ["//evil.invalid/health", "https://evil.invalid/health",
                      "/health#fragment", "/health\n", "/health/../owner/backtests",
                      "/%68ealth", "/health\\x"]:
            with self.subTest(route=route), self.assertRaises(ValueError):
                http.request(route, env={})

    def test_header_injection_is_rejected_before_network(self):
        for secret in ["a\r\nX-Evil: x", "a\x00", "a\t", "a" * 4097]:
            with self.subTest(secret=repr(secret[:10])), self.assertRaises(ValueError):
                http.request("/owner/backtests", purpose="research", token=secret, env={})

    def test_private_transport_never_uses_an_ambient_proxy_and_redirect_is_refused(self):
        value = "http://trading.railway.internal:8080"
        req = http.request("/health", env={"TRADING_BASE_URL": value,
                                           "JIAOYI_TRADING_PRIVATE_ORIGIN": value})
        with patch.object(http.urllib.request, "build_opener") as build:
            http.open_request(req, 3)
        handlers = build.call_args.args
        self.assertTrue(any(isinstance(h, http.NoRedirect) for h in handlers))
        proxies = [h for h in handlers if isinstance(h, urllib.request.ProxyHandler)]
        self.assertEqual(len(proxies), 1)
        self.assertEqual(proxies[0].proxies, {})
        self.assertIsNone(http.NoRedirect().redirect_request(req, None, 302, "", {}, "https://evil.invalid"))
        build.return_value.open.assert_called_once_with(req, timeout=3)

    def test_unvalidated_request_cannot_use_the_transport(self):
        with patch.object(http.urllib.request, "build_opener") as build:
            with self.assertRaises(ValueError):
                http.open_request(urllib.request.Request("https://evil.invalid"), 3)
            build.assert_not_called()

    def test_liveness_json_is_not_account_evidence(self):
        for route in ("/health", "/operations/reports", "/owner/setup-assessments"):
            for value in ({"status": "ok"}, {}, [], {"reports": "wrong"}):
                with self.subTest(route=route, value=value), self.assertRaises(ValueError):
                    http.ensure_diagnostic(route, value)
        fixtures = {
            "/health": {"broker_accounts": [], "trading": [], "strategy3": []},
            "/operations/reports": {"reports": []},
            "/owner/setup-assessments": {"accounts": []},
        }
        for route, value in fixtures.items():
            self.assertIs(http.ensure_diagnostic(route, value), value)


if __name__ == "__main__":
    unittest.main()
