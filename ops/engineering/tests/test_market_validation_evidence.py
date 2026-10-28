import copy
import datetime as dt
import importlib.util
import json
from pathlib import Path
import subprocess
import types
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('validation_evidence', Path(__file__).resolve().parents[1] / 'market_validation_evidence.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)


class MarketEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.now = dt.datetime(2026, 10, 29, 1, tzinfo=dt.timezone.utc)
        self.since = self.now - dt.timedelta(hours=3)
        self.data = {'accounts': [{'account_id': 43002148, 'observed_at': '2026-10-29T00:59:50+00:00',
            'snapshot': {'observed_at': '2026-10-29T00:59:50Z', 'source_complete': True,
                'balance_stable': True, 'balance': 100, 'float': -5, 'equity': 95}}],
            'statement_report': {'status': 'sent', 'html': '<style>CSSNOISE</style><p>Balance100 &lt;tag&gt;</p>',
                'html_chars': 57}, 'lifecycle': []}

    def test_snapshot_freshness_and_unknown_float_are_not_zero(self):
        data = copy.deepcopy(self.data)
        out = m._assess(data, self.since, self.now)
        row = next(x for x in out['accounts'] if x['account_id'] == 43002148)
        self.assertTrue(row['source_complete'], 'equivalentZ/offset clocks must match')
        self.assertEqual((row['balance'], row['floating'], row['equity']), (100, -5, 95))
        for change in ('incomplete', 'stale'):
            data = copy.deepcopy(self.data)
            if change == 'incomplete': data['accounts'][0]['snapshot']['source_complete'] = False
            else: data['accounts'][0]['observed_at'] = '2026-10-29T00:40:00Z'
            row = m._assess(data, self.since, self.now)['accounts'][0]
            self.assertIsNone(row['floating']); self.assertIsNone(row['equity'])
            self.assertEqual(row['balance'], 100 if change == 'incomplete' else None)

    def test_missing_costs_and_ambiguous_ownership_cannot_become_pass_or_reverse(self):
        data = copy.deepcopy(self.data)
        data['lifecycle'] = [{'owner_count': 2, 'risk': 1, 'max_risk': 2,
            'opening_receipt_volume': 100, 'money': {'all_closing_costs_explicit': False,
            'known_closing_cash_net': 20, 'valid': True, 'opening_volume': 100, 'closing_volume': 100}}]
        out = m._assess(data, self.since, self.now); position = out['lifecycle'][0]
        self.assertEqual(position['ownership_state'], 'UNKNOWN')
        self.assertEqual(position['entry_risk_check'], 'UNKNOWN')
        self.assertIsNone(position['money']['known_closing_cash_net'])
        self.assertIsNone(position['position_net_r'])
        self.assertEqual(position['reverse_state'], 'UNKNOWN')
        document = m.render(out)
        self.assertNotIn('CSSNOISE', document)
        self.assertIn('&lt;tag&gt;', document)
        self.assertNotIn('<tag>', document)
        data = copy.deepcopy(self.data)
        data['lifecycle'] = [{'owner_count': 1, 'money': {'valid': False, 'all_closing_costs_explicit': True,
            'known_closing_cash_net': 20}, 'fills': [{'kind': 'open', 'fill_price': 123.45,
            'slippage': .01, 'latency_ms': 28, 'outcome': 'filled'}]}]
        out = m._assess(data, self.since, self.now)
        self.assertIsNone(out['lifecycle'][0]['money']['known_closing_cash_net'])
        self.assertIn('actual fill 123.45', m.render(out))

    def test_psql_uses_readonly_transaction_and_no_connection_secret_in_arguments_or_errors(self):
        coordinator = types.SimpleNamespace(config=types.SimpleNamespace(env={
            'ENGINEERING_VALIDATION_DATABASE_URL': 'postgres://SECRET-PASSWORD@example/test'}))
        done = subprocess.CompletedProcess([], 0, json.dumps(self.data), '')
        with patch.object(m.subprocess, 'run', return_value=done) as run:
            result = m.collect(coordinator, self.since, self.now)
        self.assertEqual(result['state'], 'available')
        args, kwargs = run.call_args
        self.assertNotIn('SECRET-PASSWORD', str(args))
        self.assertIn('BEGIN READ ONLY;', kwargs['input'])
        self.assertIn('ROLLBACK;', kwargs['input'])
        self.assertIn('default_transaction_read_only=on', kwargs['env']['PGOPTIONS'])
        self.assertEqual(kwargs['timeout'], 10)
        with patch.object(m.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, [], stderr='SECRET-PASSWORD')):
            failed = m.collect(coordinator, self.since, self.now)
        self.assertEqual(failed['state'], 'unknown')
        self.assertNotIn('SECRET-PASSWORD', json.dumps(failed))

    def test_saved_setup_arithmetic_is_assessed_without_claiming_registration_or_edge(self):
        position = {'ownership_state': 'PASS', 'strategy': 's3', 'side': 'buy', 'relative_stop': 100000,
            'prereg_hash': m.MIDPOINT_REGISTRATION,
            'setup': {'band': {'open': 100, 'sigma': .01, 'half_width': 1, 'lower': 99, 'upper': 101},
                'bid': 101.5, 'ask': 101.7, 'entry': {'side': 'buy', 'entry_price': 101.7, 'relative_stop': 100000},
                'order': {'plan': {'broker_target': 104}}}}
        checked = m._setup_checks(position)
        self.assertEqual(checked['registration'], 'UNKNOWN')
        self.assertTrue(all(checked[key] == 'PASS' for key in ('band', 'direction', 'trigger', 'original_sl', 'original_tp')))
        # A historical side-quote rule can enter on ask while its midpoint is
        # inside the band. It must not fail a later midpoint registration.
        position['prereg_hash'] = 'a' * 64
        position['setup']['bid'] = 100.5; position['setup']['ask'] = 101.1
        self.assertEqual(m._setup_checks(position)['trigger'], 'UNKNOWN')
        position['prereg_hash'] = m.MIDPOINT_REGISTRATION
        self.assertEqual(m._setup_checks(position)['trigger'], 'FAIL')
        position['setup']['bid'] = 100; position['setup']['ask'] = 100.2
        position['setup']['band']['half_width'] = 2
        position['setup']['entry']['relative_stop'] = 90000
        position['setup']['order']['plan']['broker_target'] = 100
        checked = m._setup_checks(position)
        self.assertTrue(all(checked[key] == 'FAIL' for key in ('band', 'trigger', 'original_sl', 'original_tp')))


if __name__ == '__main__': unittest.main()
