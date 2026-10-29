"""Three acceptance journeys through real coordinator, queue and report code.

Only HTTP, Telegram, inference and remote release boundaries are simulated. These
tests do not place orders or use a production account/database.
"""
import copy
import contextlib
import datetime as dt
import html
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
import urllib.request
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "worker.py"
spec = importlib.util.spec_from_file_location("incident_acceptance_worker", MODULE)
w = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w)

import day_review
import incidents
import reporting


class IncidentAcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.env = {
            "ENGINEERING_STATE_DIR": str(self.root), "PATH": os.environ["PATH"],
            "ENGINEERING_ENABLED": "1", "ENGINEERING_AUTH_MODE": "chatgpt",
            "GH_TOKEN": "fixture-gh", "RAILWAY_TOKEN": "fixture-railway",
            "ENGINEERING_DAILY_SESSION_LIMIT": "36",
            "ENGINEERING_BUDGET_ACK": "session-limits-are-not-a-dollar-cap",
            "TELEGRAM_BOT_TOKEN": "fixture-telegram", "TELEGRAM_OWNER_CHAT_ID": "313940231",
        }
        self.c = w.Coordinator(w.Config(self.env))
        self.no_network = patch.object(
            urllib.request, "urlopen", side_effect=AssertionError("unexpected network access"))
        self.no_network.start()

    def tearDown(self):
        self.no_network.stop()
        self.temp.cleanup()

    def health(self, position=7, symbol="TEST", digest="a" * 64):
        return {"status": "ok", "strategy3": [{
            "account_id": 49236051, "prereg_hash": digest,
            "liveness": {"state": "alive", "fast_monitor": {
                "state": "observing", "unresolved_open_count": 0}},
            "open_positions": [{"position_id": position, "symbol": symbol, "stop": None}],
        }]}

    def operations(self, at=None):
        return {"as_of": (at or dt.datetime.now(dt.timezone.utc)).isoformat(), "reports": []}

    def full_incident(self, coordinator=None, symbol="TEST"):
        coordinator = coordinator or self.c
        state = json.loads(coordinator.store.last_event("incident_state") or "{}")
        return next(row for row in state.values() if row.get("symbol") == symbol)

    def job(self, jid):
        return next(row for row in self.c.store.jobs() if row["id"] == jid)

    def proven_report(self, incident, paths=None, disposition="proven_defect"):
        return {"outcome": "no_change", "acceptance_passed": False,
                "summary": "A reproducible evidence-adapter defect requires a bounded repair.",
                "tests": "Controlled negative and positive evidence demonstrate the defect.",
                "owner_decisions": "No trading-policy change is requested.",
                "incident": {"id": incident["id"], "disposition": disposition,
                    "evidence_ids": incident["evidence_ids"],
                    "repair_proposal": {
                        "summary": "Correct the original-decision evidence projection without changing execution.",
                        "paths": paths or ["crates/jiaoyi/src/stop_loss_review.rs"],
                        "acceptance_checks": [
                            "Missing original mark remains unknown and a valid original mark is preserved."]}}}

    def test_report_without_model_allowance_survives_restart_and_escalates_deadline(self):
        # The real source adapter must register/report without reserving a model
        # session, even when the model lane is busy and the allowance is zero.
        self.c.config.base_limit = 0
        self.c.active_session = {"id": 999, "at": w.now(), "started": 0, "timeout": 300}
        burst = self.health()
        burst["strategy3"][0]["open_positions"] += [
            {"position_id": 20 + n, "symbol": "BURST" + str(n), "stop": None}
            for n in range(3)]
        snapshots = [burst, self.operations()]
        replies = [io.BytesIO(json.dumps(value).encode()) for value in snapshots]
        with patch.object(incidents.urllib.request, "urlopen", side_effect=replies), \
             patch.object(w, "bounded_process", side_effect=AssertionError("report cannot call a model")), \
             patch.object(reporting, "send_document", side_effect=AssertionError(
                 "slow Telegram transport must not block the observer")) as send:
            self.c.incidents.poll()
        send.assert_not_called()
        incident = self.full_incident()
        self.assertEqual(incident["kind"], "protection_unverified")
        self.assertEqual(incident["state"], "analysis_pending")
        self.assertEqual(incident["delivery_state"], "created")
        self.assertEqual(self.c.store.used(), 0)
        self.assertIsNone(self.c.store.reserve(self.c.config.limit))
        reports = list((self.c.store.private / "incident-reports").glob("*.html"))
        self.assertEqual(len(reports), 4, "all factual reports precede optional slow delivery")
        received = dt.datetime.fromisoformat(incident["received_at"])
        for report in reports:
            self.assertGreaterEqual(report.stat().st_mtime - received.timestamp(), -1)
            self.assertLessEqual(report.stat().st_mtime - received.timestamp(), 60)
        html = reports[0].read_text()
        self.assertIn("№", html)
        self.assertIn("§", html)
        self.assertIn("SGT", html)
        self.assertIn("analysis", html.lower())
        self.assertNotIn("fixture-telegram", html)

        # Delivery is independently claimed only after all four files exist.
        with patch.object(reporting, "send_document", side_effect=reporting.DeliveryError(
                "acknowledgement_lost", uncertain=True)) as send:
            delivery = self.c.reporter.incident_delivery_tick()
        self.assertEqual(send.call_count, 1)
        self.assertEqual(delivery["state"], "uncertain")
        incident = json.loads(self.c.store.last_event("incident_state"))[delivery["incident_id"]]
        tracked_symbol = incident["symbol"]
        self.c.incidents.tick()
        self.assertEqual(self.full_incident(symbol=tracked_symbol)["delivery_state"], "uncertain")

        # A lost acknowledgement is never blindly replayed by a restarted
        # reporter. The pending analysis and original receipt survive restart.
        restarted = w.Coordinator(self.c.config)
        with patch.object(reporting, "send_document", side_effect=AssertionError("ambiguous replay")):
            restarted.incidents.observe(*snapshots)
            receipt = restarted.reporter.publish_incident(self.full_incident(restarted, symbol=tracked_symbol))
        self.assertEqual(receipt["state"], "uncertain")
        self.assertEqual(len(restarted.store.jobs()), 4)
        self.assertEqual(restarted.store.used(), 0)
        self.assertEqual(self.full_incident(restarted, symbol=tracked_symbol)["received_at"], incident["received_at"])

        # Missing analysis is visible after the five-minute target; it does not
        # become completed merely because reporting/observation are healthy.
        with patch.object(reporting, "send_document", return_value=42):
            restarted.incidents.tick(clock=received + dt.timedelta(seconds=301))
        overdue = self.full_incident(restarted, symbol=tracked_symbol)
        self.assertIsNotNone(overdue.get("deadline_missed_at"))
        self.assertEqual(overdue["state"], "analysis_pending")
        safe = self.health()
        safe["strategy3"][0]["open_positions"][0]["stop"] = 100
        safe["strategy3"][0]["today"] = {"trades_today": [
            {"kind": "broker_exit", "outcome": "STOP_LOSS", "symbol": "TEST"}]}
        with patch.object(reporting, "send_document", side_effect=AssertionError("normal SL spam")):
            restarted.incidents.observe(safe, self.operations(), clock=received)
        self.assertEqual(len(restarted.store.jobs()), 4)

    def test_one_unavailable_snapshot_is_not_a_new_critical_file_every_poll(self):
        # № 17,703. Versions 39–47 were one missed public read, paged again
        # after each brief success. A minute of continuous reads may clear it.
        quiet = {"status": "ok", "trading": [], "strategy3": []}
        ops = {"as_of": "2026-10-30T20:29:00+00:00", "reports": []}
        with patch.object(reporting, "send_document", return_value=70):
            self.c.incidents.observe(None, None)
            first = self.source_incident()
            reports = lambda: list((self.c.store.private / "incident-reports").glob("*.html"))
            for _ in range(8):
                self.c.incidents.observe(None, None)
            self.assertEqual(self.source_incident()["version"], first["version"])
            self.assertEqual(len(reports()), 1)
            self.c.incidents.observe(quiet, ops)
            self.c.incidents.observe(None, None)
            self.assertEqual(self.source_incident()["version"], first["version"])
            self.assertTrue(self.source_incident()["condition_present"])
            self.assertEqual(len(reports()), 1)
            for _ in range(incidents.SOURCE_RECOVERY_POLLS - 1):
                self.c.incidents.observe(quiet, ops)
            self.assertTrue(self.source_incident()["condition_present"])
            self.c.incidents.observe(quiet, ops)
            self.assertFalse(self.source_incident()["condition_present"])
            self.assertEqual(len(reports()), 1)
            self.c.incidents.observe(None, None)
            self.assertGreater(self.source_incident()["version"], first["version"])
            self.assertEqual(len(reports()), 2)

    def source_incident(self):
        state = json.loads(self.c.store.last_event("incident_state") or "{}")
        return next(row for row in state.values() if row.get("kind") == "sources_unavailable")

    def test_material_identity_and_loss_grain_are_preserved_without_hindsight_inference(self):
        # This is shaped like the real /health episode serializer: dates/hash/
        # decision mark/accounts, but no P&L. Copies must not become losses.
        row = {"symbol": "TEST", "side": "buy", "session_date": "2026-10-28",
               "mark_minutes": "30", "prereg_hash": "a" * 64,
               "accounts": [46130058, 49236051], "copies": 2,
               "last_closed_at": "2026-10-28T13:00:00Z"}
        health = {"episodes": {"rows": [row, dict(row, side="sell", mark_minutes="60")]}}
        unknown = day_review.episode_review(health)
        self.assertEqual(unknown["outcome_state"], "unknown")
        self.assertEqual(unknown["two_sided_losses"], [])
        self.assertEqual(unknown["copied_losses"], [])
        self.assertFalse(unknown["pr"])
        later = copy.deepcopy(health)
        for item in later["episodes"]["rows"]:
            item.update(session_date="2026-10-29", prereg_hash="b" * 64)
        self.assertNotEqual(unknown["signature"], day_review.episode_review(later)["signature"])
        measured = copy.deepcopy(health)
        measured["episodes"]["rows"][0]["outcomes"] = [
            {"account_id": a, "state": "measured", "net_r": -1} for a in row["accounts"]]
        measured["episodes"]["rows"][1]["outcomes"] = [
            {"account_id": a, "state": "measured", "net_r": 1} for a in row["accounts"]]
        reading = day_review.episode_review(measured)
        self.assertEqual(reading["copied_losses"], ["TEST"])
        self.assertEqual(reading["two_sided_losses"], [])
        uncopied = copy.deepcopy(measured)
        for item, account in zip(uncopied["episodes"]["rows"], row["accounts"]):
            item["accounts"] = [account]
            item["outcomes"] = [x for x in item["outcomes"] if x["account_id"] == account]
        self.assertEqual(day_review.episode_review(uncopied)["copied_losses"], [])

        # One verified opening with two close rows remains one opening. Closing
        # channel is not substituted for original program ownership.
        position = {"ownership_verified": True, "account_id": 49236051,
            "position_id": 7, "entry_key": "single", "symbol": "TEST", "side": "buy",
            "session_date": "2026-10-28", "prereg_hash": "a" * 64,
            "position_complete": True, "net_r": -1, "mark_minutes": "30",
            "opened_at": "2026-10-28T10:00:00Z", "rule_effective_at": "2026-10-28T09:00:00Z",
            "one_entry_rule_in_force": True, "rule_prereg_hash": "a" * 64}
        partials = [dict(position, deal_id=1, channel="iPhone"), dict(position, deal_id=2)]
        reading = day_review.statement_day(partials, one_entry_in_force=True)
        self.assertEqual(reading["repeats"], [])
        self.assertFalse(reading["pr"])

        # The durable incident identity changes for actual new position/hash
        # evidence, not for polling-clock churn. No inference is reserved here.
        with patch.object(reporting, "send_document", return_value=41):
            self.c.incidents.observe(self.health(), self.operations())
            initial = self.full_incident()
            self.c.incidents.observe(self.health(), self.operations())
            self.assertEqual(len(self.c.store.jobs()), 1)
            self.c.incidents.observe(self.health(position=8, digest="b" * 64), self.operations())
        current = self.full_incident()
        self.assertNotEqual(current["fingerprint"], initial["fingerprint"])
        self.assertNotEqual(current["analysis_job"], initial["analysis_job"])
        self.assertEqual(self.c.store.used(), 0)

        # Material updates must preserve a repair that has already reached the
        # remote release path. Losing this link can create a competing fix/PR.
        with self.subTest("material update retains a published repair"), \
             patch.object(reporting, "send_document", return_value=42):
            self.c.finish_review(self.job(current["analysis_job"]), self.proven_report(current))
            linked = self.full_incident()
            repair_id = linked["repair_job"]
            self.c.store.update(repair_id, "ci_wait", pr=987, head="e" * 40)
            self.c.incidents.observe(self.health(position=9, digest="c" * 64), self.operations())
            updated = self.full_incident()
            self.assertEqual(updated["repair_job"], repair_id)
            self.assertEqual(self.job(repair_id)["pr"], 987)
            self.c.finish_review(self.job(updated["analysis_job"]), self.proven_report(updated))
            self.assertEqual(self.full_incident()["repair_job"], repair_id)
            self.assertEqual(len([j for j in self.c.store.jobs()
                if j["task"].startswith(incidents.REPAIR_PREFIX)]), 1)

            healthy = self.health(position=9, digest="c" * 64)
            healthy["strategy3"][0]["open_positions"][0]["stop"] = 100
            self.c.incidents.observe(healthy, self.operations())
            self.assertNotEqual(self.full_incident()["state"], "closed",
                "healthy source is not task-specific verification of a published repair")
            self.assertEqual(self.full_incident()["repair_job"], repair_id)

        # A complete healthy poll establishes absence, not permanent suppression
        # of the same fault when it returns with otherwise identical evidence.
        with self.subTest("same fault after observed absence is a new occurrence"), \
             patch.object(reporting, "send_document", return_value=43) as send:
            recurring = self.health(position=10, symbol="RECURRENT")
            self.c.incidents.observe(recurring, self.operations())
            previous = self.full_incident(symbol="RECURRENT")
            healthy = copy.deepcopy(recurring)
            healthy["strategy3"][0]["open_positions"][0]["stop"] = 100
            self.c.incidents.observe(healthy, self.operations())
            reports_before_return = len(list((self.c.store.private / "incident-reports").glob("*.html")))
            self.c.incidents.observe(recurring, self.operations())
            returned = self.full_incident(symbol="RECURRENT")
            self.assertGreater(returned["version"], previous["version"])
            self.assertNotEqual(returned["analysis_job"], previous["analysis_job"])
            self.assertGreater(len(list((self.c.store.private / "incident-reports").glob("*.html"))),
                               reports_before_return)

        # Match the Rust today() serializer's field names rather than inventing
        # an adapter-only time field. Recorded rejection time must reach evidence.
        with self.subTest("real action serializer timestamp survives projection"), \
             patch.object(reporting, "send_document", return_value=44):
            action_at = "2026-10-28T20:00:00+00:00"
            action = self.health(symbol="AMENDMENT")
            action["strategy3"][0]["open_positions"] = []
            action["strategy3"][0]["today"] = {"trades_today": [{
                "at": action_at, "symbol": "AMENDMENT", "kind": "trail", "side": "buy",
                "outcome": "ORDER_REJECTED", "price": None, "volume": None,
                "order_to_fill_ms": 120, "slippage": None}]}
            self.c.incidents.observe(action, self.operations())
            amendment = self.full_incident(symbol="AMENDMENT")
            self.assertEqual(amendment["kind"], "amendment_failed")
            self.assertEqual(amendment["evidence"]["action_at"], action_at)

        # A bounded source may contain several rejected amendments/unknown
        # positions for one symbol. Its order is not new economic evidence.
        with self.subTest("multiple source rows are coalesced and order independent"):
            multiple = self.health(symbol="MULTIACTION")
            multiple["strategy3"][0]["open_positions"] = []
            multiple["strategy3"][0]["today"] = {"trades_today": [
                {"at": "2026-10-28T20:00:00Z", "symbol": "MULTIACTION", "kind": "trail",
                 "outcome": "ORDER_REJECTED", "order_to_fill_ms": 120},
                {"at": "2026-10-28T20:01:00Z", "symbol": "MULTIACTION", "kind": "trail",
                 "outcome": "ORDER_REJECTED", "order_to_fill_ms": 121}]}
            before = len(self.c.store.jobs())
            self.c.incidents.observe(multiple, self.operations())
            first = self.full_incident(symbol="MULTIACTION")
            self.assertEqual(len(self.c.store.jobs()), before + 1)
            multiple["strategy3"][0]["today"]["trades_today"].reverse()
            self.c.incidents.observe(multiple, self.operations())
            self.assertEqual(len(self.c.store.jobs()), before + 1)
            self.assertEqual(self.full_incident(symbol="MULTIACTION")["version"], first["version"])
            multiple["strategy3"][0]["today"]["trades_today"].append({
                "at": "2026-10-28T20:02:00Z", "symbol": "MULTIACTION", "kind": "trail",
                "outcome": "ORDER_REJECTED", "order_to_fill_ms": 122})
            self.c.incidents.observe(multiple, self.operations())
            self.assertEqual(len(self.c.store.jobs()), before + 2)

            stops = self.health(position=70, symbol="MULTISTOP")
            stops["strategy3"][0]["open_positions"].append({
                "position_id": 71, "symbol": "MULTISTOP", "stop": None})
            before = len(self.c.store.jobs())
            self.c.incidents.observe(stops, self.operations())
            first = self.full_incident(symbol="MULTISTOP")
            self.assertEqual(len(self.c.store.jobs()), before + 1)
            stops["strategy3"][0]["open_positions"].reverse()
            self.c.incidents.observe(stops, self.operations())
            self.assertEqual(len(self.c.store.jobs()), before + 1)
            self.assertEqual(self.full_incident(symbol="MULTISTOP")["version"], first["version"])

    def test_nonrepeat_defect_reaches_verified_closure_and_deadline_holds_mutations(self):
        with patch.object(reporting, "send_document", return_value=41):
            self.c.incidents.observe(self.health(), self.operations())
            incident = self.full_incident()
            review = self.job(incident["analysis_job"])
            report = self.proven_report(incident)
            report["summary"] = "MODEL-ANALYSIS: <script>alert('unsafe')</script> evidence adapter mismatch."
            result = self.c.finish_review(review, report)
            self.c.finish_review(review, report)
        analysis_html = "\n".join(path.read_text() for path in
            (self.c.store.private / "incident-reports").glob("*.html"))
        self.assertIn(html.escape(report["summary"], quote=True), analysis_html)
        self.assertIn("model-reported", analysis_html.lower())
        self.assertNotIn(report["summary"], analysis_html)
        self.assertNotIn(report["summary"], json.dumps(self.c.status()))
        self.assertNotIn(html.escape(report["summary"], quote=True), json.dumps(self.c.status()))
        repair_jobs = [j for j in self.c.store.jobs() if j["task"].startswith(incidents.REPAIR_PREFIX)]
        self.assertEqual(len(repair_jobs), 1, "a review must dispatch one evidenced non-repeat repair")
        repair = repair_jobs[0]
        self.assertEqual(result["repair_job"], repair["id"])
        self.assertIn("stop_loss_review.rs", repair["task"])
        self.assertIn("acceptance_checks", repair["task"])
        self.c.store.update(repair["id"], "completed", result=json.dumps({"outcome": "no_change"}))
        self.c.incidents.tick()
        self.assertNotEqual(self.full_incident()["state"], "closed")

        # Exercise the actual exact-head gate: a pending check cannot merge;
        # all required checks must pass before the deployed revision is verified.
        head, merged = "c" * 40, "d" * 40
        self.c.store.update(repair["id"], "ci_wait", pr=123, head=head)
        all_green = False
        merges = []
        def github(*args, **kwargs):
            if args[:3] == ("api", "--method", "PUT"):
                merges.append(args)
                self.assertTrue(kwargs.get("write"))
                self.assertIn("sha=" + head, args)
                return json.dumps({"merged": True, "sha": merged})
            if args[-1].endswith("/pulls/123"):
                return json.dumps({"state": "open", "merged": False, "base": {"ref": "main"},
                    "head": {"sha": head, "ref": "engineering/job-" + str(repair["id"]),
                             "repo": {"full_name": w.REPO}}})
            self.assertIn("check-runs", args[-1])
            return json.dumps({"check_runs": [{"name": name,
                "status": "completed" if all_green or name != "docker" else "in_progress",
                "conclusion": "success" if all_green or name != "docker" else None,
                "app": {"slug": "github-actions"}} for name in ("check", "docker", "engineering")]})
        with patch.object(self.c, "gh", side_effect=github):
            self.c.release(self.job(repair["id"]))
            self.assertEqual(merges, [])
            self.assertEqual(self.job(repair["id"])["state"], "ci_wait")
            all_green = True
            self.c.release(self.job(repair["id"]))
        self.assertEqual(len(merges), 1)
        deployment = {"data": {"serviceInstance": {"latestDeployment": {
            "id": "verified-deployment", "status": "SUCCESS", "meta": {"commitHash": merged}}}}}
        with patch.object(w.urllib.request, "urlopen", side_effect=[
                io.BytesIO(json.dumps(deployment).encode()), io.BytesIO(b'{"status":"ok"}')]):
            self.c.release(self.job(repair["id"]))
        self.assertEqual(self.job(repair["id"])["state"], "verification_queued")
        self.c.incidents.tick()
        self.assertNotEqual(self.full_incident()["state"], "closed")
        self.c.store.update(repair["id"], "completed", result=json.dumps({
            "outcome": "verified", "tests": "The affected missing-mark path and valid-mark path passed."}))
        with patch.object(reporting, "send_document", return_value=42):
            self.c.incidents.tick()
        self.assertEqual(self.full_incident()["state"], "closed")

        # A genuine policy decision or protected-path proposal stays held; the
        # engineering worker never silently converts it to a trading-rule repair.
        with patch.object(reporting, "send_document", return_value=43):
            for symbol, paths, disposition in [
                    ("UNSAFE", ["config/settings-changes.json"], "proven_defect"),
                    ("POLICY", None, "owner_policy")]:
                self.c.incidents.observe(self.health(position=10, symbol=symbol), self.operations())
                item = self.full_incident(symbol=symbol)
                outcome = self.c.finish_review(self.job(item["analysis_job"]),
                    self.proven_report(item, paths=paths, disposition=disposition))
                self.assertIsNone(outcome.get("repair_job"))
        self.assertEqual(len([j for j in self.c.store.jobs()
                              if j["task"].startswith(incidents.REPAIR_PREFIX)]), 1)

        with self.subTest("old repair proof cannot close a materially newer fault"), \
             patch.object(reporting, "send_document", return_value=45):
            old_source = self.health(position=80, symbol="STALE_REPAIR")
            self.c.incidents.observe(old_source, self.operations())
            old_incident = self.full_incident(symbol="STALE_REPAIR")
            dispatched = self.c.finish_review(self.job(old_incident["analysis_job"]),
                self.proven_report(old_incident))
            old_repair = dispatched["repair_job"]
            self.c.store.update(old_repair, "ci_wait", pr=222, head="f" * 40)
            new_source = self.health(position=81, symbol="STALE_REPAIR", digest="b" * 64)
            self.c.incidents.observe(new_source, self.operations())
            latest = self.full_incident(symbol="STALE_REPAIR")
            self.assertNotEqual(latest["fingerprint"], old_incident["fingerprint"])
            self.assertEqual(latest["repair_job"], old_repair)
            # Simulate the trusted verification boundary for the old evidence;
            # it has not demonstrated the newer position/hash's behavior.
            self.c.store.update(old_repair, "completed", merged="a" * 40,
                result=json.dumps({"outcome": "verified", "tests": "Original fault only."}))
            self.c.incidents.tick()
            self.assertNotEqual(self.full_incident(symbol="STALE_REPAIR")["state"], "closed")

        with self.subTest("a malformed review remains unresolved"), \
             patch.object(reporting, "send_document", return_value=46):
            self.c.incidents.observe(self.health(position=90, symbol="MALFORMED"), self.operations())
            item = self.full_incident(symbol="MALFORMED")
            malformed = self.proven_report(item)
            malformed.pop("incident")
            result = self.c.finish_review(self.job(item["analysis_job"]), malformed)
            self.assertFalse(result.get("applied"))
            self.assertEqual(self.job(item["analysis_job"])["state"], "verification_pending")
            self.assertNotEqual(self.full_incident(symbol="MALFORMED")["state"], "closed")

        with self.subTest("normal execution does not close an unresolved economic goal"):
            current = self.health(digest="b" * 64)
            current["strategy3"][0]["open_positions"][0]["stop"] = 100
            operations = self.operations()
            operations["reports"] = [{"account_id": 49236051, "strategy": "s3", "report": {
                "assessment": {"state": "below_target", "history_complete": True, "window": {
                    "position_ids": list(range(101, 121)), "count": 20, "wins": 1,
                    "win_rate": 0.05, "net_r": -16, "realized_payoff": 0.25,
                    "last_closed_at": "2026-10-27T13:00:00Z"}}}}]
            self.c.incidents.observe(current, operations)
            state = json.loads(self.c.store.last_event("incident_state"))
            goal = next(item for item in state.values() if item["kind"] == "goal_below_target")
            self.assertEqual(goal["evidence"]["current_prereg_hash"], "b" * 64)
            self.assertNotIn("prereg_hash", goal["evidence"],
                "the running hash is not the historical position cohort")
            self.assertIn("not supplied", goal["evidence"]["cohort_registration_scope"])
            assessment = self.proven_report(goal, disposition="normal_loss")
            assessment["summary"] = "Original executions followed their rules; the mixed historical window remains below target."
            assessment["incident"]["repair_proposal"] = None
            outcome = self.c.finish_review(self.job(goal["analysis_job"]), assessment)
            goal_after = json.loads(self.c.store.last_event("incident_state"))[goal["id"]]
            self.assertNotEqual(goal_after["state"], "closed")
            self.assertTrue(goal_after["next_action"])
            self.assertIsNone(outcome.get("repair_job"))

        cutoff = w.Config(dict(self.env, ENGINEERING_WORK_UNTIL="2026-10-29T07:30:00+08:00"))
        self.assertTrue(cutoff.mutation_allowed("2026-10-29T07:29:59+08:00"))
        self.assertFalse(cutoff.mutation_allowed("2026-10-29T07:30:00+08:00"))
        held = self.c.store.enqueue(incidents.REPAIR_PREFIX + " a held repair")
        read_only = self.c.store.enqueue(incidents.PREFIX + " a read-only analysis")
        reserved = self.c.store.reserve(36, allow_mutation=False)
        self.assertTrue(reserved["task"].startswith(incidents.PREFIX))
        self.assertNotEqual(reserved["id"], held)
        self.assertEqual(self.job(held)["state"], "queued")
        with patch.object(self.c.config, "mutation_allowed", return_value=False), \
             patch.object(self.c, "gh", side_effect=AssertionError("merge after cutoff")):
            self.c.store.update(repair["id"], "ci_wait")
            self.c.release(self.job(repair["id"]))
            with self.assertRaisesRegex(RuntimeError, "window|deadline|closed"):
                self.c.command(["unreachable-mutating-command"], write=True)
            with patch.object(reporting, "send_document", return_value=44):
                self.c.incidents.observe(self.health(position=11, symbol="HELD"), self.operations())
                item = self.full_incident(symbol="HELD")
                held_result = self.c.finish_review(self.job(item["analysis_job"]), self.proven_report(item))
        self.assertEqual(held_result["state"], "repair_held")
        self.assertIsNone(held_result.get("repair_job"))

        # Capture the real verifier's prompt after an expired work window.
        # Source/HTTP inspection remains possible, but the earlier generic
        # regression-test permission must be explicitly overridden.
        with self.subTest("post-cutoff verification cannot invite a compiler"):
            import test_verification as verification_fixtures
            check = verification_fixtures.VerificationTests(methodName="runTest")
            check.setUp()
            try:
                repo, job = check.fixture()
                check.config.work_until = dt.datetime(2000, 1, 1, tzinfo=dt.timezone.utc)
                def inspect_closed_prompt(args, options):
                    self.assertEqual(args[0], "codex")
                    self.assertIn("OWNER WORK WINDOW CLOSED.", args[-1])
                    final = args[-1].rsplit("OWNER WORK WINDOW CLOSED.", 1)[1]
                    self.assertIn("Do not run tests, compilers, cargo", final)
                    self.assertIn("cached", final)
                    self.assertIn("HTTP evidence only", final)
                report = {"outcome": "verification_pending", "acceptance_passed": False,
                    "summary": "Cached/source/HTTP evidence lacks the affected positive proof.",
                    "tests": "No tests, compilers or builds were run after the work window.",
                    "owner_decisions": "Resume engineering before any new build is attempted."}
                check.execute(repo, job, report, inspect_closed_prompt)
                self.assertEqual(check.c.store.jobs()[0]["state"], "verification_pending")
            finally:
                check.tearDown()


if __name__ == "__main__":
    unittest.main()
