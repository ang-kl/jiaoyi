"""Observer episodes enter the existing incident lane without counter/clock churn."""
import copy
import datetime as dt
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
import urllib.request
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("assessment_incident_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)
import incidents


class AssessmentIncidentTests(unittest.TestCase):
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
        self.no_network = patch.object(urllib.request, "urlopen", side_effect=AssertionError("unexpected network"))
        self.no_network.start()

    def tearDown(self):
        self.no_network.stop()
        self.temp.cleanup()

    def health(self, kind="deadline_missed", episode=1, state="active", at=None, account=49236051):
        at = at or self.at
        entry = {"id": f"setup-assessment:{account}:{kind}", "kind": kind,
            "reason": incidents.ASSESSMENT_REASONS[kind], "episode": episode, "state": state,
            "opened_at": (self.at - dt.timedelta(seconds=10)).isoformat(),
            "last_observed_at": at.isoformat(), "resolved_at": at.isoformat() if state == "resolved" else None,
            "evidence_revision": 30, "occurrences": 1, "next_check_at": (at + dt.timedelta(seconds=1)).isoformat(),
            "private_cash": 1234, "position_id": 98765}
        return {"status": "ok", "setup_assessment": {
            "incident_recovery_max_age_secs": 15, "accounts": [{
                "account_id": account, "state": "assessed", "assessed_at": at.isoformat(),
                "evidence_complete": True, "timing": {"missed_deadlines": 9}, "incidents": [entry]}]}}

    def observe(self, health, at=None, operations=True, coordinator=None):
        at = at or self.at
        return (coordinator or self.c).incidents.observe(health,
            {"as_of": at.isoformat(), "reports": []} if operations else None, clock=at)

    def state(self, event=incidents.STATE_EVENT, coordinator=None):
        return json.loads((coordinator or self.c).store.last_event(event) or "{}")

    def row(self, health):
        return health["setup_assessment"]["accounts"][0]

    def fill_capacity(self):
        state = {f"existing-{i}": {"id": f"existing-{i}", "kind": "review_proven_defect",
            "key": f"existing-{i}", "state": "missing_evidence", "condition_present": True,
            "version": 1, "received_at": self.at.isoformat(), "analysis_job": None}
            for i in range(incidents.MAX_INCIDENTS)}
        with self.c.store.connect() as db:
            self.c.incidents._save(db, state, self.at.isoformat())
        return state

    def test_one_episode_ignores_counters_timestamps_duplicates_and_private_fields(self):
        health = self.health()
        self.observe(health)
        original = self.state()
        self.assertEqual(len(original), 1)
        incident = next(iter(original.values()))
        self.assertEqual(incident["kind"], "setup_assessment_deadline_missed")
        self.assertEqual(len(self.c.store.jobs()), 1)
        for i in range(1, 6):
            at = self.at + dt.timedelta(seconds=i)
            row = self.row(health)
            row["assessed_at"] = at.isoformat()
            row["timing"]["missed_deadlines"] += 100
            row["incidents"][0].update(last_observed_at=at.isoformat(), occurrences=i + 1,
                evidence_revision=1000 + i, next_check_at=(at + dt.timedelta(seconds=1)).isoformat())
            self.observe(health, at)
        self.assertEqual(self.state(), original)
        self.assertEqual(len(self.c.store.jobs()), 1)
        self.assertEqual(self.c.store.used(), 0)
        task = self.c.store.jobs()[0]["task"]
        self.assertNotIn("private_cash", task)
        self.assertNotIn("98765", task)
        self.assertNotIn("occurrences", task)
        self.assertEqual(len(list((self.c.store.private / "incident-reports").glob("*.html"))), 1)

    def test_recovery_is_explicit_fresh_same_episode_and_not_a_repair_or_new_session(self):
        self.observe(self.health())
        identity = next(iter(self.state()))
        later = self.at + dt.timedelta(seconds=20)
        healthy = self.health(state="resolved", at=later)
        cases = []
        for field, value in (("assessed_at", self.at.isoformat()), ("assessed_at", None),
                             ("assessed_at", (later + dt.timedelta(seconds=1)).isoformat()),
                             ("state", "disconnected"), ("evidence_complete", False)):
            bad = copy.deepcopy(healthy)
            self.row(bad)[field] = value
            cases.append(bad)
        for field, value in (("last_observed_at", self.at.isoformat()), ("episode", 2),
                             ("resolved_at", None), ("state", "active")):
            bad = copy.deepcopy(healthy)
            self.row(bad)["incidents"][0][field] = value
            cases.append(bad)
        bad = copy.deepcopy(healthy)
        self.row(bad)["incidents"] = []
        cases.extend((bad, {}, None))
        for snapshot in cases:
            with self.subTest(snapshot=snapshot):
                self.observe(snapshot, later)
                self.assertTrue(self.state()[identity]["condition_present"])
        jobs_before = len(self.c.store.jobs())
        # A failed unrelated operations read cannot invalidate fresh source recovery.
        self.observe(healthy, later, operations=False)
        current = self.state()[identity]
        self.assertFalse(current["condition_present"])
        self.assertNotEqual(current["state"], "closed")
        self.assertEqual(current["version"], 1)
        self.assertEqual(len(self.c.store.jobs()), jobs_before)
        self.assertEqual(self.c.store.used(), 0)
        # Old active body must not reopen the same episode; real recurrence does.
        self.observe(self.health(at=later), later)
        self.assertFalse(self.state()[identity]["condition_present"])
        self.observe(self.health(episode=2, at=later), later)
        self.assertTrue(self.state()[identity]["condition_present"])
        self.assertEqual(self.state()[identity]["version"], 2)

    def test_permanent_miss_counter_and_missing_or_malformed_episodes_are_not_faults(self):
        health = self.health(state="resolved")
        self.row(health)["timing"]["missed_deadlines"] = 1_000_000
        self.observe(health)
        self.assertEqual(self.state(), {})
        invalid = []
        for field, value in (("episode", True), ("episode", 0), ("reason", "private position 98765"),
                             ("id", "setup-assessment:1:deadline_missed"), ("kind", "unknown")):
            bad = self.health()
            self.row(bad)["incidents"][0][field] = value
            invalid.append(bad)
        duplicate = self.health()
        self.row(duplicate)["incidents"] *= 2
        invalid.append(duplicate)
        duplicate_account = self.health()
        duplicate_account["setup_assessment"]["accounts"] *= 2
        invalid.append(duplicate_account)
        for snapshot in invalid:
            self.observe(snapshot)
            self.assertEqual(self.state(), {})
        self.assertEqual(self.c.store.jobs(), [])

    def test_full_lane_retains_factual_overflow_without_jobs_and_survives_restart(self):
        original = self.fill_capacity()
        status = self.observe(self.health())
        self.assertEqual(self.state(), original)
        held = self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)
        self.assertEqual(len(held), 1)
        item = next(iter(held.values()))
        self.assertEqual(item["state"], "capacity_blocked")
        self.assertIsNone(item["analysis_job"])
        self.assertEqual(item["delivery_state"], "created")
        self.assertEqual(status["capacity"]["state_rejected"], 1)
        self.assertEqual(status["capacity"]["observer_overflow_blocked"], 1)
        self.assertEqual(self.c.store.jobs(), [])
        self.assertEqual(self.c.store.used(), 0)
        restarted = w.Coordinator(self.c.config)
        self.observe(self.health(at=self.at + dt.timedelta(seconds=1)), self.at + dt.timedelta(seconds=1), coordinator=restarted)
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT, restarted), held)
        self.assertEqual(len(list((self.c.store.private / "incident-reports").glob("*.html"))), 1)

    def test_overflow_handoff_only_after_closed_slot_preserves_other_unresolved_items(self):
        original = self.fill_capacity()
        self.observe(self.health())
        held = next(iter(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT).values()))
        with self.c.store.connect() as db:
            state = self.c.incidents._load(db)
            state["existing-0"]["state"] = "closed"
            self.c.incidents._save(db, state, self.at.isoformat())
        later = self.at + dt.timedelta(seconds=1)
        self.observe(self.health(at=later), later)
        current = self.state()
        self.assertEqual(len(current), incidents.MAX_INCIDENTS)
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT), {})
        self.assertEqual(current[held["id"]]["state"], "analysis_pending")
        self.assertEqual(current[held["id"]]["first_received_at"], held["first_received_at"])
        self.assertEqual(len(self.c.store.jobs()), 1)
        for identity, item in original.items():
            if identity != "existing-0":
                self.assertEqual(current[identity], item)
        self.observe(self.health(at=later), later)
        self.assertEqual(len(self.c.store.jobs()), 1)

    def test_new_source_generation_preserves_episode_counter_and_restart_accepts_true_recurrence(self):
        self.fill_capacity()
        self.observe(self.health(episode=4))
        later = self.at + dt.timedelta(seconds=10)
        self.observe(self.health(episode=4, state="resolved", at=later), later)
        original = self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)
        identity = next(iter(original))
        restarted = w.Coordinator(self.c.config)
        replay = self.health(episode=4, at=later)
        self.row(replay)["generation"] = 2
        self.observe(replay, later, coordinator=restarted)
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT, restarted), original)
        recurrence = self.health(episode=5, at=later)
        self.row(recurrence)["generation"] = 2
        self.observe(recurrence, later, coordinator=restarted)
        item = self.state(incidents.ASSESSMENT_OVERFLOW_EVENT, restarted)[identity]
        self.assertEqual(item["evidence"]["source_episode"], 5)
        self.assertTrue(item["condition_present"])
        self.assertEqual(item["version"], 2)
        self.assertEqual(restarted.store.jobs(), [])

    def test_overflow_factual_report_resumes_after_interruption_without_duplicate_delivery(self):
        self.fill_capacity()
        with patch.object(self.c.incidents, "_publish"):
            self.observe(self.health())
        held = next(iter(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT).values()))
        self.assertEqual(held["delivery_state"], "pending")
        restarted = w.Coordinator(self.c.config)
        restarted.incidents.tick(clock=self.at)
        item = self.state(incidents.ASSESSMENT_OVERFLOW_EVENT, restarted)[held["id"]]
        self.assertEqual(item["delivery_state"], "created")
        restarted.incidents.tick(clock=self.at)
        self.assertEqual(len(list((self.c.store.private / "incident-reports").glob("*.html"))), 1)
        self.assertEqual(restarted.store.jobs(), [])

    def test_overflow_recovery_needs_fresh_explicit_receipt_and_never_dispatches_recovered_episode(self):
        self.fill_capacity()
        self.observe(self.health())
        identity = next(iter(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)))
        later = self.at + dt.timedelta(seconds=20)
        for snapshot in (None, {}, self.health(state="resolved")):
            self.observe(snapshot, later)
            self.assertTrue(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)[identity]["condition_present"])
        self.observe(self.health(state="resolved", at=later), later)
        item = self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)[identity]
        self.assertEqual(item["state"], "source_recovered")
        self.assertFalse(item["condition_present"])
        self.observe(self.health(at=later), later)
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT)[identity], item)
        self.assertEqual(self.c.store.jobs(), [])

    def test_projection_cap_retains_observer_receipt_and_overflow_limit_is_hard(self):
        self.fill_capacity()
        health = self.health()
        health["strategy3"] = [{"account_id": 49236051, "open_positions": [
            {"position_id": i + 1, "symbol": f"SYMBOL{i}", "stop": None}
            for i in range(incidents.MAX_INCIDENTS)]}]
        status = self.observe(health)
        self.assertEqual(status["capacity"]["projection_omitted"], 1)
        self.assertEqual(status["capacity"]["observer_overflow_retained"], 1)
        with self.c.store.connect() as db:
            current = self.c.incidents._load(db, incidents.ASSESSMENT_OVERFLOW_EVENT)
            template = next(iter(current.values()))
            full = {f"held-{i}": {**template, "id": f"held-{i}"} for i in range(incidents.MAX_ASSESSMENT_OVERFLOW)}
            self.c.incidents._save(db, full, self.at.isoformat(), incidents.ASSESSMENT_OVERFLOW_EVENT)
        status = self.observe(self.health(kind="disconnected"))
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT), full)
        self.assertEqual(status["capacity"]["overflow_rejected"], 1)
        self.assertEqual(self.c.store.jobs(), [])

    def test_capped_projection_updates_higher_episode_in_existing_retained_slot(self):
        self.observe(self.health())
        identity = next(iter(self.state()))
        health = self.health(episode=2)
        health["strategy3"] = [{"account_id": 49236051, "open_positions": [
            {"position_id": i + 1, "symbol": f"SYMBOL{i}", "stop": None}
            for i in range(incidents.MAX_INCIDENTS)]}]
        # Protection candidates retain their existing order; the observer lies
        # beyond the capped projection but already owns its state slot.
        self.assertFalse(any(item["kind"].startswith("setup_assessment_")
                             for item in incidents.candidates(health, {"reports": []}, self.at)))
        status = self.observe(health)
        current = self.state()
        self.assertEqual(len(current), incidents.MAX_INCIDENTS)
        self.assertEqual(current[identity]["evidence"]["source_episode"], 2)
        self.assertEqual(current[identity]["version"], 2)
        self.assertEqual(status["capacity"]["projection_omitted"], 1)
        self.assertEqual(self.state(incidents.ASSESSMENT_OVERFLOW_EVENT), {})
        observer_jobs = [job for job in self.c.store.jobs(("queued",))
                         if "setup_assessment_deadline_missed" in job["task"]]
        self.assertEqual(len(observer_jobs), 2)
        jobs_before = len(self.c.store.jobs(("queued",)))
        self.observe(health)
        self.assertEqual(len(self.c.store.jobs(("queued",))), jobs_before)
        self.assertEqual(self.c.store.used(), 0)


if __name__ == "__main__":
    unittest.main()
