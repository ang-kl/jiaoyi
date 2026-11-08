"""Two explicit authority tiers. A replay proxy never becomes broker history."""
from broker_calendar import contract_snapshot
from rebuild_preflight import require
from replay_inventory import canonical, ms, permitted
from unified_replay import slots


def verify_current(account, symbol, raw, receipt_account, captured_at):
    require(account==receipt_account, 'calendar_account_receipt_mismatch')
    result=contract_snapshot(account,symbol,raw,captured_at)
    return {'tier':'A','status':'VERIFIED_CURRENT_SNAPSHOT','provenance':'authenticated_cTrader_capture',
        'account_id':account,'symbol':symbol,'symbol_id':raw['symbolId'],'captured_at':captured_at,
        'raw_symbol_sha256':canonical(raw),'schedule_timezone':result['broker_timezone'],
        'historical_authority':False,'facts':result}


def verify_historical(current, episode, metadata, decoded_bars, opening_quote, closing_quote):
    require(current['tier']=='A' and current['status']=='VERIFIED_CURRENT_SNAPSHOT','current_contract_unverified')
    require((current['account_id'],current['symbol'])==(episode['account_id'],episode['symbol']), 'historical_contract_join')
    session=metadata['session'];require(session is not None,'native_session_absent')
    start=ms(session['open']);end=start+14400000
    require(permitted(start,end+5001),'historical_protected_or_cutoff')
    required=slots(metadata['prior_45_calendar_days'])
    require(required and all(permitted(t,t+1800000) for t in required),'historical_protected_preparation')
    missing=[t for t in required if t not in decoded_bars]
    fresh=lambda q,t: bool(q) and t<=q['quote_ms']<=t+5000 and q['side_max_age_ms']<=6000
    window=episode['source_window_utc']
    reasons=[]
    if not (ms(window['from'])<=start and ms(window['to'])>=end):reasons.append('native_window_not_covered')
    if missing:reasons.append('native_m30_preparation_incomplete')
    if not fresh(opening_quote,start) or not fresh(closing_quote,end):reasons.append('fresh_boundary_missing')
    return {'tier':'B','status':'QUALIFIED_PROXY_REPLAY' if not reasons else 'REFUSED_PROXY_REPLAY',
        'provenance':'historical_iana_plus_current_broker_spec','historical_broker_authority':False,
        'episode_id':episode['episode_id'],'reasons':reasons,'required_native_bars':len(required),
        'missing_native_bars':len(missing),'native_session':session,'current_snapshot_sha256':current['raw_symbol_sha256'],
        'broker_holiday_maintenance_and_terms':'unverified historically; absent bars are not manufactured or silently exempted',
        'full_execution_path_qualified':False,'financial_promotion_allowed':False}
