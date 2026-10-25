-- The gated order path and Strategy 1, the tick-triggered trend (owner's
-- naming, № 16,806; H1 is its pre-registration label). Owner, 25-10-2026:
-- "I want live to execute trading by 1 PM today", "Keep the gate; demo
-- trades at 1 PM", and for live "Allow the minimum volume".
--
-- There is no on/off switch (owner, 24-10-2026: "why do i need
-- JIAOYI_TRADING_ENABLED, removed it. it should not be doing so"). Every
-- permitted demo account trades Strategy 1 as soon as the service runs; a
-- live account sends nothing until the edge gate passes.

-- 1. What the gate wrote before each frame. Append-only. The unique entry
--    key is what stops an intent being sent twice, after a restart included.
CREATE TABLE trading_intents (
    id                BIGSERIAL PRIMARY KEY,
    account_id        BIGINT NOT NULL,
    entry_key         TEXT NOT NULL UNIQUE,
    client_msg_id     TEXT NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('open', 'amend_stop', 'close')),
    symbol_id         BIGINT,
    position_id       BIGINT,
    side              TEXT CHECK (side IN ('buy', 'sell')),
    volume            BIGINT,            -- cents of units
    relative_stop     BIGINT,            -- 1/100000 of a price
    stop_price        DOUBLE PRECISION,
    take_profit       DOUBLE PRECISION,  -- an amend's, carried from the position (an amend replaces it)
    risk              DOUBLE PRECISION,  -- deposit currency, recomputed by the gate
    max_risk          DOUBLE PRECISION NOT NULL,
    settings_version  BIGINT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER trading_intents_append_only BEFORE UPDATE OR DELETE ON trading_intents
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX trading_intents_account ON trading_intents (account_id, id DESC);

-- 2. Every execution message received: answers to our requests and the
--    broker's own (a stop hit). Append-only, with the raw bytes.
CREATE TABLE trading_events (
    id               BIGSERIAL PRIMARY KEY,
    account_id       BIGINT NOT NULL,
    entry_key        TEXT,
    client_msg_id    TEXT,
    payload_type     INTEGER NOT NULL,
    kind             TEXT NOT NULL,
    position_id      BIGINT,
    order_id         BIGINT,
    deal_id          BIGINT,
    symbol_id        BIGINT,
    side             TEXT,
    price            DOUBLE PRECISION,
    volume           BIGINT,
    stop_loss        DOUBLE PRECISION,
    label            TEXT,
    error_code       TEXT,
    description      TEXT,
    is_server_event  BOOLEAN,
    raw              BYTEA NOT NULL,
    received_at      TIMESTAMPTZ NOT NULL
);
CREATE TRIGGER trading_events_append_only BEFORE UPDATE OR DELETE ON trading_events
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX trading_events_position ON trading_events (account_id, position_id);
-- The gate and the trader both receive every answer to an order (one
-- broadcast message, one receive time); this keeps it recorded once.
-- Events without a request id (the broker's own) never collide here.
CREATE UNIQUE INDEX trading_events_once
    ON trading_events (account_id, client_msg_id, kind, received_at);

-- 3. The trader's decisions (a signal, a skip and why, a refusal, a send,
--    a reconcile). Recorded when they change, not on every tick. Append-only.
CREATE TABLE trading_decisions (
    id                BIGSERIAL PRIMARY KEY,
    account_id        BIGINT NOT NULL,
    at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    symbol            TEXT,
    kind              TEXT NOT NULL,
    reason            TEXT NOT NULL,
    detail            JSONB NOT NULL DEFAULT '{}'::jsonb,
    settings_version  BIGINT
);
CREATE TRIGGER trading_decisions_append_only BEFORE UPDATE OR DELETE ON trading_decisions
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX trading_decisions_account ON trading_decisions (account_id, id DESC);

-- 4. Reviewed settings changes from config/settings-changes.json, each
--    applied once through settings::change (bounds-checked, logged).
CREATE TABLE settings_seed_applied (
    seed_id     TEXT PRIMARY KEY,
    version     BIGINT NOT NULL,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5. Settings.
CREATE TEMP TABLE seed12 (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed12 VALUES
 (1, 'trading.live_requires_edge_pass', 'integer', 0, 1, '1',
  '1: a live account sends nothing until the Strategy 1 (H1) held-out trial under the running pre-registration hash has verdict PASS. Demo accounts do not wait',
  'owner 24-10-2026 ("live trading waits for a demonstrated edge"), kept 25-10 ("Keep the gate; demo trades at 1 PM")'),
 (2, 'trading.risk_pct', 'number', 0, 5, '0.6',
  'Percent of fresh equity (balance + unrealized P&L, broker-stated) risked per trade at the initial stop',
  'rulebook §5; pre-registration 24-10-2026 (H1 size)'),
 (3, 'trading.cap_factor', 'number', 1, 2, '1.1',
  'The gate refuses an open whose recomputed risk exceeds cap_factor × risk_pct × equity',
  'rulebook §5 (hard size cap at send: 1.1)'),
 (4, 'trading.live_allow_min_volume', 'integer', 0, 1, '1',
  '1: on a live account, when risk_pct sizing is below the broker minimum, take exactly the minimum if its risk is within min_volume_max_risk_pct; demo accounts always skip',
  'owner 25-10-2026 ("Allow the minimum volume", live only)'),
 (5, 'trading.min_volume_max_risk_pct', 'number', 0, 100, '10',
  'The most a minimum-volume trade may risk, percent of fresh equity',
  'owner 25-10-2026: at $38.98 this admits ETHUSD''s $2.85 (7.3%) and nothing else'),
 (6, 'trading.trend_open_risk_pct', 'number', 0, 100, '3.0',
  'The most Strategy 1 may hold in open risk at its stops, percent of fresh equity: half of a 6% account limit; the other half is for Strategy 2 (stat arb, not built)',
  'owner, № 16,806 (strategy 1 holds at most half of each account''s open-risk limit)'),
 (7, 'trading.reconcile_secs', 'integer', 30, 3600, '300',
  'Seconds between reconciles with the broker''s open positions (the broker wins)',
  'PR B plan (reconcile at start and periodically); value self-derived'),
 (8, 'trading.trail_amend_secs', 'integer', 1, 3600, '60',
  'At most one trailing-stop amend per position in this many seconds',
  'PR B plan (trail amend at most every 60 s)'),
 (9, 'trading.order_wait_secs', 'integer', 1, 120, '20',
  'Seconds the gate waits for the broker''s answer before an order is Unconfirmed (never resent)',
  'value self-derived'),
 (10, 'trading.retry_secs', 'integer', 5, 3600, '60',
  'Seconds before a trading session that stopped on an error connects again',
  'value self-derived');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed12 ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT (SELECT version FROM settings_version) + ord, key, NULL, NULL, value, 'migration',
       'seeded default: ' || source
FROM seed12 ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, (SELECT version FROM settings_version) + ord FROM seed12;

UPDATE settings_version SET version = version + (SELECT count(*) FROM seed12);
