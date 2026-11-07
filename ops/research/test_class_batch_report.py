import unittest
from class_batch_report import score, summarize


def episode(value, reason='simulated_entry'):
    trade = None if reason != 'simulated_entry' else {
        'trailing_scenario_net_r': value,
        'fixed_stop_scenario_net_r': None,
        'exit_comparison': {'candidate': {'net_r': None, 'data_issues': []},
                            'baseline': {'net_r': None, 'data_issues': ['censored']}}}
    return {'status': 'complete', 'replay': {'studies': {'rule': {
        'marks': [{'reason': reason}], 'trade': trade}}}}


class ScoreTests(unittest.TestCase):
    def test_denominators_censoring_and_exit_arms_stay_separate(self):
        rows = [episode(2), episode(-1), episode(0), episode(None),
                episode(None, 'candidate_cost_ge_r'), {'status': 'failed'}]
        s = score(rows, 'rule', 'trailing')
        self.assertEqual((s['episodes'], s['complete'], s['entries'], s['scenario_closed']), (6,5,4,3))
        self.assertEqual((s['wins'], s['losses'], s['breakevens'], s['censored_or_invalid']), (1,1,1,1))
        self.assertAlmostEqual(s['scenario_win_rate'], 1/3)
        self.assertAlmostEqual(s['scenario_mean_r'], 1/3)
        self.assertEqual(s['scenario_pf_r'], 2)
        fixed = score(rows, 'rule', 'fixed')
        self.assertEqual(fixed['censored_or_invalid'], 4)
        self.assertIsNone(fixed['scenario_win_rate'])
        self.assertEqual(fixed['qualified_net_closes'], 0)
        self.assertFalse(fixed['financial_pass'])

    def test_missing_jobs_cannot_be_hidden_from_report(self):
        with self.assertRaisesRegex(ValueError, 'coverage differs'):
            summarize({'jobs': [{'key': 'expected'}]}, {'rows': []})
