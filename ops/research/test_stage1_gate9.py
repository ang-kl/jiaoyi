import copy
import unittest
from unittest.mock import patch
import stage1_gate9 as s
from broker_readonly import Broker
from rebuild_preflight import Refusal

class Stage1GuardTests(unittest.TestCase):
    def authority(self):
        a=s.Stage1Authority.__new__(s.Stage1Authority)
        a.plan={'episodes':[{'symbol_id':7,'from':'2024-11-11T07:00:00Z','to':'2024-11-11T11:00:00Z'}],
                'bar_segments':[{'symbol_id':7,'from':'2024-10-01T07:00:00Z','to':'2024-10-01T08:00:00Z'}]}
        a.ticks_ready=False
        return a
    def check(self,a,kind,payload):
        with patch.object(s,'sha',return_value='frozen'),patch.object(s,'read',return_value={'plan_sha256':'frozen'}):
            return a.authorize(kind,payload)
    def test_default_collectors_still_paused(self):
        with self.assertRaisesRegex(RuntimeError,'paused'):Broker().authorize()
    def test_tick_before_complete_preparation_refused(self):
        a=self.authority();p={'ctidTraderAccountId':s.ACCOUNT,'symbolId':7,'type':1,'fromTimestamp':s.ms('2024-11-11T07:00:00Z'),'toTimestamp':s.ms('2024-11-11T11:00:00Z')}
        with self.assertRaisesRegex(Refusal,'preparation_barrier'):self.check(a,2145,p)
        a.ticks_ready=True;self.check(a,2145,p)
        p['toTimestamp']+=6000
        with self.assertRaisesRegex(Refusal,'unreserved_tick'):self.check(a,2145,p)
    def test_m30_exact_interval_and_no_count(self):
        a=self.authority();p=s.bar_request(a.plan['bar_segments'][0]);self.check(a,2137,p)
        with self.assertRaisesRegex(Refusal,'interval_only'):self.check(a,2137,dict(p,count=500))
        p['fromTimestamp']-=1
        with self.assertRaisesRegex(Refusal,'unreserved_m30'):self.check(a,2137,p)
    def test_no_order_or_wrong_account(self):
        a=self.authority()
        with self.assertRaisesRegex(Refusal,'forbidden'):self.check(a,2106,{})
        with self.assertRaisesRegex(Refusal,'account'):self.check(a,2102,{'ctidTraderAccountId':43002148})

if __name__=='__main__':unittest.main()
