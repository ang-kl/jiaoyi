import copy
import unittest

import run_repaired_offline_replay as p


class RepairedOfflineTests(unittest.TestCase):
    def model(self, gross=0.0, exit_price=100.0):
        entry = dict(side='long', fill=100.0, initial_r=1.0, volume=100)
        outcome = dict(remaining_volume=0, scenario_gross_r=gross,
                       fills=[dict(price=exit_price, volume=100)], trail_applied=1)
        costs = {b: gross-(100.0+exit_price)*float(b)/10000 for b in p.BPS}
        return dict(study=dict(trade=dict(entry=entry, exit_comparison=dict(candidate=outcome, baseline=copy.deepcopy(outcome))), marks=[]),
                    sensitivities=dict(trailing=costs, fixed=copy.deepcopy(costs)))

    def test_closed_nonentries_and_gross_flat_are_not_fabricated_net_winners(self):
        rows = [dict(status='complete', models={'s3_two_stage': self.model()}),
                dict(status='complete', models={'s3_two_stage': dict(status='missing_required_class_peer')}),
                dict(status='failed', models={})]
        s = p.score(rows, 's3_two_stage', '24')
        self.assertEqual((s['episodes'], s['entries'], s['closed_trades'], s['wins'], s['losses'], s['net_breakevens'], s['gross_flat_exits'], s['nonentries']),
                         (3, 1, 1, 0, 1, 0, 1, 2))
        self.assertAlmostEqual(s['mean_net_r'], -0.48)
        empty = p.score(rows[1:], 's3_two_stage', '24')
        self.assertIsNone(empty['win_rate'])
        self.assertIsNone(empty['profit_factor'])

    def test_independent_fill_accounting_detects_false_net_costs_or_censored_returns(self):
        models = {m: self.model() for m in p.NEW_MODELS}
        self.assertEqual(p.verify_native_accounting(models), 24)
        models[p.NEW_MODELS[0]]['sensitivities']['trailing']['24'] = 0.0
        with self.assertRaisesRegex(ValueError, 'net cost'):
            p.verify_native_accounting(models)
        models = {m: self.model() for m in p.NEW_MODELS}
        models[p.NEW_MODELS[0]]['study']['trade']['exit_comparison']['candidate']['remaining_volume'] = 100
        with self.assertRaisesRegex(ValueError, 'censored'):
            p.verify_native_accounting(models)

    def test_ci_and_parity_fail_closed_without_changing_numbers(self):
        checks = [dict(name=n, status='COMPLETED', conclusion='SUCCESS') for n in p.REQUIRED]
        self.assertEqual(p.ci_state(checks), 'success')
        for missing in [[], checks[:2], [dict(name=n, state=None) for n in p.REQUIRED], checks+[checks[0]]]:
            self.assertEqual(p.ci_state(missing), 'pending')
        self.assertEqual(p.ci_state(checks[:2]+[dict(checks[2], conclusion='FAILURE')]), 'failed')
        old = {'study': {'notes': ['old explanatory text'], 'return': -1.0}}
        new = {'study': {'notes': ['new explanatory text'], 'return': -1.0}}
        self.assertEqual(p.without_notes(old), p.without_notes(new))
        new['study']['return'] = 1.0
        self.assertNotEqual(p.without_notes(old), p.without_notes(new))
