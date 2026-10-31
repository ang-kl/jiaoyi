//! Goal evidence against real PostgreSQL and existing action-owner tables.
use chrono::{DateTime, Duration, Utc};
use jiaoyi_store::{
    self as store,
    goals::{self, AssessmentState},
};
use sqlx::PgPool;

const A: i64 = 47790949;
const B: i64 = 46130058;
fn t0() -> DateTime<Utc> {
    Utc::now() - Duration::days(2)
}

async fn position(
    pool: &PgPool,
    account: i64,
    code: &str,
    id: i64,
    r: Option<f64>,
    opening_history: i64,
) {
    position_with_requested_volume(pool, account, code, id, r, opening_history, Some(100)).await;
}

async fn position_with_requested_volume(
    pool: &PgPool,
    account: i64,
    code: &str,
    id: i64,
    r: Option<f64>,
    opening_history: i64,
    requested_volume: Option<i64>,
) {
    let key = format!("{code}:{account}:1:{id}");
    let at = t0() + Duration::minutes(id);
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'buy',$3,100000,1,1)")
        .bind(account).bind(&key).bind(requested_volume).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,symbol_id,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$4,1,100,''::bytea,$5)")
        .bind(account).bind(&key).bind(id).bind(id*10).bind(at-Duration::hours(1)).execute(pool).await.unwrap();
    if opening_history > 0 {
        deal(
            pool,
            account,
            id,
            id * 10,
            opening_history,
            false,
            None,
            at - Duration::hours(1),
        )
        .await;
    }
    if let Some(r) = r {
        deal(
            pool,
            account,
            id,
            id * 10 + 1,
            opening_history,
            true,
            Some((r * 100.0).round() as i64),
            at,
        )
        .await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn deal(
    pool: &PgPool,
    account: i64,
    position: i64,
    id: i64,
    volume: i64,
    closing: bool,
    net: Option<i64>,
    at: DateTime<Utc>,
) {
    sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,money_digits,is_closing,close_gross_profit,close_swap,close_commission,close_pnl_conversion_fee,close_money_digits,close_closed_volume,raw,fetched_at) VALUES($1,$2,$2,$3,1,$4,$4,'BUY','FILLED',$5,$5,2,$6,$7,$8,$9,$10,$11,$12,''::bytea,now())")
      .bind(account).bind(id).bind(position).bind(volume).bind(at).bind(closing)
      .bind(net.map(|n|n+40)).bind(closing.then_some(0i64)).bind(closing.then_some(-40i64))
      .bind(closing.then_some(0i64)).bind(closing.then_some(2i32)).bind(closing.then_some(volume))
      .execute(pool).await.unwrap();
}
async fn passing(pool: &PgPool, account: i64, code: &str) {
    for id in 1..=20 {
        position(
            pool,
            account,
            code,
            id,
            Some(if id <= 15 { 2.0 } else { -1.0 }),
            100,
        )
        .await;
    }
}
async fn fetch(pool: &PgPool, account: i64, age: i64, ok: bool, future: bool) {
    let through = Utc::now() + Duration::seconds(if future { 60 } else { -age - 1 });
    sqlx::query("INSERT INTO deal_fetches(account_id,window_from,window_to,fetched_at,status,complete,error) VALUES($1,$2,$3,$3,$4,false,$5)")
     .bind(account).bind(through-Duration::days(7)).bind(through).bind(if ok {"ok"}else{"failed"})
     .bind((!ok).then_some("broker unavailable")).execute(pool).await.unwrap();
}
async fn close_receipt(
    pool: &PgPool,
    account: i64,
    code: &str,
    id: i64,
    volume: i64,
    attempt: i64,
) {
    let key = format!("{code}-end:{account}:{id}:{attempt}");
    let at = t0() + Duration::minutes(id) + Duration::seconds(attempt);
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,position_id,volume,max_risk) VALUES($1,$2,$2,'close',$3,$4,1)")
      .bind(account).bind(&key).bind(id).bind(volume).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,$4,$5,''::bytea,$6)")
      .bind(account).bind(&key).bind(id).bind(id*10+attempt).bind(volume).bind(at).execute(pool).await.unwrap();
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn latest_window_and_account_strategy_scope_are_real_sources(pool: PgPool) {
    passing(&pool, A, "s3").await;
    position(&pool, A, "s3", 21, Some(-1.0), 100).await;
    passing(&pool, B, "s3").await;
    // Another strategy on A gets independent positions/intent identities.
    for id in 100..120 {
        position(&pool, A, "s1", id, Some(-1.0), 100).await;
    }
    fetch(&pool, A, 0, true, false).await;
    fetch(&pool, B, 0, true, false).await;
    let a = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(a.window.position_ids, (2..=21).collect::<Vec<_>>());
    assert_eq!((a.window.wins, a.window.losses), (14, 6));
    assert_eq!(a.state, AssessmentState::BelowTarget);
    assert_eq!(
        goals::assess(&pool, B, "s3").await.unwrap().state,
        AssessmentState::WindowMeetsNumericReference
    );
    let s1 = goals::assess(&pool, A, "s1").await.unwrap();
    assert_eq!(s1.window.wins, 0);
    assert_eq!(s1.window.position_ids, (100..120).collect::<Vec<_>>());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn original_requested_volume_must_prove_the_filled_share_of_risk(pool: PgPool) {
    for (index, requested) in [None, Some(0), Some(-100), Some(50)]
        .into_iter()
        .enumerate()
    {
        let account = A + index as i64;
        passing(&pool, account, "s3").await;
        fetch(&pool, account, 0, true, false).await;
        assert_eq!(
            goals::assess(&pool, account, "s3").await.unwrap().state,
            AssessmentState::WindowMeetsNumericReference
        );
        position_with_requested_volume(&pool, account, "s3", 21, Some(2.0), 100, requested).await;
        let assessment = goals::assess(&pool, account, "s3").await.unwrap();
        assert_eq!(
            assessment.state,
            AssessmentState::Unknown,
            "requested volume {requested:?} cannot prove the risk share for 100 filled units"
        );
        assert_eq!(assessment.measured_closed_positions, 20);
        assert_eq!(assessment.unmeasured_positions, 1);
        assert!(!assessment.window.position_ids.contains(&21));
        assert!(!assessment.history_complete);
        assert!(assessment
            .missing_reasons
            .iter()
            .any(|reason| reason.contains("original requested volume")));
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn valid_partial_fill_risk_and_unclosed_positions_keep_their_meaning(pool: PgPool) {
    for (account, code) in [(A, "s1"), (B, "s3")] {
        passing(&pool, account, code).await;
        // Only half of the request filled: the recorded risk of 1 belongs to
        // 200 requested units, so the 100-unit closed position's risk is 0.5.
        position_with_requested_volume(&pool, account, code, 21, Some(2.0), 100, Some(200)).await;
        // An incomplete open is outside the fully closed cohort.
        position_with_requested_volume(&pool, account, code, 22, None, 100, None).await;
        fetch(&pool, account, 0, true, false).await;
        let assessment = goals::assess(&pool, account, code).await.unwrap();
        assert_eq!(
            assessment.state,
            AssessmentState::WindowMeetsNumericReference
        );
        assert_eq!(assessment.measured_closed_positions, 21);
        assert_eq!(assessment.unmeasured_positions, 0);
        assert_eq!(assessment.window.position_ids, (2..=21).collect::<Vec<_>>());
        assert_eq!(assessment.window.net_r, 27.0);
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn terminal_exit_waits_for_history_while_partial_exit_stays_open(pool: PgPool) {
    passing(&pool, A, "s3").await;
    position(&pool, A, "s3", 99, None, 100).await;
    close_receipt(&pool, A, "s3", 99, 40, 1).await;
    fetch(&pool, A, 0, true, false).await;
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::WindowMeetsNumericReference,
        "40 of 100 is still open"
    );
    close_receipt(&pool, A, "s3", 99, 60, 2).await;
    let pending = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(pending.state, AssessmentState::Unknown);
    assert!(pending
        .missing_reasons
        .iter()
        .any(|r| r.contains("terminal exit receipts")));
    deal(
        &pool,
        A,
        99,
        991,
        40,
        true,
        Some(-40),
        t0() + Duration::minutes(99) + Duration::seconds(1),
    )
    .await;
    deal(
        &pool,
        A,
        99,
        992,
        60,
        true,
        Some(-60),
        t0() + Duration::minutes(99) + Duration::seconds(2),
    )
    .await;
    fetch(&pool, A, 0, true, false).await;
    let settled = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(settled.state, AssessmentState::BelowTarget);
    assert_eq!(settled.window.position_ids.last(), Some(&99));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn performance_does_not_price_missing_closing_costs_as_zero(pool: PgPool) {
    position(&pool, A, "s3", 1, Some(2.0), 100).await;
    for (column, original) in [
        ("close_swap", 0_i64),
        ("close_commission", -40),
        ("close_pnl_conversion_fee", 0),
    ] {
        sqlx::query(&format!(
            "UPDATE deals SET {column}=NULL WHERE account_id=$1 AND deal_id=11"
        ))
        .bind(A)
        .execute(&pool)
        .await
        .unwrap();
        let (closed, unmeasured) = store::trading::closed_trades(&pool, A, "s3:")
            .await
            .unwrap();
        assert!(closed.is_empty(), "{column}: unknown costs are not net R");
        assert_eq!(unmeasured, 1, "{column}");
        let p = store::trading::performance(&pool, A, "s3").await.unwrap();
        assert_eq!((p.trades, p.unmeasured), (0, 1), "{column}");
        sqlx::query(&format!(
            "UPDATE deals SET {column}=$2 WHERE account_id=$1 AND deal_id=11"
        ))
        .bind(A)
        .bind(original)
        .execute(&pool)
        .await
        .unwrap();
        let p = store::trading::performance(&pool, A, "s3").await.unwrap();
        assert_eq!((p.trades, p.unmeasured), (1, 0), "{column}");
        assert!((p.net_r - 2.0).abs() < 1e-12, "{column}");
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn closed_performance_batches_history_without_changing_position_semantics(pool: PgPool) {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    for id in 1..=64 {
        position(
            &pool,
            A,
            "s3",
            id,
            Some(if id % 2 == 0 { 2.0 } else { -1.0 }),
            100,
        )
        .await;
    }
    // Same broker position id on another account and a different strategy
    // must not leak into the selected owned cohort.
    position(&pool, B, "s3", 1, Some(99.0), 100).await;
    position(&pool, A, "s1", 1000, Some(99.0), 100).await;
    deal(&pool, A, 1001, 10010, 100, false, None, t0()).await;
    deal(&pool, A, 1001, 10011, 100, true, Some(9900), t0()).await;
    // An additional intent claiming an existing position keeps the earliest
    // original intent/risk rather than duplicating that position.
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,'s3:later','s3:later','open',1,'buy',100,100000,99,99)")
        .bind(A).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,symbol_id,volume,raw,received_at) VALUES($1,'s3:later','s3:later',2126,'ORDER_FILLED',1,1,100,''::bytea,now())")
        .bind(A).execute(&pool).await.unwrap();
    position(&pool, A, "s3", 90, None, 100).await; // still open
    position(&pool, A, "s3", 91, Some(1.0), 0).await; // no positive opening history
    position(&pool, A, "s3", 92, Some(1.0), 100).await;
    sqlx::query("UPDATE deals SET close_commission=NULL WHERE account_id=$1 AND deal_id=921")
        .bind(A)
        .execute(&pool)
        .await
        .unwrap();
    let missing_risk_key = format!("s3:{A}:1:93");
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'buy',100,100000,NULL,1)")
        .bind(A).bind(&missing_risk_key).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,symbol_id,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',93,930,1,100,''::bytea,now())")
        .bind(A).bind(&missing_risk_key).execute(&pool).await.unwrap();
    deal(&pool, A, 93, 930, 100, false, None, t0()).await;
    deal(&pool, A, 93, 931, 100, true, Some(100), t0()).await;
    position_with_requested_volume(&pool, A, "s3", 94, Some(2.0), 100, Some(200)).await;
    position_with_requested_volume(&pool, A, "s3", 95, Some(2.0), 100, None).await;

    let acquisitions = Arc::new(AtomicUsize::new(0));
    let counter = acquisitions.clone();
    let measured = sqlx::postgres::PgPoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .idle_timeout(None)
        .max_lifetime(None)
        .before_acquire(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(true) })
        })
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    drop(measured.acquire().await.unwrap());
    acquisitions.store(0, Ordering::SeqCst);
    let (closed, unmeasured) = store::trading::closed_trades(&measured, A, "s3:")
        .await
        .unwrap();
    assert_eq!(closed.len(), 65);
    assert_eq!(unmeasured, 3);
    assert_eq!(
        closed.iter().map(|t| t.position_id).collect::<Vec<_>>(),
        (1..=64).chain([94]).collect::<Vec<_>>()
    );
    for trade in &closed[..64] {
        assert_eq!(
            trade.r,
            if trade.position_id % 2 == 0 {
                2.0
            } else {
                -1.0
            }
        );
        assert_eq!(trade.entry_key, format!("s3:{A}:1:{}", trade.position_id));
    }
    assert_eq!(
        closed[64].r, 4.0,
        "original risk is scaled by the filled share"
    );
    assert!(
        !closed.iter().any(|t| t.position_id == 95),
        "missing original request remains unmeasured, never a full-risk fallback"
    );
    assert_eq!(
        acquisitions.load(Ordering::SeqCst),
        1,
        "batched source facts and money share one snapshot connection, independent of position count"
    );
    acquisitions.store(0, Ordering::SeqCst);
    let empty = store::trading::closed_trades(&measured, B, "s1:")
        .await
        .unwrap();
    assert!(empty.0.is_empty());
    assert_eq!(empty.1, 0);
    assert_eq!(acquisitions.load(Ordering::SeqCst), 1);
    measured.close().await;
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn required_and_optional_missing_cost_evidence_cannot_pass(pool: PgPool) {
    passing(&pool, A, "s3").await;
    fetch(&pool, A, 0, true, false).await;
    for column in ["close_commission", "close_pnl_conversion_fee"] {
        let original = if column == "close_commission" { -40 } else { 0 };
        sqlx::query(&format!(
            "UPDATE deals SET {column}=NULL,fetched_at=now() WHERE account_id=$1 AND deal_id=11"
        ))
        .bind(A)
        .execute(&pool)
        .await
        .unwrap();
        let missing = goals::assess(&pool, A, "s3").await.unwrap();
        assert_eq!(missing.state, AssessmentState::Unknown, "{column}");
        assert_eq!(
            missing.measured_closed_positions, 19,
            "unverified costs do not count as measured net R"
        );
        assert!(missing
            .missing_reasons
            .iter()
            .any(|r| r.contains("closing costs")));
        sqlx::query(&format!(
            "UPDATE deals SET {column}=$2,fetched_at=now() WHERE account_id=$1 AND deal_id=11"
        ))
        .bind(A)
        .bind(original as i64)
        .execute(&pool)
        .await
        .unwrap();
    }
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::WindowMeetsNumericReference
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn stale_failed_and_future_history_are_unknown_until_fresh_live_poll(pool: PgPool) {
    passing(&pool, A, "s3").await;
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::Unknown
    );
    fetch(&pool, A, 7200, true, false).await;
    let stale = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(stale.state, AssessmentState::Unknown);
    assert!(stale
        .missing_reasons
        .iter()
        .any(|r| r.contains("liveness horizon")));
    fetch(&pool, A, 0, false, false).await;
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::Unknown
    );
    fetch(&pool, A, 0, true, false).await;
    let fresh = goals::assess(&pool, A, "s3").await.unwrap();
    assert!(fresh.history_complete);
    assert_eq!(
        fresh.state,
        AssessmentState::WindowMeetsNumericReference,
        "complete=false is a live, not failed, window"
    );
    fetch(&pool, A, 7200, true, false).await;
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::WindowMeetsNumericReference,
        "a later archival job does not replace the current coverage frontier"
    );
    fetch(&pool, A, 0, true, true).await;
    assert_eq!(
        goals::assess(&pool, A, "s3").await.unwrap().state,
        AssessmentState::Unknown,
        "a future coverage frontier cannot be trusted"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn incomplete_opening_history_cannot_turn_half_a_position_into_a_closed_win(pool: PgPool) {
    passing(&pool, A, "s3").await;
    position(&pool, A, "s3", 99, Some(2.0), 50).await;
    close_receipt(&pool, A, "s3", 99, 50, 1).await;
    fetch(&pool, A, 0, true, false).await;
    let incomplete = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(incomplete.state, AssessmentState::Unknown);
    assert_eq!(incomplete.measured_closed_positions, 20);
    assert!(!incomplete.window.position_ids.contains(&99));
    assert!(incomplete
        .missing_reasons
        .iter()
        .any(|r| r.contains("opening history trails")));
    sqlx::query("UPDATE deals SET volume=100,filled_volume=100,fetched_at=now() WHERE account_id=$1 AND deal_id=990").bind(A).execute(&pool).await.unwrap();
    let matured = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(matured.measured_closed_positions, 20);
    assert_eq!(matured.state, AssessmentState::WindowMeetsNumericReference);
    assert!(
        !matured.window.position_ids.contains(&99),
        "the actual 100-volume position remains partly open"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn future_broker_close_cannot_displace_the_actual_latest_window(pool: PgPool) {
    passing(&pool, A, "s3").await;
    fetch(&pool, A, 0, true, false).await;
    sqlx::query(
        "UPDATE deals SET executed_at=$2,fetched_at=now() WHERE account_id=$1 AND deal_id=11",
    )
    .bind(A)
    .bind(Utc::now() + Duration::hours(1))
    .execute(&pool)
    .await
    .unwrap();
    let future = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(future.state, AssessmentState::Unknown);
    assert!(future
        .missing_reasons
        .iter()
        .any(|r| r.contains("broker close timestamp")));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn terminal_receipts_must_match_the_actual_broker_closing_deals(pool: PgPool) {
    passing(&pool, A, "s3").await;
    // Same position/quantity, a different broker deal: totals alone are not proof.
    close_receipt(&pool, A, "s3", 20, 100, 2).await;
    fetch(&pool, A, 0, true, false).await;
    let unmatched = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(unmatched.state, AssessmentState::Unknown);
    assert!(unmatched
        .missing_reasons
        .iter()
        .any(|r| r.contains("terminal exit receipts")));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn terminal_close_with_no_opening_history_is_not_silently_skipped(pool: PgPool) {
    passing(&pool, A, "s3").await;
    position(&pool, A, "s3", 99, None, 0).await;
    position(&pool, A, "s3", 98, None, 100).await;
    let before_close = store::trading::performance(&pool, A, "s3").await.unwrap();
    assert_eq!(
        (before_close.trades, before_close.unmeasured),
        (20, 0),
        "no history/no close and a genuinely open neighbour are not closed outcomes"
    );
    close_receipt(&pool, A, "s3", 99, 100, 1).await;
    deal(
        &pool,
        A,
        99,
        991,
        100,
        true,
        Some(-100),
        t0() + Duration::minutes(99) + Duration::seconds(1),
    )
    .await;
    fetch(&pool, A, 0, true, false).await;
    let legacy = store::trading::performance(&pool, A, "s3").await.unwrap();
    assert_eq!(
        (legacy.trades, legacy.unmeasured),
        (20, 1),
        "a positive closing history with no opening must not disappear from legacy qualification"
    );
    let missing = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(missing.state, AssessmentState::Unknown);
    assert_eq!(missing.unmeasured_positions, 1);
    assert!(missing
        .missing_reasons
        .iter()
        .any(|r| r.contains("opening history trails")));
    deal(
        &pool,
        A,
        99,
        990,
        100,
        false,
        None,
        t0() + Duration::minutes(99) - Duration::hours(1),
    )
    .await;
    let legacy = store::trading::performance(&pool, A, "s3").await.unwrap();
    assert_eq!((legacy.trades, legacy.unmeasured), (21, 0));
    let settled = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(settled.measured_closed_positions, 21);
    assert_eq!(settled.state, AssessmentState::BelowTarget);
    assert_eq!(settled.window.position_ids.last(), Some(&99));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn mixed_reversal_history_is_unmeasured_without_erasing_account_cash(pool: PgPool) {
    let id = 70;
    // The actual NatGas quantity/cash shape: one original SELL10,000, then a
    // BUY20,000 that closes10,000 and opens10,000, further BUY scale-ins,
    // and a final SELL200,000. The legacy is_closing split cannot certify
    // the original strategy risk for this mixed lifecycle.
    let key = format!("s1:{A}:1:{id}");
    let opening_at = t0() + Duration::minutes(id) - Duration::hours(1);
    // Receipts are append-only: construct the original intent and fill once.
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'sell',10000,38800,38.8,38.8)")
        .bind(A).bind(&key).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,deal_id,symbol_id,side,volume,raw,received_at) VALUES($1,$2,$2,2126,'ORDER_FILLED',$3,700,1,'sell',10000,''::bytea,$4)")
        .bind(A).bind(&key).bind(id).bind(opening_at).execute(&pool).await.unwrap();
    deal(&pool, A, id, 700, 10_000, false, None, opening_at).await;
    sqlx::query("UPDATE deals SET trade_side='SELL' WHERE account_id=$1 AND deal_id=700")
        .bind(A)
        .execute(&pool)
        .await
        .unwrap();
    let start = t0() + Duration::minutes(id);
    deal(&pool, A, id, 701, 20_000, true, Some(-30), start).await;
    deal(
        &pool,
        A,
        id,
        702,
        90_000,
        false,
        None,
        start + Duration::minutes(1),
    )
    .await;
    deal(
        &pool,
        A,
        id,
        703,
        50_000,
        false,
        None,
        start + Duration::minutes(2),
    )
    .await;
    deal(
        &pool,
        A,
        id,
        704,
        50_000,
        false,
        None,
        start + Duration::minutes(3),
    )
    .await;
    deal(
        &pool,
        A,
        id,
        705,
        200_000,
        true,
        Some(-9850),
        start + Duration::minutes(4),
    )
    .await;
    sqlx::query("UPDATE deals SET close_closed_volume=10000,close_gross_profit=-30,close_commission=0 WHERE account_id=$1 AND deal_id=701")
        .bind(A).execute(&pool).await.unwrap();
    sqlx::query("UPDATE deals SET trade_side='SELL',close_gross_profit=-9850,close_commission=0 WHERE account_id=$1 AND deal_id=705")
        .bind(A).execute(&pool).await.unwrap();
    fetch(&pool, A, 0, true, false).await;
    let cash_before: i64 = sqlx::query_scalar("SELECT SUM(close_gross_profit+close_swap-ABS(close_commission)-ABS(close_pnl_conversion_fee))::bigint FROM deals WHERE account_id=$1 AND position_id=$2 AND is_closing")
        .bind(A).bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(cash_before, -9880);
    let (closed, unmeasured) = store::trading::closed_trades(&pool, A, "s1:")
        .await
        .unwrap();
    assert!(
        closed.is_empty(),
        "mixed history must not become measured S1 R: {closed:?}"
    );
    assert_eq!(unmeasured, 1);
    let performance = store::trading::performance(&pool, A, "s1").await.unwrap();
    assert_eq!((performance.trades, performance.unmeasured), (0, 1));
    let assessment = goals::assess(&pool, A, "s1").await.unwrap();
    assert_eq!(assessment.state, AssessmentState::Unknown);
    assert_eq!(
        assessment.unmeasured_positions, 1,
        "count the position only once"
    );
    assert!(assessment
        .missing_reasons
        .iter()
        .any(|r| r.contains("closing history exceeds")));
    assert!(assessment
        .missing_reasons
        .iter()
        .any(|r| r.contains("original requested volume")));
    let cash_after: i64 = sqlx::query_scalar("SELECT SUM(close_gross_profit+close_swap-ABS(close_commission)-ABS(close_pnl_conversion_fee))::bigint FROM deals WHERE account_id=$1 AND position_id=$2 AND is_closing")
        .bind(A).bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(
        cash_after, cash_before,
        "qualification must preserve broker cash"
    );
    let deal_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM deals WHERE account_id=$1 AND position_id=$2")
            .bind(A)
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(deal_count, 6);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn legacy_risk_requires_a_positive_request_covering_all_opening_history(pool: PgPool) {
    for (index, requested) in [None, Some(0), Some(-100), Some(50)]
        .into_iter()
        .enumerate()
    {
        let account = A + index as i64;
        position_with_requested_volume(&pool, account, "s1", 1, Some(-2.0), 100, requested).await;
        fetch(&pool, account, 0, true, false).await;
        let (closed, unmeasured) = store::trading::closed_trades(&pool, account, "s1:")
            .await
            .unwrap();
        assert!(
            closed.is_empty(),
            "requested {requested:?} cannot certify risk: {closed:?}"
        );
        assert_eq!(unmeasured, 1);
        let assessment = goals::assess(&pool, account, "s1").await.unwrap();
        assert_eq!(assessment.state, AssessmentState::Unknown);
        assert_eq!(assessment.unmeasured_positions, 1);
        assert!(assessment
            .missing_reasons
            .iter()
            .any(|r| r.contains("original requested volume")));
    }
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn complete_and_partially_filled_requests_keep_valid_risk_with_partial_exits(pool: PgPool) {
    position_with_requested_volume(&pool, A, "s1", 1, Some(2.0), 100, Some(100)).await;
    position_with_requested_volume(&pool, A, "s1", 2, None, 100, Some(200)).await;
    let at = t0() + Duration::minutes(2);
    deal(&pool, A, 2, 21, 40, true, Some(80), at).await;
    let (closed, unmeasured) = store::trading::closed_trades(&pool, A, "s1:")
        .await
        .unwrap();
    assert_eq!(
        closed.len(),
        1,
        "a partly closed position stays outside the cohort"
    );
    assert_eq!(unmeasured, 0);
    deal(
        &pool,
        A,
        2,
        22,
        60,
        true,
        Some(120),
        at + Duration::seconds(1),
    )
    .await;
    let (closed, unmeasured) = store::trading::closed_trades(&pool, A, "s1:")
        .await
        .unwrap();
    assert_eq!(closed.len(), 2);
    assert_eq!(unmeasured, 0);
    assert_eq!(closed[0].r, 2.0, "fully filled original risk");
    assert_eq!(
        closed[1].r, 4.0,
        "100 of 200 filled scales risk to one half"
    );
}
