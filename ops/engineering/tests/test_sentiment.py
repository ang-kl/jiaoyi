"""A permit is spent only to block a send or unblock a close."""

from datetime import datetime, timezone

from sentiment import Budget, record, worth_a_permit


NOW = datetime(2026, 10, 28, 0, 0, tzinfo=timezone.utc)


def test_an_affirmation_spends_nothing():
    b = Budget(limit=10)
    assert not worth_a_permit("affirm")
    assert not worth_a_permit("sentiment")
    assert not b.spend("XAUUSD", "2026-10-28", "affirm", NOW)
    assert b.used == {}
    assert record("XAUUSD", "2026-10-28", "affirm", "headline", "grok", "t") is None


def test_a_real_reason_spends_once_and_stops_at_ten():
    b = Budget(limit=10)
    assert b.spend("XAUUSD", "2026-10-28", "block_a_send", NOW)
    assert not b.spend("XAUUSD", "2026-10-28", "block_a_send", NOW)
    assert b.spend("XAUUSD", "2026-10-28", "unblock_a_close", NOW)
    for i in range(8):
        assert b.spend(f"S{i}", "2026-10-28", "block_a_send", NOW)
    assert not b.spend("NZDUSD", "2026-10-28", "block_a_send", NOW)
    reading = record("XAUUSD", "2026-10-28", "block_a_send", "halt", "grok", "t")
    assert reading["effect"] == "block_a_send"
    assert "side" not in reading
