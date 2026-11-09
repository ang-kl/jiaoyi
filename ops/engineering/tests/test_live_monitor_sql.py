"""Focused monitor evidence checks: partial fills, money gaps, coverage and read-only IO."""
import copy
import datetime as dt
import json
import subprocess
import unittest
from unittest.mock import patch

from monitoring import sql_evidence as m

START = dt.datetime(2026, 11, 9, 0, tzinfo=dt.timezone.utc)
NOW = START + dt.timedelta(hours=2)


def deal(deal_id, closing, volume=100, **extra):
    return {'deal_id': deal_id, 'order_id': deal_id, 'symbol_id': 1, 'side': 'sell' if closing else 'buy',
            'filled_volume': volume, 'is_closing': closing, 'deal_status': 'FILLED',
            'executed_at': (START + dt.timedelta(hours=1 if closing else -1)).isoformat(),
            'money_digits': 2, 'close_money_digits': 2, 'close_gross_profit': 1000,
            'close_swap': -10, 'close_commission': 20, 'close_pnl_conversion_fee': 5, **extra}


def position():
    return {'owner_count': 1, 'position_id': 11, 'entry_key': 's3:test', 'symbol': 'BTCUSD',
            'symbol_id': 1, 'side': 'buy', 'requested_volume': 100,
            'deals': [deal(1, False), deal(2, True)],
            'receipts': [{'entry_key': 's3:test', 'intent_kind': 'open', 'deal_id': 1,
                          'order_id': 1, 'volume': 100}], 'exits': []}


class EvidenceTests(unittest.TestCase):
    def test_closed_cash_has_exact_fees_and_missing_zero_is_unknown(self):
        p = position()
        self.assertEqual(m._position(p)['net'], 9.65)
        p['deals'][1]['close_commission'] = None
        self.assertEqual(m._position(p)['state'], 'incomplete')
        self.assertIn('closing_costs_unverified', m._position(p)['reasons'])

    def test_partial_close_and_duplicate_receipts_are_not_extra_closed_trades(self):
        p = position()
        p['receipts'].append(copy.deepcopy(p['receipts'][0]))
        self.assertEqual(m._position(p)['state'], 'closed')
        p['deals'][1]['filled_volume'] = 50
        self.assertEqual(m._position(p)['state'], 'open')
        p['deals'][1]['filled_volume'] = 101
        self.assertEqual(m._position(p)['state'], 'incomplete')

    def test_unowned_increment_and_missing_exit_receipt_do_not_claim_complete(self):
        p = position()
        p['deals'][0]['order_id'] = 99
        p['deals'][0]['deal_id'] = 99
        self.assertEqual(m._position(p)['state'], 'incomplete')
        p = position()
        p['exits'] = [{'kind': 'broker_exit', 'entry_key': 's3-broker:43002148:11:90', 'volume': 100}]
        self.assertIn('broker_exit_missing_from_history', m._position(p)['reasons'])

    def test_history_union_requires_no_gap_and_fresh_through_cutoff(self):
        def fetch(a, b):
            return {'window_from': (START + dt.timedelta(seconds=a)).isoformat(),
                    'window_to': (START + dt.timedelta(seconds=b)).isoformat(),
                    'fetched_at': NOW.isoformat(), 'status': 'ok', 'has_error': False}
        self.assertTrue(m._history([fetch(-100, 3600), fetch(3600, 7190)], START, NOW)['history_complete'])
        self.assertFalse(m._history([fetch(-100, 3600), fetch(3601, 7190)], START, NOW)['history_complete'])
        self.assertFalse(m._history([fetch(-100, 6000)], START, NOW)['history_complete'])
        # SQL may discard ONLY intervals ending before start: overlapping
        # intervals beginning earlier still establish complete coverage.
        overlapping = [fetch(-600, 3600), fetch(3600, 7190)]
        historical = [fetch(-100000 - n * 100, -99900 - n * 100) for n in range(1000)]
        self.assertEqual(m._history(historical + overlapping, START, NOW),
                         m._history(overlapping, START, NOW))


    def test_psql_url_is_env_only_readonly_and_failure_text_never_leaks(self):
        url = 'postgresql://user:supersecret@localhost:5432/test_db'
        with patch.object(m.subprocess, 'run') as run, patch.object(m, 'assess', return_value={'status': 'complete'}):
            run.return_value = subprocess.CompletedProcess([], 0, '{"transaction_read_only":"on"}', '')
            self.assertEqual(m.collect(url, START, NOW)['status'], 'complete')
            args, kwargs = run.call_args
            self.assertNotIn(url, repr(args))
            self.assertNotIn('supersecret', repr(args))
            self.assertEqual(kwargs['env']['PGPASSWORD'], 'supersecret')
            self.assertIn('default_transaction_read_only=on', kwargs['env']['PGOPTIONS'])
            self.assertIn('REPEATABLE READ READ ONLY', kwargs['input'])
            self.assertIn('ROLLBACK', kwargs['input'])
            self.assertNotIn('PGSERVICE', kwargs['env'])
            run.return_value = subprocess.CompletedProcess([], 1, '', 'password=supersecret')
            self.assertNotIn('supersecret', json.dumps(m.collect(url, START, NOW)))
            self.assertEqual(m.collect(url, START, NOW)['reason'], 'readonly_query_failed')

    def test_timeout_and_invalid_scope_do_not_become_zero_metrics(self):
        with patch.object(m.subprocess, 'run', side_effect=subprocess.TimeoutExpired('psql', 3.5)):
            result = m.collect('postgresql://user:secret@localhost/db', START, NOW)
        self.assertEqual(result, m._unavailable('readonly_query_timeout'))
        self.assertEqual(m.collect('postgresql://user:secret@localhost/db', NOW, START)['reason'], 'invalid_monitor_window')
        self.assertEqual(m.collect_sql_evidence({}, deadline=0)['reason'], 'live_monitor_database_url_missing')



# Opt-in integration against the existing LOCAL development container only.
# No deployment URL is read, and each invocation drops its isolated fixture DB.
class LocalPostgresTests(unittest.TestCase):
    def test_local_fixture_sql_executes_and_readonly_blocks_insert(self):
        import os
        import uuid
        if os.environ.get('LIVE_MONITOR_LOCAL_SQL_TEST') != '1':
            self.skipTest('set LIVE_MONITOR_LOCAL_SQL_TEST=1 for isolated local PostgreSQL fixture')
        database = 'monitor18393_test_' + uuid.uuid4().hex[:12]
        def psql(sql, db=database, params=()):
            return subprocess.run(['docker', 'exec', '-i', 'jiaoyi-dev-postgres', 'sh', '-c',
                'db="$1"; shift; exec psql -X -q -A -t -U "$POSTGRES_USER" -d "$db" --set=ON_ERROR_STOP=1 "$@"',
                'sh', db, *params], input=sql, text=True, capture_output=True, timeout=15)
        setup = psql('CREATE DATABASE ' + database + ';', db='postgres')
        self.assertEqual(setup.returncode, 0, setup.stderr)
        try:
            schema = '''
CREATE TABLE accounts(account_id bigint,currency text);
CREATE TABLE trading_intents(id bigint,account_id bigint,entry_key text,client_msg_id text,kind text,
 symbol_id bigint,side text,volume bigint,created_at timestamptz);
CREATE TABLE strategy3_orders(account_id bigint,entry_key text,kind text,symbol text,
 prereg_hash text,decided_at timestamptz,detail jsonb);
CREATE TABLE trading_events(id bigint,account_id bigint,entry_key text,client_msg_id text,kind text,
 received_at timestamptz,raw bytea,volume bigint,position_id bigint,deal_id bigint,order_id bigint,
 error_code text,description text);
CREATE TABLE deals(account_id bigint,position_id bigint,deal_id bigint,order_id bigint,broker_symbol_id bigint,
 trade_side text,filled_volume bigint,deal_status text,is_closing boolean,executed_at timestamptz,fetched_at timestamptz,
 money_digits integer,close_money_digits integer,close_gross_profit bigint,close_swap bigint,
 close_commission bigint,close_pnl_conversion_fee bigint);
CREATE TABLE strategy3_fills(id bigint,account_id bigint,position_id bigint,entry_key text,
 kind text,outcome text,volume bigint,sent_at timestamptz,answered_at timestamptz);
CREATE TABLE trading_decisions(id bigint,account_id bigint,symbol text,kind text,at timestamptz,reason text,detail jsonb);
CREATE TABLE deal_fetches(account_id bigint,window_from timestamptz,window_to timestamptz,fetched_at timestamptz,status text,error text);
CREATE TABLE report_account_snapshots(account_id bigint,sample_key text,observed_at timestamptz,detail jsonb);
INSERT INTO accounts VALUES(43002148,'USD');
INSERT INTO trading_intents VALUES(1,43002148,'s3:test','client','open',1,'buy',100,'2026-11-09 00:10Z');
INSERT INTO strategy3_orders VALUES(43002148,'s3:test','open','BTCUSD','hash','2026-11-09 00:10Z','{"production_policy":"s3_macro_pullback_owner_18392_v1"}');
INSERT INTO trading_events VALUES(1,43002148,'s3:test','client','ORDER_FILLED','2026-11-09 00:11Z','',100,11,1,1,null,null);
INSERT INTO deals VALUES(43002148,11,1,1,1,'BUY',100,'FILLED',false,'2026-11-09 00:11Z','2026-11-09 01:59Z',2,null,null,null,null,null),
(43002148,11,2,2,1,'SELL',100,'FILLED',true,'2026-11-09 01:00Z','2026-11-09 01:59Z',2,2,1000,-10,20,5);
INSERT INTO deal_fetches VALUES(43002148,'2026-11-08 00:00Z','2026-11-09 01:59Z','2026-11-09 01:59Z','ok',null);
-- Historical receipts can be large in production; they do not prove any
-- part of this monitor window and must not inflate the returned evidence.
INSERT INTO deal_fetches
SELECT 43002148, '2026-01-01'::timestamptz+n*interval '1 hour',
 '2026-01-01'::timestamptz+(n+1)*interval '1 hour', '2026-11-09 01:59Z', 'ok', null
FROM generate_series(1,1000) n;

'''
            result = psql(schema)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = psql(m.SQL_PATH.read_text(), params=('--set=since=2026-11-09T00:00:00Z',
                          '--set=until=2026-11-09T02:00:00Z', '--set=timeout_ms=3000ms'))
            self.assertEqual(result.returncode, 0, result.stderr)
            raw = json.loads(result.stdout)
            self.assertEqual(raw['transaction_read_only'], 'on')
            self.assertEqual(len(raw['attempts']), 1)
            self.assertEqual(len(raw['deal_fetches']), 1)
            self.assertLess(m._clock(raw['deal_fetches'][0]['window_from']), START)
            self.assertEqual(m._clock(raw['deal_fetches'][0]['window_to']), NOW-dt.timedelta(minutes=1))

            self.assertEqual(m._position(raw['positions'][0])['net'], 9.65)
            data = m.assess(raw, START, NOW)
            metrics = data['accounts']['43002148']['metrics']
            self.assertEqual(metrics['new_entries'], 1)
            self.assertEqual(metrics['closed_trades'], 1)
            self.assertEqual(metrics['wins'], 1)
            self.assertEqual(metrics['net_realized_pnl'], 9.65)
            self.assertIsNone(metrics['refused_entries'])
            self.assertIsNone(metrics['net_unrealized_pnl'])
            refused = psql("BEGIN READ ONLY; INSERT INTO accounts VALUES(9,'USD'); ROLLBACK;")
            self.assertNotEqual(refused.returncode, 0)
            self.assertIn('read-only', refused.stderr)
        finally:
            dropped = psql('DROP DATABASE ' + database + ';', db='postgres')
            self.assertEqual(dropped.returncode, 0, dropped.stderr)

if __name__ == '__main__':
    unittest.main()
