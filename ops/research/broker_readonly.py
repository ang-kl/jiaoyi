"""Bounded JSON cTrader research reads through the configured HTTPS proxy.

No order, refresh, subscription or mutation message is permitted.
"""
import json
import os
import time
import urllib.request

READ_TYPES = {2100: 2101, 2102: 2103, 2114: 2115, 2116: 2117, 2145: 2146}
ACCOUNT = 49236051


class BrokerError(RuntimeError):
    pass


class Broker:
    def __enter__(self):
        from websockets.sync.client import connect
        proxy = urllib.request.getproxies().get('https')
        if not proxy:
            raise BrokerError('configured_proxy_required')
        self.ws = connect('wss://demo.ctraderapi.com:5036/', proxy=proxy,
                          open_timeout=12, close_timeout=3, max_size=8_000_000,
                          ping_interval=None)
        self.counter = 0
        self.last = 0.0
        try:
            self.request(2100, dict(clientId=os.environ['CTRADER_CLIENT_ID'],
                                   clientSecret=os.environ['CTRADER_CLIENT_SECRET']))
            self.request(2102, dict(ctidTraderAccountId=ACCOUNT,
                                   accessToken=os.environ['CTRADER_ACCESS_TOKEN']))
        except Exception:
            self.ws.close()
            raise
        return self

    def __exit__(self, *_):
        self.ws.close()

    def request(self, kind, payload):
        if kind not in READ_TYPES or payload.get('ctidTraderAccountId', ACCOUNT) != ACCOUNT:
            raise BrokerError('read_type_or_account_refused')
        time.sleep(max(0, .8 - (time.monotonic() - self.last)))
        self.counter += 1
        identity = 'research40-' + str(self.counter)
        self.last = time.monotonic()
        self.ws.send(json.dumps(dict(clientMsgId=identity, payloadType=kind, payload=payload)))
        deadline = time.monotonic() + 12
        for _ in range(40):
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise BrokerError('response_deadline')
            message = json.loads(self.ws.recv(timeout=remaining))
            if message.get('clientMsgId') != identity:
                continue
            if message.get('payloadType') in (50, 2142):
                code = message.get('payload', {}).get('errorCode', 'broker_error')
                raise BrokerError(code if isinstance(code, str) and code.replace('_', '').isalnum() else 'broker_error')
            if message.get('payloadType') != READ_TYPES[kind]:
                raise BrokerError('unexpected_response_type')
            result = message.get('payload', {})
            if result.get('ctidTraderAccountId', ACCOUNT) != ACCOUNT:
                raise BrokerError('response_account_mismatch')
            return result
        raise BrokerError('unrelated_response_limit')


def decode(data, first, last):
    """Same checked add-then-subtract convention as native tickdata::decode."""
    if not data:
        return []
    for sign in (1, -1):
        ticks = []
        timestamp = int(data[0]['timestamp'])
        price = int(data[0]['tick'])
        for index, item in enumerate(data):
            if index:
                timestamp += sign * int(item['timestamp'])
                price += sign * int(item['tick'])
            if not (first <= timestamp <= last and 0 < price <= 2**63 - 1):
                break
            if ticks and timestamp > ticks[-1]['ts_ms']:
                break
            ticks.append(dict(ts_ms=timestamp, price_e5=price))
        else:
            return ticks
    raise BrokerError('tick_decode_failed')


def ticks(broker, symbol_id, side, first, last, budget, deadline):
    if first < 0 or last < first or last - first > 86_405_000 or side not in (1, 2):
        raise BrokerError('invalid_tick_window')
    pending = [(first, last)]
    output = []
    while pending:
        if budget['pages'] >= 120 or time.monotonic() >= deadline:
            raise BrokerError('page_or_time_budget')
        a, b = pending.pop()
        budget['pages'] += 1
        result = broker.request(2145, dict(ctidTraderAccountId=ACCOUNT, symbolId=symbol_id,
                                        type=side, fromTimestamp=a, toTimestamp=b))
        rows = decode(result.get('tickData', []), a, b)
        more = result.get('hasMore')
        if not isinstance(more, bool):
            raise BrokerError('missing_truncation_flag')
        if more:
            if not rows or a == b:
                raise BrokerError('unresolvable_tick_truncation')
            mid = a + (b - a) // 2
            pending.extend([(mid + 1, b), (a, mid)])
        else:
            budget['rows'] += len(rows)
            if budget['rows'] > 200_000:
                raise BrokerError('tick_row_budget')
            output.extend(reversed(rows))
    return output
