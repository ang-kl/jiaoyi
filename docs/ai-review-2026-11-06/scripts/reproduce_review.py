"""Read-only reproduction of revised handoff, separate from production/research seals."""
import argparse
import collections
import datetime as dt
import hashlib
import json
import statistics
import sys
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repository', type=Path, required=True, help='checkout of reviewed source revision')
parser.add_argument('--fresh', type=Path, required=True, help='20 original recent collection jobs and manifests')
parser.add_argument('--development', type=Path, required=True, help='40 original development jobs')
parser.add_argument('--baseline', type=Path, default=Path(__file__).resolve().parents[1] / 'data/cost-context-results.json')
parser.add_argument('--output', type=Path, required=True, help='directory for derived review result')
args = parser.parse_args()
ROOT = args.repository.resolve()
sys.path.insert(0, str(ROOT / 'ops/research'))
import cost_context as cc
import market_collection as mc

OUT = args.output.resolve()
OUT.mkdir(parents=True, exist_ok=True)
FRESH = args.fresh.resolve()
manifest = json.loads((FRESH / 'collection-plan.json').read_text())
receipts = json.loads((FRESH / 'collection-summary.json').read_text())
by_symbol = {r['symbol']: r for r in receipts['rows']}
plan = json.loads((ROOT / 'docs/cost-context-plan-2026-11-05.json').read_text())
corpus, provenance = {}, []
ticks = 0
for cls, req in manifest['requests']:
    path = FRESH / (req['symbol'] + '.json')
    raw = path.read_bytes()
    sha = hashlib.sha256(raw).hexdigest()
    assert sha == by_symbol[req['symbol']]['file_sha256']
    job = json.loads(raw)
    mc.validate(job, req)
    assert job['status'] == 'complete'
    source = job['result']['input']
    lo, hi = cc.ms(req['from']), cc.ms(req['to']) + 5000
    for side in ('bids', 'asks'):
        stream = source[side]
        assert stream and all(isinstance(x['ts_ms'], int) and isinstance(x['price_e5'], int)
                              and x['price_e5'] > 0 and lo <= x['ts_ms'] <= hi for x in stream)
        assert all(a['ts_ms'] <= b['ts_ms'] for a, b in zip(stream, stream[1:]))
        assert len(stream) == job['result']['coverage'][side[:-1] + '_ticks']
        ticks += len(stream)
    key = (req['symbol'], lo)
    assert key not in corpus
    corpus[key] = cc.quotes(source)
    provenance.append(dict(symbol=req['symbol'], file_sha256=sha,
                           input_sha256=job['result']['input_sha256'], request=req))

fresh = cc.evaluate(plan, corpus)
assert len(fresh) == 240
old_path = args.baseline.resolve()
old = json.loads(old_path.read_text())
assert old['plan_sha256'] == hashlib.sha256((ROOT / 'docs/cost-context-plan-2026-11-05.json').read_bytes()).hexdigest()
assert old['analysis_sha256'] == hashlib.sha256((ROOT / 'ops/research/cost_context.py').read_bytes()).hexdigest()
old_corpus = {}
for name, sha in plan['input_files'].items():
    path = args.development.resolve() / name
    assert hashlib.sha256(path.read_bytes()).hexdigest() == sha
    job = json.loads(path.read_text())
    old_corpus[(job['request']['symbol'], cc.ms(job['request']['from']))] = cc.quotes(job['result']['input'])
reproduced_old = cc.evaluate(plan, old_corpus)
assert reproduced_old == old['rows'], 'saved development diagnostics not reproduced exactly'
pooled = old['rows'] + fresh


def net(row, bps):
    o = row['outcome']
    return o['executable_before_fees_bps'] - (
        o['executable_before_fees_bps'] - o['stress_after_slippage_bps']) * bps / 24


def metrics(rows):
    seen = [r for r in rows if r['outcome'] is not None]
    return dict(expected=len(rows), observed=len(seen), statuses=dict(collections.Counter(r['status'] for r in rows)),
                direction_wins=sum(r['outcome']['mid_direction_correct'] for r in seen),
                scenarios={str(bp): dict(wins=sum(net(r, bp) > 0 for r in seen),
                                         mean_bps=statistics.mean(net(r, bp) for r in seen) if seen else None)
                           for bp in (0, 2, 3.74, 24)},
                by_symbol={s: dict(observed=len(rs),
                                   gross_wins=sum(net(r, 0) > 0 for r in rs),
                                   wins_2bp=sum(net(r, 2) > 0 for r in rs),
                                   gross_mean_bps=statistics.mean(net(r, 0) for r in rs) if rs else None,
                                   mean_2bp_bps=statistics.mean(net(r, 2) for r in rs) if rs else None)
                           for s in sorted({r['symbol'] for r in rows})
                           for rs in [[r for r in seen if r['symbol'] == s]]})


def select(rows, cls, rule, horizon, symbols=None, unanimous=False):
    picked = []
    for row in rows:
        if (row['asset_class'], row['variant'], row['horizon_minutes']) != (cls, rule, horizon):
            continue
        if symbols is not None and row['symbol'] not in symbols:
            continue
        row = dict(row)
        if unanimous and row['status'] == 'observed':
            peer = [x['return_bps'] for x in row['peer_components']]
            if not peer or not (all(x > 0 for x in peer) or all(x < 0 for x in peer)):
                row['status'] = 'peer_disagreement'
                row['outcome'] = None
        picked.append(row)
    return picked


specs = {
    'oil_peer_180': ('energy', 'peer_continuation', 180, ['SpotBrent', 'SpotCrude'], False),
    'oil_own_180': ('energy', 'own_momentum', 180, ['SpotBrent', 'SpotCrude'], False),
    'oil_always_long_180': ('energy', 'always_long', 180, ['SpotBrent', 'SpotCrude'], False),
    'metal_reversal_30': ('metal', 'peer_reversal', 30, None, False),
    'metal_unanimous_reversal_30': ('metal', 'peer_reversal', 30, None, True),
    'metal_unanimous_reversal_180': ('metal', 'peer_reversal', 180, None, True),
    'metal_own_30': ('metal', 'own_momentum', 30, None, False),
    'indices_own_30': ('equity_index', 'own_momentum', 30, None, False),
    'indices_own_180': ('equity_index', 'own_momentum', 180, None, False),
    'indices_always_long_180': ('equity_index', 'always_long', 180, None, False),
    'crypto_peer_180': ('crypto', 'peer_continuation', 180, None, False),
    'crypto_own_180': ('crypto', 'own_momentum', 180, None, False),
    'fx_reversal_180': ('fx', 'peer_reversal', 180, None, False),
    'fx_unanimous_reversal_180': ('fx', 'peer_reversal', 180, None, True),
    'fx_continuation_180': ('fx', 'peer_continuation', 180, None, False),
    'natgas_own_180': ('energy', 'own_momentum', 180, ['NatGas'], False),
}
tables = {name: {period: metrics(select(rows, *args)) for period, rows in
                 [('development', old['rows']), ('fresh', fresh), ('pooled', pooled)]}
          for name, args in specs.items()}

# Quote checkpoints independently read from reconstructed source, including zero spread.
checkpoints = {}
for (symbol, start), qs in corpus.items():
    mark = start + 3600000
    decision = cc.first(qs, mark, mark + 5000)
    entry = cc.first(qs, decision[0] + 1, mark + 5000) if decision else None
    exits = {str(h): cc.first(qs, mark + h * 60000, mark + h * 60000 + 5000) for h in (30, 60, 180)}
    checkpoints[symbol] = dict(decision=decision, entry=entry, exits=exits)

report = dict(trace='№18,270 §§1–6', generated_at_utc=dt.datetime.now(dt.timezone.utc).isoformat(),
              main_sha='64739b45a65b285595e60ae679354cc015f499fd',
              script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              analysis_code_sha256=old['analysis_sha256'],
              collection_manifest_sha256=hashlib.sha256((FRESH / 'collection-plan.json').read_bytes()).hexdigest(),
              prior_result_sha256=hashlib.sha256(old_path.read_bytes()).hexdigest(),
              development_rows_reproduced_exactly=len(reproduced_old),
              tick_count=ticks, provenance=provenance,
              fresh_statuses=dict(collections.Counter(r['status'] for r in fresh)),
              tables=tables, checkpoints=checkpoints,
              matched_controls_fresh=cc.matched_controls(fresh),
              fresh_rows=fresh,
              qualification='Independent reproduction of fixed-horizon exploratory diagnostics, not native strategy replay; reduced costs are assumptions; no financial pass',
              financial_pass=False)
(OUT / 'fresh-reconsideration.json').write_text(json.dumps(report, indent=2, allow_nan=False) + '\n')
print('ticks:', ticks, 'fresh statuses:', report['fresh_statuses'])
for name, t in tables.items():
    print(name, json.dumps({k: {x: v[x] for x in ('observed','scenarios')} for k, v in t.items()}))
print('EURUSD decision', checkpoints['EURUSD']['decision'])
