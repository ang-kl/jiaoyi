import copy
import datetime as dt
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from replay_inventory import canonical, sha, ms
from rebuild_preflight import (Refusal, Identity, verify_plan, verify_transition, split_budget_guard,
                               PartitionStore, CalendarTerms, verified_json)
import rebuild_evidence as evidence


def identity():
    return dict(candidate_id='frozen', hypothesis_sha256='1'*64, code_sha256='2'*64,
                calendar_sha256='3'*64, contract_sha256='4'*64, cost_sha256='5'*64,
                spent_registry_sha256='6'*64, rfc_state='offline-v1')


def preparation():
    start = ms('2024-03-11T13:30:00Z')
    times = [start - 3*86400000, start - 3*86400000 + 1800000]
    meta = {'e': {'symbol': 'US500', 'session': {'open': '2024-03-11T13:30:00Z'},
                  'prior_45_calendar_days': [{'open': '2024-03-08T13:30:00Z', 'close': '2024-03-08T14:30:00Z', 'lunch': None}]}}
    registry = dict(identity=identity(), transitions=[], horizon_ms=14400000, tail_ms=5001, spent_intervals=[])
    job = dict(episode_id='e', symbol='US500', account_id=1, **{'from': '2024-03-11T13:30:00Z', 'to': '2024-03-11T17:30:00Z'},
        required_m30_slots=times, preparation_segments=[dict(from_ms=times[0], to_ms=times[-1]+1800000, mode='verified_cache', source_path='bars', sha256='a'*64)],
        protocol_status='clean_first_read', source_path='ticks', tick_receipt_sha256='b'*64, tick_mode='verified_cache',
        path_coverage=dict(opening_fresh=True, closing_tail_fresh=True, from_ms=start, through_ms=start+14400000,
                           unexpected_gaps=0, ordering_ambiguous=False))
    plan = dict(identity=identity(), scheduled_episodes=1, episodes=[job])
    bars = {(1, 'US500'): {t: dict(ts_ms=t, low_e5=99, open_e5=100, close_e5=101, high_e5=102, volume=1) for t in times}}
    refs = {(1, 'US500'): {t: ['bars'] for t in times}}
    verified = {'bars': 'a'*64, 'ticks': 'b'*64}
    return plan, registry, meta, bars, refs, verified


class RebuildTests(unittest.TestCase):
    def test_preflight_blocks_pr195_defects_and_identity_overrides(self):
        fixture = preparation()
        self.assertEqual(len(verify_plan(*fixture, stage='evaluation')['episodes']), 1)
        mutations = [
            ('window', lambda p: p['episodes'][0].update({'from': '2024-03-11T14:30:00Z'})),
            ('45day', lambda p: p['episodes'][0]['required_m30_slots'].pop()),
            ('spent', lambda p: p['episodes'][0]['preparation_segments'][0].update(mode='blocked')),
            ('identity', lambda p: p['identity'].update(candidate_id='prompt_override')),
            ('boundary', lambda p: p['episodes'][0]['path_coverage'].update(closing_tail_fresh=False)),
            ('continuity', lambda p: p['episodes'][0]['path_coverage'].update(unexpected_gaps=1)),
            ('denominator', lambda p: p.update(scheduled_episodes=2)),
        ]
        for reason, mutate in mutations:
            args = copy.deepcopy(fixture); mutate(args[0])
            with self.subTest(reason=reason), self.assertRaises(Refusal):
                verify_plan(*args, stage='evaluation')
        args = copy.deepcopy(fixture)
        args[3][(1, 'US500')].pop(next(iter(args[3][(1, 'US500')])))
        with self.assertRaisesRegex(Refusal, 'decoded_complete'):
            verify_plan(*args, stage='ticks')
        args = copy.deepcopy(fixture)
        args[0]['episodes'][0]['preparation_segments'][0]['mode'] = 'unspent'
        verify_plan(*args, stage='preparation')
        with self.assertRaisesRegex(Refusal, 'decoded_before_ticks'):
            verify_plan(*args, stage='ticks')

    def test_pause_blocks_both_clients_before_any_network(self):
        from broker_readonly import Broker
        import market_collection
        import broker_backtest
        for call in [lambda: Broker().__enter__(), lambda: Broker().request(2145, {}),
                     lambda: market_collection.call('https://invalid', 'unused', {}),
                     lambda: broker_backtest.call('https://invalid', 'unused', {})]:
            with self.assertRaisesRegex(RuntimeError, 'research_collection_paused'):
                call()

    def test_offline_restriction_is_inherited_by_native_child(self):
        code = ('from rebuild_preflight import offline_process; offline_process(); '
                'import subprocess; r=subprocess.run(["python3","-c",'
                '"import socket; socket.socket()"],capture_output=True,text=True); '
                'assert r.returncode != 0 and "Operation not permitted" in r.stderr')
        result = subprocess.run([sys.executable, '-c', code], cwd=Path(__file__).parent, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_cache_hash_protected_alias_and_bounded_partition_dedup(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)/'source.json'; p.write_text('{}'); digest = sha(p)
            with self.assertRaisesRegex(Refusal, 'quarantined'):
                verified_json(p, digest, [digest])
            store = PartitionStore(Path(tmp)/'parts')
            ref = store.put(b'{"recorded":1}', 'fixture', '2030-01-01')
            again = store.put(b'{"recorded":1}', 'second_consumer', '2031-01-01')
            self.assertEqual(ref['path'], again['path'])
            self.assertEqual(store.read(ref), b'{"recorded":1}')
            Path(ref['path']).write_bytes(b'corrupt')
            with self.assertRaises(Refusal): store.read(ref)

    def test_dated_calendar_dst_early_close_and_unknown_terms(self):
        c = CalendarTerms('reference_venue', 'fixture-v1', 'America/New_York', '2024-01-01', '2024-12-31',
                          'declared_fixture_not_broker_authority', 'f'*64, {'2024-11-29': '13:00:00'}, (), '17:00:00')
        self.assertEqual(c.local_instant('2024-03-08', '09:30:00'), '2024-03-08T14:30:00+00:00')
        self.assertEqual(c.local_instant('2024-03-11', '09:30:00'), '2024-03-11T13:30:00+00:00')
        self.assertEqual(c.close('2024-11-29', '16:00:00'), '2024-11-29T18:00:00+00:00')
        self.assertEqual(c.local_instant('2024-11-04', c.funding_time), '2024-11-04T22:00:00+00:00')
        london = CalendarTerms('reference_venue', 'fixture-v1', 'Europe/London', '2024-01-01', '2024-12-31',
                               'declared_fixture_not_broker_authority', 'f'*64, {}, (), None)
        self.assertEqual(london.local_instant('2024-03-11', '08:00:00'), '2024-03-11T08:00:00+00:00')
        self.assertEqual(london.local_instant('2024-04-02', '08:00:00'), '2024-04-02T07:00:00+00:00')
        with self.assertRaisesRegex(Refusal, 'ambiguous'):
            c.local_instant('2024-11-03', '01:30:00')
        with self.assertRaisesRegex(Refusal, 'effective_dates'):
            c.close('2023-11-29', '16:00:00')
        unknown = CalendarTerms('broker', 'unverified', 'UTC', '2024-01-01', '2024-12-31', 'unknown', '', {}, (), None)
        with self.assertRaisesRegex(Refusal, 'unverified'):
            unknown.close('2024-11-29', '20:00:00')

    def test_trials_and_inspected_preparation_cannot_be_called_independent(self):
        dep = dict(symbol='US500', account_id=1, start_ms=ms('2024-01-01T00:00:00Z'), end_ms=ms('2024-01-02T00:00:00Z'), role='preparation')
        reservation = dict(candidate_id='c', maximum_trials=1, registered_candidates=['c'], dependencies=[dep],
                           embargo_ms=14400000, maximum_holding_ms=14400000, development_labels=[])
        trial = dict(trial_id='1', candidate_id='c', status='failed')
        split_budget_guard({'candidate_id': 'c'}, reservation, [], [trial])
        with self.assertRaisesRegex(Refusal, 'already_inspected_preparation'):
            split_budget_guard({'candidate_id': 'c'}, reservation, [dep], [trial])
        with self.assertRaisesRegex(Refusal, 'trial_budget'):
            split_budget_guard({'candidate_id': 'c'}, reservation, [], [trial, trial])
        changed = identity(); changed['candidate_id'] = 'new'
        with self.assertRaisesRegex(Refusal, 'unregistered_rfc_transition'):
            verify_transition(identity(), changed, {'approved': True}, [])


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.count = 0

    def receipt(self, value):
        self.count += 1
        p = Path(self.tmp.name)/(str(self.count)+'.json'); p.write_text(json.dumps(value))
        return dict(path=str(p), sha256=sha(p), pointer='')

    def round_trip(self, at=1000, position='p'):
        key = dict(account_id=1, environment='demo', symbol='US500', contract_version='v1', strategy_version='s3',
                   order_type='market', size_bucket='minimum', session_regime='native')
        deals = []
        for leg, side, price, commission, swap, conversion, offset in [
            ('entry', 'buy', '100', -100, 0, 0, 0), ('exit', 'sell', '110', -200, -50, -10, 100)]:
            d = dict(key=key, deal_id=position+leg, position_id=position, order_id=position+leg,
                     leg=leg, volume=10, price=price, side=side, decision_price=price, dispatch_price=price,
                     sent_ms=at+offset, acknowledged_ms=at+offset+1, filled_ms=at+offset+2, money_digits=2,
                     commission_units=commission, swap_units=swap, conversion_fee_units=conversion,
                     quote_to_deposit='1', settled=True)
            deals.append(d | {'receipt': self.receipt(d)})
        statement = dict(account_id=1, position_id=position, commission_convention='closing_total', money_digits=2,
                         gross='100', fees_and_financing='-2.60', net='97.40')
        contract = dict(account_id=1, environment='demo', symbol='US500', version='v1', point_value_per_native_unit='1',
                        effective_from_ms=0, effective_to_ms=10000000)
        return dict(key=key, position_id=position, deals=deals, commission_convention='closing_total',
                    statement=statement, statement_receipt=self.receipt(statement), contract_receipt=self.receipt(contract))

    def test_round_trip_money_fee_scaling_and_missing_or_conflicting_evidence(self):
        e = self.round_trip(); result = evidence.reconcile_position(e)
        self.assertEqual(evidence.number(result['money']['net']), evidence.number('97.40'))
        self.assertFalse(result['stress_subtracted_from_realised_money'])
        for name, mutate in [('missing', lambda x: x['deals'][1].update(swap_units=None)),
                             ('overfill', lambda x: x['deals'][1].update(volume=11)),
                             ('fee_convention', lambda x: x.update(commission_convention='per_leg')),
                             ('account', lambda x: x['key'].update(account_id=2)),
                             ('statement', lambda x: x['statement'].update(net='98.4'))]:
            v = copy.deepcopy(e); mutate(v)
            with self.subTest(name=name), self.assertRaises((Refusal, TypeError)):
                evidence.reconcile_position(v)

    def test_cost_model_requires_separate_reconciled_time_blocks(self):
        training = self.round_trip(at=1000, position='train')
        holdouts = [self.round_trip(at=3000, position='h1'), self.round_trip(at=4000, position='h2')]
        model = dict(key=training['key'], version='m1', as_of_ms=2000, mean_round_trip_bps='26',
                     sparse_fallback_bps=None, provenance='reconciled_round_trip')
        data = dict(training=[dict(round_trip=training, at_ms=1102, time_block='train')],
                    holdout=[dict(round_trip=v, at_ms=at, time_block=b) for v, at, b in zip(holdouts, [3102,4102], ['week1','week2'])],
                    maximum_error_bound_bps='1')
        self.assertEqual(evidence.verify_calibration(model, data)['status'], 'PASSED')
        model['provenance'] = '13_demo_entry_samples'
        with self.assertRaisesRegex(Refusal, 'entry_only'):
            evidence.verify_calibration(model, data)

    def test_execution_partial_fills_restart_and_actual_money_required(self):
        rt = self.round_trip(); key = rt['key']; events = []
        for n, (kind, oid, volume) in enumerate([('sent','pentry',10), ('ack','pentry',0), ('fill','pentry',4),
                                               ('fill','pentry',6), ('restart',None,0), ('reconciled',None,0),
                                               ('sent','pexit',10), ('ack','pexit',0), ('fill','pexit',10)]):
            e = dict(kind=kind, order_id=oid, volume=volume, key=key, at_ms=n, event_id=str(n))
            if kind == 'reconciled': e['broker_filled_volumes'] = {'pentry': 10}
            events.append(e)
        decisions = [dict(kernel_action='exit', observed_action='exit', latency_ms=5)]
        record = dict(mode='demo_canary', key=key, owner_authorisation_trace='fixture-only', events=events,
                      events_receipt=self.receipt(events), final_broker_filled_volumes={'pentry':10,'pexit':10},
                      decisions=decisions, decisions_receipt=self.receipt(decisions), maximum_latency_ms=10,
                      completed_positions=[rt])
        self.assertEqual(evidence.verify_execution_lifecycle(record)['status'], 'PASSED')
        bad = copy.deepcopy(record); bad['events'][1]['kind'] = 'fill'; bad['events'][1]['volume'] = 1
        bad['events_receipt'] = self.receipt(bad['events'])
        with self.assertRaisesRegex(Refusal, 'fill_before_ack'):
            evidence.verify_execution_lifecycle(bad)
        bad = copy.deepcopy(record); bad['completed_positions'] = []
        with self.assertRaisesRegex(Refusal, 'no_matched'):
            evidence.verify_execution_lifecycle(bad)

    def test_unknown_financial_and_operational_evidence_never_pass(self):
        with self.assertRaises(Refusal): evidence.financial_gate([], {})
        with self.assertRaises(Refusal): evidence.lifecycle_scorecard([], {}, {})
        with self.assertRaises(Refusal): evidence.number(float('nan'))

    def test_raw_ctrader_parser_retains_closed_volume_and_refuses_absent_fees(self):
        rt = self.round_trip(); key = rt['key']
        raw = dict(account_id=1, environment='demo', deal=dict(dealId=1, orderId=2, positionId=3, symbolId=4,
            dealStatus=2, filledVolume=10, volume=10, tradeSide=2, executionPrice=110, executionTimestamp=1102,
            closePositionDetail=dict(moneyDigits=2, commission=200, swap=-50, pnlConversionFee=10, closedVolume=10,
                                     quoteToDepositConversionRate=1, grossProfit=10000)))
        context = dict(key=key, order_id=2, position_id=3, symbol_id=4, leg='exit', decision_price=110,
                       dispatch_price=110, sent_ms=1100, acknowledged_ms=1101, settled=True)
        result = evidence.parse_proto_deal(self.receipt(raw), self.receipt(context))
        self.assertEqual((result['commission_units'],result['conversion_fee_units'],result['swap_units']),(-200,-10,-50))
        del raw['deal']['closePositionDetail']['pnlConversionFee']
        with self.assertRaisesRegex(Refusal,'raw_money_fields_missing'):
            evidence.parse_proto_deal(self.receipt(raw),self.receipt(context))


if __name__ == '__main__':
    unittest.main()
