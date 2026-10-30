"""An unresolved analysis is not a pending deployment verification."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("pending_review_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class PendingReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.c = w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": self.temp.name, "ENGINEERING_ENABLED": "1",
            "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture-gh",
            "RAILWAY_TOKEN": "fixture-rail", "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap"}))
        self.c.phase = "idle"

    def tearDown(self):
        self.temp.cleanup()

    def job(self, jid):
        return next(row for row in self.c.store.jobs(("review_pending", "verification_pending", "queued"))
                    if row["id"] == jid)

    def legacy(self, task, *, unresolved=True, **release):
        jid = self.c.store.enqueue(task)
        result = json.dumps({"outcome": "verification_pending", "summary": "Original private uncertainty"})
        self.c.store.update(jid, "verification_pending", result=result, **release)
        if unresolved:
            self.c.store.event("incident_review_unresolved", jid,
                               json.dumps({"reason": "structured_source_linked_defect_required"}))
        return jid, result

    def test_missing_evidence_review_keeps_its_report_without_claiming_verification(self):
        jid = self.c.store.enqueue(w.stop_loss_reviews.PREFIX + " original setup")
        report = {"outcome": "no_change", "summary": "Geometry passes; admission evidence unknown.",
                  "tests": "Read-only inspection", "owner_decisions": "None", "acceptance_passed": False,
                  "incident": {"id": None, "disposition": "missing_evidence",
                               "evidence_ids": ["43002148:345154939:637097369"], "repair_proposal": None},
                  "incidents": None}
        applied = self.c.finish_review(self.job(jid), report)
        self.assertFalse(applied["applied"])
        saved = self.job(jid)
        self.assertEqual(saved["state"], "review_pending")
        stored = json.loads(saved["result"])
        self.assertEqual(stored["summary"], report["summary"])
        self.assertEqual(stored["outcome"], "no_change")
        self.assertEqual(stored["review_pending_reason"], "structured_source_linked_defect_required")
        self.assertFalse(stored["acceptance_passed"])
        activity = self.c.status()["activity"]
        self.assertEqual(activity["reason"], "no_queued_work_review_pending")
        self.assertEqual(activity["pending_verification"], {"count": 0, "ids": []})
        self.assertEqual(activity["pending_reviews"]["ids"], [jid])
        self.assertEqual(activity["pending_reviews"]["items"], [{
            "id": jid, "kind": "stop_loss", "reason_code": "structured_source_linked_defect_required"}])
        self.assertNotIn(report["summary"], json.dumps(activity))
        self.assertIsNone(self.c.store.reserve(36))
        self.assertEqual(self.c.store.used(), 0)

    def test_restart_reclassifies_only_proven_legacy_review_records_and_preserves_history(self):
        migrated = [self.legacy(prefix + " review") for prefix in (w.stop_loss_reviews.PREFIX, w.incidents.PREFIX)]
        untouched = [self.legacy("Verify an existing release"),
                     self.legacy(w.incidents.PREFIX + " review", unresolved=False),
                     self.legacy("prefix " + w.incidents.PREFIX + " not a review")]
        for column, value in (("pr", 134), ("head", "a" * 40), ("merged", "b" * 40)):
            untouched.append(self.legacy(w.incidents.PREFIX + " unusual release metadata", **{column: value}))
        with self.c.store.connect() as db:
            before = [tuple(row) for row in db.execute("SELECT id,at,job,kind,detail FROM events ORDER BY id")]
        self.c.store.recover()
        self.c.store.recover()
        for jid, result in migrated:
            self.assertEqual(self.job(jid)["state"], "review_pending")
            self.assertEqual(self.job(jid)["result"], result)
        for jid, result in untouched:
            self.assertEqual(self.job(jid)["state"], "verification_pending")
            self.assertEqual(self.job(jid)["result"], result)
        with self.c.store.connect() as db:
            after = [tuple(row) for row in db.execute("SELECT id,at,job,kind,detail FROM events ORDER BY id")]
            events = db.execute("SELECT job FROM events WHERE kind='review_pending_reclassified'").fetchall()
        self.assertEqual(after[:len(before)], before)
        self.assertEqual(sorted(row[0] for row in events), [jid for jid, _ in migrated])
        activity = self.c.status()["activity"]
        self.assertEqual(activity["pending_verification"]["count"], len(untouched))
        self.assertEqual(activity["pending_reviews"]["count"], 2)
        self.assertIsNone(self.c.store.reserve(36))
        self.assertEqual(self.c.store.used(), 0)

    def test_public_reason_is_a_code_even_if_stored_result_is_malformed_or_contains_prose(self):
        ids = []
        for value in ("not json", json.dumps({"review_pending_reason": "private token or prose"}),
                      json.dumps({"review_pending_reason": "x" * 100})):
            jid = self.c.store.enqueue(w.incidents.PREFIX + " private task")
            self.c.store.update(jid, "review_pending", result=value)
            ids.append(jid)
        activity = self.c.status()["activity"]
        self.assertEqual(activity["pending_reviews"]["ids"], ids)
        self.assertTrue(all(row["reason_code"] == "unresolved_review" for row in activity["pending_reviews"]["items"]))
        self.assertNotIn("private", json.dumps(activity))


if __name__ == "__main__":
    unittest.main()
