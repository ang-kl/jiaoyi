-- Hot read-path indexes for position outcome qualification, Strategy 3 pre-open
-- receipt lookups, and per-account tick freshness checks (No.18571).

CREATE INDEX IF NOT EXISTS deals_account_position
    ON deals (account_id, position_id);

CREATE INDEX IF NOT EXISTS strategy3_fills_account_position
    ON strategy3_fills (account_id, position_id);

CREATE INDEX IF NOT EXISTS strategy1_fills_account_position
    ON strategy1_fills (account_id, position_id);

CREATE INDEX IF NOT EXISTS trading_intents_open_prefix
    ON trading_intents (account_id, kind, entry_key text_pattern_ops);

CREATE INDEX IF NOT EXISTS trading_decisions_s3_preopen_lookup
    ON trading_decisions (
        account_id,
        symbol,
        (detail->'session'->>'open'),
        (detail->>'prereg_hash'),
        id DESC
    )
    WHERE kind = 's3_preopen';

CREATE INDEX IF NOT EXISTS ticks_account_received_at
    ON ticks (account_id, received_at DESC);
