-- R1b: every research job attempt records the engine version it ran on
-- (jiaoyi::edge::ENGINE_VERSION). Attempts made before this migration ran
-- on engine 1 (the #9 merge, 9207eac1) and keep NULL: nothing is rewritten.
ALTER TABLE research_job_attempts ADD COLUMN engine_version INTEGER;
