//! Account-wide, read-only setup assessment. The independent history session
//! supplies broker evidence; a pure one-second evaluator never awaits IO.
//! Ownership controls management, not observation. Nothing here can send an order.

mod evaluate;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_ctrader::{messages as pb, AccountSession};
use jiaoyi_store::sqlx::{self, PgPool, Row};
use prost::Message;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::{broadcast::error::RecvError, Notify};
use tokio::time::Instant;

use crate::shutdown::Shutdown;
use evaluate::{Input, PendingMargin, Quote, StrategyEvidence, SymbolInfo};

const POLL: Duration = Duration::from_secs(60);
const MAX_OBJECTS: usize = 256;
const MAX_SYMBOLS: usize = 512;
const MAX_TRANSITIONS: usize = 64;

#[derive(Default)]
struct Published {
    generation: u64,
    latest: Value,
}
static REPORTS: OnceLock<Mutex<BTreeMap<i64, Published>>> = OnceLock::new();
fn reports() -> &'static Mutex<BTreeMap<i64, Published>> {
    REPORTS.get_or_init(Default::default)
}

/// Private data is opt-in at the authenticated handler. Public reports expose
/// coverage and timing only, never object IDs, prices, quantities or cash.
pub fn report(accounts: &[i64], private: bool) -> Value {
    let map = reports().lock().unwrap();
    let rows: Vec<Value> = accounts.iter().map(|id| {
        let Some(p) = map.get(id) else {
            return json!({"account_id":id,"state":"starting","assessment_complete":false});
        };
        if private { return p.latest.clone(); }
        let x=&p.latest;
        json!({"account_id":id,"generation":p.generation,"revision":x["revision"],
            "state":x["state"],"assessed_at":x["assessed_at"],"book_at":x["book_at"],
            "evidence_complete":x["evidence_complete"],"position_count":x["position_count"],
            "pending_entry_count":x["pending_entry_count"],"protective_order_count":x["protective_order_count"],
            "breach_count":x["breach_count"],"unknown_count":x["unknown_count"],
            "timing":x["timing"],"source_state":x["source_state"],
            "persistence_state":x["persistence_state"],"no_trade_authority":true})
    }).collect();
    json!({"schema_version":1,"mode":"read_only_setup_assessment","interval_ms":1000,
        "evidence_note":"cached streaming evidence; a timer does not certify fresh broker truth",
        "accounts":rows})
}

struct State {
    input: Input,
    event_seq: u64,
    source_error: Option<&'static str>,
    quotes_received: u64,
    last_event_at: Option<DateTime<Utc>>,
    assessed_event_seq: u64,
    last_event_latency_ms: Option<i64>,
    seen: VecDeque<Vec<u8>>,
    changes: VecDeque<Value>,
    signature: Value,
    persisted_signature: Value,
    last_objects: HashMap<String, Value>,
    persistence_state: &'static str,
    completions: u64,
    missed_deadlines: u64,
    last_completed: Option<Instant>,
    max_interval_ms: u128,
    intervals: VecDeque<u128>,
}

impl State {
    fn new(account: i64, generation: u64) -> Self {
        Self {
            input: Input {
                account_id: account,
                generation,
                revision: 0,
                now: Utc::now(),
                book_at: None,
                complete: false,
                connected: true,
                positions: vec![],
                orders: vec![],
                quotes: HashMap::new(),
                symbols: HashMap::new(),
                trader: None,
                strategy: HashMap::new(),
                margin_floor_pct: None,
                equity: None,
                equity_at: None,
                gross_pnl_by_position: HashMap::new(),
                pending_margin: None,
            },
            event_seq: 0,
            source_error: Some("awaiting_initial_book"),
            quotes_received: 0,
            last_event_at: None,
            assessed_event_seq: 0,
            last_event_latency_ms: None,
            seen: VecDeque::new(),
            changes: VecDeque::new(),
            signature: Value::Null,
            persisted_signature: Value::Null,
            last_objects: HashMap::new(),
            persistence_state: "not_yet_persisted",
            completions: 0,
            missed_deadlines: 0,
            last_completed: None,
            max_interval_ms: 0,
            intervals: VecDeque::new(),
        }
    }

    fn invalidate(&mut self, reason: &'static str) {
        self.event_seq = self.event_seq.saturating_add(1);
        self.input.revision = self.input.revision.saturating_add(1);
        self.input.complete = false;
        self.input.equity = None;
        self.input.gross_pnl_by_position.clear();
        self.input.pending_margin = None;
        self.input.strategy.clear();
        self.source_error = Some(reason);
    }

    fn quote(&mut self, q: &pb::ProtoOaSpotEvent, at: DateTime<Utc>) {
        if q.ctid_trader_account_id != self.input.account_id
            || !self.input.symbols.contains_key(&q.symbol_id)
        {
            return;
        }
        let v = self.input.quotes.entry(q.symbol_id).or_insert(Quote {
            bid: None,
            ask: None,
            bid_at: None,
            ask_at: None,
        });
        if let Some(b) = q.bid.filter(|_| v.bid_at.is_none_or(|old| at >= old)) {
            v.bid = Some(b as f64 / 100_000.0);
            v.bid_at = Some(at);
        }
        if let Some(a) = q.ask.filter(|_| v.ask_at.is_none_or(|old| at >= old)) {
            v.ask = Some(a as f64 / 100_000.0);
            v.ask_at = Some(at);
        }
        self.quotes_received = self.quotes_received.saturating_add(1);
    }

    fn event(&mut self, event: pb::ProtoOaExecutionEvent, at: DateTime<Utc>) -> bool {
        if event.ctid_trader_account_id != self.input.account_id {
            return false;
        }
        let bytes = Sha256::digest(event.encode_to_vec()).to_vec();
        if self.seen.contains(&bytes) {
            return false;
        }
        self.seen.push_back(bytes);
        while self.seen.len() > 128 {
            self.seen.pop_front();
        }
        self.invalidate("broker_change_awaiting_reconciliation");
        self.last_event_at = Some(at);
        if let Some(p) = event.position {
            let old = self
                .input
                .positions
                .iter()
                .position(|x| x.position_id == p.position_id);
            let stale = old.is_some_and(|i| {
                self.input.positions[i]
                    .utc_last_update_timestamp
                    .zip(p.utc_last_update_timestamp)
                    .is_some_and(|(a, b)| b < a)
            });
            if !stale {
                if p.position_status == 1 {
                    // Optional fields in an event may be partial. They remain
                    // provisional/unknown until the complete broker book arrives.
                    if let Some(i) = old {
                        self.input.positions[i] = p;
                    } else if self.input.positions.len() < MAX_OBJECTS {
                        self.input.positions.push(p);
                    }
                } else if let Some(i) = old {
                    self.input.positions.remove(i);
                }
            }
        }
        if let Some(o) = event.order {
            let old = self
                .input
                .orders
                .iter()
                .position(|x| x.order_id == o.order_id);
            let stale = old.is_some_and(|i| {
                self.input.orders[i]
                    .utc_last_update_timestamp
                    .zip(o.utc_last_update_timestamp)
                    .is_some_and(|(a, b)| b < a)
            });
            if !stale {
                if o.order_status == 1 {
                    if let Some(i) = old {
                        self.input.orders[i] = o;
                    } else if self.input.orders.len() < MAX_OBJECTS {
                        self.input.orders.push(o);
                    }
                } else if let Some(i) = old {
                    self.input.orders.remove(i);
                }
            }
        }
        true
    }

    fn account_event(&mut self, payload_type: u32, payload: &[u8], at: DateTime<Utc>) -> bool {
        use pb::ProtoOaPayloadType as Pt;
        let parsed = match payload_type as i32 {
            x if x == Pt::ProtoOaTrailingSlChangedEvent as i32 => {
                pb::ProtoOaTrailingSlChangedEvent::decode(payload).map(|e| {
                    let account = e.ctid_trader_account_id;
                    if account == self.input.account_id {
                        if let Some(p) = self
                            .input
                            .positions
                            .iter_mut()
                            .find(|p| p.position_id == e.position_id)
                        {
                            if p.utc_last_update_timestamp
                                .is_none_or(|old| e.utc_last_update_timestamp >= old)
                            {
                                p.stop_loss = Some(e.stop_price);
                                p.utc_last_update_timestamp = Some(e.utc_last_update_timestamp);
                            }
                        }
                    }
                    account
                })
            }
            x if x == Pt::ProtoOaTraderUpdateEvent as i32 => {
                pb::ProtoOaTraderUpdatedEvent::decode(payload).map(|e| e.ctid_trader_account_id)
            }
            x if x == Pt::ProtoOaMarginChangedEvent as i32 => {
                pb::ProtoOaMarginChangedEvent::decode(payload).map(|e| e.ctid_trader_account_id)
            }
            _ => return false,
        };
        match parsed {
            Ok(account) if account != self.input.account_id => false,
            Ok(_) => {
                self.invalidate("account_or_protection_change");
                self.last_event_at = Some(at);
                true
            }
            Err(_) => {
                self.invalidate("account_event_decode_gap");
                self.last_event_at = Some(at);
                true
            }
        }
    }

    fn install(&mut self, sequence: u64, book: pb::ProtoOaReconcileRes, at: DateTime<Utc>) -> bool {
        if sequence != self.event_seq || book.ctid_trader_account_id != self.input.account_id {
            return false;
        }
        let positions: Vec<_> = book
            .position
            .into_iter()
            .filter(|p| p.position_status == 1)
            .collect();
        if positions.len() > MAX_OBJECTS || book.order.len() > MAX_OBJECTS {
            self.source_error = Some("book_exceeds_bounded_capacity");
            self.input.complete = false;
            return false;
        }
        let unique_positions: HashSet<_> = positions.iter().map(|p| p.position_id).collect();
        let unique_orders: HashSet<_> = book.order.iter().map(|o| o.order_id).collect();
        if unique_positions.len() != positions.len()
            || unique_orders.len() != book.order.len()
            || unique_positions.iter().any(|id| *id <= 0)
            || unique_orders.iter().any(|id| *id <= 0)
        {
            self.source_error = Some("invalid_book_identity");
            self.input.complete = false;
            return false;
        }
        self.input.positions = positions;
        self.input.orders = book.order;
        self.input.strategy.clear();
        self.input.pending_margin = None;
        self.input.book_at = Some(at);
        self.input.complete = true;
        self.input.revision = self.input.revision.saturating_add(1);
        self.source_error = None;
        true
    }
}

type Shared = Arc<Mutex<State>>;

struct SessionGuard {
    state: Shared,
}
impl Drop for SessionGuard {
    fn drop(&mut self) {
        {
            let mut s = self.state.lock().unwrap();
            s.input.connected = false;
            s.invalidate("disconnected");
        }
        publish(&self.state);
    }
}

fn fresh_mid(q: &Quote, now: DateTime<Utc>) -> Option<(f64, DateTime<Utc>)> {
    let (b, a, bt, at) = (q.bid?, q.ask?, q.bid_at?, q.ask_at?);
    let oldest = bt.min(at);
    (b > 0.0
        && a >= b
        && a.is_finite()
        && oldest <= now
        && (now - oldest).num_milliseconds() <= 6000)
        .then_some(((b + a) / 2.0, oldest))
}

fn conversion_rates(input: &mut Input) {
    let Some(deposit) = input.trader.as_ref().map(|t| t.deposit_asset_id) else {
        return;
    };
    let direct: Vec<_> = input
        .symbols
        .values()
        .filter_map(|m| {
            Some((
                m.light.base_asset_id?,
                m.light.quote_asset_id?,
                m.symbol.symbol_id,
            ))
        })
        .collect();
    for m in input.symbols.values_mut() {
        m.quote_to_deposit = None;
        m.conversion_at = None;
        let Some(quote) = m.light.quote_asset_id else {
            continue;
        };
        if quote == deposit {
            m.quote_to_deposit = Some(1.0);
            m.conversion_at = Some(input.now);
            continue;
        }
        for (base, q, id) in &direct {
            let Some((mid, at)) = input.quotes.get(id).and_then(|q| fresh_mid(q, input.now)) else {
                continue;
            };
            let rate = if *base == quote && *q == deposit {
                Some(mid)
            } else if *base == deposit && *q == quote {
                Some(1.0 / mid)
            } else {
                None
            };
            if rate.is_some_and(|r| r.is_finite() && r > 0.0) {
                m.quote_to_deposit = rate;
                m.conversion_at = Some(at);
                break;
            }
        }
    }
}

fn material(v: &Value) -> Value {
    let checks = |x: &Value| -> Vec<Value> {
        x.as_array()
            .map(|a| {
                a.iter()
                    .map(|c| json!({"name":c["name"],"state":c["state"],"reason":c["reason"]}))
                    .collect()
            })
            .unwrap_or_default()
    };
    let objects:Vec<Value>=v["objects"].as_array().map(|a|a.iter().map(|o|json!({
        "kind":o["kind"],"id":o["id"],"side":o["side"],"volume":o["volume"],"entry":o["entry"],
        "stop_loss":o["stop_loss"],"take_profit":o["take_profit"],"strategy_key":o["strategy"]["entry_key"],
        "checks":checks(&o["checks"])})).collect()).unwrap_or_default();
    json!({"state":v["state"],"source_state":v["source_state"],"objects":objects,"checks":checks(&v["account_checks"])})
}

fn publish(shared: &Shared) {
    let mut s = shared.lock().unwrap();
    s.input.now = Utc::now();
    let mut input = s.input.clone();
    conversion_rates(&mut input);
    let mut value = serde_json::to_value(evaluate::evaluate(&input)).unwrap();
    let now = Instant::now();
    if let Some(last) = s.last_completed {
        let elapsed = now.duration_since(last).as_millis();
        s.max_interval_ms = s.max_interval_ms.max(elapsed);
        if elapsed > 1500 {
            s.missed_deadlines = s
                .missed_deadlines
                .saturating_add((elapsed / 1000).saturating_sub(1) as u64);
        }
        s.intervals.push_back(elapsed);
        while s.intervals.len() > 300 {
            s.intervals.pop_front();
        }
    }
    s.last_completed = Some(now);
    s.completions = s.completions.saturating_add(1);
    value["state"] = json!(if !input.connected {
        "disconnected"
    } else if input.complete {
        "assessed"
    } else {
        "provisional"
    });
    value["source_state"] = json!(s.source_error.unwrap_or("observed"));
    value["last_event_at"] = json!(s.last_event_at);
    let sig = material(&value);
    let current: HashMap<String, Value> = value["objects"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|o| (format!("{}:{}", o["kind"], o["id"]), o.clone()))
        .collect();
    if sig != s.signature {
        let previous = s.last_objects.clone();
        for (key, old) in &previous {
            if input.complete && !current.contains_key(key) {
                s.changes.push_back(json!({"at":input.now,"kind":"object_no_longer_observed","last_assessment":old,"book_revision":input.revision}));
            }
        }
        for (key, object) in &current {
            let fingerprint = |o: &Value| material(&json!({"objects":[o]}));
            if previous
                .get(key)
                .is_none_or(|old| fingerprint(old) != fingerprint(object))
            {
                s.changes.push_back(json!({"at":input.now,"kind":"object_assessment_changed","revision":input.revision,"assessment":object}));
            }
        }
        // One compact account record; individual changes keep a bounded
        // last-assessment trail without copying the entire book 64 times.
        s.changes.push_back(json!({"at":input.now,"kind":"account_assessment_changed","revision":input.revision,
            "state":value["state"],"source_state":value["source_state"],"breach_count":value["breach_count"],"unknown_count":value["unknown_count"]}));
        while s.changes.len() > MAX_TRANSITIONS {
            s.changes.pop_front();
        }
        s.signature = material(&value);
    }
    if input.complete {
        s.last_objects = current;
    } else {
        s.last_objects.extend(current);
    }
    if s.assessed_event_seq != s.event_seq {
        s.last_event_latency_ms = s
            .last_event_at
            .map(|at| (input.now - at).num_milliseconds());
        s.assessed_event_seq = s.event_seq;
    }
    let mut intervals: Vec<_> = s.intervals.iter().copied().collect();
    intervals.sort_unstable();
    let pct = |p: usize| {
        intervals
            .get((intervals.len().saturating_sub(1) * p) / 100)
            .copied()
    };
    value["timing"] = json!({"completed":s.completions,"missed_deadlines":s.missed_deadlines,
        "max_interval_ms":s.max_interval_ms,"p95_interval_ms":pct(95),"p99_interval_ms":pct(99),
        "quotes_received":s.quotes_received,"interval_sample_count":intervals.len(),
        "last_event_to_assessment_ms":s.last_event_latency_ms,
        "last_event_age_ms":s.last_event_at.map(|at|(input.now-at).num_milliseconds())});
    value["transitions"] = json!(s.changes);
    value["persistence_state"] = json!(s.persistence_state);
    let mut map = reports().lock().unwrap();
    if let Some(slot) = map.get_mut(&input.account_id) {
        if slot.generation == input.generation {
            slot.latest = value;
        }
    }
}

async fn strategy_evidence(
    pool: &PgPool,
    account: i64,
    positions: &[pb::ProtoOaPosition],
) -> HashMap<i64, StrategyEvidence> {
    let mut out = HashMap::new();
    for p in positions {
        let Ok(a) = crate::authority::position_authority(pool, account, p.position_id).await else {
            continue;
        };
        if a.validate_current(p).is_err() {
            continue;
        }
        let query = if a.entry_key.starts_with("s3:") {
            "SELECT 's3'::text AS strategy, side, detail, decided_at AS at, volume, entry_key FROM strategy3_orders WHERE account_id=$1 AND entry_key=$2 AND kind='open'"
        } else {
            "SELECT 's1'::text AS strategy,i.side,COALESCE((SELECT d.detail FROM trading_decisions d WHERE d.account_id=i.account_id AND d.detail->>'entry_key'=i.entry_key AND d.kind='sent' ORDER BY d.id DESC LIMIT 1),'{}'::jsonb) || jsonb_build_object('opening_max_risk',i.max_risk,'original_relative_stop',i.relative_stop) AS detail,i.created_at AS at,i.volume,i.entry_key FROM trading_intents i WHERE i.account_id=$1 AND i.entry_key=$2 AND i.kind='open'"
        };
        let Ok(Some(r)) = sqlx::query(query)
            .bind(account)
            .bind(&a.entry_key)
            .fetch_optional(pool)
            .await
        else {
            continue;
        };
        let Some(volume) = r.get::<Option<i64>, _>("volume") else {
            continue;
        };
        let Some(side) = r.get::<Option<String>, _>("side") else {
            continue;
        };
        out.insert(
            p.position_id,
            StrategyEvidence {
                strategy: r.get("strategy"),
                side,
                detail: r.get("detail"),
                recorded_at: r.get("at"),
                original_volume: volume,
                entry_key: a.entry_key,
            },
        );
    }
    out
}

#[allow(clippy::collapsible_if)]
fn equity(
    trader: &pb::ProtoOaTrader,
    book: &pb::ProtoOaReconcileRes,
    pnl: &pb::ProtoOaGetPositionUnrealizedPnLRes,
) -> Option<f64> {
    if trader.ctid_trader_account_id != book.ctid_trader_account_id
        || pnl.ctid_trader_account_id != book.ctid_trader_account_id
    {
        return None;
    }
    let digits = trader.money_digits.filter(|d| *d <= 18)?;
    if pnl.money_digits > 18 {
        return None;
    }
    let ids: HashSet<_> = book
        .position
        .iter()
        .filter(|p| p.position_status == 1)
        .map(|p| p.position_id)
        .collect();
    let mut seen = HashSet::new();
    let mut value = trader.balance as f64 / 10f64.powi(digits as i32);
    for p in &pnl.position_unrealized_pn_l {
        if !seen.insert(p.position_id) {
            return None;
        }
        if !ids.contains(&p.position_id) {
            if p.net_unrealized_pn_l != 0
                || p.gross_unrealized_pn_l != 0
                || !book.order.iter().any(|o| {
                    o.position_id == Some(p.position_id)
                        && !o.closing_order.unwrap_or(false)
                        && o.order_type != 4
                })
            {
                return None;
            }
        }
        value += p.net_unrealized_pn_l as f64 / 10f64.powi(pnl.money_digits as i32);
    }
    (ids.is_subset(&seen) && value.is_finite()).then_some(value)
}

async fn collect(
    session: &AccountSession<'_>,
    state: &Shared,
    wake: &Notify,
    context_wake: &Notify,
    margin_wake: &Notify,
    mut stop: Shutdown,
) {
    let mut listed: HashMap<i64, pb::ProtoOaLightSymbol> = HashMap::new();
    let mut subscribed = HashSet::new();
    let mut timer = tokio::time::interval(POLL);
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {_ = stop.wait()=>return,_=timer.tick()=>{},_=wake.notified()=>{}}
        let result = async {
            let sequence = state.lock().unwrap().event_seq;
            let book = session.margin_book().await?;
            let at = Utc::now();
            let trader = session.trader().await?;
            let pnl = session.unrealized_pnl().await?;
            let pnl_at = Utc::now();
            let held: HashSet<i64> = book
                .position
                .iter()
                .map(|p| p.trade_data.symbol_id)
                .chain(book.order.iter().map(|o| o.trade_data.symbol_id))
                .collect();
            if listed.is_empty() || held.iter().any(|id| !listed.contains_key(id)) {
                listed = session
                    .symbols_list()
                    .await?
                    .symbol
                    .into_iter()
                    .map(|s| (s.symbol_id, s))
                    .collect();
            }
            let mut needed = held.clone();
            for id in &held {
                let Some(quote) = listed.get(id).and_then(|s| s.quote_asset_id) else {
                    continue;
                };
                if quote == trader.deposit_asset_id {
                    continue;
                }
                if let Some(m) = listed
                    .values()
                    .filter(|m| m.enabled != Some(false))
                    .find(|m| {
                        (m.base_asset_id == Some(quote)
                            && m.quote_asset_id == Some(trader.deposit_asset_id))
                            || (m.quote_asset_id == Some(quote)
                                && m.base_asset_id == Some(trader.deposit_asset_id))
                    })
                {
                    needed.insert(m.symbol_id);
                }
            }
            if needed.len() > MAX_SYMBOLS {
                anyhow::bail!("assessment symbol capacity exceeded");
            }
            let ids: Vec<_> = needed.iter().copied().collect();
            let symbols = if ids.is_empty() {
                vec![]
            } else {
                session.symbols_by_id(&ids).await?.symbol
            };
            {
                let mut s = state.lock().unwrap();
                for m in symbols {
                    if let Some(light) = listed.get(&m.symbol_id) {
                        s.input.symbols.insert(
                            m.symbol_id,
                            SymbolInfo {
                                symbol: m,
                                light: light.clone(),
                                quote_to_deposit: None,
                                conversion_at: None,
                            },
                        );
                    }
                }
                s.input.symbols.retain(|id, _| needed.contains(id));
                s.input.quotes.retain(|id, _| needed.contains(id));
            }
            let new: Vec<_> = needed.difference(&subscribed).copied().collect();
            if !new.is_empty() {
                session.subscribe_spots(&new).await?;
                subscribed.extend(new);
            }
            // This connection is the history reader, not a strategy reader.
            // Only subscriptions installed by this observer are removed.
            let removed: Vec<_> = subscribed.difference(&needed).copied().collect();
            if !removed.is_empty() {
                session.unsubscribe_spots(&removed).await?;
                for id in removed {
                    subscribed.remove(&id);
                }
            }
            let observed_equity = equity(&trader, &book, &pnl);
            let accepted = {
                let mut s = state.lock().unwrap();
                let ok = s.install(sequence, book, at);
                if ok {
                    s.input.trader = Some(trader);
                    s.input.equity = observed_equity;
                    s.input.equity_at = Some(pnl_at);
                    s.input.gross_pnl_by_position = if observed_equity.is_some() {
                        pnl.position_unrealized_pn_l
                            .iter()
                            .map(|p| {
                                (
                                    p.position_id,
                                    p.gross_unrealized_pn_l as f64
                                        / 10f64.powi(pnl.money_digits as i32),
                                )
                            })
                            .collect()
                    } else {
                        HashMap::new()
                    };
                }
                ok
            };
            if !accepted {
                wake.notify_one();
                return Ok::<(), anyhow::Error>(());
            }
            context_wake.notify_one();
            margin_wake.notify_one();
            Ok(())
        }
        .await;
        if result.is_err() {
            let mut s = state.lock().unwrap();
            s.input.complete = false;
            s.source_error = Some("source_read_failed");
        }
        // Coalesce change bursts; no overlapping broker reads or retry spin.
        tokio::select! {_=stop.wait()=>return,_=tokio::time::sleep(Duration::from_millis(250))=>{}}
    }
}

/// Same gross-side aggregation and conservative projected-margin equation as
/// the existing entry gate, without adding a candidate order or any authority.
async fn pending_estimate(
    session: &AccountSession<'_>,
    input: &Input,
) -> anyhow::Result<PendingMargin> {
    let mut current = BTreeMap::<i64, [i64; 2]>::new();
    let mut future = BTreeMap::<i64, [i64; 2]>::new();
    let mut by_symbol = BTreeMap::<i64, Vec<(i64, i64, usize)>>::new();
    let add = |map: &mut BTreeMap<i64, [i64; 2]>,
               id: i64,
               side: i32,
               volume: i64|
     -> anyhow::Result<usize> {
        let index = match side {
            1 => 0,
            2 => 1,
            _ => anyhow::bail!("invalid side"),
        };
        if id <= 0 || volume <= 0 {
            anyhow::bail!("invalid exposure");
        }
        let v = &mut map.entry(id).or_default()[index];
        *v = v
            .checked_add(volume)
            .ok_or_else(|| anyhow::anyhow!("volume overflow"))?;
        Ok(index)
    };
    let mut used = 0.0;
    for p in &input.positions {
        add(
            &mut current,
            p.trade_data.symbol_id,
            p.trade_data.trade_side,
            p.trade_data.volume,
        )?;
        let digits = p
            .money_digits
            .filter(|d| *d <= 18)
            .ok_or_else(|| anyhow::anyhow!("margin precision absent"))?;
        used += p
            .used_margin
            .ok_or_else(|| anyhow::anyhow!("used margin absent"))? as f64
            / 10f64.powi(digits as i32);
    }
    future.extend(current.clone());
    for o in input
        .orders
        .iter()
        .filter(|o| o.order_status == 1 && o.closing_order != Some(true) && o.order_type != 4)
    {
        let filled = o
            .executed_volume
            .filter(|v| *v >= 0 && *v <= o.trade_data.volume)
            .ok_or_else(|| anyhow::anyhow!("pending quantity incomplete"))?;
        let remaining = o.trade_data.volume - filled;
        let side = add(
            &mut future,
            o.trade_data.symbol_id,
            o.trade_data.trade_side,
            remaining,
        )?;
        by_symbol
            .entry(o.trade_data.symbol_id)
            .or_default()
            .push((o.order_id, remaining, side));
    }
    if by_symbol.is_empty() {
        anyhow::bail!("no pending exposure");
    }
    let observed_at = Utc::now();
    let mut initial_margin = 0.0;
    let mut final_margin = 0.0;
    let mut by_order = HashMap::new();
    for (symbol, amounts) in &future {
        let before = current.get(symbol).copied().unwrap_or_default();
        let volumes: BTreeSet<i64> = before
            .into_iter()
            .chain(*amounts)
            .chain(
                by_symbol
                    .get(symbol)
                    .into_iter()
                    .flatten()
                    .map(|(_, v, _)| *v),
            )
            .filter(|v| *v > 0)
            .collect();
        let reply = session
            .expected_margin(*symbol, volumes.into_iter().collect())
            .await?;
        let digits = reply
            .money_digits
            .filter(|d| *d <= 18)
            .ok_or_else(|| anyhow::anyhow!("expected-margin precision absent"))?;
        let value = |volume: i64, side: usize| -> anyhow::Result<f64> {
            if volume == 0 {
                return Ok(0.0);
            }
            let rows: Vec<_> = reply.margin.iter().filter(|m| m.volume == volume).collect();
            if rows.len() != 1 {
                anyhow::bail!("missing/duplicate expected margin");
            }
            let raw = if side == 0 {
                rows[0].buy_margin
            } else {
                rows[0].sell_margin
            };
            if raw < 0 {
                anyhow::bail!("negative expected margin");
            }
            Ok(raw as f64 / 10f64.powi(digits as i32))
        };
        for side in 0..2 {
            initial_margin += value(before[side], side)?;
            final_margin += value(amounts[side], side)?;
        }
        for (order, volume, side) in by_symbol.get(symbol).into_iter().flatten() {
            by_order.insert(*order, value(*volume, *side)?);
        }
    }
    if !final_margin.is_finite() || final_margin < initial_margin {
        anyhow::bail!("invalid gross exposure estimate");
    }
    let projected_margin = final_margin.max(used + (final_margin - initial_margin));
    let warning_pct = session.margin_calls().await.ok().and_then(|r| {
        r.margin_call
            .into_iter()
            .map(|c| c.margin_level_threshold)
            .filter(|v| v.is_finite() && *v > 0.0)
            .reduce(f64::max)
    });
    let after = session.margin_book().await?;
    let mut before_positions = input.positions.clone();
    let mut after_positions: Vec<_> = after
        .position
        .into_iter()
        .filter(|p| p.position_status == 1)
        .collect();
    before_positions.sort_by_key(|p| p.position_id);
    after_positions.sort_by_key(|p| p.position_id);
    let mut before_orders = input.orders.clone();
    let mut after_orders = after.order;
    before_orders.sort_by_key(|o| o.order_id);
    after_orders.sort_by_key(|o| o.order_id);
    let trader = session.trader().await?;
    if before_positions != after_positions
        || before_orders != after_orders
        || input.trader.as_ref() != Some(&trader)
    {
        anyhow::bail!("book changed during projection");
    }
    Ok(PendingMargin {
        projected_margin,
        observed_at,
        revision: input.revision,
        warning_pct,
        by_order,
    })
}

async fn pending_margins(
    session: &AccountSession<'_>,
    state: &Shared,
    wake: &Notify,
    mut stop: Shutdown,
) {
    loop {
        tokio::select! {_=stop.wait()=>return,_=wake.notified()=>{}}
        let input = state.lock().unwrap().input.clone();
        if !input.complete {
            continue;
        }
        let estimate = pending_estimate(session, &input).await.ok();
        let mut s = state.lock().unwrap();
        if s.input.revision == input.revision {
            s.input.pending_margin = estimate;
        }
    }
}

async fn context(pool: &PgPool, state: &Shared, wake: &Notify, mut stop: Shutdown) {
    loop {
        tokio::select! {_=stop.wait()=>return,_=wake.notified()=>{}}
        let (account, revision, positions) = {
            let s = state.lock().unwrap();
            (
                s.input.account_id,
                s.input.revision,
                s.input.positions.clone(),
            )
        };
        let strategy = strategy_evidence(pool, account, &positions).await;
        let floor = jiaoyi_store::settings::snapshot(pool, Some(account))
            .await
            .ok()
            .and_then(|s| s.number("trading.margin_level_floor_pct").ok());
        let mut s = state.lock().unwrap();
        if s.input.revision == revision {
            s.input.strategy = strategy;
            s.input.margin_floor_pct = floor;
        }
    }
}

async fn persist(pool: &PgPool, state: &Shared, mut stop: Shutdown) {
    let account = state.lock().unwrap().input.account_id;
    if let Ok(Some(row)) =
        sqlx::query("SELECT detail FROM setup_assessment_snapshots WHERE account_id=$1")
            .bind(account)
            .fetch_optional(pool)
            .await
    {
        let detail: Value = row.get("detail");
        let mut s = state.lock().unwrap();
        let mut prior: VecDeque<_> = detail["transitions"]
            .as_array()
            .into_iter()
            .flatten()
            .rev()
            .take(MAX_TRANSITIONS)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        prior.append(&mut s.changes);
        while prior.len() > MAX_TRANSITIONS {
            prior.pop_front();
        }
        s.changes = prior;
        // Only historical transitions are restored. A saved book never
        // certifies current positions or acquires management authority.
    }
    // Make restored history part of the first durable replacement, even if
    // the only current observation is still an empty bootstrap snapshot.
    publish(state);
    let mut timer = tokio::time::interval(Duration::from_secs(5));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {_=stop.wait()=>return,_=timer.tick()=>{}}
        let (account, signature, dirty) = {
            let s = state.lock().unwrap();
            (
                s.input.account_id,
                s.signature.clone(),
                s.signature != s.persisted_signature,
            )
        };
        if !dirty {
            continue;
        }
        let latest = reports()
            .lock()
            .unwrap()
            .get(&account)
            .map(|p| p.latest.clone())
            .unwrap_or(Value::Null);
        if latest.is_null() {
            continue;
        }
        let ok=sqlx::query("INSERT INTO setup_assessment_snapshots(account_id,observed_at,detail) VALUES($1,$2,$3) ON CONFLICT(account_id) DO UPDATE SET observed_at=EXCLUDED.observed_at,detail=EXCLUDED.detail")
            .bind(account).bind(Utc::now()).bind(latest).execute(pool).await.is_ok();
        let mut s = state.lock().unwrap();
        s.persistence_state = if ok { "saved" } else { "write_failed" };
        if ok {
            s.persisted_signature = signature;
        }
    }
}

/// Runs concurrently with history/deal work on its existing read-only session.
/// The lifetime guard marks evidence disconnected on cancellation or reconnect.
pub async fn run_session(pool: &PgPool, session: &AccountSession<'_>, mut shutdown: Shutdown) {
    let account = session.account().id();
    let generation = {
        let mut m = reports().lock().unwrap();
        let p = m.entry(account).or_default();
        p.generation = p.generation.saturating_add(1);
        p.generation
    };
    let shared = Arc::new(Mutex::new(State::new(account, generation)));
    let _guard = SessionGuard {
        state: shared.clone(),
    };
    let wake = Notify::new();
    let context_wake = Notify::new();
    let margin_wake = Notify::new();
    let mut spots = session.connection().spot_events();
    let mut executions = session.connection().executions();
    let mut ends = session.connection().session_ends();
    let read = collect(
        session,
        &shared,
        &wake,
        &context_wake,
        &margin_wake,
        shutdown.clone(),
    );
    let evidence = context(pool, &shared, &context_wake, shutdown.clone());
    let margin = pending_margins(session, &shared, &margin_wake, shutdown.clone());
    let save = persist(pool, &shared, shutdown.clone());
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    tokio::pin!(read, save, evidence, margin);
    loop {
        tokio::select! {
            _=shutdown.wait()=>return,
            _=&mut read=>return,
            _=&mut save=>return,
            _=&mut evidence=>return,
            _=&mut margin=>return,
            _=tick.tick()=>publish(&shared),
            e=ends.recv()=>match e {Ok(e) if e.affects(account)=>return,Err(RecvError::Closed)=>return,_=>{}},
            q=spots.recv()=>match q {
                Ok(q)=>shared.lock().unwrap().quote(&q.event,q.received_at),
                Err(RecvError::Lagged(_))=>{shared.lock().unwrap().input.quotes.clear();wake.notify_one();},
                Err(RecvError::Closed)=>return,
            },
            e=executions.recv()=>match e {
                Ok(e) if e.payload_type==pb::ProtoOaPayloadType::ProtoOaExecutionEvent as u32=>{
                    match pb::ProtoOaExecutionEvent::decode(e.payload.as_slice()) {
                        Ok(event)=>if shared.lock().unwrap().event(event,e.received_at) {wake.notify_one();publish(&shared);},
                        Err(_)=>{shared.lock().unwrap().invalidate("execution_decode_gap");wake.notify_one();},
                    }
                },
                Ok(e)=>{let changed=shared.lock().unwrap().account_event(e.payload_type,&e.payload,e.received_at);if changed{wake.notify_one();publish(&shared);}},
                Err(RecvError::Lagged(_))=>{shared.lock().unwrap().invalidate("execution_stream_gap");wake.notify_one();publish(&shared);},
                Err(RecvError::Closed)=>return,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn position(id: i64, volume: i64) -> pb::ProtoOaPosition {
        pb::ProtoOaPosition {
            position_id: id,
            position_status: 1,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: 5,
                volume,
                trade_side: 1,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn book(volume: i64) -> pb::ProtoOaReconcileRes {
        pb::ProtoOaReconcileRes {
            ctid_trader_account_id: 7,
            position: vec![position(12, volume)],
            ..Default::default()
        }
    }
    #[test]
    fn newer_manual_change_wins_over_an_inflight_snapshot() {
        let mut s = State::new(7, 1);
        assert!(s.install(0, book(100), Utc::now()));
        let start = s.event_seq;
        assert!(s.event(
            pb::ProtoOaExecutionEvent {
                ctid_trader_account_id: 7,
                position: Some(position(12, 200)),
                ..Default::default()
            },
            Utc::now()
        ));
        assert!(!s.install(start, book(100), Utc::now()));
        assert_eq!(s.input.positions[0].trade_data.volume, 200);
        assert!(!s.input.complete);
        assert!(s.install(s.event_seq, book(200), Utc::now()));
        assert!(s.input.complete);
    }
    #[test]
    fn duplicate_events_do_not_double_count_or_revise_again() {
        let mut s = State::new(7, 1);
        let e = pb::ProtoOaExecutionEvent {
            ctid_trader_account_id: 7,
            position: Some(position(12, 200)),
            ..Default::default()
        };
        assert!(s.event(e.clone(), Utc::now()));
        let revision = s.input.revision;
        assert!(!s.event(e, Utc::now()));
        assert_eq!(s.input.positions.len(), 1);
        assert_eq!(s.input.revision, revision);
    }
    #[test]
    fn material_fingerprint_excludes_tick_money_and_time_but_keeps_manual_edits() {
        let mut a = json!({"state":"assessed","source_state":"observed","objects":[{"id":4,"kind":"position","volume":100,"stop_loss":2.0,"checks":[{"name":"risk","state":"pass","reason":"within_limit","value":2.0}]}],"assessed_at":"old"});
        let before = material(&a);
        a["assessed_at"] = json!("new");
        a["objects"][0]["checks"][0]["value"] = json!(8.0);
        assert_eq!(before, material(&a));
        a["objects"][0]["stop_loss"] = json!(3.0);
        assert_ne!(before, material(&a));
    }
    #[test]
    fn equity_requires_exact_position_coverage_and_money_precision() {
        let t = pb::ProtoOaTrader {
            ctid_trader_account_id: 7,
            balance: 10000,
            money_digits: Some(2),
            ..Default::default()
        };
        let mut pnl = pb::ProtoOaGetPositionUnrealizedPnLRes {
            ctid_trader_account_id: 7,
            money_digits: 2,
            ..Default::default()
        };
        assert_eq!(equity(&t, &book(100), &pnl), None);
        pnl.position_unrealized_pn_l
            .push(pb::ProtoOaPositionUnrealizedPnL {
                position_id: 12,
                net_unrealized_pn_l: -100,
                gross_unrealized_pn_l: -100,
            });
        assert_eq!(equity(&t, &book(100), &pnl), Some(99.0));
    }

    #[tokio::test]
    async fn pending_projection_matches_broker_side_and_deduplicates_volume_reads() {
        use jiaoyi_core::{allowlist::Allowlist, secret::Secret};
        use jiaoyi_ctrader::testkit::FakeBroker;
        let account = 706;
        let trader = pb::ProtoOaTrader {
            ctid_trader_account_id: account,
            balance: 100000,
            money_digits: Some(2),
            deposit_asset_id: 1,
            ..Default::default()
        };
        let order = |id, volume, side| pb::ProtoOaOrder {
            order_id: id,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: 5,
                volume,
                trade_side: side,
                ..Default::default()
            },
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            ..Default::default()
        };
        let b = pb::ProtoOaReconcileRes {
            ctid_trader_account_id: account,
            order: vec![order(1, 100, 1), order(2, 200, 2)],
            ..Default::default()
        };
        let (conn, log) = FakeBroker {
            account_id: account,
            trader: Some(trader.clone()),
            scripted_books: Arc::new(Mutex::new(VecDeque::from([b.clone()]))),
            margin_reply: Some(pb::ProtoOaExpectedMarginRes {
                ctid_trader_account_id: account,
                money_digits: Some(2),
                margin: vec![
                    pb::ProtoOaExpectedMargin {
                        volume: 100,
                        buy_margin: 600,
                        sell_margin: 400,
                    },
                    pb::ProtoOaExpectedMargin {
                        volume: 200,
                        buy_margin: 1200,
                        sell_margin: 1100,
                    },
                ],
                ..Default::default()
            }),
            ..Default::default()
        }
        .spawn();
        conn.application_auth("id", &Secret::new("test"))
            .await
            .unwrap();
        let list = Allowlist::parse(Some("706")).unwrap();
        let session = conn
            .account_auth(&list.permit(account).unwrap(), &Secret::new("test"))
            .await
            .unwrap();
        let mut s = State::new(account, 1);
        assert!(s.install(0, b, Utc::now()));
        s.input.trader = Some(trader);
        let estimated = pending_estimate(&session, &s.input).await.unwrap();
        assert_eq!(estimated.projected_margin, 17.0);
        assert_eq!(estimated.by_order.get(&1), Some(&6.0));
        assert_eq!(estimated.by_order.get(&2), Some(&11.0));
        let log = log.lock().unwrap();
        assert_eq!(log.margin_requests, vec![(5, vec![100, 200])]);
        assert!(log.orders.is_empty());
    }

    #[test]
    fn manual_protection_and_account_events_invalidate_immediately() {
        let mut s = State::new(7, 1);
        assert!(s.install(0, book(100), Utc::now()));
        let event = pb::ProtoOaTrailingSlChangedEvent {
            ctid_trader_account_id: 7,
            position_id: 12,
            order_id: 91,
            stop_price: 99.0,
            utc_last_update_timestamp: 100,
            ..Default::default()
        };
        assert!(s.account_event(
            pb::ProtoOaPayloadType::ProtoOaTrailingSlChangedEvent as u32,
            &event.encode_to_vec(),
            Utc::now()
        ));
        assert_eq!(s.input.positions[0].stop_loss, Some(99.0));
        assert!(!s.input.complete);
        let other = pb::ProtoOaMarginChangedEvent {
            ctid_trader_account_id: 8,
            ..Default::default()
        };
        let revision = s.input.revision;
        assert!(!s.account_event(
            pb::ProtoOaPayloadType::ProtoOaMarginChangedEvent as u32,
            &other.encode_to_vec(),
            Utc::now()
        ));
        assert_eq!(s.input.revision, revision);
        let ours = pb::ProtoOaTraderUpdatedEvent {
            ctid_trader_account_id: 7,
            ..Default::default()
        };
        assert!(s.account_event(
            pb::ProtoOaPayloadType::ProtoOaTraderUpdateEvent as u32,
            &ours.encode_to_vec(),
            Utc::now()
        ));
        assert!(s.input.revision > revision);
    }

    #[test]
    fn duplicate_book_ids_are_unknown_and_replacement_clears_old_strategy() {
        let mut s = State::new(7, 1);
        let mut b = book(100);
        b.position.push(position(12, 100));
        assert!(!s.install(0, b, Utc::now()));
        assert!(!s.input.complete);
        s.input.strategy.insert(
            12,
            StrategyEvidence {
                strategy: "s3".into(),
                side: "buy".into(),
                detail: json!({}),
                recorded_at: Utc::now(),
                original_volume: 100,
                entry_key: "old".into(),
            },
        );
        assert!(s.install(0, book(200), Utc::now()));
        assert!(s.input.strategy.is_empty());
    }

    #[test]
    fn closure_retains_last_assessment_and_reconnect_never_reuses_quotes() {
        let account = 704;
        reports().lock().unwrap().insert(
            account,
            Published {
                generation: 1,
                latest: Value::Null,
            },
        );
        let shared = Arc::new(Mutex::new(State::new(account, 1)));
        {
            let mut s = shared.lock().unwrap();
            let mut b = book(100);
            b.ctid_trader_account_id = account;
            assert!(s.install(0, b, Utc::now()));
        }
        publish(&shared);
        {
            let mut s = shared.lock().unwrap();
            assert!(s.install(
                0,
                pb::ProtoOaReconcileRes {
                    ctid_trader_account_id: account,
                    ..Default::default()
                },
                Utc::now()
            ));
        }
        publish(&shared);
        assert!(shared
            .lock()
            .unwrap()
            .changes
            .iter()
            .any(|v| v["kind"] == "object_no_longer_observed" && v["last_assessment"]["id"] == 12));
        let old = SessionGuard { state: shared };
        reports().lock().unwrap().insert(
            account,
            Published {
                generation: 2,
                latest: json!({"account_id":account,"generation":2,"state":"new_session"}),
            },
        );
        drop(old);
        assert_eq!(
            report(&[account], true)["accounts"][0]["state"],
            "new_session"
        );
        let fresh = State::new(account, 2);
        assert!(fresh.input.quotes.is_empty());
        assert!(fresh.input.positions.is_empty());
        assert!(!fresh.input.complete);
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn bounded_history_survives_restart_without_restoring_broker_state(pool: PgPool) {
        let account = 705;
        sqlx::query("INSERT INTO setup_assessment_snapshots(account_id,observed_at,detail) VALUES($1,now(),$2)")
            .bind(account).bind(json!({"transitions":[{"kind":"previous_close","source":"historical"}],"objects":[{"id":999}]}))
            .execute(&pool).await.unwrap();
        reports().lock().unwrap().insert(
            account,
            Published {
                generation: 1,
                latest: Value::Null,
            },
        );
        let shared = Arc::new(Mutex::new(State::new(account, 1)));
        publish(&shared);
        let (trigger, stop) = crate::shutdown::channel();
        let save = persist(&pool, &shared, stop);
        tokio::pin!(save);
        let verified = async {
            loop {
                if shared.lock().unwrap().persistence_state == "saved" {
                    break;
                }
                tokio::task::yield_now().await;
            }
        };
        tokio::select! {_=&mut save=>panic!("persistence stopped"),_=tokio::time::timeout(Duration::from_secs(3),verified)=>{}}
        trigger.fire();
        save.await;
        let detail: Value =
            sqlx::query_scalar("SELECT detail FROM setup_assessment_snapshots WHERE account_id=$1")
                .bind(account)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(detail["transitions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["kind"] == "previous_close"));
        assert!(detail["objects"].as_array().unwrap().is_empty());
        assert!(!detail["evidence_complete"].as_bool().unwrap());
        assert!(shared.lock().unwrap().input.positions.is_empty());
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn external_symbol_is_assessed_during_blocked_io_without_trading(pool: PgPool) {
        use jiaoyi_core::{allowlist::Allowlist, secret::Secret};
        use jiaoyi_ctrader::testkit::FakeBroker;
        let account_id = 703;
        let mut p = position(12, 100);
        p.price = Some(100.0);
        p.stop_loss = Some(90.0);
        p.take_profit = Some(130.0);
        let (conn, log) = FakeBroker {
            account_id,
            positions: Arc::new(Mutex::new(vec![p])),
            light_symbols: vec![pb::ProtoOaLightSymbol {
                symbol_id: 5,
                symbol_name: Some("MANUAL_ONLY".into()),
                enabled: Some(true),
                base_asset_id: Some(2),
                quote_asset_id: Some(1),
                ..Default::default()
            }],
            symbols: vec![pb::ProtoOaSymbol {
                symbol_id: 5,
                min_volume: Some(100),
                step_volume: Some(100),
                max_volume: Some(1000000),
                lot_size: Some(100),
                schedule_time_zone: Some("UTC".into()),
                schedule: vec![pb::ProtoOaInterval {
                    start_second: 0,
                    end_second: 604800,
                }],
                ..Default::default()
            }],
            quote_sequence: (0..80)
                .map(|i| (i * 50, 5, (Some(10_000_000 + i), Some(10_010_000 + i))))
                .collect(),
            trader: Some(pb::ProtoOaTrader {
                ctid_trader_account_id: account_id,
                balance: 100000,
                money_digits: Some(2),
                deposit_asset_id: 1,
                ..Default::default()
            }),
            ..Default::default()
        }
        .spawn();
        conn.application_auth("id", &Secret::new("test"))
            .await
            .unwrap();
        let list = Allowlist::parse(Some("703")).unwrap();
        let session = conn
            .account_auth(&list.permit(account_id).unwrap(), &Secret::new("test"))
            .await
            .unwrap();
        // Keep every DB request pending while broker collection and pure
        // evaluation continue on their independent futures.
        let constrained = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect_with((*pool.connect_options()).clone())
            .await
            .unwrap();
        let held = constrained.acquire().await.unwrap();
        let (trigger, stop) = crate::shutdown::channel();
        let assessment = run_session(&constrained, &session, stop);
        let slow_history = tokio::time::sleep(Duration::from_secs(30));
        tokio::pin!(assessment, slow_history);
        let verify = async {
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let v = report(&[account_id], true);
                    let a = &v["accounts"][0];
                    if a["timing"]["completed"].as_u64().unwrap_or(0) >= 4
                        && a["objects"][0]["quote_at"].is_string()
                    {
                        assert_eq!(a["position_count"], 1);
                        assert_eq!(a["objects"][0]["symbol"], "MANUAL_ONLY");
                        assert!(a["objects"][0]["strategy"].is_null());
                        assert!(a["timing"]["quotes_received"].as_u64().unwrap_or(0) > 20);
                        let public = report(&[account_id], false).to_string();
                        assert!(!public.contains("MANUAL_ONLY"));
                        assert!(!public.contains("stop_loss"));
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .unwrap();
        };
        tokio::select! {_=&mut assessment=>panic!("observer stopped"),_=&mut slow_history=>panic!("slow history unexpectedly completed"),_=verify=>{}}
        drop(held);
        trigger.fire();
        assessment.await;
        let v = report(&[account_id], true);
        assert_eq!(v["accounts"][0]["state"], "disconnected");
        let log = log.lock().unwrap();
        assert!(log.orders.is_empty());
        assert!(log.amends.is_empty());
        assert!(log.closes.is_empty());
        assert!(log
            .payload_types
            .contains(&(pb::ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as u32)));
    }
}
