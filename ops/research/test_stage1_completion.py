import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import stage1_completion as s


class CompletionTests(unittest.TestCase):
    def test_partition_rejects_unrequested_crypto_gaps(self):
        jobs=[{'episode_id':str(i),'symbol':'US500','date':'2024-11-11','required_m30_slots':[0]} for i in range(30)]
        jobs += [{'episode_id':str(30+i),'symbol':symbol,'date':'2024-11-11','required_m30_slots':[0,1800000]} for symbol in ['BTCUSD','ETHUSD'] for i in range(5)]
        p={'episodes':jobs,'bar_segments':[{'key':symbol,'symbol':symbol,'from':'1970-01-01T00:00:00Z','to':'1970-01-01T01:00:00Z'} for symbol in ['BTCUSD','ETHUSD']]}
        bars={(s.ACCOUNT,symbol):{0:{}} for symbol in ['US500','BTCUSD','ETHUSD']}
        ready,blocked,gaps=s.partition(p,bars)
        self.assertEqual((len(ready),len(blocked)),(30,10));self.assertEqual(len(gaps['BTCUSD']),1)
        p['bar_segments']=[]
        with self.assertRaisesRegex(s.Refusal,'unspent_crypto_gap'):s.partition(p,bars)

    def test_scope_bound_tick_capability_and_uncertain_ledger(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);plan=root/'plan.json';plan.write_text('{}');s.save(root/'freeze.json',{'plan_sha256':s.sha(plan)})
            job={'episode_id':'one','symbol_id':1,'from':'2024-11-11T07:00:00Z','to':'2024-11-11T11:00:00Z'}
            (root/'ledger.jsonl').write_text(json.dumps({'state':'started','key':'job-one'})+'\n')
            auth=object.__new__(s.CompletionAuthority);auth.plan={'episodes':[job]};auth.current_job=job
            payload={'ctidTraderAccountId':s.ACCOUNT,'symbolId':1,'type':1,'fromTimestamp':s.ms(job['from']),'toTimestamp':s.ms(job['to'])}
            with patch.object(s,'WORK',root),patch.object(s,'PLAN',plan):
                auth.authorize(2145,payload)
                for kind in [2137,2106,2114]:
                    with self.assertRaises(s.Refusal):auth.authorize(kind,payload)
                for change in [{'symbolId':2},{'ctidTraderAccountId':1},{'toTimestamp':s.ms(job['to'])+5001}]:
                    with self.assertRaises(s.Refusal):auth.authorize(2145,dict(payload,**change))
                with self.assertRaisesRegex(s.Refusal,'uncertain_or_duplicate'):s.ledger()

    def test_breakeven_stop_is_not_net_flat(self):
        strict={'entry':{'fill':100},'metrics':{'net_by_bps':{'3.74':-0.1}},'outcome':{'fills':[{'reason':'gap_through_stop'}],'active_stop':100,'trail_applied':1}}
        self.assertEqual(s.strict_exit_classification(strict),'breakeven_stop_after_1r')
        strict['outcome']['trail_applied']=0
        self.assertEqual(s.strict_exit_classification(strict),'initial_stop_loss')
        strict['outcome']['fills']=[]
        self.assertEqual(s.strict_exit_classification(strict),'unresolved')


if __name__=='__main__':unittest.main()
