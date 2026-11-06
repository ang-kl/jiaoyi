"""Three Python acceptance groups; two Rust groups live beside Strategy 3."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import class_research as classes
import milestone
from research_gates import DATA_GAP_REASONS

ROOT = Path(__file__).resolve().parents[2]


def episodes(values):
    return [dict(symbol='A', **{'from': f'2025-08-{i+1:02}T00:00:00Z'}, study={
        'trade': {'trailing_scenario_net_r': v, 'entry': {'initial_r': 1.0}}})
        for i, v in enumerate(values)]


class PhaseATests(unittest.TestCase):
    def test_score_fails_on_data_gap_when_trade_is_none(self):
        # Enough profitable closes to pass absent the missing-data episode.
        base = episodes([2.0, -1.0])
        for score in (lambda r: classes.score(r, 2, 3, ['A']),
                      lambda r: milestone.score(r, 2, 3, 'development')):
            for reason in sorted(DATA_GAP_REASONS | {'flat_predictor', 'peer_disagreement', 'no_breakout'}):
                for channel in ('status', 'marks', 'quality'):
                    if channel == 'quality' and reason != 'zero_tick_stream':
                        continue
                    with self.subTest(reason=reason, channel=channel, scorer=score):
                        study = {'trade': None}
                        if channel == 'status':
                            study['status'] = reason
                        elif channel == 'marks':
                            study['marks'] = [{'reason': reason}, {'reason': reason}]
                        else:
                            study['quality'] = {'bid_ticks': 0, 'ask_ticks': 10}
                        result = score(base + [dict(symbol='A', **{'from': '2025-08-03T00:00:00Z'}, study=study)])
                        gap = reason in DATA_GAP_REASONS
                        self.assertEqual(result['passes_screen'], not gap)
                        self.assertEqual(result['data_gaps'], {reason: 1} if gap else {})
                        self.assertEqual(any('data coverage gaps' in r for r in result['reasons']), gap)
        # Rank only passing candidates. Variance can outweigh a higher mean;
        # n=1 keeps a descriptive mean but cannot bypass minimum evidence gates.
        stable = classes.score(episodes([2, 2]), 2, 2, ['A'])
        volatile = classes.score(episodes([10, -1]), 2, 2, ['A'])
        single = classes.score(episodes([99]), 2, 1, ['A'])
        self.assertGreater(volatile['mean_net_r'], stable['mean_net_r'])
        self.assertLess(volatile['lcb_net_r'], stable['lcb_net_r'])
        self.assertFalse(single['passes_screen'])
        self.assertEqual(classes.select({'classes': {'test': {'stable': stable, 'volatile': volatile, 'single': single}}}), {'test': 'stable'})
        self.assertEqual(milestone.score(episodes([2, 2]), 2, 2, 'development')['lcb_net_r'], stable['lcb_net_r'])

    def test_reuse_only_preserves_collection_receipt(self):
        # Derive the hash from the immutable plan, not a copied prompt's typo.
        source_bytes = (ROOT / 'docs/class-prediction-plan-2026-11-05.json').read_bytes()
        source = hashlib.sha256(source_bytes).hexdigest()
        self.assertEqual(source, '8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaaefda608f597e88ac')
        digest = hashlib.sha256(b'new analysis').hexdigest()
        with tempfile.TemporaryDirectory() as folder:
            out = Path(folder)
            receipt = out / 'plan-sha256.txt'
            original = source + '\n'
            receipt.write_text(original)
            for wrong in ('0' * 64, None, '8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaae648e642cb3a2d20'):
                with self.assertRaises(RuntimeError):
                    classes.verify_receipt(out, {'reuse_only': True, 'source_collection_plan_sha256': wrong}, digest)
                self.assertFalse((out / 'analysis-plan-sha256.txt').exists())
                self.assertEqual(receipt.read_text(), original)
            self.assertEqual(classes.verify_receipt(out, {'reuse_only': True, 'source_collection_plan_sha256': source}, digest), source)
            self.assertEqual(receipt.read_text(), original)
            self.assertEqual((out / 'analysis-plan-sha256.txt').read_text(), digest + '\n')
            with self.assertRaises(RuntimeError):
                classes.verify_receipt(out, {}, digest)
        plan = json.loads(source_bytes)
        req = classes.request(plan, 'JPN225', '2025-08-05T00:00:00Z', '2025-08-05T04:00:00Z')
        other = copy.deepcopy(req)
        other['tick_replay']['entry_rule'] = 'class_reversion1h'
        self.assertTrue(classes.request_matches(other, req, True))
        self.assertFalse(classes.request_matches(other, req, False))
        other['tick_replay']['volume'] = 100
        self.assertFalse(classes.request_matches(other, req, True))
        for field, value in [('account', 1), ('commission_round_trip', 1), ('slippage_bps_per_side', 2), ('to', '2025-08-05T05:00:00Z')]:
            other = copy.deepcopy(req)
            other[field] = value
            self.assertFalse(classes.request_matches(other, req, True))
        self.assertEqual(req['tick_replay']['entry_rule'], plan['rules'][0])

    def test_reproduce_review_exact_match(self):
        development = os.environ.get('JIAOYI_RESEARCH_DEVELOPMENT')
        fresh = os.environ.get('JIAOYI_RESEARCH_FRESH')
        if not development or not fresh:
            self.skipTest('set JIAOYI_RESEARCH_DEVELOPMENT and JIAOYI_RESEARCH_FRESH to original local corpora; no collection performed')
        with tempfile.TemporaryDirectory() as out:
            subprocess.run([sys.executable, str(ROOT / 'docs/ai-review-2026-11-06/scripts/reproduce_review.py'),
                            '--repository', str(ROOT), '--development', development, '--fresh', fresh,
                            '--output', out], check=True, capture_output=True, text=True, timeout=120)
            result = json.loads((Path(out) / 'fresh-reconsideration.json').read_text())
            expected = json.loads((ROOT / 'docs/ai-review-2026-11-06/reviews/fresh-reconsideration.json').read_text())
            self.assertEqual(result['development_rows_reproduced_exactly'], 480)
            self.assertEqual(result['tick_count'], 1281012)
            self.assertEqual(result['fresh_statuses'], {'observed': 222, 'missing_predictor': 18})
            for key in expected.keys() - {'generated_at_utc', 'script_sha256'}:
                self.assertEqual(result[key], expected[key], key)


if __name__ == '__main__':
    unittest.main()
