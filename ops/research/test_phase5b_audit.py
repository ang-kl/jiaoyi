import unittest
from phase5b_audit import window_check,exclusion_audit,force_planned_peer_failure

class Phase5BAuditTests(unittest.TestCase):
    def test_source_window_is_not_relabelled_as_native_session(self):
        def check(source,native):
            return window_check({'source_window_utc':{'from':'2024-08-12T'+source[0]+':00Z','to':'2024-08-12T'+source[1]+':00Z'}},
                {'session':{'open':'2024-08-12T'+native+':00Z'},'profile':{}})
        oil=check(('13:00','17:00'),'07:00')
        self.assertFalse(oil['covers']);self.assertEqual(oil['overlap_minutes'],0)
        index=check(('14:30','18:30'),'13:30')
        self.assertFalse(index['covers']);self.assertEqual(index['missing_open_minutes'],60)
        self.assertEqual(index['overlap_minutes'],180)
        self.assertTrue(check(('14:30','18:30'),'14:30')['covers'])

    def test_completed_exclusions_do_not_include_failed_sources(self):
        jobs=[{'key':str(i),'symbol':'BTCUSD','date':'2024-03-04','cohort':'fixture'} for i in range(160)]
        ends={'job-'+str(i):{'status':'failed' if i>=158 else 'complete','sha256':'fixture'} for i in range(160)}
        excluded=[{'key':str(i),'coverage':{'reasons':['missing_45day_fixed_window_M30'] if i<148 else ['missing_fresh_tick_boundary'] if i<158 else ['tick_source_failed','missing_45day_fixed_window_M30']}} for i in range(62,160)]
        r=exclusion_audit({'jobs':jobs,'bar_segments':[]},{'rows':[{'key':str(i)} for i in range(62)],'coverage':{'excluded':excluded}},ends)
        self.assertEqual(r['exclusive_completed_source_reasons'],{'missing_45day_fixed_window_M30':86,'missing_fresh_tick_boundary':10})
        self.assertEqual(r['completed_but_excluded']+r['evaluated_in_pr195']+r['failed_tick_sources'],160)

    def test_failed_planned_peer_cannot_become_absent(self):
        inp={'peer_context':{'status':'complete','drift_5':1.,'drift_20':1.},'peer_available':True}
        out=force_planned_peer_failure(inp,{'source_status':'failed'})
        self.assertTrue(out['peer_available'])
        self.assertEqual(out['peer_context']['status'],'planned_peer_source_failed')
        self.assertIsNone(out['peer_context']['drift_5'])
