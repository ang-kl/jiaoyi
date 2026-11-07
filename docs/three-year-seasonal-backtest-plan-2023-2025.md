# Three-Year Seasonal Backtest Plan (2023–2025)

№18,321 §§1–8 · 07-11-2026 11:35:00 SGT (+08:00) · Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Strategic Rationale & Synthesis of №18,318 and №18,319

§1.1. **Why a Three-Year Seasonal Backtest is Required**:
- **Escaping the 2-Date Trap**: Previous repository trials tested isolated 2-date batches in July and August (e.g. PR #168 and PR #177), capturing low-liquidity summer doldrums and Q2 earnings noise rather than generalizable market structure.
- **Integrating №18,318 Empirical Findings**: Audited 1,281,012 live ticks on Pepperstone cTrader and academic literature ([Barber & Odean](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=1498663)), proving that win rate is dictated by the **Spread-to-Range Tax** ($	au$). We eliminate high-friction destroyers (`EURUSD`, `NatGas`, altcoins) and focus exclusively on low-friction instruments (`SpotBrent`, `SpotCrude`, `US500`, `NAS100`, `XAUUSD`) where friction consumes $< 1.5\%$ of the intraday move.
- **Integrating №18,319 Structural Edge Frontier**: Algos dominate sub-second execution, but fail under regime shifts; humans excel at macro supply/demand synthesis and multi-hour to multi-day time horizon arbitrage. This plan evaluates models at horizons ($	ext{M30}, 60	ext{m}, 180	ext{m}$) where the signal-to-noise ratio is high and transaction costs are fully amortized.
- **Hard Research Seal Invariant**: All testing dates remain strictly prior to **24 October 2025** (`/owner/backtests` endpoint). The holdout validation window (`19 August – 5 September 2025`) remains sealed.

## §2. The Three-Year Macro Regime Matrix (2023–2025)

Testing 60 trading days across 12 distinct cohorts spans three completely different global macroeconomic environments:

| Year | Macroeconomic Regime | Key Market Drivers | Strategy Testing Objective |
| :--- | :--- | :--- | :--- |
| **2023** | **Terminal Rate Hike & Banking Stress** | Fed terminal rate reaches 5.25–5.50%; SVB collapse; OPEC+ production cuts; gold de-dollarization surge. | Test strategy resilience during aggressive liquidity withdrawal, yield spikes, and banking panic. |
| **2024** | **AI Capex Surge & Middle East Disruptions** | Red Sea shipping diversions; Bitcoin ETF institutional approval; tech capex boom; US election liquidity surge. | Test momentum capture in trending equity indices, geopolitical crude spikes, and crypto breakouts. |
| **2025** | **Global Realignment & Easing Cycle** | Monetary easing follow-through, fiscal realignment, physical commodity supply tightness, tariff repricing. | Benchmark against recent repository trials up to the 24 October 2025 development cut-off. |

## §3. The Liquid Core Universe vs. Blacklisted Instruments

§3.1. **The 8 Liquid Benchmark Symbols** (Allocated 100% of Backtest Compute):

| Asset Class | Symbol | Typical Spread | Spread Drag ($	au$) | Primary Session | Rationale & Strategy Fit |
| :--- | :--- | :--- | :---: | :--- | :--- |
| **`energy`** | `SpotCrude` | $0.025 (3.35 bp) | 1.35% | 13:00–17:00 UTC | Top empirical performer. 100% win rate on 180m momentum. Physical inventory trend persistence. |
| **`energy`** | `SpotBrent` | $0.028 (3.75 bp) | 1.42% | 13:00–17:00 UTC | Tight paired peer coherence with WTI; robust cTrader execution (-1.38 bp slippage). |
| **`equity_index`** | `US500` | 0.35 pts (0.60 bp) | 0.54% | 14:30–18:30 UTC | Global equity anchor. Sub-1bp spread. Strategy 3 v7 adaptive sigma bands & ORB-30m. |
| **`equity_index`** | `NAS100` | 1.20 pts (0.62 bp) | 0.85% | 14:30–18:30 UTC | High beta trend persistence; deep CME E-mini liquidity proxy on Pepperstone Razor. |
| **`metal`** | `XAUUSD` | $0.18 (0.66 bp) | 0.56% | 07:00–11:00 UTC | Global gold benchmark. London open momentum; sovereign reserve de-dollarization trend. |
| **`metal`** | `XAGUSD` | $0.015 (4.80 bp) | 2.10% | 07:00–11:00 UTC | High-beta precious metal companion; verifies cross-asset peer coherence with Gold. |
| **`equity_index`** | `GER40` | 1.20 pts (0.65 bp) | 1.95% | 08:00–12:00 UTC | European morning cash session anchor; eliminates US afternoon dependencies. |
| **`crypto`** | `BTCUSD` | $15.0 (2.20 bp) | 2.80% | 00:00–04:00 UTC | Deepest institutional crypto depth; tests volatility expansion filters. |

§3.2. **Blacklisted Instruments (0 Jobs Allocated)**:
- `NatGas`: 25% win rate in empirical audit, +3.74 bp slippage penalty, weekend gap risk.
- `EURUSD` / `USDCAD`: Intraday spread drag exceeds 8.5%–12.5% of range; pure HFT machine hegemony.
- `Altcoins` (`BCH`, `LTC`, `AVAX`, `DOT`): 100% `gap_before_quote` in PR #177; 28%+ cost drag.
- `PGMs` (`XPTUSD`, `XPDUSD`): Illiquid industrial metals; wide spreads, severe stop slippage.

## §4. The Twelve Seasonal Cohorts (60 Trading Days)

```
2023 Calendar Year (20 Dates):
├── Cohort 01 (2023 Q1 Winter / Terminal Hike Peak) : 2023-01-23 to 2023-01-27 (5 days)
├── Cohort 02 (2023 Q2 Spring / SVB Bank Stress)    : 2023-05-15 to 2023-05-19 (5 days)
├── Cohort 03 (2023 Q3 Summer / OPEC+ Cuts & Chop)  : 2023-07-24 to 2023-07-28 (5 days)
└── Cohort 04 (2023 Q4 Autumn / Yield Peak Pivot)   : 2023-11-13 to 2023-11-17 (5 days)

2024 Calendar Year (20 Dates):
├── Cohort 05 (2024 Q1 Winter / Red Sea Disruption) : 2024-01-22 to 2024-01-26 (5 days)
├── Cohort 06 (2024 Q2 Spring / Crude Driving Ramp) : 2024-05-13 to 2024-05-17 (5 days)
├── Cohort 07 (2024 Q3 Summer / Tech Earnings Dip)  : 2024-07-22 to 2024-07-26 (5 days)
└── Cohort 08 (2024 Q4 Autumn / US Election Rally)  : 2024-11-11 to 2024-11-15 (5 days)

2025 Calendar Year (20 Dates):
├── Cohort 09 (2025 Q1 Winter / Policy Realignment) : 2025-02-10 to 2025-02-14 (5 days)
├── Cohort 10 (2025 Q2 Spring / Physical Deficit)   : 2025-05-12 to 2025-05-16 (5 days)
├── Cohort 11 (2025 Q3 Summer / Baseline Benchmark) : 2025-07-21 to 2025-07-25 (5 days)
└── Cohort 12 (2025 Q4 Autumn / Pre-Holdout Cut-Off): 2025-10-06 to 2025-10-10 (5 days)
```

## §5. Strategy Models Evaluated

§5.1. **Model 1: Production Strategy 3 v7 (`s3_ticks.rs` with 45-day M30 Bars)**:
- Feeds 45 calendar days of pre-computed `M30` historical bars to construct the adaptive $\sigma$ noise band.
- Enforces $Q = 1.38$ target, $1.0R$ initial stop, and 30-second Chandelier trailing stop (high water mark $- 1.0R$ for longs).

§5.2. **Model 2: Opening Range Breakout with Range Expansion Gate (`ORB-60m-VolExp`)**:
- Evaluates session breakouts only when opening range $R > 1.25 \times \text{median}_{20}(R)$, filtering out 60%+ of low-volume consolidation traps.

§5.3. **Model 3: Centaur Multi-Hour Macro Momentum (`Macro-180m`)**:
- Replicates the top-performing 180-minute momentum rule from №18,318 that produced 100% win rates on Crude, Brent, and Gold.

§5.4. **Controls**: Matched `always_long` and fixed-horizon `own_momentum` executed for every job.

## §6. Dynamic Cost Gating (Overcoming the 48 bp Double-Bind)

§6.1. To prevent the adverse selection trap identified in PR #177 (where a flat 48 bp gate vetoed liquid majors and admitted toxic altcoins):
- **Primary Gate**: $24\text{ bp/side}$ ($48\text{ bp}$ round-trip) slippage reserve $C$ for formal baseline compliance.
- **Empirical Reality Track**: Logs execution outcomes at empirical cTrader slippage ($2.0\text{ bp/side}$ for indices/gold/crypto; $3.74\text{ bp/side}$ for crude).
- **Dynamic Volatility Threshold**: $\text{Gate} = \max(2.5 \times \text{Spread}, 0.15 \times \text{ATR}_{M30})$.

## §7. Staged Execution Plan & Compute Budget

| Stage | Cohort Scope | Dates | Target Symbols | Total Jobs | Est. Execution | Output Deliverable |
| :--- | :--- | :---: | :--- | :---: | :---: | :--- |
| **Stage 1 (Pilot)** | 6 Cohorts (2 per year across 2023, 2024, 2025) | `30` | `SpotCrude`, `SpotBrent`, `US500`, `XAUUSD` (`4` symbols) | **`120`** | ~25–35 min | `doc/THREE_YEAR_PILOT_REPORT-18322-2026-11-07.md` |
| **Stage 2 (Full)** | All 12 Cohorts across all 3 years | `60` | All `8` Liquid Benchmark symbols | **`480`** | ~75–90 min | `doc/THREE_YEAR_FULL_SEASONAL_REPORT-18323-2026-11-07.md` |

## §8. Quantitative Acceptance & Stopping Rules

§8.1. **Pass Criteria for Repository Mainline Admission**:
1. $\ge 20$ closed trades per asset class spanning $\ge 4$ independent cohorts.
2. $\ge 50\%$ net win rate under primary $24\text{ bp/side}$ trailing cost accounting.
3. Positive net expectancy (`mean_net_r > 0`) and `profit_factor > 1.25` across all 3 years.
4. Zero unhandled data gaps (`data_gap_reasons == []`).

§8.2. **Definitive Stopping Rule**:
- If any candidate model fails to achieve positive net expectancy across both 2023 and 2024 under its active season, parameter tuning on that candidate is permanently stopped. Three-year failure proves structural lack of edge.

