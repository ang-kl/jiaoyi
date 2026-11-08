#!/usr/bin/env python3
"""Human/independent-AI report from the measured Phase4 audit only."""
import json
from pathlib import Path
from phase4_forensics import RESULT,SCOPE,ROOT,read,sha
from unified_replay_report import table,fmt,pct,markdown_body

REPORT=ROOT/'doc/REBUILD_PHASE4_FORENSIC_AUDIT_REPORT-18362-2026-11-08.md'
VERIFY=ROOT/'docs/rebuild_phase4_verification_18362.json'

def render():
    r=read(RESULT);v=read(VERIFY)
    if sha(RESULT)!=v['result_sha256'] or sha(SCOPE)!=r['scope_sha256']:raise ValueError('report evidence mismatch')
    ledgers=r['ledgers'];a=ledgers['tier1_7_of_7']['anatomy'];b=ledgers['tier2_6_of_8']['anatomy'];c=ledgers['vetoed_3']['anatomy']
    up=r['tier2_upgrade_audit'];plan=r['independent_evaluation_plan'];candidate=r['frozen_candidate_hypothesis'];allrows=[t for z in ledgers.values() for t in z['trades']]
    ledger_tables={}
    for group,ledger in ledgers.items():
        ledger_tables[group]=table(['Episode ID','Symbol','Date','Direction','Impulse R bp','5-session drift','20-session drift','Peer status','Entry UTC','Exit UTC','Exit class','Net R at 3.74 bp','Net R at 24 bp'],[
            [t['episode_id'],t['symbol'],t['date'],t['direction'],t['impulse_r_bps'],pct(t['macro_5d_drift']),pct(t['macro_20d_drift']),t['peer_status'],t['entry_time_utc'],t['exit_time_utc'],t['exit_reason'],t['net_r_at_3_74bp'],t['net_r_at_24bp']] for t in ledger['trades']])
    anatomy=table(['Group','Trades','Strict targets','BE-stop exits','Initial stops','Forced closes','Profitable forced closes at 3.74 bp','Wins at 3.74 bp','Wins at 24 bp','Mean R at 3.74 bp','Mean R at 24 bp','Distinct dates'],[
        [k,*[x['anatomy'][f] for f in ['trades','strict_target_1_38r_hits','breakeven_after_1r_exits','initial_stop_exits','forced_window_closes','profitable_forced_window_closes','wins_at_3_74bp','wins_at_24bp','mean_net_r_at_3_74bp','mean_net_r_at_24bp','unique_dates_count']]] for k,x in ledgers.items()])
    prices=table(['Group / episode','Symbol','Fill','Exit','Initial risk price','Gross target distance','Actual gross R','Symmetric break-even cost bp/side','Stop amendments applied','Raw exit reason','Continuity caveat'],[
        [t['group']+' / '+t['episode_id'],t['symbol'],t['entry_price'],t['exit_price'],t['initial_r_price'],t['gross_target_distance'],t['gross_r'],t['symmetric_cost_break_even_bps'],t['trail_applied'],t['raw_exit_reason'],', '.join(t['path_data_issues'])] for t in allrows])
    veto=ledgers['vetoed_3']['trades'];own=sum(t['effective_veto']=='own_5d_20d' for t in veto);peer=sum(t['effective_veto']=='peer_disagreement' for t in veto);also=sum(t['peer_would_conflict'] for t in veto)
    categories=table(['Mutually exclusive upgrade disposition','Episodes'],up['categories'].items())
    symbols=[]
    for s in candidate['symbols']:
        rs=[x for x in up['rows'] if x['symbol']==s]
        symbols.append([s,len(rs),sum(x['local_preparation_status']=='partial' for x in rs),sum(x['local_preparation_status']=='none_in_required_lookback' for x in rs),*[sum(x['category']==cat for x in rs) for cat in ['tier1_ready_local','only_unspent_history_missing','needs_spent_gap_or_calendar_reconciliation','protected_45day_dependency']]])
    symbol_table=table(['Symbol','Tier 2 episodes','Partial local prep','No local prep in lookback','Ready now','Only unspent gaps','Spent/reserved/calendar gaps','Protected prep'],symbols)
    gap_table=table(['Episode','Symbol','Date','Required prep slots','Local prep slots','Missing prep slots','Opening M30 present','Missing within spent/reserved intervals','Missing outside retained intervals','Disposition'],[
        [x['episode_id'],x['symbol'],x['date'],x['required_prep_slots'],x['verified_local_prep_slots'],x['missing_prep_slots'],x['opening_M30_present'],x['missing_in_spent_or_reserved_intervals'],x['missing_unspent_under_retained_catalog'],x['category']] for x in up['rows']])
    narrow=next(x for x in up['rows'] if x['symbol']=='US500' and x['date']=='2024-11-04')
    windows=table(['Week beginning','Scheduled dates','Prep envelope begins UTC','Envelope ends UTC','Tick jobs ceiling','M30 segment ceiling'],[
        [w['week_start'],', '.join(w['dates']),w['preparation_envelope_from'],w['envelope_to'],sum(j['date'] in w['dates'] for j in plan['jobs']),sum(j['week_start']==w['week_start'] for j in plan['m30_segments'])] for w in plan['weeks']])
    selected_summary=table(['Frozen component','Specification'],[(k,candidate[k]) for k in ['id','economic_hypothesis','falsification','primary_model','source_mode','impulse','entry','macro','planned_peer_policy','initial_r','exit','sizing']])
    gates_table=table(['RFC gate','Disposition','Evidence / remaining requirement'],[[g['gate'],g['status'],g['reason']] for g in r['rfc_gates']])
    hashes=table(['Group / episode','Tick receipt SHA256','Native output SHA256'],[[t['group']+' / '+t['episode_id'],t['tick_receipt_sha256'],t['native_output_sha256']] for t in allrows])
    thin=min((t for t in ledgers['tier2_6_of_8']['trades'] if t['net_r_at_3_74bp']>0),key=lambda t:t['symmetric_cost_break_even_bps'])
    text=f'''# Phase 4 — Arm C trade anatomy and one frozen hypothesis

№18,362 §§1–12 · {v['verified_at_sgt']} · by <Codex (GPT-6), effort: not exposed>

## TLDR

**Tier 1's seven wins are mostly completed targets:** {a['strict_target_1_38r_hits']} strict +1.38R target hits and {a['profitable_forced_window_closes']} profitable forced four-hour close. There are {a['breakeven_after_1r_exits']} breakeven-stop exits. All seven are short trades concentrated in five dates in the April 2025 cohort. This supports a specific development hypothesis, not a proven general edge.

**Tier 2's six wins are more dependent on the clock and cost assumption:** {b['strict_target_1_38r_hits']} target hits, {b['profitable_forced_window_closes']} profitable forced closes and {b['losing_trades']} losses. All eight become losses at 24 bp/side stress pricing. A target hit is therefore not sufficient evidence of positive net expectancy.

**The three vetoed paths are hypothetical B trades, not executed C trades.** All three are unresolved under strict four-hour bracket accounting and negative at forced close. Own 5/20-session direction vetoes all {own}; effective peer-only vetoes: {peer}. Their mean avoided counterfactual return is {c['mean_net_r_at_24bp']:.6f}R under the 24 bp stress scenario.

**Tier-2 upgrade inventory:** 167 episodes; none ready locally. {up['categories'].get('only_unspent_history_missing',0)} need only missing intervals outside the retained request catalog; {up['categories'].get('needs_spent_gap_or_calendar_reconciliation',0)} need gaps within previously requested/reserved intervals or calendars reconciled; {up['categories'].get('protected_45day_dependency',0)} need sealed preparation and cannot be upgraded under the present seal. No episode is after 24 October 2025.

**Frozen candidate:** `s3_macro_pullback_v1`, the full-M30 48 bp impulse / 3.74 bp admission rule with primary 24 bp stress evaluation. The opening-range 28 bp variant is not silently substituted. Four metadata-selected historical weeks reserve 20 dates × 8 symbols = 160 tick jobs, plus at most 256 deduplicated M30 segment jobs. None is executed. The November week needs explicit holiday/short-session verification before these ceilings become executable jobs.

**Goal progress:** 18 previously simulated paths audited; 0 new independent observations; 0 new broker requests; 0 account-goal qualification. The original 73% / 1:1.38 latest-20 goal per strategy/account and interim 50% per-class research screen remain unchanged. RFC gates fully passed: {r['rfc_gates_passed']}/12, inherited bounded engineering passes G2/G3; freezing a proposal does not pass the financial gates.

## §1. Evidence, terminology and the question answered

This audit reads Phase3 at commit `145f9154426ba355d95cb2413f9d4a634a5051ef` (PR #193), its immutable native-output receipts, the Phase1 manifest and verified cached quote files. It does not rerun new strategy arms or inspect new market dates. All18 entries and exits are reconciled to recorded executable-side quotes; target fills are the replay engine's target-level assumption once the recorded quote crosses that level. These are broker-data **simulated paths**, not actual cTrader execution deals.

For each row, gross R is signed `(exit-entry)/initial_r`; cost-scenario net R subtracts `(entry+exit)*bps/10000/initial_r`. Recorded bid/ask spread is already included once. Decimal arithmetic independently checks both cost columns. Commission, swap, conversion, actual dispatch latency, queue fills and full round-trip slippage remain unverified. The current contract grid is a recorded scenario, not a verified historical contract specification.

Strict target means the target resolved inside the observed window, with outward grid rounding around gross1.38R. A `forced_window_close` uses the separate boundary-close calculation after native session open+4h, not four hours after entry. A stop moved to entry is distinct from a trade actually exiting at that stop; a gap through an amended entry stop can still lose money. The audit classifies actual exit state rather than guessing from positive/negative P&L.

{anatomy}

## §2. Tier 1 — complete seven-trade Arm C ledger

The six strict targets are the stronger part of this result. The remaining winner is a marked-to-market close, not another observed target completion. The three represented instruments include two correlated oil contracts. The short-only concentration in one selected macro week prevents extrapolation to long trades, normal-volatility periods, metals, crypto or live accounts.

{ledger_tables['tier1_7_of_7']}

## §3. Tier 2 — complete eight-trade Arm C ledger

The two losses comprise one initial-stop exit and one negative forced close. Three other forced closes are profitable only under the lower cost assumption. The narrowest winner is {thin['symbol']} on {thin['date']}: break-even symmetric slippage is approximately **{thin['symmetric_cost_break_even_bps']:.6f} bp/side**, only {thin['symmetric_cost_break_even_bps']-3.74:.6f} bp/side above the assumed3.74. An omitted fee or slightly worse execution can reverse it.

The three target hits also turn negative under24bp stress because their risk distance is small relative to the stated reserve. This is direct evidence of cost fragility, not an argument that costs should be reduced until results pass.

{ledger_tables['tier2_6_of_8']}

## §4. Three B paths rejected by Arm C

All {own} effective vetoes come from the own5/20-session trend test, evaluated before the peer test. {also} path(s) would additionally conflict with the peer's direction; that is not counted a second time as an effective peer veto. Missing context is not responsible for these three decisions. C entered none of them. The avoided mean is a conditional retrospective difference, not realised profit or proof of future filtering accuracy.

{ledger_tables['vetoed_3']}

## §5. Price geometry, costs and exit mechanics

{prices}

The same fixed admission reserve also contributes to B/C's structural risk distance. Consequently, lower-cost admission changes both selection and risk/target geometry; fixed-entry repricing isolates accounting only after those entries are chosen. This known coupling is preserved for a faithful candidate replication. Separating risk geometry from cost would require a different registered version, not a silent amendment labelled the same strategy.

Every selected path carries recorded continuity caveats. A quote matching a modelled fill proves price-path consistency, not guaranteed executable liquidity. No profitable forced close is relabelled as a strict target; no censored path is turned into a zero-return observation.

## §6. What can be upgraded from Tier 2?

{categories}

{symbol_table}

All167 symbols/episode combinations have some local M30 files for their symbol somewhere in the catalog. That is weaker than having the required history for a particular date: {up['local_preparation_status'].get('partial',0)} have partial usable preparation and {up['local_preparation_status'].get('none_in_required_lookback',0)} have none inside the required lookback. No complete45-day preparation exists for these167 definitions under the existing rules.

The January2024 NAS100/XAGUSD collector requested only the four-hour intraday M30 job window. Its embedded bars must not be treated as a spent45-day preparation interval. Correctly reading that collector moves nine cases from the initial conservative gap category into the unspent-history category. A regression test now protects this distinction. Other inferred envelopes remain explicitly conservative; definitive broker-global request history is not available.

**Smallest concrete missing slice:** US5004November2024 has {narrow['verified_local_prep_slots']} of {narrow['required_prep_slots']} required slots. Its missing range is `{json.dumps(narrow['unspent_missing_runs'])}`. The collection boundary was shifted by the later UTC session opening across DST, leaving the earlier preparation day's first hour outside the requested range. This is a specific collection-boundary diagnosis, not evidence that all incomplete M30 history is broken. The future plan now starts preparation at the earlier of the 45-day UTC boundary and the dated native session opens, so it includes this hour across DST. A focused regression covers that boundary. No refetch or code change to the trading strategy is performed.

The17 protected cases occur on October2025 labels whose45-day context reaches the sealed19August–5September interval. They cannot be upgraded by simply collecting the missing bars. The132 spent/reserved-gap cases need source/decoder or dated-calendar reconciliation before any further request. Holidays and maintenance may legitimately have no bar; counting coded weekdays alone is not an authoritative market calendar. The18 potential unspent repairs would still be reused development data, and their new Tier1 signals/outcomes are unknown until qualified; existing Tier2 profits do not transfer automatically.

## §7. One frozen candidate economic hypothesis

{selected_summary}

The hypothesised cause is persistence in a pre-existing directional flow after a temporary counter-move. Institutional rebalancing is a possible mechanism, **not measured evidence in this cache**. The observable forecast is continuation after a same-direction impulse and retracement during the remaining session. It is falsifiable by negative independent stress-cost expectancy, failure against the fixed B control, concentration in one event/class or sensitivity to unverified execution assumptions.

The three admitted research arms are not three candidates to optimise after testing. Only C is the candidate. A and B are unchanged controls; no-trade is a zero-return reference. Planned peers are deliberately scheduled uniformly. A failed planned peer is a data failure, not a convenient transition to the peer-absent policy that helped some earlier US500 results.

## §8. Frozen independent-historical evaluation design

{windows}

{plan['selection']}

The retained registry contains {len(r['spent_registry']['intervals'])} deduplicated interval records and {len(r['spent_registry']['reserved_dates'])} date reservations from {len(r['spent_registry']['metadata_hashes'])} metadata files. Tick labels and45-day preparation bundles do not overlap retained market/request intervals, the fixed protected interval or each other. Selection used metadata, not price/return results.

**Limits of independence:** {r['spent_registry']['unavailable_historical_references']} historical references were unavailable in the earlier inventory. The overlap claim is therefore against the retained catalog, not universal broker/chat history. These2023 dates also precede later development samples; this is frozen historical replication, not chronological forward validation. G9 remains partial. Before execution, reconcile current global reservations and unavailable references where their bounds are uncertain; abort on an overlap rather than quietly relabelling a reused date.

The four-week/eight-symbol scope is the minimum number of weeks requested while preserving the existing core universe. It is a **maximum of160 tick jobs and256 M30 jobs**, not416 raw broker messages: tick pagination can require multiple messages and must retain existing per-job caps. M30 segments are deduplicated across thefive dates and use interval-only mode without count500. The plan does not launch a collector, consume an engineering-model session allowance or create a paid API call.

**Calendar checkpoint:** {plan['calendar_attention']} Holidays/shortened sessions must be classified using dated authoritative broker/venue evidence before any outcome is revealed. Exclusions stay in the denominator, and dates are not replaced after looking at performance. In particular, November 23–24 must not be assumed to be two ordinary US cash sessions.

Primary score: forced4h partial-cost netR at24bp/side plus recorded spread, with strict exit anatomy reported separately. Secondary3.74bp is a fixed sensitivity, not measured round-trip cost. Unknown fees and financing block financial claims. Candidate risk/entry parameters do not change if later cost calibration is inconvenient.

{plan['pilot_screen']}

{plan['uncertainty']}

{plan['stop_rule']}

Four weeks may produce too few candidate closes, especially by class. That result is insufficient evidence, not permission to keep sampling until a target hit rate appears. The account-level73% goal needs actual independently verified account trading outcomes; it is not certified by this pilot even if its descriptive win percentage is high.

## §9. Acceptance gates and next authorised boundary

{gates_table}

Phase4 completes the forensic audit and freezes the hypothesis/plan. It does not complete the eight-phase rebuild or authorise fresh collection/live deployment. Before a later execution phase, resolve the unspent-history audit bounds, calendars/contracts and round-trip execution costs; then collect exactly the reserved qualified inputs once and evaluate the frozen candidate/controls together. Preserve current proven implementation and risk rules.

## §10. Verification and reproducibility

- Audit JSON SHA256:`{sha(RESULT)}`.
- Scope SHA256:`{r['scope_sha256']}`; builder SHA256:`{r['builder_sha256']}`.
- {r['verification']['ledger_rows']} cached paths; {r['verification']['independent_net_arithmetic_checks']} independent decimal net-cost checks; {r['verification']['recorded_entry_and_exit_quote_checks']} recorded-side price checks; {r['verification']['native_cache_receipts_verified']} native output receipts reconciled.
- {r['verification']['future_jobs_overlap_checks']} future tick/preparation dependency checks; no broker requests.
- Five focused regression tests cover cost arithmetic/exit classification, independent-week overlap, gate inflation the January intraday-M30 boundary and the future DST preparation boundary. The report verification artifact records full research-suite and layout results.
- No Rust source or live trading rule changed. Phase3 parity evidence is inherited and hash-linked; Phase4 does not pretend to have generated new backtest observations.

```text
python3 ops/research/phase4_forensics.py
python3 -m unittest discover -s ops/research -p test_phase4_forensics.py
python3 ops/research/phase4_report.py
```

Reproduction needs the immutable Phase1 cache and Phase3 native receipts at the frozen paths, plus the existing calendar metadata binary. A GitHub-only checkout can read all delivered ledgers/plans but cannot recreate raw quote checks without those cached files. Missing or modified receipts fail closed. The verification JSON is separate from the deterministic measured audit.

## §11. Complete167-episode upgrade register

{gap_table}

Exact missing M30 runs, source paths, reservations and planned request intervals are retained in the audit JSON. No missing interval is represented as successfully collected.

## §12. Receipt register and instruction for Gemini

{hashes}

Read this report, `docs/rebuild_phase4_forensic_audit_and_hypothesis_freeze_18362.json`, `docs/rebuild_phase4_scope_18362.json` and `docs/rebuild_phase4_verification_18362.json` alongside Phase3. Check the 18 ledger rows, target-versus-clock anatomy, effective versus secondary peer veto, exact upgrade categories, cheap-cost fragility and four-week dependency reservations. Challenge causal/economic claims and execution prerequisites separately from CI success. Do not promote these selected wins or mark planned dates as executed. The original goals, serial/paragraph numbering, SGT timestamps, preserving proven fixes and “Time is Money” remain the invariants.
'''
    # Only prose tokens are normalised; hashes, filenames and JSON keys remain untouched.
    for a,b in [('All18','All 18'),('all18','all 18'),('gross1.38R','gross 1.38R'),('All167','All 167'),('these167','these 167'),('complete45','complete 45'),('spent45','spent 45'),('January2024','January 2024'),('US5004November2024','US500 4 November 2024'),('The17','The 17'),('October2025','October 2025'),('whose45','whose 45'),('sealed19August–5September','sealed 19 August–5 September'),('The132','The 132'),('The18','The 18'),('These2023','These 2023'),('of160','of 160'),('and256','and 256'),('not416','not 416'),('thefive','the five'),('without count500','without count=500'),('forced4h','forced 4h'),('Secondary3.74bp','Secondary 3.74 bp'),('account-level73%','account-level 73%'),('Complete167','Complete 167'),('the18','the 18'),('at24bp','at 24 bp'),('under24bp','under 24 bp'),('assumed3.74','assumed 3.74'),('own5/20','own 5/20'),('and45-day','and 45-day'),('Phase4','Phase 4'),('Phase3','Phase 3'),('Phase1','Phase 1')]:text=text.replace(a,b)
    REPORT.write_text(text)
    body,toc=markdown_body(text)
    css='body{font:16px/1.6 system-ui,sans-serif;color:#172d40;background:#f4f7fa;margin:0}main{max-width:1440px;margin:auto;padding:28px}h1,h2{line-height:1.25}h2{margin-top:40px}p{max-width:1120px}table{border-collapse:collapse;background:white;font-size:13px}td,th{padding:10px;border:1px solid #cbd7e1;vertical-align:top}th{background:#173c58;color:white;text-align:left;position:sticky;top:0}tr:nth-child(even){background:#edf3f7}.table-scroll{overflow:auto;max-height:620px;border:1px solid #cbd7e1;margin:20px 0}pre{overflow:auto;background:#173c58;color:white;padding:18px}code{overflow-wrap:anywhere}nav{display:flex;gap:12px;flex-wrap:wrap}a{color:#065db5}@media(max-width:600px){main{padding:16px}h1{font-size:26px}.table-scroll{max-height:460px}}'
    REPORT.with_suffix('.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi Phase4 — №18,362</title><style>'+css+'</style><main><details><summary>Contents</summary><nav>'+''.join(f'<a href="#{a}">{t}</a>' for a,t in toc)+'</nav></details>'+body+'</main></html>')
    print(REPORT)

if __name__=='__main__':render()
