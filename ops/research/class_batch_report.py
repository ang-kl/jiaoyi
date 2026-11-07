"""Report frozen batch results without pooling competing rules or claiming an edge."""
import argparse
from collections import Counter
from datetime import datetime, timezone, timedelta
import html
import hashlib
import json
import re
from pathlib import Path
from class_batch import digest

ARMS = {'trailing': ('trailing_scenario_net_r', 'candidate'),
        'fixed': ('fixed_stop_scenario_net_r', 'baseline')}


def score(rows, rule, arm):
    field, side = ARMS[arm]
    values, reasons, issues = [], Counter(), Counter()
    entries = censored = qualified = complete = 0
    for row in rows:
        if row['status'] != 'complete':
            reasons[row['status']] += 1
            continue
        complete += 1
        study = row['replay']['studies'][rule]
        reasons.update(mark['reason'] for mark in study['marks'])
        trade = study['trade']
        if trade is None:
            continue
        entries += 1
        outcome = trade['exit_comparison'][side]
        issues.update(outcome['data_issues'])
        qualified += outcome['net_r'] is not None
        if trade[field] is None:
            censored += 1
        else:
            values.append(trade[field])
    wins = sum(v > 0 for v in values)
    losses = sum(v < 0 for v in values)
    profit = sum(v for v in values if v > 0)
    loss = -sum(v for v in values if v < 0)
    return dict(episodes=len(rows), complete=complete, entries=entries,
                scenario_closed=len(values), wins=wins, losses=losses,
                breakevens=len(values)-wins-losses, censored_or_invalid=censored,
                scenario_win_rate=wins/len(values) if values else None,
                scenario_mean_r=sum(values)/len(values) if values else None,
                scenario_pf_r=profit/loss if loss else None,
                pf_status='available' if loss else ('no_losses' if values else 'no_closes'),
                qualified_net_closes=qualified, reasons=dict(reasons), data_issues=dict(issues),
                financial_pass=False)


def summarize(plan, results):
    expected = {j['key']: j for j in plan['jobs']}
    rows = results['rows']
    if len(rows) != len(expected) or {r['key'] for r in rows} != set(expected):
        raise ValueError('expected job coverage differs')
    if results['plan_sha256'] != digest(plan):
        raise ValueError('result/plan hash mismatch')
    for r in rows:
        j = expected[r['key']]
        if (r['symbol'], r['asset_class'], r['date']) != (j['request']['symbol'], j['class'], j['date']):
            raise ValueError('episode identity differs')
        if r['status'] == 'complete' and set(r['replay']['studies']) != set(plan['rules']):
            raise ValueError('missing or unexpected hypothesis')
    groups = {}
    for cls, symbols in plan['classes'].items():
        for name, subset in [(cls, [r for r in rows if r['asset_class'] == cls])] + [
                (s, [r for r in rows if r['symbol'] == s]) for s in symbols]:
            groups[name] = {rule: {arm: score(subset, rule, arm) for arm in ARMS} for rule in plan['rules']}
    unique_entry_episodes = sum(any(s['trade'] is not None for s in r.get('replay', {}).get('studies', {}).values()) for r in rows)
    coverage = []
    for cls, symbols in plan['classes'].items():
        for s in symbols:
            subset = [r for r in rows if r['symbol'] == s]
            quality = [next(iter(r['replay']['studies'].values()))['quality']
                       for r in subset if r['status'] == 'complete']
            coverage.append(dict(symbol=s, asset_class=cls, status=dict(Counter(r['status'] for r in subset)),
                                 ticks=sum(q['bid_ticks']+q['ask_ticks'] for q in quality),
                                 long_intervals=sum(q['long_intervals'] for q in quality),
                                 stale_side=sum(q['stale_side'] for q in quality),
                                 same_millisecond_groups=sum(q['same_millisecond_groups'] for q in quality),
                                 crossed=sum(q['crossed'] for q in quality)))
    return dict(trace='№18,309–18,310 §§1–8; by Codex (GPT-6), effort not exposed',
                plan_sha256=results['plan_sha256'], binary_sha256=results['binary_sha256'],
                status=dict(Counter(r['status'] for r in rows)), coverage=coverage,
                unique_entry_episodes=unique_entry_episodes,
                scores=groups, financial_pass=False, validation_opened=False,
                interpretation='Scenario scores only; competing rules and exit arms are not independent trades.')


def fmt(value, percent=False):
    if value is None:
        return 'N/A'
    return f'{value*100:.1f}%' if percent else f'{value:.4f}'


def score_table(scores):
    headers = ['Rule', 'Exit', 'Entries', 'Closed', 'Wins', 'Win %', 'Mean R', 'PF(R)', 'Censored/invalid']
    rows = []
    for rule, arms in scores.items():
        for arm, s in arms.items():
            rows.append([rule.replace('class_', ''), arm, s['entries'], s['scenario_closed'], s['wins'],
                         fmt(s['scenario_win_rate'], True), fmt(s['scenario_mean_r']),
                         fmt(s['scenario_pf_r']), s['censored_or_invalid']])
    return table(headers, rows)


def table(headers, rows):
    e = lambda v: html.escape(str(v))
    return '<div class="scroll"><table><thead><tr>' + ''.join('<th>'+e(x)+'</th>' for x in headers) + '</tr></thead><tbody>' + ''.join(
        '<tr>'+''.join('<td>'+e(x)+'</td>' for x in row)+'</tr>' for row in rows) + '</tbody></table></div>'


def write_report(plan, results, evidence, destination):
    summary = summarize(plan, results)
    events = [json.loads(line) for line in (evidence/'ledger.jsonl').read_text().splitlines()]
    started = [e for e in events if e['state'] == 'started']
    finished = [e for e in events if e['state'] == 'finished']
    if len(started) != len({e['key'] for e in started}) or len(finished) != len({e['key'] for e in finished}):
        raise ValueError('duplicate collection receipts')
    if {e['key'] for e in started} != {e['key'] for e in finished}:
        raise ValueError('unresolved collection receipt')
    if any(e['plan_sha256'] != digest(plan) for e in started):
        raise ValueError('collection plan hash mismatch')
    rows_by_key = {r['key']: r for r in results['rows']}
    for event in finished:
        raw = evidence/('job-'+event['key']+'.json')
        if (hashlib.sha256(raw.read_bytes()).hexdigest() != event['file_sha256']
                or rows_by_key[event['key']]['file_sha256'] != event['file_sha256']
                or rows_by_key[event['key']]['status'] != event['status']):
            raise ValueError('source/receipt/replay mismatch')
    summary['collection_receipts'] = events
    now = datetime.now(timezone(timedelta(hours=8))).strftime('%d-%m-%Y %H:%M SGT')
    title = 'Forty-symbol bounded backtest: five asset classes'
    provenance = json.loads((evidence/'freeze.json').read_text())
    summary['provenance'] = provenance
    summary['generated_at_sgt'] = now
    base = 'CLASS40_BACKTEST-18310-2026-11-07'
    destination.mkdir(parents=True, exist_ok=True)
    (destination/(base+'.json')).write_text(json.dumps(summary, indent=2, allow_nan=False)+'\n')
    # Compact inspectable native evidence: remove repeated explanatory notes only.
    compact = json.loads(json.dumps(results))
    for row in compact['rows']:
        for study in row.get('replay', {}).get('studies', {}).values():
            study.pop('notes', None)
    (destination/(base+'-episodes.json')).write_text(json.dumps(compact, separators=(',', ':'), allow_nan=False)+'\n')
    status = ', '.join(f'{k}: {v}' for k,v in summary['status'].items())
    crypto_strict = summary['scores']['crypto']['class_chop_reversion1h']['trailing']
    crypto_observed = summary['scores']['crypto']['class_chop_reversion_observed1h']['trailing']
    energy_observed = summary['scores']['energy']['class_chop_reversion_observed1h']['trailing']
    index_chop = summary['scores']['equity_index']['class_chop_reversion1h']['trailing']
    fx_entries = [v['trailing']['entries'] for v in summary['scores']['fx'].values()]
    findings = f'''FX entry counts across the six rules: **{fx_entries}**. Crypto low-efficiency
reversal (trailing) closed **{crypto_strict['scenario_closed']}**, won
**{crypto_strict['wins']}** ({fmt(crypto_strict['scenario_win_rate'], True)}), mean
**{fmt(crypto_strict['scenario_mean_r'])}R**, PF(R)
**{fmt(crypto_strict['scenario_pf_r'])}**. Its observed-feature counterpart closed
**{crypto_observed['scenario_closed']}**, won **{crypto_observed['wins']}**
({fmt(crypto_observed['scenario_win_rate'], True)}), mean
**{fmt(crypto_observed['scenario_mean_r'])}R**. Energy observed low-efficiency reversal
closed **{energy_observed['scenario_closed']}**, won **{energy_observed['wins']}**,
mean **{fmt(energy_observed['scenario_mean_r'])}R**. Index low-efficiency reversal
closed **{index_chop['scenario_closed']}**, won **{index_chop['wins']}**, mean
**{fmt(index_chop['scenario_mean_r'])}R**. All metal rule/exit results are in the tables.
These are separate scenario cohorts, not trades to pool or evidence to rank a winner.
'''
    intro = f'''## §1. TLDR

The frozen 40-symbol / 80-episode pilot has collection status **{status}**.
There were **{summary['unique_entry_episodes']} distinct symbol/date episodes with at least one
simulated entry** across the six hypotheses. This is not an independent trade count.
Six existing first-hour hypotheses were evaluated separately, with fixed stops and
original-R trailing exits. **No symbol or asset class qualifies for deployment from this pilot.**
The two development dates cannot establish the interim 50% net win-rate goal, much less
73% / 1:1.38 over the latest 20 closed trades per strategy/account. Newly qualified
account trades: **0**. Validation remains sealed. This is broader research coverage,
not demonstrated financial progress.

{findings}

## §2. Scope and frozen method

Five supported program classes: 7 equity indices, 14 FX pairs, 4 metals, 3 energy
contracts and 12 crypto contracts. This does not include individual equities,
soft commodities, exchange options or bonds; China stocks and indices are excluded.
Dates: **7 and 14 July 2025**, four hours from each registered session open plus
five seconds for a closing quote. Demo account 49236051 supplies historical bid/ask
updates; no trading request is made. US indices use 09:30 New York, GER40 09:00 Berlin,
UK100 08:00 London, JPN225 09:00 Tokyo, AUS200 10:00 Sydney; FX/metals/energy 07:00 UTC,
crypto 00:00 UTC. These are existing research conventions, not proven best sessions.

Frozen rules: high-efficiency first-hour momentum and reversal, low-efficiency
reversal, and each rule's existing observed-feature counterpart. They use a causal
first-hour range and next strictly later fresh quote. All six are shown; the report
selects no winner. They are **class research hypotheses, not a replay of production
S3's entry preparation or the account admission state**.

Cost scenario: 24 basis points per side, zero explicit round-trip commission,
observed bid/ask spread charged once. These assumptions are **not broker-calibrated**.
A cost reserve at least as large as original R vetoes the entry. The engine uses
SL, TP planning, conditional partial TP, fixed-stop comparison, 30-second original-R
trailing proposals and session close. Quantity is the current broker minimum, not
historical account risk sizing. At the minimum quantity a partial exit may be
unavailable; the target planner handles that constraint.

## §3. Evidence and interpretation

Every symbol has two predeclared jobs. Collection failures, empty history, missing
features, cost vetoes, censored exits and losses remain distinct. Only fully closed
scenario positions enter win rates; zero closes yields N/A. PF is gross positive
scenario R divided by absolute negative scenario R, **not a dollar portfolio PF**.
Do not pool competing hypotheses or fixed/trailing arms as independent trades.

Historical tick completeness, same-millisecond cross-side sequencing, historical
contract terms, actual execution latency, commission/funding and account admission
are unverified. Long gaps are flags, not automatically proof of dropped broker data.
Current contracts were rechecked against all 40 exact symbols before collection.
No order, live strategy parameter, live symbol universe or validation seal changed.

## §4. Reproducibility and checks

Frozen implementation commit: `{provenance['commit']}`. Plan SHA256:
`{summary['plan_sha256']}`. Native replay binary SHA256:
`{summary['binary_sha256']}`. Source hashes are in the companion JSON.
Raw bid/ask job files and append-only collection receipts are retained outside Git
in `/workspace/jiaoyi-class40-evidence-20261107/`; this path is not an archival
service. Raw quotes are not duplicated in the repository. The episodes JSON
contains source hashes, native features, reasons, exit fills and quality flags;
independent raw reproduction needs the retained private corpus.

Reproduce locally with the configured HTTPS proxy and cTrader credentials kept in
private environment variables (the collector requires Python `websockets`):

`cargo build -p jiaoyi-core --example class_batch_replay --locked`

`python ops/research/class_batch.py replay --plan docs/class-batch-40-plan-18309-2026-11-07.json --output /workspace/jiaoyi-class40-evidence-20261107`

`python ops/research/class_batch_report.py --evidence /workspace/jiaoyi-class40-evidence-20261107 --destination doc`

Use the existing corpus for replay. The `collect` action is the separately bounded
read phase: receipts are written before jobs; failed or uncertain jobs are not
silently retried. Never remove receipts to obtain a second budget.

Three new collector checks cover signed delta decoding, disjoint truncated-page
splitting/budgets, and rejection of order messages/wrong accounts. Existing class
causality/session tests now include all 40 symbols. Rust format, core Clippy and
focused tests passed locally; repository CI and merge receipt belong to the PR.

## §5. Reading the tables

The HTML report provides a class table for all six rules and both exits, followed
by per-symbol tables. The JSON provides exact values and decision-reason counts.
The coverage table below counts each symbol/date once, not once per hypothesis.
'''
    md = '# '+title+'\n\n№18,309–18,310 · §§1–8 · '+now+'\nBy Codex (GPT-6), effort not exposed.\n\n'+intro
    md += '\n| Class | Rule | Entries | Trailing closed/wins | Trailing win % | Trailing mean R | Trailing PF(R) | Fixed closed/wins | Fixed win % | Fixed mean R | Fixed PF(R) |\n|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n'
    for cls in plan['classes']:
        for rule in plan['rules']:
            a = summary['scores'][cls][rule]['trailing']
            b = summary['scores'][cls][rule]['fixed']
            md += f"| {cls} | {rule.replace('class_', '')} | {a['entries']} | {a['scenario_closed']}/{a['wins']} | {fmt(a['scenario_win_rate'],True)} | {fmt(a['scenario_mean_r'])} | {fmt(a['scenario_pf_r'])} | {b['scenario_closed']}/{b['wins']} | {fmt(b['scenario_win_rate'],True)} | {fmt(b['scenario_mean_r'])} | {fmt(b['scenario_pf_r'])} |\n"
    md += '\n| Class | Symbol | Status | Bid + ask updates | Long gaps | Stale-side snapshots |\n|---|---|---|---:|---:|---:|\n'
    for c in summary['coverage']:
        md += f"| {c['asset_class']} | {c['symbol']} | {c['status']} | {c['ticks']} | {c['long_intervals']} | {c['stale_side']} |\n"
    md += '''
## §6. Failure prevention and next decisions

1. Audit actual cost assumptions before claiming that a cost gate proves there is
   no edge. Do not lower the reserve merely to produce trades. Collect independent
   historical spread, commission, slippage and contract evidence for any new model.
2. Separate missing-feature/quote coverage problems from rejected signals. Diagnose
   the reason and quantify coverage before changing sampling or entry rules.
3. For an economically plausible candidate, document a mechanism specific to its
   instrument/class and predeclare rules, costs, development dates and a new bounded
   sample budget. Two common dates across correlated symbols are not independent
   evidence of broad generalisation. Do not tune these six rules against these results.
4. Verify implementation causality and fill/exit accounting with focused regressions;
   then run untouched validation dates under the existing seal policy. Require net
   expectancy, PF, uncertainty, drawdown and realistic cost stress, not just 50% wins.
5. Reconcile account-level sizing, margin, pending-order state and deployment parity
   before any separately authorised promotion. A passing CI build is not this gate.

## §7. Independent AI review request

Read the frozen plan and implementation, summary JSON and native episode evidence.
Challenge the first-hour hypothesis, session convention, 24bp-per-side cost scenario,
current-versus-historical contract mismatch and quote coverage. Verify every
numerator/denominator, missing-data case and arm separately. Distinguish a result
under assumptions from a verified trading edge. Propose the smallest evidence-backed
next experiment; do not recommend deployment or reserve reduction from win rate alone.

## §8. Files

'''
    md += f'- [Readable HTML]({base}.html)\n- [Scores and provenance]({base}.json)\n- [Native episode evidence]({base}-episodes.json)\n- [Frozen plan](../docs/class-batch-40-plan-18309-2026-11-07.json)\n'
    (destination/(base+'.md')).write_text(md)
    # No JS, CDN or external service. Plain HTML stays readable offline.
    sections = []
    for cls, symbols in plan['classes'].items():
        sections.append('<section><h2>'+html.escape(cls)+'</h2>'+score_table(summary['scores'][cls]))
        for symbol in symbols:
            reasons = {rule: summary['scores'][symbol][rule]['trailing']['reasons'] for rule in plan['rules']}
            sections.append('<details><summary>'+symbol+' — two dates, six rules</summary>'+score_table(summary['scores'][symbol])+
                            '<pre>'+html.escape(json.dumps(reasons, indent=2))+'</pre></details>')
        sections.append('</section>')
    overview = table(['Class','Symbol','Collection status','Bid + ask updates','Long gaps','Stale side','Same-ms groups'],[
        [c['asset_class'],c['symbol'],c['status'],c['ticks'],c['long_intervals'],c['stale_side'],c['same_millisecond_groups']]
        for c in summary['coverage']])
    def paragraph(text):
        content = html.escape(text.strip()).replace('\n', ' ')
        content = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', content)
        content = re.sub(r'`([^`]+)`', r'<code>\1</code>', content)
        return '<h2>'+content[3:]+'</h2>' if content.startswith('## ') else '<p>'+content+'</p>'
    narrative = ''.join(paragraph(p) for p in intro.split('\n\n'))
    page = '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>'+title+'</title><style>body{font:16px/1.6 system-ui,sans-serif;color:#182333;background:#f4f6f9;max-width:1280px;margin:auto;padding:24px}section,header{background:white;padding:24px;margin:20px 0;border-radius:12px}h1{font-size:30px}h2{font-size:23px}table{border-collapse:collapse;width:100%;font-size:14px}th,td{padding:9px;text-align:left;border-bottom:1px solid #d5dbe4}th{background:#e9eef6}.scroll{overflow-x:auto}summary{cursor:pointer;font-weight:650;padding:14px}pre{white-space:pre-wrap;font-size:13px}.status{background:#fff1c8;padding:16px;border-left:5px solid #8c5900}a{color:#144ca4}p{max-width:100ch}code{overflow-wrap:anywhere}</style><header><h1>'+title+'</h1><p>№18,309–18,310 · '+now+' · Codex (GPT-6), effort not exposed</p><div class="status"><b>TLDR:</b> '+html.escape(status)+'. No symbol or class is qualified for deployment. Zero newly qualified account trades; 50% interim and 73% / 1:1.38 goals remain unverified.</div></header><section>'+narrative+'</section><section><h2>Coverage: 40 symbols</h2>'+overview+'</section>'+''.join(sections)+'<section><h2>Next steps and review</h2><p>Calibrate costs from independent evidence; diagnose missing features before changing rules; freeze any new economically motivated experiment before untouched validation. Check expectancy, PF, uncertainty, drawdown, account parity and costs. Do not select a winner or pool arms from this pilot.</p><p><a href="'+base+'.md">Full method and five verification gates</a> · <a href="'+base+'.json">Scores and provenance JSON</a> · <a href="'+base+'-episodes.json">Native episode evidence</a></p></section></html>'
    (destination/(base+'.html')).write_text(page)
    print(json.dumps(dict(status=summary['status'],symbols=len(summary['coverage']),financial_pass=False)))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--evidence', type=Path, required=True)
    p.add_argument('--destination', type=Path, default=Path('doc'))
    args = p.parse_args()
    write_report(json.loads((args.evidence/'plan.json').read_text()),
                 json.loads((args.evidence/'replay-results.json').read_text()), args.evidence, args.destination)
