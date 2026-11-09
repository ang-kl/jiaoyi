BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;
SET LOCAL statement_timeout = :'timeout_ms';
SET LOCAL lock_timeout = '300ms';
WITH bounds AS (
 SELECT :'since'::timestamptz AS since, :'until'::timestamptz AS until
), scope(account_id) AS (VALUES (43002148::bigint),(46130058),(47790949),(49236051)),
core(symbol) AS (VALUES ('SpotCrude'),('SpotBrent'),('US500'),('NAS100'),('XAUUSD'),('XAGUSD'),('BTCUSD'),('ETHUSD')),
opens AS MATERIALIZED (
 SELECT i.id,i.account_id,i.entry_key,i.client_msg_id,i.symbol_id,i.side,i.volume,i.created_at,
 o.decided_at,o.symbol,o.prereg_hash,o.detail->>'production_policy' AS production_policy
 FROM trading_intents i JOIN strategy3_orders o USING(account_id,entry_key)
 JOIN scope a USING(account_id) JOIN core c ON c.symbol=o.symbol, bounds b
 WHERE i.kind='open' AND o.kind='open' AND i.created_at < b.until
), cohort AS (
 SELECT o.*, (SELECT min(e.received_at) FROM trading_events e
  WHERE e.account_id=o.account_id AND e.entry_key=o.entry_key AND e.volume>0
  AND e.position_id IS NOT NULL AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
  AND e.received_at<b.until) AS first_fill_at,
 (SELECT COALESCE(jsonb_agg(jsonb_build_object('id',e.id,'client_msg_id',e.client_msg_id,'kind',e.kind,'received_at',e.received_at,
 'raw_hex',CASE WHEN e.kind IN ('ENTRY_VALIDATED_BEFORE_SEND','REFUSED_BEFORE_SEND') THEN encode(e.raw,'hex') END,
 'volume',e.volume,'position_id',e.position_id,'deal_id',e.deal_id,'error_code',e.error_code,'description',e.description)
 ORDER BY e.received_at,e.id),'[]'::jsonb) FROM trading_events e
 WHERE e.account_id=o.account_id AND e.entry_key=o.entry_key AND e.received_at<b.until) AS events
 FROM opens o,bounds b WHERE o.created_at>=b.since OR EXISTS
 (SELECT 1 FROM trading_events e WHERE e.account_id=o.account_id AND e.entry_key=o.entry_key
 AND e.received_at>=b.since AND e.received_at<b.until)
), owned AS MATERIALIZED (
 SELECT o.account_id,e.position_id,count(DISTINCT o.entry_key) AS owner_count,min(o.entry_key) AS entry_key,
 min(o.symbol) AS symbol,min(o.symbol_id) AS symbol_id,min(o.side) AS side,min(o.volume) AS requested_volume,
 min(e.received_at) AS first_open_receipt_at
 FROM opens o JOIN trading_events e USING(account_id,entry_key),bounds b
 WHERE e.position_id IS NOT NULL AND e.volume>0 AND e.received_at<b.until
 AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 GROUP BY o.account_id,e.position_id
), positions AS (
 SELECT o.*,
 (SELECT COALESCE(jsonb_agg(jsonb_build_object('deal_id',d.deal_id,'order_id',d.order_id,
 'symbol_id',d.broker_symbol_id,'side',d.trade_side,'filled_volume',d.filled_volume,'deal_status',d.deal_status,
 'is_closing',d.is_closing,'executed_at',d.executed_at,'money_digits',d.money_digits,
 'close_money_digits',d.close_money_digits,'close_gross_profit',d.close_gross_profit,
 'close_swap',d.close_swap,'close_commission',d.close_commission,'close_pnl_conversion_fee',d.close_pnl_conversion_fee)
 ORDER BY d.executed_at,d.deal_id),'[]'::jsonb) FROM deals d
 WHERE d.account_id=o.account_id AND d.position_id=o.position_id AND d.filled_volume>0
 AND d.executed_at<b.until AND d.fetched_at<=b.until) AS deals,
 (SELECT COALESCE(jsonb_agg(jsonb_build_object('entry_key',e.entry_key,'kind',e.kind,'intent_kind',i.kind,
 'position_id',e.position_id,'deal_id',e.deal_id,'order_id',e.order_id,'volume',e.volume,'received_at',e.received_at)
 ORDER BY e.id),'[]'::jsonb) FROM trading_events e LEFT JOIN trading_intents i USING(account_id,entry_key)
 WHERE e.account_id=o.account_id AND e.position_id=o.position_id AND e.volume>0 AND e.received_at<b.until
 AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')) AS receipts,
 (SELECT COALESCE(jsonb_agg(jsonb_build_object('entry_key',f.entry_key,'kind',f.kind,'volume',f.volume,
 'at',COALESCE(f.answered_at,f.sent_at)) ORDER BY f.id),'[]'::jsonb) FROM strategy3_fills f
 WHERE f.account_id=o.account_id AND f.position_id=o.position_id AND f.kind IN ('broker_exit','tp1','session_end')
 AND f.outcome IN ('filled','partly_filled') AND f.volume>0 AND COALESCE(f.answered_at,f.sent_at)<b.until) AS exits
 FROM owned o,bounds b WHERE EXISTS(SELECT 1 FROM deals d WHERE d.account_id=o.account_id
 AND d.position_id=o.position_id AND d.is_closing AND d.executed_at>=b.since AND d.executed_at<b.until)
 OR EXISTS(SELECT 1 FROM strategy3_fills f WHERE f.account_id=o.account_id AND f.position_id=o.position_id
 AND f.kind IN ('broker_exit','tp1','session_end') AND COALESCE(f.answered_at,f.sent_at)>=b.since
 AND COALESCE(f.answered_at,f.sent_at)<b.until)
 OR EXISTS(SELECT 1 FROM trading_events e LEFT JOIN trading_intents i USING(account_id,entry_key)
 WHERE e.account_id=o.account_id AND e.position_id=o.position_id AND e.volume>0
 AND e.received_at>=b.since AND e.received_at<b.until
 AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 AND (i.kind='close' OR e.entry_key IS NULL))
), closing_cash AS (
 SELECT d.account_id,d.position_id,d.deal_id,d.deal_status,d.executed_at,d.money_digits,d.close_money_digits,
 d.close_gross_profit,d.close_swap,d.close_commission,d.close_pnl_conversion_fee,
 o.owner_count,o.symbol
 FROM deals d JOIN scope a USING(account_id) LEFT JOIN owned o USING(account_id,position_id),bounds b
 WHERE d.is_closing AND d.filled_volume>0 AND d.executed_at>=b.since AND d.executed_at<b.until AND d.fetched_at<=b.until
), latest_states AS (
 SELECT DISTINCT ON(d.account_id,d.symbol,d.kind) d.account_id,d.symbol,d.kind,d.at,d.reason,
 jsonb_build_object('session',d.detail->'session','open',d.detail->'open','open_at',d.detail->'open_at',
 'close_at',d.detail->'close_at','macro_context',d.detail->'macro_context','state',d.detail->'state',
 'detail',d.detail->'detail','band',d.detail->'band','production_policy',d.detail->'production_policy',
 'prereg_hash',d.detail->'prereg_hash') AS detail
 FROM trading_decisions d JOIN scope a USING(account_id) JOIN core c USING(symbol),bounds b
 WHERE d.at<b.until AND d.at>=b.until-interval '2 days'
 AND d.kind IN ('s3_session','s3_pullback_state','s3_pullback_assessment','s3_refused','s3_skip','s3_sent')
 ORDER BY d.account_id,d.symbol,d.kind,d.id DESC
), decisions AS (
 SELECT d.account_id,d.kind,d.reason,count(*) AS recorded_decisions
 FROM trading_decisions d JOIN scope a USING(account_id) JOIN core c USING(symbol),bounds b
 WHERE d.at>=b.since AND d.at<b.until AND d.kind LIKE 's3_%'
 GROUP BY d.account_id,d.kind,d.reason
), goals AS (
 SELECT DISTINCT ON(d.account_id) d.account_id,d.at,d.detail
 FROM trading_decisions d JOIN scope a USING(account_id),bounds b
 WHERE d.kind='s3_supervisor_report' AND d.at<b.until ORDER BY d.account_id,d.id DESC
), fetches AS (
 SELECT d.account_id,d.window_from,d.window_to,d.fetched_at,d.status,d.error IS NOT NULL AS has_error
 FROM deal_fetches d JOIN scope a USING(account_id),bounds b
 -- A receipt ending before monitor start cannot contribute to continuous
 -- [start,through] coverage. Keep crossing intervals, not an arbitrary LIMIT.
 WHERE d.fetched_at<=b.until AND d.window_from<=d.window_to AND d.window_to>=b.since
 AND d.window_from<=b.until
), marks AS (
 SELECT r.account_id,r.observed_at,jsonb_build_object('observed_at',r.detail->'observed_at',
 'source_complete',r.detail->'source_complete','balance_stable',r.detail->'balance_stable','pnl_coverage_complete',r.detail->'pnl_coverage_complete',
 'equity_basis',r.detail->'equity_basis','currency',r.detail->'currency',
 'balance',r.detail->'balance','float',r.detail->'float','equity',r.detail->'equity',
 'failure_reasons',r.detail->'failure_reasons') AS detail
 FROM report_account_snapshots r JOIN scope a USING(account_id),bounds b
 WHERE r.sample_key='latest' AND r.observed_at<=b.until
)
SELECT jsonb_build_object('window_start',b.since,'observed_at',b.until,
 'transaction_read_only',current_setting('transaction_read_only'),
 'accounts',COALESCE((SELECT jsonb_agg(jsonb_build_object('account_id',s.account_id,'currency',a.currency))
 FROM scope s LEFT JOIN accounts a USING(account_id)),'[]'::jsonb),
 'attempts',COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY c.account_id,c.created_at,c.entry_key) FROM cohort c),'[]'::jsonb),
 'positions',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY p.account_id,p.position_id) FROM positions p),'[]'::jsonb),
 'closing_cash',COALESCE((SELECT jsonb_agg(to_jsonb(c) ORDER BY c.account_id,c.executed_at,c.deal_id) FROM closing_cash c),'[]'::jsonb),
 'states',COALESCE((SELECT jsonb_agg(to_jsonb(s)) FROM latest_states s),'[]'::jsonb),
 'decisions',COALESCE((SELECT jsonb_agg(to_jsonb(d)) FROM decisions d),'[]'::jsonb),
 'goals',COALESCE((SELECT jsonb_agg(to_jsonb(g)) FROM goals g),'[]'::jsonb),
 'deal_fetches',COALESCE((SELECT jsonb_agg(to_jsonb(f)) FROM fetches f),'[]'::jsonb),
 'account_snapshots',COALESCE((SELECT jsonb_agg(to_jsonb(m)) FROM marks m),'[]'::jsonb))
FROM bounds b;
ROLLBACK;
