#!/usr/bin/env python3
"""Publish receipt-derived forensic results; never runs collection or strategy arms."""
import collections
import copy
import datetime as dt
import html
import statistics
from pathlib import Path

from cost_components import summarise
from replay_inventory import ROOT, read, save, sha, canonical, stamp
from rebuild_preflight import require, offline_process
from unified_replay_report import table, markdown_body

WORK = Path('/workspace/jiaoyi-cost-reservation-18372')
PLAN = ROOT/'docs/rebuild_rfc_260deals_analysis_plan_18372.json'
RESULT = ROOT/'docs/rebuild_rfc_260deals_and_clean_reservation_18372.json'
REPORT = ROOT/'doc/REBUILD_RFC_260DEALS_AND_RESERVATION_REPORT-18372-2026-11-08.md'
CODE = ['cost_components.py', 'calendar_authority.py', 'clean_reservation.py',
        'position_forensics.py', 'cost_reservation_report.py', 'test_cost_reservation.py', 'rebuild.py']


def num(value, places=4):
    return 'Not identifiable' if value is None else f'{float(value):,.{places}f}'


def pct(value):
    return num(100*value, 2)+'%'


def group_table(groups, kind):
    return table([kind, 'Positions', 'Wins', 'Win rate', 'Gross USD', 'Commission USD',
                  'Swap USD', 'Net USD', 'Booked drag USD', 'Median booked drag bp', 'UTC entry → exit range'],
        [[' / '.join(map(str, g['key'])), s['positions'], s['wins'], pct(s['win_rate']),
          *[num(s[k], 2) for k in ('gross_pnl_money', 'commission_money', 'swap_money',
                                   'net_pnl_money', 'recorded_fee_financing_drag_money')],
          num(s['median_recorded_fee_financing_drag_bps']), s['first_entry_utc']+' → '+s['last_exit_utc']]
         for g in groups for s in [g['summary']]])


def build():
    f = read(WORK/'forensics.json'); r = read(WORK/'reservation.json'); plan = read(PLAN)
    require(f['frozen_plan_sha256'] == sha(PLAN), 'forensic_plan_changed')
    require(r['transition']['policy_sha256'] == sha(PLAN), 'reservation_plan_changed')
    require(r['proof']['refusal_count'] == 0 and r['not_a_collection_permit'], 'reservation_claim')
    require(len(f['positions_271']) == 271 and f['summary_statement_matched260']['positions'] == 260,
            'published_denominators')
    previous = read(ROOT/'docs/rebuild_rfc_offline_gates_completion_18368.json')
    gates = copy.deepcopy(previous['gate_status_ledger'])
    changes = {
        'G5': ('BLOCKED_HISTORICAL_AUTHORITY',
            'Tier A parses captured current broker specs. Tier B verifies frozen native IANA sessions, decoded preparation and actual quote boundaries for 37 reused scenarios, with historical_iana_plus_current_broker_spec provenance. Requested proxy engineering policy passes; original historical-authority gate does not.',
            'Historical broker/venue early-close, maintenance, funding and contract terms remain unknown. Current 2026 snapshots do not backdate authority to 2023–2025.'),
        'G6': ('PASSED',
            '260 positions / 264 closing fills reconcile statement money, signed fees, swaps, conversion and closed quantity. All 264 dated observed lot conversions agree with the current snapshot. No extra spread/slippage debit is applied to actual P&L. Scope: these real 2026 account observations, not historical replay contracts.', None),
        'G7': ('PARTIAL_COMPONENT_CALIBRATION',
            'Date-ordered retrospective splits, held-out errors, daily-block diagnostics and sparse fallbacks exist for all four components. Two fee/financing and seven historical-spread buckets fit; entry and exit-reference buckets remain sparse. No full round-trip execution-cost model is asserted.',
            'Matched causal reference/dispatch quotes and accepted resting-order provenance for both legs, sufficient account-specific independent cost observations, and stable held-out errors. Retrospective component splits are not fresh independent alpha evidence.'),
        'G8': ('BLOCKED_EXTERNAL_EVIDENCE',
            '143 actual deal/recorded-level pairs have checked signed and absolute deviations. Source code shows closest-level inference and local fallback; these are not acknowledgement, accepted-level or trigger receipts.',
            'Actual send/accept/amend/cancel/fill/restart sequences and frozen-policy shadow decision parity.'),
        'G9': ('PASSED',
            'Owner-authorised reservation-only state transition preserves the candidate, registers 160 exact native-window jobs, allows only verified causal preparation reuse, and enforces unseen local evaluation intervals and a one-candidate/one-trial ceiling. Local planning refusal count is zero. Missing preparation still blocks the tick stage.', None)
    }
    for g in gates:
        if g['gate'] in changes:
            g['status'], g['evidence'], g['remaining_blocker'] = changes[g['gate']]
            g['code_and_tests'] = 'cost_components / calendar_authority / clean_reservation; test_cost_reservation (six tests); published receipt-derived ledgers'
    count = sum(g['status'] == 'PASSED' for g in gates)
    require(count == 6, 'gate_count')
    stamp_path = WORK/'report-stamp.json'
    if not stamp_path.exists():
        save(stamp_path, {'generated_at_sgt': dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat()})
    generated = read(stamp_path)['generated_at_sgt']
    compact = copy.deepcopy(r)
    for job in compact['episodes']:
        required = job.pop('required_m30_slots')
        runs = []
        for t in required:
            if runs and runs[-1]['to_ms'] == t:
                runs[-1]['to_ms'] += 1800000
            else:
                runs.append({'from_ms':t, 'to_ms':t+1800000})
        expanded = [t for run in runs for t in range(run['from_ms'],run['to_ms'],1800000)]
        require(expanded == required and canonical(expanded) == job['required_m30_slots_sha256'],
                'compact_preparation_slots_drift')
        job['required_m30_slot_runs'] = runs
        job['required_m30_slot_count'] = len(required)
    compact['slot_encoding'] = 'Half-open runs, step 1800000ms; expand and verify required_m30_slots_sha256 before use'
    # Keep source data in its original receipt once. The result contains compact
    # sufficient observations and exact plan slots, not copies of raw tick data.
    out = dict(f, schema='jiaoyi.rebuild-cost-reservation.v1', generated_at_sgt=generated,
        source_hashes=plan['source_hashes'], code_sha256={p:sha(ROOT/'ops/research'/p) for p in CODE},
        reservation=compact, gate_status_ledger=gates, rfc_gates_passed_before=4,
        rfc_gates_passed_now=count, rfc_gates_total=12,
        practical_gate5_proxy_policy='PASSED_FOR_37_REUSED_NATIVE_SCENARIOS_NOT_HISTORICAL_BROKER_AUTHORITY',
        goal_progress={'new_independent_trades':0, 'financial_pass':False,
            'account_goal':'At least 15 net wins of latest 20, per strategy/account, with reference 1:1.38 R:R; repeated independent evidence required',
            'strategy_returns_recomputed':False},
        verification={'focused_tests':6, 'research_suite_tests':105, 'research_suite_existing_skips':1,
            'offline_boundary':'Python audit + inherited Linux seccomp on analysis and reservation builders',
            'source_unchanged':'eligible local market receipts checked before and after reservation; frozen native/calendar metadata matches archived Phase4 engine',
            'ci':'Required check/docker/engineering receipts and merged file hashes will be appended to the PR after completion'},
        limitations=['All historical contract authority remains explicitly unknown.',
            'The 260-position P&L pool mixes accounts and some strategy/manual labels; it is not candidate qualification.',
            'Full execution cost is not gross minus net. Prices already embed spread and execution shortfall.',
            'Four reserved calendar weeks are not four independent regimes; three cluster in November 2024.',
            'The November 25 week contains US Thanksgiving/early-close dates; historical CFD hours remain unverified and may cause coverage refusal.',
            'Local retained collection/inspection ledger scope only; older unavailable non-ledger references are not silently certified.',
            'Planning success does not authorise collection, imply decoded preparation, or certify future executable paths.'])
    save(RESULT, out)
    s = f['summary_statement_matched260']; all_s = f['summary_quantity_complete271']
    entries = f['component_observations']['entry_dispatch_slippage']
    exits = f['component_observations']['exit_level_deviation_proxy']
    components = f['component_calibration']
    matched_exit_count = sum(v['in_statement_matched260'] for v in exits)
    observed_exits = collections.defaultdict(list)
    for v in exits: observed_exits[(v['account_id'], v['symbol'], v['purpose'])].append(v)
    exit_table = table(['Account', 'Symbol', 'Inferred level label', 'Fills', 'Median signed bp', 'Median absolute bp', 'Max absolute bp'],
        [[*key, len(v), num(statistics.median(float(x['value_bps']) for x in v)),
          num(statistics.median(float(x['absolute_deviation_bps']) for x in v)),
          num(max(float(x['absolute_deviation_bps']) for x in v))] for key,v in sorted(observed_exits.items())])
    fitted = [(name,m) for name,c in components.items() for m in c['models'] if m['status'] == 'COMPONENT_TEMPORAL_DIAGNOSTIC']
    fitted_table = table(['Component', 'Account / symbol', 'Train / held-out', 'Train mean bp', 'Held-out MAE bp', 'Bias bp', 'Daily blocks', 'Normal 95% half-width bp'],
        [[name, str(m['account_id'])+' / '+m['symbol'], str(len(m['training_ids']))+' / '+str(len(m['held_out_ids'])),
          num(m['frozen_training_mean_bps']), num(m['held_out_mae_bps']), num(m['held_out_bias_bps']),
          len(m['daily_error_means_bps']), num(m['normal_95_halfwidth_bps_diagnostic'])] for name,m in fitted])
    split_table = table(['Component', 'Observations', 'UTC cutoff', 'Train', 'Held-out', 'Boundary exclusions', 'Fitted buckets'],
        [[name, c['observations'], c['cutoff_utc'],
          *[sum(len(m[k]) for m in c['models']) for k in ('training_ids','held_out_ids','boundary_excluded_ids')],
          c['trained_buckets']] for name,c in components.items()])
    cost_table = table(['Symbol', 'Matched positions', 'Median booked fee/financing price units', 'Median booked fee/financing bp', 'Full execution-cost median bp'],
        [[g['key'][0],g['summary']['positions'],num(g['summary']['median_recorded_fee_financing_drag_price_units'],8),
          num(g['summary']['median_recorded_fee_financing_drag_bps']), 'Not identifiable'] for g in f['forensic_groups']['symbol']])
    weeks = table(['Week starts', 'Required cache slots reused', 'Missing distinct native slots within week', 'Episodes'],
        [[w['monday'],w['cached_unique_slots'],w['missing_unique_slots'],40] for w in r['selected_weeks']])
    disk = table(['Episode', 'Symbol', 'Date', 'Candidate already used', 'Proxy prep/boundaries', 'Side gaps >6s', 'Ambiguous side times', 'Observed path qualified'],
        [[e['episode_id'],e['symbol'],e['date'],e['used_by_candidate'],e['authority']['status'],
          e['observed_path_checks']['side_gaps_over6s'],e['observed_path_checks']['ambiguous_side_timestamps'],
          e['full_execution_path_qualified']] for e in r['on_disk_complete_native_scenarios']])
    text = f'''# Real-position costs and clean reservation — №18,372

№18,372 §§1–10 · {generated} · by Codex (GPT-6), effort not exposed · base PR #198 / `{plan['base_commit']}`.

## §1. TLDR and progress towards the goal

**The 260 statement-matched positions lost ${num(-float(s['net_pnl_money']),2)} net, with {s['wins']}/{s['positions']} net wins ({pct(s['win_rate'])}).** Gross fill-to-fill P&L was ${num(s['gross_pnl_money'],2)}. Recorded commission and financing reduced it by a further ${num(s['recorded_fee_financing_drag_money'],2)}. The gross loss already existed; removing those booked charges would not repair it. Spread and slippage can already be embedded in gross P&L, so this does not show that all execution costs were small.

**Engineering gates advance from 4/12 to 6/12:** G6 passes for the 260 reconciled actual positions, and G9 passes for the explicitly versioned local pre-collection reservation. The requested current/historical-proxy verifier is implemented, but original G5 historical broker authority remains unproved. G7 has honest component calibration, not a complete round-trip execution-cost model.

**Reservation:** {r['summary']['planned_episodes']} episodes in four calendar weeks; zero planning refusals; {r['summary']['reused_verified_m30_segments']} verified M30 source files reused; {r['summary']['new_m30_segments_needed']} bounded M30 request intervals and {r['summary']['new_tick_episodes_needed']} tick episodes still needed. Nothing was collected. Tick collection correctly remains refused until preparation is decoded.

**Goal progress:** zero new independent trades and no new strategy-return experiment. The project target remains at least 15 net wins in each strategy/account's latest 20 trades (75%, the first attainable count at or above 73%), with reference 1:1.38 R:R. The pooled historic win rate below is descriptive, not a strategy/account qualification result. Financial pass remains false; validation remains sealed; live rules are unchanged.

## §2. Denominators, dates and exact money reconciliation

These are positions, not 271 independent deal messages. The 271 quantity-complete position lifecycles include partial closes and real netting reversals. Exactly 260 have every closing fill matched to retained statements: 264 closing fills. All 264 dated lot conversions inferred from statement lots and broker closed native quantity agree with current contract lot sizes. This establishes the observed quantity conversion at those trades; it does not certify all historic contract terms.

{table(['Evidence scope','Positions','Wins','Win rate','Gross USD','Commission USD','Swap USD','Net USD'], [[label,x['positions'],x['wins'],pct(x['win_rate']),*[num(x[k],2) for k in ('gross_pnl_money','commission_money','swap_money','net_pnl_money')]] for label,x in [('All quantity-complete',all_s),('Fully statement matched',s),('Other complete, not fully statement matched',f['summary_other11_quantity_complete'])]])}

Matched data range: first entry `{s['first_entry_utc']}`, last exit `{s['last_exit_utc']}`. The all-271 ledger extends through `{all_s['last_exit_utc']}`. Money is USD as recorded in the retained account statements. Demo dollars and live dollars are shown separately; their descriptive sum is not the owner's investable portfolio return. The additional 11 positions are retained rather than silently discarded or certified as statement-matched.

{group_table(f['forensic_groups']['account'],'Statement-matched account')}

{group_table(f['forensic_groups_all271']['account'],'All quantity-complete account')}

### Matched positions by symbol

{group_table(f['forensic_groups']['symbol'],'Symbol')}

### All 271 quantity-complete positions by symbol

{group_table(f['forensic_groups_all271']['symbol'],'Symbol')}

Actual money uses Decimal broker moneyDigits, signed swap/conversion and the broker closing commission total once. It does not charge opening commission again or append hypothetical spread/slippage to already realised broker net P&L. Gross + commission + swap + conversion fee = net is checked. Conversion fees are zero in this retained set; that is an observation, not a permanent fee assumption.

## §3. What the cost figures mean

`Booked fee/financing drag = gross − net`. Per-position price drag divides each closing money charge by its recorded quote/deposit conversion and then by total native units. Basis points divide total money drag by the sum of closing-reference entry price × closed native units × recorded conversion. Credits produce negative drag. This fixed denominator is a descriptive normalisation, not the margin, account equity, R-risk, or an independent midpoint benchmark.

Price-unit medians are shown only within a symbol. Money sums can be aggregated; incompatible price units cannot. Full round-trip execution cost remains null, not zero, because matching causal benchmarks for both legs and accepted resting-order provenance are absent. A zero median booked fee does not imply zero trading cost.

{cost_table}

The largest losses in this matched set include ETHUSD (0/64 wins, −$5,186.35), BTCUSD (2/42, −$3,994.41), NatGas (0/16, −$2,625.20) and JPN225 (0/17, −$2,567.92). These are observed outcomes, not causal proof of one defective signal. Strategy labels include S3, S1 and unlabelled/mixed positions; this is not a backtest of the frozen macro-pullback candidate. No reverse-direction edge or profitable repair is inferred from these losses.

## §4. Entry and resting-level exit evidence

All 17 captured entry dispatch observations are retained below, with account and symbol attribution. They are not automatically 17 of the 260 positions: {sum(v['in_statement_matched260'] for v in entries)} overlap that matched position set. Positive signed entry deviation is adverse under the dispatch-side convention; negative is favourable. Entry deviation alone is not round-trip cost.

{table(['Account','Symbol','Position','UTC observation','Side','Signed bp','Dispatch spread price units','In matched260'], [[v['account_id'],v['symbol'],v['position_id'],stamp(v['at_ms']),v['purpose'],num(v['value_bps']),num(v['dispatch_spread'],8),v['in_statement_matched260']] for v in sorted(entries,key=lambda x:x['at_ms'])])}

**Exit reference found:** {len(exits)} deal/recorded-level pairs, of which {matched_exit_count} belong to the matched260 positions. {f['exit_reference_exclusions'].get('no_recorded_level_or_closing_detail',0)} broker-exit rows lack usable recorded-level/close detail and remain explicit exclusions. The stored row's fill and volume are checked against its raw cTrader deal. Absolute `|fill − recorded level|` and signed exit-side deviation are computed; adverse is positive. Full precision observations and raw deal/order hashes are in JSON.

**Critical provenance finding:** `fills.rs:exit_level` selects the nearer stored SL/TP, while `strategy3.rs:broker_exit` can fall back from report fields to local owned-position levels. Consequently “stop_loss” or “take_profit” here is an inferred nearest-level label. It is not proof that a broker resting order triggered, that the level was the last broker-accepted amendment, or that the deviation is pure execution slippage. Server-triggered exits need accepted order/amendment history and trigger/fill context; inventing a client dispatch timestamp would be wrong.

{exit_table}

## §5. Frozen temporal calibration and held-out verification

The plan was frozen before calculation with SHA256 `{sha(PLAN)}`. For each component, the earliest 70% of distinct observation dates form training; remaining dates form a chronological holdout. Fee positions crossing the cutoff are explicitly boundary-excluded. Each account/environment/symbol/purpose bucket is separate. No demo estimate is borrowed by live. At least three training observations and two held-out observations are required; otherwise there is no fitted prediction. The only fitted estimator is the training arithmetic mean, fixed without inspecting holdout values.

{split_table}

{fitted_table}

The nine fitted component buckets comprise two BTCUSD demo fee/financing buckets and seven historical spread buckets, all on their own account. There are no fitted dispatch or exit-reference buckets at the declared minimum sample sizes. All sparse buckets, observation IDs, cutoff exclusions, frozen predictions and individual holdout errors are retained in JSON. The historical spreads come from 1,710 deduplicated causal quote checkpoints, not the 2026 execution events.

Uncertainty is reported as daily block errors plus a normal 95% half-width around daily mean error where at least two days exist. These are diagnostics: blocks may be dependent; some have only two days; zero error on two zero-fee observations is weak evidence. These records were already inspected in earlier work, so the temporal split is retrospective, not a pristine independent evaluation. No calibrated component is silently inserted into live costs or historical strategy admission. The assumed 2/3.74/24 bp scenarios retain their prior labels.

## §6. Two authority tiers and complete on-disk scenarios

**Tier A — current authority:** parsed authenticated account/symbol snapshots, raw hashes, broker IANA schedule zone, current contract fields and captured holiday records. Their authority is at capture time. Display precision is not independent minimum-tick-size evidence.

**Tier B — historical replay proxy:** `historical_iana_plus_current_broker_spec`. Exact frozen native-calendar metadata is compared with the archived Phase4 engine, including US index 09:30 New York DST behaviour. Required native M30 timestamps and fresh opening/closing quote boundaries must exist; missing bars are not manufactured or declared holidays. London DST conversion is covered by the existing verifier fixtures. Unknown historical broker holidays, maintenance, funding and contract multipliers stay unknown. The current snapshot is not backdated.

The requested practical proxy policy passes for the 37 reused native-complete scenarios. This does not turn the original RFC G5 requirement for dated authoritative examples into a pass. Any future promotion requiring that authority remains blocked. The candidate, native sessions, risk geometry and cost assumptions are unchanged.

{len(r['on_disk_complete_native_scenarios'])} on-disk scenarios have complete native preparation and fresh tick boundaries; all were already used by the candidate. **Untouched complete scenarios: {r['untouched_complete_tier1_episodes_already_on_disk']}.** Additional explicit side-gap, same-timestamp price ambiguity and closing coverage checks qualify {r['full_execution_path_qualified_on_disk']} under the existing observed-path criterion (US500 9–10 April 2025). A >6-second update gap may reflect market inactivity; it remains insufficient evidence for full quote-path reconstruction under this declared guard. Neither qualified reused path supplies a new independent trade or proves broker execution parity.

{disk}

## §7. Versioned reservation with zero local planning refusals

The deterministic search covers {r['search']['symbol_date_definitions']} symbol/date definitions from {r['search']['from']} to before {r['search']['to_exclusive']}. It excludes the sealed 19 August–5 September 2025 interval and protected preparation dependencies, all locally inspected evaluation-price intervals, failed/spent missing preparation, and development-label embargo overlaps. {r['search']['clean_complete_weeks']} full eight-symbol weeks survive. Selection uses minimum distinct missing preparation slots, then date, with no new price outcomes inspected and no optimisation of trade returns.

{weeks}

These are non-overlapping calendar weeks, not four independent regimes. Three November weeks are adjacent. Their causal warm-up may include earlier selected weeks. The entire candidate/batch must remain frozen: no refitting, selection or adaptive stopping after seeing any week's returns. The final November week contains Thanksgiving/early-close dates; historical CFD dealing hours remain unverified. If later collection fails coverage, preserve the failure; do not silently switch dates or count an empty path as a success.

The owner-authorised transition changes only reservation policy: `{plan['reservation']['original_state']}` → `{plan['reservation']['state']}`. Verified causal preparation may be reused, while evaluation intervals must be unseen in the retained registry and missing preparation must be unspent. The exact 160 jobs and source hashes are frozen, with one candidate and at most one trial. The old strict guard and collection pause are preserved. This is a new registered plan, not a relabelling of the previously refused reservation.

{table(['Item','Measured/planned count'], [[k,v] for k,v in r['summary'].items()])}

Reused count means 45 distinct verified source files, not 45 job references. The 332 prospective M30 requests deduplicate overlapping dependencies, span at most seven days each, and use interval-only mode without count=500. Request intervals are half-open; any future broker adapter must preserve the exact boundary semantics. All 160 tick windows still need collection. Exact episode slots are encoded as half-open 30-minute runs with counts and SHA256; expansion is verified against the full native slot list. Deduplicated request intervals, reused source paths/hashes and the transition/proof are in JSON.

**State distinction:** preparation planning passes; tick-stage refusal is `{r['ticks_stage_refusal']}`; evaluation-ready is false; collection authority is false. A broker may return insufficient history or a holiday path may fail. The reservation certifies local provenance and request partitioning, not future data availability. Older unavailable non-ledger references remain a scope limitation; no claim is made about unknown external research performed elsewhere.

## §8. All twelve original RFC gates

{table(['Gate','Original acceptance condition','Status','Evidence / remaining requirement'], [[g['gate'],g['name'],g['status'],g['evidence']+(' Remaining: '+g['remaining_blocker'] if g['remaining_blocker'] else '')] for g in gates])}

G6's pass is deliberately about the 260 observed account positions. Missing historical contract authority belongs to G5 and missing full expected-cost estimation to G7; they do not invalidate actual reconciled statement arithmetic. G9 passes the local pre-evaluation split/trial audit, while data capture, path qualification and financial tests remain separate barriers. CI success is not a financial pass.

## §9. Verification, failure prevention and next bounded step

Six focused regression tests cover money/quantity versus full-cost distinctions; signed exit-level joins; held-out contamination and account separation; proxy calendar boundaries; spent-gap versus verified-prep reuse; and registered-transition/immutable-job/trial protection. The complete research suite passes 105 tests with one existing skip. Frozen Phase4 metadata and engine receipts are checked, and eligible local market source hashes are rechecked after the run. Analysis processes enforce an inherited Linux seccomp network denial as well as Python socket denial. No broker call, order, new backtest result, protected-date read or live-rule edit occurs here.

Reproduce with the canonical offline CLI: `python3 ops/research/rebuild.py cost-reservation`. It requires the retained local receipt files and archived native executable; absence is a refusal, never a fabricated replacement. It calculates the forensic ledger, verifies the reservation and renders this report without invoking collection or a new strategy arm.

The next steps are finite and evidence-dependent:

- Review this cost decomposition and reservation. Do not lower live costs or declare the historical proxy to be broker-certified.
- If price collection is later authorised, activate a reviewed collection state for this exact reservation; collect/decode all missing M30 preparation before ticks. Fail closed on any missing/spent or holiday-conflicting dependency.
- Collect all reserved tick windows as one frozen batch, then verify boundaries, side ages, ambiguity and full holding coverage before replay. Do not replace failed dates after looking at returns.
- Capture accepted bracket/amendment/trigger context and full causal execution benchmarks through the bounded evidence path before promoting exit-level proxies into a cost model or claiming G8 parity.
- Run one unchanged candidate against its matched controls, report all failures and account/class uncertainty, and require financial/shadow/lifecycle gates before any live migration. More trades or a lower assumed charge is not proof of an edge.

## §10. Audit references and account/symbol appendix

Primary sources are the retained authenticated broker deal/symbol receipts and statement matches from PR #198, the Phase1 checkpoint manifest, the 544-definition/616-M30 unified inventory, the Phase4 frozen candidate and the spent registry. Result SHA256: `{sha(RESULT)}`. Frozen analysis plan: [JSON](../docs/rebuild_rfc_260deals_analysis_plan_18372.json). Full ledger, all component observations/predictions, exact reservation and source hashes: [result JSON](../docs/rebuild_rfc_260deals_and_clean_reservation_18372.json).

{table(['Input','SHA256'], [[k,v] for k,v in plan['source_hashes'].items()])}

### Matched260 account × symbol

{group_table(f['forensic_groups']['account_symbol'],'Account / symbol')}

### All271 account × symbol

{group_table(f['forensic_groups_all271']['account_symbol'],'Account / symbol')}

### Strategy labels within matched260

{group_table(f['forensic_groups']['account_strategy'],'Account / observed labels')}

The JSON contains the full 271-position ledger, not merely these aggregates. No ZIP, duplicate tick archive, new service or paid model route is introduced.
'''
    REPORT.write_text(text)
    body, toc = markdown_body(text)
    nav = ''.join(f'<a href="#{a}">{label}</a>' for a,label in toc)
    style = '''body{margin:0;background:#f3f5f7;color:#152536;font:16px/1.65 system-ui,sans-serif}main{max-width:1180px;margin:auto;padding:36px 24px}h1{font-size:2rem;line-height:1.2}h2{margin-top:2.4rem;border-top:1px solid #c6d0dc;padding-top:1.2rem}h3{margin-top:1.6rem}p,li{max-width:95ch}a{color:#064bb6}code{font-size:.88em;overflow-wrap:anywhere;background:#e7edf5;padding:2px 4px}nav{display:flex;flex-wrap:wrap;gap:8px 16px;background:#fff;border:1px solid #ccd5df;padding:16px;border-radius:10px}nav a{font-size:.9rem}.table-scroll{overflow:auto;max-height:640px;border:1px solid #ccd5df;background:white;border-radius:8px;margin:20px 0}table{border-collapse:collapse;min-width:680px;width:100%;font-size:.87rem}th,td{text-align:left;padding:10px 12px;border-bottom:1px solid #dfe5ec;vertical-align:top;max-width:500px;overflow-wrap:anywhere}th{background:#e8eef6;position:sticky;top:0}tr:nth-child(even){background:#f8fafc}pre{overflow:auto}strong{color:#102c4e}@media(max-width:600px){main{padding:20px 14px}h1{font-size:1.55rem}body{font-size:15px}nav{display:block}nav a{display:block;margin:6px 0}.table-scroll{max-height:480px}}'''
    contents = '<details><summary>Contents — ten report sections</summary><nav aria-label="Report sections">'+nav+'</nav></details>'
    body = body.replace('<h2 id="section-2">', contents+'<h2 id="section-2">', 1)
    REPORT.with_suffix('.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi №18,372 — costs and reservation</title><style>'+style+'</style><main>'+body+'</main></html>')
    return out


if __name__ == '__main__':
    offline_process()
    result = build()
    print('Written', RESULT, REPORT, 'gates', result['rfc_gates_passed_now'])
