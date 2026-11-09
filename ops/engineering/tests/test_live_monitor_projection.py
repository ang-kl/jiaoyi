"""Focused failure-mode tests for the read-only owner checkpoint projection."""
import copy
from datetime import datetime, timedelta, timezone
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from monitoring.projection import ACCOUNTS, CORE_SYMBOLS, build_snapshot

NOW = datetime(2026, 11, 9, 2, 20, tzinfo=timezone.utc)
START = NOW - timedelta(hours=1)
OPEN = "2026-11-09T00:00:00Z"
AT = (NOW-timedelta(seconds=1)).isoformat()


def health():
    return {"strategy3": [
        {"account_id": account, "prereg_hash": "current",
         "owner_pause": {"paused": False},
         "liveness": {"fast_monitor": {"session_connected": True, "state": "observing",
                      "last_watchdog_completed_at": AT, "last_quote_processed_receipt_at": AT,
                      "last_quote_received_at": AT}},
         "preopen_preparation": {"symbols": [
             {"symbol": symbol, "session_relation": "current", "session_open": OPEN,
              "state": "prepared", "base_preparation_ready": True} for symbol in CORE_SYMBOLS]},
         "trigger_distance": {symbol: {"at": AT, "bid": 105., "ask": 105.2,
                                       "open": 100., "reason": None} for symbol in CORE_SYMBOLS}}
        for account in ACCOUNTS]}


def sql():
    return {"status": "complete", "window_start": START.isoformat(), "observed_at": NOW.isoformat(),
            "accounts": {str(account): {"coverage": {"history_complete": True,
                          "persisted_intent_events_complete": True, "refusals_all_stages_complete": False},
                          "metrics": {"new_entries": 0, "refused_entries": None,
                                      "confirmed_refused_entries": 0, "closed_trades": 0,
                                      "wins": 0, "losses": 0, "breakevens": 0,
                                      "net_realized_pnl": 0}, "symbols": {}}
                         for account in ACCOUNTS}}


def checkpoint(**overrides):
    args = dict(health=health(), setup_assessments=None, operations_reports=None,
                sql_evidence=None, started_at=START, observed_at=NOW, checkpoint_number=1)
    args.update(overrides)
    return build_snapshot(**args)


class ProjectionTests(unittest.TestCase):
    def test_unavailable_sources_are_unknown_not_zero_or_false_success(self):
        report = checkpoint(health={"error": "HTTP500"}, sql_evidence={"status": "unavailable"})
        self.assertFalse(report["system_health"]["all_4_accounts_connected_and_unpaused"])
        self.assertFalse(report["system_health"]["connection_evidence_complete"])
        self.assertIsNone(report["system_health"]["max_dispatch_latency_ms"])
        self.assertEqual(len(report["symbol_state_matrix_8_core"]), 8)
        self.assertEqual(len(report["accounts_summary_24h"]), 4)
        for item in report["accounts_summary_24h"].values():
            for key in ("open", "entered", "refused", "closed", "wins", "realized_pnl", "unrealized_pnl"):
                self.assertIsNone(item[key], key)
        for item in report["symbol_state_matrix_8_core"].values():
            self.assertEqual(item["state"], "Unknown")
            self.assertIsNone(item["mid"])
        self.assertFalse(report["authority"]["broker_actions"])

    def test_accounts_keep_different_states_and_stale_quotes_or_expired_arms_do_not_qualify(self):
        h, e = health(), sql()
        arm = {"armed_at": (NOW-timedelta(minutes=20)).isoformat(), "session": {"open": OPEN, "close": "2026-11-09T04:00:00Z"},
               "origin": 100., "extreme": 110., "level": 105., "impulse_bps": 1000., "impulse_range": 10.,
               "long": True, "macro_context": {"drift_5": .02, "drift_20": .03, "latest_bar_end": "2026-11-08T04:00:00Z"}}
        e["accounts"]["43002148"]["symbols"]["BTCUSD"] = {
            "latest_arm": {"at": AT, "kind": "s3_pullback_state", "detail": {"prereg_hash": "current", "state": arm}}}
        e["accounts"]["46130058"]["symbols"]["BTCUSD"] = {
            "latest_decision": {"at": AT, "kind": "s3_pullback_assessment", "reason": "vetoed_by_5d_20d_macro_trend",
                                "detail": {"session": {"open": OPEN}, "detail": {"macro_context": {"drift_5": -.03, "drift_20": -.02}}}}}
        h["strategy3"][2]["trigger_distance"]["BTCUSD"]["reason"] = "opening_midpoint_unavailable"
        stale = h["strategy3"][3]["trigger_distance"]["BTCUSD"]
        stale["at"] = (NOW-timedelta(minutes=10)).isoformat()
        expired = copy.deepcopy(arm)
        expired["armed_at"] = (NOW-timedelta(hours=5)).isoformat()
        e["accounts"]["49236051"]["symbols"]["BTCUSD"] = {
            "latest_arm": {"at": AT, "detail": {"prereg_hash": "current", "state": expired}}}
        h["strategy3"][0]["trigger_distance"]["BTCUSD"]["open"] = 999.
        report = checkpoint(health=h, sql_evidence=e)
        btc = report["symbol_state_matrix_8_core"]["BTCUSD"]
        self.assertEqual(btc["state"], "Mixed")
        self.assertIsNone(btc["mid"])
        self.assertEqual(btc["accounts"]["43002148"]["state"], "ArmedPullbackWaiting50Pct")
        self.assertEqual(btc["accounts"]["43002148"]["open_mid"], 100., "an armed setup retains its original opening reference")
        self.assertEqual(btc["accounts"]["46130058"]["state"], "VetoedByMacro5d20d")
        self.assertEqual(btc["accounts"]["47790949"]["state"], "BlockedMissingOpeningReference")
        self.assertEqual(btc["accounts"]["49236051"]["state"], "Unknown")
        self.assertTrue(btc["accounts"]["49236051"]["quote_stale_or_unavailable"])
        self.assertIsNone(btc["accounts"]["49236051"]["mid"])
        # A prior-window veto must not label the new window's current state.
        e["accounts"]["46130058"]["symbols"]["BTCUSD"]["latest_decision"]["detail"]["session"]["open"] = "2026-11-08T20:00:00Z"
        changed = checkpoint(health=h, sql_evidence=e)
        self.assertEqual(changed["symbol_state_matrix_8_core"]["BTCUSD"]["accounts"]["46130058"]["state"], "WatchingImpulse")

    def test_incomplete_ledgers_remain_null_and_unrealized_requires_broker_provenance(self):
        e = sql()
        a = e["accounts"]["43002148"]
        a["coverage"]["history_complete"] = False
        a["metrics"].update({"closed_trades": None, "wins": None, "losses": None, "breakevens": None,
                             "net_realized_pnl": None, "confirmed_refused_entries": 2,
                             "refusal_reasons": {"ENTRY_FX_UNAVAILABLE: missing path": 2},
                             "observed_closed_trades": 1, "observed_closed_net_pnl": 20.})
        a["account_snapshot"] = {"observed_at": AT, "detail": {"balance": 1000., "equity": 1100., "float": 100.}}
        report = checkpoint(sql_evidence=e)
        summary = report["accounts_summary_24h"]["43002148_live"]
        self.assertIsNone(summary["refused"])
        self.assertEqual(summary["confirmed_refusal_lower_bound"], 2)
        self.assertEqual(summary["refusal_reasons"]["ENTRY_FX_UNAVAILABLE: missing path"], 2)
        self.assertIsNone(checkpoint(checkpoint_number=None)["checkpoint_number"])
        self.assertIsNone(summary["closed"])
        self.assertIsNone(summary["realized_pnl"])
        self.assertIsNone(summary["unrealized_pnl"])
        a["account_snapshot"]["detail"].update({"source_complete": True, "pnl_coverage_complete": True,
            "balance_stable": True, "equity_basis": "balance_plus_broker_net_unrealized_excluding_unverified_credit"})
        report = checkpoint(sql_evidence=e)
        self.assertEqual(report["accounts_summary_24h"]["43002148_live"]["unrealized_pnl"], 100.)
        a["account_snapshot"]["observed_at"] = (NOW-timedelta(minutes=3)).isoformat()
        self.assertIsNone(checkpoint(sql_evidence=e)["accounts_summary_24h"]["43002148_live"]["unrealized_pnl"])


if __name__ == "__main__":
    unittest.main()
