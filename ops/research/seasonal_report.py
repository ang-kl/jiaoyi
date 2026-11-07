"""Scenario accounting, weekly matched controls, and the owner's final review block."""
import argparse
import collections
import datetime as dt
import html
import itertools
import json
from pathlib import Path
import statistics

from seasonal_pilot import PLAN, EXECUTION, MODELS, load, sha, events

ARMS={m:['trailing','fixed'] if m in MODELS[:3] else ['fixed_horizon'] for m in MODELS}
BPS=['24','2','3.74']


def model(row,name): return row.get('replay',{}).get('result',{}).get('models',{}).get(name,{})

def observation(row,name,arm,bps):
    m=model(row,name);trade=m.get('study',{}).get('trade');control=m.get('control')
    if arm != 'fixed_horizon':
        value=m.get('sensitivities',{}).get(arm,{}).get(bps)
        r=trade['entry']['initial_r'] if trade else None
        entered=trade is not None
    else:
        value=control.get('net_by_bps',{}).get(bps) if control else None
        r=control['initial_r'] if control else None
        entered=control is not None
    return entered,value,r


def reasons(row,name):
    if row['status']!='complete': return [row['status']]
    m=model(row,name)
    return sorted(set([m['status']] if 'status' in m else [x['reason'] for x in m.get('study',{}).get('marks',[])]))


def gap(row,name):
    if row['status']!='complete' or row.get('bar_component_failures',0):return True
    q=row['replay']['result']['quality']
    if any(q.get(k,0)>0 for k in ('missing_side','stale_side','crossed','long_intervals','same_millisecond_groups')):return True
    if not q.get('bid_ticks') or not q.get('ask_ticks'):return True
    if any(r.startswith(('missing_','no_fresh_','no_next_')) for r in reasons(row,name)):return True
    entered,value,_=observation(row,name,ARMS.get(name,['trailing'])[0],'24')
    return entered and value is None


def score(rows,name,arm,bps,min_closed):
    vals=[];prices=[];entries=0;counts=collections.Counter();weeks=set()
    for row in rows:
        entered,value,r=observation(row,name,arm,bps);entries+=entered
        counts.update(reasons(row,name))
        if value is not None:
            vals.append(value);prices.append(value*r)
            weeks.add(tuple(dt.date.fromisoformat(row['date']).isocalendar()[:2]))
    wins=sum(x>0 for x in vals);loss=-sum(x for x in vals if x<0);profit=sum(x for x in vals if x>0)
    by_symbol={}
    for symbol in sorted({r['symbol'] for r in rows}):
        outcomes=[observation(r,name,arm,bps) for r in rows if r['symbol']==symbol]
        by_symbol[symbol]=sum(v*r for _,v,r in outcomes if v is not None) if any(v is not None for _,v,_ in outcomes) else None
    failures=[]
    if len(vals)<min_closed:failures.append('insufficient_closes')
    if len(weeks)<4:failures.append('fewer_than_four_closed_weeks')
    if not vals or wins/len(vals)<.5:failures.append('net_win_rate_below_50_or_unavailable')
    if not vals or statistics.mean(vals)<=0:failures.append('nonpositive_or_unavailable_expectancy')
    if not vals or profit<=loss:failures.append('pf_not_above_one_or_unavailable')
    if any(v is None or v<=0 for v in by_symbol.values()):failures.append('nonpositive_or_unavailable_symbol_price_pnl')
    gaps=sum(gap(row,name) or (observation(row,name,arm,bps)[0] and observation(row,name,arm,bps)[1] is None) for row in rows)
    if gaps:failures.append('coverage_or_sequence_flags')
    return dict(episodes=len(rows),collected=sum(r['status']=='complete' for r in rows),entries=entries,closed=len(vals),wins=wins,
                losses=sum(v<0 for v in vals),breakevens=sum(v==0 for v in vals),censored_or_invalid=entries-len(vals),
                win_rate=wins/len(vals) if vals else None,wins_per_scheduled_episode=wins/len(rows) if rows else None,
                mean_r=statistics.mean(vals) if vals else None,pf=profit/loss if loss else None,
                pf_status='finite' if loss else ('no_losses' if vals else 'no_closes'),
                net_price_units_by_symbol=by_symbol,weeks=len(weeks),data_gap_or_flag_episodes=gaps,
                reasons=dict(counts),numerical_screen_pass=not failures,rejection_reasons=failures,financial_pass=False)


def paired(rows,name,control):
    by_week=collections.defaultdict(list);pairs=[]
    for row in rows:
        _,a,_=observation(row,name,'trailing','24');_,b,_=observation(row,control,'fixed_horizon','24')
        if a is not None and b is not None:
            week=tuple(dt.date.fromisoformat(row['date']).isocalendar()[:2]);delta=a-b
            by_week[week].append(delta);pairs.append(dict(key=row['key'],delta_r=delta))
    means=[statistics.mean(v) for _,v in sorted(by_week.items())]
    pvalue=None
    if means:
        observed=sum(means)
        pvalue=sum(sum(s*x for s,x in zip(signs,means))>=observed-1e-12 for signs in itertools.product((-1,1),repeat=len(means)))/(2**len(means))
    return dict(control=control,matched=len(pairs),unmatched_episodes=len(rows)-len(pairs),matched_weeks=len(means),
                mean_delta_r=statistics.mean(p['delta_r'] for p in pairs) if pairs else None,
                weekly_mean_delta_r=statistics.mean(means) if means else None,
                one_sided_weekly_sign_flip_p=pvalue,
                significance_pass=len(means)>=4 and pvalue is not None and pvalue<=.05,
                caveat='Same symbol/date complete pairs; differing entry/exit policies and correlated weeks limit causal inference.',pairs=pairs)


def summarize(result,execution):
    rows=result['rows'];expected={j['key'] for j in execution['jobs']}
    if len(rows)!=80 or {r['key'] for r in rows}!=expected:raise ValueError('missing/duplicate episode denominator')
    if result['plan_sha256']!=sha(PLAN):raise ValueError('plan identity mismatch')
    scores={};groups={'overall':rows}
    for field in ('symbol','asset_class','cohort'):
        for value in sorted({r[field] for r in rows}):groups[field+':'+value]=[r for r in rows if r[field]==value]
    for key,subset in groups.items():
        scores[key]={name:{arm:{bps:score(subset,name,arm,bps,10 if key.startswith('symbol:') else 20) for bps in BPS} for arm in ARMS[name]} for name in MODELS}
    comparisons={name:[paired(rows,name,c) for c in MODELS[3:]] for name in MODELS[:3]}
    cohort={}
    for co in sorted({r['cohort'] for r in rows}):
        cohort[co]={}
        for symbol in sorted({r['symbol'] for r in rows}):
            s=score([r for r in rows if r['cohort']==co and r['symbol']==symbol],'production_s3','trailing','24',10)
            cohort[co][symbol]={k:s[k] for k in ('closed','wins','mean_r')}
    overall={name:{k:scores['overall'][name][ARMS[name][0]]['24'][k] for k in ('closed','wins','win_rate','mean_r','pf')} for name in MODELS}
    gap_count=sum(any(gap(r,m) for m in MODELS) for r in rows)
    review=dict(trace='№18,316',plan_sha256=result['plan_sha256'],pr_merge_commit=None,
                total_jobs_completed=sum(r['status']=='complete' for r in rows),
                cohort_breakdown_scope='production_s3 / trailing / primary 24bp per side',cohort_breakdown=cohort,
                overall_summary=overall,data_gap_count=gap_count,
                report_file='doc/SEASONAL_PILOT_REPORT-18316-2026-11-07.md',validation_opened=False,financial_pass=False)
    years={}
    for name in MODELS[:3]:
        years[name]={year:score([r for r in rows if r['date'].startswith(year)],name,'trailing','24',10) for year in ('2024','2025')}
    stops={name:all(v['mean_r'] is not None and v['mean_r']<=0 for v in by_year.values()) for name,by_year in years.items()}
    arithmetic=[]
    for row in rows:
        for name in MODELS:
            m=model(row,name);sp=m.get('prepared_spread')
            marks=m.get('study',{}).get('marks',[]) if 'study' in m else [dict(reason=m.get('status'),cost=m.get('cost'),r=m.get('r'))]
            for mark in marks:
                c,r=mark.get('cost'),mark.get('r')
                if sp is not None and c is not None and r and mark['reason'] in ('cost_ge_r_session_veto','dispatch_cost_ge_r_session_veto'):
                    arithmetic.append(dict(key=row['key'],model=name,primary_cost=c,r=r,
                        lower_costs={b:sp+(c-sp)*float(b)/24 for b in BPS[1:]},
                        lower_cost_less_r={b:sp+(c-sp)*float(b)/24<r for b in BPS[1:]},simulated_entry=False))
    gates={}
    for name in MODELS[:3]:
        failures=[]
        for k,v in scores.items():
            if k.startswith(('symbol:','asset_class:')) and not v[name]['trailing']['24']['numerical_screen_pass']:
                failures.append(k)
        if not all(x['significance_pass'] for x in comparisons[name]):failures.append('matched_control_significance')
        gates[name]=dict(passes_development_screen=not failures,reasons=failures,validation_opened=False)
    return dict(scores=scores,matched_controls=comparisons,by_year=years,development_gates=gates,
                veto_cost_arithmetic_only=arithmetic,stop_tuning_due_to_nonpositive_both_years=stops,
                review_block=review,financial_pass=False,validation_opened=False)


def fmt(value):return 'N/A' if value is None else f'{value:.5f}'
def table(headers,rows):
    e=lambda x:html.escape(str(x))
    return '<div class="scroll"><table><thead><tr>'+''.join('<th>'+e(x)+'</th>' for x in headers)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+e(x)+'</td>' for x in r)+'</tr>' for r in rows)+'</tbody></table></div>'


def verify_sources(out,result,execution):
    es=events(out)
    if es!=result['receipts'] or result['provenance']!=json.loads((out/'freeze.json').read_text()):raise ValueError('receipt/provenance changed')
    if result['execution_sha256']!=sha(EXECUTION):raise ValueError('execution identity changed')
    starts=[e['key'] for e in es if e['state']=='started'];done=[e for e in es if e['state']=='finished']
    ends={e['key']:e for e in done}
    if len(starts)!=len(set(starts)) or len(ends)!=len(done) or set(starts)!=set(ends):raise ValueError('uncertain or duplicate receipt')
    expected={'job-'+r['key'] for r in result['rows'] if r['status']!='missing'}
    if {k for k in ends if k.startswith('job-')}!=expected:raise ValueError('receipt coverage missing')
    if any(e['execution_sha256']!=sha(EXECUTION) for e in es if e['state']=='started'):raise ValueError('receipt execution mismatch')
    for key,e in ends.items():
        if sha(out/(key+'.json'))!=e['file_sha256']:raise ValueError('raw source changed')
    for row in result['rows']:
        if row['status']!='missing' and ends['job-'+row['key']]['file_sha256']!=row['file_sha256']:raise ValueError('row source mismatch')


def report(out,root):
    result=json.loads((out/'replay-results.json').read_text());execution=load()
    verify_sources(out,result,execution)
    summary=summarize(result,execution)
    # Keep native marks/fills/flags and compact only repeated explanatory notes.
    for row in result['rows']:
        for m in row.get('replay',{}).get('result',{}).get('models',{}).values():m.get('study',{}).pop('notes',None)
    result['summary']=summary
    result['generated_at_sgt']=dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()
    target=root/'docs/seasonal-pilot-results-2024-2025.json'
    target.write_text(json.dumps(result,separators=(',',':'),allow_nan=False)+'\n')
    block=summary['review_block'];n=block['total_jobs_completed']
    headings=['Model','Exit','bp/side','Entries','Closed','Wins','Win rate','Mean R','PF(R)','Censored']
    rows=[]
    for name,arms in summary['scores']['overall'].items():
        for arm,costs in arms.items():
            for bps,s in costs.items():rows.append([name,arm,bps,s['entries'],s['closed'],s['wins'],fmt(s['win_rate']),fmt(s['mean_r']),fmt(s['pf']),s['censored_or_invalid']])
    md=f'''# Stage 1 multi-year seasonal pilot — review report

№18,316 §§1–8 · {result['generated_at_sgt']} · Codex (GPT-6), effort not exposed.

## §1. TLDR

**{n}/80 episode sources completed and replayed**; **{block['data_gap_count']}/80 episodes
carry a missing-data, sequence, preparation or censoring flag in at least one model**.
This is a fixed 20-date, four-symbol pilot across four seasonal cohort weeks.
It does not establish the interim 50% goal or the original 73% / 1:1.38 account goal.
No validation opened, no strategy promoted, no live account or production universe changed.
Two-year cohort labels are not proof of seasonality or evidence of a complete-year test.

## §2. Frozen method and corrections

The owner's source JSON and Markdown are preserved verbatim. Read the
[execution addendum](../docs/seasonal-execution-addendum-18315.md) for every
pre-outcome interpretation: DST-aware US500; 07:00 UTC oil window; 00:00 UTC BTC;
45-day nominal M30 view excluding protected 19 August–5 September 2025 data;
ORB-30/60 high/low >1.25 times the exact prior-20-session median; four-hour quotes.
The supplied full universe contains eight symbols, not seven. Stage 2 did not run.

S3 shares native band, mark, cost veto, dispatch recheck, targets and exits.
Its reconstructed opening spread/volume is not an authentic historical pre-open
receipt. Its actual session end is retained: an open trade at hour four is censored.
ORB exits close at hour four; controls at hour one hold to hour four. Account risk,
fees, historical contracts and broker execution are not reconstructed. These are
recorded-path scenarios, not production-account parity or qualified financial returns.

Primary admission/geometry remains 24 bp per side. 2 and 3.74 bp are **uncalibrated
sensitivities on identical admitted fills**, not newly admitted trades. Bid/ask
spread is counted once; zero extra commission remains an incomplete assumption.
Win rate uses all closed trades including breakevens. Failures, vetoes, missing data
and censored episodes stay in the 80-episode table and coverage denominator.
N/A is not zero return. PF(R) uses normalised R, not combined account dollars.

## §3. Overall results — each model and exit kept separate

| {' | '.join(headings)} |
| {' | '.join('---' for _ in headings)} |
'''
    for row in rows:md+='| '+' | '.join(map(str,row))+' |\n'
    md+='\n## §4. Every symbol and cohort: primary trailing scenarios\n\n| Cohort | Symbol | Model | Reason(s) | Entries | Closed | Wins | Mean R |\n|---|---|---|---|---:|---:|---:|---:|\n'
    detail=[]
    for cohort in sorted({r['cohort'] for r in result['rows']}):
        for symbol in sorted({r['symbol'] for r in result['rows']}):
            subset=[r for r in result['rows'] if r['cohort']==cohort and r['symbol']==symbol]
            for name in MODELS:
                s=score(subset,name,ARMS[name][0],'24',10)
                row=[cohort,symbol,name,json.dumps(s['reasons']),s['entries'],s['closed'],s['wins'],fmt(s['mean_r'])]
                detail.append(row);md+='| '+' | '.join(map(str,row))+' |\n'
    md+='''
## §5. Matched controls and uncertainty

Compare on the same symbol/date only when both candidate and control close;
unmatched cases remain reported. Entry times and exit rules differ, so this is a
strategy comparison rather than an isolated directional effect. The predeclared
one-sided weekly sign-flip test has minimum p=0.0625 with four weeks, so Stage 1
cannot satisfy p<=0.05 by itself. No parameter or statistical gate was loosened.

'''
    md+='```json\n'+json.dumps(summary['matched_controls'],indent=2)+'\n```\n'
    md+='''
## §6. Failure prevention and next decision

1. Keep source receipts and distinguish successful downloads from complete history,
   complete simulated positions, and verified real fills. No failed job is retried
   and no date replaced to improve performance. Same-millisecond flags are sequencing
   uncertainty, not proof that the instrument is illiquid.
2. Review each model's missing-band/range/quote reasons before proposing changes.
   A four-hour censored S3 position is not a losing close; a cost veto is not a trade.
3. Calibrate costs independently with representative paired broker fills/fees;
   lower-cost sensitivity gains cannot rescue primary admission or certify an edge.
4. Stop tuning a candidate whose completed primary scenarios have nonpositive means
   in both years under this plan. Missing or censored evidence is inconclusive.
   Any broader full-session collection or new hypothesis needs a new frozen budget.
5. No automatic Stage 2 or validation. Before any promotion require sufficient
   independent weeks/closes, per-symbol and class economics, matched-control evidence,
   intact coverage, cost stress and account/execution parity. Software tests are separate.

'''
    md+='Lower-cost arithmetic on rejected marks (not trades): `'+json.dumps(collections.Counter(b for v in summary['veto_cost_arithmetic_only'] for b,ok in v['lower_cost_less_r'].items() if ok))+'`. Full rejected-mark calculations are in JSON.\n'
    md+='Both-year stopping flags: `'+json.dumps(summary['stop_tuning_due_to_nonpositive_both_years'])+'`.\n'
    md+=f'''
## §7. Reproducibility, checks and publication

Plan SHA256: `{result['plan_sha256']}`.
Execution SHA256: `{result['execution_sha256']}`.
Collection/replay commit: `{result['provenance']['commit']}`.
Actual pre-read freeze: `{result['provenance']['frozen_at_utc']}`.
Binary SHA256: `{result['provenance']['binary_sha256']}`.

Raw data and append-only segment/episode receipts remain at `{out}` in this
workspace, not an archival service. Repository JSON includes all native outcomes,
source hashes, receipts, code hashes and per-symbol/class/cohort/cost scores;
raw quotes are not duplicated into Git. Independent raw reproduction requires
that retained corpus. Run `seasonal_pilot.py replay` then `seasonal_report.py`
against it with the frozen native `seasonal_replay` binary. Plan and report PRs
carry the required CI and merge receipts.

Focused checks cover protected-date segmentation, DST, M30 decoding, omitted
receipts, exact prior-session ranges, causal ORB preparation, same-entry sensitivities,
HTTP scope preservation and S3 regression compatibility.

## §8. Independent AI review and copy-paste block

Read this report, the addendum, native result JSON and frozen input plan. Challenge
coverage, historical contract and fee assumptions, the conditional entry denominators,
the four-hour S3 censoring, and paired-comparison selection. Do not pool models or
infer an edge from a small winning subset. Propose only the smallest evidence-backed
next experiment, preserving losses and failed gates.

The final chat block adds the actual report PR merge commit after publication.
Full results: [JSON](../docs/seasonal-pilot-results-2024-2025.json).
'''
    base=root/'doc/SEASONAL_PILOT_REPORT-18316-2026-11-07'
    base.with_suffix('.md').write_text(md)
    narrative='<h1>Stage 1 seasonal pilot</h1><p>№18,316 · '+html.escape(result['generated_at_sgt'])+'</p><p class="notice"><b>TLDR:</b> '+str(n)+'/80 sources replayed; '+str(block['data_gap_count'])+'/80 flagged episodes. No financial qualification or validation opening. Interim 50% goal remains unproven.</p><p>24bp/side is primary. Lower costs reprice identical fills. Production S3 can remain censored at the four-hour boundary. Read the method and limitations before interpreting these tables.</p>'
    page='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Seasonal pilot</title><style>body{font:16px/1.55 system-ui;color:#172738;max-width:1300px;margin:auto;padding:24px;background:#f7f9fc}.notice{background:#fff0c4;padding:18px}table{border-collapse:collapse;width:100%;font-size:14px}td,th{padding:9px;border-bottom:1px solid #cbd5e1;text-align:left}th{background:#e6edf5}.scroll{overflow:auto}pre{white-space:pre-wrap;overflow-wrap:anywhere}a{color:#1450a0}</style>'+narrative+'<h2>All cost and exit scenarios</h2>'+table(headings,rows)+'<h2>Every symbol and cohort</h2>'+table(['Cohort','Symbol','Model','Reasons','Entries','Closed','Wins','Mean R'],detail)+'<h2>Review files</h2><p><a href="'+base.name+'.md">Complete method, review instructions and failure-prevention gates</a> · <a href="../docs/seasonal-pilot-results-2024-2025.json">Native evidence and scores</a></p><p>Draft for quantitative review; no live promotion.</p></html>'
    base.with_suffix('.html').write_text(page)
    (out/'review-block.json').write_text(json.dumps(block,indent=2,allow_nan=False)+'\n')
    print(json.dumps(block,allow_nan=False))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[2]);args=p.parse_args()
    report(args.output,args.root)
