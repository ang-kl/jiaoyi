import copy
import unittest
import run_three_year_pipeline as p

class ThreeYearTests(unittest.TestCase):
    def test_scope_seal_and_prior_failure_retained(self):
        manifest=p.load();self.assertEqual(len(manifest['jobs']),120)
        self.assertEqual(len(manifest['bar_segments']),176)
        reused=[j for j in manifest['jobs'] if 'reuse' in j]
        self.assertEqual(len(reused),10)
        failed=[j for j in reused if j['reuse']['receipt']['status']=='failed']
        self.assertEqual([j['key'] for j in failed],['US500-2024-01-24'])
        self.assertEqual(p.window_open('SpotCrude','2023-07-24').hour,13)
        self.assertEqual(p.window_open('US500','2023-07-24').hour,13)
        self.assertEqual(p.window_open('US500','2023-01-23').hour,14)

    def test_ci_requires_three_explicit_completed_successes(self):
        good=[dict(name=n,status='COMPLETED',conclusion='SUCCESS') for n in p.REQUIRED]
        self.assertEqual(p.check_ci(good),'success')
        for bad in [[],good[:2],[dict(name=n) for n in p.REQUIRED],good+[good[0]],good[:2]+[dict(good[2],status='IN_PROGRESS',conclusion='')]]:
            self.assertEqual(p.check_ci(bad),'pending')
        self.assertEqual(p.check_ci(good[:2]+[dict(good[2],conclusion='FAILURE')]),'failed')

    def test_missing_and_failed_jobs_are_not_literal_winners(self):
        rows=[{**{k:j[k] for k in ['key','symbol','date','cohort','asset_class']},'status':'failed'} for j in p.load()['jobs']]
        summary=p.summarize(rows)
        for m in p.MODELS:
            s=summary['scores']['overall'][m][p.ARMS[m][0]]['24']
            self.assertEqual((s['episodes'],s['closed'],s['wins']),(120,0,0))
            self.assertIsNone(s['win_rate']);self.assertIsNone(s['pf']);self.assertIsNone(s['mean_r'])
        self.assertFalse(any(g['passes_development_screen'] for g in summary['development_gates'].values()))
        models={m:{'status':'missing_test_source'} for m in p.MODELS}
        models['orb_60m']={'study':{'trade':{'entry':{'initial_r':1}},'marks':[{'reason':'simulated_entry'}]},'sensitivities':{a:{b:-0.4 for b in p.BPS} for a in p.ARMS['orb_60m']}}
        rows[0].update(status='complete',replay={'result':{'models':models,'quality':{'bid_ticks':2,'ask_ticks':2}}})
        s=p.summarize(rows)['scores']['overall']['orb_60m']['trailing']['24']
        self.assertEqual((s['closed'],s['wins'],s['mean_r'],s['pf']),(1,0,-0.4,0.0))
