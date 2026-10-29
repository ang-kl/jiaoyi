import io
import json
import time
import unittest
import urllib.error
from unittest.mock import patch

from test_worker import Fixture, w
import reporting as r


def data():
    return {"since": "2026-10-26T15:49:18+00:00", "until": "2026-10-26T21:49:18+00:00",
            "state": "idle", "sessions": 5, "limit": 18,
            "commits": [{"sha": "abc123", "subject": "gate <check>"}],
            "jobs": [{"id": 3, "state": "failed", "summary": "<script>no()</script>", "tests": "381 reported; CI pending"}]}


class ReportTests(Fixture, unittest.TestCase):
    def test_source_incident_names_service_and_safe_per_route_receipts(self):
        incident = {"id": "source-1", "kind": "sources_unavailable", "severity": "critical",
                    "account_id": None, "strategy": None, "symbol": None,
                    "summary": "Source <unavailable>", "source_receipts": {
                        "/health": {"route": "https://private.invalid/?token=PRIVATE_ROUTE_SECRET",
                            "ok": False, "http_status": 503, "error_code": "http",
                            "observed_at": "2026-10-29T20:13:13Z", "last_success_at": "2026-10-29T20:12:03Z",
                            "elapsed_ms": 8123, "error_body": "PRIVATE_ERROR_BODY"},
                        "/operations/reports": {"route": "/operations/reports", "ok": True,
                            "http_status": 200, "error_code": "<script>PRIVATE_ERROR_SECRET</script>",
                            "observed_at": "2026-10-29T20:13:14Z", "last_success_at": "2026-10-29T20:13:14Z",
                            "elapsed_ms": 74},
                        "https://private.invalid/PRIVATE_EXTRA_ROUTE": {"ok": False},
                    }}
        body = r.render_incident(incident, "2026-10-29T20:13:15Z")
        self.assertIn("Service: jiaoyi public snapshots", body)
        self.assertNotIn("Account None", body)
        for text in ("Source fetch receipts", "/health", "/operations/reports", "failed", "available",
                     "HTTP 503", "http", "30 Oct 2026 04:13:13 SGT", "30 Oct 2026 04:12:03 SGT", "8123ms"):
            self.assertIn(text, body)
        self.assertIn("Source &lt;unavailable&gt;", body)
        for text in ("PRIVATE_ROUTE_SECRET", "PRIVATE_ERROR_BODY", "PRIVATE_ERROR_SECRET", "PRIVATE_EXTRA_ROUTE", "<script>"):
            self.assertNotIn(text, body)
        ordinary = r.render_incident({"id": "position-1", "kind": "protection_unverified",
                                      "account_id": 49236051, "strategy": "s3", "symbol": "BTCUSD"})
        self.assertIn("Account 49236051", ordinary)
        self.assertNotIn("Source fetch receipts", ordinary)

    def test_html_escapes_untrusted_reports_and_keeps_release_distinction(self):
        body = r.render(data())
        self.assertIn("&lt;script&gt;", body)
        self.assertNotIn("<script>", body)
        self.assertIn("unreleased candidates", body)
        self.assertIn("4.5×", body)
        self.assertIn("zero LLM calls", body)
        self.assertIn("27 Oct 2026 05:49 SGT", body)
        self.assertIn("№E-20261027-054918", body)
        self.assertIn("[§E-20261027-054918.1]", body)

    def test_digest_omits_individual_stop_loss_findings_but_keeps_aggregate_and_engineering(self):
        c = w.Coordinator(self.config(TELEGRAM_BOT_TOKEN="private-test-token", TELEGRAM_OWNER_CHAT_ID="313940231"))
        ordinary = c.store.enqueue("authorised engineering repair")
        c.store.update(ordinary, "completed", result=json.dumps({"summary": "ENGINEERING-RESULT", "tests": "engineering assertion"}))
        for _ in range(105):
            jid = c.store.enqueue(w.stop_loss_reviews.PREFIX + " ORIGINAL SETUP REVIEW")
            c.store.update(jid, "completed", result=json.dumps({"summary": "PRIVATE-SL-SUMMARY", "tests": "PRIVATE-SL-CHECK"}))
        # Filtering must happen before the job cap; reviews cannot crowd out engineering.
        report = r.collect(c)
        self.assertEqual([j["id"] for j in report["jobs"]], [ordinary])
        self.assertEqual(report["internal_review_count"], 105)
        body = r.render(report)
        self.assertIn("ENGINEERING-RESULT", body)
        self.assertIn("105", body)
        self.assertNotIn("PRIVATE-SL-SUMMARY", body)
        self.assertNotIn("PRIVATE-SL-CHECK", body)
        self.assertEqual(c.store.used(), 0)

    def test_digest_groups_queue_housekeeping_but_retains_material_attempts_and_receipts(self):
        report = data()
        housekeeping = [{"id": i, "state": "superseded", "attempts": 0,
                         "summary": "No preserved model summary; state is superseded",
                         "tests": "No independently verified test receipt"}
                        for i in range(170, 208)]
        report["jobs"] = [
            {"id": 148, "state": "completed", "attempts": 1,
             "summary": "Observed source failure remains unresolved", "tests": "Read-only source inspection; no behavioral verification"},
            {"id": 149, "state": "failed", "attempts": 1,
             "summary": "Repair failed", "diagnostic": "CI check failed", "tests": "No independently verified test receipt"},
            {"id": 150, "state": "ci_wait", "attempts": 1,
             "summary": "Candidate waiting for CI", "tests": "Regression passed; deployment pending"},
            {"id": 151, "state": "superseded", "attempts": 1,
             "summary": "Attempt with preserved evidence", "tests": "A real attempted validation"},
            *housekeeping,
        ]
        body = r.render(report)
        self.assertEqual(body.count("38 unattempted superseded queue records"), 1)
        self.assertNotIn("No preserved model summary; state is superseded", body)
        self.assertNotIn("No independently verified test receipt", body)
        self.assertNotIn("Job 170", body)
        for job in report["jobs"][:4]:
            self.assertIn("<td>" + str(job["id"]) + "</td>", body)
            self.assertIn(job["summary"], body)
        self.assertIn("CI check failed", body)
        self.assertIn("Read-only source inspection; no behavioral verification", body)
        self.assertIn("Regression passed; deployment pending", body)
        self.assertIn("A real attempted validation", body)
        self.assertEqual(body.count("— reported validation"), 3)

    def test_real_multipart_format_and_positive_telegram_receipt(self):
        class Reply(io.BytesIO):
            pass
        def post(request, timeout):
            self.assertEqual(request.get_method(), "POST")
            self.assertEqual(timeout, 25)
            self.assertIn(b'name="chat_id"', request.data)
            self.assertIn(b'313940231', request.data)
            self.assertIn(b'filename="report.html"', request.data)
            self.assertIn(b'Content-Type: text/html', request.data)
            self.assertIn("λ".encode(), request.data)
            return Reply(b'{"ok":true,"result":{"message_id":42}}')
        with patch.object(r.urllib.request, "urlopen", side_effect=post):
            self.assertEqual(r.send_document("private-test-token", "313940231", "report.html", "<html>λ</html>", "report"), 42)

    def test_token_url_never_appears_in_errors_and_missing_ack_is_uncertain(self):
        error = urllib.error.HTTPError("https://api.telegram.org/botprivate-test-token/sendDocument", 401, "private-test-token", {}, None)
        with patch.object(r.urllib.request, "urlopen", side_effect=error):
            with self.assertRaises(r.DeliveryError) as caught:
                r.send_document("private-test-token", "313940231", "a.html", "html", "report")
        self.assertNotIn("private-test-token", str(caught.exception))
        self.assertFalse(caught.exception.uncertain)
        with patch.object(r.urllib.request, "urlopen", return_value=io.BytesIO(b'{"ok":true,"result":{}}')):
            with self.assertRaises(r.DeliveryError) as caught:
                r.send_document("private-test-token", "313940231", "a.html", "html", "report")
        self.assertTrue(caught.exception.uncertain)

    def test_durable_receipt_dedup_and_uncertain_send_hold(self):
        c = w.Coordinator(self.config(TELEGRAM_BOT_TOKEN="private-test-token", TELEGRAM_OWNER_CHAT_ID="313940231"))
        reporter = c.reporter
        with self.store.connect() as db:
            db.execute("UPDATE engineering_report_delivery SET next_due=0")
        with patch.object(r, "collect", return_value=data()), patch.object(r, "send_document", return_value=42) as post:
            reporter.tick()
            reporter.tick()
        self.assertEqual(post.call_count, 1)
        self.assertEqual(reporter.status()["delivery"]["message_id"], 42)
        with self.store.connect() as db:
            db.execute("UPDATE engineering_report_delivery SET next_due=0")
        with patch.object(r, "collect", return_value=data()), patch.object(r, "send_document", side_effect=r.DeliveryError("unknown", uncertain=True)) as post:
            reporter.tick()
            w.Coordinator(c.config).reporter.tick()
        self.assertEqual(post.call_count, 1)
        self.assertEqual(reporter.status()["delivery"]["state"], "uncertain")

    def test_failed_post_does_not_stamp_success_and_tokens_stay_out_of_model(self):
        c = w.Coordinator(self.config(TELEGRAM_BOT_TOKEN="private-test-token", TELEGRAM_OWNER_CHAT_ID="313940231"))
        with self.store.connect() as db:
            db.execute("UPDATE engineering_report_delivery SET next_due=0")
        with patch.object(r, "collect", return_value=data()), patch.object(r, "send_document", side_effect=r.DeliveryError("telegram_http_400")):
            c.reporter.tick()
        state = c.reporter.status()["delivery"]
        self.assertIsNone(state["message_id"])
        self.assertEqual(state["state"], "retry")
        env = w.clean_environment(c.config, self.root, self.root, 10001)
        self.assertNotIn("TELEGRAM_BOT_TOKEN", env)
        self.assertNotIn("TELEGRAM_OWNER_CHAT_ID", env)
