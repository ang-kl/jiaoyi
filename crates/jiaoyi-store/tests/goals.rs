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
    let key = format!("{code}:{account}:1:{id}");
    let at = t0() + Duration::minutes(id);
    sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,side,volume,relative_stop,risk,max_risk) VALUES($1,$2,$2,'open',1,'buy',100,100000,1,1)")
        .bind(account).bind(&key).execute(pool).await.unwrap();
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
    let missing = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(missing.state, AssessmentState::Unknown);
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
    let settled = goals::assess(&pool, A, "s3").await.unwrap();
    assert_eq!(settled.measured_closed_positions, 21);
    assert_eq!(settled.state, AssessmentState::BelowTarget);
    assert_eq!(settled.window.position_ids.last(), Some(&99));
}
