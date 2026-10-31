//! Shared outcome qualification at the store, limits and Kelly-input boundaries.
//! These are local PostgreSQL fixtures, never orders sent to a broker.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::strategy3::kelly;
use jiaoyi_store::{
    self as store,
    goals::{self, AssessmentState},
    history::{self, DealRow},
    trading,
};
use serde_json::json;
use sqlx::PgPool;

const A: i64 = 47790949;
const B: i64 = 46130058;
const HASH: &str = "qualification-current-hash";

#[derive(Clone, Copy)]
struct Opening<'a> {
    account: i64,
    strategy: &'a str,
    id: i64,
    requested: i64,
    confirmed: i64,
    history: i64,
    symbol: i64,
    risk: f64,
}

impl Opening<'_> {
    fn key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.strategy, self.account, self.symbol, self.id
        )
    }
    fn deal_id(&self) -> i64 {
        self.id * 100
    }
}

fn opening(account: i64, strategy: &str, id: i64) -> Opening<'_> {
    Opening {
        account,
        strategy,
        id,
        requested: 100,
        confirmed: 100,
        history: 100,
        symbol: 1,
        risk: 1.0,
    }
}

fn at(id: i64) -> DateTime<Utc> {
    Utc::now() - Duration::days(2) + Duration::minutes(id)
}

fn deal(o: Opening<'_>, deal_id: i64, order_id: i64, volume: i64, net: Option<i64>) -> DealRow {
    let closing = net.is_some();
    let timestamp = at(o.id) - Duration::seconds(if closing { 0 } else { 3600 });
    DealRow {
        deal_id,
        order_id,
        position_id: o.id,
        broker_symbol_id: o.symbol,
        volume,
        filled_volume: volume,
        trade_side: if closing { "SELL" } else { "BUY" }.into(),
        deal_status: "FILLED".into(),
        created_at: timestamp,
        executed_at: timestamp,
        last_update_at: Some(timestamp),
        execution_price: Some(1.0),
        margin_rate: None,
        commission: Some(0),
        money_digits: Some(2),
        base_to_usd_rate: None,
        is_closing: closing,
        close_entry_price: closing.then_some(1.0),
        close_gross_profit: net,
        close_swap: closing.then_some(0),
        close_commission: closing.then_some(0),
        close_balance: None,
        close_quote_to_deposit_rate: closing.then_some(1.0),
        close_closed_volume: closing.then_some(volume),
        close_balance_version: None,
        close_money_digits: closing.then_some(2),
        close_pnl_conversion_fee: closing.then_some(0),
        raw: format!("{deal_id}:{order_id}:{volume}:{net:?}").into_bytes(),
    }
}

async fn ingest(pool: &PgPool, account: i64, row: DealRow) -> history::DealUpsert {
    history::upsert_deals(pool, account, &[row], Utc::now())
        .await
        .unwrap()
}

async fn open(pool: &PgPool, o: Opening<'_>) {
    let key = o.key();
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',$3,'buy',$4,100000,$5,1)")
        .bind(o.account).bind(&key).bind(o.symbol).bind(o.requested).bind(o.risk)
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$4,$4,$5,'buy',$6,''::bytea,$7)")
        .bind(o.account).bind(&key).bind(o.id).bind(o.deal_id()).bind(o.symbol).bind(o.confirmed)
        .bind(at(o.id)-Duration::hours(1)).execute(pool).await.unwrap();
    if o.history > 0 {
        ingest(
            pool,
            o.account,
            deal(o, o.deal_id(), o.deal_id(), o.history, None),
        )
        .await;
    }
}

async fn close_receipt(pool: &PgPool, o: Opening<'_>, deal_id: i64, volume: i64) {
    let key = format!("{}-end:{}:{}:{deal_id}", o.strategy, o.account, o.id);
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,position_id,volume,max_risk) VALUES($1,$2,$2,'close',$3,$4,1)")
        .bind(o.account).bind(&key).bind(o.id).bind(volume).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$4,$4,$5,'sell',$6,''::bytea,$7)")
        .bind(o.account).bind(&key).bind(o.id).bind(deal_id).bind(o.symbol).bind(volume)
        .bind(at(o.id)).execute(pool).await.unwrap();
}

async fn fresh(pool: &PgPool, account: i64) {
    let through = Utc::now() - Duration::seconds(1);
    sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete) VALUES($1,$2,$3,$3,'ok',false)")
        .bind(account).bind(through-Duration::days(7)).bind(through).execute(pool).await.unwrap();
}

async fn s3_order(pool: &PgPool, o: Opening<'_>, hash: &str) {
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,decided_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'open','TEST',$4,$5,'buy',$6,$7)")
        .bind(o.account).bind(o.key()).bind(hash).bind(o.symbol).bind(o.id)
        .bind(at(o.id)-Duration::hours(1)).bind(o.requested).execute(pool).await.unwrap();
}

async fn selected_kelly_inputs(pool: &PgPool) -> Vec<trading::ClosedTrade> {
    // This is the production actor's reader and selector, with persisted source
    // rows. It verifies consumer inputs, not a broker order-admission action.
    let keys = store::strategy3::open_keys_for_hash(pool, A, 1, HASH)
        .await
        .unwrap();
    let (closed, _) = trading::closed_trades(pool, A, "s3:").await.unwrap();
    closed
        .into_iter()
        .filter(|t| keys.iter().any(|key| key == &t.entry_key))
        .collect()
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn terminal_receipt_before_history_is_unknown_until_real_ingestion(pool: PgPool) {
    let o = opening(A, "s1", 1);
    open(&pool, o).await;
    close_receipt(&pool, o, 101, 100).await;
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s1:").await.unwrap();
    assert!(closed.is_empty());
    assert_eq!(unknown, 1);
    let before = jiaoyi::limits::health(&pool, A, &jiaoyi::limits::S1)
        .await
        .unwrap();
    assert_eq!(before["evidence_state"], "incomplete");
    assert_eq!(before["limit_status"], "unknown");
    assert!(before["net_r"].is_null());
    assert_eq!(
        goals::assess(&pool, A, "s1").await.unwrap().state,
        AssessmentState::Unknown
    );
    assert_eq!(
        ingest(&pool, A, deal(o, 101, 101, 100, Some(-100)))
            .await
            .inserted,
        1
    );
    let after = jiaoyi::limits::health(&pool, A, &jiaoyi::limits::S1)
        .await
        .unwrap();
    assert_eq!(after["closed_trades"], 1);
    assert_eq!(after["unmeasured"], 0);
    assert_eq!(after["evidence_state"], "measured");
    assert_eq!(after["net_r"], -1.0);
    assert_eq!(
        goals::assess(&pool, A, "s1")
            .await
            .unwrap()
            .measured_closed_positions,
        1
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn history_revision_retains_alert_then_valid_reconciliation_can_clear(pool: PgPool) {
    for id in 1..=10 {
        let o = opening(A, "s1", id);
        open(&pool, o).await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(-100)),
        )
        .await;
    }
    let o = opening(A, "s1", 10);
    close_receipt(&pool, o, 1001, 100).await;
    fresh(&pool, A).await;
    assert_eq!(
        jiaoyi::limits::check(&pool, A, &jiaoyi::limits::S1, json!({}), None)
            .await
            .unwrap(),
        vec!["drawdown_r"]
    );
    let mut revised = deal(o, 1001, 1001, 100, Some(-100));
    revised.filled_volume = 0;
    revised.raw = b"broker source revision: filled quantity zero".to_vec();
    assert_eq!(ingest(&pool, A, revised).await.changed, 1);
    let health = jiaoyi::limits::health(&pool, A, &jiaoyi::limits::S1)
        .await
        .unwrap();
    assert_eq!(health["closed_trades"], 9);
    assert_eq!(health["unmeasured"], 1);
    assert_eq!(health["evidence_state"], "incomplete");
    assert_eq!(health["retained_limit_alerts"], json!(["drawdown_r"]));
    jiaoyi::limits::check(&pool, A, &jiaoyi::limits::S1, json!({}), None)
        .await
        .unwrap();
    let cleared: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trading_decisions WHERE account_id=$1 AND kind='limit_cleared'",
    )
    .bind(A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cleared, 0, "missing source evidence is not recovery");
    assert_eq!(
        goals::assess(&pool, A, "s1").await.unwrap().state,
        AssessmentState::Unknown
    );
    // The authoritative correction now proves a complete, positive final close;
    // maximum drawdown is nine R. This is a genuine qualified later clearance.
    assert_eq!(
        ingest(&pool, A, deal(o, 1001, 1001, 100, Some(100)))
            .await
            .changed,
        1
    );
    assert!(
        jiaoyi::limits::check(&pool, A, &jiaoyi::limits::S1, json!({}), None)
            .await
            .unwrap()
            .is_empty()
    );
    let health = jiaoyi::limits::health(&pool, A, &jiaoyi::limits::S1)
        .await
        .unwrap();
    assert_eq!(health["unmeasured"], 0);
    assert_eq!(health["limit_status"], "within_limits");
    assert_eq!(health["max_drawdown_r"], 9.0);
    let cleared: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trading_decisions WHERE account_id=$1 AND kind='limit_cleared'",
    )
    .bind(A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cleared, 1);
    let revisions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM reconcile_events WHERE account_id=$1 AND kind='deal_changed'",
    )
    .bind(A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        revisions, 2,
        "real upserts retain both reconciliation receipts"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn persisted_kelly_cohort_excludes_incomplete_and_other_hash_symbol_account(pool: PgPool) {
    for id in 1..=19 {
        let o = opening(A, "s3", id);
        open(&pool, o).await;
        s3_order(&pool, o, HASH).await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(-100)),
        )
        .await;
    }
    for (o, hash) in [
        (opening(A, "s3", 101), "old-hash"),
        (
            Opening {
                symbol: 2,
                ..opening(A, "s3", 102)
            },
            HASH,
        ),
        (opening(B, "s3", 1), HASH),
    ] {
        open(&pool, o).await;
        s3_order(&pool, o, hash).await;
        ingest(
            &pool,
            o.account,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(9900)),
        )
        .await;
    }
    let candidate = Opening {
        history: 50,
        ..opening(A, "s3", 20)
    };
    open(&pool, candidate).await;
    s3_order(&pool, candidate, HASH).await;
    close_receipt(&pool, candidate, 2001, 50).await;
    ingest(&pool, A, deal(candidate, 2001, 2001, 50, Some(100))).await;
    fresh(&pool, A).await;
    let selected = selected_kelly_inputs(&pool).await;
    assert_eq!(
        selected.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        (1..=19).collect::<Vec<_>>()
    );
    let reading = kelly::read(&selected.iter().map(|t| t.r).collect::<Vec<_>>());
    assert_eq!(reading.n, 19);
    assert!(!kelly::skip(&reading));
    let (_, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!(unknown, 1);
    let goal = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(goal.state, AssessmentState::Unknown);
    assert!(!goal.window.position_ids.contains(&20));
    // Opening history catches up: the genuine half-close stays outside Kelly.
    ingest(&pool, A, deal(candidate, 2000, 2000, 100, None)).await;
    assert_eq!(selected_kelly_inputs(&pool).await.len(), 19);
    assert_eq!(trading::closed_trades(&pool, A, "s3:").await.unwrap().1, 0);
    close_receipt(&pool, candidate, 2002, 50).await;
    ingest(&pool, A, deal(candidate, 2002, 2002, 50, Some(100))).await;
    let selected = selected_kelly_inputs(&pool).await;
    assert_eq!(
        selected.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        (1..=20).collect::<Vec<_>>()
    );
    assert_eq!(selected.last().unwrap().r, 2.0);
    let reading = kelly::read(&selected.iter().map(|t| t.r).collect::<Vec<_>>());
    assert_eq!(reading.n, 20);
    assert!(kelly::skip(&reading));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn opening_increments_need_owned_provenance_even_inside_original_request(pool: PgPool) {
    for id in 1..=3 {
        let o = Opening {
            confirmed: 50,
            history: 50,
            ..opening(A, "s3", id)
        };
        open(&pool, o).await;
        let extra_order = if id == 2 { o.deal_id() } else { 9000 + id };
        ingest(&pool, A, deal(o, o.deal_id() + 2, extra_order, 25, None)).await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 75, Some(75)),
        )
        .await;
        if id == 1 {
            // Explicitly another owner's opening. The third position instead
            // has no provenance for its extra order; neither is the owned fill.
            let key = "external:known-addition";
            sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'buy',25,100000,1,1)")
                .bind(A).bind(key).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',1,9001,102,1,'buy',25,''::bytea,$3)")
                .bind(A).bind(key).bind(at(1)).execute(&pool).await.unwrap();
        }
    }
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!(
        closed.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(unknown, 2);
    assert_eq!(
        closed[0].r, 1.0,
        "75 of 100 owned units scales the original risk to 0.75"
    );
    let assessment = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(assessment.state, AssessmentState::Unknown);
    assert_eq!(assessment.measured_closed_positions, 1);
    assert_eq!(assessment.unmeasured_positions, 2);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM deals WHERE account_id=$1")
        .bind(A)
        .fetch_one(&pool)
        .await
        .unwrap();
    let cash: i64 = sqlx::query_scalar(
        "SELECT SUM(close_gross_profit)::bigint FROM deals WHERE account_id=$1 AND is_closing",
    )
    .bind(A)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (count, cash),
        (9, 225),
        "qualification preserves every source row and its cash"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn full_partial_and_broker_exit_neighbours_survive_receipt_echoes(pool: PgPool) {
    let full = opening(A, "s3", 1);
    open(&pool, full).await;
    ingest(&pool, A, deal(full, 101, 101, 100, Some(100))).await;
    let partial = Opening {
        requested: 200,
        ..opening(A, "s3", 2)
    };
    open(&pool, partial).await;
    close_receipt(&pool, partial, 201, 40).await;
    ingest(&pool, A, deal(partial, 201, 201, 40, Some(80))).await;
    let broker = opening(A, "s3", 3);
    open(&pool, broker).await;
    let key = "s3-broker:47790949:3:301";
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,decided_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',1,3,'sell',$4,100)")
        .bind(A).bind(key).bind(HASH).bind(at(3)).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,side,outcome,sent_at,answered_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',3,'sell','filled',$4,$4,100)")
        .bind(A).bind(key).bind(HASH).bind(at(3)).execute(&pool).await.unwrap();
    ingest(&pool, A, deal(broker, 301, 301, 100, Some(-100))).await;
    // Replayed opening receipt: immutable originals stay intact and the same
    // DealID must not turn 100 opened into 200 merely because it was delivered twice.
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) SELECT account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at + interval '1 millisecond' FROM trading_events WHERE account_id=$1 AND position_id=1 AND deal_id=100")
        .bind(A).execute(&pool).await.unwrap();
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!(unknown, 0);
    assert_eq!(
        closed
            .iter()
            .map(|t| (t.position_id, t.r))
            .collect::<Vec<_>>(),
        vec![(1, 1.0), (3, -1.0)]
    );
    close_receipt(&pool, partial, 202, 60).await;
    ingest(&pool, A, deal(partial, 202, 202, 60, Some(120))).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!(unknown, 0);
    assert_eq!(
        closed
            .iter()
            .map(|t| (t.position_id, t.r))
            .collect::<Vec<_>>(),
        vec![(1, 1.0), (2, 4.0), (3, -1.0)]
    );
    let assessment = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(assessment.state, AssessmentState::InsufficientEvidence);
    assert_eq!(
        (
            assessment.measured_closed_positions,
            assessment.unmeasured_positions
        ),
        (3, 0)
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn invalid_precision_and_nonfinite_r_are_unknown_without_changing_valid_cash(pool: PgPool) {
    let bad_digits = opening(A, "s1", 1);
    open(&pool, bad_digits).await;
    let mut row = deal(bad_digits, 101, 101, 100, Some(-100));
    row.close_money_digits = Some(-1000);
    row.raw = b"malformed close precision".to_vec();
    ingest(&pool, A, row).await;
    let overflow_r = Opening {
        risk: 1e-310,
        ..opening(A, "s1", 2)
    };
    open(&pool, overflow_r).await;
    ingest(&pool, A, deal(overflow_r, 201, 201, 100, Some(100))).await;
    let valid = opening(A, "s1", 3);
    open(&pool, valid).await;
    ingest(&pool, A, deal(valid, 301, 301, 100, Some(-100))).await;
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s1:").await.unwrap();
    assert_eq!(
        closed.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        vec![3]
    );
    assert_eq!(unknown, 2);
    assert!(closed.iter().all(|t| t.r.is_finite()));
    let health = jiaoyi::limits::health(&pool, A, &jiaoyi::limits::S1)
        .await
        .unwrap();
    assert_eq!(health["evidence_state"], "incomplete");
    assert_eq!(health["net_r"], -1.0);
    assert!(health["to_limits"]["drawdown_room_r"].is_null());
    assert_eq!(
        goals::assess(&pool, A, "s1").await.unwrap().state,
        AssessmentState::Unknown
    );
    ingest(&pool, A, deal(bad_digits, 101, 101, 100, Some(-100))).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s1:").await.unwrap();
    assert_eq!(
        closed.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        vec![1, 3]
    );
    assert_eq!(unknown, 1);
    assert_eq!(closed.iter().map(|t| t.r).sum::<f64>(), -2.0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn another_deal_cannot_replace_a_receipts_missing_quantity(pool: PgPool) {
    let local = opening(A, "s3", 1);
    open(&pool, local).await;
    close_receipt(&pool, local, 101, 100).await;
    ingest(&pool, A, deal(local, 101, 101, 100, Some(100))).await;

    let broker = opening(A, "s3", 2);
    open(&pool, broker).await;
    let key = "s3-broker:47790949:2:201";
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,decided_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',1,2,'sell',$4,100)")
        .bind(A).bind(key).bind(HASH).bind(at(2)).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,side,outcome,sent_at,answered_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',2,'sell','filled',$4,$4,100)")
        .bind(A).bind(key).bind(HASH).bind(at(2)).execute(&pool).await.unwrap();
    ingest(&pool, A, deal(broker, 201, 201, 100, Some(100))).await;

    let split = Opening {
        confirmed: 50,
        history: 50,
        ..opening(A, "s3", 3)
    };
    open(&pool, split).await;
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',3,300,302,1,'buy',50,''::bytea,$3)")
        .bind(A).bind(split.key()).bind(at(3)-Duration::minutes(30)).execute(&pool).await.unwrap();
    ingest(&pool, A, deal(split, 302, 300, 50, None)).await;
    close_receipt(&pool, split, 301, 100).await;
    ingest(&pool, A, deal(split, 301, 301, 100, Some(100))).await;
    assert_eq!(
        trading::closed_trades(&pool, A, "s3:")
            .await
            .unwrap()
            .0
            .len(),
        3
    );

    // Production ingestion preserves a zero-filled receipt-named row while
    // another close supplies equal aggregate quantity. Neither receipt is met.
    for o in [local, broker] {
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 0, Some(100)),
        )
        .await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 2, o.deal_id() + 2, 100, Some(100)),
        )
        .await;
    }
    // Likewise one owned opening deal cannot conceal another named fill.
    ingest(&pool, A, deal(split, 300, 300, 100, None)).await;
    ingest(&pool, A, deal(split, 302, 300, 0, None)).await;
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert!(closed.is_empty());
    assert_eq!(unknown, 3);
    let goal = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(goal.state, AssessmentState::Unknown);
    assert_eq!(
        (goal.measured_closed_positions, goal.unmeasured_positions),
        (0, 3)
    );
    assert!(goal
        .missing_reasons
        .iter()
        .any(|r| r.contains("opening history trails")));
    assert!(goal
        .missing_reasons
        .iter()
        .any(|r| r.contains("terminal exit receipts")));

    // Exact per-deal reconciliation restores legitimate whole and split fills.
    for o in [local, broker] {
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(100)),
        )
        .await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 2, o.deal_id() + 2, 0, Some(100)),
        )
        .await;
    }
    ingest(&pool, A, deal(split, 300, 300, 50, None)).await;
    ingest(&pool, A, deal(split, 302, 300, 50, None)).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!((closed.len(), unknown), (3, 0));
    assert!(closed.iter().all(|t| t.r == 1.0));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn unkeyed_fills_require_matching_position_and_quantity_until_catchup(pool: PgPool) {
    for id in 1..=3 {
        let o = opening(A, "s3", id);
        open(&pool, o).await;
        sqlx::query("INSERT INTO trading_events(account_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,2126,'ORDER_FILLED',$2,$3,$3,1,'sell',100,''::bytea,$4)")
            .bind(A).bind(id).bind(o.deal_id()+1).bind(at(id)).execute(&pool).await.unwrap();
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(100)),
        )
        .await;
    }
    // A smaller canonical broker record cannot discharge the unkeyed100.
    let key = "s3-broker:47790949:2:201";
    sqlx::query("INSERT INTO strategy3_orders(account_id,entry_key,label,prereg_hash,kind,symbol,symbol_id,position_id,side,decided_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',1,2,'sell',$4,50)")
        .bind(A).bind(key).bind(HASH).bind(at(2)).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO strategy3_fills(account_id,entry_key,label,prereg_hash,kind,symbol,position_id,side,outcome,sent_at,answered_at,volume) VALUES($1,$2,'jiaoyi-s3',$3,'broker_exit','TEST',2,'sell','filled',$4,$4,50)")
        .bind(A).bind(key).bind(HASH).bind(at(2)).execute(&pool).await.unwrap();
    assert_eq!(
        trading::closed_trades(&pool, A, "s3:")
            .await
            .unwrap()
            .0
            .len(),
        3
    );

    let first = opening(A, "s3", 1);
    ingest(&pool, A, deal(first, 101, 101, 0, Some(100))).await;
    ingest(&pool, A, deal(first, 102, 102, 100, Some(100))).await;
    let second = opening(A, "s3", 2);
    ingest(&pool, A, deal(second, 201, 201, 50, Some(50))).await;
    ingest(&pool, A, deal(second, 202, 202, 50, Some(50))).await;
    let third = opening(A, "s3", 3);
    let mut wrong_position = deal(third, 301, 301, 100, Some(100));
    wrong_position.position_id = 999;
    wrong_position.raw.extend_from_slice(b":different-position");
    ingest(&pool, A, wrong_position).await;
    ingest(&pool, A, deal(third, 302, 302, 100, Some(100))).await;
    fresh(&pool, A).await;
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert!(closed.is_empty());
    assert_eq!(unknown, 3);
    let goal = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(goal.state, AssessmentState::Unknown);
    assert_eq!(goal.unmeasured_positions, 3);
    assert_eq!(
        goal.missing_reasons
            .iter()
            .filter(|r| r.contains("unclassified owned fill"))
            .count(),
        3
    );

    for o in [first, second, third] {
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 1, o.deal_id() + 1, 100, Some(100)),
        )
        .await;
        ingest(
            &pool,
            A,
            deal(o, o.deal_id() + 2, o.deal_id() + 2, 0, Some(100)),
        )
        .await;
    }
    let (closed, unknown) = trading::closed_trades(&pool, A, "s3:").await.unwrap();
    assert_eq!((closed.len(), unknown), (3, 0));
    assert!(closed.iter().all(|t| t.r == 1.0));
}
