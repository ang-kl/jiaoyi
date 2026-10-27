"""The meter alerts at 11:00 SGT and on every $10, and a poll is not a call."""

from datetime import datetime, timezone, timedelta
from decimal import Decimal

from api_cost import assess_missed_deadline, daily_due, estimate, notices, standing_poll_allowed, steps_crossed

SGT = timezone(timedelta(hours=8))


def test_a_clock_is_not_a_reason_and_the_rate_is_the_published_one():
    assert standing_poll_allowed(15) is False
    assert standing_poll_allowed(30) is False
    # 1,000 fresh input and 200 output: 0.002 + 0.0012.
    assert estimate(1000, 200) == Decimal("0.003200")
    assert estimate(1000, 0, cached_tokens=1000) == Decimal("0.000500")


def test_ten_dollar_steps_name_30_50_and_100_and_eleven_is_once_a_day():
    assert steps_crossed(Decimal("9"), Decimal("10")) == [Decimal("10")]
    assert steps_crossed(Decimal("29"), Decimal("50")) == [Decimal("30"), Decimal("40"), Decimal("50")]
    before_eleven = datetime(2026, 10, 28, 10, 59, tzinfo=SGT)
    at_eleven = datetime(2026, 10, 28, 11, 0, tzinfo=SGT)
    assert daily_due(before_eleven, None) is False
    assert daily_due(at_eleven, None) is True
    assert daily_due(at_eleven, "2026-10-28") is False
    text = notices(Decimal("29"), Decimal("30"), at_eleven, None)
    assert any("crossed $30" in line and "named threshold" in line for line in text)
    assert any("11:00 SGT" in line for line in text)
    assert notices(Decimal("0"), Decimal("0"), before_eleven, None) == []


def test_a_missed_deadline_is_assessed_without_a_call():
    got = assess_missed_deadline(
        "AUS200", "deadline_missed",
        {"commission_ready": None, "history_ready": None, "metadata_ready": None, "spread_ready": None},
    )
    assert got["api"] is False
    assert got["assess"] is True
    assert "commission_ready" in got["missing"]
    assert assess_missed_deadline("JPN225", "prepared_on_time", {"spread_ready": True})["assess"] is False
