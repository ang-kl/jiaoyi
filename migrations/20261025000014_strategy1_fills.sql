-- Strategy 1 records its orders' answers as Strategy 3 does (owner approved
-- the instrument plan, 25-10-2026, after "show your instrument design").
-- Claude · № 17,103 25-Oct (ordered: owner approved the instrument plan, "show your instrument design")
--
-- Each Strategy 1 order's answer: the open, a trailing-stop amend, or the
-- broker's own exit (the trailed stop, a stop-out or a close made outside
-- this program), once per order. The fill against the quote at the decision
-- (slippage, positive = adverse), the spread then, and the order-to-answer
-- time. Prices, volumes and milliseconds only: no money. Append-only.
-- The pre-registration hash is the 24-10 file's (H1, sha256 9105c5eb...).
CREATE TABLE strategy1_fills (
    id               BIGSERIAL PRIMARY KEY,
    account_id       BIGINT NOT NULL,
    -- An open's or an amend's intent key (trading_intents), or a broker
    -- exit's own key (s1-broker:<account>:<position>:<deal>).
    entry_key        TEXT NOT NULL UNIQUE,
    label            TEXT NOT NULL,
    prereg_hash      TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('open', 'trail', 'broker_exit')),
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
CREATE TRIGGER strategy1_fills_append_only BEFORE UPDATE OR DELETE ON strategy1_fills
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX strategy1_fills_account ON strategy1_fills (account_id, id DESC);

-- `GET /trading/orders` reads each order's decision by its entry key.
CREATE INDEX trading_decisions_entry_key ON trading_decisions ((detail->>'entry_key'))
    WHERE detail ? 'entry_key';
-- Each order's events by its entry key.
CREATE INDEX trading_events_entry_key ON trading_events (entry_key, id);
-- Strategy 3's fills by key, for the order list.
CREATE INDEX strategy3_fills_entry_key ON strategy3_fills (entry_key);
