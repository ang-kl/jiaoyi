-- Edge verification, data only (owner-approved plan, 24-10-2026): the
-- broker's historical ticks and deal history, an inventory of what exists, and
-- the trial log. Everything here is read from the broker; nothing trades.

-- ------------------------------------------------------------------ ticks

-- Historical bid and ask ticks as the broker returns them
-- (ProtoOAGetTickDataReq). Price in 1/100000, as sent. `dup` numbers ticks
-- with the same timestamp and price (0, 1, ...) so repeats are kept and a
-- re-download inserts nothing new. Partitioned by UTC month of the tick.
CREATE TABLE ticks_history (
    account_id        BIGINT NOT NULL,
    broker_symbol_id  BIGINT NOT NULL,
    side              TEXT NOT NULL CHECK (side IN ('bid', 'ask')),
    ts                TIMESTAMPTZ NOT NULL,
    price_e5          BIGINT NOT NULL CHECK (price_e5 > 0),
    dup               INTEGER NOT NULL DEFAULT 0 CHECK (dup >= 0),
    source            TEXT NOT NULL DEFAULT 'broker_history' CHECK (source = 'broker_history'),
    fetch_id          BIGINT NOT NULL
) PARTITION BY RANGE (ts);
CREATE UNIQUE INDEX ticks_history_key ON ticks_history
    (account_id, broker_symbol_id, side, ts, price_e5, dup);

CREATE FUNCTION jiaoyi_ensure_history_partition(month date) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
    first date := date_trunc('month', month)::date;
    name text := 'ticks_history_p' || to_char(first, 'YYYYMM');
BEGIN
    IF to_regclass(name) IS NULL THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF ticks_history FOR VALUES FROM (%L) TO (%L)',
            name,
            (first::timestamp AT TIME ZONE 'UTC'),
            ((first + interval '1 month')::timestamp AT TIME ZONE 'UTC'));
    END IF;
    RETURN name;
END
$$;

-- One row per tick request window: what was asked, what came back. Append-only.
-- kind 'download' stores the ticks; kind 'probe' only counts them, to find how
-- far back the broker's history goes.
CREATE TABLE history_fetches (
    id                BIGSERIAL PRIMARY KEY,
    account_id        BIGINT NOT NULL,
    broker_symbol_id  BIGINT NOT NULL,
    side              TEXT NOT NULL CHECK (side IN ('bid', 'ask')),
    kind              TEXT NOT NULL CHECK (kind IN ('download', 'probe')),
    window_from       TIMESTAMPTZ NOT NULL,
    window_to         TIMESTAMPTZ NOT NULL,
    fetched_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    status            TEXT NOT NULL CHECK (status IN ('ok', 'failed')),
    -- TRUE when the window had ended before the fetch (nothing more can arrive).
    complete          BOOLEAN NOT NULL,
    ticks             INTEGER NOT NULL DEFAULT 0,
    stored            INTEGER NOT NULL DEFAULT 0,
    pages             INTEGER NOT NULL DEFAULT 0,
    first_tick        TIMESTAMPTZ,
    last_tick         TIMESTAMPTZ,
    -- How the time and price differences were read ('add' or 'subtract').
    delta_signs       TEXT[] NOT NULL DEFAULT '{}',
    error             TEXT,
    CHECK ((status = 'failed') = (error IS NOT NULL)),
    CHECK (window_from <= window_to)
);
CREATE TRIGGER history_fetches_append_only BEFORE UPDATE OR DELETE ON history_fetches
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX history_fetches_lookup ON history_fetches
    (account_id, broker_symbol_id, side, kind, window_from);

-- ------------------------------------------------------------------ deals

-- The account's deals exactly as the broker reports them (ProtoOADeal).
-- Money fields are the broker's integers; divide by 10^money_digits.
-- A re-read that differs is applied, and the old and new values are written
-- to reconcile_events first.
CREATE TABLE deals (
    account_id                   BIGINT NOT NULL,
    deal_id                      BIGINT NOT NULL,
    order_id                     BIGINT NOT NULL,
    position_id                  BIGINT NOT NULL,
    broker_symbol_id             BIGINT NOT NULL,
    volume                       BIGINT NOT NULL,
    filled_volume                BIGINT NOT NULL,
    trade_side                   TEXT NOT NULL,
    deal_status                  TEXT NOT NULL,
    created_at                   TIMESTAMPTZ NOT NULL,
    executed_at                  TIMESTAMPTZ NOT NULL,
    last_update_at               TIMESTAMPTZ,
    execution_price              DOUBLE PRECISION,
    margin_rate                  DOUBLE PRECISION,
    commission                   BIGINT,
    money_digits                 INTEGER,
    base_to_usd_rate             DOUBLE PRECISION,
    -- closePositionDetail, for a closing deal
    is_closing                   BOOLEAN NOT NULL,
    close_entry_price            DOUBLE PRECISION,
    close_gross_profit           BIGINT,
    close_swap                   BIGINT,
    close_commission             BIGINT,
    close_balance                BIGINT,
    close_quote_to_deposit_rate  DOUBLE PRECISION,
    close_closed_volume          BIGINT,
    close_balance_version        BIGINT,
    close_money_digits           INTEGER,
    close_pnl_conversion_fee     BIGINT,
    raw                          BYTEA NOT NULL,
    fetched_at                   TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (account_id, deal_id)
);
CREATE INDEX deals_account_symbol_time ON deals (account_id, broker_symbol_id, executed_at);
CREATE TRIGGER deals_no_delete BEFORE DELETE ON deals
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- One row per deal-list window. Append-only.
CREATE TABLE deal_fetches (
    id           BIGSERIAL PRIMARY KEY,
    account_id   BIGINT NOT NULL,
    window_from  TIMESTAMPTZ NOT NULL,
    window_to    TIMESTAMPTZ NOT NULL,
    fetched_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    status       TEXT NOT NULL CHECK (status IN ('ok', 'failed')),
    complete     BOOLEAN NOT NULL,
    deals        INTEGER NOT NULL DEFAULT 0,
    inserted     INTEGER NOT NULL DEFAULT 0,
    changed      INTEGER NOT NULL DEFAULT 0,
    error        TEXT,
    CHECK ((status = 'failed') = (error IS NOT NULL))
);
CREATE TRIGGER deal_fetches_append_only BEFORE UPDATE OR DELETE ON deal_fetches
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX deal_fetches_lookup ON deal_fetches (account_id, window_from);

-- What ProtoOATraderRes said about each account, each time it was read.
CREATE TABLE account_snapshots (
    id                      BIGSERIAL PRIMARY KEY,
    account_id              BIGINT NOT NULL,
    read_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    trader_login            BIGINT,
    deposit_asset_id        BIGINT NOT NULL,
    balance                 BIGINT NOT NULL,
    money_digits            INTEGER,
    registration_at         TIMESTAMPTZ,
    raw                     BYTEA NOT NULL
);
CREATE TRIGGER account_snapshots_append_only BEFORE UPDATE OR DELETE ON account_snapshots
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- ------------------------------------------------------------------ trials

-- Every research trial, pass or fail (docs/storage.md). Append-only, so the
-- final result is judged against the number of tries.
CREATE TABLE research_trials (
    trial_id          BIGSERIAL PRIMARY KEY,
    hypothesis        TEXT NOT NULL,
    rulebook_hash     TEXT,
    params            JSONB NOT NULL,
    data_range        JSONB NOT NULL,
    -- 'development' or 'held_out'; the held-out block is run once.
    sample            TEXT NOT NULL CHECK (sample IN ('development', 'held_out')),
    results_json      JSONB,
    verdict           TEXT,
    settings_version  BIGINT NOT NULL,
    run_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes             TEXT
);
CREATE TRIGGER research_trials_append_only BEFORE UPDATE OR DELETE ON research_trials
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- ------------------------------------------------------------------ settings

CREATE TEMP TABLE seed4 (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed4 VALUES
 (1, 'history.tick_days_back', 'integer', 1, 3650, '7',
  'Days of bid and ask ticks downloaded per resolved symbol, newest first, in whole broker weeks',
  'self-derived starting depth: storage is the limit (docs/storage.md, "Ticks history"); raise once measured'),
 (2, 'history.probe_years', 'integer', 0, 50, '15',
  'Years back the availability probe searches for the earliest week of ticks (0: no probe)',
  'self-derived; owner plan: "as far back as the broker allows"'),
 (3, 'history.interval_hours', 'integer', 1, 168, '6',
  'Hours between history syncs (deals, then ticks, then the probe)', 'self-derived'),
 (4, 'deals.window_days', 'integer', 1, 31, '7',
  'Days per deal-history request (a window that reports hasMore is split)', 'self-derived');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed4 ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT (SELECT version FROM settings_version) + ord, key, NULL, NULL, value, 'migration',
       'seeded default: ' || source
FROM seed4 ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, (SELECT version FROM settings_version) + ord FROM seed4;

UPDATE settings_version SET version = version + (SELECT count(*) FROM seed4);
