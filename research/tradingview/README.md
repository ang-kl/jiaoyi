# BTC entry-confirmation research v3

Trace: №18,234 §§1–4 · 5 November 2026, 08:34 SGT · by <Codex (GPT-6), effort: not exposed>.

## Problem and hypothesis

The completed v2 comparison failed its economic screen. Baseline A: 52 trades,
9 wins, net -USD203.60, PF0.214. Reassessment B: 236 trades, 39 wins,
net -USD824.33, PF0.207. All 52 original entries and their exit times/net outcomes
matched; 184 additions lost about USD620.74 on rounded rows. B's price-only gross
was about +USD42.09 against USD866.47 of cost-proxy charges. These are simulator
results, not actual broker charges or full Rust execution parity.

V3 tests whether requiring two consecutive same-direction half-hour breakouts
reduces losing entries. This may delay entry, remove winners or worsen returns.
It is a hypothesis, not an established repair. The known 4 November rebound is
already inspected development evidence, not an untouched validation period.

## Change and comparator

Use `Jiaoyi_S3_BTC_Confirmation_v3.pine`. Reassessment defaults ON to reproduce B.
The new `Require two consecutive same-side breakouts` switch defaults OFF.

- OFF: reproduction of v2 Run B under identical data/settings.
- ON: candidate C. The current and immediately preceding completed half-hour
  marks must both break their respective bands on the same side and pass the
  spread-width check. Both must be within the test and entry windows.
- An inside-band, opposite-side, missing-history, wide-spread or nonadjacent mark
  breaks confirmation. A new UTC session resets it. The previous mark need not
  pass cost admission; cost C < R is still required at the actual entry mark.
- Cost assessment remains before the confirmation gate. Quantity, original R,
  Q=1.38, TP calculation, trailing, session flatten and one submitted entry per
  session remain unchanged. No reversals or second entries are added.
- Diagnostic code 10 means cost passes but confirmation is pending. The
  `Audit consecutive breakout confirmed` plot is 0/1 at valid calculation marks.
  Diagnostics use the bar closing at the mark; a chart timestamp can label the
  preceding minute. Code 8 means submission, not proof of a broker fill.

V2 is retained unchanged as the reference. Neither script connects to a broker.
They model whole-position TP1 and bar-close trailing, not the complete production
partial-exit, sizing, account-capacity or tick-admission paths.

## Bounded test for today

Use standard PEPPERSTONE:BTCUSD one-minute candles, chart/export timezone SGT.
Freeze identical chart history and settings for both runs:

| Setting | Value |
|---|---|
| Capital / quantity | USD10,000 / 0.01 |
| Spread / commission reserve | 15 USD/BTC / 0 |
| Slippage reserve | 24 bp per side |
| Properties commission / slippage | 0.25% per side / 0 ticks |
| Trailing / reassessment | ON / ON |
| Script execution / order delay | Bar close only / one tick |
| Additional tick or order-fill recalculation | OFF |
| Pyramiding | 0 |
| Bar detail | High, unchanged between runs |
| Entry dates | 1 Jan 2026 00:00 UTC to 5 Nov 2026 00:00 UTC |

The date endpoints display as 08:00 in SGT. Preserve the exact loaded history
from B; warm-up needs 14 valid same-mark observations within 30 prior sessions.
An input date does not itself load history.

First check OFF reproduces the uploaded B (236 trades, -USD824.33) when coverage
is identical. Then enable only confirmation, export C's trade list and overview.
Retain settings and diagnostic chart values if available. Do not also lower
costs, disable trailing or alter the one-entry rule in this comparison.

## Acceptance fixed before C runs

1. Baseline parity and one-entry/session checks must pass. Diagnose discrepancies
   before treating C as evidence.
2. Demonstrate a pending-confirmation mark and a later confirmed mark from the
   diagnostic plots. Compilation and emulator execution must succeed.
3. To advance, C must have positive net P&L, PF >1, higher net P&L than B and no
   worse TradingView maximum equity drawdown at identical quantity/costs.
   Less-negative results are informative but do not pass this screen.
4. Fewer than 20 closed candidate trades spanning at least three UTC calendar
   weeks is inconclusive. This is a modest exploratory screen, not statistical
   significance. A passing candidate still needs a separately frozen untouched
   or prospective/shadow validation. No promotion to live trading follows from
   this historical test alone.

The project target remains risk:reward 1:1.38 and >=73% net wins over the latest
20 qualified closes per strategy/account (15/20 =75%). Simulator windows do not
qualify as broker-account goal windows. No financial progress is claimed.

## Focused validation receipt

- Reviewed diff: one confirmation condition gates order submission; exits are
  byte-identical to v2, and cost/entry-limit expressions are retained.
- Checked v3 source resets confirmation on session boundaries and missing
  history, requires exact 30-minute adjacency, and uses only completed marks.
- Existing CSV comparator reconciles uploaded B to 236 trades, 39 wins,
  -USD824.33 and 52 unchanged baseline entries. This validates comparison inputs,
  not execution of v3.
- Pine compilation, v3 OFF parity and candidate performance remain PENDING
  TradingView execution. No Pine execution connector is available in this session.

No runtime Rust source, registration, account rule, order, Railway setting or
worker policy is changed. Research and log paths are outside the trading
service's configured source watch patterns.
