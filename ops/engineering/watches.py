"""Five watches on public trading evidence. No broker access, no rule change.

Reads the snapshots the coordinator already fetches (`/health`,
`/operations/reports`). Emits a Telegram event only when one of the four
triggers changes. Unchanged evidence produces no event and no session.

The live stop is not read as something to amend. Recovery is unavailable
from these public routes; the watch records that and does not invent a rate.
"""

V3_FILE = "docs/preregistration-2026-10-27-strategy3-v3.md"
GOAL_WINS = 15
GOAL_PAYOFF = 1.38
ACCOUNT_47790949 = 47790949


def _rows(value):
    return value if isinstance(value, list) else []


def _window(report):
    assessment = ((report or {}).get("report") or {}).get("assessment") or {}
    return assessment.get("window") or {}


def hash_watch(health):
    """Running registration file and hash, per Strategy 3 account."""
    rows = []
    mismatch = False
    for row in _rows(health.get("strategy3")):
        file = row.get("prereg_file")
        digest = row.get("prereg_hash")
        ok = file == V3_FILE and isinstance(digest, str) and len(digest) == 64
        if not ok:
            mismatch = True
        rows.append({
            "account_id": row.get("account_id"),
            "prereg_file": file,
            "prereg_hash": digest,
            "matches_v3": ok,
        })
    return {"rows": rows, "mismatch": mismatch or not rows}


def latest20_watch(operations):
    """Gap to 15/20 wins and to a 1.38 realized payoff. Does not retune."""
    gaps = []
    for row in _rows(operations.get("reports")):
        window = _window(row)
        wins = window.get("wins")
        count = window.get("count")
        payoff = window.get("realized_payoff")
        gaps.append({
            "account_id": row.get("account_id"),
            "strategy": row.get("strategy"),
            "wins": wins,
            "count": count,
            "wins_short": None if not isinstance(wins, int) else max(0, GOAL_WINS - wins),
            "payoff": payoff,
            "payoff_short": None if not isinstance(payoff, (int, float)) else round(GOAL_PAYOFF - float(payoff), 6),
            "sufficient": isinstance(count, int) and count >= 20,
        })
    return {"gaps": gaps}


def cost_gate_watch(health):
    """Skip reasons that name the cost reserve, by symbol, from today's decisions."""
    tally = {}
    for row in _rows(health.get("strategy3")):
        today = row.get("today") or {}
        for decision in _rows(today.get("recent_decisions")):
            reason = decision.get("reason") or ""
            if "cost reserve C" not in reason:
                continue
            symbol = decision.get("symbol") or "unknown"
            tally[symbol] = tally.get(symbol, 0) + 1
    return {"skips_by_symbol": tally}


def recovery_watch():
    """Same-session recovery is not on the public routes. Do not invent it."""
    return {
        "available": False,
        "reason": "public /health and /operations/reports do not carry the post-stop path",
        "threshold_crossed": False,
    }


def account_47790949_watch(operations):
    """Whether the unmatched extra P&L entry is still named."""
    for row in _rows(operations.get("account_source_diagnostics")):
        if row.get("account_id") != ACCOUNT_47790949:
            continue
        text = json_text(row)
        open_item = "unmatched" in text or row.get("acceptance_passed") is False
        return {
            "present": True,
            "still_open": open_item,
            "acceptance_passed": row.get("acceptance_passed"),
        }
    return {"present": False, "still_open": None, "acceptance_passed": None}


def json_text(value):
    import json
    return json.dumps(value, sort_keys=True, default=str)


def watches(health, operations):
    return {
        "hash": hash_watch(health),
        "latest20": latest20_watch(operations),
        "cost_gate": cost_gate_watch(health),
        "recovery": recovery_watch(),
        "account_47790949": account_47790949_watch(operations),
    }


def _open_fills(health):
    found = []
    for row in _rows(health.get("strategy3")):
        today = row.get("today") or {}
        for trade in _rows(today.get("trades_today")):
            if trade.get("kind") == "open" and trade.get("outcome") in ("filled", "partly_filled"):
                found.append({
                    "account_id": row.get("account_id"),
                    "symbol": trade.get("symbol"),
                    "side": trade.get("side"),
                    "at": trade.get("at"),
                })
    return found


def _limits(health):
    reached = []
    for section in ("trading", "strategy3"):
        for row in _rows(health.get(section)):
            performance = row.get("performance") or {}
            hit = performance.get("limit_reached")
            if hit:
                reached.append({
                    "account_id": row.get("account_id"),
                    "strategy": row.get("strategy") or section,
                    "limit_reached": hit,
                })
    return reached


def events(previous, health, operations):
    """The four Telegram events. `previous` is the last watches() plus open fills and limits.

    Returns (events, current_state). Unchanged state returns no events.
    """
    current = watches(health, operations)
    current["open_fills"] = _open_fills(health)
    current["limits"] = _limits(health)
    previous = previous or {}
    out = []

    if current["hash"]["mismatch"] and not previous.get("hash", {}).get("mismatch"):
        out.append({"event": "hash_mismatch", "detail": current["hash"]})
    elif current["hash"]["mismatch"] and previous.get("hash", {}).get("mismatch"):
        pass
    elif not current["hash"]["mismatch"] and previous.get("hash", {}).get("mismatch"):
        out.append({"event": "hash_matched", "detail": current["hash"]})

    prev_limits = {json_text(row) for row in previous.get("limits") or []}
    curr_limits = {json_text(row) for row in current["limits"]}
    if curr_limits != prev_limits and (curr_limits or prev_limits):
        out.append({
            "event": "limit_changed",
            "detail": {"now": current["limits"], "cleared": not curr_limits and bool(prev_limits)},
        })

    prev_fills = {json_text(row) for row in previous.get("open_fills") or []}
    for fill in current["open_fills"]:
        if json_text(fill) not in prev_fills:
            out.append({"event": "entry_sent", "detail": fill})

    if current["recovery"]["threshold_crossed"] and not previous.get("recovery", {}).get("threshold_crossed"):
        out.append({"event": "recovery_threshold", "detail": current["recovery"]})

    return out, current
