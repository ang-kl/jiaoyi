"""Cost and identity regressions for real incident/store/report batching."""
import datetime as dt
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("incident_batch_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)
import incidents


class IncidentBatchingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.c = w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt",
            "GH_TOKEN": "fixture-gh", "RAILWAY_TOKEN": "fixture-railway",
            "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap",
        }))
        self.at = dt.datetime.now(dt.timezone.utc)

    def tearDown(self):
        self.temp.cleanup()

    def snapshot(self, count=17):
        return {"status": "ok", "strategy3": [{
            "account_id": 49236051, "prereg_hash": "a" * 64,
            "liveness": {"state": "alive", "fast_monitor": {"state": "observing"}},
            "open_positions": [], "preopen_preparation": {"symbols": [
                {"symbol": "TEST" + str(i), "state": "deadline_missed",
                 "deadline_at": (self.at - dt.timedelta(minutes=1)).isoformat(),
                 "session_open": (self.at + dt.timedelta(minutes=9)).isoformat(),
                 "session_close": (self.at + dt.timedelta(hours=2)).isoformat()}
                for i in range(count)]}}]}

    def observe(self, count=17):
        health = self.snapshot(count)
        ops = {"as_of": self.at.isoformat(), "reports": []}
        self.c.incidents.observe(health, ops, clock=self.at)
        return health, ops

    def state(self):
        return json.loads(self.c.store.last_event("incident_state"))

    def report(self, job_id):
        return {"outcome": "no_change", "acceptance_passed": False,
            "summary": "Each source deadline needs separately verified preparation evidence.",
            "tests": "Read-only source observation; no inference about profitability.",
            "owner_decisions": "None", "incident": None,
            "incidents": [{"id": item["id"], "disposition": "missing_evidence",
                           "evidence_ids": item["evidence_ids"], "repair_proposal": None}
                          for item in self.state().values() if item["analysis_job"] == job_id]}

    def test_seventeen_reports_need_three_sessions_and_compaction_survives_restart(self):
        snapshots = self.observe()
        original = self.state()
        self.assertEqual(len(self.c.store.jobs(("queued",))), 17)
        html = list((self.c.store.private / "incident-reports").glob("*.html"))
        self.assertEqual(len(html), 17)
        result = self.c.incidents.compact_pending_reviews()
        self.assertEqual((result["batched_jobs"], result["incidents"]), (3, 17))
        self.assertEqual(len(result["superseded_jobs"]), 14)
        queued = self.c.store.jobs(("queued",))
        self.assertEqual(len(queued), 3)
        self.assertTrue(all(len(row["task"]) <= 16000 for row in queued))
        self.assertEqual(sorted(item["analysis_batch_size"] for item in self.state().values()), [1] + [8] * 16)
        for identity, item in self.state().items():
            self.assertEqual(item["evidence_ids"], original[identity]["evidence_ids"])
        self.assertEqual(len(list((self.c.store.private / "incident-reports").glob("*.html"))), 17)
        self.assertEqual(self.c.store.used(), 0)
        restarted = w.Coordinator(self.c.config)
        before = restarted.store.last_event("incident_state")
        restarted.incidents.observe(*snapshots, clock=self.at + dt.timedelta(seconds=10))
        again = restarted.incidents.compact_pending_reviews()
        self.assertEqual(again["superseded_jobs"], [])
        self.assertEqual(restarted.store.last_event("incident_state"), before)
        self.assertEqual(len(restarted.store.jobs(("queued",))), 3)
        with self.c.store.connect() as db:
            receipts = db.execute("SELECT detail FROM events WHERE kind='incident_review_batched'").fetchall()
        self.assertEqual(len(receipts), 14)
        self.assertTrue(all(json.loads(row[0])["source_task"].startswith(incidents.PREFIX) for row in receipts))

    def test_batch_exact_identity_and_evidence_are_validated_before_any_disposition(self):
        self.observe(4)
        self.c.incidents.compact_pending_reviews()
        job = self.c.store.reserve(36)
        report = self.report(job["id"])
        report["incidents"][0]["disposition"] = "research"
        applied = self.c.finish_review(job, report)
        self.assertTrue(applied["applied"])
        states = [item["state"] for item in self.state().values()]
        self.assertEqual(states.count("research"), 1)
        self.assertEqual(states.count("missing_evidence"), 3)
        self.assertEqual(self.c.store.jobs(("completed",))[0]["id"], job["id"])

        # A malformed later member cannot partially apply an earlier valid one.
        changed = self.snapshot(4)
        changed["strategy3"][0]["prereg_hash"] = "b" * 64
        self.c.incidents.observe(changed, {"reports": []}, clock=self.at)
        self.c.incidents.compact_pending_reviews()
        job = self.c.store.reserve(36)
        report = self.report(job["id"])
        report["incidents"][-1]["evidence_ids"] = ["stale-evidence"]
        result = self.c.finish_review(job, report)
        self.assertFalse(result["applied"])
        self.assertEqual({item["state"] for item in self.state().values()}, {"analysis_incomplete"})
        self.assertTrue(all(item.get("review_applied_job") != job["id"] for item in self.state().values()))
        self.assertEqual(self.c.store.jobs(("verification_pending",))[0]["id"], job["id"])
        report = self.report(job["id"])
        report["incidents"][-1] = dict(report["incidents"][0])
        self.assertEqual(self.c.incidents.apply_review(job, report)["reason"], "incident_batch_identity_mismatch")

        old_batch = self.report(job["id"])
        changed["strategy3"][0]["preopen_preparation"]["symbols"][0]["session_open"] = (
            self.at + dt.timedelta(minutes=10)).isoformat()
        self.c.incidents.observe(changed, {"reports": []}, clock=self.at + dt.timedelta(seconds=1))
        newest = next(item for item in self.state().values() if item["symbol"] == "TEST0")
        self.assertNotEqual(newest["analysis_job"], job["id"])
        self.assertFalse(self.c.finish_review(job, old_batch)["applied"])
        self.assertEqual(self.state()[newest["id"]]["state"], "analysis_pending")
        self.assertTrue(all(item.get("review_applied_job") != job["id"] for item in self.state().values()))

        # A syntactically valid normal-loss claim cannot complete an unresolved
        # below-target analysis, even when other members have valid dispositions.
        report = self.report(job["id"])
        target = report["incidents"][0]
        with self.c.store.connect() as db:
            state = self.c.incidents._load(db)
            state[target["id"]]["kind"] = "goal_below_target"
            state[target["id"]]["condition_present"] = True
            self.c.incidents._save(db, state, self.at.isoformat())
        target["disposition"] = "normal_loss"
        self.assertFalse(self.c.finish_review(job, report)["applied"])
        self.assertEqual(self.state()[target["id"]]["state"], "analysis_incomplete")
        self.assertEqual(self.c.store.jobs(("verification_pending",))[0]["id"], job["id"])

    def test_running_attempted_and_linked_repairs_stay_put_and_new_evidence_survives(self):
        self.observe(6)
        original = self.state()
        members = list(original.values())
        active = self.c.store.reserve(36)
        attempted = next(item for item in members if item["analysis_job"] != active["id"])
        repair_linked = next(item for item in members if item["analysis_job"] not in (active["id"], attempted["analysis_job"]))
        repair_id = self.c.store.enqueue(incidents.REPAIR_PREFIX + "existing bounded repair")
        with self.c.store.connect() as db:
            db.execute("UPDATE jobs SET attempts=1 WHERE id=?", (attempted["analysis_job"],))
            state = self.c.incidents._load(db)
            state[repair_linked["id"]]["repair_job"] = repair_id
            self.c.incidents._save(db, state, self.at.isoformat())
        self.c.incidents.compact_pending_reviews()
        state = self.state()
        for item in (attempted, repair_linked):
            self.assertEqual(state[item["id"]]["analysis_job"], item["analysis_job"])
        self.assertEqual(state[repair_linked["id"]]["repair_job"], repair_id)
        active_member = next(item for item in members if item["analysis_job"] == active["id"])
        self.assertEqual(state[active_member["id"]]["analysis_job"], active["id"])
        old_report = {**self.report(active["id"]), "incidents": None,
                      "incident": self.report(active["id"])["incidents"][0]}
        changed = self.snapshot(6)
        changed["strategy3"][0]["preopen_preparation"]["symbols"][int(active_member["symbol"][4:])]["session_open"] = (
            self.at + dt.timedelta(minutes=10)).isoformat()
        self.c.incidents.observe(changed, {"reports": []}, clock=self.at + dt.timedelta(seconds=1))
        self.c.incidents.compact_pending_reviews()
        new_member = self.state()[active_member["id"]]
        self.assertNotEqual(new_member["analysis_job"], active["id"])
        self.assertNotEqual(new_member["evidence_ids"], active_member["evidence_ids"])
        self.assertFalse(self.c.incidents.apply_review(active, old_report)["applied"])
        self.assertEqual(self.state()[active_member["id"]]["state"], "analysis_pending")
        self.assertEqual(self.c.store.used(), 1)


if __name__ == "__main__":
    unittest.main()
