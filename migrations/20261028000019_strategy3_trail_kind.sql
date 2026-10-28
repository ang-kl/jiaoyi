-- The v5 Chandelier trail writes kind 'trail'. The 0013 check allowed
-- only open, tp1, session_end and broker_exit, so record_order failed
-- after the intent was stored and no amend frame was sent.
-- Grok 4.7 · № 17,655, 28-10-2026. This does not change a registration,
-- a stop distance, or a size. It lets the registered trail be stored.

ALTER TABLE strategy3_orders DROP CONSTRAINT strategy3_orders_kind_check;
ALTER TABLE strategy3_orders ADD CONSTRAINT strategy3_orders_kind_check
    CHECK (kind IN ('open', 'tp1', 'session_end', 'broker_exit', 'trail'));

ALTER TABLE strategy3_fills DROP CONSTRAINT strategy3_fills_kind_check;
ALTER TABLE strategy3_fills ADD CONSTRAINT strategy3_fills_kind_check
    CHECK (kind IN ('open', 'tp1', 'session_end', 'broker_exit', 'trail'));
