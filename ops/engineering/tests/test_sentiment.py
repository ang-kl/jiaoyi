"""Sentiment is a measurement. It does not spend a permit twice, and it does not trade."""

from datetime import datetime, timezone

from sentiment import Budget, record


def test_one_call_per_symbol_per_session_and_a_ceiling_of_ten():
    b = Budget(limit=10)
    now = datetime(2026, 10, 28, 0, 0, tzinfo=timezone.utc)
    assert b.spend("XAUUSD", "2026-10-28", now)
    assert not b.spend("XAUUSD", "2026-10-28", now)
    assert b.spend("EURUSD", "2026-10-28", now)
    for i in range(8):
        assert b.spend(f"S{i}", "2026-10-28", now)
    assert not b.spend("NZDUSD", "2026-10-28", now)


def test_a_reading_is_a_measurement_and_not_an_order():
    r = record("XAUUSD", "2026-10-28", "neutral", "no headline", "grok", "2026-10-28T00:00:00Z")
    assert r["effect"] == "measurement"
    assert "side" not in r
    assert "volume" not in r
    assert "stop" not in r
