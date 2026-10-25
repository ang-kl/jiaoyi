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

## 11. The order path (owner, 25-10-2026)

Owner, 25-10-2026: *"I want live to execute trading by 1 PM today"*, then *"Keep the gate; demo trades at 1 PM"* and, for live, *"Allow the minimum volume"*. This section records what the order path does and where it departs from the sections above. Nothing above is edited.

- **One gate.** Every order, amend and close goes through `jiaoyi_ctrader::trading::send`. It re-checks the allowlist, the account's switch and its edge gate. For an open it also reads the broker's account type and open positions fresh: on a NETTED account no entry on a symbol with any position; on a HEDGED account no entry beside a position on the same symbol; an unknown account type refuses every entry. It requires a stop, a volume on the step and at or above the broker minimum (never rounded up), and the broker's quote→deposit rate; it recomputes the risk and refuses above the allowance (§5's hard cap). The intent is written before the frame, under a unique entry key, so an order is never sent twice. Amends move an owned stop only in the trade's favour; closes and amends touch only positions this program opened.
- **The strategy is the pre-registered H1 default** (`docs/preregistration-2026-10-24.md`): breakout 50, stop 3 × ATR(20), with an entry trigger on the tick and a trail behind the best bid (ask for shorts) since the fill. Where §4, §5 and §7 differ (once-a-day signals, a trail on closes, "no tick entries"), H1 is what runs: it is the hypothesis under test, approved by the owner on 24-10.
- **Switches, per account, all off by default** (migration 0012): `trading.enabled`, `trading.require_edge_pass` (1: nothing is sent until the H1 held-out trial under the running pre-registration hash is PASS), `trading.allow_min_volume` with `trading.min_volume_max_risk_pct` (10). §6a says there is no trading switch (owner, 24-10). The owner's 25-10 orders, demo first and live behind the edge gate, need one per account; it is set only by a logged settings change. The reviewed changes are listed in `config/settings-changes.json`, and each entry is applied once at start-up through `settings::change`.
- **Not yet enforced:** §5's cost guard and §6's portfolio limits (12 positions, 6% total risk, 2.4% per class, the drawdown brake). Until they are, each account holds at most one H1 position per symbol, each at the per-trade risk.
- **Unverifiable until the first demo fill:** whether the broker accepts a relative stop on a MARKET order on these accounts; the shape of its execution events (accepted, then filled, carrying the request's id); what an amend answers with; the account type each account reports; and commission against the account's own deals.
