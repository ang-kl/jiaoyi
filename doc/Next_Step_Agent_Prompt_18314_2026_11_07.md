# Turnkey Autonomous Coding Agent Prompt — Multi-Year Seasonal Backtest Plan & Stage 1 Pilot Execution (`№18,315`)

№18,315 §§1–8 · 07-11-2026 11:10:00 SGT (+08:00) · Target Repository: `https://github.com/ang-kl/jiaoyi`

## §1. Master Execution Directive for Coding Agent

§1.1. You are operating as the autonomous engineer in `https://github.com/ang-kl/jiaoyi`. The owner instructed: `"Run attached next-step-agent-prompt-18314-2026-11-07.md"`. This file is completely self-contained—you do not need to look for external files. It embeds the exact Markdown text, JSON configuration, git PR workflow, Stage 1 collection/replay execution, report generation to `doc/`, and the final summary copy-paste block.

§1.2. **Mandatory Invariants**:
- Use serial `№18,315` (and next sequential `№18,316+`), `SGT (+08:00)` timestamps, and `§` paragraph numbering across commits, PRs, and reports.
- All 40 dates across the 8 seasonal cohorts are strictly between `2024-01-22` and `2025-10-10`, strictly before the `2025-10-24` `/owner/backtests` development seal.
- Do NOT open validation (`19 Aug – 5 Sep 2025`), alter live account permissions, modify the 17 production symbols, or change the `73%` net wins / `1:1.38` R:R account goal.
- Never remove collection receipts or retry failed jobs to manufacture a pass; retain all losses, cost-vetoed episodes, and data gaps in denominators.

---

## §2. Step 1: Write `docs/multi-year-seasonal-plan-2024-2025.json`

§2.1. Create `docs/multi-year-seasonal-plan-2024-2025.json` on branch `plan/multi-year-seasonal-backtest-2024-2025` with this exact content:

```json
{
  "trace": "№18,314 §§1–8",
  "by": "Gemini 3.1 Pro (Independent Reviewer)",
  "frozen_at_utc": "2026-11-07T02:55:00.000000+00:00",
  "scope": "Multi-year stratified seasonal backtest plan across 2024 and 2025; pre-holdout development only; no live orders or validation opening.",
  "account": 49236051,
  "development_seal_cutoff": "2025-10-24",
  "classes": {
    "equity_index": [
      "US500",
      "NAS100",
      "GER40"
    ],
    "energy": [
      "SpotCrude",
      "SpotBrent"
    ],
    "crypto": [
      "BTCUSD",
      "ETHUSD"
    ],
    "metal": [
      "XAUUSD"
    ]
  },
  "native_volume": {
    "US500": 10,
    "NAS100": 10,
    "GER40": 10,
    "SpotCrude": 100,
    "SpotBrent": 100,
    "BTCUSD": 1,
    "ETHUSD": 1,
    "XAUUSD": 100
  },
  "seasonal_cohorts": {
    "2024_Q1_Winter": [
      "2024-01-22",
      "2024-01-23",
      "2024-01-24",
      "2024-01-25",
      "2024-01-26"
    ],
    "2024_Q2_Spring": [
      "2024-05-13",
      "2024-05-14",
      "2024-05-15",
      "2024-05-16",
      "2024-05-17"
    ],
    "2024_Q3_Summer": [
      "2024-07-22",
      "2024-07-23",
      "2024-07-24",
      "2024-07-25",
      "2024-07-26"
    ],
    "2024_Q4_Autumn": [
      "2024-11-11",
      "2024-11-12",
      "2024-11-13",
      "2024-11-14",
      "2024-11-15"
    ],
    "2025_Q1_Winter": [
      "2025-02-10",
      "2025-02-11",
      "2025-02-12",
      "2025-02-13",
      "2025-02-14"
    ],
    "2025_Q2_Spring": [
      "2025-05-12",
      "2025-05-13",
      "2025-05-14",
      "2025-05-15",
      "2025-05-16"
    ],
    "2025_Q3_Summer": [
      "2025-07-21",
      "2025-07-22",
      "2025-07-23",
      "2025-07-24",
      "2025-07-25"
    ],
    "2025_Q4_PreHoldout": [
      "2025-10-06",
      "2025-10-07",
      "2025-10-08",
      "2025-10-09",
      "2025-10-10"
    ]
  },
  "stage_1_pilot_cohorts": [
    "2024_Q1_Winter",
    "2024_Q3_Summer",
    "2025_Q2_Spring",
    "2025_Q4_PreHoldout"
  ],
  "stage_1_pilot_symbols": [
    "SpotCrude",
    "SpotBrent",
    "US500",
    "BTCUSD"
  ],
  "models": [
    "production_s3_m30_sigma_band",
    "opening_range_breakout_30m",
    "opening_range_breakout_60m"
  ],
  "controls": [
    "always_long",
    "own_momentum_fixed_horizon"
  ],
  "cost_gates": {
    "primary_decision_slippage_bps_per_side": 24.0,
    "calibrated_benchmarks_bps_per_side": [
      2.0,
      3.74
    ]
  },
  "thresholds": {
    "min_closed_per_symbol": 10,
    "min_closed_per_class": 20,
    "min_weeks": 4,
    "min_win_rate": 0.5,
    "min_profit_factor": 1.0,
    "min_mean_net_r": 0.0
  }
}
```

---

## §3. Step 2: Write `docs/multi-year-seasonal-backtest-plan-2024-2025.md`

§3.1. Create `docs/multi-year-seasonal-backtest-plan-2024-2025.md` with this exact content:

```markdown
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
```

---

## §4. Step 3: Run Git Commands to Open PR & Merge to `main`

§4.1. Run the following shell sequence to create the PR, wait for CI, and squash-merge:

```bash
git checkout -b plan/multi-year-seasonal-backtest-2024-2025
git add docs/multi-year-seasonal-backtest-plan-2024-2025.md \
        docs/multi-year-seasonal-plan-2024-2025.json
git commit -m "Research & Docs: freeze multi-year seasonal backtest plan for 2024–2025 (№18,315)"
git push -u origin plan/multi-year-seasonal-backtest-2024-2025

gh pr create \
  --title "Research & Docs: freeze multi-year seasonal backtest plan for 2024–2025" \
  --body "## №18,315 §§1–4 · Summary
- **§1. Multi-Year Stratified Seasonal Architecture**: Replaces the 2-date summer doldrum trap with an 8-cohort seasonal plan across 2024 and 2025 (40 trading dates total, all strictly pre-24 Oct 2025).
- **§2. Liquid Core Universe**: Restricts testing to 7 liquid benchmark symbols (\`SpotCrude\`, \`SpotBrent\`, \`US500\`, \`NAS100\`, \`GER40\`, \`BTCUSD\`, \`ETHUSD\`, \`XAUUSD\`), eliminating illiquid altcoin quote gaps.
- **§3. Dual Model Tracks**: Tests production Strategy 3 v7 (\`s3_ticks.rs\` with 45-day M30 bars) side-by-side with Opening Range Breakout (with volatility expansion filters) and matched controls.
- **§4. Staged Execution**: Authorizes Stage 1 Pilot (4 seasonal cohorts, 4 symbols = 80 jobs) before full Stage 2 expansion."

gh pr checks --watch
gh pr merge --squash --delete-branch
```

---

## §5. Step 4: Execute the Stage 1 Pilot Collection & Replay (80 Jobs)

§5.1. Pull the updated `main` branch:
```bash
git checkout main && git pull origin main
```

§5.2. Execute the Stage 1 Pilot collection and replay for the **4 seasonal cohorts** (`2024_Q1_Winter`, `2024_Q3_Summer`, `2025_Q2_Spring`, `2025_Q4_PreHoldout` = 20 dates) across the **4 core symbols** (`SpotCrude`, `SpotBrent`, `US500`, `BTCUSD`):
- Run `python3 ops/research/market_collection.py` (or the existing `/owner/backtests` client) for each date/symbol.
- Run the production Strategy 3 replay adapter (`cargo run -p jiaoyi-core --example s3_ticks_replay -- ...`) and Opening Range Breakout evaluation.
- Save the complete results to `docs/seasonal-pilot-results-2024-2025.json`.

---

## §6. Step 5: Generate and Commit the Report to `doc/`

§6.1. Generate the markdown report `doc/SEASONAL_PILOT_REPORT-18316-2026-11-07.md` summarizing:
- Total episodes collected (`80`) and verified (`80`).
- Admitted entries and closed scenario counts by symbol and seasonal cohort.
- Net win rate, `mean_net_r`, and profit factor at $24\text{ bp/side}$ and $2\text{ bp/side}$.
- Matched comparison against `always_long` and `own_momentum`.
- Append the entry to `log/CODEX_Log.txt`.

§6.2. Commit and push the report directly:
```bash
git checkout -b report/seasonal-pilot-results-18316
git add doc/SEASONAL_PILOT_REPORT-18316-2026-11-07.md \
        docs/seasonal-pilot-results-2024-2025.json \
        log/CODEX_Log.txt
git commit -m "Research: Stage 1 multi-year seasonal pilot report across 2024–2025 (№18,316)"
git push -u origin report/seasonal-pilot-results-18316
gh pr create --title "Research: Stage 1 multi-year seasonal pilot report across 2024–2025" --body "№18,316 · Stage 1 seasonal pilot report"
gh pr checks --watch
gh pr merge --squash --delete-branch
```

---

## §7. Step 6: Output Block to Print for the Owner to Copy-Paste Back Here

§7.1. At the very end of your execution, print the following formatted JSON block directly into the chat/terminal so the owner can copy-paste it back into Gemini for quantitative review:

```
=== COPY-PASTE FOR GEMINI REVIEW ===
{
  "trace": "№18,316",
  "plan_sha256": "<SHA256 of docs/multi-year-seasonal-plan-2024-2025.json>",
  "pr_merge_commit": "<Git commit hash of merged PR>",
  "total_jobs_completed": 80,
  "cohort_breakdown": {
    "2024_Q1_Winter": { "SpotCrude": { "closed": 0, "wins": 0, "mean_r": 0.0 }, ... },
    "2024_Q3_Summer": { ... },
    "2025_Q2_Spring": { ... },
    "2025_Q4_PreHoldout": { ... }
  },
  "overall_summary": {
    "production_s3": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 },
    "orb_range_expansion": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 },
    "always_long_control": { "closed": 0, "wins": 0, "win_rate": 0.0, "mean_r": 0.0, "pf": 0.0 }
  },
  "data_gap_count": 0,
  "report_file": "doc/SEASONAL_PILOT_REPORT-18316-2026-11-07.md"
}
====================================
```
