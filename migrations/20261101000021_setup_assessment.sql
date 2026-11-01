-- Private latest setup assessments and a bounded material-change ring.
-- One row/account; no per-second history or duplicate broker fill archive.
CREATE TABLE setup_assessment_snapshots (
    account_id BIGINT PRIMARY KEY,
    observed_at TIMESTAMPTZ NOT NULL,
    detail JSONB NOT NULL CHECK (jsonb_typeof(detail) = 'object')
);
