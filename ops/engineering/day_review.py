"""Read-only observations with explicit position, session and outcome evidence.

Copies and opposite directions are topology, not proof of losses or their cause.
No result here sends an order, changes a rule, or grants general PR authority.
"""
import datetime as dt
import hashlib
import json
import math
import re


def _positive(value):
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


def _number(value):
    if isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value):
        return float(value)
    return None


def _session(value):
    if not isinstance(value, str):
        return None
    try:
        parsed = dt.date.fromisoformat(value)
    except ValueError:
        return None
    return parsed.isoformat() if value == parsed.isoformat() else None


def _time(value):
    if not isinstance(value, str):
        return None
    try:
        parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    return parsed if parsed.tzinfo is not None else None


def _hash(value):
    return value if isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) else None


def _side(value):
    return value.lower() if isinstance(value, str) and value.lower() in ("buy", "sell") else None


def _signature(rows):
    ordered = sorted(rows, key=lambda row: json.dumps(row, sort_keys=True, separators=(",", ":")))
    return hashlib.sha256(json.dumps(ordered, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()).hexdigest()


def _reading(rows, unknown=0, scope="published episodes, not a single-day loss report"):
    """Only explicit measured outcomes support descriptive loss patterns."""
    two_sided = set()
    copied = set()
    copied_losses = set()
    two_sided_losses = set()
    sides = {}
    losses = {}
    known = 0
    missing = unknown
    patterns = []
    for row in rows:
        symbol = row["symbol"]
        # Do not combine sessions/registrations to manufacture two-sided losses.
        scope_key = (symbol, row["session_date"], row["prereg_hash"])
        sides.setdefault(scope_key, set()).add(row["side"])
        if row["copies"] >= 2:
            copied.add(symbol)
        negative = set()
        for outcome in row["outcomes"]:
            if outcome["state"] != "measured":
                missing += 1
                continue
            known += 1
            if outcome["net_r"] < 0:
                negative.add(outcome["account_id"])
        if negative:
            losses.setdefault(scope_key, set()).add(row["side"])
        if len(negative) >= 2:
            copied_losses.add(symbol)
            patterns.append({"kind": "copied_side_loss", "symbol": symbol,
                             "session_date": row["session_date"], "prereg_hash": row["prereg_hash"],
                             "mark_minutes": row["mark_minutes"], "side": row["side"],
                             "accounts": sorted(negative)})
    for key, observed_sides in sides.items():
        if len(observed_sides) >= 2:
            two_sided.add(key[0])
    for key, losing_sides in losses.items():
        if len(losing_sides) >= 2:
            two_sided_losses.add(key[0])
            patterns.append({"kind": "two_sided_losses_observed", "symbol": key[0],
                             "session_date": key[1], "prereg_hash": key[2]})
    kind = ("loss_pattern" if patterns else "copy_pattern" if copied else
            "directions_observed" if two_sided else "unknown" if missing else "none")
    state = "partial" if known and missing else "measured" if known else "unknown"
    return {"kind": kind, "outcome_state": state, "evidence_scope": scope,
            "fundamental": [], "trend": [], "held": [],
            "two_sided_symbols": sorted(two_sided), "copied_symbols": sorted(copied),
            "two_sided_losses": sorted(two_sided_losses), "copied_losses": sorted(copied_losses),
            "loss_patterns": patterns, "observations": rows, "unknown_rows": unknown,
            "repeats": [], "close_or_reverse": "not_registered", "sentiment": "not_called",
            "pr": False, "signature": _signature(rows + [{"unknown_rows": unknown}]),
            "reason": ("Measured outcomes describe losses; their cause and a reversal outcome are not established."
                       if patterns else "Copies/directions are observations; absent measured P&L remains unknown.")}


def episode_review(health, session_date=None):
    """Preserve the export's episode identity; absent outcomes stay unknown.

    Optional outcomes must be [{account_id, state:'measured', net_r:finite}].
    The existing /health export without outcomes establishes only topology.
    """
    health = health if isinstance(health, dict) else {}
    source = health.get("episodes") or {}
    source = source.get("rows") or [] if isinstance(source, dict) else []
    rows = []
    unknown = 0
    wanted = _session(session_date) if session_date is not None else None
    if session_date is not None and wanted is None:
        return _reading([], 1, "invalid requested session")
    for raw in source if isinstance(source, list) else []:
        if not isinstance(raw, dict):
            unknown += 1
            continue
        session = _session(raw.get("session_date"))
        if wanted is not None and session != wanted:
            continue
        symbol, side, digest = raw.get("symbol"), _side(raw.get("side")), _hash(raw.get("prereg_hash"))
        if not isinstance(symbol, str) or not symbol or side is None or session is None or digest is None:
            unknown += 1
            continue
        accounts = sorted({account for account in raw.get("accounts", []) if _positive(account)}) if isinstance(raw.get("accounts"), list) else []
        outcomes = {}
        conflicting = set()
        for outcome in raw.get("outcomes", []) if isinstance(raw.get("outcomes"), list) else []:
            if (not isinstance(outcome, dict) or not _positive(outcome.get("account_id"))
                    or outcome.get("account_id") not in accounts):
                continue
            account = outcome["account_id"]
            net_r = _number(outcome.get("net_r"))
            measured = outcome.get("state") == "measured" and net_r is not None
            value = {"account_id": account, "state": "measured" if measured else "unknown", "net_r": net_r if measured else None}
            if account in outcomes and outcomes[account] != value:
                conflicting.add(account)
            outcomes[account] = value
        for account in accounts:
            if account not in outcomes or account in conflicting:
                outcomes[account] = {"account_id": account, "state": "unknown", "net_r": None}
        if not accounts:
            unknown += 1
        rows.append({"symbol": symbol, "side": side, "session_date": session,
                     "prereg_hash": digest, "mark_minutes": raw.get("mark_minutes"),
                     "accounts": accounts, "copies": len(accounts),
                     "last_closed_at": raw.get("last_closed_at"),
                     "outcomes": [outcomes[a] for a in sorted(outcomes)]})
    return _reading(rows, unknown, "session " + wanted if wanted else "latest published episodes; no implicit one-day filter")


def _rule_proven(row):
    opened, effective = _time(row.get("opened_at")), _time(row.get("rule_effective_at"))
    return (row.get("one_entry_rule_in_force") is True
            and _hash(row.get("prereg_hash")) is not None
            and row.get("rule_prereg_hash") == row.get("prereg_hash")
            and opened is not None and effective is not None and opened >= effective)


def statement_day(deals, one_entry_in_force=False, session_date=None):
    """Review explicitly owned position records, never infer opens from deal rows.

    `one_entry_in_force` is retained only for call compatibility and is not proof.
    Required identity: ownership_verified, account[_id], position_id, entry_key,
    symbol, side, session_date and prereg_hash. Measured outcomes additionally need
    position_complete=True and net_r. Repeated entry evidence additionally needs
    each later entry's opened_at/rule_effective_at/rule_prereg_hash proof.
    A manual close's channel does not change verified opening ownership.
    """
    wanted = _session(session_date) if session_date is not None else None
    if session_date is not None and wanted is None:
        return _reading([], 1, "invalid requested session")
    positions = {}
    invalid = set()
    unknown = 0
    for raw in deals if isinstance(deals, list) else []:
        if not isinstance(raw, dict) or raw.get("ownership_verified") is not True:
            unknown += 1
            continue
        account = raw.get("account_id", raw.get("account"))
        ident, entry = raw.get("position_id"), raw.get("entry_key")
        session = _session(raw.get("session_date"))
        if wanted is not None and session != wanted:
            continue
        digest, side, symbol = _hash(raw.get("prereg_hash")), _side(raw.get("side")), raw.get("symbol")
        if (not _positive(account) or not _positive(ident) or not isinstance(entry, str) or not entry
                or not isinstance(symbol, str) or not symbol or session is None or digest is None or side is None):
            unknown += 1
            continue
        key = (account, ident)
        net_r = _number(raw.get("net_r")) if raw.get("position_complete") is True else None
        row = {"account": account, "position_id": ident, "entry_key": entry, "symbol": symbol,
               "side": side, "session_date": session, "prereg_hash": digest,
               "net_r": net_r, "opened_at": raw.get("opened_at"), "rule_proven": _rule_proven(raw),
               "mark_minutes": raw.get("mark_minutes") if isinstance(raw.get("mark_minutes"), str) else None}
        if key in positions and positions[key] != row:
            invalid.add(key)
        positions[key] = row
    for key in invalid:
        positions.pop(key, None)
        unknown += 1
    # One entry_key mapped to multiple positions is not complete entry identity.
    entry_positions = {}
    for key, row in positions.items():
        entry_positions.setdefault((row["account"], row["entry_key"]), set()).add(key)
    for keys in entry_positions.values():
        if len(keys) > 1:
            for key in keys:
                positions.pop(key, None)
                unknown += 1
    episodes = {}
    entry_groups = {}
    for row in positions.values():
        # Across accounts, only an explicit saved decision mark can prove a
        # common signal. An entry key/position ID is account-local identity.
        mark = row["mark_minutes"] or "unclassified:" + str(row["account"]) + ":" + row["entry_key"]
        key = (row["symbol"], row["side"], row["session_date"], row["prereg_hash"], mark)
        ep = episodes.setdefault(key, {"symbol": row["symbol"], "side": row["side"],
            "session_date": row["session_date"], "prereg_hash": row["prereg_hash"],
            "mark_minutes": mark, "accounts": [], "copies": 0,
            "last_closed_at": None, "outcomes": []})
        ep["accounts"].append(row["account"])
        ep["copies"] += 1
        ep["outcomes"].append({"account_id": row["account"], "state": "measured" if row["net_r"] is not None else "unknown", "net_r": row["net_r"]})
        entry_groups.setdefault((row["account"], row["symbol"], row["session_date"]), []).append(row)
    for ep in episodes.values():
        ep["accounts"] = sorted(set(ep["accounts"]))
        ep["copies"] = len(ep["accounts"])
        # Several positions on one account are not several copied accounts.
        # Do not fabricate one outcome from conflicting per-position R bases.
        account_outcomes = {}
        for outcome in ep["outcomes"]:
            account = outcome["account_id"]
            account_outcomes[account] = ({"account_id": account, "state": "unknown", "net_r": None}
                                         if account in account_outcomes else outcome)
        ep["outcomes"] = [account_outcomes[account] for account in sorted(account_outcomes)]
    reading = _reading(list(episodes.values()), unknown, "verified position records" + (" for session " + wanted if wanted else "; sessions kept separate"))
    repeats = []
    for (account, symbol, session), entries in sorted(entry_groups.items()):
        if len(entries) < 2:
            continue
        entries.sort(key=lambda row: (_time(row["opened_at"]) or dt.datetime.max.replace(tzinfo=dt.timezone.utc), row["entry_key"]))
        proof = all(_time(row["opened_at"]) is not None for row in entries) and any(row["rule_proven"] for row in entries[1:])
        repeats.append({"account": account, "symbol": symbol, "session_date": session,
                        "opens": len(entries), "entry_keys": [row["entry_key"] for row in entries],
                        "post_rule_repeat_proven": proof})
    reading["repeats"] = repeats
    reading["pr"] = any(row["post_rule_repeat_proven"] for row in repeats)
    reading["signature"] = _signature([{"positions": [positions[key] for key in sorted(positions)], "unknown": unknown}])
    return reading
