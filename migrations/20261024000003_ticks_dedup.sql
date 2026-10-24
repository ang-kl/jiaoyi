-- A natural key for recorded ticks, so a retried flush cannot insert a quote
-- twice (review finding on #3). Each recording session has its own id and
-- numbers its quotes in order; (account, session, seq, received_at) is unique.
-- received_at is part of the key because a unique index on a partitioned
-- table must include the partition key. Rows recorded before this migration
-- have no session or sequence and are left as they are.
ALTER TABLE ticks ADD COLUMN session_id TEXT;
ALTER TABLE ticks ADD COLUMN seq BIGINT;
CREATE UNIQUE INDEX ticks_session_seq ON ticks (account_id, session_id, seq, received_at);
