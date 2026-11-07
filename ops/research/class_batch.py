"""Execute a frozen forty-symbol development batch; no orders or model API.

Collection receipts are written before reads. Uncertain jobs are never retried.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

from broker_readonly import ACCOUNT, Broker, BrokerError, ticks


def digest(value):
    return hashlib.sha256(json.dumps(value, separators=(',', ':'), allow_nan=False).encode()).hexdigest()


def save(path, value):
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, separators=(',', ':'), allow_nan=False))
    temporary.chmod(0o600)
    temporary.replace(path)


def receipt(path, event):
    with path.open('a') as stream:
        stream.write(json.dumps(event) + '\n')
        stream.flush()
        os.fsync(stream.fileno())


def collect(plan, output):
    jobs = plan['jobs']
    if len(jobs) != 80 or len({j['request']['symbol'] for j in jobs}) != 40:
        raise ValueError('exact frozen 40-symbol/80-episode batch required')
    if len({j['key'] for j in jobs}) != len(jobs):
        raise ValueError('duplicate jobs')
    for j in jobs:
        req = j['request']
        first = dt.datetime.fromisoformat(req['from'].replace('Z', '+00:00'))
        last = dt.datetime.fromisoformat(req['to'].replace('Z', '+00:00'))
        if (req['account'] != ACCOUNT or last - first != dt.timedelta(hours=4)
                or last + dt.timedelta(seconds=5) > dt.datetime(2025, 10, 24, tzinfo=dt.timezone.utc)
                or not j['key'].replace('-', '').isalnum()):
            raise ValueError('account/window/seal/key refused')
    output.mkdir(parents=True, exist_ok=True)
    frozen = output / 'plan.json'
    if frozen.exists() and json.loads(frozen.read_text()) != plan:
        raise ValueError('frozen plan differs')
    save(frozen, plan)
    ledger = output / 'ledger.jsonl'
    events = [json.loads(line) for line in ledger.read_text().splitlines()] if ledger.exists() else []
    started = {e['key'] for e in events if e['state'] == 'started'}
    finished = {e['key'] for e in events if e['state'] == 'finished'}
    if started - finished:
        raise ValueError('uncertain prior read; inspect, never reset/retry')
    total_bytes = sum(p.stat().st_size for p in output.glob('job-*.json'))
    with Broker() as broker:
        for j in jobs:
            target = output / ('job-' + j['key'] + '.json')
            if target.exists():
                saved = json.loads(target.read_text())
                if saved['request'] != j['request'] or j['key'] not in finished:
                    raise ValueError('saved job identity/receipt mismatch')
                if saved['status'] == 'complete' and digest(saved['result']['input']) != saved['result']['input_sha256']:
                    raise ValueError('saved source hash mismatch')
                continue
            if j['key'] in started or len(started) >= 80 or total_bytes >= 256_000_000:
                raise ValueError('spent job or batch budget exhausted')
            receipt(ledger, dict(key=j['key'], state='started', plan_sha256=digest(plan), at=dt.datetime.now(dt.timezone.utc).isoformat()))
            started.add(j['key'])
            req = j['request']
            first = int(dt.datetime.fromisoformat(req['from'].replace('Z', '+00:00')).timestamp() * 1000)
            last = int(dt.datetime.fromisoformat(req['to'].replace('Z', '+00:00')).timestamp() * 1000) + 5000
            budget = dict(pages=0, rows=0)
            job = dict(request=req, status='failed', symbol_id=j['symbol_id'], plan_sha256=digest(plan))
            print('started', j['key'], flush=True)
            try:
                deadline = time.monotonic() + 240
                bids = ticks(broker, j['symbol_id'], 1, first, last, budget, deadline)
                asks = ticks(broker, j['symbol_id'], 2, first, last, budget, deadline)
                source = dict(bars=[], bids=bids, asks=asks, contract=j['contract'])
                job.update(status='complete', result=dict(input=source, input_sha256=digest(source),
                           contract_metadata='current_snapshot_not_historical',
                           quote_continuity='unproven', study=None))
            except BrokerError as error:
                job['error'] = str(error)
            except Exception as error:
                job['error'] = type(error).__name__
            job['budget_used'] = budget
            encoded = json.dumps(job, separators=(',', ':')).encode()
            if len(encoded) > 32_000_000 or total_bytes + len(encoded) > 256_000_000:
                raise ValueError('source/batch byte cap; receipt left unresolved')
            save(target, job)
            total_bytes += target.stat().st_size
            receipt(ledger, dict(key=j['key'], state='finished', status=job['status'],
                                file_sha256=hashlib.sha256(target.read_bytes()).hexdigest()))
            print('finished', j['key'], job['status'], budget, job.get('error', ''), flush=True)
            if job['status'] != 'complete' and job.get('error') not in {
                'tick_row_budget', 'page_or_time_budget', 'tick_decode_failed',
                'unresolvable_tick_truncation', 'missing_truncation_flag',
            }:
                raise RuntimeError('transport/auth/source failure; stop without automatic retry')


def replay(plan, output, binary):
    results = []
    for j in plan['jobs']:
        source = output / ('job-' + j['key'] + '.json')
        row = dict(key=j['key'], symbol=j['request']['symbol'], asset_class=j['class'],
                   date=j['date'], status='missing')
        if source.exists():
            job = json.loads(source.read_text())
            if job['request'] != j['request'] or job['plan_sha256'] != digest(plan):
                raise ValueError('source/plan identity mismatch')
            row['status'] = job['status']
            row['budget_used'] = job['budget_used']
            row['file_sha256'] = hashlib.sha256(source.read_bytes()).hexdigest()
            if job['status'] == 'complete':
                process = subprocess.run([str(binary), str(source)], capture_output=True, text=True, timeout=90)
                if process.returncode:
                    raise RuntimeError('native replay rejected ' + j['key'] + ': ' + process.stderr[:300])
                row['replay'] = json.loads(process.stdout)
            else:
                row['error'] = job.get('error')
        results.append(row)
    return dict(plan_sha256=digest(plan), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                rows=results, financial_pass=False, validation_opened=False)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=['collect', 'replay'])
    p.add_argument('--plan', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--binary', type=Path, default=Path('target/debug/examples/class_batch_replay'))
    args = p.parse_args()
    plan = json.loads(args.plan.read_text())
    if args.action == 'collect':
        collect(plan, args.output)
    else:
        save(args.output / 'replay-results.json', replay(plan, args.output, args.binary.resolve()))
