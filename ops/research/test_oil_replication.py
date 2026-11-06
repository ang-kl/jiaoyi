import copy
import json
from pathlib import Path
import tempfile
import unittest

import oil_replication as oil


def source_job(request, plan):
    start=oil.replay.context.ms(request['from'])
    source=dict(bars=[],bids=[dict(ts_ms=start,price_e5=6500000)],
                asks=[dict(ts_ms=start,price_e5=6501000)],
                contract=dict(digits=3,min_volume=100,step_volume=100))
    return dict(id='test',status='complete',request=request,result=dict(
        environment='demo',qualification='DEVELOPMENT_SCENARIO_NOT_FINANCIAL_PASS',
        symbol_id=1,input=source,input_sha256=oil.market_collection.source_digest(source),tick_pages=2))


def winning_rows(plan):
    return [dict(symbol=r['symbol'],rule='own_momentum',**{'from':r['from']},
        status='simulated_entry',replay=dict(status='simulated_entry',input=dict(initial_r=1),
            sensitivities=[dict(arm='trailing',bps_per_side=24,scenario_net_r=1)],
            comparison=dict(candidate=dict(data_issues=[])))) for r in plan['collection_requests']]


class OilReplicationTests(unittest.TestCase):
    def test_frozen_plan_identity_and_source_contract_are_enforced(self):
        plan=oil.load_plan();request=plan['collection_requests'][0]
        self.assertEqual(plan['primary_candidate'],'own_momentum')
        self.assertEqual(len(plan['collection_requests']),20)
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder)/'plan.json';p.write_bytes(oil.PLAN.read_bytes()+b' ')
            with self.assertRaises(ValueError):oil.load_plan(p)
        job=source_job(request,plan);oil.validate_job(job,request,plan)
        tied=copy.deepcopy(job)
        tied['result']['input']['bids'].append(copy.deepcopy(tied['result']['input']['bids'][0]))
        tied['result']['input_sha256']=oil.market_collection.source_digest(tied['result']['input'])
        oil.validate_job(tied,request,plan)  # Native collector permits equal milliseconds.
        wrong=copy.deepcopy(job);wrong['request']['account']=43002148
        with self.assertRaises(ValueError):oil.validate_job(wrong,request,plan)
        wrong=copy.deepcopy(job);wrong['result']['input']['contract']['min_volume']=200
        wrong['result']['input_sha256']=oil.market_collection.source_digest(wrong['result']['input'])
        with self.assertRaises(ValueError):oil.validate_job(wrong,request,plan)

    def test_incomplete_or_censored_evidence_cannot_pass_a_winning_score(self):
        plan=oil.load_plan();rows=winning_rows(plan)
        self.assertTrue(oil.score(rows,plan)['passes_screen'])
        missing=copy.deepcopy(rows);missing[0].update(status='missing_next_fresh_entry',replay=None)
        self.assertFalse(oil.score(missing,plan)['passes_screen'])
        censored=copy.deepcopy(rows);censored[0]['replay']['sensitivities'][0]['scenario_net_r']=None
        self.assertEqual(oil.score(censored,plan)['censored'],1)
        self.assertFalse(oil.score(censored,plan)['passes_screen'])
        gap=copy.deepcopy(rows);gap[0]['replay']['comparison']['candidate']['data_issues']=['gap_before_quote']
        self.assertFalse(oil.score(gap,plan)['passes_screen'])
        self.assertFalse(oil.score(rows[:10],plan)['passes_screen'])

    def test_collection_budget_preflight_and_uncertain_post_prevent_retries(self):
        plan=oil.load_plan();posts=[];current=None
        def api(base,password,request=None):
            nonlocal current
            if request is not None:
                posts.append(request);current=source_job(request,plan)
                return dict(id='test')
            return current
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder)
            self.assertEqual(oil.collect(out,plan,'unused',api=api,sleep=lambda _:None),2)
            self.assertEqual(len(posts),2)
            with self.assertRaises(FileNotFoundError):oil.collect(out,plan,'unused',api=api,sleep=lambda _:None)
            self.assertEqual(len(posts),2)
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder);calls=[]
            def uncertain(base,password,request=None):
                if request is None:return None
                calls.append(request);raise TimeoutError('indeterminate submission')
            with self.assertRaises(TimeoutError):oil.collect(out,plan,'unused',api=uncertain)
            with self.assertRaises(ValueError):oil.collect(out,plan,'unused',api=uncertain)
            self.assertEqual(len(calls),1)
            self.assertEqual(oil.state_at(out)['attempts'][0]['status'],'failed_or_uncertain')
        with tempfile.TemporaryDirectory() as folder:
            out=Path(folder);state=oil.state_at(out)
            for request in plan['collection_requests']:oil.reserve(out,state,request,plan)
            with self.assertRaises(ValueError):oil.reserve(out,state,plan['collection_requests'][0],plan)
            self.assertEqual(len(json.loads((out/'ledger.json').read_text())['attempts']),20)


if __name__=='__main__':
    unittest.main()
