#!/usr/bin/env python3
"""Render the frozen unified replay results without rerunning or tuning strategies."""
import argparse
import collections
import datetime as dt
import html
import json
from pathlib import Path
import re
from unified_replay import ROOT, PLAN, RESULT, ARMS, MODES, BPS, sha, read, save

REPORT=ROOT/'doc/REBUILD_PHASE2_PULLBACK_REPORT-18358-2026-11-08.md'
VERIFY=ROOT/'docs/rebuild_phase2_verification_18358.json'

def fmt(x):
    if x is None:return 'Unavailable'
    if isinstance(x,float):return f'{x:.6g}'
    return str(x)
def pct(x):return 'Unavailable' if x is None else f'{100*x:.2f}%'
def table(headers,rows):
    return '| '+' | '.join(headers)+' |\n| '+' | '.join(['---']*len(headers))+' |\n'+'\n'.join('| '+' | '.join(fmt(v).replace('|',' / ').replace('\n',' ') for v in r)+' |' for r in rows)
def markdown_body(text):
    def inline(value):
        value=html.escape(value)
        value=re.sub(r'`([^`]+)`',r'<code>\1</code>',value)
        value=re.sub(r'\*\*(.*?)\*\*',r'<strong>\1</strong>',value)
        return re.sub(r'\[([^\]]+)\]\(([^)]+)\)',r'<a href="\2">\1</a>',value)
    body=[];toc=[];intable=False;incode=False;inlist=False
    for line in text.splitlines():
        if line.startswith('```'):
            body.append('</code></pre>' if incode else '<pre><code>');incode=not incode;continue
        if incode:body.append(html.escape(line)+'\n');continue
        if line.startswith('|'):
            if not intable:body.append('<div class="table-scroll" tabindex="0"><table>');intable=True;first=True
            cells=[x.strip() for x in line.strip('|').split('|')]
            if all(re.fullmatch('[-: ]+',x) for x in cells):continue
            tag='th' if first else 'td';body.append('<tr>'+''.join(f'<{tag}>'+inline(x)+f'</{tag}>' for x in cells)+'</tr>');first=False;continue
        if intable:body.append('</table></div>');intable=False
        if line.startswith('- '):
            if not inlist:body.append('<ul>');inlist=True
            body.append('<li>'+inline(line[2:])+'</li>');continue
        if inlist:body.append('</ul>');inlist=False
        if not line:continue
        if line.startswith('#'):
            n=len(line)-len(line.lstrip('#'));label=line[n:].strip();anchor='section-'+str(len(toc)+1)
            if n==2:toc.append((anchor,inline(label)))
            body.append(f'<h{n}'+(f' id="{anchor}"' if n==2 else '')+'>'+inline(label)+f'</h{n}>')
        else:body.append('<p>'+inline(line)+'</p>')
    if intable:body.append('</table></div>')
    if inlist:body.append('</ul>')
    return ''.join(body),toc

def render(result_path=RESULT,report_path=REPORT):
    r=read(result_path);p=read(PLAN);v=read(VERIFY);c=r['coverage'];allrows=r['rows'];s=r['summary']['full_qualified_core']
    if sha(result_path)!=v['result_sha256'] or sha(PLAN)!=r['plan_sha256']:raise ValueError('report evidence hash mismatch')
    timestamp=v['verified_at_sgt']
    metrics=[]
    for arm in ARMS:
        for mode in MODES:
            for cost in BPS:
                x=s[arm][mode][cost]
                metrics.append([arm,mode,cost,x['entries'],x['priced_closed'],x['wins'],x['losses'],x['unresolved'],pct(x['net_win_rate']),x['mean_net_r'],x['profit_factor'] if x['profit_factor_status']=='finite' else x['profit_factor_status']])
    matrix=table(['Arm','Resolution','bp/side','Entries','Priced closes','Wins','Losses','Unresolved','Net win rate','Mean net R','PF / status'],metrics)
    coverage=table(['Symbol','Eligible definitions','Qualified sessions','Missing 45d prep','Missing opening M30','Missing fresh open quote','24bp forced A: wins/closes','24bp forced B: wins/closes'],[
        [symbol,sum(x['symbol']==symbol for x in allrows+r['excluded'])+sum(x.get('symbol')==symbol for x in r['same_session_aliases']),len([x for x in allrows if x['symbol']==symbol]),
         *[sum(x['symbol']==symbol and reason in x.get('coverage',{}).get('reasons',[]) for x in r['excluded']) for reason in ['missing_45day_session_preparation','missing_opening_M30','missing_fresh_open_quote']],
         *[f"{r['summary'][symbol][arm]['forced']['24']['wins']}/{r['summary'][symbol][arm]['forced']['24']['priced_closed']}" for arm in ARMS[:2]]]
        for symbol in p['symbols']])
    trades=[];context=[]
    for row in allrows:
        if not row['native'].get('arms'):continue
        n=row['native'];own=row['own_context'];peer=row['peer_context'];context.append([row['symbol'],row['date'],pct(own['drift_5']),pct(own['drift_20']),pct(peer['drift_5']) if peer else 'Absent',pct(peer['drift_20']) if peer else 'Absent',n['macro']['reason'],n['pullback']['status']])
        for arm in ARMS:
            a=n['arms'][arm];o=a.get('strict')
            if not o:continue
            e=o['entry'];f=a['forced'];ext=a['continuation'];fill=f.get('outcome',{}).get('fills',[])
            trades.append([row['symbol'],row['date'],arm,e['side'],e['entered_at'],e['fill'],e['initial_r'],e['broker_tp_distance'],o['status'],o['metrics']['net_by_bps']['24'] if o.get('metrics') else None,
                           f['metrics']['net_by_bps']['24'] if f.get('metrics') else None,fill[0]['reason'] if fill else 'Unresolved',ext['status'],ext['metrics']['net_by_bps']['24'] if ext.get('metrics') else None])
    trades_table=table(['Symbol','Date','Arm','Side','Entry UTC','Entry price','Original R price','TP distance','Strict status','Strict net R24','Forced net R24','Forced exit','Continuation status','Continuation net R24'],trades)
    context_table=table(['Symbol','Date','Own5d','Own20d','Peer5d','Peer20d','Macro decision','Pullback'],context)
    join_rows=[]
    for j in r['cost_evidence_joins']:
        if j['key']['symbol'] not in p['symbols']:continue
        join_rows.append([j['key']['account_id'],j['key']['symbol'],','.join(map(str,j['symbol_ids'])),len(j['observed_entry_samples']),j['median_entry_slippage_bps'],len(j['position_money_evidence']),j['round_trip_model_status']])
    joins_table=table(['Account','Symbol','Account-scoped IDs','Demo entry n','Median signed entry bp','Position-money records','Round-trip status'],join_rows)
    benchmarks=[]
    for arm in ARMS:
        for mode in MODES:
            for name in ['demo_entry_plus_2_exit','demo_entry_plus_3.74_exit']:
                x=s[arm][mode][name];benchmarks.append([arm,mode,name,x['priced_closed'],x['cost_unavailable'],x['unresolved'],pct(x['net_win_rate']),x['mean_net_r']])
    bench_table=table(['Arm','Resolution','Later demo entry + assumed exit','Priced closes','Closed but benchmark missing','Unresolved','Win rate','Mean R'],benchmarks)
    session_table=table(['Symbol','Date','Episode','Prior159','Protocol','Prepared slots','Peer','Admission','B result','C result'],[
        [x['symbol'],x['date'],x['episode_id'],x['prior159_member'],x['protocol_status'],x['coverage']['required_prep_slots'],x['coverage']['peer_episode_id'] or 'Absent',x['native']['status'],
        x['native'].get('arms',{}).get(ARMS[1],{}).get('status','no_impulse'),x['native'].get('arms',{}).get(ARMS[2],{}).get('status','no_impulse')] for x in allrows])
    exclusion_table=table(['Symbol','Date','Episode','Reason','Missing prep slots'],[
        [x['symbol'],x['date'],x['episode_id'],', '.join(x.get('coverage',{}).get('reasons',[x.get('status','excluded')])),x.get('coverage',{}).get('missing_prep_slots')] for x in r['excluded']])
    gate_rows=[['G1','Partial','Frozen plan, source/code/binary/manifest and account identity links; inherited unresolved provenance remains.'],
       ['G2','Passed: legacy migration parity','All 159 original24bp baseline decision marks, strict paths and forced paths reproduce exactly before new arms.'],
       ['G3','Passed for this bounded offline runner','One manifest command, denied network, hashed native receipts, explicit denominators and byte-identical verified resume; older runners retained as historical reproducibility tools.'],
       ['G4','Partial','Per-experiment45-day/native4h coverage enforced; missing slots/tails rejected or censored; continuous execution and dated broker calendar not certified.'],
       ['G5','Partial','Existing DST-aware code registry reused unchanged; historical broker holidays/maintenance remain unverified.'],
       ['G6','Partial','Typed account/symbol cost and retained position-money joins; zero complete round-trip cost samples.'],
       ['G7','Not satisfied','13 retained demo entry observations are not historical/live round-trip calibration.'],
       ['G8','Partial','Shared existing two-stage tick/M30 mechanics reused; live broker/restart/partial-fill parity not established.'],
       ['G9','Not satisfied for promotion','Protected dependencies rejected; all 35 evaluated sessions reuse development data. Phase 1 historical exposure caveat retained.'],
       ['G10','Not satisfied','Two correlated-regime pullback trades cannot establish durable expectancy or class/account goals.'],
       ['G11','Deferred','No shadow or canary deployment.'],['G12','Deferred','Live rules untouched; full rebuilt trading lifecycle and rollback not exercised.']]
    gates=table(['Gate','Status','Evidence or limitation'],gate_rows)
    text=f'''# Phase 2 — Unified offline runner and pullback evaluation

№18,358 §§1–9 · {timestamp} · by <Codex (GPT-6), effort: not exposed> · Repository: `ang-kl/jiaoyi`

## §1. TL;DR and measured goal progress

**The infrastructure milestone works. Pullback is a promising development observation, not a qualified edge.** All **159** old baseline episodes reproduce exactly. The new comparison admits **{c['qualified_unique_core_sessions']} unique sessions** out of **{c['core_eligible_definitions']} core-symbol definitions**, with the protected receipts excluded.

At the **24 bp per-side stress assumption**, the four-hour forced-close comparison is:

- **A — breakout:** 3 entries, 0/3 positive closes, mean **−1.064041R**.
- **B — pullback:** 2 entries, 2/2 positive closes, mean **+0.653291R**.
- **C — macro-filtered pullback:** the same 2 entries and outcomes as B. **No incremental filter benefit is demonstrated.**

Strict tick evidence is smaller: B/C have **one target win and one unresolved trade**. The unresolved SpotCrude trade is positive at forced window end; a subsequent **conditional M30 approximation** reaches its target. These are alternative resolutions of the same trade, never extra independent wins. PF is **undefined/no losses**, not an invented large finite value, for the two-win pullback sample.

**Goal progress:** 0 new independent trade observations; 0 account/strategy qualifications. The 73% / reference 1:1.38 goal over the latest 20 trades and interim 50% research target remain unqualified. A 2/2 point estimate on reused tariff-week data is too small to establish either target; metals and crypto have no qualifying trade evidence here. Successful CI establishes software checks, not market edge.

Read the [frozen plan](../docs/rebuild_phase2_runner_and_pullback_plan_18358.json), [full measured results](../docs/rebuild_phase2_runner_and_pullback_results_18358.json), and [verification](../docs/rebuild_phase2_verification_18358.json). No broker requests, live trades, production rule changes or paid model calls occur in this phase.

## §2. Frozen interpretation and economic hypothesis

The proposed mechanism is that waiting for a retracement can improve an entry after a short-term impulse, while preceding multi-day direction can restrict countertrend exposure. This is a hypothesis, not evidence of institutional flow. The “macro” arm uses only **technical price-trend context**; it contains no fundamental, earnings, news, positioning or macroeconomic release data.

The baseline remains the actual native S3v3 **half-hour mark** rule, with causal 14-session sigma, full 24 bp cost veto and 48 bp risk floor at decision and dispatch. The source code does not know the future peak. The impulse is the first admitted baseline event and its recorded dispatch midpoint. The fixed 50% retracement is set from that midpoint and the fresh session-opening midpoint; later highs/lows do not move it. A strictly later executable ask/bid must touch the rounded level. A touch that has already crossed the origin is rejected. Limit queue/priority is not observable, so the touch fill is a research approximation.

For B/C, structural risk is `ceil_grid(abs(entry − open_mid) + observed spread + 2×entry×24/10000)`. The stop lies beyond the origin; the original R then controls the +1R entry-price stop amendment and outward-rounded 1.38R gross target. **This changes both entry timing and risk/target geometry.** The result cannot isolate a pure entry-timing effect. The cost-dependent geometry follows the owner's requested definition; the sensitivity columns hold that geometry and all fills fixed.

Five- and 20-day drift use the last 5/20 **complete coded trading sessions**, drawn from the 45-calendar-day preparation, with every bar completed before today's open. Both own drifts must strictly agree with the impulse direction. If an eligible same-account peer episode exists that date, complete peer context is required and neither peer drift may oppose it. An absent same-date peer is explicitly allowed by the frozen conditional rule; missing context for a present peer is a veto.

The new observation window is **native session open to open + 4 hours**. Thus oil now uses 07:00–11:00UTC. The legacy parity run preserves its old 07:00–17:00 source path and 13:00–17:00 reporting window. The 35-session new comparison is deliberately separate from the 159-episode legacy performance table.

## §3. Coverage, exclusions and protected data

{coverage}

The 379 eligible manifest definitions include 263 core-symbol definitions and 116 other symbols outside this experiment. Of the 263, 224 lack complete 45-day coded-session preparation; some also lack the opening M30 bar. Of the 39 with complete preparation, four oil episodes on 7–8 November 2024 lack a fresh opening quote within 5 seconds, leaving 35. Exclusion reasons can overlap and must not be summed as independent counts. All 35 are in the original 159 source subset, so **the full qualified-core and prior159-qualified summaries are identical**. The other 124 prior episodes still passed legacy parity; they did not silently become successful new-arm evaluations.

All six quarantined receipt hashes are rejected, including the duplicate alias. No protected prices or outcomes are evaluated. The earlier discovery of protected preparation means historical untouched holdout status remains **not established**. These reused high-volatility cohorts are development evidence, not independent validation.

Complete coded-session slots are a bounded research coverage check. Historical broker calendars, holidays, quote-sequence completeness, queue priority and contract terms remain unverified. Long quote intervals and same-millisecond ordering issues remain in each native quality record. No omitted price path is interpolated into a purported actual fill.

## §4. Complete cost and resolution matrix

“Net R” below is **net of the stated partial cost scenario**. Recorded bid/ask fills include spread once. Additional adverse slippage is `(entry+exit)×bps/10000/R`. Commission, swap, conversion and other unrecorded round-trip components remain unknown; zero extra charges in the arithmetic are an omission, not a verified zero fee.

Strict mode counts actual simulated bracket exits before the four-hour boundary; surviving positions remain unresolved. Forced mode separately uses a fresh closing quote within 5 seconds, with existing brackets first. Continuation preserves the strict state and processes only consecutive cached M30 bars up to 72 elapsed hours from entry. The first gap, protected interval, uncertain OHLC ordering or partial deadline remains censored. Short ask OHLC uses the explicitly assumed frozen entry-spread proxy, and financing is not included.

{matrix}

Both lower-cost columns **reprice the same entries**. They do not reopen the cost gate or search for additional favourable trades. Mean R is per priced closed trade and does not turn no-entry or censored episodes into zero-profit observations.

## §5. Individual trades and what changed

{trades_table}

**US500, 8 April 2025:** the breakout short stopped before the later retracement. The pullback short entered after that retracement and reached its target inside the observed tick window. Its risk and target distances were also different, so the improvement is attributable to the combined frozen redesign, not proven entry timing alone.

**SpotCrude, 11 April 2025:** the breakout short moved its stop to entry and closed at gross breakeven, which is a loss after the stress reserve. The later pullback short remained open at 11:00 UTC. Its forced-close result was +0.311540R; the conditional M30 target resolution was +0.879624R. That M30 result is not an additional tick-verified win.

**US500, 11 April 2025:** the baseline long lost at forced close. Price never reached the fixed retracement before the deadline, so B took no trade. C also rejected the conflicting 5d/20d direction. Since B already abstained, this does not establish incremental value from C.

{context_table}

The two pullback trades occur in the same April 2025 tariff-volatility cohort, on different dates/assets. They are not representative random samples or evidence of stable performance across regimes. No metal or crypto edge was demonstrated. There is no honest basis for advertising “100% win rate achieved” or promoting this candidate to live trading.

## §6. Typed cost evidence and empirical benchmark limits

{joins_table}

The complete inventory retains 13 demo entry observations. Joins require **account plus symbol**; numeric symbol IDs are not global. Current account-specific contracts and retained position-money records remain attached without inventing a round-trip cost model. Statement account totals are never attributed wholesale to a symbol. Unknown benchmarks remain null.

For research account 49236051, SpotCrude has one retained entry benchmark of approximately −0.630 bp; the US500 benchmark is absent. These observations were captured later than the historical experiment. They are used **only as retrospective sensitivity benchmarks**, never causal admission or macro features. An assumed 2 or 3.74 bp exit leg does not turn the measured entry leg into a measured round trip. Other demo accounts and live accounts are not pooled.

{bench_table}

## §7. Verification, acceptance gates and next decision

- Frozen plan SHA256: `{r['plan_sha256']}`.
- Result SHA256: `{sha(result_path)}`.
- Exact baseline parity: 159/159 decision-mark, strict-path and forced-path comparisons.
- Verified source files: {c['verified_local_files']}; quarantined receipt hashes rejected: 6.
- Independent gross/net arithmetic checks: {r['verification']['independent_cost_checks']}.
- Focused tests: two Rust and four Python tests, plus retained existing exit tests in CI.
- Resumed run: 159 baseline and 35 experiment cache hits; output bytes match the initial run.
- Native source and binary hashes, source receipts and plan identity are recorded. No network calls are permitted by the Python runner; native adapters are local-file programs without a broker transport route.

{gates}

**Next:** keep B unchanged as a candidate awaiting replication. Do not tune the retracement or reduce costs to improve this tiny result. First register an independent evaluation scope, resolve the known historical-holdout exposure, and identify the smallest missing preparation/contract/cost evidence. If entry timing itself is the next question, freeze a separate matched-risk control before evaluation; this run changed both timing and geometry. C needs a sample where it actually changes B's entries before its incremental value can be assessed.

No further broker collection, historical-spec retrieval, live migration or capital exposure is performed here. Within the current offline cache the qualified comparison is complete. The original goals, traceability, SGT timestamps, preserving proven fixes, and “Time is Money” remain the operating invariants.

Reproduce from the exact cached sources, with a fresh work directory for a different code revision:

```text
cargo build -p jiaoyi-core --example native_target_sizing_replay --example manifest_pullback_replay --example replay_inventory_metadata --locked --offline
python3 ops/research/unified_replay.py --workdir /tmp/jiaoyi-phase2-replay --output /tmp/jiaoyi-phase2-results.json
python3 -m unittest discover -s ops/research -p test_unified_replay.py
cargo test -p jiaoyi-core research::manifest_pullback --locked --offline
```

Running the same command again verifies the cached native receipts and returns deterministic results. Missing or changed files fail closed. A Git clone contains the report and result evidence; replay also requires the Phase 1 manifest's local source cache. Original historical scripts remain for old frozen experiments; the new comparison uses the canonical manifest command.

## §8. Complete evaluated-session register

{session_table}

## §9. Complete excluded-definition register

{exclusion_table}
'''
    report_path.write_text(text)
    body,headers=markdown_body(text)
    nav='<nav>'+''.join(f'<a href="#{i}">{t}</a>' for i,t in headers)+'</nav>'
    pos=body.find('<h2',body.find('<h2')+1);body=body[:pos]+nav+body[pos:]
    css='''body{font:17px/1.6 system-ui,sans-serif;color:#19344a;background:#fff;max-width:1260px;margin:24px auto;padding:0 20px}h1,h2,h3{line-height:1.25}h2{margin-top:2.6rem;border-top:2px solid #d5e1eb;padding-top:1.2rem}a{color:#075b99}p,li{overflow-wrap:anywhere}nav{display:flex;flex-wrap:wrap;gap:10px;padding:18px;background:#eef5fa}nav a{font-size:14px}.table-scroll{overflow:auto;max-height:620px;border:1px solid #d5e1eb;margin:20px 0}table{border-collapse:collapse;font-size:14px;min-width:700px;width:100%}th,td{text-align:left;padding:9px;border-bottom:1px solid #d5e1eb;vertical-align:top}th{position:sticky;top:0;background:#e5eff7;color:#153a56}tr:nth-child(even){background:#f6f9fc}pre{overflow:auto;padding:16px;background:#edf2f6;font-size:13px}code{font-size:.88em;overflow-wrap:anywhere}.status{padding:16px;border-left:5px solid #b66e13;background:#fff1db} @media(max-width:600px){body{font-size:16px;padding:0 14px;margin:18px auto}h1{font-size:28px}.table-scroll{max-height:480px}}'''
    page=f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi Phase 2 — №18,358</title><style>{css}</style></head><body><div class="status"><strong>Infrastructure verified · Trading qualification: NO</strong><br>159 baseline matches · 35 qualified sessions · 2 pullback trades · 0 new broker calls</div>{body}</body></html>'
    report_path.with_suffix('.html').write_text(page)
    print(report_path)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--results',type=Path,default=RESULT);p.add_argument('--report',type=Path,default=REPORT);a=p.parse_args();render(a.results,a.report)
