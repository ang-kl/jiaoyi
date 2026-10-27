# Owner Telegram HTML reports

Both reporters use Telegram `sendDocument` with a real `.html` attachment, not a
message pretending to be an HTML file. Credentials remain service variables:
`TELEGRAM_BOT_TOKEN` and `TELEGRAM_OWNER_CHAT_ID`. No token or recipient value is
committed. They do not configure a webhook, poll bot updates, trade, or call an
LLM to write a report.

## Engineering service

The existing `jiaoyi-engineering` coordinator renders its durable job/event
records and observed `origin/main` commits. Candidates, reported tests, CI,
merges and verified deployments are separate evidence stages. Failed proposals
remain visible; model reports are saved before a later handoff failure can
replace the job result. Retained-workspace diagnostics check the exact Git
pointer, scope gate, format log and local commit stage. A single controlled
`git push --dry-run` may classify remote authentication failure without updating
any remote ref. These diagnostics do not rerun model work.

With Telegram configured, the deterministic digest runs every six hours by
default. `ENGINEERING_TELEGRAM_INTERVAL_SECONDS` optionally changes that cadence
(300–86400 seconds). The first scheduled digest is six hours after installation;
the requested initial six-hour audit is delivered separately. Delivery status
and the reporter source digest appear under `/status.telegram_reporting`.

SQLite delivery state and HTML files stay under `/data/private`. Success is
recorded only after a positive Telegram message receipt. Definitive rejections
retry after15minutes; unknown POST outcomes/interrupted sends are held rather
than blindly replayed. The independent thread does not block model work.

The owner changed the daily top-level attempt allowance from4 to18. Preserve18.
This permits4.5times as many attempts, not a predictable token multiplier or a
money ceiling; child agents, reasoning and retries also consume shared Codex
allowance. ChatGPT/Codex plan authentication remains selected. Failed attempts
still count. Reporting itself makes zero LLM calls; Railway costs are separate.

## Trading service

Set `TELEGRAM_TRADING_REPORT_SCHEDULE=ny_close` only on `jiaoyi`. The owner chose
New York17:00 accounting cutoff and17:15delivery:05:15SGTduringUS daylight
saving and06:15SGTduringstandardtime. Windows follow the actual New York timezone
and can span23or25hours atDST transitions. SGT is the primary display timezone.
The first enabled startup catches up the latest eligible closed period once.

Reports include owned daily closing-deal cash net by account/strategy, fully
closed-position counts, known wins, explicit unpriced costs/history coverage,
latest20 goal assessments and the latest reconciled exposure. Closing commission
is both sides on the proven broker convention; opening commission is not
subtracted twice. Partial-close cash flow is distinct from a fully closed
position outcome. Account currencies are never added together. Missing fees,
digits or source coverage are not silently treated as zero. Exposure receipts
can be stale and do not reconstruct missing broker stop prices.

A dedicated asynchronous task and PostgreSQL `telegram_reports` outbox isolate
reporting from trading/position exits. Confirmed receipts deduplicate by NY
period across restarts. Definitive rejection has bounded retries; ambiguous
outcomes remain held for checking. No entry, pending order, stop/target, sizing,
activation or owner-pause rule is changed.

## Proven bot-trade reference

Read-only source: `ang-kl/bot-trade/agent/services/telegram.js` (`sendDocument`)
and `agent/services/journal.js` (deterministic daily HTML). Its UTC-day journal
is not a market-close schedule. Its stamp-before-send behavior was not copied:
failed delivery must not be recorded as success. No bot-trade code was modified.

## Traceability and allowance invariants

Added by the owner27October2026, alongside all existing invariants: preserve the
agreed goals, serial/paragraph numbering and SGT date/time. Chat keeps its current
continuous sequence. Engineering HTMLs use a stable E-period reference; trading
HTMLs use a stable T-NY-date reference, with numbered paragraphs and SGT generation
and source cutoffs. Retry does not invent a different report period.

The18 allowance is not a quota. Unresolved retained proposals hold fresh model
calls; they do not pause trading or stop reporting/CI polling. Unchanged automatic
assessment evidence spends no new reservation. Learning must cite an actual
loss/benchmark gap and proven applicable source, rather than deploying every
framework in a domain list. The owner-reported GitHub token expiry is25November
2026 (exact instant unspecified); GH_TOKEN_EXPIRES_ON optionally carries that
nonsecret date into future engineering digests.

## Lean reporting delivery and protected follow-up — 27 October 2026

Document ENG-20261027-LEAN; recorded 27 October 2026, 07:31 SGT. The owner's
queued contract is active after its 07:20 SGT activation deadline. This document
does not introduce a chat serial. The reference HTML attachment was not available
in this worker; no claim is made to have inspected it or unseen chats.

[§ENG-20261027-LEAN.1] **Flow assessed.** Existing broker history and original
risk feed the goal assessment, which persists through the supervisor and feeds
the deterministic NY-close HTML/outbox. The separate engineering coordinator
uses its durable job/CI/deployment receipts for its six-hour digest. Neither
reporter adds LLM calls. Read-only `/health` and `/operations/reports` returned
200 at 07:28:33 SGT; operations source cutoff was 07:28:33.547875 SGT. Eight
monitors were alive. This establishes endpoint availability, not current
delivery receipts, deployment of this candidate or economic acceptance.

[§ENG-20261027-LEAN.2] **Trading HTML candidate.** Put latest-20 benchmark gaps
before daily cash accounting. For complete, current assessment evidence, show
the win-rate difference from 73%, the fixed-cohort shortfall from 15/20, and
the realized payoff difference from 1.38. Ratio display is rounded explicitly;
payoff is not planned SL/TP geometry. Incomplete samples show their count deficit
and unknown goal gaps; missing/stale evidence and undefined payoff remain
unknown with a reason. Replace raw reconciliation JSON with per-strategy owned,
unresolved and external counts, retaining SGT receipt/source times and unknown
missing fields. Existing T-period serials and numbered paragraphs remain.
No new data, schema, framework, retention policy or delivery route is needed.

[§ENG-20261027-LEAN.3] **Observed benchmark, not a defect.** At the source cutoff,
all three demo S3 windows had 6/20 = 30% net wins: 43 percentage points below 73%,
and nine net wins below the discrete 15/20 requirement for those fixed cohorts.
Their realized payoff observations were 2.5747365453583857 (46130058),
1.2035371684240823 (47790949), and 2.634581208474028 (49236051). Against 1.38,
the rounded differences are respectively +1.194737, −0.176463 and +1.254581.
Live S3 had 1/12 wins, eight measured closes short of 20; every S1 sample had
zero, 20 short. These insufficient samples cannot establish the latest-20 goal.
No registered loss is reclassified as a software bug. No trading rule is tuned.
The objective remains ≥73% net wins and risk:reward 1:1.38 separately for each
strategy/account. One passing overlapping window does not prove consistency.

[§ENG-20261027-LEAN.4] **Comparable deltas deferred.** The report currently
receives the latest assessments; no prior comparable baseline has been
validated/selected. It explicitly reports prior delta unknown. Do not assume
adjacent supervisor events are comparable: they can reflect heartbeat, missing
evidence, corrected receipts or overlapping cohorts. A future bounded comparison
must identify both source times, account/strategy, cohort identities and common
cost/risk definitions, and explain overlap and revisions. The latest 100 events
are a bounded view, not proof that an earlier cohort does not exist. No overall
“% closer” is supported. Financial movement attributable to this engineering
change is not measurable; observed market outcomes are not causal evidence.

[§ENG-20261027-LEAN.5] **API policy recorded; enforcement not verified.** The
owner's API ceiling is **$40 TOTAL for the project until explicitly changed**,
with no daily/session reset. It does not authorize paid inference, an API-route
switch or spending to the ceiling. ChatGPT/Codex-plan authentication remains
selected. Attempts, child agents and shared-plan usage are separate from API
billing; hosting/Actions costs are separate too. Actual cumulative project API
spend, remaining balance and provider hard-cap controls were not inspected and
remain unknown, not $0 or $40 remaining. No billing or hard-cap enforcement is
claimed. Deterministic rendering/delivery makes zero LLM calls.

[§ENG-20261027-LEAN.6] **Protected runtime proposal for the parent coordinator.**
The engineering renderer/collector and cleanup implementation are under
`ops/engineering/`, outside this runner's editable scope. No such changes are
delivered here. A separate reviewed task should:

- In `reporting.py::render/collect`, prioritize delivered/deferred/blocked/
  not-verifiable outcomes, benchmark evidence and owner actions. Replace repeated
  test/diagnostic dumps and the unconditional generic flow diagram with concise
  validation counts and references to canonical receipts. Include a diagram only
  for an evidenced material bottleneck. Add the total API-policy statement from
  §5 and explicit unknown financial deltas when no valid baseline is available.
- Before changing cleanup in `Reporter.tick` (currently keeps 40 HTML files) or
  `Coordinator.serve` (50 logs), measure artifact bytes/count/age and growth on the
  existing `jiaoyi-engineering` volume, including queue metadata and retained
  work references. Read metadata only; do not read authentication content. Report
  actual filesystem/DB sizes with SGT cutoffs. Those measurements are unavailable
  in this task, not zero. Never extrapolate trading-DB history sizes to this volume.
- Classify active authentication, queued/running/incomplete jobs, retained
  proposals, ambiguous delivery files and canonical red/green/CI/deployment
  receipts as protected. Only propose removal of proven redundant/expired
  artifacts after evidence is retained. Use an exact dry-run manifest and verify
  recovery and receipt links before/after any approved deletion. No blanket
  cleanup, duplicated market history, new telemetry schema or new platform.

[§ENG-20261027-LEAN.7] **Next action and gates.** Parent reviews this candidate;
unchanged exact-head `check`, `docker` and `engineering` CI must pass before its
authorized release. Then verify the exact deployment and a generated HTML with
known gaps/unknowns and existing outbox receipt behavior. Local tests are not CI
or delivery receipts. Protected runtime work is deferred to separate review;
comparable financial movement and economic acceptance remain not verifiable.
Owner-only pause/risk decisions, live limits, activation, sizing, stops/targets,
broker ownership, the one order gate, existing exits and paused research remain.


## Approved four-section account report (27 October 2026)

The owner approved `daily-trading-report-proposal.md`. The new report separates
whole-account accounting (all broker deals, including external/manual activity)
from program-owned strategy attribution and the invariant latest-20 goal.
Live/demo and currency totals remain separate; no guessed dollar labels or FX.
Balance plus broker net unrealized P&L is explicitly derived equity, with
unverified credit treatment and future closing commissions identified separately.

The existing read-only history session captures account marks in a separately
polled future; failures do not terminate history or alter execution. Only the
latest mark and the first mark within 90 seconds after each NY17:00 are kept in
`report_account_snapshots`; actual source times/delay are retained. This is a
near-close observation, not an invented exact historical mark. Cash-flow reads
cover the actual adjacent mark interval; missing baselines or unverified flows
withhold returns. MTD/YTD need a complete valid linked return chain. Simple
percentages are withheld when gross external flows occurred, even if net flows
cancel. Existing records supply deal cash/closed positions and execution.

Open/pending stop-price projections exclude unquoted future exit charges and
gaps; they are distinct from complete-cost risk. Unknown costs/protection never
become zero. Margin aggregation is numeric only when the broker's calculation
mode and per-position precision support it. No new risk threshold is introduced.

Four concise HTML sections replace raw exposure JSON and repeated empty execution
rows. Supported benchmark gaps/deltas remain prominent, with no fabricated
combined progress percentage. A delivered legacy layout can be edited in place
once current marks exist, with a bounded attempt count and the same Telegram
message/period. Confirmed revised HTML is retained only after the returned message
ID matches. No duplicate daily document, new sender, paid feed or LLM call.
