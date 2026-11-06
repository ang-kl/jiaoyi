#!/usr/bin/env python3
"""Frozen development-only context experiment using the native exit engine."""
import argparse
import bisect
import collections
import datetime as dt
import hashlib
import html
import json
import math
from pathlib import Path
import statistics
import subprocess

import class_research
import cost_context as context

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / 'docs/full-exit-research-plan-2026-11-06.json'
FROZEN_SHA256 = 'b202c9a5c9f4603576f48c9ea9dafc374a422581177beb7b32114a5edf14464b'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def frozen_plan(path=PLAN):
    if digest(path) != FROZEN_SHA256:
        raise ValueError('frozen plan changed; no silent retuning')
    return json.loads(path.read_text())


def iso(at):
    return dt.datetime.fromtimestamp(at / 1000, dt.timezone.utc).isoformat().replace('+00:00', 'Z')


def features(qs, start):
    opening = context.first(qs, start, start + 60000)
    mark = start + 3600000
    decision = context.first(qs, mark, mark + 5000)
    if decision is None:
        return None, 'missing_decision_quote'
    samples = [opening] + [context.prior(qs, start + m * 60000)
                           for m in range(5, 60, 5)] + [decision]
    if any(x is None for x in samples):
        return None, 'missing_first_hour_samples'
    prefix = qs[bisect.bisect_left(qs, (start,)):bisect.bisect_right(qs, decision)]
    observed_range = max(map(context.mid, prefix)) - min(map(context.mid, prefix))
    if observed_range <= 0:
        return None, 'missing_or_flat_preparation_range'
    return dict(opening=opening, decision=decision, observed_range=observed_range,
                median_spread=statistics.median(x[2] - x[1] for x in samples),
                sample_times=[x[0] for x in samples]), None


def signal(rule, symbol, cls, start, at, own_open, own_quote, corpus):
    if rule == 'always_long':
        return 1, [], None
    if rule == 'own_momentum':
        side = context.sign(context.mid(own_quote) - context.mid(own_open))
        return side, [], None if side else 'flat_predictor'
    factor, peers = context.factor(symbol, 'energy' if cls == 'oil' else cls,
                                   start, at, corpus, {})
    if factor is None:
        return None, peers, 'missing_predictor'
    if rule == 'unanimous_peer_reversal':
        signs = {context.sign(p['return_bps']) for p in peers}
        if len(signs) != 1 or 0 in signs:
            return None, peers, 'peer_disagreement'
        factor = -factor
    side = context.sign(factor)
    return side, peers, None if side else 'flat_predictor'


def prepare(rule, symbol, cls, job, corpus, volume):
    req = job['request']
    start, end = context.ms(req['from']), context.ms(req['to'])
    qs = corpus[(symbol, start)]
    row = dict(symbol=symbol, asset_class=cls, rule=rule, **{'from': req['from']},
               input_sha256=job['result']['input_sha256'], status=None, replay=None,
               decision_peers=[], fill_peers=[])
    f, reason = features(qs, start)
    if reason:
        row['status'] = reason
        return row, None
    row['features'] = f
    side, peers, reason = signal(rule, symbol, cls, start, f['decision'][0],
                                 f['opening'], f['decision'], corpus)
    row['decision_peers'] = peers
    if reason:
        row['status'] = reason
        return row, None
    # Same positive half-up grid rounding and reserve arithmetic as the native
    # adapter. A known decision-cost veto must precede any fill search; native
    # replay independently checks the reserve again at decision and fill.
    contract = job['result']['input']['contract']
    quantum = 10.0 ** (-contract['digits'])
    initial_r = max(1, math.floor(f['observed_range'] / quantum + 0.5)) * quantum
    decision_cost = f['median_spread'] + 2.0 * 24.0 / 10000.0 * f['decision'][2 if side > 0 else 1]
    row['decision_admission'] = dict(initial_r=initial_r, cost_reserve=decision_cost)
    if decision_cost >= initial_r:
        row['status'] = 'candidate_cost_ge_r'
        return row, None
    entry = context.first(qs, f['decision'][0] + 1, start + 3605000)
    if entry is None:
        row['status'] = 'missing_next_fresh_entry'
        return row, None
    fill_side, peers, reason = signal(rule, symbol, cls, start, entry[0],
                                     f['opening'], entry, corpus)
    row['fill_peers'] = peers
    if reason == 'missing_predictor':
        row['status'] = reason
        return row, None
    if reason or fill_side != side:
        row['status'] = 'candidate_changed_before_fill'
        return row, None
    entry_index = bisect.bisect_left(qs, entry)
    path = qs[entry_index:bisect.bisect_left(qs, (end + 5000,))]
    request = dict(decision_at=iso(f['decision'][0]), entered_at=iso(entry[0]),
                   decision_price=f['decision'][2 if side > 0 else 1],
                   fill=entry[2 if side > 0 else 1], side='long' if side > 0 else 'short',
                   observed_range=f['observed_range'], median_spread=f['median_spread'],
                   volume=volume, **contract, session_end=req['to'], quotes=[
                       dict(at=iso(q[0]), bid=q[1], ask=q[2],
                            gap_before=i > 0 and q[0] - path[i-1][0] > 6000)
                       for i, q in enumerate(path)])
    row['side'] = side
    return row, request


def score_rows(rows, arm, bps, minimum, expected, symbols):
    scored = []
    for row in rows:
        replay = row['replay']
        trade = None
        if replay and replay['status'] == 'simulated_entry':
            net = next(s['scenario_net_r'] for s in replay['sensitivities']
                       if s['arm'] == arm and s['bps_per_side'] == bps)
            trade = dict(trailing_scenario_net_r=net, entry=replay['input'])
        scored.append(dict(symbol=row['symbol'], **{'from': row['from']},
                           study=dict(status=row['status'], trade=trade)))
    result = class_research.score(scored, minimum, expected, symbols)
    result['skips'] = dict(collections.Counter(r['status'] for r in rows
                                              if r['status'] != 'simulated_entry'))
    # Only sequential episodes of one symbol; never invent a cross-asset equity curve.
    result['max_scenario_drawdown_r'] = None
    if len(symbols) == 1:
        cumulative = peak = drawdown = 0.0
        for row in sorted(scored, key=lambda r: r['from']):
            trade = row['study']['trade']
            if trade and trade['trailing_scenario_net_r'] is not None:
                cumulative += trade['trailing_scenario_net_r']
                peak = max(peak, cumulative)
                drawdown = max(drawdown, peak-cumulative)
        result['max_scenario_drawdown_r'] = drawdown if result['closed'] else None
    return result


def summarise(rows, plan):
    summaries, per_symbol, matched = [], [], []
    dates = len(plan['development_dates'])
    for cls, symbols in plan['classes'].items():
        for rule in plan['rules'][cls]:
            subset = [r for r in rows if r['asset_class'] == cls and r['rule'] == rule]
            for arm in ('fixed_stop', 'trailing'):
                for bps in plan['costs']['realised_sensitivity_bps_per_side']:
                    result = score_rows(subset, arm, bps, 4, dates * len(symbols), symbols)
                    summaries.append(dict(asset_class=cls, rule=rule, arm=arm, bps=bps, **result))
                    for symbol in symbols:
                        sample = [r for r in subset if r['symbol'] == symbol]
                        per_symbol.append(dict(symbol=symbol, rule=rule, arm=arm, bps=bps,
                                               **score_rows(sample, arm, bps, 1, dates, [symbol])))
        index = {(r['symbol'], r['from'], r['rule']): r for r in rows if r['asset_class'] == cls}
        candidate = plan['candidates'][cls]
        for control in plan['rules'][cls]:
            if control == candidate:
                continue
            for bps in plan['costs']['realised_sensitivity_bps_per_side']:
                pairs = []
                for row in rows:
                    if row['asset_class'] != cls or row['rule'] != candidate:
                        continue
                    other = index[(row['symbol'], row['from'], control)]
                    def net(r):
                        if not r['replay'] or r['status'] != 'simulated_entry':
                            return None
                        return next(s['scenario_net_r'] for s in r['replay']['sensitivities']
                                    if s['arm'] == 'trailing' and s['bps_per_side'] == bps)
                    a, b = net(row), net(other)
                    if a is not None and b is not None:
                        pairs.append((a, b, row['side'] != other['side']))
                matched.append(dict(asset_class=cls, candidate=candidate, control=control,
                                    bps=bps, matched=len(pairs), changed_direction=sum(p[2] for p in pairs),
                                    mean_delta_r=statistics.mean(p[0]-p[1] for p in pairs) if pairs else None))
    eligible = [s['asset_class'] for s in summaries if s['rule'] == plan['candidates'][s['asset_class']]
                and s['arm'] == 'trailing' and s['bps'] == 24 and s['passes_screen']]
    return summaries, per_symbol, matched, eligible


def report(result, path):
    def table(rows, columns):
        return '<div class="scroll"><table><tr>' + ''.join('<th>'+html.escape(c)+'</th>' for c in columns) + '</tr>' + ''.join(
            '<tr>'+''.join('<td>'+html.escape(str(r.get(c)))+'</td>' for c in columns)+'</tr>' for r in rows)+'</table></div>'
    primary = [s for s in result['summary'] if s['arm'] == 'trailing' and s['bps'] == 24]
    candidates = frozen_plan()['candidates']
    findings = []
    for cls, rule in candidates.items():
        s = next(s for s in primary if s['asset_class'] == cls and s['rule'] == rule)
        mean = 'unavailable' if s['mean_net_r'] is None else f"{s['mean_net_r']:+.4f}R"
        findings.append(f"{cls}: {s['wins']}/{s['closed']} closed scenarios win, mean {mean}; "
                        f"{s['censored']} censored. Skips: {s['skips']}.")
    low = next(s for s in result['summary'] if s['asset_class']=='equity_index'
               and s['rule']=='own_momentum' and s['arm']=='trailing' and s['bps']==2)
    if low['mean_net_r'] is not None and low['mean_net_r'] <= 0:
        findings.append(f"At2bp/side indices have{low['wins']}/{low['closed']} wins but "
                        f"mean{low['mean_net_r']:+.4f}R: lower costs alone do not rescue observed expectancy.")
    oil = next(s for s in result['matched_controls'] if s['asset_class']=='oil'
               and s['control']=='own_momentum' and s['bps']==24)
    if oil['matched'] and oil['changed_direction']==0 and oil['mean_delta_r']==0:
        findings.append(f"Oil's peer and own-momentum controls are identical on all{ oil['matched']} "
                        "matched closed episodes; no incremental peer value is demonstrated.")
    oil_dates = sorted({r['from'][:10] for r in result['rows'] if r['asset_class']=='oil'
                        and r['rule']=='peer_continuation' and r['status']=='simulated_entry'
                        and r['replay']['comparison']['candidate']['scenario_gross_r'] is not None})
    findings.append(f"Oil closed-scenario dates: {oil_dates}. Same-day symbols are correlated evidence.")
    path.write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi full-exit development experiment</title><style>body{font:16px/1.5 system-ui;max-width:1400px;margin:24px auto;padding:16px;color:#203040}.scroll{overflow:auto}table{border-collapse:collapse;font-size:13px}th,td{padding:8px;border-bottom:1px solid #ddd;text-align:left;vertical-align:top}code{overflow-wrap:anywhere}p{max-width:1050px}</style><h1>Full SL/TP/trailing development replay</h1>'''
        + '<p>'+html.escape(result['trace'])+'</p><h2>TLDR</h2>'
        + f'<p>{len(result["eligible_for_separate_validation"])} of 3 frozen candidates pass the primary development screen. No validation data were opened, no live rule changed and no account-goal progress was earned. The 50% interim goal remains unproven.</p>'
        + f'<p>{len(result["rows"])} rule-episode rows from 22 target-symbol episodes on two previously seen dates. Controls, exit arms and cost sensitivities reuse these same episodes; they are not additional independent trades.</p>'
        + '<p>'+html.escape(' '.join(findings))+'</p>'
        + '<h2>Primary 24bp-per-side trailing outcomes</h2>'
        + table(primary, ['asset_class','rule','closed','censored','wins','net_win_rate','mean_net_r','profit_factor','passes_screen','skips','reasons'])
        + '<h2>What was frozen</h2><p>Oil follows the other crude benchmark; metals reverse the inverse-dollar peer proxy only under strict peer unanimity; indices follow their own first-hour move. Own-momentum and always-long controls are retained. Thirteen causal observations establish spread and readiness; the observed first-hour range fixes original R. No path-efficiency filter or retrospective parameter search is added.</p>'
        + '<p>The native Q=1.38 target planner handles minimum-volume whole closes or TP1 partial plus runner. The existing Rust engine compares fixed stops and original-R trailing at S3 cadence, checks old broker brackets first and assumes next-quote amendment acknowledgement. Hour-four closes require an executable quote in the five-second tail; otherwise outcomes remain censored.</p>'
        + '<h2>Costs and interpretation</h2><p>Entry admission and target geometry retain 24bp per side plus the causal median spread reserve. Sensitivities at2,3.74 and24bp per side hold that geometry and the same fills fixed. Executable spread is already in bid/ask fills and is not charged twice; slippage drag uses the entry and volume-weighted partial/final exits. Zero additional commission is an explicit unknown-fee assumption. These are scenario returns, not calibrated or qualified net results.</p>'
        + '<details><summary>All fixed-stop/trailing and cost scenarios</summary>'
        + table(result['summary'], ['asset_class','rule','arm','bps','closed','wins','net_win_rate','mean_net_r','profit_factor','passes_screen','reasons'])+'</details>'
        + '<h2>Matched controls</h2><p>Only identical episodes closed in both compared arms enter these differences. Unmatched/missing episodes remain visible in the full JSON and primary table.</p>'
        + table(result['matched_controls'], ['asset_class','candidate','control','bps','matched','changed_direction','mean_delta_r'])
        + '<details><summary>Every symbol: primary trailing scenario</summary>'
        + table([s for s in result['symbols'] if s['arm']=='trailing' and s['bps']==24], ['symbol','rule','closed','wins','net_win_rate','mean_net_r','max_scenario_drawdown_r','reasons'])+'</details>'
        + '<h2>Failure prevention and next gate</h2><ol><li>Verify frozen source files, original collection receipt and native original-study parity before calculating new outcomes.</li><li>Reject future/stale peer features, missing preparations and unavailable next-quote fills. Retain skipped and censored episodes.</li><li>Use the primary24bp trailing scenario for the frozen candidate only: at least4 class closes, each symbol represented, ≥50% wins, positive expectancy/PF and positive per-symbol price-unit outcomes. Optimistic cost scenarios cannot substitute.</li><li>Only after development passes, evaluate the already frozen separate dates without retuning: ≥20 class closes, ≥5 per symbol across3 weeks. This development-only command cannot collect or open that evaluation.</li><li>Qualify historical contract/coverage and paired actual costs, then shadow-test before considering promotion.</li></ol>'
        + '<p>Two dates cannot establish robustness. Historical liquidity, actual fees/funding, dispatch/rejection, account sizing, margin and portfolio conflicts remain unmodelled. Recent collection-only data and the historical research seal remain intact. Original73% /1:1.38 per strategy/account goal unchanged.</p>'
        + '<h2>Reproduction</h2><p>Frozen plan SHA256: <code>'+result['plan_sha256']+'</code>. All source/code/binary hashes, fills, skip reasons and cost scenarios are in full-exit-results.json. No raw tick archive, ZIP, model API or external messages were generated.</p><p>Draft for review.</p></html>')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--sources', type=Path, required=True)
    p.add_argument('--verifier', type=Path, required=True)
    p.add_argument('--replay', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    plan = frozen_plan()
    if (a.sources / 'plan-sha256.txt').read_text().strip() != plan['source_collection_plan_sha256']:
        raise ValueError('collection receipt mismatch')
    a.output.mkdir(parents=True, exist_ok=False)
    (a.output / 'plan-sha256.txt').write_text(FROZEN_SHA256+'\n')
    jobs, corpus, provenance = {}, {}, []
    for name, sha in plan['input_files'].items():
        path = a.sources / name
        if path.stat().st_size > 32_000_000 or digest(path) != sha:
            raise ValueError('source differs from frozen file manifest')
        job = json.loads(path.read_text())
        verified = json.loads(subprocess.run([str(a.verifier), str(path)], check=True,
                              capture_output=True, text=True, timeout=90).stdout)
        req = job['request']; key = (req['symbol'], context.ms(req['from']))
        if key in jobs or req['from'][:10] not in plan['development_dates']:
            raise ValueError('duplicate or non-development source')
        jobs[key] = job
        corpus[key] = context.quotes(job['result']['input'])
        provenance.append(dict(file=name, file_sha256=sha, input_sha256=verified['input_sha256'],
                               source_algorithm_sha256=verified['source_algorithm_sha256'],
                               verifier_algorithm_sha256=verified['replay_algorithm_sha256']))
    rows = []
    for cls, symbols in plan['classes'].items():
        for symbol in symbols:
            for (s, _), job in sorted(jobs.items()):
                if s != symbol:
                    continue
                volume = plan['native_volume'][symbol]
                if job['request']['tick_replay']['volume'] != volume:
                    raise ValueError('native volume mismatch')
                for rule in plan['rules'][cls]:
                    row, request = prepare(rule, symbol, cls, job, corpus, volume)
                    if request:
                        row['replay'] = json.loads(subprocess.run([str(a.replay)], input=json.dumps(request),
                            check=True, capture_output=True, text=True, timeout=90).stdout)
                        row['status'] = row['replay']['status']
                    rows.append(row)
    summary, symbols, matched, eligible = summarise(rows, plan)
    files = ['ops/research/full_exit_research.py','ops/research/cost_context.py',
             'ops/research/class_research.py','ops/research/research_gates.py',
             'crates/jiaoyi-core/examples/context_exit_replay.rs',
             'crates/jiaoyi-core/src/research/exit_replay.rs',
             'crates/jiaoyi-core/src/strategy3/targets.rs','crates/jiaoyi-core/src/strategy3/mod.rs']
    result = dict(trace='№18,280 §§1–6 · '+dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT')+' · by Codex (GPT-6), effort not exposed',
                  plan_sha256=FROZEN_SHA256, source_collection_plan_sha256=plan['source_collection_plan_sha256'],
                  code_sha256={f:digest(ROOT/f) for f in files},
                  binary_sha256={'verifier':digest(a.verifier),'replay':digest(a.replay)},
                  provenance=provenance, rows=rows, summary=summary, symbols=symbols,
                  matched_controls=matched, eligible_for_separate_validation=eligible,
                  validation_opened=False, financial_pass=False)
    (a.output/'full-exit-results.json').write_text(json.dumps(result, indent=2, allow_nan=False)+'\n')
    report(result, a.output/'Jiaoyi_Full_Exit_Research.html')
    print(json.dumps(dict(rows=len(rows), statuses=dict(collections.Counter(r['status'] for r in rows)),
                         eligible_for_separate_validation=eligible, financial_pass=False)))


if __name__ == '__main__':
    main()
