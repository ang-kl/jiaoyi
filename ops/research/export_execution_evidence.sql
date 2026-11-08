-- Read-only optional exporter for the RUNNING service database, not a local test DB.
-- psql -XAtq --set=ON_ERROR_STOP=1 --set=from_utc='...' --set=to_utc='...' \
--   "$JIAOYI_EVIDENCE_DATABASE_URL" -f ops/research/export_execution_evidence.sql
-- Never use a credential in command text. Output is private evidence, not a log.
-- Dates are required, at most 31 days; 2000 rows plus an explicit truncation flag.
BEGIN READ ONLY;
SET LOCAL statement_timeout = '10s';
SET LOCAL lock_timeout = '2s';
WITH bounds AS (
    SELECT :'from_utc'::timestamptz AS first_at, :'to_utc'::timestamptz AS last_at
), selected AS (
    SELECT e.id, e.account_id, e.entry_key, e.client_msg_id, e.payload_type,
           e.kind, e.position_id, e.order_id, e.deal_id, e.symbol_id,
           e.side, e.price, e.volume, e.stop_loss, e.received_at,
           encode(e.raw, 'hex') AS raw_protobuf_or_dispatch_json_hex
    FROM trading_events e CROSS JOIN bounds b
    WHERE e.account_id IN (49236051, 46130058, 47790949, 43002148)
      AND (e.payload_type = 2126 OR e.kind = 'ENTRY_VALIDATED_BEFORE_SEND')
      AND b.first_at <= b.last_at AND b.last_at-b.first_at <= interval '31 days'
      AND e.received_at >= b.first_at AND e.received_at <= b.last_at
    ORDER BY e.received_at, e.id LIMIT 2001
), limited AS (
    SELECT * FROM selected ORDER BY received_at, id LIMIT 2000
)
SELECT jsonb_build_object(
    'schema', 'jiaoyi.private-execution-evidence.v1',
    'exported_at_utc', current_timestamp,
    'window_valid', (SELECT first_at<=last_at AND last_at-first_at<=interval '31 days' FROM bounds),
    'has_more', (SELECT count(*)>2000 FROM selected),
    'rows', COALESCE((SELECT jsonb_agg(to_jsonb(limited) ORDER BY received_at,id) FROM limited), '[]'::jsonb),
    'qualification', 'Recorded events only; absent exit dispatch/restart/shadow receipts remain absent'
);
ROLLBACK;
