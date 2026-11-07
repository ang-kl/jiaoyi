#!/usr/bin/env python3
"""Offline receipt inventory. No collection client, database or network capability."""
import argparse
import collections
import csv
import datetime as dt
from fractions import Fraction
from decimal import Decimal
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
UTC = dt.timezone.utc
SEAL_START = int(dt.datetime(2025, 8, 19, tzinfo=UTC).timestamp()*1000)
SEAL_END = int(dt.datetime(2025, 9, 6, tzinfo=UTC).timestamp()*1000)
RESEARCH_CUTOFF = int(dt.datetime(2025, 10, 24, tzinfo=UTC).timestamp()*1000)
OFFSETS = [0, 30, 60, 180, 240]
CORE = ['SpotCrude', 'SpotBrent', 'US500', 'NAS100', 'XAUUSD', 'XAGUSD', 'BTCUSD', 'ETHUSD']
HASH = re.compile(r'^[a-f0-9]{64}$')
ACCOUNT_ENV = {43002148: 'live', 46130058: 'demo', 47790949: 'demo', 49236051: 'demo'}

def read(p): return json.loads(Path(p).read_text())
def sha(p):
    h=hashlib.sha256()
    with Path(p).open('rb') as f:
        for block in iter(lambda:f.read(1024*1024),b''): h.update(block)
    return h.hexdigest()
def canonical(x): return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()
def save(p,x): Path(p).write_text(json.dumps(x,indent=2,sort_keys=True,allow_nan=False)+'\n')
def ms(s): return int(dt.datetime.fromisoformat(s.replace('Z','+00:00')).timestamp()*1000)
def stamp(t): return dt.datetime.fromtimestamp(t/1000,UTC).isoformat().replace('+00:00','Z')
def overlaps(a,b,c=SEAL_START,d=SEAL_END): return a<d and c<b
def permitted(a,b): return a<b and not overlaps(a,b) and b<=RESEARCH_CUTOFF
def protected_dependency(a,b,bars,quotes=None):
    if not permitted(a,b+(5001 if quotes is not None else 0)):return True
    if any(not permitted(bar['ts_ms'],bar['ts_ms']+1800000) for bar in bars):return True
    if quotes is not None:
        if any(not permitted(ms(bar['at']),ms(bar['at'])+1800000) for bar in quotes.get('bars',[])):return True
        return any(not a<=t['ts_ms']<=b+5000 or not permitted(t['ts_ms'],t['ts_ms']+1)
                   for side in ['bids','asks'] for t in quotes[side])
    return False
def walk(x):
    if isinstance(x,dict):
        yield x
        for v in x.values(): yield from walk(v)
    elif isinstance(x,list):
        for v in x: yield from walk(v)

def quantiles(values):
    """Exact type-7 linear interpolation; JSON carries both rational and decimal."""
    v=sorted(Fraction(x) for x in values)
    names=['min','p10','p25','p50_median','p75','p90','max']
    if not v: return {'n':0,'values':{n:None for n in names},'exact':{n:None for n in names}}
    out={}
    for name,p in zip(names,[Fraction(0),Fraction(1,10),Fraction(1,4),Fraction(1,2),Fraction(3,4),Fraction(9,10),Fraction(1)]):
        h=(len(v)-1)*p;i=h.numerator//h.denominator
        out[name]=v[i]+(v[min(i+1,len(v)-1)]-v[i])*(h-i)
    return {'n':len(v),'values':{n:float(x) for n,x in out.items()},'exact':{n:f'{x.numerator}/{x.denominator}' for n,x in out.items()}}

def checkpoints(bids,asks,targets,digits):
    """Recorded quote-side updates only, stable within-side order, no interpolation."""
    if not 0<=digits<=5: raise ValueError('unsupported precision')
    events=[];quantum=10**(5-digits)
    for side,rows in enumerate([bids,asks]):
        previous=None
        for t in rows:
            at,price=t['ts_ms'],t['price_e5']
            if (previous is not None and at<previous) or price<=0 or price%quantum:
                raise ValueError('unordered, invalid or off-grid recorded tick')
            previous=at;events.append((at,side,price))
    events.sort(key=lambda e:e[0])
    pending=sorted(set(targets.values()));out={n:None for n in targets}
    bid=ask=None;i=0
    for target in pending:
        found=None
        while i<len(events) and events[i][0]<=target+5000:
            at=events[i][0]
            while i<len(events) and events[i][0]==at:
                _,side,price=events[i]
                if side: ask=(at,price)
                else: bid=(at,price)
                i+=1
            if at<target:continue
            if bid and ask and max(at-bid[0],at-ask[0])<=6000 and ask[1]>=bid[1]:
                found={'target_ms':target,'quote_ms':at,'bid_e5':bid[1],'ask_e5':ask[1],
                    'spread_e5':ask[1]-bid[1],'quote_delay_ms':at-target,
                    'side_max_age_ms':max(at-bid[0],at-ask[0])}
                break
        # Duplicate targets (strategy and experiment open) reuse the same observation.
        for other,at in targets.items():
            if at==target:out[other]=found
    return out

def reconcile(scope):
    rows=[]
    for entry in scope['files']:
        p=Path(entry['path'])
        actual=sha(p) if p.exists() else None
        rows.append({'path':str(p),'expected_sha256':entry['sha256'],'actual_sha256':actual,
                     'status':'match' if actual==entry['sha256'] else 'missing' if actual is None else 'mismatch'})
    bad=[x for x in rows if x['status']!='match']
    if bad:raise ValueError('frozen inventory changed: '+str(bad[:3]))
    return rows

def load_verified_episode(manifest,episode_id):
    """Single entry point for future consumers; inventory eligibility is not financial qualification."""
    matches=[e for e in manifest['episodes'] if e['episode_id']==episode_id]
    if len(matches)!=1:raise ValueError('episode identity must resolve exactly once')
    ep=matches[0]
    if not ep['eligible_for_research']:raise ValueError('protected or unqualified source dependency')
    if sha(ep['source_path'])!=ep['tick_receipt_sha256']:raise ValueError('episode receipt mismatch')
    source=read(ep['source_path']);kind,q,bars,start=market(source)
    a=ms(start or source['request']['from']);b=ms(source['request']['to'])
    dependencies=bars+source.get('payload',{}).get('peer_m30',[])
    if kind!='ticks' or protected_dependency(a,b,dependencies,q):raise ValueError('episode dependency crossed seal')
    return source

def verify_prior_baseline(manifest,prior):
    available={e['tick_receipt_sha256'] for e in manifest['episodes']}
    available.update(e['receipt_sha256'] for e in manifest['duplicate_receipts'])
    missing=[j['path'] for j in prior['jobs'] if j['sha256'] not in available]
    if missing:raise ValueError('prior frozen episodes lost from unified inventory: '+str(missing))
    return {'prior_episodes_reconciled':len(prior['jobs']),'missing':0,
            'scope':'file identities only; no strategy path changed or rerun by inventory'}

def market(x):
    if not isinstance(x,dict):return None
    if isinstance(x.get('payload'),dict) and 'quotes' in x['payload']:
        q=x['payload']['quotes'];return 'ticks',q,x['payload'].get('m30',[]),x['payload'].get('source_from')
    if isinstance(x.get('result'),dict) and isinstance(x['result'].get('input'),dict) and 'bids' in x['result']['input']:
        q=x['result']['input'];return 'ticks',q,[],None
    if 'bids' in x and 'asks' in x:
        return 'ticks',{'bids':x['bids'],'asks':x['asks'],'contract':x.get('request',{}).get('contract',{})},x.get('bars',[]),None
    if isinstance(x.get('bars'),list) and 'request' in x:return 'bars',None,x['bars'],None
    return None

def freeze(workspace,output):
    workspace=Path(workspace);paths=set();roots=[]
    for p in sorted(workspace.iterdir()):
        if not p.is_dir() or p.name.startswith('.') or (p/'.git').exists():continue
        if p.name.startswith('jiaoyi') and 'rebuild-phase1' not in p.name:roots.append(p)
    roots += [ROOT/'docs',ROOT/'doc',ROOT/'log']
    for root in roots:
        for p in root.rglob('*'):
            if p.is_file() and p.suffix in {'.json','.jsonl','.csv','.log','.txt'} and '18355' not in p.name and not re.search(r'auth|secret|token|credential',p.name,re.I):
                paths.add(p)
    # Owner statements are source evidence, not instructions or model backtests.
    for p in (workspace/'attachments').rglob('*.csv'):
        if 'statement' in p.name.lower():paths.add(p)
    files=[]
    own_log=ROOT/'log/CODEX_Log.txt'
    if own_log in paths:
        snapshot=workspace/'jiaoyi-rebuild-phase1-18355'/'prior-CODEX_Log.txt'
        snapshot.write_bytes(own_log.read_bytes());paths.remove(own_log);paths.add(snapshot)
    for p in sorted(paths):
        files.append({'path':str(p),'sha256':sha(p),'bytes':p.stat().st_size})
    scope={'schema':'jiaoyi.offline-inventory-scope.v1','trace':'№18,355',
        'frozen_at_sgt':dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).isoformat(),
        'roots':[str(p) for p in roots],'files':files,
        'excluded':'Other Git worktrees duplicate history; credentials, build outputs and unrelated scratch projects excluded. Missing old /tmp caches are reported through prior-reference reconciliation.',
        'seal':{'from':stamp(SEAL_START),'to_exclusive':stamp(SEAL_END),'research_cutoff':stamp(RESEARCH_CUTOFF)},
        'sampling':'first non-crossed quote at/after checkpoint within5s; both recorded sides age<=6s; grouped millisecond updates; exact type7 quantiles; bps=20000*(ask-bid)/(ask+bid)',
        'new_broker_requests':0,'live_rules_changed':False}
    save(output,scope);print('FROZEN',len(files),'files',sha(output))

def index_sources(scope):
    """Resolve historical receipts against the frozen file catalog, retaining every mismatch."""
    catalog={x['path']:x for x in scope['files']};by_hash=collections.defaultdict(list);by_name=collections.defaultdict(list)
    for p,e in catalog.items():by_hash[e['sha256']].append(p);by_name[Path(p).name].append(p)
    data={};ledgers={};refs=[];claims=collections.defaultdict(list);jobs={};classes={};protocol=collections.defaultdict(set)
    for path in catalog:
        p=Path(path)
        if p.suffix=='.json':
            try:x=read(p)
            except (ValueError,UnicodeError):continue
            data[path]=x
            # Frozen plans and result manifests are metadata authorities for identity.
            if not isinstance(x,dict):continue
            for class_name,syms in x.get('classes',{}).items() if isinstance(x.get('classes'),dict) else []:
                if isinstance(syms,list):
                    for s in syms:
                        if isinstance(s,str):classes.setdefault(s,class_name)
            all_jobs=x.get('jobs',[]) if isinstance(x.get('jobs'),list) else []
            accounts={j.get('request',{}).get('account') for j in all_jobs if isinstance(j,dict)}-{None}
            context_account=x.get('account',next(iter(accounts)) if len(accounts)==1 else None)
            all_jobs=all_jobs+(x.get('bar_segments',[]) if isinstance(x.get('bar_segments'),list) else [])
            for j in all_jobs:
                if not isinstance(j,dict):continue
                req=j.get('request',j);symbol=req.get('symbol',j.get('symbol'))
                if symbol:
                    cls=j.get('asset_class',j.get('class'))
                    if cls:classes.setdefault(symbol,cls)
                    if req.get('from') and req.get('to'):
                        jobs.setdefault((symbol,req['from'],req['to']),[]).append((path,dict(j,inventory_context_account=context_account)))
            for row in x.get('rows',[]) if isinstance(x.get('rows'),list) else []:
                if isinstance(row,dict) and row.get('key') and row.get('protocol_flags'):
                    h=row.get('file_sha256',row.get('source_sha256'))
                    if isinstance(h,str):protocol[(row['key'],h)].update(row['protocol_flags'])
        elif p.name=='ledger.jsonl':
            ledgers[path]=[json.loads(s) for s in p.read_text().splitlines() if s.strip()]
    def add_ref(origin,h,hint=None,kind='frozen_reference'):
        if not isinstance(h,str) or not HASH.fullmatch(h):return
        candidates=[]
        if hint:
            p=Path(hint)
            candidates=[str(p)] if p.is_absolute() else [str(ROOT/p),str(Path(origin).parent/p)]
            candidates += by_name.get(p.name,[]) if not p.is_absolute() else []
            candidates=list(dict.fromkeys(candidates))
        else:candidates=by_hash.get(h,[])
        existing=[p for p in candidates if p in catalog]
        matches=[p for p in existing if catalog[p]['sha256']==h]
        explicit=bool(hint and (Path(hint).is_absolute() or (Path(origin).parent/hint).exists() or (ROOT/hint).exists()))
        status='match' if matches else 'mismatch' if existing and explicit else 'unavailable_locally'
        record={'origin':origin,'expected_sha256':h,'hint':hint,'status':status,'matching_paths':matches,'kind':kind}
        refs.append(record)
        for p in matches:claims[p].append({'origin':origin,'kind':kind,'sha256':h})
    for path,events in ledgers.items():
        for e in events:
            if e.get('state')!='finished':continue
            h=e.get('sha256',e.get('file_sha256'));key=e.get('key','')
            choices=[Path(path).parent/(prefix+key+'.json') for prefix in ['', 'job-', 'replay-']]
            existing=[p for p in choices if str(p) in catalog]
            matching=[p for p in existing if catalog[str(p)]['sha256']==h]
            add_ref(path,h,str(matching[0] if matching else existing[0] if existing else choices[0]),'collection_ledger')
    for path,x in data.items():
        if market(x):continue  # A source cannot certify its own receipt hash.
        for node in walk(x):
            for k,v in node.items():
                if isinstance(k,str) and k.endswith(('.json','.jsonl','.csv','.log')) and isinstance(v,str) and HASH.fullmatch(v):
                    add_ref(path,v,k)
            h=node.get('file_sha256',node.get('sha256'))
            hint=node.get('path',node.get('file'))
            if isinstance(h,str) and (hint is None or isinstance(hint,str)):
                add_ref(path,h,hint)
    # Deduplicate repeated references within nested frozen snapshots.
    refs=list({canonical(x):x for x in refs}.values());refs.sort(key=lambda x:(x['origin'],x.get('hint') or '',x['expected_sha256']))
    return catalog,data,claims,refs,jobs,classes,protocol

def code_audit():
    rows=[]
    for folder in [ROOT/'crates',ROOT/'ops/research']:
        for p in sorted(folder.rglob('*')):
            if p.suffix not in {'.rs','.py','.json','.proto'} or '__pycache__' in str(p) or p.name.startswith('replay_inventory'):continue
            text=p.read_text(errors='replace')
            patterns=[s for s in ['dispatch_to_fill_bps','commission','swap','slippage_bps_per_side','ProtoOADeal','ProtoOASymbolByIdRes'] if s in text]
            if patterns:rows.append({'path':str(p.relative_to(ROOT)),'sha256':sha(p),'matches':patterns,
                'evidence_kind':'synthetic_test_or_fixture' if '/tests/' in str(p) or p.name.startswith('test_') else 'implementation_not_observation'})
    return rows

def broker_weekly_schedule(record):
    """Literal ProtoOAInterval half-open seconds from Sunday in scheduleTimeZone."""
    intervals=sorted((x['startSecond'],x['endSecond']) for x in record.get('schedule',[]))
    if not intervals:return {'status':'unverified_requires_broker_spec','open_intervals':None,'closed_intervals':None}
    if any(not 0<=a<b<=604800 for a,b in intervals):raise ValueError('invalid recorded broker weekly interval')
    def label(t):
        days=['Sunday','Monday','Tuesday','Wednesday','Thursday','Friday','Saturday','next Sunday']
        day,seconds=divmod(t,86400);hour,seconds=divmod(seconds,3600);minute,second=divmod(seconds,60)
        return f'{days[day]} {hour:02}:{minute:02}:{second:02}'
    gaps=[];last=0
    for a,b in intervals:
        if a>last:gaps.append((last,a))
        last=max(last,b)
    if last<604800:gaps.append((last,604800))
    render=lambda rows:[{'from':label(a),'to_exclusive':label(b),'duration_seconds':b-a} for a,b in rows]
    return {'status':'recorded_current_snapshot_not_historical','iana_timezone':record.get('scheduleTimeZone'),
        'open_intervals':render(intervals),'closed_intervals':render(gaps),
        'maintenance_vs_weekend_classification':'Closed intervals are observed schedule gaps; business reason not inferred.',
        'swap_first_charge_minutes_utc':record.get('swapTime'),'holiday_rules_recorded':record.get('holiday')}

def execution_audit(data):
    orders={};entry_costs={};deals={};contracts=[];aggregates=[];symbol_names={}
    for path,x in data.items():
        if market(x) or not isinstance(x,dict):continue
        # Replayed/synthetic order paths never enter empirical execution statistics.
        if any(part in str(path).lower() for part in ['/tests/','fixture','replay-']):continue
        account=x.get('ctidTraderAccountId')
        if account is not None:
            for d in x.get('deal',[]):
                if isinstance(d,dict) and d.get('dealId') is not None:
                    key=(int(account),d['dealId']);old=deals.get(key)
                    if old is None or d.get('utcLastUpdateTimestamp',0)>old['record'].get('utcLastUpdateTimestamp',0):
                        deals[key]={'account_id':int(account),'record':d,'source':path}
            for c in x.get('symbol',[]):
                if isinstance(c,dict) and c.get('symbolId') is not None:
                    if c.get('symbolName'):symbol_names[(int(account),c['symbolId'])]=c['symbolName']
                    if 'digits' in c:contracts.append({'account_id':int(account),'record':c,'source':path})
        rows=x.get('rows',[])
        if isinstance(rows,list):
            for row in rows:
                if not isinstance(row,dict):continue
                if all(k in row for k in ['account_id','entry_key','fill_price','created_at']):
                    key=(row['account_id'],row['entry_key'],row['created_at'])
                    if key not in orders or len(json.dumps(row))>len(json.dumps(orders[key]['record'])):
                        orders[key]={'record':row,'source':path}
                elif 'entry_slippage_bps_median' in row:aggregates.append({'source':path,**row})
        for row in x.get('attributed_entries',[]):
            if not isinstance(row,dict):continue
            e=row.get('entry_cost')
            if e and all(k in e for k in ['dispatch_price','fill_price','dispatch_to_fill_bps']) and row.get('side') in ['buy','sell']:
                calc=(1 if row['side']=='buy' else -1)*(e['fill_price']-e['dispatch_price'])/e['dispatch_price']*10000
                if abs(calc-e['dispatch_to_fill_bps'])>1e-8:raise ValueError('entry slippage arithmetic mismatch')
                key=(row['account_id'],row.get('entry_key'),row.get('position_id'))
                entry_costs[key]={'account_id':row['account_id'],'environment':row.get('environment'),
                    'symbol':row['symbol'],'position_id':row.get('position_id'),'side':row['side'],
                    'dispatch_price':e['dispatch_price'],'fill_price':e['fill_price'],'slippage_bps':calc,
                    'volume':row.get('filled_volume'),'at':row.get('created_at'),'source':path,
                    'evidence_kind':'retained_attributed_entry_artifact; original export may be unavailable'}
    positions=collections.defaultdict(list)
    for value in deals.values():positions[(value['account_id'],value['record'].get('positionId'))].append(value)
    position_evidence=[]
    for (account,position),records in sorted(positions.items()):
        opens=[v for v in records if 'closePositionDetail' not in v['record']]
        closes=[v for v in records if 'closePositionDetail' in v['record']]
        ov=sum(v['record'].get('filledVolume',0) for v in opens)
        cv=sum(v['record']['closePositionDetail'].get('closedVolume',0) for v in closes)
        settled=[]
        for v in closes:
            c=v['record']['closePositionDetail'];required=['grossProfit','commission','swap','pnlConversionFee','moneyDigits','quoteToDepositConversionRate','closedVolume']
            if all(k in c for k in required):
                # Preserve actual signed broker money fields; closing commission is the recorded round-trip convention.
                settled.append({'deal_id':v['record']['dealId'],'money_digits':c['moneyDigits'],
                    'gross_profit_native':c['grossProfit'],'commission_native':c['commission'],
                    'swap_native':c['swap'],'conversion_fee_native':c['pnlConversionFee'],
                    'closed_volume':c['closedVolume'],'conversion_rate':c['quoteToDepositConversionRate'],'source':v['source']})
        linked=[v for v in orders.values() if v['record'].get('account_id')==account and v['record'].get('position_id')==position]
        entry_quote=[v for v in entry_costs.values() if v['account_id']==account and v['position_id']==position]
        # None of the compact order schema's spread/slippage fields proves an exit dispatch quote.
        exit_quotes=[v for v in linked if isinstance(v['record'].get('exit_cost'),dict) and all(k in v['record']['exit_cost'] for k in ['dispatch_price','fill_price','dispatch_at','side'])]
        qualified=bool(opens and closes and ov==cv and len(settled)==len(closes) and entry_quote and exit_quotes)
        position_evidence.append({'account_id':account,'environment':ACCOUNT_ENV.get(account,'unverified'),
            'position_id':position,'symbol_ids':sorted({v['record']['symbolId'] for v in records}),
            'opening_deals':len(opens),'closing_deals':len(closes),'opening_volume':ov,'closed_volume':cv,
            'volume_reconciles':bool(opens and ov==cv),'complete_closing_money_records':len(settled),
            'closing_money_evidence':settled,'linked_order_records':len(linked),'entry_dispatch_records':len(entry_quote),
            'exit_dispatch_records':len(exit_quotes),'round_trip_cost_qualified':qualified,
            'missing':[n for n,ok in [('opening_and_closing_volume',bool(opens and closes and ov==cv)),('settled_money',bool(closes and len(settled)==len(closes))),('entry_dispatch_quote',bool(entry_quote)),('exit_dispatch_quote',bool(exit_quotes))] if not ok]})
    empirical_orders=[v for v in orders.values() if v['record'].get('fill_price') is not None]
    for c in contracts:
        c['symbol']=symbol_names.get((c['account_id'],c['record']['symbolId']))
        c['temporal_authority']='recorded current snapshot; not verified historical2023-2025 terms'
        c['source_sha256']=sha(c['source']);c['contract_version']=canonical(c['record'])
        c['schedule_registry']=broker_weekly_schedule(c['record'])
    return {'orders':orders,'entry_costs':list(entry_costs.values()),'contracts':contracts,
        'report':{'unique_order_records':len(orders),'filled_order_records':len(empirical_orders),'raw_deal_records':len(deals),
            'actual_dispatch_to_fill_entries':len(entry_costs),'verified_round_trip_cost_samples':sum(p['round_trip_cost_qualified'] for p in position_evidence),
            'positions':position_evidence,'attributed_entries':list(entry_costs.values()),
            'aggregate_cost_sources':sorted({x['source'] for x in aggregates}),
            'aggregate_warning':'Aggregate medians and code/test fixtures do not create matched round-trip observations.',
            'required_capture':['Account-scoped ProtoOASymbolByIdRes plus symbol-name/asset resolution and capture time.',
                'Opening and closing ProtoOADeal/partial fills, quantities, settled fees, swap and deposit-currency conversion.',
                'Decision and dispatch bid/ask, side, quantity and timestamps for both legs; actual order/ack/fill identity.'],
            'cost_rates':{'24bp':'assumed_research_stress_reserve','2bp':'unverified_sensitivity','3.74bp':'unverified_sensitivity'}}}

def statement_audit(scope):
    """Closing-deal money evidence only: statements contain no dispatch quotes."""
    deals={};sources=[];conflicts=[]
    for item in scope['files']:
        p=Path(item['path'])
        if p.suffix!='.csv':continue
        match=re.search(r'(Demo|Live).*?\((\d+)\).*?statement',p.name)
        if not match:continue
        account=int(match[2]);header=None;section=None;count=0
        with p.open(encoding='utf-8-sig',newline='') as f:
            for line,values in enumerate(csv.reader(f),1):
                if len(values)==1:section=values[0];header=None;continue
                if section!='Deals' or not values:continue
                if header is None:header=values;continue
                if not re.fullmatch(r'DID\d+',values[0]):continue
                if len(values)!=len(header):raise ValueError('statement width mismatch')
                row=dict(zip(header,values));key=(account,int(row['Deal ID'][3:]));count+=1
                def decimal(name):return Decimal(re.sub(r'[\s,]','',row[name]))
                record={'account_id':account,'deal_id':key[1],'symbol':row['Symbol'],
                    'commission_usd':str(decimal('Commissions')),'net_usd':str(decimal('Net USD')),
                    'opening_time_sgt':row['Opening time (UTC+8)'],'closing_time_sgt':row['Closing Time (UTC+8)'],
                    'closing_quantity_reported':row['Closing quantity'],'source':str(p),'line':line}
                if key in deals and any(deals[key][k]!=record[k] for k in ['symbol','commission_usd','net_usd']):
                    conflicts.append({'account_id':account,'deal_id':key[1],'sources':[deals[key]['source'],str(p)]})
                else:deals.setdefault(key,record)
        sources.append({'path':str(p),'sha256':item['sha256'],'account_id':account,'closing_deal_rows':count})
    per=[]
    for account in sorted({k[0] for k in deals}):
        rs=[r for (a,_),r in deals.items() if a==account]
        per.append({'account_id':account,'unique_closed_deal_ids':len(rs),'symbols':sorted({r['symbol'] for r in rs}),
            'recorded_commission_sum_usd':str(sum((Decimal(r['commission_usd']) for r in rs),Decimal(0))),
            'dispatch_to_fill_samples':0})
    return {'files':sources,'unique_closing_deals':len(deals),'per_account':per,'conflicts':conflicts,
        'qualification':'Deduplicated statement closing deals, not full positions. No dispatch quotes or separated historical swap rates; no cost model inferred.'}

def build(scope_path,outdir,binary):
    def deny(event,args):
        if event in ['socket.connect','socket.getaddrinfo']:raise RuntimeError('offline inventory prohibits network')
    sys.addaudithook(deny)
    scope=read(scope_path);reconciliation=reconcile(scope)
    catalog,data,claims,refs,jobs,classes,protocol=index_sources(scope)
    # The standard research account is used only when its frozen plan supplies the identity.
    native=[];episodes=[];segments=[];unregistered=[];failed=[];bar_index=collections.defaultdict(dict);bar_conflicts=[]
    for path,x in sorted(data.items()):
        candidate=market(x)
        if not candidate:
            if isinstance(x,dict) and x.get('status') in ['failed','captured_raw'] and claims[path] and (Path(path).name.startswith(('job-','bar-','peer-'))):
                # Failed sources often intentionally retain only an error, so recover bounds from the frozen plan.
                key=Path(path).stem
                possible=[(origin,j) for options in jobs.values() for origin,j in options if key in [j.get('key'),'job-'+str(j.get('key')),'bar-'+str(j.get('key')),'peer-'+str(j.get('key'))]]
                origin,j=possible[0] if possible else (None,{})
                req=x.get('request',j.get('request',j))
                bounds={'from':req.get('from'),'to':req.get('to')}
                failed.append({'path':path,'status':x['status'],'sha256':catalog[path]['sha256'],'symbol':req.get('symbol'),
                    'request_window_utc':bounds,'plan':origin,'reason':x.get('error','raw_response_not_qualified'),
                    'protected_interval':not permitted(ms(bounds['from']),ms(bounds['to'])) if all(bounds.values()) else None})
            continue
        kind,q,bars,source_from=candidate;req=x.get('request',{});symbol=req.get('symbol')
        if not symbol:unregistered.append({'path':path,'reason':'missing_symbol_identity'});continue
        plans=jobs.get((symbol,req.get('from'),req.get('to')),[])
        plan=next((j for _,j in plans if j.get('symbol_id') is not None),{})
        account=req.get('account',next((j.get('request',{}).get('account',j.get('inventory_context_account')) for _,j in plans if j.get('request',{}).get('account',j.get('inventory_context_account')) is not None),None))
        collection_code_account=account is None and any(part in path for part in ['jiaoyi-jan2024-peer-','jiaoyi-peer-','jiaoyi-three-year-pilot-','jiaoyi-high-vol-','jiaoyi-seasonal-stage1-'])
        if collection_code_account:account=49236051  # verified broker_readonly.ACCOUNT; no broker connection/import
        symbol_id=x.get('symbol_id',x.get('result',{}).get('symbol_id',req.get('symbol_id',plan.get('symbol_id'))))
        if not claims[path]:
            unregistered.append({'path':path,'sha256':catalog[path]['sha256'],'reason':'no_matching_prior_receipt'});continue
        if x.get('status')!='complete':
            failed.append({'path':path,'symbol':symbol,'status':x.get('status'),'sha256':catalog[path]['sha256']});continue
        a,b=ms(source_from or req['from']),ms(req['to'])
        dependencies=bars+x.get('payload',{}).get('peer_m30',[])
        protected=protected_dependency(a,b,dependencies,q if kind=='ticks' else None)
        # Gate on metadata and all dependency timestamps before quote sampling or bar values enter the union.
        base={'source_path':path,'receipt_sha256':catalog[path]['sha256'],'symbol':symbol,'account_id':account,
            'symbol_id':symbol_id,'account_identity_authority':'verified_collection_code_ACCOUNT' if collection_code_account else 'recorded_request_or_frozen_plan',
            'source_window_utc':{'from':stamp(a),'to':stamp(b)},
            'eligible_for_research':not protected,'seal_status':'quarantined_protected_dependency' if protected else 'outside_protected_intervals',
            'receipt_sources':claims[path]}
        if kind=='bars':
            base.update(segment_id=canonical([account,symbol,a,b,catalog[path]['sha256']])[:24],bar_count=len(bars))
            segments.append(base)
        else:
            if not q.get('contract') or not req.get('from') or not req.get('to'):
                unregistered.append({'path':path,'reason':'missing_contract_or_request'});continue
            cls=classes.get(symbol,plan.get('asset_class',plan.get('class','unverified')))
            key=canonical([account,symbol,a,b,catalog[path]['sha256']])[:24]
            flags=sorted(protocol.get((symbol+'-'+req['from'][:10],catalog[path]['sha256']),[]))
            base.update(episode_id=key,date=req['from'][:10],cohort=plan.get('cohort','registered_local_cache'),asset_class=cls,
                tick_receipt_sha256=catalog[path]['sha256'],experiment_window_utc={'from':req['from'],'to':req['to']},
                contract=q['contract'],native_volume=req.get('tick_replay',{}).get('volume',q['contract'].get('min_volume')),
                protocol_flags=flags,
                protocol_status='inherited_repeat_interval' if flags else 'clean_first_read' if any(c['kind']=='collection_ledger' for c in claims[path]) else 'unverified_receipt_history',
                m30_embedded_count=len(bars),_quotes=q,_bars=bars)
            episodes.append(base)
            if not protected:native.append({'key':key,'symbol':symbol,'class':cls,'date':base['date']})
        if not protected:
            for bar in bars:
                if 'ts_ms' not in bar:continue
                bucket=bar_index[(account,symbol)];t=bar['ts_ms']
                if t in bucket and bucket[t][0]!=bar:
                    bar_conflicts.append({'account_id':account,'symbol':symbol,'ts_ms':t,'sources':[bucket[t][1],path]})
                else:bucket[t]=(bar,path)
    for conflict in bar_conflicts:
        bar_index[(conflict['account_id'],conflict['symbol'])].pop(conflict['ts_ms'],None)
    metadata=json.loads(subprocess.check_output([str(binary)],input=json.dumps(native),text=True,timeout=120))
    profiles={};contract_specs=collections.defaultdict(list);samples=collections.defaultdict(dict);duplicates=[];canonical_episodes=[];seen={}
    for ep in episodes:
        q=ep.pop('_quotes');embedded=ep.pop('_bars');account=ep['account_id'];symbol=ep['symbol'];key=ep['episode_id']
        quote_hash=canonical({'bids':q['bids'],'asks':q['asks']});ep['quote_stream_sha256']=quote_hash
        identity=(account,symbol,ep['source_window_utc']['from'],ep['source_window_utc']['to'],quote_hash)
        if identity in seen:
            duplicates.append({'episode_id':key,'canonical_episode_id':seen[identity],'source_path':ep['source_path'],
                'receipt_sha256':ep['receipt_sha256'],'eligible_for_research':ep['eligible_for_research'],
                'seal_status':ep['seal_status'],'interpretation':'identical tick stream; source-specific preparation remains subject to its own seal'});continue
        seen[identity]=key;canonical_episodes.append(ep)
        spec={'account_id':account,'environment':ACCOUNT_ENV.get(account,'unverified'),'symbol_id':ep['symbol_id'],
            'asset_class':ep['asset_class'],'native_volume':ep['native_volume'],**ep['contract'],
            'price_precision':ep['contract']['digits'],'code_price_quantum':float(Fraction(1,10**ep['contract']['digits'])),
            'grid_status':'verified_from_code_and_recorded_prices','contract_time_status':'current_snapshot_not_historical',
            'broker_tick_increment_status':'unverified_requires_broker_spec','point_tick_value_status':'unverified_requires_broker_spec',
            'source':ep['source_path'],'source_sha256':ep['receipt_sha256']}
        if not any(all(v.get(k)==spec[k] for k in ['account_id','symbol_id','digits','min_volume','step_volume','native_volume']) for v in contract_specs[symbol]):contract_specs[symbol].append(spec)
        if not ep['eligible_for_research']:
            ep.update(m30_prep_status='missing_segments',prep_reason='protected_dependency_quarantined',has_post_window_m30_continuation=False);continue
        m=metadata[key];profile=m['profile'];pid=canonical(profile)[:24]
        profiles[pid]={'profile_id':pid,'profile':profile,'authority':'verified_from_code','holidays_included':False,
            'broker_maintenance_breaks':{'status':'unverified_requires_broker_spec','windows':None},
            'historical_45day_broker_bar_count':None}
        ep['calendar_profile_id']=pid;ep['session_window_utc']=m['session'];required=[]
        for session in m['prior_45_calendar_days']:
            start,end=ms(session['open']),ms(session['close']);lunch=session.get('lunch')
            required.extend(t for t in range(start,end,1800000) if t+1800000<=end and not (lunch and ms(lunch[0])<=t<ms(lunch[1])))
        available=bar_index[(account,symbol)];missing=[t for t in required if t not in available]
        ep['m30_prep_status']='complete_session_aware' if profile is not None and required and not missing else 'missing_segments'
        ep['prep_reason']='code_strategy_weekday_slots_only; holidays/broker schedule unverified'
        ep['prior45_strategy_m30_expected']=len(required) if profile is not None else None
        ep['prior45_strategy_m30_missing']=len(missing) if profile is not None else None
        ep['first_missing_strategy_slots_utc']=[stamp(t) for t in missing[:8]]
        end=ms(ep['experiment_window_utc']['to']);times=sorted(t for t in available if t>=end)
        ep['has_post_window_m30_continuation']=bool(times)
        ep['has_post_window_m30_within_72h']=bool(times and times[0]<end+72*3600000)
        contiguous=0
        while end+contiguous*1800000 in available:contiguous+=1
        ep['post_window_m30_contiguous_bars']=contiguous
        ep['first_post_window_m30_utc']=stamp(times[0]) if times else None
        targets={f'window:{offset}':ms(ep['experiment_window_utc']['from'])+offset*60000 for offset in OFFSETS}
        if m['session']:targets.update({f'strategy:{offset}':ms(m['session']['open'])+offset*60000 for offset in OFFSETS})
        points=checkpoints(q['bids'],q['asks'],targets,ep['contract']['digits']);ep['spread_checkpoints']=points
        for name,v in points.items():
            if v is not None:
                # An overlapping/duplicate episode cannot weight the same checkpoint twice.
                sample_id=(account,v['target_ms'],v['quote_ms'],v['bid_e5'],v['ask_e5'])
                samples[(symbol,name)][sample_id]=v
    execs=execution_audit(data)
    execs['report']['statement_evidence']=statement_audit(scope)
    by_symbol={}
    for symbol in sorted(set(CORE)|set(contract_specs)):
        spreads={}
        for anchor in ['strategy','window']:
            spreads[anchor]={}
            for offset in OFFSETS:
                values=list(samples[(symbol,f'{anchor}:{offset}')].values())
                price=quantiles([Fraction(v['spread_e5'],100000) for v in values])
                bps=quantiles([Fraction(20000*v['spread_e5'],v['bid_e5']+v['ask_e5']) for v in values])
                spreads[anchor][str(offset)]={'price_units':price,'bps_mid_denominator':bps,
                    'unique_observations':len(values),'eligible_episode_opportunities':sum(ep['symbol']==symbol and ep['eligible_for_research'] for ep in canonical_episodes)}
        ev=[e for e in execs['entry_costs'] if e['symbol']==symbol]
        by_symbol[symbol]={'contracts':contract_specs[symbol],'spread_quantiles':spreads,
            'recorded_broker_specs':[c for c in execs['contracts'] if c['symbol']==symbol],
            'measured_dispatch_to_fill_entries':len(ev),'entry_slippage_bps_quantiles':quantiles([Fraction(str(e['slippage_bps'])) for e in ev]),
            'commission_and_swap_status':'Historical rates unverified_requires_broker_spec; current snapshot fields and actual settled transaction amounts are separately retained where available.',
            'round_trip_cost_model_calibrated':False}
    registry={'schema':'jiaoyi.calendar-registry.v1','trace':'№18,355','profiles':profiles,
        'compiled_from':'crates/jiaoyi-core/src/strategy3/sessions.rs','code_sha256':sha(ROOT/'crates/jiaoyi-core/src/strategy3/sessions.rs'),
        'native_adapter_sha256':sha(binary),'recorded_broker_contract_snapshots':execs['contracts'],
        'interpretation':'Strategy sessions and recorded current broker schedules are separate authorities. NYMEX/London reference calendar claims require dated specifications; never substitute a guessed fixed offset.',
        'ny_indices_timezone':'America/New_York','london_code_timezone':'Europe/London','crypto_code_timezone':'UTC',
        'nymex_reference_timezone':'America/New_York','nymex_reference_hours_status':'unverified_requires_broker_spec',
        'new_broker_requests':0}
    builder={'python_sha256':sha(Path(__file__)),
        'rust_adapter_source_sha256':sha(ROOT/'crates/jiaoyi-core/examples/replay_inventory_metadata.rs'),
        'rust_adapter_binary_sha256':sha(binary),'cost_definition':'recorded spread in e5; bps relative to contemporaneous mid; type7 exact rational quantiles; no assumed slippage applied'}
    manifest={'schema':'jiaoyi.unified-replay-manifest.v1','trace':'№18,355','scope_sha256':sha(scope_path),'builder':builder,
        'calendar_registry_sha256':canonical(registry),'episodes':canonical_episodes,'duplicate_receipts':duplicates,
        'm30_segments':segments,'m30_conflicts':bar_conflicts,'unregistered_market_candidates':unregistered,'failed_sources':failed,
        'prior_receipt_reconciliation':refs,'frozen_file_reconciliation':reconciliation,
        'summary':{'source_files':len(catalog),'episodes':len(canonical_episodes),'eligible_episodes':sum(e['eligible_for_research'] for e in canonical_episodes),
            'quarantined_episodes':sum(not e['eligible_for_research'] for e in canonical_episodes),
            'quarantined_tick_receipts':sum(not e['eligible_for_research'] for e in canonical_episodes+duplicates),'duplicate_receipts':len(duplicates),
            'm30_segments':len(segments),'prior_reference_statuses':dict(collections.Counter(x['status'] for x in refs)),
            'unregistered_market_candidates':len(unregistered),'failed_sources':len(failed),'symbols':len(by_symbol)},
        'validation_opened':False,'new_broker_requests':0,'live_rules_changed':False,
        'historical_holdout_status':'Legacy protected preparation is quarantined; historical pristine/unseen status is not established.' if any(not e['eligible_for_research'] for e in canonical_episodes) else 'No protected dependency found in registered sources.',
        'coverage_warning':'Any future replay must check its entire lookback/holding/tail interval; future bars existing is not proof of a contiguous72h path.'}
    manifest['prior159_identity_parity']=verify_prior_baseline(manifest,read(ROOT/'docs/native_target_sizing_offline_plan_18353.json'))
    inventory={'schema':'jiaoyi.cost-contract-inventory.v1','trace':'№18,355','scope_sha256':sha(scope_path),'builder':builder,
        'unified_manifest_sha256':canonical(manifest),'symbols':by_symbol,'execution_evidence':execs['report'],
        'code_and_fixture_audit':code_audit(),'methodology':scope['sampling'],
        'classifications':{'measured':'recorded quotes and attributable actual order/deal receipts','synthetic':'test fixtures and native replay outcomes','unverified':'assumed reserve rates, undocumented historical contracts, calendars and complete round-trip costs'},
        'new_broker_requests':0,'validation_opened':False,'financial_pass':False,'live_rules_changed':False}
    reconcile(scope)
    out=Path(outdir);out.mkdir(exist_ok=True,parents=True)
    for name,value in [('rebuild_phase1_session_calendars_18355.json',registry),('rebuild_phase1_unified_replay_manifest_18355.json',manifest),('rebuild_phase1_cost_and_contract_inventory_18355.json',inventory)]:save(out/name,value)
    print(json.dumps({'summary':manifest['summary'],'round_trip_cost_samples':execs['report']['verified_round_trip_cost_samples'],'entry_cost_samples':execs['report']['actual_dispatch_to_fill_entries']},indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);sub=p.add_subparsers(dest='action',required=True)
    f=sub.add_parser('freeze');f.add_argument('--workspace',default='/workspace');f.add_argument('--scope',required=True)
    b=sub.add_parser('build');b.add_argument('--scope',required=True);b.add_argument('--output',required=True);b.add_argument('--binary',default=str(ROOT/'target/debug/examples/replay_inventory_metadata'))
    a=p.parse_args()
    if a.action=='freeze':freeze(a.workspace,a.scope)
    else:build(a.scope,a.output,Path(a.binary))
