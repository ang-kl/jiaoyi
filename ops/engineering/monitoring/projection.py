"""Read-only, pure projections of bounded live-monitor evidence (owner №18,393).

No broker calls, database access, order authority, synthetic P&L or success
substitution live here. Account differences and unavailable data remain visible.
"""
from __future__ import annotations

from datetime import datetime, timedelta, timezone
import math
from typing import Any
from zoneinfo import ZoneInfo

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
CORE_SYMBOLS = ("SpotCrude", "SpotBrent", "US500", "NAS100", "XAUUSD", "XAGUSD", "BTCUSD", "ETHUSD")
UTC = timezone.utc


def obj(value: Any) -> dict:
    return value if isinstance(value, dict) else {}


def rows(value: Any) -> list:
    return value if isinstance(value, list) else []


def number(value: Any) -> float | None:
    if value is None or isinstance(value, bool):
        return None
    try:
        result = float(value)
    except (TypeError, ValueError, OverflowError):
        return None
    return result if math.isfinite(result) else None


def count(value: Any) -> int | None:
    value = number(value)
    return int(value) if value is not None and value >= 0 and value.is_integer() else None


def instant(value: Any) -> datetime | None:
    try:
        parsed = value if isinstance(value, datetime) else datetime.fromisoformat(str(value).replace("Z", "+00:00"))
        return parsed.astimezone(UTC) if parsed.tzinfo else None
    except (TypeError, ValueError, OverflowError):
        return None


def iso(value: datetime) -> str:
    return value.astimezone(UTC).isoformat().replace("+00:00", "Z")


def age(value: Any, now: datetime) -> float | None:
    at = instant(value)
    return (now - at).total_seconds() if at is not None and at <= now else None


def fresh(value: Any, now: datetime, seconds: float) -> bool:
    elapsed = age(value, now)
    return elapsed is not None and elapsed <= seconds


def account_rows(payload: Any, key: str = "accounts") -> dict[int, dict]:
    result = {}
    for row in rows(obj(payload).get(key)):
        if not isinstance(row, dict):
            continue
        try:
            account = int(row.get("account_id"))
        except (ValueError, TypeError):
            continue
        if account in ACCOUNTS:
            result[account] = row
    return result


def same(values: list) -> Any:
    return values[0] if values and all(v == values[0] for v in values) else None


def _unwrap(value: Any) -> dict:
    value = obj(value)
    return obj(value.get("Ok")) if "Ok" in value else value


def _current_session(health: dict, symbol: str) -> dict:
    return next((r for r in rows(obj(health.get("preopen_preparation")).get("symbols"))
                 if obj(r).get("symbol") == symbol and r.get("session_relation") == "current"), {})


def _scope(record: Any, policy: Any, session_open: Any, now: datetime) -> dict:
    record = obj(record)
    at = instant(record.get("at") or record.get("recorded_at"))
    if at is None or at > now:
        return {}
    detail = obj(record.get("detail"))
    recorded_hash = detail.get("prereg_hash")
    if recorded_hash and policy and recorded_hash != policy:
        return {}
    recorded_open = obj(detail.get("session")).get("open") or detail.get("open_at")
    if recorded_open and session_open and instant(recorded_open) != instant(session_open):
        return {}
    return record


def _reason_state(reason: Any) -> str | None:
    reason = str(reason or "").lower()
    if "opening_midpoint" in reason or "opening reference" in reason:
        return "BlockedMissingOpeningReference"
    if "vetoed_by_5d_20d" in reason or "macro_trend" in reason:
        return "VetoedByMacro5d20d"
    if "vetoed_by_peer" in reason or "peer_conflict" in reason or "peer_disagree" in reason or "peer_coherence" in reason:
        return "VetoedByPeer"
    if "impulse_below" in reason or "no_impulse" in reason or "inside_band" in reason:
        return "WatchingImpulse"
    if "cost_ge_r" in reason:
        return "VetoedByCost"
    if "market_closed" in reason or "outside production" in reason:
        return "MarketClosed"
    return None


def symbol_projection(*, health: dict, symbol: str, evidence: dict, assessment: dict, now: datetime) -> dict:
    trigger = obj(obj(health.get("trigger_distance")).get(symbol))
    current = _current_session(health, symbol)
    window_open = current.get("session_open")
    policy = health.get("prereg_hash")
    session = _scope(evidence.get("latest_macro"), policy, window_open, now)
    decision = _scope(evidence.get("latest_decision"), policy, window_open, now)
    state_record = obj(evidence.get("latest_arm"))
    state_detail = obj(state_record.get("detail"))
    arm = obj(state_detail.get("state"))
    arm_at = instant(arm.get("armed_at"))
    arm_session = obj(arm.get("session"))
    expiry = (arm_at + timedelta(hours=4) if symbol in ("BTCUSD", "ETHUSD") and arm_at
              else instant(arm_session.get("close")))
    arm_record_at = instant(state_record.get("at"))
    arm_valid = bool(arm and arm_at and arm_at <= now and expiry and now < expiry
                     and arm_record_at and arm_record_at <= now and policy
                     and state_detail.get("prereg_hash") == policy)
    if not arm_valid:
        arm = {}
    detail = obj(decision.get("detail"))
    nested = obj(detail.get("detail"))
    sd = obj(session.get("detail"))
    context = (obj(arm.get("macro_context")) or _unwrap(nested.get("macro_context"))
               or _unwrap(sd.get("macro_context")) or _unwrap(obj(sd.get("detail")).get("macro_context")))
    band = obj(nested.get("band"))
    opening = number(arm.get("origin")) if arm_valid else number(trigger.get("open"))
    if opening is None:
        opening = number(sd.get("open"))
    impulse = number(arm.get("impulse_bps"))
    if impulse is None:
        impulse_quote = obj(nested.get("impulse_quote"))
        ib, ia = number(impulse_quote.get("bid")), number(impulse_quote.get("ask"))
        origin = number(band.get("open"))
        if ib is not None and ia is not None and origin and ia >= ib > 0:
            impulse = abs((ib + ia) / 2 - origin) / origin * 10000
    bid, ask = number(trigger.get("bid")), number(trigger.get("ask"))
    quote_at = trigger.get("at")
    quote_fresh = fresh(quote_at, now, 6) and bid is not None and ask is not None and ask >= bid > 0
    reason = trigger.get("reason")
    decision_reason = decision.get("reason")
    state = "Unknown"
    state_at = quote_at
    state_source = "/health.trigger_distance"
    if reason:
        state = _reason_state(reason) or "BlockedDataUnavailable"
    elif opening is not None and quote_fresh:
        state = "WatchingImpulse"
    if not reason and decision:
        mapped = _reason_state(decision_reason)
        if mapped:
            state, state_at, state_source = mapped, decision.get("at"), "sql.latest_assessment"
    if arm_valid:
        state, state_at, state_source = "ArmedPullbackWaiting50Pct", state_record.get("at"), "sql.latest_pullback_state"
    # Only a fresh complete owner broker inventory can assert a current position.
    position_rows = []
    if assessment.get("evidence_complete") is True and fresh(assessment.get("assessed_at"), now, 15) and fresh(assessment.get("book_at"), now, 120):
        position_rows = [p for p in rows(assessment.get("objects")) if obj(p).get("kind") == "position"
                         and p.get("symbol") == symbol and obj(p.get("strategy")).get("strategy") in ("s3", "Strategy 3: macro pullback")]
    if position_rows:
        stages = []
        for position in position_rows:
            entry, stop = number(position.get("entry")), number(position.get("stop_loss"))
            if entry is None or stop is None or position.get("side") not in ("buy", "sell"):
                stages.append("PositionOpenProtectionUnknown")
            else:
                locked = ((position.get("side") == "buy" and stop >= entry)
                          or (position.get("side") == "sell" and stop <= entry))
                stages.append("PositionOpenStage2BreakevenLocked" if locked else "PositionOpenStage1")
        state = same(stages) or "MultiplePositionStages"
        state_at, state_source = assessment.get("book_at"), "/owner/setup-assessments"
    macro_state = "unknown"
    if state == "VetoedByMacro5d20d":
        macro_state = "veto"
    elif arm_valid:
        macro_state = "aligned_at_arm"
    result = {
        "state": state, "state_at": state_at, "state_source": state_source,
        "state_evidence_age_seconds": age(state_at, now),
        "reason": state_record.get("reason") if arm_valid else reason or decision_reason,
        "current_evaluation_reason": reason,
        "session_open": arm_session.get("open") if arm_valid else window_open,
        "current_evaluation_window_open": window_open,
        "open_mid": opening,
        "opening_reference_source_at": (obj(obj(state_detail.get("detail")).get("opening_quote")).get("observed_at") if arm_valid else obj(sd.get("opening_quote")).get("observed_at")),
        "impulse_bps": impulse, "impulse_r_bps": impulse,
        "impulse_observed_at": arm.get("armed_at") or decision.get("at"),
        "impulse_range_price": number(arm.get("impulse_range")),
        "risk_price": None,
        "risk_evidence": "No original executed risk denominator in this signal snapshot; impulse range is reported separately",
        "drift_5d": number(context.get("drift_5")), "drift_20d": number(context.get("drift_20")),
        "macro_context_latest_bar_end": context.get("latest_bar_end"), "macro_5d_20d": macro_state,
        "pullback_ref": number(arm.get("level")), "pullback_entry_ref": number(arm.get("level")),
        "arm_expires_at": iso(expiry) if arm_valid else None,
        "armed_state_recorded_at": state_record.get("at") if arm_valid else None,
        "bid": bid, "ask": ask, "quote_at": quote_at, "quote_age_seconds": age(quote_at, now),
        "quote_projection_recent": bool(quote_fresh), "quote_stale_or_unavailable": not quote_fresh,
        "quote_timestamp_basis": "actor trigger projection timestamp; individual side receipt timestamps are not exposed by this route",
        "two_sided_receipt_freshness_verified": False,
        "mid": (bid + ask) / 2 if quote_fresh else None,
        "preparation": {k: current.get(k) for k in ("state", "base_preparation_ready", "all_window_coefficients_ready", "missing_marks", "macro_context_error")},
        "entry_preparation": obj(trigger.get("entry_preparation")),
        "position_ids": [p.get("position_id") or p.get("id") for p in position_rows],
    }
    return result


def _rolling_report(operations: dict, account: int, stored: Any = None) -> dict:
    for row in rows(operations.get("reports")):
        if obj(row).get("account_id") == account and row.get("strategy") == "s3":
            assessment = obj(obj(row.get("report")).get("assessment"))
            window = obj(assessment.get("window"))
            return {"count": window.get("count"), "win_rate": number(window.get("win_rate")),
                    "assessed_at": assessment.get("assessed_at"), "state": assessment.get("state"),
                    "history_complete": assessment.get("history_complete"),
                    "missing_reasons": assessment.get("missing_reasons"), "source": "/operations/reports"}
    stored = obj(stored)
    if stored:
        window = obj(stored.get("window"))
        return {"count": window.get("count"), "win_rate": number(window.get("win_rate")),
                "assessed_at": stored.get("assessed_at"), "state": stored.get("state"),
                "history_complete": stored.get("history_complete"),
                "missing_reasons": stored.get("missing_reasons"), "source": "sql.official_latest20",
                "policy_caveat": stored.get("policy_caveat")}
    return {"count": None, "win_rate": None, "state": "unavailable", "source": "/operations/reports"}


def build_snapshot(*, health: dict | None, setup_assessments: dict | None,
                   operations_reports: dict | None, sql_evidence: dict | None,
                   started_at: datetime, observed_at: datetime, checkpoint_number: int | None,
                   previous_snapshot: dict | None = None, source_status: dict | None = None) -> dict:
    """Project one checkpoint, preserving unknowns and per-account differences."""
    start, now = instant(started_at), instant(observed_at)
    if start is None or now is None or now < start:
        raise ValueError("monitor timestamps must be timezone-aware and ordered")
    if checkpoint_number is not None and (not isinstance(checkpoint_number, int) or isinstance(checkpoint_number, bool) or checkpoint_number < 1):
        raise ValueError("checkpoint_number must be positive or None for an on-demand/completion record")
    health, operations, sql = obj(health), obj(operations_reports), obj(sql_evidence)
    by_account = account_rows(health, "strategy3")
    setup = account_rows(setup_assessments) or account_rows(health.get("setup_assessment"))
    sql_accounts = obj(sql.get("accounts"))
    projected, accounts, connection = {}, {}, {}
    for account in ACCOUNTS:
        h, a, e = by_account.get(account, {}), setup.get(account, {}), obj(sql_accounts.get(str(account)))
        symbols = obj(e.get("symbols"))
        projected[str(account)] = {symbol: symbol_projection(health=h, symbol=symbol,
            evidence=obj(symbols.get(symbol)), assessment=a, now=now) for symbol in CORE_SYMBOLS}
        monitor = obj(obj(h.get("liveness")).get("fast_monitor"))
        paused = obj(h.get("owner_pause")).get("paused")
        connected = monitor.get("session_connected")
        connection[str(account)] = {"connected": connected if isinstance(connected, bool) else None,
            "paused": paused if isinstance(paused, bool) else None,
            "actor_state": monitor.get("state"), "active_stage": monitor.get("active_stage"),
            "last_processed_quote_at": monitor.get("last_quote_processed_receipt_at"),
            "last_observer_quote_at": monitor.get("last_quote_received_at"),
            "processed_quote_age_seconds": age(monitor.get("last_quote_processed_receipt_at"), now),
            "connection_evidence_current": fresh(monitor.get("last_watchdog_completed_at"), now, 15)}
        book_current = a.get("evidence_complete") is True and fresh(a.get("assessed_at"), now, 15) and fresh(a.get("book_at"), now, 120)
        open_count = count(a.get("position_count")) if book_current else None
        metrics = obj(e.get("metrics"))
        coverage = obj(e.get("coverage"))
        complete = sql.get("status") == "complete" and instant(sql.get("window_start")) == start
        metric = lambda key: number(metrics.get(key)) if complete else None
        counter = lambda key: count(metrics.get(key)) if complete else None
        rolling = _rolling_report(operations, account, e.get("latest20") if complete else None)
        snapshot_row = obj(e.get("account_snapshot"))
        snapshot = obj(snapshot_row.get("detail")) or snapshot_row
        snapshot_at = snapshot_row.get("observed_at") or snapshot.get("observed_at")
        measured_float = (snapshot.get("source_complete") is True
                          and snapshot.get("pnl_coverage_complete") is True
                          and snapshot.get("balance_stable") is True
                          and snapshot.get("equity_basis") == "balance_plus_broker_net_unrealized_excluding_unverified_credit"
                          and fresh(snapshot_at, now, 120))
        unrealized = number(snapshot.get("float")) if measured_float else None
        environment = "live" if account == 43002148 else "demo"
        accounts[f"{account}_{environment}"] = {
            "open": open_count, "open_positions_count": open_count,
            "open_scope": "whole account broker inventory, including manual/external positions",
            "book_at": a.get("book_at"), "book_evidence_current": book_current,
            "strategy3_open_positions_count": sum(len(v["position_ids"]) for v in projected[str(account)].values()) if book_current and isinstance(a.get("objects"), list) else None,
            "armed_pullbacks_count": sum(v["state"] == "ArmedPullbackWaiting50Pct" for v in projected[str(account)].values()) if complete and isinstance(h.get("prereg_hash"), str) and h["prereg_hash"] else None,
            "entered": counter("new_entries"), "refused": counter("refused_entries"),
            "closed": counter("closed_trades"), "wins": counter("wins"), "losses": counter("losses"),
            "be": counter("breakevens"), "realized_pnl": metric("net_realized_pnl"),
            "unrealized_pnl": unrealized,
            "unrealized_pnl_reason": "broker measured account net float; excludes future exit commission" if unrealized is not None else "Current complete broker net-P&L measurement unavailable",
            "unrealized_pnl_source_at": snapshot_at, "currency": snapshot.get("currency") or e.get("currency"),
            "account_realized_pnl": metric("account_net_realized_pnl"),
            "realized_pnl_through": coverage.get("history_through"),
            "closed_position_lifetime_net_pnl": metric("closed_position_net_pnl"),
            "trade_metrics_scope": "Strategy 3 owned positions; monitoring window only",
            "rolling_20_trade_win_rate": rolling["win_rate"] if (rolling["count"] == 20
                and rolling.get("history_complete") is True
                and rolling.get("state") in ("below_target", "window_meets_numeric_reference")
                and fresh(rolling.get("assessed_at"), now, 600)) else None,
            "rolling_window_evidence": rolling, "coverage": coverage,
            "confirmed_refusal_lower_bound": counter("confirmed_refused_entries"),
            "persisted_entry_attempts": counter("persisted_entry_attempts"),
            "broker_rejected_entries": counter("broker_rejected_entries"),
            "partially_filled_entries": counter("partially_filled_entries"),
            "confirmed_missing_fx_conversion_refusals": counter("confirmed_missing_conversion_path_errors"),
            "confirmed_subminimum_volume_refusals": counter("confirmed_subminimum_volume_refusals"),
            "max_touch_to_final_guard_ms": metric("max_touch_to_final_guard_ms"),
            "touch_to_final_guard_samples": counter("samples"),
            "uncertain_entry_attempts": counter("uncertain_entry_attempts"),
            "observed_incomplete_subsets": {key: metric(key) for key in metrics if key.startswith("observed_")},
            "refusal_reasons": obj(metrics.get("refusal_reasons")) or obj(e.get("refusal_reasons")),
            "new_entries_since_start": counter("new_entries"),
            "refused_entries_since_start": counter("refused_entries"),
            "closed_trades_since_start": counter("closed_trades"),
            "breakeven_closes": counter("breakevens"),
            "realized_net_pnl_money": metric("net_realized_pnl"),
            "unrealized_pnl_money": unrealized,
        }
    matrix = {}
    for symbol in CORE_SYMBOLS:
        values = {str(account): projected[str(account)][symbol] for account in ACCOUNTS}
        state = same([v["state"] for v in values.values()]) or "Mixed"
        matrix[symbol] = {"state": state, "aggregation": "consensus only; account values are authoritative",
            **{key: same([v[key] for v in values.values()]) for key in
               ("impulse_bps", "macro_5d_20d", "pullback_ref", "mid", "open_mid", "drift_5d", "drift_20d", "bid", "ask")},
            "accounts": values}
    system_metrics = obj(sql.get("system_metrics")) if sql.get("status") == "complete" and instant(sql.get("window_start")) == start else {}
    events_after = instant(obj(previous_snapshot).get("timestamp_utc")) or start
    events = []
    for account, h in by_account.items():
        for event in rows(obj(h.get("today")).get("recent_decisions")):
            at = instant(obj(event).get("at"))
            if at and events_after < at <= now:
                events.append({"account_id": account, "at": event.get("at"), "symbol": event.get("symbol"),
                               "kind": event.get("kind"), "reason": event.get("reason")})
    events.sort(key=lambda x: (x["at"], x["account_id"]))
    return {
        "trace": "№18,393", "checkpoint_number": checkpoint_number,
        "timestamp_sgt": now.astimezone(ZoneInfo("Asia/Singapore")).strftime("%d-%m-%Y %H:%M:%S SGT"),
        "timestamp_utc": iso(now), "monitoring_started_at": iso(start),
        "elapsed_monitoring_hours": round((now-start).total_seconds()/3600, 6),
        "system_health": {
            "all_4_accounts_connected_and_unpaused": all(v["connected"] is True and v["paused"] is False and v["connection_evidence_current"] for v in connection.values()),
            "connection_evidence_complete": all(v["connected"] is not None and v["paused"] is not None for v in connection.values()),
            "accounts": connection, "max_dispatch_latency_ms": None,
            "dispatch_latency_evidence": "trigger-to-socket-write timestamps are not currently persisted; no sub-250ms claim",
            "max_touch_to_final_guard_ms": number(system_metrics.get("max_touch_to_final_guard_ms")),
            "touch_to_final_guard_samples": count(system_metrics.get("touch_to_final_guard_samples", system_metrics.get("samples"))),
            "confirmed_missing_fx_conversion_refusals": count(system_metrics.get("confirmed_missing_conversion_path_errors")),
            "confirmed_subminimum_volume_refusals": count(system_metrics.get("confirmed_subminimum_volume_refusals")),
            "refusal_count_scope": "Confirmed persisted entry attempts only; absent all-stage instrumentation keeps requested total error counts unknown",
            "fx_conversion_errors_count": count(system_metrics.get("fx_conversion_errors_count")),
            "min_volume_refusal_count": count(system_metrics.get("min_volume_refusal_count")),
            "unexpected_entry_refusals_count": count(system_metrics.get("unexpected_entry_refusals_count")),
        },
        "symbol_state_matrix_8_core": matrix, "accounts_summary_24h": accounts,
        "notable_events_since_last_checkpoint": events[-100:],
        "events_coverage": "bounded latest HTTP decisions; not an exhaustive event ledger",
        "terminal_exit_state_coverage": "Broker target versus stop attribution is not recorded by these sources; a positive/negative close is not treated as proof of target/stop hit",
        "source_status": source_status or {}, "sql_coverage": sql.get("coverage"),
        "authority": {"broker_actions": False, "strategy_changes": False, "automatic_pause": False},
        "goal": {"net_win_rate": .73, "reward_risk": 1.38, "window": 20,
                 "scope": "each strategy on each account", "consistency_proven": False},
    }
