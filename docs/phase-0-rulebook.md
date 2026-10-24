# Phase 0 rulebook: jiaoyi daily trend system

Status: **DRAFT for the owner's approval.** Nothing in this file can be traded until the owner approves it and phase 3 (the history test) passes.

Every number here is fixed in advance. Changing a number after a test has run counts as a new trial. It is recorded in the trial log (`docs/storage.md`, table `research_trials`) and judged as such.

Ordered by the owner, 24-10-2026 (bot-trade conversation, № 16,790–16,795):
- rebuild the whole program from the ground up;
- new repository `ang-kl/jiaoyi`;
- Rust;
- PostgreSQL;
- services in the existing Railway project;
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
| All other accounts | Not used | — |

The old bot-trade program and the new system never trade the same account at the same time.

## 3. Universe (candidates; phase 1 confirms each on each account)

All 20 names below appear in the bot's own broker deal statements (bot-trade `agent/config/tick-shadow-sim.json`, measured 21-08-2026).

| Class | Symbols |
|---|---|
| Equity indices (7) | US500, NAS100, US30, GER40, UK100, JPN225, AUS200 |
| FX (6) | EURUSD, GBPUSD, USDJPY, AUDUSD, USDCAD, NZDUSD |
| Metals (2) | XAUUSD, XAGUSD |
| Energy (3) | USOIL, UKOIL, NATGAS |
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
