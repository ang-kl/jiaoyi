-- Settings (owner, 24-10-2026: "no hard code settings, the UI should have
-- setting to tweak. And traceability is important.") and the tick record.

-- ------------------------------------------------------------------ settings

-- What each setting is, its bounds, and where its default comes from.
CREATE TABLE setting_definitions (
    key          TEXT PRIMARY KEY,
    kind         TEXT NOT NULL CHECK (kind IN ('integer', 'number', 'symbol_list')),
    min_value    DOUBLE PRECISION,
    max_value    DOUBLE PRECISION,
    description  TEXT NOT NULL,
    source       TEXT NOT NULL,          -- where the seeded default comes from
    CHECK (min_value IS NULL OR max_value IS NULL OR min_value <= max_value)
);

-- Current values. account_id NULL is the global value; an account row overrides it.
CREATE TABLE settings (
    key          TEXT NOT NULL REFERENCES setting_definitions (key),
    account_id   BIGINT,
    value        JSONB NOT NULL,
    version      BIGINT NOT NULL,        -- the settings version that set this value
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX settings_key_scope ON settings (key, COALESCE(account_id, 0));

-- The settings version: bumped by every change, in the same transaction.
CREATE TABLE settings_version (
    singleton  BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    version    BIGINT NOT NULL
);

-- Every change: who, when, why, old and new value. Append-only.
CREATE TABLE settings_changes (
    id          BIGSERIAL PRIMARY KEY,
    version     BIGINT NOT NULL UNIQUE,
    key         TEXT NOT NULL REFERENCES setting_definitions (key),
    account_id  BIGINT,
    old_value   JSONB,                   -- NULL: there was no value at this scope
    new_value   JSONB,                   -- NULL: the account override was removed
    changed_by  TEXT NOT NULL CHECK (length(btrim(changed_by)) > 0),
    reason      TEXT NOT NULL CHECK (length(btrim(reason)) > 0),
    changed_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER settings_changes_append_only BEFORE UPDATE OR DELETE ON settings_changes
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- Seed the defaults. Each seed is a logged change by 'migration'.
CREATE TEMP TABLE seed (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed VALUES
 (1, 'universe.symbols', 'symbol_list', NULL, NULL,
  '[{"name":"US500","class":"equity_index"},{"name":"NAS100","class":"equity_index"},
    {"name":"US30","class":"equity_index"},{"name":"GER40","class":"equity_index"},
    {"name":"UK100","class":"equity_index"},{"name":"JPN225","class":"equity_index"},
    {"name":"AUS200","class":"equity_index"},
    {"name":"EURUSD","class":"fx"},{"name":"GBPUSD","class":"fx"},{"name":"USDJPY","class":"fx"},
    {"name":"AUDUSD","class":"fx"},{"name":"USDCAD","class":"fx"},{"name":"NZDUSD","class":"fx"},
    {"name":"XAUUSD","class":"metal"},{"name":"XAGUSD","class":"metal"},
    {"name":"USOIL","class":"energy"},{"name":"UKOIL","class":"energy"},{"name":"NATGAS","class":"energy"},
    {"name":"BTCUSD","class":"crypto"},{"name":"ETHUSD","class":"crypto"}]',
  'Candidate symbols: exact broker names (never aliased) and asset class',
  'rulebook §3'),
 (2, 'indicators.atr_length', 'integer', 2, 500, '20',
  'ATR length in daily bars, Wilder smoothing', 'rulebook §5'),
 (3, 'verdict.min_history_years', 'number', 0, 100, '10',
  'Minimum years of daily history at the broker (all classes but crypto)', 'rulebook §3'),
 (4, 'verdict.min_history_years_crypto', 'number', 0, 100, '6',
  'Minimum years of daily history at the broker for crypto', 'rulebook §3'),
 (5, 'verdict.max_spread_to_atr', 'number', 0, 10, '0.10',
  'A symbol is dropped when its median spread exceeds this fraction of one daily ATR', 'rulebook §3'),
 (6, 'spread.sample_secs', 'integer', 5, 3600, '60',
  'Seconds of spot quotes sampled per load to measure the median spread', 'owner phase-1 brief, 24-10-2026'),
 (7, 'load.interval_hours', 'integer', 1, 168, '24',
  'Hours between data loads', 'owner phase-1 brief, 24-10-2026 ("repeat once a day")'),
 (8, 'crosscheck.symbols_per_class', 'integer', 0, 20, '3',
  'Symbols per asset class re-read from the broker after each load and compared with the stored bars',
  'rulebook §9, phase 1 gate'),
 (9, 'crosscheck.days', 'integer', 2, 366, '30',
  'Days of recent bars re-read for the cross-check', 'self-derived'),
 (10, 'ticks.retention_days', 'integer', 1, 3650, '30',
  'Days of recorded quotes kept; older daily partitions are dropped', 'owner, 24-10-2026'),
 (11, 'ticks.flush_millis', 'integer', 100, 60000, '1000',
  'Milliseconds between batched writes of recorded quotes', 'self-derived'),
 (12, 'bars.tick.updates_per_bar', 'integer', 1, 1000000, '100',
  'Tick bars: quote updates per bar (N)', 'self-derived starting value, not from research'),
 (13, 'bars.imbalance.initial_expected_ticks', 'number', 1, 1000000, '100',
  'Tick-imbalance bars: E0[T] before the first bar closes', 'self-derived starting value'),
 (14, 'bars.imbalance.initial_expected_imbalance', 'number', 0.0001, 1, '0.1',
  'Tick-imbalance bars: |2P[b=1]-1| before the first bar closes', 'self-derived starting value'),
 (15, 'bars.imbalance.ewma_span_bars', 'number', 1, 10000, '20',
  'Tick-imbalance bars: EWMA span, in bars, for E0[T] and 2P[b=1]-1', 'self-derived starting value'),
 (16, 'bars.imbalance.min_expected_imbalance', 'number', 0.0001, 1, '0.05',
  'Tick-imbalance bars: floor on |2P[b=1]-1| in the threshold (stops a zero threshold)', 'self-derived guard'),
 (17, 'bars.imbalance.max_ticks_per_bar', 'integer', 1, 10000000, '10000',
  'Tick-imbalance bars: a bar closes at this many updates whatever its imbalance', 'self-derived guard');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT ord, key, NULL, NULL, value, 'migration', 'seeded default: ' || source FROM seed ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, ord FROM seed;

INSERT INTO settings_version (version) SELECT max(ord) FROM seed;

-- Which settings version each load used.
ALTER TABLE load_runs ADD COLUMN settings_version BIGINT;

-- ------------------------------------------------------------------ ticks

-- The live spot quote stream, as received. Prices are in cTrader's relative
-- unit (1/100000 of a price), exactly as sent. A side is NULL when the event
-- did not carry it: the proxy sends only the side that changed.
-- broker_timestamp is ProtoOASpotEvent.timestamp as sent ("The Unix time for
-- spot"; its unit is not stated by Spotware and is checked against received_at).
CREATE TABLE ticks (
    account_id        BIGINT NOT NULL,
    broker_symbol_id  BIGINT NOT NULL,
    received_at       TIMESTAMPTZ NOT NULL,
    broker_timestamp  BIGINT,
    bid_e5            BIGINT,
    ask_e5            BIGINT,
    CHECK (bid_e5 IS NOT NULL OR ask_e5 IS NOT NULL)
) PARTITION BY RANGE (received_at);
CREATE INDEX ticks_account_symbol_time ON ticks (account_id, broker_symbol_id, received_at);

-- Create the UTC-day partition holding `day` if it does not exist.
CREATE FUNCTION jiaoyi_ensure_tick_partition(day date) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
    name text := 'ticks_p' || to_char(day, 'YYYYMMDD');
BEGIN
    IF to_regclass(name) IS NULL THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF ticks FOR VALUES FROM (%L) TO (%L)',
            name,
            (day::timestamp AT TIME ZONE 'UTC'),
            ((day + 1)::timestamp AT TIME ZONE 'UTC'));
    END IF;
    RETURN name;
END
$$;

-- Drop whole-day partitions that ended before `today - keep_days`.
-- This is the only deletion of recorded data, and it is the retention setting.
CREATE FUNCTION jiaoyi_drop_tick_partitions(today date, keep_days integer) RETURNS SETOF text
LANGUAGE plpgsql AS $$
DECLARE
    part record;
    day date;
BEGIN
    IF keep_days < 1 THEN
        RAISE EXCEPTION 'keep_days must be at least 1, got %', keep_days;
    END IF;
    FOR part IN
        SELECT c.relname FROM pg_inherits i
        JOIN pg_class c ON c.oid = i.inhrelid
        JOIN pg_class p ON p.oid = i.inhparent
        WHERE p.relname = 'ticks' AND c.relname ~ '^ticks_p[0-9]{8}$'
    LOOP
        day := to_date(substr(part.relname, 8), 'YYYYMMDD');
        IF day + 1 <= today - keep_days THEN
            EXECUTE format('DROP TABLE %I', part.relname);
            RETURN NEXT part.relname;
        END IF;
    END LOOP;
END
$$;
