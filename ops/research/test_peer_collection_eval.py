import json
from pathlib import Path
import tempfile
import unittest

import peer_collection_eval as p


class PeerCollectionTests(unittest.TestCase):
    def test_spent_intervals_block_renamed_or_partial_rereads(self):
        spent=dict(symbol_id=1,**{'from':'2025-05-12T00:00:00Z','to':'2025-05-12T04:00:05.001Z'})
        self.assertTrue(p.interval_overlap(spent,dict(spent,key='new-name')))
        self.assertTrue(p.interval_overlap(spent,dict(spent,**{'from':'2025-05-12T02:00:00Z'})))
        self.assertFalse(p.interval_overlap(spent,dict(spent,symbol_id=2)))
        self.assertFalse(p.interval_overlap(spent,dict(spent,**{'from':spent['to'],'to':'2025-05-12T05:00:00Z'})))

    def test_receipts_keep_failure_and_refuse_uncertain_repeats_or_tampering(self):
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp);ledger=out/'ledger.jsonl'
            ledger.write_text(json.dumps(dict(state='started',key='tick-test'))+'\n')
            with self.assertRaisesRegex(ValueError,'uncertain'):p.completed(out)
            source=out/'tick-test.json';source.write_text('{"status":"failed","error":"timeout"}')
            with ledger.open('a') as f:f.write(json.dumps(dict(state='finished',key='tick-test',status='failed',sha256=p.sha(source)))+'\n')
            self.assertEqual(p.completed(out)['tick-test']['status'],'failed')
            source.write_text('{"status":"complete"}')
            with self.assertRaisesRegex(ValueError,'hash'):p.completed(out)

    def test_latest20_uses_chronology_and_never_calls_16_of_22_an_account_pass(self):
        rows=[]
        # First16 wins, final6 losses =>14 wins in actual latest20, below15.
        for i in range(22):
            value=1 if i<16 else -1
            control=dict(closed_at=f'2025-05-12T00:{i:02}:00Z',gross_r=value,net_by_bps={'24':value})
            rows.append(dict(key=str(i),status='complete',models={'s3_two_stage':{'control':control}}))
        summary=p.score(rows,'s3_two_stage','24')
        self.assertLess(summary['win_rate'],0.73)
        window=p.latest20(list(reversed(rows)),'s3_two_stage')
        self.assertEqual((window['window_size'],window['wins']),(20,14))
        self.assertFalse(window['numerical_73_percent_threshold_met'])
        self.assertFalse(window['account_goal_satisfied'])
        self.assertFalse(p.latest20(rows[:5],'s3_two_stage')['enough_closes'])


if __name__=='__main__':unittest.main()
