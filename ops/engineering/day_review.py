"""One-day cross-account review. No close, no reverse, no sentiment call.

Owner, № 17,688, 29-10-2026 03:38 SGT: do not pause. Look across accounts
for one day. Decide whether the loss is a trend (one side, copied) or the
rule (both sides). A trend does not authorise a new exit. Both sides losing
means a reverse is the same break the other way. A model call that does not
block a send or restore a stop is not made.

A pull request is allowed only when the running one-entry rule was already
in force and the same account, symbol and session was opened again. That is
a software defect. A loss, a cluster, or a fast stop is not.

Grok 4.7. Other models: do not fit a close threshold to the day you just read.
Do not wire reverse_side to a live order from this report.
"""


def _side_net(deals):
    sides = {}
    for deal in deals:
        if deal.get("bot") is False:
            continue
        side = deal.get("side") or deal.get("Opening Direction")
        symbol = deal.get("symbol") or deal.get("Symbol")
        account = deal.get("account")
        net = deal.get("net")
        if net is None:
            net = deal.get("Net USD")
        day = (deal.get("day") or deal.get("Opening time (UTC+8)") or "")[:11]
        try:
            net = float(str(net).replace(",", "").replace("\u00a0", ""))
        except (TypeError, ValueError):
            continue
        row = sides.setdefault(symbol, {})
        item = row.setdefault(side, {"net": 0.0, "n": 0, "accounts": set(), "days": set()})
        item["net"] += net
        item["n"] += 1
        if account is not None:
            item["accounts"].add(account)
        if day:
            item["days"].add(day)
    return sides


def statement_day(deals, one_entry_in_force=False):
    """Classify one statement window. `one_entry_in_force` is true only after v6 is the running hash."""
    grouped = {}
    repeats = []
    for deal in deals:
        if deal.get("bot") is False:
            continue
        symbol = deal.get("symbol") or deal.get("Symbol")
        account = deal.get("account")
        day = (deal.get("day") or deal.get("Opening time (UTC+8)") or "")[:11]
        grouped.setdefault((account, symbol, day), 0)
        grouped[(account, symbol, day)] += 1
    for key, count in sorted(grouped.items()):
        if count >= 2 and key[0] is not None:
            repeats.append({"account": key[0], "symbol": key[1], "day": key[2], "opens": count})

    fundamental = []
    trend = []
    held = []
    for symbol, sides in sorted(_side_net(deals).items()):
        losing = [side for side, item in sides.items() if item["net"] < 0]
        winning = [side for side, item in sides.items() if item["net"] > 0]
        accounts = set()
        for item in sides.values():
            accounts |= item["accounts"]
        if len(losing) >= 2:
            fundamental.append(symbol)
        elif len(losing) == 1 and len(accounts) >= 2:
            trend.append(symbol)
        elif winning and not losing:
            held.append(symbol)

    defect = bool(repeats) and one_entry_in_force
    if fundamental:
        reason = (
            "Both sides lost. A reverse is the other side of the same break. "
            "Do not reverse. Do not edit the entry from this day."
        )
    elif trend:
        reason = (
            "One side was copied across accounts and lost. Closing before the stop "
            "is not registered. Do not invent that threshold from this day."
        )
    else:
        reason = "No two-sided loss and no copied losing side in this window."
    return {
        "kind": "fundamental" if fundamental else "trend" if trend else "none",
        "fundamental": fundamental,
        "trend": trend,
        "held": held,
        "repeats": repeats,
        "close_or_reverse": "not_registered",
        "sentiment": "not_called",
        "pr": defect,
        "reason": reason,
    }


def episode_review(health):
    """What /health already publishes. No P&L, so a copy is not called a loss."""
    rows = ((health or {}).get("episodes") or {}).get("rows") or []
    by_symbol = {}
    for row in rows:
        symbol = row.get("symbol") or "unknown"
        item = by_symbol.setdefault(symbol, {"sides": set(), "copies": 1})
        if row.get("side"):
            item["sides"].add(row.get("side"))
        try:
            item["copies"] = max(item["copies"], int(row.get("copies") or 1))
        except (TypeError, ValueError):
            pass
    fundamental = sorted(symbol for symbol, item in by_symbol.items() if len(item["sides"]) >= 2)
    trend = sorted(
        symbol for symbol, item in by_symbol.items()
        if symbol not in fundamental and item["copies"] >= 2
    )
    kind = "fundamental" if fundamental else "trend" if trend else "none"
    return {
        "kind": kind,
        "fundamental": fundamental,
        "trend": trend,
        "close_or_reverse": "not_registered",
        "sentiment": "not_called",
        "pr": False,
        "signature": kind + "|" + ",".join(fundamental) + "|" + ",".join(trend),
        "reason": (
            "Both sides are in the latest episodes. Do not reverse."
            if kind == "fundamental" else
            "A side was copied. The stop remains the exit until a registration says otherwise."
            if kind == "trend" else
            "No cluster in the latest episodes."
        ),
    }
