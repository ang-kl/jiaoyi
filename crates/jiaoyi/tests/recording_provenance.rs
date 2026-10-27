//! Recorder source evidence against the real PostgreSQL and protobuf client.

use std::collections::HashMap;
use std::time::Duration;

use jiaoyi::config::BrokerConfig;
use jiaoyi::{recorder, shutdown};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::testkit::FakeBroker;
use jiaoyi_store::{self as store, ticks};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{timeout, Instant};

const ACCOUNT: i64 = 49236051;

fn account() -> PermittedAccount {
    Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(ACCOUNT)
        .unwrap()
}

fn broker() -> BrokerConfig {
    BrokerConfig {
        demo_host: "unused".into(),
        live_host: "unused".into(),
        port: 0,
        client_id: Some("client".into()),
        client_secret: Some(Secret::new("fixture-secret")),
        seed_access_token: Some(Secret::new("fixture-token")),
        seed_refresh_token: Some(Secret::new("fixture-refresh")),
        seed_expires_at: None,
    }
}

async fn prepared(pool: &PgPool) {
    store::upsert_account(pool, ACCOUNT, Environment::Demo)
        .await
        .unwrap();
    let run = store::start_run(pool, ACCOUNT).await.unwrap();
    sqlx::query(
        "INSERT INTO universe_resolutions
            (load_run_id,account_id,name,asset_class,status,broker_symbol_id)
         VALUES ($1,$2,'EURUSD','fx','resolved',1)",
    )
    .bind(run)
    .bind(ACCOUNT)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO symbols
            (account_id,broker_symbol_id,name,asset_class,digits,pip_position,raw,read_at)
         VALUES ($1,1,'EURUSD','fx',5,4,''::bytea,now())",
    )
    .bind(ACCOUNT)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn recording_start_links_to_exact_tick_session_without_claiming_coverage(pool: PgPool) {
    prepared(&pool).await;
    let (conn, log) = FakeBroker {
        account_id: ACCOUNT,
        quotes: HashMap::from([(
            1,
            vec![(Some(108_000), Some(108_008)), (None, Some(108_009))],
        )]),
        ..Default::default()
    }
    .spawn();
    let status = recorder::StatusMap::default();
    let n = recorder::record(
        &pool,
        &conn,
        &broker(),
        &account(),
        &status,
        recorder::RecordOptions {
            until: Some(Instant::now() + Duration::from_millis(300)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(n, 2);
    let markers: Vec<(String, String, Value)> = sqlx::query_as(
        "SELECT kind,reason,detail FROM trading_decisions
         WHERE account_id=$1 AND kind LIKE 'tick_recording_%' ORDER BY id",
    )
    .bind(ACCOUNT)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(markers.len(), 1, "no marker per tick or fabricated gap");
    assert_eq!(markers[0].0, "tick_recording_started");
    assert_eq!(markers[0].1, "spot_receiver_subscribed");
    let detail = &markers[0].2;
    assert_eq!(detail["marker_version"], 1);
    assert!(detail["last_assigned_seq"].is_null());
    assert!(detail.get("lost_count").is_none());
    assert!(detail.get("complete").is_none());
    assert!(detail["observed_at"].as_str().is_some());
    let rows = sqlx::query_as::<
        _,
        (
            String,
            i64,
            chrono::DateTime<chrono::Utc>,
            Option<i64>,
            Option<i64>,
        ),
    >("SELECT session_id,seq,received_at,bid_e5,ask_e5 FROM ticks ORDER BY seq")
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(rows.iter().all(|r| detail["session_id"] == r.0));
    let observed_at: chrono::DateTime<chrono::Utc> =
        serde_json::from_value(detail["observed_at"].clone()).unwrap();
    assert!(rows.iter().all(|r| r.2 >= observed_at));
    assert_eq!((rows[0].1, rows[1].1), (0, 1));
    assert_eq!((rows[1].3, rows[1].4), (None, Some(108_009)));
    let replay: Vec<_> = rows
        .into_iter()
        .map(|r| ticks::TickRow {
            account_id: ACCOUNT,
            session_id: r.0,
            seq: r.1,
            received_at: r.2,
            broker_symbol_id: 1,
            broker_timestamp: None,
            bid_e5: r.3,
            ask_e5: r.4,
        })
        .collect();
    assert_eq!(ticks::insert(&pool, &replay).await.unwrap(), 0);
    assert!(log.lock().unwrap().orders.is_empty());
    assert!(log.lock().unwrap().tick_requests.is_empty());
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn actual_receiver_lag_is_persisted_without_fabricated_ticks(pool: PgPool) {
    prepared(&pool).await;
    const SENT: u64 = 5000;
    let (conn, _) = FakeBroker {
        account_id: ACCOUNT,
        quotes: HashMap::from([(
            1,
            (0..SENT)
                .map(|n| (Some(108_000 + n), Some(108_008 + n)))
                .collect(),
        )]),
        ..Default::default()
    }
    .spawn();
    let mut observer = conn.spot_events();
    // Let the real client receive all quotes while record() waits to persist
    // START, deterministically overflowing its real broadcast receiver.
    let mut hold = pool.begin().await.unwrap();
    sqlx::query("LOCK TABLE trading_decisions IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *hold)
        .await
        .unwrap();
    let status = recorder::StatusMap::default();
    let (stop, receiver) = shutdown::channel();
    let (task_pool, task_status) = (pool.clone(), status.clone());
    let task = tokio::spawn(async move {
        recorder::record(
            &task_pool,
            &conn,
            &broker(),
            &account(),
            &task_status,
            recorder::RecordOptions {
                shutdown: Some(receiver),
                ..Default::default()
            },
        )
        .await
    });
    timeout(Duration::from_secs(5), async {
        loop {
            match observer.recv().await {
                Ok(spot) if spot.event.bid == Some(108_000 + SENT - 1) => break,
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(e) => panic!("fixture spot receiver closed: {e}"),
            }
        }
    })
    .await
    .unwrap();
    hold.commit().await.unwrap();
    timeout(Duration::from_secs(5), async {
        loop {
            let s = status.lock().unwrap().get(&ACCOUNT).cloned();
            if s.is_some_and(|s| s.ticks_written + s.ticks_missed_lagging == SENT) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    stop.fire();
    let written = task.await.unwrap().unwrap();
    let gaps: Vec<Value> = sqlx::query_scalar(
        "SELECT detail FROM trading_decisions WHERE account_id=$1 AND kind='tick_recording_gap'",
    )
    .bind(ACCOUNT)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0]["marker_version"], 1);
    assert!(gaps[0]["last_assigned_seq"].is_null());
    let lost = gaps[0]["lost_count"].as_u64().unwrap();
    assert!(lost > 0);
    assert_eq!(written + lost, SENT);
    let persisted: i64 = sqlx::query_scalar("SELECT count(*) FROM ticks")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(persisted as u64, written, "lost quotes are never filled in");
    let linked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trading_decisions WHERE account_id=$1
         AND kind='tick_recording_started' AND detail->>'session_id'=$2",
    )
    .bind(ACCOUNT)
    .bind(gaps[0]["session_id"].as_str().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(linked, 1);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn source_marker_failure_is_returned_before_recording_is_reported_started(pool: PgPool) {
    prepared(&pool).await;
    sqlx::raw_sql(
        "CREATE FUNCTION reject_source_marker() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN
           IF NEW.kind LIKE 'tick_recording_%' THEN RAISE EXCEPTION 'marker fixture refusal'; END IF;
           RETURN NEW;
         END $$;
         CREATE TRIGGER source_marker_refusal BEFORE INSERT ON trading_decisions
         FOR EACH ROW EXECUTE FUNCTION reject_source_marker();",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (conn, _) = FakeBroker {
        account_id: ACCOUNT,
        ..Default::default()
    }
    .spawn();
    let status = recorder::StatusMap::default();
    let error = recorder::record(
        &pool,
        &conn,
        &broker(),
        &account(),
        &status,
        recorder::RecordOptions::default(),
    )
    .await
    .unwrap_err();
    assert!(format!("{error:#}").contains("persist tick recording source marker"));
    assert_eq!(status.lock().unwrap()[&ACCOUNT].sessions_started, 0);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM trading_decisions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "failure must not produce a success marker");
}
