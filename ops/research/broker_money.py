"""Receipt-backed realised money, separate from unobserved execution costs.

Closing commission is the commission allocated to CLOSED volume. Opening deal
commission must not be charged a second time. Missing conversion/fees are not
zero. Decimal arithmetic retains broker precision and signed financing.
"""
import collections
import csv
from dataclasses import asdict, dataclass
from decimal import Decimal
from pathlib import Path
import re

from rebuild_evidence import RealisedMoney, number
from rebuild_preflight import require
from replay_inventory import canonical, sha


@dataclass(frozen=True)
class ClosingMoney:
    account_id: int
    environment: str
    position_id: int
    deal_id: int
    closed_volume: int
    money_digits: int
    gross: str
    swap: str
    commission: str
    conversion_fee: str
    net: str
    conversion_rate: str
    receipt_sha256: str


def closing_money(account, environment, deal):
    require(type(account) is int and account > 0 and environment in ('demo', 'live'), 'money_account')
    require(deal['dealStatus'] in (2, 3), 'unfilled_deal')
    c = deal['closePositionDetail']
    required = ('grossProfit', 'swap', 'commission', 'pnlConversionFee', 'moneyDigits', 'closedVolume')
    require(all(type(c.get(k)) is int for k in required), 'missing_or_noninteger_closing_money')
    require(0 <= c['moneyDigits'] <= 8 and c['closedVolume'] > 0, 'money_scale_or_quantity')
    require(c['closedVolume'] <= deal['filledVolume'], 'closed_volume_exceeds_fill')
    rate = number(c.get('quoteToDepositConversionRate'))
    require(rate > 0, 'missing_conversion_rate')
    scale = Decimal(10)**c['moneyDigits']
    gross, swap = Decimal(c['grossProfit'])/scale, Decimal(c['swap'])/scale
    commission, conversion = -abs(Decimal(c['commission']))/scale, -abs(Decimal(c['pnlConversionFee']))/scale
    return ClosingMoney(account, environment, int(deal['positionId']), int(deal['dealId']),
        c['closedVolume'], c['moneyDigits'], str(gross), str(swap), str(commission),
        str(conversion), str(gross+swap+commission+conversion), str(rate), canonical(deal))


def statements(files):
    """Deduplicate by account AND closing deal; preserve all statement receipts."""
    rows, sources = {}, []
    for ref in files:
        path = Path(ref['path'])
        require(path.exists() and sha(path) == ref['sha256'], 'statement_source_changed')
        section, header = None, None
        count = 0
        with path.open(encoding='utf-8-sig', newline='') as stream:
            for line, values in enumerate(csv.reader(stream), 1):
                if len(values) == 1:
                    section, header = values[0], None
                    continue
                if section != 'Deals' or not values:
                    continue
                if header is None:
                    header = values
                    continue
                if not re.fullmatch(r'DID\d+', values[0]):
                    continue
                require(len(values) == len(header), 'statement_width')
                raw = dict(zip(header, values))
                def dec(k):
                    return str(number(re.sub(r'[\s,]', '', raw[k])))
                row = {'account_id': ref['account_id'], 'deal_id': int(raw['Deal ID'][3:]),
                       'symbol': raw['Symbol'], 'commission': dec('Commissions'), 'net': dec('Net USD'),
                       'entry_price': dec('Entry price'), 'exit_price': dec('Closing price'),
                       'closing_quantity': raw['Closing quantity'],
                       'opening_direction': raw['Opening Direction'], 'closing_direction': raw['Closing Direction']}
                key = (row['account_id'], row['deal_id'])
                source = {'path': str(path), 'sha256': ref['sha256'], 'line': line}
                if key in rows:
                    require(rows[key]['row'] == row, 'conflicting_statement_deal')
                    rows[key]['sources'].append(source)
                else:
                    rows[key] = {'row': row, 'sources': [source]}
                count += 1
        sources.append({**ref, 'parsed_closes': count})
    return rows, sources


def ledger(account, environment, deals, names, contracts, statement_rows):
    require(len({d['dealId'] for d in deals}) == len(deals), 'duplicate_deal_id')
    groups = collections.defaultdict(list)
    for d in deals:
        groups[int(d['positionId'])].append(d)
    result = []
    for position, records in sorted(groups.items()):
        records.sort(key=lambda d: (d['executionTimestamp'], d['dealId']))
        opened = closed = 0
        running_qty, basis = 0, Decimal(0)
        side = None
        reversals = 0
        money, reasons, comparisons = [], [], []
        symbols = {int(d['symbolId']) for d in records}
        if len(symbols) != 1:
            reasons.append('position_symbol_conflict')
        symbol_id = min(symbols)
        for d in records:
            if d['dealStatus'] not in (2, 3):
                continue
            qty = int(d['filledVolume'])
            require(qty > 0, 'nonpositive_filled_volume')
            if 'closePositionDetail' not in d:
                opened += qty
                if side is not None and side != d['tradeSide']:
                    reasons.append('entry_side_reversal_requires_offset_matcher')
                side = d['tradeSide']
                basis += number(d.get('executionPrice'))*qty
                running_qty += qty
                continue
            try:
                m = closing_money(account, environment, d)
            except (ValueError, KeyError) as exc:
                reasons.append('closing_money_refused:'+str(exc))
                continue
            closed += m.closed_volume
            c = d['closePositionDetail']
            price = number(d['executionPrice'])
            qty = m.closed_volume
            precision = contracts.get(symbol_id, {}).get('digits')
            point = Decimal(10)**(-precision) if type(precision) is int else None
            # cents of base units and recorded quote/deposit conversion; this is
            # a diagnostic until historical contract semantics are established.
            sign = 1 if d['tradeSide'] == 2 else -1
            inferred_gross = sign*(price-number(c['entryPrice']))*qty/100*number(m.conversion_rate)
            unit = Decimal(10)**(-m.money_digits)
            comp = {'deal_id': m.deal_id, 'source_money': asdict(m),
                    'cent_unit_gross_diagnostic': str(inferred_gross),
                    'gross_difference': str(inferred_gross-number(m.gross)),
                    'gross_within_one_money_unit': abs(inferred_gross-number(m.gross)) <= unit,
                    'historical_contract_value_verified': False}
            if running_qty < qty or not running_qty:
                reasons.append('opening_quantity_missing_from_bounded_capture')
                comp['entry_basis_status'] = 'missing_opening_quantity'
            else:
                avg = basis/running_qty
                comp['matched_entry_vwap'] = str(avg)
                comp['entry_basis_status'] = ('matches_current_display_precision' if point is not None and
                    abs(avg-number(c['entryPrice'])) <= point else 'entry_basis_difference_or_unknown_precision')
                if side == d['tradeSide']:
                    reasons.append('closing_direction_conflict')
                basis -= avg*qty
                running_qty -= qty
                if running_qty == 0:
                    side = None
            # A netting reversal closes part of this fill and opens its remainder
            # at the SAME execution price. Count that quantity exactly once.
            residual = int(d['filledVolume'])-qty
            if residual:
                reversals += 1
                if running_qty:
                    reasons.append('reversal_without_flattened_prior_book')
                opened += residual
                running_qty += residual
                basis += price*residual
                side = d['tradeSide']
            stmt = statement_rows.get((account, m.deal_id))
            if stmt:
                s = stmt['row']
                def display_matches(reported, actual):
                    display_unit = Decimal(10)**number(reported).as_tuple().exponent
                    return abs(number(reported)-actual) <= display_unit/2
                money_matched = number(s['net']) == number(m.net) and number(s['commission']) == number(m.commission)
                matched = money_matched and display_matches(s['exit_price'], price) and display_matches(s['entry_price'], number(c['entryPrice']))
                matched = matched and s['symbol'] == names.get(symbol_id)
                comp['statement'] = {'status': 'matched' if matched else 'mismatch',
                                     'money_exact': money_matched, 'prices_at_reported_decimal_precision': True, **stmt}
                if not matched:
                    reasons.append('statement_money_price_or_symbol_mismatch')
            else:
                comp['statement'] = {'status': 'not_in_retained_statements'}
            comparisons.append(comp)
            money.append(m)
        quantity_complete = opened > 0 and opened == closed and running_qty == 0
        if not quantity_complete:
            reasons.append('not_a_complete_round_trip_in_captured_window')
        gross = sum((number(m.gross) for m in money), Decimal(0))
        net = sum((number(m.net) for m in money), Decimal(0))
        realised = RealisedMoney(str(gross), str(net-gross), str(net), canonical([asdict(m) for m in money]))
        result.append({'account_id': account, 'environment': environment, 'position_id': position,
            'symbol': names.get(symbol_id, 'unresolved'), 'symbol_id': symbol_id,
            'opening_volume': opened, 'closed_volume': closed, 'quantity_complete': quantity_complete,
            'netting_reversal_fills': reversals,
            'deal_ids': [d['dealId'] for d in records], 'labels': sorted({d.get('label','unlabelled') for d in records}),
            'closing_deals': comparisons, 'realised_money': asdict(realised),
            'money_evidence_complete': quantity_complete and not reasons and bool(money) and
                all(c['statement']['status']=='matched' for c in comparisons),
            'execution_cost_calibration_eligible': False,
            'missing_execution_evidence': ['both-leg dispatch quote/quantity/time and execution-event linkage',
                'historically effective contract value and order/strategy version'],
            'reasons': sorted(set(reasons))})
    return result
