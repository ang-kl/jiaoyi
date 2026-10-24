-- R1 of the edge test: the research engine for the pre-registered H1 jobs.
-- The rules are docs/preregistration-2026-10-24.md, sha256
-- 9105c5eb812ff6b3d267627a53e64d924018c799753d70e598830381a909098f, approved
-- by the owner on 24-10-2026. Every job and trial records that hash; the
-- runner refuses a job whose hash is not the one compiled into the service.

-- 1. Jobs. The trial cap of the pre-registration: the default plus the 3×3
--    grid per hypothesis, nothing else. Jobs are inserted here and only here.
CREATE TABLE research_jobs (
    id            BIGSERIAL PRIMARY KEY,
    hypothesis    TEXT NOT NULL CHECK (hypothesis IN ('H1', 'H2')),
    stage         TEXT NOT NULL CHECK (stage IN ('development', 'held_out')),
    setting       JSONB NOT NULL,
    -- The order the runner takes jobs in (the default first).
    run_order     INTEGER NOT NULL UNIQUE,
    prereg_hash   TEXT NOT NULL CHECK (prereg_hash ~ '^[0-9a-f]{64}$'),
    status        TEXT NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'running', 'done', 'refused', 'failed')),
    reason        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at    TIMESTAMPTZ,
    finished_at   TIMESTAMPTZ,
    UNIQUE (hypothesis, stage, setting, prereg_hash),
    CHECK ((status = 'pending') = (started_at IS NULL)),
    CHECK ((status IN ('done', 'refused', 'failed')) = (finished_at IS NOT NULL)),
    CHECK (status NOT IN ('refused', 'failed') OR reason IS NOT NULL)
);

-- Every status a job takes, in order. Append-only.
CREATE TABLE research_job_events (
    id       BIGSERIAL PRIMARY KEY,
    job_id   BIGINT NOT NULL REFERENCES research_jobs (id),
    at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    status   TEXT NOT NULL,
    reason   TEXT
);
CREATE TRIGGER research_job_events_append_only BEFORE UPDATE OR DELETE ON research_job_events
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- Every attempt at a job: when it started and the deployed commit
-- (RAILWAY_GIT_COMMIT_SHA, or 'unknown'). Append-only. A job interrupted by a
-- stop produced no result and nothing partial is stored, so it is attempted
-- again (owner's decision, 24-10-2026); its trial lists every attempt.
CREATE TABLE research_job_attempts (
    id          BIGSERIAL PRIMARY KEY,
    job_id      BIGINT NOT NULL REFERENCES research_jobs (id),
    attempt     INTEGER NOT NULL CHECK (attempt >= 1),
    started_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    commit      TEXT NOT NULL,
    UNIQUE (job_id, attempt)
);
CREATE TRIGGER research_job_attempts_append_only BEFORE UPDATE OR DELETE ON research_job_attempts
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();

-- A job is never deleted and its definition never changes. Its status moves
-- pending → running → done | refused | failed; running → pending only when an
-- attempt was interrupted before writing anything. A finished job never runs
-- again.
CREATE FUNCTION jiaoyi_research_job_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'research_jobs is append-only: a job is never deleted';
    END IF;
    IF NEW.id <> OLD.id OR NEW.hypothesis <> OLD.hypothesis OR NEW.stage <> OLD.stage
       OR NEW.setting <> OLD.setting OR NEW.run_order <> OLD.run_order
       OR NEW.prereg_hash <> OLD.prereg_hash OR NEW.created_at <> OLD.created_at THEN
        RAISE EXCEPTION 'research_jobs: a job''s definition never changes';
    END IF;
    IF NOT ((OLD.status = 'pending' AND NEW.status = 'running')
            OR (OLD.status = 'running' AND NEW.status IN ('done', 'refused', 'failed', 'pending'))) THEN
        RAISE EXCEPTION 'research_jobs: status % -> % is not allowed (a finished job never runs again)',
            OLD.status, NEW.status;
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER research_jobs_guard BEFORE UPDATE OR DELETE ON research_jobs
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_research_job_guard();

CREATE FUNCTION jiaoyi_research_job_event() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO research_job_events (job_id, status, reason) VALUES (NEW.id, NEW.status, NEW.reason);
    RETURN NEW;
END
$$;
CREATE TRIGGER research_jobs_event AFTER INSERT OR UPDATE ON research_jobs
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_research_job_event();

-- 2. Trials carry the pre-registration hash and the job they answer; a job
--    has at most one trial.
ALTER TABLE research_trials
    ADD COLUMN prereg_hash TEXT CHECK (prereg_hash ~ '^[0-9a-f]{64}$'),
    ADD COLUMN job_id BIGINT UNIQUE REFERENCES research_jobs (id);

-- 3. Spread samples: the median ask − bid of one full week per calendar year
--    per symbol (the second full week of June), streamed from the broker's
--    tick history and never stored raw. Development years only: the schema
--    refuses a week ending after the held-out seal too. Each row also holds
--    the bar-basis observations of that week (are the stored daily bars bid,
--    ask or mid prices?). Append-only; a failed sample may be taken again.
CREATE TABLE research_spreads (
    id                BIGSERIAL PRIMARY KEY,
    account_id        BIGINT NOT NULL,
    broker_symbol_id  BIGINT NOT NULL,
    symbol            TEXT NOT NULL,
    year              INTEGER NOT NULL,
    week_from         TIMESTAMPTZ NOT NULL,
    week_to           TIMESTAMPTZ NOT NULL,
    status            TEXT NOT NULL CHECK (status IN ('measured', 'no_ticks', 'failed')),
    -- Median ask − bid over the week's quote updates with both sides known, price units.
    median_spread     NUMERIC,
    samples           BIGINT NOT NULL DEFAULT 0,
    bid_ticks         BIGINT NOT NULL DEFAULT 0,
    ask_ticks         BIGINT NOT NULL DEFAULT 0,
    -- Wall time of the download, for the measured tick rate.
    download_ms       BIGINT NOT NULL DEFAULT 0,
    -- {"bid": n, "ask": n, "mid": n, "observations": n, "uninformative": n}
    basis             JSONB,
    error             TEXT,
    prereg_hash       TEXT NOT NULL CHECK (prereg_hash ~ '^[0-9a-f]{64}$'),
    measured_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((status = 'measured') = (median_spread IS NOT NULL)),
    CHECK ((status = 'failed') = (error IS NOT NULL)),
    CHECK (week_from < week_to),
    CHECK (week_to <= TIMESTAMPTZ '2025-10-24 00:00:00+00')
);
CREATE TRIGGER research_spreads_append_only BEFORE UPDATE OR DELETE ON research_spreads
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE UNIQUE INDEX research_spreads_once ON research_spreads (account_id, broker_symbol_id, year)
    WHERE status <> 'failed';

-- 4. The H1 jobs: the 9 development cells, breakout {20, 50, 100} × stop
--    {2, 3, 4} × ATR, the default (50, 3) first; then the one held-out job at
--    the default, refused unless the development gate is met under the same
--    hash. Prices: live 43002148.
INSERT INTO research_jobs (hypothesis, stage, setting, run_order, prereg_hash)
SELECT 'H1', stage, jsonb_build_object('breakout', b, 'stop_atr', k, 'account_id', 43002148,
                                       'default', b = 50 AND k = 3),
       ord, '9105c5eb812ff6b3d267627a53e64d924018c799753d70e598830381a909098f'
FROM (VALUES
    ('development', 50, 3, 1),
    ('development', 20, 2, 2),
    ('development', 20, 3, 3),
    ('development', 20, 4, 4),
    ('development', 50, 2, 5),
    ('development', 50, 4, 6),
    ('development', 100, 2, 7),
    ('development', 100, 3, 8),
    ('development', 100, 4, 9),
    ('held_out', 50, 3, 10)
) AS g (stage, b, k, ord);
