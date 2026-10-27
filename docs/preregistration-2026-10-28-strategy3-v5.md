# Pre-registration: Strategy 3 v5 — trailing stop (28-10-2026)

This file supersedes the exit rule of `docs/preregistration-2026-10-28-strategy3-v4.md` for new Strategy 3 decisions. The service embeds this file and hashes it (SHA-256). Every Strategy 3 decision, intent and trade recorded after this registration carries this hash and the label `jiaoyi-s3`. A further changed rule is a further pre-registration with a further hash.

**Trailing note for other models.** Owner, 28-10-2026: “change the forbid.” The 25 October sentence “No stop amends. Strategy 3 has no trailing rule, so it sends no stop amend. No amend can therefore drop its broker take-profit.” is retired for new decisions. The amend path already carries the position’s take-profit, so an amend does not delete it. The trail is a Chandelier hung from the best price by the original stop distance R. It moves only in the trade’s favour. It does not change the entry, the size, the open-risk budget, or the 1.38 target.

## What changes from the 28 Oct v4 registration

- **Trailing stop.** For an owned Strategy 3 position, the program tracks the best executable price (highest bid for a long, lowest ask for a short). The new stop is best − R for a long and best + R for a short, where R is the original stop distance from the entry. The stop is amended only when the new stop is better than the broker’s current stop. The amend carries the position’s existing take-profit. An amend is not sent more often than once per 30 seconds per position. A refused or rejected amend is tried again on a later quote.

## What does not change

- The 28 Oct Kelly skip, the 27 Oct mark decision, the cost gate, Q = 1.38, the original stop, TP1, TP2, the session rules, sizing, the open-risk budget, the single gate, and the owner pause.

## Approval

- Owner, 28-10-2026: “change the forbid.”

## Entry, as traded from this registration

Unchanged from v4. The mark decision, the band, the side, the cost gate (C ≥ R skips), and the Kelly skip are the same.

## Exit, as traded from this registration

- The original stop is R from the fill, sent as the order’s relative stop.
- TP1 is entry ± (1.38R + C). The close fraction is f = (R + C) / (TP1 distance + R). The runner’s TP2 is TP1 ± R. A size that cannot be split closes whole at TP1.
- **Trail.** After the position is open, the stop may move to best − R (long) or best + R (short), never further from the best than R, and never worse than the current stop. The amend carries the take-profit the broker holds. No amend is sent in the session’s last 30 minutes.
- Session-end close is unchanged. One Strategy 3 position per symbol. No new entry in the session’s last 30 minutes.

## What is not changed by this registration

- Sizing, the open-risk budget, the single order gate, ownership and intent-before-send are unchanged.
- Live still runs by the owner’s earlier order, not by a PASS.
- The trail does not write `risk_pct`, the live minimum, or the open-risk budget.
