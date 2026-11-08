"""Typed external-evidence admission for RFC G6–G12. Missing is never zero.

The input envelope carries raw immutable receipt references. Verifiers reconcile
the values themselves; a caller's `verified: true` flag is not accepted.
Money is signed deposit currency using Decimal. Closing-total commission must
not be charged again on the opening leg. Stress reserves are a different type.
"""
from dataclasses import dataclass, fields
from decimal import Decimal, InvalidOperation
import json
import math
import statistics
from pathlib import Path

from rebuild_preflight import Refusal, require, verified_json
from replay_inventory import canonical


def number(value):
    require(value is not None and not isinstance(value, bool), 'missing_money_or_numeric_value')
    try:
        v = Decimal(str(value))
    except (InvalidOperation, ValueError) as e:
        raise Refusal('invalid_decimal') from e
    require(v.is_finite(), 'nonfinite_number')
    return v


def parse(cls, value):
    require(isinstance(value, dict) and set(value) == {f.name for f in fields(cls)}, 'schema:' + cls.__name__)
    return cls(**value)


@dataclass(frozen=True)
class EvidenceKey:
    account_id: int
    environment: str
    symbol: str
    contract_version: str
    strategy_version: str
    order_type: str
    size_bucket: str
    session_regime: str

    def check(self):
        require(isinstance(self.account_id, int) and self.account_id > 0, 'account_identity')
        require(self.environment in ('demo', 'live'), 'environment_identity')
        require(all(isinstance(getattr(self, n), str) and getattr(self, n) for n in
                    ('symbol', 'contract_version', 'strategy_version', 'order_type', 'size_bucket', 'session_regime')),
                'cost_bucket_identity')


@dataclass(frozen=True)
class Receipt:
    path: str
    sha256: str
    pointer: str

    def extract(self):
        value = verified_json(self.path, self.sha256)
        require(self.pointer == '' or self.pointer.startswith('/'), 'receipt_pointer')
        for part in self.pointer.split('/')[1:]:
            key = part.replace('~1', '/').replace('~0', '~')
            value = value[int(key)] if isinstance(value, list) else value[key]
        return value


@dataclass(frozen=True)
class Deal:
    key: dict
    deal_id: str
    position_id: str
    order_id: str
    leg: str
    volume: int
    price: str
    side: str
    decision_price: str
    dispatch_price: str
    sent_ms: int
    acknowledged_ms: int
    filled_ms: int
    money_digits: int
    commission_units: int
    swap_units: int
    conversion_fee_units: int
    quote_to_deposit: str
    settled: bool
    receipt: dict

    def check(self):
        require(self.leg in ('entry', 'exit') and self.side in ('buy', 'sell'), 'deal_side_or_leg')
        require(isinstance(self.volume, int) and not isinstance(self.volume, bool) and self.volume > 0, 'deal_volume')
        require(self.sent_ms <= self.acknowledged_ms <= self.filled_ms, 'declared_ack_fill_order')
        require(self.settled is True and 0 <= self.money_digits <= 8, 'money_not_settled')
        for field in ('commission_units', 'swap_units', 'conversion_fee_units'):
            require(isinstance(getattr(self, field), int) and not isinstance(getattr(self, field), bool), 'money_integer_units')
        for field in ('price', 'decision_price', 'dispatch_price', 'quote_to_deposit'):
            require(number(getattr(self, field)) > 0, 'nonpositive_price_or_conversion')
        source = parse(Receipt, self.receipt).extract()
        # The adapter must retain the raw broker document and a normalised record
        # bound to it. Match every economic field, not merely the deal ID.
        expected = {f.name: getattr(self, f.name) for f in fields(self) if f.name != 'receipt'}
        require(source == expected, 'deal_receipt_fields_do_not_reconcile')


@dataclass(frozen=True)
class RealisedMoney:
    gross: str
    fees_and_financing: str
    net: str
    evidence_sha256: str


@dataclass(frozen=True)
class StressReserve:
    bps_per_side: str
    version: str


@dataclass(frozen=True)
class ExpectedExecutionCost:
    key: dict
    version: str
    as_of_ms: int
    mean_round_trip_bps: str
    sparse_fallback_bps: str | None
    provenance: str


def reconcile_position(envelope):
    require(set(envelope) == {'key', 'position_id', 'deals', 'commission_convention',
                            'statement', 'statement_receipt', 'contract_receipt'}, 'round_trip_schema')
    key = parse(EvidenceKey, envelope['key'])
    key.check()
    contract = parse(Receipt, envelope['contract_receipt']).extract()
    require(contract['account_id'] == key.account_id and contract['environment'] == key.environment
            and contract['symbol'] == key.symbol and contract['version'] == key.contract_version,
            'contract_identity')
    require(number(contract['point_value_per_native_unit']) > 0, 'contract_value_unverified')
    deals = [parse(Deal, v) for v in envelope['deals']]
    require(deals and len({d.deal_id for d in deals}) == len(deals), 'missing_or_duplicate_deals')
    for d in deals:
        d.check()
        require(d.key == envelope['key'], 'deal_account_or_environment_join')
        require(d.position_id == envelope['position_id'], 'position_join')
        require(contract['effective_from_ms'] <= d.filled_ms < contract['effective_to_ms'], 'historical_contract_unverified')
    entries = [d for d in deals if d.leg == 'entry']
    exits = [d for d in deals if d.leg == 'exit']
    require(entries and exits and sum(d.volume for d in entries) == sum(d.volume for d in exits), 'round_trip_quantity')
    require(len({d.side for d in entries}) == 1 and len({d.side for d in exits}) == 1
            and entries[0].side != exits[0].side, 'mixed_or_reversed_lifecycle')
    # Reversals/scale-in after an exit need a different versioned lot matcher.
    require(max(d.filled_ms for d in entries) <= min(d.filled_ms for d in exits), 'interleaved_lifecycle_requires_lot_matcher')
    convention = envelope['commission_convention']
    require(convention in ('per_leg', 'closing_total'), 'commission_convention_unknown')
    statement = envelope['statement']
    require(parse(Receipt, envelope['statement_receipt']).extract() == statement, 'statement_receipt_mismatch')
    require(statement['account_id'] == key.account_id and statement['position_id'] == envelope['position_id'], 'statement_identity')
    require(statement['commission_convention'] == convention, 'commission_convention_unreconciled')
    entry_price = sum(number(d.price) * d.volume for d in entries) / sum(d.volume for d in entries)
    sign = 1 if entries[0].side == 'buy' else -1
    gross = sum(sign * (number(d.price) - entry_price) * d.volume *
                number(contract['point_value_per_native_unit']) * number(d.quote_to_deposit) for d in exits)
    fees = Decimal(0)
    for d in deals:
        scale = Decimal(10) ** d.money_digits
        commission = d.commission_units if convention == 'per_leg' or d.leg == 'exit' else 0
        fees += (Decimal(commission) + d.swap_units + d.conversion_fee_units) / scale
    tolerance = Decimal(1) / (Decimal(10) ** statement['money_digits'])
    require(abs(gross - number(statement['gross'])) <= tolerance, 'statement_gross_mismatch')
    require(abs(fees - number(statement['fees_and_financing'])) <= tolerance, 'statement_fee_mismatch')
    require(abs(gross + fees - number(statement['net'])) <= tolerance, 'statement_net_mismatch')
    slippage = []
    for d in deals:
        direction = Decimal(1 if d.side == 'buy' else -1)
        slippage.append({'deal_id': d.deal_id, 'leg': d.leg,
            'dispatch_slippage_bps': str(direction * (number(d.price) / number(d.dispatch_price) - 1) * 10000),
            'decision_slippage_bps': str(direction * (number(d.price) / number(d.decision_price) - 1) * 10000),
            'send_to_ack_ms': d.acknowledged_ms - d.sent_ms, 'ack_to_fill_ms': d.filled_ms - d.acknowledged_ms})
    return {'key': envelope['key'], 'money': RealisedMoney(str(gross), str(fees), str(gross + fees), canonical(envelope)).__dict__,
            'deal_count': len(deals), 'slippage': slippage, 'spread_charged_again': False,
            'stress_subtracted_from_realised_money': False}


def verify_calibration(model_value, envelope):
    model = parse(ExpectedExecutionCost, model_value)
    key = parse(EvidenceKey, model.key)
    key.check()
    require(model.provenance == 'reconciled_round_trip', 'entry_only_or_assumed_cost_model')
    training = envelope['training']
    test = envelope['holdout']
    require(training and test, 'missing_cost_training_or_holdout')
    observed = []
    training_costs = []
    for kind, observations in [('training', training), ('holdout', test)]:
        for row in observations:
            joined = reconcile_position(row['round_trip'])
            require(joined['key'] == model.key, 'cost_bucket_or_environment_transfer')
            receipt_end = max(d['filled_ms'] for d in row['round_trip']['deals'])
            require(row['at_ms'] == receipt_end, 'cost_observation_timestamp')
            require((receipt_end <= model.as_of_ms) == (kind == 'training'), 'cost_asof_leakage')
            # Spread is already in executable prices. Round-trip slippage + actual
            # fee bps uses notional computed from the reconciled contract/entries.
            rt = row['round_trip']
            contract = parse(Receipt, rt['contract_receipt']).extract()
            entries = [d for d in rt['deals'] if d['leg'] == 'entry']
            notional = sum(number(d['price']) * d['volume'] * number(contract['point_value_per_native_unit']) *
                           number(d['quote_to_deposit']) for d in entries)
            require(notional > 0, 'cost_notional')
            volumes = {d['deal_id']: d['volume'] for d in rt['deals']}
            cost = sum(sum(number(s['dispatch_slippage_bps']) * volumes[s['deal_id']] for s in joined['slippage'] if s['leg'] == leg) /
                       sum(d['volume'] for d in rt['deals'] if d['leg'] == leg) for leg in ('entry', 'exit'))
            cost -= number(joined['money']['fees_and_financing']) / notional * 10000
            if kind == 'holdout':
                observed.append({'block': row['time_block'], 'cost_bps': float(cost)})
            else:
                training_costs.append(cost)
    fitted_mean = sum(training_costs) / len(training_costs)
    require(abs(fitted_mean-number(model.mean_round_trip_bps)) <= Decimal('0.00000001'),
            'cost_model_not_fitted_to_frozen_training')
    errors = [r['cost_bps'] - float(number(model.mean_round_trip_bps)) for r in observed]
    blocks = {b: statistics.mean([e for e, r in zip(errors, observed) if r['block'] == b]) for b in {r['block'] for r in observed}}
    require(len(blocks) >= 2, 'insufficient_independent_cost_blocks')
    uncertainty = 1.96 * statistics.stdev(blocks.values()) / math.sqrt(len(blocks))
    tolerance = float(number(envelope['maximum_error_bound_bps']))
    require(tolerance > 0, 'cost_tolerance_unregistered')
    bound = abs(statistics.mean(blocks.values())) + uncertainty
    passed = bound <= tolerance
    require(passed or model.sparse_fallback_bps is not None, 'sparse_cost_without_fallback')
    return {'status': 'PASSED' if passed else 'BLOCKED_UNCERTAIN_COST_FALLBACK_ONLY',
            'fitted_training_mean_bps': str(fitted_mean), 'training_samples': len(training_costs),
            'observations': len(errors), 'independent_blocks': len(blocks), 'mae_bps': statistics.mean(map(abs, errors)),
            'mean_error_bps': statistics.mean(errors), 'block_normal_95_halfwidth_bps': uncertainty,
            'uncertainty_method': 'normal approximation on predeclared independent time blocks; small-block limits apply',
            'error_bound_bps': bound, 'fallback_bps': model.sparse_fallback_bps,
            'calibration_sha256': canonical(envelope)}


def verify_execution_lifecycle(record):
    """Broker acknowledgements, partial fills, restart and final book reconciliation."""
    require(record['mode'] in ('shadow', 'demo_canary', 'live_canary'), 'execution_mode')
    key = parse(EvidenceKey, record['key']); key.check()
    require(record['mode'] == 'shadow' or bool(record['owner_authorisation_trace']), 'canary_not_authorised')
    events = record['events']
    require(events and events == parse(Receipt, record['events_receipt']).extract(), 'execution_receipt_mismatch')
    sent = {}; ack = set(); filled = {}; seen = set(); last = -1; unresolved = False
    for e in events:
        require(e['at_ms'] >= last and e['event_id'] not in seen, 'event_order_or_duplicate')
        last = e['at_ms']; seen.add(e['event_id'])
        require(e['key'] == record['key'], 'event_account_or_policy_join')
        kind = e['kind']; oid = e.get('order_id')
        if kind == 'sent':
            require(oid not in sent and not unresolved, 'duplicate_or_unreconciled_send')
            require(isinstance(e['volume'], int) and e['volume'] > 0, 'order_volume')
            sent[oid] = e['volume']
        elif kind == 'ack':
            require(oid in sent and oid not in ack, 'ack_without_unique_send')
            ack.add(oid)
        elif kind == 'fill':
            require(oid in ack and e['volume'] > 0, 'fill_before_ack')
            filled[oid] = filled.get(oid, 0) + e['volume']
            require(filled[oid] <= sent[oid], 'fill_over_quantity')
        elif kind == 'restart':
            unresolved = True
        elif kind == 'reconciled':
            require(e['broker_filled_volumes'] == filled, 'restart_book_mismatch')
            unresolved = False
        elif kind == 'reject':
            require(oid in sent and oid not in filled, 'invalid_reject')
        else:
            raise Refusal('unsupported_execution_event')
    require(not unresolved and filled == record['final_broker_filled_volumes'], 'unreconciled_final_book')
    require(record['decisions'] == parse(Receipt, record['decisions_receipt']).extract(), 'shadow_decisions_receipt')
    for row in record['decisions']:
        require(row['kernel_action'] == row['observed_action'] and row['latency_ms'] <= record['maximum_latency_ms'],
                'shadow_decision_or_latency_divergence')
    require(bool(record['decisions']), 'no_shadow_decisions')
    money = [reconcile_position(x) for x in record['completed_positions']]
    require(money and all(x['key'] == record['key'] for x in money), 'no_matched_lifecycle_money')
    deal_totals = {}
    unique_deals = set()
    for position in record['completed_positions']:
        for deal in position['deals']:
            require(deal['deal_id'] not in unique_deals, 'duplicate_lifecycle_deal')
            unique_deals.add(deal['deal_id'])
            oid = deal['order_id']
            deal_totals[oid] = deal_totals.get(oid, 0) + deal['volume']
    require(deal_totals == filled, 'execution_fills_and_money_deals_disagree')
    return {'status': 'PASSED', 'events': len(events), 'reconciled_positions': len(money), 'receipt_sha256': canonical(record)}


def financial_gate(rows, criteria):
    """Never accept a supplied win count. Trade records retain account/class keys."""
    required = ('minimum_closes', 'minimum_independent_blocks', 'minimum_win_rate',
                'minimum_mean_net_r', 'maximum_tail_loss_r', 'minimum_control_delta_r')
    require(all(k in criteria for k in required), 'financial_criteria_not_frozen')
    require(rows and len({r['trade_id'] for r in rows}) == len(rows), 'missing_or_duplicate_trades')
    buckets = {}
    for row in rows:
        require(row['evidence_kind'] == 'independent_verified' and row['resolved'] is True,
                'financial_evidence_not_independent_or_resolved')
        require({k: v for k, v in row.items() if k != 'receipt'} == parse(Receipt, row['receipt']).extract(), 'trade_receipt_mismatch')
        key = (row['account_id'], row['strategy_id'], row['asset_class'])
        buckets.setdefault(key, []).append(row)
    outputs = []
    for key, group in buckets.items():
        returns = [float(number(r['net_r'])) for r in group]
        blocks = {b: statistics.mean(float(number(r['net_r'])) for r in group if r['time_block'] == b)
                  for b in {r['time_block'] for r in group}}
        lower = (statistics.mean(blocks.values()) - 1.96 * statistics.stdev(blocks.values()) / math.sqrt(len(blocks))) if len(blocks) >= 2 else None
        control_delta = statistics.mean(float(number(r['net_r']) - number(r['matched_control_r'])) for r in group)
        passed = (len(group) >= criteria['minimum_closes'] and len(blocks) >= criteria['minimum_independent_blocks']
                  and sum(r > 0 for r in returns) / len(returns) >= criteria['minimum_win_rate']
                  and lower is not None and lower > criteria['minimum_mean_net_r']
                  and min(returns) >= -criteria['maximum_tail_loss_r']
                  and control_delta > criteria['minimum_control_delta_r'])
        outputs.append({'key': key, 'status': 'PASSED' if passed else 'BLOCKED_FINANCIAL_EVIDENCE',
                        'closes': len(group), 'mean_net_r': statistics.mean(returns), 'block_lower_bound_r': lower})
    return outputs


def lifecycle_scorecard(rows, policy, operational_receipt):
    require(set(policy) == {'live_max_positions', 'broker_minimum_only', 'entry_margin_floor_percent'}, 'owner_policy_schema')
    require(policy['live_max_positions'] == 1 and policy['broker_minimum_only'] is True,
            'owner_policy_changed_without_registration')
    require(policy['entry_margin_floor_percent'] == 250, 'owner_margin_policy_changed')
    actual = parse(Receipt, operational_receipt).extract()
    for check in ('entry_pause_preserves_exits', 'worker_failure_preserves_protection',
                  'restart_reconciles_before_entry', 'rollback_preserves_original_plan',
                  'existing_pending_and_manual_risk_counted', 'monitor_stale_fails_closed'):
        require(actual.get(check) is True, 'operational_lifecycle_missing:' + check)
    grouped = {}
    for row in rows:
        result = reconcile_position(row['round_trip'])
        require(number(row['original_r_money']) > 0, 'original_r_unverified')
        key = (result['key']['account_id'], result['key']['strategy_version'])
        grouped.setdefault(key, []).append((row['closed_ms'], number(result['money']['net']) / number(row['original_r_money'])))
    require(grouped, 'no_account_scorecards')
    output = []
    for key, group in grouped.items():
        selected = sorted(group)[-20:]; values = [r for _, r in selected]
        winners = [r for r in values if r > 0]; losses = [-r for r in values if r < 0]
        payoff = (sum(winners) / len(winners)) / (sum(losses) / len(losses)) if winners and losses else None
        output.append({'account_strategy': key, 'closed': len(values), 'wins': len(winners),
            'goal_pass': len(values) == 20 and len(winners) >= 15 and payoff is not None and payoff >= Decimal('1.38'),
            'realised_payoff_ratio': str(payoff) if payoff is not None else None})
    return output


def read_envelope(path):
    p = Path(path)
    require(p.stat().st_size <= 32_000_000, 'evidence_size_budget')
    return json.loads(p.read_text(), parse_constant=lambda _: (_ for _ in ()).throw(Refusal('nonfinite_json')))


def parse_proto_deal(deal_receipt, dispatch_receipt):
    """Parse captured ProtoOADeal plus account-scoped dispatch context, offline.

    Raw ProtoOADeal has no account ID. The transport receipt must retain the
    authenticated account and environment, not infer them from symbolId.
    Optional absent monetary fields are rejected, including an absent zero fee.
    """
    envelope = parse(Receipt, deal_receipt).extract()
    context = parse(Receipt, dispatch_receipt).extract()
    key = parse(EvidenceKey, context['key']); key.check()
    require(envelope['account_id'] == key.account_id and envelope['environment'] == key.environment,
            'raw_deal_account_join')
    raw = envelope['deal']
    require(raw['orderId'] == context['order_id'] and raw['positionId'] == context['position_id']
            and raw['symbolId'] == context['symbol_id'], 'raw_deal_dispatch_join')
    require(raw['dealStatus'] in (2, 3), 'unfilled_deal_not_a_cost_sample')
    require(0 < raw['filledVolume'] <= raw['volume'], 'raw_filled_volume')
    close = raw.get('closePositionDetail')
    require((close is not None) == (context['leg'] == 'exit'), 'raw_deal_leg')
    money = close if close is not None else raw
    required = ('moneyDigits', 'commission') + (('swap', 'pnlConversionFee', 'closedVolume',
                                                'quoteToDepositConversionRate', 'grossProfit') if close else ())
    require(all(k in money and money[k] is not None for k in required), 'raw_money_fields_missing')
    if close:
        require(close['closedVolume'] == raw['filledVolume'], 'raw_closed_volume')
    else:
        require(all(k in context for k in ('entry_swap_units', 'entry_conversion_fee_units', 'quote_to_deposit')),
                'entry_money_context_missing')
    require(raw['tradeSide'] in (1, 2), 'raw_trade_side')
    return {'key': context['key'], 'deal_id': str(raw['dealId']), 'order_id': str(raw['orderId']),
        'position_id': str(raw['positionId']), 'leg': context['leg'], 'side': 'buy' if raw['tradeSide'] == 1 else 'sell',
        'volume': raw['filledVolume'], 'price': str(raw['executionPrice']),
        'decision_price': str(context['decision_price']), 'dispatch_price': str(context['dispatch_price']),
        'sent_ms': context['sent_ms'], 'acknowledged_ms': context['acknowledged_ms'], 'filled_ms': raw['executionTimestamp'],
        'money_digits': money['moneyDigits'], 'commission_units': -abs(money['commission']),
        'swap_units': close['swap'] if close else context['entry_swap_units'],
        'conversion_fee_units': -abs(close['pnlConversionFee']) if close else context['entry_conversion_fee_units'],
        'quote_to_deposit': str(close['quoteToDepositConversionRate']) if close else str(context['quote_to_deposit']),
        'settled': context['settled']}
