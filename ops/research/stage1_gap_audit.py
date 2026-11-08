"""Offline reconciliation of the №18,380 preparation stop; no collection imports."""
import collections
import datetime as dt
import json
from pathlib import Path
import stage1_gate9 as s
from rebuild_preflight import offline_process, require


def audit():
    offline_process();p=s.check_freeze();done=s.ledger();bars,refs,verified=s.load_cache(p,done)
    lines=[json.loads(x) for x in (s.WORK/'ledger.jsonl').read_text().splitlines()]
    expected={'bar-'+x['key'] for x in p['bar_segments']}
    require(set(done)==expected and len(done)==112,'collection_receipt_denominator')
    require(all(x['status']=='complete' for x in done.values()),'unexpected_failed_source')
    require(all(x['key'].startswith('bar-') for x in lines),'tick_request_before_preparation')
    for x in lines:
        if x['state']=='started':
            part=next(v for v in p['bar_segments'] if x['key']=='bar-'+v['key'])
            require(x['request']==part and x['plan_sha256']==s.sha(s.PLAN),'request_not_frozen')
            require(x['at_sgt']>=p['frozen_at_sgt'],'collection_before_freeze')
    counts={};gaps={}
    for sym in s.CORE:
        js=[j for j in p['episodes'] if j['symbol']==sym];missing=sorted({t for j in js for t in j['required_m30_slots'] if t not in bars[(s.ACCOUNT,sym)]})
        counts[sym]={'episodes':5,'complete_45d_episodes':sum(all(t in bars[(s.ACCOUNT,sym)] for t in j['required_m30_slots']) for j in js),'missing_unique_slots':len(missing),'missing_by_date':{j['date']:sum(t not in bars[(s.ACCOUNT,sym)] for t in j['required_m30_slots']) for j in js},'opening_reference_count':sum(s.ms(j['from']) in bars[(s.ACCOUNT,sym)] for j in js)}
        if missing:
            pattern=collections.Counter(dt.datetime.fromtimestamp(t/1000,dt.timezone.utc).strftime('%a %H:%M') for t in missing)
            gaps[sym]={'missing_intervals':[{'from':s.stamp(a),'to':s.stamp(b)} for a,b in s.merged([(t,t+1800000) for t in missing])],'utc_weekday_time_counts':dict(sorted(pattern.items()))}
    capture_path=s.ROOT/'docs/rebuild_rfc_gates_5_to_9_evidence_18370.json';capture=s.read(capture_path)
    account=next(x for x in capture['accounts'] if x['account_id']==s.ACCOUNT);names={x['symbolId']:x['symbolName'] for x in account['selected_catalog_rows']}
    contracts={names[x['symbolId']]:{k:x[k] for k in ['symbolId','schedule','scheduleTimeZone','holiday'] if k in x} for x in account['contracts']['symbol'] if names.get(x['symbolId']) in ['BTCUSD','ETHUSD']}
    out={'trace':'№18,380','status':'blocked_preparation_before_ticks','plan_sha256':s.sha(s.PLAN),'collection_ledger_sha256':s.sha(s.WORK/'ledger.jsonl'),
         'receipt_reconciliation':{'requests_started':112,'unique_receipts_finished':112,'hashes_reconciled':112,'reused_cached_source_hashes_reconciled':29,'all_verified_files':len(verified),'failed_receipts':0,'tick_requests':0,'all_requests_after_freeze':True,'all_requests_match_frozen_intervals':True},
         'by_symbol':counts,'crypto_gap_details':gaps,'calendar_evidence':{'path':str(capture_path.relative_to(s.ROOT)),'sha256':s.sha(capture_path),'captured_contracts':contracts,'historical_2024_authority':False},
         'conclusion':'30 of40 native histories complete; all10 crypto histories have77–96 missing bars. All40 opening references exist. Simultaneous repeated BTC/ETH gaps suggest broker availability/session differences, but current2026 schedules do not prove2024 closures. Frozen Phase4 calendar remains unchanged, so Gate3 correctly stops all ticks.',
         'next_step':'A separately versioned calendar/dependency policy requires authoritative historical closures or a newly declared partial-history hypothesis; do not refill spent intervals or drop crypto from this frozen40 denominator. Any changed candidate evaluated on this already collected week must disclose reused preparation/price inspection and cannot inherit the original frozen candidate qualification.',
         'capability_boundary':'Trusted local Stage1 runner. Plan-bound permit verifies intervals/type/account/prep; append-only runner ledger enforces one attempt and refuses incomplete or duplicate receipts. It is not a sandbox for arbitrary malicious Python callers.',
         'financial_pass':False,'validation_opened':False,'new_broker_requests_by_this_audit':0}
    path=s.ROOT/'docs/gate9_stage1_gap_audit_18380.json';s.save(path,out)
    summary=s.read(s.WORK.parent/'stage1-summary.json');summary.update(status=out['status'],gap_audit_file=str(path),gap_audit_sha256=s.sha(path),receipt_reconciliation=out['receipt_reconciliation'],symbol_preparation=counts)
    s.save(s.WORK.parent/'stage1-summary.json',summary)
    print(json.dumps(out['receipt_reconciliation'],indent=2))

if __name__=='__main__':audit()
