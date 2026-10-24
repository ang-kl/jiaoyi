# Pre-registration: H1 and H2 edge tests (24-10-2026)

Status: **APPROVED by the owner, 24-10-2026, 18:50 SGT**, by approving the plan in the bot-trade conversation (№ 16,852, ordered by "Do 16,850·D Next").

This file is frozen.
- Every research trial records the SHA-256 of this file's content as its pre-registration hash.
- A change to any rule below is a new pre-registration in a new file, recorded as such. It never edits this one.

Strategy sources:
- **H1:** the tick-triggered trend (bot-trade conversation № 16,806; rulebook §4–§5).
- **H2:** statistical arbitrage on tick-imbalance bars (№ 16,805·B).
- **Test design:** rulebook §8, and the edge-verification plan adopted on 24-10-2026.

## Data inventory behind the design (read-only, `/research/inventory`, 24-10-2026 10:46Z)

Source: live 43002148 (real-money prices). Demo 49236051 is a cross-check only.

| Data | Coverage |
|---|---|
| Daily bars | FX 1978/1989/1996–; metals 1998/2002–; indices 2008/2014–; crypto 2010/2016–; energy 2013/2014–. Demo matches within a few bars |
| Broker tick history (earliest week, probe) | FX and metals 2013-07-22; indices 2015-01-19/26; BTC 2017-05-15; ETH 2017-11-20; SpotCrude 2020-04-20; NatGas 2020-11-16; SpotBrent not yet probed |
| Ticks stored | 58.0M rows for about 1.7 weeks of 19 symbols, both sides. One week of all symbols is about 30M rows |
| Download speed (measured) | 76 symbol-side-weeks plus 203 probe steps in 50 min |
| Deals (commission) | Live: 434 deals, 160 positions, 21-10-2025 → 16-01-2026. Demo: 38 deals, 18 positions, 20–23 Oct 2026 |
| Spreads | Measured from stored ticks: about 1.7 weeks, Oct 2026 only |

What these facts force:
- **No raw tick archive.** A year of raw ticks (about 1.5B rows) does not fit the 50 GB database disk.
  - Research reads broker ticks as a stream and stores only the bars and results it derives.
  - Raw `ticks_history` stays within `history.tick_days_back`.
- **H1 runs on daily bars.** Ticks are fetched only to re-check the fills of held-out trades on their entry and exit days.
- **H2 runs on sampled weeks of ticks:**
  - development: 52 random weeks, 2019-01 → 2025-10, seed 20261024;
  - held-out: every second week of the last 12 months (26 weeks).

## Shared by H1 and H2

- **Held-out period:** 2025-10-24 → 2026-10-23, the last 12 months.
  - It is opened ONCE, and only if the default setting passed every development rule below.
  - Otherwise the verdict is "no demonstrated edge" and the held-out data stays sealed.
- **Development period:** everything before 2025-10-24.
- **Prices:** live 43002148's bars and ticks. Fills at the quote side: buys at the ask, sells at the bid.
- **Costs,** each labelled measured or estimated:
  - **Spread:** from ticks (measured). For daily-bar years before tick history it is an estimate: the earliest measured median.
  - **Commission:** per symbol class, from live's own deal history (measured).
  - **Swap:** today's broker rates applied to each night held (estimated for past years).
  - **Slippage:** 0×, 1× and 2× the median spread. Pass/fail is judged at 1×.
- **Unit:** R = price move ÷ initial stop distance, net of all costs. Money is shown beside R.
- **Control:** 1,000 random-entry runs on the same symbols or baskets.
  - Same number of entries per symbol per year.
  - Same exit rules and costs.
  - Direction random.
  - H1 is also compared with buy-and-hold over its days in the market.
- **PASS:** all of these on the held-out data, at the default setting:
  1. Mean net R per trade > 0, AND the 95% lower bound of a week-block bootstrap (10,000 resamples) > 0.
  2. Mean net R above the 95th percentile of the 1,000 control runs.
  3. Profit factor ≥ 1.3 (owner's figure).
  4. At least 80 trades for H1, or 200 trades for H2.
     - Fewer gives the verdict **"insufficient evidence"**: not a pass, and it goes to the owner.
- **Development gate**, before the held-out data is opened:
  - the default setting meets rules 1–3 on development data;
  - at least 6 of the 9 grid cells have profit factor > 1.0.
- **Trial cap:** the default plus a 3×3 grid per hypothesis, nothing else.
  - Every trial is written to `research_trials` with the pre-registration hash.
  - The Sharpe ratio is reported deflated for the number of trials.
- **Report, per strategy:** trades, period, symbols, net R and money with the bootstrap range, the result against the control, per-year results, and the strongest evidence against the finding.
- **Consequence:** a strategy without a PASS is not traded live. Owner, 24-10: live trading waits for a demonstrated edge.

## H1: tick-triggered trend (№ 16,806)

- **Universe:** the 20 symbols. Each symbol's test starts once both 100 daily bars and its spread measure exist.
- **Filter:** EMA(50) of completed daily closes above EMA(100) → longs only; below → shorts only.
- **Entry:**
  - long: the first ask at or above the highest high of the previous 50 completed days;
  - short: the first bid at or below the lowest low of the previous 50 completed days;
  - one position per symbol.
  - Daily-bar fill: the level, or the day's open if the day opened beyond it, moved to the side of the quote the order fills on: buys at the ask, sells at the bid.
    - The quote basis of the broker's daily bars (bid, ask or mid) is measured against stored ticks and recorded, never assumed.
    - Costs are always adverse:
      - a buy fills at the bar price converted to the ask, plus slippage;
      - a sell fills at the bar price converted to the bid, minus slippage.
    - The same rule applies to exits.
- **Exit:**
  - the stop starts 3 × ATR(20) from the fill (Wilder, completed days);
  - it trails 3 × ATR behind the best price since entry, and only ever moves in the trade's favour;
  - no take-profit.
  - Daily-bar rule: on a day whose range reaches both a new peak and the stop, the stop is assumed hit first (worst case).
  - Held-out trades are re-checked tick by tick on their entry and exit days, and the tick result is the one counted.
- **Size:** risk 0.6% of equity per trade, below the minimum volume → skipped. This affects money only; R is unaffected.
- **Grid:** breakout length {20, 50, 100} × stop {2, 3, 4} × ATR. The default is (50, 3).

## H2: stat arb on tick-imbalance bars (№ 16,805·B)

- **Baskets:**
  - US indices: US500, NAS100, US30. US2000 is not in the universe, so it is not added.
  - FX dollar block: EURUSD, GBPUSD, AUDUSD, NZDUSD.
  - Metals: XAUUSD, XAGUSD.
- **Bars:** tick-imbalance bars per instrument. The expected-imbalance window is set in development to about 50 bars a day. The spread is sampled at each bar close of the basket's first leg, with the other legs at their latest mid price.
- **Hedge ratios:**
  - Fitted every day at 21:00 UTC on the previous 5 trading days: OLS for a pair, Johansen for 3–4 legs.
  - A basket trades that day only if its spread rejects a unit root, p < 0.05.
- **Entry and exit:**
  - enter at |z| ≥ 2, where z uses the fitting window's mean and standard deviation;
  - exit at z = 0;
  - stop at |z| ≥ 4, or after 3 × the spread's half-life, capped at 5 days.
- **Cost guard:** enter only if the expected move back to the mean is at least 3 × the round-trip cost of all legs.
- **Execution:**
  - each leg fills at the next quote after the decision;
  - a stress run fills one leg one quote later and is reported.
- **Size:** 0.6% of equity at the 4-standard-deviation stop.
- **Grid:** entry z {1.5, 2.0, 2.5} × bars a day {25, 50, 100}. The default is (2.0, 50).
