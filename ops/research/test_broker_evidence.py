import copy
import datetime as dt
import unittest

from broker_evidence import check_request, collect_deals
from broker_money import closing_money, ledger
from broker_calendar import contract_snapshot, local_to_utc
from rebuild_preflight import Refusal, Interval
from rebuild_reservation import partition_slots


def deal(identity, side, quantity, price, close=None):
    value = dict(dealId=identity, positionId=10, orderId=100+identity, symbolId=1,
                 volume=quantity, filledVolume=quantity, executionPrice=price,
                 executionTimestamp=identity, tradeSide=side, dealStatus=2, commission=100)
    if close:
        value['closePositionDetail'] = close
    return value


def money(quantity, entry, gross, commission=0, swap=0):
    return dict(closedVolume=quantity, entryPrice=entry, grossProfit=gross,
                commission=commission, swap=swap, pnlConversionFee=0,
                moneyDigits=2, quoteToDepositConversionRate=1)


class BrokerEvidenceTests(unittest.TestCase):
    def test_read_allowlist_blocks_prices_orders_other_accounts_and_unbounded_history(self):
        payload = dict(ctidTraderAccountId=49236051, fromTimestamp=10, toTimestamp=20, maxRows=1000)
        check_request(49236051, 2133, payload, 10, 20)
        for kind in (2106, 2107, 2110, 2111, 2128, 2137, 2145):
            with self.subTest(kind=kind), self.assertRaises(Refusal):
                check_request(49236051, kind, payload, 10, 20)
        for change in ({'ctidTraderAccountId': 43002148}, {'fromTimestamp': 0}, {'maxRows': 5000}, {'orderId': 12}):
            with self.assertRaises(Refusal):
                check_request(49236051, 2133, payload|change, 10, 20)

    def test_truncated_deal_page_bisects_and_retains_complete_time_denominator(self):
        class Broker:
            first, last, account = 10, 20, 49236051
            def __init__(self): self.calls = []
            def request(self, kind, payload):
                self.calls.append((payload['fromTimestamp'], payload['toTimestamp']))
                a, b = self.calls[-1]
                return {'hasMore': b-a > 5, 'deal': [{'dealId': a, 'executionTimestamp': a}]}
        broker = Broker()
        value = collect_deals(broker, lambda n, raw: {'path': str(n), 'sha256': 'a'*64})
        self.assertEqual(broker.calls, [(10,20), (10,15), (16,20)])
        self.assertEqual([d['raw']['dealId'] for d in value['deals']], [10,16])
        class Broken(Broker):
            def request(self, *_): return {'deal': []}
        with self.assertRaisesRegex(Refusal, 'truncation'):
            collect_deals(Broken(), lambda *_: {})

    def test_realised_money_has_signed_swap_and_no_duplicate_opening_commission(self):
        d = deal(2, 2, 100, 101, money(100,100,100,commission=15,swap=5))
        got = closing_money(49236051,'demo',d)
        self.assertEqual(got.net, '0.90')
        self.assertEqual(got.commission, '-0.15')
        broken = copy.deepcopy(d);del broken['closePositionDetail']['pnlConversionFee']
        with self.assertRaisesRegex(Refusal, 'missing'):
            closing_money(49236051,'demo',broken)

    def test_reversal_splits_closing_and_opening_volume_without_losing_money(self):
        rows = [deal(1,1,100,100), deal(2,2,200,101,money(100,100,100)),
                deal(3,1,100,99,money(100,101,200))]
        result = ledger(49236051,'demo',rows,{1:'US500'},{1:{'digits':2}}, {})[0]
        self.assertTrue(result['quantity_complete'])
        self.assertEqual((result['opening_volume'],result['closed_volume']), (200,200))
        self.assertEqual(result['realised_money']['net'], '3')
        self.assertEqual(result['netting_reversal_fills'], 1)
        self.assertEqual(result['reasons'], [])
        self.assertFalse(result['execution_cost_calibration_eligible'])

    def test_statement_price_rounding_does_not_relax_money_equality(self):
        ds = [deal(1,1,100,100.125), deal(2,2,100,101.125,money(100,100.125,100))]
        row = dict(account_id=49236051,deal_id=2,symbol='US500',commission='0.00',net='1.00',
                   entry_price='100.13',exit_price='101.13')
        statement = {(49236051,2): {'row':row,'sources':[]}}
        got = ledger(49236051,'demo',ds,{1:'US500'},{1:{'digits':2}},statement)[0]
        self.assertTrue(got['money_evidence_complete'])
        statement[(49236051,2)]['row']['net'] = '1.01'
        got = ledger(49236051,'demo',ds,{1:'US500'},{1:{'digits':2}},statement)[0]
        self.assertFalse(got['money_evidence_complete'])
        self.assertIn('statement_money_price_or_symbol_mismatch',got['reasons'])

    def test_current_broker_calendar_never_asserts_historical_authority(self):
        raw = dict(symbolId=1,digits=2,scheduleTimeZone='America/New_York',schedule=[dict(startSecond=100,endSecond=200)],
            holiday=[dict(holidayId=2,holidayDate=(dt.date(2026,11,26)-dt.date(1970,1,1)).days,
                name='Early close',scheduleTimeZone='Europe/Moscow',isRecurring=False,startSecond=81000,endSecond=86399)])
        got = contract_snapshot(49236051,'US500',raw,'2026-11-08T10:00:00Z')
        self.assertEqual(got['dated_holidays'][0]['utc_bounds']['from'], '2026-11-26T19:30:00+00:00')
        self.assertFalse(got['historical_2023_2025_authority'])
        self.assertIsNone(got['historical_effective_from'])
        self.assertEqual(local_to_utc(dt.date(2026,7,1),34200,'America/New_York'),'2026-07-01T13:30:00+00:00')
        self.assertEqual(local_to_utc(dt.date(2026,11,9),34200,'America/New_York'),'2026-11-09T14:30:00+00:00')

    def test_preparation_partitions_never_relabel_spent_missing_as_collectable(self):
        start = 1675209600000
        required = [start+i*1800000 for i in range(3)]
        rows = partition_slots(required,{start:{}},{start:{'source'}},{'source':'a'*64},
            [Interval('US500',49236051,start+1800000,start+3600000,'m30')],49236051,'US500')
        self.assertEqual([r['mode'] for r in rows], ['verified_cache','blocked_spent','unspent'])
        self.assertEqual(sum(r['to_ms']-r['from_ms'] for r in rows), 5400000)
        with self.assertRaisesRegex(Refusal, 'protected'):
            partition_slots([1755561600000],{},{},{},[],49236051,'US500')


if __name__ == '__main__':
    unittest.main()
