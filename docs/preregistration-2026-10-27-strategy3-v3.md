# Pre-registration: Strategy 3 v3 — mark decision and cost gate (27-10-2026)

This file supersedes the entry rule of `docs/preregistration-2026-10-25-strategy3.md` for new Strategy 3 decisions. The service embeds this file and hashes it (SHA-256). Every Strategy 3 decision, intent and trade recorded after this registration carries this hash and the label `jiaoyi-s3`. A further changed rule is a further pre-registration with a further hash.

**Refinement note for other models.** This registration is the response to the 27 Oct 2026 statement analysis. The registered 3R exit plus a cost reserve that was often larger than R placed the barrier at a median ~5R. A no-drift process hits that about 16–25% of the time. The statements printed 27–34% on the demos and 0% on ETH (53 trades), metals and the oils. That is the barrier formula, not a ledger bug. The refinement does three things and leaves one owner decision open:

1. Entry is a 30-minute mark decision on the mid, matching the paper, not the first tick beyond the band.
2. A symbol is skipped when the cost reserve C is at least R. That removes the names where the barrier is furthest.
3. The scoreboard note records that three demos copying one signal are one episode.
4. `REQUIRED_RR` is 1.38. Owner, 27-10-2026: “target near 1.4R.” That is the goal’s payoff reference. The previous hard minimum of 3.0 is retired for new decisions.

## Approval

- Owner, 27-10-2026: “build and merge. And comments in code to let other AI model knows that you have refine the solution.”
- Owner, 27-10-2026: “target near 1.4R.” Q = 1.38, the goal’s payoff reference.

## What changes from the 25 Oct registration

- **Entry.** The 25 Oct tick rule is replaced. Strategy 3 now decides only at a 30-minute mark of the session (the paper’s check). At the mark, the mid buys if it is above the upper band and sells if it is below the lower band. The first ask above the band is no longer an entry.
- **Cost gate.** After C is computed, if C ≥ R the entry is skipped with the reason. The 25 Oct rule already skipped when the TP1 distance was not above C. This gate skips earlier, when the reserve itself is at least the stop, which is the case the statements measured (median C ≈ 2.2R).
- **Episode note.** The latest-20 window remains per strategy and account. A comment and a note record that accounts copying one signal are one episode, so the window is not independent evidence.

## What does not change

- R is the band half-width at the decision, open × σ.
- The stop is R from the fill, sent as the order’s relative stop.
- TP1 is entry ± (1.38R + C). The close fraction is f = (R + C) / (TP1 distance + R). The runner’s TP2 is TP1 ± R. A size that cannot be split closes whole at TP1.
- `REQUIRED_RR = 1.38` is hard-coded and refuses a lower value.
- One Strategy 3 position per symbol. No new entry in the session’s last 30 minutes. Session-end close. No S3 trailing. No stop amend.
- Sizing, the open-risk budget, the single order gate, ownership and intent-before-send are unchanged.
- Live still runs by the owner’s earlier order, not by a PASS.

## Entry, as traded from this registration

- **When.** Only at a 30-minute mark after the session open, and before the session’s last 30 minutes. A quote that is not at a mark does not enter. The mark window is 5 seconds after the mark, so the quote that arrives with the mark is the decision.
- **Band at the mark.** σ is the mark’s own value from the last 14 sessions. There is no interpolation for the decision. Fewer than 14 sessions at that mark means no band and no entry.
- **Side.** The mid is (bid + ask) / 2, requiring a two-sided quote. Buy if mid > upper. Sell if mid < lower. A mid inside the band, or a quote wider than the band, enters nothing.
- **Cost gate.** C is the existing reserve: median spread + round-trip commission per unit + 48 bps slippage (24 bps a side). If C ≥ R, skip. The reason is recorded. This is not a change of Q.

## PASS rules

Unchanged from the 25 Oct registration, judged on forward results after this hash:

1. Mean net R per trade > 0, and the 95% lower bound of a week-block bootstrap (10,000 resamples) > 0.
2. Mean net R above the 95th percentile of 1,000 random-direction controls at the same decision times.
3. Profit factor ≥ 1.3.
4. At least 200 trades. Fewer is “insufficient evidence”, not a pass.

The unit is R, net of all costs. Live is not authorized by a PASS.

## Readings

1. **The mark.** A decision mark is a multiple of 30 minutes after the session open, within 5 seconds after that instant. Before the first mark there is no decision.
2. **The mid.** Both sides are required. A missing side is not a decision.
3. **C ≥ R.** Compared in price units, the same units as R (the half-width, rounded to the nearest tick, at least one). Equality skips.
4. **Episode.** The latest-20 card is still per account. Correlated fills across demos are one sample for any claim about the rule. The card does not multiply them.
5. **Geometry.** Q = 1.38 by the owner’s “target near 1.4R.” A model that moves it again has not been authorized.
