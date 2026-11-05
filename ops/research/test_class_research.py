import unittest
from class_research import score,select,windows

def rows(symbol,values):
    return [dict(symbol=symbol,**{'from':f'2025-08-{i+1:02}T00:00:00Z'},study={'trade':{'trailing_scenario_net_r':v,'entry':{'initial_r':1}}}) for i,v in enumerate(values)]
class ClassResearchTests(unittest.TestCase):
    def test_class_aggregate_cannot_hide_failing_symbol_or_censored_data(self):
        r=score(rows('A',[3,3,3])+rows('B',[-1]),4,4,['A','B'])
        self.assertEqual(r['net_win_rate'],.75);self.assertFalse(r['passes_screen'])
        self.assertFalse(score(rows('A',[3,None]),1,2,['A'])['passes_screen'])
        self.assertFalse(score(rows('A',[3,3]),2,3,['A'])['passes_screen'])
    def test_fifty_percent_alone_and_duplicate_outcomes_do_not_pass(self):
        self.assertFalse(score(rows('A',[.1,-1]),2,2,['A'])['passes_screen'])
        rr=rows('A',[3,3]);rr[1]['from']=rr[0]['from'];self.assertFalse(score(rr,2,2,['A'])['passes_screen'])
        self.assertFalse(score(rows('A',[3,3]),4,2,['A'])['passes_screen'])
        r=score(rows('A',[2,-1]),2,2,['A']);self.assertTrue(r['passes_screen']);self.assertLess(r['win_rate_wilson95'][0],.5)
        self.assertEqual(select({'classes':{'crypto':{'one':r,'two':dict(r,passes_screen=False,mean_net_r=99)}}}),{'crypto':'one'})
    def test_timezone_windows_cross_utc_dates_and_dst_without_changing_horizon(self):
        p={'development_dates':['2025-08-05','2025-10-20']}
        au=list(windows(p,'development','AUS200'));self.assertIn('2025-10-19T23:00',au[1][1]);self.assertIn('2025-08-05T00:00',au[0][1])
        us=list(windows(p,'development','NAS100'));self.assertIn('13:30',us[0][1])
if __name__=='__main__':unittest.main()
