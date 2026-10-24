-- PR #6 (deploy 7765158a, 24-10-2026). Demo 49236051 was refused with
-- CH_ACCESS_TOKEN_INVALID on every account authorisation, even straight after
-- a refresh, while live 43002148 authorised with the same token; and every
-- refresh ended the live sessions (ProtoOAAccountsTokenInvalidatedEvent).
-- A refresh renews a grant but never adds an account to it, so the grant is
-- now read before an account is authorised.

-- 1. Every read of a token's grant (ProtoOAGetAccountListByAccessTokenReq),
--    made before authorising one account. The token itself is never stored
--    here: only the first 16 hex characters of the SHA-256 of the access token.
CREATE TABLE token_grants (
    id                 BIGSERIAL PRIMARY KEY,
    read_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- The account the read was made for, and the host it was made on.
    account_id         BIGINT NOT NULL,
    host               TEXT NOT NULL,
    token_fingerprint  TEXT NOT NULL CHECK (token_fingerprint ~ '^[0-9a-f]{16}$'),
    token_source       TEXT NOT NULL CHECK (token_source IN ('env_seed', 'refresh')),
    -- What the read says about that account:
    --   listed: in the grant, with an environment that agrees (or none given);
    --   not_listed: the grant omits it (a refresh cannot add it);
    --   environment_mismatch: in the grant, but isLive disagrees with the
    --     environment the account was declared with;
    --   rejected: the broker refused the read itself (expired or invalid token).
    verdict            TEXT NOT NULL
                       CHECK (verdict IN ('listed', 'not_listed', 'environment_mismatch', 'rejected')),
    reason             TEXT,
    permission_scope   TEXT,
    -- [{ctid, is_live, login}] as the broker listed them; NULL when rejected.
    accounts           JSONB,
    CHECK ((verdict = 'rejected') = (accounts IS NULL)),
    CHECK ((verdict = 'listed') = (reason IS NULL))
);
CREATE TRIGGER token_grants_append_only BEFORE UPDATE OR DELETE ON token_grants
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX token_grants_account ON token_grants (account_id, id DESC);

-- 2. The env seed the stored token descends from, by fingerprint. A NEW seed
--    (a different fingerprint) replaces the stored token; the same seed keeps
--    the stored, refreshed one. NULL for a row stored before this migration.
ALTER TABLE oauth_tokens ADD COLUMN seed_fingerprint TEXT
    CHECK (seed_fingerprint ~ '^[0-9a-f]{16}$');

-- 3. History windows cut short because the broker ended the account session
--    (a refresh elsewhere invalidated the token, the account disconnected or
--    the socket closed) are not broker failures of that week. 'broker': the
--    broker refused or failed the request; 'session_ended': the session ended
--    under it, and the next pass fetches the window again. NULL on a failed
--    row recorded before this migration: the two were not told apart.
ALTER TABLE history_fetches ADD COLUMN failure_kind TEXT
    CHECK (failure_kind IN ('broker', 'session_ended')),
    ADD CHECK (failure_kind IS NULL OR status = 'failed');
ALTER TABLE deal_fetches ADD COLUMN failure_kind TEXT
    CHECK (failure_kind IN ('broker', 'session_ended')),
    ADD CHECK (failure_kind IS NULL OR status = 'failed');

-- 4. Partition creation raises no notice. The deploy log carried
--    `relation "ticks_p2026102x" already exists, skipping` from CREATE TABLE
--    IF NOT EXISTS on every check. An existing partition is now found with
--    to_regclass before any CREATE; the advisory lock and the duplicate_table
--    (42P07) handler stay, for two creators racing.

CREATE OR REPLACE FUNCTION jiaoyi_ensure_tick_partition(day date) RETURNS text
LANGUAGE plpgsql AS $$
DECLARE
    name text := 'ticks_p' || to_char(day, 'YYYYMMDD');
BEGIN
    IF to_regclass(name) IS NOT NULL THEN
        RETURN name;
    END IF;
    PERFORM pg_advisory_xact_lock(hashtext('jiaoyi_partition'), hashtext(name));
    -- Created by another session while this one waited for the lock.
    IF to_regclass(name) IS NOT NULL THEN
        RETURN name;
    END IF;
    BEGIN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF ticks FOR VALUES FROM (%L) TO (%L)',
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
    IF to_regclass(name) IS NOT NULL THEN
        RETURN name;
    END IF;
    PERFORM pg_advisory_xact_lock(hashtext('jiaoyi_partition'), hashtext(name));
    IF to_regclass(name) IS NOT NULL THEN
        RETURN name;
    END IF;
    BEGIN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF ticks_history FOR VALUES FROM (%L) TO (%L)',
            name,
            (first::timestamp AT TIME ZONE 'UTC'),
            ((first + interval '1 month')::timestamp AT TIME ZONE 'UTC'));
    EXCEPTION WHEN duplicate_table THEN
        NULL;
    END;
    RETURN name;
END
$$;

-- 5. Seconds before a history sync whose session ended re-authorises and
--    resumes at the first unfinished window.

CREATE TEMP TABLE seed7 (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed7 VALUES
 (1, 'history.session_retry_secs', 'integer', 0, 3600, '5',
  'Seconds before a history sync whose account session ended re-authorises and resumes at the first unfinished window',
  'owner order, PR #6 item A (24-10-2026: "re-authorise ... resume from the first unfinished window"); value self-derived');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed7 ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT (SELECT version FROM settings_version) + ord, key, NULL, NULL, value, 'migration',
       'seeded default: ' || source
FROM seed7 ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, (SELECT version FROM settings_version) + ord FROM seed7;

UPDATE settings_version SET version = version + (SELECT count(*) FROM seed7);
