import unittest
import json
import hashlib
import tempfile
from pathlib import Path
from class_batch_report import score, summarize, verify_receipts
from class_batch import digest, ensure_provenance


def episode(value, reason='simulated_entry'):
    trade = None if reason != 'simulated_entry' else {
        'trailing_scenario_net_r': value,
        'fixed_stop_scenario_net_r': None,
        'exit_comparison': {'candidate': {'net_r': None, 'data_issues': []},
                            'baseline': {'net_r': None, 'data_issues': ['censored']}}}
    return {'status': 'complete', 'replay': {'studies': {'rule': {
        'marks': [{'reason': reason}], 'trade': trade}}}}


class ScoreTests(unittest.TestCase):
    def test_denominators_censoring_and_exit_arms_stay_separate(self):
        rows = [episode(2), episode(-1), episode(0), episode(None),
                episode(None, 'candidate_cost_ge_r'), {'status': 'failed'}]
        s = score(rows, 'rule', 'trailing')
        self.assertEqual((s['episodes'], s['complete'], s['entries'], s['scenario_closed']), (6,5,4,3))
        self.assertEqual((s['wins'], s['losses'], s['breakevens'], s['censored_or_invalid']), (1,1,1,1))
        self.assertAlmostEqual(s['scenario_win_rate'], 1/3)
        self.assertAlmostEqual(s['scenario_mean_r'], 1/3)
        self.assertEqual(s['scenario_pf_r'], 2)
        fixed = score(rows, 'rule', 'fixed')
        self.assertEqual(fixed['censored_or_invalid'], 4)
        self.assertIsNone(fixed['scenario_win_rate'])
        self.assertEqual(fixed['qualified_net_closes'], 0)
        self.assertFalse(fixed['financial_pass'])

    def test_missing_jobs_cannot_be_hidden_from_report(self):
        with self.assertRaisesRegex(ValueError, 'coverage differs'):
            summarize({'jobs': [{'key': 'expected'}]}, {'rows': []})


class ProvenanceTests(unittest.TestCase):
    def test_every_collected_job_requires_intact_receipts_and_raw_source(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            plan = {'test': True}
            raw = root/'job-a.json'
            raw.write_text('{}')
            sha = hashlib.sha256(raw.read_bytes()).hexdigest()
            results = {'rows': [{'key': 'a', 'status': 'complete', 'file_sha256': sha}]}
            ledger = root/'ledger.jsonl'
            ledger.write_text('')
            with self.assertRaisesRegex(ValueError, 'receipt coverage'):
                verify_receipts(plan, results, root)
            events = [dict(key='a', state='started', plan_sha256=digest(plan)),
                      dict(key='a', state='finished', status='complete', file_sha256=sha)]
            ledger.write_text(''.join(json.dumps(e)+'\n' for e in events))
            self.assertEqual(verify_receipts(plan, results, root), events)
            raw.write_text('{"tampered":true}')
            with self.assertRaisesRegex(ValueError, 'source/receipt/replay mismatch'):
                verify_receipts(plan, results, root)

    def test_fresh_collection_records_provenance_before_reads_and_preserves_it(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            ensure_provenance({'test': True}, root)
            original = (root/'freeze.json').read_bytes()
            manifest = json.loads(original)
            self.assertTrue(manifest['commit'])
            self.assertIn('ops/research/class_batch.py', manifest['source_hashes'])
            ensure_provenance({'test': True}, root)
            self.assertEqual((root/'freeze.json').read_bytes(), original)
            with self.assertRaisesRegex(ValueError, 'frozen source or plan changed'):
                ensure_provenance({'test': False}, root)
            (root/'freeze.json').unlink()
            (root/'ledger.jsonl').write_text('')
            with self.assertRaisesRegex(ValueError, 'missing pre-read provenance'):
                ensure_provenance({'test': True}, root)
