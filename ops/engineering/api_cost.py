"""List-price meter for the Grok API, and when to tell the owner.

Owner, 28-10-2026, № 17,637. The halt call uses grok-4-1-fast-non-reasoning
with reasoning effort none. After 15 May 2026 that slug is served as
grok-4.3 at effort none. The bill uses the model the response returns,
not the slug that was sent. List prices, not an xAI invoice:
fast $0.20 / $0.05 cached / $0.50 per 1M (xAI card);
grok-4.3 $1.25 / $0.20 / $2.50 (27 Oct 2026 list);
grok-4.7 $2 / $0.50 / $6. An unknown model is not priced.

A standing 15- or 30-minute sentiment poll is not a reason to call.
Telegram: 11:00 SGT daily, and every $10, with $30, $50 and $100 named.
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from decimal import Decimal, ROUND_HALF_UP

SGT = timezone(timedelta(hours=8))
STEP = Decimal("10")
NAMED = {Decimal("30"), Decimal("50"), Decimal("100")}
HALT_MODEL = "grok-4-1-fast-non-reasoning"

# input, cached input, output, per million tokens.
RATES = {
    "grok-4-1-fast-non-reasoning": (Decimal("0.20"), Decimal("0.05"), Decimal("0.50")),
    "grok-4-1-fast": (Decimal("0.20"), Decimal("0.05"), Decimal("0.50")),
    "grok-4-1-fast-reasoning": (Decimal("0.20"), Decimal("0.05"), Decimal("0.50")),
    "grok-4.3": (Decimal("1.25"), Decimal("0.20"), Decimal("2.50")),
    "grok-4.7": (Decimal("2"), Decimal("0.50"), Decimal("6")),
}


def halt_model(env: dict | None = None) -> str:
    """Empty JIAOYI_GROK_MODEL means the fast non-reasoning slug.
    JIAOYU_GROK_MODEL is a typo and is not read."""
    chosen = (env or {}).get("JIAOYI_GROK_MODEL") or HALT_MODEL
    return chosen.strip() or HALT_MODEL


def halt_request(symbol: str, session: str, env: dict | None = None) -> dict:
    """One closed question. No sentiment and no reasoning trace."""
    return {
        "model": halt_model(env),
        "reasoning_effort": "none",
        "max_output_tokens": 40,
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "halt",
                "strict": True,
                "schema": {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["halt", "name"],
                    "properties": {
                        "halt": {"type": "boolean"},
                        "name": {"type": ["string", "null"]},
                    },
                },
            },
        },
        "input": (
            f"Symbol {symbol}, session {session}. "
            "Reply with the schema only. halt is true only for a named halt or holiday. "
            "No sentiment and no outlook."
        ),
    }


def standing_poll_allowed(minutes: int) -> bool:
    """Neither 15 nor 30. A clock is not a reason."""
    return False


def estimate(model: str, input_tokens: int, output_tokens: int, cached_tokens: int = 0) -> Decimal:
    if model not in RATES:
        raise ValueError("no list price for the model the response returned: " + model)
    if min(input_tokens, output_tokens, cached_tokens) < 0:
        raise ValueError("token counts are not negative")
    if cached_tokens > input_tokens:
        raise ValueError("cached tokens are part of the input")
    fresh_rate, cached_rate, output_rate = RATES[model]
    fresh = Decimal(input_tokens - cached_tokens)
    cost = (
        fresh * fresh_rate
        + Decimal(cached_tokens) * cached_rate
        + Decimal(output_tokens) * output_rate
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
