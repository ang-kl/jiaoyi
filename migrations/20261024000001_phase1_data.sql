-- Phase 1: data only. Tables from docs/storage.md, types finalised here.
--
-- Rules (docs/storage.md):
--   * Append-only tables refuse UPDATE and DELETE by trigger.
--   * No guessed values: a value the broker did not give is NULL, and the row
--     says why (a *_missing_reason column, or the null_reasons JSON object
--     whose keys are column names).

CREATE FUNCTION jiaoyi_refuse_change() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'table % is append-only: % refused', TG_TABLE_NAME, TG_OP
        USING ERRCODE = 'insufficient_privilege';
END
$$;

-- Accounts this service may load. Config, one row per permitted account.
CREATE TABLE accounts (
    account_id               BIGINT PRIMARY KEY,               -- ctidTraderAccountId
    environment              TEXT NOT NULL CHECK (environment IN ('demo', 'live')),
    login                    BIGINT,
    login_missing_reason     TEXT,
    currency                 TEXT,
    currency_missing_reason  TEXT,
    role                     TEXT,
    role_missing_reason      TEXT,
    enabled                  BOOLEAN NOT NULL,
    first_seen_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((login IS NULL) = (login_missing_reason IS NOT NULL)),
    CHECK ((currency IS NULL) = (currency_missing_reason IS NOT NULL)),
    CHECK ((role IS NULL) = (role_missing_reason IS NOT NULL))
);

-- Every account refused by the allowlist or by its environment declaration.
CREATE TABLE account_refusals (
    id          BIGSERIAL PRIMARY KEY,
    account_ref TEXT NOT NULL,          -- as given; may not be a number
    reason      TEXT NOT NULL,
    source      TEXT NOT NULL,          -- e.g. 'startup'
    at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER account_refusals_append_only BEFORE UPDATE OR DELETE ON account_refusals
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- One row per load of one account. Written at start, closed once at finish.
CREATE TABLE load_runs (
    id                     BIGSERIAL PRIMARY KEY,
    account_id             BIGINT NOT NULL REFERENCES accounts (account_id),
    started_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at            TIMESTAMPTZ,
    status                 TEXT NOT NULL DEFAULT 'running'
                           CHECK (status IN ('running', 'ok', 'partial', 'failed')),
    symbols_resolved       INTEGER NOT NULL DEFAULT 0,
    symbols_unresolved     INTEGER NOT NULL DEFAULT 0,
    bars_inserted          INTEGER NOT NULL DEFAULT 0,
    bars_changed           INTEGER NOT NULL DEFAULT 0,
    bars_unchanged         INTEGER NOT NULL DEFAULT 0,
    bars_refused           INTEGER NOT NULL DEFAULT 0,
    forming_bars_dropped   INTEGER NOT NULL DEFAULT 0,
    spreads_measured       INTEGER NOT NULL DEFAULT 0,
    crosscheck             JSONB NOT NULL DEFAULT '[]'::jsonb,
    errors                 JSONB NOT NULL DEFAULT '[]'::jsonb,
    CHECK ((status = 'running') = (finished_at IS NULL))
);
CREATE INDEX load_runs_account_started ON load_runs (account_id, started_at DESC);

-- How each universe name resolved on each load. Append-only.
CREATE TABLE universe_resolutions (
    id                BIGSERIAL PRIMARY KEY,
    load_run_id       BIGINT NOT NULL REFERENCES load_runs (id),
    account_id        BIGINT NOT NULL REFERENCES accounts (account_id),
    name              TEXT NOT NULL,
    asset_class       TEXT NOT NULL,
    status            TEXT NOT NULL CHECK (status IN ('resolved', 'missing', 'ambiguous')),
    broker_symbol_id  BIGINT,
    candidate_ids     BIGINT[] NOT NULL DEFAULT '{}',
    reason            TEXT,
    at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((status = 'resolved') = (broker_symbol_id IS NOT NULL)),
    CHECK ((status = 'resolved') = (reason IS NULL))
);
CREATE TRIGGER universe_resolutions_append_only BEFORE UPDATE OR DELETE ON universe_resolutions
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX universe_resolutions_latest ON universe_resolutions (account_id, name, id DESC);

-- Symbol details as the broker gives them (ProtoOASymbol), per account.
-- Current state: replaced on each read; the raw message is kept beside it.
CREATE TABLE symbols (
    account_id              BIGINT NOT NULL REFERENCES accounts (account_id),
    broker_symbol_id        BIGINT NOT NULL,
    name                    TEXT NOT NULL,
    asset_class             TEXT NOT NULL,
    listed_enabled          BOOLEAN,              -- ProtoOALightSymbol.enabled
    trading_mode            TEXT,                 -- ProtoOATradingMode name
    digits                  INTEGER NOT NULL,
    pip_position            INTEGER NOT NULL,
    lot_size                BIGINT,               -- in cents of volume
    min_volume              BIGINT,               -- in cents of volume
    max_volume              BIGINT,
    volume_step             BIGINT,
    value_per_point         NUMERIC,              -- not given by the broker; see null_reasons
    schedule_time_zone      TEXT,
    measurement_units       TEXT,
    null_reasons            JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw                     BYTEA NOT NULL,       -- the ProtoOASymbol bytes as received
    read_at                 TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (account_id, broker_symbol_id)
);

-- Costs as measured or read at a point in time. Append-only.
CREATE TABLE symbol_costs (
    id                               BIGSERIAL PRIMARY KEY,
    account_id                       BIGINT NOT NULL,
    broker_symbol_id                 BIGINT NOT NULL,
    load_run_id                      BIGINT REFERENCES load_runs (id),
    measured_at                      TIMESTAMPTZ NOT NULL,
    spread_median                    NUMERIC,     -- price units
    spread_samples                   INTEGER NOT NULL,
    spread_window_secs               INTEGER NOT NULL,
    commission_per_lot_side          NUMERIC,     -- not derived in phase 1; see null_reasons
    commission_type                  TEXT,
    precise_trading_commission_rate  BIGINT,
    commission_deprecated            BIGINT,
    precise_min_commission           BIGINT,
    min_commission_type              TEXT,
    min_commission_asset             TEXT,
    swap_long                        DOUBLE PRECISION,
    swap_short                       DOUBLE PRECISION,
    swap_calculation_type            TEXT,
    swap_rollover_3days              TEXT,
    swap_period_hours                INTEGER,
    swap_time_minutes                INTEGER,
    source                           TEXT NOT NULL,
    null_reasons                     JSONB NOT NULL DEFAULT '{}'::jsonb,
    FOREIGN KEY (account_id, broker_symbol_id) REFERENCES symbols (account_id, broker_symbol_id),
    CHECK (spread_samples >= 0),
    CHECK ((spread_median IS NULL) = (spread_samples = 0))
);
CREATE TRIGGER symbol_costs_append_only BEFORE UPDATE OR DELETE ON symbol_costs
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX symbol_costs_latest ON symbol_costs (account_id, broker_symbol_id, measured_at DESC);

-- Closed daily bars, the account's own, under its own symbol id.
-- A re-read that differs is applied (the broker is the authority) and the old
-- and new values are recorded in reconcile_events first.
CREATE TABLE daily_bars (
    account_id        BIGINT NOT NULL,
    broker_symbol_id  BIGINT NOT NULL,
    bar_date          DATE NOT NULL,          -- UTC date of bar_open_utc
    bar_open_utc      TIMESTAMPTZ NOT NULL,   -- utcTimestampInMinutes
    open              NUMERIC(24, 5) NOT NULL,
    high              NUMERIC(24, 5) NOT NULL,
    low               NUMERIC(24, 5) NOT NULL,
    close             NUMERIC(24, 5) NOT NULL,
    tick_volume       BIGINT NOT NULL,
    fetched_at        TIMESTAMPTZ NOT NULL,
    load_run_id       BIGINT REFERENCES load_runs (id),
    PRIMARY KEY (account_id, broker_symbol_id, bar_date),
    FOREIGN KEY (account_id, broker_symbol_id) REFERENCES symbols (account_id, broker_symbol_id),
    CHECK (low <= open AND open <= high AND low <= close AND close <= high)
);
CREATE TRIGGER daily_bars_no_delete BEFORE DELETE ON daily_bars
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- The cTrader OAuth token. One row per token owner, replaced on refresh.
-- Never logged. The refresh token is single-use: the new one must be stored.
CREATE TABLE oauth_tokens (
    token_owner                 TEXT PRIMARY KEY,
    access_token                TEXT NOT NULL,
    refresh_token               TEXT NOT NULL,
    token_type                  TEXT,
    expires_at                  TIMESTAMPTZ,
    expires_at_missing_reason   TEXT,
    obtained_at                 TIMESTAMPTZ NOT NULL,
    source                      TEXT NOT NULL CHECK (source IN ('env_seed', 'refresh')),
    CHECK ((expires_at IS NULL) = (expires_at_missing_reason IS NOT NULL))
);

-- Every disagreement between the database and the broker, and its resolution.
CREATE TABLE reconcile_events (
    id          BIGSERIAL PRIMARY KEY,
    account_id  BIGINT,
    at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    kind        TEXT NOT NULL,
    subject     TEXT NOT NULL,
    detail      JSONB NOT NULL,
    resolution  TEXT NOT NULL
);
CREATE TRIGGER reconcile_events_append_only BEFORE UPDATE OR DELETE ON reconcile_events
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX reconcile_events_account_at ON reconcile_events (account_id, at DESC);
