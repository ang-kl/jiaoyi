#!/usr/bin/env python3
"""Owner-authorised, bounded contract/deal evidence export (№18,370).

Independent of the paused PRICE collector. No price/history/subscription/order,
refresh-token or mutation request is in this client's allowlist. Raw auth frames
and exception messages are never persisted. Captures are immutable; rerunning
against an existing output directory refuses rather than recollects.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import time
import urllib.request

from rebuild_preflight import require
from replay_inventory import CORE

ACCOUNTS = {49236051: 'demo', 46130058: 'demo', 47790949: 'demo', 43002148: 'live'}
READ_TYPES = {2114: 2115, 2116: 2117, 2133: 2134}
DAY = 86_400_000


def utc():
    return dt.datetime.now(dt.timezone.utc).isoformat()


def save(path, value):
    raw = (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()
    path.write_bytes(raw)
    return {'path': str(path), 'sha256': hashlib.sha256(raw).hexdigest()}


def check_request(account, kind, payload, first, last):
    require(account in ACCOUNTS and kind in READ_TYPES, 'evidence_read_type_or_account_refused')
    require(payload.get('ctidTraderAccountId') == account, 'evidence_request_account_mismatch')
    fields = {2114: {'ctidTraderAccountId', 'includeArchivedSymbols'},
              2116: {'ctidTraderAccountId', 'symbolId'},
              2133: {'ctidTraderAccountId', 'fromTimestamp', 'toTimestamp', 'maxRows'}}
    require(set(payload) <= fields[kind], 'evidence_unexpected_request_field')
    if kind == 2116:
        ids = payload.get('symbolId')
        require(isinstance(ids, list) and 0 < len(ids) <= 100 and
                all(type(i) is int and i > 0 for i in ids), 'evidence_symbol_budget')
    if kind == 2133:
        a, b = payload.get('fromTimestamp'), payload.get('toTimestamp')
        require(type(a) is int and type(b) is int and first <= a <= b <= last and
                b-a <= 31*DAY and payload.get('maxRows') == 1000, 'evidence_deal_window_budget')


class EvidenceBroker:
    def __init__(self, account, first, last, ledger):
        require(account in ACCOUNTS and 0 <= first <= last and last-first <= 31*DAY,
                'evidence_account_or_window')
        self.account, self.first, self.last, self.ledger = account, first, last, ledger
        self.counter, self.sent_at = 0, 0.0

    def __enter__(self):
        from websockets.sync.client import connect
        proxy = urllib.request.getproxies().get('https')
        require(bool(proxy), 'configured_proxy_required')
        self.ws = connect(f'wss://{ACCOUNTS[self.account]}.ctraderapi.com:5036/',
                          proxy=proxy, open_timeout=12, close_timeout=3,
                          max_size=8_000_000, ping_interval=None)
        try:
            self._frame(2100, 2101, {'clientId': os.environ['CTRADER_CLIENT_ID'],
                                   'clientSecret': os.environ['CTRADER_CLIENT_SECRET']}, auth=True)
            self._frame(2102, 2103, {'ctidTraderAccountId': self.account,
                                   'accessToken': os.environ['CTRADER_ACCESS_TOKEN']}, auth=True)
        except Exception:
            self.ws.close()
            raise
        return self

    def __exit__(self, *_):
        self.ws.close()

    def _frame(self, kind, expected, payload, auth=False):
        require(self.counter < 68, 'evidence_request_budget')
        time.sleep(max(0, .8 - (time.monotonic()-self.sent_at)))
        self.counter += 1
        mid = f'evidence18370-{self.account}-{self.counter}'
        row = {'account_id': self.account, 'type': kind, 'at_utc': utc(), 'status': 'sent'}
        if not auth:
            row['request'] = payload
        self.ledger.append(row)
        self.sent_at = time.monotonic()
        self.ws.send(json.dumps({'clientMsgId': mid, 'payloadType': kind, 'payload': payload}))
        deadline = time.monotonic()+15
        for _ in range(40):
            require(time.monotonic() < deadline, 'evidence_response_deadline')
            message = json.loads(self.ws.recv(timeout=max(.01, deadline-time.monotonic())))
            if message.get('clientMsgId') != mid:
                continue
            if message.get('payloadType') in (50, 2142):
                code = message.get('payload', {}).get('errorCode', '')
                row['status'] = 'broker_refused'
                row['code'] = code if isinstance(code, str) and code.replace('_', '').isalnum() else 'redacted'
                raise ValueError('broker_evidence_refused')
            require(message.get('payloadType') == expected, 'evidence_response_type')
            result = message.get('payload', {})
            require(result.get('ctidTraderAccountId', self.account) == self.account,
                    'evidence_response_account_mismatch')
            row['status'] = 'received'
            row['received_at_utc'] = utc()
            return result
        raise ValueError('evidence_unrelated_response_budget')

    def request(self, kind, payload):
        check_request(self.account, kind, payload, self.first, self.last)
        return self._frame(kind, READ_TYPES[kind], payload)


def collect_deals(broker, save_page):
    """Bisect truncated time ranges. Never skip a millisecond or accept hasMore.

    A saturated single-millisecond page is explicitly incomplete. Only terminal
    complete pages are joined; conflicting duplicate deals refuse the export.
    """
    pending, pages, deals = [(broker.first, broker.last)], [], {}
    while pending:
        require(len(pages) < 60, 'evidence_deal_page_budget')
        a, b = pending.pop()
        result = broker.request(2133, {'ctidTraderAccountId': broker.account,
            'fromTimestamp': a, 'toTimestamp': b, 'maxRows': 1000})
        require(type(result.get('hasMore')) is bool, 'evidence_missing_truncation_flag')
        ref = save_page(len(pages), result)
        pages.append({'from_ms': a, 'to_ms': b, 'has_more': result['hasMore'], 'receipt': ref})
        if result['hasMore']:
            require(a < b, 'evidence_saturated_millisecond')
            mid = (a+b)//2
            pending.extend([(mid+1, b), (a, mid)])
            continue
        for i, deal in enumerate(result.get('deal', [])):
            require(a <= int(deal['executionTimestamp']) <= b, 'deal_outside_requested_interval')
            key = int(deal['dealId'])
            if key in deals:
                require(deals[key]['raw'] == deal, 'conflicting_deal_duplicate')
            deals[key] = {'raw': deal, 'receipt': {**ref, 'pointer': f'/deal/{i}'}}
        require(len(deals) <= 20_000, 'evidence_deal_row_budget')
    return {'pages': pages, 'deals': [deals[k] for k in sorted(deals)], 'complete_requested_window': True}


def capture(output):
    require(not output.exists(), 'immutable_capture_already_exists')
    output.mkdir(parents=True)
    now = dt.datetime.now(dt.timezone.utc)
    first = int(dt.datetime(2026, 10, 24, tzinfo=dt.timezone.utc).timestamp()*1000)
    last = int(now.timestamp()*1000)
    require(0 < last-first <= 31*DAY, 'directive_window_expired_requires_new_scope')
    plan = {'trace': '№18,370', 'authorisation': 'read-only symbol contracts and closed deals',
            'accounts': ACCOUNTS, 'from_ms': first, 'to_ms': last,
            'core_symbols': list(CORE), 'additional_symbols': 'symbols actually present in returned deals',
            'maximum_protocol_requests_per_account': 68, 'maximum_deal_pages_per_account': 60,
            'new_price_requests': 0, 'orders_permitted': False, 'historical_terms_authority': False}
    plan_ref = save(output/'capture-plan.json', plan)  # before the first broker frame
    ledger, accounts = [], []
    for account in ACCOUNTS:
        row = {'account_id': account, 'environment': ACCOUNTS[account], 'status': 'incomplete'}
        try:
            with EvidenceBroker(account, first, last, ledger) as broker:
                catalog = broker.request(2114, {'ctidTraderAccountId': account, 'includeArchivedSymbols': False})
                catalog_ref = save(output/f'{account}-catalog.json', catalog)
                deals = collect_deals(broker, lambda n, value:
                    save(output/f'{account}-deals-{n}.json', value))
                wanted = {int(v['raw']['symbolId']) for v in deals['deals']}
                wanted |= {int(s['symbolId']) for s in catalog.get('symbol', []) if s.get('symbolName') in CORE}
                specs = broker.request(2116, {'ctidTraderAccountId': account, 'symbolId': sorted(wanted)})
                row.update(status='complete', catalog_receipt=catalog_ref, deals=deals,
                           contracts_receipt=save(output/f'{account}-contracts.json', specs))
        except Exception as e:
            row['error_type'] = type(e).__name__  # never log credential-bearing exception text
        accounts.append(row)
        save(output/'capture.json', {'plan': plan, 'plan_receipt': plan_ref, 'accounts': accounts,
                                    'protocol_requests': ledger, 'price_requests': 0})
    return {'accounts': [{k: r[k] for k in ('account_id', 'status')} for r in accounts],
            'protocol_requests': len(ledger), 'new_price_requests': 0}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    print(json.dumps(capture(p.parse_args().output), indent=2))
