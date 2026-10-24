-- Owner ruling, 24-10-2026: "use the broker's names for energy". Brent is
-- SpotBrent on the broker: the name appears in energy_candidates of load run 6
-- on account 43002148 (/phase1/report). One logged change of universe.symbols,
-- made the way the settings change path makes it: the version is bumped once
-- and the old and new values are written to settings_changes.
-- SpotBrent joins the energy class after its last member. No futures (-F) or
-- perpetual (-PERP) names are added. The demo account has not loaded yet, so
-- its energy names are not known and are not guessed.

WITH cur AS (
    SELECT value FROM settings WHERE key = 'universe.symbols' AND account_id IS NULL
), elems AS (
    SELECT t.e, t.ord::numeric AS ord
    FROM cur, jsonb_array_elements(cur.value) WITH ORDINALITY AS t(e, ord)
), last_energy AS (
    SELECT max(ord) AS ord FROM elems WHERE e->>'class' = 'energy'
), nv AS (
    SELECT jsonb_agg(x.e ORDER BY x.ord) AS value
    FROM (
        SELECT e, ord FROM elems
        UNION ALL
        SELECT jsonb_build_object('name', 'SpotBrent', 'class', 'energy'), last_energy.ord + 0.5
        FROM last_energy
    ) x
), v AS (
    UPDATE settings_version SET version = version + 1 RETURNING version
)
INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT v.version, 'universe.symbols', NULL, cur.value, nv.value, 'owner',
       'owner 24-10: use the broker''s names for energy; Brent is SpotBrent on the broker (energy_candidates, load run 6 on 43002148)'
FROM v, cur, nv;

UPDATE settings
SET value = (SELECT new_value FROM settings_changes
             WHERE key = 'universe.symbols' ORDER BY version DESC LIMIT 1),
    version = (SELECT version FROM settings_version),
    updated_at = now()
WHERE key = 'universe.symbols' AND account_id IS NULL;
