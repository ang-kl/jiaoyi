# Phase 0 rulebook: jiaoyi daily trend system

Status: **DRAFT for the owner's approval.** Nothing in this file can be traded until the owner approves it and phase 3 (the history test) passes.

Every number here is fixed in advance. Changing a number after a test has run counts as a new trial. It is recorded in the trial log (`docs/storage.md`, table `research_trials`) and judged as such.

Ordered by the owner, 24-10-2026 (bot-trade conversation, № 16,790–16,795):
- rebuild the whole program from the ground up;
- new repository `ang-kl/jiaoyi`;
- Rust;
- PostgreSQL;
- services in the Railway project `jiaoyi_prj` (service `jiaoyi`), which the owner created on 24-10-2026 (it replaces the earlier plan to use the existing project);
- start phase 0.

The research behind this design is in the bot-trade conversation's report *Why the trading bot loses*, 24-10-2026. It is cited below as **[R]**.

## 1. Goal and gates

| Item | Value | Source |
|---|---|---|
| Return goal | 12–20% a year, at a 25% volatility target | [R]: trend funds earn about Sharpe 0.45–0.8; return ≈ Sharpe × volatility. **Target not confirmed by the owner** (offered 15/25/35%). |
| Phase 3 acceptance | Over the out-of-sample period, all of these, net of spread, commission and swap:<br>• profit factor ≥ 1.3;<br>• ≥ 200 closed trades;<br>• worst drawdown < 20% at the risk in §5. | Owner: profit factor 1.3; [R]: sample-size findings |
| Robustness | At least 6 of the 9 grid cells in §8 have profit factor > 1.0 | Self-derived guard against one lucky setting |
| If phase 3 fails | Stop. Report. Nothing goes live. No gate is lowered. | Owner rule, bot-trade CLAUDE.md |
| Units | Every result in **R**: price move ÷ initial stop distance, net of costs. Money is shown beside R, never instead of it. | [R]: money-based verdicts hid the edge |

## 2. Accounts

| Account | Role | When |
|---|---|---|
| Demo 49236051 (login 5361573) | First and only account trading the new system | Phase 5, after phase 4 gates |
| Live 43002148 (login 1251442) | Real money, small risk | Phase 6, owner's word only |
| All other accounts | **Refused** | Never |

**Only these two accounts are permitted** (owner, 24-10-2026: "only two account (live and demo) permitted"):
- The list is the variable `JIAOYI_ALLOWED_ACCOUNTS=49236051,43002148`.
- The program checks it at start-up and again in the order sender, before every order.
- An order for any other account is refused and recorded, never sent.
- A missing or empty list means no account is permitted and nothing trades.

The old bot-trade program and the new system never trade the same account at the same time.

### Demo accounts for data (owner, 25-10-2026)

Added by the owner on 25-10-2026 (bot-trade conversation, № 16,972). The two accounts above and the text above are unchanged.

| Account | Owner's figures | Role | When |
|---|---|---|---|
| Demo 46130058 (number 5203012) | leverage 1:200, USD | Data; orders as the owner sets them | Now |
| Demo 47790949 (number 5306502) | leverage 1:25, USD | Data; orders as the owner sets them | Now |

- **Purpose.** The goal is to trade across demo leverages.
  - The owner first chose "Data now, orders after the edge".
  - At about 08:30 SGT on 25-10-2026 the owner set the three demo accounts to trade from 13:00 SGT that day. The order path is a separate change.
  - The change that reads these variables places no order.
- **How an account is added.** Each one is named by a variable that holds one account id: `JIAOYI_DEMO_ACCOUNT_ID`, or `JIAOYI_DEMO_ACCOUNT_ID_<N>` with N a positive integer.
  - The owner set `JIAOYI_DEMO_ACCOUNT_ID_2=46130058` and `JIAOYI_DEMO_ACCOUNT_ID_3=47790949` ("Code reads the new variables").
  - Such an account is permitted and declared demo.
  - `JIAOYI_ALLOWED_ACCOUNTS` is still required. A missing or empty list still refuses start-up.
- **Refusals.**
  - A malformed variable is refused and recorded under its name. It does not stop start-up.
  - A variable that names an account `JIAOYI_ACCOUNT_ENVIRONMENTS` declares live is refused as contradictory. That account keeps the pair's declaration.
- **No tick history.** An account named only by such a variable does not download tick history (owner: "Skip their tick history"; the Postgres volume held 26.8 GB of 50 GB with two accounts). Daily bars, live ticks, deals, the availability probe and the token-grant check run as for 49236051.
- **Nothing else changes.** "All other rules and capabilities should not change with added accounts."
- **Leverage check.** The owner's stated leverage is checked against the broker's trader record: table `trader_reads`, shown on `/health`.

## 3. Universe (candidates; phase 1 confirms each on each account)

All 20 names below appear in the bot's own broker deal statements (bot-trade `agent/config/tick-shadow-sim.json`, measured 21-08-2026).

| Class | Symbols |
|---|---|
| Equity indices (7) | US500, NAS100, US30, GER40, UK100, JPN225, AUS200 |
| FX (6) | EURUSD, GBPUSD, USDJPY, AUDUSD, USDCAD, NZDUSD |
| Metals (2) | XAUUSD, XAGUSD |
| Energy (3) | NatGas (broker name, owner 24-10), SpotCrude (broker name, owner 24-10), SpotBrent (broker name, owner 24-10) |
| Crypto (2) | BTCUSD, ETHUSD |

Phase 1 drops a symbol that fails any of these checks, and records why:
- not tradable on the account;
- less than 10 years of daily history at the broker (crypto: 6 years);
- median spread greater than 10% of 1 daily ATR(20).

## 4. Signal (once a day, after each symbol's daily bar closes)

Every input is the account's **own** closed daily bars, under its **own** symbol id. The bar still forming is never read.

| Rule | Value |
|---|---|
| Trend filter | EMA(50) of close above EMA(100) → longs only. Below → shorts only. |
| Long entry | Today's close is the highest close of the last 50 days, and the filter is long. |
| Short entry | Today's close is the lowest close of the last 50 days, and the filter is short. |
| Order | Market order after the daily close, once the market is open. Skipped if the price has moved more than 0.5 × ATR(20) beyond the signal close. |
| One position per symbol | No adding to a position. No reversing on the same day. |

Source: Clenow, *Following the Trend*, and the Turtle rules (breakout with a trend filter). These rules are a subset of the time-series momentum evidence in [R].

## 5. Stops, exits and sizing

| Rule | Value |
|---|---|
| ATR | ATR(20) on daily bars, Wilder smoothing |
| Initial stop | 3 × ATR(20) from the fill. Placed at the **broker** with the order. |
| Trailing exit | Once a day after the close, the stop moves to 3 × ATR(20) behind the highest close since entry (lowest, for shorts). It only ever moves in the trade's favour. Broker-side. |
| Take profit | **None.** Winners run until the trailing stop. ([R]: fixed 2–3R targets and early closes cut winners to +0.42R.) |
| Risk per trade | **0.6% of account equity** per 1R, at the 25% volatility target. Would be 0.4% at 15% and 0.85% at 35%. This mapping is self-derived; phase 3 measures the volatility it actually produces. |
| Volume | `volume = floor_to_step(risk_money ÷ (stop_distance × value_per_point_in_account_currency))` |
| Volume too small | Below the broker's minimum volume, the trade is **skipped**, never rounded up (owner rule). |
| Hard size cap at send | The order sender recomputes the risk from the volume it is about to send. It **refuses** the order if that risk exceeds 1.1 × the allowed risk money. No code path sends an order without this check. ([R]: a 72.5-lot order once risked about 20× its limit.) |
| Cost guard | Round-trip spread + commission > 10% of 1R → skip the trade. |

## 6. Portfolio limits

| Limit | Value |
|---|---|
| Open positions | At most 12 |
| Total open risk (sum of current R at stop) | At most 6% of equity |
| Open risk per asset class | At most 2.4% of equity |
| Drawdown brake | Equity 25% below its peak → no new entries. Open positions keep their stops. The owner decides what happens next. |
| Manual trading | None on the system's accounts. A position the system did not open is reported, never managed ([R]: human and adopted trades cost 24% of the loss). |

## 6a. No trading switch

There is no on/off trading switch. Owner, 24-10-2026: *"you don't need to be bothered about a trading or not-trading switch. It should always want to trade."* Once its phase gates are passed, the system trades whenever its rules say so. The only things that stop an entry are the rules themselves: the signal (§4), the cost guard and the hard size cap (§5), the portfolio limits and the drawdown brake (§6), and the account allowlist (§2). Each refusal is recorded with its reason.

## 7. What the system never does

- Intraday or tick entries.
- Fibonacci, value-area, fair-value-gap, VWAP or RSI rules.
- LLM or "premise" closes.
- A position without a broker-side stop.
- Changing a setting without recording a new trial.
- Rounding a volume up.
- Inventing a missing number. A missing price, cost or balance means **no trade**, and the record says why.

## 8. Phase 3 test design (fixed before any test runs)

- **Data:** the broker's own daily bars and costs (phase 1).
- **Split:** the first 60% of each symbol's history is for development and the last 40% is out of sample. The out-of-sample block is run **once**.
- **Grid (9 trials, all reported):** breakout length {20, 50, 100} × stop multiple {2, 3, 4}. The default (50, 3) must pass §1 on its own; the grid is only the robustness check.
- **Costs in the test:**
  - spread from the broker's quotes;
  - commission per the broker's schedule, re-measured in phase 1;
  - swap charged every night held, at the broker's rate;
  - slippage of 0.1 × ATR(20) per side until phase 5 measures the real figure.
- **Correction for multiple tries:** every trial is logged. The Sharpe ratio is reported deflated for the number of trials (Bailey & López de Prado).

## 9. Phases (from the approved plan)

| # | Phase | Gate |
|---|---|---|
| 0 | This rulebook and the storage design | Owner approves |
| 1 | Data: own daily bars and costs per account | Cross-checked against the broker for 3 symbols per class |
| 2 | Simulator: one engine for testing and trading | Reproduces 3 hand-calculated trades exactly |
| 3 | History test per §8 | §1 acceptance, or stop |
| 4 | Execution core: cTrader client, hard size cap, broker stops, reconciliation | Demo-only tests, including a deliberately oversized order refused |
| 5 | Demo 49236051 | First 50 trades match the simulator within an agreed band |
| 6 | Live 43002148 | Owner's word |
| 7 | Old program retired | Owner's word |

## 10. Open owner decisions

1. **Volatility target:** 15%, 25% (this draft) or 35%.
2. **Universe:** approve the 20 candidates in §3, or name changes.
3. **This rulebook:** approve as written, or name changes.

## 11. The order path and Strategy 1 (owner, 25-10-2026)

Owner, 25-10-2026: *"I want live to execute trading by 1 PM today"*, then *"Keep the gate; demo trades at 1 PM"* and, for live, *"Allow the minimum volume"*. This section records what the order path does and where it departs from the sections above. Nothing above is edited.

- **Strategy 1: tick-triggered trend** (the owner's naming, № 16,806; H1 is its pre-registration label, `docs/preregistration-2026-10-24.md`). It runs the pre-registered default: breakout 50, stop 3 × ATR(20), the entry triggered on the tick, and a trail behind the best bid (ask for shorts) since the fill. Where §4, §5 and §7 differ (once-a-day signals, a trail on closes, "no tick entries"), Strategy 1 is what runs, because it is the hypothesis under test, approved by the owner on 24-10.
- **No on/off switch** (§6a; owner, 24-10: *"why do i need JIAOYI_TRADING_ENABLED, removed it. it should not be doing so"*). Every permitted demo account trades Strategy 1 as soon as the service runs. A live account runs the same loop, and waits for the H1 held-out trial under the running pre-registration hash to say PASS while `trading.live_requires_edge_pass` is 1.
- **The live edge gate is lifted** (owner, 25-10-2026, in writing: *"Live too, from now"*, overriding the 24-10 rule that live waits for a demonstrated edge). It ships as a reviewed, logged settings change in `config/settings-changes.json`: `trading.live_requires_edge_pass` 0. The gate's code and its tests are kept; only the value changes. `/health` and the status page show a live account as `trading (edge gate lifted by the owner DD-MM)`, from the settings log.
- **One gate.** Every order, amend and close goes through `jiaoyi_ctrader::trading::send`. It re-checks the allowlist and the edge gate. For an open it also reads the broker's account type and open positions fresh: on a NETTED account there is no entry on a symbol with any position, on a HEDGED account no entry beside a position on the same symbol, and an unknown account type refuses every entry. It requires a stop, a volume on the step and at or above the broker minimum (never rounded up), and the broker's quote→deposit rate. It recomputes the risk and refuses anything above the allowance (§5's hard cap). The intent is written before the frame, under a unique entry key, so an order is never sent twice. Amends move an owned stop only in the trade's favour, from the trader's view and from the broker's own stop, read fresh; amends and closes touch only positions this program opened. cTrader's amend REPLACES a position's protection, so a stop-only amend would delete its take-profit: the gate reads the position fresh and sends its take-profit again (Strategy 1 has none; Strategy 3 will).
- **An answer that comes late is kept** (Codex round 1 on #14). An open is unresolved while it has neither a fill nor an end recorded: accepted only, or no answer within `trading.order_wait_secs`. It is never resent. A late answer to a frame of the session is recorded under its intent. Any fill, whatever the last answer, is read from the broker at once (Codex round 2): a fill in several deals, a partial fill whose remainder was cancelled or expired, or a remainder filled after the wait. The broker's aggregate volume, entry price and stop are adopted and trailed before any further entry is admitted; an execution event carries one deal, not the position. A partly filled open stays unresolved until it fills in full or its remainder ends (cancelled, expired, rejected, or absent at reconcile after its grace): the remainder's share of the intent's risk (requested less filled, at the same stop distance) is held in the budget beside the adopted part's, never both for the same volume (Codex round 3). An unresolved open's risk is held in the open-risk budget and its symbol takes no entry. Reconcile adopts a position with this program's label on an unresolved open's symbol (after a restart too), at the broker's volume. Only a position at the requested volume resolves the open (`RECONCILED_FILL`); a smaller one (`RECONCILED_PARTIAL_FILL`) keeps it unresolved, its remainder reserved as above, and the adopted open claims no second position (Codex round 4). An unresolved open with nothing at the broker after one reconcile interval is released (`RECONCILED_ABSENT`), never resent. A signal skipped for a missing conversion quote may be taken when the quote arrives.
- **Size.** Strategy 1 sizes at 0.6% of fresh equity, and below the broker minimum it skips. On a live account (`trading.live_allow_min_volume` 1) the broker minimum is taken instead when its risk is within `trading.min_volume_max_risk_pct` (10%) of equity, and only while Strategy 1 holds no other position or unresolved open on the account (owner, 25-10-2026: *"One minimum trade at a time"*). That single minimum position replaces the open-risk budget check for itself; a second entry waits until it closes. Demo accounts never take the minimum.
- **Open-risk budget** (owner, № 16,806). Strategy 1 holds at most `trading.trend_open_risk_pct` (3%) of equity in open risk at its stops: half of a 6% account limit. The other half is for Strategy 2 (stat arb on tick bars, not built). An entry that would exceed the budget is skipped with the reason.
  - The conflict raised on 25-10 (at live's $38.98 the budget is $1.17, and ETHUSD's minimum-volume trade risks $2.85, 7.3%) is settled by the owner's *"One minimum trade at a time"* above. Entries sized at 0.6% keep the 3% budget, unresolved opens included.
- **Not yet enforced:** §5's cost guard, and §6's limits other than the open-risk budget (12 positions, 2.4% per class, the drawdown brake).
- **Reviewed settings changes** are listed in `config/settings-changes.json` (the first: the lifted live edge gate). Each entry is applied once at start-up through `settings::change`: bounds-checked and logged.
- **Unverifiable until the first demo fill:**
  - whether the broker accepts a relative stop on a MARKET order on these accounts;
  - the shape of its execution events (accepted, then filled, carrying the request's id);
  - what an amend answers with;
  - the account type each account reports;
  - commission against the account's own deals.

## 12. Strategy 3: intraday momentum (owner, 25-10-2026)

- **Approval.** The owner approved Strategy 3 v2 at about 10:45 SGT, after *"Trade more often, More markets, Use the bot-trade program live trading Sl, tp1, tp2 methodology"*. The entry is *"Tick-triggered only"* (about 10:50 SGT). Live trades too: *"Live too, from now"*; *"Ignore read-only, do actual trading. That is mandate."*
- **The rules** are `docs/preregistration-2026-10-25-strategy3.md`, frozen before any trade.
- **What it trades.** All 20 symbols, each on its own session:
  - the indices on their exchange cash sessions, in their own time zones;
  - FX, metals and energy 07:00–20:00 UTC;
  - crypto the UTC day.
- **Entry.** The noise band is the session's open × (1 ± σ). The first ask above it buys; the first bid below it sells.
- **Exits** are bot-trade's `risk_coverage_partial_v1`:
  - the stop is R;
  - the planned part closes at TP1 = 3R + C;
  - the runner's broker take-profit is TP2 = TP1 + R;
  - everything left closes at the session end.
- **Size.**
  - Demo takes 0.6% of fresh equity.
  - Live takes one broker-minimum trade at a time, at most 10% of equity, with no other position of this program open.
- **Open-risk budget.** Strategy 3 holds at most `trading.s3_open_risk_pct` (3%) of equity in open risk at its stops.
- **Co-holding.** On a HEDGED account, Strategy 1 and Strategy 3 may each hold the same symbol, and each manages only its own label. External positions still block an entry. On a NETTED account, any position blocks.
- **Fills** follow §11's rules for Strategy 1 (Codex round 3 on #15):
  - Any fill is read from the broker at once: the aggregate volume, entry, stop and take-profit. An execution event carries one deal, not the position.
  - A partly filled open is linked to its plan through the deal's position, so its TP1 and session-end close are kept.
  - Reconcile adopts an unresolved open in full only at the requested volume. A smaller position is `RECONCILED_PARTIAL_FILL`, with the remainder's risk held in the budget until it fills, ends or is absent after the grace.
- **Where this departs from the sections above.** Live trades Strategy 3 without a PASS, by the owner's order. The PASS rules of the pre-registration are judged on the forward demo results. Nothing above is edited.
