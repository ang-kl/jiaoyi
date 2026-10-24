-- Review of #4 (round 2): a probe step that fails (e.g. rate-limited) is not
-- an empty week. Each probe search is now recorded as a whole: complete with
-- its result, or incomplete with the reason and no result. Only a completed
-- search counts for the re-probe cooldown. The individual requests stay in
-- history_fetches (kind 'probe'), failed ones with their error.
CREATE TABLE history_probes (
    id                       BIGSERIAL PRIMARY KEY,
    account_id               BIGINT NOT NULL,
    broker_symbol_id         BIGINT NOT NULL,
    side                     TEXT NOT NULL CHECK (side IN ('bid', 'ask')),
    started_at               TIMESTAMPTZ NOT NULL,
    finished_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- 'complete': the earliest week with ticks was found;
    -- 'no_ticks': the newest complete week already held none;
    -- 'incomplete': a step failed; no result is recorded.
    status                   TEXT NOT NULL CHECK (status IN ('complete', 'no_ticks', 'incomplete')),
    searched_from            TIMESTAMPTZ NOT NULL,
    searched_to              TIMESTAMPTZ NOT NULL,
    steps                    INTEGER NOT NULL,
    earliest_week_with_ticks TIMESTAMPTZ,
    latest_empty_week_before TIMESTAMPTZ,
    error                    TEXT,
    CHECK ((status = 'complete') = (earliest_week_with_ticks IS NOT NULL)),
    CHECK ((status = 'incomplete') = (error IS NOT NULL))
);
CREATE TRIGGER history_probes_append_only BEFORE UPDATE OR DELETE ON history_probes
    FOR EACH ROW EXECUTE FUNCTION jiaoyi_refuse_change();
CREATE INDEX history_probes_lookup ON history_probes (account_id, broker_symbol_id, finished_at DESC);
