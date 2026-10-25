-- Strategy 3, intraday momentum (owner-approved 25-10-2026 ~10:45 SGT; entry
-- "Tick-triggered only", ~10:50 SGT). Pre-registration:
-- docs/preregistration-2026-10-25-strategy3.md. Every row below carries that
-- file's SHA-256 and the label jiaoyi-s3.

-- 1. Each Strategy 3 order as decided, written with its intent, before the
--    frame: the quote at the decision, the band, σ, R, the cost reserve C
--    and its parts, and the plan (TP1, TP2, close volume). Append-only.
--    A broker_exit row stands for an order the broker executed itself (the
--    stop or the take-profit), one per deal, so its cost is reported too.
CREATE TABLE strategy3_orders (
    id            BIGSERIAL PRIMARY KEY,
    account_id    BIGINT NOT NULL,
    entry_key     TEXT NOT NULL UNIQUE,
    label         TEXT NOT NULL,
    prereg_hash   TEXT NOT NULL,
    kind          TEXT NOT NULL CHECK (kind IN ('open', 'tp1', 'session_end', 'broker_exit')),
    symbol        TEXT NOT NULL,
    symbol_id     BIGINT NOT NULL,
    position_id   BIGINT,
    side          TEXT CHECK (side IN ('buy', 'sell')),
    session_date  DATE,
    decided_at    TIMESTAMPTZ NOT NULL,
    bid           DOUBLE PRECISION,
    ask           DOUBLE PRECISION,
    volume        BIGINT,
    detail        JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER strategy3_orders_append_only BEFORE UPDATE OR DELETE ON strategy3_orders
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX strategy3_orders_account ON strategy3_orders (account_id, id DESC);
CREATE INDEX strategy3_orders_position ON strategy3_orders (account_id, position_id);

-- 2. Each order's answer: when it was sent and answered, the fill price
--    against the quote at the decision (slippage, positive = adverse), the
--    spread then, and the order-to-fill time. Append-only. The commission
--    comes from the broker's deals (table deals) by position.
CREATE TABLE strategy3_fills (
    id               BIGSERIAL PRIMARY KEY,
    account_id       BIGINT NOT NULL,
    entry_key        TEXT NOT NULL REFERENCES strategy3_orders (entry_key),
    label            TEXT NOT NULL,
    prereg_hash      TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('open', 'tp1', 'session_end', 'broker_exit')),
    symbol           TEXT NOT NULL,
    position_id      BIGINT,
    side             TEXT,
    outcome          TEXT NOT NULL,
    sent_at          TIMESTAMPTZ NOT NULL,
    answered_at      TIMESTAMPTZ,
    latency_ms       BIGINT,
    decision_price   DOUBLE PRECISION,
    fill_price       DOUBLE PRECISION,
    slippage         DOUBLE PRECISION,
    spread           DOUBLE PRECISION,
    volume           BIGINT,
    detail           JSONB NOT NULL DEFAULT '{}'::jsonb,
    recorded_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TRIGGER strategy3_fills_append_only BEFORE UPDATE OR DELETE ON strategy3_fills
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX strategy3_fills_account ON strategy3_fills (account_id, id DESC);

-- 3. The setting: Strategy 3's open risk at its stops, percent of fresh equity.
INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
VALUES ('trading.s3_open_risk_pct', 'number', 0, 100,
        'The most Strategy 3 may hold in open risk at its stops, percent of fresh equity',
        'owner 25-10-2026, Strategy 3 v2 approval ("Strategy 3 open risk <= trading.s3_open_risk_pct (3)")');

UPDATE settings_version SET version = version + 1;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT version, 'trading.s3_open_risk_pct', NULL, NULL, '3'::jsonb, 'migration',
       'seeded default: owner 25-10-2026, Strategy 3 v2 approval'
FROM settings_version;

INSERT INTO settings (key, account_id, value, version)
SELECT 'trading.s3_open_risk_pct', NULL, '3'::jsonb, version FROM settings_version;
