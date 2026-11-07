import unittest
from native_target_sizing import observation,identity,score

class NativeCostAccountingTests(unittest.TestCase):
    def test_arm_cost_and_censoring_are_not_replaced_with_baseline_results(self):
        o={'entry':{'entered_at':'x','side':'long','fill':100.,'initial_r':1.,'volume':100},
           'metrics':{'gross_r':.1,'net_by_bps':{'24':-.38,'2':.06,'3.74':.0252}}}
        self.assertEqual(observation({'bps':24.,'strict':o},'strict'),('loss',-.38))
        self.assertEqual(observation({'bps':2.,'strict':o},'strict'),('win',.06))
        self.assertEqual(observation({'bps':2.,'strict':dict(o,metrics=None)},'strict'),('unresolved',None))
        changed=dict(o,entry=dict(o['entry'],entered_at='later'))
        self.assertNotEqual(identity(o),identity(changed))
        rows=[{'arms':{'x':{'bps':2.,'strict':o}}},{'arms':{'x':{'bps':2.,'strict':dict(o,metrics=None)}}}]
        s=score(rows,'x','strict');self.assertEqual((s['entries'],s['closed'],s['wins'],s['unresolved']),(2,1,1,1))

if __name__=='__main__':unittest.main()
