# Pre-registration: Strategy 3, intraday momentum (frozen 25-10-2026)

This file is fixed before Strategy 3 trades. The service embeds it and hashes it (SHA-256). Every Strategy 3 decision, intent and trade records that hash and the label `jiaoyi-s3`. A changed rule is a new pre-registration with a new hash.

## Approval

- **Owner, 25-10-2026, before the approval:** "Trade more often, More markets, Use the bot-trade program live trading Sl, tp1, tp2 methodology".
- **Approved (Strategy 3 v2):** 25-10-2026, about 10:45 SGT. The owner also ordered live trading: "Live too, from now"; "Ignore read-only, do actual trading. That is mandate."
- **Entry rule replaced:** 25-10-2026, about 10:50 SGT. The owner chose "Tick-triggered only". This replaces the 30-minute check in the approved text below (see "Entry, as traded").

## Sources

- Zarattini, Aziz & Barbon (2024), *Beat the Market: An Effective Intraday Momentum Strategy for S&P500 ETF (SPY)*, Swiss Finance Institute Research Paper 24-97. It gives the noise band: the open × (1 ± σ), with σ the mean absolute move from the open at that time of day over the last 14 days.
  - The paper tested decisions at 30-minute marks. Its evidence therefore applies only loosely to the tick-triggered entry traded here.
- bot-trade, `agent/services/momentum-target-policy.js`, policy `risk_coverage_partial_v1` (`planMomentumTargets`): the SL, TP1 and TP2 arithmetic. Its own test numbers are the golden values of the port (`crates/jiaoyi-core/src/strategy3/targets.rs`).
- bot-trade, `agent/config/momentum-target-slippage.json`: per-side slippage by class, owner-confirmed 02-10-2026 (reply to № 10,585·B·5). Every class there is 24 bps.

## Approved rules (as approved, verbatim)

- Markets: all 20 symbols. Indices on their exchange cash sessions, DST-aware (US500/NAS100/US30 NYSE 09:30–16:00 America/New_York; GER40 Xetra 09:00–17:30 Europe/Berlin; UK100 LSE 08:00–16:30 Europe/London; JPN225 TSE 09:00–15:30 Asia/Tokyo excluding the 11:30–12:30 lunch; AUS200 ASX 10:00–16:00 Australia/Sydney); FX majors, metals and energy 07:00–20:00 UTC; BTCUSD/ETHUSD on the UTC day. Note Europe left summer time TODAY (25-10-2026) and the US leaves on 1 Nov: test both. Use a maintained tz crate (chrono-tz via crates.io is fine), not hand rules. A session with no ticks in its first 30 min is skipped (holiday).
- Entry: every 30 min after the session open, noise band = open × (1 ± σ), σ = mean over the last 14 sessions of |price at that time of day ÷ that session's open − 1|; open = the session's first M30 trendbar open (fetch M30 trendbars from the broker: ProtoOAGetTrendbarsReq is a read request); fewer than 14 sessions → skip with reason. At each check, buy if the current mid is above the upper band, sell if below the lower band. One Strategy 3 position per symbol; after it exits, a new entry may come at a later check. No new entry in the session's last 30 min (the session-end exit would close it at once — document this as my reading).
- Exits (bot-trade risk_coverage_partial_v1, port its arithmetic and reuse its own test numbers as golden values): R = the band half-width at entry (open × σ); SL = entry ∓ R as the broker stop; cost reserve C per unit = median spread (recorded ticks) + round-trip commission per unit (account's own deals, as edge/costs.rs) + 2 × the per-class slippage bps from bot-trade agent/config/momentum-target-slippage.json (owner-confirmed 02-10-2026) × price; TP1 = entry ± (3R + C); close fraction f = (R + C) / (TP1 distance + R) at TP1 (rounded up to the volume step, at least the minimum), managed by the trader as a partial ClosePositionReq; runner TP2 = TP1 ± R sent as the broker take-profit with the order; if the size cannot be split, the whole position closes at TP1 (broker TP = TP1); TP1 distance <= C → skip (bot-trade's target_price_invalid).
- Additions (approved): anything still open closes at the session end (market close); Strategy 3 open risk <= trading.s3_open_risk_pct (3); on HEDGED accounts Strategy 1 and Strategy 3 may each hold the same symbol (each manages only its own label); external positions still block; NETTED: any position blocks.
- Size: 0.6% of fresh equity at the stop, floored to the step, below the minimum skipped (demo); live: the owner's "one minimum trade at a time" (<= 10% of equity, only while no other position of this program is open on that account), from PR #14's live rule.
- Every amend carries the position's existing take-profit (cTrader amends replace protection; PR #14 is adding this to the gate).
- Records per trade: the quote at decision, band, σ, R, C, send time, fill time and price (slippage vs the decision quote), spread at entry and exit, commission from the deal. A route GET /trading/costs summarises ACTUAL cost and order-to-fill latency per symbol from these fills (this replaces the step-0 passive cost table).
- Status page: a Strategy 3 section per account (open positions, today's decisions and trades); no money on public routes.

## Entry, as traded (owner, 25-10-2026 ~10:50 SGT: "Tick-triggered only")

This supersedes the "Entry" rule above. The open, σ, the 14 sessions and the skip rule are as above.

- **Trigger.** Strategy 3 enters on the first quote beyond the noise band:
  - a buy on the first ask above the upper band;
  - a sell on the first bid below the lower band.
- **Continuous band.** The band is evaluated at the quote's own instant, by time of day. σ for that instant is interpolated linearly between the 30-minute marks of the last 14 sessions.
- **Unchanged:** one Strategy 3 position per symbol; re-entry after an exit; no new entry in the session's last 30 minutes. The exits, sizing, budget and records are as above.

## PASS rules (copied from `docs/preregistration-2026-10-24.md`), judged on the forward demo results

1. Mean net R per trade > 0, AND the 95% lower bound of a week-block bootstrap (10,000 resamples) > 0.
2. Mean net R above the 95th percentile of 1,000 random-direction controls at the same decision times.
3. Profit factor ≥ 1.3 (owner's figure).
4. At least 200 trades.
   - Fewer gives the verdict **"insufficient evidence"**: not a pass, and it goes to the owner.

The unit is R, the price move ÷ the initial stop distance, net of all costs. The demo accounts' own fills, commissions and exits are the data. Live trading runs by the owner's order ("Live too, from now"), not by a PASS.

## Readings (self-derived where the text leaves a choice; each is recorded with the decisions)

**The band**
1. **The session open.** It is the open of the M30 bar that starts at the session open. If that bar is missing or has no ticks, the session is skipped as a holiday, with the reason.
2. **The price at a mark.** It is the close of the M30 bar ending at that mark. σ at a mark takes the last 14 sessions that have a price there, looking back over at most 30 sessions. Fewer than 14 means no band at that mark, with the reason. Between marks, both marks must have a σ.
3. **No band, no entry.** There is no band before 30 minutes after the open. Entries start there and stop at 30 minutes before the close. JPN225 takes no entry during the lunch break.
4. **A quote beyond both sides.** A quote whose ask is above the upper band and whose bid is below the lower band (a spread wider than the band) triggers nothing. A quote exactly on the band is inside it.

**The order and its exits**

5. **R and the stop.** R is open × σ at the trigger's instant, rounded to the nearest tick (at least one). The stop is sent as the order's relative stop, R from the fill.
6. **The plan's entry.** The plan uses the triggering quote: the ask for a buy, the bid for a sell. TP1 and TP2 are distances from the fill, because the broker anchors a relative take-profit to the fill.
7. **TP1.** The partial close at TP1 is sent when the bid reaches TP1 for a long, or the ask for a short. It is sent once per position.
8. **TP2.** The relative take-profit sent with the order is the runner's TP2, or TP1 when the size cannot be split.

**The cost reserve C**

9. **The spread** is the median ask − bid of the account's last 50,000 recorded two-sided quotes on the symbol, within 7 days. Without one, the entry is skipped.
10. **The commission per round trip** is twice the class median of |commission| ÷ filled units over the account's own deals (`edge/costs.rs`'s book). It is converted to price units with the live quote → deposit rate.
11. **The slippage** is 24 bps a side for every class: bot-trade's index_cfd, fx, commodity (metal and energy) and crypto figures. It is charged at the triggering quote's price.

**Sizing and the session end**

12. **Sizing.** A demo account sizes 0.6% (`trading.risk_pct`) of fresh equity at the stop, floored to the volume step. Below the broker minimum, the entry is skipped. A live account takes exactly the broker minimum, when that risks at most 10% of equity (`trading.min_volume_max_risk_pct`) and no position of this program is open on the account.
13. **The session end.** At the session close, every Strategy 3 position on the symbol is closed with a ClosePositionReq. A Strategy 3 position found at start-up outside its symbol's session is closed at once. The positions are held through JPN225's lunch.
14. **Entry keys.** One key per symbol, session and entry number, so a session can re-enter after an exit, and an entry is never sent twice.
15. **The bars read.** The band reads 45 calendar days of M30 bars from the broker, in weekly requests, once per symbol per session, at the session's first quote after 30 minutes. A broker error is retried after 5 minutes.
16. **A trigger that does not go.** A trigger that is skipped or refused (sizing, the budget, a missing input or the gate) is taken again on that symbol only after 60 seconds. The gate reads the broker twice per try.
17. **No stop amends.** Strategy 3 has no trailing rule, so it sends no stop amend. No amend can therefore drop its broker take-profit.
