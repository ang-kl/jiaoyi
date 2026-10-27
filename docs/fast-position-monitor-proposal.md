# Independent fast position monitoring — implementation proposal

**Document serial:** ENG-20261027-FAST-MONITOR

**Prepared:** 27-10-2026, 21:34:53 SGT (Asia/Singapore)

**Conversation reference:** № 17,535; owner request: “craft a .MD file about what you want to do”

**Status:** Original proposal retained below; implementation and validation record added in §FPM.17–20. Production activation requires the recorded release read-back.

**Evidence cutoff:** Local source inspection at 21:34:53 SGT; production was not queried for this proposal.

**Jiaoyi baseline:** `0b2a41c17f66640bb38d0b0111149d35aaad53cd`.

## 1. Outcome and project boundary

[§FPM.1] Add an independent, deterministic position-monitoring backstop inside the existing Rust `jiaoyi` service. Its purpose is to detect delayed protection processing promptly and keep already-authorized position management responsive when historical data or analytical work is slow. It must work without an AI session, an open chat or the owner's MacBook. This document specifies the work; it does not queue an unattended job or change trading rules.

[§FPM.2] Preserve the financial goal: **risk:reward 1:1.38 and at least 73% net wins over the latest 20 fully closed owned positions, separately for every strategy/account**. Fifteen wins out of twenty is 75%; fewer than twenty is insufficient evidence. Preserve registered acceptance criteria and distinguish planned stop/target geometry from realized net payoff. Faster monitoring is an engineering improvement to validate, not evidence that the financial goal has been reached.

## 2. What exists and what is missing

[§FPM.3] Source inspection shows that S1 and S3 already process incoming quotes and broker execution events; accepted broker-side SL/TP remain with cTrader. S3 also has a one-second session clock. However, historical reads, reconciliation and quote handlers are awaited within their respective strategy loops. A timer added to that same loop would not establish independence while a handler is awaiting slow work. This is an architectural exposure; this review has not reproduced a current production protection delay or established its cause.

[§FPM.4] Reuse the isolation and overlap controls in bot-trade's [pinned fast monitor](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/fast-monitor.js). It separates a default three-second ticker from a default sixty-second protection band. Its source records the lesson that slow protection-band calls previously skipped fast ticks. Normal position reviews are volume-adaptive, ordinarily one to three minutes; spikes can receive faster reviews. These are source defaults and recorded lessons, not verified current bot-trade runtime settings or a three-second execution guarantee. Do not import its different stop, loss-cap, profit-taking or time-exit policies.

## 3. Smallest useful design

[§FPM.5] Use the existing trading service and broker subscriptions. Start with a **three-second independent watchdog** and reuse the existing approximately sixty-second broker-book sampling where its coverage and freshness are sufficient. The watchdog reads bounded in-memory snapshots: quote receipt time, strategy processing/completion time, broker-book observation time and unresolved execution status. A heartbeat must distinguish scheduler activity from a completed position check. No broker reconciliation every three seconds, new Railway service, agent framework or inference loop is required.

```text
Broker quotes and executions
            |
            v
Bounded current snapshots ----> Independent 3-second watchdog
            |                         |
            v                         v
Registered exit evaluation     Fresh / overdue / unknown
            |                         |
            v                         v
One serialized execution path  Existing health + compact evidence
            |
            v
Existing cTrader gate ----> Broker protection / execution confirmation

Historical and analytical work runs outside the protection-critical path.
```

[§FPM.6] First deliver a read-only watchdog and a controlled starvation test. If the test demonstrates that slow non-protection work delays registered management, make the smallest separation required. The eventual timer and quote paths must share an **exit-only evaluator** and one authoritative execution/state owner. Do not call an entry-capable quote handler from the watchdog. Do not add another writer beside a blocked strategy actor or hold a common state lock across slow historical requests. An independently ticking observer alone is not completion of the responsive-management outcome.

[§FPM.7] Preserve safe execution under overlap, reconnect and restart. Serialize actions for the same account/position, retain existing intent deduplication, and route every amend or close through `jiaoyi_ctrader::trading::send`. Use bounded queues and single-flight reads. A timed-out request is not assumed cancelled: keep its unresolved state until execution or fresh broker reconciliation resolves it. Duplicate quote, timer and broker events must not cause a repeated partial close or overwrite newer protection.

## 4. Trading safeguards

[§FPM.8] The [factory charter](factory-charter.md), [S1 rulebook](phase-0-rulebook.md) and [S3 registration](preregistration-2026-10-25-strategy3.md) remain authoritative:

- Manage only positions with established program ownership. Account presence or an API label alone is insufficient.
- Inventory existing pre-orders, SL and TP before any management change. Report external/manual or ambiguous orders; do not adopt, cancel or amend them.
- Preserve S1's registered trailing method and amendment pacing. Compare against fresh broker protection, never loosen a stop, and preserve the current TP when an amend replaces protection.
- Preserve S3's original stop, registered TP1/TP2 and session exit. **No S3 trailing or stop amendments.**
- Use the registered executable quote side: bid to exit a long, ask to exit a short. Missing, stale or inconsistent observations remain unknown; do not manufacture a price or freshness certificate.
- Preserve sizing, account scope, limits, entry criteria and owner-only pause decisions. A monitor fault or engineering shutdown does not create a trading pause.
- A broker-triggered SL/TP prompts internal evidence collection. It does not automatically authorize a new stop rule, setup change or AI call.

## 5. Validation that must expose failure

[§FPM.9] Use targeted behavioral tests, including the current fake-broker facilities where applicable. A demonstrated defect needs a failing assertion before its repair. Validate these material risks rather than adding tests that merely mirror the implementation:

| Scenario | Required result |
|---|---|
| Historical/analytical future is deliberately held pending | Watchdog completion remains observable; after any management-path repair, a registered exit can also progress without waiting for that future. |
| A slow broker read overlaps several timer firings | At most one request remains in flight; no socket/request pile-up or false completion heartbeat. |
| Quote and timer request the same partial close | One authorized close intent/frame, with execution/reconcile confirmation before retry. |
| Reconnect, late fill or restart during an unresolved action | Reconcile actual broker state before another action; retain existing intent identity and ownership checks. |
| Missing quote, old book, crossed market or external position | Explicit unknown/refusal for the affected action; no fabricated protection or unauthorized management. |
| Existing tighter SL or TP is present | No stop loosening or TP deletion; S3 produces no stop amend. |
| Timer, actor or read task stalls | Health distinguishes the stalled component from a living HTTP server and records the last actual completion. |

## 6. Milestones and release

[§FPM.10] Keep one bounded assignment at a time. At the first thirty-minute checkpoint, report evidence and the smallest remaining outcome; narrow the work rather than repeatedly reassessing or building a general framework. This is a work-budget checkpoint, not a promised deployment time.

| Milestone | Deliverable and exit evidence |
|---|---|
| M0 — Establish the baseline | Pin deployed revision; confirm actual owned/pending positions and existing protection; identify the specific awaited work and measure or reproduce its effect. Source inspection alone does not prove a live defect. |
| M1 — Independent observation | Read-only watchdog, bounded snapshots, accurate last-completion health, and tests proving independence and no overlapping read accumulation. |
| M2 — Minimal management repair, if demonstrated | Separate only the proven blocking work; retain one exit evaluator and execution owner. Pass starvation, duplicate-action and broker-protection checks. If no defect is demonstrated, log the hypothesis and defer speculative execution changes. |
| M3 — Review and required CI | Journal actual checks and limits. Create one focused implementation PR only when there is a justified change. Required CI must pass on the exact unchanged head; standing owner auto-merge authorization applies. |
| M4 — Railway read-back | Verify deployed commit, startup/reconnect and completed monitoring on all configured live/demo accounts. Initial observation is read-only on all accounts; it does not introduce a demo-only trading policy. Enable only behavior already authorized by registered rules, with the same account scope. |
| M5 — Close the engineering scope | Record measured monitoring and confirmation delays, unresolved cases and rollback receipt. Distinguish watchdog delivery from any management repair and from financial acceptance. Do not force trades to obtain a receipt. |

[§FPM.11] If the release causes a regression, roll back the engineering revision through the existing deployment path, preserving broker-held protection and registered exits. Do not pause strategies, cancel pre-orders, duplicate a running executor or restart blindly during an unresolved order. Verify the restored deployed revision and reconcile any ambiguous action. A required strategy/risk-policy change remains an owner decision, not an engineering shortcut.

## 7. Evidence, reporting and cost

[§FPM.12] Keep only current bounded state plus material transitions and action confirmations in existing storage. The compact health evidence should show last started/completed times, oldest relevant observation age, overrun/skipped count and unresolved actions. Measure local processing delays with a monotonic clock; identify broker timestamps separately. A scheduler interval is not a measured end-to-end latency. Persist no per-tick history, duplicate broker archive or new telemetry schema merely for this monitor. Public routes continue to exclude account money and secrets.

[§FPM.13] Use existing engineering digests and the daily trading report; **no individual stop-loss Telegram messages**. Summarize what was changed, what failed/passed, release identity and what still requires attention. For goal tracking, retain the per-strategy/account latest-20 scorecard, showing wins out of twenty, the gap to fifteen wins for a complete window, and the separately defined reward/payoff evidence. Show a run-to-run delta only for valid comparable measurements. This proposal produces **no new trading evidence, so financial progress is not measured**; do not report an invented percentage improvement or zero gap.

[§FPM.14] Runtime monitoring uses **zero LLM/API calls** and consumes no engineering AI session by itself. Engineering implementation and review still use the existing plan allowance; hosting and broker traffic are not claimed free. Preserve the allowance as a ceiling, the $40 total paid-API budget and no automatic paid fallback. Add no new paid service or dependency without a demonstrated need. “Remember that Time is Money” and “It's the will to prepare to win that matters” mean useful evidence and a bounded release, not activity for its own sake.

## 8. Completion contract

[§FPM.15] **Expected result:** independent monitoring with truthful freshness/completion evidence; any demonstrated protection-path blockage minimally repaired without changing registered trading policy. **Close as done** only for outcomes with tests and deployed read-back. Label untested runtime behavior or unobserved action confirmation **Not Verifiable**, and retain it as open. Stop work on the owner's instruction. Do not describe the trading goal, the full handover or continuous profitability as complete because this engineering task passes.

## 9. Inspectable sources

[§FPM.16] Relevant evidence and constraints:

- [Bot-trade fast monitor at the pinned revision](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/fast-monitor.js): `cadenceMs`, `startFastMonitor`, separate tick/band guards and completed-pass receipts. Its incident comments are recorded historical evidence, not a current jiaoyi incident reproduction.
- Jiaoyi [S1 loop](../crates/jiaoyi/src/trading.rs) and [S3 loop](../crates/jiaoyi/src/strategy3.rs): quote, broker-event and timed branches await their handlers; baseline pinned in the header.
- [Existing report snapshot collector](../crates/jiaoyi/src/trading_report_snapshot.rs) and [history scheduling](../crates/jiaoyi/src/history.rs): reuse suitable observations rather than introducing another polling service.
- [Verified trailing mechanisms and remaining evidence gaps](verified-trading-mechanisms-2026-10-27.md), [statement-loss lessons](lessons-2026-10-27-statements.md), [lean owner contract](queued-lean-owner-contract.md) and [engineering-worker contract](engineering-worker.md).
- [Dated journal](../journal/2026-10-27.md): actual work, validation and release receipts remain distinct from this proposal.

## 10. Implementation and evidence record

[§FPM.17] Owner authorization: chat №17,577 §1 (28-10-2026 05:46 SGT), implement/merge this plan and resume engineering; №17,581 §1 (05:59 SGT), include chat references in every PR. Prepared 28-10-2026 06:12:45 SGT by <Codex (GPT-6), effort: not exposed by this session>. Current implementation incorporates main through #54, including #49/#50's current Strategy 3 v3 registration, #51/#52's watches/logging and #53's explicit stop-cause diagnostic. Existing positions continue to use their frozen opening plans; the original proposal's v2 source is historical, not a replacement for v3.

[§FPM.18] Demonstrated two specific historical-data blockages with scripted broker replies held for ten seconds: a periodic S1 daily-bar refresh prevented an existing trail, and S3 preparation for a flat ETH symbol prevented an owned BTC position's TP1. The failing fixtures produced no required protective action; the repaired fixtures produced the registered S1 amend and exactly one S3 partial close before the historical reply. S1 now polls a single bounded refresh future beside quote/execution processing. S3 continues owned-position exits, broker execution processing and its session clock while awaiting preparation, with one mutable trader and the existing execution gate. Flat-symbol entries retain their first eligible decision quote. Neither remedy creates a second trading-state writer.

[§FPM.19] An independent three-second watchdog reads current in-memory progress; actor timers call only the shared registered exit evaluators. Side-specific quotes older than six seconds, future timestamps and fresh crossed quotes are refused by the additive timer. Quote handling retains its registered logic. Health distinguishes watchdog completion from actor work, actual exit-pass completion, received/processed quote time and existing broker-book sample age. Unknown exit resolution remains null rather than a fabricated count. No new broker poll, database schema, raw-tick archive, inference or individual stop-loss notification is added. A slow reconciliation, execution gate or database operation can still delay its actor; the watchdog exposes that delay. Three seconds is a cadence, not an end-to-end execution guarantee.

[§FPM.20] Targeted checks cover held reads, single-flight refresh, no timer entry, TP1 overlap/deduplication, stale/future quote refusal, cancellation, reconnect cleanup, original protection and ownership. Necessary startup corrections to #51's watches compare the actual v3 digest, read typed production P&L coverage evidence with explicit unknowns, and deduplicate limit transitions by account/strategy/limit rather than moving measurements. Every generated/repaired PR now carries the owner's actual chat references and SGT preparation time. Exact test counts and release receipts belong in the dated journal/log. Required exact-head CI, Railway monitor observations and engineering resume remain the release gates. Financial progress from this code change is not measured; no passing trading window or historical loss recovery is claimed.
