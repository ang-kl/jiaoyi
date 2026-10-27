-- Private whole-account report marks, measured by the existing history session.
-- `latest` is replaced; close:YYYY-MM-DD is the first actual observation within
-- 90 seconds after NY 17:00. It is not an invented historical account balance.
CREATE TABLE report_account_snapshots (
    account_id BIGINT NOT NULL,
    sample_key TEXT NOT NULL CHECK (sample_key = 'latest' OR sample_key ~ '^close:[0-9]{4}-[0-9]{2}-[0-9]{2}$'),
    observed_at TIMESTAMPTZ NOT NULL,
    detail JSONB NOT NULL CHECK (jsonb_typeof(detail) = 'object'),
    PRIMARY KEY (account_id, sample_key)
);
