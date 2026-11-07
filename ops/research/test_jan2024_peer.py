import copy
import unittest

import jan2024_peer as p


class JanuaryPeerTests(unittest.TestCase):
    def row(self,date='2024-01-22',net=.8):
        entry=dict(entered_at=date+'T13:30:01Z',side='long',fill=100.,initial_r=1.,volume=100)
        outcome=dict(remaining_volume=0,scenario_gross_r=1.,trail_applied=0,
                     fills=[dict(at=date+'T14:00:00Z',price=101.,volume=100,reason='target')])
        model=dict(study=dict(trade=dict(entry=entry,exit_comparison=dict(candidate=outcome)),marks=[]),
                   sensitivities=dict(trailing={'24':net}))
        return dict(key='SpotCrude-'+date,symbol='SpotCrude',status='complete',models={'s3_two_stage':model})

    def test_duplicate_january_win_is_not_new_and_conflicting_outcome_stops(self):
        jan=self.row();may=self.row('2025-05-15',-1.7)
        r=p.cumulative([('old',[jan]),('may',[may]),('new',[copy.deepcopy(jan)])])
        self.assertEqual((r['summary']['closed_trades'],r['summary']['wins'],r['summary']['losses']),(2,1,1))
        self.assertEqual(r['duplicate_observations_removed'],1)
        altered=copy.deepcopy(jan);altered['models']['s3_two_stage']['sensitivities']['trailing']['24']=1.38
        with self.assertRaisesRegex(ValueError,'conflicting'):p.cumulative([('old',[jan]),('changed',[altered])])

    def pair(self):
        req={'from':'2024-01-22T14:30:00Z','to':'2024-01-22T18:30:00Z'}
        start=int(p.instant(req['from']).timestamp()*1000)
        source=dict(request=req,payload=dict(quotes={'bids':[1],'asks':[2]},m30=[{'own':1}],source_from=req['from'],peer_symbol=None,peer_m30=[]))
        peer=dict(status='complete',request=dict(req,symbol='NAS100'),bars=[dict(ts_ms=start+i*1800000,volume=1) for i in range(8)])
        return source,peer

    def test_peer_overlay_keeps_primary_inputs_and_excludes_later_bars(self):
        source,peer=self.pair();before=copy.deepcopy(source);result=p.overlay(source,peer)
        self.assertEqual(source,before)
        for field in ['quotes','m30','source_from']:self.assertEqual(result['payload'][field],source['payload'][field])
        self.assertEqual(result['request'],source['request'])
        self.assertEqual(len(result['payload']['peer_m30']),6)
        self.assertEqual(result['payload_sha256'],p.digest(result['payload']))

    def test_failed_or_time_shifted_peer_cannot_manufacture_confirmation(self):
        source,peer=self.pair();peer['status']='failed'
        self.assertEqual(p.overlay(source,peer)['payload']['peer_m30'],[])
        peer['request']['from']='2024-01-22T13:30:00Z'
        with self.assertRaisesRegex(ValueError,'interval mismatch'):p.overlay(source,peer)


if __name__=='__main__':unittest.main()
