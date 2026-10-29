"""The source adapter preserves good evidence while bounding/sanitizing failures."""
import datetime as dt
import importlib.util
import io
import json
from pathlib import Path
import threading
import unittest
import urllib.error
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "source_snapshots.py"
spec = importlib.util.spec_from_file_location("source_snapshot_tests", MODULE)
snapshots = importlib.util.module_from_spec(spec)
spec.loader.exec_module(snapshots)


class Reply(io.BytesIO):
    status = 200


class SourceSnapshotsTests(unittest.TestCase):
    def test_independent_reads_preserve_operations_while_health_times_out(self):
        # A serial fetch would deadlock here: health cannot finish until the
        # operations request is made, and that successful response must survive.
        operations_started = threading.Event()

        def open_source(request, timeout):
            if request.full_url.endswith("/health"):
                self.assertTrue(operations_started.wait(2), "source reads must run independently")
                raise TimeoutError("secret URL and response text must not escape")
            operations_started.set()
            return Reply(b'{"reports":[],"as_of":"2026-10-29T20:00:00Z"}')

        with patch.object(snapshots.urllib.request, "urlopen", side_effect=open_source):
            health, operations, receipts = snapshots.fetch("https://user:SECRET@example.invalid")
        self.assertIsNone(health)
        self.assertEqual(operations["reports"], [])
        self.assertEqual(receipts["/health"]["error_code"], "timeout")
        self.assertEqual(receipts["/operations/reports"]["http_status"], 200)
        self.assertTrue(receipts["/operations/reports"]["ok"])
        self.assertNotIn("SECRET", json.dumps(receipts))
        for route, receipt in receipts.items():
            self.assertEqual(receipt["route"], route)
            self.assertIsNotNone(dt.datetime.fromisoformat(receipt["observed_at"]).tzinfo)
            self.assertGreaterEqual(receipt["elapsed_ms"], 0)

    def test_fixed_failure_categories_discard_remote_error_content(self):
        failures = [
            (urllib.error.HTTPError("https://SECRET", 503, "SECRET body", {}, io.BytesIO(b"SECRET")), "http", 503),
            (urllib.error.URLError(TimeoutError("SECRET")), "timeout", None),
            (urllib.error.URLError("SECRET transport"), "transport", None),
            (Reply(b"SECRET malformed body"), "invalid_json", 200),
            (Reply(b"[]"), "invalid_shape", 200),
        ]
        for bad, category, status in failures:
            with self.subTest(category=category):
                def open_source(request, timeout):
                    if request.full_url.endswith("/health"):
                        if isinstance(bad, Exception):
                            raise bad
                        return bad
                    return Reply(b'{"reports":[]}')
                with patch.object(snapshots.urllib.request, "urlopen", side_effect=open_source):
                    health, operations, receipts = snapshots.fetch("https://example.invalid")
                self.assertIsNone(health)
                self.assertEqual(operations, {"reports": []})
                self.assertEqual(receipts["/health"]["error_code"], category)
                self.assertEqual(receipts["/health"]["http_status"], status)
                self.assertNotIn("SECRET", json.dumps(receipts))

    def test_oversized_body_is_read_only_to_limit_plus_one(self):
        sizes = []

        class BoundedReply(Reply):
            def read(self, size=-1):
                sizes.append(size)
                return super().read(size)

        def open_source(request, timeout):
            return BoundedReply(b"x" * 100) if request.full_url.endswith("/health") else Reply(b"{}")

        with patch.object(snapshots.urllib.request, "urlopen", side_effect=open_source):
            health, operations, receipts = snapshots.fetch("https://example.invalid", maxbytes=16)
        self.assertIsNone(health)
        self.assertEqual(operations, {})
        self.assertEqual(sizes, [17])
        self.assertEqual(receipts["/health"]["error_code"], "oversize")
        self.assertEqual(receipts["/health"]["bytes"], 17)


if __name__ == "__main__":
    unittest.main()
