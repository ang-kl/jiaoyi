"""Owner-requested incident mute preserves observation and scheduled reporting."""
import unittest
from pathlib import Path
from unittest.mock import patch
from test_worker import Fixture, w
from test_reporting import data
import reporting as r

class IncidentRoutingTests(Fixture, unittest.TestCase):
    def coordinator(self, enabled="0"):
        return w.Coordinator(self.config(TELEGRAM_BOT_TOKEN="test-token", TELEGRAM_OWNER_CHAT_ID="313940231",
            ENGINEERING_INCIDENT_TELEGRAM_ENABLED=enabled))

    def incident(self, **extra):
        return {"id":"routing-test", "version":1, "kind":"protection_unverified",
                "severity":"critical", "summary":"Protection needs review", **extra}

    def test_muted_incident_is_saved_but_scheduled_report_still_sends(self):
        c=self.coordinator()
        with patch.object(r, "send_document", return_value=42) as send:
            receipt=c.reporter.publish_incident(self.incident())
            self.assertEqual(receipt["state"], "created")
            self.assertEqual(receipt["reason"], "incident_telegram_disabled")
            self.assertFalse(receipt["notification_pending"])
            self.assertTrue(Path(receipt["path"]).is_file())
            c.reporter._send_incident(receipt)
            send.assert_not_called()
            with c.store.connect() as db:
                db.execute("UPDATE engineering_report_delivery SET next_due=0")
            with patch.object(r, "collect", return_value=data()):
                c.reporter.tick()
            self.assertEqual(send.call_count, 1)
        status=c.reporter.status()
        self.assertTrue(status["configured"])
        self.assertFalse(status["incident_telegram_enabled"])
        self.assertFalse(status["chat_push_configured"])
        self.assertEqual(c.store.used(), 0)

    def test_preexisting_pending_delivery_is_suppressed_across_restart(self):
        c=self.coordinator("1")
        with patch.object(r, "send_document") as send:
            queued=c.reporter.publish_incident(self.incident(defer_delivery=True))
            self.assertTrue(queued["notification_pending"])
            muted=self.coordinator()
            receipt=muted.reporter.incident_delivery_tick()
            self.assertEqual(receipt["key"], queued["key"])
            self.assertEqual(receipt["reason"], "incident_telegram_disabled")
            self.assertFalse(receipt["notification_pending"])
            restarted=self.coordinator("1")
            self.assertIsNone(restarted.reporter.incident_delivery_tick())
            self.assertFalse(restarted.reporter.publish_incident(self.incident(defer_delivery=True))["notification_pending"])
            send.assert_not_called()

    def test_watch_is_recorded_without_telegram_alert(self):
        c=self.coordinator()
        with patch.object(w.watches, "events", return_value=([{"event":"limit_reached","detail":{}}], {})), \
             patch.object(w.watches, "watches", return_value={"observed":True}), \
             patch.object(w, "send_message") as send:
            c._emit_watch_events({}, {})
            send.assert_not_called()
        self.assertIsNotNone(c.store.last_event("watch_state"))
        self.assertIn("observed", c.store.last_event("watch_reading"))
        self.assertEqual(c.store.last_event("watch_telegram_suppressed"), "limit_reached")
