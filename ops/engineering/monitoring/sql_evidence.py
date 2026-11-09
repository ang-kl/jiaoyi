"""Private read-only monitor facts. No broker calls, schema writes or inferred P&L."""
from __future__ import annotations

import datetime as dt
from decimal import Decimal, InvalidOperation
import json
import math
import os
from pathlib import Path
import subprocess
import time
from urllib.parse import parse_qsl, unquote, urlsplit

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
SYMBOLS = ('SpotCrude', 'SpotBrent', 'US500', 'NAS100', 'XAUUSD', 'XAGUSD', 'BTCUSD', 'ETHUSD')
FILL_KINDS = {'ORDER_FILLED', 'ORDER_PARTIAL_FILL', 'RECONCILED_FILL', 'RECONCILED_PARTIAL_FILL'}
SQL_PATH = Path(__file__).with_suffix('.sql')
MAX_OUTPUT = 8 * 1024 * 1024


def _clock(value):
    value = dt.datetime.fromisoformat(value.replace('Z', '+00:00')) if isinstance(value, str) else value
    if not isinstance(value, dt.datetime) or value.tzinfo is None:
        raise ValueError('aware_clock_required')
    return value.astimezone(dt.timezone.utc)


def _number(value):
    return type(value) in (int, float) and math.isfinite(value)


def _unavailable(reason):
    return {'status': 'unavailable', 'reason': reason, 'accounts': {}, 'system_metrics': {}}


def _environment(connection, timeout_ms):
    p = urlsplit(connection)
    if p.scheme not in ('postgres', 'postgresql') or not p.hostname or not p.username or not p.path[1:]:
        raise ValueError('invalid_database_configuration')
    ssl = {'sslmode': 'PGSSLMODE', 'sslrootcert': 'PGSSLROOTCERT', 'sslcert': 'PGSSLCERT',
           'sslkey': 'PGSSLKEY', 'sslcrl': 'PGSSLCRL', 'channel_binding': 'PGCHANNELBINDING'}
    options = parse_qsl(p.query, keep_blank_values=True, strict_parsing=True)
    if any(k not in ssl for k, _ in options) or len({k for k, _ in options}) != len(options):
        raise ValueError('unsupported_database_option')
    env = {k: v for k, v in os.environ.items() if k in ('PATH', 'LANG', 'LC_ALL', 'SSL_CERT_FILE', 'SSL_CERT_DIR')}
    env.update(PGHOST=unquote(p.hostname), PGPORT=str(p.port or 5432), PGUSER=unquote(p.username),
               PGDATABASE=unquote(p.path[1:]), PGPASSFILE='/dev/null', PGCONNECT_TIMEOUT='2',
               PGAPPNAME='jiaoyi-readonly-monitor-18393',
               PGOPTIONS=f'-c default_transaction_read_only=on -c statement_timeout={timeout_ms}')
    if p.password is not None:
        env['PGPASSWORD'] = unquote(p.password)
    env.update({ssl[k]: v for k, v in options})
    return env


def _cash(deal):
    """Same closing-side commission convention as Rust position_money()."""
    digits = deal.get('close_money_digits')
    if digits is None:
        digits = deal.get('money_digits')
    values = [deal.get(k) for k in ('close_gross_profit', 'close_swap', 'close_commission', 'close_pnl_conversion_fee')]
    if (deal.get('deal_status') not in ('FILLED', 'PARTIALLY_FILLED') or
            type(digits) is not int or not 0 <= digits <= 18 or any(type(v) is not int for v in values)):
        return None
    gross, swap, commission, fee = (Decimal(v) for v in values)
    return (gross + swap - abs(commission) - abs(fee)) / (Decimal(10) ** digits)


def _history(fetches, since, now):
    valid = []
    for f in fetches:
        if f.get('status') != 'ok' or f.get('has_error'):
            continue
        start, end, fetched = (_clock(f[k]) for k in ('window_from', 'window_to', 'fetched_at'))
        if start <= end <= now and end <= fetched <= now:
            valid.append((start, end, fetched))
    through = since
    newest = None
    for start, end, fetched in sorted(valid):
        if start <= through and end >= since:
            through = max(through, end)
            newest = max(newest or fetched, fetched)
    complete = through > since and (now - through).total_seconds() <= 600 and newest is not None and (now - newest).total_seconds() <= 600
    return {'history_complete': complete, 'history_through': through.isoformat() if through > since else None,
            'history_fetched_at': newest.isoformat() if newest else None,
            'history_reason': 'continuous recorded successful history through stated cutoff' if complete else 'history gap, missing receipt or stale cutoff',
            'history_max_age_secs': 600}


def _position(p):
    """Conservative cash lifecycle subset; official rolling-R stays the Rust assessment."""
    deals = p.get('deals') or []
    openings = [d for d in deals if not d['is_closing']]
    closing = [d for d in deals if d['is_closing']]
    opened = sum(d['filled_volume'] for d in openings)
    closed = sum(d['filled_volume'] for d in closing)
    receipts = p.get('receipts') or []
    own = [r for r in receipts if r.get('entry_key') == p['entry_key'] and r.get('intent_kind') == 'open']
    deal_volumes = {}
    for r in own:
        if r.get('deal_id') is not None:
            deal_volumes[r['deal_id']] = max(deal_volumes.get(r['deal_id'], 0), r['volume'])
    observed_open = max(sum(deal_volumes.values()), max((r['volume'] for r in own), default=0))
    reasons = []
    if p['owner_count'] != 1 or observed_open <= 0:
        reasons.append('opening_ownership_ambiguous')
    if opened < observed_open or any(d.get('deal_status') not in ('FILLED', 'PARTIALLY_FILLED') for d in deals):
        reasons.append('opening_history_or_status_incomplete')
    for d in openings:
        if (d.get('symbol_id') != p['symbol_id'] or str(d.get('side')).lower() != p['side'] or not any(
                r.get('deal_id') == d['deal_id'] or r.get('order_id') == d['order_id'] for r in own)):
            reasons.append('opening_increment_without_owned_receipt')
    for r in receipts:
        matching = [d for d in deals if d['deal_id'] == r.get('deal_id') and d['filled_volume'] >= r['volume']]
        if r.get('deal_id') is not None and not matching:
            reasons.append('execution_receipt_missing_from_history')
        if r.get('deal_id') is None and r.get('entry_key') is None:
            reasons.append('unclassified_execution_receipt')
    exits = {}
    for r in receipts:
        if r.get('intent_kind') == 'close':
            key = r.get('deal_id') if r.get('deal_id') is not None else r.get('entry_key')
            exits[key] = max(exits.get(key, 0), r['volume'])
    # Broker-exit rows have a durable deal id in their key. Validate each against history.
    for f in p.get('exits') or []:
        if f['kind'] == 'broker_exit':
            try:
                deal_id = int(f['entry_key'].rsplit(':', 1)[1])
            except (ValueError, IndexError):
                reasons.append('broker_exit_identity_unknown')
                continue
            if not any(d['deal_id'] == deal_id and d['filled_volume'] >= f['volume'] for d in closing):
                reasons.append('broker_exit_missing_from_history')
        elif not any(r.get('entry_key') == f['entry_key'] for r in receipts):
            reasons.append('exit_fill_without_matching_receipt')
    if sum(exits.values()) > closed:
        reasons.append('exit_receipts_exceed_history')
    if closed > opened or (closed == opened and (opened <= 0 or not p.get('requested_volume') or opened > p['requested_volume'])):
        reasons.append('closing_quantity_inconsistent')
    amounts = [_cash(d) for d in closing]
    if any(v is None for v in amounts):
        reasons.append('closing_costs_unverified')
    if reasons:
        return {'state': 'incomplete', 'reasons': sorted(set(reasons)), 'position_id': p['position_id']}
    if closed < opened:
        return {'state': 'open', 'position_id': p['position_id']}
    return {'state': 'closed', 'position_id': p['position_id'], 'symbol': p['symbol'],
            'closed_at': max(_clock(d['executed_at']) for d in closing).isoformat(),
            'net': float(sum(amounts, Decimal(0)))}


def _summary(rows):
    return {'closed_trades': len(rows), 'wins': sum(r['net'] > 0 for r in rows),
            'losses': sum(r['net'] < 0 for r in rows), 'breakevens': sum(r['net'] == 0 for r in rows),
            'closed_position_net_pnl': sum(r['net'] for r in rows)}


def assess(data, since, now):
    since, now = _clock(since), _clock(now)
    if data.get('transaction_read_only') != 'on':
        return _unavailable('readonly_transaction_unverified')
    from .dispatch_evidence import summarize_dispatch
    attempts = data.get('attempts') or []
    dispatch = summarize_dispatch(attempts, since, now)
    out = {'status': 'complete', 'window_start': since.isoformat(), 'observed_at': now.isoformat(),
           'accounts': {}, 'system_metrics': dispatch.get('system_metrics', {}),
           'scope': 'four accounts / eight core symbols; existing committed records only; read-only repeatable-read snapshot'}
    for account in ACCOUNTS:
        key = str(account)
        select = lambda name: [r for r in data.get(name, []) if r.get('account_id') == account]
        aa = select('attempts')
        coverage = _history(select('deal_fetches'), since, now)
        lifecycle = [_position(p) for p in select('positions')]
        closed = [p for p in lifecycle if p['state'] == 'closed' and since <= _clock(p['closed_at']) < now]
        incomplete = [p for p in lifecycle if p['state'] == 'incomplete']
        metrics = dict(dispatch.get('accounts', {}).get(key, {}))
        metrics.update(new_entries=sum(a.get('first_fill_at') is not None and since <= _clock(a['first_fill_at']) < now for a in aa),
                       persisted_entry_attempts=sum(since <= _clock(a['created_at']) < now for a in aa),
                       refused_entries=None)
        coverage.update(refusals_all_stages_complete=False, persisted_intent_events_complete=True,
                        new_entries_scope='distinct S3 first positive fill receipts in [start,end); partial fills count once',
                        incomplete_position_lifecycles=len(incomplete),
                        close_scope='complete owned core-S3 positions; lifetime net cash classified by last close timestamp',
                        realized_pnl_scope='closing-deal cash booked in interval, including partial closes; distinct from closed-position lifetime P&L')
        observed = _summary(closed)
        metrics.update({'observed_' + k: v for k, v in observed.items()})
        through = _clock(coverage['history_through']) if coverage['history_through'] else since
        qualified = [p for p in closed if _clock(p['closed_at']) <= through]
        metrics.update(_summary(qualified) if coverage['history_complete'] and not incomplete else {k: None for k in observed})
        cash = select('closing_cash')
        within = [d for d in cash if _clock(d['executed_at']) <= through]
        def cash_sum(rows):
            amounts = [_cash(r) for r in rows]
            return float(sum(amounts, Decimal(0))) if all(v is not None for v in amounts) else None
        core_cash = [d for d in within if d.get('owner_count') == 1 and d.get('symbol') in SYMBOLS]
        ambiguous = any(d.get('owner_count', 0) and d.get('owner_count') != 1 for d in within)
        coverage['cash_missing_cost_deals'] = sum(_cash(d) is None for d in within)
        metrics['net_realized_pnl'] = cash_sum(core_cash) if coverage['history_complete'] and not incomplete and not ambiguous else None
        metrics['account_net_realized_pnl'] = cash_sum(within) if coverage['history_complete'] else None
        metrics['observed_account_closing_cash_subtotal'] = float(sum((_cash(d) for d in cash if _cash(d) is not None), Decimal(0)))
        metrics['net_unrealized_pnl'] = None
        coverage['unrealized_reason'] = 'no strategy-specific broker net-unrealized snapshot; account float is separate'
        goal = next(iter(select('goals')), None)
        latest20 = (goal.get('detail') or {}).get('assessment') if goal else None
        if latest20 is not None:
            latest20 = {**latest20, 'policy_caveat': 'official per-account S3 latest20 can span registrations; not proof of the new policy alone',
                        'recorded_at': goal.get('at')}
        snapshots = select('account_snapshots')
        snapshot = snapshots[0] if snapshots else None
        symbols = {s: {} for s in SYMBOLS}
        for row in select('states'):
            dest = symbols[row['symbol']]
            if row['kind'] == 's3_session':
                dest['latest_macro'] = row
            elif row['kind'] == 's3_pullback_state':
                dest['latest_arm'] = row
            if row['kind'] != 's3_session' and (not dest.get('latest_decision') or _clock(row['at']) > _clock(dest['latest_decision']['at'])):
                dest['latest_decision'] = row
        out['accounts'][key] = {'metrics': metrics, 'coverage': coverage, 'symbols': symbols,
                                'currency': next((a.get('currency') for a in select('accounts')), None),
                                'latest20': latest20, 'account_snapshot': snapshot,
                                'recorded_decision_counts': select('decisions'),
                                'incomplete_lifecycles': incomplete}
    return out


def collect(database_url, since, now, timeout=3.5):
    """Run one fixed SELECT snapshot; URL/password never appear in argv or errors."""
    try:
        since, now = _clock(since), _clock(now)
        if not since <= now or (now - since).total_seconds() > 49 * 3600:
            return _unavailable('invalid_monitor_window')
        if not _number(timeout) or not 0.1 <= timeout <= 5:
            return _unavailable('invalid_query_timeout')
        timeout_ms = max(50, int((timeout - 0.1) * 1000))
        env = _environment(database_url, timeout_ms)
        result = subprocess.run(['psql', '-X', '-q', '-A', '-t', '-w', '--set=ON_ERROR_STOP=1',
                                 '--set', f'since={since.isoformat()}', '--set', f'until={now.isoformat()}',
                                 '--set', f'timeout_ms={timeout_ms}ms'],
                                input=SQL_PATH.read_text(), text=True, capture_output=True,
                                env=env, timeout=timeout, check=False)
        if result.returncode:
            return _unavailable('readonly_query_failed')
        if len(result.stdout.encode()) > MAX_OUTPUT:
            return _unavailable('evidence_output_limit')
        data = json.loads(result.stdout)
        return assess(data, since, now)
    except FileNotFoundError:
        return _unavailable('readonly_psql_unavailable')
    except subprocess.TimeoutExpired:
        return _unavailable('readonly_query_timeout')
    except (ValueError, TypeError, KeyError, OSError, OverflowError, InvalidOperation):
        return _unavailable('readonly_evidence_invalid')


def collect_sql_evidence(env, *, deadline, since=None, now=None):
    connection = env.get('LIVE_MONITOR_DATABASE_URL')
    if not connection:
        return _unavailable('live_monitor_database_url_missing')
    since = since or env.get('LIVE_MONITOR_STARTED_AT')
    if not since:
        return _unavailable('monitor_start_missing')
    remaining = deadline - time.monotonic()
    if remaining < 0.1:
        return _unavailable('readonly_query_deadline_elapsed')
    return collect(connection, since, now or dt.datetime.now(dt.timezone.utc), timeout=min(3.5, remaining))
