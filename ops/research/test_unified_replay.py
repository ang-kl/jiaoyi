import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unified_replay import (CostKey, cost_joins, causal_context, slots, immutable_read, sha, ms, stamp, score)

class UnifiedReplayTests(unittest.TestCase):
    def test_source_mutation_and_quarantined_alias_are_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'x.json';p.write_text('{}');h=sha(p)
            self.assertEqual(immutable_read(p,h,set()),{})
            with self.assertRaisesRegex(ValueError,'quarantined'):immutable_read(p,h,{h})
            p.write_text('{"changed":true}')
            with self.assertRaisesRegex(ValueError,'mismatch'):immutable_read(p,h,set())
    def test_context_requires_complete_causal_sessions(self):
        first=ms('2024-01-01T00:00:00Z');sessions=[];bars={}
        for n in range(20):
            t=first+n*86400000;sessions.append({'open':stamp(t),'close':stamp(t+1800000),'lunch':None});bars[t]={'open_e5':100,'close_e5':101+n}
        end=first+20*86400000
        self.assertEqual(causal_context(sessions,bars,end)['status'],'complete')
        with self.assertRaisesRegex(ValueError,'causal'):causal_context(sessions,bars,first)
        bars.pop(first);self.assertEqual(causal_context(sessions,bars,end)['status'],'missing_segments')
    def test_cost_join_cannot_pool_another_account_or_default_unknown_to_zero(self):
        inventory={'execution_evidence':{'attributed_entries':[{'account_id':1,'symbol':'BTCUSD','environment':'demo','slippage_bps':2.}], 'positions':[]},'symbols':{}}
        manifest={'episodes':[{'account_id':2,'symbol':'BTCUSD'}]}
        joins=cost_joins(inventory,manifest)
        self.assertEqual(joins[CostKey(1,'BTCUSD')].median_entry_slippage_bps,2.)
        self.assertIsNone(joins[CostKey(2,'BTCUSD')].median_entry_slippage_bps)
        rows=[{'native':{'arms':{'a':{'strict':{'metrics':None}}}}},{'native':{'arms':{}}}]
        s=score(rows,'a','strict','24');self.assertEqual((s['entries'],s['unresolved'],s['no_entry']),(1,1,1));self.assertIsNone(s['net_win_rate'])
    def test_offline_guard_fails_before_socket_connection(self):
        code='import sys,socket; from unified_replay import deny_network; sys.addaudithook(deny_network); socket.create_connection(("127.0.0.1",9))'
        r=subprocess.run([sys.executable,'-c',code],cwd=Path(__file__).parent,capture_output=True,text=True)
        self.assertNotEqual(r.returncode,0);self.assertIn('offline replay prohibits network',r.stderr)
