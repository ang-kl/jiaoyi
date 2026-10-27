"""Pre-trade sentiment as a measurement.

Owner, 28-10-2026: use the Grok API for real-time social sentiment, live
breaking news, and fast tracking of market-moving events to affirm setups.
This module records a reading. It does not enter, skip, or size. The reading
is a journal field, the same class as MAE and MFE. A filter becomes a rule
only after its own window under its own hash.

One call per symbol per session. The daily ceiling is 10. Unchanged evidence
spends nothing.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone


class Budget:
    """A ceiling, not a quota. Ten permits a day."""

    def __init__(self, limit: int = 10):
        self.limit = limit
        self.used: dict[str, int] = {}
        self.seen: set[tuple[str, str]] = set()

    def day(self, now: datetime) -> str:
        return now.astimezone(timezone.utc).date().isoformat()

    def allow(self, symbol: str, session: str, now: datetime) -> bool:
        key = (symbol, session)
        if key in self.seen:
            return False
        if self.used.get(self.day(now), 0) >= self.limit:
            return False
        return True

    def spend(self, symbol: str, session: str, now: datetime) -> bool:
        if not self.allow(symbol, session, now):
            return False
        self.seen.add((symbol, session))
        day = self.day(now)
        self.used[day] = self.used.get(day, 0) + 1
        return True


def record(symbol: str, session: str, sentiment: str, headline: str, source: str, at: str) -> dict:
    """A reading. It does not change a side, a stop, or a target."""
    return {
        "kind": "sentiment",
        "symbol": symbol,
        "session": session,
        "sentiment": sentiment,
        "headline": headline,
        "source": source,
        "at": at,
        "effect": "measurement",
    }


def to_json(reading: dict) -> str:
    return json.dumps(reading, sort_keys=True)
