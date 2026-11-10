import copy
import datetime as dt
import json
import unittest
from pathlib import Path

from monitoring.dispatch_evidence import summarize_dispatch, POLICY

START = '2026-11-09T00:00:00+00:00'
END = '2026-11-09T02:00:00+00:00'
AT = '2026-11-09T00:01:00+00:00'


def attempt(key='s3:test', events=None):
    return {'account_id': 43002148, 'entry_key': key, 'client_msg_id': 't1',
            'symbol_id': 1, 'side': 'buy', 'production_policy': POLICY,
            'decided_at': AT, 'created_at': AT, 'events': events or []}


def event(kind, ident=1, raw=None, **kw):
    return dict(id=ident, kind=kind, client_msg_id='t1', received_at='2026-11-09T00:01:01+00:00',
                raw_hex=json.dumps(raw).encode().hex() if raw is not None else None, **kw)


class DispatchEvidence(unittest.TestCase):
    def test_success_latency_identity_dedup_and_null_socket_measurement(self):
        raw = {'account_id': 43002148, 'symbol_id': 1, 'selected_side': 'buy',
               'policy': 'entry-dispatch-v1-2026-11-01', 'result': 'passed', 'stage': 'after_writer_lock',
               'checked_at': '2026-11-09T00:01:00.125+00:00', 'pullback_admitted_price': {'bounded': True}}
        row = attempt(events=[event('ENTRY_VALIDATED_BEFORE_SEND', raw=raw)])
        out = summarize_dispatch([row, copy.deepcopy(row)], START, END)['system_metrics']
        self.assertEqual(out['samples'], 1)
        self.assertEqual(out['max_touch_to_final_guard_ms'], 125)
        self.assertIsNone(out['max_dispatch_latency_ms'])
        wrong = copy.deepcopy(row)
        wrong['entry_key'] = 's3:wrong'
        raw['account_id'] = 7
        wrong['events'] = [event('ENTRY_VALIDATED_BEFORE_SEND', raw=raw)]
        malformed = attempt('s3:malformed', [event('ENTRY_VALIDATED_BEFORE_SEND')])
        malformed['events'][0]['raw_hex'] = 'ff'
        out = summarize_dispatch([wrong, malformed], START, END)['system_metrics']
        self.assertEqual(out['invalid_guard_receipts'], 2)
        self.assertEqual(out['samples'], 0)
        self.assertIsNone(out['max_touch_to_final_guard_ms'])

    def test_real_six_confirmed_turn_receipts_and_wrong_schema_rejection(self):
        rows = json.loads((Path(__file__).parent / 'fixtures/dispatch_18409_six.json').read_text())
        def run(rows):
            return summarize_dispatch(rows, '2026-11-09T11:47:00Z', '2026-11-09T22:49:35Z')['system_metrics']
        out = run(rows)
        self.assertEqual((out['samples'], out['invalid_guard_receipts']), (6, 0))
        self.assertLess(out['max_confirmation_to_final_guard_ms'], 20)
        self.assertGreater(out['max_touch_to_final_guard_ms'], 2000)
        for fault in ('unknown_policy', 'policy_mismatch', 'reversed_time', 'wrong_side', 'missing_proof'):
            row = copy.deepcopy(rows[0])
            event = row['events'][0]
            raw = json.loads(bytes.fromhex(event['raw_hex']))
            if fault == 'unknown_policy':
                row['production_policy'] = 's3_macro_pullback_owner_99999_v1'
            elif fault == 'policy_mismatch':
                raw['pullback_admitted_price']['policy'] = POLICY
            elif fault == 'reversed_time':
                raw['pullback_admitted_price']['confirmed_turn']['touched_at'] = '2027-01-01T00:00:00Z'
            elif fault == 'wrong_side':
                raw['pullback_admitted_price']['confirmed_turn']['long'] = True
            else:
                del raw['pullback_admitted_price']['confirmed_turn']
            event['raw_hex'] = json.dumps(raw).encode().hex()
            self.assertEqual(run([row])['invalid_guard_receipts'], 1, fault)

    def test_refusals_are_unique_fills_override_and_risk_caps_are_not_missing_fx(self):
        missing = {'code': 'ENTRY_FX_UNAVAILABLE', 'reason': 'unavailable',
                   'evidence': {'fx_reason': 'conversion quote stale/incomplete for broker symbol 7'}}
        fx = attempt('s3:fx', [event('REFUSED_BEFORE_SEND', raw=missing, error_code='ENTRY_FX_UNAVAILABLE')])
        risk = copy.deepcopy(missing)
        risk['evidence']['fx_reason'] = 'prepared candidate cash-risk limit exceeded'
        capped = attempt('s3:risk', [event('REFUSED_BEFORE_SEND', raw=risk, error_code='ENTRY_FX_UNAVAILABLE')])
        rejected = attempt('s3:reject', [event('ORDER_REJECTED', error_code='BAD_VOLUME', description='volume 0 is below the broker minimum 1')])
        partial = attempt('s3:partial', [event('ORDER_REJECTED'), event('ORDER_PARTIAL_FILL', 2, volume=1, position_id=4)])
        minrisk = attempt('s3:minrisk', [event('REFUSED_BEFORE_SEND', error_code='RISK', description='minimum-volume risk ceiling')])
        out = summarize_dispatch([fx, copy.deepcopy(fx), capped, rejected, partial, minrisk], START, END)['system_metrics']
        self.assertEqual(out['confirmed_refused_entries'], 3)
        self.assertEqual(out['broker_rejected_entries'], 1)
        self.assertEqual(out['partially_filled_entries'], 1)
        self.assertEqual(out['confirmed_missing_conversion_path_errors'], 1)
        self.assertEqual(out['confirmed_subminimum_volume_refusals'], 1)
        self.assertIsNone(out['unexpected_entry_refusals_count'])

    def test_empty_window_missing_events_and_boundary_do_not_fabricate_zero_latency(self):
        out = summarize_dispatch([], START, END)['system_metrics']
        self.assertEqual(out['confirmed_refused_entries'], 0)
        self.assertIsNone(out['fx_conversion_errors_count'])
        self.assertIsNone(out['max_touch_to_final_guard_ms'])
        late = event('REFUSED_BEFORE_SEND')
        late['received_at'] = END
        row = attempt(events=[late])
        out = summarize_dispatch([row], START, END)['system_metrics']
        self.assertEqual(out['confirmed_refused_entries'], 0)
        self.assertEqual(out['uncertain_entry_attempts'], 1)
        with self.assertRaises(ValueError):
            summarize_dispatch([], '2026-11-09', END)


if __name__ == '__main__':
    unittest.main()
