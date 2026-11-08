#!/usr/bin/env python3
"""Phase 5 measured-result report; no collection or execution capability."""
import collections
import math
import datetime as dt
from pathlib import Path
from phase5_evaluation import ROOT,PLAN,RESULT,WORK,CANDIDATE,CORE,CLASSES,read,save,sha
from phase4_forensics import gates
from unified_replay_report import table,fmt,pct,markdown_body
import unified_replay as u

REPORT=ROOT/'doc/REBUILD_PHASE5_INDEPENDENT_EVALUATION_REPORT-18364-2026-11-08.md'
VERIFY=ROOT/'docs/rebuild_phase5_verification_18364.json'
C=u.ARMS[2]

def anatomy(rows):
    counts=collections.Counter()
    for r in rows:
        a=r['native'].get('arms',{}).get(C,{});f=a.get('forced')
        if not f:continue
        if not f.get('metrics'):counts['unresolved']+=1;continue
        reason=f['outcome']['fills'][0]['reason'];entry=f['entry'];o=f['outcome']
        category='strict_target' if reason=='broker_take_profit' else 'forced_window_close' if reason=='session_end' else 'breakeven_stop' if o['trail_applied'] and abs(o['active_stop']-entry['fill'])<1e-7 else 'initial_stop'
        counts[category]+=1
        if category=='forced_window_close' and f['metrics']['net_by_bps']['24']>0:counts['profitable_forced_at24']+=1
    return dict(counts)

def render():
    r=read(RESULT);p=read(PLAN);v=read(VERIFY)
    if sha(RESULT)!=v['result_sha256'] or sha(PLAN)!=r['plan_sha256']:raise ValueError('report source mismatch')
    cov=r['coverage'];rows=r['rows'];summary=r['summary'];allc=summary['all'][C]
    primary=allc['forced']['24'];low=allc['forced']['3.74'];counts=anatomy(rows)
    gross=primary['mean_gross_r'];drag=gross-primary['mean_net_r'] if gross is not None else None
    models=table(['Arm','Mode','Cost bp/side','Entries','Priced closes','Wins','Losses','Flat','Unresolved','Win rate','Mean net R','PF'],[
        [a,m,c,*[summary['all'][a][m][c][f] for f in ['entries','priced_closed','wins','losses','net_flat','unresolved']],pct(summary['all'][a][m][c]['net_win_rate']),summary['all'][a][m][c]['mean_net_r'],summary['all'][a][m][c]['profit_factor']]
        for a in u.ARMS for m in u.MODES for c in ['24','3.74','2']])
    class_rows=[];pilot=[]
    for c in sorted(set(CLASSES.values())):
        metric=summary[c][C]['forced']['24'];bm=summary[c][u.ARMS[1]]['forced']['24'];eligible=[x for x in rows if x['asset_class']==c]
        priced=[x for x in eligible if (x['native'].get('arms',{}).get(C,{}).get('forced') or {}).get('metrics')]
        weeks=len({x['cohort'] for x in priced});unavailable_peer=any(x['peer_context']['status']!='complete' for x in eligible)
        def opportunity(arm):
            vals=[]
            for x in eligible:
                f=x['native'].get('arms',{}).get(arm,{}).get('forced')
                if f and not f.get('metrics'):return None
                vals.append(f['metrics']['net_by_bps']['24'] if f else 0.)
            return sum(vals)/len(vals) if vals else None
        cv,bv=opportunity(C),opportunity(u.ARMS[1]);delta=cv-bv if cv is not None and bv is not None else None
        reasons=[]
        if metric['priced_closed']<20:reasons.append('fewer_than20_closes')
        if weeks<3:reasons.append('fewer_than3_weeks')
        if metric['net_win_rate'] is None or metric['net_win_rate']<.5:reasons.append('below50pct_or_unavailable')
        if metric['mean_net_r'] is None or metric['mean_net_r']<=0:reasons.append('nonpositive_or_unavailable_stress_expectancy')
        if delta is None or delta<=0:reasons.append('no_positive_C_minus_B_opportunity_return')
        if unavailable_peer:reasons.append('peer_data_abstention_confounds_comparison')
        pilot.append({'asset_class':c,'priced_closes':metric['priced_closed'],'weeks':weeks,'C_minus_B_R_per_eligible_opportunity':delta,'screen_passed':not reasons,'reasons':reasons,'financial_pass':False})
        class_rows.append([c,len(eligible),metric['priced_closed'],metric['wins'],pct(metric['net_win_rate']),metric['mean_net_r'],weeks,delta,'insufficient/fail' if reasons else 'pilot screen only'])
    by_class=table(['Class','Eligible episodes','Closes','Wins','Win rate at24bp','Mean net R','Weeks with closes','C minus B R/opportunity','Disposition'],class_rows)
    by_symbol=table(['Symbol','Eligible','Closes','Wins at24bp','Win rate at24bp','Mean R at24bp','Wins at3.74bp','Mean R at3.74bp'],[
        [s,summary[s][C]['forced']['24']['episodes'],*[summary[s][C]['forced']['24'][f] for f in ['priced_closed','wins']],pct(summary[s][C]['forced']['24']['net_win_rate']),summary[s][C]['forced']['24']['mean_net_r'],summary[s][C]['forced']['3.74']['wins'],summary[s][C]['forced']['3.74']['mean_net_r']] for s in CORE])
    by_week=table(['Week','Eligible','Closes','Wins','Mean R at24bp','Mean R at3.74bp'],[
        [w,summary[w][C]['forced']['24']['episodes'],summary[w][C]['forced']['24']['priced_closed'],summary[w][C]['forced']['24']['wins'],summary[w][C]['forced']['24']['mean_net_r'],summary[w][C]['forced']['3.74']['mean_net_r']] for w in p['dates']])
    controls=table(['Auxiliary control','Cost bp/side','Eligible','Entries','Closes','Wins','Unresolved','Mean R','PF'],[
        [a,c,*[r['controls']['all'][a][c][f] for f in ['episodes','entries','priced_closed','wins','unresolved','mean_net_r','profit_factor']]] for a in ['always_long','own_momentum'] for c in ['24','3.74','2']])
    ledger=[]
    for row in rows:
        a=row['native'].get('arms',{}).get(C,{});f=a.get('forced')
        if not f:continue
        i=f['entry'];o=f.get('outcome',{});fills=o.get('fills',[]);m=f.get('metrics') or {}
        ledger.append([row['key'],i['side'],i['entered_at'],fills[0]['at'] if fills else None,fills[0]['reason'] if fills else f['status'],i['initial_r'],m.get('net_by_bps',{}).get('24'),m.get('net_by_bps',{}).get('3.74'),a['strict']['status'],a['continuation']['status']])
    ledger_table=table(['Episode','Side','Entry UTC','Exit UTC','Forced-path exit','Initial R price','NetR24','NetR3.74','Strict status','72h M30 status'],ledger)
    register=[];excluded={x['key']:x for x in cov['excluded']};included={x['key']:x for x in rows}
    for j in p['jobs']:
        x=included.get(j['key'],excluded.get(j['key']));n=x.get('native',{});c=x['coverage']
        register.append([j['key'],j['cohort'],x['source_status'],c['required_prep_slots'],c['missing_prep_slots'],'; '.join(c['reasons']) or n.get('macro',{}).get('reason',n.get('status')),n.get('arms',{}).get(C,{}).get('status','not_entered'),x['source_sha256']])
    coverage_table=table(['Episode','Cohort','Tick source','Required M30','Missing M30','Eligibility / macro decision','Candidate status','Tick receipt SHA256'],register)
    reasons=collections.Counter(reason for e in cov['excluded'] for reason in e['coverage']['reasons'])
    missing=table(['Exclusion reason (overlapping counts)','Episodes'],reasons.items())
    paired_rows=[]
    for k,val in r['paired_macro']['all']['24']['categories'].items():paired_rows.append([k,val['entries'],val['priced_closed'],val['wins'],val['losses'],val['mean_net_r']])
    paired_table=table(['B path disposition under C','Entries','Priced closes','Wins','Losses','B mean net R24'],paired_rows)
    reg=gates();reg[1]['reason']='159 legacy and35 Phase2 outputs reproduced exactly before collection; original session defaults retained.'
    reg[2]['reason']='Frozen bounded collection separated from network-denied native evaluation; failures and all160 slots retained.'
    reg[3]['reason']=f"{len(rows)} sessions qualified under coded fixed windows; {len(cov['excluded'])} excluded before outcomes. Missing/ambiguous continuations are censored."
    reg[8]['reason']='One owner-amended candidate and untouched retained evaluation windows frozen before collection. Preparation reused; retrospective selection and unavailable historical references limit independence claims.'
    reg[9]['reason']='See fixed per-class screens; selected point estimates do not establish calibrated financial qualification.'
    gates_table=table(['Gate','Status','Evidence / gap'],[[g['gate'],g['status'],g['reason']] for g in reg])
    goal_gap=max(0,20-primary['priced_closed'])
    n=primary['priced_closed'];rate=primary['net_win_rate'];interval='unavailable'
    if n:
        z=1.95996398454;den=1+z*z/n;centre=(rate+z*z/(2*n))/den;half=z*math.sqrt(rate*(1-rate)/n+z*z/(4*n*n))/den
        interval=pct(centre-half)+' to '+pct(centre+half)

    text=f'''# Phase 5 — Fixed-window macro pullback: measured historical evaluation

№18,364 §§1–11 · {v['verified_at_sgt']} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**This tests the owner-approved changed candidate `{CANDIDATE}`, not the frozen Phase 4 candidate.** The requested fixed UTC windows alter the opening reference, historical sigma and causal trend context. There are no live rule changes.

**Primary four-hour result at 24 bp/side:** {primary['wins']}/{primary['priced_closed']} wins ({pct(primary['net_win_rate'])}), mean net R **{fmt(primary['mean_net_r'])}**, profit factor **{fmt(primary['profit_factor'])}**. At the uncalibrated 3.74 bp sensitivity, the same chosen entries produce {low['wins']}/{low['priced_closed']} wins and mean {fmt(low['mean_net_r'])}R. Before the extra cost scenario, mean executable-price gross R is {fmt(gross)}; the24bp scenario subtracts an average {fmt(drag)}R. Recorded spread is included once; fees, swap, conversion and true round-trip slippage remain unverified.

**Coverage:** {cov['tick_sources_completed']}/160 tick sources completed; **{len(rows)} sessions were eligible for native evaluation**, with {len(cov['excluded'])} excluded before outcomes. {r['new_m30_requests']} new M30 requests, {r['cached_m30_reused']} cached slices reused, {p['limits']['blocked_m30_slices']} spent slices left blocked. A completed download is not a qualified trade.

Both candidate closes carry quote-gap flags. The reported returns are replay scenarios; native execution-qualified returns remain unavailable.

**Goal progress:** {primary['priced_closed']} pooled priced candidate closes; {goal_gap} more would be needed even to reach a pooled 20-close count. This is not the per-strategy/account milestone. No account has qualified for 73% wins / 1:1.38 risk:reward; the gross target remains 1.38R, and realised net payoff may differ. Per-class 50% screens are below. No automatic extra sampling is authorised by a shortfall.

**Uncertainty:** the descriptive 95% Wilson interval for the pooled primary win rate is {interval}. This treats closes as independent Bernoulli observations; correlated peers and weeks weaken that assumption. It is not an account-level confidence guarantee.

**Decision:** no financial promotion. Keep the candidate and this batch immutable. Coverage/calendar, cost calibration and class-level evidence must be assessed before any separately registered next experiment. All failures and censored paths remain in the report.

## §1. What was frozen, and what changed

The owner explicitly selected fixed UTC windows after the mismatch with Phase 4 was identified. Energy uses13:00–17:00, indices14:30–18:30, metals07:00–11:00 and crypto00:00–04:00. All are UTC; indices intentionally do not move with New York daylight saving in this changed candidate.

Each historical sigma reference and 5/20-session trend uses the same four-hour clock window: weekdays for noncrypto and every day for crypto. The existing14-session sigma,48bp floor, durable full-cost veto, later fresh-quote dispatch,50% retracement, structural risk calculation, gross1.38R target and next-quote entry-stop amendment are retained. Last entry decision mark is3h30m; a retest must occur before4h. This is a disclosed change in session/context definition, not an exact replication of the7/7 Phase4 sample.

Primary admission remains3.74bp/side and primary outcome pricing24bp/side; changing admission and repricing fixed entries are different operations. The structural risk includes the admission reserve, so a cheap admission assumption affects both selection and geometry. No parameter is retuned after this batch.

## §2. Provenance and independence

Plan SHA256: `{r['plan_sha256']}`. Code freeze commit: `{r['code_freeze']['commit']}`. Results SHA256: `{sha(RESULT)}`. Broker reads use demo account49236051 and only the existing read-only message allowlist. No orders, subscriptions, live-account reads, model API calls or Telegram messages are required.

The20 owner-selected dates have no overlap with retained development evaluation intervals. Some45-day preparation overlaps earlier requests and is reused by verified receipt. This is frozen retrospective historical evaluation, not chronological forward validation. The530 unavailable Phase1 references are mixed source/code/report revisions; none is an unavailable collection-ledger reference. They must not be described as530 proven missing tick receipts, nor does their absence prove universal broker/chat independence.

Historical broker trading calendars and contract terms remain unverified. Fixed weekday slots are a coded scenario. Missing bars may indicate a holiday, maintenance, unavailable history or a defect; they are not fabricated or removed after seeing outcomes. Failed or uncertain requests remain spent. The protected19August–5September2025 interval was excluded from every dependency.

## §3. Eligibility and data limitations

{missing}

All eligible/excluded decisions were recorded before any new candidate outcome. Own preparation must be complete across45 calendar days, with a valid opening M30 bar and fresh recorded quote boundaries. Planned peer failures/missing context cause macro abstention, never a convenient peer-absent pass. Macro-data abstentions are separated from directional vetoes below.

The two failed tick jobs were NAS100 on 12 and 14 August 2024. Both exceeded the frozen 200,000-row budget; neither was retried. They also lacked required preparation, so exclusion-reason totals overlap. All 239 newly requested M30 segments returned complete receipts, but existing blocked slices and missing expected calendar slots still prevent full preparation coverage. A successful request does not imply complete historical coverage.

Continuous executable liquidity is not established by a complete downloaded interval: separate bid/ask streams can contain gaps, stale updates and unknown same-millisecond ordering. Raw quality flags are retained in the JSON. Current contract snapshots cannot certify historical minimum lots, precision changes or dollar P&L. Profit factors aggregate R-normalised returns, not statement dollars; different symbols' price units are never summed as portfolio cash.

## §4. Candidate and matched A/B controls

{models}

`strict` counts only bracket resolution inside the tick window; open paths remain unresolved. `forced` is the primary score and uses a fresh executable boundary quote within5s. `continuation` resumes only unresolved strict paths on recorded M30 bars, capped at72 hours from entry. M30 is a bid-price proxy; short exits use the frozen entry spread proxy. Intrabar ambiguity, missing bars, overnight/weekend gaps and partial deadline bars remain unresolved. Continuation returns never replace primary four-hour results.

Exit anatomy of the primary candidate: `{counts}`. Stop amendments are not automatically breakeven exits, and breakeven at entry can still lose after costs.

## §5. Per-class, symbol and week results

{by_class}

{by_symbol}

{by_week}

The frozen per-class screen requires at least20 closed trades across three weeks, win rate≥50%, positive stress-cost expectancy and positive C-minus-B return per eligible opportunity. Unresolved comparative paths or peer-data abstentions prevent a clean incremental-filter claim. Small weekly samples and dependence between peers limit inference. A pooled score cannot pass a class or account.

## §6. Requested always-long / own-momentum controls

{controls}

Both auxiliary controls use the same qualified sessions, first-hour range and next fresh quote, then a four-hour close. They retain their original24bp admission and have no candidate bracket exits; therefore they are descriptive benchmarks, not perfectly matched entry/exit ablations. A versus B versus C is the stronger controlled comparison for entry and macro-filter effects. No control result is selected as a replacement candidate.

{paired_table}

## §7. Every candidate trade

{ledger_table}

The JSON retains full entry/exit state, context, preparation quality, source hashes and native receipt identities. Positive boundary drift is not relabelled as a target hit. Profits from M30 approximations are not represented as broker executions.

The BTC long on 4 March 2024 entered at 63,350 with an initial risk distance of 263.39 (41.58 bp). It reached the gross target, but its stress-priced result was only +0.222209R. The ETH long on 7 March entered at 3,830.20 with an initial risk distance of 14.87 (38.82 bp), then crossed its stop; its stress-priced loss was −2.235992R. The 48 bp floor applies to the opening impulse, not the smaller post-retracement structural risk. This explains the large stress-cost burden without changing the frozen rules after observing results.

Both candidate paths contain `gap_before_quote` flags. Native qualified gross/net results remain unavailable; the reported numbers are explicitly scenario calculations from the recorded quotes. Both exits occur inside four hours, so the 72-hour continuation adds no candidate resolution. These two trades cannot establish a repeatable edge, even before unverified fees, financing and execution effects.

## §8. RFC gates and failure prevention

{gates_table}

Exactly2/12 gates retain bounded engineering passes. CI verifies software behaviour; it cannot establish an economic edge. Prevent recurrence by keeping separate: frozen identity and rules; source/calendar eligibility; native replay and arithmetic; costs; independent per-class evidence; authorised shadow/live operations. No future repair may overwrite this result or replay spent broker requests as fresh evidence.

Before any next batch, address the actual excluded dependencies from the register, establish dated calendars/contracts and capture matched account-scoped round-trip costs. If the fully specified candidate fails the registered class screen, retain that failure and formulate a new economic hypothesis rather than repeatedly tuning these dates. If evidence is insufficient, publish that fact without extending until a favourable sample appears.

## §9. Verification and reproduction

-159 legacy and35 Phase2 replay outputs reproduced exactly before collection.
- Two complete offline runs produced byte-identical result JSON; all 399 collection receipt hashes reconciled.
- Focused fixed-window and collection/accounting tests; full research tests and CI recorded in verification and PR.
- Native cost arithmetic checks: {r['verification']['native_arithmetic_checks']}.
- Coverage freeze SHA256: `{r['verification']['coverage_before_outcomes_sha256']}`.
- Protected validation unopened; live rules unchanged; evaluation network guard enabled.

The original sequence was `prepare`, commit the plan/code, `collect`, then `evaluate`. For offline reproduction, use a separate checkout of the frozen commit `{r['code_freeze']['commit']}` with the retained cache and exact frozen binary, then run `python3 ops/research/phase5_evaluation.py evaluate`. The runner deliberately rejects a different commit or source hash. Generate the report using `python3 ops/research/phase5_report.py` from this publication commit. No new collection is needed for reproduction.

Collection is resumable only with matching immutable receipts and code freeze; uncertain attempts fail closed. Reproduction requires retained raw files and the frozen binary, not merely the GitHub JSON. Do not execute collection again to manufacture a new independent sample. Before any future collection, union this Phase 5 plan and its receipt register with the Phase 1/4 inventories; the frozen Phase 4 registry alone is now incomplete.

## §10. Complete160-episode register

{coverage_table}

## §11. Instructions for independent AI review

Read this report with the Phase5 plan, results and verification JSON under `docs/`, and Phase4's frozen original. Check the owner-authorised session change, complete160 denominator, cached/spent interval handling, strict versus forced versus M30 accounting, per-class comparisons and cost sensitivity. Challenge any purported edge against the measured evidence. Preserve original goals, serial/paragraph numbering, SGT timestamps, proven fixes and the “Time is Money” invariant.
'''
    for a,b in [('uses13','uses 13'),('indices14','indices 14'),('metals07','metals 07'),('crypto00','crypto 00'),('existing14','existing 14'),(',48bp',', 48 bp'),(',50%',', 50%'),('gross1.38','gross 1.38'),('is3h30m','is 3h30m'),('before4h','before 4h'),('remains3.74','remains 3.74'),('pricing24','pricing 24'),('account49236051','account 49236051'),('The20','The 20'),('Some45','Some 45'),('The530','The 530'),('as530','as 530'),('protected19August','protected 19 August'),('September2025','September 2025'),('across45','across 45'),('within5s','within 5s'),('at72','at 72'),('least20','least 20'),('original24','original 24'),('Exactly2/12','Exactly 2/12'),('-159','- 159'),('and35','and 35'),('Phase2','Phase 2'),('Phase4','Phase 4'),('Phase5','Phase 5'),('Phase1','Phase 1'),('Complete160','Complete 160'),('complete160','complete 160')]:text=text.replace(a,b)
    for a,b in [('the24bp','the 24 bp'),('the7/7','the 7/7'),('5September','5 September'),('all160','all 160'),('bounds;3.74bp','bounds; 3.74 bp')]:text=text.replace(a,b)
    REPORT.write_text(text)
    body,toc=markdown_body(text)
    css='body{font:16px/1.6 system-ui,sans-serif;color:#183247;background:#f5f8fb;margin:0}main{max-width:1400px;margin:auto;padding:28px}p{max-width:1120px;overflow-wrap:anywhere}h1,h2{line-height:1.25}h2{margin-top:38px}table{border-collapse:collapse;background:white;font-size:13px}td,th{padding:10px;border:1px solid #c8d7e2;vertical-align:top;text-align:left}th{background:#193f5c;color:white;position:sticky;top:0}tr:nth-child(even){background:#edf3f7}.table-scroll{overflow:auto;max-height:600px;margin:20px 0}pre{overflow:auto;background:#193f5c;color:white;padding:18px}code{overflow-wrap:anywhere}nav{display:flex;gap:12px;flex-wrap:wrap}a{color:#075eaa}@media(max-width:600px){main{padding:16px}h1{font-size:26px}.table-scroll{max-height:450px}}'
    REPORT.with_suffix('.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Phase 5 №18,364</title><style>'+css+'</style><main><details><summary>Contents</summary><nav>'+''.join(f'<a href="#{a}">{t}</a>' for a,t in toc)+'</nav></details>'+body+'</main></html>')
    save(WORK/'class-screen.json',pilot);print(REPORT)

if __name__=='__main__':render()
