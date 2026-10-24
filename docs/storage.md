# Storage design: PostgreSQL

Status: **DRAFT, part of phase 0.** Owner, 24-10-2026: "yes, start phase 0 with Postgres".

## Principles

1. **One managed PostgreSQL database** in the Railway project `jiaoyi_prj` (created by the owner, 24-10-2026; region asia-southeast1), separate from bot-trade's project. Demo and live share it, and every row carries `account_id`.
2. **The event log is the source of truth.** Rows are written once and never updated or deleted. Current state is derived from them.
3. **Intent before send.** An order intent row with its own `client_order_id` is committed before the order goes to the broker. After a restart, the intent is matched against the broker by that id, so an order is never sent twice.
4. **The broker is the authority on positions.** At every start, and every hour, the database is reconciled against the broker. When they disagree, the broker wins and the disagreement is stored as an event.
5. **One writer.** Only the trader service writes trading tables. The web service connects with a read-only role.
6. **Migrations live in the repo** (`migrations/`, applied by `sqlx migrate` on deploy) and are tested against a real Postgres in CI.
7. **No guessed values.** A column that cannot be read from the broker is NULL, with a reason column beside it. It is never filled with a default that looks real.

## Tables (outline; types finalised in phase 1)

| Table | Kind | Key columns |
|---|---|---|
| `accounts` | config | `account_id`, `login`, `environment` (demo/live), `currency`, `role`, `enabled` |
| `symbols` | per account | `account_id`, `broker_symbol_id`, `name`, `digits`, `lot_size`, `min_volume`, `volume_step`, `value_per_point`, `read_at` |
| `daily_bars` | market data | `account_id`, `broker_symbol_id`, `bar_date`, `open`, `high`, `low`, `close`, `tick_volume`, `fetched_at`. Primary key (account, symbol, date). Closed bars only. |
| `symbol_costs` | market data | `account_id`, `broker_symbol_id`, `measured_at`, `spread_median`, `commission_per_lot_side`, `swap_long`, `swap_short`, `source` |
| `signals` | event | `id`, `account_id`, `symbol_id`, `bar_date`, `side`, `signal_close`, `atr`, `filter_state`, `decision` (enter/skip), `skip_reason` |
| `order_intents` | event | `client_order_id` (primary key), `signal_id`, `account_id`, `side`, `volume`, `stop_price`, `risk_money`, `risk_cap_check` (the hard-cap verdict), `created_at` |
| `order_events` | event | `id`, `client_order_id`, `kind` (sent / accepted / rejected / filled / partial / cancelled), `broker_order_id`, `broker_position_id`, `price`, `volume`, `commission`, `at`, `raw` |
| `stop_events` | event | `id`, `broker_position_id`, `old_stop`, `new_stop`, `reason` (initial/trail), `broker_confirmed`, `at` |
| `positions` | derived state | One row per open position, rebuilt from events and checked against the broker |
| `trades` | derived | One row per closed position: entry, exit, initial R, realised R net of costs, money, hold days, exit reason |
| `reconcile_events` | event | Every disagreement between the database and the broker, and how it was resolved |
| `equity_snapshots` | event | Balance and equity per account at each daily close, read from the broker |
| `research_trials` | research log | `trial_id`, `rulebook_hash`, `params`, `data_range`, `results_json`, `run_at`. Every history test run, pass or fail. |

## Phase 1 as built (24-10-2026)

The phase-1 migrations (`migrations/`) create the tables below. Types are final for these tables; the trading tables above stay an outline until their phase.

| Table | Kind | Notes |
|---|---|---|
| `accounts` | config | Permitted accounts only. `login`, `currency` and `role` are NULL with a reason: phase 1 does not read them. |
| `account_refusals` | event, append-only | Every account refused at start-up (not on `JIAOYI_ALLOWED_ACCOUNTS`, or no declared environment), with the reason. |
| `load_runs` | one row per load | Account, start and finish, status (`ok` / `partial` / `failed`), counts, cross-check results, errors, and the settings version the load read. |
| `universe_resolutions` | event, append-only | How each universe name resolved on each load: `resolved`, `missing` or `ambiguous`, with the reason. A miss without a reason is refused by the database. |
| `symbols` | current state | `ProtoOASymbol` fields as the broker gives them, plus the raw message bytes. `value_per_point` is NULL with a reason (the broker does not give it). Per-column reasons for NULLs are in `null_reasons`. |
| `symbol_costs` | event, append-only | Median spread from spot quotes, and commission and swap fields exactly as the broker gives them. `commission_per_lot_side` is NULL with a reason (not derived in phase 1). |
| `daily_bars` | market data | Closed bars only. Prices exact (`NUMERIC`, relative ÷ 100000). Deletes are refused. A re-read that differs is applied, and the old and new values are written to `reconcile_events` first. |
| `ticks` | market data, partitioned by UTC day | The live spot stream as received: bid and ask in cTrader's relative unit (1/100000), the broker's timestamp as sent, and the receive time. A side the event did not carry is NULL (the proxy sends only the side that changed). Whole days older than `ticks.retention_days` are dropped; that is the only deletion of recorded data. |
| `oauth_tokens` | secret state | One row, replaced on refresh, or by a new env seed (below). The refresh token is single-use. Never logged. `seed_fingerprint`: the env seed it descends from. |
| `token_grants` | event, append-only | Every read of the token's grant, made before authorising an account (migration 0007): the account, host, token fingerprint and source, the verdict, and the accounts listed. |
| `reconcile_events` | event, append-only | Every disagreement with the broker: changed bars, cross-check mismatches, refused trendbars, bar-date collisions. |
| `setting_definitions`, `settings`, `settings_version`, `settings_changes` | settings | See below. |

### Settings

Owner, 24-10-2026: *"no hard code settings, the UI should have setting to tweak. And traceability is important."*

- Every tunable number is a row in `settings`: a global value, optionally overridden per account (the account value wins).
- `setting_definitions` holds each key's kind, bounds, description and the source of its default (the rulebook section, the owner's words, or "self-derived").
- A value outside its bounds is refused with the reason. Nothing is clamped.
- Every change bumps `settings_version` and is written to the append-only `settings_changes`: key, scope, old value, new value, who, when, and a required reason. The migration's seeds are logged the same way, by `migration`.
- The code reads the settings at each decision. Each load run stores the version it read.
- The account allowlist is not a setting. It is an identity boundary and stays in the environment.

### Fixes from the first deploy (migration 0005, 24-10-2026)

- **Partitions** (`ticks` daily, `ticks_history` monthly) are created under a transaction advisory lock keyed on the partition name, with `IF NOT EXISTS`; a concurrent creator's SQLSTATE 42P07 counts as success. (Migration 0007 checks `to_regclass` first and drops `IF NOT EXISTS`: see below.)
- **Loads**: a run that failed on authorisation after the token was refreshed is tried again at once (`load.auth_retry_attempts`, `load.auth_retry_secs`, doubling). A failed or partial run is retried after `load.retry_minutes` (default 5), not after `load.interval_hours`.
- **Spreads**: a median from fewer than `spread.min_samples` quote updates (default 30) is not a measurement; the verdict is `undecided` ("spread not measured: N samples"). Each measurement records whether the market was open, from the symbol's own schedule (`symbol_costs.market_open`, with `market_state_reason`; only UTC schedules are evaluated).
- **Universe** (owner, 24-10-2026: "use the broker's names for energy, like NatGas, etc."): one logged change by `owner` — NATGAS → NatGas, USOIL → SpotCrude, UKOIL removed until its broker name is known. Each load stores the broker's symbol list (`load_runs.listed_symbols`), and `/phase1/report` lists `energy_candidates` per account. Migration 0008 adds SpotBrent to energy, one more logged change by `owner` (Brent is SpotBrent on the broker: `energy_candidates`, load run 6 on 43002148). No futures (-F) or perpetual (-PERP) names. The demo account's names are not known until it loads, and are not guessed.

### Token grant check and seed replacement (migration 0007, 24-10-2026)

Deploy 7765158a: demo 49236051 was refused (CH_ACCESS_TOKEN_INVALID) on every authorisation, even straight after a refresh, while live 43002148 authorised with the same token; and every refresh ended the live sessions. A refresh renews a grant; it never adds an account.

- **Grant check.** After the application is authorised and before an account is, `ProtoOAGetAccountListByAccessTokenReq` (read-only) lists the accounts the token's grant covers. Each read goes to `token_grants`, the token by fingerprint only (first 16 hex characters of its SHA-256). An account the grant omits is refused: `token grant does not include account N (grant lists: [...])`. One listed with an `isLive` that disagrees with its declared environment is refused too. Neither refreshes the token; the load, the recorder and the history sync re-read the grant every `load.retry_minutes`. If the read itself is refused as an expired or invalid token, the token is refreshed once and the grant read again. A listed account that is still refused gets the single refresh, as before. `/health` shows each account's grant state (`listed`, `not_listed`, `environment_mismatch`, `unread`) with the reason and fingerprint; `/phase1/report` shows the last read.
- **Seed replacement.** A seed in the environment whose fingerprint differs from `oauth_tokens.seed_fingerprint` replaces the stored token (source `env_seed`), and `reconcile_events` records it (`token_seed_replaced`, fingerprints only). The same seed keeps the stored, refreshed token. A row stored before 0007 has no fingerprint; it is judged once. It is replaced if it came from a different seed, or if it came from a refresh and the seed's grant is accepted and adds an account (or the stored token is refused). Otherwise it is kept, and the seed's fingerprint is adopted (`token_seed_baseline`).

### History sync: session ends and partitions (migration 0007)

- When the broker ends the account session mid-sync (token invalidated, account disconnected, socket closed), the pass stops. The window cut short is recorded with `failure_kind = 'session_ended'` (others `broker`). `/research/inventory` counts that week as missing, not failed. The sync re-authorises from the shared token store after `history.session_retry_secs` (default 5) and resumes at the first unfinished window. If a resumed pass ends before storing a window, it waits the normal 5 minutes.
- `/research/inventory` lists each account's distinct fetch errors (tick windows, deal windows, probe steps) with count and last time.
- Partitions: an existing partition is found with `to_regclass` before any CREATE, so the "already exists, skipping" notice cannot occur; the advisory lock and the 42P07 handling stay for racing creators.

### UI access (design; the UI is not built in phase 1)

- The UI is served by the same binary.
- Access is by a password in the environment variable `JIAOYI_UI_PASSWORD`, set by the owner. It is never generated, logged or written in the code. If it is missing, the UI refuses all access and says why.
- Sessions use an HttpOnly cookie.
- The only write routes change settings, through the bounds check and the change log above. The UI cannot place orders.

## Edge verification data (owner-approved plan, 24-10-2026)

Read-only against the broker. Migration 0004.

| Table | Kind | Notes |
|---|---|---|
| `ticks_history` | market data, partitioned by UTC month | The broker's historical bid and ask ticks (`ProtoOAGetTickDataReq`), `source = broker_history`, price in 1/100000 as sent. `dup` numbers identical ticks at one instant, so repeats are kept and a re-download stores nothing twice. |
| `history_fetches` | event, append-only | Every tick request window: what was asked, how many ticks came back, how many were new, which difference sign decoded them, and any error. `kind = probe` rows only count ticks, to find how far back the history goes. |
| `history_probes` | event, append-only | Every probe search as a whole (migration 0006): `complete` with the earliest week found, `no_ticks`, or `incomplete` with the failing step's error and no result. Only a completed search starts the re-probe cooldown; a failed request is never read as an empty week. |
| `deals` | broker record | Every deal exactly as `ProtoOADeal` gives it, money as the broker's integers with `money_digits`, plus the raw message. Deletes are refused; a deal the broker later reports differently is written to `reconcile_events` first. |
| `deal_fetches` | event, append-only | Every deal-list window and its outcome. |
| `account_snapshots` | event, append-only | `ProtoOATraderRes` each time it is read: login, deposit asset, balance, registration date. |
| `research_trials` | research log, append-only | Every trial, pass or fail, with its parameters, data range, sample (`development` or `held_out`) and settings version. |

### Ticks history: depth and size

- Tick requests span at most one week (Spotware). Windows are Monday-to-Monday UTC weeks on a fixed grid, so a sync resumes, and a week fetched completely is not fetched again.
- The depth downloaded is the setting `history.tick_days_back` (default 7 days). The probe (`history.probe_years`, default 15) finds the earliest week with ticks without downloading the years in between.
- Size is the limit, not the broker. As an order of magnitude only (not measured), a busy FX pair can carry 100,000–200,000 quote changes a day per side; 20 symbols × 2 sides × a year would be on the order of 100 GB. The depth is raised once the first week's real volume is measured from `/research/inventory`.

## Research engine: the pre-registered edge test (migration 0009)

The rules are `docs/preregistration-2026-10-24.md` (sha256 `9105c5eb…`), embedded in the binary and hashed at start. A job under any other hash is refused.

| Table | Kind | Notes |
|---|---|---|
| `research_jobs` | job queue | The trial cap, inserted by the migration only: the 9 H1 development cells (breakout {20, 50, 100} × stop {2, 3, 4} × ATR, the default (50, 3) first) and 1 held-out job at the default. Status `pending` → `running` → `done` / `refused` / `failed`. A job's definition never changes, a finished job never runs again, and nothing is deleted (trigger). |
| `research_job_attempts` | event, append-only | Every attempt: start time and deployed commit (`RAILWAY_GIT_COMMIT_SHA`, or `unknown`). An attempt interrupted by a stop wrote nothing, so the job is attempted again (owner, 24-10-2026). Its trial lists every attempt, and a changed commit is reported under the evidence against. |
| `research_job_events` | event, append-only | Every status a job takes, with the reason. |
| `research_trials` | research log, append-only | Now with `prereg_hash` and `job_id` (one trial per job). The trial and the job's `done` are one transaction. |
| `research_spreads` | event, append-only | One June week (the second full week) per symbol and development year (2013–2025): the median ask − bid of the broker's ticks, which are streamed and never stored raw; the bar quote-basis observations; ticks and wall time. The schema refuses a week ending after the seal. |

- **Seal.** One data-access layer per stage. A development job reads only bars that end (open + 24 h) by 2025-10-24 00:00 UTC, and ticks before then. A request past that is refused, never clamped. Held-out jobs read up to 2026-10-24 00:00 UTC.
- **Gate.** The held-out job is refused, with the reason, unless the development default met PASS rules 1–3 and at least 6 of the 9 cells have a profit factor above 1.0, under the same hash. The verdict is then "no demonstrated edge".
- **Inputs before an attempt.** A job waits, unattempted, until every universe symbol's June samples exist and the deposit currency is established. The deposit currency is the trader record's deposit asset, matched with the asset the broker lists as USD (loads now keep the base and quote asset ids). The cross-check is that closing deals on USD-quoted symbols convert at 1. Either failing fails the job.
- **Commission** comes from the live deals (21-10-2025 → 16-01-2026, the source the pre-registration names). It reads only the class, commission, money digits and filled volume, never a price; a test pins the query.
- **Readings** where the text leaves a choice are written into every trial's params, `readings`.
- `GET /research/jobs` and `GET /research/trials` show counts and R only, plus the deflated Sharpe once all 9 development trials exist. Notional money and the trade list stay in the stored trial.

## Volume and retention

- **Daily bars:** about 20 symbols × 20 years × 260 days ≈ 104,000 rows per account.
- **Trading events:** a few thousand rows a year.
- **Ticks:** not measured yet. As an order of magnitude, 20 symbols on 2 accounts at about one update a second is roughly 3.5 million rows a day, about 0.3 GB a day with the index; 30 days of retention would hold about 10 GB. The real rate is read from `/health` after the first day on Railway.
- **Retention:** nothing is deleted, except whole days of `ticks` older than `ticks.retention_days`. Railway's managed backups cover the database.

## Not stored

- Intraday bars. (Recorded quotes are stored in `ticks` since phase 1: owner, 24-10-2026.)
- LLM text.
- Anything from the old bot-trade SQLite file. The old program's records stay where they are.
