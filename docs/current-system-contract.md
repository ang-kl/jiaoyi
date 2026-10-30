# Current system contract

**Document:** CONTRACT-20261030-CURRENT · **Prepared:** 30-10-2026, 20:35 SGT.
**Source:** owner “proceed to strengthen”, acknowledged № 17,919 §1 (20:30 SGT), following the C++ assessment № 17,918. By <Codex (GPT-6), effort: not exposed>.

[§CURRENT.1] This is the current-state entry point for engineering and handover. It corrects obsolete descriptions of a research-only, non-trading service. It creates no trading rule, registration, risk setting or new execution authority. Original documents remain preserved as dated evidence; later explicit owner decisions and active registrations govern their amended scope.

## Component responsibilities

| Component | Responsibility | What its health does not prove |
|---|---|---|
| Rust `jiaoyi` account actors | Broker subscriptions, S1/S3 decisions, owned-position exits and reconciliation; registered rules remain deterministic. | A running actor does not establish an edge or a bounded end-to-end broker response. |
| `jiaoyi_ctrader::trading::send` | Shared trading gate: account/ownership checks, permitted quantities/protection, intent before send, recorded execution outcome and unresolved-state handling. | A timeout does not prove that a frame was never accepted or authorise a duplicate send. |
| Independent three-second watchdog | Read-only progress observation, with received/processed quotes and actual exit-check completion distinguished. | Its tick is not an exit, broker-protection certificate or three-second execution guarantee. |
| PostgreSQL | Durable settings, decisions, intents, fills, history, registration and report evidence. | Persisted observations are not necessarily current broker state. |
| Python `jiaoyi-engineering` | Observe material changes, retain incidents, coordinate permitted bounded model work, test/release and verify behaviour. | A healthy observer does not establish available model sessions, an open mutation window or completed repairs. |
| cTrader | Authoritative broker orders, fills, pending state and accepted broker-side protection. | Historical acknowledgements do not certify every current SL/TP or prevent gaps and slippage. |
| Dividend reporting aid | Draft calendar and evidence note under its [separate contract](dividend-reporting-contract.md). | A calendar entry is not a signal, stock-preparation completion or order permission. |

[§CURRENT.2] S1 follows its daily trend specification with later owner amendments. S3 uses the [v6 registration](preregistration-2026-10-28-strategy3-v6.md), including one recorded entry and one durable cost veto per account/symbol/session. Scheduled entry marks coexist with quote-driven management; observing ticks does not make every strategy a tick-entry strategy. S3's registered Chandelier, TP1/TP2 and session exit remain active. Old “no S3 trailing” proposal text is historical. Existing trades retain their original opening plan, risk and registration attribution. Do not edit embedded registrations merely to improve prose.

## Authority and financial acceptance

[§CURRENT.3] The [factory charter](factory-charter.md) records the owner's later instruction to run live. Original research gates and failed trials remain economic evidence; live permission is not a research PASS. Only the owner pauses a strategy or changes reserved risk policy. A paused strategy retains its existing exits. Engineering shutdown, quota exhaustion or a dividend-data failure does not create a trading pause.

[§CURRENT.4] The live rule remains one broker-minimum program position at a time across strategies, with the owner's minimum-contract risk cap. The [margin policy](margin-buffer-entry-policy.md) records the 250% floor and fresh whole-account capacity checks, including existing positions, pending entries and stops. External/manual positions are reported and counted where required for capacity; they are not adopted or managed. Strategy 2 and a live basket exception are not activated by this document. Database settings and their audit trail determine effective values; a historic default is not proof of today's configuration.

[§CURRENT.5] The invariant goal is risk:reward 1:1.38 and at least 73% net wins over the latest 20 fully closed owned positions, separately per strategy/account. Fifteen wins meet the count; fewer than 20 is insufficient. Registered geometry, realised net payoff and win rate are distinct. Retain losses, missing costs and failed windows; overlapping passing windows do not establish independent consistency. Builds, monitor heartbeats and dividends do not establish progress towards this goal. Every reported delta requires comparable, qualified source evidence.

## Preparation and release boundaries

[§CURRENT.6] [S3 preparation](s3-preopen-preparation.md) begins at T-20 with a T-10 deadline under the registered session clocks. The session opening price, current executable quote, entry cost/R, ownership and current margin cannot be certified before they exist. Stock preparation is a separate [cache-only candidate view](queued-us-europe-stock-preparation.md): AAPL/MSFT on NASDAQ and SAP/SIE ordinary shares on Xetra, excluding China. Broker identity and actual stock history/quote preparation remain unverified; a ticker match or dividend calendar does not complete them.

[§CURRENT.7] Engineering work follows the [worker contract](engineering-worker.md), protected paths, explicit owner scope, session ceiling, authentication route and mutation deadline. Check these independently. “Time is Money” requires minimal demonstrated repairs, not using every permit. Keep raw evidence bounded and claims qualified. The owner's active repair instruction does not silently rewrite the separate worker's global permissions or allowance.

[§CURRENT.8] Required CI covers formatting, denied Clippy warnings, Rust/PostgreSQL tests, Python coordinator tests and image builds. Exact code identity, integration base, deployment identity and affected behaviour are separate checks. One successful health read is insufficient. Account-entry locks within one process do not by themselves prove cross-process writer exclusion; do not run competing executors during a migration. Verify component release scope and safe handover rather than assuming that a repository SHA establishes every deployed image or dependency. The [30 October strengthening evidence matrix](strengthening-validation-2026-10-30.md) separates observed checks, pending release verification and future evidence gates.

[§CURRENT.9] Handover records retain chat `№`, paragraph `§`, SGT source/preparation times and `by <AI model, effort>`, with unknown model information explicit. Consult `log/CODEX_Log.txt`, `log/GROK_Log.txt`, the current chat and relevant PR receipts before allocating a serial. Journal actual failed/passed checks and unresolved matters. Public routes retain privacy limits; owner HTML reports may show authorised account figures through their existing delivery route. No individual stop-loss Telegram is authorised by this map.
