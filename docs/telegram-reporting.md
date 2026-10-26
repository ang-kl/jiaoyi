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
