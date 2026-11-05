import unittest
from milestone import score

def rows(values):
    return [{'from':f'2025-09-{i+1:02}T00:00:00Z','study':{'trade':{'trailing_scenario_net_r':v,'entry':{'initial_r':1.0}}}} for i,v in enumerate(values)]
class MilestoneTests(unittest.TestCase):
    def test_fifty_percent_without_positive_expectancy_fails(self):
        r=score(rows([.1,-1]*10),20,20,'validation')
        self.assertEqual(r['net_win_rate'],.5);self.assertFalse(r['passes_screen']);self.assertLess(r['profit_factor'],1)
    def test_censoring_missing_episodes_and_small_samples_cannot_pass(self):
        r=score(rows([1,1,1,1,1,None]),5,6,'development');self.assertFalse(r['passes_screen']);self.assertEqual(r['censored'],1)
        self.assertFalse(score(rows([1]*5),5,6,'development')['passes_screen'])
        self.assertFalse(score(rows([1]*4),5,4,'development')['passes_screen'])
    def test_zero_returns_remain_in_denominator_and_uncertainty_is_reported(self):
        r=score(rows([2,-1,2,-1,0,0]),5,6,'development')
        self.assertAlmostEqual(r['net_win_rate'],1/3);self.assertFalse(r['passes_screen'])
        r=score(rows([2,-1]*10),20,20,'validation');self.assertTrue(r['passes_screen']);self.assertLess(r['win_rate_wilson95'][0],.5)
    def test_variable_r_cannot_hide_larger_price_unit_losses(self):
        rr=rows([1,-.5]);rr[1]['study']['trade']['entry']['initial_r']=10
        r=score(rr,2,2,'development');self.assertGreater(r['mean_net_r'],0);self.assertLess(r['net_price_units'],0);self.assertFalse(r['passes_screen'])
if __name__=='__main__':unittest.main()
