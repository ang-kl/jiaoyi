"""A completed assessment can retain missing evidence without inventing a defect."""
import contextlib
import copy
import datetime as dt
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("nonrepair_sl_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)


class NonrepairStopLossReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.c = w.Coordinator(w.Config({
            "ENGINEERING_STATE_DIR": self.temp.name, "ENGINEERING_ENABLED": "1",
            "ENGINEERING_AUTH_MODE": "chatgpt", "GH_TOKEN": "fixture-gh",
            "RAILWAY_TOKEN": "fixture-rail", "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_WORK_UNTIL": "2026-10-28T23:30:00+00:00",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap"}))
        case = {"reason": "stop_loss", "ownership_verified": True, "account_id": 46130058,
                "deal_id": 329214615, "position_id": 254331404, "symbol_id": 10028,
                "strategy": "s3", "environment": "demo", "closed_at": "2026-10-29T09:08:19Z",
                "net_costs_complete": False, "net_r": None, "missing_evidence": ["complete_net_costs"]}
        self.case = w.stop_loss_reviews.case_facts(case, dt.datetime.now(dt.timezone.utc))
        self.task = w.stop_loss_reviews.review_task([self.case])
        self.report = {"outcome": "no_change", "summary": "Admission evidence remains unknown; no defect proven.",
                       "tests": "Read-only original evidence review", "owner_decisions": "Await new evidence",
                       "acceptance_passed": False, "incident": {
                           "id": None, "disposition": "missing_evidence", "evidence_ids": [self.case["case_id"]],
                           "repair_proposal": None}, "incidents": None}

    def tearDown(self):
        self.temp.cleanup()

    def job(self, jid):
        with self.c.store.connect() as db:
            return dict(db.execute("SELECT * FROM jobs WHERE id=?", (jid,)).fetchone())

    def test_source_linked_nonrepair_assessments_complete_without_an_incident_or_fact_acceptance(self):
        for disposition in ("normal_loss", "research", "missing_evidence", "owner_policy"):
            with self.subTest(disposition=disposition):
                report = copy.deepcopy(self.report)
                report["incident"]["disposition"] = disposition
                if disposition == "owner_policy":
                    report["outcome"] = "owner_blocked"
                jid = self.c.store.enqueue(self.task)
                with patch.object(self.c.incidents, "apply_review", side_effect=AssertionError("no incident repair")), \
                     contextlib.redirect_stdout(io.StringIO()):
                    result = self.c.finish_review(self.job(jid), report)
                self.assertTrue(result["applied"])
                self.assertEqual(self.job(jid)["state"], "owner_blocked" if disposition == "owner_policy" else "completed")
                self.assertEqual(json.loads(self.job(jid)["result"]), report)
                self.assertFalse(json.loads(self.job(jid)["result"])["acceptance_passed"])
        self.assertIsNone(self.c.store.last_event("incident_state"))
        self.assertIsNone(self.c.store.reserve(36))
        self.assertEqual(self.c.store.used(), 0)
        self.assertFalse(self.c.config.mutation_allowed())

    def test_foreign_partial_duplicate_and_repair_claims_still_require_incident_validation(self):
        variants = []
        for key, value in (("evidence_ids", ["1:2:3"]), ("evidence_ids", []),
                           ("evidence_ids", [self.case["case_id"]] * 2), ("id", "foreign-incident"),
                           ("repair_proposal", {"summary": "unsupported repair"}), ("disposition", "proven_defect")):
            report = copy.deepcopy(self.report); report["incident"][key] = value
            variants.append((self.task, report))
        report = copy.deepcopy(self.report); report["incidents"] = [report["incident"]]
        variants.append((self.task, report))
        report = copy.deepcopy(self.report); report["repair_proposal"] = {"summary": "conflicting repair"}
        variants.append((self.task, report))
        variants.append((w.stop_loss_reviews.PREFIX + " malformed packet", self.report))
        second = {**self.case, "deal_id": 329214616, "case_id": "46130058:329214616:254331404"}
        variants.append((w.stop_loss_reviews.review_task([self.case, second]), self.report))
        malformed = {**self.case, "case_id": "1:2:3"}
        variants.append((w.stop_loss_reviews.review_task([malformed]), self.report))
        variants.append((w.incidents.PREFIX + " incident review", self.report))
        for task, report in variants:
            with self.subTest(task=task[:25], incident=report["incident"]):
                jid = self.c.store.enqueue(task)
                with patch.object(self.c.incidents, "apply_review", return_value={"applied": False, "reason": "invalid_disposition"}) as apply:
                    self.c.finish_review(self.job(jid), report)
                apply.assert_called_once()
                self.assertEqual(self.job(jid)["state"], "review_pending")
        report = {**self.report, "acceptance_passed": True}
        jid = self.c.store.enqueue(self.task)
        with self.assertRaisesRegex(RuntimeError, "financial acceptance"):
            self.c.finish_review(self.job(jid), report)
        self.assertEqual(self.c.store.used(), 0)

    def test_restart_reconciles_only_original_valid_reports_and_retains_rejection_history(self):
        valid = self.c.store.enqueue(self.task)
        originals = [valid]
        keep = []
        for variant in ("foreign", "no_rejection", "release", "invalid_json", "incident", "latest_invalid"):
            jid = self.c.store.enqueue(w.incidents.PREFIX + " source" if variant == "incident" else self.task)
            keep.append((jid, variant)); originals.append(jid)
        for jid in originals:
            variant = next((name for target, name in keep if target == jid), None)
            report = copy.deepcopy(self.report)
            if variant == "foreign": report["incident"]["evidence_ids"] = ["1:2:3"]
            detail = "invalid json" if variant == "invalid_json" else json.dumps(report)
            self.c.store.event("model_report", jid, detail)
            if variant == "latest_invalid": self.c.store.event("model_report", jid, "invalid latest report")
            if variant != "no_rejection":
                self.c.store.event("incident_review_unresolved", jid, '{"reason":"structured_source_linked_defect_required"}')
            self.c.store.update(jid, "review_pending", result=json.dumps({**report, "review_pending_reason": "structured_source_linked_defect_required"}),
                                pr=135 if variant == "release" else None)
        with self.c.store.connect() as db:
            before = [tuple(row) for row in db.execute("SELECT * FROM events ORDER BY id")]
        self.c.store.recover()
        self.c.store.recover()
        self.assertEqual(self.job(valid)["state"], "completed")
        self.assertEqual(json.loads(self.job(valid)["result"]), self.report)
        for jid, _ in keep: self.assertEqual(self.job(jid)["state"], "review_pending")
        with self.c.store.connect() as db:
            after = [tuple(row) for row in db.execute("SELECT * FROM events ORDER BY id")]
            reconciled = db.execute("SELECT job FROM events WHERE kind='stop_loss_assessment_reconciled'").fetchall()
        self.assertEqual(after[:len(before)], before)
        self.assertEqual([row[0] for row in reconciled], [valid])
        reused = json.loads(w.review_memory.context(self.c.store, self.task, 9999))
        self.assertEqual([row["source_job_id"] for row in reused["reports"]], [valid])
        self.assertEqual(reused["reports"][0]["assessment_disposition"], "missing_evidence")
        self.assertIn("never authority", reused["notice"])
        self.assertIsNone(self.c.store.reserve(36))
        self.assertEqual(self.c.store.used(), 0)
        self.assertFalse(self.c.config.mutation_allowed())


if __name__ == "__main__":
    unittest.main()
