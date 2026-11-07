# Turnkey Autonomous Coding Agent Prompt — Three-Year Seasonal Backtest Plan & Stage 1 Pilot Execution (`№18,322`)

№18,322 §§1–8 · 07-11-2026 11:38:00 SGT (+08:00) · Target Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Master Execution Directive for Coding Agent

§1.1. You are operating as the autonomous engineer in `https://github.com/ang-kl/jiaoyi`. The owner instructed: `"Run attached next-step-agent-prompt-18322-2026-11-07.md"`. This prompt is completely turnkey and self-contained—it embeds the exact Markdown text, JSON configuration, git PR workflow, Stage 1 collection/replay execution (120 jobs), report generation to `doc/`, and the final summary copy-paste block.

§1.2. **Mandatory Invariants**:
- Use serial `№18,322` (and next sequential `№18,323+`), `SGT (+08:00)` timestamps, and `§` paragraph numbering across commits, PRs, and reports.
- All 60 dates across the 12 seasonal cohorts are strictly between `2023-01-23` and `2025-10-10`, strictly before the `2025-10-24` `/owner/backtests` development seal.
- Do NOT open validation (`19 Aug – 5 Sep 2025`), alter live account permissions, modify the 17 production symbols, or change the `73%` net wins / `1:1.38` R:R account goal.
- Never remove collection receipts or retry failed jobs to manufacture a pass; retain all losses, cost-vetoed episodes, and data gaps in denominators.

---

## §2. Step 1: Write `docs/three-year-seasonal-plan-2023-2025.json`

§2.1. Create `docs/three-year-seasonal-plan-2023-2025.json` on branch `plan/three-year-seasonal-backtest-2023-2025` with this exact content:

```json
{
  "plan_id": "three-year-seasonal-backtest-plan-2023-2025",
  "serial": "\u211618,321",
  "timestamp_sgt": "07-11-2026 11:35:00 SGT",
  "hard_research_seal": "2025-10-24",
  "total_years": 3,
  "total_cohorts": 12,
  "total_trading_dates": 60,
  "universe_active": [
    {
      "symbol": "SpotCrude",
      "name": "WTI Light Sweet Crude Oil",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "energy",
      "typical_spread": "0.025 USD (3.35 bps)",
      "spread_drag_pct": "1.35%",
      "primary_window_utc": "13:00-17:00",
      "strategy_fit": "180m Momentum & ORB-60m (Physical supply chain trend inertia)"
    },
    {
      "symbol": "SpotBrent",
      "name": "Brent North Sea Crude Oil",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "energy",
      "typical_spread": "0.028 USD (3.75 bps)",
      "spread_drag_pct": "1.42%",
      "primary_window_utc": "13:00-17:00",
      "strategy_fit": "180m Momentum & ORB-60m (Paired crude peer coherence)"
    },
    {
      "symbol": "US500",
      "name": "S&P 500 Cash CFD",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "equity_index",
      "typical_spread": "0.35 pts (0.60 bps)",
      "spread_drag_pct": "0.54%",
      "primary_window_utc": "14:30-18:30",
      "strategy_fit": "Strategy 3 v7 & ORB-30m (CME E-mini liquidity, sub-1bp friction)"
    },
    {
      "symbol": "NAS100",
      "name": "Nasdaq 100 Cash CFD",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "equity_index",
      "typical_spread": "1.20 pts (0.62 bps)",
      "spread_drag_pct": "0.85%",
      "primary_window_utc": "14:30-18:30",
      "strategy_fit": "Strategy 3 v7 & ORB-30m (Deep liquidity, high intraday range)"
    },
    {
      "symbol": "XAUUSD",
      "name": "Spot Gold",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "metal",
      "typical_spread": "0.18 USD (0.66 bps)",
      "spread_drag_pct": "0.56%",
      "primary_window_utc": "07:00-11:00",
      "strategy_fit": "London Open Momentum & ORB-60m (Sovereign reserve flow trend)"
    },
    {
      "symbol": "XAGUSD",
      "name": "Spot Silver",
      "tier": "Tier 1 (High Alpha)",
      "asset_class": "metal",
      "typical_spread": "0.015 USD (4.80 bps)",
      "spread_drag_pct": "2.10%",
      "primary_window_utc": "07:00-11:00",
      "strategy_fit": "Precious metals peer coherence with Gold; high-beta trend"
    },
    {
      "symbol": "GER40",
      "name": "DAX 40 Cash CFD",
      "tier": "Tier 2 (Tactical)",
      "asset_class": "equity_index",
      "typical_spread": "1.20 pts (0.65 bps)",
      "spread_drag_pct": "1.95%",
      "primary_window_utc": "08:00-12:00",
      "strategy_fit": "Frankfurt/London morning cash session breakout"
    },
    {
      "symbol": "BTCUSD",
      "name": "Bitcoin / USD",
      "tier": "Tier 2 (Tactical)",
      "asset_class": "crypto",
      "typical_spread": "15.0 USD (2.20 bps)",
      "spread_drag_pct": "2.80%",
      "primary_window_utc": "00:00-04:00",
      "strategy_fit": "Volatility expansion breakout; institutional ETF flow momentum"
    }
  ],
  "universe_blacklisted": [
    {
      "symbol": "NatGas",
      "reason": "Widow-maker microstructural decay: 25% win rate, +3.74 bp slippage penalty, roll curve gaps."
    },
    {
      "symbol": "EURUSD / USDCAD",
      "reason": "Spread-to-range tax destroys edge: 1.0 pip spread on 12-15 pip intraday move = 8.5% drag."
    },
    {
      "symbol": "Altcoins (BCH/LTC/AVAX)",
      "reason": "100% gap_before_quote warnings in PR #177; thin order books, 28%+ friction drag."
    },
    {
      "symbol": "PGMs (XPTUSD/XPDUSD)",
      "reason": "Illiquid industrial metals; wide spreads, severe stop slippage (-0.92R to -1.59R in PR #177)."
    }
  ],
  "models": [
    {
      "model_id": "S3_V7_M30",
      "name": "Production Strategy 3 v7 (s3_ticks.rs)",
      "description": "Adaptive 45-day M30 sigma-noise bands with 30s Chandelier trailing stop (Target Q=1.38, Risk 1.0R).",
      "execution_cadence": "30-minute bar boundaries",
      "filter": "Mid crosses adaptive sigma threshold; peer coherence check"
    },
    {
      "model_id": "ORB_60M_VOL_EXP",
      "name": "Opening Range Breakout with Range Expansion Gate (ORB-60m)",
      "description": "First 60-minute causal session high/low breakout with dynamic range expansion filter R > 1.25 * median20(R).",
      "execution_cadence": "End of first session hour",
      "filter": "Eliminates low-volume summer chop; enters only with institutional volume participation"
    },
    {
      "model_id": "MACRO_180M_MOM",
      "name": "Centaur Multi-Hour Macro Momentum (180m)",
      "description": "Multi-hour trend following capturing physical commodity and index momentum (proven 100% win rate on 5 Nov 2026).",
      "execution_cadence": "Rolling 180-minute window",
      "filter": "Unanimous directional alignment across asset class peers"
    }
  ],
  "cohorts": [
    {
      "id": "2023-Q1-WINTER",
      "year": 2023,
      "quarter": "Q1",
      "season": "Winter / Heating Peak & Terminal Rate Surge",
      "dates": [
        "2023-01-23",
        "2023-01-24",
        "2023-01-25",
        "2023-01-26",
        "2023-01-27"
      ],
      "macro_theme": "Aggressive central bank rate hikes, winter distillate draws, tech earnings kickoff"
    },
    {
      "id": "2023-Q2-SPRING",
      "year": 2023,
      "quarter": "Q2",
      "season": "Spring / US Driving Season & SVB Aftermath",
      "dates": [
        "2023-05-15",
        "2023-05-16",
        "2023-05-17",
        "2023-05-18",
        "2023-05-19"
      ],
      "macro_theme": "Pre-Memorial Day refinery crude ramp, regional US banking stabilization, debt ceiling negotiations"
    },
    {
      "id": "2023-Q3-SUMMER",
      "year": 2023,
      "quarter": "Q3",
      "season": "Summer / Doldrums & Energy Supply Cuts",
      "dates": [
        "2023-07-24",
        "2023-07-25",
        "2023-07-26",
        "2023-07-27",
        "2023-07-28"
      ],
      "macro_theme": "OPEC+ voluntary production cuts, summer liquidity trough, Fed July 25bp rate hike"
    },
    {
      "id": "2023-Q4-AUTUMN",
      "year": 2023,
      "quarter": "Q4",
      "season": "Autumn / Yield Curve Peak & Year-End Reversal",
      "dates": [
        "2023-11-13",
        "2023-11-14",
        "2023-11-15",
        "2023-11-16",
        "2023-11-17"
      ],
      "macro_theme": "10-year US Treasury yield pull-back from 5.0%, massive risk-on rally kickoff in indices and gold"
    },
    {
      "id": "2024-Q1-WINTER",
      "year": 2024,
      "quarter": "Q1",
      "season": "Winter / Red Sea Disruption & AI Capex Wave",
      "dates": [
        "2024-01-22",
        "2024-01-23",
        "2024-01-24",
        "2024-01-25",
        "2024-01-26"
      ],
      "macro_theme": "Middle East tanker rerouting around Cape of Good Hope, Bitcoin ETF institutional approval, mega-cap AI capex"
    },
    {
      "id": "2024-Q2-SPRING",
      "year": 2024,
      "quarter": "Q2",
      "season": "Spring / Physical Refinery Ramp & Inflation Stickiness",
      "dates": [
        "2024-05-13",
        "2024-05-14",
        "2024-05-15",
        "2024-05-16",
        "2024-05-17"
      ],
      "macro_theme": "Refinery utilization surge, central bank gold buying acceleration, sticky CPI print repricing"
    },
    {
      "id": "2024-Q3-SUMMER",
      "year": 2024,
      "quarter": "Q3",
      "season": "Summer / Doldrums & Pre-Easing Repositioning",
      "dates": [
        "2024-07-22",
        "2024-07-23",
        "2024-07-24",
        "2024-07-25",
        "2024-07-26"
      ],
      "macro_theme": "Q2 tech earnings volatility, pre-Jackson Hole policy easing speculation, low-volume summer chop"
    },
    {
      "id": "2024-Q4-AUTUMN",
      "year": 2024,
      "quarter": "Q4",
      "season": "Autumn / US Election Flow & Global Easing Cycle",
      "dates": [
        "2024-11-11",
        "2024-11-12",
        "2024-11-13",
        "2024-11-14",
        "2024-11-15"
      ],
      "macro_theme": "Post-US Presidential Election liquidity surge, aggressive index momentum breakouts, crypto halving rally"
    },
    {
      "id": "2025-Q1-WINTER",
      "year": 2025,
      "quarter": "Q1",
      "season": "Winter / Global Macro Realignment",
      "dates": [
        "2025-02-10",
        "2025-02-11",
        "2025-02-12",
        "2025-02-13",
        "2025-02-14"
      ],
      "macro_theme": "Fiscal realignment, global tariff discussions, winter energy demand inventory draws"
    },
    {
      "id": "2025-Q2-SPRING",
      "year": 2025,
      "quarter": "Q2",
      "season": "Spring / Physical Commodity Tightness",
      "dates": [
        "2025-05-12",
        "2025-05-13",
        "2025-05-14",
        "2025-05-15",
        "2025-05-16"
      ],
      "macro_theme": "Refinery capacity bottlenecks, physical commodity deficit trends, sovereign reserve de-dollarization"
    },
    {
      "id": "2025-Q3-SUMMER",
      "year": 2025,
      "quarter": "Q3",
      "season": "Summer / Pre-Holdout Benchmark Baseline",
      "dates": [
        "2025-07-21",
        "2025-07-22",
        "2025-07-23",
        "2025-07-24",
        "2025-07-25"
      ],
      "macro_theme": "Benchmark comparative cohort matching historical July/August development trials"
    },
    {
      "id": "2025-Q4-AUTUMN",
      "year": 2025,
      "quarter": "Q4",
      "season": "Autumn / Pre-24 October Development Cut-Off",
      "dates": [
        "2025-10-06",
        "2025-10-07",
        "2025-10-08",
        "2025-10-09",
        "2025-10-10"
      ],
      "macro_theme": "Terminal development cohort strictly preceding the 24 October 2025 research seal"
    }
  ],
  "execution_stages": [
    {
      "stage": 1,
      "name": "Pilot Calibration",
      "cohort_count": 6,
      "cohort_ids": [
        "2023-Q1-WINTER",
        "2023-Q3-SUMMER",
        "2024-Q1-WINTER",
        "2024-Q3-SUMMER",
        "2025-Q1-WINTER",
        "2025-Q3-SUMMER"
      ],
      "symbols": [
        "SpotCrude",
        "SpotBrent",
        "US500",
        "XAUUSD"
      ],
      "dates_count": 30,
      "total_jobs": 120,
      "runtime_estimate": "25-35 minutes",
      "output_file": "doc/THREE_YEAR_PILOT_REPORT-18322-2026-11-07.md"
    },
    {
      "stage": 2,
      "name": "Full Three-Year Seasonal Verification",
      "cohort_count": 12,
      "cohort_ids": [
        "2023-Q1-WINTER",
        "2023-Q2-SPRING",
        "2023-Q3-SUMMER",
        "2023-Q4-AUTUMN",
        "2024-Q1-WINTER",
        "2024-Q2-SPRING",
        "2024-Q3-SUMMER",
        "2024-Q4-AUTUMN",
        "2025-Q1-WINTER",
        "2025-Q2-SPRING",
        "2025-Q3-SUMMER",
        "2025-Q4-AUTUMN"
      ],
      "symbols": [
        "SpotCrude",
        "SpotBrent",
        "US500",
        "NAS100",
        "XAUUSD",
        "XAGUSD",
        "GER40",
        "BTCUSD"
      ],
      "dates_count": 60,
      "total_jobs": 480,
      "runtime_estimate": "75-90 minutes",
      "output_file": "doc/THREE_YEAR_FULL_SEASONAL_REPORT-18323-2026-11-07.md"
    }
  ],
  "acceptance_criteria": {
    "min_trades_per_class": 20,
    "min_win_rate_net": 0.5,
    "target_win_rate_net": 0.73,
    "target_rr": 1.38,
    "min_mean_net_r": 0.05,
    "min_profit_factor": 1.25,
    "max_data_gaps": 0
  }
}
```

---

## §3. Step 2: Write `docs/three-year-seasonal-backtest-plan-2023-2025.md`

§3.1. Create `docs/three-year-seasonal-backtest-plan-2023-2025.md` with this exact content:

```markdown
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

```

---

## §4. Step 3: Run Git Commands to Open PR & Merge to `main`

§4.1. Run the following shell sequence to create the PR, wait for CI, and squash-merge:

```bash
git checkout -b plan/three-year-seasonal-backtest-2023-2025
git add docs/three-year-seasonal-backtest-plan-2023-2025.md \
        docs/three-year-seasonal-plan-2023-2025.json
git commit -m "Research & Docs: freeze three-year seasonal backtest plan for 2023–2025 (№18,322)"
git push -u origin plan/three-year-seasonal-backtest-2023-2025

gh pr create \
  --title "Research & Docs: freeze three-year seasonal backtest plan for 2023–2025" \
  --body "## №18,322 §§1–4 · Summary
- **§1. Three-Year Stratified Seasonal Architecture**: Expands multi-year testing across 2023, 2024, and 2025 (12 cohorts, 60 trading dates total, all strictly pre-24 Oct 2025 development seal).
- **§2. Microstructural Universe (№18,318 Integration)**: Allocates 100% of compute to 8 liquid benchmark symbols (\`SpotCrude\`, \`SpotBrent\`, \`US500\`, \`NAS100\`, \`GER40\`, \`XAUUSD\`, \`XAGUSD\`, \`BTCUSD\`), strictly blacklisting high-friction destroyers (\`NatGas\`, \`EURUSD\`, \`USDCAD\`, altcoins).
- **§3. Human vs Machine Structural Alignment (№18,319 Integration)**: Tests production Strategy 3 v7 (\`s3_ticks.rs\` with 45-day M30 bars) side-by-side with Opening Range Breakout (with volatility expansion filters) and 180m macro momentum.
- **§4. Staged Execution**: Authorizes Stage 1 Pilot (6 seasonal cohorts across all 3 years, 4 symbols = 120 jobs) before full Stage 2 expansion."

gh pr checks --watch
gh pr merge --squash --delete-branch
```

---

## §5. Step 4: Execute the Stage 1 Pilot Collection & Replay (120 Jobs)

§5.1. Pull the updated `main` branch:
```bash
git checkout main && git pull origin main
```

§5.2. Execute the Stage 1 Pilot collection and replay for the **6 seasonal cohorts** across all three years:
- `2023-Q1-WINTER` (`2023-01-23` to `2023-01-27`)
- `2023-Q3-SUMMER` (`2023-07-24` to `2023-07-28`)
- `2024-Q1-WINTER` (`2024-01-22` to `2024-01-26`)
- `2024-Q3-SUMMER` (`2024-07-22` to `2024-07-26`)
- `2025-Q1-WINTER` (`2025-02-10` to `2025-02-14`)
- `2025-Q3-SUMMER` (`2025-07-21` to `2025-07-25`)

Across the **4 core liquid benchmark symbols**:
- `SpotCrude`
- `SpotBrent`
- `US500`
- `XAUUSD`

Execution sequence:
- Run `python3 ops/research/market_collection.py` (or `/owner/backtests` client) for each date/symbol.
- Run production Strategy 3 replay adapter (`cargo run -p jiaoyi-core --example s3_ticks_replay -- ...`) feeding 45-day M30 bars.
- Run Opening Range Breakout (`ORB-60m-VolExp`) and matched controls (`always_long`, `own_momentum`).
- Save the raw pilot results to `docs/three-year-pilot-results-2023-2025.json`.

---

## §6. Step 5: Generate and Commit the Report to `doc/`

§6.1. Generate the markdown report `doc/THREE_YEAR_PILOT_REPORT-18323-2026-11-07.md` summarizing:
- Total episodes collected (`120`) and verified (`120`).
- Admitted entries and closed scenario counts by symbol and 3-year seasonal cohort.
- Net win rate, `mean_net_r`, and profit factor evaluated under both 24 bp/side and empirical cTrader slippage (2.0 bp for index/gold, 3.74 bp for crude).
- Matched comparison against `always_long` and `own_momentum`.
- Append the entry to `log/CODEX_Log.txt`.

§6.2. Commit and push the report directly:
```bash
git checkout -b report/three-year-pilot-results-18323
git add doc/THREE_YEAR_PILOT_REPORT-18323-2026-11-07.md \
        docs/three-year-pilot-results-2023-2025.json \
        log/CODEX_Log.txt
git commit -m "Research: Stage 1 three-year seasonal pilot report across 2023–2025 (№18,323)"
git push -u origin report/three-year-pilot-results-18323
gh pr create --title "Research: Stage 1 three-year seasonal pilot report across 2023–2025" --body "№18,323 · Stage 1 three-year pilot report across 120 jobs"
gh pr checks --watch
gh pr merge --squash --delete-branch
```

---

## §7. Step 6: Output Block to Print for the Owner to Copy-Paste Back Here

§7.1. At the very end of your execution, print the following formatted JSON block directly into the chat/terminal so the owner can copy-paste it back into Gemini for quantitative audit:

```
=== COPY-PASTE FOR GEMINI REVIEW ===
{
  "trace": "№18,323",
  "plan_sha256": "<SHA256 of docs/three-year-seasonal-plan-2023-2025.json>",
  "pr_merge_commit": "<Git commit hash of merged PR>",
  "total_jobs_completed": 120,
  "cohort_breakdown": {
    "2023_Q1_Winter": { "SpotCrude": { "closed": 0, "wins": 0, "mean_r": 0.0 }, "SpotBrent": { ... }, "US500": { ... }, "XAUUSD": { ... } },
    "2023_Q3_Summer": { ... },
    "2024_Q1_Winter": { ... },
    "2024_Q3_Summer": { ... },
    "2025_Q1_Winter": { ... },
    "2025_Q3_Summer": { ... }
  },
  "overall_summary": {
    "production_s3": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 },
    "orb_range_expansion": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 },
    "macro_180m_momentum": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 },
    "always_long_control": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 }
  },
  "data_gap_count": 0,
  "report_file": "doc/THREE_YEAR_PILOT_REPORT-18323-2026-11-07.md"
}
====================================
```
