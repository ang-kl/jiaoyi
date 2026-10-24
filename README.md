# jiaoyi

A daily trend-following trading system for cTrader accounts, written in Rust, with PostgreSQL storage, deployed in its own Railway project, `jiaoyi_prj`.

It is the ground-up rebuild ordered by the owner on 24-10-2026, replacing the bot-trade program.

## Status

Data and edge verification: the service reads each permitted account's own daily bars, symbol details, costs, live quote stream, historical ticks and deal history from cTrader into PostgreSQL, and reports what exists. It contains no code that places, amends, closes or cancels an order: the client's payload allowlist names only read requests. Live trading waits until the edge verification shows an edge (owner-approved plan, 24-10-2026).

- [`docs/phase-0-rulebook.md`](docs/phase-0-rulebook.md) — the trading rules, with every number fixed in advance, and the gates each phase must pass.
- [`docs/storage.md`](docs/storage.md) — the PostgreSQL design: event log, intent before send, broker as the authority.

Nothing trades until phase 3, the history test, passes. Live trading needs the owner's word.

## Running

One binary, `jiaoyi`, deployed by `railway.toml` (Dockerfile build, healthcheck `/health`).

| Variable | Required | Meaning |
|---|---|---|
| `JIAOYI_ALLOWED_ACCOUNTS` | yes | cTrader account ids (`ctidTraderAccountId`) this service may touch, e.g. `49236051,43002148`. Missing or empty: the service refuses to start. |
| `JIAOYI_ACCOUNT_ENVIRONMENTS` | yes | Which server each account is on, e.g. `49236051:demo,43002148:live`. An allowlisted account without one is refused and recorded; the server is not guessed. |
| `DATABASE_URL` | yes | PostgreSQL. Migrations run on start. |
| `CTRADER_CLIENT_ID`, `CTRADER_CLIENT_SECRET` | yes | The Open API application. |
| `CTRADER_ACCESS_TOKEN`, `CTRADER_REFRESH_TOKEN` | first start, or a new grant | Seed the token store. After a refresh the database holds the current pair; the same seed is not stored over it again. A NEW seed (a different access token, e.g. after re-granting the app access to an added account) replaces the stored pair, and the replacement is recorded by fingerprint (docs/storage.md). |
| `CTRADER_ACCESS_TOKEN_EXPIRES_AT` | no | RFC 3339 expiry of the seed token; without it the token is refreshed only when the broker rejects it. |
| `CTRADER_DEMO_HOST`, `CTRADER_LIVE_HOST`, `CTRADER_PORT` | no | Default `demo.ctraderapi.com`, `live.ctraderapi.com`, `5035`. |
| `PORT` | no | HTTP port, default 8080. |

Every tunable number is a setting in the database (`docs/storage.md`, "Settings").

HTTP: `GET /health` (status, last load per account, tick recorders, history sync, settings version), `GET /research/inventory` (per account and symbol: what history exists — daily bars, live and historical ticks, deals — its coverage, gaps and what is missing; counts and dates only) `GET /phase1/report` (per account and symbol: resolution, broker id, bars, first and last date, years of history, median spread, spread ÷ ATR, and the rulebook §3 verdict with its reasons; per account, `energy_candidates`: the broker's own names that may be energy instruments), and `GET /research/bars?account=&symbol=&from=&to=` (development-period daily bars exactly as stored, at most 5,000 a reply with `next_from` to continue; allowlisted accounts only; a `to` on or after 2025-10-24, or a bar running past that seal, is refused, never clamped: docs/preregistration-2026-10-24.md), `GET /research/jobs` (the pre-registered research jobs, their status, reasons and attempts, and the measured tick download rate of the spread samples) and `GET /research/trials` (each job's trial: counts and R, the bootstrap range, the control, per-year results, the evidence against, the rules and the verdict; the deflated Sharpe once the 9 development trials exist; no money).

Tests need a PostgreSQL: set `DATABASE_URL` and run `cargo test --workspace`. Each database test creates its own fresh database.
