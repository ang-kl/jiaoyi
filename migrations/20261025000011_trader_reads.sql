-- Demo accounts by variable (owner, 25-10-2026): the broker's own account
-- facts, every time history sync step 1 reads the trader record
-- (ProtoOATraderRes). The owner stated 1:200 for 46130058 and 1:25 for
-- 47790949; this table is what the broker says. Append-only. A field the
-- broker omits is NULL, never a default (proto2 optional fields).
-- Enumerations are kept as the broker's integers. Money (balance, bonuses)
-- is in this table only: no route shows it.
CREATE TABLE trader_reads (
    id                              BIGSERIAL PRIMARY KEY,
    account_id                      BIGINT NOT NULL,
    read_at                         TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- ProtoOATrader.leverageInCents: 1:50 is 5000.
    leverage_in_cents               BIGINT,
    max_leverage                    BIGINT,
    balance                         BIGINT NOT NULL,
    balance_version                 BIGINT,
    money_digits                    INTEGER,
    deposit_asset_id                BIGINT NOT NULL,
    -- ProtoOAAccountType: 0 HEDGED, 1 NETTED, 2 SPREAD_BETTING.
    account_type                    INTEGER,
    is_limited_risk                 BOOLEAN,
    limited_risk_margin_calculation_strategy INTEGER,
    total_margin_calculation_type   INTEGER,
    access_rights                   INTEGER,
    swap_free                       BOOLEAN,
    fair_stop_out                   BOOLEAN,
    stop_out_strategy               INTEGER,
    manager_bonus                   BIGINT,
    ib_bonus                        BIGINT,
    non_withdrawable_bonus          BIGINT,
    trader_login                    BIGINT,
    broker_name                     TEXT,
    registration_at                 TIMESTAMPTZ
);
CREATE TRIGGER trader_reads_append_only BEFORE UPDATE OR DELETE ON trader_reads
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX trader_reads_latest ON trader_reads (account_id, id DESC);
