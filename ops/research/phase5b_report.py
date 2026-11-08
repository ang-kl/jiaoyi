#!/usr/bin/env python3
"""Readable Phase5B report, computed from the frozen offline audit artifact."""
import collections
from pathlib import Path
from phase5b_audit import ROOT,RESULT,SCOPE,WORK,SPEC,read,sha,save,CORE
from phase4_forensics import gates
from unified_replay_report import table,fmt,pct,markdown_body
import unified_replay as u

REPORT=ROOT/'doc/REBUILD_PHASE5B_EXACT_CANDIDATE_AUDIT_REPORT-18366-2026-11-08.md'
VERIFY=ROOT/'docs/rebuild_phase5b_verification_18366.json'
C=u.ARMS[2]

def row_metrics(s):
    lo=s['forced']['3.74'];hi=s['forced']['24']
    return [lo['episodes'],lo['priced_closed'],lo['wins'],lo['losses'],lo['unresolved'],pct(lo['net_win_rate']),lo['mean_net_r'],lo['profit_factor'],hi['wins'],hi['mean_net_r'],hi['profit_factor']]

def render():
    r=read(RESULT);v=read(VERIFY);scope=read(SCOPE);cov=r['coverage'];audit=r['phase5_exclusion_audit'];reg=cov['register']
    if sha(RESULT)!=v['result_sha256']:raise ValueError('result/report mismatch')
    exact=r['summary']['new_batch']['tier1_r48']['all'][C];diag=r['summary']['new_batch']['tier2_r28']['all'][C]
    a=exact['forced']['3.74'];b=exact['forced']['24'];d=diag['forced']['3.74'];stress=diag['forced']['24']
    primary_sentence=('It generated no entries: both eligible sessions had no admitted impulse. There is therefore no new realised win rate or mean return for the exact candidate.' if not a['entries'] else f"It produced {a['priced_closed']} priced closes, {a['wins']} wins at 3.74 bp/side and mean {fmt(a['mean_net_r'])}R; at 24 bp/side, mean return is {fmt(b['mean_net_r'])}R.")
    headers=['Dataset','Track','Episodes','Closes','Wins3.74','Losses3.74','Unresolved','Win%3.74','MeanR3.74','PF3.74','Wins24','MeanR24','PF24']
    outcomes=table(headers,[[group,track,*row_metrics(r['summary'][group][track]['all'][C])] for group in ['new_batch','prior_manifest','combined'] for track in ['tier1_r48','tier2_r48','tier2_r28']])
    per_symbol=table(['Track','Symbol',*headers[2:]],[[track,s,*row_metrics(r['summary']['new_batch'][track][s][C])] for track in ['tier1_r48','tier2_r28'] for s in CORE])
    modes=table(['Dataset','Track','Mode','Cost','Closes','Wins','Losses','Unresolved','MeanR'],[[group,track,mode,cost,*[r['summary'][group][track]['all'][C][mode][cost][k] for k in ['priced_closed','wins','losses','unresolved','mean_net_r']]] for group in ['new_batch','combined'] for track in ['tier1_r48','tier2_r28'] for mode in u.MODES for cost in ['3.74','24']])
    by_class=table(['Track','Class',*headers[2:]],[[track,s,*row_metrics(r['summary']['new_batch'][track][s][C])] for track in ['tier1_r48','tier2_r28'] for s in ['energy','equity_index','metal','crypto']])
    windows=table(['Symbol','Completed sources','Cover native4h','Wrong window','Failed source'],[[s,sum(x for k,x in vals.items() if k!='failed_source'),vals.get('covers_native_window',0),vals.get('missing_native_window',0),vals.get('failed_source',0)] for s,vals in r['window_alignment_by_symbol'].items()])
    reasons=table(['PR195 completed-source exclusion','Count'],audit['exclusive_completed_source_reasons'].items())
    denom=table(['Dataset','Exact Tier1','Tier2 diagnostic'],[[k,x['tier1'],x['tier2']] for k,x in r['eligible_counts'].items()])
    dates=table(['Frozen Phase4 week','Dates'],[[w['dates'][0],', '.join(w['dates'])] for w in read(SPEC)['independent_evaluation_plan']['weeks']])
    ledger=[]
    for row in r['rows']:
        f=row['native'].get('arms',{}).get(C,{}).get('forced')
        if not f:continue
        i=f['entry'];o=f.get('outcome',{});m=f.get('metrics') or {};exits=o.get('fills',[])
        ledger.append([row['origin'],row['tier'],row['threshold_bps'],row['symbol'],row['date'],row['episode_id'],i['side'],i['entered_at'],exits[0]['at'] if exits else None,exits[0]['reason'] if exits else f['status'],m.get('net_by_bps',{}).get('3.74'),m.get('net_by_bps',{}).get('24'),'; '.join(o.get('data_issues',[]))])
    trades=table(['Dataset','Tier','Floor','Symbol','Date','Episode','Side','Entry UTC','Exit UTC','Exit reason','NetR3.74','NetR24','Quote issues'],ledger)
    reg_by_id={x['episode_id']:x for x in reg}
    register_rows=[]
    for old in audit['rows']:
        q=reg_by_id[old['episode_id']];tc=q.get('tier1_coverage',{});t2=q.get('tier2_coverage',{})
        register_rows.append([old['key'],old['state'],'; '.join(old['reasons']),q['window'].get('native_from'),q['window']['covers'],tc.get('missing_prep_slots'),q.get('tier','excluded'),'; '.join(tc.get('reasons',[]))+' / '+'; '.join(t2.get('reasons',[])),old['receipt_sha256']])
    register=table(['Episode','PR195 state','PR195 reasons','Native open UTC','Window covered','Native M30 missing','Phase5B track','Native eligibility reasons','Tick receipt SHA256'],register_rows)
    native_reasons=collections.Counter(x for row in reg if row['origin']=='phase5_new_batch' for x in row.get('tier1_coverage',{}).get('reasons',[]))
    partial_reasons=collections.Counter(x for row in reg if row['origin']=='phase5_new_batch' for x in row.get('tier2_coverage',{}).get('reasons',[]))
    exclusions=table(['Track','Overlapping reason','Count'],[['Tier1',k,n] for k,n in native_reasons.items()]+[['Tier2 fallback',k,n] for k,n in partial_reasons.items()])
    controls=table(['Dataset','Track','Arm','Closes','Wins3.74','MeanR3.74','Wins24','MeanR24'],[[group,track,arm,*[r['summary'][group][track]['all'][arm]['forced']['3.74'][k] for k in ['priced_closed','wins','mean_net_r']],*[r['summary'][group][track]['all'][arm]['forced']['24'][k] for k in ['wins','mean_net_r']]] for group in ['new_batch','combined'] for track in ['tier1_r48','tier2_r28'] for arm in u.ARMS])
    g=gates();g[1]['reason']='All202 original Phase3 Tier1-r48 / Tier2-r28 paths at3.74bp reproduced exactly using Rust built from the literal Phase4 commit.'
    g[2]['reason']='Zero broker calls; immutable source and eligibility freeze; deterministic native receipts and cost arithmetic.'
    g[3]['reason']='New sources:2 exact Tier1,72 Tier2 diagnostics;68 completed sources have wrong native windows. Full denominators retained.'
    g[8]['reason']='Exact candidate identity restored for supported cached windows; original independent date matrix remains unexecuted; Phase5 dates already exposed.'
    g[9]['reason']='No independently qualified class/account result. Reused development results cannot qualify the candidate.'
    gate_table=table(['Gate','Status','Evidence / remaining work'],[[x['gate'],x['status'],x['reason']] for x in g])
    text=f'''# Phase 5B — Exact Phase-4 candidate and exclusion audit

№18,366 §§1–11 · {v['verified_at_sgt']} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**The exact frozen candidate was replayed on {a['episodes']} eligible new-batch episodes.** {primary_sentence} All reported trade returns are recorded-quote scenarios with unverified execution costs.

**The original independent evaluation remains incomplete.** None of Phase5's 20 dates matches the frozen Phase4 date matrix, and {r['frozen_independent_plan_tick_sources_covered']}/160 original planned windows have covering cached ticks. Replaying the original rules on a supported subset does not complete its original test plan.

**The 96 exclusions in PR #195 reconcile:** 86 completed sources lacked fixed-window M30 preparation; 10 lacked a fresh tick boundary. Two failed Nasdaq sources are outside those 96. All 160 scheduled slots, 158 completed tick receipts, 239 new M30 receipts and 75 reused slices are retained in the audit.

**Tier2 is a different diagnostic model.** The new72-episode Tier2 track at28bp produced {d['priced_closed']} closes, {d['wins']} wins, mean {fmt(d['mean_net_r'])}R at3.74bp; repriced at24bp it has {stress['wins']} wins and mean {fmt(stress['mean_net_r'])}R. It uses opening-range/partial context and cannot be relabelled `s3_macro_pullback_v1`.

**Goal progress:** {a['priced_closed']}/20 pooled exact-candidate closes in this new batch, with no per-class/account qualification. The 73% / 1:1.38 latest-20 goal remains unchanged. RFC engineering gates remain 2/12; financial pass is false. No broker calls or live-rule changes occurred.

## §1. Exact specification and executable identity

The candidate is loaded unchanged from Phase4 commit `{r['frozen_phase4']['commit']}`. Rust adapters were compiled from a separate checkout at that commit; source hashes are compared against Git objects and the frozen entry/exit hashes. All202 relevant Phase3 reference paths were then reproduced exactly. The Phase5 fixed-window adapter is not used.

Frozen admission is 3.74 bp/side, the Tier1 impulse threshold is 48 bp, entry is a later executable 50% retracement, structural risk includes the original cost reserve, entry-price stop follows +1R, and gross target is +1.38R. The primary horizon is native open +4h. The 5/20 trend context uses complete native sessions across 45 calendar days. Recorded spread is included once. Fees, swap, conversion, historical contract terms and round-trip slippage remain unverified.

The “macro” input is causal historical price drift, not fundamental/news data or observed institutional flow. The exact frozen economic hypothesis remains a hypothesis.

## §2. Dates and windows: two distinct mismatches

{dates}

These dates are preserved verbatim. The Phase5 April/October 2023 and March/August 2024 labels are a diagnostic dataset requested for this audit; they do not replace the frozen independent dates. Their price outcomes have already been examined in Phase5, which limits independence.

Native oil sessions open at 07:00 UTC, versus the cached 13:00–17:00 oil windows. US indices open at 09:30 New York, translating to 13:30 UTC during daylight saving and 14:30 UTC during standard time. Phase5 used 14:30 UTC throughout. Metals 07:00–11:00 and crypto 00:00–04:00 tick windows align. Full historical M30 context still differs: native metal/oil days span 07:00–20:00, US cash sessions 09:30–16:00 New York, and crypto days 00:00–24:00 UTC, rather than four-hour context.

{windows}

Of 158 completed sources, 90 cover the native four-hour label window and 68 do not: 40 oil and 28 index sources. The two failed Nasdaq jobs are counted separately. Missing opening hours are not reconstructed from M30 bars or shifted to a later session.

## §3. Exact reconciliation of 96 excluded completed sources

{reasons}

The arithmetic is 160 scheduled = 62 previously evaluated + 96 completed but excluded + 2 failed. The old report's 98 total exclusions include both failures; 96 refers only to completed-source exclusions. Both Nasdaq failures hit the frozen 200,000-row budget and remain failed/spent.

All399 Phase5 collection receipt hashes were reconciled. The75 reused M30 slices refer to {audit['reused_unique_files']} distinct verified source files. A successful M30 request can still contain fewer bars than the coded calendar requires; no missing bar is invented and no historical holiday is assumed from absence alone.

Native-rule exclusions below can overlap. They are not interchangeable with PR #195's mutually exclusive 86 + 10 reasons.

{exclusions}

## §4. Eligibility and evidence tiers

The combined catalogue contains {cov['combined_manifest_episode_definitions']} episode definitions and {cov['m30_source_files']} M30 source files. Of these, {cov['core_episode_definitions']} episode definitions concern the frozen eight symbols; {cov['outside_core_definitions']} concern other symbols and are outside this candidate's scope. The register preserves failed, protected, repeated and otherwise ineligible core definitions alongside the selected sessions.

{denom}

Tier1 is the exact frozen sigma candidate. Tier2 is used only for otherwise excluded clean native-window sources, with the original Phase3 opening-range and endpoint-context rules; its48bp and28bp variants remain separate diagnostics. Missing planned peer data causes abstention rather than an absent-peer pass. Protected receipts and inherited repeat intervals remain outside clean tiers; duplicate account/symbol/session identities are not double counted.

The exact new Tier1 episodes are XAUUSD and XAGUSD on 20 October 2023. Their inclusion follows the frozen full-session coverage rules, not their outcomes. Prior-manifest counts of 35 Tier1 and 167 Tier2 remain unchanged. Combined totals are 37 Tier1 and 239 Tier2 episodes; they do not constitute a new independent sample.

## §5. Measured candidate and diagnostic results

{outcomes}

{by_class}

{per_symbol}

MeanR and profit factor aggregate normalised research outcomes, not account dollars. A gross1.38R target is not a realised1.38 net reward-to-risk ratio. Flat/no-entry/unresolved paths remain distinct. Empty samples have unavailable means and rates, never fabricated zeros.

## §6. Strict versus forced versus continuation

{modes}

Forced native4h close is the frozen primary score. Strict counts only bracket-resolved paths; unresolved paths remain visible.72h M30 continuation is a separate non-confirmatory diagnostic already available in the engine and is not selected as the winning horizon. Recorded M30 ambiguity, gaps and executable-price proxies retain their limitations.

## §7. Controls and every candidate path

{controls}

{trades}

The JSON retains A/B/C native outputs, exact source/input/receipt hashes, paired macro decisions and quote-quality flags. A positive replay does not erase `qualified_gross_r`/`net_r` unavailability in the native engine. Prior wins remain reused development evidence. Directional vetoes and data-driven abstention are separate in the paired tables in JSON.

In the new 28 bp diagnostic, all six retained trades already have negative executable-price gross returns: mean −0.843949R before the extra slippage scenario. Five lose around their original stop; the BTC long on 4 March exits close to its amended entry-price stop and becomes a larger net loss after costs. Every path carries `gap_before_quote`. Thus cost calibration is necessary, but reducing the assumed charge alone would not repair this observed entry/exit performance.

The macro direction filter vetoes nine unfiltered pullback paths: three winners and six losers at 3.74 bp. Another two losing paths are blocked by unavailable own context. The six retained paths all lose. Avoiding some losing opportunities is not evidence that the retained entries have positive expectancy; missing-data abstentions must not be credited as macro foresight.

## §8. What would be needed next

1. Preserve the original dates, native windows and full-session context as a single immutable specification generated from the frozen native calendar.
2. Reconcile historical holidays, maintenance and actual missing preparation before collecting again. Full coded weekday slots are not authoritative historical broker calendars.
3. Produce a metadata-only gap/overlap inventory for the original160 windows. It must include Phase5's new M30 requests: parts of the original planned dates now lie within previously read preparation. Do not call those prices untouched or refetch spent intervals.
4. Any later authorised collection must acquire missing native tick windows and necessary history under an explicit revised independence assessment. This phase makes no such requests.
5. Calibrate matched account-scoped round-trip costs before interpreting3.74bp sensitivity as executable economics. Then assess the predeclared class screens and per-account goals without tuning these already inspected dates.

Completing unsupported windows requires different data, not changing the original model's clocks or replacing missing ticks with synthetic evidence.

## §9. Verification and RFC gates

{gate_table}

- Exact Phase4 source/binary provenance and202 original native path comparisons.
- {r['verification']['native_arithmetic_checks']} independent native cost arithmetic checks.
- {r['verification']['verified_market_file_count']} verified market source files; all399 Phase5 receipts reconciled.
- Three focused regression tests; full research test outcome and deterministic rerun proof in verification JSON.
- Protected validation unopened, offline network guard active, zero new broker requests.
- Coverage SHA256:`{scope['coverage_sha256']}`; result SHA256:`{sha(RESULT)}`.

Reproduction requires the retained raw caches and frozen Rust adapters. Run `python3 ops/research/phase5b_audit.py prepare` and `evaluate` only with matching scope/source/binary hashes; source changes require a separately registered audit. The report is generated by `python3 ops/research/phase5b_report.py`. GitHub artifacts contain the audit and native outputs, not the complete raw tick cache.

## §10. All160 Phase5 source dispositions

{register}

## §11. Instructions for Gemini

Read this report with `docs/rebuild_phase5b_exact_frozen_candidate_audit_18366.json`, the Phase5B scope/verification JSON and the exact Phase4 specification. Verify the86+10 completed exclusions,68 incompatible completed windows,2 exact new Tier1 episodes, separate Tier2 models, unchanged independent dates and source provenance. Do not call a Tier2 result an exact frozen-candidate win, or a combined development result independent validation. Preserve the73% /1:1.38 account goal, serial/paragraph references, SGT timestamps and “Time is Money” invariant.
'''
    replacements=[('The96','The 96'),('those96','those 96'),('All160','All 160'),('All202','All 202'),('All399','All 399'),('Phase5','Phase 5'),('Phase4','Phase 4'),('Phase3','Phase 3'),('Tier1','Tier 1'),('Tier2','Tier 2'),('the73%','the 73%'),('The73%','The 73%'),('remain2/12','remain 2/12'),('dates20','dates 20'),('on20October2023','on 20 October 2023'),('sample.72h','sample. 72h')]
    for x,y in replacements:text=text.replace(x,y)
    for x,y in [('the86+10','the 86 + 10'),('The75','The 75'),('the86','the 86'),('the96','the 96'),('original160','original 160'),('The5/20','The 5/20'),('its48bp and28bp','its 48 bp and 28 bp'),('gross1.38R','gross 1.38R'),('realised1.38','realised 1.38'),('visible.72h','visible. 72h'),('interpreting3.74bp','interpreting 3.74 bp'),('native4h','native 4h'),('windows.72h','windows. 72h'),('at3.74','at 3.74'),('at24bp','at 24 bp'),('at28bp','at 28 bp'),('is3.74','is 3.74'),('is48bp','is 48 bp'),('executable50%','executable 50%'),('across45','across 45'),('frozen200,000','frozen 200,000'),('Of158','Of 158'),('new72','new 72'),('native sessions09:30','native sessions 09:30'),('open07:00','open 07:00'),('open09:30','open 09:30'),('3.74bp','3.74 bp'),('all160','all 160'),('the73% /1:1.38','the 73% / 1:1.38')]:text=text.replace(x,y)
    REPORT.write_text(text)
    body,toc=markdown_body(text)
    css='body{font:16px/1.6 system-ui,sans-serif;color:#173449;background:#f4f7fa;margin:0}main{max-width:1400px;margin:auto;padding:28px}h1,h2{line-height:1.25}h2{margin-top:36px}p{max-width:1120px;overflow-wrap:anywhere}table{border-collapse:collapse;background:white;font-size:13px}td,th{padding:10px;border:1px solid #c8d4df;vertical-align:top;text-align:left}th{background:#204866;color:white;position:sticky;top:0}tr:nth-child(even){background:#edf2f7}.table-scroll{overflow:auto;max-height:600px;margin:20px 0}pre{overflow:auto;background:#203c53;color:white;padding:18px}code{overflow-wrap:anywhere}nav{display:flex;flex-wrap:wrap;gap:12px}a{color:#075eaa}@media(max-width:600px){main{padding:16px}h1{font-size:26px}.table-scroll{max-height:450px}}'
    REPORT.with_suffix('.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Phase5B №18,366</title><style>'+css+'</style><main><details><summary>Contents</summary><nav>'+''.join(f'<a href="#{a}">{t}</a>' for a,t in toc)+'</nav></details>'+body+'</main></html>')
    print(REPORT)

if __name__=='__main__':render()
