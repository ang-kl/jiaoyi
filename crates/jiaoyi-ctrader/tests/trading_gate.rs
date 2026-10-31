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
    margin_unavailable: bool,
}

impl Ledger for MemLedger {
    fn admission_namespace(&self) -> String {
        format!("{:p}", Arc::as_ptr(&self.intents))
    }
    async fn margin_floor_pct(&self, _account_id: i64) -> Result<f64, String> {
        if self.margin_unavailable {
            Err("owner margin floor unavailable".into())
        } else {
            Ok(200.0)
        }
    }

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
        light_symbols: [ETH, 5]
            .into_iter()
            .map(|symbol_id| pb::ProtoOaLightSymbol {
                symbol_id,
                base_asset_id: Some(2),
                quote_asset_id: Some(1),
                ..Default::default()
            })
            .collect(),
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
        price: Some(100.0),
        stop_loss: Some(90.0),
        used_margin: Some(100),
        money_digits: Some(2),
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

// Claude · № 17,020 25-Oct (ordered: owner's Strategy 3 v2 approval)
/// Owner, 25-10-2026: on a HEDGED account Strategy 1 and Strategy 3 may each
/// hold the same symbol; an external position still blocks; a second
/// position of the same strategy does not go.
#[tokio::test]
async fn hedged_lets_strategy_1_and_strategy_3_hold_one_symbol() {
    let f = fake();
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("jiaoyi-s1")));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open_s3(Some(1_000_000)),
        "s3",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    assert_eq!(orders(&log), 1);
    // A second Strategy 3 position on the symbol is refused.
    let f = fake();
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("jiaoyi-s3")));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let r = send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open_s3(None),
        "s3b",
        Duration::from_secs(2),
    )
    .await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("one per symbol")),
        "{r:?}"
    );
    assert_eq!(orders(&log), 0);
}

#[tokio::test]
async fn netted_refuses_strategy_3_beside_strategy_1() {
    let f = FakeBroker {
        trader: Some(trader(Some(NETTED))),
        ..fake()
    };
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("jiaoyi-s1")));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let r = send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open_s3(None),
        "s3",
        Duration::from_secs(2),
    )
    .await;
    assert!(
        matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("NETTED")),
        "{r:?}"
    );
    assert_eq!(orders(&log), 0);
}

#[tokio::test]
async fn the_broker_take_profit_is_sent_with_the_order_and_checked() {
    let (conn, log) = fake().spawn();
    let s = session_on(&conn).await;
    send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open_s3(Some(1_000_000)),
        "tp",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    let o = log.lock().unwrap().orders[0].clone();
    assert_eq!(o.relative_take_profit, Some(1_000_000));
    assert_eq!(o.relative_stop_loss, Some(28_457_000));
    assert_eq!(o.label.as_deref(), Some("jiaoyi-s3"));
    // Off the symbol's 2 digits (1/100000 units: a multiple of 1000), or
    // not positive: refused, nothing written.
    for bad in [Some(1_000_500), Some(0), Some(-1_000)] {
        let (conn, log) = fake().spawn();
        let s = session_on(&conn).await;
        let r = send(
            &s,
            &allow(),
            &MemLedger::default(),
            &on(),
            open_s3(bad),
            "tp-bad",
            Duration::from_secs(2),
        )
        .await;
        assert!(
            matches!(&r, Err(SendError::Refused(x)) if x.reason.contains("take-profit")),
            "{bad:?}: {r:?}"
        );
        assert_eq!(orders(&log), 0);
    }
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
        relative_take_profit: None,
        limits: LIMITS,
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-h1".into(),
    }
}

/// Strategy 3's open on ETH: a stop and a broker take-profit.
fn open_s3(relative_take_profit: Option<i64>) -> Command {
    Command::Open {
        symbol_id: ETH,
        side: Side::Buy,
        volume: 1,
        relative_stop: 28_457_000,
        relative_take_profit,
        limits: LIMITS,
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-s3".into(),
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
        relative_take_profit: None,
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
        relative_take_profit: None,
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
        relative_take_profit: None,
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
    drop(conn);
    let (fresh, fresh_log) = fake().spawn();
    let fresh_session = session_on(&fresh).await;
    let replay = send(
        &fresh_session,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "k",
        Duration::from_millis(300),
    )
    .await;
    assert!(
        matches!(replay, Err(SendError::Refused(ref refused)) if refused.reason.contains("already recorded"))
    );
    assert_eq!(
        orders(&fresh_log),
        0,
        "reconnect cannot replay an unresolved intent"
    );
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

fn margin_fake(volume: i64, raw: i64, digits: Option<u32>) -> FakeBroker {
    FakeBroker {
        margin_reply: Some(pb::ProtoOaExpectedMarginRes {
            payload_type: None,
            ctid_trader_account_id: ACCOUNT,
            money_digits: digits,
            margin: vec![pb::ProtoOaExpectedMargin {
                volume,
                buy_margin: raw,
                sell_margin: raw,
            }],
        }),
        ..fake()
    }
}
async fn margin_send(
    f: FakeBroker,
    command: Command,
    policy: Policy,
) -> (
    Result<jiaoyi_ctrader::trading::Sent, SendError>,
    MemLedger,
    Arc<Mutex<FakeLog>>,
) {
    let (conn, log) = f.spawn();
    let session = session_on(&conn).await;
    let ledger = MemLedger::default();
    let result = send(
        &session,
        &allow(),
        &ledger,
        &policy,
        command,
        "margin-test",
        Duration::from_millis(100),
    )
    .await;
    (result, ledger, log)
}

#[tokio::test]
async fn margin_buffer_refuses_capacity_only_entry_before_intent_or_frame() {
    // $1,000 equity and $600 projected margin: affordable, but only 166.7%.
    let (result, ledger, log) =
        margin_send(margin_fake(1, 60_000, Some(2)), open(1, 28_457_000), on()).await;
    assert!(
        matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains("below floor")),
        "{result:?}"
    );
    assert!(ledger.intents.lock().unwrap().is_empty());
    assert_eq!(orders(&log), 0);
}

#[tokio::test]
async fn margin_precision_is_response_specific_and_floor_equality_passes() {
    for (amount, digits) in [(50_000, 2), (5_000_000, 4)] {
        let (result, _, log) = margin_send(
            margin_fake(1, amount, Some(digits)),
            open(1, 28_457_000),
            on(),
        )
        .await;
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(orders(&log), 1);
    }
}

#[tokio::test]
async fn margin_stop_reserve_must_survive_the_account_warning() {
    // Nominal margin 250% passes; a $400 stop loss leaves exactly 150%.
    let (result, ledger, log) = margin_send(
        margin_fake(100, 40_000, Some(2)),
        open(100, 40_000_000),
        Policy {
            edge: Ok(()),
            max_risk: 500.0,
        },
    )
    .await;
    assert!(
        matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains("after stop reserves")),
        "{result:?}"
    );
    assert!(ledger.intents.lock().unwrap().is_empty());
    assert_eq!(orders(&log), 0);
}

#[tokio::test]
async fn margin_refuses_missing_precision_wrong_account_and_duplicate_estimates() {
    let absent = margin_fake(1, 50_000, None);
    let mut wrong = margin_fake(1, 50_000, Some(2));
    wrong.margin_reply.as_mut().unwrap().ctid_trader_account_id += 1;
    let mut duplicate = margin_fake(1, 50_000, Some(2));
    let row = duplicate.margin_reply.as_ref().unwrap().margin[0];
    duplicate.margin_reply.as_mut().unwrap().margin.push(row);
    for f in [absent, wrong, duplicate] {
        let (result, ledger, log) = margin_send(f, open(1, 28_457_000), on()).await;
        assert!(matches!(result, Err(SendError::Refused(_))), "{result:?}");
        assert!(ledger.intents.lock().unwrap().is_empty());
        assert_eq!(orders(&log), 0);
    }
}

#[tokio::test]
async fn margin_refuses_changed_balance_and_incompatible_gsl_account() {
    let f = fake();
    let mut changed = trader(Some(HEDGED));
    changed.balance += 1;
    f.scripted_traders
        .lock()
        .unwrap()
        .extend([trader(Some(HEDGED)), changed]);
    let mut gsl = fake();
    gsl.trader.as_mut().unwrap().is_limited_risk = Some(true);
    for (f, reason) in [(f, "state changed"), (gsl, "GSL")] {
        let (result, ledger, log) = margin_send(f, open(1, 28_457_000), on()).await;
        assert!(
            matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains(reason)),
            "{result:?}"
        );
        assert!(ledger.intents.lock().unwrap().is_empty());
        assert_eq!(orders(&log), 0);
    }
}

#[tokio::test]
async fn margin_reserves_manual_pending_entry_and_does_not_cancel_it() {
    let f = fake();
    let pending = pb::ProtoOaOrder {
        order_id: 7,
        trade_data: pb::ProtoOaTradeData {
            symbol_id: ETH,
            volume: 100,
            trade_side: 1,
            ..Default::default()
        },
        order_type: 2,
        order_status: 1,
        executed_volume: Some(0),
        limit_price: Some(3000.0),
        stop_loss: Some(2900.0),
        ..Default::default()
    };
    let book = pb::ProtoOaReconcileRes {
        payload_type: None,
        ctid_trader_account_id: ACCOUNT,
        position: vec![],
        order: vec![pending],
    };
    f.scripted_books
        .lock()
        .unwrap()
        .extend([book.clone(), book]);
    let (result, _, log) = margin_send(f, open(1, 28_457_000), on()).await;
    assert!(result.is_ok(), "{result:?}");
    assert!(log
        .lock()
        .unwrap()
        .margin_requests
        .iter()
        .any(|(id, vol)| *id == ETH && vol.contains(&101)));
    assert!(log.lock().unwrap().amends.is_empty());
    assert!(log.lock().unwrap().closes.is_empty());
}

#[tokio::test]
async fn margin_refuses_unprotected_manual_pending_entry_without_altering_it() {
    let f = fake();
    let book = pb::ProtoOaReconcileRes {
        payload_type: None,
        ctid_trader_account_id: ACCOUNT,
        position: vec![],
        order: vec![pb::ProtoOaOrder {
            order_id: 7,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: ETH,
                volume: 100,
                trade_side: 1,
                ..Default::default()
            },
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            limit_price: Some(3000.0),
            ..Default::default()
        }],
    };
    f.scripted_books.lock().unwrap().push_back(book);
    let (result, ledger, log) = margin_send(f, open(1, 28_457_000), on()).await;
    assert!(
        matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains("pending entry has no verified stop")),
        "{result:?}"
    );
    assert!(ledger.intents.lock().unwrap().is_empty());
    assert_eq!(orders(&log), 0);
    assert!(log.lock().unwrap().amends.is_empty());
    assert!(log.lock().unwrap().closes.is_empty());
}

#[tokio::test]
async fn margin_unavailable_does_not_prevent_owned_amend_or_close() {
    let f = fake();
    let p = position(9, ETH, Some("jiaoyi-h1"));
    f.positions.lock().unwrap().push(p);
    let (conn, log) = f.spawn();
    let session = session_on(&conn).await;
    let ledger = MemLedger {
        margin_unavailable: true,
        ..Default::default()
    };
    ledger.owned.lock().unwrap().push(9);
    for (key, command) in [
        (
            "amend",
            Command::AmendStop {
                position_id: 9,
                side: Side::Buy,
                current_stop: Some(90.0),
                new_stop: 95.0,
            },
        ),
        (
            "close",
            Command::Close {
                position_id: 9,
                volume: 100,
            },
        ),
    ] {
        assert!(send(
            &session,
            &allow(),
            &ledger,
            &on(),
            command,
            key,
            Duration::from_millis(100)
        )
        .await
        .is_ok());
    }
    assert!(log.lock().unwrap().margin_requests.is_empty());
    assert_eq!(log.lock().unwrap().amends.len(), 1);
    assert_eq!(log.lock().unwrap().closes.len(), 1);
}

#[tokio::test]
async fn margin_accounts_for_pending_future_position_and_linked_protection_without_adoption() {
    for net in [0, 1] {
        let f = fake();
        let entry = pb::ProtoOaOrder {
            order_id: 7,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: ETH,
                volume: 100,
                trade_side: 1,
                ..Default::default()
            },
            order_type: 2,
            order_status: 1,
            executed_volume: Some(0),
            position_id: Some(77),
            limit_price: Some(3000.0),
            stop_loss: Some(2900.0),
            ..Default::default()
        };
        let protection = pb::ProtoOaOrder {
            order_id: 8,
            trade_data: pb::ProtoOaTradeData {
                symbol_id: ETH,
                volume: 100,
                trade_side: 2,
                ..Default::default()
            },
            order_type: 4,
            order_status: 1,
            closing_order: Some(true),
            position_id: Some(77),
            stop_loss: Some(2900.0),
            take_profit: Some(3200.0),
            ..Default::default()
        };
        let book = pb::ProtoOaReconcileRes {
            payload_type: None,
            ctid_trader_account_id: ACCOUNT,
            position: vec![],
            order: vec![entry, protection],
        };
        f.scripted_books
            .lock()
            .unwrap()
            .extend([book.clone(), book]);
        f.scripted_pnl
            .lock()
            .unwrap()
            .push_back(pb::ProtoOaGetPositionUnrealizedPnLRes {
                payload_type: None,
                ctid_trader_account_id: ACCOUNT,
                money_digits: 2,
                position_unrealized_pn_l: vec![pb::ProtoOaPositionUnrealizedPnL {
                    position_id: 77,
                    gross_unrealized_pn_l: net,
                    net_unrealized_pn_l: net,
                }],
            });
        let (result, ledger, log) = margin_send(f, open(1, 28_457_000), on()).await;
        if net == 0 {
            assert!(result.is_ok(), "{result:?}");
            assert_eq!(orders(&log), 1);
            assert!(log
                .lock()
                .unwrap()
                .margin_requests
                .iter()
                .any(|(_, v)| v.contains(&101)));
        } else {
            assert!(
                matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains("nonzero P&L")),
                "{result:?}"
            );
            assert_eq!(orders(&log), 0);
            assert!(ledger.intents.lock().unwrap().is_empty());
        }
        assert!(!ledger.owned.lock().unwrap().contains(&77));
        assert!(log.lock().unwrap().amends.is_empty());
        assert!(log.lock().unwrap().closes.is_empty());
    }
}

#[tokio::test]
async fn margin_currency_identity_cannot_understate_the_gates_stop_risk() {
    let mut command = open(1, 28_457_000);
    if let Command::Open {
        quote_to_deposit, ..
    } = &mut command
    {
        *quote_to_deposit = Some(0.1);
    }
    let (result, ledger, log) = margin_send(fake(), command, on()).await;
    assert!(
        matches!(result,Err(SendError::Refused(ref r)) if r.reason.contains("currency identity")),
        "{result:?}"
    );
    assert!(ledger.intents.lock().unwrap().is_empty());
    assert_eq!(orders(&log), 0);
}

// The caller can retain an old side while the broker holds the same ID in
// another direction. These are synthetic states, not a claim that a particular
// real cTrader position was reversed. Historical ownership alone is insufficient.
#[tokio::test]
async fn an_amend_rejects_a_direction_that_differs_from_fresh_broker_state() {
    for (broker_side, stale_side, current_stop, new_stop) in [
        (1, Side::Sell, 90.0, 85.0),  // Stale sell would widen a buy stop.
        (2, Side::Buy, 110.0, 115.0), // Stale buy would widen a sell stop.
    ] {
        let f = fake();
        let mut p = position(77, ETH, Some("jiaoyi-s1"));
        p.trade_data.trade_side = broker_side;
        p.stop_loss = Some(current_stop);
        f.positions.lock().unwrap().push(p);
        let (conn, log) = f.spawn();
        let s = session_on(&conn).await;
        let ledger = MemLedger::default();
        ledger.owned.lock().unwrap().push(77);
        let result = send(
            &s,
            &allow(),
            &ledger,
            &on(),
            Command::AmendStop {
                position_id: 77,
                side: stale_side,
                current_stop: Some(current_stop),
                new_stop,
            },
            "wrong-direction",
            Duration::from_secs(2),
        )
        .await;
        assert!(
            matches!(&result, Err(SendError::Refused(r)) if r.reason.contains("broker direction")),
            "{result:?}"
        );
        assert!(log.lock().unwrap().amends.is_empty(), "no amendment frame");
        assert!(ledger.intents.lock().unwrap().is_empty(), "no send intent");
    }
}

#[tokio::test]
async fn an_amend_rejects_an_unrecognised_broker_direction() {
    let f = fake();
    let mut p = position(77, ETH, Some("jiaoyi-s1"));
    p.trade_data.trade_side = 99;
    f.positions.lock().unwrap().push(p);
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    ledger.owned.lock().unwrap().push(77);
    let result = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        Command::AmendStop {
            position_id: 77,
            side: Side::Buy,
            current_stop: Some(90.0),
            new_stop: 95.0,
        },
        "unknown-direction",
        Duration::from_secs(2),
    )
    .await;
    assert!(
        matches!(&result, Err(SendError::Refused(r)) if r.reason.contains("broker direction")),
        "{result:?}"
    );
    assert!(log.lock().unwrap().amends.is_empty());
    assert!(ledger.intents.lock().unwrap().is_empty());
}

#[tokio::test]
async fn matching_directions_keep_tightening_and_preserve_take_profit() {
    for (broker_side, side, current_stop, new_stop, take_profit) in [
        (1, Side::Buy, 90.0, 95.0, 120.0),
        (2, Side::Sell, 110.0, 105.0, 80.0),
    ] {
        let f = fake();
        let mut p = position(77, ETH, Some("jiaoyi-s1"));
        p.trade_data.trade_side = broker_side;
        p.stop_loss = Some(current_stop);
        p.take_profit = Some(take_profit);
        f.positions.lock().unwrap().push(p);
        let (conn, log) = f.clone().spawn();
        let s = session_on(&conn).await;
        let ledger = MemLedger::default();
        ledger.owned.lock().unwrap().push(77);
        let sent = send(
            &s,
            &allow(),
            &ledger,
            &on(),
            Command::AmendStop {
                position_id: 77,
                side,
                current_stop: Some(current_stop),
                new_stop,
            },
            "matching-direction",
            Duration::from_secs(2),
        )
        .await
        .unwrap();
        assert!(matches!(sent.outcome, Outcome::Amended { .. }));
        let amends = &log.lock().unwrap().amends;
        assert_eq!(amends.len(), 1);
        assert_eq!(amends[0].stop_loss, Some(new_stop));
        assert_eq!(amends[0].take_profit, Some(take_profit));
        let positions = f.positions.lock().unwrap();
        assert_eq!(positions[0].stop_loss, Some(new_stop));
        assert_eq!(positions[0].take_profit, Some(take_profit));
        let intents = ledger.intents.lock().unwrap();
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].take_profit, Some(take_profit));
    }
}
