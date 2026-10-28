"""Private, bounded statement and owned-position evidence; never a broker writer."""
import datetime as dt
import hashlib
import html
from html.parser import HTMLParser
import json
import math
import os
import subprocess
from urllib.parse import parse_qsl, unquote, urlsplit

ACCOUNTS = (43002148, 46130058, 47790949, 49236051)
# Only the reviewed v6 registration is replayed with this midpoint rule.
# Historical side-quote rules and future registrations remain unknown here.
MIDPOINT_REGISTRATION = 'd54344b9d463532c3b1e2d81bd07d671ed0d2fceae9ee923f1c8988b18b2ab32'
MAX_OUTPUT = 2 * 1024 * 1024
SSL_ENV = {
    'sslmode': 'PGSSLMODE', 'sslrootcert': 'PGSSLROOTCERT', 'sslcert': 'PGSSLCERT',
    'sslkey': 'PGSSLKEY', 'sslcrl': 'PGSSLCRL', 'sslcrldir': 'PGSSLCRLDIR',
    'ssl_min_protocol_version': 'PGSSLMINPROTOCOLVERSION',
    'ssl_max_protocol_version': 'PGSSLMAXPROTOCOLVERSION', 'channel_binding': 'PGCHANNELBINDING',
}
SQL = r"""
BEGIN READ ONLY;
SET LOCAL statement_timeout='8000ms';
WITH statement AS (
 SELECT report_key,period_from,period_to,status,sent_at,message_id,
 length(html) AS html_chars,left(html,100000) AS html
 FROM telegram_reports WHERE status='sent' AND period_to<=:'until'::timestamptz
 ORDER BY period_to DESC LIMIT 1
), bounds AS (
 SELECT LEAST(:'since'::timestamptz,COALESCE((SELECT period_to FROM statement),:'since'::timestamptz)) AS since
), owners AS (
 SELECT e.account_id,e.position_id,count(DISTINCT i.entry_key) AS owner_count,min(i.id) AS intent_id,
 min(e.received_at) AS opened_at,max(e.received_at) AS last_open_receipt_at
 FROM trading_events e JOIN trading_intents i ON i.account_id=e.account_id AND i.entry_key=e.entry_key
 WHERE e.account_id IN (43002148,46130058,47790949,49236051) AND i.kind='open'
 AND (i.entry_key LIKE 's1:%' OR i.entry_key LIKE 's3:%')
 AND e.position_id IS NOT NULL AND e.volume>0 AND e.received_at<=:'until'::timestamptz
 AND e.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 GROUP BY e.account_id,e.position_id
), candidates AS (
 SELECT o.* FROM owners o WHERE EXISTS (
 SELECT 1 FROM trading_events e,bounds b WHERE e.account_id=o.account_id AND e.position_id=o.position_id
 AND e.received_at>=b.since AND e.received_at<=:'until'::timestamptz)
 OR EXISTS(SELECT 1 FROM deals d,bounds b WHERE d.account_id=o.account_id AND d.position_id=o.position_id
 AND d.executed_at>=b.since AND d.executed_at<=:'until'::timestamptz)
 ORDER BY o.opened_at DESC,o.account_id,o.position_id LIMIT 200
), positions AS (
 SELECT o.account_id,o.position_id,o.owner_count,o.opened_at,i.entry_key,i.side,i.volume AS requested_volume,
 i.relative_stop,i.stop_price AS intended_stop,i.take_profit AS intended_tp,i.risk,i.max_risk,i.settings_version,
 CASE WHEN i.entry_key LIKE 's3:%' THEN 's3' ELSE 's1' END AS strategy,
 COALESCE(s3.symbol,s1.symbol,s.name) AS symbol,COALESCE(s3.prereg_hash,s1.prereg_hash) AS prereg_hash,
 CASE WHEN i.entry_key LIKE 's3:%' THEN jsonb_build_object('entry',s3.detail->'entry','order',s3.detail->'order',
 'band',s3.detail->'band','session',s3.detail->'session','bid',s3.bid,'ask',s3.ask)
 ELSE jsonb_build_object('ema_fast',sd.detail->'ema_fast','ema_slow',sd.detail->'ema_slow',
 'level',sd.detail->'level','atr',sd.detail->'atr','bid',sd.detail->'bid','ask',sd.detail->'ask') END AS setup,
 (SELECT GREATEST(COALESCE(MAX(e.volume),0),COALESCE((SELECT SUM(x.volume) FROM (
 SELECT DISTINCT ON(e2.deal_id) e2.volume FROM trading_events e2
 WHERE e2.account_id=o.account_id AND e2.position_id=o.position_id AND e2.entry_key=i.entry_key
 AND e2.volume>0 AND e2.deal_id IS NOT NULL AND e2.received_at<=:'until'::timestamptz
 AND e2.kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')
 ORDER BY e2.deal_id,e2.id DESC) x),0)) FROM trading_events e
 WHERE e.account_id=o.account_id AND e.position_id=o.position_id AND e.entry_key=i.entry_key
 AND e.received_at<=:'until'::timestamptz AND e.kind IN
 ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL')) AS opening_receipt_volume,
 (SELECT jsonb_agg(x ORDER BY x.received_at) FROM (SELECT e.kind,e.received_at,e.price,e.volume,e.stop_loss,
 e.error_code,e.entry_key,ti.kind AS intent_kind,ti.stop_price AS requested_stop,ti.take_profit AS requested_tp
 FROM trading_events e LEFT JOIN trading_intents ti ON ti.account_id=e.account_id AND ti.entry_key=e.entry_key
 WHERE e.account_id=o.account_id AND e.position_id=o.position_id AND e.received_at<=:'until'::timestamptz
 ORDER BY e.received_at DESC,e.id DESC LIMIT 40) x) AS events,
 (SELECT jsonb_agg(x ORDER BY x.at) FROM (SELECT f.kind,f.outcome,COALESCE(f.answered_at,f.sent_at) AS at,
 f.fill_price,f.slippage,f.latency_ms,f.volume,f.detail->>'reason' AS reason
 FROM strategy3_fills f WHERE f.account_id=o.account_id AND f.position_id=o.position_id
 AND COALESCE(f.answered_at,f.sent_at)<=:'until'::timestamptz
 UNION ALL SELECT f.kind,f.outcome,COALESCE(f.answered_at,f.sent_at),f.fill_price,f.slippage,f.latency_ms,
 f.volume,f.detail->>'reason' FROM strategy1_fills f WHERE f.account_id=o.account_id AND f.position_id=o.position_id
 AND COALESCE(f.answered_at,f.sent_at)<=:'until'::timestamptz ORDER BY at DESC LIMIT 40) x) AS fills,
 (SELECT jsonb_build_object('deals',COUNT(*),'opening_volume',COALESCE(SUM(filled_volume) FILTER(WHERE NOT is_closing),0),
 'closing_volume',COALESCE(SUM(filled_volume) FILTER(WHERE is_closing),0),
 'closing_deals',COUNT(*) FILTER(WHERE is_closing),'first_deal_at',MIN(executed_at),'last_deal_at',MAX(executed_at),
 'last_close_at',MAX(executed_at) FILTER(WHERE is_closing),
 'valid',COALESCE(BOOL_AND(deal_status IN ('FILLED','PARTIALLY_FILLED')),false),
 'all_closing_costs_explicit',COALESCE(BOOL_AND(NOT is_closing OR (close_gross_profit IS NOT NULL
 AND close_swap IS NOT NULL AND close_commission IS NOT NULL AND close_pnl_conversion_fee IS NOT NULL
 AND COALESCE(COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18,false))),false),
 'known_closing_cash_net',SUM(CASE WHEN is_closing AND close_gross_profit IS NOT NULL AND close_swap IS NOT NULL
 AND close_commission IS NOT NULL AND close_pnl_conversion_fee IS NOT NULL
 AND COALESCE(close_money_digits,money_digits) BETWEEN 0 AND 18 THEN
 (close_gross_profit::float8+close_swap::float8-abs(close_commission::float8)-abs(close_pnl_conversion_fee::float8))
 /power(10::float8,COALESCE(close_money_digits,money_digits)) END))
 FROM deals d WHERE d.account_id=o.account_id AND d.position_id=o.position_id AND d.filled_volume>0
 AND d.executed_at<=:'until'::timestamptz AND d.fetched_at<=:'until'::timestamptz) AS money
 FROM candidates o JOIN trading_intents i ON i.id=o.intent_id
 LEFT JOIN symbols s ON s.account_id=i.account_id AND s.broker_symbol_id=i.symbol_id
 LEFT JOIN strategy3_orders s3 ON s3.entry_key=i.entry_key AND s3.account_id=i.account_id AND s3.kind='open'
 LEFT JOIN strategy1_fills s1 ON s1.entry_key=i.entry_key AND s1.account_id=i.account_id AND s1.kind='open'
 LEFT JOIN LATERAL(SELECT detail FROM trading_decisions WHERE account_id=i.account_id
 AND kind='sent' AND detail->>'entry_key'=i.entry_key ORDER BY id LIMIT 1) sd ON true
)
SELECT jsonb_build_object('effective_since',(SELECT since FROM bounds),
 'statement_report',(SELECT to_jsonb(s) FROM statement s),
 'accounts',(SELECT jsonb_agg(jsonb_build_object('account_id',a.account_id,'environment',a.environment,
 'currency',a.currency,'observed_at',r.observed_at,'snapshot',jsonb_build_object(
 'observed_at',r.detail->'observed_at','source_complete',r.detail->'source_complete','balance_stable',r.detail->'balance_stable',
 'balance',r.detail->'balance','float',r.detail->'float','equity',r.detail->'equity','open_risk',r.detail->'open_risk',
 'open_positions',r.detail->'open_positions','pending_orders',r.detail->'pending_orders','margin_level',r.detail->'margin_level',
 'broker_leverage',r.detail->'broker_leverage','failure_reasons',r.detail->'failure_reasons')) ORDER BY a.account_id)
 FROM accounts a LEFT JOIN report_account_snapshots r ON r.account_id=a.account_id AND r.sample_key='latest'
 WHERE a.account_id IN (43002148,46130058,47790949,49236051)),
 'lifecycle',COALESCE((SELECT jsonb_agg(to_jsonb(p) ORDER BY opened_at,account_id,position_id) FROM positions p),'[]'::jsonb),
 'candidate_limit',200,'scope','unique program opening receipts; bounded post-statement execution/history evidence, not current broker protection');
ROLLBACK;
"""


def _clock(value):
    value = dt.datetime.fromisoformat(value.replace('Z', '+00:00')) if isinstance(value, str) else value
    if not isinstance(value, dt.datetime) or value.tzinfo is None:
        raise ValueError('aware evidence clock required')
    return value.astimezone(dt.timezone.utc)


def _finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def _psql_environment(connection):
    """libpq does not expand a connection URI supplied through PGDATABASE."""
    parsed = urlsplit(connection)
    database = unquote(parsed.path[1:]) if parsed.path.startswith('/') else ''
    if parsed.scheme not in ('postgres', 'postgresql') or not parsed.hostname or not parsed.username or not database:
        raise ValueError('invalid PostgreSQL connection URI')
    options = parse_qsl(parsed.query, keep_blank_values=True, strict_parsing=True)
    if any(key not in SSL_ENV for key, _ in options) or len({key for key, _ in options}) != len(options):
        raise ValueError('unsupported PostgreSQL connection option')
    # Never inherit a service/host-address/password-file selector that could
    # override this explicitly configured connection. Do not read those files.
    environment = {key: value for key, value in os.environ.items() if key not in (
        'PGHOST', 'PGHOSTADDR', 'PGPORT', 'PGUSER', 'PGPASSWORD', 'PGDATABASE',
        'PGSERVICE', 'PGSERVICEFILE', 'PGPASSFILE', *SSL_ENV.values())}
    environment.update(PGHOST=unquote(parsed.hostname), PGPORT=str(parsed.port or 5432),
        PGUSER=unquote(parsed.username), PGDATABASE=database, PGCONNECT_TIMEOUT='2',
        PGOPTIONS='-c default_transaction_read_only=on -c statement_timeout=8000')
    if parsed.password is not None:
        environment['PGPASSWORD'] = unquote(parsed.password)
    environment.update({SSL_ENV[key]: value for key, value in options})
    return environment


def _setup_checks(position):
    """Replay saved arithmetic only, without claiming a validated market edge."""
    setup = position.get('setup') or {}
    entry = setup.get('entry') or {}; band = setup.get('band') or {}
    side = position.get('side'); sign = 1 if side == 'buy' else -1 if side == 'sell' else None
    result = {'registration': 'UNKNOWN', 'band': 'UNKNOWN', 'direction': 'UNKNOWN',
              'trigger': 'UNKNOWN', 'original_sl': 'UNKNOWN', 'original_tp': 'UNKNOWN'}
    if position.get('ownership_state') != 'PASS': return result
    def state(value): return 'PASS' if value else 'FAIL'
    if position.get('strategy') == 's3':
        keys = ('open', 'sigma', 'lower', 'upper', 'half_width')
        if all(_finite(band.get(key)) for key in keys):
            o, sigma, lower, upper, half = (band[key] for key in keys)
            close = lambda a, b: math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
            result['band'] = state(o > 0 and sigma > 0 and half > 0 and close(half, o * sigma)
                                   and close(lower, o - half) and close(upper, o + half))
        if sign is not None and entry.get('side') in ('buy', 'sell'):
            result['direction'] = state(entry['side'] == side)
        bid, ask = setup.get('bid'), setup.get('ask')
        if (position.get('prereg_hash') == MIDPOINT_REGISTRATION and sign is not None
                and all(_finite(x) for x in (bid, ask, band.get('lower'), band.get('upper')))):
            mid = (bid + ask) / 2
            result['trigger'] = state(ask >= bid and ask - bid <= band['upper'] - band['lower']
                and (mid > band['upper'] if sign == 1 else mid < band['lower']))
        relative, saved = position.get('relative_stop'), entry.get('relative_stop')
        if type(relative) is int and type(saved) is int:
            result['original_sl'] = state(relative > 0 and relative == saved)
        target = ((setup.get('order') or {}).get('plan') or {}).get('broker_target')
        price = entry.get('entry_price')
        if sign is not None and _finite(target) and _finite(price):
            result['original_tp'] = state(price > 0 and target > 0 and sign * (target - price) > 0)
    elif position.get('strategy') == 's1':
        fast, slow = setup.get('ema_fast'), setup.get('ema_slow')
        if sign is not None and _finite(fast) and _finite(slow):
            result['direction'] = state(sign * (fast - slow) > 0)
        level, quote = setup.get('level'), setup.get('ask' if sign == 1 else 'bid')
        if sign is not None and _finite(level) and _finite(quote):
            result['trigger'] = state(sign * (quote - level) >= 0)
        result['original_tp'] = state(position.get('intended_tp') is None)
    return result


class _Text(HTMLParser):
    def __init__(self):
        super().__init__(); self.parts = []; self.hidden = 0

    def handle_starttag(self, tag, attrs):
        if tag in ('style', 'script'): self.hidden += 1
        elif tag in ('p', 'tr', 'h1', 'h2', 'h3', 'li'): self.parts.append('\n')

    def handle_endtag(self, tag):
        if tag in ('style', 'script'): self.hidden = max(0, self.hidden - 1)
        elif tag in ('td', 'th'): self.parts.append(' | ')

    def handle_data(self, data):
        if not self.hidden: self.parts.append(data)


def _assess(data, since, until):
    statement = data.get('statement_report') or {}
    body = statement.pop('html', '')
    parser = _Text(); parser.feed(body if isinstance(body, str) else '')
    statement['text'] = '\n'.join(' '.join(line.split()) for line in ''.join(parser.parts).splitlines() if line.strip())[:6000]
    statement['html_complete'] = isinstance(body, str) and statement.get('html_chars') == len(body)
    statement['sha256'] = hashlib.sha256(body.encode()).hexdigest() if statement['html_complete'] else None
    rows = {row.get('account_id'): row for row in data.get('accounts') or []}
    accounts = []
    for account in ACCOUNTS:
        row = rows.get(account, {'account_id': account}); snap = row.pop('snapshot', None) or {}
        try: age = (until - _clock(row.get('observed_at'))).total_seconds()
        except (ValueError, TypeError): age = None
        try: matching_clock = _clock(snap.get('observed_at')) == _clock(row.get('observed_at'))
        except (ValueError, TypeError): matching_clock = False
        fresh = age is not None and 0 <= age <= 180 and matching_clock
        complete = fresh and snap.get('source_complete') is True
        row.update(source_complete=complete, source_age_seconds=age,
                   balance=snap.get('balance') if fresh and snap.get('balance_stable') is True and _finite(snap.get('balance')) else None,
                   floating=snap.get('float') if complete and _finite(snap.get('float')) else None,
                   equity=snap.get('equity') if complete and _finite(snap.get('equity')) else None,
                   open_risk=snap.get('open_risk') if complete and _finite(snap.get('open_risk')) else None,
                   open_positions=snap.get('open_positions'), pending_orders=snap.get('pending_orders'),
                   margin_level=snap.get('margin_level') if complete else None,
                   failure_reasons=snap.get('failure_reasons') or ([] if complete else ['snapshot_missing_stale_or_incomplete']))
        accounts.append(row)
    positions = data.get('lifecycle') or []
    for position in positions:
        owned = position.get('owner_count') == 1
        position['ownership_state'] = 'PASS' if owned else 'UNKNOWN'
        position['saved_setup_checks'] = _setup_checks(position)
        risk, cap = position.get('risk'), position.get('max_risk')
        position['entry_risk_check'] = 'UNKNOWN' if not owned or not (_finite(risk) and _finite(cap)) else 'PASS' if 0 < risk <= cap else 'FAIL'
        events = position.get('events') or []
        position['last_saved_broker_stop'] = next((row.get('stop_loss') for row in reversed(events)
            if row.get('kind') in ('ORDER_FILLED', 'ORDER_PARTIAL_FILL', 'ORDER_REPLACED') and _finite(row.get('stop_loss'))), None)
        money = position.get('money') or {}
        if money.get('all_closing_costs_explicit') is not True or money.get('valid') is not True:
            money['known_closing_cash_net'] = None
        # Balanced recorded volumes do not establish history coverage. The full
        # position benchmark remains unknown here; the qualified goal API owns it.
        position['fully_closed_in_recorded_deals'] = bool(owned and money.get('valid') is True
            and money.get('opening_volume', 0) > 0 and money.get('opening_volume') == money.get('closing_volume')
            and money.get('opening_volume') == position.get('opening_receipt_volume'))
        position['position_net_r'] = None
        position['net_r_reason'] = 'Use qualified goal evidence: this bounded reader does not establish complete opening-to-close history coverage'
        position['reverse_state'] = 'UNKNOWN'
        position['reverse_reason'] = 'No explicit linked close-and-reverse receipt is exposed; opposite entries alone are not reversals'
    return {'state': 'available', 'as_of': until.isoformat(), 'monitoring_since': since.isoformat(),
            'effective_since': data.get('effective_since'), 'scope': data.get('scope'),
            'statement_report': statement, 'accounts': accounts, 'lifecycle': positions,
            'checks': [{'name': 'stored statement report', 'state': 'PASS' if statement.get('status') == 'sent' else 'UNKNOWN',
                        'observed': bool(statement), 'reason': 'Stored deterministic account HTML; not a new broker statement download'},
                       {'name': 'owned lifecycle', 'state': 'UNKNOWN' if any(p['ownership_state'] != 'PASS' for p in positions) else 'PASS' if positions else 'NOT_EXERCISED',
                        'observed': len(positions), 'reason': 'At most200 positions; only unique original intent/fill ownership qualifies'},
                       {'name': 'reverse', 'state': 'UNKNOWN' if positions else 'NOT_EXERCISED',
                        'observed': False, 'reason': 'No explicit linked reversal evidence in this source'}],
            'limits': ['At most200 owned-position candidates and40 saved events/fills each; current protection uses the separate broker receipt view',
                       'Account cash includes all account activity; strategy attribution requires unique program ownership',
                       'No complete-history certificate or causal market explanation is manufactured from bounded rows']}


def collect(coordinator, since, until):
    since, until = _clock(since), _clock(until)
    if since > until: raise ValueError('evidence window reversed')
    connection = coordinator.config.env.get('ENGINEERING_VALIDATION_DATABASE_URL')
    if not connection:
        return {'state': 'unknown', 'as_of': until.isoformat(), 'scope': 'private statement/lifecycle',
                'reason': 'private_readonly_database_not_configured', 'accounts': [], 'lifecycle': [], 'checks': [], 'statement_report': {}}
    def unavailable(reason):
        return {'state': 'unknown', 'as_of': until.isoformat(), 'scope': 'private statement/lifecycle',
                'reason': reason, 'accounts': [], 'lifecycle': [], 'checks': [], 'statement_report': {}}
    try:
        environment = _psql_environment(connection)
    except (ValueError, TypeError):
        return unavailable('private_readonly_configuration_invalid')
    try:
        result = subprocess.run(['psql', '-X', '-q', '-A', '-t', '--set=ON_ERROR_STOP=1',
            '--set=since=' + since.isoformat(), '--set=until=' + until.isoformat()], input=SQL,
            env=environment, text=True, capture_output=True, timeout=10, check=True)
        if len(result.stdout.encode()) > MAX_OUTPUT: raise ValueError('bounded evidence exceeded')
        data = json.loads(result.stdout)
        if not isinstance(data, dict): raise ValueError('expected object')
        return _assess(data, since, until)
    except subprocess.CalledProcessError as error:
        # Fixed categories only: stderr can contain a connection credential.
        category = 'connection_failed' if error.returncode == 2 else 'query_failed' if error.returncode == 3 else 'process_failed'
        return unavailable('private_readonly_' + category)
    except subprocess.TimeoutExpired:
        return unavailable('private_readonly_timeout')
    except FileNotFoundError:
        return unavailable('private_readonly_psql_missing')
    except (OSError, ValueError, subprocess.SubprocessError, TypeError):
        return unavailable('private_readonly_invalid_response')


def render(evidence):
    """Private HTML only. All source text is escaped, never interpreted."""
    esc = lambda value: html.escape(str(value if value is not None else 'unknown'))
    if evidence.get('state') != 'available':
        return '<h2>Account statements and lifecycle</h2><p>UNKNOWN: ' + esc(evidence.get('reason')) + '.</p>'
    statement = evidence.get('statement_report') or {}
    out = ['<h2>Account statement evidence</h2><p>Stored report: ' + esc(statement.get('report_key'))
           + '; period ' + esc(statement.get('period_from')) + ' → ' + esc(statement.get('period_to'))
           + '; delivery ' + esc(statement.get('status')) + '; message ' + esc(statement.get('message_id'))
           + '. Post-statement lifecycle window begins ' + esc(evidence.get('effective_since')) + '.</p>',
           '<table><tr><th>Account</th><th>Currency</th><th>Balance</th><th>Float</th><th>Equity</th><th>Open risk</th><th>Source</th></tr>']
    for row in evidence.get('accounts') or []:
        out.append('<tr>' + ''.join('<td>' + esc(row.get(key)) + '</td>' for key in
            ('account_id', 'currency', 'balance', 'floating', 'equity', 'open_risk', 'source_complete')) + '</tr>')
    out += ['</table><h2>Own setups and entries</h2>']
    positions = evidence.get('lifecycle') or []
    if not positions: out.append('<p>NOT EXERCISED: no bounded program-owned execution/history candidate since the statement cutoff.</p>')
    out.append('<p>Observed candidate positions: ' + str(len(positions)) + '; showing at most20. Checks below assess saved arithmetic, not full registration eligibility or predictive edge.</p>')
    for row in positions[:20]:
        setup = row.get('setup') or {}; entry = setup.get('entry') or {}; band = setup.get('band') or {}
        out.append('<p><b>' + esc(row.get('account_id')) + ' / ' + esc(row.get('strategy')) + ' / '
            + esc(row.get('symbol')) + ' / position ' + esc(row.get('position_id')) + '</b>: '
            + esc(row.get('side')) + '; opened ' + esc(row.get('opened_at')) + '; ownership '
            + esc(row.get('ownership_state')) + '; gate risk/cap ' + esc(row.get('risk')) + '/' + esc(row.get('max_risk'))
            + ' (' + esc(row.get('entry_risk_check')) + '); original hash ' + esc(row.get('prereg_hash'))
            + '; original band ' + esc(band.get('lower')) + '–' + esc(band.get('upper'))
            + '; saved decision bid/ask ' + esc(setup.get('bid')) + '/' + esc(setup.get('ask'))
            + '; saved entry price ' + esc(entry.get('entry_price'))
            + '; original R ' + esc(entry.get('r')) + '. Saved arithmetic: '
            + esc('; '.join(key + '=' + value for key, value in (row.get('saved_setup_checks') or {}).items()))
            + '. Registration remains UNKNOWN until original applicable rules are independently matched.</p>')
        for fill in [f for f in row.get('fills') or [] if f.get('kind') == 'open'][-4:]:
            out.append('<p>Entry receipt: ' + esc(fill.get('at')) + '; outcome ' + esc(fill.get('outcome'))
                + '; actual fill ' + esc(fill.get('fill_price')) + '; slippage ' + esc(fill.get('slippage'))
                + '; latency ms ' + esc(fill.get('latency_ms')) + '; filled volume ' + esc(fill.get('volume')) + '.</p>')
    out += ['<h2>SL, TP and closes</h2>']
    for row in positions[:20]:
        fills = row.get('fills') or []; money = row.get('money') or {}
        closes = [f for f in fills if f.get('kind') in ('broker_exit', 'tp1', 'session_end')]
        setup = row.get('setup') or {}; entry = setup.get('entry') or {}
        price, relative = entry.get('entry_price'), row.get('relative_stop')
        original_stop = (price - relative / 100000 if row.get('side') == 'buy' else price + relative / 100000) if (
            _finite(price) and type(relative) is int and relative > 0 and row.get('side') in ('buy', 'sell')) else None
        plan_target = (((setup.get('order') or {}).get('plan') or {}).get('broker_target'))
        out.append('<p>' + esc(row.get('account_id')) + '/' + esc(row.get('position_id'))
            + ': original relative SL ' + esc(row.get('relative_stop')) + '; intended TP ' + esc(row.get('intended_tp'))
            + '; original decision-price SL ' + esc(original_stop) + '; saved plan broker target ' + esc(plan_target)
            + (' (S1 originally has no TP)' if row.get('strategy') == 's1' else '')
            + '; last saved broker SL ' + esc(row.get('last_saved_broker_stop'))
            + ' (historical receipt, not current protection); closing deals ' + esc(money.get('closing_deals'))
            + '; explicit costs ' + esc(money.get('all_closing_costs_explicit'))
            + '; known closing cash subtotal ' + esc(money.get('known_closing_cash_net'))
            + '; recorded volume fully closed ' + esc(row.get('fully_closed_in_recorded_deals'))
            + '; qualified position net R unknown here.</p>')
        for close in closes[-8:]:
            out.append('<p>Close receipt: ' + esc(close.get('at')) + '; ' + esc(close.get('kind')) + '/'
                + esc(close.get('reason')) + '; outcome ' + esc(close.get('outcome')) + '; fill '
                + esc(close.get('fill_price')) + '; slippage ' + esc(close.get('slippage'))
                + '; latency ms ' + esc(close.get('latency_ms')) + '.</p>')
    out.append('<h2>Close/reverse assessment</h2><p>'
        + ('UNKNOWN: no explicit linked close-and-reverse receipt is exposed. Opposite entries or copied losses are not reversal proof; no reverse is proposed from losses. S1 keeps its no-same-day-reversal rule; S3 requires its own registered entry gates.' if positions else 'NOT EXERCISED: no observed owned lifecycle candidate.')
        + '</p><details><summary>Generated statement report context</summary><pre>'
        + esc(statement.get('text')) + '</pre></details>')
    return ''.join(out)
