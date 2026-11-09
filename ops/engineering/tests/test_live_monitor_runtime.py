import datetime as dt
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from monitoring.runtime import Monitor, UTC


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.env = {"LIVE_MONITOR_ENABLED": "1", "JIAOYI_UI_PASSWORD": "test-owner-secret"}
        self.start = dt.datetime(2026, 11, 9, 3, tzinfo=UTC)

    def monitor(self):
        return Monitor(self.env, Path(self.temp.name))

    @staticmethod
    def capture(plan, at, index, previous, **kwargs):
        return {"trace": "№18,393", "run_id": "18393-20261109", "checkpoint_number": index,
                "timestamp_sgt": at.isoformat(), "system_health": {"max_dispatch_latency_ms": None}}

    def test_twelve_checkpoints_restart_dedup_and_separate_terminal_observation(self):
        with patch.object(Monitor, "_capture", side_effect=self.capture) as fetch:
            for i in range(12):
                monitor = self.monitor()
                self.assertEqual(len(monitor.tick(self.start + dt.timedelta(hours=2 * i))), 1)
                self.assertEqual(monitor.tick(self.start + dt.timedelta(hours=2 * i)), [])
            monitor = self.monitor()
            terminal = monitor.tick(self.start + dt.timedelta(hours=24))
            self.assertEqual(terminal[0]["record_kind"], "completion")
            self.assertEqual(monitor.tick(self.start + dt.timedelta(hours=26)), [])
            self.assertEqual(len(monitor._records()), 12)
            self.assertEqual(fetch.call_count, 13)
            self.assertEqual(monitor.status()["state"], "completed")

    def test_missed_intervals_are_not_backfilled_with_current_readings(self):
        with patch.object(Monitor, "_capture", side_effect=self.capture) as fetch:
            monitor = self.monitor()
            monitor.tick(self.start)
            rows = monitor.tick(self.start + dt.timedelta(hours=7))
            self.assertEqual([r.get("status") for r in rows],
                             ["missed_while_monitor_unavailable", "missed_while_monitor_unavailable", None])
            self.assertIsNone(rows[0]["observed_at"])
            self.assertEqual(fetch.call_count, 2)
            self.assertEqual(rows[-1]["schedule_lateness_seconds"], 3600)

    def test_private_routes_fail_closed_and_read_does_not_advance_schedule(self):
        monitor = self.monitor()
        with patch.object(Monitor, "_capture", side_effect=self.capture):
            monitor.tick(self.start)
        for route in ("/owner/live-monitor", "/owner/live-monitor.jsonl", "/owner/live-monitor/once"):
            self.assertEqual(monitor.owner_response(route, {})[0], 401)
        status, mime, body = monitor.owner_response("/owner/live-monitor.jsonl",
                                                  {"Authorization": "Bearer test-owner-secret"})
        self.assertEqual(status, 200)
        self.assertEqual(len(body.splitlines()), 1)
        self.assertEqual(len(monitor._records()), 1)
        self.assertNotIn("test-owner-secret", json.dumps(monitor.status()))
        monitor.env.pop("JIAOYI_UI_PASSWORD")
        self.assertEqual(monitor.owner_response("/owner/live-monitor", {})[0], 503)

    def test_schedule_change_cannot_restart_a_persisted_run(self):
        with patch.object(Monitor, "_capture", side_effect=self.capture):
            self.monitor().tick(self.start)
        changed = Monitor({**self.env, "LIVE_MONITOR_HOURS": "48"}, Path(self.temp.name))
        with self.assertRaises(ValueError):
            changed.tick(self.start)
