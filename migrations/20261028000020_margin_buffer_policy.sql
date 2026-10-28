-- Owner, 28-10-2026, chat №17,666: margin buffer for new entries.
-- Register the policy field; activation value still requires the owner's
-- percentage. No balance, pause, size, stop or basket setting is changed.
INSERT INTO setting_definitions(key,kind,min_value,max_value,description,source)
VALUES ('trading.margin_level_floor_pct','number',100,100000,
 'Minimum projected post-entry margin level, percent. Entry also requires remaining equity after stop reserves above broker margin warnings. Exits bypass this check.',
 'owner 28-10-2026: use a margin buffer for new entries; keep one-position live rule');
