-- Test-only account universe; production settings writes remain in settings.rs.
INSERT INTO settings(key, account_id, value, version)
VALUES ('universe.symbols', $1, $2, 1)
