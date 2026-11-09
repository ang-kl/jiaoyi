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
    send, Command, EventRecord, Intent, IntentWrite, Ledger, ManagementAuthority, Outcome, Policy,
    SendError, Side, VolumeLimits, HEDGED, NETTED,
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
    authorities: Arc<Mutex<HashMap<i64, ManagementAuthority>>>,
    fail: bool,
    fail_fill_receipt: bool,
    margin_unavailable: bool,
    intent_delay: Duration,
    fail_unsent_receipt: bool,
    fail_guard_receipt: bool,
}

impl MemLedger {
    fn prove(&self, position_id: i64, side: Side, volume: i64) {
        self.owned.lock().unwrap().push(position_id);
        self.authorities.lock().unwrap().insert(
            position_id,
            ManagementAuthority {
                entry_key: "fixture-owned-opening".into(),
                symbol_id: ETH,
                side,
                remaining_volume: volume,
            },
        );
    }
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
        {
            let mut v = self.intents.lock().unwrap();
            if v.iter().any(|x| x.entry_key == i.entry_key) {
                return Ok(IntentWrite::AlreadyRecorded);
            }
            v.push(i.clone());
        }
        tokio::time::sleep(self.intent_delay).await;
        Ok(IntentWrite::Recorded)
    }
    async fn record_event(&self, e: &EventRecord) -> Result<(), String> {
        if self.fail_guard_receipt && e.kind == "ENTRY_VALIDATED_BEFORE_SEND" {
            return Err("injected validation receipt failure after write".into());
        }
        if self.fail_unsent_receipt && e.kind == "REFUSED_BEFORE_SEND" {
            return Err("injected unsent receipt failure".into());
        }
        if self.fail_fill_receipt && e.kind == "ORDER_FILLED" {
            return Err("injected durable fill write failure".into());
        }
        if let Some(p) = e.position_id {
            if e.kind == "ORDER_FILLED" {
                self.owned.lock().unwrap().push(p);
                if let Some(i) = self.intents.lock().unwrap().iter().find(|i| {
                    Some(i.entry_key.as_str()) == e.entry_key.as_deref() && i.kind == "open"
                }) {
                    self.authorities.lock().unwrap().insert(
                        p,
                        ManagementAuthority {
                            entry_key: i.entry_key.clone(),
                            symbol_id: i.symbol_id.unwrap(),
                            side: i.side.unwrap(),
                            remaining_volume: e.volume.unwrap(),
                        },
                    );
                }
            }
        }
        self.events.lock().unwrap().push(e.clone());
        Ok(())
    }
    async fn owns_position(&self, _account_id: i64, position_id: i64) -> Result<bool, String> {
        Ok(self.owned.lock().unwrap().contains(&position_id))
    }
    async fn management_authority(
        &self,
        _account_id: i64,
        position_id: i64,
    ) -> Result<ManagementAuthority, String> {
        self.authorities
            .lock()
            .unwrap()
            .get(&position_id)
            .cloned()
            .ok_or("authority incomplete: fixture lacks qualified opening".into())
    }
    async fn record_authority_absence(
        &self,
        _account_id: i64,
        position_id: i64,
    ) -> Result<(), String> {
        self.authorities.lock().unwrap().remove(&position_id);
        Ok(())
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

/// №18,403 supersedes S1/S3 same-symbol coholding: every non-S3 label
/// now reserves that symbol for the external owner on this account.
#[tokio::test]
async fn s3_yields_to_every_non_s3_label_including_strategy1() {
    let f = fake();
    f.positions
        .lock()
        .unwrap()
        .push(position(9, ETH, Some("jiaoyi-s1")));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let result = send(
        &s,
        &allow(),
        &MemLedger::default(),
        &on(),
        open_s3(Some(1_000_000)),
        "s3",
        Duration::from_secs(2),
    )
    .await;
    assert!(matches!(result, Err(SendError::Refused(_))));
    assert_eq!(orders(&log), 0);
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
    ledger.prove(9, Side::Buy, 100);
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
        ledger.prove(77, stale_side, 100);
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
            matches!(&result, Err(SendError::Refused(r)) if r.reason.contains("direction")),
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
    ledger.prove(77, Side::Buy, 100);
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
        matches!(&result, Err(SendError::Refused(r)) if r.reason.contains("direction")),
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
        ledger.prove(77, side, 100);
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

#[tokio::test]
async fn a_historical_owner_without_current_authority_cannot_manage() {
    let f = fake();
    f.positions
        .lock()
        .unwrap()
        .push(position(77, ETH, Some("jiaoyi-s3")));
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    ledger.owned.lock().unwrap().push(77);
    for (key, command) in [
        (
            "historical-amend",
            Command::AmendStop {
                position_id: 77,
                side: Side::Buy,
                current_stop: Some(90.0),
                new_stop: 95.0,
            },
        ),
        (
            "historical-close",
            Command::Close {
                position_id: 77,
                volume: 100,
            },
        ),
    ] {
        let result = send(
            &s,
            &allow(),
            &ledger,
            &on(),
            command,
            key,
            Duration::from_secs(1),
        )
        .await;
        assert!(matches!(result, Err(SendError::Refused(_))));
    }
    let log = log.lock().unwrap();
    assert!(log.amends.is_empty() && log.closes.is_empty());
    assert!(ledger.intents.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_close_refuses_both_reversals_and_even_an_owned_fraction_of_enlarged_exposure() {
    for (original, current_side, current_volume, requested) in [
        (Side::Buy, 2, 13, 13),
        (Side::Sell, 1, 13, 13),
        (Side::Buy, 1, 26, 13),
        (Side::Buy, 1, 13, 14),
    ] {
        let f = fake();
        let mut p = position(77, ETH, Some("jiaoyi-s3"));
        p.trade_data.trade_side = current_side;
        p.trade_data.volume = current_volume;
        f.positions.lock().unwrap().push(p);
        let (conn, log) = f.spawn();
        let s = session_on(&conn).await;
        let ledger = MemLedger::default();
        ledger.prove(77, original, 13);
        let result = send(
            &s,
            &allow(),
            &ledger,
            &on(),
            Command::Close {
                position_id: 77,
                volume: requested,
            },
            "qualified-close",
            Duration::from_secs(1),
        )
        .await;
        assert!(matches!(result, Err(SendError::Refused(_))), "{result:?}");
        assert!(log.lock().unwrap().closes.is_empty());
        assert!(ledger.intents.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn a_proven_reduced_residual_keeps_its_close() {
    let f = fake();
    let mut p = position(77, ETH, Some("jiaoyi-s3"));
    p.trade_data.volume = 5;
    f.positions.lock().unwrap().push(p);
    let (conn, log) = f.spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    ledger.prove(77, Side::Buy, 5);
    let result = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        Command::Close {
            position_id: 77,
            volume: 5,
        },
        "reduced-close",
        Duration::from_secs(1),
    )
    .await
    .unwrap();
    assert!(matches!(result.outcome, Outcome::Filled { .. }));
    assert_eq!(log.lock().unwrap().closes.len(), 1);
}

#[tokio::test]
async fn a_post_send_fill_persistence_failure_is_unresolved_and_never_resent() {
    let f = fake();
    let (conn, log) = f.clone().spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger {
        fail_fill_receipt: true,
        ..Default::default()
    };
    let result = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "fill-write-failure",
        Duration::from_secs(1),
    )
    .await;
    assert!(matches!(result, Err(SendError::Evidence(_))), "{result:?}");
    assert_eq!(
        log.lock().unwrap().orders.len(),
        1,
        "frame was sent: not a refusal"
    );
    assert_eq!(ledger.intents.lock().unwrap().len(), 1);
    assert!(
        f.positions.lock().unwrap()[0].stop_loss.is_some(),
        "broker protection retained"
    );
    // Even after the broker later reports no position, the original entry key
    // remains consumed; this refusal is not merely symbol-capacity admission.
    f.positions.lock().unwrap().clear();
    let retry = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "fill-write-failure",
        Duration::from_secs(1),
    )
    .await;
    assert!(matches!(retry, Err(SendError::Refused(r)) if r.reason.contains("already recorded")));
    assert_eq!(log.lock().unwrap().orders.len(), 1);
}

#[tokio::test]
async fn shared_gate_absence_revokes_authority_before_same_shape_reappearance() {
    let f = fake();
    let (conn, log) = f.clone().spawn();
    let s = session_on(&conn).await;
    let ledger = MemLedger::default();
    ledger.prove(77, Side::Buy, 100);
    let close = Command::Close {
        position_id: 77,
        volume: 100,
    };
    let result = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        close.clone(),
        "observed-absent",
        Duration::from_secs(1),
    )
    .await;
    assert!(matches!(result, Err(SendError::Refused(_))));
    f.positions
        .lock()
        .unwrap()
        .push(position(77, ETH, Some("jiaoyi-s3")));
    let result = send(
        &s,
        &allow(),
        &ledger,
        &on(),
        close,
        "reappeared",
        Duration::from_secs(1),
    )
    .await;
    assert!(matches!(result, Err(SendError::Refused(_))));
    assert!(log.lock().unwrap().closes.is_empty());
    assert_eq!(f.positions.lock().unwrap()[0].stop_loss, Some(90.0));
}

#[tokio::test]
async fn entry_guard_rejects_after_slow_intent_write_and_persists_only_proven_unsent() {
    use jiaoyi_ctrader::entry_guard::{EntryGuard, Signal};
    for fail_unsent_receipt in [false, true] {
        let f = FakeBroker {
            quotes: HashMap::from([(ETH, vec![(Some(300_000_000), Some(300_100_000))])]),
            ..fake()
        };
        let (conn, log) = f.spawn();
        let session = session_on(&conn).await;
        let mut quotes = conn.spot_events();
        session.subscribe_spots(&[ETH]).await.unwrap();
        quotes.recv().await.unwrap();
        let ledger = MemLedger {
            intent_delay: Duration::from_millis(80),
            fail_unsent_receipt,
            ..Default::default()
        };
        let guard = EntryGuard {
            connection_generation: conn.entry_generation(),
            account_id: ACCOUNT,
            symbol_id: ETH,
            side: Side::Buy,
            signal: Signal::Trend { level: 2990.0 },
            cost: None,
            expires: tokio::time::Instant::now() + Duration::from_millis(40),
            account_revision: conn.entry_revision(ACCOUNT),
        };
        let result = jiaoyi_ctrader::trading::send_entry(
            &session,
            &allow(),
            &ledger,
            &on(),
            open(1, 28_457_000),
            "expired-persisted",
            Duration::from_secs(1),
            &guard,
        )
        .await;
        assert_eq!(orders(&log), 0);
        assert_eq!(ledger.intents.lock().unwrap().len(), 1);
        if fail_unsent_receipt {
            assert!(matches!(result, Err(SendError::Evidence(_))));
            assert!(
                ledger.events.lock().unwrap().is_empty(),
                "failed terminal receipt leaves intent unresolved"
            );
        } else {
            assert!(
                matches!(result, Err(SendError::Refused(e)) if e.reason.contains("ENTRY_EXPIRED"))
            );
            assert_eq!(ledger.events.lock().unwrap()[0].kind, "REFUSED_BEFORE_SEND");
            let receipt: serde_json::Value =
                serde_json::from_slice(&ledger.events.lock().unwrap()[0].raw).unwrap();
            assert_eq!(receipt["evidence"]["bid"]["price"], 3000.0);
            assert_eq!(receipt["evidence"]["ask"]["price"], 3001.0);
            assert_eq!(receipt["evidence"]["deadline_elapsed"], true);
            assert_eq!(receipt["evidence"]["stage"], "after_writer_lock");
            assert!(
                receipt["evidence"]["bid"]["receipt_age_ms"]
                    .as_i64()
                    .unwrap()
                    >= 40
            );
            let repeated = jiaoyi_ctrader::trading::send_entry(
                &session,
                &allow(),
                &ledger,
                &on(),
                open(1, 28_457_000),
                "expired-persisted",
                Duration::from_secs(1),
                &guard,
            )
            .await;
            assert!(
                matches!(repeated, Err(SendError::Refused(e)) if e.reason.contains("already recorded"))
            );
            assert_eq!(orders(&log), 0);
            let control = MemLedger::default();
            let valid = EntryGuard {
                expires: tokio::time::Instant::now() + Duration::from_secs(1),
                ..guard
            };
            let normal_started = std::time::Instant::now();
            let sent = jiaoyi_ctrader::trading::send_entry(
                &session,
                &allow(),
                &control,
                &on(),
                open(1, 28_457_000),
                "next-permitted",
                Duration::from_secs(1),
                &valid,
            )
            .await
            .unwrap();
            assert!(matches!(sent.outcome, Outcome::Filled { .. }));
            assert!(
                sent.events
                    .iter()
                    .all(|e| e.kind != "ENTRY_VALIDATED_BEFORE_SEND"),
                "returned event list remains broker executions only"
            );
            let recorded = control.events.lock().unwrap();
            let validated: Vec<_> = recorded
                .iter()
                .filter(|e| e.kind == "ENTRY_VALIDATED_BEFORE_SEND")
                .collect();
            assert_eq!(validated.len(), 1);
            let snapshot: serde_json::Value = serde_json::from_slice(&validated[0].raw).unwrap();
            assert_eq!(snapshot["result"], "passed");
            assert_eq!(snapshot["deadline_elapsed"], false);
            assert_eq!(snapshot["bid"]["price"], 3000.0);
            assert_eq!(snapshot["ask"]["price"], 3001.0);
            assert_eq!(snapshot["connection_generation"], conn.entry_generation());
            assert_eq!(snapshot["stage"], "after_writer_lock");
            eprintln!(
                "valid guarded fake-broker open including acknowledgement: {} ms",
                normal_started.elapsed().as_millis()
            );
            assert_eq!(orders(&log), 1);
        }
    }
}

#[tokio::test]
async fn successful_entry_guard_receipt_failure_after_write_is_uncertain_and_not_resent() {
    use jiaoyi_ctrader::entry_guard::{EntryGuard, Signal};
    let f = FakeBroker {
        quotes: HashMap::from([(ETH, vec![(Some(300_000_000), Some(300_100_000))])]),
        ..fake()
    };
    let (conn, log) = f.spawn();
    let session = session_on(&conn).await;
    let mut quotes = conn.spot_events();
    session.subscribe_spots(&[ETH]).await.unwrap();
    quotes.recv().await.unwrap();
    let ledger = MemLedger {
        fail_guard_receipt: true,
        ..Default::default()
    };
    let guard = EntryGuard {
        connection_generation: conn.entry_generation(),
        account_id: ACCOUNT,
        symbol_id: ETH,
        side: Side::Buy,
        signal: Signal::Trend { level: 2990.0 },
        cost: None,
        expires: tokio::time::Instant::now() + Duration::from_secs(1),
        account_revision: conn.entry_revision(ACCOUNT),
    };
    let result = jiaoyi_ctrader::trading::send_entry(
        &session,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "validation-receipt-failed",
        Duration::from_secs(1),
        &guard,
    )
    .await;
    assert!(matches!(result, Err(SendError::Evidence(e)) if e.contains("after write")));
    for _ in 0..20 {
        if orders(&log) == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(orders(&log), 1, "frame preceded the receipt write");
    assert_eq!(ledger.intents.lock().unwrap().len(), 1);
    assert!(
        ledger.events.lock().unwrap().is_empty(),
        "no false fill or unsent terminal event"
    );
    let again = jiaoyi_ctrader::trading::send_entry(
        &session,
        &allow(),
        &ledger,
        &on(),
        open(1, 28_457_000),
        "validation-receipt-failed",
        Duration::from_secs(1),
        &guard,
    )
    .await;
    assert!(matches!(again, Err(SendError::Refused(_))));
    assert_eq!(
        orders(&log),
        1,
        "uncertain attempt cannot emit a duplicate frame"
    );
}

fn prepared_fake() -> FakeBroker {
    FakeBroker {
        quotes: HashMap::from([(ETH, vec![(Some(300_000_000), Some(300_100_000))])]),
        symbols: vec![pb::ProtoOaSymbol {
            symbol_id: ETH,
            digits: 2,
            pip_position: 2,
            min_volume: Some(1),
            step_volume: Some(1),
            max_volume: Some(100_000),
            trading_mode: Some(0),
            enable_short_selling: Some(true),
            schedule_time_zone: Some("UTC".into()),
            schedule: vec![pb::ProtoOaInterval {
                start_second: 0,
                end_second: 604800,
            }],
            ..Default::default()
        }],
        ..fake()
    }
}
fn prepared_guard(
    conn: &Connection,
    p: &jiaoyi_ctrader::trading::PreparedEntry,
) -> jiaoyi_ctrader::entry_guard::EntryGuard {
    use jiaoyi_ctrader::entry_guard::{EntryGuard, Signal};
    EntryGuard {
        connection_generation: p.generation(),
        account_revision: p.revision(),
        account_id: ACCOUNT,
        symbol_id: ETH,
        side: Side::Buy,
        signal: Signal::PullbackBounded {
            origin: 2990.,
            level: 3010.,
            impulse_range: 40.,
            admitted_entry: 3014.5,
            price_digits: 2,
            peer: None,
            trigger_quote: conn.quote_snapshot(ACCOUNT, ETH).unwrap(),
            margin: p.receipt.fx_guard.clone(),
            calendar: p.calendar.clone(),
        },
        cost: None,
        expires: p.expires(),
    }
}

#[tokio::test]
async fn prepared_dispatch_uses_no_broker_reads_and_preserves_intent_before_send() {
    let (conn, log) = prepared_fake().spawn();
    let session = session_on(&conn).await;
    let mut quotes = conn.spot_events();
    session.subscribe_spots(&[ETH]).await.unwrap();
    quotes.recv().await.unwrap();
    let ledger = MemLedger::default();
    let command = open(1, 28_457_000);
    let policy = on();
    let prepared = jiaoyi_ctrader::trading::prepare_entry(
        &session,
        &allow(),
        &ledger,
        &policy,
        &command,
        false,
    )
    .await
    .unwrap();
    let mut altered = prepared_guard(&conn, &prepared);
    if let jiaoyi_ctrader::entry_guard::Signal::PullbackBounded {
        margin: Some(proof),
        ..
    } = &mut altered.signal
    {
        proof.candidate_max_risk = Some(policy.max_risk + 1.);
    }
    let invalid = jiaoyi_ctrader::trading::send_prepared_entry(
        &session,
        &allow(),
        &ledger,
        &policy,
        command.clone(),
        "prepared-altered-proof",
        Duration::from_secs(1),
        &altered,
        &prepared,
    )
    .await;
    assert!(
        matches!(invalid,Err(SendError::Refused(e)) if e.reason.contains("bound FX/risk/calendar proof"))
    );
    assert!(ledger.intents.lock().unwrap().is_empty());
    assert_eq!(orders(&log), 0);
    let before = log.lock().unwrap().payload_types.len();
    let guard = prepared_guard(&conn, &prepared);
    let started = std::time::Instant::now();
    let sent = jiaoyi_ctrader::trading::send_prepared_entry(
        &session,
        &allow(),
        &ledger,
        &policy,
        command,
        "prepared-fast",
        Duration::from_secs(1),
        &guard,
        &prepared,
    )
    .await
    .unwrap();
    let elapsed = started.elapsed();
    assert!(matches!(sent.outcome, Outcome::Filled { .. }));
    assert_eq!(ledger.intents.lock().unwrap().len(), 1);
    let payloads = log.lock().unwrap().payload_types[before..].to_vec();
    assert_eq!(
        payloads,
        vec![pb::ProtoOaPayloadType::ProtoOaNewOrderReq as u32],
        "no trader/book/margin/symbol reads allowed after touch"
    );
    eprintln!(
        "prepared fixture dispatch and response: {}us",
        elapsed.as_micros()
    );
    assert!(
        elapsed < Duration::from_millis(250),
        "bounded in-memory fixture exceeded250ms: {elapsed:?}"
    );
}

#[tokio::test]
async fn prepared_expiry_after_durable_intent_invalidates_other_connection_epoch() {
    let fake = prepared_fake();
    let (conn, log) = fake.clone().spawn();
    let (other, _) = fake.spawn();
    let session = session_on(&conn).await;
    let second = session_on(&other).await;
    let mut quotes = conn.spot_events();
    session.subscribe_spots(&[ETH]).await.unwrap();
    quotes.recv().await.unwrap();
    let ledger = MemLedger {
        intent_delay: Duration::from_millis(80),
        ..Default::default()
    };
    let command = open(1, 28_457_000);
    let policy = on();
    let mut prepared = jiaoyi_ctrader::trading::prepare_entry(
        &session,
        &allow(),
        &ledger,
        &policy,
        &command,
        false,
    )
    .await
    .unwrap();
    let untouched = jiaoyi_ctrader::trading::prepare_entry(
        &second,
        &allow(),
        &ledger,
        &policy,
        &command,
        false,
    )
    .await
    .unwrap();
    // Make the original source expire while the real durable write is pending.
    prepared.restrict_source_start(tokio::time::Instant::now() - Duration::from_millis(4960));
    let guard = prepared_guard(&conn, &prepared);
    let result = jiaoyi_ctrader::trading::send_prepared_entry(
        &session,
        &allow(),
        &ledger,
        &policy,
        command.clone(),
        "prepared-expired",
        Duration::from_secs(1),
        &guard,
        &prepared,
    )
    .await;
    assert!(matches!(result,Err(SendError::Refused(e)) if e.reason.contains("ENTRY_EXPIRED")));
    assert_eq!(orders(&log), 0);
    assert_eq!(ledger.intents.lock().unwrap().len(), 1);
    assert_eq!(conn.entry_revision(ACCOUNT), untouched.revision());
    assert!(
        untouched
            .validate(&second, &ledger.admission_namespace(), &command, &policy)
            .unwrap_err()
            .reason
            .contains("account changed"),
        "a local attempted send must invalidate a different connection before broker events"
    );
}

#[tokio::test]
async fn background_preparation_never_holds_dispatch_lock_during_broker_wait() {
    let fake = prepared_fake();
    let delays = fake.trader_reply_delays.clone();
    let (conn, log) = fake.spawn();
    let session = session_on(&conn).await;
    let mut quotes = conn.spot_events();
    session.subscribe_spots(&[ETH]).await.unwrap();
    quotes.recv().await.unwrap();
    let ledger = MemLedger::default();
    let command = open(1, 28_457_000);
    let policy = on();
    let allowed = allow();
    let prepared = jiaoyi_ctrader::trading::prepare_entry(
        &session, &allowed, &ledger, &policy, &command, false,
    )
    .await
    .unwrap();
    delays.lock().unwrap().push_back(Duration::from_millis(150));
    let refresh = jiaoyi_ctrader::trading::prepare_entry(
        &session, &allowed, &ledger, &policy, &command, false,
    );
    let dispatch = async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let guard = prepared_guard(&conn, &prepared);
        jiaoyi_ctrader::trading::send_prepared_entry(
            &session,
            &allowed,
            &ledger,
            &policy,
            command.clone(),
            "prepared-refresh-race",
            Duration::from_secs(1),
            &guard,
            &prepared,
        )
        .await
    };
    let (refresh, sent) = tokio::join!(refresh, dispatch);
    assert!(
        sent.is_ok(),
        "background refresh must not refuse the existing fresh plan: {sent:?}"
    );
    assert!(
        refresh.is_err(),
        "the concurrent local attempt invalidates the refreshed snapshot"
    );
    assert_eq!(orders(&log), 1);
}

#[tokio::test]
async fn s3_current_label_overrides_historical_authority_for_close_and_amend() {
    for label in [None, Some(""), Some("jiaoyi-s1"), Some("jiaoyi-s3:")] {
        let f = fake();
        let p = position(9, ETH, label);
        let volume = p.trade_data.volume;
        f.positions.lock().unwrap().push(p);
        let (conn, log) = f.spawn();
        let session = session_on(&conn).await;
        let ledger = MemLedger::default();
        ledger.prove(9, Side::Buy, volume);
        ledger
            .authorities
            .lock()
            .unwrap()
            .get_mut(&9)
            .unwrap()
            .entry_key = "s3:owned-opening".into();
        for cmd in [
            Command::Close {
                position_id: 9,
                volume,
            },
            Command::AmendStop {
                position_id: 9,
                side: Side::Buy,
                current_stop: None,
                new_stop: 2700.0,
            },
        ] {
            let result = send(
                &session,
                &allow(),
                &ledger,
                &on(),
                cmd,
                "s3-law1",
                Duration::from_secs(2),
            )
            .await;
            assert!(
                matches!(&result,Err(SendError::Refused(e)) if e.reason.contains("label ownership")),
                "{result:?}"
            );
        }
        assert!(log.lock().unwrap().closes.is_empty());
        assert!(log.lock().unwrap().amends.is_empty());
        assert!(ledger.intents.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn s3_pending_external_order_blocks_then_releases_without_cancellation() {
    for label in [None, Some(""), Some("jiaoyi-s1")] {
        let f = fake();
        f.scripted_books
            .lock()
            .unwrap()
            .push_back(pb::ProtoOaReconcileRes {
                ctid_trader_account_id: ACCOUNT,
                order: vec![pb::ProtoOaOrder {
                    order_id: 7,
                    trade_data: pb::ProtoOaTradeData {
                        symbol_id: ETH,
                        volume: 100,
                        trade_side: 1,
                        label: label.map(str::to_owned),
                        ..Default::default()
                    },
                    order_type: 2,
                    order_status: 1,
                    ..Default::default()
                }],
                ..Default::default()
            });
        let (conn, log) = f.spawn();
        let session = session_on(&conn).await;
        let ledger = MemLedger::default();
        let result = send(
            &session,
            &allow(),
            &ledger,
            &on(),
            open_s3(Some(1_000_000)),
            "blocked",
            Duration::from_secs(2),
        )
        .await;
        assert!(
            matches!(&result, Err(SendError::Refused(r)) if r.reason.contains("yield_to_human")),
            "{result:?}"
        );
        assert_eq!(orders(&log), 0);
        assert!(ledger.intents.lock().unwrap().is_empty());
        // The next reconcile snapshot has no pending order. No manual reset
        // or bot cancellation is required to release the symbol.
        let result = send(
            &session,
            &allow(),
            &ledger,
            &on(),
            open_s3(Some(1_000_000)),
            "released",
            Duration::from_secs(2),
        )
        .await;
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(orders(&log), 1);
        assert!(log.lock().unwrap().amends.is_empty());
        assert!(log.lock().unwrap().closes.is_empty());
    }
}

#[tokio::test]
async fn confirmed_prepared_pair_checks_exact_broker_book_before_consuming_intent() {
    use jiaoyi_ctrader::entry_guard::{EntryGuard, Signal};
    for label in [None, Some("jiaoyi-s3")] {
        let mut f = prepared_fake();
        f.quotes
            .insert(5, vec![(Some(20_050_000), Some(20_070_000))]);
        f.positions.lock().unwrap().push(position(777, 5, label));
        let book = f.positions.clone();
        let (conn, log) = f.spawn();
        let session = session_on(&conn).await;
        let mut quotes = conn.spot_events();
        session.subscribe_spots(&[ETH, 5]).await.unwrap();
        quotes.recv().await.unwrap();
        quotes.recv().await.unwrap();
        let ledger = MemLedger::default();
        let command = open_s3(Some(1_000_000));
        let policy = on();
        let make_guard = |p: &jiaoyi_ctrader::trading::PreparedEntry| {
            let now = chrono::Utc::now();
            EntryGuard {
                connection_generation: p.generation(),
                account_revision: p.revision(),
                account_id: ACCOUNT,
                symbol_id: ETH,
                side: Side::Buy,
                signal: Signal::PullbackConfirmed {
                    origin: 2990.,
                    level: 3000.,
                    impulse_range: 20.,
                    admitted_entry: 3002.5,
                    price_digits: 2,
                    peer: Some((5, 200.)),
                    proof: jiaoyi_core::strategy3::macro_pullback::TurnProof {
                        origin: 2990.,
                        extreme: 3010.,
                        long: true,
                        touched_at: now - chrono::Duration::seconds(2),
                        turn_extreme: 2998.8,
                        turn_extreme_at: now - chrono::Duration::seconds(1),
                        confirmed_at: now,
                        confirmed_bid: 3000.,
                        confirmed_ask: 3001.,
                    },
                    margin: p.receipt.fx_guard.clone(),
                    calendar: Some(p.calendar.clone()),
                },
                cost: None,
                expires: p.expires(),
            }
        };
        let prepared = jiaoyi_ctrader::trading::prepare_entry(
            &session,
            &allow(),
            &ledger,
            &policy,
            &command,
            false,
        )
        .await
        .unwrap();
        let guard = make_guard(&prepared);
        let result = jiaoyi_ctrader::trading::send_prepared_entry(
            &session,
            &allow(),
            &ledger,
            &policy,
            command.clone(),
            "pair-held",
            Duration::from_secs(1),
            &guard,
            &prepared,
        )
        .await;
        assert!(
            matches!(&result,Err(SendError::Refused(r)) if r.reason=="pair_risk_reservation_active"),
            "{result:?}"
        );
        assert!(ledger.intents.lock().unwrap().is_empty());
        assert_eq!(orders(&log), 0);
        book.lock().unwrap().clear();
        // Refreshing the broker admission, not mutating an actor's local view,
        // is what grants authority after the sibling has gone away.
        let released = jiaoyi_ctrader::trading::prepare_entry(
            &session,
            &allow(),
            &ledger,
            &policy,
            &command,
            false,
        )
        .await
        .unwrap();
        let guard = make_guard(&released);
        let sent = jiaoyi_ctrader::trading::send_prepared_entry(
            &session,
            &allow(),
            &ledger,
            &policy,
            command,
            "pair-released",
            Duration::from_secs(1),
            &guard,
            &released,
        )
        .await
        .unwrap();
        assert!(matches!(sent.outcome, Outcome::Filled { .. }));
        assert_eq!(orders(&log), 1);
    }
}
