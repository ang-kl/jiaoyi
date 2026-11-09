"""Pure attribution of existing S3 intent events; never infers socket latency."""
from __future__ import annotations

from collections import Counter
import datetime as dt
import json
import math
import re

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
POLICY = 's3_macro_pullback_owner_18392_v1'
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


def _empty():
    return dict(confirmed_refused_entries=0, broker_rejected_entries=0,
                partially_filled_entries=0, uncertain_entry_attempts=0,
                confirmed_missing_conversion_path_errors=0, confirmed_subminimum_volume_refusals=0,
                max_touch_to_final_guard_ms=None, samples=0, invalid_guard_receipts=0,
                persisted_attempts_observed=0, invalid_attempt_records=0, refusal_reasons={},
                max_dispatch_latency_ms=None, refused_entries=None,
                fx_conversion_errors_count=None, min_volume_refusal_count=None,
                unexpected_entry_refusals_count=None,
                refusal_scope='distinct persisted S3 opening intents; pre-admission failures and background vetoes are not counted',
                latency_scope='touch receipt to successful final guard after writer lock, before socket bytes; not socket-write duration')


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
            admitted = (raw or {}).get('pullback_admitted_price')
            ok = (attempt.get('production_policy') == POLICY and raw is not None
                  and attempt.get('client_msg_id') is not None
                  and event.get('client_msg_id') == attempt.get('client_msg_id')
                  and attempt.get('side') in ('buy', 'sell')
                  and type(attempt.get('symbol_id')) is int and attempt['symbol_id'] > 0
                  and raw.get('policy') == 'entry-dispatch-v1-2026-11-01'
                  and raw.get('result') == 'passed' and raw.get('stage') == 'after_writer_lock'
                  and raw.get('account_id') == account and raw.get('symbol_id') == attempt.get('symbol_id')
                  and raw.get('selected_side') == attempt.get('side')
                  and isinstance(admitted, dict) and admitted.get('bounded') is True
                  and checked is not None and decided is not None and decided <= checked <= received)
            if ok:
                valid.append((checked, (checked-decided).total_seconds()*1000))
            else:
                out['invalid_guard_receipts'] += 1
        if valid:
            _, latency = max(valid)
            out['samples'] += 1
            out['max_touch_to_final_guard_ms'] = max(out['max_touch_to_final_guard_ms'] or 0, latency)
    system = _empty()
    sums = ('confirmed_refused_entries', 'broker_rejected_entries', 'partially_filled_entries',
            'uncertain_entry_attempts', 'confirmed_missing_conversion_path_errors',
            'confirmed_subminimum_volume_refusals', 'samples', 'invalid_guard_receipts',
            'persisted_attempts_observed', 'invalid_attempt_records')
    for name in sums:
        system[name] = sum(a[name] for a in accounts.values())
    values = [a['max_touch_to_final_guard_ms'] for a in accounts.values() if a['samples']]
    system['max_touch_to_final_guard_ms'] = max(values) if values else None
    system['refusal_reasons'] = dict(sum((Counter(a['refusal_reasons']) for a in accounts.values()), Counter()))
    system['source_counts'] = {k: system[k] for k in ('persisted_attempts_observed', 'samples', 'invalid_guard_receipts', 'invalid_attempt_records')}
    system['touch_to_final_guard_samples'] = system['samples']
    for out in accounts.values():
        out['touch_to_final_guard_samples'] = out['samples']
    system['sample_window'] = 'successful guard receipt recorded in [start,end); touch may precede start'
    return {'system_metrics': system, 'accounts': accounts}
