# jiaoyi

A deterministic trading system for cTrader accounts, written in Rust with PostgreSQL storage, deployed in the Railway project `jiaoyi_prj`. A separate Python engineering worker observes evidence and performs authorised, bounded maintenance.

It is the ground-up rebuild ordered by the owner on 24-10-2026, replacing the bot-trade program.

## Status

**The service places and manages trades.** The owner authorised live operation after the original 24 October research-only plan. Strategy 1 runs the daily trend rules; Strategy 3 runs the registered intraday rules, currently [v6](docs/preregistration-2026-10-28-strategy3-v6.md). Both retain broker-side protection and send trading frames through one execution gate. The HTTP routes remain read-only; that does not make the trading service read-only.

Start with the [current system contract](docs/current-system-contract.md) for component responsibilities, authoritative rules and verification boundaries. The [factory charter](docs/factory-charter.md) records the owner's later decisions and invariants. Historical research gates and failed results remain evidence; owner-authorised live trading is not a financial PASS.

The financial goal remains risk:reward **1:1.38** and at least **73% net wins over the latest 20 fully closed owned positions**, separately per strategy/account. Fifteen of twenty meets the win-count threshold; fewer than twenty is insufficient evidence. Registered geometry, realised payoff, software correctness and economic acceptance are separate claims.

Other entry points:

- [Original rulebook](docs/phase-0-rulebook.md) and [S1 registration](docs/preregistration-2026-10-24.md): retained historical research specification, read together with later owner amendments.
- [Storage](docs/storage.md): intent before send, broker authority and retained evidence.
- [Engineering worker](docs/engineering-worker.md): separate permissions, allowance, queue and release checks.
- [Dividend reporting contract](docs/dividend-reporting-contract.md): draft-only analytical aid; no order authority. Its proposed timed routine is not an operational feed or scheduler.

## Running

The trading binary, `jiaoyi`, is deployed by `railway.toml` (Dockerfile build, healthcheck `/health`). The separate `jiaoyi-engineering` service uses `ops/engineering/`; its deployment and configuration follow the [worker contract](docs/engineering-worker.md). A healthy observer does not establish that model allowance or mutation authority is available.

| Variable | Required | Meaning |
|---|---|---|
| `JIAOYI_ALLOWED_ACCOUNTS` | yes | cTrader account ids (`ctidTraderAccountId`) this service may touch, e.g. `49236051,43002148`. Missing or empty: the service refuses to start. |
| `JIAOYI_ACCOUNT_ENVIRONMENTS` | yes | Which server each account is on, e.g. `49236051:demo,43002148:live`. An allowlisted account without one is refused and recorded; the server is not guessed. |
| `JIAOYI_DEMO_ACCOUNT_ID`, `JIAOYI_DEMO_ACCOUNT_ID_<N>` | no | One more demo account each, N a positive integer, e.g. `JIAOYI_DEMO_ACCOUNT_ID_2=46130058` (owner, 25-10-2026). Permitted and declared demo, in addition to the pair above. No tick history download, unless the pair names the account too. A malformed one is refused and recorded under its name; start-up goes on. |
| `DATABASE_URL` | yes | PostgreSQL. Migrations run on start. |
| `CTRADER_CLIENT_ID`, `CTRADER_CLIENT_SECRET` | yes | The Open API application. |
| `CTRADER_ACCESS_TOKEN`, `CTRADER_REFRESH_TOKEN` | first start, or a new grant | Seed the token store. After a refresh the database holds the current pair; the same seed is not stored over it again. A NEW seed (a different access token, e.g. after re-granting the app access to an added account) replaces the stored pair, and the replacement is recorded by fingerprint (docs/storage.md). |
| `CTRADER_ACCESS_TOKEN_EXPIRES_AT` | no | RFC 3339 expiry of the seed token; without it the token is refreshed only when the broker rejects it. |
| `CTRADER_DEMO_HOST`, `CTRADER_LIVE_HOST`, `CTRADER_PORT` | no | Default `demo.ctraderapi.com`, `live.ctraderapi.com`, `5035`. |
| `PORT` | no | HTTP port, default 8080. |

Operational and risk settings are recorded in the database (`docs/storage.md`, "Settings"). Registered strategy parameters also live in versioned code and pre-registration documents. An explanatory documentation edit must not alter an embedded registration or silently change its hash.

Public GET routes expose qualified evidence without account money or secrets:

| Route | Purpose and limit |
|---|---|
| `/`, `/health` | Status, strategy state, source diagnostics and monitor progress. The independent three-second watchdog observes actor progress; it does not send exits or guarantee three-second execution. |
| `/operations`, `/operations/reports` | Per-strategy/account assessments, source qualification and margin-admission receipts. A last candidate receipt is not a current margin certificate. |
| `/operations/stop-loss-reviews`, `/operations/trailing-receipts` | Recorded trade-review and request/acknowledgement evidence, with original ownership and missing facts retained. |
| `/operations/stock-preparation` | On-demand US/European stock candidate view. Broker identity and actual stock-input preparation remain explicitly unverified; this route does not activate stock trading. |
| `/research/inventory`, `/phase1/report` | Historical coverage, gaps, broker symbol resolution and original rulebook checks. |
| `/research/bars?account=&symbol=&from=&to=` | Stored development-period daily bars, at most 5,000 per reply with `next_from`; a `to` on or after 2025-10-24 or a bar crossing that seal is refused. |
| `/research/jobs`, `/research/trials` | Registered attempts, development/held-out scope, measured results and retained failed verdicts. |
| `/trading/orders`, `/trading/executions` | Recorded order/execution diagnostics; source timestamps and account scope matter. |
| `/trading/costs` | Recorded execution-cost summaries. Entry quotes, broker stop/TP levels and missing recovered-exit references must not be conflated; aggregates are not paired current-registration cost validation. |

Tests need a PostgreSQL: set `DATABASE_URL` and run `cargo test --workspace`. Each database test creates its own fresh database.

`GET /trading/executions?account=&since=&limit=1..500` is the read-only O-1 diagnostic view for recorded events tied to Strategy 3 opening fills. It reports event identifiers/type, whether correlation and a strategy label are present, whether the label matches, and whether the original payload contains a closing deal. Accounts are restricted to the running service's allowlist; raw payloads, label text, correlation IDs, prices and money remain private. `decodable: false` means payload-derived fields are unknown (`null`). Records are newest first; `since` is RFC 3339.
