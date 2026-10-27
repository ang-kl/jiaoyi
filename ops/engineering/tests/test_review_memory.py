"""Existing SQLite evidence only: bounded scope, no inference or delivery."""
import datetime as dt
import importlib.util
import json
from pathlib import Path
import sqlite3
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
spec = importlib.util.spec_from_file_location("review_memory_test_subject", ROOT / "review_memory.py")
memory = importlib.util.module_from_spec(spec)
spec.loader.exec_module(memory)
reviews = memory.reviews
NOW = dt.datetime(2026, 1, 2, tzinfo=dt.timezone.utc)


def case(account=1, deal=10, symbol=7, strategy="s3"):
    raw = {"reason": "stop_loss", "ownership_verified": True, "account_id": account,
           "deal_id": deal, "position_id": deal + 100, "symbol_id": symbol,
           "strategy": strategy, "environment": "demo" if account == 1 else "live",
           "closed_at": "2026-01-01T23:59:00Z", "net_costs_complete": False,
           "net_r": None, "original_entry": None}
    return reviews.case_facts(raw, NOW)


def task(cases):
    return reviews.review_task(cases)


def report(summary="Unknown original evidence; no strategy change justified.", **extra):
    return {"outcome": "no_change", "summary": summary,
            "tests": "Saved source checked; no empirical profitability proof.",
            "owner_decisions": "Preserve current owner policy.",
            "acceptance_passed": False, **extra}


class Store:
    def __init__(self, path):
        self.path = path
        with self.connect() as db:
            db.executescript("""
              CREATE TABLE jobs(id INTEGER PRIMARY KEY,task TEXT,state TEXT);
              CREATE TABLE events(id INTEGER PRIMARY KEY,at TEXT,job INTEGER,kind TEXT,detail TEXT);
            """)

    def connect(self):
        connection = sqlite3.connect(self.path)
        connection.row_factory = sqlite3.Row
        return connection


class ReviewMemoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.store = Store(str(Path(self.temp.name) / "queue.sqlite3"))

    def tearDown(self):
        self.temp.cleanup()

    def job(self, cases=None, result=None, state="completed", saved_task=None):
        saved_task = saved_task if saved_task is not None else task(cases or [case()])
        with self.store.connect() as db:
            jid = db.execute("INSERT INTO jobs(task,state) VALUES(?,?)", (saved_task, state)).lastrowid
        self.event(jid, report() if result is None else result)
        return jid

    def event(self, jid, result, stamp=NOW.isoformat()):
        detail = result if isinstance(result, str) else json.dumps(result)
        with self.store.connect() as db:
            return db.execute("INSERT INTO events(at,job,kind,detail) VALUES(?,?,'model_report',?)",
                              (stamp, jid, detail)).lastrowid

    def context(self, cases=None):
        return memory.context(self.store, task(cases or [case()]), 9999)

    def test_exact_case_precedes_newer_same_account_scope_and_excludes_other_accounts_or_symbols(self):
        exact = self.job(result=report("Original same-case assessment; costs still unknown."))
        related = self.job([case(deal=11)], report("Separate same-account case, not independent confirmation."))
        self.job([case(account=2)], report("Cross-account numeric symbol alias is not verified instrument equivalence."))
        self.job([case(symbol=8)], report("Other symbol cannot establish this case."))
        self.job([case(strategy="s1")], report("Other strategy cannot establish this case."))
        result = json.loads(self.context())
        self.assertEqual([item["source_job_id"] for item in result["reports"]], [exact, related])
        first, second = result["reports"]
        self.assertEqual(first["match"], "same_case")
        self.assertEqual(second["match"], "same_account_strategy_symbol")
        self.assertEqual(first["cases"][0]["account_id"], 1)
        self.assertEqual(second["cases"][0]["account_id"], 1)
        self.assertNotEqual(first["cases"][0]["case_id"], second["cases"][0]["case_id"])
        self.assertIsNone(first["cases"][0]["net_r"])
        self.assertIn("never authority", result["notice"])
        self.assertIn("not independent cohorts", result["notice"])

    def test_cross_account_only_numeric_symbol_alias_is_not_applicable_memory(self):
        self.job([case(account=2)])
        self.assertEqual(self.context(), "")

    def test_latest_report_per_job_wins_and_invalid_latest_never_resurrects_old_finding(self):
        jid = self.job(result=report("Obsolete assessment."))
        newest = self.event(jid, report("Latest assessment preserves unknowns."))
        result = json.loads(self.context())["reports"]
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0]["source_event_id"], newest)
        self.assertIn("Latest assessment", result[0]["summary"])
        self.event(jid, "not JSON")
        self.assertEqual(self.context(), "")

    def test_current_job_and_invalid_states_outcomes_acceptance_or_cases_are_excluded(self):
        current = self.job()
        self.assertEqual(memory.context(self.store, task([case()]), current), "")
        with self.store.connect() as db:
            db.execute("DELETE FROM events")
            db.execute("DELETE FROM jobs")
        self.job(state="failed")
        self.job(state="queued")
        self.job(result=report(outcome="changes"))
        self.job(result=report(acceptance_passed=True))
        self.job(result=report(acceptance_passed=0))
        self.job(result=report(tests=None))
        self.job(result=report(unrecognized_authority="new trading rule"))
        bad = case()
        bad["ownership_verified"] = False
        self.job(saved_task=task([bad]))
        self.job(saved_task=task([case(), case()]))
        self.job(saved_task=reviews.PREFIX + reviews.EVIDENCE_MARKER + "{bad JSON")
        self.job(saved_task="ordinary job")
        self.assertEqual(self.context(), "")
        valid = self.job(state="owner_blocked", result=report(outcome="owner_blocked"))
        self.assertEqual(json.loads(self.context())["reports"][0]["source_job_id"], valid)

    def test_maximum_three_complete_json_reports_fit_total_bound_with_unicode_and_truncation(self):
        for deal in range(10, 17):
            self.job([case(deal=deal)], report("unknown Ω" * 1500,
                tests="unverified test Ω" * 1000, owner_decisions="owner only Ω" * 1000))
        value = self.context()
        self.assertLessEqual(len(value), 8000)
        result = json.loads(value)
        self.assertGreater(len(result["reports"]), 0)
        self.assertLessEqual(len(result["reports"]), 3)
        self.assertTrue(all(len(item["summary"]) <= 900 for item in result["reports"]))
        self.assertEqual(result["reports"][0]["truncated_fields"], ["summary", "tests", "owner_decisions"])
        # With normal short findings, the selection reaches the fixed cap.
        with self.store.connect() as db:
            db.execute("DELETE FROM events")
            db.execute("DELETE FROM jobs")
        for deal in range(10, 15):
            self.job([case(deal=deal)])
        self.assertEqual(len(json.loads(self.context())["reports"]), 3)

    def test_scan_is_twenty_events_and_oversized_malformed_or_future_sources_are_skipped(self):
        self.job()
        for _ in range(20):
            self.job([case(symbol=8)])
        self.assertEqual(self.context(), "", "applicable old evidence is outside bounded scan")
        with self.store.connect() as db:
            db.execute("DELETE FROM events")
            db.execute("DELETE FROM jobs")
        self.job(result="x" * (memory.MAX_REPORT_BYTES + 1))
        self.job(saved_task=reviews.PREFIX + "x" * reviews.MAX_TASK)
        jid = self.job()
        self.event(jid, report(), "2999-01-01T00:00:00Z")
        self.assertEqual(self.context(), "")

    def test_group_case_identities_survive_and_private_task_fields_do_not_cross(self):
        first, second = case(), case(account=2)
        first["balance"] = "PRIVATE_CASH"
        first["original_entry"] = {"notes": "PRIVATE_NOTE"}
        self.job([first, second])
        value = self.context()
        result = json.loads(value)["reports"][0]
        self.assertEqual([item["account_id"] for item in result["cases"]], [1, 2])
        self.assertNotIn("PRIVATE", value)
        self.assertEqual(result["matched_case_ids"], [first["case_id"]])

    def test_no_memory_has_no_store_call_and_valid_memory_only_reads_existing_tables(self):
        with patch.object(self.store, "connect", side_effect=AssertionError("no applicable task")):
            self.assertEqual(memory.context(self.store, "ordinary task", 1), "")
            self.assertEqual(memory.context(self.store, task([case()]), True), "")
        self.job()
        with self.store.connect() as db:
            before = [tuple(row) for row in db.execute("SELECT * FROM events")]
            tables = [row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")]
        # Existing bridge imports urllib; reuse must never call its remote path.
        with patch.object(reviews.urllib.request, "urlopen", side_effect=AssertionError("no network")):
            self.assertTrue(self.context())
        with self.store.connect() as db:
            self.assertEqual([tuple(row) for row in db.execute("SELECT * FROM events")], before)
            self.assertEqual([row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")], tables)


if __name__ == "__main__":
    unittest.main()
