"""When a Grok permit is worth spending.

Owner, 28-10-2026: time is money. A call that is stored and not acted on
is forbidden. Two reasons spend a permit, and nothing else:

- block_a_send: a mark has already passed the cost gate, and a named halt
  or holiday would stop the order that would otherwise go.
- unblock_a_close: an owned position has no stop, or an amend failed, and
  the call is what restores the protection.

Affirming a setup, filling a daily reading, and sentiment for its own sake
spend nothing. This does not create the 15-of-20. It stops the spend that
cannot.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone

REASONS = frozenset({"block_a_send", "unblock_a_close"})


def worth_a_permit(reason: str) -> bool:
    return reason in REASONS


class Budget:
    """A ceiling, not a quota. Ten permits a day. A bad reason spends none."""

    def __init__(self, limit: int = 10):
        self.limit = limit
        self.used: dict[str, int] = {}
        self.seen: set[tuple[str, str, str]] = set()

    def day(self, now: datetime) -> str:
        return now.astimezone(timezone.utc).date().isoformat()

    def allow(self, symbol: str, session: str, reason: str, now: datetime) -> bool:
        if not worth_a_permit(reason):
            return False
        if (symbol, session, reason) in self.seen:
            return False
        if self.used.get(self.day(now), 0) >= self.limit:
            return False
        return True

    def spend(self, symbol: str, session: str, reason: str, now: datetime) -> bool:
        if not self.allow(symbol, session, reason, now):
            return False
        self.seen.add((symbol, session, reason))
        day = self.day(now)
        self.used[day] = self.used.get(day, 0) + 1
        return True


def record(symbol: str, session: str, reason: str, headline: str, source: str, at: str) -> dict | None:
    """A reading that can change a send. Anything else is not recorded."""
    if not worth_a_permit(reason):
        return None
    return {
        "kind": "grok",
        "symbol": symbol,
        "session": session,
        "reason": reason,
        "headline": headline,
        "source": source,
        "at": at,
        "effect": reason,
    }


def to_json(reading: dict) -> str:
    return json.dumps(reading, sort_keys=True)
