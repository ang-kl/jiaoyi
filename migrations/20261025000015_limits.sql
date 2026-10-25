-- The factory's limits and the owner's pause (owner, 25-10-2026: the pause
-- numbers "fine", № 17,105; then "only human can pause trade" and "always
-- live trade", № 17,115).
-- Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115)
--
-- The limits only ALERT: when a strategy on an account reaches one, /health
-- and the status page say "LIMIT REACHED: owner's decision needed" and a
-- decision is recorded. Nothing pauses on its own.
--
-- The pause keys are the owner's: no code writes them. They are changed
-- only through settings::change on the owner's word (changed_by 'owner',
-- the owner's words as the reason). A paused strategy takes no new entry on
-- that account; its open positions keep every exit.

CREATE TEMP TABLE seed15 (
    ord int, key text, kind text, min_value double precision, max_value double precision,
    value jsonb, description text, source text
) ON COMMIT DROP;

INSERT INTO seed15 VALUES
 (1, 'trading.pause_drawdown_r', 'number', 1, 100, '10',
  'Limit alert: a strategy on an account whose largest peak-to-trough fall in cumulative net R (closed trades) reaches this many R is flagged "LIMIT REACHED: owner''s decision needed". It alerts only; only the owner pauses',
  'owner 25-10-2026, № 17,105 ("pause numbers fine"); alert only, № 17,115 ("only human can pause trade")'),
 (2, 'trading.pause_pf_floor', 'number', 0, 5, '0.8',
  'Limit alert: after at least trading.pause_pf_min_trades closed trades, a profit factor (sum of winning R / sum of losing R) below this is flagged. It alerts only',
  'owner 25-10-2026, № 17,105; alert only, № 17,115'),
 (3, 'trading.pause_pf_min_trades', 'integer', 10, 1000, '50',
  'The closed trades a strategy needs on an account before its profit factor is judged against trading.pause_pf_floor',
  'owner 25-10-2026, № 17,105; alert only, № 17,115'),
 (4, 'trading.s1_paused', 'integer', 0, 1, '0',
  'The owner''s pause of Strategy 1 on an account: 1 = no new entry (open positions keep their exits). Changed only on the owner''s word; no code writes it',
  'owner 25-10-2026, № 17,115 ("only human can pause trade")'),
 (5, 'trading.s3_paused', 'integer', 0, 1, '0',
  'The owner''s pause of Strategy 3 on an account: 1 = no new entry (open positions keep their exits). Changed only on the owner''s word; no code writes it',
  'owner 25-10-2026, № 17,115 ("only human can pause trade")');

INSERT INTO setting_definitions (key, kind, min_value, max_value, description, source)
SELECT key, kind, min_value, max_value, description, source FROM seed15 ORDER BY ord;

INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
SELECT (SELECT version FROM settings_version) + ord, key, NULL, NULL, value, 'migration',
       'seeded default: ' || source
FROM seed15 ORDER BY ord;

INSERT INTO settings (key, account_id, value, version)
SELECT key, NULL, value, (SELECT version FROM settings_version) + ord FROM seed15;

UPDATE settings_version SET version = version + (SELECT count(*) FROM seed15);
