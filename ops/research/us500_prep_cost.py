#!/usr/bin/env python3
"""№18,346: offline US500 diagnosis and matched-fill ledgers, never cost fitting."""
import argparse
import datetime as dt
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import time

from run_repaired_offline_replay import ci_state, sgt, without_notes

ROOT = Path(__file__).resolve().parents[2]
BRANCH = 'diag/us500-prep-and-cost-calibration-18346'
PLAN = 'docs/us500-prep-cost-plan-18346.json'
RESULT = 'docs/us500_prep_and_cost_calibration_18346.json'
REPORT = 'doc/US500_PREP_AND_COST_CALIBRATION_REPORT-18346-2026-11-07.md'
BASE = 'docs/offline-repaired-engine-results-18331.json'
MAY = 'docs/clean_peer_collection_results_18338.json'
JAN = 'docs/jan2024_peer_parity_results_18342.json'
COST = 'docs/oil-cost-and-replication-2026-11-06/cost-assessment.json'
AMENDMENT = 'docs/seasonal-collector-amendment-18316.json'
CORPUS = Path('/workspace/jiaoyi-three-year-pilot-18325')
OLD = Path('/workspace/jiaoyi-seasonal-stage1-18316')
RATES = dict(US500=2.0, NAS100=2.0, XAUUSD=2.0, XAGUSD=2.0,
             SpotCrude=3.74, SpotBrent=3.74, BTCUSD=2.5, ETHUSD=2.5)
CODE = ['ops/research/us500_prep_cost.py', 'ops/research/test_us500_prep_cost.py',
        'ops/research/run_us500_prep_and_cost_calibration.sh', 'ops/research/seasonal_pilot.py']


def read(path): return json.loads(Path(path).read_text())
def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def save(path, value): Path(path).write_text(json.dumps(value, indent=2, allow_nan=False)+'\n')
def cmd(*args): return subprocess.check_output(args, cwd=ROOT, text=True, timeout=120).strip()
def gh(*args): return json.loads(cmd('gh', *args))


def journal(section, message):
    path = ROOT/'log/CODEX_Log.txt'; text = path.read_text(); tag = f'№18,346 {section}'
    if tag in text: return
    n = max(map(int, re.findall(r'\[§LOG\.(\d+)\]', text)), default=0)+1
    path.write_text(text+f'\n[§LOG.{n}] {tag} · {sgt()}; by <Codex (GPT-6), effort: not exposed>.\n'+message+'\n')


def opening_coverage(bars, start_ms):
    """Require the six actual opening bars, not a blanket 45-day bar count."""
    valid = {b['ts_ms'] for b in bars if b['volume'] > 0}
    missing = [start_ms+i*1800000 for i in range(6) if start_ms+i*1800000 not in valid]
    return dict(required_bars=6, missing_ts_ms=missing, complete=not missing)


def matched_trades(groups, model):
    unique = {}; duplicates = 0
    for trace, rows in groups:
        for row in rows:
            m = row.get('models', {}).get(model, {}); trade = m.get('study', {}).get('trade')
            if not trade: continue
            entry = trade['entry']; outcome = trade['exit_comparison']['candidate']
            key = (row['symbol'], entry['entered_at'], entry['side'])
            identity = without_notes(dict(entry=entry, outcome=outcome,
                                           primary_net=m['sensitivities']['trailing']['24']))
            if key in unique:
                if unique[key]['identity'] != identity: raise ValueError('conflicting trade identity')
                unique[key]['traces'].append(trace); duplicates += 1
                unique[key]['protocol_flags'] = sorted(set(unique[key]['protocol_flags']) | set(row.get('protocol_flags', [])))
                continue
            unique[key] = dict(symbol=row['symbol'], key=row['key'], identity=identity,
                               traces=[trace], protocol_flags=row.get('protocol_flags', []))
    return list(unique.values()), duplicates


def net_on_fills(record, bps):
    if not math.isfinite(bps) or bps < 0: raise ValueError('invalid assumed cost')
    e = record['identity']['entry']; o = record['identity']['outcome']
    if o['remaining_volume']:
        if o['scenario_gross_r'] is not None or record['identity']['primary_net'] is not None:
            raise ValueError('censored trade counted closed')
        return None
    if e['volume'] <= 0 or e['initial_r'] <= 0 or sum(f['volume'] for f in o['fills']) != e['volume']:
        raise ValueError('invalid risk or volume conservation')
    average = sum(f['price']*f['volume'] for f in o['fills'])/e['volume']
    sign = 1 if e['side'] == 'long' else -1
    gross = sign*(average-e['fill'])/e['initial_r']
    if not math.isclose(gross, o['scenario_gross_r'], abs_tol=1e-8, rel_tol=1e-8):
        raise ValueError('gross accounting mismatch')
    value = gross-(e['fill']+average)*bps/10000/e['initial_r']
    if not math.isfinite(value): raise ValueError('nonfinite outcome')
    if bps == 24 and not math.isclose(value, record['identity']['primary_net'], abs_tol=1e-8, rel_tol=1e-8):
        raise ValueError('primary native net accounting mismatch')
    return value


def stats(values, entries):
    values = [v for v in values if v is not None]; n = len(values)
    profit = sum(v for v in values if v > 0); loss = -sum(v for v in values if v < 0)
    return dict(entries=entries, closed_trades=n, wins=sum(v > 0 for v in values),
                losses=sum(v < 0 for v in values), net_breakevens=sum(v == 0 for v in values),
                censored=entries-n, win_rate=sum(v > 0 for v in values)/n if n else None,
                mean_net_r=statistics.mean(values) if n else None,
                profit_factor_r=profit/loss if loss else None,
                financial_pass=False, account_goal_73pct_20trade_satisfied=False)


def ledgers(records):
    rows = []
    for r in records:
        bps = RATES[r['symbol']]
        rows.append(dict(symbol=r['symbol'], key=r['key'], traces=r['traces'],
                         protocol_flags=r['protocol_flags'], entry=r['identity']['entry'],
                         outcome=r['identity']['outcome'], ledger_a_net_r=net_on_fills(r, 24),
                         ledger_b_assumed_bps_per_side=bps, ledger_b_net_r=net_on_fills(r, bps)))
    return dict(ledger_a_stress_24bp=stats([r['ledger_a_net_r'] for r in rows], len(rows)),
                ledger_b_unverified_cost_sensitivity=stats([r['ledger_b_net_r'] for r in rows], len(rows)),
                entry_and_exit_paths_identical=True, admission_reserve_bps_per_side=24,
                new_entries=0, rows=rows)


def prepare():
    target = ROOT/PLAN
    if target.exists(): return read(target)
    public = [BASE, MAY, JAN, COST, AMENDMENT, 'docs/three-year-execution-18324.json',
              'docs/seasonal-execution-18315.json']
    raw = [CORPUS/('job-'+r['key']+'.json') for r in read(ROOT/BASE)['rows'] if r['symbol'] == 'US500']
    raw += list(OLD.glob('bar-2024_Q1_Winter-US500-*.json'))+[OLD/'job-US500-2024-01-24.json', OLD/'freeze.json']
    p = dict(trace='№18,346 §§1–6', frozen_at_sgt=sgt(), by='Codex (GPT-6), effort not exposed',
             source_sha256={str(p):sha(p) for p in [ROOT/x for x in public]+raw},
             code_sha256={x:sha(ROOT/x) for x in CODE+['ops/research/run_repaired_offline_replay.py']},
             proposed_rates_bps_per_side=RATES, rate_status='unverified sensitivity assumptions, not calibrated costs',
             scope='Audit 30 retained US500 slots and eight failed January M30 components; deduplicated S3 union of 18331/18338/18342 and matched January macro only.',
             protocol='Reprice identical native SL/TP/two-stage paths. Keep 24bp admission, all prior losses, duplicates and censoring. No new entry search or rule tuning.',
             limits=dict(new_broker_requests=0, database_reads=0, paid_model_calls=0, restored_source_rows=0),
             validation_opened=False, production_changed=False,
             outcome='Measured results only. Restored episodes require actual source bars; proposed 1400-count rule is not applied.')
    attachments = [Path('/workspace/attachments/c5a1ddb1-b4ea-4d4f-8135-73d45c839672/run_us500_prep_and_cost_calibration.bash'),
                   Path('/workspace/attachments/bd73738d-6427-4e96-a390-770bf919160e/Pasted text.txt')]
    p['attachment_sha256'] = {x.name:sha(x) for x in attachments}
    save(target, p)
    journal('§§1–6', 'Frozen offline audit and matched-fill ledgers; zero new broker reads. Lower rates are assumptions.\nRetain January source failures and previous losses; no 1400-bar workaround, fabricated 14 restorations or fixed 75% result.')
    return p


def verify(p):
    for name, expected in p['source_sha256'].items():
        if sha(name) != expected: raise ValueError('source drift: '+name)
    for name, expected in p['code_sha256'].items():
        if sha(ROOT/name) != expected or hashlib.sha256(subprocess.check_output(['git','show','HEAD:'+name], cwd=ROOT)).hexdigest() != expected:
            raise ValueError('commit frozen code before evaluation: '+name)


def audit_prep(p):
    audit = []; failures = []
    for name in p['source_sha256']:
        path = Path(name)
        if path.parent == OLD and path.name.startswith('bar-'):
            value = read(path); failures.append(dict(file=path.name, status=value['status'], error=value.get('error'), sha256=sha(path)))
        if path.parent != CORPUS: continue
        value = read(path); row = dict(key=path.stem.removeprefix('job-'), status=value['status'], sha256=sha(path))
        if value['status'] != 'complete': row['error'] = value['error']
        else:
            bars = value['payload']['m30']; req = value['request']
            start = int(dt.datetime.fromisoformat(req['from'].replace('Z','+00:00')).timestamp()*1000)
            row.update(m30_bars=len(bars), positive_volume_bars=sum(b['volume'] > 0 for b in bars),
                       failed_components=value['bar_component_failures'], session_open_utc=req['from'],
                       opening_range_180m=opening_coverage(bars, start))
        audit.append(row)
    amendment = read(ROOT/AMENDMENT); old_commit = read(OLD/'freeze.json')['commit']
    old_code = cmd('git','show',old_commit+':ops/research/seasonal_pilot.py')
    return dict(slots=audit, failed_january_components=failures,
                missing_primary_slots=sum(x['status'] != 'complete' for x in audit),
                empty_m30_slots=sum(x.get('m30_bars') == 0 for x in audit),
                opening_range_complete_slots=sum(x.get('opening_range_180m', {}).get('complete', False) for x in audit),
                nonempty_sources_below_proposed_1400=[x['key'] for x in audit if 0 < x.get('positive_volume_bars',0) < 1400],
                old_request_included_count_500='count=500' in old_code,
                existing_transport_fix_commit='67d21e6', prior_transport_diagnostic=amendment['diagnostic_evidence'],
                recorded_failure='bar_identity_or_truncation_unverified; interrupted tick job retained',
                exact_us500_response_predicate='Not Verifiable: original rejected response body was not retained',
                calendar_2160_requirement_found=False, restored_episodes=0,
                current_change='Preserve distinct period, symbol, truncation and missing-completeness error codes; accepted data rules unchanged.',
                data_repair_status='Not recoverable offline from retained failed responses; no fabricated M30 bars')


def cost_evidence():
    evidence = read(ROOT/COST); entries = evidence['attributed_entries']; summary = {}
    for symbol in RATES:
        rows = [r for r in entries if r['symbol'] == symbol]
        slips = []
        for r in rows:
            e = r['entry_cost']; sign = 1 if r['side'] == 'buy' else -1
            slip = sign*(e['fill_price']-e['dispatch_price'])/e['dispatch_price']*10000
            if not math.isclose(slip, e['dispatch_to_fill_bps'], abs_tol=1e-8): raise ValueError('entry cost mismatch')
            slips.append(slip)
        summary[symbol] = dict(attributed_entries=len(rows), live_attributed_entries=sum(r['environment']=='live' for r in rows),
                               min_entry_slippage_bps=min(slips) if slips else None,
                               max_entry_slippage_bps=max(slips) if slips else None,
                               proposed_bps_per_side=RATES[symbol], calibrated=False)
    return dict(source=COST, sha256=sha(ROOT/COST), per_symbol=summary,
                retained_total_attributed_entries=len(entries), source_newest_order_at_utc=evidence['newest_order_at_utc'],
                verified_round_trip_samples=evidence['verified_round_trip_samples_in_this_review'],
                calibrated_per_side_bps=None, status='NOT_VERIFIABLE', qualifications=evidence['qualifications'],
                warning='Entry slippage is not spread, round-trip slippage, settled commissions or swaps. Demo 2026 entries do not calibrate historical 2023–2025/live costs.')


def run(p, out):
    verify(p)
    groups = [(name, read(ROOT/name)['rows']) for name in [BASE, MAY, JAN]]
    trades, duplicate = matched_trades(groups, 's3_two_stage')
    macro, _ = matched_trades([groups[-1]], 'macro_180m_two_stage')
    result = dict(trace='№18,346', generated_at_sgt=sgt(), freeze_commit=cmd('git','rev-parse','HEAD'),
                  plan_sha256=sha(ROOT/PLAN), us500_preparation_audit=audit_prep(p), cost_evidence=cost_evidence(),
                  s3_two_stage=ledgers(trades), january_macro=ledgers(macro), duplicate_s3_observations_removed=duplicate,
                  new_broker_requests=0, new_native_replays=0,
                  execution='Independent accounting over existing native full SL/TP/two-stage fills; not a new strategy/admission replay.',
                  financial_pass=False, validation_opened=False, production_changed=False, stage2_authorized=False)
    old = read(ROOT/JAN)['cumulative_s3']['summary']; current = result['s3_two_stage']['ledger_a_stress_24bp']
    if any(current[k] != old[k] for k in ['closed_trades','wins','losses']): raise ValueError('baseline population changed')
    if not math.isclose(current['mean_net_r'], old['mean_net_r'], abs_tol=1e-8): raise ValueError('baseline net drift')
    save(ROOT/RESULT, result); write_report(result)
    journal('§§7–10', 'Completed offline audit; 0 restored US500 episodes. Distinct decoder errors added without weakening validation.\nS3 ledger A: '+json.dumps(current)+'\nS3 ledger B (unverified sensitivity, same fills): '+json.dumps(result['s3_two_stage']['ledger_b_unverified_cost_sensitivity'])+'\nNo empirical round-trip calibration, financial PASS, new entries, broker reads or live change.')
    verify(p); save(out/'result-receipt.json', dict(result_sha256=sha(ROOT/RESULT), report_sha256=sha(ROOT/REPORT)))
    print(json.dumps({k:result[k] for k in ['new_broker_requests','duplicate_s3_observations_removed','financial_pass']}), flush=True)


def fmt(v): return 'unavailable' if v is None else f'{v:.6f}'


def write_report(r):
    lines = ['# US500 preparation audit and matched-fill cost ledgers', '',
             f'№18,346 §§1–10 · {r["generated_at_sgt"]} · by Codex (GPT-6), effort not exposed.', '',
             '## §1. TLDR', '',
             '**No US500 episodes were restored. The proposed costs are not empirically calibrated.** The submitted script wrote predetermined profits without changing a validator or running a backtest. This corrected offline run uses retained evidence and actual native fills. It makes no financial PASS or live change.', '',
             '| Evaluation | Ledger | Closes | Wins | Mean net R | PF(R) |',
             '| --- | --- | ---: | ---: | ---: | ---: |']
    for label, key in [('Cumulative S3','s3_two_stage'), ('January macro','january_macro')]:
        for name, ledger in [('A: 24 bp/side','ledger_a_stress_24bp'), ('B: proposed rates, unverified','ledger_b_unverified_cost_sensitivity')]:
            s = r[key][ledger]
            lines.append(f'| {label} | {name} | {s["closed_trades"]} | {s["wins"]} | {fmt(s["mean_net_r"])} | {fmt(s["profit_factor_r"])} |')
    a = r['us500_preparation_audit']; s = r['s3_two_stage']['ledger_a_stress_24bp']
    lines += ['', f'**Goal progress: {s["closed_trades"]}/20 unique closed research trades**, with {s["censored"]} censored entries. One repeated January winner is deduplicated; previous losses remain. The original **73% net wins / reference risk:reward 1:1.38 over the latest 20 trades for each strategy on each account** is unproven. Pooled research results and the interim 50% target do not qualify an account.', '',
              '## §2. What actually failed', '',
              f'The 30 retained US500 slots contain {a["missing_primary_slots"]} failed primary, {a["empty_m30_slots"]} complete tick files with zero M30 bars, and {a["opening_range_complete_slots"]} slots with all six positive-volume opening M30 bars. Eight January M30 components retain `bar_identity_or_truncation_unverified`; 24 January retains `interrupted_reserved_job_no_retry` in its original receipt.', '',
              'The original collector sent `count=500` together with interval bounds. The documented diagnostic returned 500 rows with `hasMore=true`, versus 326 with `hasMore=false` in interval mode. PR #179 (`67d21e6`) already removed count mode. That diagnostic used BTC components: the discarded US500 responses do not reveal which identity/truncation predicate failed. The recorded failure is established; the exact US500 broker-response field remains **Not Verifiable**.', '',
              'No validator requiring 2,160 continuous M30 bars was found. Current code validates each returned bar and requires specific opening/prior-session bars; weekend gaps between unrelated bars are allowed. US500 cash-open alignment already uses 09:30 America/New_York (14:30 UTC in January, 13:30 UTC in July). Do not encode a fixed UTC maintenance schedule across DST without dated broker evidence.', '',
              'A blanket minimum of 1,400 bars cannot establish session completeness. It would also reject these nonempty sources: '+', '.join(a['nonempty_sources_below_proposed_1400'])+'. A large unrelated bar count cannot replace missing opening bars.', '',
              'The implemented change splits the ambiguous decoder error into period, symbol, truncation and missing-completeness errors while preserving its acceptance rules. Existing failed files remain immutable. No bars are bridged, imputed, refetched or manufactured.', '',
              '## §3. Cost evidence and assumptions', '',
              'The retained November 2026 audit has 13 measured demo entries, zero attributed live entries and **zero verified round-trip cost samples**. These are partial entry diagnostics, not sufficient execution, fee and swap calibration for historical 2023–2025 trading.', '',
              '| Symbol | Proposed bp/side | Measured entries | Entry slippage range (bp) | Calibrated? |',
              '| --- | ---: | ---: | --- | --- |']
    for symbol, e in r['cost_evidence']['per_symbol'].items():
        lines.append(f'| {symbol} | {e["proposed_bps_per_side"]} | {e["attributed_entries"]} | {fmt(e["min_entry_slippage_bps"])} to {fmt(e["max_entry_slippage_bps"])} | No |')
    lines += ['', 'Ledger A retains 24 bp per side. Ledger B uses the supplied 2 bp indices/metals, 3.74 bp oils and 2.5 bp crypto as **unverified sensitivity assumptions**. Both retain exactly the same entries, original risk, fills, SL/TP/two-stage amendments, deadlines and censoring. Admission remains frozen at 24 bp; lower rates do not admit new trades. No new native strategy replay is claimed.', '',
              '`net R = signed(exit VWAP − entry fill) / original risk − (entry fill + exit VWAP) × bp/10,000 / original risk`. Executable bid/ask already incorporates spread; it is not charged again. Additional commission is zero in this sensitivity because the inherited experiment assumed zero, not because actual commission/swap was verified as zero. PF is calculated from equal-R trade results, not dollar account returns.', '',
              'The supplied “0.60 bp live spread”, “80× hurdle”, restored 14 episodes, 8/12 and 18/24 win claims are unsupported by this run. Reducing assumed cost can change net arithmetic; it cannot establish a measured edge or restore missing observations.', '',
              '## §4. Validation and next action', '',
              'Three focused tests cover session-gap/decoder behaviour, irrelevant bar counts versus exact opening slots, and matched-fill costs/censoring/deduplication. All primary ledger values are independently checked against native results. Frozen input hashes are rechecked after analysis. The validation holdout (19 August–5 September 2025) stays sealed. Zero broker/market-data requests, database queries, paid model calls or trading changes occur.', '',
              'Next: specify a separate bounded repair of the eight failed US500 M30 intervals and the single failed tick interval if complete January evaluation is required. Preserve the old failures and mark replacement provenance; do not call repaired, previously observed data a fresh holdout. To calibrate cost, obtain paired decision/dispatch/fill observations for both entry and exit, actual settled fees/swaps and contract/conversion terms, separated by account, symbol, session and order size. Freeze any resulting cost and admission experiment before evaluating untouched dates. Do not lower costs merely to reach a win target.', '',
              f'[Frozen plan](../{PLAN}) · [Measured JSON](../{RESULT}) · [Prior cost evidence](../{COST})', '']
    (ROOT/REPORT).write_text('\n'.join(lines))


def publish(out):
    if cmd('git','branch','--show-current') != BRANCH: raise ValueError('wrong publication branch')
    r = read(ROOT/RESULT); verify(read(ROOT/PLAN))
    paths = CODE+[PLAN, RESULT, REPORT, 'log/CODEX_Log.txt']
    if not set(cmd('git','diff','--name-only','HEAD').splitlines()) <= set(paths): raise ValueError('unrelated changes')
    cmd('git','add','--',*paths)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:
        cmd('git','commit','-m','research: audit US500 preparation and measured cost sensitivities (№18,346 §§1–10)')
    head = cmd('git','rev-parse','HEAD'); hashes = {name:sha(ROOT/name) for name in paths}
    cmd('git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',BRANCH)
    body = out/'pr-body.md'
    body.write_text(f'№18,346 §§1–10 · {sgt()} · by Codex (GPT-6), effort not exposed.\n\nThe supplied script asserted restored US500 data and calibrated profits without implementing either. This offline runner audits retained failures, distinguishes decoder errors without weakening validation, and measures two cost ledgers on identical native fills. The existing count-mode transport fix is retained. No January data is restored and proposed low costs remain unverified.\n\nThree focused regression tests plus existing seasonal tests; native 24bp accounting parity, deduplicated identities, censoring and source hashes checked. No broker reads, live changes or financial PASS.\n\nReport: `{REPORT}`.\n')
    existing = gh('pr','list','--repo','ang-kl/jiaoyi','--head',BRANCH,'--state','open','--json','url')
    url = existing[0]['url'] if existing else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',BRANCH,'--title','Research: US500 preparation diagnosis and matched-fill cost ledgers (№18,346 §§1–10)','--body-file',str(body))
    print('REPORT_PR '+url, flush=True); number = url.rsplit('/',1)[-1]
    save(out/'publication.json', dict(url=url, head=head, artifact_sha256=hashes)); deadline = time.monotonic()+1800
    while True:
        pr = gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup')
        if pr['state'] != 'OPEN' or pr['headRefOid'] != head: raise ValueError('PR changed')
        state = ci_state(pr['statusCheckRollup']); print('CI '+state, flush=True)
        if state == 'failed': raise ValueError('CI failed; no merge')
        if state == 'success': break
        if time.monotonic() > deadline: raise ValueError('CI timeout; no merge')
        time.sleep(15)
    cmd('gh','pr','merge',number,'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr = gh('pr','view',number,'--repo','ang-kl/jiaoyi','--json','state,mergeCommit')
    if pr['state'] != 'MERGED': raise ValueError('merge unconfirmed')
    cmd('git','fetch','origin','main')
    for name,h in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+name],cwd=ROOT)).hexdigest() != h: raise ValueError('merged artifact mismatch')
    block = dict(trace='№18,346', status='offline_audit_and_sensitivity_completed', pr_url=url,
                 pr_merge_commit=pr['mergeCommit']['oid'], us500_episodes_restored=0,
                 ledger_a_stress_24bp=r['s3_two_stage']['ledger_a_stress_24bp'],
                 ledger_b_unverified_sensitivity=r['s3_two_stage']['ledger_b_unverified_cost_sensitivity'],
                 empirical_cost_calibration=False, new_broker_requests=0, financial_pass=False,
                 validation_opened=False, report_file=REPORT, verified_at_sgt=sgt())
    save(out/'review-block.json',block); update=out/'pr-update.json'
    save(update,dict(body=body.read_text()+'\nVerified on main:\n```json\n'+json.dumps(block,indent=2)+'\n```\n'))
    cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(update),'--jq','.html_url')
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('action', choices=['prepare','run','publish','all'])
    ap.add_argument('--output',type=Path,default=Path('/workspace/jiaoyi-us500-prep-cost-18346'))
    args = ap.parse_args(); out=args.output; out.mkdir(parents=True,exist_ok=True)
    if cmd('git','branch','--show-current') != BRANCH:
        if (ROOT/RESULT).exists(): print('Already recorded; read '+REPORT+'. No sources reread.'); return
        if cmd('git','diff','--name-only','HEAD'): raise ValueError('preserve existing tracked edits')
        cmd('git','fetch','origin','main');cmd('git','switch','-c',BRANCH,'origin/main')
    if args.action in ['prepare','all']:
        prepare()
        if args.action == 'prepare': return
        for pattern in ['test_us500_prep_cost.py','test_seasonal_pilot.py']:
            subprocess.run(['python3','-m','unittest','discover','-s','ops/research','-p',pattern],cwd=ROOT,check=True)
        paths = CODE+[PLAN,'log/CODEX_Log.txt']
        if not set(cmd('git','diff','--cached','--name-only').splitlines()) <= set(paths): raise ValueError('unrelated staged changes')
        cmd('git','add','--',*paths)
        if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:
            cmd('git','commit','-m','research: freeze offline US500 audit and same-fill cost comparison (№18,346 §§1–6)')
    if args.action in ['run','all']: run(read(ROOT/PLAN),out)
    if args.action in ['publish','all']: publish(out)


if __name__ == '__main__': main()
