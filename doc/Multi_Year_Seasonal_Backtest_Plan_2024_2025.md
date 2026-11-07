# Multi-Year Seasonal Backtest Plan (2024–2025)

№18,314 §§1–8 · 07-11-2026 10:55:00 SGT (+08:00) · Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Strategic Rationale & Invariant Boundaries

§1.1. **Why Multi-Year Seasonal Stratification Replaces the 2-Date Trap**:
- Every previous trial in `ang-kl/jiaoyi` evaluated only **two isolated dates** (e.g. `5` & `12 August 2025` in PR #160/#168, or `7` & `14 July 2025` in PR #177). Two isolated dates reflect single-day macro weather rather than repeatable market dynamics.
- Moreover, July and August are the **lowest-liquidity summer doldrum months of the calendar year**, coinciding with Q2 US earnings disruptions. Testing exclusively in summer biased results toward low-volume chop, false breakouts, and pre-market data gaps.
- This plan establishes a **40-date, 8-cohort stratified seasonal backtest across 2024 and 2025**, capturing all four distinct market seasons across two complete macroeconomic calendar years before the 24 October 2025 development seal.

§1.2. **Non-Negotiable Invariants**:
- All dates are strictly prior to **24 October 2025**, fully respecting the `/owner/backtests` pre-holdout development seal.
- Account goal remains **73% net wins (15 of 20) at reference risk-reward 1:1.38** over the latest 20 trades per strategy/account.
- Interim research gate: $\ge 50\%$ net wins, positive net expectancy (`mean_net_r > 0`), `profit_factor > 1.0`, positive net price-unit P&L per symbol, and zero data-coverage gaps.
- Unbiased accounting: Retain all losses, cost-vetoed episodes, and censored trades in denominators; never delete adverse outcomes.

## §2. The Liquid Core Universe (7 Symbols Across 4 Asset Classes)

§2.1. PR #177 proved that diluting tests across 40 symbols added illiquid altcoins and thin PGMs where **100% of entered trades had `gap_before_quote` data flags** and 62.5% of episodes failed scheduled 5-minute sampling. This multi-year plan restricts testing strictly to the **7 high-liquidity benchmark symbols**:

| Asset Class | Symbols | Primary Session Window (UTC) | Liquidity & Market Microstructure Rationale |
| :--- | :--- | :--- | :--- |
| **`equity_index`** | `US500`, `NAS100`, `GER40` | `14:30–18:30 UTC` (US); `08:00–12:00 UTC` (GER) | Top global equity benchmarks; continuous institutional market depth; eliminates quote gaps. |
| **`energy`** | `SpotCrude` (WTI), `SpotBrent` | `13:00–17:00 UTC` (NYMEX/ICE liquid core) & `07:00–11:00 UTC` | Tests the paired crude benchmark during active trading rather than illiquid pre-market hours. |
| **`crypto`** | `BTCUSD`, `ETHUSD` | `00:00–04:00 UTC` | Deepest 24/7 liquidity; avoids altcoin tick starvation (`LINK`, `AVAX`, `DOT`). |
| **`metal`** | `XAUUSD` (Spot Gold) | `07:00–11:00 UTC` (London open) | Global physical and institutional gold benchmark; deepest precious metal liquidity. |

## §3. Eight Seasonal Cohorts Across 2024 & 2025 (40 Trading Dates)

§3.1. Sampling 5 consecutive trading days (Monday through Friday) per quarter captures full weekly market flow across all 4 seasonal macro regimes:

```
2024 Calendar Year (20 Dates):
├── Cohort 1 (2024 Q1 Winter / Heating Peak)    : 2024-01-22 to 2024-01-26 (Q4 earnings kickoff; winter heating draws)
├── Cohort 2 (2024 Q2 Spring / Driving Ramp)    : 2024-05-13 to 2024-05-17 (Pre-Memorial Day refinery crude ramp)
├── Cohort 3 (2024 Q3 Summer / Doldrums & Tech) : 2024-07-22 to 2024-07-26 (Summer liquidity dip; Q2 tech earnings)
└── Cohort 4 (2024 Q4 Autumn / US Election Rally): 2024-11-11 to 2024-11-15 (Post-election institutional flow surge)

2025 Calendar Year (20 Dates):
├── Cohort 5 (2025 Q1 Winter / Macro Pivot)     : 2025-02-10 to 2025-02-14 (Tech earnings follow-through; winter macro)
├── Cohort 6 (2025 Q2 Spring / Pre-Summer Ramp) : 2025-05-12 to 2025-05-16 (Refinery expansion; spring trend runs)
├── Cohort 7 (2025 Q3 Summer / August Baseline) : 2025-07-21 to 2025-07-25 (Pre-holdout summer baseline benchmark)
└── Cohort 8 (2025 Q4 Pre-Holdout Cut-off)     : 2025-10-06 to 2025-10-10 (Pre-24 Oct holdout cut-off; Q4 kickoff)
```

## §4. Models Under Test (Side-by-Side Execution)

§4.1. **Model Track A — Production Strategy 3 v7 (`s3_ticks.rs` with 45-day `M30` Bars)**:
- Replays the actual production Strategy 3 engine (`crates/jiaoyi-core/src/research/s3_ticks.rs`), feeding the required 45 calendar days of historical `M30` bars (`BAR_DAYS = 45`) to compute the adaptive $\sigma$ noise band.
- Decision at 30-minute marks; entry when mid crosses upper/lower noise band; $Q = 1.38$ target; $1.0R$ stop; 30-second Chandelier trailing stop (best $- R$ for long, best $+ R$ for short).

§4.2. **Model Track B — Opening Range Breakout with Range-Expansion Filter (`ORB-30m` & `ORB-60m`)**:
- Causal first 30-minute (or 60-minute) high/low breakout.
- **Range Expansion Gate**: Enters only when opening range $R > 1.25 \times \text{median}_{20}(R)$, filtering out low-volume summer chop and ensuring institutional participation.

§4.3. **Matched Controls**:
- Every episode runs alongside `own_momentum` (fixed horizon) and `always_long` to ensure any positive outcome reflects true predictive edge rather than passive market drift.

## §5. Dual Cost-Gate Accounting (Resolving the 48 bps Trap)

§5.1. **Primary Screen Gate ($24\text{ bp/side}$)**:
- Retains the conservative $24\text{ bp/side}$ ($48\text{ bp}$ round-trip) slippage reserve $C$ for admission and target geometry, preserving strict continuity with existing research seals.

§5.2. **Calibrated Empirical Sensitivity ($2.0\text{ bp}$ and $3.74\text{ bp/side}$)**:
- Simultaneously logs execution outcomes evaluated at the empirical cTrader slippage benchmarks from `cost_coverage.groups` ($2.0\text{ bp/side}$ for Indices/Crypto/Gold; $3.74\text{ bp/side}$ for Oil).
- Distinguishes whether a non-entry is an **economic feasibility failure** (vetoed solely by the 48 bp assumption) versus a **signal failure**.

## §6. Staged Execution Plan & cTrader Job Budgeting

§6.1. To stay well within cTrader API rate limits and Railway memory constraints:

| Stage | Scope & Cohorts | Dates | Target Symbols | Total Broker Jobs | Expected Execution Time | Output Deliverable |
| :--- | :--- | :---: | :--- | :---: | :---: | :--- |
| **Stage 1 (Pilot)** | 4 Seasonal Cohorts (1 per season across 2024–2025) | `20` | `SpotCrude`, `SpotBrent`, `US500`, `BTCUSD` (`4` symbols) | **`80`** | ~15–20 minutes | `docs/seasonal-pilot-results-2024-2025.json` |
| **Stage 2 (Full)** | Complete 8 Seasonal Cohorts across both years | `40` | All `7` Liquid Core symbols | **`280`** | ~45–60 minutes | `docs/multi-year-seasonal-results-2024-2025.json` |

## §7. Acceptance & Stopping Criteria

§7.1. **Pass Criteria for Opening Validation**:
- $\ge 20$ closed scenarios per class across $\ge 3$ distinct calendar weeks.
- $\ge 50\%$ net win rate per symbol and class under primary $24\text{ bp/side}$ trailing exits.
- Positive net expectancy (`mean_net_r > 0`) and `profit_factor > 1.0`.
- Statistically significant positive delta over matched `always_long` and `own_momentum` controls.

§7.2. **Definitive Stopping Rule**:
- If a candidate fails positive net expectancy across both 2024 and 2025 under its active season, **halt all further parameter tuning on that candidate**. Multi-year seasonal failure constitutes definitive proof of lack of edge.
