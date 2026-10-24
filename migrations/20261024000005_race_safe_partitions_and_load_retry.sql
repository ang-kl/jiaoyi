-- Fixes from the first Railway deploy (452da09a) and its live report, 24-10-2026.

-- 1. Partition creation is race-safe. Two recorders created ticks_p20261024
--    at the same moment and one got "relation already exists". Creation now
--    takes a transaction-scoped advisory lock keyed on the partition name,
--    uses IF NOT EXISTS, and treats SQLSTATE 42P07 (duplicate_table) as done.

CREATE OR REPLACE FUNCTION jiaoyi_ensure_tick_partition(day date) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
    name text := 'ticks_p' || to_char(day, 'YYYYMMDD');
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('jiaoyi_partition'), hashtext(name));
    BEGIN
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF ticks FOR VALUES FROM (%L) TO (%L)',
            name,
            (day::timestamp AT TIME ZONE 'UTC'),
            ((day + 1)::timestamp AT TIME ZONE 'UTC'));
    EXCEPTION WHEN duplicate_table THEN
        NULL;
    END;
    RETURN name;
END
$$;

CREATE OR REPLACE FUNCTION jiaoyi_ensure_history_partition(month date) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
    first date := date_trunc('month', month)::date;
    name text := 'ticks_history_p' || to_char(first, 'YYYYMM');
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('jiaoyi_partition'), hashtext(name));
    BEGIN
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF ticks_history FOR VALUES FROM (%L) TO (%L)',
            name,
            (first::timestamp AT TIME ZONE 'UTC'),
            ((first + interval '1 month')::timestamp AT TIME ZONE 'UTC'));
    EXCEPTION WHEN duplicate_table THEN
        NULL;
    END;
    RETURN name;
END
$$;

-- 2. A failed load is retried soon, not after load.interval_hours.
-- 3. A spread from too few quotes is not a measurement (spread.min_samples),
--    and each measurement records whether the market was open.

ALTER TABLE symbol_costs ADD COLUMN market_open BOOLEAN;
ALTER TABLE symbol_costs ADD COLUMN market_state_reason TEXT;

CREATE TEMP TABLE seed5 (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed5 VALUES
 (1, 'load.retry_minutes', 'integer', 1, 1440, '5',
  'Minutes before a failed or partial load of an account is tried again',
  'owner deploy review, 24-10-2026 ("default 5 min")'),
 (2, 'load.auth_retry_attempts', 'integer', 0, 10, '2',
  'Immediate re-tries of a load that failed on authorisation after the token was refreshed',
  'owner deploy review, 24-10-2026 ("bounded retry count"); count self-derived'),
 (3, 'load.auth_retry_secs', 'integer', 0, 600, '5',
  'Seconds before the first immediate re-try; each further re-try waits twice as long',
  'owner deploy review, 24-10-2026 ("backoff"); value self-derived'),
 (4, 'spread.min_samples', 'integer', 1, 100000, '30',
  'Fewest quote updates for a median spread to count as measured; fewer leaves the verdict undecided',
  'owner deploy review, 24-10-2026 (one stale weekend quote judged "keep"); value self-derived');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed5 ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT (SELECT version FROM settings_version) + ord, key, NULL, NULL, value, 'migration',
       'seeded default: ' || source
FROM seed5 ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, (SELECT version FROM settings_version) + ord FROM seed5;

UPDATE settings_version SET version = version + (SELECT count(*) FROM seed5);

-- 4. Owner ruling, 24-10-2026: "use the broker's names for energy, like
--    NatGas, etc." One logged change of universe.symbols, made the way the
--    settings change path makes it: the version is bumped and the old and new
--    values are written to settings_changes.
--    NATGAS -> NatGas (the broker's near-name on 43002148);
--    USOIL  -> SpotCrude (the WTI name bot-trade traded on this broker);
--    UKOIL  removed: its broker name is not known and is not guessed.

-- The broker's symbol list as each load read it (id, name, enabled), so the
-- report can show candidate names. NULL for loads made before this migration.
ALTER TABLE load_runs ADD COLUMN listed_symbols JSONB;

WITH cur AS (
    SELECT value FROM settings WHERE key = 'universe.symbols' AND account_id IS NULL
), nv AS (
    SELECT jsonb_agg(
               CASE t.e->>'name'
                   WHEN 'NATGAS' THEN jsonb_build_object('name', 'NatGas', 'class', t.e->>'class')
                   WHEN 'USOIL' THEN jsonb_build_object('name', 'SpotCrude', 'class', t.e->>'class')
                   ELSE t.e
               END
               ORDER BY t.ord) AS value
    FROM cur, jsonb_array_elements(cur.value) WITH ORDINALITY AS t(e, ord)
    WHERE t.e->>'name' <> 'UKOIL'
), v AS (
    UPDATE settings_version SET version = version + 1 RETURNING version
)
INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT v.version, 'universe.symbols', NULL, cur.value, nv.value, 'owner',
       'owner 24-10: use the broker''s names for energy'
FROM v, cur, nv;

UPDATE settings
SET value = (SELECT new_value FROM settings_changes
             WHERE key = 'universe.symbols' ORDER BY version DESC LIMIT 1),
    version = (SELECT version FROM settings_version),
    updated_at = now()
WHERE key = 'universe.symbols' AND account_id IS NULL;
