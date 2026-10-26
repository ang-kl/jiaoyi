-- Delivery receipts only; no trading settings, intents or broker mutations.
CREATE TABLE telegram_reports (
    report_key TEXT PRIMARY KEY,
    period_from TIMESTAMPTZ NOT NULL,
    period_to TIMESTAMPTZ NOT NULL,
    html TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('sending', 'sent', 'retry', 'uncertain')),
    attempts INTEGER NOT NULL DEFAULT 1,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at TIMESTAMPTZ,
    message_id BIGINT,
    last_error TEXT,
    CHECK (period_from < period_to),
    CHECK ((status = 'sent') = (sent_at IS NOT NULL AND message_id IS NOT NULL))
);
