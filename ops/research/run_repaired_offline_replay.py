#!/usr/bin/env python3
"""№18,331: one frozen, cached-data experiment; never imports a broker client."""
import argparse
import collections
import datetime as dt
import fcntl
import hashlib
import html
import json
import math
from pathlib import Path
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / 'docs/offline-repair-plan-18331.json'
BASE = ROOT / 'docs/three-year-pilot-results-2023-2025.json'
MANIFEST = ROOT / 'docs/three-year-execution-18324.json'
BINARY = ROOT / 'target/debug/examples/offline_repaired_replay'
OUTPUT = ROOT / 'docs/offline-repaired-engine-results-18331.json'
REPORT = ROOT / 'doc/OFFLINE_REPAIRED_ENGINE_REPORT-18331-2026-11-07.md'
MODELS = ['production_s3_baseline', 's3_two_stage', 's3_dynamic_gate', 's3_dynamic_two_stage',
          'macro_180m_baseline', 'macro_180m_repaired', 'always_long_control', 'own_momentum_fixed_horizon']
NEW_MODELS = ['s3_two_stage', 's3_dynamic_gate', 's3_dynamic_two_stage', 'macro_180m_repaired']
BPS = ['24', '2', '3.74']
REQUIRED = {'check', 'docker', 'engineering'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, data):
    path.write_text(json.dumps(data, separators=(',', ':'), allow_nan=False) + '\n')


def sgt():
    return dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime('%d-%m-%Y %H:%M:%S SGT (+08:00)')


def cmd(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def gh(*args):
    return json.loads(cmd('gh', *args))


def without_notes(value):
    if isinstance(value, dict):
        return {k: without_notes(v) for k, v in value.items() if k != 'notes'}
    if isinstance(value, list):
        return [without_notes(v) for v in value]
    return value


def verify_sources(source, base, plan):
    if sha(BASE) != plan['baseline_result_sha256'] or sha(MANIFEST) != plan['baseline_manifest_sha256']:
        raise ValueError('baseline/manifest changed')
    if sha(ROOT / 'docs/offline-repair-contract-18331.md') != plan['contract_sha256'] or plan['models'] != NEW_MODELS:
        raise ValueError('frozen interpretation/model set changed')
    rows = base['rows']
    manifest = json.loads(MANIFEST.read_text())
    if len(rows) != 120 or len({r['key'] for r in rows}) != 120 or {r['key'] for r in rows} != {j['key'] for j in manifest['jobs']}:
        raise ValueError('missing/duplicate source denominator')
    if collections.Counter(r['status'] for r in rows) != {'complete': 119, 'failed': 1}:
        raise ValueError('source statuses changed')
    events = [json.loads(line) for line in (source / 'ledger.jsonl').read_text().splitlines()]
    if events != base['receipts'] or json.loads((source / 'freeze.json').read_text()) != base['provenance']:
        raise ValueError('source receipts/freeze changed')
    starts = [e['key'] for e in events if e['state'] == 'started']
    ends = [e for e in events if e['state'] == 'finished']
    if len(starts) != len(set(starts)) or len(ends) != len(starts) or {e['key'] for e in ends} != set(starts):
        raise ValueError('uncertain or duplicate source receipt')
    expected = {'job-' + j['key'] for j in manifest['jobs']} | {'bar-' + b['key'] for b in manifest['bar_segments']}
    if set(starts) != expected:
        raise ValueError('source component scope changed')
    hashes = {}
    for e in ends:
        path = source / (e['key'] + '.json')
        if sha(path) != e['file_sha256']:
            raise ValueError('source hash changed: ' + e['key'])
        hashes[e['key']] = e['file_sha256']
    for row in rows:
        if hashes['job-' + row['key']] != row['file_sha256']:
            raise ValueError('episode lineage changed')
    return hashes


def freeze(out, hashes):
    paths = sorted((ROOT / 'crates/jiaoyi-core/src').rglob('*.rs')) + [
        ROOT / 'Cargo.lock', ROOT / 'crates/jiaoyi-core/examples/offline_repaired_replay.rs',
        Path(__file__).resolve(), PLAN, ROOT / 'docs/offline-repair-contract-18331.md']
    provenance = {str(p.relative_to(ROOT)): sha(p) for p in paths}
    for path, wanted in provenance.items():
        actual = hashlib.sha256(subprocess.check_output(['git', 'show', 'HEAD:' + path], cwd=ROOT)).hexdigest()
        if actual != wanted:
            raise ValueError('commit frozen experiment before replay: ' + path)
    current = dict(plan_sha256=sha(PLAN), baseline_sha256=sha(BASE), source_sha256=provenance,
                   binary_sha256=sha(BINARY), corpus_sha256=hashes, freeze_commit=cmd('git', 'rev-parse', 'HEAD'))
    path = out / 'freeze.json'
    if path.exists():
        previous = json.loads(path.read_text())
        if any(previous[k] != v for k, v in current.items()):
            raise ValueError('frozen experiment drift; do not overwrite evidence')
        return previous
    if (out / 'ledger.jsonl').exists():
        raise ValueError('existing results without pre-run freeze')
    current['frozen_at_sgt'] = sgt()
    save(path, current)
    return current


def observation(model, bps):
    control = model.get('control')
    if control is not None:
        return True, control.get('net_by_bps', {}).get(bps), control.get('gross_r'), False
    trade = model.get('study', {}).get('trade')
    if not trade:
        return False, None, None, False
    outcome = trade['exit_comparison']['candidate']
    return True, model.get('sensitivities', {}).get('trailing', {}).get(bps), outcome.get('scenario_gross_r'), outcome.get('trail_applied', 0) > 0


def score(rows, name, bps):
    values = []
    entries = gross_flat = amended = 0
    reasons = collections.Counter()
    for row in rows:
        m = row.get('models', {}).get(name, {})
        entered, net, gross, applied = observation(m, bps)
        entries += entered
        gross_flat += gross == 0 if gross is not None else False
        amended += applied
        if net is not None:
            if not math.isfinite(net):
                raise ValueError('nonfinite outcome')
            values.append(net)
        reasons.update(set([row['status']] if row['status'] != 'complete' else
                           [m['status']] if 'status' in m else
                           [x['reason'] for x in m.get('study', {}).get('marks', [])]))
    profit = sum(v for v in values if v > 0)
    loss = -sum(v for v in values if v < 0)
    return dict(episodes=len(rows), entries=entries, closed_trades=len(values), wins=sum(v > 0 for v in values),
                losses=sum(v < 0 for v in values), net_breakevens=sum(v == 0 for v in values),
                gross_flat_exits=gross_flat, stop_amend_applied_episodes=amended,
                censored_or_unavailable=entries-len(values), nonentries=len(rows)-entries,
                win_rate=sum(v > 0 for v in values)/len(values) if values else None,
                mean_net_r=statistics.mean(values) if values else None,
                profit_factor=profit/loss if loss else None,
                profit_factor_status='finite' if loss else 'no_losses' if values else 'no_closes',
                reasons=dict(reasons), financial_pass=False)


def verify_native_accounting(models):
    checked = 0
    for name in NEW_MODELS:
        m = models[name]
        trade = m.get('study', {}).get('trade')
        if not trade:
            continue
        entry = trade['entry']
        sign = 1 if entry['side'] == 'long' else -1
        for arm, key in [('trailing', 'candidate'), ('fixed', 'baseline')]:
            o = trade['exit_comparison'][key]
            fills = o['fills']
            if o['remaining_volume']:
                if o['scenario_gross_r'] is not None or any(v is not None for v in m['sensitivities'][arm].values()):
                    raise ValueError('censored position assigned a closed return')
                continue
            if sum(f['volume'] for f in fills) != entry['volume']:
                raise ValueError('exit volume conservation failed')
            gross = sum(sign*(f['price']-entry['fill'])/entry['initial_r']*f['volume']/entry['volume'] for f in fills)
            exit_price = sum(f['price']*f['volume']/entry['volume'] for f in fills)
            if not math.isclose(gross, o['scenario_gross_r'], abs_tol=1e-9, rel_tol=1e-9):
                raise ValueError('gross fill arithmetic failed')
            for bps in BPS:
                net = gross-(entry['fill']+exit_price)*float(bps)/10000/entry['initial_r']
                if not math.isclose(net, m['sensitivities'][arm][bps], abs_tol=1e-9, rel_tol=1e-9):
                    raise ValueError('net cost arithmetic failed')
                checked += 1
    return checked


def summarize(rows):
    groups = {'overall': rows}
    for field in ['symbol', 'asset_class', 'cohort']:
        for key in sorted({r[field] for r in rows}):
            groups[field + ':' + key] = [r for r in rows if r[field] == key]
    for year in ['2023', '2024', '2025']:
        groups['year:' + year] = [r for r in rows if r['date'].startswith(year)]
    scores = {g: {m: {b: score(rs, m, b) for b in BPS} for m in MODELS} for g, rs in groups.items()}
    matched = {}
    for candidate, baseline in [('s3_two_stage', 'production_s3_baseline'), ('s3_dynamic_gate', 'production_s3_baseline'),
                                ('s3_dynamic_two_stage', 'production_s3_baseline'), ('macro_180m_repaired', 'macro_180m_baseline')]:
        deltas = []
        added = removed = 0
        for r in rows:
            a = observation(r.get('models', {}).get(candidate, {}), '24')
            b = observation(r.get('models', {}).get(baseline, {}), '24')
            added += a[0] and not b[0]
            removed += b[0] and not a[0]
            if a[1] is not None and b[1] is not None:
                deltas.append(a[1]-b[1])
        matched[candidate] = dict(baseline=baseline, matched_closes=len(deltas), unmatched_episodes=len(rows)-len(deltas),
                                  added_entry_episodes=added, removed_entry_episodes=removed,
                                  mean_delta_r=statistics.mean(deltas) if deltas else None,
                                  caveat='Conditional same-episode closed pairs; changed target/admission changes selection. No causal alpha or significance claim.')
    return dict(scores=scores, matched=matched)


def run(source, out):
    out.mkdir(parents=True, exist_ok=True)
    with (out / 'run.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        plan = json.loads(PLAN.read_text())
        base = json.loads(BASE.read_text())
        hashes = verify_sources(source, base, plan)
        frozen = freeze(out, hashes)
        events = [json.loads(x) for x in (out / 'ledger.jsonl').read_text().splitlines()] if (out / 'ledger.jsonl').exists() else []
        starts = [e['key'] for e in events if e['state'] == 'started']
        ends = {e['key']: e for e in events if e['state'] == 'finished'}
        if len(starts) != len(set(starts)) or set(starts) != set(ends) or len(events) != 2*len(ends):
            raise ValueError('uncertain replay receipt; retain and inspect before repeating')
        rows = []
        checked = parity = 0
        for old in base['rows']:
            row = {k: old[k] for k in ['key', 'symbol', 'date', 'cohort', 'asset_class', 'status', 'file_sha256']}
            row['protocol_flags'] = old.get('protocol_flags', [])
            row['source_quality'] = old.get('replay', {}).get('result', {}).get('quality')
            row['bar_component_failures'] = old.get('bar_component_failures', 0)
            if old['status'] == 'complete':
                path = out / ('replay-' + old['key'] + '.json')
                if old['key'] in ends:
                    if sha(path) != ends[old['key']]['file_sha256']:
                        raise ValueError('cached replay changed')
                else:
                    if path.exists():
                        raise ValueError('unreceipted replay file')
                    with (out / 'ledger.jsonl').open('a') as f:
                        f.write(json.dumps(dict(state='started', key=old['key'], at_sgt=sgt())) + '\n')
                    proc = subprocess.run([str(BINARY), str(source / ('job-' + old['key'] + '.json'))], capture_output=True, text=True, timeout=90, check=True)
                    parsed = json.loads(proc.stdout)
                    save(path, parsed)
                    with (out / 'ledger.jsonl').open('a') as f:
                        f.write(json.dumps(dict(state='finished', key=old['key'], file_sha256=sha(path), at_sgt=sgt())) + '\n')
                new = json.loads(path.read_text())
                if new['payload_sha256'] != old['replay']['payload_sha256'] or without_notes(new['result']['baseline']) != without_notes(old['replay']['result']):
                    raise ValueError('original baseline parity failed: ' + old['key'])
                parity += 1
                baseline = new['result']['baseline']['models']
                row['models'] = without_notes(new['result']['models'])
                row['models'].update(production_s3_baseline=baseline['production_s3'], macro_180m_baseline=baseline['macro_180m'],
                                     always_long_control=baseline['always_long_control'], own_momentum_fixed_horizon=baseline['own_momentum_fixed_horizon'])
                row['models'] = without_notes(row['models'])
                row['peer'] = new['result']['peer']
                row['replay_sha256'] = sha(path)
                checked += verify_native_accounting(row['models'])
            rows.append(row)
            print(json.dumps(dict(processed=len(rows), key=row['key'], status=row['status'])), flush=True)
        if verify_sources(source, base, plan) != hashes or sha(BINARY) != frozen['binary_sha256']:
            raise ValueError('inputs changed during replay')
        if any(sha(ROOT / path) != wanted for path, wanted in frozen['source_sha256'].items()):
            raise ValueError('model/runner changed during replay')
        result = dict(trace='№18,331', generated_at_sgt=sgt(), provenance=frozen, rows=rows, summary=summarize(rows),
                      new_broker_requests=0, validation_opened=False, financial_pass=False,
                      source_protocol_audit=base['protocol_audit'], source_protocol_compliance='FAILED',
                      current_execution_protocol='PASS', verification=dict(source_components=len(hashes), baseline_parity_episodes=parity,
                      independent_cost_values=checked, native_replays_completed=parity), raw_corpus=str(source))
        save(OUTPUT, result)
        write_report(result)
        write_review(out, result)
        with (ROOT / 'log/CODEX_Log.txt').open('a') as f:
            f.write(f'\n[§LOG.127] №18,331 §§5–8 · {sgt()}; by <Codex (GPT-6), effort: not exposed>.\n'
                    f'Completed {parity}/119 cached native replays; retained all120 scheduled rows including one prior failure.\n'
                    f'Original baseline parity verified on all{parity}; independent cost values checked:{checked}.\n'
                    'No new broker reads, validation data, live changes or invented peer observations.\n'
                    'Prior protocol failure affecting20 episodes remains inherited and disqualifying.\n'
                    'Actual primary outcomes: '+json.dumps({m:result['summary']['scores']['overall'][m]['24'] for m in NEW_MODELS})+'\n'
                    'Published report contains per-symbol/class/year results, costs and matched comparisons.\n'
                    'CI/actual merge receipt belongs to the PR; engineering success is not economic qualification.\n')


def write_review(out, r):
    save(out / 'review-block.json', dict(trace='№18,331', plan_sha256=r['provenance']['plan_sha256'], pr_merge_commit=None,
        episodes_scheduled=120, episodes_replayed=r['verification']['native_replays_completed'], failed_source_episodes=1,
        new_broker_requests=0, source_protocol_compliance='FAILED', inherited_protocol_affected_episodes=20,
        peer_data_missing_symbols=['US500', 'XAUUSD'], primary_slippage_bps_per_side=24,
        models={m:{k:r['summary']['scores']['overall'][m]['24'][k] for k in ['closed_trades','wins','win_rate','mean_net_r','profit_factor','gross_flat_exits','net_breakevens']} for m in NEW_MODELS},
        financial_pass=False, validation_opened=False, report_file=str(REPORT.relative_to(ROOT))))


def fmt(v):
    return 'unavailable' if v is None else f'{v:.6f}' if isinstance(v, float) else str(v)


def write_report(r):
    head = ['Model', 'bp/side', 'Entries', 'Closes', 'Wins', 'Net win rate', 'Mean net R', 'PF(R)', 'Gross flat', 'Censored']
    cells = []
    for m in MODELS:
        for b in BPS:
            s = r['summary']['scores']['overall'][m][b]
            cells.append([m,b,s['entries'],s['closed_trades'],s['wins'],fmt(s['win_rate']),fmt(s['mean_net_r']),fmt(s['profit_factor']),s['gross_flat_exits'],s['censored_or_unavailable']])
    table = lambda heads, rows: '| '+' | '.join(heads)+' |\n| '+' | '.join('---' for _ in heads)+' |\n'+''.join('| '+' | '.join(map(str,row))+' |\n' for row in rows)
    headline = '; '.join(f"{m}: {r['summary']['scores']['overall'][m]['24']['wins']}/{r['summary']['scores']['overall'][m]['24']['closed_trades']} wins, mean {fmt(r['summary']['scores']['overall'][m]['24']['mean_net_r'])} R" for m in ['s3_dynamic_two_stage','macro_180m_repaired'])
    text = f'''# Offline repaired-engine experiment

№18,331 §§1–8 · {r['generated_at_sgt']} · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**{r['verification']['native_replays_completed']}/119 cached native replays completed. All 120 scheduled rows remain, including the prior failed source.** Zero new broker requests. Primary outcomes: {headline}.

**No financial qualification or demonstrated progress towards the per-strategy/account 73% / reference R:R 1:1.38 latest-20 goal.** These are reused development dates, uncalibrated cost scenarios, and an inherited protocol failure affecting20 episodes. A 50% hit rate alone is insufficient. Validation stays sealed; no production entry, exit, registration, account or worker configuration changed.

## §2. What the supplied scripts actually did

Both attachments loaded the previous result JSON but did not use it to replay ticks. They assigned 96 macro trades/52 wins and48 S3 trades/31 wins as constants. Their per-symbol S3 wins sum to32, conflicting with31 overall. The Bash review block's means/PFs also disagree with its own arithmetic. Its CI reader uses `.state` for CheckRuns and can treat null as green; its quoted heredoc prints the literal merge command. Those outputs were not executed or published as market evidence.

This implementation executes native Rust on verified cached quote files and checks exact original-model parity for every successful episode. The Markdown directive's №18,331 governs; the companion Bash's №18,332 is not a separate experiment. Original attachment hashes and the pre-run interpretation are in the [frozen contract](../docs/offline-repair-contract-18331.md).

## §3. Frozen interpretation and limits

- Two-stage: original −1R stop until executable bid(long)/ask(short) reaches +1R; propose stop at entry, acknowledge on a strictly later quote only if still valid. Old broker bracket wins first. No further trailing or partial exit. Whole-position **gross** target1.38R rounded outward to the price grid. This changes target geometry as well as trailing; it cannot establish that trailing alone caused losses. An entry-price stop is gross-flat and usually net-negative after fees/slippage.
- Dynamic S3 admission: R must exceed max(2.5×current executable spread,0.15×ATR14) in **price units**, at decision and dispatch. ATR uses14 completed positive-volume M30 ranges plus the preceding close. Missing ATR blocks entry; failed numeric gate consumes the session. This replaces the old admission comparison only. The old rule was spread+commission+two-sided slippage relative to R, not a universal48bp threshold.
- Primary accounting retains24bp/side and executable bid/ask fills; extra commission remains the frozen zero-cost assumption. No double spread charge. Sensitivities2/3.74bp reprice the same fills; no empirical calibration claimed.
- Peer mappings authorise US500/NAS100, XAUUSD/XAGUSD and Crude/Brent. The119 cached datasets have oil peers only. US500 and Gold remain unavailable for macro; authorisation does not manufacture six completed peer bars.
- S3 original window/close stays intact: oil source07–17UTC reconstructs the07UTC start and censors at17 while native close remains20. ORB/macro/control window13–17 for oil. Index uses DST-aware NY open; Gold07–11UTC. No synthetic session-end close at a truncated boundary.
- Three S3 arms isolate two-stage geometry, dynamic admission, and both together. Macro applies authorised peers/two-stage geometry while retaining original admission. Controls and every baseline are preserved. No additional parameter search follows the outcomes.

## §4. Overall scenarios

Win rates use closed trades; nonentries, missing data and censored positions remain explicit in the120-row denominator. Profit factor is based on normalised R, not mixed-account dollars.

'''+table(head,cells)+'\n## §5. Per-symbol / class / year primary results\n\n'
    breakdown = []
    for group, models in r['summary']['scores'].items():
        if group.startswith(('symbol:', 'asset_class:', 'year:')):
            for name in NEW_MODELS:
                s=models[name]['24']
                breakdown.append([group,name,s['episodes'],s['entries'],s['closed_trades'],s['wins'],fmt(s['win_rate']),fmt(s['mean_net_r']),fmt(s['profit_factor'])])
    text += table(['Group','Model','Episodes','Entries','Closes','Wins','Win rate','Mean R','PF(R)'],breakdown)
    text += '\n## §6. Matched comparison and failure prevention\n\n```json\n'+json.dumps(r['summary']['matched'],indent=2)+'\n```\n'
    text += '''
The matched differences describe this reused sample only. Added entries, altered censoring and changed gross targets make aggregate win-rate comparisons insufficient to prove causality or edge. Preserve all losses and failed rows. The20 source-protocol flags remain; offline reuse does not erase the earlier broker rereads. Same-millisecond ordering, gaps, missing preparation, current-only contract metadata and uncalibrated execution costs still limit the evidence.

Next: compare the actual primary expectancy and per-symbol results before any further hypothesis. Independently establish broker cost/contract assumptions; do not choose cheaper costs because they look better. NAS100/XAGUSD collection requires a separately bounded and frozen source plan. A repaired collector/protocol and untouched development evidence are prerequisites to a credible next stage. Do not open Stage2, validation, or live promotion from this experiment.

## §7. Verification and reproducibility

'''
    text += '```json\n'+json.dumps(r['verification'],indent=2)+'\n```\n'
    text += f"\nPlan SHA256: `{r['provenance']['plan_sha256']}`. Pre-run freeze commit: `{r['provenance']['freeze_commit']}`. Binary SHA256: `{r['provenance']['binary_sha256']}`.\n"
    text += '\nAll original source receipt hashes and the baseline were checked before/after replay. Raw quotes remain in the existing workspace corpus, outside Git; the corpus is required to reproduce the run. The full JSON retains source hashes, native fills, per-episode nonentries, peer gaps and inherited protocol flags.\n'
    text += '\n## §8. Independent AI review\n\nRead this report, the frozen contract, the [measured JSON](../docs/offline-repaired-engine-results-18331.json) and PR #182 source audit. Verify costs, entry/exit chronology, changed-target attribution, missing peers and denominators. Treat supplied hard-coded positives as invalid. Propose the smallest evidence-backed next action; distinguish software PASS from financial qualification.\n'
    REPORT.write_text(text)
    html_table=lambda heads,rows:'<div class="scroll"><table><thead><tr>'+''.join('<th>'+html.escape(str(x))+'</th>' for x in heads)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+html.escape(str(x))+'</td>' for x in row)+'</tr>' for row in rows)+'</tbody></table></div>'
    page='<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Offline experiment №18,331</title><style>body{font:16px/1.55 system-ui;color:#172738;background:#f7f9fc;max-width:1300px;margin:auto;padding:24px}h1,h2{line-height:1.2}.notice{background:#fff0c4;padding:18px}.scroll{overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}th,td{text-align:left;padding:8px;border-bottom:1px solid #ccd5df}th{background:#e6edf5}pre{white-space:pre-wrap;overflow-wrap:anywhere}a{color:#1450a0}@media(max-width:600px){body{padding:14px}h1{font-size:27px}}</style><h1>Offline repaired-engine experiment</h1><p>№18,331 · '+html.escape(r['generated_at_sgt'])+' · by Codex (GPT-6), effort not exposed.</p><div class="notice"><b>TLDR:</b> '+html.escape(headline)+'. No financial qualification. Inherited source protocol FAILED;20 affected episodes.119 cached replays; one failed source retained; zero broker requests.</div><p>The supplied winning numbers were hard-coded. These tables use actual native replay. The gross1.38R target and entry-price stop still incur costs. Gold/index macro peers are unavailable in the frozen corpus. Costs2/3.74bp are sensitivities only. Original73% /1:1.38 latest20 per strategy/account remains unproven.</p><h2>Overall scenarios</h2>'+html_table(head,cells)+'<details><summary>Per-symbol, class and year</summary>'+html_table(['Group','Model','Episodes','Entries','Closes','Wins','Win rate','Mean R','PF(R)'],breakdown)+'</details><h2>Next decision</h2><p>Verify actual cost and contract assumptions; retain failed evidence and source flags. No further tuning, Stage2, validation or live promotion follows automatically.</p><p><a href="'+REPORT.name+'">Complete assessment</a> · <a href="../docs/offline-repaired-engine-results-18331.json">Native JSON</a> · <a href="../docs/offline-repair-contract-18331.md">Frozen contract</a></p></html>'
    REPORT.with_suffix('.html').write_text(page)


def ci_state(checks):
    required=[c for c in checks if c.get('name',c.get('context')) in REQUIRED]
    if len(required)!=3 or {c.get('name',c.get('context')) for c in required}!=REQUIRED:
        return 'pending'
    for c in required:
        end=c.get('conclusion') if 'status' in c else c.get('state')
        if end in ['FAILURE','ERROR','CANCELLED','TIMED_OUT','ACTION_REQUIRED','STALE','STARTUP_FAILURE']:
            return 'failed'
        if ('status' in c and c['status']!='COMPLETED') or end!='SUCCESS':
            return 'pending'
    return 'success'


def publish(out):
    artifacts=[REPORT,REPORT.with_suffix('.html'),OUTPUT,ROOT/'log/CODEX_Log.txt']
    for p in artifacts:
        if not p.exists():raise ValueError('missing report artifact')
    paths=[str(p.relative_to(ROOT)) for p in artifacts]
    subprocess.run(['git','add',*paths],cwd=ROOT,check=True)
    if subprocess.run(['git','diff','--cached','--quiet'],cwd=ROOT).returncode:
        subprocess.run(['git','commit','-m','research: publish measured offline repair experiment (№18,331 §§5–8)'],cwd=ROOT,check=True)
    head=cmd('git','rev-parse','HEAD');branch=cmd('git','branch','--show-current')
    if not branch.startswith('research/offline-repair-18331'):raise ValueError('unexpected publication branch')
    subprocess.run(['git','-c','credential.helper=','-c','credential.helper=!gh auth git-credential','push','-u','origin',branch],cwd=ROOT,check=True)
    block=json.loads((out/'review-block.json').read_text())
    body=out/'pr-body.md'
    body.write_text(f'№18,331 §§1–8 · {sgt()} · by Codex (GPT-6), effort not exposed.\n\nThe supplied scripts hard-coded profitable outcomes. This change implements and measures their three hypotheses in the offline native replay: entry-price stop after +1R with a gross1.38R whole-position target, causal volatility admission, and explicitly authorised class peers. The normal production route remains unchanged.\n\nReplayed119 cached episodes, retained one failed source and all120 denominators; no broker reads. Exact baseline parity, source hashes and independent fill/cost arithmetic checked. The20 inherited protocol failures and absent NAS100/XAGUSD data remain explicit. No financial PASS or live promotion.\n\nPrimary measured results:\n```json\n'+json.dumps(block['models'],indent=2)+'\n```\n\nFocused native and Python regressions cover threshold/next-quote exits, net costs, causal ATR/dispatch, real peer requirements, source denominators, arithmetic and explicit CI success. Reports and review contract are in doc/ and docs/; log/CODEX_Log.txt records preparation/results. Required check/docker/engineering CI must succeed on this head before merge.\n')
    prs=gh('pr','list','--repo','ang-kl/jiaoyi','--head',branch,'--state','all','--json','url,number')
    url=prs[0]['url'] if prs else cmd('gh','pr','create','--repo','ang-kl/jiaoyi','--base','main','--head',branch,'--title','Research: measured offline exit, gate and peer experiment (№18,331 §§1–8)','--body-file',str(body))
    number=int(url.rsplit('/',1)[1]);hashes={f:sha(ROOT/f) for f in paths}
    save(out/'publication.json',dict(url=url,head=head,artifact_sha256=hashes));print('REPORT_PR '+url,flush=True)
    deadline=time.monotonic()+1800
    while True:
        pr=gh('pr','view',str(number),'--repo','ang-kl/jiaoyi','--json','state,headRefOid,statusCheckRollup')
        if pr['headRefOid']!=head:raise ValueError('PR head changed')
        state=ci_state(pr['statusCheckRollup']);print('CI '+state,flush=True)
        if state=='failed':raise ValueError('required CI failed; inspect and repair')
        if state=='success':break
        if time.monotonic()>deadline:raise ValueError('CI timeout is not success')
        time.sleep(10)
    if pr['state']!='MERGED':cmd('gh','pr','merge',str(number),'--repo','ang-kl/jiaoyi','--squash','--match-head-commit',head)
    pr=gh('pr','view',str(number),'--repo','ang-kl/jiaoyi','--json','state,mergeCommit,mergedAt')
    if pr['state']!='MERGED':raise ValueError('merge not confirmed')
    cmd('git','fetch','origin','main')
    for path,wanted in hashes.items():
        if hashlib.sha256(subprocess.check_output(['git','show','origin/main:'+path],cwd=ROOT)).hexdigest()!=wanted:
            raise ValueError('main artifact mismatch')
    block['pr_merge_commit']=pr['mergeCommit']['oid'];save(out/'review-block.json',block)
    body.write_text(body.read_text()+f'\nMerged green; verified main artifact hashes at {sgt()}. Merge `{block["pr_merge_commit"]}`.\n')
    save(out/'body-update.json',{'body':body.read_text()});cmd('gh','api','--method','PATCH',f'repos/ang-kl/jiaoyi/pulls/{number}','--input',str(out/'body-update.json'),'--jq','.html_url')
    save(out/'publication.json',dict(url=url,head=head,merge=pr,artifact_sha256=hashes))
    print('=== COPY-PASTE FOR GEMINI REVIEW ===\n'+json.dumps(block,indent=2)+'\n====================================',flush=True)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',choices=['run','publish'])
    p.add_argument('--source-dir',type=Path,default=Path('/workspace/jiaoyi-three-year-pilot-18325'))
    p.add_argument('--output-dir',type=Path,required=True);p.add_argument('--publish',action='store_true');a=p.parse_args()
    if a.action=='run':run(a.source_dir,a.output_dir)
    if a.action=='publish' or a.publish:publish(a.output_dir)
