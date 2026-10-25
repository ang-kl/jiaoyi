//! The order gate against the scripted broker (owner, 25-10-2026): every
//! refusal writes nothing to the socket; the intent is recorded before the
//! frame; an entry key is never sent twice.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jiaoyi_core::allowlist::Allowlist;
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages as pb;
use jiaoyi_ctrader::testkit::{FakeBroker, FakeLog};
use jiaoyi_ctrader::trading::{
    send, Command, EventRecord, Intent, IntentWrite, Ledger, Outcome, Policy, SendError, Side,
    VolumeLimits, HEDGED, NETTED,
};
use jiaoyi_ctrader::Connection;

const ACCOUNT: i64 = 49236051;
const ETH: i64 = 22;

/// An in-memory ledger. `fail` refuses every intent.
#[derive(Default, Clone)]
struct MemLedger {
    intents: Arc<Mutex<Vec<Intent>>>,
    events: Arc<Mutex<Vec<EventRecord>>>,
    owned: Arc<Mutex<Vec<i64>>>,
    fail: bool,
}

impl Ledger for MemLedger {
    async fn record_intent(&self, i: &Intent) -> Result<IntentWrite, String> {
        if self.fail {
            return Err("database down".into());
        }
        let mut v = self.intents.lock().unwrap();
        if v.iter().any(|x| x.entry_key == i.entry_key) {
            return Ok(IntentWrite::AlreadyRecorded);
        }
        v.push(i.clone());
        Ok(IntentWrite::Recorded)
    }
    async fn record_event(&self, e: &EventRecord) -> Result<(), String> {
        if let Some(p) = e.position_id {
            if e.kind == "ORDER_FILLED" {
                self.owned.lock().unwrap().push(p);
            }
        }
        self.events.lock().unwrap().push(e.clone());
        Ok(())
    }
    async fn owns_position(&self, _account_id: i64, position_id: i64) -> Result<bool, String> {
        Ok(self.owned.lock().unwrap().contains(&position_id))
    }
}

fn trader(account_type: Option<i32>) -> pb::ProtoOaTrader {
    pb::ProtoOaTrader {
        ctid_trader_account_id: ACCOUNT,
        balance: 100_000,
        deposit_asset_id: 1,
        money_digits: Some(2),
        account_type,
        ..Default::default()
    }
}

fn fake() -> FakeBroker {
    FakeBroker {
        account_id: ACCOUNT,
        fill_quotes: HashMap::from([(ETH, (3000.0, 3001.0))]),
        trader: Some(trader(Some(HEDGED))),
        ..Default::default()
    }
}

fn position(id: i64, symbol_id: i64, label: Option<&str>) -> pb::ProtoOaPosition {
    pb::ProtoOaPosition {
        position_id: id,
        trade_data: pb::ProtoOaTradeData {
            symbol_id,
            volume: 100,
            trade_side: 1,
            label: label.map(str::to_string),
            ..Default::default()
        },
        position_status: 1,
        swap: 0,
        ..Default::default()
    }
}

/// Refused on a broker in this state: no order frame is written.
async fn refused_on(f: FakeBroker, expect: &str) {
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let r = send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open(1, 28_457_000),
        "k",
        Duration::from_secs(2),
    )
    .await;
    match r {
        Err(SendError::Refused(r)) => assert!(r.reason.contains(expect), "{}", r.reason),
        other => panic!("expected a refusal containing {expect:?}, got {other:?}"),
    }
    assert_eq!(orders(&log), 0);
}

#[tokio::test]
async fn netted_refuses_an_entry_on_a_symbol_with_any_position() {
    let f = FakeBroker {
        trader: Some(trader(Some(NETTED))),
        ..fake()
    };
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("bot-trade")));
    refused_on(f, "NETTED").await;
}

#[tokio::test]
async fn netted_with_no_position_on_the_symbol_sends() {
    let f = FakeBroker {
        trader: Some(trader(Some(NETTED))),
        ..fake()
    };
    // A position on another symbol does not net against this one.
    f.positions.lock().unwrap().push(position(9, 5, None));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open(1, 28_457_000),
        "k",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    assert_eq!(orders(&log), 1);
}

#[tokio::test]
async fn hedged_refuses_an_entry_beside_an_external_position() {
    let f = fake();
    f.positions.lock().unwrap().push(position(9, ETH, None));
    refused_on(f, "external").await;
}

#[tokio::test]
async fn hedged_refuses_a_second_position_of_ours_on_a_symbol() {
    let f = fake();
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("jiaoyi-h1")));
    refused_on(f, "one per symbol").await;
}

#[tokio::test]
async fn an_unknown_account_type_refuses_entries() {
    let f = FakeBroker {
        trader: Some(trader(None)),
        ..fake()
    };
    refused_on(f, "no account type").await;
    let f = FakeBroker {
        trader: Some(trader(Some(2))),
        ..fake()
    };
    refused_on(f, "not HEDGED or NETTED").await;
}

async fn session_on(conn: &Connection) -> jiaoyi_ctrader::AccountSession<'_> {
    conn.application_auth("id", &Secret::new("secret"))
        .await
        .unwrap();
    let list = Allowlist::parse(Some("49236051")).unwrap();
    conn.account_auth(&list.permit(ACCOUNT).unwrap(), &Secret::new("tok"))
        .await
        .unwrap()
}

fn allow() -> Allowlist {
    Allowlist::parse(Some("49236051")).unwrap()
}

fn on() -> Policy {
    Policy {
        edge: Ok(()),
        max_risk: 10.0,
    }
}

const LIMITS: VolumeLimits = VolumeLimits {
    min: 1,
    step: 1,
    max: Some(100_000),
    digits: 2,
};

/// ETHUSD-like: 1 cent of volume = 0.01 ETH; a 284.57 stop on 1 cent risks 2.8457.
fn open(volume: i64, relative_stop: i64) -> Command {
    Command::Open {
        symbol_id: ETH,
        side: Side::Buy,
        volume,
        relative_stop,
        limits: LIMITS,
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-h1".into(),
    }
}

fn orders(log: &Arc<Mutex<FakeLog>>) -> usize {
    log.lock().unwrap().orders.len()
}

async fn refused(
    allowlist: &Allowlist,
    ledger: &MemLedger,
    policy: &Policy,
    cmd: Command,
    expect: &str,
) {
    let (conn, log) = fake().spawn();
    let s = session_on(&conn).await;
    let r = send(
        &s,
        allowlist,
        ledger,
        policy,
        cmd,
        "k1",
        Duration::from_secs(2),
    )
    .await;
    match r {
        Err(SendError::Refused(r)) => assert!(r.reason.contains(expect), "{}", r.reason),
        other => panic!("expected a refusal containing {expect:?}, got {other:?}"),
    }
    assert_eq!(orders(&log), 0, "a refusal writes no order frame");
    assert!(log.lock().unwrap().amends.is_empty());
    assert!(log.lock().unwrap().closes.is_empty());
}

#[tokio::test]
async fn an_account_off_the_allowlist_is_refused() {
    let other = Allowlist::parse(Some("43002148")).unwrap();
    refused(
        &other,
        &MemLedger::default(),
        &on(),
        open(1, 28_457_000),
        "allowlist",
    )
    .await;
}

#[tokio::test]
async fn an_unmet_edge_gate_sends_nothing() {
    let p = Policy {
        edge: Err("no held-out PASS under 9105c5eb".into()),
        ..on()
    };
    refused(
        &allow(),
        &MemLedger::default(),
        &p,
        open(1, 28_457_000),
        "edge gate not met",
    )
    .await;
}

#[tokio::test]
async fn an_order_without_a_stop_is_refused() {
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        open(1, 0),
        "no stop",
    )
    .await;
}

#[tokio::test]
async fn a_stop_off_the_symbols_digits_is_refused() {
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        open(1, 28_457_001),
        "digits",
    )
    .await;
}

#[tokio::test]
async fn an_order_above_the_risk_allowance_is_refused() {
    // 4 cents × 284.57 = 11.38 > 10.
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        open(4, 28_457_000),
        "above the allowed",
    )
    .await;
}

#[tokio::test]
async fn volume_below_the_minimum_is_refused_never_rounded_up() {
    let cmd = Command::Open {
        symbol_id: ETH,
        side: Side::Buy,
        volume: 1,
        relative_stop: 28_457_000,
        limits: VolumeLimits { min: 2, ..LIMITS },
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-h1".into(),
    };
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        cmd,
        "below the broker minimum",
    )
    .await;
}

#[tokio::test]
async fn volume_off_the_step_is_refused() {
    let cmd = Command::Open {
        symbol_id: ETH,
        side: Side::Buy,
        volume: 3,
        relative_stop: 28_457_000,
        limits: VolumeLimits { step: 2, ..LIMITS },
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-h1".into(),
    };
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        cmd,
        "multiple of the step",
    )
    .await;
}

#[tokio::test]
async fn an_unknown_conversion_rate_is_refused() {
    let cmd = Command::Open {
        symbol_id: ETH,
        side: Side::Buy,
        volume: 1,
        relative_stop: 28_457_000,
        limits: LIMITS,
        quote_to_deposit: None,
        label: "jiaoyi-h1".into(),
    };
    refused(&allow(), &MemLedger::default(), &on(), cmd, "quote→deposit").await;
}

#[tokio::test]
async fn a_failing_ledger_means_no_frame() {
    let ledger = MemLedger {
        fail: true,
        ..Default::default()
    };
    refused(
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "could not be recorded",
    )
    .await;
}

#[tokio::test]
async fn an_open_is_recorded_before_it_is_sent_and_its_fill_is_kept() {
    let (conn, log) = fake().spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    let sent = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "h1:49236051:22:2026-10-25:buy",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    match &sent.outcome {
        Outcome::Filled {
            position_id,
            price,
            volume,
        } => {
            assert_eq!(*position_id, Some(1001));
            assert_eq!(*price, Some(3001.0));
            assert_eq!(*volume, Some(1));
        }
        o => panic!("{o:?}"),
    }
    let intents = ledger.intents.lock().unwrap().clone();
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].client_msg_id, sent.client_msg_id);
    assert!((intents[0].risk.unwrap() - 2.8457).abs() < 1e-9);
    let kinds: Vec<String> = ledger
        .events
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.kind.clone())
        .collect();
    assert_eq!(kinds, vec!["ORDER_ACCEPTED", "ORDER_FILLED"]);
    let o = log.lock().unwrap().orders[0].clone();
    assert_eq!(o.relative_stop_loss, Some(28_457_000));
    assert_eq!(o.take_profit, None);
    assert_eq!(o.relative_take_profit, None);
    assert_eq!(o.stop_loss, None, "a MARKET order takes the relative stop");
    assert_eq!(o.label.as_deref(), Some("jiaoyi-h1"));
    assert_eq!(o.order_type, 1, "MARKET");
}

#[tokio::test]
async fn an_entry_key_is_never_sent_twice() {
    let f = fake();
    let positions = f.positions.clone();
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    let wait = Duration::from_secs(2);
    send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "same",
        wait,
    )
    .await
    .unwrap();
    // The position is gone (stopped out): only the key stops a resend now.
    positions.lock().unwrap().clear();
    let again = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "same",
        wait,
    )
    .await;
    assert!(
        matches!(&again, Err(SendError::Refused(r)) if r.reason.contains("already recorded")),
        "{again:?}"
    );
    assert_eq!(orders(&log), 1);
}

#[tokio::test]
async fn a_rejected_order_is_reported_as_rejected() {
    let f = FakeBroker {
        reject_orders: Some("MARKET_CLOSED".into()),
        ..fake()
    };
    let (conn, _log) = f.spawn();
    let s = session_on(&conn).await;
    let sent = send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open(1, 28_457_000),
        "k",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    assert_eq!(
        sent.outcome,
        Outcome::Rejected {
            code: Some("MARKET_CLOSED".into()),
            description: None
        }
    );
}

#[tokio::test]
async fn no_answer_is_unconfirmed_not_resent() {
    let f = FakeBroker {
        silent_orders: true,
        ..fake()
    };
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    let sent = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "k",
        Duration::from_millis(300),
    )
    .await
    .unwrap();
    assert_eq!(sent.outcome, Outcome::Unconfirmed);
    assert_eq!(orders(&log), 1);
    assert_eq!(ledger.intents.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn an_amend_moves_an_owned_stop_only_in_the_trades_favour() {
    let (conn, log) = fake().spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    let wait = Duration::from_secs(2);
    send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "open",
        wait,
    )
    .await
    .unwrap();
    // The fill was 3001.00 with a 284.57 stop: 2716.43.
    let worse = Command::AmendStop {
        position_id: 1001,
        side: Side::Buy,
        current_stop: Some(2716.43),
        new_stop: 2700.0,
    };
    let r = send(&s, &allow(), &ledger, &on(), worse, "a1", wait).await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("favour")),
        "{r:?}"
    );
    assert!(log.lock().unwrap().amends.is_empty());
    // The trader's own stop is checked too, before any read: a stop below
    // the one it holds is refused even where the broker's is lower.
    let behind = Command::AmendStop {
        position_id: 1001,
        side: Side::Buy,
        current_stop: Some(2850.0),
        new_stop: 2800.0,
    };
    let r = send(&s, &allow(), &ledger, &on(), behind, "a0", wait).await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("favour from 2850")),
        "{r:?}"
    );
    assert!(log.lock().unwrap().amends.is_empty());
    let better = Command::AmendStop {
        position_id: 1001,
        side: Side::Buy,
        current_stop: Some(2716.43),
        new_stop: 2800.0,
    };
    let sent = send(&s, &allow(), &ledger, &on(), better, "a2", wait)
        .await
        .unwrap();
    assert!(matches!(sent.outcome, Outcome::Amended { .. }), "{sent:?}");
    let a = log.lock().unwrap().amends[0];
    assert_eq!(a.stop_loss, Some(2800.0));
    assert_eq!(a.take_profit, None);
}

#[tokio::test]
async fn a_position_this_program_did_not_open_is_never_amended_or_closed() {
    let wait = Duration::from_secs(2);
    let amend = Command::AmendStop {
        position_id: 77,
        side: Side::Sell,
        current_stop: Some(10.0),
        new_stop: 9.0,
    };
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        amend,
        "not opened by this program",
    )
    .await;
    let close = Command::Close {
        position_id: 77,
        volume: 1,
    };
    refused(
        &allow(),
        &MemLedger::default(),
        &on(),
        close,
        "not opened by this program",
    )
    .await;
    let _ = wait;
}

#[tokio::test]
async fn an_amend_carries_the_take_profit_and_passes_the_brokers_own_stop() {
    // cTrader's amend REPLACES the protection: a stop-only amend would
    // delete a take-profit (Strategy 3 sets one). The fake replaces too.
    let f = fake();
    let (conn, log) = f.clone().spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    let wait = Duration::from_secs(2);
    send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "open",
        wait,
    )
    .await
    .unwrap();
    f.positions.lock().unwrap()[0].take_profit = Some(3500.0);
    let amend = |current: f64, new: f64| Command::AmendStop {
        position_id: 1001,
        side: Side::Buy,
        current_stop: Some(current),
        new_stop: new,
    };
    let sent = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        amend(2716.43, 2800.0),
        "a1",
        wait,
    )
    .await
    .unwrap();
    assert!(matches!(sent.outcome, Outcome::Amended { .. }), "{sent:?}");
    let a = log.lock().unwrap().amends[0];
    assert_eq!(a.stop_loss, Some(2800.0));
    assert_eq!(a.take_profit, Some(3500.0), "the take-profit is carried");
    assert_eq!(f.positions.lock().unwrap()[0].take_profit, Some(3500.0));
    let i = ledger.intents.lock().unwrap().last().cloned().unwrap();
    assert_eq!(i.take_profit, Some(3500.0), "and recorded on the intent");
    // The broker's stop moved past the trader's view: an amend that would
    // loosen it is refused, from the fresh read.
    f.positions.lock().unwrap()[0].stop_loss = Some(2900.0);
    let r = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        amend(2800.0, 2850.0),
        "a2",
        wait,
    )
    .await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("the broker's 2900")),
        "{r:?}"
    );
    // A position no longer open at the broker is not amended.
    f.positions.lock().unwrap().clear();
    let r = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        amend(2900.0, 2950.0),
        "a3",
        wait,
    )
    .await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("not open at the broker")),
        "{r:?}"
    );
    assert_eq!(log.lock().unwrap().amends.len(), 1);
}
