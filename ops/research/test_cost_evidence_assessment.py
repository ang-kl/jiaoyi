import copy
import unittest

import cost_evidence_assessment as audit


class CostEvidenceTests(unittest.TestCase):
    def test_missing_decision_keeps_valid_dispatch_fill_evidence(self):
        row={'kind':'open','side':'buy','fill_price':101,'entry_cost':{
            'decision_price':None,'dispatch_price':100,'fill_price':101,'dispatch_spread':0.2,
            'dispatch_quote_age_ms':20,'decision_to_dispatch_bps':None,
            'dispatch_to_fill_bps':100,'decision_to_fill_bps':None}}
        result=audit.verified_entry(row)
        self.assertEqual(result['dispatch_to_fill_bps'],100)
        self.assertIsNone(result['decision_price'])
        self.assertIsNone(result['decision_to_fill_bps'])
        wrong=copy.deepcopy(row);wrong['entry_cost']['decision_to_fill_bps']=0
        with self.assertRaises(ValueError):audit.verified_entry(wrong)
        wrong=copy.deepcopy(row);wrong['entry_cost']['dispatch_to_fill_bps']=-100
        with self.assertRaises(ValueError):audit.verified_entry(wrong)

    def test_signed_attribution_is_independently_recomputed(self):
        row={'kind':'open','side':'sell','fill_price':99,'entry_cost':{
            'decision_price':101,'dispatch_price':100,'fill_price':99,'dispatch_spread':0.2,
            'dispatch_quote_age_ms':20,'decision_to_dispatch_bps':100,
            'dispatch_to_fill_bps':100,'decision_to_fill_bps':200}}
        self.assertIsNotNone(audit.verified_entry(row))
        wrong=copy.deepcopy(row);wrong['entry_cost']['dispatch_to_fill_bps']=-100
        with self.assertRaises(ValueError):audit.verified_entry(wrong)
        row['entry_cost']=None
        self.assertIsNone(audit.verified_entry(row))

    def test_empty_attribution_is_unknown_and_never_a_zero_cost_calibration(self):
        row={'strategy':'s3','account_id':43002148,'entry_key':'test','kind':'open',
             'symbol':'SpotCrude','fill_price':100,'entry_cost':None,'created_at':'2026-11-06T00:00:00Z'}
        result=audit.assess([row],{'unique_rows':1,'filled_opens':1,'measured_entries':0},[],[])
        self.assertEqual(result['summary']['oil']['filled_opens'],1)
        self.assertEqual(result['summary']['oil']['measured_entries'],0)
        self.assertIsNone(result['calibrated_per_side_bps'])
        self.assertFalse(result['financial_pass'])
        self.assertIsNone(result['brokerwide_complete_cost_sample_count'])
        with self.assertRaises(ValueError):audit.assess([row,row],{},[],[])


if __name__=='__main__':unittest.main()
