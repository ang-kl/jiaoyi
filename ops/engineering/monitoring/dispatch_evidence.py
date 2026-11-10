"""Pure attribution of existing S3 intent events; never infers socket latency."""
from __future__ import annotations

from collections import Counter
import datetime as dt
import json
import math
import re

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
# Explicit schema registry: an unknown future policy is not automatically trusted.
POLICY = 's3_macro_pullback_owner_18392_v1'  # historical bounded-touch schema
POLICY_SCHEMAS = {
    's3_macro_pullback_owner_18377_v1': 'touch',
    's3_macro_pullback_owner_18380_v1': 'touch',
    POLICY: 'bounded',
    's3_macro_pullback_owner_18407_v1': 'confirmed_turn',
    's3_macro_pullback_owner_18409_v1': 'confirmed_turn',
}
FILLS = {'ORDER_FILLED', 'ORDER_PARTIAL_FILL', 'RECONCILED_FILL', 'RECONCILED_PARTIAL_FILL'}
FULL = {'ORDER_FILLED', 'RECONCILED_FILL'}
REJECTS = {'ORDER_REJECTED', 'ORDER_ERROR', 'ERROR_RES', 'ORDER_CANCELLED', 'ORDER_EXPIRED'}


def _clock(value):
    try:
        value = dt.datetime.fromisoformat(value.replace('Z', '+00:00')) if isinstance(value, str) else value
        return value.astimezone(dt.timezone.utc) if isinstance(value, dt.datetime) and value.tzinfo else None
    except (ValueError, OverflowError):
        return None


def _number(value):
    return type(value) in (int, float) and math.isfinite(value)


def _raw(event):
    raw = event.get('raw_hex')
    if not isinstance(raw, str) or len(raw) > 131072:
        return None
    try:
        value = json.loads(bytes.fromhex(raw).decode('utf-8'),
                           parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite_json')))
        return value if isinstance(value, dict) else None
    except (ValueError, UnicodeError):
        return None


def _admission_times(attempt, raw, checked, decided):
    """Decode the recorded proof schema, not a new execution permission."""
    admitted = raw.get('pullback_admitted_price')
    schema = POLICY_SCHEMAS.get(attempt.get('production_policy'))
    if not isinstance(admitted, dict) or checked is None or decided is None:
        return None
    if schema == 'bounded':
        return (decided, None) if admitted.get('bounded') is True else None
    if schema == 'touch':
        return (decided, None) if (_number(admitted.get('price')) and admitted['price'] > 0
            and type(admitted.get('digits')) is int and 0 <= admitted['digits'] <= 5) else None
    if schema != 'confirmed_turn' or admitted.get('policy') != attempt.get('production_policy'):
        return None
    proof = admitted.get('confirmed_turn')
    if not isinstance(proof, dict):
        return None
    touch, turn, confirmed = (_clock(proof.get(k)) for k in ('touched_at', 'turn_extreme_at', 'confirmed_at'))
    if (touch is None or turn is None or confirmed is None or not touch <= turn <= confirmed <= checked
            or type(proof.get('long')) is not bool or proof['long'] != (attempt.get('side') == 'buy')
            or not all(_number(proof.get(k)) and proof[k] > 0 for k in
                       ('origin', 'extreme', 'turn_extreme', 'confirmed_bid', 'confirmed_ask'))
            or proof['confirmed_bid'] > proof['confirmed_ask']
            or not _number(admitted.get('price')) or admitted['price'] <= 0):
        return None
    # PostgreSQL timestamps have microsecond precision; the JSON proof retains ns.
    if abs((confirmed - decided).total_seconds()) > 0.000001:
        return None
    return touch, confirmed


def _empty():
    return dict(confirmed_refused_entries=0, broker_rejected_entries=0,
                partially_filled_entries=0, uncertain_entry_attempts=0,
                confirmed_missing_conversion_path_errors=0, confirmed_subminimum_volume_refusals=0,
                max_touch_to_final_guard_ms=None, max_confirmation_to_final_guard_ms=None, samples=0, invalid_guard_receipts=0,
                persisted_attempts_observed=0, invalid_attempt_records=0, refusal_reasons={},
                max_dispatch_latency_ms=None, refused_entries=None,
                fx_conversion_errors_count=None, min_volume_refusal_count=None,
                unexpected_entry_refusals_count=None,
                refusal_scope='distinct persisted S3 opening intents; pre-admission failures and background vetoes are not counted',
                latency_scope='recorded touch and (for confirmed-turn policies) confirmation to final guard after writer lock; not socket-write duration')


def _missing_fx(code, raw):
    # This same code also covers margin/cash-risk breaches; those are not
    # evidence that an FX quote or conversion path was missing.
    reason = str((raw or {}).get('evidence', {}).get('fx_reason', '')).lower()
    return code == 'ENTRY_FX_UNAVAILABLE' and any(s in reason for s in (
        'conversion quote missing', 'conversion quote stale/incomplete',
        'broker conversion chain missing', 'conversion chain does not connect',
        'conversion chain identity is broken', 'conversion chain result invalid',
        'conversion identity or path length invalid'))


def summarize_dispatch(attempts, since, now):
    since, now = _clock(since), _clock(now)
    if since is None or now is None or since > now:
        raise ValueError('valid_aware_monitor_window_required')
    accounts = {str(a): _empty() for a in ACCOUNTS}
    grouped = {}
    for attempt in attempts:
        account, key = attempt.get('account_id'), attempt.get('entry_key')
        if account not in ACCOUNTS or not isinstance(key, str) or not key:
            continue
        identity = (account, key)
        if identity not in grouped:
            grouped[identity] = dict(attempt, events=list(attempt.get('events') or []))
        else:
            old = grouped[identity]
            fields = ('client_msg_id', 'symbol_id', 'side', 'production_policy', 'decided_at')
            if any(old.get(k) != attempt.get(k) for k in fields):
                old['_conflict'] = True
            old['events'].extend(attempt.get('events') or [])
    in_window = lambda at: at is not None and since <= at < now
    for (account, _), attempt in grouped.items():
        out = accounts[str(account)]
        out['persisted_attempts_observed'] += 1
        if attempt.get('_conflict'):
            out['invalid_attempt_records'] += 1
            continue
        events = {}
        for event in attempt['events']:
            at = _clock(event.get('received_at'))
            if at is None or at >= now:
                continue
            identity = event.get('id')
            if identity is None:
                identity = (event.get('kind'), event.get('received_at'), event.get('deal_id'), event.get('error_code'))
            events.setdefault(identity, event)
        events = list(events.values())
        current = [e for e in events if in_window(_clock(e.get('received_at')))]
        fills = [e for e in events if e.get('kind') in FILLS and _number(e.get('volume')) and e['volume'] > 0 and e.get('position_id') is not None]
        first_fill = _clock(attempt.get('first_fill_at'))
        any_fill = bool(fills) or (first_fill is not None and first_fill < now)
        if fills and any(e in current for e in fills) and not any(e.get('kind') in FULL for e in fills):
            out['partially_filled_entries'] += 1
        local = [e for e in current if e.get('kind') == 'REFUSED_BEFORE_SEND'
                 and attempt.get('client_msg_id') is not None
                 and e.get('client_msg_id') == attempt.get('client_msg_id')]
        rejected = [e for e in current if e.get('kind') in REJECTS]
        passed = [e for e in events if e.get('kind') == 'ENTRY_VALIDATED_BEFORE_SEND']
        # A later confirmed fill overrides an earlier rejection. Contradictory
        # local-unsent and successful-write records are not wholly refused.
        if not any_fill and local and not passed and not rejected:
            out['confirmed_refused_entries'] += 1
            reasons = set()
            fx = minimum = False
            for event in local:
                raw = _raw(event)
                code = str(event.get('error_code') or (raw or {}).get('code') or 'UNKNOWN')
                reason = str(event.get('description') or (raw or {}).get('reason') or 'unspecified')
                reasons.add(f'{code}: {reason}')
                fx |= _missing_fx(code, raw)
                minimum |= bool(re.search(r'volume\s+\d+\s+is below the broker minimum', reason, re.I))
            out['confirmed_missing_conversion_path_errors'] += int(fx)
            out['confirmed_subminimum_volume_refusals'] += int(minimum)
            out['refusal_reasons'] = dict(Counter(out['refusal_reasons']) + Counter(reasons))
        elif not any_fill and rejected:
            out['broker_rejected_entries'] += 1
            out['confirmed_subminimum_volume_refusals'] += int(any(re.search(
                r'volume\s+\d+\s+is below the broker minimum', str(e.get('description') or ''), re.I)
                for e in rejected))
            reasons = {f"{e.get('error_code') or e.get('kind')}: {e.get('description') or 'unspecified'}" for e in rejected}
            out['refusal_reasons'] = dict(Counter(out['refusal_reasons']) + Counter(reasons))
        elif not any_fill and (in_window(_clock(attempt.get('created_at'))) or current):
            out['uncertain_entry_attempts'] += 1
        # At most one successful guard sample per opening intent. Event window
        # is [start,end); the touch itself can precede start and is disclosed.
        valid = []
        for event in current:
            if event.get('kind') != 'ENTRY_VALIDATED_BEFORE_SEND':
                continue
            raw = _raw(event)
            checked = _clock((raw or {}).get('checked_at'))
            decided = _clock(attempt.get('decided_at'))
            received = _clock(event.get('received_at'))
            times = _admission_times(attempt, raw or {}, checked, decided)
            ok = (times is not None and raw is not None
                  and attempt.get('client_msg_id') is not None
                  and event.get('client_msg_id') == attempt.get('client_msg_id')
                  and attempt.get('side') in ('buy', 'sell')
                  and type(attempt.get('symbol_id')) is int and attempt['symbol_id'] > 0
                  and raw.get('policy') == 'entry-dispatch-v1-2026-11-01'
                  and raw.get('result') == 'passed' and raw.get('stage') == 'after_writer_lock'
                  and raw.get('account_id') == account and raw.get('symbol_id') == attempt.get('symbol_id')
                  and raw.get('selected_side') == attempt.get('side')
                  and checked is not None and decided is not None and decided <= checked <= received)
            if ok:
                valid.append((checked, (checked-times[0]).total_seconds()*1000,
                              (checked-times[1]).total_seconds()*1000 if times[1] is not None else None))
            else:
                out['invalid_guard_receipts'] += 1
        if valid:
            _, latency, confirmation_latency = max(valid, key=lambda v: v[0])
            out['samples'] += 1
            out['max_touch_to_final_guard_ms'] = max(out['max_touch_to_final_guard_ms'] or 0, latency)
            if confirmation_latency is not None:
                out['max_confirmation_to_final_guard_ms'] = max(out['max_confirmation_to_final_guard_ms'] or 0, confirmation_latency)
    system = _empty()
    sums = ('confirmed_refused_entries', 'broker_rejected_entries', 'partially_filled_entries',
            'uncertain_entry_attempts', 'confirmed_missing_conversion_path_errors',
            'confirmed_subminimum_volume_refusals', 'samples', 'invalid_guard_receipts',
            'persisted_attempts_observed', 'invalid_attempt_records')
    for name in sums:
        system[name] = sum(a[name] for a in accounts.values())
    values = [a['max_touch_to_final_guard_ms'] for a in accounts.values() if a['samples']]
    system['max_touch_to_final_guard_ms'] = max(values) if values else None
    confirmations = [a['max_confirmation_to_final_guard_ms'] for a in accounts.values()
                     if a['max_confirmation_to_final_guard_ms'] is not None]
    system['max_confirmation_to_final_guard_ms'] = max(confirmations) if confirmations else None
    system['refusal_reasons'] = dict(sum((Counter(a['refusal_reasons']) for a in accounts.values()), Counter()))
    system['source_counts'] = {k: system[k] for k in ('persisted_attempts_observed', 'samples', 'invalid_guard_receipts', 'invalid_attempt_records')}
    system['touch_to_final_guard_samples'] = system['samples']
    for out in accounts.values():
        out['touch_to_final_guard_samples'] = out['samples']
    system['sample_window'] = 'successful guard receipt recorded in [start,end); touch may precede start'
    return {'system_metrics': system, 'accounts': accounts}
