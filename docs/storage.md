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
| `oauth_tokens` | secret state | One row, replaced on refresh. The refresh token is single-use. Never logged. |
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

### UI access (design; the UI is not built in phase 1)

- The UI is served by the same binary.
- Access is by a password in the environment variable `JIAOYI_UI_PASSWORD`, set by the owner. It is never generated, logged or written in the code. If it is missing, the UI refuses all access and says why.
- Sessions use an HttpOnly cookie.
- The only write routes change settings, through the bounds check and the change log above. The UI cannot place orders.

## Volume and retention

- **Daily bars:** about 20 symbols × 20 years × 260 days ≈ 104,000 rows per account.
- **Trading events:** a few thousand rows a year.
- **Ticks:** not measured yet. As an order of magnitude, 20 symbols on 2 accounts at about one update a second is roughly 3.5 million rows a day, about 0.3 GB a day with the index; 30 days of retention would hold about 10 GB. The real rate is read from `/health` after the first day on Railway.
- **Retention:** nothing is deleted, except whole days of `ticks` older than `ticks.retention_days`. Railway's managed backups cover the database.

## Not stored

- Intraday bars. (Recorded quotes are stored in `ticks` since phase 1: owner, 24-10-2026.)
- LLM text.
- Anything from the old bot-trade SQLite file. The old program's records stay where they are.
