"""Parse dated broker snapshots without backdating their authority."""
import datetime as dt
from decimal import Decimal
from zoneinfo import ZoneInfo

from rebuild_preflight import require
from replay_inventory import canonical


def local_to_utc(date, seconds, timezone):
    require(type(seconds) is int and 0 <= seconds <= 86400, 'holiday_seconds')
    local = dt.datetime.combine(date, dt.time())+dt.timedelta(seconds=seconds)
    zone = ZoneInfo(timezone)
    a, b = local.replace(tzinfo=zone, fold=0), local.replace(tzinfo=zone, fold=1)
    require(a.utcoffset() == b.utcoffset(), 'ambiguous_calendar_local_time')
    utc = a.astimezone(dt.timezone.utc)
    require(utc.astimezone(zone).replace(tzinfo=None) == local, 'nonexistent_calendar_local_time')
    return utc.isoformat()


def contract_snapshot(account, symbol, raw, captured_at):
    require(raw.get('symbolId') is not None and type(raw.get('digits')) is int, 'contract_identity')
    timezone = raw.get('scheduleTimeZone')
    require(isinstance(timezone, str) and bool(timezone), 'missing_broker_schedule_zone')
    ZoneInfo(timezone)
    schedule = sorted(raw.get('schedule', []), key=lambda v: v['startSecond'])
    for i, s in enumerate(schedule):
        a, b = s['startSecond'], s['endSecond']
        require(type(a) is int and type(b) is int and 0 <= a < b <= 7*86400, 'broker_weekly_interval')
        require(i == 0 or schedule[i-1]['endSecond'] <= a, 'overlapping_broker_intervals')
    holidays = []
    for h in raw.get('holiday', []):
        date = dt.date(1970, 1, 1)+dt.timedelta(days=int(h['holidayDate']))
        bounds = None
        if type(h.get('startSecond')) is int and type(h.get('endSecond')) is int:
            require(h['startSecond'] <= h['endSecond'], 'holiday_interval')
            bounds = {'from': local_to_utc(date, h['startSecond'], h['scheduleTimeZone']),
                      'to': local_to_utc(date, h['endSecond'], h['scheduleTimeZone'])}
        holidays.append({'holiday_id': h['holidayId'], 'date': date.isoformat(), 'name': h['name'],
            'timezone': h['scheduleTimeZone'], 'recurring': h['isRecurring'], 'utc_bounds': bounds,
            'interpretation': 'broker-declared exception at capture; not proof of earlier schedules or realised trading'})
    fields = ('minVolume', 'maxVolume', 'stepVolume', 'lotSize', 'measurementUnits',
              'preciseTradingCommissionRate', 'commissionType', 'preciseMinCommission',
              'swapLong', 'swapShort', 'swapCalculationType', 'swapPeriod', 'swapTime',
              'swapRollover3Days', 'rolloverCommission', 'rolloverCommission3Days',
              'pnlConversionFeeRate', 'skipSWAPPeriods', 'chargeSwapAtWeekends')
    return {'account_id': account, 'symbol': symbol, 'symbol_id': raw['symbolId'],
        'captured_at_utc': captured_at, 'broker_symbol_sha256': canonical(raw),
        'broker_timezone': timezone, 'weekly_intervals_since_sunday': schedule,
        'display_digits': raw['digits'], 'display_quantum': str(Decimal(10)**(-raw['digits'])),
        'minimum_tick_increment_status': 'not_a_field_in_ProtoOASymbol; display precision is not tick-size authority',
        'contract_fields': {k: raw.get(k) for k in fields}, 'dated_holidays': holidays,
        'historical_effective_from': None, 'historical_effective_to': None,
        'historical_2023_2025_authority': False,
        'funding_semantics': 'raw current broker fields only; dated historical rules and actual charges remain separate'}
