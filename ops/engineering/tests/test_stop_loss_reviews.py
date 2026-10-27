"""No inference or remote service: exercise loss-review queue and privacy gates."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / file)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result
w = module("sl_test_worker", "worker.py")
sl = module("sl_reviews", "stop_loss_reviews.py")
NOW = "2026-10-27T02:00:00+00:00"


def case(account=1, deal=10, close="2026-10-27T01:59:00Z", **extra):
    return {"reason": "stop_loss", "ownership_verified": True, "account_id": account,
            "deal_id": deal, "position_id": 100 + deal, "symbol_id": 7, "strategy": "s3",
            "environment": "demo" if account == 1 else "live", "closed_at": close,
            "net_costs_complete": False, "net_r": None,
            "original_entry": {"recorded_at": "2026-10-27T01:00:00Z", "side": "buy", "entry_rule_passed": True}, **extra}


class StopLossReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.c = types.SimpleNamespace(store=w.Store(self.temp.name))

    def tearDown(self):
        self.temp.cleanup()

    def poll(self, cases, at=NOW, **kwargs):
        data = {"schema_version": 1, "as_of": at, "cases": cases}
        class Response:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, limit): return json.dumps(data).encode()[:limit]
        with patch.object(sl.urllib.request, "urlopen", return_value=Response()):
            return sl.poll(self.c, at=at, **kwargs)

    def test_startup_only_seeds_recent_episode_and_dedup_survives_restart(self):
        historical = case(deal=8, close="2026-10-26T01:59:00Z")
        older_recent = case(deal=9, close="2026-10-27T01:40:00Z")
        latest = case()
        result = self.poll([historical, older_recent, latest])
        self.assertEqual(len(result["queued_job_ids"]), 1)
        self.assertIn('"deal_id":10', self.c.store.jobs()[0]["task"])
        self.assertNotIn('"deal_id":8', self.c.store.jobs()[0]["task"])
        self.c.store = w.Store(self.temp.name)
        with self.c.store.connect() as db:
            rows = db.execute("SELECT COUNT(*) FROM events").fetchone()[0]
        self.assertEqual(self.poll([historical, older_recent, latest])["queued_job_ids"], [])
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM events").fetchone()[0], rows)
        self.assertEqual(self.c.store.used(), 0)

    def test_new_ids_group_across_accounts_but_keep_original_findings_separate(self):
        self.poll([])
        result = self.poll([case(), case(account=2, deal=11, close="2026-10-27T01:59:40Z")])
        self.assertEqual(len(result["queued_job_ids"]), 1)
        task = self.c.store.jobs()[0]["task"]
        self.assertTrue(task.startswith(sl.PREFIX))
        self.assertIn('"account_id":1', task)
        self.assertIn('"account_id":2', task)
        self.assertIn("not independent confirmations", task)
        separate = self.poll([case(), case(account=2, deal=11), case(deal=12, strategy="s1"), case(deal=13, symbol_id=8)])
        self.assertEqual(len(separate["queued_job_ids"]), 2)

    def test_incomplete_costs_and_missing_setup_review_now_then_follow_up_once(self):
        first = case(original_entry=None)
        self.assertEqual(len(self.poll([first])["queued_job_ids"]), 1)
        task = self.c.store.jobs()[0]["task"]
        self.assertIn('"original_entry":null', task)
        self.assertIn('"net_r":null', task)
        self.assertEqual(self.poll([first])["queued_job_ids"], [])
        original_job = self.c.store.jobs()[0]["id"]
        self.c.store.update(original_job, "completed", result='{"outcome":"no_change"}')
        complete = {**first, "net_costs_complete": True, "net_r": -1.02}
        self.assertEqual(len(self.poll([complete])["queued_job_ids"]), 1)
        self.assertIn("NET-COST FOLLOW-UP", self.c.store.jobs()[0]["task"])
        self.assertEqual(self.poll([{**complete, "net_r": -1.03}])["queued_job_ids"], [])

    def test_costs_before_review_starts_enrich_same_job_without_another_reservation(self):
        self.poll([case(), case(account=2, deal=11)])
        original = self.c.store.jobs()[0]["id"]
        self.assertEqual(self.poll([case(net_costs_complete=True, net_r=-1.0), case(account=2, deal=11)])["queued_job_ids"], [])
        self.assertEqual(len(self.c.store.jobs()), 1)
        job = self.c.store.jobs()[0]
        self.assertEqual(job["id"], original)
        self.assertIn('"net_r":-1.0', job["task"])
        self.assertIn('"account_id":2', job["task"])
        self.assertEqual(self.c.store.used(), 0)

    def test_matching_account_exit_arriving_next_poll_joins_queued_episode(self):
        self.poll([case()])
        result = self.poll([case(), case(account=2, deal=11, close="2026-10-27T01:59:30Z")])
        self.assertEqual(result["queued_job_ids"], [])
        self.assertEqual(len(self.c.store.jobs()), 1)
        task = self.c.store.jobs()[0]["task"]
        self.assertIn('"account_id":1', task)
        self.assertIn('"account_id":2', task)

    def test_invalid_ownership_reason_ids_timestamps_never_queue(self):
        bad = [case(reason="take_profit"), case(ownership_verified=False), case(account_id=True),
               case(deal_id=0), case(closed_at="untrusted-secret"), case(strategy="s2"),
               case(closed_at="2026-10-28T00:00:00Z"), case(environment="unknown")]
        self.assertEqual(self.poll(bad)["queued_job_ids"], [])
        self.assertEqual(self.c.store.jobs(), [])
        duplicate = case()
        self.assertEqual(self.poll([duplicate, copy.deepcopy(duplicate)])["queued_job_ids"], [])

    def test_private_fields_and_arbitrary_original_notes_do_not_cross_bridge(self):
        secret = "SECRET-broker-token-and-cash"
        raw = case(balance=secret, currency=secret, raw_payload=secret, notes=secret,
                   original_entry={"notes": secret, "price": secret, "reasoning": secret, "side": "buy", "planned_reward_r": 1.38})
        self.assertEqual(len(self.poll([raw])["queued_job_ids"]), 1)
        with self.c.store.connect() as db:
            saved = json.dumps([tuple(row) for row in db.execute("SELECT task,result FROM jobs")])
            saved += json.dumps([tuple(row) for row in db.execute("SELECT detail FROM events")])
        self.assertNotIn(secret, saved)
        self.assertNotIn('"price"', self.c.store.jobs()[0]["task"])
        self.assertIn('"planned_reward_r":1.38', self.c.store.jobs()[0]["task"])
        self.assertLessEqual(len(self.c.store.jobs()[0]["task"]), 16000)

    def test_fixed_setup_checks_preserve_unknowns_and_reason_is_not_broker_confirmation(self):
        raw = case(reason_evidence="recorded_inference", checks={"entry_trigger_valid": False,
                    "original_session_valid": True, "unknown_private_field": "secret"},
                   missing_evidence=["complete_net_costs", "entry_trigger", "secret"],
                   net_costs_complete=True, net_r=10 ** 1000)
        self.poll([raw])
        task = self.c.store.jobs()[0]["task"]
        evidence = json.loads(task.split(sl.EVIDENCE_MARKER, 1)[1])["cases"][0]
        self.assertEqual(evidence["reason_evidence"], "recorded_inference")
        self.assertFalse(evidence["checks"]["entry_trigger_valid"])
        self.assertIsNone(evidence["checks"]["original_stop_geometry_valid"])
        self.assertIsNone(evidence["net_r"])
        self.assertEqual(evidence["missing_evidence"], ["complete_net_costs", "entry_trigger"])
        self.assertNotIn("secret", task)

    def test_old_only_startup_does_not_consume_a_review(self):
        old = case(close="2026-10-27T00:00:00Z")
        self.assertEqual(self.poll([old])["queued_job_ids"], [])
        self.assertEqual(self.poll([old])["queued_job_ids"], [])
        self.assertEqual(self.c.store.jobs(), [])

    def test_owner_immediate_review_seeds_latest_episode_without_historical_backlog(self):
        old = case(deal=8, close="2026-10-26T01:59:00Z")
        earlier = case(deal=9, close="2026-10-27T00:00:00Z")
        latest = case(close="2026-10-27T01:00:00Z")
        result = self.poll([old, earlier, latest], review_latest=True)
        self.assertEqual(len(result["queued_job_ids"]), 1)
        self.assertIn('"deal_id":10', self.c.store.jobs()[0]["task"])
        self.assertNotIn('"deal_id":9', self.c.store.jobs()[0]["task"])
        self.assertEqual(self.poll([old, earlier, latest], review_latest=True)["queued_job_ids"], [])
        self.assertEqual(self.c.store.used(), 0)

    def test_unavailable_and_oversized_sources_are_safe_and_do_not_write(self):
        with patch.object(sl.urllib.request, "urlopen", side_effect=RuntimeError("https://secret-token@broker/private")):
            result = sl.poll(self.c, at=NOW)
        self.assertEqual(result, {"reason": "source_unavailable", "queued_job_ids": []})
        class Large:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def read(self, size): return b"x" * size
        with patch.object(sl.urllib.request, "urlopen", return_value=Large()):
            self.assertEqual(sl.poll(self.c, at=NOW)["reason"], "source_too_large")
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT COUNT(*) FROM events").fetchone()[0], 0)

    def completed_review(self):
        self.c.config = types.SimpleNamespace(env={"TELEGRAM_BOT_TOKEN": "private-bot-token",
                                                    "TELEGRAM_OWNER_CHAT_ID": "12345",
                                                    "GH_TOKEN": "private-github-token"})
        self.poll([case(net_costs_complete=True, net_r=-1.02)])
        job = self.c.store.jobs()[0]
        self.c.store.update(job["id"], "completed", result='{"outcome":"no_change"}')
        with self.c.store.connect() as db:
            db.execute("UPDATE jobs SET updated=? WHERE id=?", (NOW, job["id"]))
        report = {"outcome": "no_change", "summary": "<script>unsafe</script> private-bot-token private-github-token original evidence unknown",
                  "tests": "FULL TEST LOG MUST NOT APPEAR", "owner_decisions": "keep existing risk & stops"}
        return job, report

    def test_completed_reviews_remain_internal_even_with_telegram_configured(self):
        job, report = self.completed_review()
        before = self.c.store.jobs()
        with patch("reporting.send_document", side_effect=AssertionError("SL review must stay internal")) as post:
            self.assertEqual(sl.deliver_review(self.c, job, report), {"state": "internal_only"})
            self.assertEqual(sl.deliver_review(self.c, job, report), {"state": "internal_only"})
        post.assert_not_called()
        self.assertEqual(self.c.store.jobs(), before)
        self.assertEqual(self.c.store.used(), 0)
        with self.c.store.connect() as db:
            self.assertEqual(db.execute("SELECT count(*) FROM events WHERE kind='stop_loss_review_delivery'").fetchone()[0], 0)

    def test_restart_and_old_uncertain_receipt_do_not_reenable_review_delivery(self):
        job, report = self.completed_review()
        self.c.store.event("stop_loss_review_delivery", job["id"], '{"state":"sending"}')
        self.c.store = w.Store(self.temp.name)
        with patch("reporting.send_document", side_effect=AssertionError("no historical review replay")) as post:
            self.assertEqual(sl.deliver_review(self.c, job, report)["state"], "internal_only")
        post.assert_not_called()
        self.assertEqual(self.c.store.jobs()[0]["state"], "completed")


if __name__ == "__main__":
    unittest.main()
