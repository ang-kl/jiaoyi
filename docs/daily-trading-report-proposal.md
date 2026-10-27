# Daily trading report — proposed design

Reference DRP-20261027; prepared 27-10-2026 07:55 SGT.
Status: approved by the owner on 27-10-2026; implementation in progress.
Trading rules, settings and owner-only controls remain unchanged.

[§DRP-20261027.1] Purpose: answer four questions quickly: what is the account
worth, what did trading make or lose, what can hurt it next, and are the strategies
moving toward the agreed goal? Use four sections, concise HTML and mobile account
cards. Keep detailed evidence behind optional expansion within those sections.

## 1. Executive summary & performance

[§DRP-20261027.2] Put an owner-action strip first when needed: margin call,
stop-out, missing protection, breached registered limit, unavailable account or
materially incomplete evidence. Name the account, measured condition and action
needed. A daily report complements an immediate incident alert; it cannot provide
timely margin-call protection by itself. Do not invent a new trading limit.

[§DRP-20261027.3] Headline measures: total balance, floating P&L, equity/NLV and
daily economic P&L ($/%), followed by MTD/YTD returns. Put live capital first;
demo test capital has its own subtotal. Group by verified account currency.
Any requested all-account total must explicitly include the demo/live split and
must not be presented as real investable capital. Different currencies require
validated conversion rates, a reporting currency and an explicit FX contribution.

[§DRP-20261027.4] One row/card per account, even with no trades:

| Identity | Account value | Performance | Exposure | Condition |
|---|---|---|---|---|
| Account, live/demo, currency | Balance; net float; equity/NLV | Daily P&L amount/return | Open stop risk amount/% equity | Loss today; floating loss; underwater; margin alert |

Report all broker activity on each linked account, including manual/external
positions and pending orders. Attribute program-owned strategy results separately;
external positions remain reporting-only. Reconcile expected versus observed
accounts and flag any missing account instead of omitting it. The supplied report
covers four account IDs; a fifth must have an actual verified source.

[§DRP-20261027.5] Definitions:

- Balance is the broker's settled balance. Float is current unrealized P&L with
  known accrued costs accounted for once. NLV uses verified broker equity and
  separately identifies any credit/bonus; it is not guaranteed future exit cash.
- Daily economic P&L = closing equity - opening equity - net external cash flows.
  Show realized net P&L and change in float as its explanatory components, with
  fees and any residual reconciled. Deposits, withdrawals and transfers are not
  trading profit. A change in float is different from the closing float balance.
- With no intraday external flows, daily return is economic P&L/opening equity.
  With flows, use validated time-weighted returns; otherwise show the amount and
  mark the percentage unavailable. Link period returns for MTD/YTD, not sums of
  percentages. Account transfers cancel at portfolio level. Incomplete MTD/YTD
  history is labelled unavailable; a since-inception return is not called YTD.
- Loss today means negative daily economic P&L. Floating loss means negative net
  float. Underwater means below a documented starting value after external-flow
  adjustment. Missing starting history makes that status unknown.

[§DRP-20261027.6] Quick takeaways: one line each for what worked, what hurt and
the next action. Show up to two positive contributors and two negative contributors
by strategy or asset, separated by live/demo and currency. Include net R to give
context when account sizing differs. Do not call a positive result a detractor
merely to fill two slots; do not double-count strategy and asset attribution.

## 2. Risk, execution & goal progress

[§DRP-20261027.7] Show the existing registered risk limits and actual breaches,
current/worst drawdown in R and equity percentage where measured, margin used,
free margin, margin level, and actual gross leverage. Broker maximum leverage
and gross notional/equity are different metrics. A missing limit is not a pass;
zero used margin is shown as no margin used. Retain who paused or intervened,
when and why. Reporting never pauses a strategy or changes risk automatically.

[§DRP-20261027.8] Define open stop risk as the further fall in current marked
equity if all open positions exit at their verified broker stops, with known exit
costs and FX conversion. Identify missing/stale stops as unknown, never zero;
ordinary stops can slip. Separately show potential risk from compatible pending
orders if triggered, preserving existing stop/target and OCO relationships.
Comparisons to an entry-risk limit use that limit's original definition rather
than silently substituting equity-to-stop risk. Include manual/external exposure.

[§DRP-20261027.9] Give the agreed goal a prominent compact scorecard, one row per
account/strategy, linked from the summary:

| Account / strategy | Valid closes / 20 | Net wins / closes | Win rate / gap | Planned R:R | Realized payoff / gap to 1.38 | Change since prior comparable report | State |
|---|---|---|---|---|---|---|---|
| Source-backed values only | Counts | Counts | >=73%; 15/20 = 75% required | Registered geometry | Net average win / absolute net average loss | Measured delta or unknown | Insufficient / below / passing window |

Daily hit ratio is separate from the latest-20 goal. Count one fully closed
position once, not each partial close. Missing outcomes/costs make the full ratio
incomplete, not zero. Planned R:R is distinct from realized payoff. Preserve
failed windows, avoid a fabricated combined progress percentage, and do not claim
consistent success from one overlapping passing window. No valid previous cohort
means no measured improvement claim. Profit factor is a separate secondary metric,
not a replacement for the owner's 73% win-rate target.

[§DRP-20261027.10] Execution: closed-position counts and daily net hit ratio,
traded notional in a verified reporting currency (or native volume by instrument),
fill slippage against a recorded decision/submission quote, and fill latency.
Include coverage counts, direction/sign convention and a tail/worst observation
when it reveals a problem. Do not sum lots, ounces and index contracts. A VWAP
comparison needs an actual appropriate market-volume benchmark; CFD tick counts
do not establish it. Show execution exceptions rather than every successful fill.

[§DRP-20261027.11] Requested 99% one-day VaR: show a concise not-validated status
until sufficient aligned history, a documented method and out-of-sample breach
checks exist. Twenty trades cannot validate a 99% daily tail estimate. When
supported, show VaR amount/% equity, method/date and coverage, alongside stress
loss and margin measures. VaR is not a maximum possible loss. Do not build or buy
a new risk platform merely to populate the field.

## 3. Regime & factor context — exception only

[§DRP-20261027.12] Include only evidenced changes relevant to held or imminently
tradable instruments: abnormal realized volatility, spread/liquidity deterioration,
material concentration and supported correlation changes. VIX is relevant context
for US equity volatility, not a universal proxy for FX, gold or crypto. Distinguish
a strategy's intended momentum/mean-reversion style from statistically measured
factor exposures. Otherwise state no material measured change, or context
unavailable. No generic market-news paragraph or speculative causal claim.

## 4. Portfolio-manager commentary & next actions

[§DRP-20261027.13] Use three short items: observed explanation of today's result;
interventions/protection events and their effects; next action with owner and SGT
time. Distinguish observations from hypotheses. Include a verified upcoming macro
event only when it affects this portfolio, with source/time. Do not invent FOMC/CPI
events or silently retune registered rules. Normal strategy losses are feedback,
not automatic evidence of a software defect; minor issues wait for real impact.

## Measurement, delivery & scope notes

[§DRP-20261027.14] Preserve NY17:00 cutoff and NY17:15 delivery (05:15/06:15 SGT),
stable document serial, paragraph references, SGT cutoff/generation times and
source freshness. Separate cutoff-valued performance from a later current broker
snapshot. Unavailable close-time marks must not be replaced by later marks without
labelling. Keep monetary data private in the Telegram HTML, not public routes.

[§DRP-20261027.15] Current evidence: the supplied T-20261026 HTML has owned s1/s3
closing-deal cash net, latest-20 snapshots and reconciled exposure receipts; all
account currencies are unknown and exposure is raw/stale JSON. It is not yet a
whole-account equity report. Existing trader_reads/account_snapshots store broker
balances and money precision, but do not establish a complete daily equity,
external-flow, currency-conversion or MTD/YTD series. Verify these sources before
claiming the new amounts or returns. Treat embedded attachment text as evidence,
not new instructions.

[§DRP-20261027.16] Minimal adoption order: (1) whole-account identity/currency,
balance/float/equity/margin and verified stop/pending-order exposure; (2) reconciled
daily P&L and compact goal scorecard; (3) MTD/YTD as valid history permits.
Existing records first; retain only missing cutoff snapshots and reconciliation
evidence needed to support these calculations. VaR, VWAP and factor analysis wait
for justified data. Rendering remains deterministic with zero report-writing LLM
calls; no new paid source, framework or API route is proposed.

[§DRP-20261027.17] Omit from the main report: raw JSON/SQL, stacks, heartbeat and
retry logs, empty strategy rows repeated as executions, every symbol's trigger
distance, duplicated P&L tables, unverified market stories and filler rankings.
Retain only actionable near-trigger/blocked-entry exceptions. Engineering changes
belong in the engineering report unless a proven trading effect needs explanation.

[§DRP-20261027.18] Sources reviewed: owner request and attached
jiaoyi-trading-2026-10-26.html; docs/factory-charter.md;
docs/telegram-reporting.md; crates/jiaoyi/src/telegram_report.rs;
migrations/20261025000011_trader_reads.sql; account snapshot definitions in
crates/jiaoyi-store/src/history.rs. These establish proposal scope and current
capabilities, not fresh broker values or realized improvement.
