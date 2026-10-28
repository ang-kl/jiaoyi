-- Owner, № 17,596, 28-10-2026: drop names the cost gate always refuses.
-- US500, USDCAD and EURUSD were skipped because C >= R. They spend
-- decisions and never enter. Removing them from the universe is a settings
-- change on the owner's word. It does not change the Strategy 3 registration.

WITH cur AS (
    SELECT value FROM settings WHERE key = 'universe.symbols' AND account_id IS NULL
), nv AS (
    SELECT jsonb_agg(t.e ORDER BY t.ord) AS value
    FROM cur, jsonb_array_elements(cur.value) WITH ORDINALITY AS t(e, ord)
    WHERE t.e->>'name' NOT IN ('US500', 'USDCAD', 'EURUSD')
), v AS (
    UPDATE settings_version SET version = version + 1 RETURNING version
)
INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT v.version, 'universe.symbols', NULL, cur.value, nv.value, 'owner',
       'owner № 17,596 28-10-2026: drop US500, USDCAD, EURUSD; cost gate always refuses them'
FROM v, cur, nv;

UPDATE settings
SET value = (SELECT new_value FROM settings_changes
             WHERE key = 'universe.symbols' ORDER BY version DESC LIMIT 1),
    version = (SELECT version FROM settings_version),
    updated_at = now()
WHERE key = 'universe.symbols' AND account_id IS NULL;
