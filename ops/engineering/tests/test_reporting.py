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
    def test_html_escapes_untrusted_reports_and_keeps_release_distinction(self):
        body = r.render(data())
        self.assertIn("&lt;script&gt;", body)
        self.assertNotIn("<script>", body)
        self.assertIn("unreleased candidates", body)
        self.assertIn("4.5×", body)
        self.assertIn("zero LLM calls", body)
        self.assertIn("27 Oct 2026 05:49 SGT", body)

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
