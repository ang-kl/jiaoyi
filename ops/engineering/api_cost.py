"""List-price meter for the Grok API, and when to tell the owner.

Owner, 28-10-2026, № 17,635. Prices checked 27 Oct 2026 for Grok 4.7:
$2 per 1M input tokens, $0.50 per 1M cached input, $6 per 1M output.
This is not an xAI invoice. A standing 15-minute or 30-minute sentiment
poll is not a reason to call. The mark stays 30 minutes, and a permit is
still only block_a_send or unblock_a_close.

Telegram: one message a day at 11:00 SGT, and one message each time the
running estimate crosses a $10 step. $30, $50 and $100 are those steps,
named as such.
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from decimal import Decimal, ROUND_HALF_UP

SGT = timezone(timedelta(hours=8))
INPUT_PER_MILLION = Decimal("2")
CACHED_PER_MILLION = Decimal("0.50")
OUTPUT_PER_MILLION = Decimal("6")
STEP = Decimal("10")
NAMED = {Decimal("30"), Decimal("50"), Decimal("100")}


def standing_poll_allowed(minutes: int) -> bool:
    """Neither 15 nor 30. A clock is not a reason."""
    return False


def estimate(input_tokens: int, output_tokens: int, cached_tokens: int = 0) -> Decimal:
    if min(input_tokens, output_tokens, cached_tokens) < 0:
        raise ValueError("token counts are not negative")
    if cached_tokens > input_tokens:
        raise ValueError("cached tokens are part of the input")
    fresh = Decimal(input_tokens - cached_tokens)
    cost = (
        fresh * INPUT_PER_MILLION
        + Decimal(cached_tokens) * CACHED_PER_MILLION
        + Decimal(output_tokens) * OUTPUT_PER_MILLION
    ) / Decimal(1_000_000)
    return cost.quantize(Decimal("0.000001"), rounding=ROUND_HALF_UP)


def steps_crossed(before: Decimal, after: Decimal) -> list[Decimal]:
    if after < before:
        return []
    start = int(before // STEP)
    end = int(after // STEP)
    return [Decimal(n) * STEP for n in range(start + 1, end + 1)]


def daily_due(now: datetime, last_date: str | None) -> bool:
    local = now.astimezone(SGT)
    if local.hour < 11:
        return False
    return last_date != local.date().isoformat()


def notices(before: Decimal, after: Decimal, now: datetime, last_daily: str | None) -> list[str]:
    out = []
    local = now.astimezone(SGT)
    for step in steps_crossed(before, after):
        name = " named threshold." if step in NAMED else ""
        out.append(
            f"Grok API list-price estimate crossed ${step}. "
            f"Running estimate ${after}.{name} Not an xAI invoice. № 17,635."
        )
    if daily_due(now, last_daily):
        out.append(
            f"Grok API daily cost, {local.date().isoformat()} 11:00 SGT. "
            f"Running estimate ${after}. No standing 15- or 30-minute sentiment poll. "
            f"Not an xAI invoice. № 17,635."
        )
    return out


def assess_missed_deadline(symbol: str, state: str, ready: dict) -> dict:
    """A missed preparation deadline. The receipt is the assessment.
    It does not spend a permit and it does not call the API."""
    if state != "deadline_missed":
        return {"symbol": symbol, "assess": False, "api": False}
    missing = [name for name, ok in ready.items() if not ok]
    return {
        "symbol": symbol,
        "assess": True,
        "api": False,
        "missing": missing,
        "effect": "no send until those inputs exist",
    }
