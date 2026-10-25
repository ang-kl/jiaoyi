//! The token's grant (ProtoOAGetAccountListByAccessTokenReq) is read before
//! an account is authorised (PR #6). Facts from deploy 7765158a, 24-10-2026:
//! demo 49236051 got CH_ACCESS_TOKEN_INVALID on every account auth, even
//! straight after a refresh, while live 43002148 authorised with the same
//! token; and every refresh invalidated the live session.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use jiaoyi::config::BrokerConfig;
use jiaoyi::{loader, recorder, report, service, shutdown};
use jiaoyi_core::allowlist::{Allowlist, Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages::{ProtoOaLightSymbol, ProtoOaSymbol};
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker, GrantEntry};
use jiaoyi_store::{self as store, settings};
use serde_json::json;
use sqlx::PgPool;

const DEMO: i64 = 49236051;
const LIVE: i64 = 43002148;

fn broker(seed: Option<(&str, &str)>) -> BrokerConfig {
    BrokerConfig {
        demo_host: "demo.example".into(),
        live_host: "live.example".into(),
        port: 0,
        client_id: Some("client".into()),
        client_secret: Some(Secret::new("client-secret")),
        seed_access_token: seed.map(|s| Secret::new(s.0)),
        seed_refresh_token: seed.map(|s| Secret::new(s.1)),
        seed_expires_at: None,
    }
}

fn permitted(id: i64) -> PermittedAccount {
    Allowlist::parse(Some("49236051,43002148"))
        .unwrap()
        .permit(id)
        .unwrap()
}

fn fake(account: i64, grant: Vec<GrantEntry>) -> FakeBroker {
    let today = Utc::now().date_naive();
    FakeBroker {
        account_id: account,
        light_symbols: vec![ProtoOaLightSymbol {
            symbol_id: 1,
            symbol_name: Some("EURUSD".into()),
            enabled: Some(true),
            ..Default::default()
        }],
        symbols: vec![ProtoOaSymbol {
            symbol_id: 1,
            digits: 5,
            pip_position: 4,
            ..Default::default()
        }],
        bars: HashMap::from([(
            1,
            (1..=30)
                .map(|i| {
                    d1_bar(
                        today - chrono::Duration::days(i),
                        108_020,
                        108_100,
                        108_000,
                        108_060,
                        10,
                    )
                })
                .collect(),
        )]),
        quotes: HashMap::from([(1, vec![(Some(108_000), Some(108_010))])]),
        grant: Some(grant),
        refresh_reply: Some(("fresh".into(), "fresh-refresh".into(), 2_628_000)),
        ..Default::default()
    }
}

fn demo_listed() -> GrantEntry {
    GrantEntry {
        ctid: DEMO,
        is_live: Some(false),
        login: Some(5361573),
    }
}

fn live_listed() -> GrantEntry {
    GrantEntry {
        ctid: LIVE,
        is_live: Some(true),
        login: Some(1251442),
    }
}

async fn setup(pool: &PgPool) {
    store::upsert_account(pool, DEMO, Environment::Demo)
        .await
        .unwrap();
    store::upsert_account(pool, LIVE, Environment::Live)
        .await
        .unwrap();
    settings::change(
        pool,
        "universe.symbols",
        None,
        Some(json!([{"name": "EURUSD", "class": "fx"}])),
        "test",
        "one symbol",
    )
    .await
    .unwrap();
    settings::change(
        pool,
        "spread.sample_secs",
        None,
        Some(json!(5)),
        "test",
        "short",
    )
    .await
    .unwrap();
    settings::change(
        pool,
        "load.auth_retry_secs",
        None,
        Some(json!(0)),
        "test",
        "no wait",
    )
    .await
    .unwrap();
}

async fn load(
    pool: &PgPool,
    f: &FakeBroker,
    account: i64,
    seed: Option<(&str, &str)>,
) -> loader::LoadOutcome {
    let policy = service::RetryPolicy::read(pool).await.unwrap();
    let (_t, stop) = shutdown::channel();
    let f2 = f.clone();
    service::load_with_retries(
        pool,
        move || {
            let f = f2.clone();
            async move { Ok(f.spawn().0) }
        },
        &broker(seed),
        &permitted(account),
        &policy,
        &stop,
    )
    .await
    .unwrap()
}

fn errors(o: &loader::LoadOutcome) -> String {
    o.summary.errors.join(" | ")
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_account_the_grant_omits_is_named_and_never_refreshed(pool: PgPool) {
    setup(&pool).await;
    // The grant lists live only, as the deploy's token evidently did.
    let f = fake(DEMO, vec![live_listed()]);
    let out = load(&pool, &f, DEMO, Some(("tok", "ref"))).await;
    assert_eq!(out.status, "failed");
    let e = errors(&out);
    assert!(
        e.contains("token grant does not include account 49236051"),
        "{e}"
    );
    assert!(
        e.contains("43002148"),
        "the grant's accounts are named: {e}"
    );
    assert_eq!(
        f.refresh_count.load(Ordering::SeqCst),
        0,
        "no refresh for an account the grant omits"
    );

    // /health names the state per account, with the reason and the
    // fingerprint of the token read; never the token.
    let h = report::health(&pool, false, HashMap::new(), HashMap::new(), HashMap::new())
        .await
        .unwrap();
    let grants = h["token_grants"].as_array().cloned().unwrap_or_default();
    let of = |id: i64| {
        grants
            .iter()
            .find(|g| g["account_id"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("{id} not in {grants:?}"))
    };
    let demo = of(DEMO);
    assert_eq!(demo["state"], "not_listed", "{demo}");
    assert!(
        demo["reason"].as_str().unwrap().contains("43002148"),
        "{demo}"
    );
    let fp = demo["token_fingerprint"].as_str().unwrap();
    assert_eq!(fp, Secret::new("tok").fingerprint());
    assert_eq!(fp.len(), 16);
    assert_eq!(of(LIVE)["state"], "unread");
    assert!(
        !h.to_string().contains("\"tok\""),
        "no token text on /health"
    );
    // /phase1/report carries the last grant read, with the accounts it listed.
    let r = serde_json::to_value(report::build(&pool).await.unwrap()).unwrap();
    let acct = r["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["account_id"] == DEMO)
        .unwrap()
        .clone();
    assert_eq!(acct["token_grant"]["verdict"], "not_listed", "{acct}");
    assert_eq!(acct["token_grant"]["accounts"][0]["ctid"], LIVE);
    assert_eq!(acct["token_grant"]["host"], "demo.example");
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_environment_mismatch_is_refused_without_a_refresh(pool: PgPool) {
    setup(&pool).await;
    // Declared demo, but the grant says the account is live.
    let f = fake(
        DEMO,
        vec![GrantEntry {
            ctid: DEMO,
            is_live: Some(true),
            login: Some(5361573),
        }],
    );
    let out = load(&pool, &f, DEMO, Some(("tok", "ref"))).await;
    assert_eq!(out.status, "failed");
    let e = errors(&out);
    assert!(e.contains("declared demo") && e.contains("live"), "{e}");
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 0);
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_listed_account_with_one_rejection_refreshes_once_then_loads(pool: PgPool) {
    setup(&pool).await;
    let mut f = fake(DEMO, vec![demo_listed(), live_listed()]);
    f.auth_rejections = Arc::new(AtomicUsize::new(1));
    let out = load(&pool, &f, DEMO, Some(("tok", "ref"))).await;
    assert_eq!(out.status, "ok", "{}", errors(&out));
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 1);
    // The grant was read and recorded, by fingerprint only.
    let rows: Vec<(i64, String, serde_json::Value)> = sqlx::query_as(
        "SELECT account_id, token_fingerprint, accounts FROM token_grants ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(!rows.is_empty());
    assert_eq!(rows[0].0, DEMO);
    assert_eq!(rows[0].1.len(), 16);
    assert!(!rows[0].1.contains("tok"));
    assert!(rows[0].2.to_string().contains("49236051"));
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_new_env_seed_replaces_the_stored_token(pool: PgPool) {
    setup(&pool).await;
    let f = fake(DEMO, vec![demo_listed()]);
    assert_eq!(
        load(&pool, &f, DEMO, Some(("A-access", "A-refresh")))
            .await
            .status,
        "ok"
    );
    // A refresh replaced seed A's token in the store.
    sqlx::query("UPDATE oauth_tokens SET access_token = 'A-refreshed', refresh_token = 'A-refreshed-r', source = 'refresh'")
        .execute(&pool)
        .await
        .unwrap();
    // The owner pastes a new grant (seed B) into the environment.
    assert_eq!(
        load(&pool, &f, DEMO, Some(("B-access", "B-refresh")))
            .await
            .status,
        "ok"
    );
    let (access, source): (String, String) =
        sqlx::query_as("SELECT access_token, source FROM oauth_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((access.as_str(), source.as_str()), ("B-access", "env_seed"));
    let events: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT detail FROM reconcile_events WHERE kind = 'token_seed_replaced'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(events.len(), 1);
    let text = events[0].to_string();
    assert!(
        !text.contains("B-access") && !text.contains("A-refreshed"),
        "fingerprints only: {text}"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn an_unchanged_env_seed_keeps_the_refreshed_token(pool: PgPool) {
    setup(&pool).await;
    let f = fake(DEMO, vec![demo_listed()]);
    assert_eq!(
        load(&pool, &f, DEMO, Some(("A-access", "A-refresh")))
            .await
            .status,
        "ok"
    );
    sqlx::query("UPDATE oauth_tokens SET access_token = 'A-refreshed', refresh_token = 'A-refreshed-r', source = 'refresh'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        load(&pool, &f, DEMO, Some(("A-access", "A-refresh")))
            .await
            .status,
        "ok"
    );
    let (access, fingerprint_matches): (String, bool) = sqlx::query_as(
        "SELECT access_token,
                seed_fingerprint = left(encode(sha256(convert_to('A-access', 'UTF8')), 'hex'), 16)
         FROM oauth_tokens",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(access, "A-refreshed", "the refreshed token is kept");
    assert!(
        fingerprint_matches,
        "the seed it came from is remembered by fingerprint"
    );
}

#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_not_listed_demo_account_does_not_invalidate_the_live_session(pool: PgPool) {
    setup(&pool).await;
    let hub = tokio::sync::broadcast::channel(16).0;
    let mut live = fake(LIVE, vec![live_listed()]);
    live.token_events = Some(hub.clone());
    assert_eq!(
        load(&pool, &live, LIVE, Some(("tok", "ref"))).await.status,
        "ok"
    );

    let status = recorder::StatusMap::default();
    let live2 = live.clone();
    let task = tokio::spawn(recorder::run_with(
        pool.clone(),
        broker(None),
        permitted(LIVE),
        status.clone(),
        move || {
            let f = live2.clone();
            async move { Ok(f.spawn().0) }
        },
        Duration::from_millis(50),
        recorder::RecordOptions::default(),
    ));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while status
        .lock()
        .unwrap()
        .get(&LIVE)
        .map_or(0, |s| s.sessions_started)
        == 0
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // Demo is not in the grant; it shares the token hub with live.
    let mut demo = fake(DEMO, vec![live_listed()]);
    demo.token_events = Some(hub.clone());
    let out = load(&pool, &demo, DEMO, None).await;
    assert_eq!(out.status, "failed");
    tokio::time::sleep(Duration::from_millis(300)).await;
    task.abort();
    assert_eq!(demo.refresh_count.load(Ordering::SeqCst), 0);
    let st = status.lock().unwrap()[&LIVE].clone();
    assert_eq!(
        st.account_session_ends, 0,
        "the live session was not invalidated: {st:?}"
    );
}

/// A token stored before seed fingerprints were kept (the Railway row was
/// refreshed before PR #6) must not be overwritten by a spent seed: the seed's
/// grant read is refused, so the stored token is kept and the seed's
/// fingerprint becomes its baseline.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_legacy_refreshed_token_is_kept_when_the_env_seed_is_spent(pool: PgPool) {
    setup(&pool).await;
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, expires_at,
                                   expires_at_missing_reason, obtained_at, source)
         VALUES ('ctrader', 'R-access', 'R-refresh', NULL, 'test', now(), 'refresh')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut f = fake(DEMO, vec![demo_listed()]);
    f.expired_tokens = HashSet::from(["S-access".to_string()]);
    assert_eq!(
        load(&pool, &f, DEMO, Some(("S-access", "S-refresh")))
            .await
            .status,
        "ok"
    );
    let (access, seed_fp): (String, Option<String>) =
        sqlx::query_as("SELECT access_token, seed_fingerprint FROM oauth_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(access, "R-access", "the spent seed did not replace it");
    assert_eq!(seed_fp, Some(Secret::new("S-access").fingerprint()));
    let kinds: Vec<String> =
        sqlx::query_scalar("SELECT kind FROM reconcile_events WHERE kind LIKE 'token_seed%'")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(kinds, vec!["token_seed_baseline".to_string()]);
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 0);
}

/// The other direction, as the hypothesis has it: the legacy refreshed
/// token's grant lists live only, and the env seed's grant (a new grant)
/// adds the demo account. The seed replaces the stored token.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_legacy_refreshed_token_is_replaced_by_a_seed_with_a_wider_grant(pool: PgPool) {
    setup(&pool).await;
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, expires_at,
                                   expires_at_missing_reason, obtained_at, source)
         VALUES ('ctrader', 'R-access', 'R-refresh', NULL, 'test', now(), 'refresh')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut f = fake(DEMO, vec![demo_listed(), live_listed()]);
    f.grant_by_token = HashMap::from([("R-access".to_string(), vec![live_listed()])]);
    assert_eq!(
        load(&pool, &f, DEMO, Some(("S-access", "S-refresh")))
            .await
            .status,
        "ok"
    );
    let (access, source): (String, String) =
        sqlx::query_as("SELECT access_token, source FROM oauth_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((access.as_str(), source.as_str()), ("S-access", "env_seed"));
    let detail: serde_json::Value = sqlx::query_scalar(
        "SELECT detail FROM reconcile_events WHERE kind = 'token_seed_replaced'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(detail["replaced_token_source"], "refresh");
    assert!(
        detail["why"].as_str().unwrap().contains("49236051"),
        "{detail}"
    );
    let text = detail.to_string();
    assert!(
        !text.contains("R-access") && !text.contains("S-access"),
        "{text}"
    );
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 0);
}

/// A legacy refreshed token whose grant already covers everything the seed's
/// does is kept, even when the seed is still accepted: the seed's refresh
/// token may be spent, and nothing would be gained.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn a_legacy_refreshed_token_is_kept_when_the_seed_adds_no_account(pool: PgPool) {
    setup(&pool).await;
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, expires_at,
                                   expires_at_missing_reason, obtained_at, source)
         VALUES ('ctrader', 'R-access', 'R-refresh', NULL, 'test', now(), 'refresh')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let f = fake(DEMO, vec![demo_listed(), live_listed()]);
    assert_eq!(
        load(&pool, &f, DEMO, Some(("S-access", "S-refresh")))
            .await
            .status,
        "ok"
    );
    let access: String = sqlx::query_scalar("SELECT access_token FROM oauth_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(access, "R-access");
    let kinds: Vec<String> =
        sqlx::query_scalar("SELECT kind FROM reconcile_events WHERE kind LIKE 'token_seed%'")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(kinds, vec!["token_seed_baseline".to_string()]);
}

/// At most one refresh per authorisation, since each refresh ends every other
/// session on the old token: the grant read refuses the token, the one
/// refresh renews it, and an account authorisation still refused is left to
/// the load's own retry (on the renewed token), not refreshed again.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn one_authorisation_refreshes_the_token_at_most_once(pool: PgPool) {
    setup(&pool).await;
    let mut f = fake(DEMO, vec![demo_listed()]);
    f.expired_tokens = HashSet::from(["tok".to_string()]);
    f.auth_rejections = Arc::new(AtomicUsize::new(1));
    let out = load(&pool, &f, DEMO, Some(("tok", "ref"))).await;
    assert_eq!(out.status, "ok", "{}", errors(&out));
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 1);
    let runs: i64 = sqlx::query_scalar("SELECT count(*) FROM load_runs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        runs, 2,
        "the refused run, then the retry on the renewed token"
    );
}

/// Codex P1 on #6 (48890c03): on boot after a new env seed, the loader, the
/// recorders and the history syncs all decide on the seed at once. A task
/// that read the stored row before another stored the seed, authorised and
/// refreshed it must not then write the seed back: the seed's refresh token
/// is spent and its access token invalidated, and the service would be left
/// with no valid token. Three tasks authorise together; the stored row is
/// from before seed fingerprints (refreshed), the seed's grant adds the demo
/// account, and the first account authorisation is refused, so one refresh
/// happens.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn concurrent_authorisations_store_a_new_seed_once_and_keep_its_refresh(pool: PgPool) {
    setup(&pool).await;
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, expires_at,
                                   expires_at_missing_reason, obtained_at, source)
         VALUES ('ctrader', 'R-access', 'R-refresh', NULL, 'test', now(), 'refresh')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut f = fake(DEMO, vec![demo_listed(), live_listed()]);
    f.grant_by_token = HashMap::from([("R-access".to_string(), vec![live_listed()])]);
    f.auth_rejections = Arc::new(AtomicUsize::new(1));
    f.pairs = vec![
        ("S-access".into(), "S-refresh".into()),
        ("R-access".into(), "R-refresh".into()),
    ];
    // Ordering, by the broker's reply times only. The first task reads the
    // seed's grant in 100 ms, so the others read the stored row before it
    // stores the seed. The others read the stored token's grant in 500 ms, so
    // (without the lock) their legacy decision, "the seed's grant adds an
    // account", lands after the first task has stored the seed, authorised,
    // been refused once and refreshed.
    let mut first = f.clone();
    first.account_list_delays =
        HashMap::from([("S-access".to_string(), Duration::from_millis(100))]);
    let mut slow = f.clone();
    slow.account_list_delays =
        HashMap::from([("R-access".to_string(), Duration::from_millis(500))]);

    let mut tasks = Vec::new();
    for fk in [first, slow.clone(), slow] {
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            let (conn, _) = fk.spawn();
            loader::authorise(
                &pool,
                &conn,
                &broker(Some(("S-access", "S-refresh"))),
                &permitted(DEMO),
            )
            .await
            .map(|_| ())
            .map_err(|e| format!("{e:#}"))
        }));
    }
    let mut outcomes = Vec::new();
    for t in tasks {
        outcomes.push(t.await.unwrap());
    }
    assert!(
        outcomes.iter().all(|o| o.is_ok()),
        "every task ends authorised: {outcomes:?}"
    );
    let (access, refresh, source): (String, String, String) =
        sqlx::query_as("SELECT access_token, refresh_token, source FROM oauth_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        (access.as_str(), refresh.as_str(), source.as_str()),
        ("fresh", "fresh-refresh", "refresh"),
        "the refreshed pair is kept, not the spent seed"
    );
    let replaced: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM reconcile_events WHERE kind = 'token_seed_replaced'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(replaced, 1);
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 1);
}

/// The same race on the plain path: the stored row descends from an older
/// seed, and five tasks start together with a new one. The seed is stored
/// once, and the one refresh it needs is kept.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn concurrent_authorisations_replace_an_old_seed_once(pool: PgPool) {
    setup(&pool).await;
    sqlx::query(
        "INSERT INTO oauth_tokens (token_owner, access_token, refresh_token, expires_at,
                                   expires_at_missing_reason, obtained_at, source, seed_fingerprint)
         VALUES ('ctrader', 'O-access', 'O-refresh', NULL, 'test', now(), 'refresh', $1)",
    )
    .bind(Secret::new("OLD-seed").fingerprint())
    .execute(&pool)
    .await
    .unwrap();
    let mut f = fake(DEMO, vec![demo_listed()]);
    f.auth_rejections = Arc::new(AtomicUsize::new(1));
    f.pairs = vec![("S-access".into(), "S-refresh".into())];
    let mut tasks = Vec::new();
    for _ in 0..5 {
        let (pool, fk) = (pool.clone(), f.clone());
        tasks.push(tokio::spawn(async move {
            let (conn, _) = fk.spawn();
            loader::authorise(
                &pool,
                &conn,
                &broker(Some(("S-access", "S-refresh"))),
                &permitted(DEMO),
            )
            .await
            .map(|_| ())
            .map_err(|e| format!("{e:#}"))
        }));
    }
    let mut outcomes = Vec::new();
    for t in tasks {
        outcomes.push(t.await.unwrap());
    }
    assert!(outcomes.iter().all(|o| o.is_ok()), "{outcomes:?}");
    let access: String = sqlx::query_scalar("SELECT access_token FROM oauth_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(access, "fresh");
    let replaced: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM reconcile_events WHERE kind = 'token_seed_replaced'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(replaced, 1);
    assert_eq!(f.refresh_count.load(Ordering::SeqCst), 1);
}
