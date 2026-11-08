#!/usr/bin/env python3
"""Render Phase 3 measured JSON; no strategy execution or collection."""
import collections
import datetime as dt
import html
from pathlib import Path
from tiered_replay import PLAN,RESULT,ROOT,read,sha,u
from unified_replay_report import table,fmt,pct,markdown_body

REPORT=ROOT/'doc/REBUILD_PHASE3_COVERAGE_AND_PULLBACK_REPORT-18360-2026-11-08.md'
VERIFY=ROOT/'docs/rebuild_phase3_verification_18360.json'

def render():
    r=read(RESULT);p=read(PLAN);v=read(VERIFY);a=r['coverage']
    if sha(RESULT)!=v['result_sha256'] or sha(PLAN)!=r['plan_sha256']:raise ValueError('report hash mismatch')
    headline=[];matrix=[];moderate=[];macro=[];benchmark=[];trades=[];context=[]
    for cell,groups in r['summary'].items():
        admission=cell.split('admission=')[1]
        for arm in u.ARMS:
            for mode in u.MODES:
                for cost in u.BPS:
                    x=groups['all'][arm][mode][cost]
                    matrix.append([cell,arm,mode,cost,x['entries'],x['priced_closed'],x['wins'],x['losses'],x['unresolved'],pct(x['net_win_rate']),x['mean_net_r'],x['profit_factor'] if x['profit_factor_status']=='finite' else x['profit_factor_status']])
            x=groups['all'][arm]['forced']['24'];y=groups['all'][arm]['forced'][admission]
            headline.append([cell,arm,x['episodes'],x['entries'],f"{x['wins']}/{x['priced_closed']}",x['unresolved'],x['mean_net_r'],f"{y['wins']}/{y['priced_closed']}",y['mean_net_r']])
            z=groups['moderate_28_to48'][arm]['forced']
            moderate.append([cell,arm,z['24']['episodes'],z['24']['entries'],f"{z['24']['wins']}/{z['24']['priced_closed']}",z['24']['mean_net_r'],f"{z[admission]['wins']}/{z[admission]['priced_closed']}",z[admission]['mean_net_r']])
            for c in ['demo_entry_plus_2_exit','demo_entry_plus_3.74_exit']:
                x=groups['all'][arm]['forced'][c]
                benchmark.append([cell,arm,c,x['priced_closed'],x['cost_unavailable'],f"{x['wins']}/{x['priced_closed']}",x['mean_net_r']])
        for cost in ['24',admission] if admission!='24' else ['24']:
            for category,x in r['paired_macro'][cell]['forced'][cost]['categories'].items():
                macro.append([cell,cost,category,x['entries'],x['priced_closed'],x['wins'],x['losses'],x['unresolved'],x['mean_net_r']])
    for row in r['rows']:
        n=row['native']
        if not n.get('impulse'):continue
        label=f"{row['tier']}/{row['threshold_bps']:g}/{row['admission_bps']:g}"
        c=row['own_context'];pc=row['peer_context'];context.append([row['episode_id'],label,row['symbol'],row['date'],c['status'],c['drift_5'],c['drift_20'],pc['status'] if pc else 'absent',n['macro']['reason'],n['pullback']['status']])
        for arm,x in n['arms'].items():
            f=x.get('forced');s=x.get('strict');cont=x.get('continuation')
            if not f:continue
            i=f['entry'];metrics=f.get('metrics')
            trades.append([row['episode_id'],label,row['symbol'],row['date'],arm,i['side'],i['entered_at'],i['fill'],i['initial_r'],s['status'],f['status'],metrics['net_by_bps']['24'] if metrics else None,metrics['net_by_bps'][f"{row['admission_bps']:g}"] if metrics else None,cont['status']])
    definitions=[]
    exclusions={x['episode_id']:x for x in a['tier2_excluded']};aliases={x['episode_id']:x for x in a['same_session_aliases']}
    for row in a['rows']:
        q=row['phase3_qualification'];reason=exclusions.get(row['episode_id'],{}).get('reasons',[])
        if row['episode_id'] in aliases:reason=['same_account_symbol_date_alias:'+aliases[row['episode_id']]['canonical_episode_id']]
        definitions.append([row['episode_id'],row['symbol'],row['date'],row['protocol_status'],q['tier'] or 'excluded',', '.join(row['phase2_exclusion_reasons']) or 'qualified',', '.join(reason),len(row['failed_m30_interval_overlaps']),row['tick_receipt_sha256']])
    symbol_rows=[]
    for symbol in p['symbols']:
        ds=[x for x in a['rows'] if x['symbol']==symbol]
        symbol_rows.append([symbol,len(ds),sum(x['tier1_selected'] for x in ds),sum(x['phase3_qualification']['tier']=='tier2' for x in ds),sum(x['phase3_qualification']['tier'] is None for x in ds)])
    gates=[
        ['G1','Partial','Plan/code/source/calendar/contract identities linked; historical contract authority remains unknown.'],
        ['G2','Pass for bounded migration','159 legacy plus35 Phase 2 outputs exactly reproduced; no live parity claim.'],
        ['G3','Pass for this offline experiment','Single manifest runner, finite policy matrix, offline guard, denominators retained, no arbitrary outcomes. Historical scripts retained as frozen compatibility evidence.'],
        ['G4','Partial','263 definitions accounted, failures/quarantine/repeats/aliases retained. Tier 2 is explicitly partial-context; continuous executable history and full holding paths not certified.'],
        ['G5','Open','Coded session registry reused. Authoritative historical early closes, maintenance, holidays and contract changes still missing.'],
        ['G6','Open','13 demo entry observations are not matched round trips; fees, financing, conversion and money reconciliation absent.'],
        ['G7','Open','No held-out round-trip cost calibration;2/3.74bp remain assumptions.'],
        ['G8','Partial','Existing two-stage exit engine reused; causal entry/dispatch tests and old outputs pass. Queue, broker acknowledgements, restarts and research/live parity remain unproved.'],
        ['G9','Development protocol only','Plan frozen before outcomes; tiers/repeats/protected dependencies separated. Historical cohort selection and previous data reuse prevent independent validation claims.'],
        ['G10','Not passed','Report all arms and costs; no financial qualification from this reused sample or unverified cost model.'],
        ['G11','Not performed','No shadow/canary broker comparison or live deployment.'],
        ['G12','Not demonstrated here','No per-strategy/account latest 20 goal qualification; existing live risk rules untouched.']]
    primary=r['summary']['tier2|floor=48|admission=24']['all']
    low=r['summary']['tier2|floor=28|admission=3.74']['all']
    b=primary[u.ARMS[1]]['forced']['24'];c=primary[u.ARMS[2]]['forced']['24']
    breakout=primary[u.ARMS[0]]['forced']['24']
    strict_macro=r['summary']['tier1|floor=48|admission=3.74']['all'][u.ARMS[2]]['forced']['24']
    strong_rows=[x for x in r['rows'] if x['tier']=='tier1' and x['threshold_bps']==48 and x['admission_bps']==3.74 and x['native'].get('arms',{}).get(u.ARMS[2],{}).get('forced')]
    strong_dates=sorted({x['date'] for x in strong_rows})
    rejected=r['paired_macro']['tier2|floor=48|admission=24']['forced']['24']['categories']['own_trend_veto']
    sensitivity=low[u.ARMS[2]]['forced']['3.74'];stress=low[u.ARMS[2]]['forced']['24']
    text=f'''# Phase 3 — Coverage decomposition and expanded pullback evaluation

№18,360 §§1–12 · {v['verified_at_sgt']} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**263 = 35 Tier-1 sessions + 228 Phase-2 exclusions.** The exclusions comprise 196 clean-history missing-preparation definitions, 4 clean-history missing-opening-quote definitions, 20 inherited-repeat definitions and 8 unverified histories. Phase 3 qualifies **{a['tier2_selected']} additional distinct Tier-2 sessions** under a separately frozen first 30-minute range rule. These are reused development observations, not new independent evidence.

**Both parity gates passed: 159/159 legacy episodes and 35/35 Phase 2 sessions.** All results below come from the native Rust replay and verified local sources. New broker requests: 0. Live changes: 0. Account qualification: 0.

**Measured forced-four-hour result:** at the strict 48 bp threshold and 24 bp admission, Tier 2's breakout lost {breakout['losses']}/{breakout['priced_closed']} trades; pullback B won {b['wins']}/{b['priced_closed']} with mean {b['mean_net_r']:.4f}R. Macro C entered {c['entries']} trades; its directional filter rejected {rejected['losses']} losing B trades. Avoiding these observed losses is useful development evidence, but zero trades is neither a win rate nor a profitable strategy.

At the 28 bp threshold and 3.74 bp admission, Tier-2 C won **{sensitivity['wins']}/{sensitivity['priced_closed']}**, mean **{sensitivity['mean_net_r']:+.4f}R**, under the same assumed cost. Repricing those exact paths at 24 bp produced **{stress['wins']}/{stress['priced_closed']}**, mean **{stress['mean_net_r']:+.4f}R**. The favourable low-cost result does not survive the registered stress cost. Tier-1 strict-threshold lower-cost admission looks stronger (C: {strict_macro['wins']}/{strict_macro['priced_closed']} stress-priced wins), but those are reused full-M30 sessions and cost-dependent geometry, not independent confirmation.

All {len(strong_rows)} of those stronger Tier-1 C trades lie on just {len(strong_dates)} dates, {strong_dates[0]} through {strong_dates[-1]}. Correlated oil/index trades in one selected macro week are not independent regime replications. For the strict-threshold Tier-2 B comparison, all {primary[u.ARMS[1]]['strict']['24']['unresolved']} positions remain unresolved at four hours before the separate forced-close calculation; continuation resolves {primary[u.ARMS[1]]['continuation']['24']['priced_closed']} and leaves {primary[u.ARMS[1]]['continuation']['24']['unresolved']} unresolved.

**Progress toward the goal:** the original 73% / 1:1.38 target applies to each strategy/account over its latest 20 trades. This research adds **0 qualifying account trades and 0 independent validation trades**. It improves coverage and identifies conditional behaviour; no target achievement or production promotion is claimed. The interim 50% research target does not override costs, representativeness or evidence gates.

At 24 bp admission, a 28–48 bp impulse normally fails the full round-trip reserve before entry. The lower admission rows deliberately expose those trades, then price the **same fills and geometry** at 24 bp stress. A lower-cost sensitivity win is not a verified broker net win.

## §1. Exact denominator and exclusions

{table(['Primary exclusion reason (mutually exclusive)','Definitions'],a['exclusive_primary_reasons'].items())}

{table(['Overlapping reason (do not sum)','Definitions'],a['overlapping_reasons'].items())}

{table(['Symbol','Core definitions','Tier 1','Tier 2','Excluded or alias'],symbol_rows)}

The 6 quarantined receipts comprise 5 canonical episodes and 1 duplicate; 4 canonical BTC episodes are core, 1 NatGas episode is outside core. They were already outside 263. The two failed US500 24 January 2024 wrappers represent one failed tick episode, also outside 263. They cannot be reassigned into the 228 without double counting.

The 24 retained failed M30 receipts cover 8 intervals each for SpotCrude,SpotBrent andUS500. Their files say `bar_identity_or_truncation_unverified`; the specific count500 root cause is **not established by those receipts alone**. The JSON records interval overlaps per affected definition, including overlaps subsequently superseded by repeat requests. Two raw BTC M30 responses are not qualified decoded bars. No failed source is reclassified as a successful zero-trade replay.

Among 200 excluded definitions marked clean_first_read, qualification additionally requires the native 4h source interval, fresh open and 30-minute boundary quotes and both tick sides near the end. Duplicate account/symbol/date definitions are collapsed before outcomes, with Tier 1 priority and then longest interval/SHA256. Full exclusions and aliases are retained in §10 and JSON. Clean collection provenance does not prove a complete executable quote path.

## §2. Frozen method and costs

Tier 1 preserves Phase 2 historical 14-session sigma bands and full 45-calendar-day coded-session preparation. Tier 2 uses the maximum absolute midpoint excursion from the opening midpoint during the first 30minutes, frozen before any decision. That symmetric radius is a causal opening-range proxy, **not sigma**. Its results stay separate and cannot establish that the original sigma model works with missing data.

At half-hour marks, both tiers require a fresh decision and a strictly later fresh executable quote within 5 seconds. Tier 2 confirms bid above the upper band or ask below the lower band. Both retain the full-cost durable veto and registered 28/48 bp risk floor at decision and dispatch. The first admitted impulse is fixed; there is no retry after a failed retest or macro veto.

Arm A enters that impulse. Arm B waits for a fixed 50% retracement to the opening midpoint and rejects a first touch beyond the origin. Its structuralR includes the registered admission reserve, so changing admission cost changes both eligibility and pullback risk geometry. Arm C uses the identical B trade only when fixed 5/20-session trends align. Tier 2 computes those drifts from the exact causal endpoint bars, records interior gaps, and never substitutes a shorter lookback. A same-date clean peer, when available, must not conflict; missing peer context blocks C. An absent peer is disclosed and allowed by the conditional rule. This is technical price context, not measured fundamental or institutional information.

Every entry uses the unchanged two-stage exit engine: −1R initial stop, entry-price stop proposal at +1R applied only on a later quote, gross 1.38R target. Entry-price breakeven is usually a net loss after costs. Strict bracket closes, separate forced 4h close and conditional consecutive-M30 continuation remain separate. Continuation stops at a cache gap, ambiguous OHLC path or 72 elapsed-hour limit; short ask bars use the disclosed frozen entry-spread proxy.

Recorded spread enters executable bid/ask prices once. Extra reserve is `(entry+exit)*bps/10000/R`. Same-account demo entry benchmarks remain retrospective sensitivity only, paired with assumed exit costs; no account pooling or missing-as-zero substitution. Commission, swap, conversion, historical contracts and true round-trip slippage remain unverified.

## §3. Main comparison — forced four-hour closes

The last two columns price the same paths at their admission assumption. The stress columns always use 24 bp/side. No-entry and unresolved outcomes are retained in their own denominators. Tier/threshold/cost variants are overlapping trials, not independent extra trades.

{table(['Tier / floor / admission','Arm','Sessions','Entries','24bp wins/closes','Unresolved','24bp meanR','Admission-cost wins/closes','Admission-cost meanR'],headline)}

## §4. Moderate 28–48 bp subgroup

This subgroup uses the admitted impulse risk divided by its fill price, before pullback resizing. A 28 bp rule may change the first admitted impulse and thus replace a later 48 bp entry; it is not necessarily a nested superset of the 48 bp trades. Costs remain applied even when larger than the eventual price move.

{table(['Cell','Arm','Moderate impulses','Entries','24bp wins/closes','24bp meanR','Admission-cost wins/closes','Admission-cost meanR'],moderate)}

## §5. Does Arm C remove losing B trades?

{table(['Cell','Evaluation bp/side','B fate under C','Entries','Priced closes','B wins','B losses','Unresolved','B meanR'],macro)}

`own_trend_veto` and `peer_trend_veto` are directional decisions. `own_context_unavailable` and `peer_context_unavailable` are data abstentions and cannot support an incremental-alpha claim. Evaluate rejected winners as well as losers, retained return and remaining sample size. Conditional results on reused, correlated regimes do not establish a causal benefit or future win rate. Paired episode IDs and strict/forced/continuation outcomes for every cost track are in the JSON.

## §6. Full resolution and pricing matrix

{table(['Cell','Arm','Resolution','Evaluation bp/side','Entries','Closes','Wins','Losses','Unresolved','Win rate','MeanR','PF/status'],matrix)}

## §7. Demo-entry benchmark sensitivities

{table(['Cell','Arm','Benchmark + assumed exit','Priced closes','Cost unavailable','Wins/closes','MeanR'],benchmark)}

## §8. Verification and rebuild acceptance

- Plan SHA256:`{r['plan_sha256']}`.
- Result SHA256:`{sha(RESULT)}`.
- Frozen code commit:`{r['code_freeze']['commit']}`.
- 159 exact legacy comparisons; 35 exact Phase 2 input/output comparisons, repeated through the default policy wrapper.
- Independent cost arithmetic checks:{r['verification']['cost_arithmetic_checks']}.
- Source files hash-verified:{r['verification']['verified_source_files']}; protected receipts explicitly rejected:{v['quarantined_receipt_rejections']}.
- Focused tests, determinism and unchanged production-file verification:see `docs/rebuild_phase3_verification_18360.json`.
- Coverage selection was saved before new outcomes:`{r['verification']['coverage_frozen_before_new_outcomes_sha256']}`.

{table(['RFC gate','Disposition','Evidence / remaining work'],gates)}

**Next decision:** retain the frozen measured arms as development evidence. Do not optimise parameters on these same dates or promote a high point win rate. Resolve the minimal historical session/contract and matched round-trip execution evidence, then register a truly independent replication with fixed costs and minimum sample requirements. Keep directional macro rejection separate from missing-context abstention. A matched-risk entry comparison is still required to isolate timing from the changed structural stop. Fresh unspent M30 collection or cTrader deal capture requires a later authorised phase; none occurs here.

The opening-range Tier-2 expansion did not reproduce the earlier full-M30 pullback result. Keep those hypotheses separate. The strict-threshold full-M30 pullback is a candidate for independent replication; the moderate-threshold result is highly cost-sensitive. This evidence does not justify weakening the production gate or replacing the live strategy with either variant.

## §9. Entered-trade and macro registers

{table(['Episode','Tier/floor/admission','Symbol','Date','Arm','Side','Entry UTC','Fill','Risk price','Strict status','Forced status','Stress netR','Admission netR','Continuation'],trades)}

{table(['Episode','Cell','Symbol','Date','Own status','Own5','Own20','Peer status','Macro reason','Pullback result'],context)}

## §10. Complete 263-definition register

{table(['Episode','Symbol','Date','Receipt history','Phase 3 tier','Phase 2 reasons','Phase 3 exclusion / alias','Failed M30 overlaps','Receipt SHA256'],definitions)}

## §11. Reproduce and hand over

```text
cargo build -p jiaoyi-core --example native_target_sizing_replay --example manifest_pullback_replay --example tiered_pullback_replay --example replay_inventory_metadata --locked --offline
python3 ops/research/tiered_replay.py --workdir /tmp/jiaoyi-phase3-replay --output /tmp/jiaoyi-phase3-results.json
python3 -m unittest discover -s ops/research -p test_tiered_replay.py
cargo test -p jiaoyi-core research::tiered_pullback --locked --offline
```

Use the frozen code commit for byte-for-byte reproduction. A GitHub clone contains the plan, measured outputs, report and hashes; replay additionally needs the Phase 1 manifest's original local files. Missing or changed files fail closed. Running the same committed implementation in the same work directory reuses verified native receipts and returns identical results. Later source revisions require a separate work directory.

## §12. Instructions for the independent reviewer

Read the Phase 3 plan,results and verification JSON alongside this report; compare to Phase 2 at 98fef7a. Check the 263/228 denominator, excluded provenance, Tier 2 definition, cost-admission versus fixed-path repricing, C's rejected winners and losers, and strict versus forced/continuation resolution. Do not pool policy trials or infer account qualification. Challenge the economic hypothesis and required independent evidence, not just whether CI compiles. Goals,№/§/SGT traceability, preserving proven fixes and “Time is Money” remain the invariants.
'''
    REPORT.write_text(text)
    body,toc=markdown_body(text)
    css='body{font:16px/1.6 system-ui,sans-serif;color:#192c3d;background:#f4f7fa;margin:0}main{max-width:1440px;margin:auto;padding:28px}h1,h2{line-height:1.25}h2{margin-top:42px}p{max-width:1100px}table{border-collapse:collapse;background:white;font-size:13px}td,th{padding:10px;border:1px solid #ccd7e0;vertical-align:top}th{background:#173c58;color:white;text-align:left;position:sticky;top:0}tr:nth-child(even){background:#edf3f7}.table-scroll{overflow:auto;max-height:650px;border:1px solid #ccd7e0;margin:20px 0}pre{overflow:auto;background:#173c58;color:white;padding:18px}code{overflow-wrap:anywhere}nav{display:flex;gap:12px;flex-wrap:wrap}a{color:#065db5}@media(max-width:600px){main{padding:16px}h1{font-size:26px}.table-scroll{max-height:480px}}'
    REPORT.with_suffix('.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi Phase 3 — №18,360</title><style>'+css+'</style><main><details><summary>Contents</summary><nav>'+''.join(f'<a href="#{a}">{t}</a>' for a,t in toc)+'</nav></details>'+body+'</main></html>')
    print(REPORT)

if __name__=='__main__':render()
